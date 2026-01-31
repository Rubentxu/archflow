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

// TODO: Implement MtsdfAtlas, GlyphRunCache, TextLayoutSystem
// See: ARQUITECTURA_FINAL_V3.md - Section 12

pub mod cache;
pub mod layout;
pub mod mtsdf;

pub use cache::GlyphRunCache;
pub use layout::TextLayoutSystem;
pub use mtsdf::MtsdfAtlas;
