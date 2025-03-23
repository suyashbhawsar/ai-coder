use crate::ai::{AIClient, AIClientFactory, AIError, AIResponse, ModelCosts};
use crate::config;
use crate::handlers::HandlerResult;
use regex::Regex;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

/// AIHandler handles all AI operations in a thread-safe manner
///
/// This struct provides methods for generating AI responses, managing models,
/// and handling concurrent requests. It uses Arc<Mutex> to allow sharing
/// between threads and implements proper error handling and timeout management.
///
/// The handler supports immediate cancellation via atomic abort flags and
/// can be safely cloned to use in background tasks.
#[derive(Clone)]
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

    pub async fn generate(
        &self,
        prompt: &str,
        abort_flag: Arc<AtomicBool>,
        global_abort: Option<Arc<AtomicBool>>,
    ) -> Result<AIResponse, AIError> {
        // First, check if Ollama is running
        self.check_service_availability().await?;

        // If we get here, service is available
        if abort_flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AIError::Cancelled("Operation aborted by user".to_string()));
        }

        // Get the client and generate
        let client = self.client.lock().await;

        // Set up a future for generation
        let generation_future = client.generate(prompt, None);

        // Set up a better abort check that uses both the local and global flags
        // and checks more frequently for better responsiveness
        let abort_flag_clone = abort_flag.clone();
        let global_abort_clone = global_abort.clone();
        let abort_check = async move {
            loop {
                // Check both abort flags using atomic operations for thread safety
                let local_aborted = abort_flag_clone.load(std::sync::atomic::Ordering::SeqCst);
                let global_aborted = global_abort_clone
                    .as_ref()
                    .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::SeqCst));

                if local_aborted || global_aborted {
                    // If locally not aborted but globally aborted, update the local flag for consistency
                    if !local_aborted && global_aborted {
                        abort_flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    return Err::<AIResponse, AIError>(AIError::Cancelled(
                        "Operation aborted by user".to_string(),
                    ));
                }

                // Check very frequently for better responsiveness (20 times per second)
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        };

        // Race the two futures
        let result = tokio::select! {
            result = generation_future => result,
            result = abort_check => result,
        };

        // Process the result
        match result {
            Ok(response) => {
                // Successfully generated response
                // Check if process_llm_output should be called based on the abort flag
                if abort_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    return Err(AIError::Cancelled(
                        "Operation aborted after generation completed".to_string(),
                    ));
                }

                // Process bash blocks with abort capability
                let processed_content = self
                    .process_llm_output(&response.content, abort_flag)
                    .await
                    .map_err(|e| {
                        AIError::InvalidResponse(format!("Failed to process bash blocks: {}", e))
                    })?;

                Ok(AIResponse {
                    content: processed_content,
                    ..response
                })
            }
            Err(e) => Err(e),
        }
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
    pub async fn process_llm_output(
        &self,
        output: &str,
        abort_flag: Arc<AtomicBool>,
    ) -> HandlerResult<String> {
        // Regular expression to match bash code blocks with flexible whitespace
        let bash_block_re = Regex::new(r"```bash\n([\s\S]*?)\n```").unwrap();

        // Check if there are any bash blocks to process
        let captures: Vec<_> = bash_block_re.captures_iter(output).collect();
        if captures.is_empty() {
            // No bash blocks found, return original content
            return Ok(output.to_string());
        }

        // Store the original text with proper line breaks
        let mut result = String::new();
        let mut last_end = 0;

        // Process each bash block
        for cap in captures {
            // Check if abort was requested using atomic operations
            if abort_flag.load(std::sync::atomic::Ordering::SeqCst) {
                // Add text until the current point and then terminate early
                result.push_str(&output[last_end..]);
                result.push_str("\n\n[Remaining bash commands aborted by user]\n");
                return Ok(result);
            }

            let full_match = cap.get(0).unwrap();
            let command = cap.get(1).unwrap();
            let cmd_str = command.as_str().trim();

            // Skip empty commands
            if cmd_str.is_empty() {
                continue;
            }

            // Add text before this match
            result.push_str(&output[last_end..full_match.start()]);

            // Add the original bash block
            result.push_str("```bash\n");
            result.push_str(cmd_str);
            result.push_str("\n```\n");

            // Execute the command and add its output right after the code block
            match crate::handlers::bash::handle_bash_command(cmd_str) {
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
