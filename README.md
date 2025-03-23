# AI Coder Interface

A terminal user interface (TUI) for interacting with large language models directly from the command line.

![Screenshot](docs/screenshot.png)

## Features

- Interactive AI chat directly in the terminal
- Execute shell commands with the `!` prefix
- Multiple AI provider support (Ollama, OpenAI, Anthropic, LMStudio)
- Dynamic model discovery and switching
- Support for all Ollama models (llama3, codellama, mistral, qwen, gemma, etc.)
- Token tracking and cost estimation
- Command history navigation
- Text selection and clipboard integration
- Customizable UI themes
- Modular configuration system
- Resilient error handling
- Logging support
- Process abortion with Escape key
- Non-blocking, concurrent operation for AI requests
- Responsive UI that never freezes
- Background task management
- Real-time progress indication with spinner
- Graceful timeout handling
- Thread-safe API interaction
- Minimalist, clean output design
- Optimized whitespace management
- Consistent UI formatting and spacing
- Efficient screen space utilization

## Installation

### Prerequisites

- Rust 1.70 or higher
- For Ollama integration: [Ollama](https://ollama.com) installed and running
- For OpenAI/Anthropic: Valid API keys for the respective services
- For LMStudio: [LMStudio](https://lmstudio.ai/) installed and running with API server enabled

### Building from source

```bash
# Clone the repository
git clone https://github.com/yourusername/ai-coder-interface-rs.git
cd ai-coder-interface-rs

# Build in release mode
cargo build --release

# Run the application
cargo run --release
```

## Usage

### Keyboard Shortcuts

- **Up/Down Arrow**: Navigate command history
- **Shift+Up/Down**: Select text in output area
- **Ctrl+C**: Copy selected text (in selection mode) or emergency abort
- **Ctrl+D**: Exit application cleanly
- **PageUp/Down**: Scroll output
- **Esc**: Abort current operation, cancel text selection, or clear input
- **Shift+Enter**: Add a new line in the input box

### Command Prefixes

- No prefix: AI mode - Ask questions or get creative responses
- `!` prefix: Execute bash commands (e.g., `!ls -la`)
- `/` prefix: CLI commands (see below)

### Available Commands

- `/help [topic]`: Show help (optional topics: ai, bash, config, theme, system, list)
- `/clear`: Clear terminal output
- `/config`: View or set configuration
- `/config provider <name>`: Set AI provider (ollama, openai, anthropic, lmstudio)
- `/config model <name>`: Set AI model for current provider
- `/config endpoint <url>`: Set API endpoint URL
- `/config api_key <key>`: Set API key (for OpenAI/Anthropic)
- `/config temperature <value>`: Set temperature (0.0-1.0)
- `/config system_prompt <text>`: Set system prompt
- `/list providers`: Show available AI providers
- `/list models`: Show available models for current provider
- `/list config`: Show all current configuration
- `/theme`: Customize UI colors
- `/system`: Display system information
- `/version`: Show version information
- `/exit` or `/quit`: Exit application

### Configuration

The application stores its configuration in `~/.ai-coder/config.yaml`. You can modify this file directly or use the `/config` command.

Example configuration:

```yaml
theme:
  primary: "#0087AF"
  secondary: "#00AF87"
  accent: "#AF8700"
  background: "default"
  foreground: "default"
ai:
  active_provider: "ollama"
  ollama:
    endpoint: "http://localhost:11434"
    current_model_index: 0
    models:
      - name: "llama3"
        temperature: 0.1
        max_tokens: 4000
        system_prompt: "You are a helpful AI coding assistant."
      - name: "codellama"
        temperature: 0.1
        max_tokens: 8000
        system_prompt: "You are a helpful AI coding assistant specializing in programming."
  openai:
    endpoint: "https://api.openai.com/v1"
    api_key: ""
    current_model_index: 0
    models:
      - name: "gpt-4o"
        temperature: 0.1
        max_tokens: 4000
        system_prompt: "You are a helpful AI coding assistant."
  anthropic:
    endpoint: "https://api.anthropic.com"
    api_key: ""
    current_model_index: 0
    models:
      - name: "claude-3-opus-20240229"
        temperature: 0.1
        max_tokens: 4000
        system_prompt: "You are a helpful AI coding assistant."
  lmstudio:
    endpoint: "http://localhost:1234/v1"
    current_model_index: 0
    models:
      - name: "local-model"
        temperature: 0.1
        max_tokens: 4000
        system_prompt: "You are a helpful AI coding assistant."
history_size: 100
mouse_enabled: true
logging_enabled: false
log_file: "ai-coder.log"
```

## Development

### Project Structure

- `src/ai`: AI client implementations and provider abstraction
  - `src/ai/types.rs`: Common interfaces and provider enum
  - `src/ai/factory.rs`: Factory pattern for client creation
  - `src/ai/ollama.rs`: Ollama-specific client implementation
- `src/app`: Core application state and logic
  - `src/app/ai_handler.rs`: AI service integration with concurrent processing
- `src/config`: Configuration management with provider-specific settings
- `src/event`: Event handling and input processing with abort signals
- `src/handlers`: Command execution and handling
  - `src/handlers/command.rs`: Built-in command implementation
  - `src/handlers/bash.rs`: Shell command execution
- `src/tui`: Terminal interface and rendering
- `src/ui`: UI components and layout with progress indicators
- `src/utils`: Utility functions and helpers
  - `src/utils/tasks.rs`: Background task management system
- `src/main.rs`: Application entry point with concurrent event loop

### Concurrency Model

The application uses a modern concurrent architecture:

- **tokio::select!** for non-blocking event handling
- **Arc<AtomicBool>** for thread-safe abort flags
- **Background Tasks** run independently without blocking the UI
- **Task Cleanup** to prevent resource leaks
- **Channel-based Communication** between UI and background tasks
- **Optimized Event Loop** that maintains UI responsiveness
- **Spinner Animation** providing real-time progress feedback

### UI Design Philosophy

The interface follows a minimalist design approach:

- **Clean Output**: No debug messages or task status clutter
- **Efficient Space**: Minimal whitespace and compact separators
- **Consistent Formatting**: Single newline between input and output
- **Automated Cleanup**: Spinner removal and whitespace management
- **Responsive Layout**: Adapts to terminal size with proper text wrapping
- **Visual Clarity**: Clear command input and response separation
- **Progress Indication**: Non-intrusive spinner animation during processing

### Running Tests

```bash
cargo test
```

### Running Benchmarks

```bash
cargo bench
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request