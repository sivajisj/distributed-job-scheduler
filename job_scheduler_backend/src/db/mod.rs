// backend/src/db/mod.rs

use deadpool_redis::{Config, Pool, Runtime};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;
use crate::models::Job;
// --- PostgreSQL Functions ---

pub async fn init_db_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Attempting to connect to PostgreSQL...");
    let pool = PgPool::connect(database_url).await?;
    info!("Successfully connected to PostgreSQL!");
    
    // Run migrations on startup (Requires sqlx-cli or equivalent setup)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    info!("Database migrations executed successfully.");

    Ok(pool)
}

// Function to create a new job in the database
pub async fn create_new_job(
    pool: &PgPool,
    job_type: &str,
    payload: &serde_json::Value,
) -> Result<super::models::Job, sqlx::Error> {
    sqlx::query_as!(
        super::models::Job,
        r#"
        INSERT INTO jobs (job_type, payload, status)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        job_type,
        payload,
        super::models::JobStatus::Queued.as_str()
    )
    .fetch_one(pool)
    .await
}

// --- Redis Functions ---

pub fn init_redis_pool(redis_url: &str) -> Result<Pool, deadpool_redis::CreatePoolError> {
    info!("Attempting to connect to Redis...");
    let cfg = Config::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
    info!("Successfully created Redis connection pool!");
    // Note: Connection check happens on first use
    Ok(pool)
}

pub async fn get_job_by_id(pool: &PgPool, job_id: Uuid) -> Result<Job, sqlx::Error> {
    sqlx::query_as::<_, Job>(
        "SELECT * FROM jobs WHERE id = $1"
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
}