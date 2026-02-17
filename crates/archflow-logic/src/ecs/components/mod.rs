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

use crate::ecs::{Component, ComponentRegistry, ComponentStorage, VecStorage};
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

// ═══════════════════════════════════════════════════════════════════════════════════════
// TextureAtlasComponent - EPIC-ECS-010
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Component that manages sprite regions in a texture atlas
///
/// This component allows entities to render sprites from a spritesheet
/// by specifying which region (by index) to render.
///
/// # Example
///
/// ```
/// use archflow_logic::ecs::components::TextureAtlasComponent;
///
/// let atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);
/// let uv = atlas.get_uv(0);  // Get UV for first sprite
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TextureAtlasComponent {
    /// Index into the texture array
    pub texture_index: u16,
    /// Width of each sprite in pixels
    pub sprite_width: u32,
    /// Height of each sprite in pixels
    pub sprite_height: u32,
    /// Number of columns in the atlas
    pub columns: u32,
    /// Number of rows in the atlas
    pub rows: u32,
    /// Current sprite index (for animation)
    pub current_sprite: u32,
    /// Whether to flip horizontally
    pub flip_x: bool,
    /// Whether to flip vertically
    pub flip_y: bool,
}

impl TextureAtlasComponent {
    /// Creates a new texture atlas component
    #[inline]
    #[must_use]
    pub fn new(
        texture_index: u16,
        sprite_width: u32,
        sprite_height: u32,
        columns: u32,
        rows: u32,
    ) -> Self {
        Self {
            texture_index,
            sprite_width,
            sprite_height,
            columns,
            rows,
            current_sprite: 0,
            flip_x: false,
            flip_y: false,
        }
    }

    /// Creates from an atlas ID with sprite index
    #[inline]
    #[must_use]
    pub fn from_atlas(atlas_id: u16, sprite_index: u32, columns: u32, rows: u32) -> Self {
        Self {
            texture_index: atlas_id,
            sprite_width: 0, // Unknown at this point
            sprite_height: 0,
            columns,
            rows,
            current_sprite: sprite_index,
            flip_x: false,
            flip_y: false,
        }
    }

    /// Get UV coordinates for a sprite by index
    /// Returns [u0, v0, u1, v1]
    #[inline]
    #[must_use]
    pub fn get_uv(&self, index: u32) -> [f32; 4] {
        if self.columns == 0 || self.rows == 0 {
            return [0.0, 0.0, 1.0, 1.0];
        }

        let col = index % self.columns;
        let row = index / self.columns;

        let mut u0 = col as f32 / self.columns as f32;
        let mut v0 = row as f32 / self.rows as f32;
        let mut u1 = (col + 1) as f32 / self.columns as f32;
        let mut v1 = (row + 1) as f32 / self.rows as f32;

        // Apply flip if needed
        if self.flip_x {
            core::mem::swap(&mut u0, &mut u1);
        }
        if self.flip_y {
            core::mem::swap(&mut v0, &mut v1);
        }

        [u0, v0, u1, v1]
    }

    /// Get UV coordinates for current sprite
    #[inline]
    #[must_use]
    pub fn current_uv(&self) -> [f32; 4] {
        self.get_uv(self.current_sprite)
    }

    /// Set sprite index
    #[inline]
    pub fn set_sprite(&mut self, index: u32) {
        self.current_sprite = index.min(self.columns.saturating_mul(self.rows).saturating_sub(1));
    }

    /// Flip horizontally
    #[inline]
    pub fn set_flip_x(&mut self, flip: bool) {
        self.flip_x = flip;
    }

    /// Flip vertically
    #[inline]
    pub fn set_flip_y(&mut self, flip: bool) {
        self.flip_y = flip;
    }
}

impl Component for TextureAtlasComponent {
    type Storage = VecStorage<TextureAtlasComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AnimationClip - Single Animation Sequence
// ═══════════════════════════════════════════════════════════════════════════════════════

/// A single animation sequence (clip)
///
/// Represents a named animation sequence like "idle", "walk", "run".
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClip {
    /// Name of the clip (e.g., "idle", "walk", "run")
    name: alloc::string::String,
    /// Starting frame index
    start_frame: u32,
    /// Ending frame index (inclusive)
    end_frame: u32,
    /// Frames per second
    fps: u32,
    /// Whether the clip loops
    loop_clip: bool,
}

impl AnimationClip {
    /// Creates a new animation clip
    #[inline]
    #[must_use]
    pub fn new(
        name: impl Into<alloc::string::String>,
        start_frame: u32,
        end_frame: u32,
        fps: u32,
        loop_clip: bool,
    ) -> Self {
        Self {
            name: name.into(),
            start_frame,
            end_frame,
            fps,
            loop_clip,
        }
    }

