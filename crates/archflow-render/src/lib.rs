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

//! # ArchFlow Render - WebGPU Rendering System
//!
//! This crate provides the WebGPU rendering pipeline for ArchFlow:
//! - 2D infinite camera with zoom-to-cursor
//! - Multi-phase instancing renderer
//! - Texture atlas with shelf packing
//! - MTSDF text rendering
//! - Shader pipelines for shapes, icons, images, and text
//!
//! ## Architecture Reference
//!
//! See `ARQUITECTURA_FINAL_V3.md` Sections 6, 9, 10 for detailed design.

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

/// Texture atlas with shelf packing
pub mod atlas;
/// 2D infinite camera
pub mod camera;
/// GPU resource management
pub mod gpu_resources;
/// Render pipelines
pub mod pipelines;
/// Multi-phase instancing renderer
pub mod renderer;
/// Shader sources
pub mod shaders;
/// WebGPU context wrapper
pub mod webgpu_context;

pub use atlas::{AtlasPacker, AtlasRect};
pub use camera::{Camera, ZOOM_INTENSITY, ZOOM_MAX, ZOOM_MIN};
pub use gpu_resources::GpuResources;
pub use pipelines::RenderPipelines;
pub use renderer::{CameraUniforms, GpuInstance, GpuRenderer, RenderPhase};
pub use shaders::{SHADER_ICON_TEXTURE, SHADER_IMAGE_ARRAY, SHADER_MTSDF_TEXT, SHADER_SDF_SHAPES};
pub use webgpu_context::WebGpuContext;
