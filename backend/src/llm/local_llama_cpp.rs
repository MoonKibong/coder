//! Local LLM Backend using llama.cpp native Rust bindings
//!
//! This backend runs GGUF models directly in-process without requiring
//! a separate llama-server. Provides better performance and simpler deployment.
//!
//! Enable with: cargo build --features local-llm

use super::LlmBackend;
use async_trait::async_trait;
use std::env;
use std::path::PathBuf;

#[cfg(feature = "local-llm")]
use std::sync::Arc;

#[cfg(feature = "local-llm")]
use tracing::{debug, info};

#[cfg(feature = "local-llm")]
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
    token::LlamaToken,
};

#[cfg(feature = "local-llm")]
use std::num::NonZeroU32;

#[cfg(feature = "local-llm")]
use std::sync::{Mutex, OnceLock};

/// Global backend instance - llama.cpp backend can only be initialized once per process
#[cfg(feature = "local-llm")]
static LLAMA_BACKEND: OnceLock<Result<LlamaBackend, String>> = OnceLock::new();

/// Initialize or get the global llama backend
#[cfg(feature = "local-llm")]
fn get_or_init_backend() -> Result<&'static LlamaBackend, String> {
    let result = LLAMA_BACKEND.get_or_init(|| {
        info!("Initializing global llama.cpp backend");
        LlamaBackend::init().map_err(|e| format!("Failed to initialize llama backend: {}", e))
    });

    match result {
        Ok(backend) => Ok(backend),
        Err(e) => Err(e.clone()),
    }
}

/// Local LLM Backend using native llama.cpp bindings
///
/// Unlike `LlamaCppBackend` which requires a separate llama-server,
/// this backend runs inference directly in-process using GGUF models.
pub struct LocalLlamaCppBackend {
    model_path: PathBuf,
    #[allow(dead_code)]
    n_ctx: u32,
    #[allow(dead_code)]
    n_threads: u32,
    #[allow(dead_code)]
    max_tokens: u32,
    #[allow(dead_code)]
    temperature: f32,
    #[cfg(feature = "local-llm")]
    model: Arc<Mutex<Option<LlamaModel>>>,
}

// LlamaModel is Send but not Sync, we handle thread safety via Mutex
#[cfg(feature = "local-llm")]
unsafe impl Send for LocalLlamaCppBackend {}
#[cfg(feature = "local-llm")]
unsafe impl Sync for LocalLlamaCppBackend {}

impl LocalLlamaCppBackend {
    /// Create a new LocalLlamaCppBackend
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            n_ctx: 4096,
            n_threads: 4,
            max_tokens: 4096,
            temperature: 0.7,
            #[cfg(feature = "local-llm")]
            model: Arc::new(Mutex::new(None)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        model_path: PathBuf,
        n_ctx: u32,
        n_threads: u32,
        max_tokens: u32,
        temperature: f32,
    ) -> Self {
        Self {
            model_path,
            n_ctx,
            n_threads,
            max_tokens,
            temperature,
            #[cfg(feature = "local-llm")]
            model: Arc::new(Mutex::new(None)),
        }
    }

    /// Create from environment variables
    ///
    /// Environment variables:
    /// - LLM_MODEL_PATH: Path to GGUF model file (required)
    /// - LLM_CONTEXT_SIZE: Context window size (default: 4096)
    /// - LLM_THREADS: Number of CPU threads (default: 4)
    /// - LLM_MAX_TOKENS: Max tokens to generate (default: 4096)
    /// - LLM_TEMPERATURE: Sampling temperature (default: 0.7)
    pub fn from_env() -> Self {
        let model_path = PathBuf::from(
            env::var("LLM_MODEL_PATH").unwrap_or_else(|_| "llm-models/codellama.gguf".to_string()),
        );

        let n_ctx = env::var("LLM_CONTEXT_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let n_threads = env::var("LLM_THREADS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4);

        let max_tokens = env::var("LLM_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);

        let temperature = env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7);

        Self {
            model_path,
            n_ctx,
            n_threads,
            max_tokens,
            temperature,
            #[cfg(feature = "local-llm")]
            model: Arc::new(Mutex::new(None)),
        }
    }

