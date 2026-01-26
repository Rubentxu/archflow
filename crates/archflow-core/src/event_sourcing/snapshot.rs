//! Snapshot - State snapshots for performance optimization
//!
//! Provides periodic state snapshots to avoid replaying all events
//! when rebuilding aggregate state.

use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Error types for snapshot operations
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SnapshotError {
    /// No snapshot found
    #[error("No snapshot found")]
    NotFound,
    /// Invalid snapshot format
    #[error("Invalid snapshot format")]
    InvalidFormat,
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// ID of the aggregate this snapshot belongs to
    pub aggregate_id: EntityId,
    /// Version of the aggregate at snapshot time
    pub version: u64,
    /// Number of events since last snapshot
    pub events_since_last: usize,
    /// When this snapshot was created
    pub created_at: SystemTime,
    /// Schema version for forward compatibility
    pub schema_version: u32,
    /// Optional human-readable description
    pub description: String,
}

impl SnapshotMetadata {
    /// Create new metadata for a snapshot
    pub fn new(
        aggregate_id: EntityId,
        version: u64,
        events_since_last: usize,
        description: String,
    ) -> Self {
        Self {
            aggregate_id,
            version,
            events_since_last,
            created_at: SystemTime::now(),
            schema_version: 1,
            description,
        }
    }
}

/// A snapshot of aggregate state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    /// Serialized state (JSON)
    pub state: Vec<u8>,
    /// Checksum for integrity verification
    pub checksum: u32,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(aggregate_id: EntityId, version: u64, state: Vec<u8>, description: String) -> Self {
        let events_since_last = 0;
        let checksum = Self::calculate_checksum(&state);

        Self {
            metadata: SnapshotMetadata::new(aggregate_id, version, events_since_last, description),
            state,
            checksum,
        }
    }

    /// Verify the snapshot integrity
    pub fn verify(&self) -> bool {
        self.checksum == Self::calculate_checksum(&self.state)
    }

    /// Calculate a simple checksum for the state
    fn calculate_checksum(state: &[u8]) -> u32 {
        state.iter().map(|b| *b as u32).sum()
    }

    /// Get the aggregate ID
    pub fn aggregate_id(&self) -> EntityId {
        self.metadata.aggregate_id
    }

    /// Get the version
    pub fn version(&self) -> u64 {
        self.metadata.version
    }
}

/// Manager for creating and restoring snapshots
#[derive(Debug, Clone, Default)]
pub struct SnapshotManager {
    /// Snapshots organized by aggregate ID
    snapshots: std::collections::HashMap<EntityId, Snapshot>,
    /// Configuration for snapshot creation
    threshold: usize,
    /// Track events since last snapshot per aggregate
    event_counts: std::collections::HashMap<EntityId, usize>,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(threshold: usize) -> Self {
        Self {
            snapshots: std::collections::HashMap::new(),
            threshold,
            event_counts: std::collections::HashMap::new(),
        }
    }

    /// Create with default threshold (1000 events)
    pub fn default() -> Self {
        Self::new(1000)
    }

    /// Record an event for an aggregate
    pub fn record_event(&mut self, aggregate_id: EntityId) -> bool {
        let count = self.event_counts.entry(aggregate_id).or_insert(0);
        *count += 1;
        *count >= self.threshold
    }

    /// Create a snapshot for an aggregate
    pub fn create_snapshot(
        &mut self,
        aggregate_id: EntityId,
        version: u64,
        state: Vec<u8>,
        description: String,
    ) -> Result<Snapshot, SnapshotError> {
        let snapshot = Snapshot::new(aggregate_id, version, state, description);

        let last_events = self.event_counts.get(&aggregate_id).copied().unwrap_or(0);

        let mut snap = snapshot.clone();
        snap.metadata.events_since_last = last_events;

        self.snapshots.insert(aggregate_id, snap.clone());
        self.event_counts.insert(aggregate_id, 0);

        Ok(snap)
    }

    /// Restore a snapshot for an aggregate
    pub fn restore_snapshot(&self, aggregate_id: EntityId) -> Result<Snapshot, SnapshotError> {
        self.snapshots
            .get(&aggregate_id)
            .cloned()
            .ok_or(SnapshotError::NotFound)
    }

