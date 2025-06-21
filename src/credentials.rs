use std::collections::HashMap;
use std::env;

/// Manages loading and caching of credential information
pub struct CredentialManager {
    credentials: HashMap<String, String>,
}

impl CredentialManager {
    /// Create a new credential manager and load credentials from the environment
    pub fn new() -> Self {
        let mut manager = Self {
            credentials: HashMap::new(),
        };
        
        // Load credentials from the environment
        manager.load_from_env();
        
        manager
    }
    
    /// Add all environment variables to the credential manager
    fn load_from_env(&mut self) {
        for (key, value) in env::vars() {
            self.credentials.insert(key, value);
        }
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