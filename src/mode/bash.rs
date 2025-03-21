use std::process::{Command, Stdio};
use std::time::Instant;

// List of restricted commands for security
const RESTRICTED_COMMANDS: [&str; 9] = [
    "rm -rf /",
    "rm -rf /*",
    "rm -rf ~",
    "rm -rf ~/",
    "mkfs",
    "> /dev/sda",
    "dd if=/dev/zero of=/dev/sda",
    ":(){ :|:& };:",
    "chmod -R 777 /",
];

// List of potentially dangerous patterns
const DANGEROUS_PATTERNS: [&str; 6] = [
    "rm -rf",
    "mkfs",
    "dd if=/dev/zero",
    "chmod -R 777",
    ":(){ ",
    "fork bomb",
];

pub fn handle_bash_command(command: &str) -> String {
    let command = command.trim();

    if command.is_empty() {
        return "⚠️ Empty command".to_string();
    }

    // Security checks
    for restricted in RESTRICTED_COMMANDS.iter() {
        if command.contains(restricted) {
            return "⚠️ This command is restricted for security reasons.".to_string();
        }
    }

    // Check for potentially dangerous commands
    for pattern in DANGEROUS_PATTERNS.iter() {
        if command.contains(pattern) {
            return "⚠️ This command contains potentially dangerous operations.\n\
                For safety reasons, it has been blocked."
                .to_string();
        }
    }

    // Execute and time the command
    let start_time = Instant::now();

    // Prepare command for execution
    let cmd_parts: Vec<String> =
        shell_words::split(command).unwrap_or_else(|_| vec![command.to_string()]);

    if cmd_parts.is_empty() {
        return "⚠️ Invalid command format".to_string();
    }

    // Execute command with timeout
    let result = Command::new(&cmd_parts[0])
        .args(&cmd_parts[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let elapsed = start_time.elapsed();

    match result {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            format_command_output(command, exit_code, &stdout, &stderr, elapsed.as_secs_f64())
        }
        Err(e) => {
            // Handle different error types
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("⚠️ Command not found: {}", cmd_parts[0])
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("⚠️ Permission denied when executing: {}", command)
            } else {
                format!("⚠️ Error executing command: {}", e)
            }
        }
    }
}

fn format_command_output(
    command: &str,
    return_code: i32,
    stdout: &str,
    stderr: &str,
    execution_time: f64,
) -> String {
    // Prepare header
    let mut result = format!("📎 Command: {}\n", command);
    result.push_str(&format!("⏱️ Execution time: {:.2}s\n", execution_time));
    result.push_str(&format!("📊 Return code: {}\n", return_code));

    // Add a separator
    result.push_str(&format!("{}\n", "─".repeat(50)));

    // Format output
    if !stdout.is_empty() {
        result.push_str("📤 STDOUT:\n");
        result.push_str(stdout.trim_end());
        result.push('\n');
    }

    if !stderr.is_empty() {
        if !stdout.is_empty() {
            result.push('\n');
        }
        result.push_str("⚠️ STDERR:\n");
        result.push_str(stderr.trim_end());
        result.push('\n');
    }

    if stdout.is_empty() && stderr.is_empty() {
        result.push_str("📄 No output\n");
    }

    result
}