    /// Check if a snapshot exists for an aggregate
    pub fn has_snapshot(&self, aggregate_id: &EntityId) -> bool {
        self.snapshots.contains_key(aggregate_id)
    }

    /// Get the version of the latest snapshot
    pub fn latest_version(&self, aggregate_id: &EntityId) -> Option<u64> {
        self.snapshots.get(aggregate_id).map(|s| s.version())
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&mut self, aggregate_id: &EntityId) -> bool {
        self.snapshots.remove(aggregate_id).is_some()
    }

    /// Get the threshold
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Set a new threshold
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
    }

    /// Get all snapshot metadata
    pub fn all_snapshots(&self) -> Vec<(&EntityId, &Snapshot)> {
        self.snapshots.iter().collect()
    }

    /// Get the number of snapshots
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if there are no snapshots
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Clear all snapshots
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.event_counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        value: i32,
        name: String,
    }

    #[test]
    fn test_create_snapshot() {
        let mut manager = SnapshotManager::new(100);
        let aggregate_id = EntityId::from_u128(1);

        let state = vec![1u8, 2, 3, 4, 5];
        let snapshot = manager
            .create_snapshot(aggregate_id, 100, state, "Test snapshot".to_string())
            .unwrap();

        assert_eq!(snapshot.version(), 100);
        assert!(snapshot.verify());
    }

    #[test]
    fn test_restore_snapshot() {
        let mut manager = SnapshotManager::new(100);
        let aggregate_id = EntityId::from_u128(1);

        let state = vec![1u8, 2, 3, 4, 5];
        manager
            .create_snapshot(aggregate_id, 100, state, "Test".to_string())
            .unwrap();

        let restored = manager.restore_snapshot(aggregate_id).unwrap();
        assert_eq!(restored.version(), 100);
        assert_eq!(restored.state, vec![1u8, 2, 3, 4, 5]);
    }

    #[test]
    fn test_threshold_reached() {
        let mut manager = SnapshotManager::new(3);

        let aggregate_id = EntityId::from_u128(1);

        assert!(!manager.record_event(aggregate_id)); // 1
        assert!(!manager.record_event(aggregate_id)); // 2
        assert!(manager.record_event(aggregate_id)); // 3 - threshold reached
    }

    #[test]
    fn test_snapshot_verification() {
        let aggregate_id = EntityId::from_u128(1);
        let state = vec![1u8, 2, 3, 4, 5];

        let snapshot = Snapshot::new(aggregate_id, 100, state, "Test".to_string());
        assert!(snapshot.verify());

        // Tamper with the state
        let mut tampered = snapshot.clone();
        tampered.state[0] = 99;
        assert!(!tampered.verify());
    }

    #[test]
    fn test_delete_snapshot() {
        let mut manager = SnapshotManager::new(100);
        let aggregate_id = EntityId::from_u128(1);

        manager
            .create_snapshot(aggregate_id, 100, vec![], "Test".to_string())
            .unwrap();

        assert!(manager.has_snapshot(&aggregate_id));

        assert!(manager.delete_snapshot(&aggregate_id));
        assert!(!manager.has_snapshot(&aggregate_id));
    }
}

#[cfg(test)]
mod journal_tests {
    use super::*;
    use crate::event_sourcing::event::DomainEvent;
    use crate::event_sourcing::event::EventMetadata;
    use crate::event_sourcing::event_journal::{
        EventJournal, JournalEntry, JournalError, UndoRedoStack,
    };

    #[test]
    fn test_undo_redo_stack() {
        let mut stack = UndoRedoStack::new(10);
        let doc_id = EntityId::from_u128(1);

        let entry = JournalEntry::new(
            vec![DomainEvent::PrimitiveCreated {
                metadata: EventMetadata::for_new_aggregate(doc_id, doc_id),
                primitive_id: EntityId::from_u128(100),
                primitive_type: "rectangle".to_string(),
                position: (0.0, 0.0),
                size: (100.0, 50.0),
            }],
            "Created rectangle".to_string(),
        );

        // Push entry
        stack.push(entry.clone());
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        // Undo
        let undone = stack.undo().unwrap();
        assert!(!stack.can_undo());
        assert!(stack.can_redo());
        assert_eq!(undone.inverse_events.len(), 1);

        // Redo
        let redone = stack.redo().unwrap();
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(redone.events.len(), 1);
    }

