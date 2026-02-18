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

//! # ArchFlow Diagram - Domain Core (C4 Model)
//!
//! Pure domain model for C4 architecture diagrams following DDD principles.
//!
//! ## Architecture Reference
//! ARQUITECTURA_FINAL_V3.md - Section 3
//!
//! ## Key Concepts
//!
//! - **C4 Model**: System, Container, Component, Code entities
//! - **Domain Commands**: Intentions to change the domain
//! - **Domain Events**: Facts that happened in the domain
//! - **Aggregates**: Consistency boundaries
//!
//! ## Modules
//!
//! - [`aggregates`] - Aggregate roots for consistency boundaries
//! - [`c4`] - C4 model entities (System, Container, Component, Code)
//! - [`commands`] - Domain commands for diagram operations
//! - [`events`] - Domain events for diagram state changes

#![no_std]
#![allow(unused_imports)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Aggregate roots for consistency boundaries.
pub mod aggregates;

/// C4 model entities (System, Container, Component, Code).
pub mod c4;

/// Domain commands for diagram operations.
pub mod commands;

/// Domain events for diagram state changes.
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

/// Prelude module - Common imports for convenience.
///
/// This module re-exports all commonly used types and constants.
/// Import with `use archflow_diagram::prelude::*;` for quick access.
pub mod prelude {
    pub use super::*;
    pub use crate::c4::*;
    pub use crate::commands::*;
    pub use crate::events::*;
}
