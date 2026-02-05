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

//! # ArchFlow Engine - Data Layer
//!
//! Core data structures for entity management with Structure of Arrays (SoA)
//! layout for cache efficiency.
//!
//! ## Architecture Reference
//! ARQUITECTURA_FINAL_V3.md - Sections 4, 5, 8
//!
//! ## Modules
//!
//! - [`command`] - Command definitions and queue
//! - [`command_log`] - Document persistence with command log
//! - [`compression`] - Batch compression for network sync
//! - [`connection_store`] - Entity connection management
//! - [`history`] - Command history for undo/redo
//! - [`security`] - Security primitives (rate limiting, signing)
//! - [`spatial`] - Spatial hash for O(1) queries
//! - [`store`] - Entity store with SoA layout

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Command definitions and queue.
pub mod command;

/// Document persistence with command log.
pub mod command_log;

/// Batch compression for network sync.
pub mod compression;

/// Entity connection management.
pub mod connection_store;

/// Command history for undo/redo.
pub mod history;

/// Security primitives (rate limiting, signing).
pub mod security;

/// Spatial hash for O(1) queries.
pub mod spatial;

/// Entity store with SoA layout.
pub mod store;

pub use command::{Command, CommandQueue, DeltaMask};
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
    ArchitectureData, EntityStore, MAX_ENTITIES, MAX_GLYPHS, MAX_TEXT_LENGTH, ShapeType, StringPool,
};
