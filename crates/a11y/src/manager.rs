//! Accessibility manager for focus, keyboard navigation, and ARIA support

use crate::{
    A11yAnnouncement, A11yBounds, A11yConfig, A11yVerbosity, FocusableElement, FocusableType,
    KeyCode, KeyEvent, KeyEventResult, LiveRegionType,
    navigation::{NavigationDirection, NavigationMode},
};
use crate::{CanvasLike, EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents accessibility properties for an element
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct A11yProperties {
    /// Role attribute (e.g., "img", "button", "application")
    pub role: String,
    /// ARIA label
    pub label: String,
    /// ARIA description
    pub description: String,
    /// ARIA live region
    pub live_region: Option<LiveRegionType>,
    /// Whether it's hidden
    pub hidden: bool,
    /// Whether it's disabled
    pub disabled: bool,
    /// Tab index
    pub tab_index: Option<i32>,
}

/// Accessibility tree node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yNode {
    /// Node ID
    pub id: String,
    /// Role
    pub role: String,
    /// Name (label)
    pub name: String,
    /// Description
    pub description: String,
    /// Children
    pub children: Vec<A11yNode>,
    /// Bounds (for screen reader positioning)
    pub bounds: Option<A11yBounds>,
    /// Whether it's focusable
    pub focusable: bool,
    /// Level (for nested elements like tree items)
    pub level: i32,
}

/// Accessibility manager for the canvas
#[derive(Debug)]
pub struct A11yManager {
    /// Configuration
    config: A11yConfig,
    /// Accessibility properties for elements
    properties: HashMap<EntityId, A11yProperties>,
    /// Focusable elements
    focusable: Vec<FocusableElement>,
    /// Current focused element
    focused: Option<EntityId>,
    /// Navigation mode
    navigation_mode: NavigationMode,
    /// Announcements queue
    announcements: Vec<A11yAnnouncement>,
}

impl A11yManager {
    /// Creates a new accessibility manager
    pub fn new() -> Self {
        Self {
            config: A11yConfig::default(),
            properties: HashMap::new(),
            focusable: Vec::new(),
            focused: None,
            navigation_mode: NavigationMode::Normal,
            announcements: Vec::new(),
        }
    }

    /// Gets current configuration
    pub fn config(&self) -> &A11yConfig {
        &self.config
    }

    /// Sets configuration
    pub fn set_config(&mut self, config: A11yConfig) {
        self.config = config;
    }

    /// Gets the currently focused element
    pub fn focused(&self) -> Option<EntityId> {
        self.focused
    }

    /// Sets the focused element
    pub fn set_focused(&mut self, id: Option<EntityId>) {
        self.focused = id;
    }

    /// Registers a focusable element
    pub fn register_focusable(
        &mut self,
        id: EntityId,
        element_type: FocusableType,
        name: impl Into<String>,
        bounds: A11yBounds,
    ) {
        let focusable = FocusableElement {
            id,
            element_type,
            name: name.into(),
            bounds,
            focus_order: self.focusable.len(),
        };
        self.focusable.push(focusable);
    }

    /// Gets the number of focusable elements
    pub fn focusable_count(&self) -> usize {
        self.focusable.len()
    }

    /// Sets accessibility properties for an element
    pub fn set_properties(&mut self, id: EntityId, props: A11yProperties) {
        self.properties.insert(id, props);
    }

    /// Moves focus to the next element
    pub fn focus_next(&mut self) -> Option<EntityId> {
        if self.focusable.is_empty() {
            return None;
        }

        let next_index = if let Some(id) = self.focused {
            // Find current index and move to next
            let current_index = self.focusable.iter().position(|el| el.id == id)?;
            (current_index + 1) % self.focusable.len()
        } else {
            // No focus yet, start at first element
            0
        };

        self.focused = Some(self.focusable[next_index].id);
        self.focused
    }

    /// Moves focus to the previous element
    pub fn focus_previous(&mut self) -> Option<EntityId> {
        if self.focusable.is_empty() {
            return None;
        }

        let current_index = self
            .focused
            .and_then(|id| self.focusable.iter().position(|el| el.id == id))
            .unwrap_or(0);

        let prev_index = if current_index == 0 {
            self.focusable.len() - 1
        } else {
            current_index - 1
        };
        self.focused = Some(self.focusable[prev_index].id);

        self.focused
    }

