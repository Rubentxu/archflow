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

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod command;
pub mod spatial;
pub mod store;

pub use command::{Command, CommandQueue};
pub use spatial::SpatialHash;
pub use store::{
    ArchitectureData, EntityStore, ShapeType, StringPool, MAX_CONNECTIONS, MAX_ENTITIES,
    MAX_GLYPHS, MAX_TEXT_LENGTH,
};
