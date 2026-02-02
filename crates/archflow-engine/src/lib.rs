// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - Data Layer (EntityStore SOA, SpatialHash, Commands)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 4, 5, 8
//
// This crate contains the core data structures for entity management:
// - EntityStore with Structure of Arrays (SoA) for cache efficiency
// - SpatialHash for O(1) spatial queries
// - Command queue for action processing
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod command;
pub mod command_log;
pub mod compression;
pub mod connection_store;
pub mod history;
pub mod security;
pub mod spatial;
pub mod store;

pub use command::{Command, CommandQueue};
pub use command_log::{CommandError, CommandLog, CommandLogMetadata};
pub use compression::{BatchBuilder, CompressedBatch, CompressionResult, CompressionSettings};
pub use connection_store::{AnchorSide, ConnectionStore, LineStyle, MAX_CONNECTIONS};
pub use history::{CommandGroup, CommandGroupBuilder, CommandHistory};
pub use security::{
    AuditEntry, AuditEventType, AuditLog, HmacSigner, ParameterSanitizer, Permission,
    PermissionChecker, SecuredCommand, SecurityError, SecurityResult, SecurityService, TokenBucket,
    UserRateLimiter,
};
pub use spatial::SpatialHash;
pub use store::{
    ArchitectureData, EntityStore, ShapeType, StringPool, MAX_ENTITIES, MAX_GLYPHS, MAX_TEXT_LENGTH,
};
