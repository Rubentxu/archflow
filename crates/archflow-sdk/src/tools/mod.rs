//! Tools System for ArchFlow SDK
//!
//! This module provides a comprehensive tools system for user interaction
//! including selection, drawing, and erasing tools.

use crate::a11y::{KeyCode, KeyEvent, Modifiers};
use crate::canvas::{Canvas, Shape, ShapeGeometry, ShapeStyle, ShapeType};
use crate::plugin::{PluginContext, PluginHost, PluginResult, Tool, ToolCategory, ToolShortcut};
use crate::selection::{SelectionDelta, SelectionManager, SelectionMode};
use archflow_core::{Color, EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Result type for tool operations
pub type ToolResult<T> = Result<T, ToolError>;

/// Errors that can occur during tool operations
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("No active document")]
    NoDocument,
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Selection error: {0}")]
    SelectionError(String),
    #[error("Shape error: {0}")]
    ShapeError(String),
}

/// Mouse event data
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
    /// Position in canvas coordinates
    pub position: Vec2,
    /// Position in screen coordinates
    pub screen_position: Vec2,
    /// Left mouse button
    pub left_button: bool,
    /// Right mouse button
    pub right_button: bool,
    /// Middle mouse button
    pub middle_button: bool,
    /// Control key
    pub ctrl: bool,
    /// Shift key
    pub shift: bool,
    /// Alt key
    pub alt: bool,
}

impl MouseEvent {
    /// Creates a new mouse event
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            screen_position: position,
            left_button: false,
            right_button: false,
            middle_button: false,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// Creates a mouse event with button state
    pub fn with_button(mut self, button: MouseButton, pressed: bool) -> Self {
        match button {
            MouseButton::Left => self.left_button = pressed,
            MouseButton::Right => self.right_button = pressed,
            MouseButton::Middle => self.middle_button = pressed,
        }
        self
    }

    /// Creates a mouse event with modifiers
    pub fn with_modifiers(mut self, ctrl: bool, shift: bool, alt: bool) -> Self {
        self.ctrl = ctrl;
        self.shift = shift;
        self.alt = alt;
        self
    }

    /// Checks if any mouse button is pressed
    pub fn any_button(&self) -> bool {
        self.left_button || self.right_button || self.middle_button
    }
}

/// Mouse button
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Cursor type for tools
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorType {
    Default,
    Crosshair,
    Move,
    Pointer,
    Text,
    Wait,
    Grab,
    Grabbing,
    NotAllowed,
    ResizeNWSE,
    ResizeNESW,
    ResizeNS,
    ResizeEW,
}

/// Selection tool state
#[derive(Clone, Debug)]
pub enum SelectToolState {
    /// Idle state
    Idle,
    /// Dragging selected shapes
    Dragging {
        /// Start position
        start: Vec2,
        /// Initial shape positions
        initial_positions: Vec<(EntityId, Vec2)>,
    },
    /// Box selection in progress
    BoxSelecting {
        /// Start position in screen coordinates
        start: Vec2,
    },
    /// Resizing a shape
    Resizing {
        /// Shape being resized
        shape_id: EntityId,
        /// Resize handle
        handle: ResizeHandle,
        /// Start position
        start: Vec2,
        /// Initial geometry
        initial_geometry: ShapeGeometry,
    },
}

/// Resize handle on a shape
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Rotation,
}

impl ResizeHandle {
    /// Returns all resize handles
    pub fn all() -> &'static [ResizeHandle] {
        &[
            ResizeHandle::TopLeft,
            ResizeHandle::Top,
            ResizeHandle::TopRight,
            ResizeHandle::Right,
            ResizeHandle::BottomRight,
            ResizeHandle::Bottom,
            ResizeHandle::BottomLeft,
            ResizeHandle::Left,
            ResizeHandle::Rotation,
        ]
    }

    /// Returns the cursor type for this handle
    pub fn cursor(&self) -> CursorType {
        match self {
            ResizeHandle::TopLeft | ResizeHandle::BottomRight => CursorType::ResizeNWSE,
            ResizeHandle::TopRight | ResizeHandle::BottomLeft => CursorType::ResizeNESW,
            ResizeHandle::Top | ResizeHandle::Bottom => CursorType::ResizeNS,
            ResizeHandle::Left | ResizeHandle::Right => CursorType::ResizeEW,
            ResizeHandle::Rotation => CursorType::Grab,
        }
    }
}

