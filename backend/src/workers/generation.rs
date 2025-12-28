//! Background worker for processing generation jobs.
//!
//! This worker processes queued generation requests asynchronously,
//! ensuring LLM resources are used efficiently without concurrent contention.

use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::domain::{GenerateInput, GenerateOptions, GenerateStatus, RequestContext};
use crate::models::_entities::generation_logs;
use crate::services::{GenerationService, SpringGenerationService};

/// Worker arguments containing the job ID to process
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationWorkerArgs {
    pub job_id: String,
}

/// Background worker for processing generation jobs
pub struct GenerationWorker {
    pub ctx: AppContext,
}

#[async_trait]
impl BackgroundWorker<GenerationWorkerArgs> for GenerationWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: GenerationWorkerArgs) -> Result<()> {
        tracing::info!("Processing generation job: {}", args.job_id);

        // Find the job
        let job = generation_logs::Entity::find()
            .filter(generation_logs::Column::JobId.eq(&args.job_id))
            .one(&self.ctx.db)
            .await?;

        let job = match job {
            Some(j) => j,
            None => {
                tracing::error!("Job not found: {}", args.job_id);
                return Ok(());
            }
        };

        // Check if already processed
        if job.status != "queued" {
            tracing::warn!("Job {} already processed (status: {})", args.job_id, job.status);
            return Ok(());
        }

        // Mark as processing
        let mut active_job: generation_logs::ActiveModel = job.clone().into();
        active_job.status = Set("processing".to_string());
        active_job.started_at = Set(Some(chrono::Utc::now().into()));
        let job = active_job.update(&self.ctx.db).await?;

        // Parse the request payload
        let payload = match &job.request_payload {
            Some(p) => p,
            None => {
                let _ = update_job_failed(&self.ctx.db, &args.job_id, "No request payload").await;
                return Ok(());
            }
        };

        let request: GenerateJobRequest = match serde_json::from_str(payload) {
            Ok(r) => r,
            Err(e) => {
                let _ = update_job_failed(&self.ctx.db, &args.job_id, &format!("Invalid payload: {}", e))
                    .await;
                return Ok(());
            }
        };

        let start_time = std::time::Instant::now();

        // Process based on product type
        let result = match request.product.as_str() {
            "spring-backend" => {
                process_spring_generation(&self.ctx.db, &request, job.user_id).await
            }
            _ => process_xframe5_generation(&self.ctx.db, &request, job.user_id).await,
        };

        let generation_time_ms = start_time.elapsed().as_millis() as i32;

        // Update job with result
        match result {
            Ok((artifacts, warnings)) => {
                let mut active_job: generation_logs::ActiveModel = job.into();
                active_job.status = Set("completed".to_string());
                active_job.artifacts = Set(Some(artifacts));
                active_job.warnings = Set(Some(serde_json::to_string(&warnings).unwrap_or_default()));
                active_job.generation_time_ms = Set(Some(generation_time_ms));
                active_job.completed_at = Set(Some(chrono::Utc::now().into()));
                active_job.update(&self.ctx.db).await?;
                tracing::info!("Job {} completed in {}ms", args.job_id, generation_time_ms);
            }
            Err(e) => {
                let _ = update_job_failed(&self.ctx.db, &args.job_id, &e.to_string()).await;
                tracing::error!("Job {} failed: {}", args.job_id, e);
            }
        }

        Ok(())
    }
}

/// Request payload stored in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateJobRequest {
    pub product: String,
    pub input: GenerateInput,
    pub options: GenerateOptions,
    pub context: RequestContext,
}

