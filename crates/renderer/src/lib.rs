//! ArchFlow Renderer - Canvas 2D rendering for WASM
//!
//! This module provides both the new Canvas 2D abstraction for WASM
//! and the legacy tessellation-based rendering for desktop.

mod path;
mod text;

pub use path::{
    FillRule, FillStyle, LineCap, LineJoin, PathTessellator, StrokeStyle, TessellatedPath,
};

pub use text::{
    FontManager, FontStyle, FontWeight, GlyphCacheEntry, ShapedLine, TextAlignment, TextBuffer,
    TextRenderer, TextStyle, TextWrap,
};

use archflow_ecs::Color;
use std::any::Any;
use std::fmt;

/// Result type for renderer operations
pub type RendererResult<T> = Result<T, RendererError>;

/// Renderer errors
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Canvas error: {0}")]
    CanvasError(String),

    #[error("Context error: {0}")]
    ContextError(String),

    #[error("Path error: {0}")]
    PathError(String),
}

/// Shape type for batch rendering
#[derive(Debug, Clone)]
pub enum ShapeType {
    Rectangle {
        corner_radius: f64,
    },
    Ellipse,
    Text {
        text: String,
        font_size: f64,
        font_family: String,
    },
}

/// A shape queued for rendering
#[derive(Debug, Clone)]
pub struct ShapeBatchItem {
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: f64,
}

/// 2D vertex for potential GPU rendering (placeholder)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// Builder for renderer configuration
#[derive(Debug, Clone)]
pub struct RendererBuilder {
    width: u32,
    height: u32,
    samples: u32,
    vsync: bool,
}

impl Default for RendererBuilder {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            samples: 4,
            vsync: true,
        }
    }
}

impl RendererBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_samples(mut self, samples: u32) -> Self {
        self.samples = samples;
        self
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
}

/// Canvas rendering context trait for abstraction
pub trait CanvasContext: fmt::Debug {
    /// Clear the canvas
    fn clear_rect(&self, x: f64, y: f64, width: f64, height: f64);
    /// Set fill style
    fn set_fill_style(&self, style: &str);
    /// Set stroke style
    fn set_stroke_style(&self, style: &str);
    /// Set line width
    fn set_line_width(&self, width: f64);
    /// Set font
    fn set_font(&self, font: &str);
    /// Fill rectangle
    fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64);
    /// Fill ellipse
    fn ellipse(
        &self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<(), String>;
    /// Fill text
    fn fill_text(&self, text: &str, x: f64, y: f64) -> Result<(), String>;
    /// Stroke path
    fn stroke(&self);
    /// Begin a new path
    fn begin_path(&self);
    /// Fill current path
    fn fill(&self);
    /// Fill with path
    fn fill_with_path(&self, path: &dyn CanvasPath);
}

/// Path abstraction for Canvas 2D
pub trait CanvasPath: fmt::Debug {
    /// Get reference as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn move_to(&self, x: f64, y: f64);
    fn line_to(&self, x: f64, y: f64);
    fn quadratic_curve_to(&self, cp_x: f64, cp_y: f64, x: f64, y: f64);
    fn close_path(&self);
}

/// Main renderer struct for Canvas 2D
pub struct Renderer {
    size: (u32, u32),
    /// Canvas context (set when initialized with canvas)
    context: Option<Box<dyn CanvasContext>>,
    /// Queued shapes for batch rendering
    pub shape_queue: Vec<ShapeBatchItem>,
    /// Background color
    background_color: Color,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(RendererBuilder::default())
    }
}

impl Renderer {
    /// Creates a new renderer
    pub fn new(config: RendererBuilder) -> Self {
        Self {
            size: (config.width, config.height),
            context: None,
            shape_queue: Vec::new(),
            background_color: Color::new(0.102, 0.102, 0.180, 1.0), // #1a1a2e
        }
    }

    /// Set the canvas context for rendering (used by WASM layer)
    pub fn set_context(&mut self, context: Box<dyn CanvasContext>) {
        self.context = Some(context);
    }

