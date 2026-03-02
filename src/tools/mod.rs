//! Tool system for the agent
//! Tools allow the agent to perform actions in the environment

use crate::types::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

pub mod executor;
pub mod fs;
pub mod shell;

pub use executor::ToolExecutor;
pub use fs::FileSystemTool;
pub use shell::ShellTool;

/// Trait for tool implementations
#[async_trait]
pub trait ToolImplementation: Send + Sync {
    /// Get the tool definition
    fn definition(&self) -> Tool;
    
    /// Execute the tool with given arguments
    async fn execute(&self, args: &str) -> Result<ToolResult>;
    
    /// Get the tool name
    fn name(&self) -> &str;
}

/// Registry of available tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolImplementation>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        
        // Register file system tool
        registry.register(Box::new(FileSystemTool::new()));
        
        // Register shell tool
        registry.register(Box::new(ShellTool::new()));
        
        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn ToolImplementation>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&dyn ToolImplementation> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Get all tool definitions
    pub fn definitions(&self) -> Vec<Tool> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Get tool names
    pub fn names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Check if a tool exists
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}
