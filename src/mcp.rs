use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Represents a single MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub name: String,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub enabled: bool,
}

impl MCPServerConfig {
    /// Create a new MCP server config
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            url: None,
            api_key: None,
            description: None,
            capabilities: Vec::new(),
            enabled: true,
        }
    }

    /// Save the server configuration to disk
    pub fn save(&self, base_dir: &Path) -> io::Result<()> {
        let server_dir = base_dir.join(&self.name);
        fs::create_dir_all(&server_dir)?;

        let config_path = server_dir.join("config.json");
        let content = serde_json::to_string_pretty(self).unwrap();
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Load the server configuration from disk
    pub fn load(name: &str, base_dir: &Path) -> io::Result<Self> {
        let config_path = base_dir.join(name).join("config.json");
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let config: MCPServerConfig = serde_json::from_str(&content).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse config.json: {}", e),
                )
            })?;
            Ok(config)
        } else {
            // Create a default config
            let config = MCPServerConfig::new(name);
            config.save(base_dir)?;
            Ok(config)
        }
    }
}

/// Manager for all MCP server configurations
pub struct MCPServerManager {
    pub servers: HashMap<String, MCPServerConfig>,
    pub base_dir: PathBuf,
}

impl MCPServerManager {
    /// Create a new MCP server manager
    pub fn new(base_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&base_dir)?;

        let mut manager = Self {
            servers: HashMap::new(),
            base_dir,
        };
        manager.load_all()?;
        Ok(manager)
    }

    /// Load all server configurations
    pub fn load_all(&mut self) -> io::Result<()> {
        self.servers.clear();

        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if let Ok(config) = MCPServerConfig::load(&name, &self.base_dir) {
                        self.servers.insert(name, config);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get server names
    pub fn get_server_names(&self) -> Vec<String> {
        self.servers.keys().cloned().collect()
    }

    /// Get a specific server configuration
    pub fn get_server(&self, name: &str) -> Option<&MCPServerConfig> {
        self.servers.get(name)
    }

    /// Add a new server configuration
    pub fn add_server(&mut self, config: MCPServerConfig) -> io::Result<()> {
        let name = config.name.clone();
        config.save(&self.base_dir)?;
        self.servers.insert(name, config);
        Ok(())
    }

    /// Remove a server configuration
    pub fn remove_server(&mut self, name: &str) -> io::Result<()> {
        if self.servers.remove(name).is_some() {
            let server_dir = self.base_dir.join(name);
            if server_dir.exists() {
                fs::remove_dir_all(server_dir)?;
            }
        }
        Ok(())
    }
    
    /// Parse input for @server commands
    /// 
    /// Returns a tuple containing:
    /// - The modified input with @server commands removed
    /// - A vector of server names mentioned in the input
    pub fn parse_server_commands(&self, input: &str) -> (String, Vec<String>) {
        let mut result = String::new();
        let mut servers = Vec::new();
        
        // Simple parser for @server mentions
        let mut in_mention = false;
        let mut current_mention = String::new();
        
        for c in input.chars() {
            if in_mention {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    current_mention.push(c);
                } else {
                    // End of mention
                    in_mention = false;
                    
                    if !current_mention.is_empty() {
                        // Check if it's a valid server
                        if self.servers.contains_key(&current_mention) {
                            servers.push(current_mention.clone());
                        } else {
                            // Not a valid server, restore the @ and the name
                            result.push('@');
                            result.push_str(&current_mention);
                        }
                        current_mention.clear();
                    }
                    
                    // Add the current character
                    result.push(c);
                }
            } else if c == '@' {
                in_mention = true;
            } else {
                result.push(c);
            }
        }
        
        // Check if we're still processing a mention at the end
        if in_mention && !current_mention.is_empty() {
            if self.servers.contains_key(&current_mention) {
                servers.push(current_mention);
            } else {
                // Not a valid server, restore the @ and the name
                result.push('@');
                result.push_str(&current_mention);
            }
        }
        
        (result.trim().to_string(), servers)
    }
}