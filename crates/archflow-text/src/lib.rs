// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Text - MTSDF Text Rendering System
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 12
//
// This crate contains the text rendering system:
// - MTSDF (Multi-channel Signed Distance Field) atlas
// - Glyph run cache for layout reuse
// - cosmic-text integration for shaping
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod cache;
pub mod layout;
pub mod mtsdf;
pub mod sdf;

pub use cache::{FlatGlyphRun, GlyphRunCache};
pub use layout::{FontId, TextLayoutSystem, DEFAULT_FONT_SIZE};
pub use mtsdf::{GlyphKey, MtsdfAtlas};
pub use sdf::{generate_sdf_glyph, SdfConfig, SdfGenerator};
