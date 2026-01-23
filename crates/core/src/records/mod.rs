// Copyright 2024 ArchFlow Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Records module - Type-safe IDs, fractional indexing, and delta-based history.

use fastrand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Type-safe ID wrapper for records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordId(String);

impl RecordId {
    /// Creates a new RecordId with validation.
    pub fn new(id: String) -> Self {
        assert!(id.len() >= 10, "Record ID too short (min 10 chars)");
        Self(id)
    }

    /// Returns the underlying string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Hash for RecordId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Fractional indexing for z-order without conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractionalIndex(String);

impl FractionalIndex {
    /// Creates a new FractionalIndex from an existing string.
    pub fn new(index: String) -> Self {
        assert!(!index.is_empty(), "FractionalIndex cannot be empty");
        Self(index)
    }

    /// Generates a new index between two existing indices (or at edges).
    pub fn between(prev: Option<&Self>, next: Option<&Self>) -> Self {
        match (prev, next) {
            (None, None) => Self("a0".to_string()),
            (Some(p), None) => Self::increment(p),
            (None, Some(n)) => Self::decrement(n),
            (Some(p), Some(n)) => Self::between_existing(p, n),
        }
    }

    fn increment(prev: &Self) -> Self {
        let last_char = prev.0.chars().last().unwrap();
        if last_char == 'z' {
            Self(format!("{}a", &prev.0[..prev.0.len() - 1]))
        } else {
            Self(format!(
                "{}{}",
                &prev.0[..prev.0.len() - 1],
                (last_char as u8 + 1) as char
            ))
        }
    }

    fn decrement(next: &Self) -> Self {
        Self(format!("a{}", next.0))
    }

    fn between_existing(prev: &Self, next: &Self) -> Self {
        let prev_bytes = prev.0.as_bytes();
        let next_bytes = next.0.as_bytes();
        let min_len = prev_bytes.len().min(next_bytes.len());

        let mut diff_pos = 0;
        while diff_pos < min_len && prev_bytes[diff_pos] == next_bytes[diff_pos] {
            diff_pos += 1;
        }

        if diff_pos >= min_len {
            Self(format!("{}a", &next.0[..diff_pos + 1]))
        } else {
            let prev_char = prev_bytes[diff_pos] as char;
            let next_char = next_bytes[diff_pos] as char;

            if (next_char as u8) - (prev_char as u8) > 1 {
                let mid_char = ((prev_char as u8 + next_char as u8) / 2) as char;
                let mut result = String::from(&prev.0[..diff_pos]);
                result.push(mid_char);
                result.push('a');
                Self(result)
            } else {
                let prefix = &prev.0[..diff_pos + 1];
                let mut rng = Rng::new();
                let suffix: String = (0..3)
                    .map(|_| {
                        let c: u8 = rng.u8(b'a'..=b'z');
                        c as char
                    })
                    .collect();
                Self(format!("{}{}", prefix, suffix))
            }
        }
    }

    /// Returns the underlying string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq for FractionalIndex {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FractionalIndex {}

impl PartialOrd for FractionalIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FractionalIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_bytes = self.0.as_bytes();
        let other_bytes = other.0.as_bytes();
        let min_len = self_bytes.len().min(other_bytes.len());

        for i in 0..min_len {
            match self_bytes[i].cmp(&other_bytes[i]) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        self_bytes.len().cmp(&other_bytes.len())
    }
}

/// Trait for all records in the store.
pub trait Record: Send + Sync + Clone + 'static {
    fn id(&self) -> &RecordId;
    fn type_name(&self) -> &str;
    fn index(&self) -> &FractionalIndex;
    fn with_index(&self, index: FractionalIndex) -> Self;
}

/// Represents a change to a record for delta-based undo/redo.
#[derive(Debug, Clone)]
pub enum RecordChange<R: Record> {
    Created {
        id: RecordId,
        record: R,
    },
    Updated {
        id: RecordId,
        old_value: R,
        new_value: R,
    },
    Deleted {
        id: RecordId,
        record: R,
    },
}

