//! AI handler module
//!
//! This module is responsible for handling AI-related commands and interactions.

use crate::ai::{
    AIClient, AIClientFactory, AIError, AIResponse, ModelCosts, OllamaClient, Provider,
};
use crate::config::get_config;
use crate::handlers::HandlerResult;
use regex::Regex;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

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
        // Create client based on configuration
        let client = match AIClientFactory::create_client() {
            Ok(client) => client,
            Err(e) => {
                // Log the error and fall back to a default client
                eprintln!("Failed to create AI client from config: {}", e);
                Box::new(OllamaClient::new("qwen2.5-coder".to_string()))
            }
        };

        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    /// Update the client based on new configuration
    pub fn update_client(&self) -> Result<(), AIError> {
        let new_client = AIClientFactory::create_client()?;
        let mut client = self.client.blocking_lock();
        *client = new_client;
        Ok(())
    }

    /// Generate a response for the given prompt
    pub async fn generate(&self, prompt: &str) -> Result<AIResponse, AIError> {
        // First, check if the AI service is running
        self.check_service_availability().await?;

        // Get the current configuration
        let config = get_config();
        let model_config = config.ai.get_active_model_config();

        // If there's a system prompt set, use it
        let context = model_config.system_prompt.as_deref();

        // Generate the response
        let client = self.client.lock().await;
        let response = client.generate(prompt, context).await?;

        // Process the response for bash code blocks
        let processed_content = self
            .process_llm_output(&response.content)
            .await
            .map_err(|e| {
                AIError::InvalidResponse(format!("Failed to process bash blocks: {}", e))
            })?;

        Ok(AIResponse {
            content: processed_content,
            ..response
        })
    }

    /// Check if the AI service is available
    async fn check_service_availability(&self) -> Result<(), AIError> {
        let config = get_config();
        let provider = config.ai.active_provider;

        // Create a client with a short timeout for checking availability
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| AIError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        match provider {
            Provider::Ollama => {
                // Try to connect to Ollama health endpoint
                let endpoint = config.ai.ollama.endpoint.clone();
                let health_url = format!("{}/api/tags", endpoint);
                match client.get(&health_url).send().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(AIError::NetworkError(format!(
                        "Ollama not available (is it running?): {}. Start Ollama with 'ollama serve' command.",
                        e
                    ))),
                }
            }
            Provider::OpenAI => {
                // For OpenAI we just check if the API key is set
                if config.ai.openai.api_key.is_empty() {
                    return Err(AIError::Authentication(
                        "OpenAI API key is not set. Please update your configuration.".to_string(),
                    ));
                }
                Ok(())
            }
            Provider::Anthropic => {
                // For Anthropic we just check if the API key is set
                if config.ai.anthropic.api_key.is_empty() {
                    return Err(AIError::Authentication(
                        "Anthropic API key is not set. Please update your configuration."
                            .to_string(),
                    ));
                }
                Ok(())
            }
            Provider::LMStudio => {
                // Check if LM Studio is running
                let endpoint = config.ai.lmstudio.endpoint.clone();
                let health_url = format!("{}/models", endpoint);
                match client.get(&health_url).send().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(AIError::NetworkError(format!(
                        "LM Studio not available (is it running?): {}. Start LM Studio and ensure the API server is enabled.",
                        e
                    ))),
                }
            }
        }
    }

    /// List available AI models
    pub async fn list_models(&self) -> Result<Vec<String>, AIError> {
        // Check if the service is available
        self.check_service_availability().await?;

        // Get current provider from config
        let config = get_config();
        let provider = config.ai.active_provider;

        // Use the factory to get models for the current provider
        AIClientFactory::get_available_models(provider).await
    }

    /// Get cost information for a specific model
    pub async fn get_model_costs(&self, model: &str) -> ModelCosts {
        let client = self.client.lock().await;
        client.get_model_costs(model)
    }

    /// Update the AI provider configuration
    pub async fn update_config(&self, provider_str: &str, model: &str) -> HandlerResult<String> {
        use crate::config::{AppConfig, update_field};
        use std::str::FromStr;

        // Parse the provider
        let provider = match Provider::from_str(provider_str) {
            Ok(p) => p,
            Err(_) => {
                return Ok(format!(
                    "⚠️ Unsupported provider: {}. Valid providers are: ollama, openai, anthropic, lmstudio",
                    provider_str
                ));
            }
        };

        // Validate model exists for the provider
        if let Ok(models) = AIClientFactory::get_available_models(provider).await {
            if !models.is_empty() && !models.contains(&model.to_string()) {
                return Ok(format!(
                    "⚠️ Model '{}' not found for provider {}. Available models: {}",
                    model,
                    provider,
                    models.join(", ")
                ));
            }
        }

        // Update the configuration
        update_field(|config: &mut AppConfig| {
            // Set the active provider
            config.ai.active_provider = provider;

            // Update the model for this provider
            match provider {
                Provider::Ollama => {
                    // Check if model exists in the list
                    let mut found = false;
                    for (i, m) in config.ai.ollama.models.iter().enumerate() {
                        if m.name == model {
                            config.ai.ollama.current_model_index = i;
                            found = true;
                            break;
                        }
                    }

                    // If model not found, add it
                    if !found {
                        config.ai.ollama.models.push(crate::config::ModelConfig {
                            name: model.to_string(),
                            ..Default::default()
                        });
                        config.ai.ollama.current_model_index = config.ai.ollama.models.len() - 1;
                    }
                }
                Provider::OpenAI => {
                    // Similar logic for OpenAI
                    let mut found = false;
                    for (i, m) in config.ai.openai.models.iter().enumerate() {
                        if m.name == model {
                            config.ai.openai.current_model_index = i;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        config.ai.openai.models.push(crate::config::ModelConfig {
                            name: model.to_string(),
                            ..Default::default()
                        });
                        config.ai.openai.current_model_index = config.ai.openai.models.len() - 1;
                    }
                }
                Provider::Anthropic => {
                    // Similar logic for Anthropic
                    let mut found = false;
                    for (i, m) in config.ai.anthropic.models.iter().enumerate() {
                        if m.name == model {
                            config.ai.anthropic.current_model_index = i;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        config.ai.anthropic.models.push(crate::config::ModelConfig {
                            name: model.to_string(),
                            ..Default::default()
                        });
                        config.ai.anthropic.current_model_index =
                            config.ai.anthropic.models.len() - 1;
                    }
                }
                Provider::LMStudio => {
                    // Similar logic for LM Studio
                    let mut found = false;
                    for (i, m) in config.ai.lmstudio.models.iter().enumerate() {
                        if m.name == model {
                            config.ai.lmstudio.current_model_index = i;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        config.ai.lmstudio.models.push(crate::config::ModelConfig {
                            name: model.to_string(),
                            ..Default::default()
                        });
                        config.ai.lmstudio.current_model_index =
                            config.ai.lmstudio.models.len() - 1;
                    }
                }
            }
        })
        .map_err(|e| AIError::ConfigError(format!("Failed to update config: {}", e)))?;

        // Update the client
        self.update_client()?;

        // Return success message
        Ok(format!(
            "✅ AI provider updated to {} with model {}",
            provider, model
        ))
    }

    /// Process LLM output to extract and execute bash code blocks
    pub async fn process_llm_output(&self, output: &str) -> HandlerResult<String> {
        // Regular expression to match bash code blocks with flexible whitespace
        let bash_block_re = Regex::new(r"```bash\n([\s\S]*?)\n```").unwrap();

        // Debug logging
        println!("Processing LLM output:\n{}", output);

        // Find all bash code blocks
        let bash_blocks: Vec<&str> = bash_block_re
            .captures_iter(output)
            .filter_map(|cap| {
                let matched = cap.get(1);
                println!("Found match: {:?}", matched.map(|m| m.as_str()));
                matched
            })
            .map(|m| m.as_str().trim())
            .filter(|s| !s.is_empty()) // Skip empty blocks
            .collect();

        println!("Found {} bash blocks", bash_blocks.len());

        if bash_blocks.is_empty() {
            // No bash blocks found, return original text
            println!("No bash blocks found, returning original text");
            return Ok(output.to_string());
        }

        // Process each bash block
        let mut result = String::new();
        result.push_str("Found and executed bash code blocks:\n\n");

        for (i, block) in bash_blocks.iter().enumerate() {
            println!("Processing block {}: {}", i + 1, block);
            result.push_str(&format!("Block {}:\n```bash\n{}\n```\n", i + 1, block));
            result.push_str("Output:\n");

            // Execute the bash block and capture output
            match crate::handlers::bash::handle_bash_command(block) {
                Ok(output) => {
                    result.push_str(&output);
                }
                Err(e) => {
                    result.push_str(&format!("⚠️ Error executing command: {}\n", e));
                }
            }
            result.push_str("\n");
        }

        Ok(result)
    }
}
