//! ECS Component for referencing Records
//!
//! This module provides components that link ECS entities to Records
//! and track synchronization state between the two data structures.

use archflow_records::RecordId;
use bevy_ecs::prelude::*;

/// Component that links an ECS entity to a Record.
///
/// Each ECS entity that represents a Record should have this component.
/// The sync system uses it to maintain bidirectional synchronization.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct RecordRef {
    /// ID of the Record this entity is linked to
    pub record_id: RecordId,

    /// Version of the Record when it was last synchronized
    pub synced_version: u64,

    /// Flag indicating if the entity has been modified locally
    pub dirty: bool,

    /// Timestamp of the last synchronization
    pub last_sync: std::time::Instant,
}

impl RecordRef {
    /// Creates a new RecordRef component.
    ///
    /// # Arguments
    ///
    /// * `record_id` - ID of the Record to link to
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_ecs_hybrid::RecordRef;
    /// use archflow_records::RecordId;
    /// use std::str::FromStr;
    ///
    /// let id = RecordId::from_str("record_1234567890").unwrap();
    /// let record_ref = RecordRef::new(id.clone());
    /// assert_eq!(record_ref.record_id, id);
    /// ```
    #[inline]
    pub fn new(record_id: RecordId) -> Self {
        Self {
            record_id,
            synced_version: 0,
            dirty: false,
            last_sync: std::time::Instant::now(),
        }
    }

    /// Marks this entity as dirty (needs synchronization to Record).
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Marks this entity as clean (synchronized to Record).
    ///
    /// # Arguments
    ///
    /// * `version` - Current version of the Record
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
        self.last_sync = std::time::Instant::now();
    }

    /// Updates the synchronized version and marks as clean.
    ///
    /// # Arguments
    ///
    /// * `version` - New version number from Record
    #[inline]
    pub fn update_version(&mut self, version: u64) {
        self.synced_version = version;
        self.clear_dirty();
    }

    /// Checks if this entity needs synchronization.
    ///
    /// # Arguments
    ///
    /// * `current_version` - Current version of the Record
    ///
    /// # Returns
    ///
    /// `true` if the entity is dirty or the Record version has changed
    #[inline]
    pub fn needs_sync(&self, current_version: u64) -> bool {
        self.dirty || self.synced_version < current_version
    }

    /// Returns the elapsed time since the last synchronization.
    #[inline]
    pub fn time_since_sync(&self) -> std::time::Duration {
        self.last_sync.elapsed()
    }
}

/// Component that indicates an ECS entity has changed.
///
/// Used by the sync system to track what type of change occurred.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Dirty {
    /// Type of change that occurred
    pub change_type: DirtyType,
}

impl Dirty {
    /// Creates a Dirty component for created entities.
    #[inline]
    pub fn created() -> Self {
        Self {
            change_type: DirtyType::Created,
        }
    }

    /// Creates a Dirty component for updated entities.
    #[inline]
    pub fn updated() -> Self {
        Self {
            change_type: DirtyType::Updated,
        }
    }

    /// Creates a Dirty component for deleted entities.
    #[inline]
    pub fn deleted() -> Self {
        Self {
            change_type: DirtyType::Deleted,
        }
    }

    /// Creates a Dirty component for entities with transform changes.
    #[inline]
    pub fn transform_changed() -> Self {
        Self {
            change_type: DirtyType::TransformChanged,
        }
    }
}

/// Types of changes that can occur to ECS entities.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum DirtyType {
    /// Entity was newly created
    Created,
    /// Entity was modified
    Updated,
    /// Entity was deleted
    Deleted,
    /// Entity's transform changed
    TransformChanged,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_record_ref_creation() {
        let id = RecordId::from_str("ref_test_001").unwrap();
        let ref_comp = RecordRef::new(id.clone());
        assert_eq!(ref_comp.record_id, id);
        assert!(!ref_comp.dirty);
        assert_eq!(ref_comp.synced_version, 0);
    }

    #[test]
    fn test_mark_dirty() {
        let id = RecordId::from_str("dirty_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);
        ref_comp.mark_dirty();
        assert!(ref_comp.dirty);
    }

    #[test]
    fn test_clear_dirty() {
        let id = RecordId::from_str("clear_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);
        ref_comp.mark_dirty();
        assert!(ref_comp.dirty);
        ref_comp.clear_dirty();
        assert!(!ref_comp.dirty);
    }

    #[test]
    fn test_update_version() {
        let id = RecordId::from_str("version_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);
        ref_comp.update_version(42);
        assert_eq!(ref_comp.synced_version, 42);
        assert!(!ref_comp.dirty);
    }

    #[test]
    fn test_needs_sync() {
        let id = RecordId::from_str("sync_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);

        // Should not need sync initially
        assert!(!ref_comp.needs_sync(0));

        // Should need sync when dirty
        ref_comp.mark_dirty();
        assert!(ref_comp.needs_sync(0));

        // Should need sync when record version increases
        ref_comp.clear_dirty();
        assert!(ref_comp.needs_sync(10));

        // Should not need sync after update
        ref_comp.update_version(10);
        assert!(!ref_comp.needs_sync(10));
    }

    #[test]
    fn test_time_since_sync() {
        let id = RecordId::from_str("time_test_001").unwrap();
        let ref_comp = RecordRef::new(id);

        let elapsed = ref_comp.time_since_sync();
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_dirty_created() {
        let dirty = Dirty::created();
        assert_eq!(dirty.change_type, DirtyType::Created);
    }

    #[test]
    fn test_dirty_updated() {
        let dirty = Dirty::updated();
        assert_eq!(dirty.change_type, DirtyType::Updated);
    }

    #[test]
    fn test_dirty_deleted() {
        let dirty = Dirty::deleted();
        assert_eq!(dirty.change_type, DirtyType::Deleted);
    }

    #[test]
    fn test_dirty_transform_changed() {
        let dirty = Dirty::transform_changed();
        assert_eq!(dirty.change_type, DirtyType::TransformChanged);
    }
}
