#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::debug_handler;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::{
    GenerateInput, GenerateOptions, GenerateResponse, GenerateStatus, RequestContext,
};
use crate::services::{GenerationService, SpringGenerationService};

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
///   }
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "status": "success",
///   "artifacts": {
///     "xml": "<Dataset...>",
///     "javascript": "this.fn_search = function()..."
///   },
///   "warnings": [],
///   "meta": {
///     "generator": "xframe5-ui-v1",
///     "timestamp": "2025-01-01T00:00:00Z",
///     "generation_time_ms": 1234
///   }
/// }
/// ```
#[debug_handler]
pub async fn generate(
    State(ctx): State<AppContext>,
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
    let user_id: Option<i32> = None;

    // Route based on product type
    match req.product.as_str() {
        "spring-backend" => {
            // Spring Framework backend generation
            let response = SpringGenerationService::generate(
                &ctx.db,
                req.input,
                &req.options,
                &req.context,
                user_id,
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
                user_id,
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
pub async fn health(State(_ctx): State<AppContext>) -> Result<Response> {
    // Check LLM availability
    let llm = crate::llm::create_backend_from_env();
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
        .prefix("agent")
        .add("/generate", post(generate))
        .add("/health", get(health))
        .add("/products", get(list_products))
}
