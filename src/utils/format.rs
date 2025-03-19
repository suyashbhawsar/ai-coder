//! Text formatting utilities
//!
//! This module provides functions for formatting text and values

/// Convert bytes to human-readable size
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

/// Truncate a string to max_length, adding ellipsis if truncated
pub fn truncate_string(input: &str, max_length: usize) -> String {
    if input.len() <= max_length {
        input.to_string()
    } else {
        let mut truncated = input.chars().take(max_length - 3).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

/// Format a duration in seconds to a human-readable string
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format a number with commas as thousands separators
pub fn format_number(num: usize) -> String {
    let mut result = String::new();
    let num_str = num.to_string();
    let len = num_str.len();
    
    for (i, c) in num_str.chars().enumerate() {
        result.push(c);
        if (len - i - 1) % 3 == 0 && i < len - 1 {
            result.push(',');
        }
    }
    
    result
}

/// Format a float with specified precision
pub fn format_float(num: f64, precision: usize) -> String {
    format!("{:.1$}", num, precision)
}

/// Format money value
pub fn format_money(amount: f64) -> String {
    format!("${:.4}", amount)
}

/// Count tokens in a string (approximation)
pub fn count_tokens(text: &str) -> usize {
    // Split by whitespace and punctuation
    let tokens: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|s| !s.is_empty())
        .collect();
    
    // Apply a multiplier for better estimation
    (tokens.len() as f64 * 1.3).round() as usize
}