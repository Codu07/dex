//! Agent core logic

use crate::llm::LLMProvider;
use crate::tools::{ToolExecutor, ToolRegistry};
use crate::types::{Conversation, Message, Task, ToolResult};
use anyhow::{Context, Result};
use colored::Colorize;
use tracing::{debug, info, warn};

/// The main Agent struct
pub struct Agent {
    llm: Box<dyn LLMProvider>,
    tool_executor: ToolExecutor,
    conversation: Conversation,
    max_iterations: u32,
}

impl Agent {
    /// Create a new agent
    pub fn new(llm: Box<dyn LLMProvider>, tool_registry: ToolRegistry) -> Self {
        let tool_executor = ToolExecutor::new(tool_registry);
        let conversation = Conversation::new();

        Self {
            llm,
            tool_executor,
            conversation,
            max_iterations: 10,
        }
    }

    /// Create with system prompt
    pub fn with_system_prompt(
        llm: Box<dyn LLMProvider>,
        tool_registry: ToolRegistry,
        system_prompt: impl Into<String>,
    ) -> Self {
        let tool_executor = ToolExecutor::new(tool_registry);
        let conversation = Conversation::with_system_prompt(system_prompt);

        Self {
            llm,
            tool_executor,
            conversation,
            max_iterations: 10,
        }
    }

    /// Set max iterations for tool loops
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Run a task
    pub async fn run(&mut self, task: &Task) -> Result<String> {
        info!("Running task: {}", task.description);

        // Add user message
        let user_message = if let Some(context) = &task.context {
            let context_str = context
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n\nContext:\n{}", task.description, context_str)
        } else {
            task.description.clone()
        };

        self.conversation.add(Message::user(user_message));

        // Main loop
        for iteration in 0..self.max_iterations {
            debug!("Iteration {}/{}", iteration + 1, self.max_iterations);

            // Get response from LLM
            let tools = self.tool_executor.registry().definitions();
            let response = self
                .llm
                .chat(self.conversation.messages(), Some(&tools))
                .await
                .context("Failed to get LLM response")?;

            // Add assistant message to conversation
            let has_tool_calls = response.message.tool_calls.is_some();
            self.conversation.add(response.message.clone());

            // Print assistant response
            if !response.message.content.is_empty() {
                println!("\n{}", response.message.content.cyan());
            }

            // Handle tool calls
            if let Some(tool_calls) = response.message.tool_calls {
                if tool_calls.is_empty() {
                    // No more tool calls, we're done
                    return Ok(response.message.content);
                }

                // Execute tools
                println!("\n{} {}", "→".yellow(), "Executing tools...".yellow());
                
                let results = self.tool_executor.execute_all(&tool_calls).await;

                // Add tool results to conversation
                for (tool_call_id, result) in results {
                    let content = match &result {
                        ToolResult::Success(content) => {
                            println!("{} {}", "✓".green(), "Tool succeeded".green());
                            content.clone()
                        }
                        ToolResult::Error(err) => {
                            println!("{} {}", "✗".red(), format!("Tool failed: {}", err).red());
                            format!("Error: {}", err)
                        }
                    };

                    self.conversation.add(Message::tool_result(tool_call_id, content));
                }
            } else if !has_tool_calls {
                // No tool calls and no pending operations, we're done
                return Ok(response.message.content);
            }
        }

        warn!("Max iterations ({}) reached", self.max_iterations);
        Ok("Max iterations reached. Task may be incomplete.".to_string())
    }

    /// Run in chat mode (interactive)
    pub async fn chat(&mut self, message: &str) -> Result<String> {
        let task = Task::new(message);
        self.run(&task).await
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.conversation.clear();
    }

    /// Get conversation history
    pub fn conversation(&self) -> &Conversation {
        &self.conversation
    }

    /// Get available tools
    pub fn available_tools(&self) -> Vec<String> {
        self.tool_executor.registry().names()
    }
}

/// Agent builder for easier construction
pub struct AgentBuilder {
    llm: Option<Box<dyn LLMProvider>>,
    tool_registry: Option<ToolRegistry>,
    system_prompt: Option<String>,
    max_iterations: u32,
}

impl AgentBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            llm: None,
            tool_registry: None,
            system_prompt: None,
            max_iterations: 10,
        }
    }

    /// Set LLM provider
    pub fn with_llm(mut self, llm: Box<dyn LLMProvider>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Set tool registry
    pub fn with_tools(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Build the agent
    pub fn build(self) -> Result<Agent> {
        let llm = self.llm.context("LLM provider is required")?;
        let tool_registry = self.tool_registry.unwrap_or_else(ToolRegistry::with_defaults);

        let mut agent = if let Some(system_prompt) = self.system_prompt {
            Agent::with_system_prompt(llm, tool_registry, system_prompt)
        } else {
            Agent::new(llm, tool_registry)
        };

        agent.max_iterations = self.max_iterations;

        Ok(agent)
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
