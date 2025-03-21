use chrono::Local;
use std::env;
use std::process;

use crate::mode::ai::{clear_history, get_model_list, get_session_cost};

// Since we can't pass the app directly here, we'll keep the function signature
// but add a new function for getting cost stats directly from the app
pub fn handle_command(command: &str) -> String {
    // Split command and arguments
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = if parts.is_empty() {
        "".to_string()
    } else {
        parts[0].to_lowercase()
    };
    let args = if parts.len() > 1 { &parts[1..] } else { &[] };

    // Command mapping - only app-specific commands
    match cmd.as_str() {
        "help" => show_help(args),
        "clear" => "/clear".to_string(), // Special return value handled by main
        "exit" | "quit" => exit_app(),
        "models" => get_model_list(),
        "cost" => get_session_cost_display(), // Use the new function
        "history" => clear_history(),
        "config" => "Use /config key value to set configuration parameters".to_string(),
        "version" => show_version(),
        "echo" => args.join(" "),
        "system" => show_system_info(),
        "" => "Type /help for available commands".to_string(),
        _ => format!("⚠️ Unknown command '{}'. Type '/help' for commands.", cmd),
    }
}

// This function will retrieve formatted cost stats
fn get_session_cost_display() -> String {
    // This is now implemented in the App, we'll access it via the get_session_cost function
    // which will directly access the stats from the thread-local App
    get_session_cost()
}

fn show_help(args: &[&str]) -> String {
    if !args.is_empty() {
        // Show help for a specific command
        let specific_cmd = args[0].to_lowercase();
        let help_topics = [
            (
                "ai",
                "📚 AI Mode Help:\n\
                Just type your question or prompt directly without any prefix.\n\
                Examples:\n\
                  - What is the capital of France?\n\
                  - Write a Python function to calculate Fibonacci numbers\n\
                  - Explain the difference between TCP and UDP",
            ),
            (
                "bash",
                "📚 Bash Mode Help:\n\
                Prefix any bash command with ! to execute it directly.\n\
                Examples:\n\
                  - !ls -la\n\
                  - !cat file.txt\n\
                  - !python script.py",
            ),
            (
                "config",
                "📚 Config Command Help:\n\
                Configure settings using /config [key] [value]\n\
                Example keys:\n\
                  - model - Set default model\n\
                  - provider - Set AI provider (e.g. ollama)\n\
                  - temperature - Set temperature (0.0-1.0)",
            ),
            (
                "models",
                "📚 Models Command Help:\n\
                Use /models to list available language models\n\
                Use /config model MODEL_NAME to switch models",
            ),
            (
                "cost",
                "📚 Cost Command Help:\n\
                Use /cost to view detailed token usage and cost information\n\
                Shows:\n\
                  - Input/Output token counts\n\
                  - Cost breakdown by token type\n\
                  - Total session cost",
            ),
        ];

        for (topic, help_text) in help_topics {
            if specific_cmd == topic {
                return help_text.to_string();
            }
        }

        return format!(
            "⚠️ No help available for '{}'. Try '/help' for general help.",
            specific_cmd
        );
    }

    // General help
    "📚 LLM-ChainFuse CLI Help:\n\n\
    Mode Prefixes:\n\
      - No prefix: AI mode - Ask questions or get creative responses\n\
      - ! prefix: Execute bash commands (e.g., !ls)\n\
      - / prefix: CLI commands (see below)\n\n\
    Available commands:\n\
      /help [topic]   - Show help (optional: ai, bash, config, models, cost)\n\
      /clear          - Clear terminal output\n\
      /config         - View or set configuration\n\
      /models         - List available AI models\n\
      /cost           - Show token usage and costs\n\
      /history        - Clear conversation history\n\
      /system         - Display system information\n\
      /version        - Show version information\n\
      /exit or /quit  - Exit application\n\n\
    Keyboard shortcuts:\n\
      - Up/Down arrow: Navigate command history\n\
      - Shift+Up/Down: Select text in output area\n\
      - Ctrl+C: Copy selected text (when in selection mode) or exit\n\
      - PageUp/Down: Scroll output\n\
      - Esc: Cancel text selection or clear input\n\
      - ?: Show/hide help overlay"
        .to_string()
}

fn exit_app() -> String {
    process::exit(0);
}

fn show_version() -> String {
    "LLM-ChainFuse CLI v0.1.0".to_string()
}

fn show_system_info() -> String {
    // Get basic system information
    let os_name = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    };

    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    format!(
        "System Information:\n\
OS: {}\n\
Rust: {}\n\
Current Time: {}\n\
Working Directory: {}",
        os_name,
        env!("CARGO_PKG_VERSION"),
        current_time,
        env::current_dir().unwrap_or_default().display()
    )
}
