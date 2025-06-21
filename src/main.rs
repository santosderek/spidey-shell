use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};


mod openai;
mod persistence;
mod mcp;
mod credentials;
mod tickets_core; // Core ticket types and traits
mod tickets; // Ticket MCP server implementation
use openai::{AzureOpenAIClient, AzureOpenAIConfig, Message};
use persistence::config::AppConfig;
use persistence::{ConversationStore, Conversation};
use mcp::{MCPServerManager, MCPServerConfig};
use credentials::CredentialManager;
use tickets::TicketMCPServer;

/// Input mode for the application (vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Normal, // For navigation and commands
    Insert, // For text input
    Visual, // For selecting text/messages
    Command, // For command input (like :q in vim)
}

/// Search direction for vim-style search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchMode {
    Forward,  // '/' search
    Backward, // '?' search
}

struct App {
    servers: Vec<String>,
    state: ListState,
    input: String,
    messages: HashMap<String, Vec<Message>>, // server -> messages
    current_conversation_id: Option<i64>,
    mcp_servers: HashMap<String, bool>, // MCP server name -> enabled
    active_mcp_servers: Vec<String>,    // Currently active MCP servers for this conversation
    conversations: Vec<Conversation>,   // All conversations from the database
    conv_id_to_index: HashMap<i64, usize>, // Mapping from conversation ID to index in the servers list
    
    // Vim-style navigation state
    chat_scroll_position: usize,       // Current scroll position in chat history
    chat_message_selected: Option<usize>, // Index of currently selected message (for operations)
    input_mode: InputMode,             // Current input mode (Normal, Insert, etc.)
    search_query: String,              // Current search query in normal mode
    search_mode: Option<SearchMode>,   // Current search mode (if active)
}

impl App {
    fn new(mcp_manager: &Option<MCPServerManager>, store: &ConversationStore) -> Self {
        let mut app = Self { 
            servers: Vec::new(),
            state: ListState::default(), 
            input: String::new(), 
            messages: HashMap::new(), 
            current_conversation_id: None,
            mcp_servers: HashMap::new(),
            active_mcp_servers: Vec::new(),
            conversations: Vec::new(),
            conv_id_to_index: HashMap::new(),
            
            // Initialize vim-style navigation state
            chat_scroll_position: 0,
            chat_message_selected: None,
            input_mode: InputMode::Insert, // Start in insert mode by default
            search_query: String::new(),
            search_mode: None,
        };
        
        // Load conversations from the database
        app.load_conversations(store);
        
        // Initialize MCP server status
        if let Some(manager) = mcp_manager {
            for name in manager.get_server_names() {
                if let Some(config) = manager.get_server(&name) {
                    app.mcp_servers.insert(name, config.enabled);
                }
            }
        }
        
        // Select the first item if available
        if !app.servers.is_empty() { 
            app.state.select(Some(0)); 
        }
        
        app
    }
    
