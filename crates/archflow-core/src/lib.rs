//! # ArchFlow Core - Shared Kernel (no_std compatible)
//!
//! This crate contains pure, immutable, Copy types that can be used in no_std
//! environments. It follows the Shared Kernel pattern from DDD.
//!
//! ## Architecture Reference
//! ARQUITECTURA_FINAL_V3.md - Section 3.2
//!
//! ## Modules
//!
//! - [`id`] - Entity identification with generation tracking
//! - [`math`] - 2D vector math and geometry primitives
//! - [`paths`] - Vector path construction and Bézier curves
//! - [`vo`] - Value objects for domain concepts
//! - [`ports`] - Port interfaces for component communication (requires std)

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

#[cfg(feature = "std")]
extern crate std;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Entity identification with generation tracking for safe entity references.
pub mod id;

/// 2D vector math and geometry primitives (Vec2, Color, Rect, Transform).
pub mod math;

/// Extended types for camera precision (Vec2f64).
pub mod types;

/// Vector path construction using Bézier curves for canvas rendering.
pub mod paths;

/// Value objects representing domain concepts (Bounds, Position, Size).
pub mod vo;

/// Port interfaces for component communication (requires std).
#[cfg(feature = "std")]
pub mod ports;

// ═══════════════════════════════════════════════════════════════════════════════
// RE-EXPORTS - Convenience for users
// ═══════════════════════════════════════════════════════════════════════════════

pub use id::{EntityId, Generation, Index};
pub use math::{Color, Rect, Transform, Vec2};
pub use paths::{Path, PathBuilder, PathCommand};
pub use types::Vec2f64;
pub use vo::{Bounds, Position, Size};

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum number of entities supported by the engine
pub const MAX_ENTITIES: u32 = 100_000;

/// Maximum number of connections supported
pub const MAX_CONNECTIONS: u32 = 200_000;

/// Maximum number of glyphs in the text buffer
pub const MAX_GLYPHS: u32 = 500_000;

/// Maximum total text length across all entities
pub const MAX_TEXT_LENGTH: u32 = 50_000;

/// Invalid entity ID (used as sentinel value)
pub const INVALID_ENTITY: EntityId = EntityId::new(u32::MAX);

// ═══════════════════════════════════════════════════════════════════════════════
// PRELUDE - Common imports for convenience
// ═══════════════════════════════════════════════════════════════════════════════

/// Prelude module - Common imports for convenience.
///
/// This module re-exports all commonly used types and constants.
/// Import with `use archflow_core::prelude::*;` for quick access.
pub mod prelude {
    pub use super::*;
    pub use crate::id::*;
    pub use crate::math::*;
    pub use crate::vo::*;
}
