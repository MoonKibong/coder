use super::LlmBackend;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::time::Duration;

/// Ollama backend - default for on-premise production
pub struct OllamaBackend {
    endpoint: String,
    model: String,
    timeout: Duration,
    client: Client,
}

impl OllamaBackend {
    pub fn new(endpoint: String, model: String, timeout_seconds: u64) -> Self {
        Self {
            endpoint,
            model,
            timeout: Duration::from_secs(timeout_seconds),
            client: Client::new(),
        }
    }

    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("LLM_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "codellama:13b".to_string()),
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
impl LlmBackend for OllamaBackend {
    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/api/generate", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama request failed ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["response"].as_str().unwrap_or("").to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/api/tags", self.endpoint);
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama server not available");
        }

        Ok(())
    }
}
