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

//! # ArchFlow Text - MTSDF Text Rendering System
//!
//! This crate provides high-quality text rendering using Multi-channel Signed Distance Fields (MTSDF).
//!
//! ## Architecture Reference
//! ARQUITECTURA_FINAL_V3.md - Section 12
//!
//! ## Modules
//!
//! - [`cache`] - Glyph run cache for layout reuse
//! - [`layout`] - Text layout system integration
//! - [`mtsdf`] - MTSDF atlas for glyph caching
//! - [`sdf`] - SDF (Signed Distance Field) generator

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

/// Glyph run cache for text layout reuse.
pub mod cache;

/// Text layout system with cosmic-text integration.
pub mod layout;

/// MTSDF (Multi-channel Signed Distance Field) atlas.
pub mod mtsdf;

/// SDF (Signed Distance Field) generator.
pub mod sdf;

pub use cache::{FlatGlyphRun, GlyphRunCache};
pub use layout::{DEFAULT_FONT_SIZE, FontId, TextLayoutSystem};
pub use mtsdf::{GlyphKey, MtsdfAtlas};
pub use sdf::{SdfConfig, SdfGenerator, generate_sdf_glyph};
