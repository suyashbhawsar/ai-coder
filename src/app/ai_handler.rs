use crate::ai::{AIClient, AIError, AIResponse, OllamaClient, ModelCosts};
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
        client.generate(prompt, None).await
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
}