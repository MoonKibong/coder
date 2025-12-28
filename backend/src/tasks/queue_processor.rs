//! Background task for processing the generation queue.
//!
//! This task runs continuously, polling for queued jobs and processing them one at a time.

use loco_rs::prelude::*;
use std::time::Duration;

use crate::workers::JobQueueProcessor;

/// Queue processor task arguments
pub struct QueueProcessorTask;

#[async_trait]
impl Task for QueueProcessorTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "queue_processor".to_string(),
            detail: "Process queued generation jobs".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        tracing::info!("Starting queue processor task");

        loop {
            match JobQueueProcessor::process_next(&ctx.db).await {
                Ok(true) => {
                    // Processed a job, immediately check for more
                    tracing::debug!("Processed a job, checking for more...");
                }
                Ok(false) => {
                    // No jobs, wait before checking again
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    tracing::error!("Queue processor error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}
