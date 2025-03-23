//! AI Coder Interface - A TUI application for interacting with LLMs
//!
//! This crate provides a terminal user interface for seamlessly working with
//! large language models directly from the command line.
//!
//! # Features
//!
//! - Interactive AI chat in the terminal
//! - Execute shell commands
//! - Customizable UI
//! - Multiple AI provider support
//! - Token tracking and cost estimation
//! - Process abortion with Escape key
//! - Non-blocking concurrent operations
//! - Thread-safe API interactions
//! - Real-time progress indication
//! - Background task management
//!
//! # Architecture
//!
//! The application is organized into several key modules:
//! - `ai` - AI client implementations with thread-safe interfaces
//! - `app` - Core application state and concurrent task management
//! - `config` - Configuration management with runtime updates
//! - `event` - Event handling with abort signal support
//! - `handlers` - Command execution in background tasks
//! - `tui` - Terminal interface with non-blocking rendering
//! - `ui` - UI rendering with progress indicators
//! - `utils` - Utility functions and logging
//!
//! # Concurrency Model
//!
//! The application uses a modern, robust concurrency architecture:
//!
//! - **Tokio Runtime**: Asynchronous execution with tokio::select for handling multiple event sources
//! - **Thread Safety**: Arc<AtomicBool> for cross-thread abort flags
//! - **Background Tasks**: Long-running operations execute in tokio tasks
//! - **Task Management**: Automatic cleanup of completed background tasks
//! - **Non-blocking UI**: Event loop never blocks on IO operations
//! - **Channel Communication**: MPSC channels for UI updates from background tasks
//! - **Progress Indication**: Spinner animation with thread-safe updates
//! - **Error Propagation**: Proper error handling across thread boundaries

pub mod ai;
pub mod app;
pub mod config;
pub mod event;
pub mod handlers;
pub mod tui;
pub mod ui;
pub mod utils;

/// Re-export primary types for convenience
pub use app::App;
pub use config::AppConfig;
pub use event::Event;
pub use tui::Tui;

/// Initialize the application
pub fn init() -> anyhow::Result<()> {
    // Initialize configuration
    config::init_config()?;

    // Initialize logging
    utils::init_logging()?;

    Ok(())
}

/// Clean up application resources
pub fn cleanup() -> anyhow::Result<()> {
    // Close logging
    utils::close_logging()?;

    // Any outstanding background tasks will be automatically aborted
    // when the tokio runtime is dropped as the program exits

    Ok(())
}
