mod ollama;
pub mod types;
mod factory;

pub use ollama::OllamaClient;
pub use types::{AIClient, AIError, AIResponse, SessionStats, TokenUsage, ModelCosts, Provider};
pub use factory::AIClientFactory;