//! Command handlers module
//!
//! This module contains handlers for different command types, including:
//! - AI command handling
//! - Bash command execution
//! - Application commands

pub mod ai;
pub mod bash;
pub mod command;

use crate::ai::AIError;
use std::fmt;

/// Command mode type
#[derive(Debug, Clone, PartialEq)]
pub enum CommandMode {
    /// AI mode for LLM interactions
    AI,
    /// Bash mode for shell commands
    Bash,
    /// Command mode for application commands
    Command,
}

impl fmt::Display for CommandMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandMode::AI => write!(f, "AI"),
            CommandMode::Bash => write!(f, "BASH"),
            CommandMode::Command => write!(f, "CMD"),
        }
    }
}

/// Result type for handlers
pub type HandlerResult<T> = Result<T, HandlerError>;

/// Error types that can occur during command handling
#[derive(Debug)]
pub enum HandlerError {
    /// AI-related errors
    AI(AIError),
    /// Bash execution errors
    Bash(String),
    /// Command parsing errors
    Parse(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerError::AI(e) => write!(f, "AI error: {}", e),
            HandlerError::Bash(e) => write!(f, "Bash error: {}", e),
            HandlerError::Parse(e) => write!(f, "Parse error: {}", e),
            HandlerError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for HandlerError {}

impl From<AIError> for HandlerError {
    fn from(err: AIError) -> Self {
        HandlerError::AI(err)
    }
}

impl From<std::io::Error> for HandlerError {
    fn from(err: std::io::Error) -> Self {
        HandlerError::Other(err.to_string())
    }
}