/// Selection tool implementation
pub struct SelectTool {
    /// Current tool state
    state: SelectToolState,
    /// Minimum drag distance to consider it a drag operation
    drag_threshold: f32,
    /// Size of resize handles in screen pixels
    handle_size: f32,
}

impl SelectTool {
    /// Creates a new selection tool
    pub fn new() -> Self {
        Self {
            state: SelectToolState::Idle,
            drag_threshold: 5.0,
            handle_size: 8.0,
        }
    }

    /// Gets resize handles for a shape's bounding box
    pub fn get_handles(&self, shape: &Shape) -> Vec<(ResizeHandle, Vec2)> {
        let bounds = shape.bounds();
        let min = bounds.min;
        let max = bounds.max;

        vec![
            (ResizeHandle::TopLeft, min),
            (ResizeHandle::Top, Vec2::new((min.x + max.x) / 2.0, min.y)),
            (ResizeHandle::TopRight, Vec2::new(max.x, min.y)),
            (ResizeHandle::Right, Vec2::new(max.x, (min.y + max.y) / 2.0)),
            (ResizeHandle::BottomRight, max),
            (
                ResizeHandle::Bottom,
                Vec2::new((min.x + max.x) / 2.0, max.y),
            ),
            (ResizeHandle::BottomLeft, Vec2::new(min.x, max.y)),
            (ResizeHandle::Left, Vec2::new(min.x, (min.y + max.y) / 2.0)),
            (
                ResizeHandle::Rotation,
                Vec2::new((min.x + max.x) / 2.0, min.y - 20.0),
            ),
        ]
    }

    /// Tests if a point hits a resize handle
    pub fn hit_test_handle(
        &self,
        shape: &Shape,
        point: Vec2,
        screen_to_canvas: impl Fn(Vec2) -> Vec2,
    ) -> Option<ResizeHandle> {
        let canvas_point = screen_to_canvas(point);
        let handles = self.get_handles(shape);

        for (handle, pos) in handles {
            let handle_rect = Rect::from_center_size(pos, Vec2::splat(self.handle_size));
            if handle_rect.contains(canvas_point) {
                return Some(handle);
            }
        }

        None
    }

    /// Updates the selection
    pub fn update_selection(&mut self, _canvas: &Canvas) -> SelectionDelta {
        // This would be called to update selection based on current state
        SelectionDelta::new()
    }
}

impl Default for SelectTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for SelectTool {
    fn id(&self) -> &str {
        "select"
    }

    fn name(&self) -> &str {
        "Select"
    }

    fn icon(&self) -> &str {
        "⬚"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Select
    }

    fn on_select(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        self.state = SelectToolState::Idle;
        Ok(())
    }

    fn on_deselect(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        self.state = SelectToolState::Idle;
        Ok(())
    }

    fn on_mouse_down(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        // Determine if we're clicking on a resize handle or starting a new selection
        // This is a simplified implementation - full version would check handles first
        self.state = SelectToolState::BoxSelecting { start: position };

        Ok(())
    }

    fn on_mouse_move(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        match &self.state {
            SelectToolState::BoxSelecting { start } => {
                let dx = position.x - start.x;
                let dy = position.y - start.y;

                // Only start box selection if we've moved beyond threshold
                if dx.hypot(dy) > self.drag_threshold {
                    // Box selection would be handled by SelectionManager
                    // which is managed externally by the canvas
                }
            }
            SelectToolState::Dragging { .. } => {
                // Handle dragging selected shapes
            }
            SelectToolState::Resizing { .. } => {
                // Handle resizing
            }
            SelectToolState::Idle => {}
        }

        Ok(())
    }

