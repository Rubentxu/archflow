//! ArchFlow Text Rendering Engine
//!
//! Professional-grade text rendering foundation using HarfBuzz for shaping and
//! Signed Distance Fields (SDF) for GPU-accelerated, resolution-independent rendering.
//!
//! # Architecture
//!
//! ```text
//! TextShaper (rustybuzz)
//!     ↓ Glyph positions
//! SDFAtlas (distance fields)
//!     ↓ UV coordinates
//! TextQuadGenerator
//!     ↓ Vertex quads
//! WebGPU Renderer (SDF shader)
//!     ↓ Crisp text
//! ```
//!
//! # Status
//!
//! This is a foundation implementation. Full Epic 3 implementation requires:
//! - Actual font files (Inter.ttf, Amiri.ttf, etc.) for testing
//! - Complete UnicodeBuffer integration for complex scripts
//! - SDF generation algorithms
//! - WebGPU shader implementation
//!
//! # Examples
//!
//! ```rust,no_run
//! use archflow_text::{TextShaper, TextRenderer};
//!
//! let mut shaper = TextShaper::new();
//! shaper.load_font("Inter".to_string(), font_data);
//! let glyphs = shaper.shape("Hello", 16.0);
//! ```

#![warn(missing_docs, rust_2018_idioms)]

pub mod error;
pub mod font;
pub mod renderer;
pub mod sdf;
pub mod shaping;

// Re-export commonly used types
pub use error::{TextError, TextResult};
pub use font::{FontCache, FontHandle, FontLoader};
pub use renderer::{TextQuad, TextRenderer};
pub use sdf::{SDFAtlas, SDFGenerator, SDFTexture};
pub use shaping::{GlyphId, GlyphPosition, TextShaper};
