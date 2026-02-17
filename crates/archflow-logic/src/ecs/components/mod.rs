// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Components Module
//
// This module provides concrete component implementations for the Entity Component System.
// These components bridge the logic layer (actuators/sensors) with the ECS architecture.
//
// Components Provided:
// - SignalStateComponent: Stores signal state (BGE-style) for entities
// - MouseSensorComponent: Configuration and state for mouse interaction sensors
// - HighlightActuatorComponent: State for highlight actuator
// - SelectActuatorComponent: State for selection actuator
// - MoveActuatorComponent: State for move/drag actuator
//
// Architecture:
// - All components implement the Component trait
// - Use VecStorage for components that most entities have
// - Use SparseSet for components that few entities have
// - TDD approach with comprehensive tests
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]

use alloc::vec::Vec;
use archflow_core::EntityId;
use archflow_core::Vec2;

use crate::ecs::{Component, ComponentRegistry, VecStorage};
use crate::signals::SignalByte;

// ═══════════════════════════════════════════════════════════════════════════════
// SignalStateComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Component that stores signal state for an entity
///
/// This component wraps SignalByte to provide BGE-style signal analysis
/// for entity interaction state (hover, click, drag, etc.).
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::SignalStateComponent;
///
/// let mut component = SignalStateComponent::default();
/// component.signal.push(true);
/// assert!(component.signal.is_positive());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SignalStateComponent {
    /// The signal state (BGE-style)
    pub signal: SignalByte,
}

impl SignalStateComponent {
    /// Creates a new SignalStateComponent with default signal
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            signal: SignalByte::default(),
        }
    }

    /// Creates a SignalStateComponent with an existing SignalByte
    #[inline(always)]
    #[must_use]
    pub const fn with_signal(signal: SignalByte) -> Self {
        Self { signal }
    }
}

impl Default for SignalStateComponent {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SignalStateComponent {
    type Storage = VecStorage<SignalStateComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// MouseSensorComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for mouse sensor behavior
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseSensorConfig {
    /// Axis to test on
    pub axis: MouseAxis,
    /// Mouse mode (movement, click, etc.)
    pub mode: MouseMode,
}

/// Axis for mouse sensor testing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseAxis {
    /// X axis
    X,
    /// Y axis,
    Y,
}

/// Mouse sensor mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseMode {
    /// Movement mode
    Movement,
    /// Click mode
    Click,
    /// Hover mode
    Hover,
}

impl Default for MouseSensorConfig {
    fn default() -> Self {
        Self {
            axis: MouseAxis::X,
            mode: MouseMode::Movement,
        }
    }
}

/// Component that stores mouse sensor configuration and state
///
/// This component configures how an entity responds to mouse interactions.
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::{MouseSensorComponent, MouseSensorConfig};
///
/// let component = MouseSensorComponent::new(100);
/// assert_eq!(component.width, 100);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MouseSensorComponent {
    /// Width of the mouse sensor area
    pub width: u32,
    /// Height of the mouse sensor area
    pub height: u32,
    /// Sensor configuration
    pub config: MouseSensorConfig,
}

impl MouseSensorComponent {
    /// Creates a new MouseSensorComponent with square dimensions
    #[inline(always)]
    #[must_use]
    pub fn new(size: u32) -> Self {
        Self {
            width: size,
            height: size,
            config: MouseSensorConfig::default(),
        }
    }

    /// Creates a new MouseSensorComponent with custom dimensions
    #[inline(always)]
    #[must_use]
    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            config: MouseSensorConfig::default(),
        }
    }

    /// Creates a MouseSensorComponent with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(size: u32, config: MouseSensorConfig) -> Self {
        Self {
            width: size,
            height: size,
            config,
        }
    }
}

impl Component for MouseSensorComponent {
    type Storage = VecStorage<MouseSensorComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// HighlightActuatorComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Component that stores highlight state for an entity
///
/// This component tracks whether an entity is highlighted and stores
/// the original color for restoration.
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::HighlightActuatorComponent;
///
/// let component = HighlightActuatorComponent::new(0xFF0000FF);
/// assert_eq!(component.highlight_color, 0xFF0000FF);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HighlightActuatorComponent {
    /// Original color before highlight
    pub original_color: Option<u32>,
    /// Current highlight color
    pub highlight_color: u32,
    /// Is currently highlighted
    pub is_highlighted: bool,
}

impl HighlightActuatorComponent {
    /// Creates a new HighlightActuatorComponent
    #[inline(always)]
    #[must_use]
    pub fn new(highlight_color: u32) -> Self {
        Self {
            original_color: None,
            highlight_color,
            is_highlighted: false,
        }
    }

    /// Sets the highlighted state and stores original color
    #[inline(always)]
    pub fn set_highlighted(&mut self, original_color: u32) {
        self.original_color = Some(original_color);
        self.is_highlighted = true;
    }

    /// Clears the highlighted state
    #[inline(always)]
    pub fn clear_highlighted(&mut self) {
        self.original_color = None;
        self.is_highlighted = false;
    }
}

impl Component for HighlightActuatorComponent {
    type Storage = VecStorage<HighlightActuatorComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SelectActuatorComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Component that stores selection state for an entity
///
/// This component tracks whether an entity is selected.
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::SelectActuatorComponent;
///
/// let component = SelectActuatorComponent::new();
/// assert!(!component.is_selected);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectActuatorComponent {
    /// Is currently selected
    pub is_selected: bool,
}