    /// Navigates focus in the specified direction
    pub fn navigate(&mut self, direction: NavigationDirection) -> Option<EntityId> {
        if self.focusable.is_empty() {
            return None;
        }

        let current_index = self
            .focused
            .and_then(|id| self.focusable.iter().position(|el| el.id == id))
            .unwrap_or(0);

        let new_index = match direction {
            NavigationDirection::Next => (current_index + 1) % self.focusable.len(),
            NavigationDirection::Previous => {
                if current_index == 0 {
                    self.focusable.len() - 1
                } else {
                    current_index - 1
                }
            }
            NavigationDirection::First => 0,
            NavigationDirection::Last => self.focusable.len() - 1,
            NavigationDirection::Up
            | NavigationDirection::Down
            | NavigationDirection::Left
            | NavigationDirection::Right => {
                // Spatial navigation - find element in direction
                self.find_element_in_direction(current_index, direction)
            }
        };

        self.focused = Some(self.focusable[new_index].id);
        self.focused
    }

    /// Finds an element in a spatial direction
    fn find_element_in_direction(&self, current: usize, direction: NavigationDirection) -> usize {
        if self.focusable.is_empty() {
            return current;
        }

        let current_element = match self.focusable.get(current) {
            Some(el) => el,
            None => return 0,
        };

        let current_center = current_element.bounds.center();

        // Find all elements in the direction cone
        let mut candidates: Vec<(usize, f32)> = self
            .focusable
            .iter()
            .enumerate()
            .filter(|(idx, el)| {
                *idx != current
                    && self.is_in_direction_cone(current_center, el.bounds.center(), direction)
            })
            .map(|(idx, el)| {
                (
                    idx,
                    self.distance_squared(current_center, el.bounds.center()),
                )
            })
            .collect();

        // Sort by distance and return the closest
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates.first().map(|(idx, _)| *idx).unwrap_or(current)
    }

    /// Checks if a target point is within the navigation cone for a direction
    fn is_in_direction_cone(
        &self,
        origin: Vec2,
        target: Vec2,
        direction: NavigationDirection,
    ) -> bool {
        let diff = target - origin;
        const MIN_DISTANCE: f32 = 10.0;

        if diff.length() < MIN_DISTANCE {
            return false;
        }

        match direction {
            NavigationDirection::Up => diff.y < 0.0 && diff.x.abs() < diff.y.abs(),
            NavigationDirection::Down => diff.y > 0.0 && diff.x.abs() < diff.y.abs(),
            NavigationDirection::Left => diff.x < 0.0 && diff.y.abs() < diff.x.abs(),
            NavigationDirection::Right => diff.x > 0.0 && diff.y.abs() < diff.x.abs(),
            _ => true,
        }
    }

    /// Calculates squared distance between two points
    fn distance_squared(&self, a: Vec2, b: Vec2) -> f32 {
        let diff = b - a;
        diff.x * diff.x + diff.y * diff.y
    }

    /// Queues an announcement
    pub fn announce(&mut self, text: impl Into<String>, priority: LiveRegionType) {
        self.announcements.push(A11yAnnouncement {
            text: text.into(),
            priority,
            interrupt: priority == LiveRegionType::Assertive,
        });
    }

    /// Gets pending announcements
    pub fn get_announcements(&mut self) -> Vec<A11yAnnouncement> {
        self.announcements.drain(..).collect()
    }

    /// Builds the accessibility tree for a canvas-like object
    pub fn build_tree(&self, canvas: &dyn CanvasLike) -> A11yNode {
        let mut children = Vec::new();

        // Add layers
        for layer in canvas.all_layers() {
            let layer_node = A11yNode {
                id: layer.id.to_string(),
                role: "group".to_string(),
                name: layer.name.clone(),
                description: format!(
                    "Layer: {} - {}",
                    layer.name,
                    if layer.visible { "visible" } else { "hidden" }
                ),
                children: Vec::new(),
                bounds: None,
                focusable: true,
                level: 1,
            };
            children.push(layer_node);
        }

        // Add shapes
        for shape in canvas.all_shapes() {
            let shape_node = A11yNode {
                id: shape.id.to_string(),
                role: shape.shape_type.to_lowercase(),
                name: format!("{} at ({}, {})", shape.shape_type, shape.x, shape.y),
                description: format!(
                    "{} shape at position ({}, {}) with size {}x{}",
                    shape.shape_type, shape.x, shape.y, shape.width, shape.height
                ),
                children: Vec::new(),
                bounds: Some(A11yBounds::new(shape.x, shape.y, shape.width, shape.height)),
                focusable: true,
                level: 2,
            };
            children.push(shape_node);
        }

        A11yNode {
            id: "canvas".to_string(),
            role: "application".to_string(),
            name: "ArchFlow Canvas".to_string(),
            description: "Interactive diagramming canvas".to_string(),
            children,
            bounds: None,
            focusable: true,
            level: 0,
        }
    }
}

impl Default for A11yManager {
    fn default() -> Self {
        Self::new()
    }
}