    /// Get the clip name
    #[inline]
    #[must_use]
    pub fn name(&self) -> &alloc::string::String {
        &self.name
    }

    /// Get start frame
    #[inline]
    #[must_use]
    pub fn start_frame(&self) -> u32 {
        self.start_frame
    }

    /// Get end frame
    #[inline]
    #[must_use]
    pub fn end_frame(&self) -> u32 {
        self.end_frame
    }

    /// Get fps
    #[inline]
    #[must_use]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Get if clip loops
    #[inline]
    #[must_use]
    pub fn loop_clip(&self) -> bool {
        self.loop_clip
    }

    /// Get frame count
    #[inline]
    #[must_use]
    pub fn frame_count(&self) -> u32 {
        self.end_frame.saturating_sub(self.start_frame) + 1
    }

    /// Get frame duration in milliseconds
    #[inline]
    #[must_use]
    pub fn frame_duration_ms(&self) -> u32 {
        if self.fps > 0 { 1000 / self.fps } else { 100 }
    }

    /// Check if a frame is within this clip
    #[inline]
    #[must_use]
    pub fn contains_frame(&self, frame: u32) -> bool {
        frame >= self.start_frame && frame <= self.end_frame
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AnimationComponent - EPIC-ECS-011
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Component that manages frame-based sprite animation
///
/// This component handles playback of sprite animations with
/// configurable frame duration and looping.
///
/// # Example
///
/// ```
/// use archflow_logic::ecs::components::AnimationComponent;
///
/// let mut anim = AnimationComponent::new(8, 100); // 8 frames, 100ms each
/// anim.play();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationComponent {
    /// Total number of frames in the animation
    pub frame_count: u32,
    /// Duration of each frame in milliseconds
    pub frame_duration_ms: u32,
    /// Current frame index (0-based)
    pub current_frame: u32,
    /// Whether the animation is currently playing
    pub is_playing: bool,
    /// Whether the animation loops
    pub loop_animation: bool,
    /// Elapsed time since last frame change
    elapsed_ms: u64,
    /// Animation clips (for multi-clip animations)
    clips: alloc::vec::Vec<AnimationClip>,
    /// Current clip index (if using clips)
    current_clip_index: Option<usize>,
}

impl AnimationComponent {
    /// Creates a new animation component
    #[inline]
    #[must_use]
    pub fn new(frame_count: u32, frame_duration_ms: u32) -> Self {
        Self {
            frame_count,
            frame_duration_ms,
            current_frame: 0,
            is_playing: false,
            loop_animation: true,
            elapsed_ms: 0,
            clips: alloc::vec::Vec::new(),
            current_clip_index: None,
        }
    }

    /// Creates with looping disabled (single-shot)
    #[inline]
    #[must_use]
    pub fn new_single_shot(frame_count: u32, frame_duration_ms: u32) -> Self {
        Self {
            frame_count,
            frame_duration_ms,
            current_frame: 0,
            is_playing: false,
            loop_animation: false,
            elapsed_ms: 0,
            clips: alloc::vec::Vec::new(),
            current_clip_index: None,
        }
    }

    /// Creates with multiple animation clips
    #[inline]
    #[must_use]
    pub fn with_clips(clips: alloc::vec::Vec<AnimationClip>) -> Self {
        let first_clip = clips.first();
        let frame_count = first_clip.map(|c| c.frame_count()).unwrap_or(1);
        let frame_duration_ms = first_clip.map(|c| c.frame_duration_ms()).unwrap_or(100);

        Self {
            frame_count,
            frame_duration_ms,
            current_frame: 0,
            is_playing: false,
            loop_animation: first_clip.map(|c| c.loop_clip).unwrap_or(true),
            elapsed_ms: 0,
            clips,
            current_clip_index: Some(0),
        }
    }

