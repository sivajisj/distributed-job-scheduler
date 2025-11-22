// backend/src/main.rs

use axum::{
    Router,
    routing::{get, post},
};
use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod db;
mod job_worker;
mod models;
mod ws;

// Application State shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_pool: RedisPool,
    pub job_tx: broadcast::Sender<models::Job>,
}

#[tokio::main]
async fn main() {
    // 1. Setup Logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "job_sheduler_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    tracing::info!("Server listening on {}", listen_addr);

    // 2. Setup Database and Redis Pools
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let db_pool = db::init_db_pool(&database_url)
        .await
        .expect("Failed to init DB");
    let redis_pool = db::init_redis_pool(&redis_url).expect("Failed to init Redis");

    // Setup WebSocket Broadcast Channel
    let (job_tx, _) = broadcast::channel(16);

    let app_state = Arc::new(AppState {
        db_pool,
        redis_pool,
        job_tx: job_tx.clone(),
    });

    // 3. Start Background Worker System
    tokio::spawn(job_worker::run_worker(app_state.clone()));
    tracing::info!("Background worker spawned.");
    let cors = CorsLayer::new()
        .allow_origin(Any) // or restrict to your frontend URL
        .allow_methods(Any)
        .allow_headers(Any);
    // 4. Setup Axum Router - FIXED: Use with_state instead of Extension
    let app = Router::new()
        .route("/", get(api::health_check))
        .route("/jobs", post(api::create_job).get(api::list_jobs))
        .route("/jobs/:id", get(api::get_job))
        .route("/ws", get(ws::ws_handler))
        .layer(cors)
        .with_state(app_state); // ← CHANGED THIS LINE

    // 5. Start HTTP Server
    let addr = listen_addr.parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