    fn load_conversations(&mut self, store: &ConversationStore) {
        // Clear current data
        self.servers.clear();
        self.messages.clear();
        self.conv_id_to_index.clear();
        
        // Add standard entries
        self.servers.push("None".to_string());
        self.servers.push("All".to_string());
        
        // Get conversations from the database
        if let Ok(conversations) = store.get_all_conversations() {
            for conversation in conversations {
                let title = conversation.title.clone();
                let conv_id = conversation.id;
                
                // Store the conversation
                self.conversations.push(conversation);
                
                // Map conversation ID to server list index
                let index = self.servers.len();
                self.conv_id_to_index.insert(conv_id, index);
                
                // Add to servers list
                self.servers.push(title.clone());
                
                // Load messages
                if let Ok(msgs) = store.get_conversation_messages(conv_id) {
                    self.messages.insert(title, msgs);
                } else {
                    self.messages.insert(title, Vec::new());
                }
            }
        }
        
        // Also load file-based history (for backward compatibility)
        for dir_entry in fs::read_dir("mcp_servers").unwrap_or_else(|_| return fs::read_dir(".").unwrap()) {
            if let Ok(entry) = dir_entry {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    
                    // Skip if we already have this conversation
                    if self.servers.contains(&name) {
                        continue;
                    }
                    
                    self.servers.push(name.clone());
                    self.messages.insert(name.clone(), load_history(&name).unwrap_or_default());
                }
            }
        }
    }

    fn current_server(&self) -> String {
        self.servers
            .get(self.state.selected().unwrap_or(0))
            .cloned()
            .unwrap_or_else(|| "All".to_string())
    }

    fn current_messages(&self) -> Vec<(String, Message)> {
        let server = self.current_server();
        if server == "All" {
            let mut res = Vec::new();
            for (srv, msgs) in &self.messages {
                for m in msgs {
                    res.push((srv.clone(), m.clone()));
                }
            }
            res
        } else {
            self
                .messages
                .get(&server)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|m| (server.clone(), m))
                .collect()
        }
    }
    
    fn get_conversation_id_by_index(&self, index: usize) -> Option<i64> {
        // Reverse lookup from index to conversation ID
        for (conv_id, idx) in &self.conv_id_to_index {
            if *idx == index {
                return Some(*conv_id);
            }
        }
        None
    }
    
    fn get_conversation_id_by_title(&self, title: &str) -> Option<i64> {
        // Find the conversation by title
        self.conversations.iter()
            .find(|c| c.title == title)
            .map(|c| c.id)
    }
    
    // Vim-style navigation methods
    
    /// Move chat scroll position up (k in vim)
    fn scroll_up(&mut self) {
        if self.chat_scroll_position > 0 {
            self.chat_scroll_position -= 1;
        }
    }
    
    /// Move chat scroll position down (j in vim)
    fn scroll_down(&mut self) {
        let message_count = self.current_messages().len();
        if message_count > 0 && self.chat_scroll_position < message_count - 1 {
            self.chat_scroll_position += 1;
        }
    }
    
    /// Move chat scroll position up by half a page (Ctrl+u in vim)
    fn scroll_half_page_up(&mut self, page_size: usize) {
        let half_page = page_size / 2;
        if self.chat_scroll_position > half_page {
            self.chat_scroll_position -= half_page;
        } else {
            self.chat_scroll_position = 0;
        }
    }
    
    /// Move chat scroll position down by half a page (Ctrl+d in vim)
    fn scroll_half_page_down(&mut self, page_size: usize) {
        let message_count = self.current_messages().len();
        let half_page = page_size / 2;
        
        if message_count > 0 {
            if self.chat_scroll_position + half_page < message_count {
                self.chat_scroll_position += half_page;
            } else {
                self.chat_scroll_position = message_count - 1;
            }
        }
    }
    
    /// Move chat scroll position up a full page (Ctrl+b in vim)
    fn scroll_page_up(&mut self, page_size: usize) {
        if self.chat_scroll_position > page_size {
            self.chat_scroll_position -= page_size;
        } else {
            self.chat_scroll_position = 0;
        }
    }
    
    /// Move chat scroll position down a full page (Ctrl+f in vim)
    fn scroll_page_down(&mut self, page_size: usize) {
        let message_count = self.current_messages().len();
        
        if message_count > 0 {
            if self.chat_scroll_position + page_size < message_count {
                self.chat_scroll_position += page_size;
            } else {
                self.chat_scroll_position = message_count - 1;
            }
        }
    }
    
    /// Go to top of chat (gg in vim)
    fn scroll_to_top(&mut self) {
        self.chat_scroll_position = 0;
    }
    
    /// Go to bottom of chat (G in vim)
    fn scroll_to_bottom(&mut self) {
        let message_count = self.current_messages().len();
        if message_count > 0 {
            self.chat_scroll_position = message_count - 1;
        } else {
            self.chat_scroll_position = 0;
        }
    }
    
    /// Switch to normal mode (Esc in vim)
    fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_mode = None;
    }
    
    /// Switch to insert mode (i in vim)
    fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
        self.search_mode = None;
    }
    
    /// Start forward search (/ in vim)
    fn start_forward_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_mode = Some(SearchMode::Forward);
        self.search_query.clear();
    }
    
    /// Start backward search (? in vim)
    fn start_backward_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_mode = Some(SearchMode::Backward);
        self.search_query.clear();
    }
    
    /// Search for the current query in the chat history
    fn search_in_chat(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        
        let messages = self.current_messages();
        if messages.is_empty() {
            return;
        }
        
        let search_query = self.search_query.to_lowercase();
        let current_pos = self.chat_scroll_position;
        
        // Search direction depends on search mode
        if let Some(mode) = self.search_mode {
            match mode {
                SearchMode::Forward => {
                    // Search from current position downward
                    for i in current_pos+1..messages.len() {
                        if messages[i].1.content.to_lowercase().contains(&search_query) {
                            self.chat_scroll_position = i;
                            return;
                        }
                    }
                    
                    // If not found, wrap around to the beginning
                    for i in 0..=current_pos {
                        if messages[i].1.content.to_lowercase().contains(&search_query) {
                            self.chat_scroll_position = i;
                            return;
                        }
                    }
                },
                SearchMode::Backward => {
                    // Search from current position upward
                    if current_pos > 0 {
                        for i in (0..current_pos).rev() {
                            if messages[i].1.content.to_lowercase().contains(&search_query) {
                                self.chat_scroll_position = i;
                                return;
                            }
                        }
                    }
                    
                    // If not found, wrap around to the end
                    for i in (current_pos..messages.len()).rev() {
                        if messages[i].1.content.to_lowercase().contains(&search_query) {
                            self.chat_scroll_position = i;
                            return;
                        }
                    }
                }
            }
        }
    }
}

