use std::collections::HashMap;
use std::error::Error;
use ratatui::widgets::ListState;

use crate::openai::Message;
use crate::persistence::{Conversation, seaorm_store::SeaOrmStore};
use crate::mcp::MCPServerManager;
use crate::persistence::config::AppConfig;
use crate::persistence::entities::conversation::Model as ConversationModel;

use super::models::{InputMode, SearchMode, ConversationMode, TitledMessage};

/// Core application state using SeaORM
pub struct SeaOrmAppState {
    // Conversation and UI state
    pub conversations_list: Vec<String>,
    pub conversation_modes: Vec<ConversationMode>,
    pub state: ListState,
    pub input: String,
    pub messages: HashMap<String, Vec<Message>>,
    pub current_conversation_id: Option<i32>,
    
    // MCP Server state
    pub mcp_servers: HashMap<String, bool>, // MCP server name -> enabled
    pub active_mcp_servers: Vec<String>,    // Currently active MCP servers for this conversation
    
    // Database state
    pub conversations: Vec<ConversationModel>,
    pub conv_id_to_index: HashMap<i32, usize>,
    
    // Navigation state
    pub chat_scroll_position: usize,
    pub chat_message_selected: Option<usize>,
    pub input_mode: InputMode,
    pub search_query: String,
    pub search_mode: Option<SearchMode>,
}

impl SeaOrmAppState {
    pub async fn new(mcp_manager: &Option<MCPServerManager>, store: &SeaOrmStore) -> Self {
        let mut app = Self { 
            conversations_list: Vec::new(),
            conversation_modes: Vec::new(),
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
        app.load_conversations(store).await;
        
        // Initialize MCP server status
        if let Some(manager) = mcp_manager {
            for name in manager.get_server_names() {
                if let Some(config) = manager.get_server(&name) {
                    app.mcp_servers.insert(name, config.enabled);
                }
            }
        }
        
        // Select the first item if available
        if !app.conversations_list.is_empty() { 
            app.state.select(Some(0)); 
        }
        
        app
    }
    
    /// Load conversations from the database using SeaORM
    pub async fn load_conversations(&mut self, store: &SeaOrmStore) {
        // Clear current data
        self.conversations_list.clear();
        self.conversation_modes.clear();
        self.messages.clear();
        self.conv_id_to_index.clear();
        self.conversations.clear();
        
        // Add special entries with their modes
        self.conversations_list.push("+ New Conversation".to_string());
        self.conversation_modes.push(ConversationMode::New);
        self.conversations_list.push("All Conversations".to_string());
        self.conversation_modes.push(ConversationMode::All);
        
        // Get conversations from the database
        if let Ok(conversations) = store.get_all_conversations().await {
            // Conversations are already sorted by created_at timestamp (newest first)
            
            for conversation in conversations {
                let title = conversation.title.clone();
                let conv_id = conversation.id;
                
                // Store the conversation
                self.conversations.push(conversation);
                
                // Map conversation ID to list index
                let index = self.conversations_list.len();
                self.conv_id_to_index.insert(conv_id, index);
                
                // Add to conversations list
                self.conversations_list.push(title.clone());
                self.conversation_modes.push(ConversationMode::None); // Regular conversation
                
                // Load messages
                if let Ok(msgs) = store.get_conversation_messages(conv_id).await {
                    self.messages.insert(title, msgs);
                } else {
                    self.messages.insert(title, Vec::new());
                }
            }
        }
    }

    /// Get the title of the currently selected conversation
    pub fn current_conversation(&self) -> String {
        self.conversations_list
            .get(self.state.selected().unwrap_or(0))
            .cloned()
            .unwrap_or_else(|| "All Conversations".to_string())
    }
    
    /// Check if the current selection is a special conversation mode
    pub fn is_special_conversation(&self) -> bool {
        if let Some(selected) = self.state.selected() {
            selected < 2 // First two items are special modes
        } else {
            true
        }
    }
    
    /// Get the mode of the currently selected conversation
    pub fn get_selected_mode(&self) -> ConversationMode {
        if let Some(selected) = self.state.selected() {
            if let Some(mode) = self.conversation_modes.get(selected) {
                return *mode;
            }
        }
        ConversationMode::None
    }

    /// Get messages for the current conversation with titles
    pub fn current_messages(&self) -> Vec<TitledMessage> {
        let conversation = self.current_conversation();
        if conversation == "All Conversations" {
            let mut res = Vec::new();
            for (srv, msgs) in &self.messages {
                for m in msgs {
                    res.push(TitledMessage::new(srv.clone(), m.clone()));
                }
            }
            res
        } else {
            self
                .messages
                .get(&conversation)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|m| TitledMessage::new(conversation.clone(), m))
                .collect()
        }
    }
    
