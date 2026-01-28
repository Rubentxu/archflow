//! Selection Manager - Box Selection and Selection State Management
//!
//! Provides:
//! - SelectionManager: Manages selection state and box selection
//! - SelectionMode: Controls selection behavior (replace, add, subtract)
//! - SelectionDelta: Changes to selection state for undo/redo

use archflow_core::{EntityId, Rect, Vec2};
use archflow_primitives::selection::{DragSelectionBox, SelectionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Mode of selection operation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Replace current selection (default)
    Replace,
    /// Add to current selection
    Add,
    /// Subtract from current selection
    Subtract,
    /// Intersect with current selection
    Intersect,
}

/// Changes to selection state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionDelta {
    /// IDs that were selected
    pub selected: Vec<EntityId>,
    /// IDs that were deselected
    pub deselected: Vec<EntityId>,
    /// Previous selection bounds
    pub previous_bounds: Option<Rect>,
    /// New selection bounds
    pub new_bounds: Option<Rect>,
}

impl SelectionDelta {
    /// Creates an empty delta
    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            deselected: Vec::new(),
            previous_bounds: None,
            new_bounds: None,
        }
    }

    /// Checks if there's any change
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty() && self.deselected.is_empty()
    }
}

/// Callback for querying shapes within a rectangle
///
/// This allows the SelectionManager to work with any spatial indexing system
/// without being tightly coupled to specific trait implementations.
#[allow(clippy::type_complexity)]
pub type ShapeQueryCallback = dyn Fn(Rect) -> Vec<EntityId> + 'static;

/// Selection manager for handling shape selection and box selection
///
/// # Example
///
/// ```rust
/// use archflow_sdk::{SelectionManager, SelectionMode};
///
/// let mut selection_manager = SelectionManager::new();
///
/// // Start box selection
/// selection_manager.start_box_selection(100.0, 100.0, SelectionMode::Replace);
///
/// // Update selection box
/// selection_manager.update_box_selection(200.0, 150.0);
///
/// // Finalize box selection
/// let delta = selection_manager.finalize_box_selection(|rect| {
///     // Query shapes within rect from your spatial index
///     Vec::new()
/// });
/// ```
pub struct SelectionManager {
    /// Currently selected shape IDs
    selected: HashSet<EntityId>,
    /// Selection bounds in canvas coordinates
    bounds: Option<Rect>,
    /// Drag selection box state
    drag_box: DragSelectionBox,
    /// Selection configuration
    config: SelectionConfig,
    /// Whether selection is currently active
    is_active: bool,
    /// Selection mode for next operation
    mode: SelectionMode,
    /// Callback for querying shapes in a rectangle
    query_callback: Option<Box<ShapeQueryCallback>>,
}

