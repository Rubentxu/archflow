//! Collaboration module for real-time CRDT-based editing
//!
//! This module provides integration between the ArchFlow SDK and the CRDT collaboration
//! system, enabling real-time collaborative editing with automatic conflict resolution.
//!
//! ## Architecture
//!
//! - **CollabRecord**: Adapter that implements `Record` trait for `Shape`
//! - **CollabManager**: Manages CRDT operations and synchronization
//! - **Awareness**: Tracks user presence (cursors, selections)
//! - **PresenceManager**: Handles real-time presence broadcasting
//!
//! ## Example
//!
//! ```rust
//! use archflow_sdk::collab::{CollabManager, CollabConfig};
//! use archflow_sdk::canvas::Shape;
//!
//! let mut collab = CollabManager::new(CollabConfig::default());
//!
//! // Create a shape
//! let shape = Shape::new_rectangle(10.0, 20.0, 100.0, 50.0);
//!
//! // Track local changes
//! collab.track_shape_creation(shape);
//!
//! // Get pending changes for synchronization
//! let pending = collab.get_pending_changes();
//! ```

use crate::canvas::{Canvas, Shape, ShapeType};
use archflow_collab::{CRDT, SiteId, VectorClock};
use archflow_core::EntityId;
use archflow_records::{
    DeltaManager, FractionalIndex, Record, RecordChange, RecordId, RecordStore,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Configuration for collaboration manager
#[derive(Clone, Debug)]
pub struct CollabConfig {
    /// Enable automatic conflict resolution
    pub auto_resolve: bool,
    /// Maximum number of pending operations before forcing sync
    pub max_pending_operations: usize,
    /// Enable presence tracking
    pub enable_presence: bool,
}

impl Default for CollabConfig {
    fn default() -> Self {
        Self {
            auto_resolve: true,
            max_pending_operations: 100,
            enable_presence: true,
        }
    }
}

/// Adapter that wraps a Shape to implement the Record trait for CRDT integration
///
/// This follows the **Adapter Pattern** to bridge the SDK's Shape type with
/// the CRDT system's Record trait without modifying the original Shape structure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollabRecord {
    /// Record ID for CRDT system
    pub record_id: RecordId,
    /// Internal shape data
    pub shape: Shape,
    /// Fractional index for conflict-free ordering
    pub index: Option<FractionalIndex>,
    /// Timestamp for Last-Writer-Wins conflict resolution
    pub timestamp: u64,
    /// Site ID that created/last modified this record
    pub site_id: SiteId,
}

impl CollabRecord {
    /// Creates a new CollabRecord from a Shape
    pub fn from_shape(shape: Shape) -> Self {
        let record_id = RecordId::from_u64(shape.id.as_u128() as u64);
        Self {
            record_id,
            shape,
            index: None,
            timestamp: 0,
            site_id: SiteId::default(),
        }
    }

    /// Creates a CollabRecord with explicit metadata
    pub fn new(
        shape: Shape,
        index: Option<FractionalIndex>,
        timestamp: u64,
        site_id: SiteId,
    ) -> Self {
        let record_id = RecordId::from_u64(shape.id.as_u128() as u64);
        Self {
            record_id,
            shape,
            index,
            timestamp,
            site_id,
        }
    }

    /// Updates the shape and timestamp
    pub fn update(&mut self, shape: Shape, timestamp: u64) {
        self.shape = shape;
        self.timestamp = timestamp;
    }
}

impl Record for CollabRecord {
    fn id(&self) -> &RecordId {
        // Convert EntityId to RecordId using from_u64
        // We need to store the RecordId, so let's add it to CollabRecord
        &self.record_id
    }

    fn type_name(&self) -> &'static str {
        match self.shape.shape_type {
            ShapeType::Rectangle => "rectangle",
            ShapeType::Ellipse => "ellipse",
            ShapeType::Line => "line",
            ShapeType::Path => "path",
            ShapeType::Text => "text",
            ShapeType::Image => "image",
            ShapeType::Group => "group",
        }
    }

    fn index(&self) -> Option<&FractionalIndex> {
        self.index.as_ref()
    }

    fn with_index(mut self, index: FractionalIndex) -> Self {
        self.index = Some(index);
        self
    }
}

