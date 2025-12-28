use super::LlmBackend;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::time::Duration;

/// OpenAI API backend - GPT-4o, GPT-4o-mini
/// WARNING: Remote provider - use for development/testing only
pub struct OpenAIBackend {
    endpoint: String,
    model: String,
    api_key: String,
    timeout: Duration,
    client: Client,
}

impl OpenAIBackend {
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
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            model: env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            api_key: env::var("LLM_API_KEY").expect("LLM_API_KEY required for OpenAI provider"),
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
impl LlmBackend for OpenAIBackend {
    fn name(&self) -> &str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI request failed ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/models", self.endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("OpenAI API not available or invalid API key");
        }

        Ok(())
    }
}
