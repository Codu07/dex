//! LLM (Large Language Model) module
//! Provides abstractions for different LLM providers

use crate::types::{LLMResponse, Message, Tool, Usage};
use anyhow::Result;
use async_trait::async_trait;

pub mod client;
pub mod openai;
pub mod provider;

pub use client::LLMClient;
pub use openai::OpenAIProvider;
pub use provider::LLMProvider;

/// Build request body for OpenAI-compatible API
#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// OpenAI-compatible response
#[derive(Debug, serde::Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(rename = "tool_calls")]
    pub tool_calls: Option<Vec<crate::types::ToolCall>>,
}

impl From<ChatCompletionResponse> for LLMResponse {
    fn from(response: ChatCompletionResponse) -> Self {
        let choice = response.choices.into_iter().next().expect("No choices in response");
        let message = Message {
            role: match choice.message.role.as_str() {
                "system" => crate::types::Role::System,
                "user" => crate::types::Role::User,
                "assistant" => crate::types::Role::Assistant,
                "tool" => crate::types::Role::Tool,
                _ => crate::types::Role::Assistant,
            },
            content: choice.message.content.unwrap_or_default(),
            tool_calls: choice.message.tool_calls,
            tool_call_id: None,
        };

        Self {
            message,
            usage: response.usage,
        }
    }
}
