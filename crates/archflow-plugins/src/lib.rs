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

//! # ArchFlow Plugins - External Integrations
//!
//! Plugins for integrating with external diagram tools and formats.
//!
//! ## Architecture Reference
//! ARQUITECTURA_FINAL_V3.md - Section 14
//!
//! ## Modules
//!
//! - [`drawio`] - Draw.io file parser (XML + Deflate + Base64)
//! - [`rasterizer`] - SVG to GPU atlas rasterizer
//! - [`svg`] - SVG parsing utilities

#![no_std]
#![allow(unused_imports)]

extern crate alloc;

/// Draw.io file parser (XML + Deflate + Base64).
pub mod drawio;

/// SVG to GPU atlas rasterizer.
pub mod rasterizer;

/// SVG parsing utilities.
pub mod svg;

pub use drawio::{DecodeError, LibraryIcon, decode_drawio_data, parse_library_xml};
pub use rasterizer::{AtlasPacker, PackedRect, SvgRasterizer};
