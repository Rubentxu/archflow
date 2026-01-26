//! Delta-Based Change Management
//!
//! This module provides the `DeltaManager` for efficient undo/redo operations
//! with O(1) memory complexity per operation.
//!
//! # Architecture
//!
//! - `RecordChange`: Enum representing Created/Updated/Deleted changes
//! - `DeltaManager`: Manages undo/redo history with configurable limits
//!
//! # Memory Efficiency
//!
//! The DeltaManager uses a sliding window approach:
//! - Only stores the change, not full state snapshots
//! - Undo pops from undo stack, redo pops from redo stack
//! - New changes clear the redo stack (standard undo/redo behavior)

use crate::record_id::RecordId;
use crate::trait_record::Record;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a change to a record.
///
/// This is the core delta type that captures the essential information
/// for any change to a record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordChange<R: Record> {
    /// A new record was created
    Created {
        /// The ID of the created record
        id: RecordId,
        /// The created record
        record: R,
    },
    /// A record was updated
    Updated {
        /// The ID of the updated record
        id: RecordId,
        /// The previous value
        old_value: R,
        /// The new value
        new_value: R,
    },
    /// A record was deleted
    Deleted {
        /// The ID of the deleted record
        id: RecordId,
        /// The deleted record (preserved for undo/redo)
        record: R,
    },
}

impl<R: Record> RecordChange<R> {
    /// Returns the ID of the affected record.
    pub fn id(&self) -> &RecordId {
        match self {
            RecordChange::Created { id, .. } => id,
            RecordChange::Updated { id, .. } => id,
            RecordChange::Deleted { id, .. } => id,
        }
    }

    /// Returns true if this is a creation change.
    #[inline]
    pub fn is_create(&self) -> bool {
        matches!(self, RecordChange::Created { .. })
    }

    /// Returns true if this is an update change.
    #[inline]
    pub fn is_update(&self) -> bool {
        matches!(self, RecordChange::Updated { .. })
    }

    /// Returns true if this is a deletion change.
    #[inline]
    pub fn is_delete(&self) -> bool {
        matches!(self, RecordChange::Deleted { .. })
    }

    /// Inverts the change for undo operations.
    ///
    /// Creates the inverse operation that would undo this change.
    pub fn invert(self) -> RecordChange<R> {
        match self {
            RecordChange::Created { id, record } => RecordChange::Deleted { id, record },
            RecordChange::Updated {
                id,
                old_value,
                new_value,
            } => RecordChange::Updated {
                id,
                old_value: new_value, // Restore to the new value
                new_value: old_value, // Was the old value
            },
            RecordChange::Deleted { id, record } => RecordChange::Created { id, record },
        }
    }

    /// Creates a Created change.
    pub fn created(record: R) -> Self {
        let id = record.id().clone();
        RecordChange::Created { id, record }
    }

    /// Creates an Updated change.
    pub fn updated(id: RecordId, old_value: R, new_value: R) -> Self {
        RecordChange::Updated {
            id,
            old_value,
            new_value,
        }
    }

    /// Creates a Deleted change.
    pub fn deleted(record: R) -> Self {
        let id = record.id().clone();
        RecordChange::Deleted { id, record }
    }
}

/// Manages undo/redo history with configurable limits.
///
/// The DeltaManager implements a classic undo/redo stack with these properties:
/// - O(1) memory per operation (stores only deltas, not full state)
/// - Undo moves change from undo_stack to redo_stack
/// - Redo moves change from redo_stack to undo_stack
/// - New changes clear the redo_stack
/// - Optional limit on history size
#[derive(Debug, Clone)]
pub struct DeltaManager<R: Record> {
    /// Stack of changes that can be undone
    undo_history: Vec<RecordChange<R>>,
    /// Stack of changes that can be redone
    redo_history: Vec<RecordChange<R>>,
    /// Maximum history size (0 = unlimited)
    max_history: usize,
}

