# LLM Backend Abstraction Pattern

## Purpose
LLM 런타임을 완전히 추상화하여 모델 교체 시 플러그인/API 변경 없이 운영

## Supported Providers

### On-Premise (Production)
| Provider | Use Case | API Key Required |
|----------|----------|------------------|
| `ollama` | Self-hosted Ollama server | No |
| `llama-cpp` | llama.cpp server (OpenAI-compatible) | No |
| `vllm` | vLLM server (OpenAI-compatible) | Optional |

### Remote (Development/Testing)
| Provider | Use Case | API Key Required |
|----------|----------|------------------|
| `groq` | Fast inference, free tier available | Yes |
| `openai` | GPT-4o, GPT-4o-mini | Yes |
| `anthropic` | Claude 3.5 Sonnet | Yes |
| `openrouter` | Multi-provider aggregator (OpenAI-compatible) | Yes |

> **Note**: Remote providers are for **development/testing only**. Production deployments must use on-premise providers to comply with 금융권 보안 요구사항.

---

## Environment Variables

Configuration via environment variables (for PoC, before admin panel):

```bash
# Provider selection
LLM_PROVIDER=ollama              # ollama | llama-cpp | vllm

# Connection
LLM_ENDPOINT=http://localhost:11434   # Server URL
LLM_MODEL=codellama:13b               # Model name

# Authentication (optional, for vLLM with auth)
LLM_API_KEY=                          # API key if required

# Performance
LLM_TIMEOUT_SECONDS=120               # Request timeout
LLM_MAX_RETRIES=2                     # Retry count on failure
```

### Provider-Specific Defaults

```bash
# === ON-PREMISE PROVIDERS (Production) ===

# Ollama (default for production)
LLM_PROVIDER=ollama
LLM_ENDPOINT=http://localhost:11434
LLM_MODEL=codellama:13b

# llama.cpp (OpenAI-compatible server)
LLM_PROVIDER=llama-cpp
LLM_ENDPOINT=http://localhost:8080
LLM_MODEL=codellama

# vLLM (OpenAI-compatible server)
LLM_PROVIDER=vllm
LLM_ENDPOINT=http://localhost:8000
LLM_MODEL=codellama/CodeLlama-13b-hf

# === REMOTE PROVIDERS (Development/Testing Only) ===

# Groq (recommended for testing - fast & free tier)
LLM_PROVIDER=groq
LLM_ENDPOINT=https://api.groq.com/openai/v1
LLM_MODEL=llama-3.3-70b-versatile
LLM_API_KEY=gsk_xxxxx

# OpenAI
LLM_PROVIDER=openai
LLM_ENDPOINT=https://api.openai.com/v1
LLM_MODEL=gpt-4o-mini
LLM_API_KEY=sk-xxxxx

# Anthropic
LLM_PROVIDER=anthropic
LLM_ENDPOINT=https://api.anthropic.com/v1
LLM_MODEL=claude-3-5-sonnet-20241022
LLM_API_KEY=sk-ant-xxxxx
```

---

