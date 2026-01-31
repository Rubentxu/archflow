// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Core - Shared Kernel (no_std compatible)
//
// This crate contains pure, immutable, Copy types that can be used in no_std
// environments. It follows the Shared Kernel pattern from DDD.
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 3.2
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

#[cfg(feature = "std")]
extern crate std;

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

pub mod id;
pub mod math;
pub mod vo;

// Ports module requires std (Box, String, Vec)
#[cfg(feature = "std")]
pub mod ports;

// ═══════════════════════════════════════════════════════════════════════════════
// RE-EXPORTS - Convenience for users
// ═══════════════════════════════════════════════════════════════════════════════

pub use id::{EntityId, Generation, Index};
pub use math::{Color, Rect, Transform, Vec2};
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

pub mod prelude {
    pub use super::*;
    pub use crate::id::*;
    pub use crate::math::*;
    pub use crate::vo::*;
}
