//! Logging utilities
//!
//! This module provides functions for application logging

use crate::config::get_config;
use crate::utils::current_datetime;
use once_cell::sync::Lazy;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::Mutex;

// Global log file handle
static LOG_FILE: Lazy<Mutex<Option<File>>> = Lazy::new(|| Mutex::new(None));

/// Initialize logging based on configuration
pub fn init_logging() -> io::Result<()> {
    let config = get_config();

    if !config.logging_enabled {
        return Ok(());
    }

    // Get log file path
    let log_path = match &config.log_file {
        Some(path) => {
            let config_dir = crate::config::get_config_dir();
            config_dir.join(path)
        }
        None => return Ok(()), // No logging if path not specified
    };

    // Create parent directory if it doesn't exist
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Open log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Store in global handle
    let mut log_file = LOG_FILE.lock().unwrap();
    *log_file = Some(file);

    // Log startup message
    log_info(&format!("Logging started at {}", current_datetime()))?;

    Ok(())
}

/// Log an informational message
pub fn log_info(message: &str) -> io::Result<()> {
    log_message("INFO", message)
}

/// Log a warning message
pub fn log_warning(message: &str) -> io::Result<()> {
    log_message("WARN", message)
}

/// Log an error message
pub fn log_error(message: &str) -> io::Result<()> {
    log_message("ERROR", message)
}

/// Log a debug message
pub fn log_debug(message: &str) -> io::Result<()> {
    log_message("DEBUG", message)
}

/// Write a log message with the given level
fn log_message(level: &str, message: &str) -> io::Result<()> {
    let config = get_config();

    if !config.logging_enabled {
        return Ok(());
    }

    let timestamp = current_datetime();
    let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);

    let mut log_file = LOG_FILE.lock().unwrap();

    if let Some(file) = log_file.as_mut() {
        file.write_all(log_line.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}

/// Close the log file
pub fn close_logging() -> io::Result<()> {
    let mut log_file = LOG_FILE.lock().unwrap();

    if let Some(mut file) = log_file.take() {
        log_info(&format!("Logging stopped at {}", current_datetime()))?;
        file.flush()?;
    }

    Ok(())
}
