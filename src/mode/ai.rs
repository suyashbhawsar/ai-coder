use crate::ai::{AIClient, OllamaClient, SessionStats};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;

static SESSION_STATS: Lazy<Mutex<SessionStats>> = Lazy::new(|| Mutex::new(SessionStats::default()));

// Simple token counting function (rough approximation)
pub fn count_tokens(text: &str) -> usize {
    // Very rough approximation - in production use a proper tokenizer
    let words: Vec<&str> = text.split_whitespace().collect();
    (words.len() as f64 * 1.3).round() as usize
}

// Process AI requests
pub fn handle_ai_response(prompt: &str) -> String {
    // Initialize Ollama client with default model
    let client = OllamaClient::new(String::from("qwen2.5-coder"));

    // Create a runtime for async operations
    let rt = Runtime::new().unwrap();

    // Execute the async operation
    match rt.block_on(client.generate(prompt, None)) {
        Ok(response) => {
            println!(
                "Debug: Response received with tokens - Prompt: {}, Completion: {}, Total: {}",
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                response.usage.total_tokens
            );

            // Update session stats
            let model_costs = client.get_model_costs(&response.model);
            if let Ok(mut stats) = SESSION_STATS.lock() {
                let old_prompt_tokens = stats.total_prompt_tokens;
                let old_completion_tokens = stats.total_completion_tokens;
                let old_cost = stats.total_cost;

                stats.update(&response.usage, &model_costs);

                println!(
                    "Debug: Session stats updated - Prompt tokens: {} -> {}, Completion tokens: {} -> {}, Cost: ${:.6} -> ${:.6}",
                    old_prompt_tokens,
                    stats.total_prompt_tokens,
                    old_completion_tokens,
                    stats.total_completion_tokens,
                    old_cost,
                    stats.total_cost
                );
            }

            // Return the response content
            response.content
        }
        Err(e) => format!("⚠️ Error: {}", e),
    }
}

// Handle AI configuration
pub fn handle_ai_config(args: &[&str]) -> String {
    if args.is_empty() {
        return "📝 AI Configuration Options:\n\
            - /config model [model_name]: Set the AI model\n\
            - /config provider [provider_name]: Set AI provider\n\
            - /config temperature [0.0-1.0]: Set response creativity\n\n\
            Current settings are stored in ~/.llm-chainfuse/config.yaml"
            .to_string();
    }

    let key = args[0].to_lowercase();
    let value = if args.len() > 1 { args[1] } else { "" };

    match key.as_str() {
        "model" => {
            if value.is_empty() {
                return "⚠️ Model name required. Usage: /config model MODEL_NAME".to_string();
            }
            format!("✅ Model set to: {}", value)
        }
        "provider" => {
            if value.is_empty() {
                return "⚠️ Provider name required. Usage: /config provider PROVIDER_NAME"
                    .to_string();
            }
            match value {
                "ollama" => "✅ Provider set to: ollama".to_string(),
                _ => format!(
                    "⚠️ Unknown provider: {}. Currently supported: ollama",
                    value
                ),
            }
        }
        "temperature" => match value.parse::<f64>() {
            Ok(temp) if (0.0..=1.0).contains(&temp) => {
                format!("✅ Temperature set to: {}", temp)
            }
            _ => "⚠️ Temperature must be between 0.0 and 1.0".to_string(),
        },
        _ => format!("⚠️ Unknown configuration key: {}", key),
    }
}

pub fn get_model_list() -> String {
    // Initialize Ollama client with default model
    let client = OllamaClient::new(String::from("qwen2.5-coder"));

    // Create a runtime for async operations
    let rt = Runtime::new().unwrap();

    // Execute the async operation
    match rt.block_on(client.models()) {
        Ok(models) => format!("Available models:\n{}", models.join("\n")),
        Err(e) => format!("⚠️ Error fetching models: {}", e),
    }
}

// Get session cost information
pub fn get_session_cost() -> String {
    // In a real app, we would get these from thread-local app state
    // For now, we'll use the static SESSION_STATS for backwards compatibility
    // This will be replaced by App's stats in the actual implementation

    if let Ok(stats) = SESSION_STATS.lock() {
        let total_tokens = stats.total_prompt_tokens + stats.total_completion_tokens;

        // Calculate individual costs
        let (input_cost, output_cost) = if total_tokens > 0 {
            let input_ratio = stats.total_prompt_tokens as f64 / total_tokens as f64;
            let output_ratio = stats.total_completion_tokens as f64 / total_tokens as f64;
            (
                stats.total_cost * input_ratio,
                stats.total_cost * output_ratio,
            )
        } else {
            (0.0, 0.0)
        };

        // This is the same format we'll use for App's stats
        format!(
            "Session statistics:\n\
            Tokens used:\n\
            - Input: {} tokens\n\
            - Output: {} tokens\n\
            - Total: {} tokens\n\n\
            Cost breakdown:\n\
            - Input cost: ${:.6}\n\
            - Output cost: ${:.6}\n\
            - Total cost: ${:.6}",
            stats.total_prompt_tokens,
            stats.total_completion_tokens,
            total_tokens,
            input_cost,
            output_cost,
            stats.total_cost
        )
    } else {
        "⚠️ Error accessing session statistics".to_string()
    }
}

// Get total cost for status display
pub fn get_total_cost() -> String {
    if let Ok(stats) = SESSION_STATS.lock() {
        format!("${:.6}", stats.total_cost)
    } else {
        "$0.000000".to_string()
    }
}

// Clear conversation history and reset stats
pub fn clear_history() -> String {
    if let Ok(mut stats) = SESSION_STATS.lock() {
        *stats = SessionStats::default();
    }
    "✅ Conversation history and statistics cleared".to_string()
}
