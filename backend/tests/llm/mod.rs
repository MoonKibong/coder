use coder::llm::{
    create_backend_from_env, AnthropicBackend, GroqBackend, LlamaCppBackend, LlmBackend,
    MockLlmBackend, MockResponse, OllamaBackend, OpenAIBackend, VllmBackend,
};
use serial_test::serial;
use std::env;

fn clear_llm_env_vars() {
    env::remove_var("LLM_PROVIDER");
    env::remove_var("LLM_ENDPOINT");
    env::remove_var("LLM_MODEL");
    env::remove_var("LLM_API_KEY");
    env::remove_var("LLM_TIMEOUT_SECONDS");
}

#[test]
#[serial]
fn test_ollama_backend_from_env() {
    clear_llm_env_vars();

    let backend = OllamaBackend::from_env();
    assert_eq!(backend.name(), "ollama");
    assert_eq!(backend.model(), "codellama:13b");
}

#[test]
#[serial]
fn test_ollama_backend_custom_config() {
    clear_llm_env_vars();
    env::set_var("LLM_ENDPOINT", "http://custom:11434");
    env::set_var("LLM_MODEL", "llama2:7b");

    let backend = OllamaBackend::from_env();
    assert_eq!(backend.name(), "ollama");
    assert_eq!(backend.model(), "llama2:7b");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_llama_cpp_backend_from_env() {
    clear_llm_env_vars();

    let backend = LlamaCppBackend::from_env();
    assert_eq!(backend.name(), "llama-cpp");
    assert_eq!(backend.model(), "codellama");
}

#[test]
#[serial]
fn test_vllm_backend_from_env() {
    clear_llm_env_vars();

    let backend = VllmBackend::from_env();
    assert_eq!(backend.name(), "vllm");
    assert_eq!(backend.model(), "codellama/CodeLlama-13b-hf");
}

#[test]
#[serial]
fn test_factory_default_is_ollama() {
    clear_llm_env_vars();

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "ollama");
}

#[test]
#[serial]
fn test_factory_selects_llama_cpp() {
    clear_llm_env_vars();
    env::set_var("LLM_PROVIDER", "llama-cpp");

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "llama-cpp");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_factory_selects_vllm() {
    clear_llm_env_vars();
    env::set_var("LLM_PROVIDER", "vllm");

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "vllm");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_factory_unknown_provider_falls_back_to_ollama() {
    clear_llm_env_vars();
    env::set_var("LLM_PROVIDER", "unknown-provider");

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "ollama");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_groq_backend_with_api_key() {
    clear_llm_env_vars();
    env::set_var("LLM_API_KEY", "gsk_test_key");

    let backend = GroqBackend::from_env();
    assert_eq!(backend.name(), "groq");
    assert_eq!(backend.model(), "llama-3.3-70b-versatile");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_openai_backend_with_api_key() {
    clear_llm_env_vars();
    env::set_var("LLM_API_KEY", "sk-test_key");

    let backend = OpenAIBackend::from_env();
    assert_eq!(backend.name(), "openai");
    assert_eq!(backend.model(), "gpt-4o-mini");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_anthropic_backend_with_api_key() {
    clear_llm_env_vars();
    env::set_var("LLM_API_KEY", "sk-ant-test_key");

    let backend = AnthropicBackend::from_env();
    assert_eq!(backend.name(), "anthropic");
    assert_eq!(backend.model(), "claude-3-5-sonnet-20241022");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_factory_selects_groq() {
    clear_llm_env_vars();
    env::set_var("LLM_PROVIDER", "groq");
    env::set_var("LLM_API_KEY", "gsk_test");

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "groq");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_factory_selects_openai() {
    clear_llm_env_vars();
    env::set_var("LLM_PROVIDER", "openai");
    env::set_var("LLM_API_KEY", "sk-test");

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "openai");

    clear_llm_env_vars();
}

#[test]
#[serial]
fn test_factory_selects_anthropic() {
    clear_llm_env_vars();
    env::set_var("LLM_PROVIDER", "anthropic");
    env::set_var("LLM_API_KEY", "sk-ant-test");

    let backend = create_backend_from_env();
    assert_eq!(backend.name(), "anthropic");

    clear_llm_env_vars();
}

#[tokio::test]
#[serial]
async fn test_anthropic_health_check_valid_key_format() {
    clear_llm_env_vars();
    env::set_var("LLM_API_KEY", "sk-ant-valid-format");

    let backend = AnthropicBackend::from_env();
    let result = backend.health_check().await;
    assert!(result.is_ok());

    clear_llm_env_vars();
}

#[tokio::test]
#[serial]
async fn test_anthropic_health_check_invalid_key_format() {
    clear_llm_env_vars();
    env::set_var("LLM_API_KEY", "invalid-key-format");

    let backend = AnthropicBackend::from_env();
    let result = backend.health_check().await;
    assert!(result.is_err());

    clear_llm_env_vars();
}

// Mock LLM Backend Tests

#[test]
fn test_mock_backend_name_and_model() {
    let mock = MockLlmBackend::new();
    assert_eq!(mock.name(), "mock");
    assert_eq!(mock.model(), "mock-model");
}

#[tokio::test]
async fn test_mock_backend_default_output() {
    let mock = MockLlmBackend::new();
    let result = mock.generate("test prompt").await.unwrap();

    assert!(result.contains("--- XML ---"));
    assert!(result.contains("--- JS ---"));
    assert!(result.contains("ds_member"));
    assert!(result.contains("fn_search"));
}

#[tokio::test]
async fn test_mock_backend_custom_responses() {
    let mock = MockLlmBackend::with_responses(vec![
        MockResponse::Success("Response 1".to_string()),
        MockResponse::Success("Response 2".to_string()),
    ]);

    assert_eq!(mock.generate("").await.unwrap(), "Response 1");
    assert_eq!(mock.generate("").await.unwrap(), "Response 2");
    assert_eq!(mock.generate("").await.unwrap(), "Response 1"); // Cycles
}

#[tokio::test]
async fn test_mock_backend_error() {
    let mock = MockLlmBackend::failing("Test error message");
    let result = mock.generate("test").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Test error message"));
}

#[tokio::test]
async fn test_mock_backend_health_check() {
    let healthy = MockLlmBackend::new();
    assert!(healthy.health_check().await.is_ok());

    let unhealthy = MockLlmBackend::unhealthy();
    assert!(unhealthy.health_check().await.is_err());
}

#[tokio::test]
async fn test_mock_backend_call_count() {
    let mock = MockLlmBackend::new();
    assert_eq!(mock.call_count(), 0);

    mock.generate("first").await.unwrap();
    assert_eq!(mock.call_count(), 1);

    mock.generate("second").await.unwrap();
    assert_eq!(mock.call_count(), 2);

    mock.generate("third").await.unwrap();
    assert_eq!(mock.call_count(), 3);
}

#[tokio::test]
async fn test_mock_backend_fail_then_succeed() {
    let mock = MockLlmBackend::fail_then_succeed();

    // First call fails
    let first = mock.generate("attempt 1").await;
    assert!(first.is_err());

    // Second call succeeds
    let second = mock.generate("attempt 2").await;
    assert!(second.is_ok());
    assert!(second.unwrap().contains("--- XML ---"));
}
