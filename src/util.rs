use std::env;
use std::path::PathBuf;
use ratatui::style::Color;

pub struct Colors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub foreground: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            primary: Color::Rgb(0, 135, 175),    // Blue
            secondary: Color::Rgb(0, 175, 135),   // Teal
            accent: Color::Rgb(175, 135, 0),      // Gold
            background: Color::Reset,             // Terminal default
            foreground: Color::Reset,             // Terminal default
        }
    }
}

// Returns the config directory for the application
pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".llm-chainfuse")
}

// Get full path to config file
pub fn get_config_file() -> PathBuf {
    get_config_dir().join("config.yaml")
}

// Get current time as a formatted string
pub fn format_time() -> String {
    use chrono::Local;
    Local::now().format("%H:%M:%S").to_string()
}

// Convert bytes to human-readable size
pub fn human_readable_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

// Get current shell
pub fn get_current_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| String::from("unknown"))
}

// Truncate a string to max_length, adding ellipsis if truncated
pub fn truncate_string(input: &str, max_length: usize) -> String {
    if input.len() <= max_length {
        input.to_string()
    } else {
        let mut truncated = input.chars().take(max_length - 3).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}
