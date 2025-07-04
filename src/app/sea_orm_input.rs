use std::error::Error;
use chrono::Utc;
use crate::openai::{AzureOpenAIClient, Message};
use crate::mcp::MCPServerManager;
use crate::persistence::config::AppConfig;
use crate::persistence::seaorm_store::SeaOrmStore;

use super::sea_orm_state::SeaOrmAppState;
use super::models::ConversationMode;

/// Handle creating a new conversation if needed
pub async fn handle_conversation_creation(
    app: &mut SeaOrmAppState, 
    config: &mut AppConfig,
    store: &SeaOrmStore
) -> Result<String, Box<dyn Error>> {
    if app.input.is_empty() {
        return Ok(app.current_conversation());
    }

    let current_conversation = app.current_conversation();
    let mode = app.get_selected_mode();

    match mode {
        ConversationMode::New => {
            // Create a new conversation with first message as the title
            let title = app.input.clone();
            
            let conversation_id = store.create_conversation(&title).await?;
            
            // Select the new conversation
            let new_index = app.conversations_list.len();
            app.conversations_list.push(title.clone());
            app.conversation_modes.push(ConversationMode::None);
            
            // Map conversation ID to index
            app.conv_id_to_index.insert(conversation_id, new_index);
            
            // Initialize message list
            app.messages.insert(title.clone(), Vec::new());
            
            // Select the new conversation
            app.state.select(Some(new_index));
            app.current_conversation_id = Some(conversation_id);
            
            // Set as most recent conversation
            config.last_conversation = Some(conversation_id);
            config.save()?;
            
            Ok(title)
        },
        _ => {
            // Use the existing conversation
            if current_conversation == "All Conversations" {
                // Cannot add messages to "All Conversations" view
                app.input.clear();
                return Ok(current_conversation);
            }

            // Get the conversation ID for the selected conversation
            if app.current_conversation_id.is_none() {
                if let Some(selected) = app.state.selected() {
                    app.current_conversation_id = app.get_conversation_id_by_index(selected);
                    
                    // Set as most recent conversation
                    if let Some(id) = app.current_conversation_id {
                        config.last_conversation = Some(id as i64);
                        config.save()?;
                    }
                }
            }

            Ok(current_conversation)
        }
    }
}

/// Process MCP server mentions in the input
pub async fn handle_mcp_server_mentions(
    app: &mut SeaOrmAppState,
    store: &SeaOrmStore,
    mcp_manager: &mut Option<MCPServerManager>
) -> Result<(), Box<dyn Error>> {
    if app.input.is_empty() {
        return Ok(());
    }

    // Check for @server mentions to enable servers
    if let Some(manager) = mcp_manager.as_ref() {
        let server_names = manager.get_server_names();
        
        for server_name in &server_names {
            let mention = format!("@{}", server_name);
            
            if app.input.contains(&mention) && app.current_conversation_id.is_some() {
                toggle_mcp_server(app, store, server_name).await?;
            }
        }
    }

    Ok(())
}