    /// Get model file name
    pub fn model_name(&self) -> &str {
        self.model_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Check if model file exists
    pub fn model_exists(&self) -> bool {
        self.model_path.exists()
    }

    /// Load the model (lazy loading, blocking)
    #[cfg(feature = "local-llm")]
    fn ensure_loaded_sync(&self) -> anyhow::Result<()> {
        // Check if already loaded
        {
            let model_guard = self
                .model
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire model lock: {}", e))?;
            if model_guard.is_some() {
                return Ok(());
            }
        }

        if !self.model_path.exists() {
            anyhow::bail!("Model file not found: {:?}", self.model_path);
        }

        info!("Loading local model: {:?}", self.model_path);
        info!(
            "Config: n_ctx={}, n_threads={}, max_tokens={}, temperature={}",
            self.n_ctx, self.n_threads, self.max_tokens, self.temperature
        );

        // Get or initialize the global llama backend
        let backend =
            get_or_init_backend().map_err(|e| anyhow::anyhow!("Backend init failed: {}", e))?;

        // Set up model parameters
        let model_params = LlamaModelParams::default();

        // Load the model from file
        let model = LlamaModel::load_from_file(backend, &self.model_path, &model_params)
            .map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

        // Store the model
        {
            let mut model_guard = self
                .model
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire model lock: {}", e))?;
            *model_guard = Some(model);
        }

        info!("Local model loaded successfully");
        Ok(())
    }

    /// Generate text using the local model (blocking)
    #[cfg(feature = "local-llm")]
    fn generate_sync(&self, prompt: &str) -> anyhow::Result<String> {
        self.ensure_loaded_sync()?;

        debug!(
            "Generating with local model: max_tokens={}, temperature={}",
            self.max_tokens, self.temperature
        );
        debug!("Prompt length: {} chars", prompt.len());

        // Get the global backend
        let backend =
            get_or_init_backend().map_err(|e| anyhow::anyhow!("Backend init failed: {}", e))?;

        let model_guard = self
            .model
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire model lock: {}", e))?;

        let model = model_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model not loaded"))?;

        // Create context parameters
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(self.n_ctx))
            .with_n_threads(self.n_threads as i32)
            .with_n_threads_batch(self.n_threads as i32);

        // Create a new context for this generation
        let mut ctx = model
            .new_context(backend, ctx_params)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {}", e))?;

        // Tokenize the prompt
        let tokens_list = model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| anyhow::anyhow!("Failed to tokenize prompt: {}", e))?;

        let prompt_token_count = tokens_list.len();
        debug!("Tokenized prompt: {} tokens", prompt_token_count);

        // Check if prompt fits in context
        let n_ctx = ctx.n_ctx() as usize;
        if prompt_token_count >= n_ctx {
            anyhow::bail!(
                "Prompt too long: {} tokens, context size: {}",
                prompt_token_count,
                n_ctx
            );
        }

        // Cap max_tokens to available context space
        let available_tokens = n_ctx - prompt_token_count;
        let max_tokens = (self.max_tokens as usize).min(available_tokens) as u32;

        // Phase 1: Prompt Processing
        let mut prompt_batch = LlamaBatch::new(prompt_token_count, 1);

        let last_idx = prompt_token_count - 1;
        for (i, token) in tokens_list.iter().enumerate() {
            let is_last = i == last_idx;
            prompt_batch
                .add(*token, i as i32, &[0], is_last)
                .map_err(|e| anyhow::anyhow!("Failed to add token to batch: {}", e))?;
        }

        ctx.decode(&mut prompt_batch)
            .map_err(|e| anyhow::anyhow!("Failed to decode prompt: {}", e))?;

        drop(prompt_batch);

        // Phase 2: Token Generation
        let mut gen_batch = LlamaBatch::new(1, 1);

