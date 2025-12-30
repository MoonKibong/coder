#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::debug_handler;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::{QAInput, QAMeta, QAOptions, QAResponse, QAStatus};
use crate::services::QAService;

/// API request for Q&A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAApiRequest {
    /// Product identifier (e.g., "xframe5-ui", "spring-backend")
    pub product: String,

    /// Q&A input (question to answer)
    pub input: QAInput,

    /// Q&A options
    #[serde(default)]
    pub options: QAOptions,
}

/// Q&A endpoint - answer questions using knowledge base
///
/// POST /agent/qa
///
/// Request:
/// ```json
/// {
///   "product": "xframe5-ui",
///   "input": {
///     "question": "How do I use Dataset in xFrame5?",
///     "context": "Building a list screen with grid"
///   },
///   "options": {
///     "language": "ko",
///     "includeExamples": true,
///     "maxReferences": 5
///   }
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "status": "success",
///   "answer": {
///     "text": "Dataset은 xFrame5에서 데이터를 관리하는...",
///     "codeExamples": [...],
///     "relatedTopics": [...]
///   },
///   "references": [...],
///   "meta": { ... }
/// }
/// ```
#[debug_handler]
pub async fn qa(State(ctx): State<AppContext>, Json(req): Json<QAApiRequest>) -> Result<Response> {
    // Validate product
    if req.product.is_empty() {
        return format::json(QAResponse {
            status: QAStatus::Error,
            answer: None,
            references: vec![],
            error: Some("Product is required".to_string()),
            meta: QAMeta::new("unknown", 0),
        });
    }

    // Validate question input
    if req.input.question.trim().is_empty() {
        return format::json(QAResponse {
            status: QAStatus::Error,
            answer: None,
            references: vec![],
            error: Some("Question is required".to_string()),
            meta: QAMeta::new("unknown", 0),
        });
    }

    // Check question length limit (5KB)
    const MAX_QUESTION_SIZE: usize = 5 * 1024;
    if req.input.question.len() > MAX_QUESTION_SIZE {
        return format::json(QAResponse {
            status: QAStatus::Error,
            answer: None,
            references: vec![],
            error: Some(format!(
                "Question exceeds maximum size limit of {} characters",
                MAX_QUESTION_SIZE
            )),
            meta: QAMeta::new("unknown", 0),
        });
    }

    // TODO: Extract user ID from JWT token when auth is integrated
    let user_id: i32 = 1; // Default to system user for now

    // Answer question
    let result =
        QAService::answer(&ctx.db, req.input, &req.product, &req.options, Some(user_id)).await;

    match result {
        Ok(response) => format::json(response),
        Err(e) => {
            tracing::error!("QA failed: {}", e);
            format::json(QAResponse {
                status: QAStatus::Error,
                answer: None,
                references: vec![],
                error: Some(format!("QA failed: {}", e)),
                meta: QAMeta::new(format!("{}-qa-v1", req.product), 0),
            })
        }
    }
}

/// Routes for the Q&A API
pub fn routes() -> Routes {
    Routes::new().prefix("agent/").add("qa", post(qa))
}
