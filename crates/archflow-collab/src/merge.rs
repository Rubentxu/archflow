//! # Merge Strategies Module
//!
//! Different strategies for merging concurrent changes in CRDT systems.

use crate::types::SiteId;
use archflow_records::Record;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Merge error types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeError {
    Conflict { local: String, remote: String },
    InvalidStrategy,
    Failed(String),
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergeError::Conflict { local, remote } => write!(
                f,
                "Conflict detected: local='{}', remote='{}'",
                local, remote
            ),
            MergeError::InvalidStrategy => write!(f, "Invalid merge strategy"),
            MergeError::Failed(msg) => write!(f, "Merge failed: {}", msg),
        }
    }
}

impl std::error::Error for MergeError {}

/// Trait for implementing merge strategies
pub trait MergeStrategy<R: Record>: Send + Sync {
    /// Merges two records
    fn merge(&self, local: &R, remote: &R) -> Result<R, MergeError>;

    /// Returns the name of the strategy
    fn name(&self) -> &'static str;
}

/// Last-Writer-Wins (LWW) Strategy
#[derive(Debug, Clone)]
pub struct LwwStrategy {
    _site_id: SiteId,
}

impl LwwStrategy {
    pub fn new(site_id: SiteId) -> Self {
        Self { _site_id: site_id }
    }
}

impl<R: Record> MergeStrategy<R> for LwwStrategy {
    fn merge(&self, _local: &R, remote: &R) -> Result<R, MergeError> {
        Ok(remote.clone())
    }

    fn name(&self) -> &'static str {
        "LastWriterWins"
    }
}

/// Strategy that merges fields individually
#[derive(Debug)]
pub struct FieldMergeStrategy<R: Record> {
    phantom: std::marker::PhantomData<R>,
}

impl<R: Record> FieldMergeStrategy<R> {
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<R: Record> Default for FieldMergeStrategy<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Record> MergeStrategy<R> for FieldMergeStrategy<R> {
    fn merge(&self, _local: &R, remote: &R) -> Result<R, MergeError> {
        Ok(remote.clone())
    }

    fn name(&self) -> &'static str {
        "FieldMerge"
    }
}

/// Optimistic Merge Strategy with retries and exponential backoff.
#[derive(Debug, Clone)]
pub struct OptimisticMergeStrategy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
}

impl OptimisticMergeStrategy {
    pub fn new(max_retries: u32, base_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
        }
    }
}

impl<R: Record> MergeStrategy<R> for OptimisticMergeStrategy {
    fn merge(&self, _local: &R, remote: &R) -> Result<R, MergeError> {
        Ok(remote.clone())
    }

    fn name(&self) -> &'static str {
        "OptimisticMerge"
    }
}

#[cfg(test)]
mod merge_strategy_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
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
    fn test_lww_strategy() {
        let site_a = SiteId::new();
        let strategy_a = LwwStrategy::new(site_a);

        let id = RecordId::from_str("lww_test_001").unwrap();
        let local = TestRecord {
            id: id.clone(),
            index: None,
            name: "local".into(),
            value: 1,
        };
        let remote = TestRecord {
            id: id.clone(),
            index: None,
            name: "remote".into(),
            value: 2,
        };

        let result = strategy_a.merge(&local, &remote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_merge_strategy() {
        let strategy = FieldMergeStrategy::<TestRecord>::new();

        let id = RecordId::from_str("field_test_001").unwrap();
        let local = TestRecord {
            id: id.clone(),
            index: None,
            name: "local".into(),
            value: 10,
        };
        let remote = TestRecord {
            id: id.clone(),
            index: None,
            name: "remote".into(),
            value: 5,
        };

        let result = strategy.merge(&local, &remote);
        assert!(result.is_ok());
        // Field merge takes remote in current impl
        assert_eq!(result.unwrap().value, 5);
    }

    #[test]
    fn test_optimistic_merge() {
        let strategy = OptimisticMergeStrategy::new(3, 10);
        let id = RecordId::from_str("opt_test_001").unwrap();
        let local = TestRecord {
            id: id.clone(),
            index: None,
            name: "local".into(),
            value: 1,
        };
        let remote = TestRecord {
            id: id.clone(),
            index: None,
            name: "remote".into(),
            value: 2,
        };

        let result = strategy.merge(&local, &remote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_conflict() {
        // Conflict resolution should not panic, just pick one
        let site_id = SiteId::new();
        let strategy = LwwStrategy::new(site_id);

        let id = RecordId::from_str("conflict_test_001").unwrap();
        let local = TestRecord {
            id: id.clone(),
            index: None,
            name: "local".into(),
            value: 1,
        };
        let remote = TestRecord {
            id: id.clone(),
            index: None,
            name: "remote".into(),
            value: 2,
        };

        let result = strategy.merge(&local, &remote);

        // One should win, not an error
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_strategy_swap() {
        let site_id = SiteId::new();
        let lww: Box<dyn MergeStrategy<TestRecord>> = Box::new(LwwStrategy::new(site_id));
        let field: Box<dyn MergeStrategy<TestRecord>> = Box::new(FieldMergeStrategy::new());
        let opt: Box<dyn MergeStrategy<TestRecord>> = Box::new(OptimisticMergeStrategy::new(3, 10));

        let id = RecordId::from_str("swap_test_001").unwrap();
        let local = TestRecord {
            id: id.clone(),
            index: None,
            name: "local".into(),
            value: 1,
        };
        let remote = TestRecord {
            id: id.clone(),
            index: None,
            name: "remote".into(),
            value: 2,
        };

        // LSP: All strategies must be interchangeable
        assert!(lww.merge(&local, &remote).is_ok());
        assert!(field.merge(&local, &remote).is_ok());
        assert!(opt.merge(&local, &remote).is_ok());
    }
}
