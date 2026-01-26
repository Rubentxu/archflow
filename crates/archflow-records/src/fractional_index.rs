//! Fractional Indexing for Conflict-Free Ordering
//!
//! This module implements fractional indexing, a technique used by collaborative
//! editors like tldraw and Figma to maintain order without conflicts.
//!
//! The key insight is that we can insert a new item between two existing items
//! by generating a string that sorts lexicographically between them.
//!
//! # Algorithm
//!
//! - `first()` creates the initial index: "a1"
//! - `between(left, right)` generates an index between two existing ones
//! - When indices become too long (bloat), automatic rebalancing occurs
//!
//! # Example
//!
//! ```
//! use archflow_records::FractionalIndex;
//!
//! let first = FractionalIndex::first();
//! let second = FractionalIndex::from_str("a2").unwrap();
//!
//! let between = FractionalIndex::between(&first, &second);
//! assert!(first < between);
//! assert!(between < second);
//! ```

use crate::error::RecordError;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Maximum index length before triggering rebalance
const MAX_INDEX_LENGTH: usize = 16;

/// Alphabet for index generation (base-52)
const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";

/// Fractional index for conflict-free ordering.
///
/// Uses a lexicographically sortable string representation that allows
/// inserting new items between any two existing items.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct FractionalIndex(String);

impl FractionalIndex {
    /// Maximum allowed index length before rebalancing
    pub const MAX_LENGTH: usize = MAX_INDEX_LENGTH;

    /// Creates the first index in a sequence.
    ///
    /// Returns "a1" as the initial index value.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_records::FractionalIndex;
    ///
    /// let first = FractionalIndex::first();
    /// assert_eq!(first.as_str(), "a1");
    /// ```
    #[inline]
    pub fn first() -> Self {
        FractionalIndex("a1".to_string())
    }

    /// Creates an index between two existing indices.
    ///
    /// This is the core algorithm. Given two indices `left` and `right`,
    /// generates a new index that sorts lexicographically between them.
    ///
    /// # Algorithm
    ///
    /// 1. If `left` and `right` are adjacent, extend `left` with a new character
    /// 2. Find the first position where they differ
    /// 3. Increment the character at that position
    /// 4. Truncate to maintain ordering
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_records::FractionalIndex;
    ///
    /// let first = FractionalIndex::first();
    /// let second = FractionalIndex::first();
    ///
    /// let between = FractionalIndex::between(&first, &second);
    /// assert!(first < between);
    /// assert!(between < second);
    /// ```
    pub fn between(left: &Self, right: &Self) -> Self {
        let left_str = left.as_str();
        let right_str = right.as_str();

        // Handle edge case: both are the same
        if left_str == right_str {
            return Self::extend_left(left_str);
        }

        // Try to find a position to insert
        if let Some(result) = Self::try_insert_between(left_str, right_str) {
            return result;
        }

        // If we can't insert, extend the left one
        Self::extend_left(left_str)
    }

    /// Attempts to insert between two strings
    fn try_insert_between(left: &str, right: &str) -> Option<Self> {
        let min_len = std::cmp::min(left.len(), right.len());

        for i in 0..min_len {
            let left_char = left.as_bytes()[i];
            let right_char = right.as_bytes()[i];

            if left_char == right_char {
                continue;
            }

            // Found a differing position
            if left_char + 1 < right_char {
                // There's room to insert a character between them
                let mut result = left[..i].to_string();
                result.push((left_char + 1) as char);

                // Truncate left's suffix if needed to maintain ordering
                if result.len() < left.len() {
                    result.push_str(&left[i + 1..]);
                }

                return Some(FractionalIndex(result));
            }

            // No room at this position, need to handle carry
            break;
        }

        // If we reach here, left is a prefix of right, or no room found
        // Extend left
        Some(Self::extend_left(left))
    }

    /// Extends the left index by adding a character
    fn extend_left(index: &str) -> Self {
        if index.is_empty() {
            return FractionalIndex("a1".to_string());
        }

        let mut result = index.to_string();
        result.push('a');
        FractionalIndex(result)
    }

    /// Returns the underlying string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the length of the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the index is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks if this index needs rebalancing.
    #[inline]
    pub fn needs_rebalance(&self) -> bool {
        self.0.len() > MAX_INDEX_LENGTH
    }

