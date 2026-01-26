//! Type-safe Record ID with validation
//!
//! This module provides `RecordId`, a type-safe identifier for records.
//! It enforces validation rules for length (10-128 chars) and character set.

use crate::error::RecordError;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

/// Type-safe identifier for records.
///
/// `RecordId` ensures that all record identifiers meet strict validation criteria:
/// - Minimum 10 characters
/// - Maximum 128 characters
/// - Only alphanumeric characters and underscores
///
/// # Examples
///
/// ```
/// use archflow_records::RecordId;
///
/// let id = RecordId::from_str("record_1234567890").unwrap();
/// assert_eq!(id.as_str(), "record_1234567890");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RecordId(String);

impl RecordId {
    /// Minimum valid ID length
    pub const MIN_LENGTH: usize = 10;

    /// Maximum valid ID length
    pub const MAX_LENGTH: usize = 128;

    /// Valid characters pattern: alphanumeric + underscore
    const VALID_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

    /// Creates a new RecordId from a validated string.
    ///
    /// Use `from_str` for parsing user input with validation.
    #[inline]
    pub fn new_unchecked(s: String) -> Self {
        RecordId(s)
    }

    /// Returns the underlying string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the underlying string, consuming the RecordId.
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Returns the length of the ID.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the ID is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a RecordId from a UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_records::RecordId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::new_v4();
    /// let id = RecordId::from_uuid(uuid);
    /// assert_eq!(id.to_uuid(), Some(uuid));
    /// ```
    pub fn from_uuid(uuid: uuid::Uuid) -> Self {
        RecordId(format!("uuid_{}", uuid))
    }

    /// Converts back to UUID if the ID was created from one.
    pub fn to_uuid(&self) -> Option<uuid::Uuid> {
        self.0
            .strip_prefix("uuid_")
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
    }

    /// Creates a RecordId from a u64 value.
    ///
    /// Useful for compact internal identifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_records::RecordId;
    ///
    /// let id = RecordId::from_u64(12345);
    /// assert_eq!(id.as_str(), "id_00000000000000012345");
    /// ```
    pub fn from_u64(value: u64) -> Self {
        RecordId(format!("id_{:020}", value))
    }

    /// Fast equality comparison for u64-based IDs.
    ///
    /// Optimized path for IDs created with `from_u64`.
    pub fn fast_eq(&self, other: &Self) -> bool {
        // Quick length check first
        if self.0.len() != other.0.len() {
            return false;
        }
        // For u64-based IDs, we can compare directly
        if self.0.starts_with("id_") && other.0.starts_with("id_") {
            return self.0 == other.0;
        }
        self.0 == other.0
    }

    /// Validates and creates a RecordId.
    fn validate(s: &str) -> Result<(), RecordError> {
        let len = s.len();

        if len < Self::MIN_LENGTH {
            return Err(RecordError::IdTooShort(len));
        }

        if len > Self::MAX_LENGTH {
            return Err(RecordError::IdTooLong(len));
        }

        // Check for valid characters
        for byte in s.as_bytes() {
            if !Self::VALID_CHARS.contains(byte) {
                return Err(RecordError::InvalidIdChars(s.to_string()));
            }
        }

        Ok(())
    }
}

impl FromStr for RecordId {
    type Err = RecordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(RecordId(s.to_string()))
    }
}

impl fmt::Display for RecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RecordId {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<RecordId> for String {
    #[inline]
    fn from(id: RecordId) -> Self {
        id.0
    }
}

impl TryFrom<String> for RecordId {
    type Error = RecordError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::validate(&s)?;
        Ok(RecordId(s))
    }
}

impl PartialEq<str> for RecordId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for RecordId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Ord for RecordId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for RecordId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod record_id_tests {
    use super::*;
    use crate::RecordId;
    use std::str::FromStr;

    #[test]
    fn test_valid_record_id_creation() {
        let id = RecordId::from_str("record_1234567890").unwrap();
        assert_eq!(id.as_str(), "record_1234567890");
        assert_eq!(id.len(), 17);
    }

