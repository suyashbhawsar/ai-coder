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
use regex::Regex;

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
        let response = client.generate(prompt, context).await?;

        println!("Raw LLM response:\n{}", response.content);

        // Process the response for bash code blocks
        let processed_content = self.process_llm_output(&response.content).await
            .map_err(|e| AIError::InvalidResponse(format!("Failed to process bash blocks: {}", e)))?;

        println!("Processed content:\n{}", processed_content);

        Ok(AIResponse {
            content: processed_content,
            ..response
        })
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
                },
                Err(e) => {
                    result.push_str(&format!("⚠️ Error executing command: {}\n", e));
                }
            }
            result.push_str("\n");
        }

        Ok(result)
    }
}