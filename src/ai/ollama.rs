use crate::ai::types::{AIClient, AIError, AIResponse, ModelCosts, ProgressStats, TokenUsage};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    context: Option<Vec<i64>>,
    options: Option<GenerateOptions>,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GenerateResponse {
    model: String,
    response: String,
    done: bool,
    context: Option<Vec<i64>>,
    prompt_eval_count: Option<usize>,
    eval_count: Option<usize>,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    sample_count: Option<usize>,
    sample_duration: Option<u64>,
    prompt_eval_duration: Option<u64>,
    eval_duration: Option<u64>,
    eval_token_count: Option<usize>,
}

pub struct OllamaClient {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaClient {
    pub fn new(model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            model,
            base_url: OLLAMA_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(base_url: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            model,
            base_url,
        }
    }

    fn count_tokens(&self, text: &str) -> usize {
        // Simple token counting approximation
        // In practice, different models might count tokens differently
        // This is a rough approximation that works reasonably well for English text
        text.split_whitespace().count()
    }
}

#[async_trait]
impl AIClient for OllamaClient {
    async fn generate(&self, prompt: &str, _context: Option<&str>) -> Result<AIResponse, AIError> {
        // Use a properly configured client with appropriate timeouts
        let client = &self.client;

        // Create the request object with streaming enabled
        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: true, // Use streaming for better progress reporting
            context: None,
            options: Some(GenerateOptions {
                num_predict: Some(2048), // Reasonable default token limit
            }),
        };

        // Send the request with proper error handling
        let response = client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                eprintln!("Ollama request error: {}", e);
                AIError::APIError(format!("Failed to send request to Ollama: {}", e))
            })?;

        // Check HTTP status
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "<could not read error body>".to_string());
            eprintln!("Ollama API error [{}]: {}", status, error_body);
            return Err(AIError::APIError(format!(
                "Ollama API returned error status: {} - {}",
                status, error_body
            )));
        }

        // Process streamed response
        let mut response_stream = response.bytes_stream();
        let mut full_content = String::new();
        let mut model_name = self.model.clone();
        let mut progress_stats = ProgressStats::new();
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;

        // Estimated token count for progress estimation
        progress_stats.estimated_total_tokens = Some(2048); // Initial estimate

        while let Some(chunk_result) = response_stream.next().await {
            let chunk = chunk_result
                .map_err(|e| AIError::APIError(format!("Error reading stream chunk: {}", e)))?;

            // Parse the chunk as JSON
            if let Ok(text) = std::str::from_utf8(&chunk) {
                if let Ok(response) = serde_json::from_str::<GenerateResponse>(text) {
                    // Add the new content
                    full_content.push_str(&response.response);

                    // Update model name if present
                    if !response.model.is_empty() {
                        model_name = response.model;
                    }

                    // Update prompt token count if provided
                    if let Some(count) = response.prompt_eval_count {
                        prompt_tokens = count;
                    }

                    // Update completion token count
                    if let Some(count) = response.eval_count {
                        completion_tokens = count;

                        // Update progress stats
                        progress_stats.update(count);

                        // Update estimated total if we have completion percentage
                        if let Some(total_duration) = response.total_duration {
                            if let Some(eval_duration) = response.eval_duration {
                                if eval_duration > 0 && total_duration > 0 {
                                    // Estimate total tokens based on how much time has been used
                                    let progress_percent =
                                        eval_duration as f64 / total_duration as f64;
                                    if progress_percent > 0.0 {
                                        let estimated_total =
                                            (count as f64 / progress_percent) as usize;
                                        progress_stats.estimated_total_tokens =
                                            Some(estimated_total);
                                    }
                                }
                            }
                        }
                    }

                    // If done is true, we've reached the end
                    if response.done {
                        // Mark progress as complete
                        progress_stats.complete();
                        break;
                    }
                }
            }
        }

        // Ensure we have token counts
        if prompt_tokens == 0 {
            prompt_tokens = self.count_tokens(prompt);
        }

        if completion_tokens == 0 {
            completion_tokens = self.count_tokens(&full_content);
        }

        let usage = TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };

        Ok(AIResponse {
            content: full_content,
            model: model_name,
            usage,
            progress: Some(progress_stats),
        })
    }

    async fn models(&self) -> Result<Vec<String>, AIError> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AIError::APIError(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::APIError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>,
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| AIError::APIError(format!("Failed to parse response: {}", e)))?;

        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    fn get_model_costs(&self, model: &str) -> ModelCosts {
        // Define costs for different Ollama models
        // These are placeholder values since Ollama is free and local
        // You can adjust these based on your needs or computational costs
        match model {
            m if m.contains("llama2") => ModelCosts {
                prompt_cost_per_1k: 0.0001,     // $0.0001 per 1K tokens
                completion_cost_per_1k: 0.0002, // $0.0002 per 1K tokens
            },
            m if m.contains("codellama") => ModelCosts {
                prompt_cost_per_1k: 0.0002,
                completion_cost_per_1k: 0.0004,
            },
            m if m.contains("qwen") => ModelCosts {
                prompt_cost_per_1k: 0.0002,
                completion_cost_per_1k: 0.0004,
            },
            _ => ModelCosts {
                prompt_cost_per_1k: 0.0001,
                completion_cost_per_1k: 0.0002,
            },
        }
    }
}
