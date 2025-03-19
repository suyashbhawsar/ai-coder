use crate::ai::{AIClient, AIError, AIResponse, OllamaClient, ModelCosts};
use crate::handlers::HandlerResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use regex::Regex;

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
        let client: Box<dyn AIClient> = Box::new(OllamaClient::new("qwen2.5-coder".to_string()));
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<AIResponse, AIError> {
        // First, check if Ollama is running
        self.check_service_availability().await?;

        // If we get here, service is available
        let client = self.client.lock().await;
        let response = client.generate(prompt, None).await?;

        // Process the response for bash blocks
        let processed_content = self.process_llm_output(&response.content).await
            .map_err(|e| AIError::InvalidResponse(format!("Failed to process bash blocks: {}", e)))?;

        Ok(AIResponse {
            content: processed_content,
            ..response
        })
    }

    // Helper method to check if the AI service is available
    async fn check_service_availability(&self) -> Result<(), AIError> {
        use reqwest::Client;
        use std::time::Duration;

        // Create a client with a short timeout for just checking availability
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| AIError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        // Try to connect to Ollama health endpoint
        match client.get("http://localhost:11434/api/tags").send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(AIError::NetworkError(format!(
                "Ollama not available (is it running?): {}. Start Ollama with 'ollama serve' command.", e
            )))
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>, AIError> {
        // Check if Ollama is running first
        self.check_service_availability().await?;

        let client = self.client.lock().await;
        client.models().await
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
                },
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