// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Plugins - External Integrations
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 14
//
// This crate contains plugins for external integrations:
// - Draw.io parser (XML + Deflate + Base64)
// - SVG rasterizer to GPU
// - Library icon loading
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod drawio;
pub mod rasterizer;
pub mod svg;

pub use drawio::{decode_drawio_data, parse_library_xml, DecodeError, LibraryIcon};
pub use rasterizer::{AtlasPacker, PackedRect, SvgRasterizer};
