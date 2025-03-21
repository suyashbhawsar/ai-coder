use crate::ai::types::{AIClient, AIError, AIResponse, ModelCosts, TokenUsage};
use async_trait::async_trait;
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
        let request = GenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            context: None,
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::APIError(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::APIError(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let generate_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| AIError::APIError(format!("Failed to parse response: {}", e)))?;

        // Calculate token counts
        // If Ollama provides token counts, use those, otherwise fall back to our approximation
        let prompt_tokens = generate_response
            .prompt_eval_count
            .unwrap_or_else(|| self.count_tokens(prompt));
        let completion_tokens = generate_response
            .eval_token_count
            .or(generate_response.eval_count)
            .unwrap_or_else(|| self.count_tokens(&generate_response.response));

        let usage = TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        };

        Ok(AIResponse {
            content: generate_response.response,
            model: generate_response.model,
            usage,
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