/// Process xFrame5 UI generation
async fn process_xframe5_generation(
    db: &DatabaseConnection,
    request: &GenerateJobRequest,
    user_id: i32,
) -> anyhow::Result<(String, Vec<String>)> {
    let response = GenerationService::generate(
        db,
        request.input.clone(),
        &request.product,
        &request.options,
        &request.context,
        Some(user_id),
    )
    .await?;

    if response.status == GenerateStatus::Error {
        return Err(anyhow::anyhow!(
            response.error.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    let artifacts = serde_json::to_string(&response.artifacts)?;
    Ok((artifacts, response.warnings))
}

/// Process Spring backend generation
async fn process_spring_generation(
    db: &DatabaseConnection,
    request: &GenerateJobRequest,
    user_id: i32,
) -> anyhow::Result<(String, Vec<String>)> {
    let response = SpringGenerationService::generate(
        db,
        request.input.clone(),
        &request.options,
        &request.context,
        Some(user_id),
    )
    .await?;

    if response.status == GenerateStatus::Error {
        return Err(anyhow::anyhow!(
            response.error.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }

    let artifacts = serde_json::to_string(&response.artifacts)?;
    Ok((artifacts, response.warnings))
}

/// Update job as failed
async fn update_job_failed(
    db: &DatabaseConnection,
    job_id: &str,
    error: &str,
) -> anyhow::Result<()> {
    let job = generation_logs::Entity::find()
        .filter(generation_logs::Column::JobId.eq(job_id))
        .one(db)
        .await?;

    if let Some(job) = job {
        let mut active_job: generation_logs::ActiveModel = job.into();
        active_job.status = Set("failed".to_string());
        active_job.error_message = Set(Some(error.to_string()));
        active_job.completed_at = Set(Some(chrono::Utc::now().into()));
        active_job.update(db).await?;
    }

    Ok(())
}

/// Job queue processor - runs continuously to process queued jobs
///
/// This is an alternative to the Loco worker queue that uses the database directly
/// for simpler on-premise deployment without Redis.
pub struct JobQueueProcessor;

impl JobQueueProcessor {
    /// Process the next queued job (returns true if a job was processed)
    pub async fn process_next(db: &DatabaseConnection) -> anyhow::Result<bool> {
        // Find the next queued job (by priority then queued_at)
        let job = generation_logs::Entity::find()
            .filter(generation_logs::Column::Status.eq("queued"))
            .order_by_asc(generation_logs::Column::Priority)
            .order_by_asc(generation_logs::Column::QueuedAt)
            .one(db)
            .await?;

        let job = match job {
            Some(j) => j,
            None => return Ok(false), // No jobs to process
        };

        let job_id = match &job.job_id {
            Some(id) => id.clone(),
            None => return Ok(false),
        };

        tracing::info!("Dequeued job: {}", job_id);

        // Mark as processing
        let mut active_job: generation_logs::ActiveModel = job.clone().into();
        active_job.status = Set("processing".to_string());
        active_job.started_at = Set(Some(chrono::Utc::now().into()));
        let job = active_job.update(db).await?;

        // Parse request
        let payload = match &job.request_payload {
            Some(p) => p.clone(),
            None => {
                update_job_failed(db, &job_id, "No request payload").await?;
                return Ok(true);
            }
        };

        let request: GenerateJobRequest = match serde_json::from_str(&payload) {
            Ok(r) => r,
            Err(e) => {
                update_job_failed(db, &job_id, &format!("Invalid payload: {}", e)).await?;
                return Ok(true);
            }
        };

        let start_time = std::time::Instant::now();

        // Process
        let result = match request.product.as_str() {
            "spring-backend" => process_spring_generation(db, &request, job.user_id).await,
            _ => process_xframe5_generation(db, &request, job.user_id).await,
        };

        let generation_time_ms = start_time.elapsed().as_millis() as i32;

        // Update result
        match result {
            Ok((artifacts, warnings)) => {
                let mut active_job: generation_logs::ActiveModel = job.into();
                active_job.status = Set("completed".to_string());
                active_job.artifacts = Set(Some(artifacts));
                active_job.warnings = Set(Some(serde_json::to_string(&warnings).unwrap_or_default()));
                active_job.generation_time_ms = Set(Some(generation_time_ms));
                active_job.completed_at = Set(Some(chrono::Utc::now().into()));
                active_job.update(db).await?;
                tracing::info!("Job {} completed in {}ms", job_id, generation_time_ms);
            }
            Err(e) => {
                update_job_failed(db, &job_id, &e.to_string()).await?;
                tracing::error!("Job {} failed: {}", job_id, e);
            }
        }

        Ok(true)
    }

    /// Get queue statistics
    pub async fn get_queue_stats(db: &DatabaseConnection) -> anyhow::Result<QueueStats> {
        let queued = generation_logs::Entity::find()
            .filter(generation_logs::Column::Status.eq("queued"))
            .count(db)
            .await?;

        let processing = generation_logs::Entity::find()
            .filter(generation_logs::Column::Status.eq("processing"))
            .count(db)
            .await?;

        let completed = generation_logs::Entity::find()
            .filter(generation_logs::Column::Status.eq("completed"))
            .count(db)
            .await?;

        let failed = generation_logs::Entity::find()
            .filter(generation_logs::Column::Status.eq("failed"))
            .count(db)
            .await?;

        Ok(QueueStats {
            queued: queued as i64,
            processing: processing as i64,
            completed: completed as i64,
            failed: failed as i64,
        })
    }

    /// Get position in queue for a specific job
    pub async fn get_queue_position(db: &DatabaseConnection, job_id: &str) -> anyhow::Result<Option<i64>> {
        // Find the job
        let job = generation_logs::Entity::find()
            .filter(generation_logs::Column::JobId.eq(job_id))
            .one(db)
            .await?;

        let job = match job {
            Some(j) => j,
            None => return Ok(None),
        };

        if job.status != "queued" {
            return Ok(None); // Not in queue
        }

        // Count jobs ahead (higher priority or earlier queued time)
        let ahead = generation_logs::Entity::find()
            .filter(generation_logs::Column::Status.eq("queued"))
            .filter(
                sea_orm::Condition::any()
                    .add(generation_logs::Column::Priority.lt(job.priority))
                    .add(
                        sea_orm::Condition::all()
                            .add(generation_logs::Column::Priority.eq(job.priority))
                            .add(generation_logs::Column::QueuedAt.lt(job.queued_at)),
                    ),
            )
            .count(db)
            .await?;

        Ok(Some(ahead as i64 + 1)) // +1 because position is 1-indexed
    }
}

/// Queue statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct QueueStats {
    pub queued: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
}
