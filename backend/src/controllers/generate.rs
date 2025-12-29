#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::debug_handler;
use axum::extract::Query;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    GenerateInput, GenerateOptions, GenerateResponse, GenerateStatus, RequestContext,
};
use crate::models::_entities::generation_logs;
use crate::services::{GenerationService, SpringGenerationService};
use crate::workers::generation::GenerateJobRequest;

/// API request for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateApiRequest {
    /// Product identifier (e.g., "xframe5-ui")
    pub product: String,

    /// Input data
    pub input: GenerateInput,

    /// Generation options
    #[serde(default)]
    pub options: GenerateOptions,

    /// Request context
    #[serde(default)]
    pub context: RequestContext,

    /// Priority for async processing (1=high, 5=low, default=3)
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_priority() -> i32 {
    3
}

/// Query parameters for generate endpoint
#[derive(Debug, Deserialize)]
pub struct GenerateQuery {
    /// Processing mode: "async" for queue-based, omit for sync
    #[serde(default)]
    pub mode: Option<String>,
}

impl GenerateQuery {
    pub fn is_async(&self) -> bool {
        self.mode.as_ref().map_or(false, |v| v == "async")
    }
}

/// Async generation response
#[derive(Debug, Serialize)]
pub struct AsyncGenerateResponse {
    /// Job ID for polling
    pub job_id: String,
    /// Initial status
    pub status: String,
    /// URL to poll for status
    pub status_url: String,
    /// Message
    pub message: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub llm_available: bool,
    pub message: Option<String>,
}

/// Generate endpoint - main entry point for code generation
///
/// POST /agent/generate
/// POST /agent/generate?mode=async (async mode)
///
/// Request:
/// ```json
/// {
///   "product": "xframe5-ui",
///   "input": {
///     "type": "db_schema",
///     "table": "member",
///     "columns": [...]
///   },
///   "options": {
///     "language": "ko",
///     "strict_mode": false
///   },
///   "context": {
///     "project": "xframe5",
///     "output": ["xml", "javascript"]
///   },
///   "priority": 3
/// }
/// ```
///
/// Sync Response:
/// ```json
/// {
///   "status": "success",
///   "artifacts": { ... },
///   "warnings": [],
///   "meta": { ... }
/// }
/// ```
///
/// Async Response:
/// ```json
/// {
///   "job_id": "uuid",
///   "status": "queued",
///   "status_url": "/agent/jobs/uuid",
///   "message": "Job queued for processing"
/// }
/// ```
#[debug_handler]
pub async fn generate(
    State(ctx): State<AppContext>,
    Query(query): Query<GenerateQuery>,
    Json(req): Json<GenerateApiRequest>,
) -> Result<Response> {
    // Validate product
    if req.product.is_empty() {
        return format::json(GenerateResponse {
            status: GenerateStatus::Error,
            artifacts: None,
            warnings: vec![],
            error: Some("Product is required".to_string()),
            meta: crate::domain::ResponseMeta {
                generator: "unknown".to_string(),
                timestamp: chrono::Utc::now(),
                generation_time_ms: 0,
            },
        });
    }

    // TODO: Extract user ID from JWT token when auth is integrated
    let user_id: i32 = 1; // Default to system user for now

    // Check if async mode is requested
    tracing::debug!("Query params: {:?}, is_async: {}", query, query.is_async());
    if query.is_async() {
        tracing::info!("Async mode requested, enqueueing job");
        return enqueue_job(&ctx, &req, user_id).await;
    }

    // Synchronous processing (legacy mode)
    tracing::info!("Sync mode, processing immediately");
    process_sync(&ctx, req, user_id).await
}