impl SelectActuatorComponent {
    /// Creates a new SelectActuatorComponent
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self { is_selected: false }
    }

    /// Sets the selected state
    #[inline(always)]
    pub fn set_selected(&mut self, selected: bool) {
        self.is_selected = selected;
    }
}

impl Default for SelectActuatorComponent {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SelectActuatorComponent {
    type Storage = VecStorage<SelectActuatorComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// MoveActuatorComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Drag axis constraint
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragAxis {
    /// No constraint (free movement)
    Both,
    /// X-axis only
    X,
    /// Y-axis only
    Y,
}

/// Component that stores move/drag state for an entity
///
/// This component tracks the drag state of an entity for move operations.
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::MoveActuatorComponent;
/// use archflow_core::Vec2;
///
/// let start_pos = Vec2::new(100.0, 100.0);
/// let component = MoveActuatorComponent::new(start_pos);
/// assert_eq!(component.start_pos, start_pos);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveActuatorComponent {
    /// Original entity position when drag started
    pub start_pos: Vec2,
    /// Last mouse position for tracking
    pub last_mouse_pos: Vec2,
    /// Axis constraint for this drag
    pub axis: DragAxis,
    /// Grid snap value (0 to disable)
    pub snap: f32,
    /// Is currently being dragged
    pub is_dragging: bool,
}

impl MoveActuatorComponent {
    /// Creates a new MoveActuatorComponent
    #[inline(always)]
    #[must_use]
    pub fn new(start_pos: Vec2) -> Self {
        Self {
            start_pos,
            last_mouse_pos: start_pos,
            axis: DragAxis::Both,
            snap: 0.0,
            is_dragging: false,
        }
    }

