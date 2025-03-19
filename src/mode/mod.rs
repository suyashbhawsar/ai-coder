use std::fmt;

pub mod ai;
pub mod bash;
pub mod command;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandMode {
    AI,
    Bash,
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