    /// Start playing
    #[inline]
    pub fn play(&mut self) {
        self.is_playing = true;
    }

    /// Pause playback
    #[inline]
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Reset to first frame
    #[inline]
    pub fn reset(&mut self) {
        // Reset to start of current clip or frame 0
        if let Some(index) = self.current_clip_index {
            if let Some(clip) = self.clips.get(index) {
                self.current_frame = clip.start_frame;
            } else {
                self.current_frame = 0;
            }
        } else {
            self.current_frame = 0;
        }
        self.elapsed_ms = 0;
    }

    /// Set a specific frame
    #[inline]
    pub fn set_frame(&mut self, frame: u32) {
        // If using clips, clamp to current clip range
        if let Some(index) = self.current_clip_index {
            if let Some(clip) = self.clips.get(index) {
                self.current_frame = frame.clamp(clip.start_frame, clip.end_frame);
                return;
            }
        }
        self.current_frame = frame.min(self.frame_count.saturating_sub(1));
    }

    /// Update animation, returns new frame if changed
    /// Returns Some(new_frame) if frame changed, None otherwise
    #[inline]
    pub fn tick(&mut self, delta_ms: u64) -> Option<u32> {
        if !self.is_playing || self.frame_count == 0 || self.frame_duration_ms == 0 {
            return None;
        }

        self.elapsed_ms += delta_ms;
        let frame_duration = self.frame_duration_ms as u64;

        if self.elapsed_ms >= frame_duration {
            self.elapsed_ms %= frame_duration;
            self.current_frame += 1;

            // Handle clip-based animation
            if let Some(index) = self.current_clip_index {
                if let Some(clip) = self.clips.get(index) {
                    if self.current_frame > clip.end_frame {
                        if self.loop_animation {
                            self.current_frame = clip.start_frame;
                        } else {
                            self.current_frame = clip.end_frame;
                            self.is_playing = false;
                            return Some(self.current_frame);
                        }
                    }
                    return Some(self.current_frame);
                }
            }

            // Default animation handling
            if self.current_frame >= self.frame_count {
                if self.loop_animation {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frame_count - 1;
                    self.is_playing = false;
                    return Some(self.current_frame);
                }
            }
            return Some(self.current_frame);
        }

        None
    }

    /// Get current frame index
    #[inline]
    #[must_use]
    pub fn current(&self) -> u32 {
        self.current_frame
    }

    /// Get number of clips
    #[inline]
    #[must_use]
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Get current clip name
    #[inline]
    #[must_use]
    pub fn current_clip_name(&self) -> Option<&alloc::string::String> {
        self.current_clip_index
            .and_then(|i| self.clips.get(i).map(|c| c.name()))
    }

    /// Play a specific clip by index
    #[inline]
    pub fn play_clip_by_index(&mut self, index: usize) -> bool {
        if let Some(clip) = self.clips.get(index) {
            self.current_clip_index = Some(index);
            self.current_frame = clip.start_frame;
            self.frame_duration_ms = clip.frame_duration_ms();
            self.loop_animation = clip.loop_clip;
            self.is_playing = true;
            self.elapsed_ms = 0;
            true
        } else {
            false
        }
    }

    /// Play a specific clip by name
    #[inline]
    pub fn play_clip(&mut self, name: &str) -> bool {
        if let Some(index) = self.clips.iter().position(|c| c.name() == name) {
            self.play_clip_by_index(index)
        } else {
            false
        }
    }

    /// Get clip by index
    #[inline]
    #[must_use]
    pub fn get_clip(&self, index: usize) -> Option<&AnimationClip> {
        self.clips.get(index)
    }

