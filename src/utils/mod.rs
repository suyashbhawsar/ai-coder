//! Utility functions and structures
//!
//! This module provides common utilities for the application

mod format;
mod logging;

pub use format::*;
pub use logging::*;

use chrono::Local;
use ratatui::style::Color;
use std::env;
use std::path::{Path, PathBuf};

/// Color scheme for the application
pub struct Colors {
    /// Primary UI color
    pub primary: Color,
    /// Secondary UI color
    pub secondary: Color,
    /// Highlight/accent color
    pub accent: Color,
    /// Background color
    pub background: Color,
    /// Text/foreground color
    pub foreground: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            primary: Color::Rgb(0, 135, 175),   // Blue
            secondary: Color::Rgb(0, 175, 135), // Teal
            accent: Color::Rgb(175, 135, 0),    // Gold
            background: Color::Reset,           // Terminal default
            foreground: Color::Reset,           // Terminal default
        }
    }
}

/// Get the current time as a formatted string
pub fn current_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

/// Get current date as a formatted string
pub fn current_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// Get current date and time as a formatted string
pub fn current_datetime() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Get the home directory
pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Get the current working directory
pub fn get_current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Get directory name from path
pub fn get_dir_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from(""))
}

/// Get the current user name
pub fn get_username() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| String::from("user"))
}

/// Get the current shell
pub fn get_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| String::from("unknown"))
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    use std::process::Command;

    // Try to execute the command with --version flag
    // which usually exists for most commands and doesn't do anything harmful
    #[cfg(target_os = "windows")]
    let result = Command::new("where").arg(command).output();

    #[cfg(not(target_os = "windows"))]
    let result = Command::new("which").arg(command).output();

    result
        .map(|output| output.status.success())
        .unwrap_or(false)
}
