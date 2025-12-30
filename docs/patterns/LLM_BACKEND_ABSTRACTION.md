# LLM Backend Abstraction Pattern

## Purpose
LLM 런타임을 완전히 추상화하여 모델 교체 시 플러그인/API 변경 없이 운영

## Supported Providers

### On-Premise (Production)
| Provider | Use Case | API Key Required |
|----------|----------|------------------|
| `ollama` | Self-hosted Ollama server | No |
| `llama-cpp` | llama.cpp server (OpenAI-compatible) | No |
| `local-llm` | Embedded llama.cpp (no server needed) | No |
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

# Local LLM (embedded llama.cpp - requires --features local-llm)
LLM_MODEL_PATH=./llm-models/your-model.gguf
LLM_CONTEXT_SIZE=4096
LLM_THREADS=4
LLM_MAX_TOKENS=4096
LLM_TEMPERATURE=0.7

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

### LocalLlamaCppBackend (Embedded - No Server)

> **Note**: Requires `--features local-llm` at build time.

```rust
/// Embedded llama.cpp backend - runs GGUF models directly in-process
/// No separate server required, simpler deployment
pub struct LocalLlamaCppBackend {
    model_path: PathBuf,
    n_ctx: u32,
    n_threads: u32,
    max_tokens: u32,
    temperature: f32,
}

impl LocalLlamaCppBackend {
    pub fn from_env() -> Self {
        Self {
            model_path: PathBuf::from(
                env::var("LLM_MODEL_PATH")
                    .unwrap_or_else(|_| "llm-models/codellama.gguf".to_string())
            ),
            n_ctx: env::var("LLM_CONTEXT_SIZE")
                .ok().and_then(|s| s.parse().ok()).unwrap_or(4096),
            n_threads: env::var("LLM_THREADS")
                .ok().and_then(|s| s.parse().ok()).unwrap_or(4),
            max_tokens: env::var("LLM_MAX_TOKENS")
                .ok().and_then(|s| s.parse().ok()).unwrap_or(4096),
            temperature: env::var("LLM_TEMPERATURE")
                .ok().and_then(|s| s.parse().ok()).unwrap_or(0.7),
        }
    }
}

#[async_trait]
impl LlmBackend for LocalLlamaCppBackend {
    fn name(&self) -> &str { "local-llama-cpp" }
    fn model(&self) -> &str { self.model_name() }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        // Runs inference directly using llama-cpp-2 crate
        // Model is loaded lazily on first request
        // Uses spawn_blocking for async compatibility
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        // Checks if model file exists
    }
}
```

**Build & Run**:
```bash
# Build with local-llm feature
cargo build --features local-llm

# Run with model path
export LLM_MODEL_PATH=./llm-models/your-model.gguf
cargo loco start --features local-llm
```

**Model Directory**: Place GGUF files in `backend/llm-models/` (git-ignored).

---

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
pub async fn generate(db: &DatabaseConnection, req: GenerateRequest) -> Result<GenerateResponse> {
    // DB config takes priority, falls back to env
    let llm = create_backend_from_db_or_env(db).await;

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

## Database Configuration (Admin Panel)

LLM configuration is stored in the database for runtime updates without server restart:

```sql
CREATE TABLE llm_configs (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    endpoint_url VARCHAR(500) NOT NULL,
    api_key TEXT,
    is_active BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

### Configuration Priority

The `create_backend_from_db_or_env()` function implements this priority:

1. **Database configuration** (if `is_active = true` exists)
2. **Environment variables** (fallback)

### Factory Functions

```rust
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

/// Create LLM backend from database configuration, falling back to environment variables.
pub async fn create_backend_from_db_or_env(db: &DatabaseConnection) -> Box<dyn LlmBackend> {
    match get_active_llm_config(db).await {
        Some(config) => {
            tracing::info!(
                "Using LLM config from database: {} ({}/{})",
                config.name,
                config.provider,
                config.model_name
            );
            create_backend_from_config(&config)
        }
        None => {
            tracing::info!("No active LLM config in database, using environment variables");
            create_backend_from_env()
        }
    }
}

/// Get the active LLM configuration from database
async fn get_active_llm_config(db: &DatabaseConnection) -> Option<llm_configs::Model> {
    llm_configs::Entity::find()
        .filter(llm_configs::Column::IsActive.eq(true))
        .one(db)
        .await
        .ok()
        .flatten()
}

/// Create LLM backend from database configuration
fn create_backend_from_config(config: &llm_configs::Model) -> Box<dyn LlmBackend> {
    let timeout_seconds = 120u64;

    match config.provider.as_str() {
        "ollama" => Box::new(OllamaBackend::new(
            config.endpoint_url.clone(),
            config.model_name.clone(),
            timeout_seconds,
        )),
        "vllm" => Box::new(VllmBackend::new(
            config.endpoint_url.clone(),
            config.model_name.clone(),
            config.api_key.clone(),
            timeout_seconds,
        )),
        "groq" => Box::new(GroqBackend::new(
            config.endpoint_url.clone(),
            config.model_name.clone(),
            config.api_key.clone().unwrap_or_default(),
            timeout_seconds,
        )),
        // ... other providers
        _ => Box::new(OllamaBackend::new(
            config.endpoint_url.clone(),
            config.model_name.clone(),
            timeout_seconds,
        )),
    }
}
```

### Admin Panel

Configure LLM settings at runtime via `/admin/llm-configs`:

- Create/edit/delete LLM configurations
- Toggle `is_active` to switch between configurations
- Only ONE configuration can be active at a time
- Changes take effect immediately (no server restart)

### Benefits

1. **Runtime configuration** - No server restart needed
2. **Customer-specific settings** - Different LLM for each deployment
3. **Easy switching** - Toggle between providers via admin panel
4. **Fallback safety** - Environment variables as backup

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

**Last Updated**: 2025-12-30
