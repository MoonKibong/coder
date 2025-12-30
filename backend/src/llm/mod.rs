mod ollama;
mod llama_cpp;
mod local_llama_cpp;
mod vllm;
mod groq;
mod openai;
mod anthropic;
mod mock;

pub use ollama::{OllamaBackend, OllamaModel, OllamaModelDetails};
pub use llama_cpp::LlamaCppBackend;
pub use local_llama_cpp::LocalLlamaCppBackend;
pub use vllm::VllmBackend;
pub use groq::GroqBackend;
pub use openai::OpenAIBackend;
pub use anthropic::AnthropicBackend;
pub use mock::{MockLlmBackend, MockResponse};

use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::env;
use std::path::PathBuf;

use crate::models::_entities::llm_configs;

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

/// Create LLM backend from database configuration, falling back to environment variables.
///
/// This function:
/// 1. Queries the database for an active LLM config (is_active = true)
/// 2. If found, creates the backend from database settings
/// 3. If not found, falls back to create_backend_from_env()
///
/// This allows runtime configuration changes via the admin panel without server restart.
pub async fn create_backend_from_db_or_env(db: &DatabaseConnection) -> Box<dyn LlmBackend> {
    // Try to get active config from database
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
    let timeout_seconds = 120u64; // Default timeout

    match config.provider.as_str() {
        "ollama" => Box::new(OllamaBackend::new(
            config.endpoint_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string()),
            config.model_name.clone(),
            timeout_seconds,
        )),
        "llama-cpp" => Box::new(LlamaCppBackend::new(
            config.endpoint_url.clone().unwrap_or_else(|| "http://localhost:8080".to_string()),
            config.model_name.clone(),
            timeout_seconds,
        )),
        "local-llama-cpp" => {
            // Use model_path from config, or fall back to model_name in default directory
            let model_path = config.model_path.clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("llm-models").join(&config.model_name)
                });

            let n_ctx = config.n_ctx.unwrap_or(4096) as u32;
            let n_threads = config.n_threads.unwrap_or(4) as u32;
            let max_tokens = config.max_tokens.unwrap_or(4096) as u32;
            let temperature = config.temperature.unwrap_or(0.7);

            Box::new(LocalLlamaCppBackend::with_config(
                model_path,
                n_ctx,
                n_threads,
                max_tokens,
                temperature,
            ))
        },
        "vllm" => Box::new(VllmBackend::new(
            config.endpoint_url.clone().unwrap_or_else(|| "http://localhost:8000".to_string()),
            config.model_name.clone(),
            config.api_key.clone(), // Optional<String>
            timeout_seconds,
        )),
        "groq" => Box::new(GroqBackend::new(
            config.endpoint_url.clone().unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string()),
            config.model_name.clone(),
            config.api_key.clone().unwrap_or_default(),
            timeout_seconds,
        )),
        "openai" => Box::new(OpenAIBackend::new(
            config.endpoint_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            config.model_name.clone(),
            config.api_key.clone().unwrap_or_default(),
            timeout_seconds,
        )),
        "anthropic" => Box::new(AnthropicBackend::new(
            config.endpoint_url.clone().unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            config.model_name.clone(),
            config.api_key.clone().unwrap_or_default(),
            timeout_seconds,
        )),
        _ => {
            tracing::warn!(
                "Unknown provider '{}' in database config, falling back to ollama",
                config.provider
            );
            Box::new(OllamaBackend::new(
                config.endpoint_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string()),
                config.model_name.clone(),
                timeout_seconds,
            ))
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