/// Collaboration manager that handles CRDT operations and synchronization
///
/// This manager bridges the Canvas with the CRDT system, tracking local changes
/// and merging remote changes while maintaining consistency.
pub struct CollabManager {
    /// CRDT instance for conflict resolution
    crdt: CRDT<CollabRecord>,
    /// Local site identifier
    site_id: SiteId,
    /// Configuration
    config: CollabConfig,
    /// Track local changes since last sync
    local_changes: Vec<RecordChange<CollabRecord>>,
    /// Delta manager for undo/redo integration
    delta_manager: DeltaManager<CollabRecord>,
    /// Presence tracking (if enabled)
    presence: Option<PresenceManager>,
}

impl CollabManager {
    /// Creates a new collaboration manager
    pub fn new(config: CollabConfig) -> Self {
        let site_id = SiteId::new();
        let crdt = CRDT::new(site_id);
        let enable_presence = config.enable_presence;

        Self {
            crdt,
            site_id,
            config,
            local_changes: Vec::new(),
            delta_manager: DeltaManager::new(),
            presence: if enable_presence {
                Some(PresenceManager::new(site_id))
            } else {
                None
            },
        }
    }

    /// Returns the local site ID
    pub fn site_id(&self) -> SiteId {
        self.site_id
    }

    /// Returns the current vector clock
    pub fn vector_clock(&self) -> &VectorClock {
        self.crdt.vector_clock()
    }

    /// Tracks a shape creation operation
    pub fn track_shape_creation(&mut self, shape: Shape) -> Result<(), CollabError> {
        let record = CollabRecord::from_shape(shape);
        self.crdt.apply_local(record)?;
        self.local_changes = self.crdt.get_changes();
        Ok(())
    }

    /// Tracks a shape update operation
    pub fn track_shape_update(
        &mut self,
        old_shape: Shape,
        new_shape: Shape,
    ) -> Result<(), CollabError> {
        let record = CollabRecord::from_shape(new_shape);
        self.crdt.apply_local(record)?;
        self.local_changes = self.crdt.get_changes();
        Ok(())
    }

    /// Tracks a shape deletion operation
    pub fn track_shape_deletion(&mut self, shape: Shape) -> Result<(), CollabError> {
        let _id = RecordId::from_u64(shape.id.as_u128() as u64);
        // Create a tombstone record
        let record = CollabRecord::from_shape(shape);
        self.crdt.apply_local(record)?;
        self.local_changes = self.crdt.get_changes();
        Ok(())
    }

    /// Gets pending changes for synchronization
    pub fn get_pending_changes(&self) -> &[RecordChange<CollabRecord>] {
        &self.local_changes
    }

    /// Clears pending changes after successful synchronization
    pub fn clear_pending(&mut self) {
        self.crdt.clear_pending();
        self.local_changes.clear();
    }

    /// Applies remote changes from other collaborators
    pub fn apply_remote_changes(
        &mut self,
        remote_clock: &VectorClock,
        remote_records: Vec<CollabRecord>,
    ) -> Result<CollabMergeResult, CollabError> {
        let before_clock = self.crdt.vector_clock().clone();

        self.crdt.merge(remote_clock, remote_records)?;

        let conflicts = !matches!(
            before_clock.relation(remote_clock),
            archflow_collab::types::CausalRelation::HappenedBefore
                | archflow_collab::types::CausalRelation::Equal
        );

        Ok(CollabMergeResult {
            conflicts_resolved: conflicts,
            new_vector_clock: self.crdt.vector_clock().clone(),
        })
    }

    /// Gets the presence manager (if enabled)
    pub fn presence(&self) -> Option<&PresenceManager> {
        self.presence.as_ref()
    }

    /// Gets mutable presence manager (if enabled)
    pub fn presence_mut(&mut self) -> Option<&mut PresenceManager> {
        self.presence.as_mut()
    }
}

