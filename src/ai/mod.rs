mod ollama;
pub mod types;

pub use ollama::OllamaClient;
pub use types::{AIClient, AIError, AIResponse, SessionStats, TokenUsage, ModelCosts};