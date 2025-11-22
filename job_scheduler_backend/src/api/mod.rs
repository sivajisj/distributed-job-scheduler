// backend/src/api/mod.rs

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;
use deadpool_redis::redis::AsyncCommands;

use crate::{AppState, db, models::*};

// Utility for handling Axum errors
type ApiResult<T> = Result<T, (StatusCode, String)>;

// --- Handlers ---

// GET /
pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}
// POST /jobs
// 1. Create job in Postgres (status: QUEUED)
// 2. Push job ID to Redis queue

pub async fn create_job(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateJobRequest>,
) -> ApiResult<Json<Job>> {
    tracing::info!("Received request to create job: {}", payload.job_type);

    // 1. Create job in DB
    let job = db::create_new_job(&app_state.db_pool, &payload.job_type, &payload.payload)
        .await
        .map_err(|e| {
            tracing::error!("DB Error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error: {}", e),
            )
        })?;

    // 2. Push job ID to Redis queue
    let mut redis_con = app_state.redis_pool.get().await.map_err(|e| {
        tracing::error!("Redis Pool Error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Redis connection error".to_string(),
        )
    })?;

    // Correct `rpush`
    let _: () = redis_con
        .rpush("job_queue", job.id.to_string())
        .await
        .map_err(|e| {
            tracing::error!("Redis RPUSH Error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Redis queue push error".to_string(),
            )
        })?;

    Ok(Json(job))
}

// GET /jobs
pub async fn list_jobs(State(app_state): State<Arc<AppState>>) -> ApiResult<Json<Vec<Job>>> {
    let jobs = sqlx::query_as::<_, Job>("SELECT * FROM jobs ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("DB List Error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch jobs".to_string(),
            )
        })?;

    Ok(Json(jobs))
}

// GET /jobs/:id
pub async fn get_job(
    State(app_state): State<Arc<AppState>>,
    Path(job_id): Path<Uuid>,
) -> ApiResult<Json<Job>> {
    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = $1")
        .bind(job_id)
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                (StatusCode::NOT_FOUND, format!("Job {} not found", job_id))
            }
            _ => {
                tracing::error!("DB Get Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to fetch job".to_string(),
                )
            }
        })?;

    Ok(Json(job))
}

// Common Pitfall: Mixing up `uuid` crate's Uuid with `sqlx`'s Uuid representation. Ensure feature flags are correct.
// Concurrency Pattern: Using `deadpool-redis` for connection pooling is crucial here to handle concurrent API requests.
