use std::error::Error;

use crate::openai::{AzureOpenAIClient, Message};
use crate::persistence::{ConversationStore, Conversation};
use crate::persistence::config::AppConfig;
use crate::mcp::MCPServerManager;

use super::state::AppState;

/// Handle text input in the app, adding characters to the input field
pub fn handle_text_input(app: &mut AppState, c: char) {
    app.input.push(c);
}

/// Handle backspace in the input field
pub fn handle_backspace(app: &mut AppState) {
    app.input.pop();
}

/// Handle navigation up in conversation list
pub fn handle_navigation_up(app: &mut AppState) {
    if let Some(selected) = app.state.selected() {
        let new = selected.saturating_sub(1);
        app.state.select(Some(new));
    }
}

/// Handle navigation down in conversation list
pub fn handle_navigation_down(app: &mut AppState) {
    if let Some(selected) = app.state.selected() {
        let new = (selected + 1).min(app.conversations_list.len().saturating_sub(1));
        app.state.select(Some(new));
    }
}

/// Handle the creation of a new conversation or switching to an existing one
pub fn handle_conversation_creation(
    app: &mut AppState,
    config: &mut AppConfig,
    store: &ConversationStore,
) -> Result<String, Box<dyn Error>> {
    let mut conversation = app.current_conversation();
    
    if conversation == "+ New Conversation" {
        // DB-backed: create new conversation
        let timestamp = chrono::Utc::now().timestamp();
        let conv_title = format!("conv_{}", timestamp);
        let conversation_id = store.create_conversation(&conv_title)?;
        config.conversation_id = Some(conversation_id);
        config.save()?;
        app.current_conversation_id = Some(conversation_id);
        
        // Create a new Conversation object
        let new_conversation = Conversation {
            id: conversation_id,
            title: conv_title.clone(),
            created_at: timestamp,
            mcp_servers: Vec::new(),
        };
        
        // Add to our conversations list
        app.conversations.push(new_conversation);
        
        // Add to conversations list
        app.conversations_list.push(conv_title.clone());
        app.conversation_modes.push(super::models::ConversationMode::None);
        let index = app.conversations_list.len() - 1;
        app.state.select(Some(index));
        app.conv_id_to_index.insert(conversation_id, index);
        app.messages.insert(conv_title.clone(), Vec::new());
        conversation = conv_title;
        
        // Clear active MCP servers for new conversation
        app.active_mcp_servers.clear();
    } else if conversation != "All Conversations" {
        // If selecting an existing conversation, set it as current
        let selected_idx = app.state.selected().unwrap_or(0);
        
        if let Some(conv_id) = app.get_conversation_id_by_index(selected_idx) {
            app.current_conversation_id = Some(conv_id);
            config.conversation_id = Some(conv_id);
            config.save()?;
            
            // Load MCP servers for this conversation
            app.active_mcp_servers = store.get_enabled_servers(conv_id)?;
        } else if let Some(conv_id) = app.get_conversation_id_by_title(&conversation) {
            // Direct lookup by title if index lookup failed
            app.current_conversation_id = Some(conv_id);
            config.conversation_id = Some(conv_id);
            config.save()?;
            
            // Load MCP servers for this conversation
            app.active_mcp_servers = store.get_enabled_servers(conv_id)?;
        }
    }
    
    if conversation == "All Conversations" {
        conversation = app.conversations_list
            .iter()
            .skip(2) // Skip the special entries
            .next()
            .cloned()
            .unwrap_or_else(|| "".to_string());
    }
    
    Ok(conversation)
}

/// Process a message and get a response
pub async fn handle_message_processing(
    app: &mut AppState,
    conversation: &str,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &mut Option<MCPServerManager>,
) -> Result<(), Box<dyn Error>> {
    if !conversation.is_empty() && !app.input.is_empty() {
        let msgs = app.messages.entry(conversation.to_string()).or_default();
        
        // Save the user message
        msgs.push(Message {
            role: "user".into(),
            content: app.input.clone(),
        });
        
        // Check if this is a special command for an MCP server
        let mut mcp_response = None;
        
        // Check for ticket commands
        if app.input.starts_with("@ticket ") {
            let cmd = app.input.trim_start_matches("@ticket ").trim();
            
            // Use existing MCP server if available, otherwise create new one
            let response = if let Some(manager) = mcp_manager.as_mut() {
                if let Some(_server) = manager.get_server("ticket") {
                    match manager.send_to_python_process("ticket", cmd) {
                        Ok(true) => "Command sent to ticket server".to_string(),
                        Ok(false) => "Failed to send command to ticket server".to_string(),
                        Err(e) => format!("Error: {}", e)
                    }
                } else {
                    "Ticket server not available".to_string()
                }
            } else {
                "MCP server manager not available".to_string()
            };
            
            // Create a response message
            mcp_response = Some(Message {
                role: "system".into(),
                content: response,
            });
        }
        
        // If we have an MCP response, use that instead of sending to OpenAI
        let reply = if let Some(response) = mcp_response {
            response
        } else {
            match oaiclient.send_chat(msgs).await {
                Ok(r) => r,
                Err(e) => Message {
                    role: "error".into(),
                    content: format!("{}", e),
                },
            }
        };
        
        // Add the reply to the messages
        msgs.push(reply);
        app.input.clear();
    }
    
    Ok(())
}

/// Save conversation data to persistent storage
pub fn handle_persistence(
    app: &AppState,
    conversation: &str,
    store: &ConversationStore,
) -> Result<(), Box<dyn Error>> {
    let empty_vec = Vec::new();
    let msgs = app.messages.get(conversation).unwrap_or(&empty_vec);
    
    if let Some(cid) = app.current_conversation_id {
        for m in msgs {
            let _ = store.add_message(cid, m);
        }
    }
    
    Ok(())
}

/// Toggle whether an MCP server is active for the current conversation
pub fn toggle_mcp_server(
    app: &mut AppState,
    store: &ConversationStore,
    server_name: &str,
) -> Result<(), Box<dyn Error>> {
    let is_active = app.active_mcp_servers.contains(&server_name.to_string());
    
    if is_active {
        app.active_mcp_servers.retain(|s| s != server_name);
        if let Some(conv_id) = app.current_conversation_id {
            let _ = store.disable_mcp_server(conv_id, server_name);
        }
    } else {
        app.active_mcp_servers.push(server_name.to_string());
        if let Some(conv_id) = app.current_conversation_id {
            let _ = store.enable_mcp_server(conv_id, server_name)?;
        }
    }
    
    Ok(())
}

/// Parse MCP server commands and update active servers
pub fn handle_mcp_server_mentions(
    app: &mut AppState,
    store: &ConversationStore,
    mcp_manager: &mut Option<MCPServerManager>,
) -> Result<(), Box<dyn Error>> {
    if let Some(manager) = mcp_manager.as_mut() {
        let (cleaned_input, mentioned_servers) = manager.parse_server_commands(&app.input);
        
        // Update active MCP servers based on mentions
        for server_name in &mentioned_servers {
            if !app.active_mcp_servers.contains(server_name) {
                app.active_mcp_servers.push(server_name.clone());
                
                // Also update in the database if we have a conversation ID
                if let Some(conv_id) = app.current_conversation_id {
                    store.enable_mcp_server(conv_id, server_name)?;
                }
            }
        }
        
        // Update the input to remove the @server commands
        if !mentioned_servers.is_empty() {
            app.input = cleaned_input;
        }
    }
    
    Ok(())
}