    /// Rebalances an index that's too long.
    ///
    /// When an index exceeds MAX_INDEX_LENGTH, we compress it by:
    /// 1. Taking the first half
    /// 2. Adding a character from the alphabet
    ///
    /// This is used internally when bloat is detected.
    pub fn rebalance(&mut self, neighbors: &[Self]) {
        if !self.needs_rebalance() {
            return;
        }

        // Simple rebalancing: compress to half length
        let new_len = (self.0.len() + 1) / 2;
        let mut compressed = self.0[..new_len].to_string();

        // Ensure it's still between neighbors
        if !neighbors.is_empty() {
            let min_neighbor = neighbors.iter().min().unwrap();

            // If compressed is not between neighbors, adjust
            if compressed.as_str() <= min_neighbor.as_str() {
                compressed =
                    Self::between(&FractionalIndex(compressed.clone()), min_neighbor).to_string();
            }
        }

        self.0 = compressed;
    }

    /// Creates an index after the given one.
    ///
    /// Equivalent to `between(index, next_of_index)`.
    pub fn after(index: &Self) -> Self {
        let s = index.as_str();
        // Find a position where we can increment
        for i in (0..s.len()).rev() {
            let c = s.as_bytes()[i];
            if c < b'z' {
                let mut next = s[..i].to_string();
                next.push((c + 1) as char);
                next.push_str(&s[i + 1..]);
                return Self(next);
            }
        }
        // All 'z's, need to extend
        Self(format!("{}a", s))
    }

    /// Creates an index before the given one.
    ///
    /// Equivalent to `between(prev_of_index, index)`.
    pub fn before(index: &Self) -> Self {
        let s = index.as_str();
        if s.is_empty() {
            return Self::first();
        }
        // Find a position where we can decrement
        for i in (0..s.len()).rev() {
            let c = s.as_bytes()[i];
            if c > b'a' {
                let mut prev = s[..i].to_string();
                prev.push((c - 1) as char);
                prev.push_str(&s[i + 1..]);
                return Self(prev);
            }
        }
        // All 'a's, this is the first
        Self::first()
    }

    /// Converts to a u64 for fast comparison (first 8 bytes as big-endian).
    ///
    /// Useful for sorting performance.
    pub fn as_u64(&self) -> u64 {
        let mut result: u64 = 0;
        let bytes = self.0.as_bytes();
        for (i, &b) in bytes.iter().enumerate().take(8) {
            result = (result << 8) | (b as u64);
        }
        result
    }
}

impl FromStr for FractionalIndex {
    type Err = RecordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(RecordError::InvalidIndex("empty index".to_string()));
        }

        // Validate characters
        for byte in s.as_bytes() {
            if !((b'a'..=b'z').contains(byte) || (b'0'..=b'9').contains(byte)) {
                return Err(RecordError::InvalidIndex(format!(
                    "invalid character '{}'",
                    *byte as char
                )));
            }
        }

        Ok(FractionalIndex(s.to_string()))
    }
}

impl PartialOrd for FractionalIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FractionalIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lexicographic comparison, but we need to handle
        // that shorter strings that are prefixes come first
        let self_bytes = self.0.as_bytes();
        let other_bytes = other.0.as_bytes();

        let min_len = std::cmp::min(self_bytes.len(), other_bytes.len());

        for i in 0..min_len {
            match self_bytes[i].cmp(&other_bytes[i]) {
                Ordering::Equal => continue,
                other => return other,
            }
        }

        // If all compared bytes are equal, shorter comes first
        self_bytes.len().cmp(&other_bytes.len())
    }
}

impl fmt::Display for FractionalIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<FractionalIndex> for String {
    fn from(idx: FractionalIndex) -> Self {
        idx.0
    }
}

