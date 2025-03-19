# Development Guidelines for ai-coder-interface-rs

## Build & Test Commands
- Build: `cargo build` (dev) or `cargo build --release` (optimized)
- Run: `cargo run` (requires Ollama running locally)
- Test all: `cargo test`
- Test specific: `cargo test utils_test::test_format_money`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Benchmarks: `cargo bench`

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