//! AI module types
//!
//! This module defines the core types used across all AI providers.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

    /// Operation cancelled by user
    #[error("Operation cancelled: {0}")]
    Cancelled(String),
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

    /// Optional progress statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub progress: Option<ProgressStats>,
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
        let completion_cost =
            (usage.completion_tokens as f64 / 1000.0) * self.completion_cost_per_1k;
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

/// Status of a background task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is waiting to start
    Pending,
    /// Task is currently running
    Running,
    /// Task has completed successfully
    Completed,
    /// Task has failed
    Failed,
    /// Task was cancelled by the user
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Running => write!(f, "Running"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Progress statistics for tracking task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStats {
    /// Total number of tokens generated so far
    pub tokens_generated: usize,

    /// Estimated total tokens that will be generated
    pub estimated_total_tokens: Option<usize>,

    /// Start time of the task
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// Last update time
    pub last_update: chrono::DateTime<chrono::Utc>,

    /// Generation rate in tokens per second
    pub tokens_per_second: f64,

    /// Estimated completion percentage (0-100)
    pub completion_percent: Option<f64>,
}

impl Default for ProgressStats {
    fn default() -> Self {
        Self {
            tokens_generated: 0,
            estimated_total_tokens: None,
            start_time: chrono::Utc::now(),
            last_update: chrono::Utc::now(),
            tokens_per_second: 0.0,
            completion_percent: None,
        }
    }
}

impl ProgressStats {
    /// Create a new progress stats instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Update progress with new token count
    pub fn update(&mut self, tokens_generated: usize) {
        let now = chrono::Utc::now();
        let elapsed = (now - self.last_update).num_milliseconds() as f64 / 1000.0;

        // Only update rate if some time has passed
        if elapsed > 0.01 {
            let new_tokens = tokens_generated.saturating_sub(self.tokens_generated) as f64;
            let instant_rate = new_tokens / elapsed;

            // Exponential moving average for tokens_per_second (alpha = 0.3)
            if self.tokens_per_second > 0.0 {
                self.tokens_per_second = 0.7 * self.tokens_per_second + 0.3 * instant_rate;
            } else {
                self.tokens_per_second = instant_rate;
            }

            self.tokens_generated = tokens_generated;
            self.last_update = now;

            // Update completion percentage if we have an estimate
            if let Some(total) = self.estimated_total_tokens {
                if total > 0 {
                    let percent = (tokens_generated as f64 / total as f64) * 100.0;
                    self.completion_percent = Some(percent.min(99.9)); // Cap at 99.9% until fully complete
                }
            }
        }
    }

    /// Mark the task as completed
    pub fn complete(&mut self) {
        self.completion_percent = Some(100.0);
    }

    /// Estimate time remaining in seconds
    pub fn estimate_remaining_seconds(&self) -> Option<f64> {
        if self.tokens_per_second <= 0.0 {
            return None;
        }

        if let Some(total) = self.estimated_total_tokens {
            let remaining_tokens = total.saturating_sub(self.tokens_generated) as f64;
            Some(remaining_tokens / self.tokens_per_second)
        } else {
            None
        }
    }

    /// Get formatted time string for estimated completion
    pub fn format_remaining_time(&self) -> String {
        if let Some(seconds) = self.estimate_remaining_seconds() {
            if seconds < 1.0 {
                return "< 1 sec".to_string();
            } else if seconds < 60.0 {
                return format!("{:.0} sec", seconds);
            } else if seconds < 3600.0 {
                let minutes = (seconds / 60.0).ceil();
                return format!("{:.0} min", minutes);
            } else {
                let hours = (seconds / 3600.0).ceil();
                return format!("{:.0} hrs", hours);
            }
        }

        "unknown".to_string()
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
