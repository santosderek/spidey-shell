use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use tracing::{info, warn, error};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AppConfig {
    pub conversation_id: Option<i64>,
    pub default_parent: Option<String>,
    pub db_path: PathBuf,
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/spidey-shell")
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> io::Result<Self> {
        let path = Self::config_file();
        if path.exists() {
            info!("Loading configuration from {}", path.display());
            let mut file = match fs::File::open(&path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to open config file: {}", e);
                    return Err(e);
                }
            };
            
            let mut content = String::new();
            if let Err(e) = file.read_to_string(&mut content) {
                error!("Failed to read config file: {}", e);
                return Err(e);
            }
            
            match serde_json::from_str(&content) {
                Ok(config) => {
                    info!("Configuration loaded successfully");
                    Ok(config)
                },
                Err(e) => {
                    warn!("Failed to parse config file, using default configuration: {}", e);
                    Ok(AppConfig::default())
                }
            }
        } else {
            info!("Config file not found at {}, using default configuration", path.display());
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let dir = Self::config_dir();
        info!("Ensuring config directory exists at {}", dir.display());
        if let Err(e) = fs::create_dir_all(&dir) {
            error!("Failed to create config directory: {}", e);
            return Err(e);
        }
        
        let path = Self::config_file();
        info!("Saving configuration to {}", path.display());
        let mut file = match fs::File::create(&path) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create config file: {}", e);
                return Err(e);
            }
        };
        
        let content = match serde_json::to_string_pretty(self) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to serialize config: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        };
        
        if let Err(e) = file.write_all(content.as_bytes()) {
            error!("Failed to write config file: {}", e);
            return Err(e);
        }
        
        info!("Configuration saved successfully");
        Ok(())
    }
}
