// backend/src/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use uuid::Uuid;

// --- API Payloads ---

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub job_type: String,
    pub payload: serde_json::Value,
}

// --- Internal & Database Models ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Queued => "QUEUED",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
        }
    }
}

// Implement FromRow for direct SQLx querying
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String, // Stored as String in DB for simple enum mapping
    pub result: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
}

// Utility struct for WebSocket updates
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    JobStatusUpdate(Job),
    ServerHeartbeat,
}