/// Enqueue a job for async processing
async fn enqueue_job(
    ctx: &AppContext,
    req: &GenerateApiRequest,
    user_id: i32,
) -> Result<Response> {
    let job_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    // Determine input type for logging
    let input_type = match &req.input {
        GenerateInput::DbSchema(_) => "db_schema",
        GenerateInput::QuerySample(_) => "query_sample",
        GenerateInput::NaturalLanguage(_) => "natural_language",
    };

    // Create job payload
    let payload = GenerateJobRequest {
        product: req.product.clone(),
        input: req.input.clone(),
        options: req.options.clone(),
        context: req.context.clone(),
    };

    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| Error::string(&format!("Failed to serialize payload: {}", e)))?;

    // Create generation log entry as queued
    let new_job = generation_logs::ActiveModel {
        job_id: Set(Some(job_id.clone())),
        product: Set(req.product.clone()),
        input_type: Set(input_type.to_string()),
        ui_intent: Set("pending".to_string()), // Will be populated during processing
        template_version: Set(1),
        status: Set("queued".to_string()),
        request_payload: Set(Some(payload_json)),
        queued_at: Set(Some(now.into())),
        priority: Set(req.priority.clamp(1, 5)),
        user_id: Set(user_id),
        ..Default::default()
    };

    new_job.insert(&ctx.db).await?;

    tracing::info!("Job {} queued for {} generation", job_id, req.product);

    format::json(AsyncGenerateResponse {
        job_id: job_id.clone(),
        status: "queued".to_string(),
        status_url: format!("/agent/jobs/{}", job_id),
        message: "Job queued for processing. Poll status_url for updates.".to_string(),
    })
}

/// Process request synchronously (legacy mode)
async fn process_sync(
    ctx: &AppContext,
    req: GenerateApiRequest,
    user_id: i32,
) -> Result<Response> {
    // Route based on product type
    match req.product.as_str() {
        "spring-backend" => {
            // Spring Framework backend generation
            let response = SpringGenerationService::generate(
                &ctx.db,
                req.input,
                &req.options,
                &req.context,
                Some(user_id),
            )
            .await;

            match response {
                Ok(resp) => format::json(resp),
                Err(e) => {
                    tracing::error!("Spring generation failed: {}", e);
                    format::json(crate::services::SpringGenerateResponse {
                        status: GenerateStatus::Error,
                        artifacts: None,
                        warnings: vec![],
                        error: Some(format!("Generation failed: {}", e)),
                        meta: crate::domain::ResponseMeta {
                            generator: "spring-backend-v1".to_string(),
                            timestamp: chrono::Utc::now(),
                            generation_time_ms: 0,
                        },
                    })
                }
            }
        }
        "xframe5-ui" | _ => {
            // xFrame5 UI generation (default)
            let response = GenerationService::generate(
                &ctx.db,
                req.input,
                &req.product,
                &req.options,
                &req.context,
                Some(user_id),
            )
            .await;

            match response {
                Ok(resp) => format::json(resp),
                Err(e) => {
                    tracing::error!("Generation failed: {}", e);
                    format::json(GenerateResponse {
                        status: GenerateStatus::Error,
                        artifacts: None,
                        warnings: vec![],
                        error: Some(format!("Generation failed: {}", e)),
                        meta: crate::domain::ResponseMeta {
                            generator: format!("{}-v1", req.product),
                            timestamp: chrono::Utc::now(),
                            generation_time_ms: 0,
                        },
                    })
                }
            }
        }
    }
}

/// Health check endpoint
///
/// GET /agent/health
#[debug_handler]
pub async fn health(State(ctx): State<AppContext>) -> Result<Response> {
    // Check LLM availability (DB config takes priority, falls back to env)
    let llm = crate::llm::create_backend_from_db_or_env(&ctx.db).await;
    let llm_check = llm.health_check().await;

    let (llm_available, message) = match llm_check {
        Ok(_) => (true, None),
        Err(e) => (false, Some(format!("LLM not available: {}", e))),
    };

    format::json(HealthResponse {
        status: if llm_available { "healthy" } else { "degraded" }.to_string(),
        llm_available,
        message,
    })
}

/// Get available products
///
/// GET /agent/products
#[debug_handler]
pub async fn list_products(State(_ctx): State<AppContext>) -> Result<Response> {
    format::json(serde_json::json!({
        "products": [
            {
                "id": "xframe5-ui",
                "name": "xFrame5 UI Generator",
                "description": "Generate xFrame5 XML views and JavaScript handlers",
                "status": "available",
                "input_types": ["db_schema", "query_sample", "natural_language"],
                "output_types": ["xml", "javascript"]
            },
            {
                "id": "spring-backend",
                "name": "Spring Framework Generator",
                "description": "Generate Spring Controller, Service, DTO, and MyBatis Mapper",
                "status": "available",
                "input_types": ["db_schema", "query_sample", "natural_language"],
                "output_types": ["controller", "service", "service_impl", "dto", "mapper", "mapper_xml"]
            }
        ]
    }))
}

/// Routes for the agent API
pub fn routes() -> Routes {
    Routes::new()
        .prefix("agent/")
        .add("generate", post(generate))
        .add("health", get(health))
        .add("products", get(list_products))
}