// This function has been replaced by App::load_conversations

fn load_history(server: &str) -> io::Result<Vec<Message>> {
    let path = format!("mcp_servers/{}/history.json", server);
    if Path::new(&path).exists() {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data).unwrap_or_default())
    } else {
        Ok(Vec::new())
    }
}

fn save_history(server: &str, msgs: &[Message]) -> io::Result<()> {
    let dir = format!("mcp_servers/{}", server);
    fs::create_dir_all(&dir)?;
    let path = format!("{}/history.json", dir);
    fs::write(path, serde_json::to_string_pretty(msgs).unwrap())?;
    Ok(())
}

// Navigation Domain - handles server/conversation selection
fn handle_navigation_up(app: &mut App) {
    if let Some(selected) = app.state.selected() {
        let new = selected.saturating_sub(1);
        app.state.select(Some(new));
    }
}

fn handle_navigation_down(app: &mut App) {
    if let Some(selected) = app.state.selected() {
        let new = (selected + 1).min(app.servers.len().saturating_sub(1));
        app.state.select(Some(new));
    }
}

// Conversation Management Domain - handles conversation creation and management
fn handle_conversation_creation(
    app: &mut App,
    config: &mut AppConfig,
    store: &ConversationStore,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut server = app.current_server();
    
    if server == "None" {
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
        
        // Add to servers list
        app.servers.push(conv_title.clone());
        let index = app.servers.len() - 1;
        app.state.select(Some(index));
        app.conv_id_to_index.insert(conversation_id, index);
        app.messages.insert(conv_title.clone(), Vec::new());
        server = conv_title;
        
        // Clear active MCP servers for new conversation
        app.active_mcp_servers.clear();
    } else if server != "All" {
        // If selecting an existing conversation, set it as current
        let selected_idx = app.state.selected().unwrap_or(0);
        
        if let Some(conv_id) = app.get_conversation_id_by_index(selected_idx) {
            app.current_conversation_id = Some(conv_id);
            config.conversation_id = Some(conv_id);
            config.save()?;
            
            // Load MCP servers for this conversation
            app.active_mcp_servers = store.get_enabled_servers(conv_id)?;
        } else if let Some(conv_id) = app.get_conversation_id_by_title(&server) {
            // Direct lookup by title if index lookup failed
            app.current_conversation_id = Some(conv_id);
            config.conversation_id = Some(conv_id);
            config.save()?;
            
            // Load MCP servers for this conversation
            app.active_mcp_servers = store.get_enabled_servers(conv_id)?;
        }
    }
    
    if server == "All" {
        server = app.servers
            .iter()
            .find(|s| *s != "All" && *s != "None")
            .cloned()
            .unwrap_or_else(|| "".to_string());
    }
    
    Ok(server)
}

