//! ArchFlow Renderers - High-Performance 2D Batch Rendering
//!
//! This crate provides WebGPU-based batch rendering for 2D graphics
//! with instancing support for maximum performance.
//!
//! ## Features
//!
//! - **Batch Rendering**: Group objects by material for minimal GPU state changes
//! - **WebGPU Instancing**: Efficient GPU rendering with per-instance transforms
//! - **Zero-Copy**: Uses `bytemuck` for direct GPU buffer upload
//! - **O(C) Complexity**: Only processes visible/changed records
//!
//! ## Architecture
//!
//! ```text
//! Renderable (Trait)
//!     ↓
//! BatchRenderer2D (organizes by material)
//!     ↓
//! RenderContext (WebGPU rendering)
//!     ↓
//! GPU (instanced draw calls)
//! ```
//!
//! ## Quick Start
//!
//! ```ignore
//! use archflow_renderers::{Renderable, BatchRenderer2D, RenderContext, Bounds, RgbaColor};
//! use wgpu::TextureFormat;
//!
//! struct MyObject {
//!     bounds: Bounds,
//!     color: RgbaColor,
//! }
//!
//! impl Renderable for MyObject {
//!     fn bounds(&self) -> Option<Bounds> { Some(self.bounds) }
//!     fn contains_point(&self, _: Vec2) -> bool { false }
//!     fn render_priority(&self) -> i32 { 0 }
//!     fn material_id(&self) -> u64 { 1 }
//!     fn color(&self) -> RgbaColor { self.color }
//! }
//!
//! // In your render loop:
//! let mut renderer = BatchRenderer2D::new(10_000);
//! renderer.prepare_frame(&visible_ids, &store);
//! context.render(&view, &renderer, &index_buffer);
//! ```

pub mod batch_renderer;
pub mod render_context;
pub mod traits;

pub use batch_renderer::{BatchRenderer2D, InstanceRaw};
pub use render_context::{RenderContext, RenderContextError};
pub use traits::{Bounds, Renderable, RgbaColor};

#[cfg(test)]
mod tests {
    use crate::{Bounds, Renderable, RgbaColor};
    use glam::Vec2;

    /// Simple test renderable for doctest examples
    #[derive(Clone, Debug, PartialEq)]
    struct SimpleRenderable {
        bounds: Bounds,
        color: RgbaColor,
        priority: i32,
        material_id: u64,
    }

    impl SimpleRenderable {
        fn new(bounds: Bounds, color: RgbaColor) -> Self {
            Self {
                bounds,
                color,
                priority: 0,
                material_id: 1,
            }
        }
    }

    impl Renderable for SimpleRenderable {
        fn bounds(&self) -> Option<Bounds> {
            Some(self.bounds)
        }

        fn contains_point(&self, point: Vec2) -> bool {
            self.bounds.contains(point)
        }

        fn render_priority(&self) -> i32 {
            self.priority
        }

        fn material_id(&self) -> u64 {
            self.material_id
        }

        fn color(&self) -> RgbaColor {
            self.color
        }
    }

    #[test]
    fn test_simple_renderable() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let color = RgbaColor::red();
        let renderable = SimpleRenderable::new(bounds, color);

        assert_eq!(renderable.bounds().unwrap(), bounds);
        assert_eq!(renderable.color(), color);
        assert!(renderable.contains_point(Vec2::new(50.0, 50.0)));
    }
}
