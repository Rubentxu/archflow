//! Demo state management
//!
//! This module handles the state machine for the interactive demo,
//! including tool selection, shape creation, interaction states,
//! multi-selection, pan/zoom navigation, and undo/redo functionality.
//!
//! # Architecture
//!
//! - [`DemoState`]: Main state container for the entire application
//! - [`InteractionState`]: Current user interaction mode
//! - [`Tool`]: Available drawing tools
//! - [`Command`]: Commands for undo/redo functionality

use std::collections::HashSet;

use crate::shapes::{RemoteCursor, Shape, ShapeId, ShapeStore, ShapeType};

/// Maximum zoom level (1000%)
pub const ZOOM_MAX: f32 = 10.0;
/// Minimum zoom level (10%)
pub const ZOOM_MIN: f32 = 0.1;
/// Default zoom step for keyboard/mouse zoom
pub const ZOOM_STEP: f32 = 0.1;
/// Nudge amount for arrow keys (1 pixel at 100% zoom)
pub const NUDGE_AMOUNT: f64 = 1.0;
/// Large nudge amount with Shift (10 pixels)
pub const LARGE_NUDGE_AMOUNT: f64 = 10.0;
/// Maximum number of commands in undo history
pub const MAX_UNDO_HISTORY: usize = 100;

/// Available tools in the demo
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    Rectangle,
    Ellipse,
    Line,
    Pan,
    Zoom,
}

/// Current interaction state
///
/// Represents the current mode of user interaction with the canvas.
#[derive(Clone, Debug, PartialEq)]
pub enum InteractionState {
    /// No active interaction
    Idle,
    /// Dragging one or more selected shapes
    Dragging {
        /// Shapes being dragged
        shape_ids: HashSet<ShapeId>,
        /// Starting position in world coordinates
        start_x: f64,
        start_y: f64,
        /// Original positions before drag started
        original_positions: Vec<(ShapeId, f64, f64)>,
    },
    /// Creating a new shape
    Creating {
        shape_type: ShapeType,
        start_x: f64,
        start_y: f64,
        current_x: f64,
        current_y: f64,
    },
    /// Resizing a shape (future feature)
    Resizing {
        shape_id: ShapeId,
        handle: ResizeHandle,
        start_x: f64,
        start_y: f64,
        original_width: f64,
        original_height: f64,
    },
    /// Box selection in progress
    BoxSelecting {
        start_x: f64,
        start_y: f64,
        current_x: f64,
        current_y: f64,
    },
    /// Panning the canvas
    Panning {
        start_x: f64,
        start_y: f64,
        original_offset: (f64, f64),
    },
}