## Core Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Provider name for logging
    fn name(&self) -> &str;

    /// Model name for logging
    fn model(&self) -> &str;

    /// Generate response from prompt
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;

    /// Health check for the backend
    async fn health_check(&self) -> anyhow::Result<()>;
}
```

---

## Provider Implementations

### OllamaBackend

```rust
pub struct OllamaBackend {
    endpoint: String,
    model: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl OllamaBackend {
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
                    .unwrap_or(120)
            ),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    fn name(&self) -> &str { "ollama" }
    fn model(&self) -> &str { &self.model }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/api/generate", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["response"].as_str().unwrap_or("").to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/api/tags", self.endpoint);
        self.client.get(&url).send().await?;
        Ok(())
    }
}
```

### LlamaCppBackend (OpenAI-compatible)

```rust
pub struct LlamaCppBackend {
    endpoint: String,
    model: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl LlamaCppBackend {
    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("LLM_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "codellama".to_string()),
            timeout: Duration::from_secs(
                env::var("LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(120)
            ),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for LlamaCppBackend {
    fn name(&self) -> &str { "llama-cpp" }
    fn model(&self) -> &str { &self.model }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        // llama.cpp server uses OpenAI-compatible /v1/completions
        let url = format!("{}/v1/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["text"].as_str().unwrap_or("").to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/health", self.endpoint);
        self.client.get(&url).send().await?;
        Ok(())
    }
}
```

### VllmBackend (OpenAI-compatible)

```rust
pub struct VllmBackend {
    endpoint: String,
    model: String,
    api_key: Option<String>,
    timeout: Duration,
    client: reqwest::Client,
}

impl VllmBackend {
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
                    .unwrap_or(120)
            ),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for VllmBackend {
    fn name(&self) -> &str { "vllm" }
    fn model(&self) -> &str { &self.model }

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
        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["text"].as_str().unwrap_or("").to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/health", self.endpoint);
        self.client.get(&url).send().await?;
        Ok(())
    }
}
```

### GroqBackend (Remote - Testing Only)

```rust
/// Groq API - Fast inference, free tier available
/// Use for development/testing when no local GPU
pub struct GroqBackend {
    endpoint: String,
    model: String,
    api_key: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl GroqBackend {
    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("LLM_ENDPOINT")
                .unwrap_or_else(|_| "https://api.groq.com/openai/v1".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string()),
            api_key: env::var("LLM_API_KEY")
                .expect("LLM_API_KEY required for Groq"),
            timeout: Duration::from_secs(
                env::var("LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60)
            ),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for GroqBackend {
    fn name(&self) -> &str { "groq" }
    fn model(&self) -> &str { &self.model }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let response = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        // Groq doesn't have a health endpoint, just try to list models
        let url = format!("{}/models", self.endpoint);
        self.client.get(&url).bearer_auth(&self.api_key).send().await?;
        Ok(())
    }
}
```

### OpenAIBackend (Remote - Testing Only)

```rust
/// OpenAI API - GPT-4o, GPT-4o-mini
pub struct OpenAIBackend {
    endpoint: String,
    model: String,
    api_key: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl OpenAIBackend {
    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("LLM_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            api_key: env::var("LLM_API_KEY")
                .expect("LLM_API_KEY required for OpenAI"),
            timeout: Duration::from_secs(
                env::var("LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(120)
            ),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for OpenAIBackend {
    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.model }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let response = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        let url = format!("{}/models", self.endpoint);
        self.client.get(&url).bearer_auth(&self.api_key).send().await?;
        Ok(())
    }
}
```

### AnthropicBackend (Remote - Testing Only)

```rust
/// Anthropic API - Claude 3.5 Sonnet
pub struct AnthropicBackend {
    endpoint: String,
    model: String,
    api_key: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl AnthropicBackend {
    pub fn from_env() -> Self {
        Self {
            endpoint: env::var("LLM_ENDPOINT")
                .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string()),
            model: env::var("LLM_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            api_key: env::var("LLM_API_KEY")
                .expect("LLM_API_KEY required for Anthropic"),
            timeout: Duration::from_secs(
                env::var("LLM_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(120)
            ),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for AnthropicBackend {
    fn name(&self) -> &str { "anthropic" }
    fn model(&self) -> &str { &self.model }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let url = format!("{}/messages", self.endpoint);
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": prompt}]
        });

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .timeout(self.timeout)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        // Anthropic: just verify API key format
        if self.api_key.starts_with("sk-ant-") {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid Anthropic API key format"))
        }
    }
}
```

---

## Factory Pattern

```rust
pub fn create_backend_from_env() -> Box<dyn LlmBackend> {
    let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

    match provider.as_str() {
        // On-premise providers (Production)
        "ollama" => Box::new(OllamaBackend::from_env()),
        "llama-cpp" => Box::new(LlamaCppBackend::from_env()),
        "vllm" => Box::new(VllmBackend::from_env()),

        // Remote providers (Development/Testing)
        "groq" => Box::new(GroqBackend::from_env()),
        "openai" => Box::new(OpenAIBackend::from_env()),
        "anthropic" => Box::new(AnthropicBackend::from_env()),

        _ => {
            tracing::warn!("Unknown LLM provider '{}', falling back to ollama", provider);
            Box::new(OllamaBackend::from_env())
        }
    }
}
```

> **Warning**: Remote providers (groq, openai, anthropic) send data to external servers.
> Use **only for development/testing**. Production must use on-premise providers.

---

## Usage in Service

```rust
pub async fn generate(req: GenerateRequest) -> Result<GenerateResponse> {
    let llm = create_backend_from_env();

    // Health check before generating
    llm.health_check().await
        .map_err(|e| Error::string(&format!("LLM server not available: {}", e)))?;

    // Generate
    let raw = llm.generate(&prompt).await?;

    // Log (without exposing LLM details)
    tracing::info!(
        "Generated response using {} (model: {})",
        llm.name(),
        llm.model()
    );

    // Parse and validate...
}
```

---

## Future: Admin Panel Configuration

When admin panel is implemented, configuration will be stored in database:

```sql
CREATE TABLE llm_configs (
    id UUID PRIMARY KEY,
    provider_type VARCHAR(50) NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    api_endpoint VARCHAR(500) NOT NULL,
    encrypted_api_key TEXT,
    is_default BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

The factory will then prioritize:
1. Database configuration (if exists)
2. Environment variables (fallback)

---

## Key Principles

### ❌ NEVER expose to plugin/API
- Provider type (ollama, llama-cpp, vllm)
- Model name
- API endpoint
- Temperature, max_tokens

### ✅ API response includes only
- Generator version (xframe5-ui-v1)
- Timestamp
- Warnings (business-level only)

---

**Last Updated**: 2025-12-28
