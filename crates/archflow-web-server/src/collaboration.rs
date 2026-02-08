// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web Server - Real-time Collaboration Module
//
// This module implements multi-user real-time collaboration using:
// - Room-based collaboration (multiple documents/sessions)
// - WebSocket message broadcast to all room participants
// - CRDT integration for conflict resolution
// - User presence tracking (join/leave/awareness)
//
// Architecture Pattern: update → merge → broadcast
//
// Reference:
// - docs/epics/EPIC-004-network-sync.md
//
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};

/// Maximum number of users per collaboration room
const MAX_USERS_PER_ROOM: usize = 16;

/// Room ID type
pub type RoomId = String;

/// User ID type
pub type UserId = String;

/// Collaboration message types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CollaborationMessage {
    /// User joined the room
    #[serde(rename = "user_joined")]
    UserJoined { user_id: UserId, room_id: RoomId },

    /// User left the room
    #[serde(rename = "user_left")]
    UserLeft { user_id: UserId, room_id: RoomId },

    /// Remote command from another user (CRDT)
    #[serde(rename = "remote_command")]
    RemoteCommand {
        from_user: UserId,
        room_id: RoomId,
        command_data: Vec<u8>,
    },

    /// User presence/cursor update
    #[serde(rename = "presence")]
    Presence {
        user_id: UserId,
        cursor: Option<CursorData>,
    },

    /// Room state snapshot (for new joiners)
    #[serde(rename = "room_state")]
    RoomState {
        room_id: RoomId,
        entity_count: usize,
    },

    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

/// User cursor position for presence awareness
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorData {
    pub x: f32,
    pub y: f32,
    pub entity_id: Option<u32>,
}

/// Connected client in a collaboration room
#[derive(Clone, Debug)]
pub struct ConnectedClient {
    /// User ID
    pub user_id: UserId,

    /// Room ID this client is in
    pub room_id: RoomId,

    /// Sender for messages to this client
    pub sender: mpsc::UnboundedSender<CollaborationMessage>,

    /// User's Lamport clock for CRDT
    pub lamport_clock: u64,
}

/// Collaboration room with multiple users
#[derive(Clone, Debug)]
pub struct CollaborationRoom {
    /// Room ID
    pub room_id: RoomId,

    /// Connected clients in this room
    pub clients: Vec<ConnectedClient>,

    /// Room's Lamport clock (max of all users' clocks)
    pub lamport_clock: u64,
}

impl CollaborationRoom {
    /// Create a new collaboration room
    #[must_use]
    pub fn new(room_id: RoomId) -> Self {
        Self {
            room_id,
            clients: Vec::new(),
            lamport_clock: 0,
        }
    }

    /// Add a client to this room
    ///
    /// Returns true if the client was added, false if room is full
    pub fn add_client(&mut self, client: ConnectedClient) -> bool {
        if self.clients.len() >= MAX_USERS_PER_ROOM {
            return false;
        }
        self.clients.push(client);
        true
    }

    /// Remove a client from this room
    ///
    /// Returns the removed client if found
    pub fn remove_client(&mut self, user_id: &UserId) -> Option<ConnectedClient> {
        let pos = self.clients.iter().position(|c| &c.user_id == user_id)?;
        Some(self.clients.remove(pos))
    }

    /// Get client by user ID
    #[must_use]
    pub fn get_client(&self, user_id: &UserId) -> Option<&ConnectedClient> {
        self.clients.iter().find(|c| &c.user_id == user_id)
    }

    /// Update room's Lamport clock
    pub fn update_clock(&mut self, new_clock: u64) {
        self.lamport_clock = self.lamport_clock.max(new_clock);
    }

    /// Broadcast a message to all clients in the room
    ///
    /// # Arguments
    ///
    /// * `message` - Message to broadcast
    /// * `exclude_user` - Optional user ID to exclude (typically the sender)
    pub fn broadcast(&self, message: CollaborationMessage, exclude_user: Option<&UserId>) {
        for client in &self.clients {
            // Skip the excluded user (typically the sender)
            if let Some(excluded) = exclude_user {
                if &client.user_id == excluded {
                    continue;
                }
            }

            // Send message to client (ignore errors, client may have disconnected)
            let _ = client.sender.send(message.clone());
        }
    }

    /// Get number of connected clients
    #[must_use]
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Check if a specific user is in this room
    #[must_use]
    pub fn has_user(&self, user_id: &UserId) -> bool {
        self.clients.iter().any(|c| &c.user_id == user_id)
    }
}

/// Manager for all collaboration rooms
pub struct CollaborationManager {
    /// All active rooms indexed by room ID
    rooms: Arc<RwLock<HashMap<RoomId, CollaborationRoom>>>,
}

