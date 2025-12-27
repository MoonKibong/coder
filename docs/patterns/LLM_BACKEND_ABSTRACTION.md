# LLM Backend Abstraction Pattern

## 목적
LLM 런타임을 완전히 추상화하여 모델 교체 시 플러그인/API 변경 없이 운영

## Core Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Generate response from prompt
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;

    /// Health check for the backend
    async fn health_check(&self) -> anyhow::Result<bool>;
}
```

## Implementations

### OllamaBackend
```rust
pub struct OllamaBackend {
    endpoint: String,  // e.g., "http://localhost:11434"
    model: String,     // e.g., "codellama:13b"
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        // POST to /api/generate
    }
}
```

### LlamaCppBackend
```rust
pub struct LlamaCppBackend {
    endpoint: String,
    // llama.cpp server configuration
}
```

## Configuration

설정은 환경별 YAML 파일에서 관리:

```yaml
# config/development.yaml
llm:
  backend: "ollama"
  endpoint: "http://localhost:11434"
  model: "codellama:13b"
  timeout_seconds: 120

# config/production.yaml
llm:
  backend: "ollama"
  endpoint: "http://internal-gpu-server:11434"
  model: "codellama:34b"
  timeout_seconds: 180
```

## Factory Pattern

```rust
pub fn create_llm_backend(config: &LlmConfig) -> Box<dyn LlmBackend> {
    match config.backend.as_str() {
        "ollama" => Box::new(OllamaBackend::new(config)),
        "llama-cpp" => Box::new(LlamaCppBackend::new(config)),
        _ => panic!("Unknown LLM backend: {}", config.backend),
    }
}
```

## 핵심 원칙

### ❌ 절대 노출 금지
- Model name (codellama, mistral, etc.)
- Temperature, max_tokens
- System prompt content
- Backend type (ollama, llama-cpp)

### ✅ API 응답에 포함 가능
- Generator version (xframe5-ui-v1)
- Timestamp
- Warnings (business-level only)

## 테스트

```rust
#[tokio::test]
async fn test_model_swap_no_api_change() {
    // 1. Generate with ollama
    // 2. Swap to llama-cpp
    // 3. Verify API response structure identical
}
```

## 확장

### 향후 Backend 추가
1. `LlmBackend` trait 구현
2. Factory에 match arm 추가
3. Config에 설정 추가
4. **플러그인 변경 없음**
