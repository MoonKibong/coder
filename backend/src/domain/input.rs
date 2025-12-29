use serde::{Deserialize, Serialize};

/// Input types for the generation API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GenerateInput {
    /// Database schema input
    DbSchema(SchemaInput),
    /// SQL query sample input
    QuerySample(QuerySampleInput),
    /// Natural language description
    NaturalLanguage(NaturalLanguageInput),
}

/// Database schema input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInput {
    /// Table name
    pub table: String,

    /// Optional schema name
    pub schema: Option<String>,

    /// Column definitions
    #[serde(default)]
    pub columns: Vec<SchemaColumn>,

    /// Primary key columns
    #[serde(default)]
    pub primary_keys: Vec<String>,

    /// Foreign key relationships
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKey>,
}

impl SchemaInput {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            schema: None,
            columns: Vec::new(),
            primary_keys: Vec::new(),
            foreign_keys: Vec::new(),
        }
    }

    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    pub fn with_column(mut self, column: SchemaColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn with_primary_key(mut self, column: impl Into<String>) -> Self {
        self.primary_keys.push(column.into());
        self
    }
}

/// Schema column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaColumn {
    /// Column name
    pub name: String,

    /// Database column type (e.g., "VARCHAR(100)", "INTEGER", "DATE")
    pub column_type: String,

    /// Is nullable?
    pub nullable: bool,

    /// Is primary key?
    pub pk: bool,

    /// Default value
    pub default: Option<String>,

    /// Column comment/description
    pub comment: Option<String>,
}

impl SchemaColumn {
    pub fn new(name: impl Into<String>, column_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            column_type: column_type.into(),
            nullable: true,
            pk: false,
            default: None,
            comment: None,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.pk = true;
        self.nullable = false;
        self
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Foreign key relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    /// Local column name
    pub column: String,

    /// Referenced table
    pub ref_table: String,

    /// Referenced column
    pub ref_column: String,
}

/// SQL query sample input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySampleInput {
    /// SELECT query
    pub query: String,

    /// Query description/purpose
    pub description: Option<String>,

    /// Sample result columns (inferred from query or provided)
    pub result_columns: Option<Vec<QueryColumn>>,
}

impl QuerySampleInput {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            description: None,
            result_columns: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Query result column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryColumn {
    /// Column alias or name
    pub name: String,

    /// Inferred or specified type
    pub column_type: Option<String>,

    /// Column label
    pub label: Option<String>,
}

/// Natural language description input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageInput {
    /// Description of what to generate
    pub description: String,

    /// Target screen type (optional, will be inferred)
    pub screen_type: Option<String>,

    /// Additional context
    pub context: Option<String>,
}

impl NaturalLanguageInput {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            screen_type: None,
            context: None,
        }
    }

    pub fn with_screen_type(mut self, screen_type: impl Into<String>) -> Self {
        self.screen_type = Some(screen_type.into());
        self
    }
}

/// Generation request from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// Product identifier (e.g., "xframe5-ui")
    pub product: String,

    /// Input type and data
    pub input: GenerateInput,

    /// Generation options
    #[serde(default)]
    pub options: GenerateOptions,

    /// Request context
    #[serde(default)]
    pub context: RequestContext,
}

/// Generation options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerateOptions {
    /// Output language (default: "ko")
    #[serde(default = "default_language")]
    pub language: String,

    /// Strict validation mode
    #[serde(default)]
    pub strict_mode: bool,

    /// Company ID for custom rules
    pub company_id: Option<String>,
}

fn default_language() -> String {
    "ko".to_string()
}

/// Request context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestContext {
    /// Project identifier
    pub project: Option<String>,

    /// Target type (e.g., "frontend")
    pub target: Option<String>,

    /// Output formats requested
    #[serde(default = "default_outputs")]
    pub output: Vec<String>,
}

fn default_outputs() -> Vec<String> {
    vec!["xml".to_string(), "javascript".to_string()]
}

/// Generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// Status of generation
    pub status: GenerateStatus,

    /// Generated artifacts
    pub artifacts: Option<GeneratedArtifacts>,

    /// Warnings during generation
    #[serde(default)]
    pub warnings: Vec<String>,

    /// Error message (if status is error)
    pub error: Option<String>,

    /// Response metadata
    pub meta: ResponseMeta,
}

/// Generation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerateStatus {
    Success,
    PartialSuccess,
    Error,
}

/// Generated artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArtifacts {
    /// Generated XML content
    pub xml: Option<String>,

    /// Generated JavaScript content
    pub javascript: Option<String>,

    /// Suggested XML filename (e.g., "task_list.xml")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml_filename: Option<String>,

    /// Suggested JavaScript filename (e.g., "task_list.js")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub js_filename: Option<String>,
}

/// Response metadata (NO LLM details exposed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Generator version (e.g., "xframe5-ui-v1")
    pub generator: String,

    /// Generation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Generation time in milliseconds
    pub generation_time_ms: u64,
}
