mod ollama;
mod llama_cpp;
mod local_llama_cpp;
mod vllm;
mod groq;
mod openai;
mod anthropic;
mod mock;

pub use ollama::OllamaBackend;
pub use llama_cpp::LlamaCppBackend;
pub use local_llama_cpp::LocalLlamaCppBackend;
pub use vllm::VllmBackend;
pub use groq::GroqBackend;
pub use openai::OpenAIBackend;
pub use anthropic::AnthropicBackend;
pub use mock::{MockLlmBackend, MockResponse};

use async_trait::async_trait;
use std::env;

/// Core trait for LLM backends.
/// All implementations must be Send + Sync for async contexts.
///
/// CRITICAL: LLM details (model name, provider, etc.) must NEVER be exposed to API/plugin.
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Provider name for internal logging only
    fn name(&self) -> &str;

    /// Model name for internal logging only
    fn model(&self) -> &str;

    /// Generate response from prompt
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;

    /// Health check for the backend
    async fn health_check(&self) -> anyhow::Result<()>;
}

/// Create LLM backend from environment variables.
///
/// Environment variables:
/// - LLM_PROVIDER: ollama | llama-cpp | local-llama-cpp | vllm | groq | openai | anthropic
/// - LLM_ENDPOINT: Server URL (for remote providers)
/// - LLM_MODEL: Model name (for remote providers)
/// - LLM_API_KEY: API key (required for remote providers)
/// - LLM_TIMEOUT_SECONDS: Request timeout (default: 120)
///
/// For local-llama-cpp provider (native llama.cpp bindings):
/// - LLM_MODEL_PATH: Path to GGUF model file
/// - LLM_CONTEXT_SIZE: Context window size (default: 4096)
/// - LLM_THREADS: Number of CPU threads (default: 4)
/// - LLM_MAX_TOKENS: Max tokens to generate (default: 4096)
/// - LLM_TEMPERATURE: Sampling temperature (default: 0.7)
pub fn create_backend_from_env() -> Box<dyn LlmBackend> {
    let provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

    match provider.as_str() {
        // On-premise providers (Production)
        "ollama" => Box::new(OllamaBackend::from_env()),
        "llama-cpp" => Box::new(LlamaCppBackend::from_env()),
        "local-llama-cpp" => Box::new(LocalLlamaCppBackend::from_env()),
        "vllm" => Box::new(VllmBackend::from_env()),

        // Remote providers (Development/Testing only)
        "groq" => Box::new(GroqBackend::from_env()),
        "openai" => Box::new(OpenAIBackend::from_env()),
        "anthropic" => Box::new(AnthropicBackend::from_env()),

        _ => {
            tracing::warn!("Unknown LLM provider '{}', falling back to ollama", provider);
            Box::new(OllamaBackend::from_env())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_provider_is_ollama() {
        // Clear any existing env var
        env::remove_var("LLM_PROVIDER");

        let backend = create_backend_from_env();
        assert_eq!(backend.name(), "ollama");
    }
}