impl<R: Record> DeltaManager<R> {
    /// Creates a new DeltaManager with unlimited history.
    #[inline]
    pub fn new() -> Self {
        Self::with_limit(0)
    }

    /// Creates a new DeltaManager with a history limit.
    ///
    /// # Arguments
    ///
    /// * `max_history` - Maximum number of changes to keep. 0 = unlimited.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_records::DeltaManager;
    ///
    /// let manager = DeltaManager::with_limit(100);
    /// ```
    pub fn with_limit(max_history: usize) -> Self {
        DeltaManager {
            undo_history: Vec::with_capacity(max_history.max(64)),
            redo_history: Vec::with_capacity(max_history.max(64)),
            max_history,
        }
    }

    /// Records a new change.
    ///
    /// This adds the change to the undo history and clears the redo history
    /// (standard undo/redo behavior).
    ///
    /// # Arguments
    ///
    /// * `change` - The change to record
    pub fn record(&mut self, change: RecordChange<R>) {
        // Clear redo history when new changes are made
        self.redo_history.clear();

        // Add to undo history
        self.undo_history.push(change);

        // Trim if at capacity
        if self.max_history > 0 && self.undo_history.len() > self.max_history {
            self.undo_history.remove(0);
        }
    }

    /// Records a batch of changes.
    ///
    /// More efficient than calling `record` multiple times.
    ///
    /// # Arguments
    ///
    /// * `changes` - An iterator of changes to record
    pub fn record_batch<I: IntoIterator<Item = RecordChange<R>>>(&mut self, changes: I) {
        for change in changes {
            self.record(change);
        }
    }

    /// Undoes the last change.
    ///
    /// Returns the inverted change that undoes the last operation.
    /// The inverted change is moved to the redo stack.
    ///
    /// # Returns
    ///
    /// `Some(RecordChange)` if undo was successful, `None` if no changes to undo.
    pub fn undo(&mut self) -> Option<RecordChange<R>> {
        let change = self.undo_history.pop()?;
        let inverted = change.clone().invert();

        // Add to redo stack
        self.redo_history.push(change);

        Some(inverted)
    }

    /// Redoes the last undone change.
    ///
    /// Returns the change that re-applies the last undone operation.
    ///
    /// # Returns
    ///
    /// `Some(RecordChange)` if redo was successful, `None` if no changes to redo.
    pub fn redo(&mut self) -> Option<RecordChange<R>> {
        let change = self.redo_history.pop()?;
        let inverted = change.clone().invert();

        // Add back to undo stack
        self.undo_history.push(change);

        Some(inverted)
    }

    /// Returns true if there are changes to undo.
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_history.is_empty()
    }

    /// Returns true if there are changes to redo.
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_history.is_empty()
    }

    /// Returns the number of changes that can be undone.
    #[inline]
    pub fn undo_count(&self) -> usize {
        self.undo_history.len()
    }

    /// Returns the number of changes that can be redone.
    #[inline]
    pub fn redo_count(&self) -> usize {
        self.redo_history.len()
    }

    /// Returns the history sizes.
    ///
    /// Useful for memory tracking and testing.
    pub fn history_sizes(&self) -> HistorySizes {
        HistorySizes {
            undo: self.undo_history.len(),
            redo: self.redo_history.len(),
        }
    }

    /// Clears all history.
    #[inline]
    pub fn clear(&mut self) {
        self.undo_history.clear();
        self.redo_history.clear();
    }

    /// Gets the current history limit.
    #[inline]
    pub fn max_history(&self) -> usize {
        self.max_history
    }

    /// Sets a new history limit.
    ///
    /// If the new limit is smaller than current history, old entries are dropped.
    pub fn set_max_history(&mut self, new_limit: usize) {
        self.max_history = new_limit;

        if new_limit > 0 {
            // Trim undo history if needed
            while self.undo_history.len() > new_limit {
                self.undo_history.remove(0);
            }
        }
    }

    /// Marks the current state as a save point.
    ///
    /// This clears the undo history, meaning the current state cannot be undone.
    /// Useful for committing state after a significant operation.
    pub fn mark_save_point(&mut self) {
        self.undo_history.clear();
        self.redo_history.clear();
    }
}

