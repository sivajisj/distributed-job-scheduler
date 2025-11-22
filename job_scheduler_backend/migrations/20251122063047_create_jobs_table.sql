-- Add migration script here
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(32) NOT NULL, -- e.g., 'QUEUED', 'RUNNING', 'COMPLETED', 'FAILED'
    result JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    finished_at TIMESTAMP WITH TIME ZONE,
    worker_id VARCHAR(64) -- Optional: to track which worker processed it
);

CREATE INDEX idx_jobs_status ON jobs (status);