    /// Get all clips
    #[inline]
    #[must_use]
    pub fn clips(&self) -> &[AnimationClip] {
        &self.clips
    }
}

impl Default for AnimationComponent {
    fn default() -> Self {
        Self::new(1, 100)
    }
}

impl Component for AnimationComponent {
    type Storage = VecStorage<AnimationComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// BlendMode - EPIC-ECS-012
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Blend mode for rendering
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    /// No blending - opaque
    Opaque,
    /// Alpha blending - standard transparency
    AlphaBlend,
    /// Additive blending - glow effect
    Add,
    /// Multiply blend - darkening
    Multiply,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Opaque
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// MaterialComponent - EPIC-ECS-012
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Component that defines material properties for rendering
///
/// This component controls how an entity is rendered including
/// color multiplication, emission, and blend modes.
///
/// # Example
///
/// ```
/// use archflow_logic::ecs::components::{MaterialComponent, BlendMode};
///
/// let material = MaterialComponent::new(
///     [1.0, 0.5, 0.5, 1.0],  // RGBA tint
///     [0.2, 0.1, 0.0],       // RGB emission
///     BlendMode::AlphaBlend,
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialComponent {
    /// Color multiply (applied after texture) [r, g, b, a]
    pub color_multiply: [f32; 4],
    /// Emission color (for glow effects) [r, g, b]
    pub emission: [f32; 3],
    /// Alpha cutoff for alpha testing
    pub alpha_cutoff: f32,
    /// Blend mode
    pub blend_mode: BlendMode,
    /// Custom shader ID (0 = default)
    pub shader_id: u32,
}

impl MaterialComponent {
    /// Creates a new material component
    #[inline]
    #[must_use]
    pub fn new(color_multiply: [f32; 4], emission: [f32; 3], blend_mode: BlendMode) -> Self {
        Self {
            color_multiply,
            emission,
            alpha_cutoff: 0.0,
            blend_mode,
            shader_id: 0,
        }
    }

    /// Creates with default values
    #[inline]
    #[must_use]
    pub fn default_material() -> Self {
        Self {
            color_multiply: [1.0, 1.0, 1.0, 1.0],
            emission: [0.0, 0.0, 0.0],
            alpha_cutoff: 0.0,
            blend_mode: BlendMode::Opaque,
            shader_id: 0,
        }
    }

    /// Create with custom shader
    #[inline]
    #[must_use]
    pub fn with_shader(mut self, shader_id: u32) -> Self {
        self.shader_id = shader_id;
        self
    }