/// Resize handle positions (future feature)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// A rectangle used for selection and bounds
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// Creates a new rectangle
    #[inline]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Checks if this rectangle intersects with another
    #[inline]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns the union of two rectangles
    #[inline]
    pub fn union(&self, other: &Rect) -> Rect {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

/// Command trait for undo/redo functionality
trait Command: std::fmt::Debug {
    /// Executes the command and returns a reverse command for undo
    fn execute(&mut self, state: &mut DemoState) -> Box<dyn Command>;
    /// Clones the command for state cloning
    fn clone_box(&self) -> Box<dyn Command>;
}

impl Clone for Box<dyn Command> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Command to create a new shape
#[derive(Debug)]
struct CreateShapeCommand {
    shape_id: ShapeId,
}

impl Command for CreateShapeCommand {
    fn execute(&mut self, state: &mut DemoState) -> Box<dyn Command> {
        // Get shape data before removal for undo
        let shape = state.shapes.get(self.shape_id).cloned();
        state.shapes.remove(self.shape_id);
        state.selected_ids_mut().remove(&self.shape_id);

        Box::new(DeleteShapeCommand {
            shape,
            removed_from_selection: true,
        })
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(CreateShapeCommand {
            shape_id: self.shape_id,
        })
    }
}

/// Command to delete a shape
#[derive(Debug)]
struct DeleteShapeCommand {
    shape: Option<Shape>,
    removed_from_selection: bool,
}

impl Command for DeleteShapeCommand {
    fn execute(&mut self, state: &mut DemoState) -> Box<dyn Command> {
        if let Some(shape) = self.shape.take() {
            self.shape = Some(shape.clone());
            let id = state.shapes.add(shape);
            self.removed_from_selection = state.selected_ids().contains(&id);
            if self.removed_from_selection {
                state.selected_ids_mut().remove(&id);
            }

            return Box::new(CreateShapeCommand { shape_id: id });
        }
        Box::new(DeleteShapeCommand {
            shape: None,
            removed_from_selection: false,
        })
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(DeleteShapeCommand {
            shape: self.shape.clone(),
            removed_from_selection: self.removed_from_selection,
        })
    }
}

/// Command to move shapes
#[derive(Debug)]
struct MoveShapesCommand {
    movements: Vec<(ShapeId, f64, f64)>, // (id, original_x, original_y)
}

impl Command for MoveShapesCommand {
    fn execute(&mut self, state: &mut DemoState) -> Box<dyn Command> {
        let mut reverse_movements = Vec::new();

        for (id, orig_x, orig_y) in &self.movements {
            if let Some(shape) = state.shapes.get_mut(*id) {
                reverse_movements.push((*id, shape.x, shape.y));
                shape.x = *orig_x;
                shape.y = *orig_y;
            }
        }

        Box::new(MoveShapesCommand {
            movements: reverse_movements,
        })
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(MoveShapesCommand {
            movements: self.movements.clone(),
        })
    }
}

/// Command to clear all shapes
#[derive(Debug)]
struct ClearCommand {
    backup_shapes: Vec<Shape>,
    backup_selection: HashSet<ShapeId>,
}

impl Command for ClearCommand {
    fn execute(&mut self, state: &mut DemoState) -> Box<dyn Command> {
        let backup = std::mem::replace(&mut state.shapes, ShapeStore::new());
        let backup_selection = std::mem::replace(&mut state.selection, HashSet::new());

        Box::new(ClearCommand {
            backup_shapes: backup.iter().cloned().collect(),
            backup_selection,
        })
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(ClearCommand {
            backup_shapes: self.backup_shapes.clone(),
            backup_selection: self.backup_selection.clone(),
        })
    }
}

/// Main demo state
#[derive(Debug)]
pub struct DemoState {
    tool: Tool,
    interaction: InteractionState,
    shapes: ShapeStore,
    selection: HashSet<ShapeId>,
    cursors: Vec<RemoteCursor>,
    pan_offset: (f64, f64),
    zoom: f32,
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    current_drag_start: Option<(f64, f64)>,
}

impl Clone for DemoState {
    fn clone(&self) -> Self {
        Self {
            tool: self.tool,
            interaction: self.interaction.clone(),
            shapes: self.shapes.clone(),
            selection: self.selection.clone(),
            cursors: self.cursors.clone(),
            pan_offset: self.pan_offset,
            zoom: self.zoom,
            undo_stack: self.undo_stack.iter().map(|c| c.clone_box()).collect(),
            redo_stack: self.redo_stack.iter().map(|c| c.clone_box()).collect(),
            current_drag_start: self.current_drag_start,
        }
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoState {
    /// Creates a new demo state
    #[inline]
    pub fn new() -> Self {
        Self {
            tool: Tool::Select,
            interaction: InteractionState::Idle,
            shapes: ShapeStore::new(),
            selection: HashSet::new(),
            cursors: Vec::new(),
            pan_offset: (0.0, 0.0),
            zoom: 1.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_drag_start: None,
        }
    }

    // === Tool Management ===

    /// Sets the current tool
    pub fn set_tool(&mut self, tool: &str) {
        self.tool = match tool {
            "rect" | "rectangle" => Tool::Rectangle,
            "ellipse" => Tool::Ellipse,
            "line" => Tool::Line,
            "pan" => Tool::Pan,
            "zoom" => Tool::Zoom,
            _ => Tool::Select,
        };
        self.interaction = InteractionState::Idle;
    }

    /// Sets the select tool
    #[inline]
    pub fn set_select_mode(&mut self) {
        self.tool = Tool::Select;
    }

    // === Selection Management ===

    /// Returns a reference to the selected shape IDs
    pub fn selected_ids(&self) -> &HashSet<ShapeId> {
        &self.selection
    }

    /// Returns a mutable reference to the selected shape IDs
    fn selected_ids_mut(&mut self) -> &mut HashSet<ShapeId> {
        &mut self.selection
    }

    /// Returns the number of selected shapes
    #[inline]
    pub fn selection_count(&self) -> usize {
        self.selection.len()
    }

    /// Checks if any shape is selected
    #[inline]
    pub fn has_selection(&self) -> bool {
        !self.selection.is_empty()
    }

    /// Gets the position of the selected shape (first one)
    pub fn get_selection_position(&self) -> Option<(f64, f64)> {
        self.selection
            .iter()
            .find_map(|id| self.shapes.get(*id).map(|shape| (shape.x, shape.y)))
    }

    /// Clears the current selection
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Adds a shape to the selection
    #[inline]
    pub fn add_to_selection(&mut self, id: ShapeId) {
        self.selection.insert(id);
    }

    /// Removes a shape from the selection
    #[inline]
    pub fn remove_from_selection(&mut self, id: ShapeId) {
        self.selection.remove(&id);
    }

    /// Toggles a shape's selection state
    #[inline]
    pub fn toggle_selection(&mut self, id: ShapeId) {
        if self.selection.contains(&id) {
            self.selection.remove(&id);
        } else {
            self.selection.insert(id);
        }
    }

    /// Selects all shapes
    pub fn select_all(&mut self) {
        for shape in self.shapes.iter() {
            self.selection.insert(shape.id);
        }
    }

    /// Selects all shapes within a bounding box
    pub fn select_in_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        let box_rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);

        for shape in self.shapes.iter() {
            let shape_rect = Rect::new(shape.x, shape.y, shape.width, shape.height);
            if box_rect.intersects(&shape_rect) {
                self.selection.insert(shape.id);
            }
        }
    }

    // === Input Handling ===

    /// Handles mouse down event
    pub fn on_mousedown(&mut self, x: f64, y: f64, button: u16) {
        let world_x = (x - self.pan_offset.0) / self.zoom as f64;
        let world_y = (y - self.pan_offset.1) / self.zoom as f64;

        // Store drag start position
        self.current_drag_start = Some((x, y));

        // Clear redo stack on new action
        self.redo_stack.clear();

        match (self.tool, button) {
            (Tool::Select, 0) => {
                // Left click - check for shape hit
                if let Some(id) = self.shapes.find_at_point(world_x, world_y) {
                    // Start dragging
                    let original_positions: Vec<(ShapeId, f64, f64)> = self
                        .selection
                        .iter()
                        .filter_map(|sid| {
                            self.shapes.get(*sid).map(|shape| (*sid, shape.x, shape.y))
                        })
                        .collect();

                    // If shift is not pressed, clear selection and select this shape
                    // Note: Shift key would need to be tracked separately
                    if !self.selection.contains(&id) {
                        self.selection.clear();
                        self.selection.insert(id);
                    }

                    self.interaction = InteractionState::Dragging {
                        shape_ids: self.selection.clone(),
                        start_x: world_x,
                        start_y: world_y,
                        original_positions,
                    };
                } else {
                    // Click on empty space - start box selection
                    self.selection.clear();
                    self.interaction = InteractionState::BoxSelecting {
                        start_x: world_x,
                        start_y: world_y,
                        current_x: world_x,
                        current_y: world_y,
                    };
                }
            }
            (Tool::Select, 2) => {
                // Right click - start panning
                self.interaction = InteractionState::Panning {
                    start_x: x,
                    start_y: y,
                    original_offset: self.pan_offset,
                };
            }
            (Tool::Pan, _) => {
                // Pan tool - start panning
                self.interaction = InteractionState::Panning {
                    start_x: x,
                    start_y: y,
                    original_offset: self.pan_offset,
                };
            }
            (Tool::Rectangle, 0) | (Tool::Ellipse, 0) | (Tool::Line, 0) => {
                self.interaction = InteractionState::Creating {
                    shape_type: match self.tool {
                        Tool::Rectangle => ShapeType::Rectangle,
                        Tool::Ellipse => ShapeType::Ellipse,
                        Tool::Line => ShapeType::Line,
                        _ => unreachable!(),
                    },
                    start_x: world_x,
                    start_y: world_y,
                    current_x: world_x,
                    current_y: world_y,
                };
            }
            _ => {}
        }
    }

    /// Handles mouse move event
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        let world_x = (x - self.pan_offset.0) / self.zoom as f64;
        let world_y = (y - self.pan_offset.1) / self.zoom as f64;

        match &mut self.interaction {
            InteractionState::Dragging {
                shape_ids: _,
                start_x,
                start_y,
                original_positions,
            } => {
                let dx = world_x - *start_x;
                let dy = world_y - *start_y;

                // Apply movement to all selected shapes
                for (id, orig_x, orig_y) in original_positions.iter() {
                    if let Some(shape) = self.shapes.get_mut(*id) {
                        shape.x = orig_x + dx;
                        shape.y = orig_y + dy;
                    }
                }
            }
            InteractionState::Creating {
                current_x,
                current_y,
                ..
            } => {
                *current_x = world_x;
                *current_y = world_y;
            }
            InteractionState::BoxSelecting {
                start_x: _,
                start_y: _,
                current_x,
                current_y,
            } => {
                *current_x = world_x;
                *current_y = world_y;
            }
            InteractionState::Panning {
                start_x,
                start_y,
                original_offset,
            } => {
                let dx = x - *start_x;
                let dy = y - *start_y;
                self.pan_offset = (original_offset.0 + dx, original_offset.1 + dy);
            }
            InteractionState::Idle | InteractionState::Resizing { .. } => {}
        }
    }

    /// Handles mouse up event
    pub fn on_mouseup(&mut self, x: f64, y: f64) {
        let world_x = (x - self.pan_offset.0) / self.zoom as f64;
        let world_y = (y - self.pan_offset.1) / self.zoom as f64;

        let previous_interaction = std::mem::replace(&mut self.interaction, InteractionState::Idle);

        match previous_interaction {
            InteractionState::Creating {
                shape_type,
                start_x,
                start_y,
                current_x: _,
                current_y: _,
            } => {
                let min_x = start_x.min(world_x);
                let min_y = start_y.min(world_y);
                let width = (world_x - start_x).abs().max(5.0);
                let height = (world_y - start_y).abs().max(5.0);

                if width >= 5.0 && height >= 5.0 {
                    let shape_id = ShapeId::next();
                    let shape = Shape {
                        id: shape_id,
                        shape_type,
                        x: min_x,
                        y: min_y,
                        width,
                        height,
                        color: [70, 130, 180, 255],
                        rotation: 0.0,
                    };
                    self.shapes.add(shape);
                    self.selection.clear();
                    self.selection.insert(shape_id);

                    // Record create command for undo
                    let create_cmd = CreateShapeCommand { shape_id };
                    self.push_undo(Box::new(create_cmd));
                }
            }
            InteractionState::Dragging {
                shape_ids: _,
                start_x: _,
                start_y: _,
                original_positions,
            } => {
                // Record move command for undo
                if !original_positions.is_empty() {
                    let move_cmd = MoveShapesCommand {
                        movements: original_positions.clone(),
                    };
                    self.push_undo(Box::new(move_cmd));
                }
            }
            InteractionState::BoxSelecting {
                start_x,
                start_y,
                current_x,
                current_y,
            } => {
                // Finalize box selection
                self.select_in_box(start_x, start_y, current_x, current_y);
            }
            InteractionState::Panning { .. } => {
                // Panning completed - nothing to record
            }
            InteractionState::Idle | InteractionState::Resizing { .. } => {}
        }

        self.current_drag_start = None;
    }

    /// Handles mouse wheel for zooming
    #[inline]
    pub fn on_wheel(&mut self, _x: f64, _y: f64, zoom_out: bool) {
        let delta = if zoom_out { -ZOOM_STEP } else { ZOOM_STEP };
        self.zoom = (self.zoom + delta).clamp(ZOOM_MIN, ZOOM_MAX);
    }

    // === Pan and Zoom ===

    /// Pans the canvas by the given delta
    #[inline]
    pub fn pan_canvas(&mut self, dx: f64, dy: f64) {
        self.pan_offset = (self.pan_offset.0 + dx, self.pan_offset.1 + dy);
    }

    /// Zooms in by the default step
    #[inline]
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + ZOOM_STEP).clamp(ZOOM_MIN, ZOOM_MAX);
    }

    /// Zooms out by the default step
    #[inline]
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - ZOOM_STEP).clamp(ZOOM_MIN, ZOOM_MAX);
    }

    /// Sets the zoom level
    #[inline]
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(ZOOM_MIN, ZOOM_MAX);
    }

    /// Returns the current zoom level
    #[inline]
    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    /// Returns the current pan offset
    #[inline]
    pub fn get_pan_offset(&self) -> (f64, f64) {
        self.pan_offset
    }

    /// Zooms to fit all shapes in the viewport
    pub fn zoom_to_fit(&mut self) {
        if self.shapes.count() == 0 {
            self.zoom = 1.0;
            return;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for shape in self.shapes.iter() {
            min_x = min_x.min(shape.x);
            min_y = min_y.min(shape.y);
            max_x = max_x.max(shape.x + shape.width);
            max_y = max_y.max(shape.y + shape.height);
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        // Simple zoom calculation - assume viewport is 800x600
        let viewport_width = 800.0;
        let viewport_height = 600.0;

        let scale_x = viewport_width / width;
        let scale_y = viewport_height / height;

        self.zoom = (scale_x.min(scale_y) as f32).clamp(ZOOM_MIN, ZOOM_MAX);
        self.pan_offset = (
            viewport_width / 2.0 - center_x * self.zoom as f64,
            viewport_height / 2.0 - center_y * self.zoom as f64,
        );
    }

    /// Zooms to fit the current selection
    pub fn zoom_to_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for id in &self.selection {
            if let Some(shape) = self.shapes.get(*id) {
                min_x = min_x.min(shape.x);
                min_y = min_y.min(shape.y);
                max_x = max_x.max(shape.x + shape.width);
                max_y = max_y.max(shape.y + shape.height);
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        if width > 0.0 && height > 0.0 {
            let viewport_width = 800.0;
            let viewport_height = 600.0;

            let scale_x = viewport_width / width;
            let scale_y = viewport_height / height;

            let new_zoom = (scale_x.min(scale_y) as f32).clamp(ZOOM_MIN, ZOOM_MAX);

            // Adjust pan to center the selection
            self.pan_offset.0 = viewport_width / 2.0 - center_x * new_zoom as f64;
            self.pan_offset.1 = viewport_height / 2.0 - center_y * new_zoom as f64;
            self.zoom = new_zoom;
        }
    }

    // === Keyboard Navigation ===

    /// Nudges the selected shapes by the given delta
    pub fn nudge_selection(&mut self, dx: f64, dy: f64) {
        if self.selection.is_empty() {
            return;
        }

        let original_positions: Vec<(ShapeId, f64, f64)> = self
            .selection
            .iter()
            .filter_map(|sid| self.shapes.get(*sid).map(|shape| (*sid, shape.x, shape.y)))
            .collect();

        for (id, orig_x, orig_y) in &original_positions {
            if let Some(shape) = self.shapes.get_mut(*id) {
                shape.x = orig_x + dx;
                shape.y = orig_y + dy;
            }
        }

        // Record move command
        let move_cmd = MoveShapesCommand {
            movements: original_positions,
        };
        self.push_undo(Box::new(move_cmd));
    }

    // === Undo/Redo ===

    /// Pushes a command onto the undo stack
    fn push_undo(&mut self, cmd: Box<dyn Command>) {
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Checks if undo is available
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Checks if redo is available
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Undoes the last command
    pub fn undo(&mut self) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            let reverse_cmd = cmd.execute(self);
            self.redo_stack.push(reverse_cmd);
        }
    }

    /// Redoes the last undone command
    pub fn redo(&mut self) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            let reverse_cmd = cmd.execute(self);
            self.undo_stack.push(reverse_cmd);
        }
    }

    // === Shape Operations ===

    /// Adds a rectangle to the canvas
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let shape = Shape {
            id: ShapeId::next(),
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width,
            height,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        };
        let shape_id = self.shapes.add(shape);
        self.selection.clear();
        self.selection.insert(shape_id);

        let create_cmd = CreateShapeCommand { shape_id };
        self.push_undo(Box::new(create_cmd));
    }

    /// Adds an ellipse to the canvas
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) {
        let shape = Shape {
            id: ShapeId::next(),
            shape_type: ShapeType::Ellipse,
            x: x - radius_x,
            y: y - radius_y,
            width: radius_x * 2.0,
            height: radius_y * 2.0,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        };
        let shape_id = self.shapes.add(shape);
        self.selection.clear();
        self.selection.insert(shape_id);

        let create_cmd = CreateShapeCommand { shape_id };
        self.push_undo(Box::new(create_cmd));
    }

    /// Adds a line to the canvas
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let shape = Shape {
            id: ShapeId::next(),
            shape_type: ShapeType::Line,
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        };
        let shape_id = self.shapes.add(shape);
        self.selection.clear();
        self.selection.insert(shape_id);

        let create_cmd = CreateShapeCommand { shape_id };
        self.push_undo(Box::new(create_cmd));
    }

    /// Deletes the currently selected shape
    pub fn delete_selected(&mut self) {
        let selected: Vec<ShapeId> = self.selection.iter().copied().collect();

        for id in &selected {
            let shape = self.shapes.get(*id).cloned();
            self.shapes.remove(*id);

            if let Some(s) = shape {
                let delete_cmd = DeleteShapeCommand {
                    shape: Some(s),
                    removed_from_selection: true,
                };
                self.push_undo(Box::new(delete_cmd));
            }
        }

        self.selection.clear();
    }

    /// Clears all shapes
    pub fn clear(&mut self) {
        if self.shapes.count() > 0 {
            let clear_cmd = ClearCommand {
                backup_shapes: self.shapes.iter().cloned().collect(),
                backup_selection: self.selection.clone(),
            };
            self.push_undo(Box::new(clear_cmd));

            self.shapes.clear();
            self.selection.clear();
        }
    }

    // === Query Methods ===

    /// Returns an iterator over all shapes
    #[inline]
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> {
        self.shapes.iter()
    }

    /// Returns the number of shapes
    #[inline]
    pub fn shape_count(&self) -> usize {
        self.shapes.count()
    }

    /// Returns the bounds of the current selection (union of all selected shapes)
    pub fn selection_bounds(&self) -> Option<Rect> {
        if self.selection.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for id in &self.selection {
            if let Some(shape) = self.shapes.get(*id) {
                min_x = min_x.min(shape.x);
                min_y = min_y.min(shape.y);
                max_x = max_x.max(shape.x + shape.width);
                max_y = max_y.max(shape.y + shape.height);
            }
        }

        Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    /// Returns the current interaction state for rendering
    pub fn interaction_state(&self) -> &InteractionState {
        &self.interaction
    }

    // === Collaboration Simulation ===

    /// Adds or updates a remote cursor
    pub fn add_remote_cursor(&mut self, x: f64, y: f64, name: &str) {
        self.cursors.retain(|c| c.name != name);
        self.cursors.push(RemoteCursor::new(x, y, name));
    }

    /// Returns remote cursors
    #[inline]
    pub fn cursors(&self) -> &[RemoteCursor] {
        &self.cursors
    }

    /// Serializes the current state as a binary delta
    pub fn serialize_delta(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for shape in self.shapes.iter() {
            result.push(shape.shape_type as u8);
            result.extend_from_slice(&shape.id.0.to_le_bytes());
            result.extend_from_slice(&shape.x.to_le_bytes());
            result.extend_from_slice(&shape.y.to_le_bytes());
            result.extend_from_slice(&shape.width.to_le_bytes());
            result.extend_from_slice(&shape.height.to_le_bytes());
        }

        result
    }
}
