//! Configuration module for the AI Coder Interface
//!
//! This module handles application configuration including:
//! - Loading/saving configuration from files
//! - Default settings
//! - User preferences
//! - Theme settings

use std::path::PathBuf;
use std::fs;
use std::io;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Theme configuration for the application UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Primary theme color as hex string (e.g., "#4B9CD3")
    pub primary: String,
    /// Secondary theme color as hex string
    pub secondary: String,
    /// Accent color for highlighting
    pub accent: String,
    /// Background color (or "default" for terminal default)
    pub background: String,
    /// Foreground/text color (or "default" for terminal default)
    pub foreground: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary: "#0087AF".to_string(),    // Blue
            secondary: "#00AF87".to_string(),  // Teal
            accent: "#AF8700".to_string(),     // Gold
            background: "default".to_string(), // Terminal default
            foreground: "default".to_string(), // Terminal default
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name
    pub name: String,
    /// Temperature for generation (0.0-1.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// System prompt to use
    pub system_prompt: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "qwen2.5-coder".to_string(),
            temperature: 0.1,
            max_tokens: 2048,
            system_prompt: None,
        }
    }
}

/// Ollama provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// Available models
    pub models: Vec<ModelConfig>,
    /// Currently selected model (index into models)
    pub current_model_index: usize,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            models: vec![
                ModelConfig::default(),
                ModelConfig {
                    name: "codellama".to_string(),
                    temperature: 0.2,
                    max_tokens: 4096,
                    system_prompt: None,
                },
            ],
            current_model_index: 0,
        }
    }
}

/// OpenAI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// API key
    pub api_key: String,
    /// Available models
    pub models: Vec<ModelConfig>,
    /// Currently selected model (index into models)
    pub current_model_index: usize,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: "".to_string(),
            models: vec![
                ModelConfig {
                    name: "gpt-4o".to_string(),
                    temperature: 0.1,
                    max_tokens: 4096,
                    system_prompt: None,
                },
                ModelConfig {
                    name: "gpt-3.5-turbo".to_string(),
                    temperature: 0.2,
                    max_tokens: 2048,
                    system_prompt: None,
                },
            ],
            current_model_index: 0,
        }
    }
}

/// Anthropic provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// API key
    pub api_key: String,
    /// Available models
    pub models: Vec<ModelConfig>,
    /// Currently selected model (index into models)
    pub current_model_index: usize,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.anthropic.com".to_string(),
            api_key: "".to_string(),
            models: vec![
                ModelConfig {
                    name: "claude-3-opus-20240229".to_string(),
                    temperature: 0.1,
                    max_tokens: 4096,
                    system_prompt: None,
                },
                ModelConfig {
                    name: "claude-3-sonnet-20240229".to_string(),
                    temperature: 0.2,
                    max_tokens: 4096,
                    system_prompt: None,
                },
            ],
            current_model_index: 0,
        }
    }
}

/// LM Studio provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// Available models
    pub models: Vec<ModelConfig>,
    /// Currently selected model (index into models)
    pub current_model_index: usize,
}

impl Default for LMStudioConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:1234/v1".to_string(),
            models: vec![
                ModelConfig {
                    name: "local-model".to_string(),
                    temperature: 0.2,
                    max_tokens: 2048,
                    system_prompt: None,
                },
            ],
            current_model_index: 0,
        }
    }
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Currently active provider
    pub active_provider: crate::ai::types::Provider,
    /// Ollama configuration
    pub ollama: OllamaConfig,
    /// OpenAI configuration
    pub openai: OpenAIConfig,
    /// Anthropic configuration
    pub anthropic: AnthropicConfig,
    /// LM Studio configuration
    pub lmstudio: LMStudioConfig,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            active_provider: crate::ai::types::Provider::Ollama,
            ollama: OllamaConfig::default(),
            openai: OpenAIConfig::default(),
            anthropic: AnthropicConfig::default(),
            lmstudio: LMStudioConfig::default(),
        }
    }
}

