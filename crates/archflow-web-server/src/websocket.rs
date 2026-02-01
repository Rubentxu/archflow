// ═══════════════════════════════════════════════════════════════════════════════
// WebSocket handler for real-time collaboration
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{error::Result, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tracing::{info, warn};

/// WebSocket handler for real-time collaboration
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket client connected");

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        info!("Received text message: {}", text);

                        // Echo back for now (TODO: implement actual collaboration)
                        if let Err(e) = sender.send(Message::Text(text)).await {
                            warn!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    Message::Close(close_frame) => {
                        info!("Client disconnected: {:?}", close_frame);
                        break;
                    }
                    Message::Ping(data) => {
                        // Respond with pong
                        if let Err(e) = sender.send(Message::Pong(data)).await {
                            warn!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket handler ended");
}
