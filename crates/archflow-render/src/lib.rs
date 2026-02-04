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

//! # ArchFlow Render - Multi-Backend Rendering System
//!
//! This crate provides rendering pipeline for ArchFlow with multiple backend support:
//! - 2D infinite camera with zoom-to-cursor
//! - Multi-phase instancing renderer
//! - Texture atlas with shelf packing
//! - MTSDF text rendering
//! - Shader pipelines for shapes, icons, images, and text
//! - Multi-backend support (WebGPU, WebGL2, Canvas 2D)
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
/// Render errors
pub mod error;
/// GPU resource management
pub mod gpu_resources;
/// Render pipelines
pub mod pipelines;
/// Multi-phase instancing renderer
pub mod renderer;
/// Renderer selector for backend detection
pub mod selector;
/// Shader sources
pub mod shaders;
/// WebGL2 context for WASM
#[cfg(feature = "wasm-bindgen")]
pub mod webgl2_context;
/// WebGL2 renderer (alternative backend)
pub mod webgl2_renderer;
/// WebGPU context wrapper
pub mod webgpu_context;

pub use atlas::{AtlasPacker, AtlasRect};
pub use camera::{Camera, ZOOM_INTENSITY, ZOOM_MAX, ZOOM_MIN};
pub use error::RenderError;
pub use gpu_resources::GpuResources;
pub use pipelines::RenderPipelines;
pub use renderer::{CameraUniforms, GpuInstance, GpuRenderer, RenderPhase, Renderer};
pub use selector::{Backend, RendererSelector};
pub use shaders::{SHADER_ICON_TEXTURE, SHADER_IMAGE_ARRAY, SHADER_MTSDF_TEXT, SHADER_SDF_SHAPES};
#[cfg(feature = "wasm-bindgen")]
pub use webgl2_context::WebGl2Context2D;
pub use webgl2_renderer::{WebGl2Context, WebGl2Program, WebGl2Renderer, draw_mode};
pub use webgpu_context::WebGpuContext;

