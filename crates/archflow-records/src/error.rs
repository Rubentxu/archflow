//! Error types for archflow-records

use thiserror::Error;

/// Main error type for record operations
#[derive(Debug, Error, PartialEq)]
pub enum RecordError {
    /// Invalid record ID format or length
    #[error("Invalid record ID: {0}")]
    InvalidId(String),

    /// Record ID is too short (minimum 10 characters)
    #[error("Record ID too short: minimum 10 characters, got {0}")]
    IdTooShort(usize),

    /// Record ID is too long (maximum 128 characters)
    #[error("Record ID too long: maximum 128 characters, got {0}")]
    IdTooLong(usize),

    /// Invalid characters in record ID
    #[error("Invalid characters in record ID: {0}")]
    InvalidIdChars(String),

    /// Record not found in store
    #[error("Record not found")]
    NotFound,

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Invalid fractional index
    #[error("Invalid fractional index: {0}")]
    InvalidIndex(String),

    /// Index bloat - needs rebalancing
    #[error("Index bloat detected, rebalancing required")]
    IndexBloat,

    /// Undo/redo history exhausted
    #[error("No more history available")]
    NoHistory,

    /// Concurrent modification detected
    #[error("Concurrent modification detected, version mismatch")]
    ConcurrentModify,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
