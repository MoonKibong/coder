use super::LlmBackend;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use std::time::Duration;

/// vLLM server backend - OpenAI-compatible API with optional authentication
pub struct VllmBackend {
    endpoint: String,
    model: String,
    api_key: Option<String>,
    timeout: Duration,
    client: Client,
}

impl VllmBackend {
    pub fn new(
        endpoint: String,
        model: String,
        api_key: Option<String>,
        timeout_seconds: u64,
    ) -> Self {
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
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "codellama/CodeLlama-13b-hf".to_string()),
            api_key: env::var("LLM_API_KEY").ok(),
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
impl LlmBackend for VllmBackend {
    fn name(&self) -> &str {
        "vllm"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/v1/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let mut request = self.client.post(&url).json(&body);

        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.timeout(self.timeout).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("vLLM request failed ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/health", self.endpoint);
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("vLLM server not available");
        }

        Ok(())
    }
}