    /// Find a conversation ID from its index in the list
    pub fn get_conversation_id_by_index(&self, index: usize) -> Option<i32> {
        // Reverse lookup from index to conversation ID
        for (conv_id, idx) in &self.conv_id_to_index {
            if *idx == index {
                return Some(*conv_id);
            }
        }
        None
    }
    
    /// Find a conversation ID from its title
    pub fn get_conversation_id_by_title(&self, title: &str) -> Option<i32> {
        // Find the conversation by title
        self.conversations.iter()
            .find(|c| c.title == title)
            .map(|c| c.id)
    }
    
    // Navigation methods
    
    /// Move chat scroll position up (k in vim)
    pub fn scroll_up(&mut self) {
        if self.chat_scroll_position > 0 {
            self.chat_scroll_position -= 1;
        }
    }
    
    /// Move chat scroll position down (j in vim)
    pub fn scroll_down(&mut self) {
        let message_count = self.current_messages().len();
        if message_count > 0 && self.chat_scroll_position < message_count - 1 {
            self.chat_scroll_position += 1;
        }
    }
    
    /// Move chat scroll position up by half a page (Ctrl+u in vim)
    pub fn scroll_half_page_up(&mut self, page_size: usize) {
        let half_page = page_size / 2;
        if self.chat_scroll_position > half_page {
            self.chat_scroll_position -= half_page;
        } else {
            self.chat_scroll_position = 0;
        }
    }
    
    /// Move chat scroll position down by half a page (Ctrl+d in vim)
    pub fn scroll_half_page_down(&mut self, page_size: usize) {
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
    pub fn scroll_page_up(&mut self, page_size: usize) {
        if self.chat_scroll_position > page_size {
            self.chat_scroll_position -= page_size;
        } else {
            self.chat_scroll_position = 0;
        }
    }
    
    /// Move chat scroll position down a full page (Ctrl+f in vim)
    pub fn scroll_page_down(&mut self, page_size: usize) {
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
    pub fn scroll_to_top(&mut self) {
        self.chat_scroll_position = 0;
    }
    
    /// Go to bottom of chat (G in vim)
    pub fn scroll_to_bottom(&mut self) {
        let message_count = self.current_messages().len();
        if message_count > 0 {
            self.chat_scroll_position = message_count - 1;
        } else {
            self.chat_scroll_position = 0;
        }
    }
    
    // Input mode methods
    
    /// Switch to normal mode (Esc in vim)
    pub fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_mode = None;
    }
    
    /// Switch to insert mode (i in vim)
    pub fn enter_insert_mode(&mut self) {
        self.input_mode = InputMode::Insert;
        self.search_mode = None;
    }
    
    /// Start forward search (/ in vim)
    pub fn start_forward_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_mode = Some(SearchMode::Forward);
        self.search_query.clear();
    }
    
    /// Start backward search (? in vim)
    pub fn start_backward_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_mode = Some(SearchMode::Backward);
        self.search_query.clear();
    }
    
    /// Search for the current query in the chat history
    pub fn search_in_chat(&mut self) {
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
                        if messages[i].message.content.to_lowercase().contains(&search_query) {
                            self.chat_scroll_position = i;
                            return;
                        }
                    }
                    
                    // If not found, wrap around to the beginning
                    for i in 0..=current_pos {
                        if messages[i].message.content.to_lowercase().contains(&search_query) {
                            self.chat_scroll_position = i;
                            return;
                        }
                    }
                },
                SearchMode::Backward => {
                    // Search from current position upward
                    if current_pos > 0 {
                        for i in (0..current_pos).rev() {
                            if messages[i].message.content.to_lowercase().contains(&search_query) {
                                self.chat_scroll_position = i;
                                return;
                            }
                        }
                    }
                    
                    // If not found, wrap around to the end
                    for i in (current_pos..messages.len()).rev() {
                        if messages[i].message.content.to_lowercase().contains(&search_query) {
                            self.chat_scroll_position = i;
                            return;
                        }
                    }
                }
            }
        }
    }
}