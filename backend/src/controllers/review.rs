#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::debug_handler;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::{
    ReviewContext, ReviewInput, ReviewMeta, ReviewOptions, ReviewResponse, ReviewStatus,
};
use crate::services::ReviewService;

/// API request for code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewApiRequest {
    /// Product identifier (e.g., "xframe5-ui", "spring-backend")
    pub product: String,

    /// Review input (code to review)
    pub input: ReviewInput,

    /// Review options
    #[serde(default)]
    pub options: ReviewOptions,

    /// Request context
    #[serde(default)]
    pub context: ReviewContext,
}

/// Review endpoint - review code for issues and improvements
///
/// POST /agent/review
///
/// Request:
/// ```json
/// {
///   "product": "xframe5-ui",
///   "input": {
///     "code": "<?xml version=\"1.0\"?>...",
///     "fileType": "xml",
///     "context": "This is a list screen for members"
///   },
///   "options": {
///     "language": "ko",
///     "reviewFocus": ["syntax", "patterns", "naming"],
///     "companyId": null
///   },
///   "context": {
///     "project": "my-project",
///     "fileName": "member_list.xml"
///   }
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "status": "success",
///   "review": {
///     "summary": "Overall assessment",
///     "issues": [...],
///     "score": { "overall": 75, "categories": {...} },
///     "improvements": [...]
///   },
///   "meta": { ... }
/// }
/// ```
#[debug_handler]
pub async fn review(
    State(ctx): State<AppContext>,
    Json(req): Json<ReviewApiRequest>,
) -> Result<Response> {
    // Validate product
    if req.product.is_empty() {
        return format::json(ReviewResponse {
            status: ReviewStatus::Error,
            review: None,
            error: Some("Product is required".to_string()),
            meta: ReviewMeta::new("unknown", 0),
        });
    }

    // Validate code input
    if req.input.code.trim().is_empty() {
        return format::json(ReviewResponse {
            status: ReviewStatus::Error,
            review: None,
            error: Some("Code is required for review".to_string()),
            meta: ReviewMeta::new("unknown", 0),
        });
    }

    // Check code size limit (50KB)
    const MAX_CODE_SIZE: usize = 50 * 1024;
    if req.input.code.len() > MAX_CODE_SIZE {
        return format::json(ReviewResponse {
            status: ReviewStatus::Error,
            review: None,
            error: Some(format!(
                "Code exceeds maximum size limit of {} bytes",
                MAX_CODE_SIZE
            )),
            meta: ReviewMeta::new("unknown", 0),
        });
    }

    // TODO: Extract user ID from JWT token when auth is integrated
    let user_id: i32 = 1; // Default to system user for now

    // Perform code review
    let result = ReviewService::review(
        &ctx.db,
        req.input,
        &req.product,
        &req.options,
        &req.context,
        Some(user_id),
    )
    .await;

    match result {
        Ok(response) => format::json(response),
        Err(e) => {
            tracing::error!("Review failed: {}", e);
            format::json(ReviewResponse {
                status: ReviewStatus::Error,
                review: None,
                error: Some(format!("Review failed: {}", e)),
                meta: ReviewMeta::new(format!("{}-review-v1", req.product), 0),
            })
        }
    }
}

/// Routes for the review API
pub fn routes() -> Routes {
    Routes::new()
        .prefix("agent/")
        .add("review", post(review))
}
