//! AI Client Factory
//!
//! This module provides factory methods for creating AI clients based on configuration

use crate::ai::{AIClient, AIError, OllamaClient, Provider};
use crate::config;

/// Factory for creating AI clients
pub struct AIClientFactory;

impl AIClientFactory {
    /// Create an AI client based on the current configuration
    pub fn create_client() -> Result<Box<dyn AIClient>, AIError> {
        let config = config::get_config();
        Self::create_client_from_config(&config.ai)
    }

    /// Create an AI client from explicit configuration
    pub fn create_client_from_config(ai_config: &config::AIConfig) -> Result<Box<dyn AIClient>, AIError> {
        match ai_config.active_provider {
            Provider::Ollama => {
                let model_config = ai_config.get_active_model_config();
                let endpoint = ai_config.get_active_endpoint();
                Ok(Box::new(OllamaClient::with_base_url(
                    endpoint,
                    model_config.name,
                )))
            }
            Provider::OpenAI => {
                // We'll implement this later
                Err(AIError::ConfigError(
                    "OpenAI support is not implemented yet".to_string(),
                ))
            }
            Provider::Anthropic => {
                // We'll implement this later
                Err(AIError::ConfigError(
                    "Anthropic support is not implemented yet".to_string(),
                ))
            }
            Provider::LMStudio => {
                // We'll implement this later
                Err(AIError::ConfigError(
                    "LM Studio support is not implemented yet".to_string(),
                ))
            }
        }
    }

    /// Get the names of all available models for the current provider
    pub async fn get_available_models(provider: Provider) -> Result<Vec<String>, AIError> {
        let config = config::get_config();
        match provider {
            Provider::Ollama => {
                let client = OllamaClient::with_base_url(
                    config.ai.ollama.endpoint.clone(),
                    "".to_string(), // Model name doesn't matter for listing
                );
                client.models().await
            }
            // For other providers, we'll return their configured models
            Provider::OpenAI => {
                Ok(config.ai.openai.models.iter().map(|m| m.name.clone()).collect())
            }
            Provider::Anthropic => {
                Ok(config.ai.anthropic.models.iter().map(|m| m.name.clone()).collect())
            }
            Provider::LMStudio => {
                Ok(config.ai.lmstudio.models.iter().map(|m| m.name.clone()).collect())
            }
        }
    }
}