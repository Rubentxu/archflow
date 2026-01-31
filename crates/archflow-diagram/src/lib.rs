// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Diagram - Domain Core (C4 Architecture Model)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 3
//
// This crate contains the pure domain model for C4 architecture diagrams.
// It follows Domain-Driven Design principles and contains no infrastructure
// dependencies.
//
// Key Concepts:
// - C4 Model: System, Container, Component, Code entities
// - Domain Commands: Intentions to change the domain
// - Domain Events: Facts that happened in the domain
// - Aggregates: Consistency boundaries
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

pub mod aggregates;
pub mod c4;
pub mod commands;
pub mod events;

// ═══════════════════════════════════════════════════════════════════════════════
// RE-EXPORTS
// ═══════════════════════════════════════════════════════════════════════════════

pub use c4::{ArchitectureData, C4EntityType, C4Level, CloudProvider};
pub use commands::DiagramCommand;
pub use events::DiagramEvent;

// ═══════════════════════════════════════════════════════════════════════════════
// PRELUDE
// ═══════════════════════════════════════════════════════════════════════════════

pub mod prelude {
    pub use super::*;
    pub use crate::c4::*;
    pub use crate::commands::*;
    pub use crate::events::*;
}
