// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - Core Component Factories
//
// EPIC-WASM-101 - Exponer Componentes ECS Core a JavaScript
// Compatible con API fluida: .insert(Transform.at(...))
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

// ============================================================================
// TRANSFORM FACTORY
// ============================================================================

/// Transform Component - Position, size for entities
#[wasm_bindgen]
pub struct Transform {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    scale_x: f32,
    scale_y: f32,
}

#[wasm_bindgen]
impl Transform {
    /// Create transform at position (x, y) with default size (50x50)
    #[wasm_bindgen]
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: 50.0,
            height: 50.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Create transform at position with size
    #[wasm_bindgen]
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Alias for withSize JavaScript style
    #[wasm_bindgen]
    pub fn withSize(mut self, width: f32, height: f32) -> Self {
        self.with_size(width, height)
    }

    /// Set position
    #[wasm_bindgen]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set rotation in degrees
    #[wasm_bindgen]
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Set scale
    #[wasm_bindgen]
    pub fn scale(mut self, scale_x: f32, scale_y: f32) -> Self {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
        self
    }

    #[wasm_bindgen]
    pub fn x(&self) -> f32 {
        self.x
    }
    #[wasm_bindgen]
    pub fn y(&self) -> f32 {
        self.y
    }
    #[wasm_bindgen]
    pub fn width(&self) -> f32 {
        self.width
    }
    #[wasm_bindgen]
    pub fn height(&self) -> f32 {
        self.height
    }
    #[wasm_bindgen]
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation
    }
    #[wasm_bindgen]
    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }
    #[wasm_bindgen]
    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }

    /// Get component type identifier (for builder detection)
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "transform".to_string()
    }
}

/// Transform factory namespace
#[wasm_bindgen]
pub struct TransformFactory;

#[wasm_bindgen]
impl TransformFactory {
    #[wasm_bindgen]
    pub fn at(x: f32, y: f32) -> Transform {
        Transform::at(x, y)
    }
    #[wasm_bindgen]
    pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Transform {
        Transform::at(x, y).with_size(width, height)
    }
}

// ============================================================================
// SHAPE FACTORY
// ============================================================================

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum JsShapeType {
    Rectangle = 0,
    Circle = 1,
    Ellipse = 2,
    Triangle = 3,
    Diamond = 4,
    Cylinder = 5,
    Line = 6,
    Arc = 7,
}

impl From<JsShapeType> for u8 {
    fn from(val: JsShapeType) -> Self {
        val as u8
    }
}

/// Shape Component - Visual shape for entities
#[wasm_bindgen]
pub struct Shape {
    shape_type: JsShapeType,
    radius: f32,
    corner_radius: f32,
}

