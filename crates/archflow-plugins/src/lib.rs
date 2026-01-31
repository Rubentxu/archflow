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

// TODO: Implement Draw.io parser, SVG rasterizer
// See: ARQUITECTURA_FINAL_V3.md - Section 14

pub mod drawio;
pub mod svg;
pub mod rasterizer;

pub use drawio::{parse_library_xml, decode_drawio_data};
pub use svg::SvgRasterizer;
