use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

// Import from the sibling modules
use crate::mcp::discovery::MCPProjectDiscovery;
use crate::mcp::python::{MCPProcessStatus, MCPPythonConfig, MCPPythonProcess};

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
    python_processes: HashMap<String, MCPPythonProcess>,
    project_discovery: Option<MCPProjectDiscovery>,
}

impl MCPServerManager {
    /// Create a new MCP server manager
    pub fn new(base_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&base_dir)?;

        // Create the projects directory for Python MCP projects
        let projects_dir = base_dir.join("projects");
        fs::create_dir_all(&projects_dir)?;

        let project_discovery = MCPProjectDiscovery::new(projects_dir);

        let mut manager = Self {
            servers: HashMap::new(),
            base_dir,
            python_processes: HashMap::new(),
            project_discovery: Some(project_discovery),
        };

        manager.load_all()?;
        Ok(manager)
    }

    /// Load all server configurations
    pub fn load_all(&mut self) -> io::Result<()> {
        self.servers.clear();
        self.python_processes.clear();

        // Load server configs from base_dir
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name != "projects" {
                        // Skip the projects directory
                        if let Ok(config) = MCPServerConfig::load(&name, &self.base_dir) {
                            self.servers.insert(name, config);
                        }
                    }
                }
            }
        }

        // Discover Python projects if project discovery is available
        if let Some(discovery) = &mut self.project_discovery {
            match discovery.scan_projects() {
                Ok(projects) => {
                    info!("Found {} Python MCP projects", projects.len());

                    // Convert projects to server configs
                    for project in projects.iter().filter(|p| p.is_valid) {
                        let name = &project.name;

                        // Create or update server config
                        if let Some(existing) = self.servers.get_mut(name) {
                            // Update existing config
                            if existing.description.is_none() {
                                existing.description = project.metadata.description.clone();
                            }
                            if existing.capabilities.is_empty() {
                                existing.capabilities = project.metadata.mcp_capabilities.clone();
                            }
                        } else {
                            // Create new config
                            let mut config = MCPServerConfig::new(name);
                            config.description = project.metadata.description.clone();
                            config.capabilities = project.metadata.mcp_capabilities.clone();
                            self.servers.insert(name.clone(), config);
                        }

                        // Create Python config for the project
                        let (name, project_path, package_name, description) =
                            discovery.create_python_config_data(project);

                        let mut python_config =
                            MCPPythonConfig::new(&name, project_path, &package_name);

                        // Set environment variables if description is available
                        if let Some(desc) = description {
                            let mut env = std::collections::HashMap::new();
                            env.insert("MCP_DESCRIPTION".to_string(), desc);
                            python_config.env = env;
                        }

                        // Create Python process
                        let process = MCPPythonProcess::new(python_config);
                        self.python_processes.insert(name.clone(), process);
                    }
                }
                Err(e) => {
                    error!("Failed to scan for Python MCP projects: {}", e);
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

    /// Start a Python MCP subprocess
    pub fn start_python_process(&mut self, name: &str) -> io::Result<bool> {
        if let Some(process) = self.python_processes.get_mut(name) {
            process.start()?;
            Ok(true)
        } else {
            warn!("No Python process registered for server: {}", name);
            Ok(false)
        }
    }

    /// Stop a Python MCP subprocess
    pub fn stop_python_process(&mut self, name: &str) -> io::Result<bool> {
        if let Some(process) = self.python_processes.get_mut(name) {
            process.stop()?;
            Ok(true)
        } else {
            warn!("No Python process registered for server: {}", name);
            Ok(false)
        }
    }

    /// Get status of a Python MCP subprocess
    pub fn get_python_process_status(&self, name: &str) -> Option<MCPProcessStatus> {
        self.python_processes.get(name).map(|p| p.status())
    }

    /// Send a request to a Python MCP subprocess
    pub fn send_to_python_process(&mut self, name: &str, data: &str) -> io::Result<bool> {
        if let Some(process) = self.python_processes.get_mut(name) {
            // Ensure the process is running
            if !process.is_running() {
                if process.has_failed() {
                    info!("Restarting failed Python MCP process: {}", name);
                    process.stop()?;
                    process.start()?;
                } else if process.status() == MCPProcessStatus::NotRunning {
                    info!("Starting Python MCP process: {}", name);
                    process.start()?;
                }

                // Wait a bit for the process to start
                let start_time = std::time::Instant::now();
                while !process.is_running() && start_time.elapsed().as_secs() < 5 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if !process.is_running() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotConnected,
                        format!("Failed to start Python MCP process: {}", name),
                    ));
                }
            }

            // Send the data
            process.send(data)?;
            Ok(true)
        } else {
            warn!("No Python process registered for server: {}", name);
            Ok(false)
        }
    }

    /// Create a new Python MCP project
    pub fn create_python_project(&mut self, name: &str) -> io::Result<PathBuf> {
        if let Some(discovery) = &self.project_discovery {
            let project_dir = discovery.create_project(name)?;

            // Reload to discover the new project
            self.load_all()?;

            Ok(project_dir)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Project discovery is not available",
            ))
        }
    }

    /// Start all Python MCP processes
    pub fn start_all_python_processes(&mut self) -> io::Result<usize> {
        let mut started = 0;
        for (name, process) in &mut self.python_processes {
            match process.start() {
                Ok(_) => {
                    info!("Started Python MCP process: {}", name);
                    started += 1;
                }
                Err(e) => {
                    error!("Failed to start Python MCP process {}: {}", name, e);
                }
            }
        }
        Ok(started)
    }

    /// Stop all Python MCP processes
    pub fn stop_all_python_processes(&mut self) -> io::Result<usize> {
        let mut stopped = 0;
        for (name, process) in &mut self.python_processes {
            match process.stop() {
                Ok(_) => {
                    info!("Stopped Python MCP process: {}", name);
                    stopped += 1;
                }
                Err(e) => {
                    error!("Failed to stop Python MCP process {}: {}", name, e);
                }
            }
        }
        Ok(stopped)
    }

    /// Install dependencies for a Python MCP project
    pub fn install_python_dependencies(&mut self, name: &str) -> io::Result<bool> {
        if let Some(discovery) = &self.project_discovery {
            if let Some(project) = discovery.get_project(name) {
                discovery.install_dependencies(&project.project_path)
            } else {
                warn!("No Python project found with name: {}", name);
                Ok(false)
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Project discovery is not available",
            ))
        }
    }
}
