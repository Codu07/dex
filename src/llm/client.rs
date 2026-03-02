//! HTTP client for LLM API calls

use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// HTTP client wrapper for LLM APIs
#[derive(Clone)]
pub struct LLMClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl LLMClient {
    /// Create a new LLM client
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            base_url: base_url.into(),
        })
    }

    /// Send a POST request
    pub async fn post<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))?;

        let status = response.status();
        
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("API error ({}): {}", status, error_text);
        }

        let result: R = response
            .json()
            .await
            .with_context(|| "Failed to parse response")?;

        Ok(result)
    }

    /// Get the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
