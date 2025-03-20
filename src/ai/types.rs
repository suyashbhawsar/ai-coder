//! AI module types
//!
//! This module defines the core types used across all AI providers.

use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Supported AI provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    /// Ollama local models
    Ollama,
    /// OpenAI API models
    OpenAI,
    /// Anthropic API models
    Anthropic,
    /// Local models via LM Studio
    LMStudio,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Ollama => write!(f, "Ollama"),
            Provider::OpenAI => write!(f, "OpenAI"),
            Provider::Anthropic => write!(f, "Anthropic"),
            Provider::LMStudio => write!(f, "LMStudio"),
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(Provider::Ollama),
            "openai" => Ok(Provider::OpenAI),
            "anthropic" => Ok(Provider::Anthropic),
            "lmstudio" => Ok(Provider::LMStudio),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self::Ollama
    }
}

/// Errors that can occur when working with AI providers
#[derive(Debug, Error)]
pub enum AIError {
    /// Network-related errors
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// API errors from the provider
    #[error("API error: {0}")]
    APIError(String),
    
    /// Invalid response errors
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Rate limit errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Content policy errors
    #[error("Content policy violation: {0}")]
    ContentPolicy(String),
    
    /// Internal server errors
    #[error("Internal server error: {0}")]
    ServerError(String),
}

/// Response from an AI completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    /// Generated text content
    pub content: String,
    
    /// Model identifier that generated the response
    pub model: String,
    
    /// Token usage information
    pub usage: TokenUsage,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens used in the prompt
    pub prompt_tokens: usize,
    
    /// Tokens generated in the completion
    pub completion_tokens: usize,
    
    /// Total tokens used (prompt + completion)
    pub total_tokens: usize,
}

/// Model cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCosts {
    /// Cost per 1,000 prompt tokens
    pub prompt_cost_per_1k: f64,
    
    /// Cost per 1,000 completion tokens
    pub completion_cost_per_1k: f64,
}

impl ModelCosts {
    /// Calculate cost for a given token usage
    pub fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let prompt_cost = (usage.prompt_tokens as f64 / 1000.0) * self.prompt_cost_per_1k;
        let completion_cost = (usage.completion_tokens as f64 / 1000.0) * self.completion_cost_per_1k;
        prompt_cost + completion_cost
    }
}

/// Session statistics for token usage and costs
#[derive(Debug, Default)]
pub struct SessionStats {
    /// Total prompt tokens used in the session
    pub total_prompt_tokens: usize,
    
    /// Total completion tokens used in the session
    pub total_completion_tokens: usize,
    
    /// Total cost incurred in the session
    pub total_cost: f64,
}

impl SessionStats {
    /// Update session stats with new usage data
    pub fn update(&mut self, usage: &TokenUsage, costs: &ModelCosts) {
        self.total_prompt_tokens += usage.prompt_tokens;
        self.total_completion_tokens += usage.completion_tokens;
        self.total_cost += costs.calculate_cost(usage);
    }
    
    /// Get total tokens used in the session
    pub fn total_tokens(&self) -> usize {
        self.total_prompt_tokens + self.total_completion_tokens
    }
}

/// Trait for AI clients
#[async_trait]
pub trait AIClient: Send + Sync {
    /// Generate a completion for the given prompt
    async fn generate(&self, prompt: &str, context: Option<&str>) -> Result<AIResponse, AIError>;
    
    /// List available models
    async fn models(&self) -> Result<Vec<String>, AIError>;
    
    /// Get cost information for a specific model
    fn get_model_costs(&self, model: &str) -> ModelCosts;
}