impl AIConfig {
    /// Get the currently active model configuration
    pub fn get_active_model_config(&self) -> ModelConfig {
        match self.active_provider {
            crate::ai::types::Provider::Ollama => {
                let idx = self.ollama.current_model_index.min(self.ollama.models.len().saturating_sub(1));
                self.ollama.models[idx].clone()
            },
            crate::ai::types::Provider::OpenAI => {
                let idx = self.openai.current_model_index.min(self.openai.models.len().saturating_sub(1));
                self.openai.models[idx].clone()
            },
            crate::ai::types::Provider::Anthropic => {
                let idx = self.anthropic.current_model_index.min(self.anthropic.models.len().saturating_sub(1));
                self.anthropic.models[idx].clone()
            },
            crate::ai::types::Provider::LMStudio => {
                let idx = self.lmstudio.current_model_index.min(self.lmstudio.models.len().saturating_sub(1));
                self.lmstudio.models[idx].clone()
            },
        }
    }
    
    /// Get the endpoint for the currently active provider
    pub fn get_active_endpoint(&self) -> String {
        match self.active_provider {
            crate::ai::types::Provider::Ollama => self.ollama.endpoint.clone(),
            crate::ai::types::Provider::OpenAI => self.openai.endpoint.clone(),
            crate::ai::types::Provider::Anthropic => self.anthropic.endpoint.clone(),
            crate::ai::types::Provider::LMStudio => self.lmstudio.endpoint.clone(),
        }
    }
    
    /// Get the API key for the currently active provider (if applicable)
    pub fn get_active_api_key(&self) -> Option<String> {
        match self.active_provider {
            crate::ai::types::Provider::Ollama => None,
            crate::ai::types::Provider::OpenAI => Some(self.openai.api_key.clone()),
            crate::ai::types::Provider::Anthropic => Some(self.anthropic.api_key.clone()),
            crate::ai::types::Provider::LMStudio => None,
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Theme settings
    pub theme: ThemeConfig,
    /// AI provider settings
    pub ai: AIConfig,
    /// History size
    pub history_size: usize,
    /// Enable mouse support
    pub mouse_enabled: bool,
    /// Enable logging
    pub logging_enabled: bool,
    /// Log file path (relative to config directory)
    pub log_file: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            ai: AIConfig::default(),
            history_size: 100,
            mouse_enabled: true,
            logging_enabled: false,
            log_file: Some("ai-coder.log".to_string()),
        }
    }
}

// Global configuration instance
static CONFIG: Lazy<Mutex<AppConfig>> = Lazy::new(|| {
    let config = load_config().unwrap_or_default();
    Mutex::new(config)
});

/// Get a reference to the application configuration
pub fn get_config() -> AppConfig {
    CONFIG.lock().unwrap().clone()
}

/// Update the application configuration
pub fn update_config(config: AppConfig) -> Result<(), io::Error> {
    let mut current = CONFIG.lock().unwrap();
    *current = config.clone();
    save_config(&config)
}

/// Update a specific field in the configuration
pub fn update_field<F>(updater: F) -> Result<(), io::Error>
where
    F: FnOnce(&mut AppConfig),
{
    let mut config = CONFIG.lock().unwrap();
    updater(&mut config);
    save_config(&config)
}

/// Get the config directory path
pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".ai-coder")
}

/// Get the config file path
pub fn get_config_file() -> PathBuf {
    get_config_dir().join("config.yaml")
}

/// Load configuration from file
pub fn load_config() -> Result<AppConfig, io::Error> {
    let config_file = get_config_file();
    
    if !config_file.exists() {
        return Ok(AppConfig::default());
    }
    
    let config_str = fs::read_to_string(config_file)?;
    serde_yaml::from_str(&config_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Save configuration to file
pub fn save_config(config: &AppConfig) -> Result<(), io::Error> {
    let config_dir = get_config_dir();
    let config_file = get_config_file();
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    
    let config_str = serde_yaml::to_string(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    fs::write(config_file, config_str)
}

/// Initialize configuration on application start
pub fn init_config() -> Result<(), io::Error> {
    let config_dir = get_config_dir();
    let config_file = get_config_file();
    
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    
    if !config_file.exists() {
        let default_config = AppConfig::default();
        save_config(&default_config)?;
    }
    
    // Load config into memory
    let loaded_config = load_config()?;
    *CONFIG.lock().unwrap() = loaded_config;
    
    Ok(())
}
