//! Application command handler
//!
//! This module handles the built-in application commands
//! like help, clear, config, etc.

use std::env;
use std::process;
use chrono::Local;
use crate::config::{get_config, update_field, AppConfig};
use crate::handlers::{HandlerResult, HandlerError};

/// Command handler for application commands
pub struct CommandHandler;

impl CommandHandler {
    /// Handle application commands
    pub fn handle_command(command: &str) -> HandlerResult<String> {
        // Split command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = if parts.is_empty() {
            return Err(HandlerError::Parse("Empty command".to_string()));
        } else {
            parts[0].to_lowercase()
        };

        let args = if parts.len() > 1 { &parts[1..] } else { &[] };

        // Command mapping
        match cmd.as_str() {
            "help" => Ok(Self::show_help(args)),
            "clear" => Ok("/clear".to_string()), // Special return value handled by app
            "exit" | "quit" => {
                process::exit(0);
            }
            "config" => Self::handle_config(args),
            "version" => Ok(Self::show_version()),
            "echo" => Ok(args.join(" ")),
            "system" => Ok(Self::show_system_info()),
            "theme" => Self::handle_theme(args),
            _ => Err(HandlerError::Parse(format!("Unknown command '{}'. Type '/help' for commands.", cmd))),
        }
    }

    /// Display help information
    fn show_help(args: &[&str]) -> String {
        if !args.is_empty() {
            // Show help for a specific command
            let specific_cmd = args[0].to_lowercase();
            let help_topics = [
                ("ai", "📚 AI Mode Help:
                    Just type your question or prompt directly without any prefix.
                    Examples:
                    - What is the capital of France?
                    - Write a Python function to calculate Fibonacci numbers
                    - Explain the difference between TCP and UDP"),

                ("bash", "📚 Bash Mode Help:
                    Prefix any bash command with ! to execute it directly.
                    Examples:
                    - !ls -la
                    - !cat file.txt
                    - !python script.py"),

                ("config", "📚 Config Command Help:
                    Configure settings using /config [key] [value]
                    Example keys:
                    - model - Set default model
                    - provider - Set AI provider (e.g. ollama)
                    - temperature - Set temperature (0.0-1.0)"),

                ("theme", "📚 Theme Command Help:
                    Customize UI colors using /theme [key] [value]
                    Keys:
                    - primary - Primary interface color
                    - secondary - Secondary interface color
                    - accent - Accent color for highlights
                    - background - Background color
                    - foreground - Text color
                    Values can be hex colors like #FF0000 or named colors"),

                ("system", "📚 System Command Help:
                    Use /system to display system information including:
                    - Operating system
                    - Version information
                    - Current working directory
                    - Runtime information"),
            ];

            for (topic, help_text) in help_topics {
                if specific_cmd == topic {
                    return help_text.to_string();
                }
            }

            return format!("⚠️ No help available for '{}'. Try '/help' for general help.", specific_cmd);
        }

        // General help
        "📚 AI Coder Interface Help:

        Mode Prefixes:
          - No prefix: AI mode - Ask questions or get creative responses
          - ! prefix: Execute bash commands (e.g., !ls)
          - / prefix: CLI commands (see below)

        Available commands:
          /help [topic]   - Show help (optional: ai, bash, config, theme, system)
          /clear          - Clear terminal output
          /config         - View or set configuration
          /theme          - Customize UI colors
          /system         - Display system information
          /version        - Show version information
          /exit or /quit  - Exit application

        Keyboard shortcuts:
          - Up/Down arrow: Navigate command history
          - Shift+Up/Down: Select text in output area
          - Ctrl+C: Copy selected text (when in selection mode) or exit
          - PageUp/Down: Scroll output
          - Esc: Cancel text selection or clear input".to_string()
    }

    /// Display version information
    fn show_version() -> String {
        format!(
            "AI Coder Interface v{}",
            env!("CARGO_PKG_VERSION")
        )
    }

    /// Display system information
    fn show_system_info() -> String {
        // Get basic system information
        let os_name = if cfg!(target_os = "windows") {
            "Windows"
        } else if cfg!(target_os = "macos") {
            "macOS"
        } else if cfg!(target_os = "linux") {
            "Linux"
        } else {
            "Unknown"
        };

        let config = get_config();
        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S ").to_string();

        format!(
            "System Information:
            OS: {}
            Version: {}
            Current Time: {}
            Working Directory: {}
            AI Provider: {}
            AI Model: {}
            Config Path: {}",
            os_name,
            env!("CARGO_PKG_VERSION"),
            current_time,
            env::current_dir().unwrap_or_default().display(),
            config.ai.provider,
            config.ai.model,
            crate::config::get_config_file().display()
        )
    }

    /// Handle config commands
    fn handle_config(args: &[&str]) -> HandlerResult<String> {
        let config = get_config();

        if args.is_empty() {
            // Display current configuration
            return Ok(format!(
                "📝 Current Configuration:
                AI Provider: {}
                Model: {}
                Temperature: {}
                Max Tokens: {}
                History Size: {}
                Mouse Enabled: {}
                Logging Enabled: {}

                Use /config [key] [value] to change settings.",
                config.ai.provider,
                config.ai.model,
                config.ai.temperature,
                config.ai.max_tokens,
                config.history_size,
                config.mouse_enabled,
                config.logging_enabled
            ));
        }

        let key = args[0].to_lowercase();
        let value = if args.len() > 1 { args[1] } else { "" };

        if value.is_empty() && key != "reset" {
            return Err(HandlerError::Parse(format!("Value required for key: {}", key)));
        }

        match key.as_str() {
            "model" => {
                update_field(|c: &mut AppConfig| {
                    c.ai.model = value.to_string();
                }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                Ok(format!("✅ Model set to: {}", value))
            },
            "provider" => {
                match value {
                    "ollama" => {
                        update_field(|c: &mut AppConfig| {
                            c.ai.provider = "ollama".to_string();
                            c.ai.endpoint = "http://localhost:11434".to_string();
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        Ok("✅ Provider set to: ollama".to_string())
                    },
                    _ => Err(HandlerError::Parse(format!(
                        "⚠️ Unknown provider: {}. Currently supported: ollama", value
                    )))
                }
            },
            "temperature" => {
                match value.parse::<f32>() {
                    Ok(temp) if (0.0..=1.0).contains(&temp) => {
                        update_field(|c: &mut AppConfig| {
                            c.ai.temperature = temp;
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        Ok(format!("✅ Temperature set to: {}", temp))
                    },
                    _ => Err(HandlerError::Parse("⚠️ Temperature must be between 0.0 and 1.0".to_string()))
                }
            },
            "maxtokens" | "max_tokens" => {
                match value.parse::<usize>() {
                    Ok(tokens) if tokens > 0 => {
                        update_field(|c: &mut AppConfig| {
                            c.ai.max_tokens = tokens;
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        Ok(format!("✅ Max tokens set to: {}", tokens))
                    },
                    _ => Err(HandlerError::Parse("⚠️ Max tokens must be a positive number".to_string()))
                }
            },
            "history" | "history_size" => {
                match value.parse::<usize>() {
                    Ok(size) if size > 0 => {
                        update_field(|c: &mut AppConfig| {
                            c.history_size = size;
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        Ok(format!("✅ History size set to: {}", size))
                    },
                    _ => Err(HandlerError::Parse("⚠️ History size must be a positive number".to_string()))
                }
            },
            "mouse" => {
                match value.to_lowercase().as_str() {
                    "true" | "yes" | "on" | "1" => {
                        update_field(|c: &mut AppConfig| {
                            c.mouse_enabled = true;
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        Ok("✅ Mouse support enabled".to_string())
                    },
                    "false" | "no" | "off" | "0" => {
                        update_field(|c: &mut AppConfig| {
                            c.mouse_enabled = false;
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        Ok("✅ Mouse support disabled".to_string())
                    },
                    _ => Err(HandlerError::Parse("⚠️ Value must be true/false, yes/no, on/off, or 1/0".to_string()))
                }
            },
            "reset" => {
                update_field(|c: &mut AppConfig| {
                    *c = AppConfig::default();
                }).map_err(|e| HandlerError::Other(format!("Failed to reset config: {}", e)))?;
                Ok("✅ Configuration reset to defaults".to_string())
            },
            _ => Err(HandlerError::Parse(format!("⚠️ Unknown configuration key: {}", key)))
        }
    }

    /// Handle theme customization
    fn handle_theme(args: &[&str]) -> HandlerResult<String> {
        let config = get_config();

        if args.is_empty() {
            // Display current theme
            return Ok(format!(
                "🎨 Current Theme:
                Primary: {}
                Secondary: {}
                Accent: {}
                Background: {}
                Foreground: {}

                Use /theme [key] [value] to change colors.",
                config.theme.primary,
                config.theme.secondary,
                config.theme.accent,
                config.theme.background,
                config.theme.foreground
            ));
        }

        let key = args[0].to_lowercase();
        let value = if args.len() > 1 { args[1] } else { "" };

        if value.is_empty() {
            return Err(HandlerError::Parse(format!("Color value required for: {}", key)));
        }

        // Validate hex color
        let hex_regex = Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();

        let color_value = if value == "default" {
            "default".to_string()
        } else if hex_regex.is_match(value) {
            value.to_string()
        } else if value.starts_with('#') {
            return Err(HandlerError::Parse("⚠️ Invalid hex color format. Use #RRGGBB".to_string()));
        } else {
            // Try to convert named color to hex
            match value.to_lowercase().as_str() {
                "red" => "#FF0000".to_string(),
                "green" => "#00FF00".to_string(),
                "blue" => "#0000FF".to_string(),
                "black" => "#000000".to_string(),
                "white" => "#FFFFFF".to_string(),
                "yellow" => "#FFFF00".to_string(),
                "cyan" => "#00FFFF".to_string(),
                "magenta" => "#FF00FF".to_string(),
                "gray" | "grey" => "#808080".to_string(),
                _ => return Err(HandlerError::Parse(format!(
                    "⚠️ Unknown color name: {}. Use hex format #RRGGBB", value
                ))),
            }
        };

        match key.as_str() {
            "primary" => {
                update_field(|c: &mut AppConfig| {
                    c.theme.primary = color_value.clone();
                }).map_err(|e| HandlerError::Other(format!("Failed to update theme: {}", e)))?;
                Ok(format!("✅ Primary color set to: {}", color_value))
            },
            "secondary" => {
                update_field(|c: &mut AppConfig| {
                    c.theme.secondary = color_value.clone();
                }).map_err(|e| HandlerError::Other(format!("Failed to update theme: {}", e)))?;
                Ok(format!("✅ Secondary color set to: {}", color_value))
            },
            "accent" => {
                update_field(|c: &mut AppConfig| {
                    c.theme.accent = color_value.clone();
                }).map_err(|e| HandlerError::Other(format!("Failed to update theme: {}", e)))?;
                Ok(format!("✅ Accent color set to: {}", color_value))
            },
            "background" => {
                update_field(|c: &mut AppConfig| {
                    c.theme.background = color_value.clone();
                }).map_err(|e| HandlerError::Other(format!("Failed to update theme: {}", e)))?;
                Ok(format!("✅ Background color set to: {}", color_value))
            },
            "foreground" => {
                update_field(|c: &mut AppConfig| {
                    c.theme.foreground = color_value.clone();
                }).map_err(|e| HandlerError::Other(format!("Failed to update theme: {}", e)))?;
                Ok(format!("✅ Foreground color set to: {}", color_value))
            },
            "reset" => {
                update_field(|c: &mut AppConfig| {
                    c.theme = crate::config::ThemeConfig::default();
                }).map_err(|e| HandlerError::Other(format!("Failed to reset theme: {}", e)))?;
                Ok("✅ Theme reset to defaults".to_string())
            },
            _ => Err(HandlerError::Parse(format!("⚠️ Unknown theme key: {}", key)))
        }
    }
}

// Add the regex crate in the scope
use regex::Regex;