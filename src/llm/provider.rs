//! LLM Provider trait

use crate::types::{LLMResponse, Message, Tool};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a chat completion request
    async fn chat(&self, messages: &[Message], tools: Option<&[Tool]>) -> Result<LLMResponse>;
    
    /// Get the model name
    fn model(&self) -> &str;
    
    /// Check if the provider is properly configured
    fn is_configured(&self) -> bool;
}