    /// Check if renderer has a valid context
    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }

    /// Queue a rectangle for rendering
    pub fn queue_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.shape_queue.push(ShapeBatchItem {
            shape_type: ShapeType::Rectangle { corner_radius: 8.0 },
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            color,
            stroke_color: None,
            stroke_width: 0.0,
        });
    }

    /// Queue a rounded rectangle for rendering
    pub fn queue_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        corner_radius: f32,
        color: Color,
    ) {
        self.shape_queue.push(ShapeBatchItem {
            shape_type: ShapeType::Rectangle {
                corner_radius: corner_radius as f64,
            },
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            color,
            stroke_color: None,
            stroke_width: 0.0,
        });
    }

    /// Queue an ellipse for rendering
    pub fn queue_ellipse(&mut self, cx: f32, cy: f32, radius_x: f32, radius_y: f32, color: Color) {
        self.shape_queue.push(ShapeBatchItem {
            shape_type: ShapeType::Ellipse,
            x: cx as f64,
            y: cy as f64,
            width: radius_x as f64 * 2.0,
            height: radius_y as f64 * 2.0,
            color,
            stroke_color: None,
            stroke_width: 0.0,
        });
    }

    /// Queue text for rendering
    pub fn queue_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        font_family: &str,
        color: Color,
    ) {
        self.shape_queue.push(ShapeBatchItem {
            shape_type: ShapeType::Text {
                text: text.to_string(),
                font_size: font_size as f64,
                font_family: font_family.to_string(),
            },
            x: x as f64,
            y: y as f64,
            width: 0.0,
            height: 0.0,
            color,
            stroke_color: None,
            stroke_width: 0.0,
        });
    }

    /// Clear all queued shapes
    pub fn clear_queue(&mut self) {
        self.shape_queue.clear();
    }

    /// Render all queued shapes to the canvas
    pub fn render(&mut self) -> RendererResult<()> {
        // Check if we have a canvas context
        let ctx = match &self.context {
            Some(ctx) => ctx,
            None => {
                // No canvas context available, skip rendering
                return Ok(());
            }
        };

        // Clear canvas with background color - overwrite entire canvas
        let bg = &self.background_color;
        let bg_color = format!(
            "rgb({},{},{})",
            (bg.r * 255.0) as u8,
            (bg.g * 255.0) as u8,
            (bg.b * 255.0) as u8
        );
        ctx.set_fill_style(&bg_color);

        // Clear entire canvas with a full-size rectangle (more reliable than clearRect)
        ctx.fill_rect(0.0, 0.0, self.size.0 as f64, self.size.1 as f64);

        // Draw all queued shapes
        for item in &self.shape_queue {
            self.render_shape(ctx.as_ref(), item)?;
        }

        // Clear queue after rendering
        self.shape_queue.clear();

        Ok(())
    }

    /// Render a single shape to the context
    fn render_shape(&self, ctx: &dyn CanvasContext, item: &ShapeBatchItem) -> RendererResult<()> {
        // Set fill style
        let r = (item.color.r * 255.0) as u8;
        let g = (item.color.g * 255.0) as u8;
        let b = (item.color.b * 255.0) as u8;
        let a = item.color.a;

        let fill_style = format!("rgba({},{},{},{})", r, g, b, a);
        ctx.set_fill_style(&fill_style);

        match &item.shape_type {
            ShapeType::Rectangle { corner_radius } => {
                self.draw_rounded_rect(ctx, item, *corner_radius)?;
            }
            ShapeType::Ellipse => {
                self.draw_ellipse(ctx, item)?;
            }
            ShapeType::Text {
                text,
                font_size,
                font_family,
            } => {
                self.draw_text(ctx, item, text, *font_size, font_family)?;
            }
        }

        Ok(())
    }

    /// Draw a rounded rectangle
    /// Draw a rounded rectangle
    fn draw_rounded_rect(
        &self,
        ctx: &dyn CanvasContext,
        item: &ShapeBatchItem,
        corner_radius: f64,
    ) -> RendererResult<()> {
        let x = item.x;
        let y = item.y;
        let width = item.width;
        let height = item.height;
        let radius = corner_radius.min(width / 2.0).min(height / 2.0);
        let pi = std::f64::consts::PI;

        ctx.begin_path();

        // Top edge and Top-Right corner
        // TR arc from 270 (North) to 360/0 (East)
        ctx.ellipse(
            x + width - radius, // TR center x
            y + radius,         // TR center y
            radius,
            radius,
            0.0,
            1.5 * pi, // Start 270 deg (North)
            2.0 * pi, // End 360 deg (East)
        )
        .map_err(|e| RendererError::PathError(e))?;

        // Right edge and Bottom-Right corner
        // BR arc from 0 (East) to 90 (South)
        ctx.ellipse(
            x + width - radius,  // BR center x
            y + height - radius, // BR center y
            radius,
            radius,
            0.0,
            0.0,      // Start 0 deg (East)
            0.5 * pi, // End 90 deg (South)
        )
        .map_err(|e| RendererError::PathError(e))?;

        // Bottom edge and Bottom-Left corner
        // BL arc from 90 (South) to 180 (West)
        ctx.ellipse(
            x + radius,          // BL center x
            y + height - radius, // BL center y
            radius,
            radius,
            0.0,
            0.5 * pi, // Start 90 deg (South)
            pi,       // End 180 deg (West)
        )
        .map_err(|e| RendererError::PathError(e))?;

        // Left edge and Top-Left corner
        // TL arc from 180 (West) to 270 (North)
        ctx.ellipse(
            x + radius, // TL center x
            y + radius, // TL center y
            radius,
            radius,
            0.0,
            pi,       // Start 180 deg (West)
            1.5 * pi, // End 270 deg (North)
        )
        .map_err(|e| RendererError::PathError(e))?;

        ctx.fill();

        Ok(())
    }

    /// Draw an ellipse
    fn draw_ellipse(&self, ctx: &dyn CanvasContext, item: &ShapeBatchItem) -> RendererResult<()> {
        let rx = item.width / 2.0;
        let ry = item.height / 2.0;

        // ellipse(x, y, radiusX, radiusY, rotation, startAngle, endAngle)
        ctx.begin_path();
        ctx.ellipse(item.x, item.y, rx, ry, 0.0, 0.0, std::f64::consts::PI * 2.0)
            .map_err(|e| RendererError::CanvasError(e))?;

        ctx.fill();

        Ok(())
    }

    /// Draw text
    fn draw_text(
        &self,
        ctx: &dyn CanvasContext,
        item: &ShapeBatchItem,
        text: &str,
        font_size: f64,
        font_family: &str,
    ) -> RendererResult<()> {
        // Set font
        let font = format!("{}px {}", font_size, font_family);
        ctx.set_font(&font);

        // Draw text
        ctx.fill_text(text, item.x, item.y)
            .map_err(|e| RendererError::CanvasError(e))?;

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Clean up context reference
        self.context.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock CanvasContext for testing
    #[derive(Debug)]
    struct MockCanvasContext;

    impl CanvasContext for MockCanvasContext {
        fn clear_rect(&self, _x: f64, _y: f64, _width: f64, _height: f64) {}
        fn set_fill_style(&self, _style: &str) {}
        fn set_stroke_style(&self, _style: &str) {}
        fn set_line_width(&self, _width: f64) {}
        fn set_font(&self, _font: &str) {}
        fn fill_rect(&self, _x: f64, _y: f64, _width: f64, _height: f64) {}
        fn ellipse(
            &self,
            _x: f64,
            _y: f64,
            _radius_x: f64,
            _radius_y: f64,
            _rotation: f64,
            _start_angle: f64,
            _end_angle: f64,
        ) -> Result<(), String> {
            Ok(())
        }
        fn fill_text(&self, _text: &str, _x: f64, _y: f64) -> Result<(), String> {
            Ok(())
        }
        fn stroke(&self) {}
        fn begin_path(&self) {}
        fn fill(&self) {}
        fn fill_with_path(&self, _path: &dyn CanvasPath) {}
    }

    // Mock CanvasPath for testing
    #[derive(Debug)]
    struct MockCanvasPath;

    impl CanvasPath for MockCanvasPath {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn move_to(&self, _x: f64, _y: f64) {}
        fn line_to(&self, _x: f64, _y: f64) {}
        fn quadratic_curve_to(&self, _cp_x: f64, _cp_y: f64, _x: f64, _y: f64) {}
        fn close_path(&self) {}
    }

    #[test]
    fn test_renderer_builder_defaults() {
        let builder = RendererBuilder::new();
        assert_eq!(builder.width, 800);
        assert_eq!(builder.height, 600);
        assert_eq!(builder.samples, 4);
        assert!(builder.vsync);
    }

    #[test]
    fn test_renderer_builder_chain() {
        let renderer = Renderer::new(
            RendererBuilder::new()
                .with_size(1024, 768)
                .with_samples(1)
                .with_vsync(false),
        );
        assert_eq!(renderer.size, (1024, 768));
        assert!(!renderer.has_context());
    }

    #[test]
    fn test_shape_batch_item_rectangle() {
        let item = ShapeBatchItem {
            shape_type: ShapeType::Rectangle {
                corner_radius: 10.0,
            },
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 150.0,
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            stroke_color: None,
            stroke_width: 0.0,
        };
        assert!(matches!(item.shape_type, ShapeType::Rectangle { .. }));
    }

    #[test]
    fn test_shape_batch_item_ellipse() {
        let item = ShapeBatchItem {
            shape_type: ShapeType::Ellipse,
            x: 100.0,
            y: 100.0,
            width: 100.0,
            height: 50.0,
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            stroke_color: None,
            stroke_width: 0.0,
        };
        assert!(matches!(item.shape_type, ShapeType::Ellipse));
    }

    #[test]
    fn test_shape_batch_item_text() {
        let item = ShapeBatchItem {
            shape_type: ShapeType::Text {
                text: "Hello".to_string(),
                font_size: 24.0,
                font_family: "Arial".to_string(),
            },
            x: 50.0,
            y: 50.0,
            width: 0.0,
            height: 0.0,
            color: Color::new(0.0, 0.0, 1.0, 1.0),
            stroke_color: None,
            stroke_width: 0.0,
        };
        if let ShapeType::Text {
            text,
            font_size,
            font_family,
        } = &item.shape_type
        {
            assert_eq!(text, "Hello");
            assert_eq!(*font_size, 24.0);
            assert_eq!(font_family, "Arial");
        }
    }

    #[test]
    fn test_queue_operations() {
        let mut renderer = Renderer::new(RendererBuilder::new());

        // Queue some shapes
        renderer.queue_rect(0.0, 0.0, 100.0, 100.0, Color::new(1.0, 0.0, 0.0, 1.0));
        renderer.queue_ellipse(50.0, 50.0, 25.0, 25.0, Color::new(0.0, 1.0, 0.0, 1.0));
        renderer.queue_text(
            "Test",
            10.0,
            10.0,
            16.0,
            "Arial",
            Color::new(0.0, 0.0, 1.0, 1.0),
        );

        assert_eq!(renderer.shape_queue.len(), 3);

        // Clear queue
        renderer.clear_queue();
        assert_eq!(renderer.shape_queue.len(), 0);
    }

    #[test]
    fn test_background_color_default() {
        let renderer = Renderer::new(RendererBuilder::new());
        assert!((renderer.background_color.r - 0.102).abs() < 0.001);
        assert!((renderer.background_color.g - 0.102).abs() < 0.001);
        assert!((renderer.background_color.b - 0.180).abs() < 0.001);
        assert!((renderer.background_color.a - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_render_without_context() {
        let mut renderer = Renderer::new(RendererBuilder::new());
        renderer.queue_rect(0.0, 0.0, 100.0, 100.0, Color::new(1.0, 0.0, 0.0, 1.0));

        // Should not panic when rendering without context
        let result = renderer.render();
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_with_context() {
        let mut renderer = Renderer::new(RendererBuilder::new());
        renderer.set_context(Box::new(MockCanvasContext));
        renderer.queue_rect(0.0, 0.0, 100.0, 100.0, Color::new(1.0, 0.0, 0.0, 1.0));

        let result = renderer.render();
        assert!(result.is_ok());
    }
}
