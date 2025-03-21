//! UI theme handling
//!
//! Provides color theme functionality for the terminal UI

use crate::config::ThemeConfig;
use ratatui::style::Color;

/// Theme structure for UI colors
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary UI color
    pub primary: Color,
    /// Secondary UI color
    pub secondary: Color,
    /// Accent color for highlights
    pub accent: Color,
    /// Background color
    pub background: Color,
    /// Foreground (text) color
    pub foreground: Color,
}

impl Theme {
    /// Create a new theme from the given theme config
    pub fn new(config: &ThemeConfig) -> Self {
        Self {
            primary: parse_hex_color(&config.primary),
            secondary: parse_hex_color(&config.secondary),
            accent: parse_hex_color(&config.accent),
            background: parse_hex_color(&config.background),
            foreground: parse_hex_color(&config.foreground),
        }
    }
}

impl Default for Theme {
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

/// Convert hex color string to ratatui Color
pub fn parse_hex_color(hex: &str) -> Color {
    if hex == "default" {
        return Color::Reset;
    }

    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Reset;
    }

    if let (Ok(r), Ok(g), Ok(b)) = (
        u8::from_str_radix(&hex[0..2], 16),
        u8::from_str_radix(&hex[2..4], 16),
        u8::from_str_radix(&hex[4..6], 16),
    ) {
        Color::Rgb(r, g, b)
    } else {
        Color::Reset
    }
}
