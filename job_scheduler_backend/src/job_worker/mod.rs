// backend/src/job_worker/mod.rs

use crate::{
    AppState, db,
    models::{Job, JobStatus},
};
use async_trait::async_trait;
use deadpool_redis::Connection as RedisCon;
use deadpool_redis::redis::AsyncCommands;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

// 9. Concurrency Pattern: Job Executor Trait
// Decouples job execution logic from the worker queue management loop.
#[async_trait]
pub trait JobExecutor: Send + Sync {
    async fn execute(&self, job: Job) -> Result<serde_json::Value, String>;
}

pub struct SimpleJobExecutor;

#[async_trait]
impl JobExecutor for SimpleJobExecutor {
    // 8. Error Handling: Uses Result<Value, String> for simple success/failure reporting
    async fn execute(&self, job: Job) -> Result<serde_json::Value, String> {
        info!("Worker processing job: {} ({})", job.id, job.job_type);

        // Simulate varying job execution time (1 to 10 seconds)
        let duration = rand::random::<u64>() % 10 + 1;
        sleep(Duration::from_secs(duration)).await;

        if job.job_type.contains("fail") {
            warn!("Job {} failed intentionally.", job.id);
            Err(format!("Simulated failure after {}s.", duration))
        } else {
            info!("Job {} completed successfully in {}s.", job.id, duration);
            Ok(serde_json::json!({
                "worker_id": "worker-1",
                "duration_s": duration,
                "input_payload": job.payload
            }))
        }
    }
}

pub async fn run_worker(app_state: Arc<AppState>) {
    let worker_id = "worker-1".to_string();
    let executor = SimpleJobExecutor;
    info!("Starting Distributed Job Worker: {}", worker_id);

    loop {
        let mut redis_con: RedisCon = match app_state.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Worker failed to get Redis connection: {}. Retrying in 5s.",
                    e
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // 3. Redis-based message queue for job distribution (BLPOP)
        let job_id_result: Result<Option<(String, String)>, _> =
            redis_con.blpop("job_queue", 5.0).await;

        let job_id = match job_id_result {
            Ok(Some((_, ref job_id))) => match Uuid::parse_str(&job_id) {
                Ok(id) => id,
                Err(e) => {
                    error!("Invalid UUID in queue: {}. Skipping.", e);
                    continue;
                }
            },
            Ok(None) => continue, // Timeout: no job found, continue loop
            Err(e) => {
                error!("Redis BLPOP error: {}. Retrying in 1s.", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let job_id = match job_id_result {
            Ok(Some((_, job_id))) => match Uuid::parse_str(&job_id) {
                Ok(id) => id,
                Err(e) => {
                    error!("Invalid UUID in queue: {}. Skipping.", e);
                    continue;
                }
            },
            Ok(None) => continue, // Timeout: no job found, continue loop
            Err(e) => {
                error!("Redis BLPOP error: {}. Retrying in 1s.", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        // 1. Fetch Job from DB (ensure job status is accurate)
        let job = match db::get_job_by_id(&app_state.db_pool, job_id).await {
            Ok(j) => j,
            Err(e) => {
                error!("Could not fetch job {} from DB: {}", job_id, e);
                continue;
            }
        };

        // 2. Mark as RUNNING and broadcast update
        let updated_job_running = update_job_status(
            &app_state.db_pool,
            job_id,
            JobStatus::Running,
            Some(&worker_id),
            Some(chrono::Utc::now()),
        )
        .await;

        if let Some(j) = &updated_job_running {
            let _ = app_state.job_tx.send(j.clone());
        }

        // 3. Execute the Job
        let execution_result = executor.execute(job.clone()).await;

        // 4. Update Job status (COMPLETED/FAILED) and broadcast
        let final_status = match execution_result {
            Ok(result) => {
                update_job_result(&app_state.db_pool, job_id, JobStatus::Completed, &result).await
            }
            Err(err) => {
                update_job_result(
                    &app_state.db_pool,
                    job_id,
                    JobStatus::Failed,
                    &serde_json::json!({"error": err, "worker": worker_id}),
                )
                .await
            }
        };

        // 5. Broadcast final status
        if let Some(j) = final_status {
            let _ = app_state.job_tx.send(j);
        }
    }
}

// Utility function to update job status (for RUNNING state)
async fn update_job_status(
    pool: &sqlx::PgPool,
    id: Uuid,
    status: JobStatus,
    worker_id: Option<&String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<Job> {
    sqlx::query_as!(
        Job,
        r#"
        UPDATE jobs
        SET status = $1, started_at = $2, worker_id = $3
        WHERE id = $4
        RETURNING *
        "#,
        status.as_str(),
        started_at,
        worker_id,
        id
    )
    .fetch_one(pool)
    .await
    .ok()
}

// Utility function to update job result (for COMPLETED/FAILED states)
async fn update_job_result(
    pool: &sqlx::PgPool,
    id: Uuid,
    status: JobStatus,
    result: &serde_json::Value,
) -> Option<Job> {
    sqlx::query_as!(
        Job,
        r#"
        UPDATE jobs
        SET status = $1, result = $2, finished_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
        status.as_str(),
        result,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| error!("Failed to update job result in DB: {}", e))
    .ok()
}