    /// Create with specific blend mode
    #[inline]
    #[must_use]
    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    /// Create with color multiply
    #[inline]
    #[must_use]
    pub fn with_color_multiply(mut self, color: [f32; 4]) -> Self {
        self.color_multiply = color;
        self
    }
}

impl Default for MaterialComponent {
    fn default() -> Self {
        Self::default_material()
    }
}

impl Component for MaterialComponent {
    type Storage = VecStorage<MaterialComponent>;
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// GpuMaterialInstance - EPIC-ECS-012
// ═══════════════════════════════════════════════════════════════════════════════════════

/// GPU-friendly material instance data
/// Layout optimized for WebGPU/WebGL2 (16-byte aligned)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GpuMaterialInstance {
    /// Color multiply [r, g, b, a]
    pub color_multiply: [f32; 4],
    /// Emission color [r, g, b]
    pub emission: [f32; 3],
    /// Padding for alignment
    pub _padding: f32,
    /// Blend mode as u32
    pub blend_mode: u32,
    /// Custom shader ID
    pub shader_id: u32,
    /// Reserved for future use
    pub _reserved: [u32; 2],
}

impl From<&MaterialComponent> for GpuMaterialInstance {
    fn from(material: &MaterialComponent) -> Self {
        let blend_mode = match material.blend_mode {
            BlendMode::Opaque => 0,
            BlendMode::AlphaBlend => 1,
            BlendMode::Add => 2,
            BlendMode::Multiply => 3,
        };

        Self {
            color_multiply: material.color_multiply,
            emission: material.emission,
            _padding: 0.0,
            blend_mode,
            shader_id: material.shader_id,
            _reserved: [0, 0],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PostEffect - EPIC-ECS-013
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Post-processing effect types
#[derive(Clone, Debug, PartialEq)]
pub enum PostEffect {
    /// Bloom glow effect
    Bloom {
        /// Minimum brightness to trigger bloom (0.0-1.0)
        threshold: f32,
        /// Bloom strength (0.0-2.0)
        intensity: f32,
        /// Blur radius (0.0-1.0)
        radius: f32,
    },
    /// Color grading adjustment
    ColorGrading {
        /// Brightness adjustment (-1.0 to 1.0)
        brightness: f32,
        /// Contrast adjustment (0.0 to 2.0)
        contrast: f32,
        /// Saturation adjustment (0.0 to 2.0)
        saturation: f32,
        /// Color temperature (-1.0 to 1.0)
        temperature: f32,
    },
    /// Grayscale conversion
    Grayscale {
        /// Grayscale intensity (0.0 to 1.0)
        intensity: f32,
    },
}

impl PostEffect {
    /// Create a bloom effect with defaults
    #[inline]
    #[must_use]
    pub fn bloom(threshold: f32, intensity: f32, radius: f32) -> Self {
        Self::Bloom {
            threshold: threshold.clamp(0.0, 1.0),
            intensity: intensity.clamp(0.0, 2.0),
            radius: radius.clamp(0.0, 1.0),
        }
    }

    /// Create a color grading effect with defaults
    #[inline]
    #[must_use]
    pub fn color_grading(
        brightness: f32,
        contrast: f32,
        saturation: f32,
        temperature: f32,
    ) -> Self {
        Self::ColorGrading {
            brightness: brightness.clamp(-1.0, 1.0),
            contrast: contrast.clamp(0.0, 2.0),
            saturation: saturation.clamp(0.0, 2.0),
            temperature: temperature.clamp(-1.0, 1.0),
        }
    }

    /// Create a grayscale effect
    #[inline]
    #[must_use]
    pub fn grayscale(intensity: f32) -> Self {
        Self::Grayscale {
            intensity: intensity.clamp(0.0, 1.0),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PostProcessPipeline - EPIC-ECS-013
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Post-processing pipeline for screen-wide effects
///
/// This is typically a resource, not attached to entities.
///
/// # Example
///
/// ```
/// use archflow_logic::ecs::components::{PostProcessPipeline, PostEffect};
///
/// let mut pipeline = PostProcessPipeline::new();
/// pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
/// pipeline.add_effect(PostEffect::color_grading(0.0, 1.0, 1.0, 0.0));
/// ```
#[derive(Clone, Debug, Default)]
pub struct PostProcessPipeline {
    /// Active effects in order
    effects: alloc::vec::Vec<PostEffect>,
    /// Enable/disable entire pipeline
    enabled: bool,
}

impl PostProcessPipeline {
    /// Creates a new post-process pipeline
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            effects: alloc::vec::Vec::new(),
            enabled: true,
        }
    }

    /// Add an effect to the pipeline
    #[inline]
    pub fn add_effect(&mut self, effect: PostEffect) {
        self.effects.push(effect);
    }

    /// Remove an effect by index
    #[inline]
    pub fn remove_effect(&mut self, index: usize) {
        if index < self.effects.len() {
            self.effects.remove(index);
        }
    }

    /// Clear all effects
    #[inline]
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// Get all effects
    #[inline]
    #[must_use]
    pub fn effects(&self) -> &[PostEffect] {
        &self.effects
    }

    /// Enable/disable pipeline
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled
    #[inline]
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get number of effects
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if empty
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

// Component trait implementation for PostProcessPipeline
impl Component for PostProcessPipeline {
    type Storage = VecStorage<Self>;
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// TextureAtlasComponent Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_texture_atlas_component_new() {
    let atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);
    assert_eq!(atlas.texture_index, 0);
    assert_eq!(atlas.sprite_width, 32);
    assert_eq!(atlas.sprite_height, 32);
    assert_eq!(atlas.columns, 4);
    assert_eq!(atlas.rows, 4);
    assert_eq!(atlas.current_sprite, 0);
    assert!(!atlas.flip_x);
    assert!(!atlas.flip_y);
}

#[test]
fn test_texture_atlas_component_from_atlas() {
    let atlas = TextureAtlasComponent::from_atlas(5, 10, 8, 8);
    assert_eq!(atlas.texture_index, 5);
    assert_eq!(atlas.current_sprite, 10);
    assert_eq!(atlas.columns, 8);
    assert_eq!(atlas.rows, 8);
}

#[test]
fn test_texture_atlas_get_uv() {
    let atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);

    // First sprite (index 0) at top-left
    let uv = atlas.get_uv(0);
    assert!((uv[0] - 0.0).abs() < f32::EPSILON);
    assert!((uv[1] - 0.0).abs() < f32::EPSILON);
    assert!((uv[2] - 0.25).abs() < f32::EPSILON);
    assert!((uv[3] - 0.25).abs() < f32::EPSILON);

    // Second sprite (index 1) at second column
    let uv = atlas.get_uv(1);
    assert!((uv[0] - 0.25).abs() < f32::EPSILON);
    assert!((uv[1] - 0.0).abs() < f32::EPSILON);
    assert!((uv[2] - 0.5).abs() < f32::EPSILON);
    assert!((uv[3] - 0.25).abs() < f32::EPSILON);

    // Fifth sprite (index 4) at second row
    let uv = atlas.get_uv(4);
    assert!((uv[0] - 0.0).abs() < f32::EPSILON);
    assert!((uv[1] - 0.25).abs() < f32::EPSILON);
    assert!((uv[2] - 0.25).abs() < f32::EPSILON);
    assert!((uv[3] - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_texture_atlas_current_uv() {
    let mut atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);
    atlas.current_sprite = 5;

    let uv = atlas.current_uv();
    // Sprite 5 should be at column 1, row 1
    assert!((uv[0] - 0.25).abs() < f32::EPSILON);
    assert!((uv[1] - 0.25).abs() < f32::EPSILON);
}

#[test]
fn test_texture_atlas_set_sprite() {
    let mut atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);

    atlas.set_sprite(10);
    assert_eq!(atlas.current_sprite, 10);

    // Setting sprite beyond max should clamp
    atlas.set_sprite(100);
    assert_eq!(atlas.current_sprite, 15); // 4 * 4 - 1 = 15
}

#[test]
fn test_texture_atlas_flip() {
    let mut atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);

    // Test flip X
    let uv_normal = atlas.get_uv(0);
    atlas.set_flip_x(true);
    let uv_flipped_x = atlas.get_uv(0);
    assert!((uv_flipped_x[0] - uv_normal[2]).abs() < f32::EPSILON);
    assert!((uv_flipped_x[2] - uv_normal[0]).abs() < f32::EPSILON);

    // Test flip Y
    atlas.set_flip_x(false);
    atlas.set_flip_y(true);
    let uv_flipped_y = atlas.get_uv(0);
    assert!((uv_flipped_y[1] - uv_normal[3]).abs() < f32::EPSILON);
    assert!((uv_flipped_y[3] - uv_normal[1]).abs() < f32::EPSILON);
}

#[test]
fn test_texture_atlas_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<TextureAtlasComponent>();

    let mut storage = registry.get_storage_mut::<TextureAtlasComponent>().unwrap();
    storage.insert(0, TextureAtlasComponent::new(0, 32, 32, 4, 4));

    let storage = registry.get_storage::<TextureAtlasComponent>().unwrap();
    assert!(storage.contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AnimationComponent Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_component_new() {
    let anim = AnimationComponent::new(8, 100);
    assert_eq!(anim.frame_count, 8);
    assert_eq!(anim.frame_duration_ms, 100);
    assert_eq!(anim.current_frame, 0);
    assert!(!anim.is_playing);
    assert!(anim.loop_animation);
}

#[test]
fn test_animation_component_play_pause() {
    let mut anim = AnimationComponent::new(8, 100);

    assert!(!anim.is_playing);

    anim.play();
    assert!(anim.is_playing);

    anim.pause();
    assert!(!anim.is_playing);
}

#[test]
fn test_animation_component_tick() {
    let mut anim = AnimationComponent::new(4, 100);
    anim.play();

    // First tick with 50ms - no frame change
    let result = anim.tick(50);
    assert!(result.is_none());
    assert_eq!(anim.current_frame, 0);

    // Second tick with 50ms - total 100ms, frame changes
    let result = anim.tick(50);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 1);
    assert_eq!(anim.current_frame, 1);
}

#[test]
fn test_animation_component_loop() {
    let mut anim = AnimationComponent::new(2, 50);
    anim.loop_animation = true;
    anim.play();

    // Tick through first frame
    let _ = anim.tick(50);
    assert_eq!(anim.current_frame, 1);

    // Tick to end - should loop back to 0
    let _ = anim.tick(50);
    assert_eq!(anim.current_frame, 0);
    assert!(anim.is_playing); // Still playing when looping
}

#[test]
fn test_animation_component_single_shot() {
    let mut anim = AnimationComponent::new_single_shot(2, 50);
    anim.play();

    // Tick through first frame
    let _ = anim.tick(50);
    assert_eq!(anim.current_frame, 1);

    // Tick to end - should stop at last frame
    let _ = anim.tick(50);
    assert_eq!(anim.current_frame, 1);
    assert!(!anim.is_playing); // Stops playing
}

#[test]
fn test_animation_component_set_frame() {
    let mut anim = AnimationComponent::new(8, 100);

    anim.set_frame(5);
    assert_eq!(anim.current_frame, 5);

    // Clamping
    anim.set_frame(100);
    assert_eq!(anim.current_frame, 7); // Clamped to frame_count - 1
}

#[test]
fn test_animation_component_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<AnimationComponent>();

    let mut storage = registry.get_storage_mut::<AnimationComponent>().unwrap();
    storage.insert(0, AnimationComponent::new(8, 100));

    let storage = registry.get_storage::<AnimationComponent>().unwrap();
    assert!(storage.contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AnimationClip Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_clip_new() {
    let clip = AnimationClip::new("idle", 0, 7, 12, true);
    assert_eq!(clip.name(), "idle");
    assert_eq!(clip.start_frame(), 0);
    assert_eq!(clip.end_frame(), 7);
    assert_eq!(clip.fps(), 12);
    assert!(clip.loop_clip());
}

#[test]
fn test_animation_clip_frame_count() {
    let clip = AnimationClip::new("test", 5, 9, 10, false);
    assert_eq!(clip.frame_count(), 5); // 9 - 5 + 1 = 5
}

#[test]
fn test_animation_clip_frame_duration_ms() {
    let clip = AnimationClip::new("test", 0, 7, 10, true);
    assert_eq!(clip.frame_duration_ms(), 100); // 1000 / 10 = 100ms
}

#[test]
fn test_animation_clip_contains_frame() {
    let clip = AnimationClip::new("test", 5, 9, 10, false);
    assert!(!clip.contains_frame(4));
    assert!(clip.contains_frame(5));
    assert!(clip.contains_frame(7));
    assert!(clip.contains_frame(9));
    assert!(!clip.contains_frame(10));
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// BlendMode Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_blend_mode_variants() {
    let _ = BlendMode::Opaque;
    let _ = BlendMode::AlphaBlend;
    let _ = BlendMode::Add;
    let _ = BlendMode::Multiply;
}

#[test]
fn test_blend_mode_default() {
    let mode = BlendMode::default();
    assert!(matches!(mode, BlendMode::Opaque));
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// MaterialComponent Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_material_component_new() {
    let material =
        MaterialComponent::new([1.0, 0.5, 0.5, 1.0], [0.2, 0.1, 0.0], BlendMode::AlphaBlend);
    assert_eq!(material.color_multiply, [1.0, 0.5, 0.5, 1.0]);
    assert_eq!(material.emission, [0.2, 0.1, 0.0]);
    assert!(matches!(material.blend_mode, BlendMode::AlphaBlend));
    assert_eq!(material.alpha_cutoff, 0.0);
    assert_eq!(material.shader_id, 0);
}

#[test]
fn test_material_component_default() {
    let material = MaterialComponent::default();
    assert_eq!(material.color_multiply, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(material.emission, [0.0, 0.0, 0.0]);
    assert!(matches!(material.blend_mode, BlendMode::Opaque));
}

#[test]
fn test_material_component_with_shader() {
    let material = MaterialComponent::default().with_shader(42);
    assert_eq!(material.shader_id, 42);
}

#[test]
fn test_material_component_with_blend_mode() {
    let material = MaterialComponent::default().with_blend_mode(BlendMode::Add);
    assert!(matches!(material.blend_mode, BlendMode::Add));
}

#[test]
fn test_material_component_in_registry() {
    let mut registry = ComponentRegistry::new();
    registry.register::<MaterialComponent>();

    let mut storage = registry.get_storage_mut::<MaterialComponent>().unwrap();
    storage.insert(0, MaterialComponent::default());

    let storage = registry.get_storage::<MaterialComponent>().unwrap();
    assert!(storage.contains(0));
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// GpuMaterialInstance Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gpu_material_instance_from_material() {
    let material =
        MaterialComponent::new([0.5, 0.5, 0.5, 0.8], [0.1, 0.2, 0.3], BlendMode::AlphaBlend)
            .with_shader(5);

    let gpu = GpuMaterialInstance::from(&material);
    assert_eq!(gpu.color_multiply, [0.5, 0.5, 0.5, 0.8]);
    assert_eq!(gpu.emission, [0.1, 0.2, 0.3]);
    assert_eq!(gpu.blend_mode, 1); // AlphaBlend = 1
    assert_eq!(gpu.shader_id, 5);
}

#[test]
fn test_gpu_material_instance_blend_modes() {
    let opaque = MaterialComponent::default().with_blend_mode(BlendMode::Opaque);
    assert_eq!(GpuMaterialInstance::from(&opaque).blend_mode, 0);

    let alpha = MaterialComponent::default().with_blend_mode(BlendMode::AlphaBlend);
    assert_eq!(GpuMaterialInstance::from(&alpha).blend_mode, 1);

    let add = MaterialComponent::default().with_blend_mode(BlendMode::Add);
    assert_eq!(GpuMaterialInstance::from(&add).blend_mode, 2);

    let multiply = MaterialComponent::default().with_blend_mode(BlendMode::Multiply);
    assert_eq!(GpuMaterialInstance::from(&multiply).blend_mode, 3);
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PostEffect Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_post_effect_bloom() {
    let bloom = PostEffect::bloom(0.8, 0.5, 0.3);
    match bloom {
        PostEffect::Bloom {
            threshold,
            intensity,
            radius,
        } => {
            assert!((threshold - 0.8).abs() < f32::EPSILON);
            assert!((intensity - 0.5).abs() < f32::EPSILON);
            assert!((radius - 0.3).abs() < f32::EPSILON);
        }
        _ => panic!("Expected Bloom variant"),
    }
}

#[test]
fn test_post_effect_color_grading() {
    let grading = PostEffect::color_grading(0.1, 1.2, 0.8, -0.2);
    match grading {
        PostEffect::ColorGrading {
            brightness,
            contrast,
            saturation,
            temperature,
        } => {
            assert!((brightness - 0.1).abs() < f32::EPSILON);
            assert!((contrast - 1.2).abs() < f32::EPSILON);
            assert!((saturation - 0.8).abs() < f32::EPSILON);
            assert!((temperature - (-0.2)).abs() < f32::EPSILON);
        }
        _ => panic!("Expected ColorGrading variant"),
    }
}

#[test]
fn test_post_effect_grayscale() {
    let gray = PostEffect::grayscale(0.75);
    match gray {
        PostEffect::Grayscale { intensity } => {
            assert!((intensity - 0.75).abs() < f32::EPSILON);
        }
        _ => panic!("Expected Grayscale variant"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PostProcessPipeline Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_post_process_pipeline_new() {
    let pipeline = PostProcessPipeline::new();
    assert!(pipeline.is_empty());
    assert_eq!(pipeline.len(), 0);
    assert!(pipeline.is_enabled());
}

#[test]
fn test_post_process_pipeline_add_effect() {
    let mut pipeline = PostProcessPipeline::new();

    pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
    assert_eq!(pipeline.len(), 1);

    pipeline.add_effect(PostEffect::grayscale(0.5));
    assert_eq!(pipeline.len(), 2);
}

#[test]
fn test_post_process_pipeline_remove_effect() {
    let mut pipeline = PostProcessPipeline::new();
    pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
    pipeline.add_effect(PostEffect::grayscale(0.5));
    pipeline.add_effect(PostEffect::color_grading(0.0, 1.0, 1.0, 0.0));

    assert_eq!(pipeline.len(), 3);

    // Remove middle effect
    pipeline.remove_effect(1);
    assert_eq!(pipeline.len(), 2);

    // Verify correct effect was removed
    let effects = pipeline.effects();
    assert!(matches!(effects[0], PostEffect::Bloom { .. }));
    assert!(matches!(effects[1], PostEffect::ColorGrading { .. }));
}

#[test]
fn test_post_process_pipeline_enabled() {
    let mut pipeline = PostProcessPipeline::new();
    assert!(pipeline.is_enabled());

    pipeline.set_enabled(false);
    assert!(!pipeline.is_enabled());

    pipeline.set_enabled(true);
    assert!(pipeline.is_enabled());
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
