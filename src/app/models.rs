use crate::openai::Message;

/// Input mode for the application (vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,  // For navigation and commands
    Insert,  // For text input
    Visual,  // For selecting text/messages
    Command, // For command input (like :q in vim)
}

/// Search direction for vim-style search
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Forward,  // '/' search
    Backward, // '?' search
}

/// Special mode for the conversation list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationMode {
    None, // Regular conversation
    New,  // Create new conversation
    All,  // View all conversations
}

/// Represents a message along with its conversation title
#[derive(Debug, Clone)]
pub struct TitledMessage {
    pub title: String,
    pub message: Message,
}

impl TitledMessage {
    pub fn new(title: String, message: Message) -> Self {
        Self { title, message }
    }
}

/// MCP server status with additional metadata
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub name: String,
    pub is_enabled: bool,
    pub is_active: bool,
}

impl ServerStatus {
    pub fn new(name: String, is_enabled: bool, is_active: bool) -> Self {
        Self {
            name,
            is_enabled,
            is_active,
        }
    }
}
