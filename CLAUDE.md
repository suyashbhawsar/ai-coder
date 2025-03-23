# Development Guidelines for ai-coder-interface-rs

## Build & Test Commands
- Build: `cargo build` (dev) or `cargo build --release` (optimized)
- Run: `cargo run` (requires Ollama running locally)
- Test all: `cargo test`
- Test specific: `cargo test utils_test::test_format_money`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Benchmarks: `cargo bench`

## Output Expectations
- **Minimal Design**: Output should be clean and minimalist without unnecessary text
- **No Debug Messages**: No debug output like "AI Response received: X chars"
- **No Task Status**: Task status messages like "[Task ABC] ✅ Completed in 5.2s" should be hidden
- **Limited Newlines**: Avoid excessive newlines that waste vertical space 
- **Clean Transitions**: Transitions between commands should be visually clean
- **Efficient Space**: Maximize content display area, minimize UI chrome
- **No Spinners in Output**: Spinner animation should be removed when response is displayed
- **Consistent Spacing**: Single newline between input and output responses
- **Trimmed Whitespace**: No trailing empty lines or excessive whitespace
- **Compact Layout**: Compact command separators with minimal whitespace

## Code Style Guidelines
- Use Rust 2021 edition idioms
- Document all public APIs with doc comments (`//!` for modules, `///` for items)
- Use thiserror for error types; implement Display for custom errors
- Use Result<T, ErrorType> for fallible functions; avoid unwrap/expect in production code
- Follow snake_case for variables and functions, PascalCase for types/traits
- Group imports: std first, then external crates, then internal modules
- Organize modules with clear boundaries; expose only necessary types (pub use)
- Use async/await for IO-bound operations
- Follow Rust's privacy model: keep implementation details private
- Prefer strong typing over stringly typed code
- Use constants for config values with clear names
- Maintain comprehensive test coverage for all utilities

## Project Expectations
- **Modularity**: Code should be highly modular with clear separation of concerns
- **Reliability**: Error handling must be robust with graceful recovery paths
- **Security**: Carefully validate all user inputs, especially in bash commands
- **Performance**: TUI should remain responsive even during network operations
- **Usability**: Interface should be intuitive with clear feedback to users
- **Extensibility**: Support for additional AI providers should be straightforward
- **Configuration**: User settings should persist across sessions
- **Documentation**: All features should have clear usage examples
- **Testing**: Critical paths should have proper test coverage
- **Resilience**: Handle network failures and service unavailability gracefully
- **Cross-platform**: Code should work on macOS, Linux, and Windows where possible
- **Interruptibility**: Any operation must be interruptible and respond to abort commands
- **Concurrency**: Background operations should run concurrently to maintain UI responsiveness
- **Non-blocking UI**: UI should never freeze even during long-running operations
- **Task Management**: Proper cleanup of background tasks to prevent resource leaks

## UI Requirements
- **Clean Status Bar**: No token usage debug output in terminal or status bar
- **Dynamic Input Box**: Input area should resize automatically based on input length
- **Blinking Cursor**: Input area must have a visible blinking vertical bar cursor ("|") that can be navigated with arrow keys
- **Text Editing**: Full editing capabilities including insertion, deletion, and cursor movement
- **Multiline Support**: Use Shift+Enter to add new lines in the input box
- **Text Wrapping**: Long text should wrap properly within input and output areas 
- **Automatic Resizing**: Input box must grow vertically when text exceeds width or has multiple lines
- **Smart Shrinking**: Input box should decrease in size when text is deleted or reduced
- **User Experience**: Clear visual indicators for input/output areas and modes
- **Responsive Layout**: UI components should adapt to terminal window size
- **Efficient Screen Space**: Maximize content area without unnecessary borders/margins
- **Error Messages**: Provide clear, user-friendly error messages with guidance
- **Visual Consistency**: Maintain consistent visual styling across components
- **Process Abortion**: Support immediate abortion of any running process with Escape key
- **Exit Command**: Support exiting the TUI cleanly with Ctrl+D
- **Background Indication**: Show clear visual indicators for background processes
- **Operation Feedback**: Provide real-time feedback on long-running operations
- **Abort Feedback**: Show clear visual confirmation when operations are aborted
- **Continued Usage**: Allow continued app usage while background tasks are running

## AI Provider Requirements
- **Provider Abstraction**: Support multiple AI providers through a common interface
- **Dynamic Model Discovery**: List available models dynamically rather than hardcoding them
- **Graceful Degradation**: Handle unavailable services without crashing
- **Model Switching**: Allow seamless switching between different AI models
- **Configuration Persistence**: Save user preferences for providers and models
- **Default Settings**: Use temperature of 0.1 for all newly added models
- **Ollama Integration**: Support all Ollama models without special-casing specific models
- **Helpful Feedback**: Provide clear instructions on how to download and configure models
- **Case Insensitivity**: Model name matching should be case-insensitive
- **Error Handling**: Robust error handling when services are unavailable
- **Command Robustness**: Commands should never crash the application
- **Service Status**: Clearly indicate when services like Ollama are unavailable
- **Low Overhead**: Fetching model information should not block the UI
- **Process Control**: All API calls must be cancellable and properly handle timeouts
- **Thread Safety**: All AI operations must be thread-safe using atomic operations
- **Asynchronous Processing**: AI requests should be processed asynchronously
- **Progress Updates**: Provide progress updates during long-running AI operations
- **Timeout Handling**: Gracefully handle timeouts during API calls
- **Memory Safety**: Ensure proper memory safety in concurrent contexts

## Configuration System
- **Central Configuration**: All settings defined in a single location (~/.ai-coder/config.yaml)
- **Provider-Specific Config**: Each provider has its own configuration block
- **Model-Specific Settings**: Each model can have custom temperature, tokens, and system prompts
- **Runtime Updates**: Configuration changes applied immediately without restart
- **Validation**: Validate configuration values and provide meaningful errors
- **Defaults**: Sensible defaults for all configuration options
- **Factory Pattern**: Use factory pattern for creating provider-specific clients
- **Client Updates**: Support updating client settings at runtime