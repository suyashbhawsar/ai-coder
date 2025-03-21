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
//!
//! # Architecture
//!
//! The application is organized into several key modules:
//! - `ai` - AI client implementations
//! - `app` - Core application state and logic
//! - `config` - Configuration management
//! - `event` - Event handling
//! - `handlers` - Command execution and handling
//! - `tui` - Terminal interface
//! - `ui` - UI rendering
//! - `utils` - Utility functions

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

    Ok(())
}
