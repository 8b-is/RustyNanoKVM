//! WebSocket handlers

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tracing::{debug, error, info};

use crate::state::AppState;

/// Create WebSocket router
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/hid", get(hid_websocket))
        .route("/terminal", get(terminal_websocket))
}

/// HID WebSocket handler
async fn hid_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_hid_connection(socket, state))
}

/// Handle HID WebSocket connection
async fn handle_hid_connection(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    info!("HID WebSocket connected");

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("HID message: {}", text);
                // Parse and handle HID commands
            }
            Ok(Message::Binary(data)) => {
                debug!("HID binary data: {} bytes", data.len());
                // Handle binary HID data
            }
            Ok(Message::Close(_)) => {
                info!("HID WebSocket closed");
                break;
            }
            Err(e) => {
                error!("HID WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Terminal WebSocket handler
async fn terminal_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_terminal_connection(socket, state))
}

/// Handle terminal WebSocket connection
async fn handle_terminal_connection(socket: WebSocket, _state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    info!("Terminal WebSocket connected");

    // In a real implementation, this would spawn a PTY and connect it to the WebSocket

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Terminal input: {}", text);
                // Send to PTY
            }
            Ok(Message::Binary(data)) => {
                // Send to PTY
            }
            Ok(Message::Close(_)) => {
                info!("Terminal WebSocket closed");
                break;
            }
            Err(e) => {
                error!("Terminal WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