    #[test]
    fn test_reject_short_id() {
        assert!(RecordId::from_str("short").is_err());
        assert_eq!(
            RecordId::from_str("short").unwrap_err(),
            RecordError::IdTooShort(5)
        );
    }

    #[test]
    fn test_reject_long_id() {
        let long = "a".repeat(200);
        assert!(RecordId::from_str(&long).is_err());
        assert_eq!(
            RecordId::from_str(&long).unwrap_err(),
            RecordError::IdTooLong(200)
        );
    }

    #[test]
    fn test_reject_invalid_chars() {
        assert!(RecordId::from_str("valid@chars!").is_err());
        assert!(RecordId::from_str("has spaces").is_err());
        assert!(RecordId::from_str("dash-also").is_err());
    }

    #[test]
    fn test_valid_special_chars() {
        // Underscores are valid
        let id = RecordId::from_str("record_with_underscores").unwrap();
        assert_eq!(id.as_str(), "record_with_underscores");

        // Numbers are valid
        let id = RecordId::from_str("1234567890").unwrap();
        assert_eq!(id.as_str(), "1234567890");

        // Mixed case is valid
        let id = RecordId::from_str("MixedCase_123").unwrap();
        assert_eq!(id.as_str(), "MixedCase_123");
    }

    #[test]
    fn test_uuid_conversion() {
        let uuid = uuid::Uuid::new_v4();
        let id = RecordId::from_uuid(uuid);
        assert_eq!(id.to_uuid(), Some(uuid));
    }

    #[test]
    fn test_uuid_roundtrip() {
        let original_uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id = RecordId::from_uuid(original_uuid);
        let recovered = id.to_uuid().unwrap();
        assert_eq!(original_uuid, recovered);
    }

    #[test]
    fn test_from_u64() {
        let id = RecordId::from_u64(12345);
        assert_eq!(id.as_str(), "id_00000000000000012345");
        assert!(id.fast_eq(&RecordId::from_u64(12345)));
        assert!(!id.fast_eq(&RecordId::from_u64(54321)));
    }

    #[test]
    fn test_record_id_fast_eq() {
        let id1 = RecordId::from_u64(12345);
        let id2 = RecordId::from_u64(12345);
        let id3 = RecordId::from_u64(54321);

        assert!(id1.fast_eq(&id2));
        assert!(!id1.fast_eq(&id3));
    }

    #[test]
    fn test_fast_eq_different_lengths() {
        let id1 = RecordId::from_u64(12345);
        let id2 = RecordId::from_str("id_0000000000000001234").unwrap(); // Different length

        assert!(!id1.fast_eq(&id2));
    }

    #[test]
    fn test_display_trait() {
        let id = RecordId::from_str("test_record_001").unwrap();
        assert_eq!(format!("{}", id), "test_record_001");
    }

    #[test]
    fn test_as_ref_trait() {
        let id = RecordId::from_str("test_record_001").unwrap();
        assert_eq!(id.as_ref(), "test_record_001");
    }

    #[test]
    fn test_boundary_values() {
        // Exactly 10 characters (minimum)
        let id = RecordId::from_str("abcdefghij").unwrap();
        assert_eq!(id.len(), 10);

        // Exactly 128 characters (maximum)
        let id = RecordId::from_str(&"a".repeat(128)).unwrap();
        assert_eq!(id.len(), 128);
    }

    #[test]
    fn test_try_from_string() {
        let s = String::from("test_record_001");
        let id = RecordId::try_from(s).unwrap();
        assert_eq!(id.as_str(), "test_record_001");
    }

    #[test]
    fn test_into_string() {
        let id = RecordId::from_str("test_record_001").unwrap();
        let s: String = id.into();
        assert_eq!(s, "test_record_001");
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;

        let id1 = RecordId::from_str("hash_test_001").unwrap();
        let id2 = RecordId::from_str("hash_test_001").unwrap();
        let id3 = RecordId::from_str("hash_test_002").unwrap();

        let mut set = HashSet::new();
        set.insert(id1.clone());
        set.insert(id2.clone());

        // Same values should have same hash
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3));
    }
}