// Message Processing Domain - handles chat message processing
async fn handle_message_processing(
    app: &mut App,
    server: &str,
    oaiclient: &AzureOpenAIClient,
) -> Result<(), Box<dyn std::error::Error>> {
    if !server.is_empty() && !app.input.is_empty() {
        let msgs = app.messages.entry(server.to_string()).or_default();
        
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
            // Create ticket server and handle the command
            let ticket_server = TicketMCPServer::new("ticket", &CredentialManager::new());
            let response = ticket_server.handle_command(cmd);
            
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

// Persistence Domain - handles saving conversation data
fn handle_persistence(
    app: &App,
    server: &str,
    store: &ConversationStore,
) -> Result<(), Box<dyn std::error::Error>> {
    let empty_vec = Vec::new();
    let msgs = app.messages.get(server).unwrap_or(&empty_vec);
    
    if let Some(cid) = app.current_conversation_id {
        for m in msgs {
            let _ = store.add_message(cid, m);
        }
    } else {
        let _ = save_history(server, msgs);
    }
    
    Ok(())
}

// Input Processing Domain - handles text input
fn handle_text_input(app: &mut App, c: char) {
    app.input.push(c);
}

fn handle_backspace(app: &mut App) {
    app.input.pop();
}

// Event Handling Domain - processes keyboard events
async fn handle_key_event(
    key_code: KeyCode,
    app: &mut App,
    config: &mut AppConfig,
    store: &ConversationStore,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &Option<MCPServerManager>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Handle keys differently based on the current input mode
    match app.input_mode {
        InputMode::Normal => {
            // In normal mode, handle vim-style navigation
            match key_code {
                KeyCode::Char('q') => return Ok(true), // Signal to quit
                KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Char('g') => app.scroll_to_top(), // gg in vim
                KeyCode::Char('G') => app.scroll_to_bottom(),
                KeyCode::Char('u') => app.scroll_half_page_up(10), // Simplified Ctrl+u
                KeyCode::Char('d') => app.scroll_half_page_down(10), // Simplified Ctrl+d
                KeyCode::Char('b') => app.scroll_page_up(20), // Simplified Ctrl+b
                KeyCode::Char('f') => app.scroll_page_down(20), // Simplified Ctrl+f
                KeyCode::Char('i') => app.enter_insert_mode(),
                KeyCode::Char('/') => app.start_forward_search(),
                KeyCode::Char('?') => app.start_backward_search(),
                KeyCode::Char('n') => app.search_in_chat(), // Repeat last search
                KeyCode::Up => handle_navigation_up(app),
                KeyCode::Down => handle_navigation_down(app),
                // Toggle MCP server with M key
                KeyCode::Char('M') => {
                    if app.current_conversation_id.is_some() && !app.mcp_servers.is_empty() {
                        if let Some(server_name) = app.mcp_servers.keys().next().cloned() {
                            let is_active = app.active_mcp_servers.contains(&server_name);
                            
                            if is_active {
                                app.active_mcp_servers.retain(|s| s != &server_name);
                                if let Some(conv_id) = app.current_conversation_id {
                                    let _ = store.disable_mcp_server(conv_id, &server_name);
                                }
                            } else {
                                app.active_mcp_servers.push(server_name.clone());
                                if let Some(conv_id) = app.current_conversation_id {
                                    let _ = store.enable_mcp_server(conv_id, &server_name);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        },
        
        InputMode::Insert => {
            // In insert mode, handle text input and commands
            match key_code {
                KeyCode::Char('q') => {
                    if app.input.is_empty() {
                        return Ok(true); // Signal to quit if input is empty
                    } else {
                        handle_text_input(app, 'q');
                    }
                },
                KeyCode::Up => handle_navigation_up(app),
                KeyCode::Down => handle_navigation_down(app),
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Enter => {
                    let server = handle_conversation_creation(app, config, store)?;
                    
                    // Process MCP server mentions if available
                    if let Some(manager) = mcp_manager {
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
                    
                    handle_message_processing(app, &server, oaiclient).await?;
                    handle_persistence(app, &server, store)?;
                    
                    // After sending a message, scroll to the bottom
                    app.scroll_to_bottom();
                }
                KeyCode::Backspace => handle_backspace(app),
                KeyCode::Char('m') => {
                    // In insert mode, 'm' is treated as regular input
                    handle_text_input(app, 'm');
                }
                // Handle all other character input in insert mode
                KeyCode::Char(c) => handle_text_input(app, c),
                _ => {}
            }
        },
        
        InputMode::Command => {
            // Handle command mode input (like vim's command line)
            match key_code {
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Enter => {
                    // Process command - for now, just go back to normal mode
                    app.enter_normal_mode();
                },
                KeyCode::Backspace => {
                    if !app.search_query.is_empty() {
                        app.search_query.pop();
                    }
                },
                KeyCode::Char(c) => {
                    app.search_query.push(c);
                },
                _ => {}
            }
        },
        
        InputMode::Visual => {
            // In visual mode (for selecting text/messages)
            match key_code {
                KeyCode::Esc => app.enter_normal_mode(),
                KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Up => handle_navigation_up(app),
                KeyCode::Down => handle_navigation_down(app),
                _ => {}
            }
        }
    }
    
    // Handle search mode separately (can be active in any mode)
    if let Some(_mode) = app.search_mode {
        match key_code {
            KeyCode::Esc => {
                app.search_mode = None;
                app.search_query.clear();
            }
            KeyCode::Enter => {
                app.search_in_chat();
                app.search_mode = None; // Exit search mode after search
            }
            KeyCode::Backspace => {
                if !app.search_query.is_empty() {
                    app.search_query.pop();
                }
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
            }
            _ => {}
        }
    }
    
    Ok(false) // Continue running
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    
    // Initialize credential manager (loads env vars and .zshenv)
    let credential_manager = CredentialManager::new();
    
    // Check if we're running in a proper terminal
    let has_tty = crossterm::tty::IsTty::is_tty(&io::stdout()) 
        && crossterm::tty::IsTty::is_tty(&io::stdin());
    
    if !has_tty {
        eprintln!("Error: This application requires a TTY terminal to run.");
        eprintln!("Please run this application in a proper terminal environment.");
        eprintln!("If you're using VS Code, try running it in the integrated terminal.");
        eprintln!("If you're using a CI/automated environment, this application is not suitable for that context.");
        std::process::exit(1);
    }
    
    // Load config and store
    let mut config = AppConfig::load().unwrap_or_default();
    let db_path = config.db_path.clone();
    let store = ConversationStore::new(db_path)?;
    
    // We can now use the credential manager to get credentials
    let api_key = credential_manager.get("AZURE_OPENAI_API_KEY")
        .or_else(|| credential_manager.get("AZURE_OPENAI_KEY"))
        .unwrap_or("");
    
    let api_base = credential_manager.get("AZURE_OPENAI_ENDPOINT").unwrap_or("");
    let deployment_id = credential_manager.get("AZURE_OPENAI_DEPLOYMENT_ID").unwrap_or("");
    let api_version = credential_manager.get("AZURE_OPENAI_API_VERSION").unwrap_or("2024-02-15-preview");
    let model = credential_manager.get("AZURE_OPENAI_MODEL").unwrap_or("gpt-4.1");
    
    let oaiconfig = AzureOpenAIConfig {
        api_key: api_key.to_string(),
        api_base: api_base.to_string(),
        deployment_id: deployment_id.to_string(),
        api_version: api_version.to_string(),
        model: model.to_string(),
    };
    
    // Check if we have required credentials
    if api_key.is_empty() || api_base.is_empty() || deployment_id.is_empty() {
        eprintln!("Warning: Missing required Azure OpenAI credentials.");
        eprintln!("Please set the following variables in your environment or ~/.zshenv:");
        eprintln!("- AZURE_OPENAI_API_KEY (or AZURE_OPENAI_KEY)");
        eprintln!("- AZURE_OPENAI_ENDPOINT");
        eprintln!("- AZURE_OPENAI_DEPLOYMENT_ID");
    }
    
    let oaiclient = AzureOpenAIClient::new(oaiconfig);
    
    // Setup MCP server manager
    let mcp_base_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/spidey-shell/mcp_servers");
    
    let mcp_manager = match MCPServerManager::new(mcp_base_dir) {
        Ok(mut manager) => {
            // Initialize the ticket MCP server
            let ticket_server = TicketMCPServer::new("ticket", &credential_manager);
            
            // Create and initialize the ticket MCP server configuration
            let ticket_config = MCPServerConfig {
                name: ticket_server.name().to_string(),
                url: None,
                api_key: None,
                description: Some("Ticket management system integration".to_string()),
                capabilities: vec![
                    "list tickets".to_string(),
                    "view ticket details".to_string(),
                    "show ticket comments".to_string()
                ],
                enabled: true,
            };
            
            // Add the ticket server to the MCP manager
            println!("Initializing Ticket MCP server...");
            if let Err(e) = manager.add_server(ticket_config) {
                eprintln!("Warning: Failed to add ticket MCP server: {}", e);
            }
            
            Some(manager)
        },
        Err(e) => {
            eprintln!("Warning: Failed to initialize MCP server manager: {}", e);
            eprintln!("MCP server functionality will be disabled.");
            None
        }
    };
    
    // Setup terminal with better error handling
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("Failed to setup terminal: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;
    
    let mut app = App::new(&mcp_manager, &store);
    let res = run_app(&mut terminal, &mut app, &mut config, &store, &oaiclient, &mcp_manager).await;
    
    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("Error: {}", err);
    }
    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config: &mut AppConfig,
    store: &ConversationStore,
    oaiclient: &AzureOpenAIClient,
    mcp_manager: &Option<MCPServerManager>,
) -> Result<(), Box<dyn std::error::Error>>
where
    B::Error: 'static + std::error::Error,
{
    loop {
        // UI Rendering Domain
        terminal.draw(|f| ui(f, app))?;
        
        // Event Polling Domain
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Event Handling Domain - delegate to domain-specific function
                let should_quit = handle_key_event(key.code, app, config, store, oaiclient, mcp_manager).await?;
                if should_quit {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.area());

    // Defensive check: only render if there are at least 2 chunks
    if chunks.len() >= 2 {
        // Define layout for the left sidebar
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(8)])
            .split(chunks[0]);
            
        // Server list (conversations)
        let items: Vec<ListItem> = app.servers.iter().enumerate().map(|(index, s)| {
            let style = if app.current_server() == *s {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            
            // Get the conversation details if this is a stored conversation
            let display_text = if let Some(conv_id) = app.get_conversation_id_by_index(index) {
                // Find the conversation in the list
                if let Some(conv) = app.conversations.iter().find(|c| c.id == conv_id) {
                    let datetime = chrono::DateTime::from_timestamp(conv.created_at, 0)
                        .map(|dt| format!("{}", dt.format("%Y-%m-%d %H:%M")))
                        .unwrap_or_else(|| "Unknown date".to_string());
                    
                    // Show timestamp for conversations
                    format!("{} ({})", s, datetime)
                } else {
                    s.to_string()
                }
            } else {
                s.to_string()
            };
            
            ListItem::new(display_text).style(style)
        }).collect();
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Conversations"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ");
        f.render_stateful_widget(list, left_chunks[0], &mut app.state.clone());
        
        // MCP server list with enhanced display
        if !app.mcp_servers.is_empty() {
            // Sort servers by active status first, then by name
            let mut sorted_servers: Vec<(&String, &bool)> = app.mcp_servers.iter().collect();
            sorted_servers.sort_by(|a, b| {
                let a_active = app.active_mcp_servers.contains(a.0);
                let b_active = app.active_mcp_servers.contains(b.0);
                
                // First sort by active status (active first)
                if a_active && !b_active {
                    return std::cmp::Ordering::Less;
                } else if !a_active && b_active {
                    return std::cmp::Ordering::Greater;
                }
                
                // Then sort by name
                a.0.cmp(b.0)
            });
            
            let mcp_items: Vec<ListItem> = sorted_servers.iter()
                .map(|(name, &enabled)| {
                    let is_active = app.active_mcp_servers.contains(*name);
                    
                    // Different prefixes for active/inactive servers
                    let prefix = if is_active {
                        "[✓] "
                    } else {
                        "[ ] "
                    };
                    
                    // Add a symbol to show server status
                    let status_symbol = if enabled {
                        "● " // Connected 
                    } else {
                        "○ " // Disconnected
                    };
                    
                    // Different styles for active/inactive servers
                    let style = if enabled && is_active {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if enabled {
                        Style::default().fg(Color::White)
                    } else if is_active {
                        Style::default().fg(Color::Yellow) // Warning - active but disabled
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    
                    ListItem::new(format!("{}{}{}", prefix, status_symbol, name)).style(style)
                })
                .collect();
            
            // Create a list title with counts
            let active_count = app.active_mcp_servers.len();
            let total_count = app.mcp_servers.len();
            let title = format!("MCP Servers [{}/{}]", active_count, total_count);
            
            let mcp_list = List::new(mcp_items)
                .block(Block::default().borders(Borders::ALL)
                    .title(title)
                    .title_style(if active_count > 0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    })
                );
            f.render_widget(mcp_list, left_chunks[1]);
            
            // Add instructions below MCP servers if there's space
            if left_chunks[1].height > app.mcp_servers.len() as u16 + 3 {
                let help_text = "Press 'M' to toggle server\nUse '@server' in messages\nEsc for normal mode";
                let help = Paragraph::new(help_text)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Left);
                    
                // Create a small area below the MCP servers list
                let help_area = Rect {
                    x: left_chunks[1].x + 1,
                    y: left_chunks[1].y + app.mcp_servers.len() as u16 + 2,
                    width: left_chunks[1].width - 2,
                    height: left_chunks[1].height - app.mcp_servers.len() as u16 - 3,
                };
                
                if help_area.height >= 3 {
                    f.render_widget(help, help_area);
                }
            }
        }

        // Chat and input area
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(chunks[1]);

        // Messages with MCP server indicators
        // Get current messages and apply highlighting for the selected message
        let current_messages = app.current_messages();
        let messages: Vec<ListItem> = current_messages
            .iter()
            .enumerate()
            .map(|(idx, (srv, m))| {
                let line = if app.current_server() == "All" {
                    format!("[{}] {}: {}", srv, m.role, m.content)
                } else {
                    format!("{}: {}", m.role, m.content)
                };
                
                // Highlight the current selected message in normal mode
                let style = if app.input_mode == InputMode::Normal && idx == app.chat_scroll_position {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    match m.role.as_str() {
                        "assistant" => Style::default().fg(Color::Green),
                        "user" => Style::default().fg(Color::White),
                        "system" => Style::default().fg(Color::Cyan),
                        "error" => Style::default().fg(Color::Red),
                        _ => Style::default(),
                    }
                };
                
                ListItem::new(line).style(style)
            })
            .collect();
            
        // Build the history title showing active MCP servers and navigation info
        let mut history_title = "History".to_string();
        if !app.active_mcp_servers.is_empty() {
            history_title = format!("{} (MCP: {})", history_title, 
                app.active_mcp_servers.join(", "));
        }
        
        // Add scrolling position information
        if !current_messages.is_empty() {
            history_title = format!("{} [{}/{}]", 
                history_title, 
                app.chat_scroll_position + 1, 
                current_messages.len());
        }
        
        // Create list state with correct selection if in normal mode
        let mut list_state = ListState::default();
        if app.input_mode == InputMode::Normal && !current_messages.is_empty() {
            list_state.select(Some(app.chat_scroll_position));
        }
        
        let history = List::new(messages)
            .block(Block::default().borders(Borders::ALL).title(history_title))
            .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
            
        f.render_stateful_widget(history, inner[0], &mut list_state);

        // Input area with mode indicator and instructions
        let mode_str = match app.input_mode {
            InputMode::Normal => "NORMAL",
            InputMode::Insert => "INSERT",
            InputMode::Visual => "VISUAL",
            InputMode::Command => "COMMAND",
        };
        
        let search_str = if let Some(mode) = app.search_mode {
            match mode {
                SearchMode::Forward => format!("/ {}", app.search_query),
                SearchMode::Backward => format!("? {}", app.search_query),
            }
        } else {
            String::new()
        };
        
        let input_title = if !search_str.is_empty() {
            format!("{} - {} (Esc to cancel, Enter to search)", mode_str, search_str)
        } else if app.input_mode == InputMode::Normal {
            format!("{} - (j/k:scroll, /,?:search, i:insert, q:quit)", mode_str)
        } else {
            format!("{} - (Enter to send, Esc for normal mode, q to quit, @server to activate)", mode_str)
        };
        
        let input_style = Style::default().fg(match app.input_mode {
            InputMode::Normal => Color::Yellow,
            InputMode::Insert => Color::Green,
            InputMode::Visual => Color::Magenta,
            InputMode::Command => Color::Cyan,
        });
        
        let input = Paragraph::new(app.input.as_str())
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title(input_title));
        f.render_widget(input, inner[1]);
    }
}