/// Store for managing records with delta-based undo/redo support.
pub struct Store<R: Record> {
    records: indexmap::IndexMap<RecordId, R>,
    undo_history: std::collections::VecDeque<Vec<RecordChange<R>>>,
    redo_history: std::collections::VecDeque<Vec<RecordChange<R>>>,
    max_history: usize,
}

impl<R: Record> Default for Store<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Record> Store<R> {
    /// Creates a new empty store with default history size (100).
    pub fn new() -> Self {
        Self {
            records: indexmap::IndexMap::new(),
            undo_history: std::collections::VecDeque::new(),
            redo_history: std::collections::VecDeque::new(),
            max_history: 100,
        }
    }

    /// Creates a store with custom history size.
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            records: indexmap::IndexMap::new(),
            undo_history: std::collections::VecDeque::new(),
            redo_history: std::collections::VecDeque::new(),
            max_history,
        }
    }

    /// Inserts or updates a record in the store.
    pub fn put(&mut self, record: R) -> Vec<RecordChange<R>> {
        let changes = match self.records.get(record.id()) {
            None => vec![RecordChange::Created {
                id: record.id().clone(),
                record: record.clone(),
            }],
            Some(old) => vec![RecordChange::Updated {
                id: record.id().clone(),
                old_value: old.clone(),
                new_value: record.clone(),
            }],
        };

        self.undo_history.push_back(changes.clone());
        if self.undo_history.len() > self.max_history {
            self.undo_history.pop_front();
        }
        self.redo_history.clear();
        self.records.insert(record.id().clone(), record);

        changes
    }

    /// Retrieves a record by ID.
    pub fn get(&self, id: &RecordId) -> Option<&R> {
        self.records.get(id)
    }

    /// Checks if a record exists.
    pub fn contains(&self, id: &RecordId) -> bool {
        self.records.contains_key(id)
    }

    /// Removes a record by ID.
    pub fn remove(&mut self, id: &RecordId) -> Option<R> {
        let removed = self.records.shift_remove(id)?;
        let changes = vec![RecordChange::Deleted {
            id: id.clone(),
            record: removed.clone(),
        }];
        self.undo_history.push_back(changes);
        self.redo_history.clear();
        Some(removed)
    }

    /// Returns an iterator over all records.
    pub fn iter(&self) -> impl Iterator<Item = &R> {
        self.records.values()
    }

    /// Returns an iterator over records sorted by index.
    pub fn iter_sorted(&self) -> impl Iterator<Item = &R> {
        let mut records: Vec<_> = self.records.values().collect();
        records.sort_by_key(|r| r.index());
        records.into_iter()
    }

    /// Returns the number of records in the store.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Undoes the last change.
    pub fn undo(&mut self) -> bool {
        if let Some(changes) = self.undo_history.pop_back() {
            for change in changes.into_iter().rev() {
                match change {
                    RecordChange::Created { id, .. } => {
                        self.records.shift_remove(&id);
                    }
                    RecordChange::Updated {
                        id,
                        old_value,
                        new_value,
                    } => {
                        self.records.insert(id.clone(), old_value.clone());
                        self.redo_history.push_back(vec![RecordChange::Updated {
                            id: id.clone(),
                            old_value: old_value,
                            new_value: new_value,
                        }]);
                    }
                    RecordChange::Deleted { id, record } => {
                        self.records.insert(id.clone(), record.clone());
                        self.redo_history.push_back(vec![RecordChange::Deleted {
                            id: id.clone(),
                            record,
                        }]);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Redoes the last undone change.
    pub fn redo(&mut self) -> bool {
        if let Some(changes) = self.redo_history.pop_back() {
            for change in changes {
                match change {
                    RecordChange::Created { id, record, .. } => {
                        self.records.insert(id, record);
                    }
                    RecordChange::Updated { id, new_value, .. } => {
                        self.records.insert(id, new_value);
                    }
                    RecordChange::Deleted { id, .. } => {
                        self.records.shift_remove(&id);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Returns true if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_history.is_empty()
    }

    /// Returns true if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_history.is_empty()
    }

    /// Clears all history and records.
    pub fn clear(&mut self) {
        self.records.clear();
        self.undo_history.clear();
        self.redo_history.clear();
    }

    /// Returns all recent changes for ECS synchronization.
    pub fn get_changes(&self) -> Vec<RecordChange<R>> {
        self.undo_history.iter().flatten().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        id: RecordId,
        type_name: String,
        index: FractionalIndex,
        value: String,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }

        fn type_name(&self) -> &str {
            &self.type_name
        }

        fn index(&self) -> &FractionalIndex {
            &self.index
        }

        fn with_index(&self, index: FractionalIndex) -> Self {
            Self {
                id: self.id.clone(),
                type_name: self.type_name.clone(),
                index,
                value: self.value.clone(),
            }
        }
    }

    #[test]
    fn test_record_id_creation() {
        let id = RecordId::new("valid_id_123".to_string());
        assert_eq!(id.as_str(), "valid_id_123");
    }

    #[test]
    #[should_panic(expected = "Record ID too short")]
    fn test_record_id_too_short() {
        RecordId::new("short".to_string());
    }

    #[test]
    fn test_first_fractional_index() {
        let index = FractionalIndex::between(None, None);
        assert_eq!(index.as_str(), "a0");
    }

    #[test]
    fn test_insert_between() {
        let a = FractionalIndex::new("a0".to_string());
        let b = FractionalIndex::new("a2".to_string());
        let mid = FractionalIndex::between(Some(&a), Some(&b));

        assert!(a < mid);
        assert!(mid < b);
    }

    #[test]
    fn test_insert_multiple_between() {
        let a = FractionalIndex::new("a0".to_string());
        let b = FractionalIndex::new("a1".to_string());

        let mut indices: Vec<String> = Vec::new();
        for _ in 0..5 {
            let idx = FractionalIndex::between(Some(&a), Some(&b));
            indices.push(idx.as_str().to_string());
        }

        let unique: std::collections::HashSet<_> = indices.iter().collect();
        assert!(unique.len() > 1, "Should generate multiple unique indices");
    }

    #[test]
    fn test_store_put_get() {
        let mut store = Store::new();
        let id = RecordId::new("test1234567".to_string());
        let index = FractionalIndex::between(None, None);
        let record = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index: index.clone(),
            value: "hello".to_string(),
        };

        store.put(record.clone());
        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved, &record);
    }

    #[test]
    fn test_store_undo() {
        let mut store = Store::new();
        let id = RecordId::new("test1234567".to_string());
        let index = FractionalIndex::between(None, None);

        let record1 = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index: index.clone(),
            value: "v1".to_string(),
        };

        store.put(record1.clone());

        let record2 = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index,
            value: "v2".to_string(),
        };

        store.put(record2);

        assert!(store.undo());
        let retrieved = store.get(&id).unwrap();
        assert_eq!(retrieved.value, "v1");
    }

    #[test]
    fn test_store_redo() {
        let mut store = Store::new();
        let id = RecordId::new("test1234567".to_string());
        let index = FractionalIndex::between(None, None);

        let record1 = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index: index.clone(),
            value: "v1".to_string(),
        };

        let record2 = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index,
            value: "v2".to_string(),
        };

        store.put(record1.clone());
        store.put(record2.clone());

        // After put(record2), the store has record2
        assert_eq!(store.get(&id).unwrap().value, "v2");

        // Undo should restore record1
        assert!(store.undo());
        assert_eq!(store.get(&id).unwrap().value, "v1");

        // Redo should restore record2
        assert!(store.redo());
        assert_eq!(store.get(&id).unwrap().value, "v2");
    }

    #[test]
    fn test_redo_cleared_on_new_change() {
        let mut store = Store::new();
        let id = RecordId::new("test1234567".to_string());
        let index = FractionalIndex::between(None, None);

        let record1 = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index: index.clone(),
            value: "v1".to_string(),
        };

        let record2 = TestRecord {
            id: id.clone(),
            type_name: "test".to_string(),
            index,
            value: "v2".to_string(),
        };

        let record3 = TestRecord {
            id,
            type_name: "test".to_string(),
            index: FractionalIndex::between(None, None),
            value: "v3".to_string(),
        };

        store.put(record1);
        store.put(record2);
        store.undo();
        store.put(record3);

        assert!(!store.redo());
    }
}