    fn on_mouse_up(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        match &self.state {
            SelectToolState::BoxSelecting { start } => {
                let dx = position.x - start.x;
                let dy = position.y - start.y;

                if dx.hypot(dy) <= self.drag_threshold {
                    // It was a click, not a drag - select single shape
                    // This would use hit testing in a full implementation
                } else {
                    // Finalize box selection
                    // This would be handled by SelectionManager
                    // which is managed externally by the canvas
                }

                self.state = SelectToolState::Idle;
            }
            SelectToolState::Dragging { .. } | SelectToolState::Resizing { .. } => {
                self.state = SelectToolState::Idle;
            }
            SelectToolState::Idle => {}
        }

        Ok(())
    }

    fn render_overlay(&self, _context: &PluginContext) -> PluginResult<()> {
        // Render selection box, resize handles, etc.
        Ok(())
    }

    fn on_key_down(
        &mut self,
        event: &KeyEvent,
        host: &mut dyn PluginHost,
    ) -> PluginResult<Option<SelectionDelta>> {
        match event.key_code {
            KeyCode::Escape => {
                // Deselect all on Escape
                self.state = SelectToolState::Idle;
                // The host would handle the actual deselection
                Ok(None)
            }
            KeyCode::Delete | KeyCode::Backspace => {
                // Delete selected shapes
                // This would be handled by the host
                Ok(None)
            }
            KeyCode::A if event.modifiers.ctrl => {
                // Select all (Ctrl+A)
                // This would be handled by the host
                Ok(None)
            }
            KeyCode::C if event.modifiers.ctrl => {
                // Copy selection (Ctrl+C)
                // This would be handled by the host
                Ok(None)
            }
            KeyCode::V if event.modifiers.ctrl => {
                // Paste (Ctrl+V)
                // This would be handled by the host
                Ok(None)
            }
            KeyCode::D if event.modifiers.ctrl => {
                // Duplicate (Ctrl+D)
                // This would be handled by the host
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn keyboard_shortcuts(&self) -> Vec<ToolShortcut> {
        vec![
            ToolShortcut::with_modifiers(
                "Ctrl+A",
                "Select all shapes",
                "select_all",
                vec![KeyCode::A],
                true,
                false,
                false,
                false,
            ),
            ToolShortcut::with_modifiers(
                "Ctrl+C",
                "Copy selection",
                "copy",
                vec![KeyCode::C],
                true,
                false,
                false,
                false,
            ),
            ToolShortcut::with_modifiers(
                "Ctrl+V",
                "Paste",
                "paste",
                vec![KeyCode::V],
                true,
                false,
                false,
                false,
            ),
            ToolShortcut::with_modifiers(
                "Ctrl+D",
                "Duplicate selection",
                "duplicate",
                vec![KeyCode::D],
                true,
                false,
                false,
                false,
            ),
            ToolShortcut::new(
                "Delete / Backspace",
                "Delete selected shapes",
                "delete",
                vec![KeyCode::Delete, KeyCode::Backspace],
            ),
            ToolShortcut::new("Escape", "Deselect all", "deselect", vec![KeyCode::Escape]),
        ]
    }
}

/// Drawing tool state
#[derive(Clone, Debug)]
pub enum DrawToolState {
    /// Idle state
    Idle,
    /// Drawing in progress
    Drawing {
        /// Start position
        start: Vec2,
        /// Current position
        current: Vec2,
    },
    /// Creating a path
    PathDrawing {
        /// Points in the path
        points: Vec<Vec2>,
    },
}

/// Shape type for drawing tool
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawShapeType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Path,
    Freehand,
}

/// Drawing tool implementation
pub struct DrawTool {
    /// Shape type being drawn
    shape_type: DrawShapeType,
    /// Current tool state
    state: DrawToolState,
    /// Default fill color
    fill_color: Color,
    /// Default stroke color
    stroke_color: Color,
    /// Default stroke width
    stroke_width: f32,
}

impl DrawTool {
    /// Creates a new drawing tool
    pub fn new(shape_type: DrawShapeType) -> Self {
        Self {
            shape_type,
            state: DrawToolState::Idle,
            fill_color: Color::rgb(0x33 as f32 / 255.0, 0x88 as f32 / 255.0, 1.0),
            stroke_color: Color::rgb(0.0, 0.0, 0.0),
            stroke_width: 2.0,
        }
    }

    /// Sets the default colors
    pub fn with_colors(mut self, fill: Color, stroke: Color) -> Self {
        self.fill_color = fill;
        self.stroke_color = stroke;
        self
    }

    /// Sets the stroke width
    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Creates the shape being drawn
    pub fn create_shape(&self, start: Vec2, end: Vec2) -> Shape {
        let (x, y) = (start.x.min(end.x), start.y.min(end.y));
        let (width, height) = ((end.x - start.x).abs(), (end.y - start.y).abs());

        match self.shape_type {
            DrawShapeType::Rectangle => Shape::new_rectangle(x, y, width, height),
            DrawShapeType::Ellipse => {
                Shape::new_ellipse(x + width / 2.0, y + height / 2.0, width / 2.0, height / 2.0)
            }
            DrawShapeType::Line | DrawShapeType::Arrow => {
                // For lines, we create a rectangle as placeholder
                // A proper implementation would have a line shape type
                Shape::new_rectangle(x, y, width.max(1.0), height.max(1.0))
            }
            DrawShapeType::Path | DrawShapeType::Freehand => {
                // Path shapes would need a different construction
                Shape::new_rectangle(x, y, width.max(1.0), height.max(1.0))
            }
        }
    }

    /// Gets the preview shape (what's currently being drawn)
    pub fn preview_shape(&self) -> Option<Shape> {
        match &self.state {
            DrawToolState::Drawing { start, current } => Some(self.create_shape(*start, *current)),
            DrawToolState::PathDrawing { points } if !points.is_empty() => {
                // Create path preview
                None
            }
            _ => None,
        }
    }
}

impl Tool for DrawTool {
    fn id(&self) -> &str {
        match self.shape_type {
            DrawShapeType::Rectangle => "draw-rectangle",
            DrawShapeType::Ellipse => "draw-ellipse",
            DrawShapeType::Line => "draw-line",
            DrawShapeType::Arrow => "draw-arrow",
            DrawShapeType::Path => "draw-path",
            DrawShapeType::Freehand => "draw-freehand",
        }
    }

    fn name(&self) -> &str {
        match self.shape_type {
            DrawShapeType::Rectangle => "Rectangle",
            DrawShapeType::Ellipse => "Ellipse",
            DrawShapeType::Line => "Line",
            DrawShapeType::Arrow => "Arrow",
            DrawShapeType::Path => "Path",
            DrawShapeType::Freehand => "Freehand",
        }
    }

    fn icon(&self) -> &str {
        match self.shape_type {
            DrawShapeType::Rectangle => "▭",
            DrawShapeType::Ellipse => "◯",
            DrawShapeType::Line => "─",
            DrawShapeType::Arrow => "→",
            DrawShapeType::Path => "⌇",
            DrawShapeType::Freehand => "✎",
        }
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Shape
    }

    fn on_select(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        self.state = DrawToolState::Idle;
        Ok(())
    }

    fn on_deselect(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        self.state = DrawToolState::Idle;
        Ok(())
    }

    fn on_mouse_down(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        match self.shape_type {
            DrawShapeType::Path | DrawShapeType::Freehand => {
                self.state = DrawToolState::PathDrawing {
                    points: vec![position],
                };
            }
            _ => {
                self.state = DrawToolState::Drawing {
                    start: position,
                    current: position,
                };
            }
        }

        Ok(())
    }

    fn on_mouse_move(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        match &mut self.state {
            DrawToolState::Drawing { current, .. } => {
                *current = position;
            }
            DrawToolState::PathDrawing { points } => {
                points.push(position);
            }
            DrawToolState::Idle => {}
        }

        Ok(())
    }

    fn on_mouse_up(&mut self, position: Vec2, host: &mut dyn PluginHost) -> PluginResult<()> {
        let shape = match &self.state {
            DrawToolState::Drawing { start, .. } => Some(self.create_shape(*start, position)),
            DrawToolState::PathDrawing { points } if points.len() >= 2 => {
                // Create path from points
                None
            }
            _ => None,
        };

        if let Some(mut s) = shape {
            s.style =
                ShapeStyle::solid(self.fill_color, Some(self.stroke_color), self.stroke_width);
            // Emit shape creation event
            host.emit_event(crate::events::CanvasEvent::ShapeCreated {
                shape_id: s.id,
                shape_data: crate::events::ShapeData::from(s),
            })?;
        }

        self.state = DrawToolState::Idle;
        Ok(())
    }

    fn render_overlay(&self, _context: &PluginContext) -> PluginResult<()> {
        // Render preview shape
        Ok(())
    }

    fn on_key_down(
        &mut self,
        event: &KeyEvent,
        _host: &mut dyn PluginHost,
    ) -> PluginResult<Option<SelectionDelta>> {
        match event.key_code {
            KeyCode::Escape => {
                // Cancel drawing
                self.state = DrawToolState::Idle;
                Ok(None)
            }
            // Switch between shape types
            KeyCode::R => {
                self.shape_type = DrawShapeType::Rectangle;
                Ok(None)
            }
            KeyCode::O => {
                self.shape_type = DrawShapeType::Ellipse;
                Ok(None)
            }
            KeyCode::L => {
                self.shape_type = DrawShapeType::Line;
                Ok(None)
            }
            KeyCode::A => {
                self.shape_type = DrawShapeType::Arrow;
                Ok(None)
            }
            KeyCode::P => {
                self.shape_type = DrawShapeType::Path;
                Ok(None)
            }
            KeyCode::F => {
                self.shape_type = DrawShapeType::Freehand;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn keyboard_shortcuts(&self) -> Vec<ToolShortcut> {
        vec![
            ToolShortcut::new("Escape", "Cancel drawing", "cancel", vec![KeyCode::Escape]),
            ToolShortcut::new("R", "Rectangle tool", "rectangle", vec![KeyCode::R]),
            ToolShortcut::new("O", "Ellipse tool", "ellipse", vec![KeyCode::O]),
            ToolShortcut::new("L", "Line tool", "line", vec![KeyCode::L]),
            ToolShortcut::new("A", "Arrow tool", "arrow", vec![KeyCode::A]),
            ToolShortcut::new("P", "Path tool", "path", vec![KeyCode::P]),
            ToolShortcut::new("F", "Freehand tool", "freehand", vec![KeyCode::F]),
        ]
    }
}

/// Erase tool mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EraseMode {
    /// Erase single shapes by clicking
    Single,
    /// Erase shapes within a lasso selection
    Lasso,
}

/// Eraser tool implementation
pub struct EraseTool {
    /// Erase mode
    mode: EraseMode,
    /// Lasso path in progress
    lasso_path: Vec<Vec2>,
    /// Whether erasing is active
    is_active: bool,
}

impl EraseTool {
    /// Creates a new eraser tool
    pub fn new(mode: EraseMode) -> Self {
        Self {
            mode,
            lasso_path: Vec::new(),
            is_active: false,
        }
    }

    /// Tests if a point is within the lasso polygon
    pub fn point_in_lasso(&self, point: Vec2) -> bool {
        if self.lasso_path.len() < 3 {
            return false;
        }

        // Simple point-in-polygon test using ray casting
        let mut inside = false;
        let n = self.lasso_path.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let pi = self.lasso_path[i];
            let pj = self.lasso_path[j];

            if ((pi.y > point.y) != (pj.y > point.y))
                && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
        }

        inside
    }

    /// Clears the lasso path
    pub fn clear_lasso(&mut self) {
        self.lasso_path.clear();
        self.is_active = false;
    }
}

impl Tool for EraseTool {
    fn id(&self) -> &str {
        match self.mode {
            EraseMode::Single => "erase-single",
            EraseMode::Lasso => "erase-lasso",
        }
    }

    fn name(&self) -> &str {
        match self.mode {
            EraseMode::Single => "Eraser",
            EraseMode::Lasso => "Lasso Erase",
        }
    }

    fn icon(&self) -> &str {
        "⌫"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Draw
    }

    fn on_select(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        self.clear_lasso();
        Ok(())
    }

    fn on_deselect(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        self.clear_lasso();
        Ok(())
    }

    fn on_mouse_down(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        match self.mode {
            EraseMode::Single => {
                // Erase shape at point - would use hit testing
                self.is_active = true;
            }
            EraseMode::Lasso => {
                self.lasso_path.push(position);
                self.is_active = true;
            }
        }

        Ok(())
    }

    fn on_mouse_move(&mut self, position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        if self.is_active && self.mode == EraseMode::Lasso {
            self.lasso_path.push(position);
        }

        Ok(())
    }

    fn on_mouse_up(&mut self, _position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        if self.mode == EraseMode::Lasso && self.lasso_path.len() >= 3 {
            // Erase shapes within lasso
            // This would query shapes and test them against the lasso
        }

        self.is_active = false;
        Ok(())
    }

    fn render_overlay(&self, _context: &PluginContext) -> PluginResult<()> {
        // Render lasso path
        Ok(())
    }

    fn on_key_down(
        &mut self,
        event: &KeyEvent,
        _host: &mut dyn PluginHost,
    ) -> PluginResult<Option<SelectionDelta>> {
        match event.key_code {
            KeyCode::Escape => {
                // Cancel erasing
                self.clear_lasso();
                self.is_active = false;
                Ok(None)
            }
            KeyCode::S => {
                // Switch to single erase mode
                self.mode = EraseMode::Single;
                self.clear_lasso();
                Ok(None)
            }
            KeyCode::L => {
                // Switch to lasso erase mode
                self.mode = EraseMode::Lasso;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn keyboard_shortcuts(&self) -> Vec<ToolShortcut> {
        vec![
            ToolShortcut::new("Escape", "Cancel erasing", "cancel", vec![KeyCode::Escape]),
            ToolShortcut::new("S", "Single erase mode", "single_mode", vec![KeyCode::S]),
            ToolShortcut::new("L", "Lasso erase mode", "lasso_mode", vec![KeyCode::L]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_event_creation() {
        let event = MouseEvent::new(Vec2::new(10.0, 20.0));
        assert_eq!(event.position, Vec2::new(10.0, 20.0));
        assert!(!event.any_button());
    }

    #[test]
    fn test_mouse_event_with_button() {
        let event = MouseEvent::new(Vec2::new(10.0, 20.0)).with_button(MouseButton::Left, true);
        assert!(event.left_button);
        assert!(event.any_button());
    }

    #[test]
    fn test_mouse_event_with_modifiers() {
        let event = MouseEvent::new(Vec2::new(10.0, 20.0)).with_modifiers(true, false, true);
        assert!(event.ctrl);
        assert!(!event.shift);
        assert!(event.alt);
    }

    #[test]
    fn test_select_tool_creation() {
        let tool = SelectTool::new();
        assert_eq!(tool.id(), "select");
        assert_eq!(tool.name(), "Select");
        assert_eq!(tool.icon(), "⬚");
        assert_eq!(tool.category(), ToolCategory::Select);
    }

    #[test]
    fn test_draw_tool_rectangle() {
        let tool = DrawTool::new(DrawShapeType::Rectangle);
        assert_eq!(tool.id(), "draw-rectangle");
        assert_eq!(tool.name(), "Rectangle");
        assert_eq!(tool.icon(), "▭");
    }

    #[test]
    fn test_draw_tool_ellipse() {
        let tool = DrawTool::new(DrawShapeType::Ellipse);
        assert_eq!(tool.id(), "draw-ellipse");
        assert_eq!(tool.name(), "Ellipse");
        assert_eq!(tool.icon(), "◯");
    }

    #[test]
    fn test_draw_tool_colors() {
        let tool = DrawTool::new(DrawShapeType::Rectangle)
            .with_colors(Color::rgb(1.0, 0.0, 0.0), Color::rgb(0.0, 0.0, 0.0))
            .with_stroke_width(3.0);

        assert_eq!(tool.stroke_width, 3.0);
    }

    #[test]
    fn test_draw_tool_create_shape() {
        let tool = DrawTool::new(DrawShapeType::Rectangle);
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 50.0);

        let shape = tool.create_shape(start, end);
        assert_eq!(shape.x, 0.0);
        assert_eq!(shape.y, 0.0);
        assert_eq!(shape.width, 100.0);
        assert_eq!(shape.height, 50.0);
    }

    #[test]
    fn test_erase_tool_single() {
        let tool = EraseTool::new(EraseMode::Single);
        assert_eq!(tool.id(), "erase-single");
        assert_eq!(tool.name(), "Eraser");
        assert_eq!(tool.icon(), "⌫");
    }

    #[test]
    fn test_erase_tool_lasso() {
        let tool = EraseTool::new(EraseMode::Lasso);
        assert_eq!(tool.id(), "erase-lasso");
        assert_eq!(tool.name(), "Lasso Erase");
    }

    #[test]
    fn test_point_in_lasso() {
        let mut tool = EraseTool::new(EraseMode::Lasso);

        // Create a simple square lasso
        tool.lasso_path = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ];

        // Point inside
        assert!(tool.point_in_lasso(Vec2::new(50.0, 50.0)));

        // Point outside
        assert!(!tool.point_in_lasso(Vec2::new(150.0, 150.0)));
    }

    #[test]
    fn test_resize_handle_cursors() {
        assert_eq!(ResizeHandle::TopLeft.cursor(), CursorType::ResizeNWSE);
        assert_eq!(ResizeHandle::TopRight.cursor(), CursorType::ResizeNESW);
        assert_eq!(ResizeHandle::Top.cursor(), CursorType::ResizeNS);
        assert_eq!(ResizeHandle::Left.cursor(), CursorType::ResizeEW);
        assert_eq!(ResizeHandle::Rotation.cursor(), CursorType::Grab);
    }

    #[test]
    fn test_erase_mode() {
        assert_eq!(EraseMode::Single, EraseMode::Single);
        assert_eq!(EraseMode::Lasso, EraseMode::Lasso);
        assert_ne!(EraseMode::Single, EraseMode::Lasso);
    }

    #[test]
    fn test_draw_shape_type() {
        assert_eq!(DrawShapeType::Rectangle, DrawShapeType::Rectangle);
        assert_eq!(DrawShapeType::Ellipse, DrawShapeType::Ellipse);
        assert_ne!(DrawShapeType::Rectangle, DrawShapeType::Ellipse);
    }

    #[test]
    fn test_cursor_type() {
        assert_eq!(CursorType::Default, CursorType::Default);
        assert_eq!(CursorType::Crosshair, CursorType::Crosshair);
        assert_ne!(CursorType::Default, CursorType::Move);
    }

    #[test]
    fn test_mouse_button() {
        assert_eq!(MouseButton::Left, MouseButton::Left);
        assert_eq!(MouseButton::Right, MouseButton::Right);
        assert_ne!(MouseButton::Left, MouseButton::Right);
    }

    #[test]
    fn test_select_tool_default() {
        let tool = SelectTool::default();
        assert_eq!(tool.id(), "select");
        assert_eq!(tool.drag_threshold, 5.0);
        assert_eq!(tool.handle_size, 8.0);
    }

    #[test]
    fn test_eraser_clear_lasso() {
        let mut tool = EraseTool::new(EraseMode::Lasso);
        tool.lasso_path = vec![Vec2::new(10.0, 10.0)];
        tool.is_active = true;

        tool.clear_lasso();

        assert!(tool.lasso_path.is_empty());
        assert!(!tool.is_active);
    }

    #[test]
    fn test_draw_tool_preview_none() {
        let tool = DrawTool::new(DrawShapeType::Rectangle);
        assert!(tool.preview_shape().is_none());
    }

    #[test]
    fn test_resize_handle_all() {
        let handles = ResizeHandle::all();
        assert_eq!(handles.len(), 9);
        assert!(handles.contains(&ResizeHandle::TopLeft));
        assert!(handles.contains(&ResizeHandle::Rotation));
    }
}
