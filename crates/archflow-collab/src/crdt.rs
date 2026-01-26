//! # CRDT Module
//!
//! Conflict-free Replicated Data Type (CRDT) based on a Last-Writer-Wins (LWW) approach
//! for automatic conflict resolution in collaborative editing.

use crate::merge::{LwwStrategy, MergeStrategy};
use crate::types::{ApplyError, CausalRelation, SiteId, VectorClock};
use archflow_records::{Record, RecordChange, RecordStore};
use std::sync::{Arc, RwLock};

/// CRDT structure that maintains a local record store and handles synchronization
/// and conflict resolution. It uses a Vector Clock to track causality and resolve
/// conflicts when concurrent updates occur.
pub struct CRDT<R: Record> {
    record_store: Arc<RwLock<RecordStore<R>>>,
    site_id: SiteId,
    vector_clock: VectorClock,
    merge_strategy: Box<dyn MergeStrategy<R>>,
    pending_operations: Vec<RecordChange<R>>,
}

impl<R: Record> CRDT<R> {
    /// Creates a new CRDT instance.
    pub fn new(site_id: SiteId) -> Self {
        CRDT {
            record_store: Arc::new(RwLock::new(RecordStore::new())),
            site_id,
            vector_clock: VectorClock::new(),
            // Default to Last-Writer-Wins based on SiteId
            merge_strategy: Box::new(LwwStrategy::new(site_id)),
            pending_operations: Vec::new(),
        }
    }

    /// Set a custom merge strategy.
    pub fn set_merge_strategy(&mut self, strategy: Box<dyn MergeStrategy<R>>) {
        self.merge_strategy = strategy;
    }

    /// Accessor for site_id
    pub fn site_id(&self) -> SiteId {
        self.site_id
    }

    /// Get current Vector Clock.
    pub fn vector_clock(&self) -> &VectorClock {
        &self.vector_clock
    }

    /// Returns a list of changes since the last synchronization.
    pub fn get_changes(&self) -> Vec<RecordChange<R>> {
        self.pending_operations.clone()
    }

    /// Clear the list of pending operations after they have been synchronized.
    pub fn clear_pending(&mut self) {
        self.pending_operations.clear();
    }

    /// Apply a local change to the CRDT.
    pub fn apply_local(&mut self, record: R) -> Result<(), ApplyError> {
        let mut store = self
            .record_store
            .write()
            .map_err(|_| ApplyError::StorageError)?;

        let id = record.id().clone();

        // Use put which internally records the change in DeltaManager
        store.put(record.clone());

        self.vector_clock.increment(self.site_id);

        // Manually track in pending_operations for collaboration broadcast
        self.pending_operations
            .push(RecordChange::Created { id, record });

        Ok(())
    }