#[wasm_bindgen]
impl Shape {
    #[wasm_bindgen]
    pub fn rectangle() -> Self {
        Self {
            shape_type: JsShapeType::Rectangle,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }
    #[wasm_bindgen]
    pub fn circle() -> Self {
        Self {
            shape_type: JsShapeType::Circle,
            radius: 25.0,
            corner_radius: 0.0,
        }
    }
    #[wasm_bindgen]
    pub fn circle_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    #[wasm_bindgen]
    pub fn ellipse() -> Self {
        Self {
            shape_type: JsShapeType::Ellipse,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }
    #[wasm_bindgen]
    pub fn triangle() -> Self {
        Self {
            shape_type: JsShapeType::Triangle,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }
    #[wasm_bindgen]
    pub fn diamond() -> Self {
        Self {
            shape_type: JsShapeType::Diamond,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }
    #[wasm_bindgen]
    pub fn rounded(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    #[wasm_bindgen]
    pub fn shape_type(&self) -> u8 {
        self.shape_type.into()
    }
    #[wasm_bindgen]
    pub fn radius(&self) -> f32 {
        self.radius
    }
    #[wasm_bindgen]
    pub fn corner_radius(&self) -> f32 {
        self.corner_radius
    }

    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "shape".to_string()
    }
}

/// Shape factory namespace
#[wasm_bindgen]
pub struct ShapeFactory;
#[wasm_bindgen]
impl ShapeFactory {
    #[wasm_bindgen]
    pub fn rectangle() -> Shape {
        Shape::rectangle()
    }
    #[wasm_bindgen]
    pub fn circle() -> Shape {
        Shape::circle()
    }
    #[wasm_bindgen]
    pub fn ellipse() -> Shape {
        Shape::ellipse()
    }
    #[wasm_bindgen]
    pub fn triangle() -> Shape {
        Shape::triangle()
    }
    #[wasm_bindgen]
    pub fn diamond() -> Shape {
        Shape::diamond()
    }
}

// ============================================================================
// COLOR FACTORY
// ============================================================================

#[wasm_bindgen]
pub struct Color {
    fill: [u8; 4],
    stroke: [u8; 4],
    stroke_width: f32,
}

#[wasm_bindgen]
impl Color {
    #[wasm_bindgen]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            fill: [r, g, b, 255],
            stroke: [0, 0, 0, 255],
            stroke_width: 1.0,
        }
    }
    #[wasm_bindgen]
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            fill: [r, g, b, a],
            stroke: [0, 0, 0, 255],
            stroke_width: 1.0,
        }
    }
    #[wasm_bindgen]
    pub fn fill(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fill = [r, g, b, 255];
        self
    }
    #[wasm_bindgen]
    pub fn fill_alpha(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.fill = [r, g, b, a];
        self
    }
    #[wasm_bindgen]
    pub fn stroke(mut self, r: u8, g: u8, b: u8) -> Self {
        self.stroke = [r, g, b, 255];
        self
    }
    #[wasm_bindgen]
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }
    #[wasm_bindgen]
    pub fn fill_array(&self) -> Vec<u8> {
        self.fill.to_vec()
    }
    #[wasm_bindgen]
    pub fn fill_packed(&self) -> u32 {
        let [r, g, b, a] = self.fill;
        (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32
    }
    #[wasm_bindgen]
    pub fn stroke_packed(&self) -> u32 {
        let [r, g, b, a] = self.stroke;
        (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32
    }
    #[wasm_bindgen]
    pub fn stroke_width_value(&self) -> f32 {
        self.stroke_width
    }

    #[wasm_bindgen]
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            let a = if hex.len() >= 8 {
                u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
            } else {
                255
            };
            Self::rgba(r, g, b, a)
        } else {
            Self::rgb(255, 255, 255)
        }
    }

    #[wasm_bindgen]
    pub fn red() -> Self {
        Self::rgb(255, 0, 0)
    }
    #[wasm_bindgen]
    pub fn green() -> Self {
        Self::rgb(0, 255, 0)
    }
    #[wasm_bindgen]
    pub fn blue() -> Self {
        Self::rgb(0, 0, 255)
    }
    #[wasm_bindgen]
    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }
    #[wasm_bindgen]
    pub fn black() -> Self {
        Self::rgb(0, 0, 0)
    }
    #[wasm_bindgen]
    pub fn yellow() -> Self {
        Self::rgb(255, 255, 0)
    }
    #[wasm_bindgen]
    pub fn cyan() -> Self {
        Self::rgb(0, 255, 255)
    }
    #[wasm_bindgen]
    pub fn magenta() -> Self {
        Self::rgb(255, 0, 255)
    }
    #[wasm_bindgen]
    pub fn transparent() -> Self {
        Self::rgba(0, 0, 0, 0)
    }

    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "color".to_string()
    }
}