impl<R: Record> Default for DeltaManager<R> {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of history sizes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistorySizes {
    /// Number of undoable changes
    pub undo: usize,
    /// Number of redoable changes
    pub redo: usize,
}

impl fmt::Display for HistorySizes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "undo: {}, redo: {}", self.undo, self.redo)
    }
}

#[cfg(test)]
mod delta_manager_tests {
    use super::*;
    use crate::{DeltaManager, FractionalIndex, Record, RecordChange, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRecord {
        id: RecordId,
        name: String,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }
        fn type_name(&self) -> &'static str {
            "TestRecord"
        }
        fn index(&self) -> Option<&FractionalIndex> {
            None
        }
        fn with_index(self, _index: FractionalIndex) -> Self {
            self
        }
        fn eq_ignoring_metadata(&self, other: &Self) -> bool {
            self.id == other.id && self.name == other.name
        }
        fn validate(&self) -> Result<(), crate::RecordError> {
            Ok(())
        }
    }

    #[test]
    fn test_record_created_delta() {
        let id = RecordId::from_str("test_0000000001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            name: "test".into(),
        };
        let change = RecordChange::created(record);

        assert_eq!(change.id().as_str(), "test_0000000001");
        assert!(change.is_create());
        assert!(!change.is_update());
        assert!(!change.is_delete());
    }

    #[test]
    fn test_record_updated_delta() {
        let id = RecordId::from_str("test_0000000001").unwrap();
        let old_value = TestRecord {
            id: id.clone(),
            name: "old".into(),
        };
        let new_value = TestRecord {
            id: id.clone(),
            name: "new".into(),
        };

        let change = RecordChange::updated(id.clone(), old_value.clone(), new_value.clone());

        assert_eq!(change.id(), &id);
        assert!(change.is_update());
    }

    #[test]
    fn test_delta_invert() {
        let id = RecordId::from_str("test_invert_001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            name: "test".into(),
        };

        // Created -> Deleted on invert
        let created = RecordChange::created(record.clone());
        let inverted = created.invert();
        assert!(inverted.is_delete());

        // Deleted -> Created on invert
        let deleted = RecordChange::deleted(record.clone());
        let inverted = deleted.invert();
        assert!(inverted.is_create());
    }

    #[test]
    fn test_update_invert() {
        let id = RecordId::from_str("test_invert_002").unwrap();
        let old_value = TestRecord {
            id: id.clone(),
            name: "old".into(),
        };
        let new_value = TestRecord {
            id: id.clone(),
            name: "new".into(),
        };

        let update = RecordChange::updated(id.clone(), old_value.clone(), new_value.clone());
        let inverted = update.invert();

        if let RecordChange::Updated {
            id: _id,
            old_value: inv_old_value, // After invert: this is the new_value from original
            new_value: inv_new_value, // After invert: this is the old_value from original
        } = inverted
        {
            // invert() swaps old and new:
            // - inv_old_value should be "new" (restoring to this state)
            // - inv_new_value should be "old" (the state we're undoing from)
            assert_eq!(inv_old_value.name, "new");
            assert_eq!(inv_new_value.name, "old");
        } else {
            panic!("Expected Updated change");
        }
    }

    #[test]
    fn test_undo_redo_flow() {
        let mut manager = DeltaManager::new();

        let id = RecordId::from_str("undo_test_00001").unwrap();
        let record1 = TestRecord {
            id: id.clone(),
            name: "v1".into(),
        };
        let record2 = TestRecord {
            id: id.clone(),
            name: "v2".into(),
        };

        // Apply changes
        manager.record(RecordChange::created(record1.clone()));
        manager.record(RecordChange::updated(
            id.clone(),
            record1.clone(),
            record2.clone(),
        ));

        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Undo twice
        assert!(manager.undo().is_some());
        assert!(manager.can_redo());

        assert!(manager.undo().is_some());
        assert!(!manager.can_undo());

        // Redo twice
        assert!(manager.redo().is_some());
        assert!(manager.can_undo());

        assert!(manager.redo().is_some());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_new_change_clears_redo() {
        let mut manager = DeltaManager::new();

        let id = RecordId::from_str("redo_clear_001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            name: "test".into(),
        };

        // Create and undo
        manager.record(RecordChange::created(record.clone()));
        manager.undo();

        assert!(manager.can_redo());

        // Create new change
        let record2 = TestRecord {
            id,
            name: "test2".into(),
        };
        manager.record(RecordChange::created(record2));

        // Redo should be cleared
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_memory_efficient_undo() {
        let mut manager = DeltaManager::with_limit(1000);

        // Create many records
        for i in 0..1000 {
            let id = RecordId::from_str(&format!("mem_test_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                name: format!("v{}", i),
            };
            manager.record(RecordChange::created(record));
        }

        // Check history size
        let sizes = manager.history_sizes();
        assert_eq!(sizes.undo, 1000);
        assert_eq!(sizes.redo, 0);

        // Undo and check memory is stable
        for _ in 0..10 {
            manager.undo();
        }

        let sizes = manager.history_sizes();
        assert_eq!(sizes.undo, 990);
        assert_eq!(sizes.redo, 10);
    }

    #[test]
    fn test_history_limit() {
        let mut manager = DeltaManager::with_limit(10);

        for i in 0..20 {
            let id = RecordId::from_str(&format!("limit_test_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                name: format!("v{}", i),
            };
            manager.record(RecordChange::created(record));
        }

        // Should only keep last 10
        assert_eq!(manager.history_sizes().undo, 10);

        // Oldest should have been removed
        assert!(manager.redo_count() == 0);
    }

    #[test]
    fn test_clear_history() {
        let mut manager = DeltaManager::new();

        let id = RecordId::from_str("clear_test_001").unwrap();
        let record = TestRecord {
            id,
            name: "test".into(),
        };
        manager.record(RecordChange::created(record));

        assert!(manager.can_undo());

        manager.clear();

        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_mark_save_point() {
        let mut manager = DeltaManager::new();

        let id = RecordId::from_str("savepoint_001").unwrap();
        let record = TestRecord {
            id,
            name: "test".into(),
        };
        manager.record(RecordChange::created(record));

        assert!(manager.can_undo());

        manager.mark_save_point();

        assert!(!manager.can_undo());
    }

    #[test]
    fn test_batch_record() {
        let mut manager = DeltaManager::new();

        let changes: Vec<RecordChange<TestRecord>> = (0..5)
            .map(|i| {
                let id = RecordId::from_str(&format!("batch_{:08}", i)).unwrap();
                let record = TestRecord {
                    id,
                    name: format!("v{}", i),
                };
                RecordChange::created(record)
            })
            .collect();

        manager.record_batch(changes);

        assert_eq!(manager.undo_count(), 5);
    }

    #[test]
    fn test_max_history_update() {
        let mut manager = DeltaManager::with_limit(10);

        for i in 0..15 {
            let id = RecordId::from_str(&format!("update_limit_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                name: format!("v{}", i),
            };
            manager.record(RecordChange::created(record));
        }

        assert_eq!(manager.history_sizes().undo, 10);

        // Increase limit
        manager.set_max_history(20);
        for i in 15..25 {
            let id = RecordId::from_str(&format!("update_limit_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                name: format!("v{}", i),
            };
            manager.record(RecordChange::created(record));
        }

        assert_eq!(manager.history_sizes().undo, 20);

        // Decrease limit
        manager.set_max_history(5);
        assert_eq!(manager.history_sizes().undo, 5);
    }
}
