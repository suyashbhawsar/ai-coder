//! Bash command handler
//!
//! This module handles execution of bash commands and provides
//! security controls and formatting of outputs.

use crate::handlers::{HandlerError, HandlerResult};
use regex::Regex;
use std::process::{Command, Stdio};
use std::time::Instant;

/// List of commands that are completely restricted for security
const RESTRICTED_COMMANDS: [&str; 12] = [
    "rm -rf /",
    "rm -rf /*",
    "rm -rf ~",
    "rm -rf ~/",
    "mkfs",
    "> /dev/sda",
    "dd if=/dev/zero of=/dev/sda",
    ":(){ :|:& };:",
    "chmod -R 777 /",
    "> /dev/null; rm", // Command injection attempts
    "$(rm",            // Command substitution attempts
    "`rm",             // Backtick command substitution
];

/// List of potentially dangerous patterns that should be blocked
const DANGEROUS_PATTERNS: [&str; 8] = [
    "rm -rf",
    "mkfs",
    "dd if=/dev/zero",
    "chmod -R 777",
    ":(){ ",
    "fork bomb",
    "wget", // External download tools
    "curl", // External download tools
];

/// Checks if a command is safe to execute
fn is_command_safe(command: &str) -> bool {
    // Check for exact matches to restricted commands
    for restricted in RESTRICTED_COMMANDS.iter() {
        if command.contains(restricted) {
            return false;
        }
    }

    // Compile regex for safe rm -rf pattern only once
    let safe_rm_pattern =
        Regex::new(r"rm\s+-rf\s+(?:\.\/)?[a-zA-Z0-9_\-\+\.]+(?:\/[a-zA-Z0-9_\-\+\.]+)*\s*$")
            .unwrap();

    // Check for dangerous patterns
    for pattern in DANGEROUS_PATTERNS.iter() {
        if command.contains(pattern) {
            // Allow specific safe cases with rm -rf that only affect current directory
            if pattern == &"rm -rf" && safe_rm_pattern.is_match(command) {
                return true;
            }
            return false;
        }
    }

    // Command passed all security checks
    true
}

/// Handle execution of a bash command
pub fn handle_bash_command(command: &str) -> HandlerResult<String> {
    // At the beginning of this function, we could add an abort check
    // But since it's not running in an async context, we'll handle abort
    // in the calling functions
    let command = command.trim();

    if command.is_empty() {
        return Err(HandlerError::Bash("Empty command".to_string()));
    }

    // Security checks
    if !is_command_safe(command) {
        return Err(HandlerError::Bash(
            "This command is restricted for security reasons.".to_string(),
        ));
    }

    // Execute and time the command
    let start_time = Instant::now();

    // For commands that use shell patterns, use the shell to interpret them
    if command.contains('*') || command.contains('?') || command.contains('[') {
        let result = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| HandlerError::Bash(format!("Failed to execute command: {}", e)))?;

        let elapsed = start_time.elapsed();
        let exit_code = result.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();

        return Ok(format_command_output(
            command,
            exit_code,
            &stdout,
            &stderr,
            elapsed.as_secs_f64(),
        ));
    }

    // For other commands, use direct execution
    let cmd_parts: Vec<String> = shell_words::split(command)
        .map_err(|e| HandlerError::Parse(format!("Failed to parse command: {}", e)))?;

    if cmd_parts.is_empty() {
        return Err(HandlerError::Parse("Invalid command format".to_string()));
    }

    let result = Command::new(&cmd_parts[0])
        .args(&cmd_parts[1..])
        .current_dir(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| HandlerError::Bash(format!("Failed to execute command: {}", e)))?;

    let elapsed = start_time.elapsed();
    let exit_code = result.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();

    Ok(format_command_output(
        command,
        exit_code,
        &stdout,
        &stderr,
        elapsed.as_secs_f64(),
    ))
}

/// Format command output with proper style and information
fn format_command_output(
    _command: &str, // Not used in the new format but kept for backwards compatibility
    return_code: i32,
    stdout: &str,
    stderr: &str,
    execution_time: f64,
) -> String {
    // Compact header with metadata
    let mut result = format!(
        "[⏱️ {:.2}s | {} | 📊 {}]\n",
        execution_time,
        if return_code == 0 { "✓" } else { "✗" },
        return_code
    );

    // Format output with cleaner headers
    if !stdout.is_empty() {
        result.push_str(stdout.trim_end());
        result.push('\n');
    }

    if !stderr.is_empty() {
        if !stdout.is_empty() {
            result.push_str("\n⚠️ STDERR:\n");
        } else {
            result.push_str("⚠️ STDERR:\n");
        }
        result.push_str(stderr.trim_end());
        result.push('\n');
    }

    if stdout.is_empty() && stderr.is_empty() {
        result.push_str("(no output)\n");
    }

    result
}
