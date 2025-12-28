use super::LlmBackend;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::time::Duration;

/// Groq API backend - Fast inference, free tier available
/// WARNING: Remote provider - use for development/testing only
pub struct GroqBackend {
    endpoint: String,
    model: String,
    api_key: String,
    timeout: Duration,
    client: Client,
}

impl GroqBackend {
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
                .unwrap_or_else(|_| "https://api.groq.com/openai/v1".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string()),
            api_key: env::var("LLM_API_KEY").expect("LLM_API_KEY required for Groq provider"),
            timeout: Duration::from_secs(
                env::var("LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for GroqBackend {
    fn name(&self) -> &str {
        "groq"
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
            anyhow::bail!("Groq request failed ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        // Groq doesn't have a dedicated health endpoint, verify by listing models
        let url = format!("{}/models", self.endpoint);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Groq API not available or invalid API key");
        }

        Ok(())
    }
}
