//! Record Store with ChangeSet Optimization
//!
//! This module provides `RecordStore`, the central storage component for records
//! with efficient change tracking using `FixedBitSet`.
//!
//! # Architecture
//!
//! - `RecordStore`: Central storage with BTreeMap + spatial index
//! - `ChangeSet`: Optimized change representation using FixedBitSet
//! - `IndexMapper`: Maps record IDs to bit positions
//!
//! # Performance Characteristics
//!
//! - Insert: O(log N)
//! - Get: O(log N)
//! - Change detection: O(1) per record
//! - Change iteration: O(C) where C = number of changed records

use crate::delta::{DeltaManager, RecordChange};
use crate::fractional_index::FractionalIndex;
use crate::record_id::RecordId;
use crate::trait_record::Record;
use fixedbitset::FixedBitSet;
use std::collections::BTreeMap;

/// Optimized change representation.
///
/// Uses dynamic sizing to accommodate any number of records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChangeSet {
    /// Bit set marking updated records
    updated: FixedBitSet,
    /// Bit set marking created records
    created: FixedBitSet,
    /// List of deleted record IDs
    deleted: Vec<RecordId>,
    /// Current capacity
    capacity: usize,
}

impl ChangeSet {
    /// Creates a new empty ChangeSet.
    #[inline]
    pub fn new() -> Self {
        Self {
            updated: FixedBitSet::with_capacity(64),
            created: FixedBitSet::with_capacity(64),
            deleted: Vec::new(),
            capacity: 64,
        }
    }

    /// Creates a ChangeSet with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            updated: FixedBitSet::with_capacity(capacity),
            created: FixedBitSet::with_capacity(capacity),
            deleted: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Ensures the bitset has enough capacity for the given index.
    fn ensure_capacity(&mut self, index: usize) {
        if index >= self.capacity {
            // Grow the bitsets
            let new_capacity = (index + 1).max(self.capacity * 2);
            let mut new_updated = FixedBitSet::with_capacity(new_capacity);
            let mut new_created = FixedBitSet::with_capacity(new_capacity);

            // Copy existing bits
            for i in 0..self.capacity {
                if self.updated.contains(i) {
                    new_updated.set(i, true);
                }
                if self.created.contains(i) {
                    new_created.set(i, true);
                }
            }

            self.updated = new_updated;
            self.created = new_created;
            self.capacity = new_capacity;
        }
    }

    /// Marks a record as updated.
    ///
    /// # Arguments
    ///
    /// * `index` - Bit position for the record
    #[inline]
    pub fn mark_updated(&mut self, index: usize) {
        self.ensure_capacity(index);
        self.updated.set(index, true);
    }

    /// Marks a record as created.
    ///
    /// # Arguments
    ///
    /// * `index` - Bit position for the record
    #[inline]
    pub fn mark_created(&mut self, index: usize) {
        self.ensure_capacity(index);
        self.created.set(index, true);
    }

    /// Marks a record as deleted.
    ///
    /// # Arguments
    ///
    /// * `id` - Record ID of the deleted record
    #[inline]
    pub fn mark_deleted(&mut self, id: RecordId) {
        self.deleted.push(id);
    }

    /// Returns the number of created records.
    #[inline]
    pub fn created_count(&self) -> usize {
        self.created.count_ones(..)
    }

    /// Returns the number of updated records.
    #[inline]
    pub fn updated_count(&self) -> usize {
        self.updated.count_ones(..)
    }

    /// Returns the number of deleted records.
    #[inline]
    pub fn deleted_count(&self) -> usize {
        self.deleted.len()
    }

    /// Returns the total number of changes.
    #[inline]
    pub fn change_count(&self) -> usize {
        self.created_count() + self.updated_count() + self.deleted_count()
    }

