pub mod openai;
pub mod persistence;
pub mod tickets;
pub mod credentials;
pub mod tickets_core;

// MCP related modules
pub mod mcp {
    pub mod python;
    pub mod discovery;
    pub mod manager;

    // Expose main types at the mcp module level for convenience
    pub use self::manager::MCPServerConfig;
    pub use self::manager::MCPServerManager;
}

// Application UI and state management
pub mod app;
