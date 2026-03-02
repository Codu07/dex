//! File system tool

use crate::tools::ToolImplementation;
use crate::types::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

/// File system operations tool
pub struct FileSystemTool {
    working_dir: std::path::PathBuf,
}

impl FileSystemTool {
    /// Create a new file system tool
    pub fn new() -> Self {
        Self {
            working_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        }
    }

    /// Set working directory
    pub fn with_working_dir(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.working_dir = path.as_ref().to_path_buf();
        self
    }

    /// Resolve path relative to working directory
    fn resolve_path(&self, path: &str) -> std::path::PathBuf {
        let path = std::path::Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Read file contents
    async fn read_file(&self, path: &str) -> Result<ToolResult> {
        let full_path = self.resolve_path(path);
        let content = tokio::fs::read_to_string(&full_path)
            .await
            .with_context(|| format!("Failed to read file: {}", full_path.display()))?;
        Ok(ToolResult::Success(content))
    }

    /// Write file contents
    async fn write_file(&self, path: &str, content: &str) -> Result<ToolResult> {
        let full_path = self.resolve_path(path);
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        tokio::fs::write(&full_path, content)
            .await
            .with_context(|| format!("Failed to write file: {}", full_path.display()))?;
        
        Ok(ToolResult::Success(format!("File written: {}", full_path.display())))
    }

    /// List directory contents
    async fn list_dir(&self, path: &str) -> Result<ToolResult> {
        let full_path = self.resolve_path(path);
        let mut entries = tokio::fs::read_dir(&full_path)
            .await
            .with_context(|| format!("Failed to read directory: {}", full_path.display()))?;
        
        let mut result = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type().await?;
            let type_str = if file_type.is_dir() {
                "dir"
            } else if file_type.is_file() {
                "file"
            } else {
                "other"
            };
            result.push(format!("[{}] {}", type_str, name));
        }
        
        Ok(ToolResult::Success(result.join("\n")))
    }
}

#[async_trait]
impl ToolImplementation for FileSystemTool {
    fn definition(&self) -> Tool {
        Tool::new(
            "fs",
            "File system operations: read, write, and list files",
            json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "list"],
                        "description": "The file system operation to perform"
                    },
                    "path": {
                        "type": "string",
                        "description": "The file or directory path"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write (for write operation)"
                    }
                },
                "required": ["operation", "path"]
            }),
        )
    }

    async fn execute(&self, args: &str) -> Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(args)
            .with_context(|| "Failed to parse arguments")?;
        
        let operation = args["operation"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'operation' argument"))?;
        let path = args["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' argument"))?;
        
        match operation {
            "read" => self.read_file(path).await,
            "write" => {
                let content = args["content"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing 'content' argument for write operation"))?;
                self.write_file(path, content).await
            }
            "list" => self.list_dir(path).await,
            _ => Ok(ToolResult::Error(format!("Unknown operation: {}", operation))),
        }
    }

    fn name(&self) -> &str {
        "fs"
    }
}

impl Default for FileSystemTool {
    fn default() -> Self {
        Self::new()
    }
}
