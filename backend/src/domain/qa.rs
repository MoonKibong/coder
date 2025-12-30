use serde::{Deserialize, Serialize};

/// Q&A input - question to ask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAInput {
    /// The question to answer
    pub question: String,

    /// Optional additional context
    pub context: Option<String>,
}

impl QAInput {
    pub fn new(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Q&A options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QAOptions {
    /// Output language (default: "ko")
    #[serde(default = "default_language")]
    pub language: String,

    /// Include code examples in answer
    #[serde(default = "default_include_examples")]
    pub include_examples: bool,

    /// Maximum number of knowledge references to include
    #[serde(default = "default_max_references")]
    pub max_references: usize,
}

fn default_language() -> String {
    "ko".to_string()
}

fn default_include_examples() -> bool {
    true
}

fn default_max_references() -> usize {
    5
}

/// Q&A request from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QARequest {
    /// Product identifier (e.g., "xframe5", "spring")
    pub product: String,

    /// Q&A input
    pub input: QAInput,

    /// Q&A options
    #[serde(default)]
    pub options: QAOptions,
}

/// Q&A response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAResponse {
    /// Status of Q&A
    pub status: QAStatus,

    /// The answer
    pub answer: Option<QAAnswer>,

    /// Knowledge base references used
    #[serde(default)]
    pub references: Vec<KnowledgeReference>,

    /// Error message (if status is error)
    pub error: Option<String>,

    /// Response metadata
    pub meta: QAMeta,
}

/// Q&A status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QAStatus {
    Success,
    Error,
}

/// The answer with optional code examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAAnswer {
    /// The answer text (markdown formatted)
    pub text: String,

    /// Code examples included in the answer
    #[serde(default)]
    pub code_examples: Vec<CodeExample>,

    /// Related topics for further learning
    #[serde(default)]
    pub related_topics: Vec<String>,
}

/// A code example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Programming language (xml, javascript, java, etc.)
    pub language: String,

    /// The code snippet
    pub code: String,

    /// Optional description of what the code does
    pub description: Option<String>,
}

/// Reference to a knowledge base entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeReference {
    /// Knowledge entry ID
    pub knowledge_id: i32,

    /// Entry name
    pub name: String,

    /// Category (component, pattern, etc.)
    pub category: String,

    /// Section within the entry
    pub section: Option<String>,

    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
}

/// Response metadata (NO LLM details exposed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAMeta {
    /// Generator version
    pub generator: String,

    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Answer time in milliseconds
    pub answer_time_ms: u64,
}

impl QAMeta {
    pub fn new(generator: impl Into<String>, answer_time_ms: u64) -> Self {
        Self {
            generator: generator.into(),
            timestamp: chrono::Utc::now(),
            answer_time_ms,
        }
    }
}

impl QAResponse {
    pub fn success(answer: QAAnswer, references: Vec<KnowledgeReference>, meta: QAMeta) -> Self {
        Self {
            status: QAStatus::Success,
            answer: Some(answer),
            references,
            error: None,
            meta,
        }
    }

    pub fn error(error: impl Into<String>, meta: QAMeta) -> Self {
        Self {
            status: QAStatus::Error,
            answer: None,
            references: vec![],
            error: Some(error.into()),
            meta,
        }
    }
}