    /// Merges remote changes into the local CRDT state.
    pub fn merge(
        &mut self,
        remote_clock: &VectorClock,
        remote_records: Vec<R>,
    ) -> Result<(), ApplyError> {
        // Compare clocks
        let relation = self.vector_clock.relation(remote_clock);

        match relation {
            CausalRelation::HappenedBefore | CausalRelation::Concurrent => {
                let mut store = self
                    .record_store
                    .write()
                    .map_err(|_| ApplyError::StorageError)?;

                for remote in remote_records {
                    let id = remote.id();
                    if let Some(local) = store.get(id) {
                        // Conflict resolution needed
                        match self.merge_strategy.merge(local, &remote) {
                            Ok(merged) => {
                                store.put(merged);
                            }
                            Err(e) => return Err(ApplyError::MergeConflict(e.to_string())),
                        }
                    } else {
                        // No local record, just apply
                        store.put(remote);
                    }
                }

                // Update local clock
                self.vector_clock.merge(remote_clock);
                Ok(())
            }
            CausalRelation::HappenedAfter => {
                // Remote is behind, nothing to do
                Ok(())
            }
            CausalRelation::Equal => {
                // Already in sync
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod crdt_tests {
    use super::*;
    use archflow_records::{FractionalIndex, RecordId};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TestRecord {
        pub id: RecordId,
        pub index: Option<FractionalIndex>,
        pub name: String,
        pub value: i32,
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
    }

    #[test]
    fn test_crdt_new() {
        let site_id = SiteId::new();
        let crdt = CRDT::<TestRecord>::new(site_id);
        assert_eq!(crdt.site_id(), site_id);
        assert!(crdt.vector_clock().is_empty());
    }

    #[test]
    fn test_apply_local_change() {
        let site_id = SiteId::new();
        let mut crdt = CRDT::<TestRecord>::new(site_id);

        let id = RecordId::from_str("record_1234567890").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "Test Name".into(),
            value: 42,
        };

        crdt.apply_local(record).unwrap();
        assert_eq!(crdt.vector_clock().get(site_id), 1);
        assert_eq!(crdt.get_changes().len(), 1);
    }

    #[test]
    fn test_merge_remote_changes() {
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let mut crdt_a = CRDT::<TestRecord>::new(site_a);
        let mut crdt_b = CRDT::<TestRecord>::new(site_b);

        let id = RecordId::from_str("record_merge_001").unwrap();
        let record_b = TestRecord {
            id: id.clone(),
            index: None,
            name: "From B".into(),
            value: 100,
        };

        crdt_b.apply_local(record_b.clone()).unwrap();

        // Merge B's changes into A
        let clock_b = crdt_b.vector_clock().clone();
        crdt_a.merge(&clock_b, vec![record_b]).unwrap();

        assert_eq!(crdt_a.vector_clock().get(site_b), 1);
        assert!(matches!(
            crdt_a.vector_clock().relation(&clock_b),
            CausalRelation::Equal
        ));
    }

    #[test]
    fn test_concurrent_update_conflict() {
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let mut crdt_a = CRDT::<TestRecord>::new(site_a);
        let mut crdt_b = CRDT::<TestRecord>::new(site_b);

        let id = RecordId::from_str("record_conflict_01").unwrap();

        // Concurrent changes
        crdt_a
            .apply_local(TestRecord {
                id: id.clone(),
                index: None,
                name: "Name A".into(),
                value: 1,
            })
            .unwrap();

        crdt_b
            .apply_local(TestRecord {
                id: id.clone(),
                index: None,
                name: "Name B".into(),
                value: 2,
            })
            .unwrap();

        // Merge B into A
        let clock_b = crdt_b.vector_clock().clone();
        let records_b = vec![TestRecord {
            id: id.clone(),
            index: None,
            name: "Name B".into(),
            value: 2,
        }];

        crdt_a.merge(&clock_b, records_b).unwrap();
    }

    #[test]
    fn test_get_changes() {
        let site_id = SiteId::new();
        let mut crdt = CRDT::<TestRecord>::new(site_id);

        let record = TestRecord {
            id: RecordId::from_str("record_change_001").unwrap(),
            index: None,
            name: "Initial".into(),
            value: 10,
        };

        crdt.apply_local(record).unwrap();
        let changes = crdt.get_changes();
        assert_eq!(changes.len(), 1);

        crdt.clear_pending();
        assert_eq!(crdt.get_changes().len(), 0);
    }

    #[test]
    fn test_vector_clock_relation() {
        // Test causal relations between vector clocks
        let site_a = SiteId::new();
        let site_b = SiteId::new();
        let site_c = SiteId::new();

        let mut clock_a = VectorClock::new();
        let mut clock_b = VectorClock::new();
        let mut clock_c = VectorClock::new();

        // Clock A: site_a incremented twice
        clock_a.increment(site_a);
        clock_a.increment(site_a);

        // Clock B: site_b incremented once
        clock_b.increment(site_b);

        // Clock C: site_c incremented three times
        clock_c.increment(site_c);
        clock_c.increment(site_c);
        clock_c.increment(site_c);

        // A and B are concurrent (no causal relationship)
        assert_eq!(clock_a.relation(&clock_b), CausalRelation::Concurrent);
        assert_eq!(clock_b.relation(&clock_a), CausalRelation::Concurrent);

        // A and C are concurrent
        assert_eq!(clock_a.relation(&clock_c), CausalRelation::Concurrent);
        assert_eq!(clock_c.relation(&clock_a), CausalRelation::Concurrent);

        // B and C are concurrent
        assert_eq!(clock_b.relation(&clock_c), CausalRelation::Concurrent);

        // Now make clock_b causal descendant of clock_a
        clock_b.merge(&clock_a);
        // After merge, clock_b should have happened after clock_a
        let relation = clock_b.relation(&clock_a);
        assert!(relation == CausalRelation::HappenedAfter || relation == CausalRelation::Equal);
    }
}
