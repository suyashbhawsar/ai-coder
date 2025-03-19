# AI Coder Interface

A terminal user interface (TUI) for interacting with large language models directly from the command line.

![Screenshot](docs/screenshot.png)

## Features

- Interactive AI chat directly in the terminal
- Execute shell commands with the `!` prefix
- Multiple AI provider support (currently Ollama)
- Token tracking and cost estimation
- Command history navigation
- Text selection and clipboard integration
- Customizable UI themes
- Configuration management
- Logging support

## Installation

### Prerequisites

- Rust 1.70 or higher
- For Ollama integration: [Ollama](https://ollama.com) installed and running

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
- **Ctrl+C**: Copy selected text (in selection mode) or exit
- **PageUp/Down**: Scroll output
- **Esc**: Cancel text selection or clear input

### Command Prefixes

- No prefix: AI mode - Ask questions or get creative responses
- `!` prefix: Execute bash commands (e.g., `!ls -la`)
- `/` prefix: CLI commands (see below)

### Available Commands

- `/help [topic]`: Show help (optional topics: ai, bash, config, theme, system)
- `/clear`: Clear terminal output
- `/config`: View or set configuration
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
  provider: "ollama"
  model: "qwen2.5-coder"
  endpoint: "http://localhost:11434"
  api_key: ""
  temperature: 0.7
  max_tokens: 2048
  system_prompt: null
history_size: 100
mouse_enabled: true
logging_enabled: false
log_file: "ai-coder.log"
```

## Development

### Project Structure

- `src/ai`: AI client implementations
- `src/app`: Core application state and logic
- `src/config`: Configuration management
- `src/event`: Event handling
- `src/handlers`: Command execution and handling
- `src/tui`: Terminal interface
- `src/ui`: UI rendering
- `src/utils`: Utility functions

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