use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::{self, BufRead};

/// Manages loading and caching of credential information
pub struct CredentialManager {
    credentials: HashMap<String, String>,
    zshenv_path: PathBuf,
}

impl CredentialManager {
    /// Create a new credential manager and load credentials from the environment
    pub fn new() -> Self {
        let mut manager = Self {
            credentials: HashMap::new(),
            zshenv_path: dirs::home_dir().unwrap_or_default().join(".zshenv"),
        };
        
        // Load credentials from the environment
        manager.load_from_env();
        
        // Try to load credentials from ~/.zshenv if it exists
        if manager.zshenv_path.exists() {
            let _ = manager.load_from_zshenv();
        }
        
        manager
    }
    
    /// Add all environment variables to the credential manager
    fn load_from_env(&mut self) {
        for (key, value) in env::vars() {
            self.credentials.insert(key, value);
        }
    }
    
    /// Load credentials from ~/.zshenv file
    fn load_from_zshenv(&mut self) -> io::Result<()> {
        let file = fs::File::open(&self.zshenv_path)?;
        let reader = io::BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse export statements like: export KEY=value
            if line.starts_with("export ") {
                if let Some(kv) = line[7..].split_once('=') {
                    let key = kv.0.trim().to_string();
                    
                    // Handle quoted values - strip surrounding quotes if present
                    let mut value = kv.1.trim().to_string();
                    if (value.starts_with('"') && value.ends_with('"')) || 
                       (value.starts_with('\'') && value.ends_with('\'')) {
                        value = value[1..value.len()-1].to_string();
                    }
                    
                    self.credentials.insert(key, value);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a credential by name
    pub fn get(&self, name: &str) -> Option<&str> {
        self.credentials.get(name).map(|s| s.as_str())
    }
    
    /// Check if a credential exists
    pub fn has(&self, name: &str) -> bool {
        self.credentials.contains_key(name)
    }
    
    /// Get all credential names
    pub fn get_all_names(&self) -> Vec<&str> {
        self.credentials.keys().map(|s| s.as_str()).collect()
    }
    
    /// Filter credentials by prefix and return their names
    pub fn get_names_by_prefix(&self, prefix: &str) -> Vec<&str> {
        self.credentials
            .keys()
            .filter(|k| k.starts_with(prefix))
            .map(|s| s.as_str())
            .collect()
    }
    
    /// Get credentials as a HashMap
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.credentials
    }
}