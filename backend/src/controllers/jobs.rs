//! Job status controller for async generation polling.
//!
//! Provides endpoints for clients to check the status of queued generation jobs.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::debug_handler;
use axum::extract::Path;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::models::_entities::generation_logs;
use crate::workers::{JobQueueProcessor, QueueStats};

/// Job status response
#[derive(Debug, Serialize, Deserialize)]
pub struct JobStatusResponse {
    /// Job ID
    pub job_id: String,
    /// Current status: queued, processing, completed, failed
    pub status: String,
    /// Position in queue (if queued)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_position: Option<i64>,
    /// Estimated wait time in seconds (rough estimate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_wait_secs: Option<i64>,
    /// Generated artifacts (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<serde_json::Value>,
    /// Warnings from generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Generation time in milliseconds (if completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_time_ms: Option<i32>,
    /// Product type
    pub product: String,
    /// Timestamps
    pub timestamps: JobTimestamps,
}

/// Job timestamps
#[derive(Debug, Serialize, Deserialize)]
pub struct JobTimestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queued_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Queue stats response
#[derive(Debug, Serialize, Deserialize)]
pub struct QueueStatsResponse {
    pub stats: QueueStats,
    /// Average processing time in ms (from recent jobs)
    pub avg_processing_time_ms: Option<i64>,
}

/// Get job status by ID
///
/// GET /agent/jobs/:job_id
#[debug_handler]
pub async fn get_job_status(
    State(ctx): State<AppContext>,
    Path(job_id): Path<String>,
) -> Result<Response> {
    let job = generation_logs::Entity::find()
        .filter(generation_logs::Column::JobId.eq(&job_id))
        .one(&ctx.db)
        .await?;

    let job = match job {
        Some(j) => j,
        None => {
            return format::json(serde_json::json!({
                "error": "Job not found",
                "job_id": job_id
            }));
        }
    };

    // Get queue position if still queued
    let queue_position = if job.status == "queued" {
        JobQueueProcessor::get_queue_position(&ctx.db, &job_id).await.ok().flatten()
    } else {
        None
    };

    // Estimate wait time (rough: 30 seconds per job ahead)
    let estimated_wait_secs = queue_position.map(|pos| pos * 30);

    // Parse artifacts if completed
    let artifacts = job.artifacts.as_ref().and_then(|a| serde_json::from_str(a).ok());

    // Parse warnings
    let warnings: Option<Vec<String>> = job
        .warnings
        .as_ref()
        .and_then(|w| serde_json::from_str(w).ok());

    format::json(JobStatusResponse {
        job_id: job_id.clone(),
        status: job.status.clone(),
        queue_position,
        estimated_wait_secs,
        artifacts,
        warnings,
        error: job.error_message.clone(),
        generation_time_ms: job.generation_time_ms,
        product: job.product.clone(),
        timestamps: JobTimestamps {
            queued_at: job.queued_at.map(|t| t.to_rfc3339()),
            started_at: job.started_at.map(|t| t.to_rfc3339()),
            completed_at: job.completed_at.map(|t| t.to_rfc3339()),
        },
    })
}

/// Get queue statistics
///
/// GET /agent/queue/stats
#[debug_handler]
pub async fn get_queue_stats(State(ctx): State<AppContext>) -> Result<Response> {
    let stats = JobQueueProcessor::get_queue_stats(&ctx.db)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    // Calculate average processing time from recent completed jobs
    use sea_orm::QueryOrder;
    let recent_jobs = generation_logs::Entity::find()
        .filter(generation_logs::Column::Status.eq("completed"))
        .filter(generation_logs::Column::GenerationTimeMs.is_not_null())
        .order_by_desc(generation_logs::Column::CompletedAt)
        .limit(10)
        .all(&ctx.db)
        .await?;

    let avg_time = if recent_jobs.is_empty() {
        None
    } else {
        let total: i64 = recent_jobs
            .iter()
            .filter_map(|j| j.generation_time_ms.map(|t| t as i64))
            .sum();
        Some(total / recent_jobs.len() as i64)
    };

    format::json(QueueStatsResponse {
        stats,
        avg_processing_time_ms: avg_time,
    })
}

/// Cancel a queued job
///
/// DELETE /agent/jobs/:job_id
#[debug_handler]
pub async fn cancel_job(
    State(ctx): State<AppContext>,
    Path(job_id): Path<String>,
) -> Result<Response> {
    use sea_orm::{ActiveModelTrait, Set};

    let job = generation_logs::Entity::find()
        .filter(generation_logs::Column::JobId.eq(&job_id))
        .one(&ctx.db)
        .await?;

    let job = match job {
        Some(j) => j,
        None => {
            return format::json(serde_json::json!({
                "error": "Job not found",
                "job_id": job_id
            }));
        }
    };

    // Only queued jobs can be cancelled
    if job.status != "queued" {
        return format::json(serde_json::json!({
            "error": "Only queued jobs can be cancelled",
            "job_id": job_id,
            "current_status": job.status
        }));
    }

    // Mark as cancelled
    let mut active_job: generation_logs::ActiveModel = job.into();
    active_job.status = Set("cancelled".to_string());
    active_job.completed_at = Set(Some(chrono::Utc::now().into()));
    active_job.update(&ctx.db).await?;

    format::json(serde_json::json!({
        "success": true,
        "job_id": job_id,
        "status": "cancelled"
    }))
}

/// Routes for job status API
pub fn routes() -> Routes {
    Routes::new()
        .prefix("agent/")
        .add("jobs/{job_id}", get(get_job_status))
        .add("jobs/{job_id}", delete(cancel_job))
        .add("queue/stats", get(get_queue_stats))
}