/// Process the message input and send to OpenAI
pub async fn handle_message_processing(
    app: &mut SeaOrmAppState,
    conversation: &str,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &mut Option<MCPServerManager>
) -> Result<(), Box<dyn Error>> {
    if app.input.is_empty() {
        return Ok(());
    }

    // Skip processing for special conversation modes
    if app.is_special_conversation() {
        app.input.clear();
        return Ok(());
    }

    // Create user message
    let user_msg = Message {
        role: "user".to_string(),
        content: app.input.clone(),
    };

    // Add user message to conversation
    let messages = app.messages.entry(conversation.to_string())
        .or_insert_with(Vec::new);
    messages.push(user_msg.clone());
    
    // Clear input
    app.input.clear();

    // Get history for context
    let message_history = app.messages
        .get(conversation)
        .cloned()
        .unwrap_or_default();
    
    // Process with MCP servers if enabled
    let mut mcp_responded = false;
    
    if let Some(conversation_id) = app.current_conversation_id {
        if let Ok(enabled_servers) = store.get_enabled_servers(conversation_id).await {
            app.active_mcp_servers = enabled_servers.clone();
            
            if !enabled_servers.is_empty() && mcp_manager.is_some() {
                if let Some(manager) = mcp_manager {
                    for server_name in &enabled_servers {
                        if let Ok(response) = manager.process_message(server_name, &user_msg.content).await {
                            let mcp_msg = Message {
                                role: "assistant".to_string(),
                                content: response,
                            };
                            
                            // Add MCP message to conversation
                            messages.push(mcp_msg.clone());
                            
                            // Save to store with server name
                            if let Err(e) = store.add_message_with_server(
                                conversation_id, &mcp_msg, server_name
                            ).await {
                                eprintln!("Error adding MCP message: {}", e);
                            }
                            
                            mcp_responded = true;
                        }
                    }
                }
            }
        }
    }

    // If no MCP server responded, use OpenAI
    if !mcp_responded {
        match oaiclient.send_chat(&message_history).await {
            Ok(response) => {
                // Add response to conversation
                messages.push(response.clone());
                
                // Save to store
                if let Some(conversation_id) = app.current_conversation_id {
                    if let Err(e) = store.add_message(conversation_id, &response).await {
                        eprintln!("Error adding response: {}", e);
                    }
                }
            },
            Err(e) => {
                let error_msg = Message {
                    role: "assistant".to_string(),
                    content: format!("Error: {}", e),
                };
                messages.push(error_msg);
            }
        }
    }

    Ok(())
}

/// Persist messages to the database
pub async fn handle_persistence(
    app: &mut SeaOrmAppState,
    conversation: &str,
    store: &SeaOrmStore
) -> Result<(), Box<dyn Error>> {
    if let Some(conversation_id) = app.current_conversation_id {
        if let Some(messages) = app.messages.get(conversation) {
            if messages.len() == 1 {
                // Save the first user message
                if let Some(message) = messages.first() {
                    store.add_message(conversation_id, message).await?;
                }
            }
        }
    }

    Ok(())
}

/// Toggle an MCP server on or off for the current conversation
pub async fn toggle_mcp_server(
    app: &mut SeaOrmAppState,
    store: &SeaOrmStore,
    server_name: &str
) -> Result<(), Box<dyn Error>> {
    if app.mcp_servers.contains_key(server_name) {
        if let Some(conversation_id) = app.current_conversation_id {
            // Check if server is already enabled
            let enabled_servers = store.get_enabled_servers(conversation_id).await?;
            let is_enabled = enabled_servers.contains(&server_name.to_string());
            
            if is_enabled {
                // Disable the server
                store.disable_mcp_server(conversation_id, server_name).await?;
                
                // Update local state
                app.active_mcp_servers.retain(|s| s != server_name);
            } else {
                // Enable the server
                store.enable_mcp_server(conversation_id, server_name).await?;
                
                // Update local state
                if !app.active_mcp_servers.contains(&server_name.to_string()) {
                    app.active_mcp_servers.push(server_name.to_string());
                }
            }
        }
    }
    
    Ok(())
}

/// Handle text input
pub fn handle_text_input(app: &mut SeaOrmAppState, c: char) {
    app.input.push(c);
}

/// Handle backspace in input
pub fn handle_backspace(app: &mut SeaOrmAppState) {
    app.input.pop();
}

/// Handle up arrow navigation
pub fn handle_navigation_up(app: &mut SeaOrmAppState) {
    if let Some(selected) = app.state.selected() {
        if selected > 0 {
            app.state.select(Some(selected - 1));
        }
    } else {
        app.state.select(Some(0));
    }
}

/// Handle down arrow navigation
pub fn handle_navigation_down(app: &mut SeaOrmAppState) {
    if let Some(selected) = app.state.selected() {
        if selected < app.conversations_list.len() - 1 {
            app.state.select(Some(selected + 1));
        }
    } else {
        app.state.select(Some(0));
    }
}