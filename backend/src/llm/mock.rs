use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::LlmBackend;

/// Mock LLM backend for testing purposes.
///
/// This backend returns predefined responses without making actual LLM calls.
/// Useful for:
/// - Unit tests that need deterministic outputs
/// - Integration tests without LLM server dependency
/// - Testing retry logic and error handling
#[derive(Clone)]
pub struct MockLlmBackend {
    /// Responses to return (cycles through if multiple calls)
    responses: Vec<MockResponse>,
    /// Current response index
    call_count: Arc<AtomicUsize>,
    /// Whether health check should succeed
    healthy: bool,
}

/// A mock response configuration
#[derive(Clone)]
pub enum MockResponse {
    /// Return a successful response
    Success(String),
    /// Return an error
    Error(String),
    /// Return a valid xFrame5 output
    XFrame5Output { xml: String, js: String },
}

impl MockLlmBackend {
    /// Create a new mock backend with default xFrame5 output
    pub fn new() -> Self {
        Self {
            responses: vec![MockResponse::XFrame5Output {
                xml: Self::default_xml(),
                js: Self::default_js(),
            }],
            call_count: Arc::new(AtomicUsize::new(0)),
            healthy: true,
        }
    }

    /// Create a mock backend with custom responses
    pub fn with_responses(responses: Vec<MockResponse>) -> Self {
        Self {
            responses,
            call_count: Arc::new(AtomicUsize::new(0)),
            healthy: true,
        }
    }

    /// Create a mock backend that always returns an error
    pub fn failing(error_message: &str) -> Self {
        Self {
            responses: vec![MockResponse::Error(error_message.to_string())],
            call_count: Arc::new(AtomicUsize::new(0)),
            healthy: true,
        }
    }

    /// Create a mock backend with unhealthy status
    pub fn unhealthy() -> Self {
        Self {
            responses: vec![],
            call_count: Arc::new(AtomicUsize::new(0)),
            healthy: false,
        }
    }

    /// Create a mock that fails first then succeeds (for retry testing)
    pub fn fail_then_succeed() -> Self {
        Self {
            responses: vec![
                MockResponse::Error("First attempt failed".to_string()),
                MockResponse::XFrame5Output {
                    xml: Self::default_xml(),
                    js: Self::default_js(),
                },
            ],
            call_count: Arc::new(AtomicUsize::new(0)),
            healthy: true,
        }
    }

    /// Get the number of generate() calls made
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    fn default_xml() -> String {
        r#"<Dataset id="ds_member">
  <Column name="id" type="STRING" size="20" />
  <Column name="name" type="STRING" size="100" />
  <Column name="email" type="STRING" size="255" />
</Dataset>

<Grid id="grid_member" dataset="ds_member">
  <Column name="id" header="ID" width="80" />
  <Column name="name" header="이름" width="150" />
  <Column name="email" header="이메일" width="200" />
</Grid>"#
            .to_string()
    }

    fn default_js() -> String {
        r#"/**
 * Member List Screen
 */

this.fn_init = function() {
    // Initialize screen
};

this.fn_search = function() {
    // TODO: Implement search API call
    var ds = this.getDataset("ds_member");
    ds.clearData();
};

this.fn_save = function() {
    // TODO: Implement save API call
    var ds = this.getDataset("ds_member");
    var changedData = ds.getChangedData();
};

this.fn_delete = function() {
    // TODO: Implement delete API call
    var ds = this.getDataset("ds_member");
    var selectedRow = ds.getSelectedRowIndex();
};"#
            .to_string()
    }
}

impl Default for MockLlmBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmBackend for MockLlmBackend {
    fn name(&self) -> &str {
        "mock"
    }

    fn model(&self) -> &str {
        "mock-model"
    }

    async fn generate(&self, _prompt: &str) -> anyhow::Result<String> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        let response_idx = idx % self.responses.len();

        match &self.responses[response_idx] {
            MockResponse::Success(text) => Ok(text.clone()),
            MockResponse::Error(msg) => Err(anyhow::anyhow!("{}", msg)),
            MockResponse::XFrame5Output { xml, js } => {
                Ok(format!("--- XML ---\n{}\n\n--- JS ---\n{}", xml, js))
            }
        }
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        if self.healthy {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Mock LLM is unhealthy"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_default_response() {
        let mock = MockLlmBackend::new();
        let result = mock.generate("test prompt").await.unwrap();

        assert!(result.contains("--- XML ---"));
        assert!(result.contains("--- JS ---"));
        assert!(result.contains("ds_member"));
    }

    #[tokio::test]
    async fn test_mock_custom_response() {
        let mock = MockLlmBackend::with_responses(vec![MockResponse::Success(
            "Custom response".to_string(),
        )]);

        let result = mock.generate("test").await.unwrap();
        assert_eq!(result, "Custom response");
    }

    #[tokio::test]
    async fn test_mock_error_response() {
        let mock = MockLlmBackend::failing("Test error");
        let result = mock.generate("test").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Test error"));
    }

    #[tokio::test]
    async fn test_mock_unhealthy() {
        let mock = MockLlmBackend::unhealthy();
        let result = mock.health_check().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_call_count() {
        let mock = MockLlmBackend::new();

        assert_eq!(mock.call_count(), 0);
        mock.generate("test").await.unwrap();
        assert_eq!(mock.call_count(), 1);
        mock.generate("test").await.unwrap();
        assert_eq!(mock.call_count(), 2);
    }

    #[tokio::test]
    async fn test_mock_cycles_responses() {
        let mock = MockLlmBackend::with_responses(vec![
            MockResponse::Success("First".to_string()),
            MockResponse::Success("Second".to_string()),
        ]);

        assert_eq!(mock.generate("").await.unwrap(), "First");
        assert_eq!(mock.generate("").await.unwrap(), "Second");
        assert_eq!(mock.generate("").await.unwrap(), "First"); // Cycles back
    }

    #[tokio::test]
    async fn test_mock_fail_then_succeed() {
        let mock = MockLlmBackend::fail_then_succeed();

        // First call fails
        let first = mock.generate("test").await;
        assert!(first.is_err());

        // Second call succeeds
        let second = mock.generate("test").await;
        assert!(second.is_ok());
        assert!(second.unwrap().contains("--- XML ---"));
    }
}
