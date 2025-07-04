use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing::{info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use crate::persistence::config::AppConfig;

static INIT: Once = Once::new();
static mut LOG_DIR: Option<PathBuf> = None;

/// Initialize the global logger with a file appender
///
/// This function configures tracing to output logs only to a log file
/// in the config directory to avoid interfering with the terminal UI.
///
/// The logs are rotated by date (YYYY-MM-DD) format
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = AppConfig::config_dir();
    let log_dir = config_dir.join("logs");

    // Create logs directory if it doesn't exist
    fs::create_dir_all(&log_dir)?;
    
    // Store log directory for future reference
    unsafe {
        LOG_DIR = Some(log_dir.clone());
    }

    INIT.call_once(|| {
        // Create log file with current date in the filename
        let log_file_path = get_log_file_path(&log_dir);
        let file = match File::create(&log_file_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to create log file: {}", e);
                return;
            }
        };

        // Build a subscriber that writes to the log file only
        // No terminal output to avoid interfering with the UI
        let file_layer = fmt::layer()
            .with_writer(file)
            .with_ansi(false)  // No ANSI colors in file output
            .with_target(true);

        // Set up filter based on RUST_LOG env var or default to info level
        let filter_layer = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        // Combine layers and set as global subscriber
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(file_layer)
            .init();

        info!(
            "Tracing initialized. Logs will be stored at: {}",
            log_file_path.display()
        );
    });

    Ok(())
}

/// Get the path to the log file with current date in the filename
fn get_log_file_path(log_dir: &Path) -> PathBuf {
    let now = chrono::Utc::now();
    let filename = format!("spidey-shell-{}.log", now.format("%Y-%m-%d"));
    log_dir.join(filename)
}

/// Get the current log directory path if available
pub fn get_log_dir() -> Option<PathBuf> {
    // Use a safer approach to avoid mutable static reference issues
    let mut result = None;
    unsafe {
        if let Some(ref path) = LOG_DIR {
            result = Some(path.clone());
        }
    }
    result
}

/// Write a message to the current log file
/// Useful when you need to log something outside the tracing system
pub fn write_to_log(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(log_dir) = get_log_dir() {
        let log_file_path = get_log_file_path(&log_dir);
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file_path)?;

        let now = chrono::Utc::now().to_rfc3339();
        let formatted_message = format!("[{}] [MANUAL_LOG] {}\n", now, message);
        file.write_all(formatted_message.as_bytes())?;
        Ok(())
    } else {
        warn!("Attempted to write to log file before initialization");
        Err("Logger not initialized".into())
    }
}
