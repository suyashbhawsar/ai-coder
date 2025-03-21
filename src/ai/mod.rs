mod factory;
mod ollama;
pub mod types;

pub use factory::AIClientFactory;
pub use ollama::OllamaClient;
pub use types::{AIClient, AIError, AIResponse, ModelCosts, Provider, SessionStats, TokenUsage};
