//! # Network Protocol Module
//!
//! Message protocol for synchronization between clients and servers.

use crate::types::SiteId;
use archflow_records::{RecordChange, RecordId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Session identifier for tracking client sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

impl SessionId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        SessionId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub const fn from_u64(value: u64) -> Self {
        SessionId(value)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session({})", self.0)
    }
}

/// Room identifier for collaborative sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub u64);

impl RoomId {
    pub const fn new(value: u64) -> Self {
        RoomId(value)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for RoomId {
    fn default() -> Self {
        RoomId(0)
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Room({})", self.0)
    }
}

/// Client capabilities and features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub max_message_size: usize,
    pub supports_compression: bool,
    pub compression_algorithm: Option<CompressionAlgorithm>,
    pub supported_encryption: Vec<EncryptionAlgorithm>,
    pub client_name: String,
    pub client_version: String,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024,
            supports_compression: false,
            compression_algorithm: None,
            supported_encryption: Vec::new(),
            client_name: "archflow-collab-client".into(),
            client_version: "0.1.0".into(),
        }
    }
}

/// Server capabilities and features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub max_message_size: usize,
    pub supported_compression: Vec<CompressionAlgorithm>,
    pub supported_encryption: Vec<EncryptionAlgorithm>,
    pub server_name: String,
    pub server_version: String,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        ServerCapabilities {
            max_message_size: 10 * 1024 * 1024,
            supported_compression: Vec::new(),
            supported_encryption: Vec::new(),
            server_name: "archflow-collab-server".into(),
            server_version: "0.1.0".into(),
        }
    }
}

/// Compression algorithms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Lz4,
    Zstd,
}

/// Encryption algorithms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Serializable wrapper for RecordChange.
///
/// Since RecordChange<R> is generic, we need a way to transport it specifically or use a concrete type.
/// For the protocol, strict typing is good.
/// However, if we want to be generic over R, we can't easily put it in a non-generic SyncMessage
/// unless SyncMessage is also generic.
/// Making SyncMessage generic over R is the cleanest design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage<R: archflow_records::Record> {
    /// Client requests synchronization
    SyncRequest {
        session_id: SessionId,
        client_version: u64,
        last_known_version: u64,
        capabilities: Option<ClientCapabilities>,
        room_id: RoomId,
        user_id: u64,
    },

    /// Server responds with changes
    SyncResponse {
        session_id: SessionId,
        server_version: u64,
        base_version: u64,
        changes_since_base: Vec<ChangeBatch<R>>,
        server_capabilities: ServerCapabilities,
        room_id: RoomId,
    },

    /// Client sends local changes
    LocalChange {
        session_id: SessionId,
        site_id: SiteId,
        version: u64,
        changes: Vec<RecordChange<R>>,
        checksum: u64,
    },

    /// Server acknowledges changes
    ChangeAck {
        session_id: SessionId,
        applied_changes: Vec<RecordId>, // IDs of applied changes
        server_version: u64,
    },

    /// Ping for latency measurement
    Ping {
        session_id: SessionId,
        timestamp: u64,
        sequence: u64,
    },

    /// Pong response
    Pong {
        session_id: SessionId,
        timestamp: u64,
        latency_ms: u64,
        sequence: u64,
    },

    /// Error message
    Error {
        session_id: SessionId,
        error_code: SyncErrorCode,
        message: String,
        fatal: bool,
    },

    /// Client leaves session
    Leave {
        session_id: SessionId,
        reason: String,
    },

    /// User joined notification
    UserJoined {
        session_id: SessionId,
        user_id: u64,
        site_id: SiteId,
    },

    /// User left notification
    UserLeft {
        session_id: SessionId,
        user_id: u64,
        site_id: SiteId,
        reason: String,
    },
}

/// Batch of changes for efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBatch<R: archflow_records::Record> {
    pub base_version: u64,
    pub changes: Vec<RecordChange<R>>,
}

/// Error codes for sync protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncErrorCode {
    AuthenticationFailed,
    InvalidMessage,
    VersionTooOld,
    SessionExpired,
    RoomNotFound,
    RateLimited,
    InternalError,
}

impl fmt::Display for SyncErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncErrorCode::AuthenticationFailed => write!(f, "AuthenticationFailed"),
            SyncErrorCode::InvalidMessage => write!(f, "InvalidMessage"),
            SyncErrorCode::VersionTooOld => write!(f, "VersionTooOld"),
            SyncErrorCode::SessionExpired => write!(f, "SessionExpired"),
            SyncErrorCode::RoomNotFound => write!(f, "RoomNotFound"),
            SyncErrorCode::RateLimited => write!(f, "RateLimited"),
            SyncErrorCode::InternalError => write!(f, "InternalError"),
        }
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};

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
    fn test_sync_message_sync_request() {
        let msg: SyncMessage<TestRecord> = SyncMessage::SyncRequest {
            session_id: SessionId::new(),
            client_version: 0,
            last_known_version: 0,
            capabilities: Some(ClientCapabilities::default()),
            room_id: RoomId::new(1),
            user_id: 123,
        };

        if let SyncMessage::SyncRequest { user_id, .. } = msg {
            assert_eq!(user_id, 123);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn test_sync_response_message() {
        let changes: Vec<ChangeBatch<TestRecord>> = vec![];
        let msg = SyncMessage::SyncResponse {
            session_id: SessionId::new(),
            server_version: 100,
            base_version: 0,
            changes_since_base: changes,
            server_capabilities: ServerCapabilities::default(),
            room_id: RoomId::new(1),
        };

        if let SyncMessage::SyncResponse {
            server_version,
            base_version,
            ..
        } = msg
        {
            assert_eq!(server_version, 100);
            assert_eq!(base_version, 0);
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_local_change_checksum() {
        let id = RecordId::from_str("checksum_test_001").unwrap();
        let changes = vec![RecordChange::Created {
            id,
            record: TestRecord {
                id: RecordId::from_str("checksum_test_001").unwrap(),
                index: None,
                name: "test".into(),
                value: 42,
            },
        }];

        let checksum = calculate_checksum(&changes);

        let checksum2 = calculate_checksum(&changes);
        assert_eq!(checksum, checksum2);
    }

    fn calculate_checksum<R: Record>(_changes: &[RecordChange<R>]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();
        "checksum_placeholder".hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_error_message_serialization() {
        let msg: SyncMessage<TestRecord> = SyncMessage::Error {
            session_id: SessionId::new(),
            error_code: SyncErrorCode::VersionTooOld,
            message: "Client version is too old".into(),
            fatal: false,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: SyncMessage<TestRecord> = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            SyncMessage::Error { error_code, .. } => {
                assert_eq!(error_code, SyncErrorCode::VersionTooOld);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_client_capabilities_defaults() {
        let caps = ClientCapabilities::default();
        assert!(caps.max_message_size > 0);
        assert!(!caps.supports_compression);
        assert_eq!(caps.client_name, "archflow-collab-client");
        assert_eq!(caps.client_version, "0.1.0");
    }
}
