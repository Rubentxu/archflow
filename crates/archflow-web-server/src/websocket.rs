// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web Server - WebSocket Handler for Real-time Collaboration
//
// This module implements the WebSocket handler that integrates with the
// collaboration system to provide real-time multi-user editing.
//
// Pattern: update → merge → broadcast
//
// Reference:
// - https://blog.logrocket.com/using-crdts-build-collaborative-rust-web-applications/
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    collaboration::{CollaborationMessage, ConnectedClient, RoomId, UserId},
    error::Result,
    AppState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tracing::{error, info, warn};

/// WebSocket sender type alias
type WsSender = futures::stream::SplitSink<WebSocket, Message>;

/// Extract room ID from WebSocket URL query parameters
///
/// Expected URL format: ws://host/ws?room=room_id&user=user_id
fn extract_connection_params(uri: &axum::http::Uri) -> Option<(RoomId, UserId)> {
    let query = uri.query()?;

    // Parse query parameters
    let mut room_id = None;
    let mut user_id = None;

    for pair in query.split('&') {
        let mut parts = pair.split('=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            match key {
                "room" => room_id = Some(value.to_string()),
                "user" => user_id = Some(value.to_string()),
                _ => {}
            }
        }
    }

    match (room_id, user_id) {
        (Some(room), Some(user)) => Some((room, user)),
        _ => None,
    }
}

/// WebSocket handler for real-time collaboration
///
/// Upgrades HTTP connection to WebSocket and manages the collaboration session.
///
/// # Query Parameters
///
/// - `room`: Room ID to join
/// - `user`: User ID for this connection
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
///
/// This is the main collaboration loop that:
/// 1. Joins a room
/// 2. Receives commands from client
/// 3. Broadcasts to other room members
/// 4. Handles cleanup on disconnect
async fn handle_socket(socket: WebSocket, state: AppState) {
    // Split WebSocket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Get connection info from the handshake URI
    // Note: We'll extract this from the first message for simplicity
    let room_id = RoomId::from("default-room");
    let user_id = UserId::from(format!("user-{}", fastrand::u32(..)));

    info!(
        "WebSocket client connected: room={}, user={}",
        room_id, user_id
    );

    // Create channel for sending messages to this client
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<CollaborationMessage>();

    // Create connected client
    let client = ConnectedClient {
        user_id: user_id.clone(),
        room_id: room_id.clone(),
        sender: tx,
        lamport_clock: 0,
    };

    // Send welcome message
    let welcome = CollaborationMessage::UserJoined {
        user_id: user_id.clone(),
        room_id: room_id.clone(),
    };

    if let Err(e) = send_message(&mut sender, &welcome).await {
        error!("Failed to send welcome message: {}", e);
        return;
    }

    // Main message loop
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if !handle_incoming_message(msg, &room_id, &user_id, &mut sender).await {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        // Connection closed
                        break;
                    }
                }
            }
            // Handle outgoing messages (from collaboration system)
            Some(msg) = rx.recv() => {
                if let Err(e) = send_message(&mut sender, &msg).await {
                    warn!("Failed to send collaboration message: {}", e);
                    break;
                }
            }
        }
    }

    info!(
        "WebSocket client disconnected: room={}, user={}",
        room_id, user_id
    );
}

/// Handle an incoming WebSocket message
///
/// Returns false if the connection should be closed
async fn handle_incoming_message(
    msg: Message,
    room_id: &RoomId,
    user_id: &UserId,
    sender: &mut WsSender,
) -> bool {
    match msg {
        Message::Text(text) => {
            if let Err(e) = handle_text_message(text, room_id, user_id, sender).await {
                error!("Error handling text message: {}", e);
                // Send error message to client
                let error_msg = CollaborationMessage::Error {
                    message: format!("Error: {}", e),
                };
                let _ = send_message(sender, &error_msg).await;
                true // Continue connection
            } else {
                true
            }
        }
        Message::Close(close_frame) => {
            info!("Client disconnecting: {:?}", close_frame);
            false // Close connection
        }
        Message::Ping(data) => {
            // Respond with pong
            if let Err(e) = sender.send(Message::Pong(data)).await {
                warn!("Failed to send pong: {}", e);
                false
            } else {
                true
            }
        }
        Message::Pong(_) => {
            // Pong received, continue
            true
        }
        _ => {
            // Ignore other message types
            true
        }
    }
}

/// Handle a text message from the client
async fn handle_text_message(
    text: String,
    room_id: &RoomId,
    user_id: &UserId,
    sender: &mut WsSender,
) -> Result<()> {
    // Try to parse as JSON
    let value: Value = serde_json::from_str(&text)?;

    // Check message type
    let msg_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'type' field"))?;

    match msg_type {
        "remote_command" => {
            // This is a command from the client to broadcast
            info!(
                "Received remote_command from {} in room {}: {}",
                user_id, room_id, text
            );

            // Echo back for now (TODO: implement actual broadcast via CollaborationManager)
            let response = CollaborationMessage::RemoteCommand {
                from_user: user_id.clone(),
                room_id: room_id.clone(),
                command_data: vec![],
            };

            send_message(sender, &response).await?;
        }
        "presence" => {
            // User presence update (cursor position, etc.)
            if let Some(data) = value.get("data") {
                info!("Presence update from {}: {}", user_id, data);
            }
        }
        "ping" => {
            // Heartbeat/ping
            let pong = CollaborationMessage::UserJoined {
                user_id: user_id.clone(),
                room_id: room_id.clone(),
            };
            send_message(sender, &pong).await?;
        }
        _ => {
            warn!("Unknown message type: {}", msg_type);
        }
    }

    Ok(())
}

/// Send a collaboration message as WebSocket text
async fn send_message(sender: &mut WsSender, msg: &CollaborationMessage) -> Result<()> {
    let json = serde_json::to_string(msg)?;
    let message = Message::Text(json);
    sender
        .send(message)
        .await
        .map_err(|e| crate::error::Error::Internal(format!("Failed to send message: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_connection_params_valid() {
        let uri = axum::http::Uri::from_static("/ws?room=test-room&user=test-user");
        let (room, user) = extract_connection_params(&uri).unwrap();

        assert_eq!(room, "test-room");
        assert_eq!(user, "test-user");
    }

    #[test]
    fn test_extract_connection_params_missing_room() {
        let uri = axum::http::Uri::from_static("/ws?user=test-user");
        let result = extract_connection_params(&uri);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_connection_params_missing_user() {
        let uri = axum::http::Uri::from_static("/ws?room=test-room");
        let result = extract_connection_params(&uri);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_connection_params_no_query() {
        let uri = axum::http::Uri::from_static("/ws");
        let result = extract_connection_params(&uri);
        assert!(result.is_none());
    }
}
