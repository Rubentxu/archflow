// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Plugins - SVG Module (Re-exports)
//
// SVG rasterizer functionality is now in rasterizer.rs
// This module re-exports the public API for convenience.
// ═══════════════════════════════════════════════════════════════════════════════

pub use crate::rasterizer::{AtlasPacker, PackedRect, SvgRasterizer};
