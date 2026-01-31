// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGPU Rendering System
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// This crate contains the WebGPU rendering pipeline:
// - Multi-Phase Instancing renderer
// - Texture Atlas with Shelf Packing
// - MTSDF text rendering
// - Shader pipelines for shapes, icons, images, text
// ═══════════════════════════════════════════════════════════════════════════════

// TODO: Implement GpuRenderer, AtlasPacker, MtsdfAtlas
// See: ARQUITECTURA_FINAL_V3.md - Sections 9, 10, 12

pub mod atlas;
pub mod camera;
pub mod renderer;

pub use atlas::AtlasPacker;
pub use camera::Camera;
pub use renderer::GpuRenderer;
