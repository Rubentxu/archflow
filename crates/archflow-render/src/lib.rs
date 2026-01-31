// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGPU Rendering System
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 6, 9, 10
//
// This crate contains the WebGPU rendering pipeline:
// - 2D Infinite Camera with zoom-to-cursor
// - Multi-Phase Instancing renderer
// - Texture Atlas with Shelf Packing
// - MTSDF text rendering
// - Shader pipelines for shapes, icons, images, text
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod atlas;
pub mod camera;
pub mod gpu_resources;
pub mod pipelines;
pub mod renderer;
pub mod shaders;
pub mod webgpu_context;

pub use atlas::{AtlasPacker, AtlasRect};
pub use camera::{Camera, ZOOM_INTENSITY, ZOOM_MAX, ZOOM_MIN};
pub use gpu_resources::GpuResources;
pub use pipelines::RenderPipelines;
pub use renderer::{CameraUniforms, GpuInstance, GpuRenderer, RenderPhase};
pub use shaders::{SHADER_ICON_TEXTURE, SHADER_IMAGE_ARRAY, SHADER_MTSDF_TEXT, SHADER_SDF_SHAPES};
pub use webgpu_context::WebGpuContext;
