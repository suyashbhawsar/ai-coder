use crate::ai::{AIClient, AIClientFactory, AIError, AIResponse, ModelCosts};
use crate::config;
use crate::handlers::HandlerResult;
use regex::Regex;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AIHandler {
    client: Arc<Mutex<Box<dyn AIClient>>>,
}

impl Default for AIHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AIHandler {
    pub fn new() -> Self {
        // Create client based on configuration
        let client = match AIClientFactory::create_client() {
            Ok(client) => client,
            Err(e) => {
                // Log the error and fall back to a default client
                eprintln!("Failed to create AI client from config: {}", e);
                Box::new(crate::ai::OllamaClient::new("qwen2.5-coder".to_string()))
            }
        };

        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    /// Update the client based on new configuration
    pub fn update_client(&self) -> Result<(), AIError> {
        match AIClientFactory::create_client() {
            Ok(new_client) => {
                match self.client.try_lock() {
                    Ok(mut client) => {
                        *client = new_client;
                        Ok(())
                    }
                    Err(_) => {
                        // If we can't get a lock immediately, don't block
                        eprintln!("Warning: Client is currently in use, will update on next use");
                        Ok(())
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to create new client: {}", e);
                // Return success anyway but log the error
                // This prevents the application from crashing
                Ok(())
            }
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<AIResponse, AIError> {
        // First, check if Ollama is running
        self.check_service_availability().await?;

        // If we get here, service is available
        let client = self.client.lock().await;
        let response = client.generate(prompt, None).await?;

        // Process the response for bash blocks
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

    // Helper method to check if the AI service is available
    async fn check_service_availability(&self) -> Result<(), AIError> {
        use crate::ai::Provider;
        use crate::config;
        use reqwest::Client;
        use std::time::Duration;

        // Get current provider from config
        let app_config = config::get_config();
        let provider = app_config.ai.active_provider;

        // Create a client with a short timeout for just checking availability
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| AIError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        match provider {
            Provider::Ollama => {
                // Try to connect to Ollama health endpoint
                let endpoint = app_config.ai.ollama.endpoint.clone();
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
                if app_config.ai.openai.api_key.is_empty() {
                    return Err(AIError::Authentication(
                        "OpenAI API key is not set. Please update your configuration.".to_string(),
                    ));
                }
                Ok(())
            }
            Provider::Anthropic => {
                // For Anthropic we just check if the API key is set
                if app_config.ai.anthropic.api_key.is_empty() {
                    return Err(AIError::Authentication(
                        "Anthropic API key is not set. Please update your configuration."
                            .to_string(),
                    ));
                }
                Ok(())
            }
            Provider::LMStudio => {
                // Check if LM Studio is running
                let endpoint = app_config.ai.lmstudio.endpoint.clone();
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

    pub async fn list_models(&self) -> Result<Vec<String>, AIError> {
        // Check if the service is available
        self.check_service_availability().await?;

        // Get current provider from config
        let app_config = config::get_config();
        let provider = app_config.ai.active_provider;

        // Use the factory to get models for the current provider
        crate::ai::AIClientFactory::get_available_models(provider).await
    }

    pub async fn get_model_costs(&self, model: &str) -> ModelCosts {
        let client = self.client.lock().await;
        client.get_model_costs(model)
    }

    /// Process LLM output to extract and execute bash code blocks
    pub async fn process_llm_output(&self, output: &str) -> HandlerResult<String> {
        // Regular expression to match bash code blocks with flexible whitespace
        let bash_block_re = Regex::new(r"```bash\n(.*?)\n```").unwrap();

        // Store the original text with proper line breaks
        let mut result = String::new();
        let mut last_end = 0;

        // Find all non-overlapping bash code blocks
        for cap in bash_block_re.captures_iter(output) {
            let full_match = cap.get(0).unwrap();
            let command = cap.get(1).unwrap();

            // Add text before this match
            result.push_str(&output[last_end..full_match.start()]);

            // Add the original bash block
            result.push_str("```bash\n");
            result.push_str(command.as_str().trim());
            result.push_str("\n```\n");

            // Execute the command and add its output right after the code block
            match crate::handlers::bash::handle_bash_command(command.as_str().trim()) {
                Ok(cmd_output) => {
                    result.push_str(&cmd_output);
                }
                Err(e) => {
                    result.push_str(&format!("[⏱️ 0.00s | ✗ | 📊 1]\n⚠️ Error: {}\n", e));
                }
            }

            last_end = full_match.end();
        }

        // Add any remaining text after the last match
        if last_end < output.len() {
            result.push_str(&output[last_end..]);
        }

        Ok(result)
    }
}
