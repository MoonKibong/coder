use super::LlmBackend;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::time::Duration;

/// Anthropic API backend - Claude 3.5 Sonnet
/// WARNING: Remote provider - use for development/testing only
pub struct AnthropicBackend {
    endpoint: String,
    model: String,
    api_key: String,
    timeout: Duration,
    client: Client,
}

impl AnthropicBackend {
    pub fn new(endpoint: String, model: String, api_key: String, timeout_seconds: u64) -> Self {
        Self {
            endpoint,
            model,
            api_key,
            timeout: Duration::from_secs(timeout_seconds),
            client: Client::new(),
        }
    }

    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("LLM_ENDPOINT")
                .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            api_key: env::var("LLM_API_KEY").expect("LLM_API_KEY required for Anthropic provider"),
            timeout: Duration::from_secs(
                env::var("LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(120),
            ),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for AnthropicBackend {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/messages", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": prompt}]
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic request failed ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        // Anthropic doesn't have a simple health check endpoint
        // Verify API key format instead
        if self.api_key.starts_with("sk-ant-") {
            Ok(())
        } else {
            anyhow::bail!("Invalid Anthropic API key format (should start with 'sk-ant-')")
        }
    }
}