#[wasm_bindgen]
pub struct ColorFactory;
#[wasm_bindgen]
impl ColorFactory {
    #[wasm_bindgen]
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::rgb(r, g, b)
    }
    #[wasm_bindgen]
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::rgba(r, g, b, a)
    }
    #[wasm_bindgen]
    pub fn from_hex(hex: &str) -> Color {
        Color::from_hex(hex)
    }
    #[wasm_bindgen]
    pub fn red() -> Color {
        Color::red()
    }
    #[wasm_bindgen]
    pub fn green() -> Color {
        Color::green()
    }
    #[wasm_bindgen]
    pub fn blue() -> Color {
        Color::blue()
    }
    #[wasm_bindgen]
    pub fn white() -> Color {
        Color::white()
    }
    #[wasm_bindgen]
    pub fn black() -> Color {
        Color::black()
    }
}

// ============================================================================
// VISIBILITY FACTORY
// ============================================================================

#[wasm_bindgen]
pub struct Visibility {
    visible: bool,
}

#[wasm_bindgen]
impl Visibility {
    #[wasm_bindgen]
    pub fn visible() -> Self {
        Self { visible: true }
    }
    #[wasm_bindgen]
    pub fn hidden() -> Self {
        Self { visible: false }
    }
    #[wasm_bindgen]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    #[wasm_bindgen]
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "visibility".to_string()
    }
}

#[wasm_bindgen]
pub struct VisibilityFactory;
#[wasm_bindgen]
impl VisibilityFactory {
    #[wasm_bindgen]
    pub fn visible() -> Visibility {
        Visibility::visible()
    }
    #[wasm_bindgen]
    pub fn hidden() -> Visibility {
        Visibility::hidden()
    }
}

// ============================================================================
// ZORDER FACTORY
// ============================================================================

#[wasm_bindgen]
pub struct ZOrder {
    layer: i32,
}

#[wasm_bindgen]
impl ZOrder {
    #[wasm_bindgen]
    pub fn new(layer: i32) -> Self {
        Self { layer }
    }
    #[wasm_bindgen]
    pub fn front() -> Self {
        Self { layer: 100 }
    }
    #[wasm_bindgen]
    pub fn middle() -> Self {
        Self { layer: 50 }
    }
    #[wasm_bindgen]
    pub fn back() -> Self {
        Self { layer: 0 }
    }
    #[wasm_bindgen]
    pub fn ui() -> Self {
        Self { layer: 1000 }
    }
    #[wasm_bindgen]
    pub fn layer(&self) -> i32 {
        self.layer
    }
    #[wasm_bindgen]
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "zorder".to_string()
    }
}

#[wasm_bindgen]
pub struct ZOrderFactory;
#[wasm_bindgen]
impl ZOrderFactory {
    #[wasm_bindgen]
    pub fn new(layer: i32) -> ZOrder {
        ZOrder::new(layer)
    }
    #[wasm_bindgen]
    pub fn front() -> ZOrder {
        ZOrder::front()
    }
    #[wasm_bindgen]
    pub fn middle() -> ZOrder {
        ZOrder::middle()
    }
    #[wasm_bindgen]
    pub fn back() -> ZOrder {
        ZOrder::back()
    }
    #[wasm_bindgen]
    pub fn ui() -> ZOrder {
        ZOrder::ui()
    }
}

// ============================================================================
// VELOCITY FACTORY
// ============================================================================

#[wasm_bindgen]
pub struct Velocity {
    vx: f32,
    vy: f32,
    ax: f32,
    ay: f32,
}

#[wasm_bindgen]
impl Velocity {
    #[wasm_bindgen]
    pub fn new(vx: f32, vy: f32) -> Self {
        Self {
            vx,
            vy,
            ax: 0.0,
            ay: 0.0,
        }
    }
    #[wasm_bindgen]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
    #[wasm_bindgen]
    pub fn vx(&self) -> f32 {
        self.vx
    }
    #[wasm_bindgen]
    pub fn vy(&self) -> f32 {
        self.vy
    }
    #[wasm_bindgen]
    pub fn set_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.vx = vx;
        self.vy = vy;
        self
    }
    #[wasm_bindgen]
    pub fn to_array(&self) -> Vec<f32> {
        vec![self.vx, self.vy, self.ax, self.ay]
    }
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "velocity".to_string()
    }
}