/// Result of a merge operation
#[derive(Clone, Debug, PartialEq)]
pub struct CollabMergeResult {
    /// Whether conflicts were resolved during the merge
    pub conflicts_resolved: bool,
    /// The new vector clock after the merge
    pub new_vector_clock: VectorClock,
}

/// Error types for collaboration operations
#[derive(Debug, thiserror::Error)]
pub enum CollabError {
    #[error("CRDT apply error: {0}")]
    ApplyError(#[from] archflow_collab::types::ApplyError),

    #[error("Shape not found: {0}")]
    ShapeNotFound(EntityId),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Sync error: {0}")]
    SyncError(String),
}

/// User presence information for collaborative editing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    /// Unique user identifier
    pub user_id: String,
    /// Display name
    pub display_name: String,
    /// User color for cursor/selection highlighting
    pub color: String,
}

/// Cursor position in canvas coordinates
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f32,
    pub y: f32,
}

impl CursorPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Selection state for a user
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserSelection {
    /// Selected shape IDs
    pub selected_ids: Vec<EntityId>,
    /// Selection bounds (if any)
    pub bounds: Option<(f32, f32, f32, f32)>,
}

/// User presence state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPresence {
    /// User information
    pub user_info: UserInfo,
    /// Current cursor position
    pub cursor: Option<CursorPosition>,
    /// Current selection
    pub selection: Option<UserSelection>,
    /// Last update timestamp
    pub last_update: u64,
}

/// Manages real-time presence for collaborative editing
pub struct PresenceManager {
    /// Local site ID
    site_id: SiteId,
    /// Local user info
    local_user: UserInfo,
    /// Local cursor position
    local_cursor: Option<CursorPosition>,
    /// Local selection
    local_selection: Option<UserSelection>,
    /// Remote users' presence
    remote_users: HashMap<SiteId, UserPresence>,
}

impl PresenceManager {
    /// Creates a new presence manager
    pub fn new(site_id: SiteId) -> Self {
        Self {
            site_id,
            local_user: UserInfo {
                user_id: uuid::Uuid::new_v4().to_string(),
                display_name: "Anonymous".to_string(),
                color: "#0066cc".to_string(),
            },
            local_cursor: None,
            local_selection: None,
            remote_users: HashMap::new(),
        }
    }

    /// Sets the local user information
    pub fn set_local_user(&mut self, user_info: UserInfo) {
        self.local_user = user_info;
    }

    /// Gets the local user information
    pub fn local_user(&self) -> &UserInfo {
        &self.local_user
    }

    /// Updates the local cursor position
    pub fn update_cursor(&mut self, position: CursorPosition) {
        self.local_cursor = Some(position);
    }

    /// Gets the local cursor position
    pub fn local_cursor(&self) -> Option<CursorPosition> {
        self.local_cursor
    }

    /// Updates the local selection
    pub fn update_selection(&mut self, selection: UserSelection) {
        self.local_selection = Some(selection);
    }

    /// Gets the local selection
    pub fn local_selection(&self) -> Option<&UserSelection> {
        self.local_selection.as_ref()
    }

    /// Updates a remote user's presence
    pub fn update_remote_presence(&mut self, site_id: SiteId, presence: UserPresence) {
        self.remote_users.insert(site_id, presence);
    }

    /// Removes a remote user
    pub fn remove_remote_user(&mut self, site_id: SiteId) {
        self.remote_users.remove(&site_id);
    }

    /// Gets all remote users
    pub fn remote_users(&self) -> &HashMap<SiteId, UserPresence> {
        &self.remote_users
    }

    /// Gets a specific remote user
    pub fn get_remote_user(&self, site_id: SiteId) -> Option<&UserPresence> {
        self.remote_users.get(&site_id)
    }

