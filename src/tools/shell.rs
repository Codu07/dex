//! Shell command execution tool

use crate::tools::ToolImplementation;
use crate::types::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;
use tokio::process::Command;

/// Shell command execution tool
pub struct ShellTool {
    working_dir: std::path::PathBuf,
    allowed_commands: Vec<String>,
}

impl ShellTool {
    /// Create a new shell tool
    pub fn new() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            allowed_commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "echo".to_string(),
                "pwd".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "head".to_string(),
                "tail".to_string(),
                "wc".to_string(),
                "git".to_string(),
                "cargo".to_string(),
                "rustc".to_string(),
            ],
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.working_dir = path.as_ref().to_path_buf();
        self
    }

    /// Set allowed commands
    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = commands;
        self
    }

    /// Check if command is allowed
    fn is_allowed(&self, command: &str) -> bool {
        let cmd = command.split_whitespace().next().unwrap_or(command);
        self.allowed_commands.iter().any(|allowed| allowed == cmd)
    }

    /// Execute shell command
    async fn execute_command(&self, command: &str, timeout_secs: u64) -> Result<ToolResult> {
        // Security check
        if !self.is_allowed(command) {
            return Ok(ToolResult::Error(
                format!("Command not allowed: {}. Allowed commands: {:?}", 
                    command.split_whitespace().next().unwrap_or(command),
                    self.allowed_commands)
            ));
        }

        // Use bash to execute the command
        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .kill_on_drop(true)
            .timeout(tokio::time::Duration::from_secs(timeout_secs))
            .output()
            .await
            .with_context(|| format!("Failed to execute command: {}", command))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            let result = if stdout.is_empty() && !stderr.is_empty() {
                format!("(stderr)\n{}", stderr)
            } else if !stderr.is_empty() {
                format!("{}\n(stderr)\n{}", stdout, stderr)
            } else {
                stdout
            };
            Ok(ToolResult::Success(result))
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            let error_msg = if !stderr.is_empty() {
                stderr
            } else {
                format!("Command failed with exit code {}", exit_code)
            };
            Ok(ToolResult::Error(error_msg))
        }
    }
}

#[async_trait]
impl ToolImplementation for ShellTool {
    fn definition(&self) -> Tool {
        Tool::new(
            "shell",
            "Execute shell commands (restricted to safe commands only)",
            json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 60, max: 300)",
                        "default": 60
                    }
                },
                "required": ["command"]
            }),
        )
    }

    async fn execute(&self, args: &str) -> Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(args)
            .with_context(|| "Failed to parse arguments")?;
        
        let command = args["command"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;
        
        let timeout = args["timeout"].as_u64().unwrap_or(60).min(300);
        
        self.execute_command(command, timeout).await
    }

    fn name(&self) -> &str {
        "shell"
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}
