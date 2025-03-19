//! AI handler module
//!
//! This module is responsible for handling AI-related commands and interactions.

use crate::ai::{AIClient, AIError, AIResponse, OllamaClient, ModelCosts};
use crate::config::get_config;
use crate::handlers::HandlerResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use std::time::Duration;

/// Handles AI interactions with different backends
pub struct AIHandler {
    client: Arc<Mutex<Box<dyn AIClient>>>,
}

impl Default for AIHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AIHandler {
    /// Create a new AI handler with configuration
    pub fn new() -> Self {
        let config = get_config();
        let client: Box<dyn AIClient> = match config.ai.provider.as_str() {
            "ollama" => Box::new(OllamaClient::new(config.ai.model.clone())),
            // Add more provider types as needed
            _ => Box::new(OllamaClient::new("qwen2.5-coder".to_string())),
        };
        
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    /// Generate a response for the given prompt
    pub async fn generate(&self, prompt: &str) -> Result<AIResponse, AIError> {
        // First, check if the AI service is running
        self.check_service_availability().await?;
        
        // Get the current configuration
        let config = get_config();
        
        // If there's a system prompt set, use it
        let context = config.ai.system_prompt.as_deref();
        
        // Generate the response
        let client = self.client.lock().await;
        client.generate(prompt, context).await
    }
    
    /// Check if the AI service is available
    async fn check_service_availability(&self) -> Result<(), AIError> {
        let config = get_config();
        
        // Create a client with a short timeout for checking availability
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| AIError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;
            
        // Get the endpoint from config
        let endpoint = match config.ai.provider.as_str() {
            "ollama" => format!("{}/api/tags", config.ai.endpoint),
            _ => format!("{}/api/tags", config.ai.endpoint),
        };
            
        // Try to connect
        match client.get(&endpoint).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(AIError::NetworkError(format!(
                "{} service not available: {}. Please ensure it's running.", 
                config.ai.provider, e
            )))
        }
    }

    /// List available AI models
    pub async fn list_models(&self) -> Result<Vec<String>, AIError> {
        // Check if the service is available
        self.check_service_availability().await?;
        
        let client = self.client.lock().await;
        client.models().await
    }
    
    /// Get cost information for a specific model
    pub async fn get_model_costs(&self, model: &str) -> ModelCosts {
        let client = self.client.lock().await;
        client.get_model_costs(model)
    }
    
    /// Update the AI provider configuration
    pub async fn update_config(&self, provider: &str, model: &str) -> HandlerResult<String> {
        use crate::config::{update_field, AppConfig};
        
        // Validate provider
        match provider {
            "ollama" => {
                // Check if the model exists for this provider
                if let Ok(models) = self.list_models().await {
                    if !models.contains(&model.to_string()) {
                        return Ok(format!("⚠️ Model '{}' not found. Available models: {}", 
                            model, models.join(", ")));
                    }
                }
            },
            _ => return Ok(format!("⚠️ Unsupported provider: {}. Currently only 'ollama' is supported.", provider)),
        }
        
        // Update the configuration
        update_field(|config: &mut AppConfig| {
            config.ai.provider = provider.to_string();
            config.ai.model = model.to_string();
        }).map_err(|e| AIError::ConfigError(format!("Failed to update config: {}", e)))?;
        
        // Return success message
        Ok(format!("✅ AI provider updated to {} with model {}", provider, model))
    }
}