    /// Returns the count of connected users (excluding local)
    pub fn remote_user_count(&self) -> usize {
        self.remote_users.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;

    #[test]
    fn test_collab_config_default() {
        let config = CollabConfig::default();
        assert!(config.auto_resolve);
        assert_eq!(config.max_pending_operations, 100);
        assert!(config.enable_presence);
    }

    #[test]
    fn test_collab_manager_creation() {
        let manager = CollabManager::new(CollabConfig::default());
        // Site ID should be generated
        assert_ne!(manager.site_id().as_u32(), 0);
    }

    #[test]
    fn test_track_shape_creation() {
        let mut manager = CollabManager::new(CollabConfig::default());

        let shape = Shape::new_rectangle(10.0, 20.0, 100.0, 50.0);
        let result = manager.track_shape_creation(shape.clone());

        assert!(result.is_ok());
        assert!(!manager.get_pending_changes().is_empty());
    }

    #[test]
    fn test_clear_pending_changes() {
        let mut manager = CollabManager::new(CollabConfig::default());

        let shape = Shape::new_rectangle(10.0, 20.0, 100.0, 50.0);
        manager.track_shape_creation(shape).unwrap();

        assert!(!manager.get_pending_changes().is_empty());

        manager.clear_pending();
        assert!(manager.get_pending_changes().is_empty());
    }

    #[test]
    fn test_collab_record_from_shape() {
        let shape = Shape::new_rectangle(10.0, 20.0, 100.0, 50.0);
        let record = CollabRecord::from_shape(shape);

        assert_eq!(record.type_name(), "rectangle");
        assert!(record.index.is_none());
        assert_eq!(record.timestamp, 0);
    }

    #[test]
    fn test_presence_manager_creation() {
        let manager = PresenceManager::new(SiteId::new());
        assert_eq!(manager.remote_user_count(), 0);
        assert!(manager.local_user().display_name == "Anonymous");
    }

    #[test]
    fn test_presence_cursor_update() {
        let mut manager = PresenceManager::new(SiteId::new());

        assert!(manager.local_cursor().is_none());

        manager.update_cursor(CursorPosition::new(100.0, 200.0));

        let cursor = manager.local_cursor();
        assert!(cursor.is_some());
        assert_eq!(cursor.unwrap().x, 100.0);
        assert_eq!(cursor.unwrap().y, 200.0);
    }

    #[test]
    fn test_presence_remote_users() {
        let mut manager = PresenceManager::new(SiteId::new());

        let site_a = SiteId::new();
        let presence = UserPresence {
            user_info: UserInfo {
                user_id: "user_a".to_string(),
                display_name: "User A".to_string(),
                color: "#ff0000".to_string(),
            },
            cursor: Some(CursorPosition::new(50.0, 50.0)),
            selection: None,
            last_update: 12345,
        };

        manager.update_remote_presence(site_a, presence);

        assert_eq!(manager.remote_user_count(), 1);
        assert!(manager.get_remote_user(site_a).is_some());
    }

    #[test]
    fn test_presence_remove_remote_user() {
        let mut manager = PresenceManager::new(SiteId::new());

        let site_a = SiteId::new();
        let presence = UserPresence {
            user_info: UserInfo {
                user_id: "user_a".to_string(),
                display_name: "User A".to_string(),
                color: "#ff0000".to_string(),
            },
            cursor: None,
            selection: None,
            last_update: 0,
        };

        manager.update_remote_presence(site_a, presence);
        assert_eq!(manager.remote_user_count(), 1);

        manager.remove_remote_user(site_a);
        assert_eq!(manager.remote_user_count(), 0);
    }

    #[test]
    fn test_user_selection() {
        let mut manager = PresenceManager::new(SiteId::new());

        let selection = UserSelection {
            selected_ids: vec![EntityId::new(), EntityId::new()],
            bounds: Some((10.0, 10.0, 100.0, 100.0)),
        };

        manager.update_selection(selection);

        let local = manager.local_selection();
        assert!(local.is_some());
        assert_eq!(local.unwrap().selected_ids.len(), 2);
    }

    #[test]
    fn test_cursor_position() {
        let pos = CursorPosition::new(42.0, 99.0);
        assert_eq!(pos.x, 42.0);
        assert_eq!(pos.y, 99.0);
    }
}