impl CollaborationManager {
    /// Create a new collaboration manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a room
    ///
    /// # Arguments
    ///
    /// * `room_id` - Room identifier
    pub async fn get_or_create_room(&self, room_id: RoomId) -> CollaborationRoom {
        let mut rooms: tokio::sync::RwLockWriteGuard<'_, HashMap<RoomId, CollaborationRoom>> =
            self.rooms.write().await;

        rooms
            .entry(room_id.clone())
            .or_insert_with(|| CollaborationRoom::new(room_id))
            .clone()
    }

    /// Remove a room if it's empty
    ///
    /// Returns true if the room was removed
    pub async fn remove_room_if_empty(&self, room_id: &RoomId) -> bool {
        let mut rooms: tokio::sync::RwLockWriteGuard<'_, HashMap<RoomId, CollaborationRoom>> =
            self.rooms.write().await;

        // Check if room exists and is empty
        if let Some(room) = rooms.get(room_id) {
            if room.client_count() == 0 {
                rooms.remove(room_id);
                return true;
            }
        }

        false
    }

    /// Get a room if it exists
    pub async fn get_room(&self, room_id: &RoomId) -> Option<CollaborationRoom> {
        let rooms: tokio::sync::RwLockReadGuard<'_, HashMap<RoomId, CollaborationRoom>> =
            self.rooms.read().await;
        rooms.get(room_id).cloned()
    }

    /// Get all active room IDs
    pub async fn get_active_rooms(&self) -> Vec<RoomId> {
        let rooms: tokio::sync::RwLockReadGuard<'_, HashMap<RoomId, CollaborationRoom>> =
            self.rooms.read().await;
        let mut result: Vec<RoomId> = rooms.keys().cloned().collect();
        result.sort();
        result
    }

    /// Broadcast a message to all clients in a room
    ///
    /// # Arguments
    ///
    /// * `room_id` - Target room
    /// * `message` - Message to broadcast
    /// * `exclude_user` - Optional user ID to exclude (typically the sender)
    pub async fn broadcast_to_room(
        &self,
        room_id: &RoomId,
        message: CollaborationMessage,
        exclude_user: Option<&UserId>,
    ) {
        if let Some(room) = self.get_room(room_id).await {
            room.broadcast(message, exclude_user);
        }
    }

    /// Process and broadcast a remote command to all room members
    ///
    /// This implements the "broadcast" part of the "update → merge → broadcast" pattern.
    /// The calling layer handles the "merge" part (CRDT resolution with EntityStore).
    ///
    /// # Arguments
    ///
    /// * `room_id` - Target room
    /// * `from_user` - User ID who sent the command
    /// * `command_data` - Serialized command data
    pub async fn broadcast_remote_command(
        &self,
        room_id: &RoomId,
        from_user: &UserId,
        command_data: Vec<u8>,
    ) {
        let message = CollaborationMessage::RemoteCommand {
            from_user: from_user.clone(),
            room_id: room_id.clone(),
            command_data,
        };

        // Broadcast to all users except the sender
        self.broadcast_to_room(room_id, message, Some(from_user))
            .await;
    }

    /// Add a client to a room
    ///
    /// # Arguments
    ///
    /// * `room_id` - Target room
    /// * `client` - Client to add
    ///
    /// Returns true if added, false if room is full
    pub async fn add_client_to_room(&self, room_id: &RoomId, client: ConnectedClient) -> bool {
        let mut rooms: tokio::sync::RwLockWriteGuard<'_, HashMap<RoomId, CollaborationRoom>> =
            self.rooms.write().await;

        if let Some(room) = rooms.get_mut(room_id) {
            room.add_client(client)
        } else {
            false
        }
    }

    /// Remove a client from their room
    ///
    /// # Arguments
    ///
    /// * `room_id` - Target room
    /// * `user_id` - User ID to remove
    ///
    /// Returns the removed client if found
    pub async fn remove_client_from_room(
        &self,
        room_id: &RoomId,
        user_id: &UserId,
    ) -> Option<ConnectedClient> {
        let mut rooms: tokio::sync::RwLockWriteGuard<'_, HashMap<RoomId, CollaborationRoom>> =
            self.rooms.write().await;

        if let Some(room) = rooms.get_mut(room_id) {
            room.remove_client(user_id)
        } else {
            None
        }
    }
}

