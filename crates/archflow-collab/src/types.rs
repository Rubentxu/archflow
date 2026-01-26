//! # Types Module
//!
//! Core type definitions for the collaboration system, including site identifiers,
//! vector clocks, causal relations, and error types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Unique identifier for a collaboration site (client or server).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SiteId(u32);

impl SiteId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        SiteId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub const fn from_u32(value: u32) -> Self {
        SiteId(value)
    }

    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl Default for SiteId {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector clock implementation for tracking causal relationships.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VectorClock {
    dots: BTreeMap<SiteId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        VectorClock {
            dots: BTreeMap::new(),
        }
    }

    pub fn increment(&mut self, site: SiteId) {
        *self.dots.entry(site).or_insert(0) += 1;
    }

    pub fn get(&self, site: SiteId) -> u64 {
        self.dots.get(&site).copied().unwrap_or(0)
    }

    pub fn dots(&self) -> &BTreeMap<SiteId, u64> {
        &self.dots
    }

    pub fn is_empty(&self) -> bool {
        self.dots.is_empty()
    }

    pub fn relation(&self, other: &VectorClock) -> CausalRelation {
        let mut happened_before = true; // self <= other
        let mut happened_after = true; // self >= other

        let all_sites: std::collections::BTreeSet<&SiteId> =
            self.dots.keys().chain(other.dots.keys()).collect();

        for site in all_sites {
            let self_count = self.get(*site);
            let other_count = other.get(*site);

            if self_count > other_count {
                happened_before = false;
            }
            if self_count < other_count {
                happened_after = false;
            }
        }

        match (happened_before, happened_after) {
            (true, true) => CausalRelation::Equal,
            (true, false) => CausalRelation::HappenedBefore,
            (false, true) => CausalRelation::HappenedAfter,
            (false, false) => CausalRelation::Concurrent,
        }
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (site, count) in &other.dots {
            let current = self.get(*site);
            self.dots.insert(*site, *count.max(&current));
        }
    }
}

/// Represents the causal relationship between two vector clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CausalRelation {
    HappenedBefore,
    HappenedAfter,
    Concurrent,
    Equal,
}

/// Error type for CRDT operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyError {
    VersionTooOld,
    ConflictDetected,
    InvalidRecord,
    StorageError,
    MergeConflict(String),
    RecordError(String),
    Other(String),
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplyError::VersionTooOld => write!(f, "Operation version is too old"),
            ApplyError::ConflictDetected => write!(f, "Conflict detected during merge"),
            ApplyError::InvalidRecord => write!(f, "Record is invalid"),
            ApplyError::StorageError => write!(f, "Storage error"),
            ApplyError::MergeConflict(msg) => write!(f, "Merge conflict: {}", msg),
            ApplyError::RecordError(msg) => write!(f, "Record error: {}", msg),
            ApplyError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ApplyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_id_new() {
        let site = SiteId::new();
        assert!(site.as_u32() > 0);
    }

    #[test]
    fn test_vector_clock_relation() {
        let mut clock_a = VectorClock::new();
        let mut clock_b = VectorClock::new();
        let s_a = SiteId::new();
        let s_b = SiteId::new();

        clock_a.increment(s_a);
        clock_b.increment(s_a);
        clock_b.increment(s_b);

        // a: {s_a:1}, b: {s_a:1, s_b:1}
        assert_eq!(clock_a.relation(&clock_b), CausalRelation::HappenedBefore);
        assert_eq!(clock_b.relation(&clock_a), CausalRelation::HappenedAfter);

        clock_a.increment(s_b);
        assert_eq!(clock_a.relation(&clock_b), CausalRelation::Equal);

        let mut clock_c = VectorClock::new();
        clock_c.increment(s_a);
        clock_c.increment(s_a);
        // c: {s_a:2}, b: {s_a:1, s_b:1} -> Concurrent
        assert_eq!(clock_c.relation(&clock_b), CausalRelation::Concurrent);
    }
}
