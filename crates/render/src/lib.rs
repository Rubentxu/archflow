//! # ArchFlow Render - Bounded Context for Rendering Operations
//!
//! This crate consolidates all rendering-related functionality:
//! - **Renderers** (from `archflow-renderers/`): GPU and software rendering
//! - **Background** (from `archflow-sdk/src/background/`): Grid and background rendering

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub use archflow_core::{Color, Rect, Vec2};

/// Render layer for organizing rendering operations
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RenderLayer {
    /// Layer ID
    pub id: u32,
    /// Layer Z-index
    pub z_index: i32,
    /// Whether the layer is visible
    pub visible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_crate_exists() {
        let layer = RenderLayer {
            id: 1,
            z_index: 0,
            visible: true,
        };
        assert_eq!(layer.id, 1);
    }
}
