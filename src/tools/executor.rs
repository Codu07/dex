//! Tool executor

use crate::tools::ToolRegistry;
use crate::types::{ToolCall, ToolResult};
use anyhow::Result;

/// Executes tool calls from the LLM
pub struct ToolExecutor {
    registry: ToolRegistry,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new(registry: ToolRegistry) -> Self {
        Self { registry }
    }

    /// Execute a tool call
    pub async fn execute(&self, tool_call: &ToolCall) -> Result<ToolResult> {
        let tool_name = &tool_call.function.name;
        let tool = self
            .registry
            .get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", tool_name))?;

        let result = tool.execute(&tool_call.function.arguments).await?;
        Ok(result)
    }

    /// Execute multiple tool calls
    pub async fn execute_all(&self, tool_calls: &[ToolCall]) -> Vec<(String, ToolResult)> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            let tool_call_id = tool_call.id.clone();
            match self.execute(tool_call).await {
                Ok(result) => results.push((tool_call_id, result)),
                Err(e) => results.push((tool_call_id, ToolResult::Error(e.to_string()))),
            }
        }

        results
    }

    /// Get the tool registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
}
