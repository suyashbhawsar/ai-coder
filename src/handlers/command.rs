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
    /// Handle list commands to show available resources
    fn handle_list_command(args: &[&str]) -> HandlerResult<String> {
        if args.is_empty() {
            return Ok("📋 Available list commands:\n- /list providers\n- /list models\n- /list config\nUse /help list for more information.".to_string());
        }
        
        let subcommand = args[0].to_lowercase();
        
        match subcommand.as_str() {
            "providers" => {
                let config = get_config();
                let active_provider = config.ai.active_provider;
                
                let provider_list = format!(
                    "📋 Available providers:
                    * Ollama{} - Local models
                    * OpenAI{} - GPT models via API
                    * Anthropic{} - Claude models via API
                    * LMStudio{} - Local models via LM Studio

                    Use /config provider <name> to change the active provider.",
                    if active_provider == crate::ai::Provider::Ollama { " (active)" } else { "" },
                    if active_provider == crate::ai::Provider::OpenAI { " (active)" } else { "" },
                    if active_provider == crate::ai::Provider::Anthropic { " (active)" } else { "" },
                    if active_provider == crate::ai::Provider::LMStudio { " (active)" } else { "" }
                );
                
                Ok(provider_list)
            },
            "models" => {
                // Get current models for active provider
                let config = get_config();
                let provider = config.ai.active_provider;
                
                // Start building result string
                let mut result = format!("📋 Models for {}:\n", provider);
                
                // We'll use this for matching active models directly in each provider case
                
                // Add models based on provider
                match provider {
                    crate::ai::Provider::Ollama => {
                        // Get the current active model
                        let current_model = config.ai.get_active_model_config().name;
                        
                        // Use a safer approach to get models from the bash command
                        // This won't crash if the command fails
                        let models_output = match crate::handlers::bash::handle_bash_command("ollama list") {
                            Ok(output) => output,
                            Err(_) => "Error: Could not run 'ollama list'".to_string()
                        };
                        
                        // Parse the output to extract model names
                        if models_output.contains("NAME") || models_output.contains("name") {
                            result.push_str("🤖 Available Ollama models:\n");
                            
                            // Skip the header line and parse each line
                            let mut model_count = 0;
                            for line in models_output.lines().skip(1) {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if !parts.is_empty() {
                                    // The first part is the model name
                                    let model_name = parts[0];
                                    if !model_name.is_empty() {
                                        let is_active = model_name == current_model;
                                        let active_marker = if is_active { " (active)" } else { "" };
                                        result.push_str(&format!("* {}{}\n", model_name, active_marker));
                                        model_count += 1;
                                    }
                                }
                            }
                            
                            if model_count == 0 {
                                result.push_str("No models found. You can download models with 'ollama pull <model>'.\n");
                            }
                        } else {
                            // Fallback to configured models
                            result.push_str("🤖 Configured Ollama models (Ollama service may not be running):\n");
                            for (i, model) in config.ai.ollama.models.iter().enumerate() {
                                let active = if i == config.ai.ollama.current_model_index { " (active)" } else { "" };
                                result.push_str(&format!("* {}{}\n", model.name, active));
                            }
                        }
                        
                        // Add helpful instructions
                        result.push_str("\nTo download a model: !ollama pull <model>\n");
                        result.push_str("To use any model: /config model <model_name>\n");
                        result.push_str("For more details on available models: !ollama list\n");
                    },
                    crate::ai::Provider::OpenAI => {
                        for (i, model) in config.ai.openai.models.iter().enumerate() {
                            let active = if i == config.ai.openai.current_model_index { " (active)" } else { "" };
                            result.push_str(&format!("* {}{}\n", model.name, active));
                        }
                    },
                    crate::ai::Provider::Anthropic => {
                        for (i, model) in config.ai.anthropic.models.iter().enumerate() {
                            let active = if i == config.ai.anthropic.current_model_index { " (active)" } else { "" };
                            result.push_str(&format!("* {}{}\n", model.name, active));
                        }
                    },
                    crate::ai::Provider::LMStudio => {
                        for (i, model) in config.ai.lmstudio.models.iter().enumerate() {
                            let active = if i == config.ai.lmstudio.current_model_index { " (active)" } else { "" };
                            result.push_str(&format!("* {}{}\n", model.name, active));
                        }
                    },
                }
                
                result.push_str("\nUse /config model <name> to change the active model.");
                Ok(result)
            },
            "config" => {
                // Delegate to the config command with no arguments
                Self::handle_config(&[])
            },
            _ => Err(HandlerError::Parse(format!("Unknown list type: {}. Use 'providers', 'models', or 'config'", subcommand))),
        }
    }
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
            "list" => Self::handle_list_command(args),
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
                    - model - Set AI model (e.g. qwen2.5-coder, gpt-4o)
                    - provider - Set AI provider (ollama, openai, anthropic, lmstudio)
                    - temperature - Set temperature (0.0-1.0)
                    - endpoint - Set API endpoint URL
                    - api_key - Set API key (for OpenAI/Anthropic)
                    - system_prompt - Set system prompt"),
                    
                ("list", "📚 List Command Help:
                    List available resources
                    Subcommands:
                    - /list providers - Show available AI providers
                    - /list models - Show available models for current provider
                    - /list config - Show all current configuration
                    Examples:
                    - /list providers
                    - /list models"),

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
          /help [topic]   - Show help (optional: ai, bash, config, theme, system, list)
          /clear          - Clear terminal output
          /config         - View or set configuration
          /theme          - Customize UI colors
          /system         - Display system information
          /version        - Show version information
          /list           - List available providers, models, etc.
          /exit or /quit  - Exit application

        AI configuration:
          /config provider <name>  - Set AI provider (ollama, openai, anthropic, lmstudio)
          /config model <name>     - Set AI model for current provider
          /config endpoint <url>   - Set API endpoint URL
          /config api_key <key>    - Set API key (for OpenAI/Anthropic)
          /list providers          - Show available providers
          /list models             - Show available models for current provider

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
        let active_model = config.ai.get_active_model_config();

        format!(
            "System Information:
            OS: {}
            Version: {}
            Current Time: {}
            Working Directory: {}
            AI Provider: {}
            AI Model: {}
            API Endpoint: {}
            Temperature: {}
            Max Tokens: {}
            Config Path: {}",
            os_name,
            env!("CARGO_PKG_VERSION"),
            current_time,
            env::current_dir().unwrap_or_default().display(),
            config.ai.active_provider,
            active_model.name,
            config.ai.get_active_endpoint(),
            active_model.temperature,
            active_model.max_tokens,
            crate::config::get_config_file().display()
        )
    }

    /// Handle config commands
    fn handle_config(args: &[&str]) -> HandlerResult<String> {
        let config = get_config();
        let active_model = config.ai.get_active_model_config();

        if args.is_empty() {
            // Display current configuration
            let api_key_display = match config.ai.get_active_api_key() {
                Some(key) if !key.is_empty() => {
                    if key.len() <= 8 {
                        "***".to_string()
                    } else {
                        format!("{}***{}", &key[0..4], &key[key.len()-4..])
                    }
                },
                _ => "not set".to_string()
            };
            
            return Ok(format!(
                "📝 Current Configuration:
                AI Provider: {}
                Endpoint: {}
                API Key: {}
                Model: {}
                Temperature: {}
                Max Tokens: {}
                System Prompt: {}
                History Size: {}
                Mouse Enabled: {}
                Logging Enabled: {}

                Use /config [key] [value] to change settings.",
                config.ai.active_provider,
                config.ai.get_active_endpoint(),
                api_key_display,
                active_model.name,
                active_model.temperature,
                active_model.max_tokens,
                active_model.system_prompt.as_deref().unwrap_or("not set"),
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
                // Update model based on provider
                let provider = config.ai.active_provider;
                
                update_field(|c: &mut AppConfig| {
                    match provider {
                        crate::ai::Provider::Ollama => {
                            // Check if model exists in the list
                            let mut found = false;
                            for (i, model) in c.ai.ollama.models.iter().enumerate() {
                                if model.name.to_lowercase() == value.to_lowercase() {
                                    c.ai.ollama.current_model_index = i;
                                    found = true;
                                    break;
                                }
                            }
                            
                            // If not found, add it
                            if !found {
                                // Add the model with a lower temperature as requested
                                c.ai.ollama.models.push(crate::config::ModelConfig {
                                    name: value.to_string(),
                                    temperature: 0.1, // Lower temperature for more deterministic outputs
                                    system_prompt: Some("You are a helpful AI coding assistant.".to_string()),
                                    ..Default::default()
                                });
                                c.ai.ollama.current_model_index = c.ai.ollama.models.len() - 1;
                            }
                        },
                        crate::ai::Provider::OpenAI => {
                            // Check if model exists in the list (case insensitive)
                            let mut found = false;
                            for (i, model) in c.ai.openai.models.iter().enumerate() {
                                if model.name.to_lowercase() == value.to_lowercase() {
                                    c.ai.openai.current_model_index = i;
                                    found = true;
                                    break;
                                }
                            }
                            
                            // If not found, add it
                            if !found {
                                c.ai.openai.models.push(crate::config::ModelConfig {
                                    name: value.to_string(),
                                    temperature: 0.1,
                                    system_prompt: Some("You are a helpful AI coding assistant.".to_string()),
                                    ..Default::default()
                                });
                                c.ai.openai.current_model_index = c.ai.openai.models.len() - 1;
                            }
                        },
                        crate::ai::Provider::Anthropic => {
                            // Check if model exists in the list (case insensitive)
                            let mut found = false;
                            for (i, model) in c.ai.anthropic.models.iter().enumerate() {
                                if model.name.to_lowercase() == value.to_lowercase() {
                                    c.ai.anthropic.current_model_index = i;
                                    found = true;
                                    break;
                                }
                            }
                            
                            // If not found, add it
                            if !found {
                                c.ai.anthropic.models.push(crate::config::ModelConfig {
                                    name: value.to_string(),
                                    temperature: 0.1,
                                    system_prompt: Some("You are a helpful AI coding assistant.".to_string()),
                                    ..Default::default()
                                });
                                c.ai.anthropic.current_model_index = c.ai.anthropic.models.len() - 1;
                            }
                        },
                        crate::ai::Provider::LMStudio => {
                            // Check if model exists in the list (case insensitive)
                            let mut found = false;
                            for (i, model) in c.ai.lmstudio.models.iter().enumerate() {
                                if model.name.to_lowercase() == value.to_lowercase() {
                                    c.ai.lmstudio.current_model_index = i;
                                    found = true;
                                    break;
                                }
                            }
                            
                            // If not found, add it
                            if !found {
                                c.ai.lmstudio.models.push(crate::config::ModelConfig {
                                    name: value.to_string(),
                                    temperature: 0.1,
                                    system_prompt: Some("You are a helpful AI coding assistant.".to_string()),
                                    ..Default::default()
                                });
                                c.ai.lmstudio.current_model_index = c.ai.lmstudio.models.len() - 1;
                            }
                        },
                    }
                }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                
                // Also update the AI client
                let app = crate::app::App::new();
                app.ai_handler.update_client()
                    .map_err(|e| HandlerError::Other(format!("Failed to update AI client: {}", e)))?;
                
                Ok(format!("✅ Model set to: {}", value))
            },
            "provider" => {
                // Parse the provider
                let provider = match value.to_lowercase().as_str() {
                    "ollama" => crate::ai::Provider::Ollama,
                    "openai" => crate::ai::Provider::OpenAI,
                    "anthropic" => crate::ai::Provider::Anthropic,
                    "lmstudio" => crate::ai::Provider::LMStudio,
                    _ => return Err(HandlerError::Parse(format!(
                        "⚠️ Unknown provider: {}. Available: ollama, openai, anthropic, lmstudio", value
                    ))),
                };
                
                update_field(|c: &mut AppConfig| {
                    c.ai.active_provider = provider;
                }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                
                // Also update the AI client
                let app = crate::app::App::new();
                app.ai_handler.update_client()
                    .map_err(|e| HandlerError::Other(format!("Failed to update AI client: {}", e)))?;
                
                Ok(format!("✅ Provider set to: {}", provider))
            },
            "temperature" => {
                match value.parse::<f32>() {
                    Ok(temp) if (0.0..=1.0).contains(&temp) => {
                        // Update temperature for current model in current provider
                        update_field(|c: &mut AppConfig| {
                            match c.ai.active_provider {
                                crate::ai::Provider::Ollama => {
                                    let idx = c.ai.ollama.current_model_index;
                                    if idx < c.ai.ollama.models.len() {
                                        c.ai.ollama.models[idx].temperature = temp;
                                    }
                                },
                                crate::ai::Provider::OpenAI => {
                                    let idx = c.ai.openai.current_model_index;
                                    if idx < c.ai.openai.models.len() {
                                        c.ai.openai.models[idx].temperature = temp;
                                    }
                                },
                                crate::ai::Provider::Anthropic => {
                                    let idx = c.ai.anthropic.current_model_index;
                                    if idx < c.ai.anthropic.models.len() {
                                        c.ai.anthropic.models[idx].temperature = temp;
                                    }
                                },
                                crate::ai::Provider::LMStudio => {
                                    let idx = c.ai.lmstudio.current_model_index;
                                    if idx < c.ai.lmstudio.models.len() {
                                        c.ai.lmstudio.models[idx].temperature = temp;
                                    }
                                },
                            }
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        
                        Ok(format!("✅ Temperature set to: {}", temp))
                    },
                    _ => Err(HandlerError::Parse("⚠️ Temperature must be between 0.0 and 1.0".to_string()))
                }
            },
            "maxtokens" | "max_tokens" => {
                match value.parse::<usize>() {
                    Ok(tokens) if tokens > 0 => {
                        // Update max_tokens for current model in current provider
                        update_field(|c: &mut AppConfig| {
                            match c.ai.active_provider {
                                crate::ai::Provider::Ollama => {
                                    let idx = c.ai.ollama.current_model_index;
                                    if idx < c.ai.ollama.models.len() {
                                        c.ai.ollama.models[idx].max_tokens = tokens;
                                    }
                                },
                                crate::ai::Provider::OpenAI => {
                                    let idx = c.ai.openai.current_model_index;
                                    if idx < c.ai.openai.models.len() {
                                        c.ai.openai.models[idx].max_tokens = tokens;
                                    }
                                },
                                crate::ai::Provider::Anthropic => {
                                    let idx = c.ai.anthropic.current_model_index;
                                    if idx < c.ai.anthropic.models.len() {
                                        c.ai.anthropic.models[idx].max_tokens = tokens;
                                    }
                                },
                                crate::ai::Provider::LMStudio => {
                                    let idx = c.ai.lmstudio.current_model_index;
                                    if idx < c.ai.lmstudio.models.len() {
                                        c.ai.lmstudio.models[idx].max_tokens = tokens;
                                    }
                                },
                            }
                        }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                        
                        Ok(format!("✅ Max tokens set to: {}", tokens))
                    },
                    _ => Err(HandlerError::Parse("⚠️ Max tokens must be a positive number".to_string()))
                }
            },
            "endpoint" => {
                // Validate URL format
                if !value.starts_with("http://") && !value.starts_with("https://") {
                    return Err(HandlerError::Parse("⚠️ Endpoint URL must start with http:// or https://".to_string()));
                }
                
                // Update endpoint for current provider
                update_field(|c: &mut AppConfig| {
                    match c.ai.active_provider {
                        crate::ai::Provider::Ollama => {
                            c.ai.ollama.endpoint = value.to_string();
                        },
                        crate::ai::Provider::OpenAI => {
                            c.ai.openai.endpoint = value.to_string();
                        },
                        crate::ai::Provider::Anthropic => {
                            c.ai.anthropic.endpoint = value.to_string();
                        },
                        crate::ai::Provider::LMStudio => {
                            c.ai.lmstudio.endpoint = value.to_string();
                        },
                    }
                }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                
                // Update client with new endpoint
                let app = crate::app::App::new();
                app.ai_handler.update_client()
                    .map_err(|e| HandlerError::Other(format!("Failed to update AI client: {}", e)))?;
                
                Ok(format!("✅ Endpoint set to: {}", value))
            },
            "api_key" => {
                // Validate that provider requires API key
                match config.ai.active_provider {
                    crate::ai::Provider::Ollama | crate::ai::Provider::LMStudio => {
                        return Err(HandlerError::Parse(format!("⚠️ {} does not require an API key", config.ai.active_provider)));
                    },
                    _ => {}
                }
                
                // Update API key for current provider
                update_field(|c: &mut AppConfig| {
                    match c.ai.active_provider {
                        crate::ai::Provider::OpenAI => {
                            c.ai.openai.api_key = value.to_string();
                        },
                        crate::ai::Provider::Anthropic => {
                            c.ai.anthropic.api_key = value.to_string();
                        },
                        _ => {} // Already handled above
                    }
                }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                
                // Update client with new API key
                let app = crate::app::App::new();
                app.ai_handler.update_client()
                    .map_err(|e| HandlerError::Other(format!("Failed to update AI client: {}", e)))?;
                
                Ok("✅ API key updated".to_string())
            },
            "system_prompt" => {
                // Update system prompt for current model in current provider
                update_field(|c: &mut AppConfig| {
                    let prompt = if value.is_empty() { None } else { Some(value.to_string()) };
                    
                    match c.ai.active_provider {
                        crate::ai::Provider::Ollama => {
                            let idx = c.ai.ollama.current_model_index;
                            if idx < c.ai.ollama.models.len() {
                                c.ai.ollama.models[idx].system_prompt = prompt;
                            }
                        },
                        crate::ai::Provider::OpenAI => {
                            let idx = c.ai.openai.current_model_index;
                            if idx < c.ai.openai.models.len() {
                                c.ai.openai.models[idx].system_prompt = prompt;
                            }
                        },
                        crate::ai::Provider::Anthropic => {
                            let idx = c.ai.anthropic.current_model_index;
                            if idx < c.ai.anthropic.models.len() {
                                c.ai.anthropic.models[idx].system_prompt = prompt;
                            }
                        },
                        crate::ai::Provider::LMStudio => {
                            let idx = c.ai.lmstudio.current_model_index;
                            if idx < c.ai.lmstudio.models.len() {
                                c.ai.lmstudio.models[idx].system_prompt = prompt;
                            }
                        },
                    }
                }).map_err(|e| HandlerError::Other(format!("Failed to update config: {}", e)))?;
                
                if value.is_empty() {
                    Ok("✅ System prompt cleared".to_string())
                } else {
                    Ok("✅ System prompt updated".to_string())
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