// backend/src/ws/mod.rs

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use std::sync::Arc;
use tracing::info;

use crate::{models::WsMessage, AppState};

// 5. WebSocket Server for real-time status updates
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state))
}

// Handles the established socket connection
async fn handle_socket(mut socket: WebSocket, app_state: Arc<AppState>) {
    info!("New WebSocket connection established.");

    // Subscribe to the real-time job update broadcast channel
    let mut rx = app_state.job_tx.subscribe();

    // Loop to listen for messages on the broadcast channel
    while let Ok(job_update) = rx.recv().await {
        let job_msg = WsMessage::JobStatusUpdate(job_update);
        
        // Serialize and send the Job update message
        match serde_json::to_string(&job_msg) {
            Ok(json_string) => {
                if socket.send(Message::Text(json_string)).await.is_err() {
                    // If sending fails (client disconnected/error), break the loop 
                    // and close the connection for this client.
                    info!("Client disconnected, closing WebSocket.");
                    break;
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize job update for WS: {}", e);
            }
        }
    }
}