impl SelectionManager {
    /// Creates a new selection manager
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            bounds: None,
            drag_box: DragSelectionBox::new(),
            config: SelectionConfig::default(),
            is_active: false,
            mode: SelectionMode::Replace,
            query_callback: None,
        }
    }

    /// Sets the query callback for finding shapes within a rectangle
    ///
    /// # Arguments
    ///
    /// * `callback` - Function that takes a rectangle and returns IDs of shapes within it
    pub fn set_query_callback<F>(&mut self, callback: F)
    where
        F: Fn(Rect) -> Vec<EntityId> + 'static,
    {
        self.query_callback = Some(Box::new(callback));
    }

    /// Gets the current selection as a vector (for API compatibility)
    pub fn selected_ids(&self) -> Vec<EntityId> {
        self.selected.iter().cloned().collect()
    }

    /// Checks if a shape is selected
    pub fn is_selected(&self, id: &EntityId) -> bool {
        self.selected.contains(id)
    }

    /// Gets the selection bounds
    pub fn bounds(&self) -> Option<Rect> {
        self.bounds
    }

    /// Checks if selection is active
    pub fn is_active(&self) -> bool {
        !self.selected.is_empty() || self.drag_box.is_active
    }

    /// Gets the drag selection box
    pub fn drag_box(&self) -> &DragSelectionBox {
        &self.drag_box
    }

    /// Gets selection configuration
    pub fn config(&self) -> &SelectionConfig {
        &self.config
    }

    /// Updates selection configuration
    pub fn set_config(&mut self, config: SelectionConfig) {
        self.config = config;
    }

    /// Sets the selection mode
    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.mode = mode;
    }

    /// Gets the current selection mode
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Starts a box selection operation
    ///
    /// # Arguments
    ///
    /// * `x` - Screen X coordinate of selection start
    /// * `y` - Screen Y coordinate of selection start
    /// * `mode` - Selection mode (replace, add, subtract, intersect)
    pub fn start_box_selection(&mut self, x: f32, y: f32, mode: SelectionMode) {
        self.mode = mode;
        self.drag_box.start(x, y, mode == SelectionMode::Add);
        self.is_active = true;
    }

    /// Updates the box selection as the user drags
    ///
    /// # Arguments
    ///
    /// * `x` - Current screen X coordinate
    /// * `y` - Current screen Y coordinate
    pub fn update_box_selection(&mut self, x: f32, y: f32) {
        if self.drag_box.is_active {
            self.drag_box.update(x, y);
        }
    }

    /// Finalizes box selection and selects shapes within the box
    ///
    /// # Arguments
    ///
    /// * `screen_to_canvas` - Function to convert screen to canvas coordinates
    /// * `commit` - Whether to commit the selection changes
    ///
    /// # Returns
    ///
    /// The selection delta with changes
    pub fn finalize_box_selection<F>(&mut self, screen_to_canvas: F, commit: bool) -> SelectionDelta
    where
        F: Fn(Vec2) -> Vec2,
    {
        let previous_bounds = self.bounds;
        let previous_selection: HashSet<EntityId> = self.selected.clone();

        // Get the selection rectangle in canvas coordinates
        if !self.drag_box.has_area() {
            // Single click - for single click, we would need hit testing which is separate
            self.drag_box.end();
            self.is_active = false;
            return SelectionDelta {
                selected: Vec::new(),
                deselected: Vec::new(),
                previous_bounds,
                new_bounds: self.bounds,
            };
        }

        // Convert drag rect from screen to canvas coordinates
        let start_canvas = screen_to_canvas(Vec2::new(
            self.drag_box.start_point().unwrap_or((0.0, 0.0)).0,
            self.drag_box.start_point().unwrap_or((0.0, 0.0)).1,
        ));
        let end_canvas = screen_to_canvas(Vec2::new(
            self.drag_box.current_point().unwrap_or((0.0, 0.0)).0,
            self.drag_box.current_point().unwrap_or((0.0, 0.0)).1,
        ));

        let canvas_rect = Rect::from_min_max(
            Vec2::new(
                start_canvas.x.min(end_canvas.x),
                start_canvas.y.min(end_canvas.y),
            ),
            Vec2::new(
                start_canvas.x.max(end_canvas.x),
                start_canvas.y.max(end_canvas.y),
            ),
        );

        // Query shapes within the selection rectangle using the callback
        let intersecting_ids = match &self.query_callback {
            Some(callback) => callback(canvas_rect),
            None => Vec::new(),
        };

        // Apply selection mode
        match self.mode {
            SelectionMode::Replace => {
                self.selected.clear();
                for id in &intersecting_ids {
                    self.selected.insert(*id);
                }
            }
            SelectionMode::Add => {
                for id in &intersecting_ids {
                    self.selected.insert(*id);
                }
            }
            SelectionMode::Subtract => {
                for id in &intersecting_ids {
                    self.selected.remove(id);
                }
            }
            SelectionMode::Intersect => {
                self.selected.retain(|id| intersecting_ids.contains(id));
            }
        }

        // Calculate new bounds
        self.bounds = if self.selected.is_empty() {
            None
        } else {
            Some(canvas_rect)
        };

        // End drag selection
        self.drag_box.end();
        self.is_active = false;

        // Calculate delta
        let selected: Vec<EntityId> = self
            .selected
            .difference(&previous_selection)
            .cloned()
            .collect();
        let deselected: Vec<EntityId> = previous_selection
            .difference(&self.selected)
            .cloned()
            .collect();

        if commit {
            SelectionDelta {
                selected,
                deselected,
                previous_bounds,
                new_bounds: self.bounds,
            }
        } else {
            // Revert changes
            self.selected = previous_selection;
            self.bounds = previous_bounds;

            SelectionDelta {
                selected: Vec::new(),
                deselected: Vec::new(),
                previous_bounds,
                new_bounds: self.bounds,
            }
        }
    }

    /// Directly selects shapes by IDs
    ///
    /// # Arguments
    ///
    /// * `ids` - Shape IDs to select
    /// * `mode` - Selection mode
    ///
    /// # Returns
    ///
    /// The selection delta
    pub fn select_shapes(&mut self, ids: &[EntityId], mode: SelectionMode) -> SelectionDelta {
        let previous_bounds = self.bounds;
        let previous_selection: HashSet<EntityId> = self.selected.clone();

        match mode {
            SelectionMode::Replace => {
                self.selected.clear();
                for id in ids {
                    self.selected.insert(*id);
                }
            }
            SelectionMode::Add => {
                for id in ids {
                    self.selected.insert(*id);
                }
            }
            SelectionMode::Subtract => {
                for id in ids {
                    self.selected.remove(id);
                }
            }
            SelectionMode::Intersect => {
                let id_set: HashSet<EntityId> = ids.iter().cloned().collect();
                self.selected.retain(|id| id_set.contains(id));
            }
        }

        // Calculate new bounds - for direct selection, we can't compute bounds
        // without the actual shape data, so we set it to None
        self.bounds = None;

        let selected: Vec<EntityId> = self
            .selected
            .difference(&previous_selection)
            .cloned()
            .collect();
        let deselected: Vec<EntityId> = previous_selection
            .difference(&self.selected)
            .cloned()
            .collect();

        SelectionDelta {
            selected,
            deselected,
            previous_bounds,
            new_bounds: self.bounds,
        }
    }

    /// Clears the selection
    ///
    /// # Returns
    ///
    /// The selection delta
    pub fn clear_selection(&mut self) -> SelectionDelta {
        let previous_bounds = self.bounds;
        let previous_selection: HashSet<EntityId> = self.selected.clone();

        self.selected.clear();
        self.bounds = None;

        let deselected: Vec<EntityId> = previous_selection.iter().cloned().collect();

        SelectionDelta {
            selected: Vec::new(),
            deselected,
            previous_bounds,
            new_bounds: self.bounds,
        }
    }

    /// Selects all shapes
    ///
    /// # Arguments
    ///
    /// * `all_ids` - All available shape IDs
    ///
    /// # Returns
    ///
    /// The selection delta
    pub fn select_all(&mut self, all_ids: &[EntityId]) -> SelectionDelta {
        let previous_bounds = self.bounds;
        let previous_selection: HashSet<EntityId> = self.selected.clone();

        self.selected.clear();
        for id in all_ids {
            self.selected.insert(*id);
        }

        self.bounds = None;

        let selected: Vec<EntityId> = self
            .selected
            .difference(&previous_selection)
            .cloned()
            .collect();
        let deselected: Vec<EntityId> = previous_selection
            .difference(&self.selected)
            .cloned()
            .collect();

        SelectionDelta {
            selected,
            deselected,
            previous_bounds,
            new_bounds: self.bounds,
        }
    }

    /// Inverts the selection
    ///
    /// # Arguments
    ///
    /// * `all_ids` - All available shape IDs
    ///
    /// # Returns
    ///
    /// The selection delta
    pub fn invert_selection(&mut self, all_ids: &[EntityId]) -> SelectionDelta {
        let previous_bounds = self.bounds;
        let previous_selection: HashSet<EntityId> = self.selected.clone();

        let all_set: HashSet<EntityId> = all_ids.iter().cloned().collect();
        let new_selection: HashSet<EntityId> =
            all_set.difference(&self.selected).cloned().collect();

        self.selected = new_selection;
        self.bounds = None;

        let selected: Vec<EntityId> = self
            .selected
            .difference(&previous_selection)
            .cloned()
            .collect();
        let deselected: Vec<EntityId> = previous_selection
            .difference(&self.selected)
            .cloned()
            .collect();

        SelectionDelta {
            selected,
            deselected,
            previous_bounds,
            new_bounds: self.bounds,
        }
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_manager_new() {
        let manager = SelectionManager::new();
        assert!(manager.selected_ids().is_empty());
        assert!(manager.bounds().is_none());
        assert!(!manager.is_active());
    }

    #[test]
    fn test_select_shapes_replace() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();
        let id3 = EntityId::new();

        let delta = manager.select_shapes(&[id1, id2], SelectionMode::Replace);

        assert!(manager.is_selected(&id1));
        assert!(manager.is_selected(&id2));
        assert!(!manager.is_selected(&id3));
        assert_eq!(manager.selected_ids().len(), 2);
        assert!(delta.selected.contains(&id1));
        assert!(delta.selected.contains(&id2));
        assert!(delta.deselected.is_empty());
    }

    #[test]
    fn test_select_shapes_add() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        manager.select_shapes(&[id1], SelectionMode::Replace);
        let delta = manager.select_shapes(&[id2], SelectionMode::Add);

        assert!(manager.is_selected(&id1));
        assert!(manager.is_selected(&id2));
        assert_eq!(manager.selected_ids().len(), 2);
        assert!(delta.selected.contains(&id2));
    }

    #[test]
    fn test_select_shapes_subtract() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        manager.select_shapes(&[id1, id2], SelectionMode::Replace);
        let delta = manager.select_shapes(&[id2], SelectionMode::Subtract);

        assert!(manager.is_selected(&id1));
        assert!(!manager.is_selected(&id2));
        assert_eq!(manager.selected_ids().len(), 1);
        assert!(delta.deselected.contains(&id2));
    }

    #[test]
    fn test_select_shapes_intersect() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();
        let id3 = EntityId::new();

        manager.select_shapes(&[id1, id2, id3], SelectionMode::Replace);
        let delta = manager.select_shapes(&[id1, id3], SelectionMode::Intersect);

        assert!(manager.is_selected(&id1));
        assert!(!manager.is_selected(&id2));
        assert!(manager.is_selected(&id3));
        assert_eq!(manager.selected_ids().len(), 2);
    }

    #[test]
    fn test_clear_selection() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        manager.select_shapes(&[id1], SelectionMode::Replace);

        let delta = manager.clear_selection();

        assert!(manager.selected_ids().is_empty());
        assert!(delta.deselected.contains(&id1));
    }

    #[test]
    fn test_select_all() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();
        let id3 = EntityId::new();

        // Select some first
        manager.select_shapes(&[id1], SelectionMode::Replace);

        let delta = manager.select_all(&[id1, id2, id3]);

        assert_eq!(manager.selected_ids().len(), 3);
        assert!(manager.is_selected(&id2));
        assert!(manager.is_selected(&id3));
    }

    #[test]
    fn test_invert_selection() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();
        let id3 = EntityId::new();

        manager.select_shapes(&[id1, id2], SelectionMode::Replace);
        let delta = manager.invert_selection(&[id1, id2, id3]);

        assert!(!manager.is_selected(&id1));
        assert!(!manager.is_selected(&id2));
        assert!(manager.is_selected(&id3));
        assert_eq!(manager.selected_ids().len(), 1);
    }

    #[test]
    fn test_box_selection_drag() {
        let mut manager = SelectionManager::new();

        // Start box selection
        manager.start_box_selection(100.0, 100.0, SelectionMode::Replace);
        assert!(manager.drag_box().is_active);

        // Update as user drags
        manager.update_box_selection(200.0, 150.0);

        // Check drag box state
        let start = manager.drag_box().start_point();
        let current = manager.drag_box().current_point();

        assert!(start.is_some());
        assert!(current.is_some());
        assert_eq!(start.unwrap(), (100.0, 100.0));
        assert_eq!(current.unwrap(), (200.0, 150.0));
    }

    #[test]
    fn test_selection_mode() {
        let mut manager = SelectionManager::new();

        assert_eq!(manager.mode(), SelectionMode::Replace);

        manager.set_mode(SelectionMode::Add);
        assert_eq!(manager.mode(), SelectionMode::Add);

        manager.set_mode(SelectionMode::Subtract);
        assert_eq!(manager.mode(), SelectionMode::Subtract);

        manager.set_mode(SelectionMode::Intersect);
        assert_eq!(manager.mode(), SelectionMode::Intersect);
    }

    #[test]
    fn test_selection_delta_empty() {
        let delta = SelectionDelta::new();
        assert!(delta.is_empty());

        let id = EntityId::new();
        let non_empty_delta = SelectionDelta {
            selected: vec![id],
            deselected: Vec::new(),
            previous_bounds: None,
            new_bounds: None,
        };
        assert!(!non_empty_delta.is_empty());
    }

    #[test]
    fn test_box_selection_with_callback() {
        let mut manager = SelectionManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        // Set up query callback
        manager.set_query_callback(move |_rect| vec![id1, id2]);

        // Start and finalize box selection
        manager.start_box_selection(0.0, 0.0, SelectionMode::Replace);
        manager.update_box_selection(100.0, 100.0);

        let delta = manager.finalize_box_selection(|v| v, true);

        assert!(manager.is_selected(&id1));
        assert!(manager.is_selected(&id2));
        assert_eq!(manager.selected_ids().len(), 2);
        assert!(delta.selected.contains(&id1));
        assert!(delta.selected.contains(&id2));
    }
}