        // Set up sampler
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::dist(1234), // Fixed seed for reproducibility
            LlamaSampler::greedy(),
        ]);

        // Generate tokens
        let mut output_tokens: Vec<LlamaToken> = Vec::new();
        let mut n_cur = prompt_token_count as i32;
        let max_pos = prompt_token_count as i32 + max_tokens as i32;
        let mut last_batch_size = prompt_token_count;

        while n_cur < max_pos {
            let new_token_id = sampler.sample(&ctx, (last_batch_size - 1) as i32);
            sampler.accept(new_token_id);

            if model.is_eog_token(new_token_id) {
                debug!("End of generation token reached");
                break;
            }

            output_tokens.push(new_token_id);

            gen_batch.clear();
            gen_batch
                .add(new_token_id, n_cur, &[0], true)
                .map_err(|e| anyhow::anyhow!("Failed to add token to batch: {}", e))?;

            n_cur += 1;
            last_batch_size = 1;

            ctx.decode(&mut gen_batch)
                .map_err(|e| anyhow::anyhow!("Failed to decode token: {}", e))?;
        }

        debug!("Generated {} tokens", output_tokens.len());

        // Convert tokens to string
        let mut output_bytes: Vec<u8> = Vec::new();
        for token in &output_tokens {
            let token_bytes = model
                .token_to_bytes(*token, Special::Plaintext)
                .map_err(|e| anyhow::anyhow!("Failed to convert token to bytes: {}", e))?;
            output_bytes.extend_from_slice(&token_bytes);
        }

        let output = String::from_utf8_lossy(&output_bytes).into_owned();
        debug!("Generated output: {} chars", output.len());

        Ok(output)
    }
}

#[async_trait]
impl LlmBackend for LocalLlamaCppBackend {
    fn name(&self) -> &str {
        "local-llama-cpp"
    }

    fn model(&self) -> &str {
        self.model_name()
    }

    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        #[cfg(feature = "local-llm")]
        {
            // Clone Arc references for the blocking task
            let model = self.model.clone();
            let model_path = self.model_path.clone();
            let n_ctx = self.n_ctx;
            let n_threads = self.n_threads;
            let max_tokens = self.max_tokens;
            let temperature = self.temperature;
            let prompt = prompt.to_string();

            // Run blocking inference in a separate thread
            let result = tokio::task::spawn_blocking(move || {
                let temp_provider = LocalLlamaCppBackend {
                    model_path,
                    n_ctx,
                    n_threads,
                    max_tokens,
                    temperature,
                    model,
                };
                temp_provider.generate_sync(&prompt)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

            Ok(result)
        }

        #[cfg(not(feature = "local-llm"))]
        {
            let _ = prompt;
            anyhow::bail!(
                "Local LLM feature not enabled. Build with: cargo build --features local-llm"
            )
        }
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        #[cfg(feature = "local-llm")]
        {
            if self.model_path.exists() {
                Ok(())
            } else {
                anyhow::bail!("Model file not found: {:?}", self.model_path)
            }
        }

        #[cfg(not(feature = "local-llm"))]
        {
            anyhow::bail!(
                "Local LLM feature not enabled. Build with: cargo build --features local-llm"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = LocalLlamaCppBackend::new(PathBuf::from("test.gguf"));
        assert_eq!(provider.name(), "local-llama-cpp");
    }

    #[test]
    fn test_model_name() {
        let provider = LocalLlamaCppBackend::new(PathBuf::from("/path/to/model.gguf"));
        assert_eq!(provider.model(), "model.gguf");
    }

    #[test]
    fn test_with_config() {
        let provider =
            LocalLlamaCppBackend::with_config(PathBuf::from("test.gguf"), 8192, 8, 2048, 0.5);
        assert_eq!(provider.n_ctx, 8192);
        assert_eq!(provider.n_threads, 8);
        assert_eq!(provider.max_tokens, 2048);
        assert!((provider.temperature - 0.5).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_health_check_no_file() {
        let provider = LocalLlamaCppBackend::new(PathBuf::from("nonexistent.gguf"));
        let result = provider.health_check().await;
        assert!(result.is_err());
    }
}