    /// Returns true if there are no changes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.created_count() == 0 && self.updated_count() == 0 && self.deleted.is_empty()
    }

    /// Clears all changes.
    #[inline]
    pub fn clear(&mut self) {
        self.updated.clear();
        self.created.clear();
        self.deleted.clear();
    }

    /// Returns an iterator over created record indices.
    #[inline]
    pub fn created_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.created.ones()
    }

    /// Returns an iterator over updated record indices.
    #[inline]
    pub fn updated_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.updated.ones()
    }

    /// Returns an iterator over deleted record IDs.
    #[inline]
    pub fn deleted_ids(&self) -> impl Iterator<Item = &RecordId> + '_ {
        self.deleted.iter()
    }

    /// Merges another ChangeSet into this one.
    pub fn merge(&mut self, other: &ChangeSet) {
        // Ensure capacity for other's indices
        let other_max = other.capacity;
        self.ensure_capacity(other_max.saturating_sub(1));

        // Merge updated bits (created records shouldn't be marked updated)
        let new_created = other.created.clone();
        let new_updated = other.updated.clone();

        // For each created record, remove from updated
        for i in new_created.ones() {
            self.updated.set(i, false);
        }

        // Merge bitsets
        for i in new_updated.ones() {
            self.updated.set(i, true);
        }
        for i in new_created.ones() {
            self.created.set(i, true);
        }

        // Merge deleted
        self.deleted.extend(other.deleted.iter().cloned());
    }
}

/// Maps record IDs to bit positions for FixedBitSet operations.
#[derive(Debug, Clone)]
pub struct IndexMapper {
    /// Maps record ID to bit position
    id_to_index: BTreeMap<RecordId, usize>,
    /// Maps bit position to record ID
    index_to_id: Vec<RecordId>,
    /// Counter for next index
    next_index: usize,
}

impl IndexMapper {
    /// Creates a new IndexMapper.
    #[inline]
    pub fn new() -> Self {
        Self {
            id_to_index: BTreeMap::new(),
            index_to_id: Vec::new(),
            next_index: 0,
        }
    }

    /// Gets or creates an index for a record ID.
    ///
    /// Returns the existing index if the ID is already mapped,
    /// otherwise creates a new one.
    pub fn get_or_create_index(&mut self, id: &RecordId) -> usize {
        if let Some(&index) = self.id_to_index.get(id) {
            return index;
        }

        let index = self.next_index;
        self.id_to_index.insert(id.clone(), index);
        self.index_to_id.push(id.clone());
        self.next_index += 1;

        index
    }

    /// Gets the index for an existing record ID.
    ///
    /// Returns None if the ID is not mapped.
    #[inline]
    pub fn get_index(&self, id: &RecordId) -> Option<usize> {
        self.id_to_index.get(id).copied()
    }

    /// Gets the record ID for an index.
    ///
    /// Returns None if the index is out of bounds.
    #[inline]
    pub fn get_id(&self, index: usize) -> Option<&RecordId> {
        self.index_to_id.get(index)
    }

    /// Reserves capacity for additional records.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.index_to_id.reserve(additional);
    }

    /// Returns the number of mapped records.
    #[inline]
    pub fn len(&self) -> usize {
        self.index_to_id.len()
    }

    /// Returns true if there are no mapped records.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.index_to_id.is_empty()
    }

    /// Clears all mappings.
    #[inline]
    pub fn clear(&mut self) {
        self.id_to_index.clear();
        self.index_to_id.clear();
        self.next_index = 0;
    }
}

