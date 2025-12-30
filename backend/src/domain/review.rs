use serde::{Deserialize, Serialize};

/// Code review input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInput {
    /// Code to review
    pub code: String,

    /// File type (xml, javascript, java)
    #[serde(default)]
    pub file_type: Option<String>,

    /// Optional context about what the code does
    pub context: Option<String>,
}

impl ReviewInput {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            file_type: None,
            context: None,
        }
    }

    pub fn with_file_type(mut self, file_type: impl Into<String>) -> Self {
        self.file_type = Some(file_type.into());
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Detect file type from code content if not specified
    pub fn detect_file_type(&self) -> String {
        if let Some(ref ft) = self.file_type {
            return ft.clone();
        }

        let code = self.code.trim();
        if code.starts_with("<?xml") || code.starts_with("<screen") || code.starts_with("<Dataset") {
            "xml".to_string()
        } else if code.contains("public class") || code.contains("public interface") || code.contains("@Controller") {
            "java".to_string()
        } else {
            "javascript".to_string()
        }
    }
}

/// Review options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewOptions {
    /// Output language (default: "ko")
    #[serde(default = "default_language")]
    pub language: String,

    /// Review focus areas
    #[serde(default = "default_review_focus")]
    pub review_focus: Vec<String>,

    /// Company ID for custom rules
    pub company_id: Option<String>,
}

fn default_language() -> String {
    "ko".to_string()
}

fn default_review_focus() -> Vec<String> {
    vec![
        "syntax".to_string(),
        "patterns".to_string(),
        "naming".to_string(),
        "performance".to_string(),
    ]
}

/// Code review request from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    /// Product identifier (e.g., "xframe5-ui", "spring-backend")
    pub product: String,

    /// Review input
    pub input: ReviewInput,

    /// Review options
    #[serde(default)]
    pub options: ReviewOptions,

    /// Request context
    #[serde(default)]
    pub context: ReviewContext,
}

/// Review context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewContext {
    /// Project identifier
    pub project: Option<String>,

    /// File name being reviewed
    pub file_name: Option<String>,
}

/// Code review response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResponse {
    /// Status of review
    pub status: ReviewStatus,

    /// Review result
    pub review: Option<ReviewResult>,

    /// Error message (if status is error)
    pub error: Option<String>,

    /// Response metadata
    pub meta: ReviewMeta,
}

/// Review status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Success,
    Error,
}

/// Review result with issues and improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// Overall assessment summary
    pub summary: String,

    /// List of issues found
    #[serde(default)]
    pub issues: Vec<ReviewIssue>,

    /// Quality score
    pub score: Option<ReviewScore>,

    /// Improvement suggestions
    #[serde(default)]
    pub improvements: Vec<String>,
}

/// Individual issue found during review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue category
    pub category: IssueCategory,

    /// Line number (0 if not applicable)
    #[serde(default)]
    pub line: u32,

    /// Issue description
    pub message: String,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Suggestion,
}

/// Issue categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    Syntax,
    Pattern,
    Naming,
    Performance,
    Security,
    BestPractice,
}

/// Quality score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewScore {
    /// Overall score (0-100)
    pub overall: u8,

    /// Category scores
    #[serde(default)]
    pub categories: CategoryScores,
}

/// Scores by category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryScores {
    pub syntax: Option<u8>,
    pub patterns: Option<u8>,
    pub naming: Option<u8>,
    pub performance: Option<u8>,
    pub security: Option<u8>,
}

/// Response metadata (NO LLM details exposed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMeta {
    /// Generator version
    pub generator: String,

    /// Review timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Review time in milliseconds
    pub review_time_ms: u64,
}

impl ReviewMeta {
    pub fn new(generator: impl Into<String>, review_time_ms: u64) -> Self {
        Self {
            generator: generator.into(),
            timestamp: chrono::Utc::now(),
            review_time_ms,
        }
    }
}

impl ReviewResponse {
    pub fn success(review: ReviewResult, meta: ReviewMeta) -> Self {
        Self {
            status: ReviewStatus::Success,
            review: Some(review),
            error: None,
            meta,
        }
    }

    pub fn error(error: impl Into<String>, meta: ReviewMeta) -> Self {
        Self {
            status: ReviewStatus::Error,
            review: None,
            error: Some(error.into()),
            meta,
        }
    }
}