#[wasm_bindgen]
pub struct VelocityFactory;
#[wasm_bindgen]
impl VelocityFactory {
    #[wasm_bindgen]
    pub fn new(vx: f32, vy: f32) -> Velocity {
        Velocity::new(vx, vy)
    }
    #[wasm_bindgen]
    pub fn zero() -> Velocity {
        Velocity::zero()
    }
}

// ============================================================================
// RENDER LAYER
// ============================================================================

#[wasm_bindgen]
pub struct RenderLayer {
    layer: i32,
    pixel_snap: bool,
}

#[wasm_bindgen]
impl RenderLayer {
    #[wasm_bindgen]
    pub fn new(layer: i32) -> Self {
        Self {
            layer,
            pixel_snap: false,
        }
    }
    #[wasm_bindgen]
    pub fn pixel_snap(mut self) -> Self {
        self.pixel_snap = true;
        self
    }
    #[wasm_bindgen]
    pub fn layer(&self) -> i32 {
        self.layer
    }
    #[wasm_bindgen]
    pub fn is_pixel_snap(&self) -> bool {
        self.pixel_snap
    }
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "renderlayer".to_string()
    }
}

#[wasm_bindgen]
pub struct RenderLayerFactory;
#[wasm_bindgen]
impl RenderLayerFactory {
    #[wasm_bindgen]
    pub fn new(layer: i32) -> RenderLayer {
        RenderLayer::new(layer)
    }
    #[wasm_bindgen]
    pub fn ui() -> RenderLayer {
        RenderLayer::new(1000).pixel_snap()
    }
}

// ============================================================================
// PIXEL GRID SNAP
// ============================================================================

#[wasm_bindgen]
pub struct PixelGridSnap {
    enabled: bool,
    snap_x: f32,
    snap_y: f32,
}

#[wasm_bindgen]
impl PixelGridSnap {
    #[wasm_bindgen]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            snap_x: 1.0,
            snap_y: 1.0,
        }
    }
    #[wasm_bindgen]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            snap_x: 1.0,
            snap_y: 1.0,
        }
    }
    #[wasm_bindgen]
    pub fn with_snap(mut self, x: f32, y: f32) -> Self {
        self.snap_x = x;
        self.snap_y = y;
        self
    }
    #[wasm_bindgen]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "pixelgridsnap".to_string()
    }
}

#[wasm_bindgen]
pub struct PixelGridSnapFactory;
#[wasm_bindgen]
impl PixelGridSnapFactory {
    #[wasm_bindgen]
    pub fn enabled() -> PixelGridSnap {
        PixelGridSnap::enabled()
    }
    #[wasm_bindgen]
    pub fn disabled() -> PixelGridSnap {
        PixelGridSnap::disabled()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_transform_creation() {
        assert_eq!(Transform::at(100.0, 200.0).x(), 100.0);
    }
    #[test]
    fn test_transform_with_size() {
        assert_eq!(
            Transform::at(100.0, 200.0).with_size(80.0, 60.0).width(),
            80.0
        );
    }
    #[test]
    fn test_shape_circle() {
        assert_eq!(Shape::circle().shape_type(), 1);
    }
    #[test]
    fn test_color_rgb() {
        assert_eq!(Color::rgb(255, 0, 0).fill_packed(), 0xFF0000FF);
    }
    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#FF0000").fill_packed(), 0xFF0000FF);
    }
    #[test]
    fn test_visibility() {
        assert!(Visibility::visible().is_visible());
    }
    #[test]
    fn test_zorder_presets() {
        assert_eq!(ZOrder::front().layer(), 100);
    }
    #[test]
    fn test_velocity() {
        assert_eq!(Velocity::new(10.0, 20.0).vx(), 10.0);
    }
}