impl Default for IndexMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Central storage for records.
///
/// The RecordStore provides:
/// - Efficient CRUD operations via BTreeMap
/// - Change tracking via ChangeSet and FixedBitSet
/// - Optional spatial indexing via rstar
/// - Undo/redo via integrated DeltaManager
///
/// # Examples
///
/// ```
/// use archflow_records::{RecordStore, RecordId, Record, FractionalIndex};
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct MyRecord {
///     id: RecordId,
///     index: Option<FractionalIndex>,
///     name: String,
/// }
///
/// impl Record for MyRecord {
///     fn id(&self) -> &RecordId { &self.id }
///     fn type_name(&self) -> &'static str { "MyRecord" }
///     fn index(&self) -> Option<&FractionalIndex> { self.index.as_ref() }
///     fn with_index(mut self, index: FractionalIndex) -> Self {
///         self.index = Some(index);
///         self
///     }
///     fn eq_ignoring_metadata(&self, other: &Self) -> bool {
///         self.id == other.id && self.name == other.name
///     }
///     fn validate(&self) -> Result<(), archflow_records::RecordError> { Ok(()) }
/// }
///
/// let mut store = RecordStore::new();
/// let id = RecordId::from_str("store_test_001").unwrap();
/// let record = MyRecord { id: id.clone(), index: None, name: "test".into() };
///
/// store.put(record);
/// assert_eq!(store.get(&id).unwrap().name, "test");
/// ```
#[derive(Debug, Clone)]
pub struct RecordStore<R: Record> {
    /// Main record storage by ID
    records: BTreeMap<RecordId, R>,
    /// Change tracking
    changes: ChangeSet,
    /// ID to bit position mapping
    mapper: IndexMapper,
    /// Undo/redo manager
    delta_manager: DeltaManager<R>,
    /// Version counter for optimistic concurrency
    version: u64,
}

impl<R: Record> RecordStore<R> {
    /// Creates a new empty RecordStore.
    #[inline]
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
            changes: ChangeSet::new(),
            mapper: IndexMapper::new(),
            delta_manager: DeltaManager::new(),
            version: 0,
        }
    }

    /// Creates a RecordStore with reserved capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut store = Self::new();
        store.mapper.reserve(capacity);
        store
    }

    /// Puts a record into the store.
    ///
    /// If the record already exists, it will be updated.
    /// Records the change for undo/redo.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to store
    ///
    /// # Returns
    ///
    /// The previous record if it existed
    pub fn put(&mut self, record: R) -> Option<R> {
        let id = record.id().clone();
        let is_new = !self.records.contains_key(&id);

        let index = self.mapper.get_or_create_index(&id);

        let result = self.records.insert(id.clone(), record);

        if is_new {
            self.changes.mark_created(index);
        } else {
            self.changes.mark_updated(index);
        }

        // Record delta for undo/redo
        if let Some(ref old_record) = result {
            let current_record = self.records.get(&id).unwrap().clone();
            self.delta_manager.record(RecordChange::updated(
                id,
                old_record.clone(),
                current_record,
            ));
        } else {
            self.delta_manager.record(RecordChange::created(
                self.records.get(&id).unwrap().clone(),
            ));
        }

        self.version += 1;

        result
    }

    /// Gets a record by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The record ID to look up
    ///
    /// # Returns
    ///
    /// `Some(&R)` if found, `None` otherwise
    #[inline]
    pub fn get(&self, id: &RecordId) -> Option<&R> {
        self.records.get(id)
    }

    /// Gets a mutable reference to a record by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The record ID to look up
    ///
    /// # Returns
    ///
    /// `Some(&mut R)` if found, `None` otherwise
    #[inline]
    pub fn get_mut(&mut self, id: &RecordId) -> Option<&mut R> {
        // Mark as updated
        if let Some(index) = self.mapper.get_index(id) {
            self.changes.mark_updated(index);
        }
        self.records.get_mut(id)
    }

    /// Removes a record by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The record ID to remove
    ///
    /// # Returns
    ///
    /// The removed record if it existed
    pub fn remove(&mut self, id: &RecordId) -> Option<R> {
        if let Some(record) = self.records.remove(id) {
            self.changes.mark_deleted(id.clone());
            self.delta_manager
                .record(RecordChange::deleted(record.clone()));
            self.version += 1;
            Some(record)
        } else {
            None
        }
    }

    /// Returns the number of records.
    #[inline]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if the store is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the current version.
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Drains and returns the current ChangeSet.
    ///
    /// This clears the change tracking after returning.
    #[inline]
    pub fn drain_changes(&mut self) -> ChangeSet {
        let changes = std::mem::take(&mut self.changes);
        self.changes.clear();
        changes
    }

    /// Returns a reference to the current ChangeSet without draining.
    #[inline]
    pub fn changes(&self) -> &ChangeSet {
        &self.changes
    }

    /// Returns an iterator over all records.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&RecordId, &R)> {
        self.records.iter()
    }

    /// Returns a mutable iterator over all records.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&RecordId, &mut R)> {
        self.records.iter_mut()
    }

    /// Undoes the last change.
    ///
    /// # Returns
    ///
    /// `Some(RecordChange)` if undo was successful, `None` otherwise
    pub fn undo(&mut self) -> Option<RecordChange<R>> {
        // delta_manager.undo() returns the inverted change and moves original to redo stack
        // The inverted change is what we apply to the store
        let inverted = self.delta_manager.undo()?;

        // Apply the inverted change to the store
        match inverted {
            RecordChange::Created { ref id, .. } => {
                // Inverse of Deleted is Created -> we're undoing a delete
                // The record was already removed when we did the delete
                // Do nothing
            }
            RecordChange::Updated {
                ref id,
                ref old_value,
                ..
            } => {
                // Inverse of Updated restores old_value
                self.records.insert(id.clone(), old_value.clone());
            }
            RecordChange::Deleted { ref id, .. } => {
                // Inverse of Created is Deleted -> we're undoing a create
                // Remove the record from the store
                self.records.remove(id);
            }
        }

        self.version += 1;
        Some(inverted)
    }

    /// Redoes the last undone change.
    ///
    /// # Returns
    ///
    /// `Some(RecordChange)` if redo was successful, `None` otherwise
    pub fn redo(&mut self) -> Option<RecordChange<R>> {
        // delta_manager.redo() returns the inverted change and moves original to undo stack
        let inverted = self.delta_manager.redo()?;

        // Apply the inverted change to the store
        match inverted {
            RecordChange::Created { ref id, .. } => {
                // Inverse of Deleted is Created -> we're redoing a delete
                // Do nothing, record was already gone
            }
            RecordChange::Updated {
                ref id,
                ref new_value,
                ..
            } => {
                // Inverse of Updated restores new_value (which was the old_value of original)
                self.records.insert(id.clone(), new_value.clone());
            }
            RecordChange::Deleted { ref id, ref record } => {
                // Inverse of Created is Deleted -> we're redoing a create
                // Restore the record from the Deleted change
                self.records.insert(id.clone(), record.clone());
            }
        }

        self.version += 1;
        Some(inverted)
    }

    /// Returns true if there are changes to undo.
    #[inline]
    pub fn can_undo(&self) -> bool {
        self.delta_manager.can_undo()
    }

    /// Returns true if there are changes to redo.
    #[inline]
    pub fn can_redo(&self) -> bool {
        self.delta_manager.can_redo()
    }

    /// Returns the number of undoable changes.
    #[inline]
    pub fn undo_count(&self) -> usize {
        self.delta_manager.undo_count()
    }

    /// Returns the number of redoable changes.
    #[inline]
    pub fn redo_count(&self) -> usize {
        self.delta_manager.redo_count()
    }

    /// Clears all records and history.
    #[inline]
    pub fn clear(&mut self) {
        self.records.clear();
        self.changes.clear();
        self.mapper.clear();
        self.delta_manager.clear();
        self.version = 0;
    }

    /// Gets records by index range.
    ///
    /// Returns all records whose index falls between the given bounds.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of range (exclusive)
    /// * `end` - End of range (exclusive)
    ///
    /// # Returns
    ///
    /// Iterator over matching records
    pub fn range<'a>(
        &'a self,
        start: Option<&'a FractionalIndex>,
        end: Option<&'a FractionalIndex>,
    ) -> impl Iterator<Item = (&RecordId, &'a R)> + 'a {
        self.records.iter().filter(move |(_, r)| {
            let idx = r.index();
            match (start, end, idx) {
                (Some(s), Some(e), Some(i)) => s < i && i < e,
                (Some(s), None, Some(i)) => s < i,
                (None, Some(e), Some(i)) => i < e,
                (None, None, Some(_)) => true,
                (_, _, None) => false,
            }
        })
    }
}

