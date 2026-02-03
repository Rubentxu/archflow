// ═══════════════════════════════════════════════════════════════════════════════
// Persistence Error Types
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use std::fmt;

/// Result type for persistence operations
pub type PersistenceResult<T> = core::result::Result<T, PersistenceError>;

/// Errors that can occur during persistence operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    /// I/O error occurred
    Io(String),
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Compression error
    Compression(String),
    /// Decompression error
    Decompression(String),
    /// Invalid format detected
    InvalidFormat(String),
    /// Version mismatch
    VersionMismatch {
        /// Expected version
        expected: u32,
        /// Actual version found
        found: u32,
    },
    /// Invalid data encountered
    InvalidData(String),
    /// Entity store error
    EntityStore(String),
    /// Spatial hash error
    SpatialHash(String),
    /// Logic bricks error
    LogicBricks(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Deserialization(msg) => write!(f, "Deserialization error: {msg}"),
            Self::Compression(msg) => write!(f, "Compression error: {msg}"),
            Self::Decompression(msg) => write!(f, "Decompression error: {msg}"),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {msg}"),
            Self::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {expected}, found {found}")
            }
            Self::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            Self::EntityStore(msg) => write!(f, "Entity store error: {msg}"),
            Self::SpatialHash(msg) => write!(f, "Spatial hash error: {msg}"),
            Self::LogicBricks(msg) => write!(f, "Logic bricks error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PersistenceError {}

impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<bincode::Error> for PersistenceError {
    fn from(err: bincode::Error) -> Self {
        match *err {
            bincode::ErrorKind::Custom(ref msg) => Self::Serialization(msg.clone()),
            _ => Self::Serialization(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PersistenceError::Io("file not found".into());
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn test_version_mismatch_display() {
        let err = PersistenceError::VersionMismatch {
            expected: 2,
            found: 1,
        };
        assert_eq!(err.to_string(), "Version mismatch: expected 2, found 1");
    }

    #[test]
    fn test_error_equality() {
        let err1 = PersistenceError::Io("test".into());
        let err2 = PersistenceError::Io("test".into());
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = PersistenceError::from(json_err);
        assert!(matches!(err, PersistenceError::Serialization(_)));
    }

    #[test]
    fn test_from_bincode_error() {
        // Create a bincode error by trying to serialize something that can't be deserialized
        use bincode::{ErrorKind, serialize};
        let data = serialize(&123u32).unwrap();
        let err: PersistenceError = bincode::deserialize::<String>(&data).unwrap_err().into();
        assert!(matches!(err, PersistenceError::Serialization(_)));
    }
}