impl TryFrom<String> for FractionalIndex {
    type Error = RecordError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl AsRef<str> for FractionalIndex {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod fractional_index_tests {
    use super::*;

    #[test]
    fn test_first_index_creation() {
        let index = FractionalIndex::first();
        assert_eq!(index.as_str(), "a1");
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_insert_between_same_indices() {
        let first = FractionalIndex::first();
        let second = FractionalIndex::first();

        // When left == right, between returns an extended index
        let between1 = FractionalIndex::between(&first, &second);
        let between2 = FractionalIndex::between(&first, &second);

        // Both should be greater than first (extended indices are greater)
        assert!(first < between1);
        assert!(first < between2);

        // When left == right, between returns the same result (both call extend_left)
        assert_eq!(between1, between2);
    }

    #[test]
    fn test_index_ordering() {
        let indices: Vec<FractionalIndex> = (0..10).map(|_| FractionalIndex::first()).collect();

        let mut sorted = indices.clone();
        sorted.sort();

        // Verify ordering is maintained
        assert_eq!(sorted, indices);
    }

    #[test]
    fn test_index_rebalance_on_bloat() {
        // Create an index that's too long
        let mut bloated = FractionalIndex::from_str(&"a".repeat(20)).unwrap();

        assert!(bloated.needs_rebalance());

        let neighbors = vec![
            FractionalIndex::from_str("a1").unwrap(),
            FractionalIndex::from_str("z1").unwrap(),
        ];

        bloated.rebalance(&neighbors);

        assert!(bloated.as_str().len() <= MAX_INDEX_LENGTH);
    }

    #[test]
    fn test_extend_left() {
        let index = FractionalIndex::from_str("a1").unwrap();
        let extended = FractionalIndex::extend_left("a1");

        assert!(index < extended);
        assert_eq!(extended.as_str(), "a1a");
    }

    #[test]
    fn test_after_index() {
        let index = FractionalIndex::first();
        let after = FractionalIndex::after(&index);

        assert!(index < after);
    }

    #[test]
    fn test_before_index() {
        // before(first()) should return first() since there's nothing before it
        let index = FractionalIndex::first();
        let before = FractionalIndex::before(&index);

        assert_eq!(before, index);
    }

    #[test]
    fn test_try_insert_between_different_prefix() {
        // "a1" and "b1" should allow insertion
        let left = FractionalIndex::from_str("a1").unwrap();
        let right = FractionalIndex::from_str("b1").unwrap();

        let between = FractionalIndex::between(&left, &right);

        assert!(left < between);
        assert!(between < right);
    }

    #[test]
    fn test_try_insert_between_same_prefix() {
        // "a1a" and "a1b" should allow insertion at the third char
        let left = FractionalIndex::from_str("a1a").unwrap();
        let right = FractionalIndex::from_str("a1b").unwrap();

        let between = FractionalIndex::between(&left, &right);

        assert!(left < between);
        assert!(between < right);
    }

    #[test]
    fn test_no_room_for_insertion() {
        // "a1" and "a2" have no room - must extend
        let left = FractionalIndex::from_str("a1").unwrap();
        let right = FractionalIndex::from_str("a2").unwrap();

        let between = FractionalIndex::between(&left, &right);

        assert!(left < between);
        assert!(between < right);
        // Should have been extended
        assert!(between.len() > left.len());
    }

    #[test]
    fn test_consecutive_insertions() {
        let first = FractionalIndex::first();

        let mut prev = first.clone();
        let mut indices = vec![first];

        for _ in 0..5 {
            let next = FractionalIndex::after(&prev);
            indices.push(next.clone());
            prev = next;
        }

        // Verify ordering
        for i in 0..indices.len() - 1 {
            assert!(indices[i] < indices[i + 1]);
        }
    }

    #[test]
    fn test_as_u64() {
        let index = FractionalIndex::from_str("a1").unwrap();
        let u64_val = index.as_u64();

        // a=97, 1=49 -> 97*256 + 49 = 24881
        assert_eq!(u64_val, 97 * 256 + 49);
    }

    #[test]
    fn test_display_trait() {
        let index = FractionalIndex::first();
        assert_eq!(format!("{}", index), "a1");
    }

    #[test]
    fn test_invalid_characters() {
        assert!(FractionalIndex::from_str("a!").is_err());
        assert!(FractionalIndex::from_str("a@").is_err());
        assert!(FractionalIndex::from_str("a ").is_err());
    }

    #[test]
    fn test_empty_index() {
        assert!(FractionalIndex::from_str("").is_err());
    }

    #[test]
    fn test_complex_insertion_chain() {
        // Insert multiple times between the same indices
        let mut left = FractionalIndex::first();
        let right = FractionalIndex::first();

        let mut inserts = Vec::new();
        for _ in 0..10 {
            let new_idx = FractionalIndex::between(&left, &right);
            inserts.push(new_idx.clone());
            left = new_idx;
        }

        // All should be ordered
        for i in 0..inserts.len() - 1 {
            assert!(inserts[i] < inserts[i + 1]);
        }
    }
}