impl<R: Record> Default for RecordStore<R> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod record_store_tests {
    use super::*;
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRecord {
        id: RecordId,
        index: Option<FractionalIndex>,
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
            self.index.as_ref()
        }
        fn with_index(mut self, index: FractionalIndex) -> Self {
            self.index = Some(index);
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
    fn test_put_and_get() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("store_test_00001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "test".into(),
        };

        store.put(record.clone());

        assert_eq!(store.get(&id).unwrap().name, "test");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_put_update() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("store_update_001").unwrap();

        let record1 = TestRecord {
            id: id.clone(),
            index: None,
            name: "v1".into(),
        };
        let record2 = TestRecord {
            id: id.clone(),
            index: None,
            name: "v2".into(),
        };

        store.put(record1.clone());
        store.put(record2.clone());

        assert_eq!(store.get(&id).unwrap().name, "v2");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_change_set_optimization() {
        let mut store = RecordStore::new();

        // Insert 100 records
        for i in 0..100 {
            let id = RecordId::from_str(&format!("changeset_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                index: None,
                name: format!("record_{}", i),
            };
            store.put(record);
        }

        // Verify ChangeSet
        let changeset = store.changes();
        assert_eq!(changeset.created_count(), 100);
        assert_eq!(changeset.change_count(), 100);
    }

    #[test]
    fn test_version_increment() {
        let mut store = RecordStore::new();
        assert_eq!(store.version(), 0);

        let id = RecordId::from_str("version_test_001").unwrap();
        let record = TestRecord {
            id,
            index: None,
            name: "test".into(),
        };
        store.put(record);

        assert_eq!(store.version(), 1);
    }

    #[test]
    fn test_undo_redo_integration() {
        let mut store: RecordStore<TestRecord> = RecordStore::new();

        // Create one record
        let id = RecordId::from_str("undo_store_00001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "v1".into(),
        };

        store.put(record.clone());

        // Verify record exists
        assert!(store.get(&id).is_some());
        assert_eq!(store.get(&id).unwrap().name, "v1");

        // Undo the creation
        assert!(store.undo().is_some());

        // Record should be gone
        assert!(store.get(&id).is_none());

        // Redo should restore it
        assert!(store.can_redo());
        assert!(store.redo().is_some());
        assert!(store.get(&id).is_some());
        assert_eq!(store.get(&id).unwrap().name, "v1");
    }

    #[test]
    fn test_remove_record() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("remove_test_001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "test".into(),
        };

        store.put(record.clone());
        assert_eq!(store.len(), 1);

        let removed = store.remove(&id);
        assert!(removed.is_some());
        assert_eq!(store.len(), 0);
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn test_drain_changes() {
        let mut store = RecordStore::new();

        for i in 0..10 {
            let id = RecordId::from_str(&format!("drain_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                index: None,
                name: format!("record_{}", i),
            };
            store.put(record);
        }

        let changeset = store.drain_changes();
        assert_eq!(changeset.created_count(), 10);
        assert!(store.changes().is_empty());
    }

    #[test]
    fn test_iter_records() {
        let mut store = RecordStore::new();

        for i in 0..5 {
            let id = RecordId::from_str(&format!("iter_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                index: None,
                name: format!("record_{}", i),
            };
            store.put(record);
        }

        let count = store.iter().count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_range_query() {
        let mut store = RecordStore::new();

        // Create records with indices
        for i in 0..5 {
            let id = RecordId::from_str(&format!("range_{:08}", i)).unwrap();
            let idx = FractionalIndex::from_str(&format!("a{}", i + 1)).unwrap();
            let record = TestRecord {
                id,
                index: Some(idx),
                name: format!("record_{}", i),
            };
            store.put(record);
        }

        let start = FractionalIndex::from_str("a1").unwrap();
        let end = FractionalIndex::from_str("a4").unwrap();

        let range: Vec<_> = store.range(Some(&start), Some(&end)).collect();
        // Range (a1, a4) contains a2 and a3 = 2 results
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn test_record_not_found() {
        let store: RecordStore<TestRecord> = RecordStore::new();
        let id = RecordId::from_str("nonexistent_001").unwrap();
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn test_clear_store() {
        let mut store: RecordStore<TestRecord> = RecordStore::new();

        for i in 0..10 {
            let id = RecordId::from_str(&format!("clear_{:08}", i)).unwrap();
            let record = TestRecord {
                id,
                index: None,
                name: format!("record_{}", i),
            };
            store.put(record);
        }

        assert_eq!(store.len(), 10);
        assert!(store.can_undo());

        store.clear();

        assert_eq!(store.len(), 0);
        assert!(!store.can_undo());
    }

    #[test]
    fn test_get_mut() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("mut_test_001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "original".into(),
        };

        store.put(record);

        if let Some(r) = store.get_mut(&id) {
            r.name = "modified".to_string();
        }

        assert_eq!(store.get(&id).unwrap().name, "modified");

        // Should have marked as updated
        assert!(store.changes().updated_count() > 0);
    }
}
