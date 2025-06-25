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
}