impl Default for CollaborationManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collaboration_manager_new() {
        let manager = CollaborationManager::new();
        let rooms: Vec<RoomId> = manager.get_active_rooms().await;
        assert_eq!(rooms.len(), 0);
    }

    #[tokio::test]
    async fn test_collaboration_manager_get_or_create_room() {
        let manager = CollaborationManager::new();

        // Create new room
        let room1 = manager.get_or_create_room("test-room".to_string()).await;

        assert_eq!(room1.room_id, "test-room");
        assert_eq!(room1.client_count(), 0);

        // Get same room (should not create new)
        let room2 = manager.get_or_create_room("test-room".to_string()).await;

        assert_eq!(room2.room_id, "test-room");
        assert_eq!(room2.client_count(), 0);
    }

    #[tokio::test]
    async fn test_collaboration_room_add_client() {
        let mut room = CollaborationRoom::new("test-room".to_string());

        let (tx, _rx) = mpsc::unbounded_channel();
        let client = ConnectedClient {
            user_id: "user1".to_string(),
            room_id: "test-room".to_string(),
            sender: tx,
            lamport_clock: 1,
        };

        let added = room.add_client(client);
        assert!(added);
        assert_eq!(room.client_count(), 1);
        assert!(room.has_user(&"user1".to_string()));
    }

    #[tokio::test]
    async fn test_collaboration_room_max_capacity() {
        let mut room = CollaborationRoom::new("test-room".to_string());

        // Fill room to capacity
        for i in 0..MAX_USERS_PER_ROOM {
            let (tx, _rx) = mpsc::unbounded_channel();
            let client = ConnectedClient {
                user_id: format!("user{}", i),
                room_id: "test-room".to_string(),
                sender: tx,
                lamport_clock: i as u64,
            };
            assert!(room.add_client(client));
        }

        assert_eq!(room.client_count(), MAX_USERS_PER_ROOM);

        // Try to add one more (should fail)
        let (tx, _rx) = mpsc::unbounded_channel();
        let extra_client = ConnectedClient {
            user_id: "extra".to_string(),
            room_id: "test-room".to_string(),
            sender: tx,
            lamport_clock: 999,
        };

        assert!(!room.add_client(extra_client));
        assert_eq!(room.client_count(), MAX_USERS_PER_ROOM);
    }

    #[tokio::test]
    async fn test_collaboration_room_remove_client() {
        let mut room = CollaborationRoom::new("test-room".to_string());

        let (tx, _rx) = mpsc::unbounded_channel();
        let client = ConnectedClient {
            user_id: "user1".to_string(),
            room_id: "test-room".to_string(),
            sender: tx,
            lamport_clock: 1,
        };

        room.add_client(client);
        assert_eq!(room.client_count(), 1);

        let removed = room.remove_client(&"user1".to_string());
        assert!(removed.is_some());
        assert_eq!(room.client_count(), 0);
        assert!(!room.has_user(&"user1".to_string()));
    }

    #[tokio::test]
    async fn test_collaboration_room_get_client() {
        let mut room = CollaborationRoom::new("test-room".to_string());

        let (tx, _rx) = mpsc::unbounded_channel();
        let client = ConnectedClient {
            user_id: "user1".to_string(),
            room_id: "test-room".to_string(),
            sender: tx,
            lamport_clock: 1,
        };

        room.add_client(client);

        let found = room.get_client(&"user1".to_string());
        assert!(found.is_some());
        assert_eq!(found.unwrap().user_id, "user1");

        let not_found = room.get_client(&"user2".to_string());
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_collaboration_room_update_clock() {
        let mut room = CollaborationRoom::new("test-room".to_string());
        assert_eq!(room.lamport_clock, 0);

        room.update_clock(5);
        assert_eq!(room.lamport_clock, 5);

        room.update_clock(3);
        assert_eq!(room.lamport_clock, 5); // Should keep max

        room.update_clock(10);
        assert_eq!(room.lamport_clock, 10);
    }

    #[tokio::test]
    async fn test_collaboration_manager_remove_empty_room() {
        let manager = CollaborationManager::new();

        // Create an empty room
        manager.get_or_create_room("test-room".to_string()).await;

        // Should successfully remove empty room
        let removed = manager.remove_room_if_empty(&"test-room".to_string()).await;
        assert!(removed);

        // Room should no longer exist
        assert!(manager.get_room(&"test-room".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_message_serialization() {
        let msg = CollaborationMessage::UserJoined {
            user_id: "user1".to_string(),
            room_id: "room1".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user_joined"));
        assert!(json.contains("user1"));
    }

    #[tokio::test]
    async fn test_remote_command_message() {
        let msg = CollaborationMessage::RemoteCommand {
            from_user: "user1".to_string(),
            room_id: "room1".to_string(),
            command_data: vec![1, 2, 3, 4],
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("remote_command"));

        // Deserialize back
        let deserialized: CollaborationMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            CollaborationMessage::RemoteCommand {
                from_user,
                room_id,
                command_data,
            } => {
                assert_eq!(from_user, "user1");
                assert_eq!(room_id, "room1");
                assert_eq!(command_data, vec![1, 2, 3, 4]);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
