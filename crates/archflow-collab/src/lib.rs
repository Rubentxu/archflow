//! # archflow-collab
//!
//! CRDT-based collaboration system for real-time document editing.

pub mod conflict;
pub mod crdt;
pub mod merge;
pub mod network;
pub mod sync_client;
pub mod sync_server;
pub mod types;

pub use conflict::{ConflictDetector, ConflictResolutionPipeline, ConflictResolver, ConflictType};
pub use crdt::CRDT;
pub use merge::{LwwStrategy, OptimisticMergeStrategy};
pub use network::{SessionId, SyncErrorCode, SyncMessage};
pub use sync_client::{RetryPolicy, SyncClient, SyncClientState};
pub use sync_server::{DefaultSyncServer, SyncServerBackend, SyncSession};
pub use types::ApplyError;
pub use types::{CausalRelation, SiteId, VectorClock};