    /// Sets the dragging state
    #[inline(always)]
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    /// Updates the last mouse position
    #[inline(always)]
    pub fn update_mouse_pos(&mut self, pos: Vec2) {
        self.last_mouse_pos = pos;
    }
}

impl Component for MoveActuatorComponent {
    type Storage = VecStorage<MoveActuatorComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ShapeType Enum - Type-safe shape types
// ═══════════════════════════════════════════════════════════════════════════════

/// Type-safe shape types for rendering
///
/// This replaces the u8 arbitrary values in EntityStore.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ShapeType {
    /// Standard rectangle shape
    Rectangle = 0,
    /// Circle shape
    Circle = 1,
    /// Ellipse shape (different aspect ratio)
    Ellipse = 2,
    /// Triangle shape
    Triangle = 3,
    /// Diamond/rhombus shape
    Diamond = 4,
    /// Cylinder shape (typically for databases)
    Cylinder = 5,
    /// Line shape
    Line = 6,
    /// Arc shape
    Arc = 7,
}

impl ShapeType {
    /// Convert from u8 (for compatibility with EntityStore)
    #[inline(always)]
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => ShapeType::Rectangle,
            1 => ShapeType::Circle,
            2 => ShapeType::Ellipse,
            3 => ShapeType::Triangle,
            4 => ShapeType::Diamond,
            5 => ShapeType::Cylinder,
            6 => ShapeType::Line,
            7 => ShapeType::Arc,
            _ => ShapeType::Rectangle,
        }
    }

    /// Convert to u8 (for compatibility with rendering)
    #[inline(always)]
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl Default for ShapeType {
    fn default() -> Self {
        ShapeType::Rectangle
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ShapeComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Component that stores shape type for an entity
///
/// This component provides type-safe shape rendering properties.
/// Use together with Transform (for position/rotation/scale) and RenderProperties (for size/layer).
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::{ShapeComponent, ShapeType};
///
/// let component = ShapeComponent::circle();
/// assert_eq!(component.shape_type, ShapeType::Circle);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeComponent {
    /// The shape type
    pub shape_type: ShapeType,
    /// Radius for circle/ellipse (0 = use default/auto)
    pub radius: f32,
    /// Corner radius for rounded rectangles
    pub corner_radius: f32,
}

impl ShapeComponent {
    /// Creates a rectangle shape (default)
    #[inline(always)]
    #[must_use]
    pub fn rectangle() -> Self {
        Self {
            shape_type: ShapeType::Rectangle,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }

    /// Creates a circle shape
    #[inline(always)]
    #[must_use]
    pub fn circle() -> Self {
        Self {
            shape_type: ShapeType::Circle,
            radius: 0.5,
            corner_radius: 0.0,
        }
    }

    /// Creates an ellipse shape
    #[inline(always)]
    #[must_use]
    pub fn ellipse() -> Self {
        Self {
            shape_type: ShapeType::Ellipse,
            radius: 0.5,
            corner_radius: 0.0,
        }
    }

    /// Creates a triangle shape
    #[inline(always)]
    #[must_use]
    pub fn triangle() -> Self {
        Self {
            shape_type: ShapeType::Triangle,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }

    /// Creates a diamond shape
    #[inline(always)]
    #[must_use]
    pub fn diamond() -> Self {
        Self {
            shape_type: ShapeType::Diamond,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }

    /// Creates a cylinder shape
    #[inline(always)]
    #[must_use]
    pub fn cylinder() -> Self {
        Self {
            shape_type: ShapeType::Cylinder,
            radius: 0.5,
            corner_radius: 0.0,
        }
    }

    /// Creates a line shape
    #[inline(always)]
    #[must_use]
    pub fn line() -> Self {
        Self {
            shape_type: ShapeType::Line,
            radius: 0.0,
            corner_radius: 0.0,
        }
    }

    /// Creates an arc shape
    #[inline(always)]
    #[must_use]
    pub fn arc() -> Self {
        Self {
            shape_type: ShapeType::Arc,
            radius: 0.5,
            corner_radius: 0.0,
        }
    }

    /// Set custom radius
    #[inline(always)]
    #[must_use]
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set corner radius for rounded shapes
    #[inline(always)]
    #[must_use]
    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

impl Default for ShapeComponent {
    fn default() -> Self {
        Self::rectangle()
    }
}

impl Component for ShapeComponent {
    type Storage = VecStorage<ShapeComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Color Helper
// ═══════════════════════════════════════════════════════════════════════════════

/// Color in ARGB format
///
/// Provides helper methods for color manipulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    /// Alpha component (0-255)
    pub a: u8,
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

impl Color {
    /// Creates a new color from ARGB components
    #[inline(always)]
    #[must_use]
    pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// Creates a new color from RGB (alpha = 255)
    #[inline(always)]
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { a: 255, r, g, b }
    }

    /// Creates a transparent color
    #[inline(always)]
    #[must_use]
    pub const fn transparent() -> Self {
        Self {
            a: 0,
            r: 0,
            g: 0,
            b: 0,
        }
    }

    /// Creates a white color
    #[inline(always)]
    #[must_use]
    pub const fn white() -> Self {
        Self {
            a: 255,
            r: 255,
            g: 255,
            b: 255,
        }
    }

    /// Creates a black color
    #[inline(always)]
    #[must_use]
    pub const fn black() -> Self {
        Self {
            a: 255,
            r: 0,
            g: 0,
            b: 0,
        }
    }

    /// Creates a red color
    #[inline(always)]
    #[must_use]
    pub const fn red() -> Self {
        Self {
            a: 255,
            r: 255,
            g: 0,
            b: 0,
        }
    }

    /// Creates a green color
    #[inline(always)]
    #[must_use]
    pub const fn green() -> Self {
        Self {
            a: 255,
            r: 0,
            g: 255,
            b: 0,
        }
    }

    /// Creates a blue color
    #[inline(always)]
    #[must_use]
    pub const fn blue() -> Self {
        Self {
            a: 255,
            r: 0,
            g: 0,
            b: 255,
        }
    }

    /// Converts to u32 (ARGB little-endian for WebGL)
    #[inline(always)]
    pub fn to_u32(self) -> u32 {
        (self.a as u32) << 24 | (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }

    /// Creates from u32
    #[inline(always)]
    #[must_use]
    pub fn from_u32(color: u32) -> Self {
        Self {
            a: ((color >> 24) & 0xFF) as u8,
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::white()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ColorComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Component that stores color properties for an entity
///
/// Provides fill, stroke, and tint for rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorComponent {
    /// Fill color
    pub fill: Color,
    /// Stroke/border color
    pub stroke: Color,
    /// Stroke width in pixels
    pub stroke_width: f32,
    /// Tint factor [r, g, b, a] multipliers
    pub tint: [f32; 4],
}

impl ColorComponent {
    /// Creates a new ColorComponent with fill color
    #[inline(always)]
    #[must_use]
    pub fn new(fill: Color) -> Self {
        Self {
            fill,
            stroke: Color::black(),
            stroke_width: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Creates with RGB values
    #[inline(always)]
    #[must_use]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(Color::rgb(r, g, b))
    }

    /// Set stroke color and width
    #[inline(always)]
    #[must_use]
    pub fn with_stroke(mut self, stroke: Color, width: f32) -> Self {
        self.stroke = stroke;
        self.stroke_width = width;
        self
    }

    /// Set tint multipliers
    #[inline(always)]
    #[must_use]
    pub fn with_tint(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.tint = [r, g, b, a];
        self
    }

    /// Get combined fill color with tint applied
    #[inline(always)]
    pub fn combined_fill(&self) -> u32 {
        let r = (self.fill.r as f32 * self.tint[0]).min(255.0) as u8;
        let g = (self.fill.g as f32 * self.tint[1]).min(255.0) as u8;
        let b = (self.fill.b as f32 * self.tint[2]).min(255.0) as u8;
        let a = (self.fill.a as f32 * self.tint[3]).min(255.0) as u8;
        Color::argb(a, r, g, b).to_u32()
    }
}

impl Default for ColorComponent {
    fn default() -> Self {
        // Default light blue
        Self::new(Color::rgb(204, 221, 238))
    }
}

impl Component for ColorComponent {
    type Storage = VecStorage<ColorComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Visibility Enum
// ═══════════════════════════════════════════════════════════════════════════════

/// Visibility state for rendering
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// Entity is visible and interactive
    Visible,
    /// Entity is hidden (not rendered)
    Hidden,
    /// Entity is visible but not interactive
    PassThrough,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Visible
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VisibilityComponent
// ═══════════════════════════════════════════════════════════════════════════════

/// Component that controls visibility of an entity
///
/// Use to hide/show entities without removing them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityComponent {
    /// Current visibility state
    pub visibility: Visibility,
}

impl VisibilityComponent {
    /// Creates a visible component
    #[inline(always)]
    #[must_use]
    pub fn visible() -> Self {
        Self {
            visibility: Visibility::Visible,
        }
    }

    /// Creates a hidden component
    #[inline(always)]
    #[must_use]
    pub fn hidden() -> Self {
        Self {
            visibility: Visibility::Hidden,
        }
    }

    /// Creates a pass-through component (visible but not interactive)
    #[inline(always)]
    #[must_use]
    pub fn pass_through() -> Self {
        Self {
            visibility: Visibility::PassThrough,
        }
    }

    /// Set visibility
    #[inline(always)]
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    /// Check if entity is visible
    #[inline(always)]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visibility == Visibility::Visible
    }
}

impl Default for VisibilityComponent {
    fn default() -> Self {
        Self::visible()
    }
}

impl Component for VisibilityComponent {
    type Storage = VecStorage<VisibilityComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// RenderProperties
// ═══════════════════════════════════════════════════════════════════════════════

/// Component for rendering properties (size and layer)
///
/// Use together with Transform (for position/rotation/scale).
/// This component provides size (width/height) and layer (z-order).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderProperties {
    /// Width in world units
    pub width: f32,
    /// Height in world units
    pub height: f32,
    /// Z-layer for rendering order
    pub layer: i32,
}

impl RenderProperties {
    /// Creates new render properties
    #[inline(always)]
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            layer: 0,
        }
    }

    /// Creates a square
    #[inline(always)]
    #[must_use]
    pub fn square(size: f32) -> Self {
        Self::new(size, size)
    }

    /// Set layer
    #[inline(always)]
    #[must_use]
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    /// Set width
    #[inline(always)]
    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    /// Set height
    #[inline(always)]
    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }
}

impl Default for RenderProperties {
    fn default() -> Self {
        Self::new(100.0, 100.0)
    }
}

impl Component for RenderProperties {
    type Storage = VecStorage<RenderProperties>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{ComponentRegistry, ComponentStorage};

    // ═══════════════════════════════════════════════════════════════════════════════
    // SignalStateComponent Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_signal_state_component_default() {
        let component = SignalStateComponent::default();
        // Default signal (0) has current bit = 0, so it's not positive
        assert!(!component.signal.is_positive());
        // and is_negative = !is_positive = true
        assert!(component.signal.is_negative());
    }

    #[test]
    fn test_signal_state_component_new() {
        let component = SignalStateComponent::new();
        assert!(!component.signal.is_positive());
    }

    #[test]
    fn test_signal_state_component_with_signal() {
        let mut signal = SignalByte::default();
        signal.push(true);

        let component = SignalStateComponent::with_signal(signal);
        assert!(component.signal.is_positive());
    }

    #[test]
    fn test_signal_state_component_push() {
        let mut component = SignalStateComponent::default();
        component.signal.push(true);
        assert!(component.signal.is_positive());
    }

    #[test]
    fn test_signal_state_component_in_registry() {
        let mut registry = ComponentRegistry::new();
        registry.register::<SignalStateComponent>();

        let mut storage = registry.get_storage_mut::<SignalStateComponent>().unwrap();
        storage.insert(0, SignalStateComponent::new());

        let storage = registry.get_storage::<SignalStateComponent>().unwrap();
        assert!(storage.contains(0));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MouseSensorComponent Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_mouse_sensor_component_new() {
        let component = MouseSensorComponent::new(100);
        assert_eq!(component.width, 100);
        assert_eq!(component.height, 100);
    }

    #[test]
    fn test_mouse_sensor_component_with_dimensions() {
        let component = MouseSensorComponent::with_dimensions(200, 150);
        assert_eq!(component.width, 200);
        assert_eq!(component.height, 150);
    }

    #[test]
    fn test_mouse_sensor_component_with_config() {
        let config = MouseSensorConfig {
            axis: MouseAxis::Y,
            mode: MouseMode::Click,
        };
        let component = MouseSensorComponent::with_config(100, config);
        assert_eq!(component.config.axis, MouseAxis::Y);
        assert_eq!(component.config.mode, MouseMode::Click);
    }

    #[test]
    fn test_mouse_sensor_config_default() {
        let config = MouseSensorConfig::default();
        assert_eq!(config.axis, MouseAxis::X);
        assert_eq!(config.mode, MouseMode::Movement);
    }

    #[test]
    fn test_mouse_sensor_axis_equality() {
        assert_eq!(MouseAxis::X, MouseAxis::X);
        assert_ne!(MouseAxis::X, MouseAxis::Y);
    }

    #[test]
    fn test_mouse_sensor_mode_equality() {
        assert_eq!(MouseMode::Movement, MouseMode::Movement);
        assert_ne!(MouseMode::Movement, MouseMode::Click);
    }

    #[test]
    fn test_mouse_sensor_component_in_registry() {
        let mut registry = ComponentRegistry::new();
        registry.register::<MouseSensorComponent>();

        let mut storage = registry.get_storage_mut::<MouseSensorComponent>().unwrap();
        storage.insert(0, MouseSensorComponent::new(100));

        let storage = registry.get_storage::<MouseSensorComponent>().unwrap();
        assert!(storage.contains(0));
        assert_eq!(storage.get(0).unwrap().width, 100);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // HighlightActuatorComponent Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_highlight_actuator_component_new() {
        let component = HighlightActuatorComponent::new(0xFF0000FF);
        assert_eq!(component.highlight_color, 0xFF0000FF);
        assert!(!component.is_highlighted);
        assert!(component.original_color.is_none());
    }

    #[test]
    fn test_highlight_actuator_component_set_highlighted() {
        let mut component = HighlightActuatorComponent::new(0xFF0000FF);
        component.set_highlighted(0x00FF00FF);

        assert!(component.is_highlighted);
        assert_eq!(component.original_color, Some(0x00FF00FF));
    }

    #[test]
    fn test_highlight_actuator_component_clear_highlighted() {
        let mut component = HighlightActuatorComponent::new(0xFF0000FF);
        component.set_highlighted(0x00FF00FF);
        component.clear_highlighted();

        assert!(!component.is_highlighted);
        assert!(component.original_color.is_none());
    }

    #[test]
    fn test_highlight_actuator_component_equality() {
        let component1 = HighlightActuatorComponent::new(0xFF0000FF);
        let component2 = HighlightActuatorComponent::new(0xFF0000FF);
        assert_eq!(component1, component2);
    }

    #[test]
    fn test_highlight_actuator_component_in_registry() {
        let mut registry = ComponentRegistry::new();
        registry.register::<HighlightActuatorComponent>();

        let mut storage = registry
            .get_storage_mut::<HighlightActuatorComponent>()
            .unwrap();
        storage.insert(0, HighlightActuatorComponent::new(0xFF0000FF));

        let storage = registry
            .get_storage::<HighlightActuatorComponent>()
            .unwrap();
        assert!(storage.contains(0));
        assert_eq!(storage.get(0).unwrap().highlight_color, 0xFF0000FF);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SelectActuatorComponent Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_actuator_component_default() {
        let component = SelectActuatorComponent::default();
        assert!(!component.is_selected);
    }

    #[test]
    fn test_select_actuator_component_new() {
        let component = SelectActuatorComponent::new();
        assert!(!component.is_selected);
    }

    #[test]
    fn test_select_actuator_component_set_selected() {
        let mut component = SelectActuatorComponent::new();
        component.set_selected(true);
        assert!(component.is_selected);

        component.set_selected(false);
        assert!(!component.is_selected);
    }

    #[test]
    fn test_select_actuator_component_equality() {
        let component1 = SelectActuatorComponent::new();
        let component2 = SelectActuatorComponent::new();
        assert_eq!(component1, component2);

        let mut component3 = SelectActuatorComponent::new();
        component3.set_selected(true);
        assert_ne!(component1, component3);
    }

    #[test]
    fn test_select_actuator_component_in_registry() {
        let mut registry = ComponentRegistry::new();
        registry.register::<SelectActuatorComponent>();

        let mut storage = registry
            .get_storage_mut::<SelectActuatorComponent>()
            .unwrap();
        storage.insert(0, SelectActuatorComponent::new());

        let storage = registry.get_storage::<SelectActuatorComponent>().unwrap();
        assert!(storage.contains(0));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MoveActuatorComponent Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_move_actuator_component_new() {
        let start_pos = Vec2::new(100.0, 100.0);
        let component = MoveActuatorComponent::new(start_pos);

        assert_eq!(component.start_pos, start_pos);
        assert_eq!(component.last_mouse_pos, start_pos);
        assert_eq!(component.axis, DragAxis::Both);
        assert_eq!(component.snap, 0.0);
        assert!(!component.is_dragging);
    }

    #[test]
    fn test_move_actuator_component_set_dragging() {
        let start_pos = Vec2::new(100.0, 100.0);
        let mut component = MoveActuatorComponent::new(start_pos);

        component.set_dragging(true);
        assert!(component.is_dragging);

        component.set_dragging(false);
        assert!(!component.is_dragging);
    }

    #[test]
    fn test_move_actuator_component_update_mouse_pos() {
        let start_pos = Vec2::new(100.0, 100.0);
        let mut component = MoveActuatorComponent::new(start_pos);

        let new_pos = Vec2::new(120.0, 130.0);
        component.update_mouse_pos(new_pos);
        assert_eq!(component.last_mouse_pos, new_pos);
    }

    #[test]
    fn test_drag_axis_equality() {
        assert_eq!(DragAxis::Both, DragAxis::Both);
        assert_eq!(DragAxis::X, DragAxis::X);
        assert_eq!(DragAxis::Y, DragAxis::Y);

        assert_ne!(DragAxis::X, DragAxis::Y);
        assert_ne!(DragAxis::Both, DragAxis::X);
    }

    #[test]
    fn test_move_actuator_component_in_registry() {
        let mut registry = ComponentRegistry::new();
        registry.register::<MoveActuatorComponent>();

        let start_pos = Vec2::new(100.0, 100.0);
        let mut storage = registry.get_storage_mut::<MoveActuatorComponent>().unwrap();
        storage.insert(0, MoveActuatorComponent::new(start_pos));

        let storage = registry.get_storage::<MoveActuatorComponent>().unwrap();
        assert!(storage.contains(0));
        assert_eq!(storage.get(0).unwrap().start_pos, start_pos);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Integration Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_multiple_components_in_registry() {
        let mut registry = ComponentRegistry::new();

        registry.register::<SignalStateComponent>();
        registry.register::<MouseSensorComponent>();
        registry.register::<HighlightActuatorComponent>();
        registry.register::<SelectActuatorComponent>();
        registry.register::<MoveActuatorComponent>();

        assert_eq!(registry.len(), 5);
        assert!(registry.is_registered::<SignalStateComponent>());
        assert!(registry.is_registered::<MouseSensorComponent>());
        assert!(registry.is_registered::<HighlightActuatorComponent>());
        assert!(registry.is_registered::<SelectActuatorComponent>());
        assert!(registry.is_registered::<MoveActuatorComponent>());
    }

    #[test]
    fn test_entity_with_multiple_components() {
        let mut registry = ComponentRegistry::new();

        registry.register::<SignalStateComponent>();
        registry.register::<MouseSensorComponent>();
        registry.register::<HighlightActuatorComponent>();
        registry.register::<SelectActuatorComponent>();
        registry.register::<MoveActuatorComponent>();

        let entity_id = 0;

        // Add all components to entity
        {
            let mut signals = registry.get_storage_mut::<SignalStateComponent>().unwrap();
            signals.insert(entity_id, SignalStateComponent::new());

            let mut mouse_sensors = registry.get_storage_mut::<MouseSensorComponent>().unwrap();
            mouse_sensors.insert(entity_id, MouseSensorComponent::new(100));

            let mut highlights = registry
                .get_storage_mut::<HighlightActuatorComponent>()
                .unwrap();
            highlights.insert(entity_id, HighlightActuatorComponent::new(0xFF0000FF));

            let mut selections = registry
                .get_storage_mut::<SelectActuatorComponent>()
                .unwrap();
            selections.insert(entity_id, SelectActuatorComponent::new());

            let start_pos = Vec2::new(100.0, 100.0);
            let mut moves = registry.get_storage_mut::<MoveActuatorComponent>().unwrap();
            moves.insert(entity_id, MoveActuatorComponent::new(start_pos));
        }

        // Verify all components are present
        let signals = registry.get_storage::<SignalStateComponent>().unwrap();
        let mouse_sensors = registry.get_storage::<MouseSensorComponent>().unwrap();
        let highlights = registry
            .get_storage::<HighlightActuatorComponent>()
            .unwrap();
        let selections = registry.get_storage::<SelectActuatorComponent>().unwrap();
        let moves = registry.get_storage::<MoveActuatorComponent>().unwrap();

        assert!(signals.contains(entity_id));
        assert!(mouse_sensors.contains(entity_id));
        assert!(highlights.contains(entity_id));
        assert!(selections.contains(entity_id));
        assert!(moves.contains(entity_id));
    }

    #[test]
    fn test_component_removal() {
        let mut registry = ComponentRegistry::new();
        registry.register::<HighlightActuatorComponent>();

        let mut storage = registry
            .get_storage_mut::<HighlightActuatorComponent>()
            .unwrap();
        storage.insert(0, HighlightActuatorComponent::new(0xFF0000FF));
        storage.insert(1, HighlightActuatorComponent::new(0x00FF00FF));

        // Remove component from entity 0
        let removed = storage.remove(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().highlight_color, 0xFF0000FF);

        // Verify removal
        assert!(!storage.contains(0));
        assert!(storage.contains(1));
    }

    #[test]
    fn test_component_mutation() {
        let mut registry = ComponentRegistry::new();
        registry.register::<SelectActuatorComponent>();

        let entity_id = 0;

        // Add component
        {
            let mut selections = registry
                .get_storage_mut::<SelectActuatorComponent>()
                .unwrap();
            selections.insert(entity_id, SelectActuatorComponent::new());
        }

        // Mutate component
        {
            let mut selections = registry
                .get_storage_mut::<SelectActuatorComponent>()
                .unwrap();
            selections
                .get_mut(entity_id)
                .map(|component: &mut SelectActuatorComponent| component.set_selected(true));
        }

        // Verify mutation
        let selections = registry.get_storage::<SelectActuatorComponent>().unwrap();
        assert!(selections.get(entity_id).unwrap().is_selected);
    }

    #[test]
    fn test_vec_storage_iteration() {
        let mut registry = ComponentRegistry::new();
        registry.register::<SignalStateComponent>();

        // Add multiple components
        let mut storage = registry.get_storage_mut::<SignalStateComponent>().unwrap();
        for i in 0..5 {
            storage.insert(i, SignalStateComponent::new());
        }

        // Iterate and count
        let storage = registry.get_storage::<SignalStateComponent>().unwrap();
        let mut count = 0;
        for _component in storage.iter() {
            count += 1;
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn test_component_id_uniqueness() {
        use crate::ecs::ComponentId;

        let signal_id = ComponentId::of::<SignalStateComponent>();
        let mouse_id = ComponentId::of::<MouseSensorComponent>();
        let highlight_id = ComponentId::of::<HighlightActuatorComponent>();
        let select_id = ComponentId::of::<SelectActuatorComponent>();
        let move_id = ComponentId::of::<MoveActuatorComponent>();

        // All IDs should be unique
        assert_ne!(signal_id, mouse_id);
        assert_ne!(signal_id, highlight_id);
        assert_ne!(signal_id, select_id);
        assert_ne!(signal_id, move_id);
        assert_ne!(mouse_id, highlight_id);
        assert_ne!(mouse_id, select_id);
        assert_ne!(mouse_id, move_id);
        assert_ne!(highlight_id, select_id);
        assert_ne!(highlight_id, move_id);
        assert_ne!(select_id, move_id);

        // Same component should have same ID
        assert_eq!(signal_id, ComponentId::of::<SignalStateComponent>());
        assert_eq!(mouse_id, ComponentId::of::<MouseSensorComponent>());
        assert_eq!(
            highlight_id,
            ComponentId::of::<HighlightActuatorComponent>()
        );
        assert_eq!(select_id, ComponentId::of::<SelectActuatorComponent>());
        assert_eq!(move_id, ComponentId::of::<MoveActuatorComponent>());
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = ComponentRegistry::new();

        registry.register::<SignalStateComponent>();
        registry.register::<MouseSensorComponent>();

        {
            let mut storage = registry.get_storage_mut::<SignalStateComponent>().unwrap();
            storage.insert(0, SignalStateComponent::new());
        }

        registry.clear();

        assert!(!registry.is_registered::<SignalStateComponent>());
        assert!(!registry.is_registered::<MouseSensorComponent>());
        assert!(registry.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AudioActuatorComponent
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Component that stores audio properties for an entity
///
/// This component allows entities to have audio playback capabilities
/// with per-entity volume, pitch, and spatial settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioActuatorComponent {
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
    /// Playback speed (0.5 to 2.0)
    pub pitch: f32,
    /// Enable looping
    pub loop_enabled: bool,
    /// Enable spatial audio (3D positioning)
    pub spatial: bool,
    /// Sound ID to play (loaded in AudioSystem)
    pub sound_id: Option<u32>,
    /// Is currently playing
    pub is_playing: bool,
}

impl Default for AudioActuatorComponent {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
            loop_enabled: false,
            spatial: false,
            sound_id: None,
            is_playing: false,
        }
    }
}

impl AudioActuatorComponent {
    /// Creates a new AudioActuatorComponent with default settings
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an AudioActuatorComponent with a specific sound
    #[inline(always)]
    #[must_use]
    pub fn with_sound(sound_id: u32) -> Self {
        Self {
            sound_id: Some(sound_id),
            ..Self::default()
        }
    }

    /// Set the volume level
    #[inline(always)]
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Set the playback pitch
    #[inline(always)]
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(0.5, 2.0);
    }

    /// Start playback
    #[inline(always)]
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Stop playback
    #[inline(always)]
    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    /// Pause playback (keeps position)
    #[inline(always)]
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
}

impl Component for AudioActuatorComponent {
    type Storage = VecStorage<AudioActuatorComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// NamedComponent - For entity naming and debugging
// ═══════════════════════════════════════════════════════════════════════════════

/// Component for storing entity name for debugging purposes.
///
/// This component allows entities to have a name that can be used
/// for logging, debugging, and identification.
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::components::NamedComponent;
///
/// let component = NamedComponent::new("Player");
/// assert_eq!(component.name(), "Player");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct NamedComponent {
    name: alloc::string::String,
}

impl NamedComponent {
    /// Creates a new NamedComponent with the given name.
    #[inline]
    #[must_use]
    pub fn new(name: impl Into<alloc::string::String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the name of the entity.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &alloc::string::String {
        &self.name
    }

    /// Sets a new name for the entity.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<alloc::string::String>) {
        self.name = name.into();
    }
}

impl Default for NamedComponent {
    fn default() -> Self {
        Self {
            name: alloc::string::String::new(),
        }
    }
}

impl Component for NamedComponent {
    type Storage = VecStorage<NamedComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ShapeType and ShapeComponent Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_shape_type_variants() {
    // All shape types should be constructible
    let _ = ShapeType::Rectangle;
    let _ = ShapeType::Circle;
    let _ = ShapeType::Ellipse;
    let _ = ShapeType::Triangle;
    let _ = ShapeType::Diamond;
    let _ = ShapeType::Cylinder;
    let _ = ShapeType::Line;
    let _ = ShapeType::Arc;
}

#[test]
fn test_shape_component_circle() {
    let shape = ShapeComponent::circle();
    assert!(matches!(shape.shape_type, ShapeType::Circle));
    assert_eq!(shape.radius, 0.5);
}

#[test]
fn test_shape_component_rectangle() {
    let shape = ShapeComponent::rectangle();
    assert!(matches!(shape.shape_type, ShapeType::Rectangle));
}

#[test]
fn test_shape_component_ellipse() {
    let shape = ShapeComponent::ellipse();
    assert!(matches!(shape.shape_type, ShapeType::Ellipse));
}

#[test]
fn test_shape_component_triangle() {
    let shape = ShapeComponent::triangle();
    assert!(matches!(shape.shape_type, ShapeType::Triangle));
}

#[test]
fn test_shape_component_diamond() {
    let shape = ShapeComponent::diamond();
    assert!(matches!(shape.shape_type, ShapeType::Diamond));
}

#[test]
fn test_shape_component_cylinder() {
    let shape = ShapeComponent::cylinder();
    assert!(matches!(shape.shape_type, ShapeType::Cylinder));
}

#[test]
fn test_shape_component_line() {
    let shape = ShapeComponent::line();
    assert!(matches!(shape.shape_type, ShapeType::Line));
}

#[test]
fn test_shape_component_arc() {
    let shape = ShapeComponent::arc();
    assert!(matches!(shape.shape_type, ShapeType::Arc));
}

#[test]
fn test_shape_component_default() {
    let shape = ShapeComponent::default();
    assert!(matches!(shape.shape_type, ShapeType::Rectangle));
}

#[test]
fn test_shape_component_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<ShapeComponent>();
    assert!(registry.is_registered::<ShapeComponent>());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Color and ColorComponent Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_color_argb() {
    let color = Color::argb(255, 128, 64, 32);
    assert_eq!(color.a, 255);
    assert_eq!(color.r, 128);
    assert_eq!(color.g, 64);
    assert_eq!(color.b, 32);
}

#[test]
fn test_color_rgb() {
    let color = Color::rgb(128, 64, 32);
    assert_eq!(color.a, 255);
    assert_eq!(color.r, 128);
    assert_eq!(color.g, 64);
    assert_eq!(color.b, 32);
}

#[test]
fn test_color_red() {
    let red = Color::rgb(255, 0, 0);
    assert_eq!(red.r, 255);
    assert_eq!(red.g, 0);
    assert_eq!(red.b, 0);
    assert_eq!(red.a, 255);
}

#[test]
fn test_color_green() {
    let green = Color::rgb(0, 255, 0);
    assert_eq!(green.r, 0);
    assert_eq!(green.g, 255);
    assert_eq!(green.b, 0);
    assert_eq!(green.a, 255);
}

#[test]
fn test_color_blue() {
    let blue = Color::rgb(0, 0, 255);
    assert_eq!(blue.r, 0);
    assert_eq!(blue.g, 0);
    assert_eq!(blue.b, 255);
    assert_eq!(blue.a, 255);
}

#[test]
fn test_color_white() {
    let white = Color::rgb(255, 255, 255);
    assert_eq!(white.r, 255);
    assert_eq!(white.g, 255);
    assert_eq!(white.b, 255);
    assert_eq!(white.a, 255);
}

#[test]
fn test_color_black() {
    let black = Color::rgb(0, 0, 0);
    assert_eq!(black.r, 0);
    assert_eq!(black.g, 0);
    assert_eq!(black.b, 0);
    assert_eq!(black.a, 255);
}

#[test]
fn test_color_transparent() {
    let transparent = Color::transparent();
    assert_eq!(transparent.a, 0);
}

#[test]
fn test_color_component_default() {
    let cc = ColorComponent::default();
    assert_eq!(cc.fill, Color::rgb(204, 221, 238));
    assert_eq!(cc.stroke, Color::rgb(0, 0, 0));
    assert_eq!(cc.stroke_width, 1.0);
    assert_eq!(cc.tint, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn test_color_component_new() {
    let fill = Color::rgb(0, 0, 255);
    let cc = ColorComponent::new(fill);
    assert_eq!(cc.fill, Color::rgb(0, 0, 255));
    assert_eq!(cc.stroke, Color::rgb(0, 0, 0));
    assert_eq!(cc.stroke_width, 1.0);
}

#[test]
fn test_color_component_from_rgb() {
    let cc = ColorComponent::from_rgb(255, 0, 0);
    assert_eq!(cc.fill, Color::rgb(255, 0, 0));
}

#[test]
fn test_color_component_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<ColorComponent>();
    assert!(registry.is_registered::<ColorComponent>());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Visibility and VisibilityComponent Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_visibility_variants() {
    let _ = Visibility::Visible;
    let _ = Visibility::Hidden;
    let _ = Visibility::PassThrough;
}

#[test]
fn test_visibility_component_default() {
    let vc = VisibilityComponent::default();
    assert!(matches!(vc.visibility, Visibility::Visible));
}

#[test]
fn test_visibility_component_hidden() {
    let vc = VisibilityComponent::hidden();
    assert!(matches!(vc.visibility, Visibility::Hidden));
}

#[test]
fn test_visibility_component_pass_through() {
    let vc = VisibilityComponent::pass_through();
    assert!(matches!(vc.visibility, Visibility::PassThrough));
}

#[test]
fn test_visibility_component_set_visibility() {
    let mut vc = VisibilityComponent::visible();
    vc.set_visibility(Visibility::Hidden);
    assert!(matches!(vc.visibility, Visibility::Hidden));
}

#[test]
fn test_visibility_component_is_visible() {
    let mut vc = VisibilityComponent::visible();
    assert!(vc.is_visible());
    vc.set_visibility(Visibility::Hidden);
    assert!(!vc.is_visible());
}

#[test]
fn test_visibility_component_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<VisibilityComponent>();
    assert!(registry.is_registered::<VisibilityComponent>());
}

// ═══════════════════════════════════════════════════════════════════════════════
// RenderProperties Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_render_properties_default() {
    let rp = RenderProperties::default();
    assert_eq!(rp.width, 100.0);
    assert_eq!(rp.height, 100.0);
    assert_eq!(rp.layer, 0);
}

#[test]
fn test_render_properties_new() {
    let rp = RenderProperties::new(200.0, 150.0);
    assert_eq!(rp.width, 200.0);
    assert_eq!(rp.height, 150.0);
    assert_eq!(rp.layer, 0);
}

#[test]
fn test_render_properties_square() {
    let rp = RenderProperties::square(50.0);
    assert_eq!(rp.width, 50.0);
    assert_eq!(rp.height, 50.0);
}

#[test]
fn test_render_properties_set_layer() {
    let mut rp = RenderProperties::default();
    rp.layer = 10;
    assert_eq!(rp.layer, 10);
}

#[test]
fn test_render_properties_with_layer() {
    let rp = RenderProperties::default().with_layer(5);
    assert_eq!(rp.layer, 5);
}

#[test]
fn test_render_properties_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<RenderProperties>();
    assert!(registry.is_registered::<RenderProperties>());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_all_components_register() {
    let mut registry = ComponentRegistry::new();
    registry.register::<ShapeComponent>();
    registry.register::<ColorComponent>();
    registry.register::<VisibilityComponent>();
    registry.register::<RenderProperties>();

    assert!(registry.is_registered::<ShapeComponent>());
    assert!(registry.is_registered::<ColorComponent>());
    assert!(registry.is_registered::<VisibilityComponent>());
    assert!(registry.is_registered::<RenderProperties>());
}