    #[test]
    fn test_undo_redo_stack_limits() {
        let mut stack = UndoRedoStack::new(3);
        let doc_id = EntityId::from_u128(1);

        for i in 0..5 {
            let entry = JournalEntry::new(
                vec![DomainEvent::PrimitiveCreated {
                    metadata: EventMetadata::new(
                        doc_id,
                        (i + 1) as u64,
                        doc_id,
                        "Created".to_string(),
                    ),
                    primitive_id: EntityId::from_u128(i),
                    primitive_type: "rectangle".to_string(),
                    position: (0.0, 0.0),
                    size: (100.0, 50.0),
                }],
                format!("Created {}", i),
            );
            stack.push(entry);
        }

        // Should only have 3 entries due to limit
        assert!(stack.can_undo());
        assert_eq!(stack.undo().unwrap().events.len(), 1);
    }

    #[test]
    fn test_event_journal() {
        let doc_id = EntityId::from_u128(1);
        let mut journal = EventJournal::new(doc_id, 0);

        // Record an event
        let event = DomainEvent::PrimitiveCreated {
            metadata: EventMetadata::for_new_aggregate(doc_id, doc_id),
            primitive_id: EntityId::from_u128(100),
            primitive_type: "rectangle".to_string(),
            position: (0.0, 0.0),
            size: (100.0, 50.0),
        };

        journal
            .record(vec![event], "Created rectangle".to_string())
            .unwrap();
        assert_eq!(journal.current_version(), 1);
        assert!(journal.can_undo());
        assert!(!journal.can_redo());

        // Undo
        let entry = journal.undo().unwrap();
        assert_eq!(journal.current_version(), 0);
        assert!(!journal.can_undo());
        assert!(journal.can_redo());
        assert_eq!(entry.description, "Created rectangle");

        // Redo
        // Redo
        let _entry = journal.redo().unwrap();
        assert_eq!(journal.current_version(), 1);
        assert!(journal.can_undo());
        assert!(!journal.can_redo());
    }

    #[test]
    fn test_event_journal_snapshot() {
        let doc_id = EntityId::from_u128(1);
        let mut journal = EventJournal::new(doc_id, 5);

        // Record some events
        for i in 1..=3 {
            let event = DomainEvent::PrimitiveMoved {
                metadata: EventMetadata::new(doc_id, 5 + i, doc_id, "Moved".to_string()),
                primitive_id: EntityId::from_u128(100),
                from: (0.0, 0.0),
                to: (i as f32 * 10.0, 0.0),
            };
            journal.record(vec![event], format!("Move {}", i)).unwrap();
        }

        assert_eq!(journal.current_version(), 8);
        assert_eq!(journal.changes_since_snapshot(), 3);

        // Mark snapshot
        journal.mark_snapshot();
        assert_eq!(journal.snapshot_version(), 8);
        assert_eq!(journal.changes_since_snapshot(), 0);
        assert!(!journal.can_undo()); // Stack cleared
    }

    #[test]
    fn test_journal_empty_operations() {
        let _doc_id = EntityId::from_u128(1);
        let mut stack = UndoRedoStack::new(10);

        // Cannot undo empty stack
        assert!(matches!(stack.undo(), Err(JournalError::NoEventsToUndo)));
        // Cannot redo empty stack
        assert!(matches!(stack.redo(), Err(JournalError::NoEventsToRedo)));
    }

    #[test]
    fn test_journal_record_empty() {
        let doc_id = EntityId::from_u128(1);
        let mut journal = EventJournal::new(doc_id, 0);

        // Recording empty events should succeed but not change version
        journal.record(vec![], "Empty".to_string()).unwrap();
        assert_eq!(journal.current_version(), 0);
    }
}
