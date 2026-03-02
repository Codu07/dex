//! OpenAI API provider

use crate::llm::{ChatCompletionRequest, ChatCompletionResponse, LLMClient};
use crate::llm::provider::LLMProvider;
use crate::types::{LLMResponse, Message, Tool};
use anyhow::{Context, Result};
use async_trait::async_trait;

/// OpenAI API provider
pub struct OpenAIProvider {
    client: LLMClient,
    model: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self> {
        let client = LLMClient::new(api_key, "https://api.openai.com/v1")?;
        
        Ok(Self {
            client,
            model: model.into(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        })
    }

    /// Create with custom base URL (for Azure, proxies, etc.)
    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self> {
        let client = LLMClient::new(api_key, base_url)?;
        
        Ok(Self {
            client,
            model: model.into(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        })
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat(&self, messages: &[Message], tools: Option<&[Tool]>) -> Result<LLMResponse> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response: ChatCompletionResponse = self
            .client
            .post("/chat/completions", &request)
            .await
            .context("Failed to get chat completion")?;

        Ok(response.into())
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn is_configured(&self) -> bool {
        !self.client.api_key().is_empty()
    }
}

/// Create provider from configuration
pub fn create_provider(
    api_key: impl Into<String>,
    model: impl Into<String>,
    api_base: Option<String>,
) -> Result<Box<dyn LLMProvider>> {
    let api_key = api_key.into();
    let model = model.into();
    
    if api_key.is_empty() {
        anyhow::bail!("API key is required");
    }

    let provider = if let Some(base_url) = api_base {
        OpenAIProvider::with_base_url(api_key, base_url, model)?
    } else {
        OpenAIProvider::new(api_key, model)?
    };

    Ok(Box::new(provider))
}
