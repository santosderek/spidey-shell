use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

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
            let mut file = fs::File::open(&path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let path = Self::config_file();
        let mut file = fs::File::create(path)?;
        let content = serde_json::to_string_pretty(self).unwrap();
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
