//! Accessibility module for ArchFlow SDK
//!
//! This module provides accessibility features including:
//! - ARIA attribute management for canvas elements
//! - Screen reader support
//! - Keyboard navigation
//! - Focus management
//! - High contrast mode support

use crate::canvas::Canvas;
use crate::layers::C4Level;
use crate::viewport::Viewport;
use archflow_core::{EntityId, Vec2};
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
    /// ARIA controls
    pub controls: Vec<String>,
    /// ARIA described by
    pub described_by: Vec<String>,
    /// ARIA labelled by
    pub labelled_by: Vec<String>,
    /// Whether it's hidden
    pub hidden: bool,
    /// Whether it's disabled
    pub disabled: bool,
    /// Whether it's expanded
    pub expanded: Option<bool>,
    /// Whether it's selected
    pub selected: Option<bool>,
    /// Tab index
    pub tab_index: Option<i32>,
}

/// Type of live region
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveRegionType {
    /// Polite live region (announced when idle)
    Polite,
    /// Assertive live region (announced immediately)
    Assertive,
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
    /// Whether it's selected
    pub selected: bool,
    /// Whether it's expanded
    pub expanded: bool,
    /// Level (for nested elements like tree items)
    pub level: i32,
}

/// Bounding box for accessibility
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl A11yBounds {
    /// Creates new bounds
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the center point of the bounds
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Returns the minimum corner
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Returns the maximum corner
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Checks if a point is contained within the bounds
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

impl Default for A11yBounds {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

/// Keyboard navigation mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationMode {
    /// Normal mode (select shapes, pan, zoom)
    Normal,
    /// Focus mode (tab through elements)
    Focus,
    /// Read mode (read canvas content)
    Read,
}

/// Focusable element in the canvas
#[derive(Clone, Debug)]
pub struct FocusableElement {
    /// Element ID
    pub id: EntityId,
    /// Element type
    pub element_type: FocusableType,
    /// Display name
    pub name: String,
    /// Bounds
    pub bounds: A11yBounds,
    /// Order in focus sequence
    pub focus_order: usize,
}

/// Types of focusable elements
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusableType {
    /// Shape element
    Shape,
    /// Layer
    Layer,
    /// Tool
    Tool,
    /// Menu item
    MenuItem,
    /// Panel
    Panel,
}

/// Screen reader announcement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yAnnouncement {
    /// Announcement text
    pub text: String,
    /// Priority
    pub priority: LiveRegionType,
    /// Whether to interrupt
    pub interrupt: bool,
}

/// Navigation direction for keyboard navigation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationDirection {
    /// Next element in tab order
    Next,
    /// Previous element in tab order
    Previous,
    /// Element above current
    Up,
    /// Element below current
    Down,
    /// Element to the left
    Left,
    /// Element to the right
    Right,
    /// First element
    First,
    /// Last element
    Last,
}

/// Accessibility configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yConfig {
    /// Enable ARIA attributes
    pub enable_aria: bool,
    /// Enable keyboard navigation
    pub enable_keyboard: bool,
    /// Enable screen reader support
    pub enable_screen_reader: bool,
    /// Enable focus indicators
    pub enable_focus_indicators: bool,
    /// Enable high contrast mode
    pub high_contrast_mode: bool,
    /// Focus indicator color
    pub focus_indicator_color: String,
    /// Focus indicator width
    pub focus_indicator_width: f32,
    /// Minimum touch target size
    pub min_touch_target_size: f32,
    /// Enable reduced motion
    pub reduced_motion: bool,
    /// Screen reader verbosity
    pub verbosity: A11yVerbosity,
}

/// Verbosity level for screen reader announcements
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum A11yVerbosity {
    /// Minimal announcements
    Minimal,
    /// Normal announcements
    Normal,
    /// Verbose announcements
    Verbose,
}

/// Key code representation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Arrow keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    /// Navigation keys
    Home,
    End,
    PageUp,
    PageDown,
    /// Action keys
    Enter,
    Space,
    Escape,
    Tab,
    /// Modifier keys
    Shift,
    Control,
    Alt,
    /// Letter keys
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// Number keys
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    /// Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    /// Other
    Backspace,
    Delete,
    Insert,
    Unknown(u32),
}

impl From<u32> for KeyCode {
    fn from(code: u32) -> Self {
        match code {
            37 => Self::ArrowLeft,
            38 => Self::ArrowUp,
            39 => Self::ArrowRight,
            40 => Self::ArrowDown,
            9 => Self::Tab,
            13 => Self::Enter,
            27 => Self::Escape,
            32 => Self::Space,
            36 => Self::Home,
            35 => Self::End,
            33 => Self::PageUp,
            34 => Self::PageDown,
            8 => Self::Backspace,
            46 => Self::Delete,
            _ if (65..=90).contains(&code) => {
                // Convert uppercase letter
                let chr = (code as u8) as char;
                match chr {
                    'A' => Self::A,
                    'B' => Self::B,
                    'C' => Self::C,
                    'D' => Self::D,
                    'E' => Self::E,
                    'F' => Self::F,
                    'G' => Self::G,
                    'H' => Self::H,
                    'I' => Self::I,
                    'J' => Self::J,
                    'K' => Self::K,
                    'L' => Self::L,
                    'M' => Self::M,
                    'N' => Self::N,
                    'O' => Self::O,
                    'P' => Self::P,
                    'Q' => Self::Q,
                    'R' => Self::R,
                    'S' => Self::S,
                    'T' => Self::T,
                    'U' => Self::U,
                    'V' => Self::V,
                    'W' => Self::W,
                    'X' => Self::X,
                    'Y' => Self::Y,
                    'Z' => Self::Z,
                    _ => Self::Unknown(code),
                }
            }
            _ if (48..=57).contains(&code) => match code {
                48 => Self::Digit0,
                49 => Self::Digit1,
                50 => Self::Digit2,
                51 => Self::Digit3,
                52 => Self::Digit4,
                53 => Self::Digit5,
                54 => Self::Digit6,
                55 => Self::Digit7,
                56 => Self::Digit8,
                57 => Self::Digit9,
                _ => Self::Unknown(code),
            },
            _ => Self::Unknown(code),
        }
    }
}

/// Keyboard modifier state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    /// Shift key pressed
    pub shift: bool,
    /// Control key pressed
    pub ctrl: bool,
    /// Alt key pressed
    pub alt: bool,
    /// Meta/Command key pressed
    pub meta: bool,
}

/// Keyboard event
#[derive(Clone, Debug)]
pub struct KeyEvent {
    /// Key code
    pub key_code: KeyCode,
    /// Modifier keys
    pub modifiers: Modifiers,
    /// Whether this is a key down event (false for key up)
    pub key_down: bool,
    /// Whether the key was repeated
    pub repeated: bool,
}

/// Result of processing a keyboard event
#[derive(Clone, Debug)]
pub struct KeyEventResult {
    /// Whether the event was handled
    pub handled: bool,
    /// Announcement to make (if any)
    pub announcement: Option<A11yAnnouncement>,
    /// Whether focus changed
    pub focus_changed: bool,
    /// New focus index (if changed)
    pub new_focus_index: Option<usize>,
}

impl Default for A11yConfig {
    fn default() -> Self {
        Self {
            enable_aria: true,
            enable_keyboard: true,
            enable_screen_reader: true,
            enable_focus_indicators: true,
            high_contrast_mode: false,
            focus_indicator_color: "#0066cc".to_string(),
            focus_indicator_width: 2.0,
            min_touch_target_size: 44.0,
            reduced_motion: false,
            verbosity: A11yVerbosity::Normal,
        }
    }
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
    /// Current focus index
    focus_index: Option<usize>,
    /// Navigation mode
    navigation_mode: NavigationMode,
    /// Announcements queue
    announcements: Vec<A11yAnnouncement>,
    /// Description for the entire canvas
    canvas_description: String,
}

impl A11yManager {
    /// Creates a new accessibility manager
    pub fn new() -> Self {
        Self {
            config: A11yConfig::default(),
            properties: HashMap::new(),
            focusable: Vec::new(),
            focus_index: None,
            navigation_mode: NavigationMode::Normal,
            announcements: Vec::new(),
            canvas_description: String::new(),
        }
    }

    /// Updates configuration
    pub fn set_config(&mut self, config: A11yConfig) {
        self.config = config;
    }

    /// Gets current configuration
    pub fn config(&self) -> &A11yConfig {
        &self.config
    }

    /// Gets mutable configuration (for testing)
    #[cfg(test)]
    pub fn config_mut(&mut self) -> &mut A11yConfig {
        &mut self.config
    }

    /// Sets accessibility properties for an element
    pub fn set_properties(&mut self, id: EntityId, props: A11yProperties) {
        self.properties.insert(id, props);
    }

    /// Gets accessibility properties for an element
    pub fn get_properties(&self, id: EntityId) -> Option<&A11yProperties> {
        self.properties.get(&id)
    }

    /// Removes accessibility properties for an element
    pub fn remove_properties(&mut self, id: EntityId) {
        self.properties.remove(&id);
    }

    /// Builds the accessibility tree for the canvas
    pub fn build_tree(&self, canvas: &Canvas) -> A11yNode {
        let mut children = Vec::new();

        // Add layers as children
        for layer in canvas.layer_manager().all_layers() {
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
                selected: false,
                expanded: true,
                level: 1,
            };
            children.push(layer_node);
        }

        // Add shapes as children
        for shape in canvas.all_shapes() {
            let shape_node = A11yNode {
                id: shape.id.to_string(),
                role: format!("{:?}", shape.shape_type).to_lowercase(),
                name: format!("{:?} at ({}, {:?})", shape.shape_type, shape.x, shape.y),
                description: format!(
                    "{} shape at position ({}, {}) with size {}x{}",
                    format!("{:?}", shape.shape_type),
                    shape.x,
                    shape.y,
                    shape.width,
                    shape.height
                ),
                children: Vec::new(),
                bounds: Some(A11yBounds {
                    x: shape.x,
                    y: shape.y,
                    width: shape.width,
                    height: shape.height,
                }),
                focusable: true,
                selected: false, // Will be updated based on actual selection state
                expanded: false,
                level: 2,
            };
            children.push(shape_node);
        }

        A11yNode {
            id: "canvas".to_string(),
            role: "application".to_string(),
            name: "ArchFlow Canvas".to_string(),
            description: self.canvas_description.clone(),
            children,
            bounds: None,
            focusable: true,
            selected: false,
            expanded: true,
            level: 0,
        }
    }

    /// Sets the navigation mode
    pub fn set_navigation_mode(&mut self, mode: NavigationMode) {
        self.navigation_mode = mode;
    }

    /// Gets the navigation mode
    pub fn navigation_mode(&self) -> NavigationMode {
        self.navigation_mode
    }

    // === Keyboard Navigation Methods ===

    /// Processes a keyboard event and returns the result
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event to process
    ///
    /// # Returns
    ///
    /// A result indicating whether the event was handled and any announcements to make
    pub fn handle_key_event(&mut self, event: KeyEvent) -> KeyEventResult {
        if !self.config.enable_keyboard || !event.key_down || event.repeated {
            return KeyEventResult {
                handled: false,
                announcement: None,
                focus_changed: false,
                new_focus_index: None,
            };
        }

        // Handle mode switching keys
        if event.key_code == KeyCode::Escape {
            self.navigation_mode = NavigationMode::Normal;
            return KeyEventResult {
                handled: true,
                announcement: Some(A11yAnnouncement {
                    text: "Exited accessibility mode".to_string(),
                    priority: LiveRegionType::Polite,
                    interrupt: false,
                }),
                focus_changed: false,
                new_focus_index: None,
            };
        }

        // Enter accessibility mode with Enter key
        if event.key_code == KeyCode::Enter && self.navigation_mode == NavigationMode::Normal {
            self.navigation_mode = NavigationMode::Focus;
            return KeyEventResult {
                handled: true,
                announcement: Some(A11yAnnouncement {
                    text: "Focus mode activated. Use arrow keys to navigate.".to_string(),
                    priority: LiveRegionType::Polite,
                    interrupt: true,
                }),
                focus_changed: false,
                new_focus_index: None,
            };
        }

        // Handle navigation based on current mode
        match self.navigation_mode {
            NavigationMode::Normal => self.handle_normal_mode(event),
            NavigationMode::Focus => self.handle_focus_mode(event),
            NavigationMode::Read => self.handle_read_mode(event),
        }
    }

    /// Handles keyboard events in normal mode
    fn handle_normal_mode(&mut self, event: KeyEvent) -> KeyEventResult {
        // Tab moves focus through interactive elements
        if event.key_code == KeyCode::Tab {
            let direction = if event.modifiers.shift {
                NavigationDirection::Previous
            } else {
                NavigationDirection::Next
            };

            let new_index = self.navigate(direction);
            if let Some(index) = new_index {
                return KeyEventResult {
                    handled: true,
                    announcement: None,
                    focus_changed: true,
                    new_focus_index: Some(index),
                };
            }
        }

        KeyEventResult {
            handled: false,
            announcement: None,
            focus_changed: false,
            new_focus_index: None,
        }
    }

    /// Handles keyboard events in focus mode
    fn handle_focus_mode(&mut self, event: KeyEvent) -> KeyEventResult {
        let direction = match event.key_code {
            KeyCode::ArrowUp | KeyCode::PageUp => Some(NavigationDirection::Up),
            KeyCode::ArrowDown | KeyCode::PageDown => Some(NavigationDirection::Down),
            KeyCode::ArrowLeft => Some(NavigationDirection::Left),
            KeyCode::ArrowRight => Some(NavigationDirection::Right),
            KeyCode::Home => Some(NavigationDirection::First),
            KeyCode::End => Some(NavigationDirection::Last),
            KeyCode::Enter | KeyCode::Space => {
                return self.handle_activation();
            }
            KeyCode::Tab if event.modifiers.shift => {
                self.navigation_mode = NavigationMode::Normal;
                return KeyEventResult {
                    handled: true,
                    announcement: Some(A11yAnnouncement {
                        text: "Exited focus mode".to_string(),
                        priority: LiveRegionType::Polite,
                        interrupt: false,
                    }),
                    focus_changed: false,
                    new_focus_index: None,
                };
            }
            _ => None,
        };

        if let Some(dir) = direction {
            let new_index = self.navigate(dir);
            if let Some(index) = new_index {
                let element = &self.focusable[index];
                let announcement = self.generate_focus_announcement(element);

                return KeyEventResult {
                    handled: true,
                    announcement,
                    focus_changed: true,
                    new_focus_index: Some(index),
                };
            }
        }

        KeyEventResult {
            handled: false,
            announcement: None,
            focus_changed: false,
            new_focus_index: None,
        }
    }

    /// Handles keyboard events in read mode
    fn handle_read_mode(&mut self, event: KeyEvent) -> KeyEventResult {
        if event.key_code == KeyCode::Escape || event.key_code == KeyCode::Tab {
            self.navigation_mode = NavigationMode::Normal;
            return KeyEventResult {
                handled: true,
                announcement: Some(A11yAnnouncement {
                    text: "Exited read mode".to_string(),
                    priority: LiveRegionType::Polite,
                    interrupt: false,
                }),
                focus_changed: false,
                new_focus_index: None,
            };
        }

        // In read mode, arrow keys read elements without moving focus
        if let Some(dir) = match event.key_code {
            KeyCode::ArrowUp | KeyCode::ArrowLeft => Some(NavigationDirection::Previous),
            KeyCode::ArrowDown | KeyCode::ArrowRight => Some(NavigationDirection::Next),
            KeyCode::Home => Some(NavigationDirection::First),
            KeyCode::End => Some(NavigationDirection::Last),
            _ => None,
        } {
            if let Some(element) = self.read_element(dir) {
                return KeyEventResult {
                    handled: true,
                    announcement: Some(A11yAnnouncement {
                        text: element.name.clone(),
                        priority: LiveRegionType::Polite,
                        interrupt: false,
                    }),
                    focus_changed: false,
                    new_focus_index: None,
                };
            }
        }

        KeyEventResult {
            handled: false,
            announcement: None,
            focus_changed: false,
            new_focus_index: None,
        }
    }

    /// Handles element activation (Enter/Space key)
    fn handle_activation(&mut self) -> KeyEventResult {
        if let Some(element) = self.focused_element() {
            let announcement = format!("Activated {}", element.name);
            return KeyEventResult {
                handled: true,
                announcement: Some(A11yAnnouncement {
                    text: announcement,
                    priority: LiveRegionType::Assertive,
                    interrupt: true,
                }),
                focus_changed: false,
                new_focus_index: None,
            };
        }

        KeyEventResult {
            handled: false,
            announcement: None,
            focus_changed: false,
            new_focus_index: None,
        }
    }

    /// Navigates focus in the specified direction
    fn navigate(&mut self, direction: NavigationDirection) -> Option<usize> {
        if self.focusable.is_empty() {
            self.rebuild_focusable();
        }

        if self.focusable.is_empty() {
            return None;
        }

        let current_index = self.focus_index.unwrap_or(0);
        let new_index = match direction {
            NavigationDirection::Next => (current_index + 1) % self.focusable.len(),
            NavigationDirection::Previous => {
                if current_index == 0 {
                    self.focusable.len() - 1
                } else {
                    current_index - 1
                }
            }
            NavigationDirection::Up | NavigationDirection::Previous => {
                self.find_element_in_direction(current_index, -1)
            }
            NavigationDirection::Down | NavigationDirection::Next => {
                self.find_element_in_direction(current_index, 1)
            }
            NavigationDirection::Left => self.find_element_in_direction(current_index, -1),
            NavigationDirection::Right => self.find_element_in_direction(current_index, 1),
            NavigationDirection::First => 0,
            NavigationDirection::Last => self.focusable.len() - 1,
        };

        self.focus_index = Some(new_index);
        Some(new_index)
    }

    /// Finds an element in a spatial direction using actual spatial queries
    ///
    /// This method implements directional navigation by:
    /// 1. Getting the current element's center point
    /// 2. Defining a search cone in the target direction
    /// 3. Finding all elements within that cone
    /// 4. Selecting the closest element
    fn find_element_in_direction(&self, current: usize, _delta: isize) -> usize {
        if self.focusable.is_empty() {
            return current;
        }

        let current_element = match self.focusable.get(current) {
            Some(el) => el,
            None => return 0,
        };

        let current_center = current_element.bounds.center();
        let direction = self.determine_navigation_direction(current_center, _delta);

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

    /// Determines the navigation direction based on delta
    fn determine_navigation_direction(
        &self,
        _current_center: Vec2,
        delta: isize,
    ) -> NavigationDirection {
        match delta {
            -1 => NavigationDirection::Up,
            1 => NavigationDirection::Down,
            _ => NavigationDirection::Next,
        }
    }

    /// Checks if a target point is within the navigation cone for a direction
    fn is_in_direction_cone(
        &self,
        origin: Vec2,
        target: Vec2,
        direction: NavigationDirection,
    ) -> bool {
        let diff = target - origin;

        // Minimum distance threshold (in pixels) to avoid jumping to nearby elements
        const MIN_DISTANCE: f32 = 10.0;

        if diff.length() < MIN_DISTANCE {
            return false;
        }

        match direction {
            NavigationDirection::Up => {
                // Target must be above (negative Y) and within 45 degrees of vertical
                diff.y < 0.0 && diff.x.abs() < diff.y.abs()
            }
            NavigationDirection::Down => {
                // Target must be below (positive Y) and within 45 degrees of vertical
                diff.y > 0.0 && diff.x.abs() < diff.y.abs()
            }
            NavigationDirection::Left => {
                // Target must be to the left (negative X) and within 45 degrees of horizontal
                diff.x < 0.0 && diff.y.abs() < diff.x.abs()
            }
            NavigationDirection::Right => {
                // Target must be to the right (positive X) and within 45 degrees of horizontal
                diff.x > 0.0 && diff.y.abs() < diff.x.abs()
            }
            _ => true,
        }
    }

    /// Calculates squared distance between two points (avoids sqrt for performance)
    fn distance_squared(&self, a: Vec2, b: Vec2) -> f32 {
        let diff = b - a;
        diff.x * diff.x + diff.y * diff.y
    }

    /// Reads an element in the specified direction without moving focus
    fn read_element(&self, direction: NavigationDirection) -> Option<&FocusableElement> {
        if self.focusable.is_empty() {
            return None;
        }

        let current_index = self.focus_index.unwrap_or(0);
        let read_index = match direction {
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
            _ => (current_index + 1) % self.focusable.len(),
        };

        self.focusable.get(read_index)
    }

    /// Generates an announcement for the currently focused element
    fn generate_focus_announcement(&self, element: &FocusableElement) -> Option<A11yAnnouncement> {
        if self.config.verbosity == A11yVerbosity::Minimal {
            return None;
        }

        let text = if self.config.verbosity == A11yVerbosity::Verbose {
            format!(
                "{} at position {}, type {}",
                element.name,
                format!("({:?})", element.bounds),
                format!("{:?}", element.element_type)
            )
        } else {
            element.name.clone()
        };

        Some(A11yAnnouncement {
            text,
            priority: LiveRegionType::Polite,
            interrupt: false,
        })
    }

    /// Announces focus change
    fn announce_focus(&mut self) {
        if let Some(element) = self.focused_element() {
            self.generate_focus_announcement(element)
                .map(|a| self.announcements.push(a));
        }
    }

    /// Moves focus to the next element
    pub fn focus_next(&mut self) -> Option<&FocusableElement> {
        if self.focusable.is_empty() {
            self.rebuild_focusable();
        }

        if self.focusable.is_empty() {
            return None;
        }

        let next_index = match self.focus_index {
            Some(i) => (i + 1) % self.focusable.len(),
            None => 0,
        };

        self.focus_index = Some(next_index);
        self.announce_focus();

        self.focusable.get(next_index)
    }

    /// Moves focus to the previous element
    pub fn focus_previous(&mut self) -> Option<&FocusableElement> {
        if self.focusable.is_empty() {
            self.rebuild_focusable();
        }

        if self.focusable.is_empty() {
            return None;
        }

        let prev_index = match self.focus_index {
            Some(0) => self.focusable.len() - 1,
            Some(i) => i - 1,
            None => self.focusable.len() - 1,
        };

        self.focus_index = Some(prev_index);
        self.announce_focus();

        self.focusable.get(prev_index)
    }

    /// Sets focus to a specific element
    pub fn set_focus(&mut self, id: EntityId) -> bool {
        for (index, element) in self.focusable.iter().enumerate() {
            if element.id == id {
                self.focus_index = Some(index);
                self.announce_focus();
                return true;
            }
        }
        false
    }

    /// Clears focus
    pub fn clear_focus(&mut self) {
        self.focus_index = None;
    }

    /// Gets the currently focused element
    pub fn focused_element(&self) -> Option<&FocusableElement> {
        self.focus_index.and_then(|i| self.focusable.get(i))
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

    /// Clears all announcements
    pub fn clear_announcements(&mut self) {
        self.announcements.clear();
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

    /// Unregisters a focusable element
    pub fn unregister_focusable(&mut self, id: EntityId) {
        self.focusable.retain(|el| el.id != id);
        // Update focus orders
        for (index, el) in self.focusable.iter_mut().enumerate() {
            el.focus_order = index;
        }
    }

    /// Updates the bounds of a focusable element
    pub fn update_focusable_bounds(&mut self, id: EntityId, bounds: A11yBounds) {
        if let Some(el) = self.focusable.iter_mut().find(|e| e.id == id) {
            el.bounds = bounds;
        }
    }

    /// Clears all focusable elements
    pub fn clear_focusable(&mut self) {
        self.focusable.clear();
        self.focus_index = None;
    }

    /// Gets the number of focusable elements
    pub fn focusable_count(&self) -> usize {
        self.focusable.len()
    }

    /// Rebuilds the focusable elements list from a canvas
    /// This is a convenience method that extracts focusable elements from a canvas
    pub fn rebuild_from_canvas(&mut self, canvas: &Canvas) {
        self.clear_focusable();

        // Add layers as focusable
        for layer in canvas.layer_manager().all_layers() {
            self.register_focusable(
                layer.id,
                FocusableType::Layer,
                format!("Layer: {}", layer.name),
                A11yBounds::new(0.0, 0.0, 100.0, 30.0), // Placeholder bounds
            );
        }

        // Add shapes as focusable
        for shape in canvas.all_shapes() {
            self.register_focusable(
                shape.id,
                FocusableType::Shape,
                format!("{:?} at ({}, {})", shape.shape_type, shape.x, shape.y),
                A11yBounds::new(shape.x, shape.y, shape.width, shape.height),
            );
        }
    }

    /// Sets the canvas description
    pub fn set_canvas_description(&mut self, description: impl Into<String>) {
        self.canvas_description = description.into();
    }

    /// Generates ARIA attributes for an element
    pub fn generate_aria_attrs(&self, id: EntityId) -> String {
        if !self.config.enable_aria {
            return String::new();
        }

        if let Some(props) = self.properties.get(&id) {
            let mut attrs = Vec::new();

            if !props.role.is_empty() {
                attrs.push(format!(r#"role="{}""#, props.role));
            }

            if !props.label.is_empty() {
                attrs.push(format!(r#"aria-label="{}""#, props.label));
            }

            if !props.description.is_empty() {
                attrs.push(format!(r#"aria-description="{}""#, props.description));
            }

            if let Some(live) = props.live_region {
                let live_value = match live {
                    LiveRegionType::Polite => "polite",
                    LiveRegionType::Assertive => "assertive",
                };
                attrs.push(format!(r#"aria-live="{}""#, live_value));
            }

            if !props.controls.is_empty() {
                attrs.push(format!(r#"aria-controls="{}""#, props.controls.join(" ")));
            }

            if !props.described_by.is_empty() {
                attrs.push(format!(
                    r#"aria-describedby="{}""#,
                    props.described_by.join(" ")
                ));
            }

            if !props.labelled_by.is_empty() {
                attrs.push(format!(
                    r#"aria-labelledby="{}""#,
                    props.labelled_by.join(" ")
                ));
            }

            if props.hidden {
                attrs.push("aria-hidden=\"true\"".to_string());
            }

            if props.disabled {
                attrs.push("aria-disabled=\"true\"".to_string());
            }

            if let Some(expanded) = props.expanded {
                attrs.push(format!(r#"aria-expanded="{}""#, expanded));
            }

            if let Some(selected) = props.selected {
                attrs.push(format!(r#"aria-selected="{}""#, selected));
            }

            if let Some(tab_index) = props.tab_index {
                attrs.push(format!(r#"tabindex="{}""#, tab_index));
            }

            attrs.join(" ")
        } else {
            String::new()
        }
    }

    /// Rebuilds the focusable elements list
    fn rebuild_focusable(&mut self) {
        // This is now a no-op since we have public methods for registration
        // Kept for backwards compatibility but should not be called directly
    }
}

/// Converts FocusableType to string
impl FocusableType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FocusableType::Shape => "shape",
            FocusableType::Layer => "layer",
            FocusableType::Tool => "tool",
            FocusableType::MenuItem => "menu item",
            FocusableType::Panel => "panel",
        }
    }
}

impl Default for A11yManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates keyboard shortcut help text
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyboardShortcutHelp {
    pub category: String,
    pub shortcuts: Vec<ShortcutInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortcutInfo {
    pub keys: String,
    pub description: String,
    pub action: String,
}

/// Generates accessibility report for the canvas
pub fn generate_a11y_report(canvas: &Canvas, manager: &A11yManager) -> A11yReport {
    let tree = manager.build_tree(canvas);

    A11yReport {
        summary: A11ySummary {
            total_elements: count_nodes(&tree),
            focusable_elements: manager.focusable.len(),
            elements_with_labels: manager
                .properties
                .values()
                .filter(|p| !p.label.is_empty())
                .count(),
            elements_with_descriptions: manager
                .properties
                .values()
                .filter(|p| !p.description.is_empty())
                .count(),
        },
        tree,
        keyboard_shortcuts: generate_shortcut_help(),
    }
}

/// Accessibility report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yReport {
    pub summary: A11ySummary,
    pub tree: A11yNode,
    pub keyboard_shortcuts: Vec<KeyboardShortcutHelp>,
}

/// Summary of accessibility state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11ySummary {
    pub total_elements: usize,
    pub focusable_elements: usize,
    pub elements_with_labels: usize,
    pub elements_with_descriptions: usize,
}

/// Counts nodes in the tree
fn count_nodes(node: &A11yNode) -> usize {
    let mut count = 1;
    for child in &node.children {
        count += count_nodes(child);
    }
    count
}

/// Generates keyboard shortcut help
fn generate_shortcut_help() -> Vec<KeyboardShortcutHelp> {
    vec![
        KeyboardShortcutHelp {
            category: "Navigation".to_string(),
            shortcuts: vec![
                ShortcutInfo {
                    keys: "Tab / Shift+Tab".to_string(),
                    description: "Move focus between elements".to_string(),
                    action: "navigate".to_string(),
                },
                ShortcutInfo {
                    keys: "Arrow Keys".to_string(),
                    description: "Navigate between elements or pan canvas".to_string(),
                    action: "navigate_pan".to_string(),
                },
                ShortcutInfo {
                    keys: "Page Up / Page Down".to_string(),
                    description: "Scroll the viewport".to_string(),
                    action: "scroll".to_string(),
                },
            ],
        },
        KeyboardShortcutHelp {
            category: "Selection".to_string(),
            shortcuts: vec![
                ShortcutInfo {
                    keys: "Click / Enter".to_string(),
                    description: "Select focused element".to_string(),
                    action: "select".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + Click".to_string(),
                    description: "Add to selection".to_string(),
                    action: "add_select".to_string(),
                },
                ShortcutInfo {
                    keys: "Escape".to_string(),
                    description: "Clear selection".to_string(),
                    action: "clear_selection".to_string(),
                },
            ],
        },
        KeyboardShortcutHelp {
            category: "Editing".to_string(),
            shortcuts: vec![
                ShortcutInfo {
                    keys: "Delete / Backspace".to_string(),
                    description: "Delete selected element".to_string(),
                    action: "delete".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + Z".to_string(),
                    description: "Undo last action".to_string(),
                    action: "undo".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + Shift + Z".to_string(),
                    description: "Redo last action".to_string(),
                    action: "redo".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + C".to_string(),
                    description: "Copy selected elements".to_string(),
                    action: "copy".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + V".to_string(),
                    description: "Paste elements".to_string(),
                    action: "paste".to_string(),
                },
            ],
        },
        KeyboardShortcutHelp {
            category: "Zoom".to_string(),
            shortcuts: vec![
                ShortcutInfo {
                    keys: "Ctrl + Plus".to_string(),
                    description: "Zoom in".to_string(),
                    action: "zoom_in".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + Minus".to_string(),
                    description: "Zoom out".to_string(),
                    action: "zoom_out".to_string(),
                },
                ShortcutInfo {
                    keys: "Ctrl + 0".to_string(),
                    description: "Reset zoom".to_string(),
                    action: "zoom_reset".to_string(),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a11y_manager_creation() {
        let manager = A11yManager::new();
        assert!(manager.config().enable_aria);
        assert!(manager.config().enable_keyboard);
    }

    #[test]
    fn test_a11y_properties() {
        let props = A11yProperties {
            role: "button".to_string(),
            label: "Click me".to_string(),
            description: "A clickable button".to_string(),
            live_region: Some(LiveRegionType::Polite),
            controls: vec!["panel1".to_string()],
            described_by: vec!["desc1".to_string()],
            labelled_by: vec!["label1".to_string()],
            hidden: false,
            disabled: false,
            expanded: None,
            selected: None,
            tab_index: Some(0),
        };

        assert_eq!(props.role, "button");
        assert_eq!(props.label, "Click me");
        assert!(props.tab_index.is_some());
    }

    #[test]
    fn test_focusable_element() {
        let element = FocusableElement {
            id: EntityId::new(),
            element_type: FocusableType::Shape,
            name: "Rectangle 1".to_string(),
            bounds: A11yBounds {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            focus_order: 0,
        };

        assert_eq!(element.element_type.as_str(), "shape");
        assert!(element.bounds.width > 0.0);
    }

    #[test]
    fn test_a11y_announcement() {
        let announcement = A11yAnnouncement {
            text: "Shape selected".to_string(),
            priority: LiveRegionType::Polite,
            interrupt: false,
        };

        assert_eq!(announcement.text, "Shape selected");
        assert!(!announcement.interrupt);
    }

    #[test]
    fn test_a11y_config_defaults() {
        let config = A11yConfig::default();
        assert!(config.enable_aria);
        assert!(config.enable_keyboard);
        assert!(config.enable_screen_reader);
        assert_eq!(config.verbosity, A11yVerbosity::Normal);
    }

    #[test]
    fn test_navigation_mode() {
        assert_eq!(NavigationMode::Normal, NavigationMode::Normal);
        assert_ne!(NavigationMode::Normal, NavigationMode::Focus);
        assert_ne!(NavigationMode::Normal, NavigationMode::Read);
    }

    #[test]
    fn test_live_region_type() {
        assert_eq!(LiveRegionType::Polite, LiveRegionType::Polite);
        assert_ne!(LiveRegionType::Polite, LiveRegionType::Assertive);
    }

    #[test]
    fn test_a11y_verbosity() {
        assert_eq!(A11yVerbosity::Minimal, A11yVerbosity::Minimal);
        assert_eq!(A11yVerbosity::Normal, A11yVerbosity::Normal);
        assert_eq!(A11yVerbosity::Verbose, A11yVerbosity::Verbose);
    }

    #[test]
    fn test_a11y_bounds() {
        let bounds = A11yBounds {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };

        assert_eq!(bounds.x, 10.0);
        assert_eq!(bounds.y, 20.0);
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 50.0);
    }

    #[test]
    fn test_count_nodes() {
        let root = A11yNode {
            id: "root".to_string(),
            role: "application".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            children: vec![
                A11yNode {
                    id: "child1".to_string(),
                    role: "group".to_string(),
                    name: "Child 1".to_string(),
                    description: String::new(),
                    children: vec![A11yNode {
                        id: "grandchild".to_string(),
                        role: "shape".to_string(),
                        name: "Shape".to_string(),
                        description: String::new(),
                        children: Vec::new(),
                        bounds: None,
                        focusable: false,
                        selected: false,
                        expanded: false,
                        level: 0,
                    }],
                    bounds: None,
                    focusable: false,
                    selected: false,
                    expanded: false,
                    level: 0,
                },
                A11yNode {
                    id: "child2".to_string(),
                    role: "shape".to_string(),
                    name: "Shape 2".to_string(),
                    description: String::new(),
                    children: Vec::new(),
                    bounds: None,
                    focusable: false,
                    selected: false,
                    expanded: false,
                    level: 0,
                },
            ],
            bounds: None,
            focusable: false,
            selected: false,
            expanded: false,
            level: 0,
        };

        assert_eq!(count_nodes(&root), 4); // root + child1 + grandchild + child2
    }

    // === Spatial Navigation Tests ===

    #[test]
    fn test_a11y_bounds_center() {
        let bounds = A11yBounds::new(0.0, 0.0, 100.0, 50.0);
        let center = bounds.center();

        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 25.0);
    }

    #[test]
    fn test_a11y_bounds_contains() {
        let bounds = A11yBounds::new(10.0, 10.0, 100.0, 50.0);

        // Point inside
        assert!(bounds.contains(Vec2::new(50.0, 30.0)));

        // Point outside - left
        assert!(!bounds.contains(Vec2::new(5.0, 30.0)));

        // Point outside - right
        assert!(!bounds.contains(Vec2::new(120.0, 30.0)));

        // Point outside - top
        assert!(!bounds.contains(Vec2::new(50.0, 5.0)));

        // Point outside - bottom
        assert!(!bounds.contains(Vec2::new(50.0, 70.0)));
    }

    #[test]
    fn test_a11y_bounds_min_max() {
        let bounds = A11yBounds::new(10.0, 20.0, 100.0, 50.0);

        assert_eq!(bounds.min(), Vec2::new(10.0, 20.0));
        assert_eq!(bounds.max(), Vec2::new(110.0, 70.0));
    }

    #[test]
    fn test_register_focusable() {
        let mut manager = A11yManager::new();
        let id = EntityId::new();

        manager.register_focusable(
            id,
            FocusableType::Shape,
            "Test Shape",
            A11yBounds::new(0.0, 0.0, 100.0, 50.0),
        );

        assert_eq!(manager.focusable_count(), 1);
        // Note: A11yManager doesn't have is_selected, that's for SelectionManager
        // assert!(manager.is_selected(&id));
    }

    #[test]
    fn test_unregister_focusable() {
        let mut manager = A11yManager::new();
        let id = EntityId::new();

        manager.register_focusable(
            id,
            FocusableType::Shape,
            "Test Shape",
            A11yBounds::new(0.0, 0.0, 100.0, 50.0),
        );

        assert_eq!(manager.focusable_count(), 1);

        manager.unregister_focusable(id);

        assert_eq!(manager.focusable_count(), 0);
    }

    #[test]
    fn test_update_focusable_bounds() {
        let mut manager = A11yManager::new();
        let id = EntityId::new();

        manager.register_focusable(
            id,
            FocusableType::Shape,
            "Test Shape",
            A11yBounds::new(0.0, 0.0, 100.0, 50.0),
        );

        manager.set_focus(id);

        manager.update_focusable_bounds(id, A11yBounds::new(50.0, 50.0, 200.0, 100.0));

        let element = manager.focused_element();
        assert!(element.is_some());
        assert_eq!(element.unwrap().bounds.x, 50.0);
    }

    #[test]
    fn test_clear_focusable() {
        let mut manager = A11yManager::new();

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 100.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 2",
            A11yBounds::new(100.0, 100.0, 100.0, 50.0),
        );

        assert_eq!(manager.focusable_count(), 2);

        manager.clear_focusable();

        assert_eq!(manager.focusable_count(), 0);
        assert!(manager.focused_element().is_none());
    }

    #[test]
    fn test_focus_navigation_next() {
        let mut manager = A11yManager::new();

        // Create 3 focusable elements
        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        let id2 = EntityId::new();
        manager.register_focusable(
            id2,
            FocusableType::Shape,
            "Shape 2",
            A11yBounds::new(100.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 3",
            A11yBounds::new(200.0, 0.0, 50.0, 50.0),
        );

        // Focus first element
        manager.set_focus(manager.focusable[0].id);
        assert_eq!(manager.focus_index, Some(0));

        // Navigate to next
        manager.focus_next();
        assert_eq!(manager.focus_index, Some(1));

        // Navigate to next (should wrap to last, then to first)
        manager.focus_next();
        assert_eq!(manager.focus_index, Some(2));

        manager.focus_next();
        assert_eq!(manager.focus_index, Some(0)); // Wrapped around
    }

    #[test]
    fn test_focus_navigation_previous() {
        let mut manager = A11yManager::new();

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 2",
            A11yBounds::new(100.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 3",
            A11yBounds::new(200.0, 0.0, 50.0, 50.0),
        );

        // Focus last element
        manager.set_focus(manager.focusable[2].id);
        assert_eq!(manager.focus_index, Some(2));

        // Navigate to previous
        manager.focus_previous();
        assert_eq!(manager.focus_index, Some(1));

        manager.focus_previous();
        assert_eq!(manager.focus_index, Some(0));

        manager.focus_previous();
        assert_eq!(manager.focus_index, Some(2)); // Wrapped around
    }

    #[test]
    fn test_spatial_navigation_up() {
        let mut manager = A11yManager::new();

        // Create elements in a vertical line
        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Bottom Shape",
            A11yBounds::new(0.0, 100.0, 50.0, 50.0),
        );

        let middle_id = EntityId::new();
        manager.register_focusable(
            middle_id,
            FocusableType::Shape,
            "Middle Shape",
            A11yBounds::new(0.0, 50.0, 50.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Top Shape",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        // Focus middle element
        manager.set_focus(middle_id);

        // Navigate up (should go to top element)
        let event = KeyEvent {
            key_code: KeyCode::ArrowUp,
            modifiers: Modifiers::default(),
            key_down: true,
            repeated: false,
        };

        manager.set_navigation_mode(NavigationMode::Focus);
        let result = manager.handle_key_event(event);

        assert!(result.handled);
        assert!(result.focus_changed);
        // Should navigate to element at index 2 (top)
    }

    #[test]
    fn test_spatial_navigation_down() {
        let mut manager = A11yManager::new();

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Top Shape",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        let middle_id = EntityId::new();
        manager.register_focusable(
            middle_id,
            FocusableType::Shape,
            "Middle Shape",
            A11yBounds::new(0.0, 50.0, 50.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Bottom Shape",
            A11yBounds::new(0.0, 100.0, 50.0, 50.0),
        );

        // Focus middle element
        manager.set_focus(middle_id);

        // Navigate down (should go to bottom element)
        let event = KeyEvent {
            key_code: KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            key_down: true,
            repeated: false,
        };

        manager.set_navigation_mode(NavigationMode::Focus);
        let result = manager.handle_key_event(event);

        assert!(result.handled);
        assert!(result.focus_changed);
    }

    #[test]
    fn test_spatial_navigation_left_right() {
        let mut manager = A11yManager::new();

        // Create elements in a horizontal line
        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Left Shape",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        let middle_id = EntityId::new();
        manager.register_focusable(
            middle_id,
            FocusableType::Shape,
            "Middle Shape",
            A11yBounds::new(100.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Right Shape",
            A11yBounds::new(200.0, 0.0, 50.0, 50.0),
        );

        // Focus middle element
        manager.set_focus(middle_id);

        // Navigate right
        let event_right = KeyEvent {
            key_code: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            key_down: true,
            repeated: false,
        };

        manager.set_navigation_mode(NavigationMode::Focus);
        let result_right = manager.handle_key_event(event_right);

        assert!(result_right.handled);
        assert!(result_right.focus_changed);
    }

    #[test]
    fn test_key_event_from_u32() {
        // Test conversion from raw key codes
        let key_up: KeyCode = 38.into();
        assert_eq!(key_up, KeyCode::ArrowUp);

        let key_down: KeyCode = 40.into();
        assert_eq!(key_down, KeyCode::ArrowDown);

        let key_a: KeyCode = 65.into();
        assert_eq!(key_a, KeyCode::A);

        let key_0: KeyCode = 48.into();
        assert_eq!(key_0, KeyCode::Digit0);
    }

    #[test]
    fn test_navigation_direction_enum() {
        assert_eq!(NavigationDirection::Up, NavigationDirection::Up);
        assert_eq!(NavigationDirection::Down, NavigationDirection::Down);
        assert_eq!(NavigationDirection::Left, NavigationDirection::Left);
        assert_eq!(NavigationDirection::Right, NavigationDirection::Right);
        assert_eq!(NavigationDirection::Next, NavigationDirection::Next);
        assert_eq!(NavigationDirection::Previous, NavigationDirection::Previous);
        assert_eq!(NavigationDirection::First, NavigationDirection::First);
        assert_eq!(NavigationDirection::Last, NavigationDirection::Last);
    }

    #[test]
    fn test_focus_mode_activation() {
        let mut manager = A11yManager::new();

        // Activate focus mode with Enter
        let event = KeyEvent {
            key_code: KeyCode::Enter,
            modifiers: Modifiers::default(),
            key_down: true,
            repeated: false,
        };

        let result = manager.handle_key_event(event);

        assert!(result.handled);
        assert_eq!(manager.navigation_mode(), NavigationMode::Focus);
        assert!(result.announcement.is_some());
        assert!(result.announcement.unwrap().text.contains("Focus mode"));
    }

    #[test]
    fn test_escape_exits_focus_mode() {
        let mut manager = A11yManager::new();

        // Enter focus mode
        manager.set_navigation_mode(NavigationMode::Focus);

        // Exit with Escape
        let event = KeyEvent {
            key_code: KeyCode::Escape,
            modifiers: Modifiers::default(),
            key_down: true,
            repeated: false,
        };

        let result = manager.handle_key_event(event);

        assert!(result.handled);
        assert_eq!(manager.navigation_mode(), NavigationMode::Normal);
        assert!(result.announcement.unwrap().text.contains("Exited"));
    }

    #[test]
    fn test_announcement_queue() {
        let mut manager = A11yManager::new();

        manager.announce("Test announcement", LiveRegionType::Polite);

        let announcements = manager.get_announcements();
        assert_eq!(announcements.len(), 1);
        assert_eq!(announcements[0].text, "Test announcement");

        // Should be cleared after getting
        let announcements2 = manager.get_announcements();
        assert_eq!(announcements2.len(), 0);
    }

    #[test]
    fn test_set_canvas_description() {
        let mut manager = A11yManager::new();

        manager.set_canvas_description("Test Canvas Description");

        // Build tree and verify description is included
        let canvas = Canvas::new(800.0, 600.0);
        let tree = manager.build_tree(&canvas);

        assert_eq!(tree.description, "Test Canvas Description");
    }

    #[test]
    fn test_aria_attribute_generation() {
        let mut manager = A11yManager::new();
        let id = EntityId::new();

        let props = A11yProperties {
            role: "button".to_string(),
            label: "Click me".to_string(),
            description: "A clickable button".to_string(),
            live_region: Some(LiveRegionType::Polite),
            controls: vec!["panel1".to_string()],
            described_by: vec!["desc1".to_string()],
            labelled_by: vec!["label1".to_string()],
            hidden: false,
            disabled: false,
            expanded: None,
            selected: None,
            tab_index: Some(0),
        };

        manager.set_properties(id, props);

        let aria = manager.generate_aria_attrs(id);

        assert!(aria.contains(r#"role="button""#));
        assert!(aria.contains(r#"aria-label="Click me""#));
        assert!(aria.contains(r#"aria-description="A clickable button""#));
        assert!(aria.contains(r#"aria-live="polite""#));
        assert!(aria.contains(r#"aria-controls="panel1""#));
        assert!(aria.contains(r#"tabindex="0""#));
    }

    #[test]
    fn test_distance_squared_calculation() {
        let manager = A11yManager::new();

        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0); // Distance should be 5, so squared is 25

        let distance_sq = manager.distance_squared(a, b);

        assert!((distance_sq - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_is_in_direction_cone_up() {
        let manager = A11yManager::new();

        let origin = Vec2::new(100.0, 100.0);
        let target_up = Vec2::new(100.0, 50.0); // Directly above
        let target_down = Vec2::new(100.0, 150.0); // Directly below
        let target_diagonal = Vec2::new(50.0, 50.0); // Diagonal (45°)

        assert!(manager.is_in_direction_cone(origin, target_up, NavigationDirection::Up));
        assert!(!manager.is_in_direction_cone(origin, target_down, NavigationDirection::Up));
        // Diagonal should not be in "up" cone (within 45°)
        assert!(!manager.is_in_direction_cone(origin, target_diagonal, NavigationDirection::Up));
    }

    #[test]
    fn test_is_in_direction_cone_down() {
        let manager = A11yManager::new();

        let origin = Vec2::new(100.0, 100.0);
        let target_down = Vec2::new(100.0, 150.0);
        let target_up = Vec2::new(100.0, 50.0);

        assert!(manager.is_in_direction_cone(origin, target_down, NavigationDirection::Down));
        assert!(!manager.is_in_direction_cone(origin, target_up, NavigationDirection::Down));
    }

    #[test]
    fn test_is_in_direction_cone_left() {
        let manager = A11yManager::new();

        let origin = Vec2::new(100.0, 100.0);
        let target_left = Vec2::new(50.0, 100.0);
        let target_right = Vec2::new(150.0, 100.0);

        assert!(manager.is_in_direction_cone(origin, target_left, NavigationDirection::Left));
        assert!(!manager.is_in_direction_cone(origin, target_right, NavigationDirection::Left));
    }

    #[test]
    fn test_is_in_direction_cone_right() {
        let manager = A11yManager::new();

        let origin = Vec2::new(100.0, 100.0);
        let target_right = Vec2::new(150.0, 100.0);
        let target_left = Vec2::new(50.0, 100.0);

        assert!(manager.is_in_direction_cone(origin, target_right, NavigationDirection::Right));
        assert!(!manager.is_in_direction_cone(origin, target_left, NavigationDirection::Right));
    }

    #[test]
    fn test_is_in_direction_cone_min_distance() {
        let manager = A11yManager::new();

        let origin = Vec2::new(100.0, 100.0);
        let target_too_close = Vec2::new(100.0, 105.0); // Only 5 pixels away

        // Should not be in direction cone due to minimum distance threshold
        assert!(!manager.is_in_direction_cone(origin, target_too_close, NavigationDirection::Down));
    }

    #[test]
    fn test_set_focus_by_id() {
        let mut manager = A11yManager::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        manager.register_focusable(
            id1,
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            id2,
            FocusableType::Shape,
            "Shape 2",
            A11yBounds::new(100.0, 0.0, 50.0, 50.0),
        );

        // Set focus to second element
        assert!(manager.set_focus(id2));
        assert_eq!(manager.focus_index, Some(1));

        // Try to set focus to non-existent element
        assert!(!manager.set_focus(EntityId::new()));
    }

    #[test]
    fn test_clear_focus() {
        let mut manager = A11yManager::new();

        let id = EntityId::new();
        manager.register_focusable(
            id,
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        manager.set_focus(id);
        assert!(manager.focused_element().is_some());

        manager.clear_focus();
        assert!(manager.focused_element().is_none());
        assert!(manager.focus_index.is_none());
    }

    #[test]
    fn test_focused_element() {
        let mut manager = A11yManager::new();

        let id = EntityId::new();
        manager.register_focusable(
            id,
            FocusableType::Shape,
            "Test Shape",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        manager.set_focus(id);

        let focused = manager.focused_element();
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().id, id);
    }

    #[test]
    fn test_focused_element_none_when_no_focus() {
        let manager = A11yManager::new();

        // No focusable elements
        assert!(manager.focused_element().is_none());

        // Focusable elements but no focus set
        let mut manager = A11yManager::new();
        manager.register_focusable(
            EntityId::new(),
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        assert!(manager.focused_element().is_none());
    }

    #[test]
    fn test_a11y_config_verbosity() {
        let mut manager = A11yManager::new();

        // Test minimal verbosity (no announcements)
        manager.config_mut().verbosity = A11yVerbosity::Minimal;
        manager.set_navigation_mode(NavigationMode::Focus);

        let id = EntityId::new();
        manager.register_focusable(
            id,
            FocusableType::Shape,
            "Test Shape",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        manager.set_focus(id);

        // With minimal verbosity, generate_focus_announcement should return None
        {
            let focused = manager.focused_element().unwrap();
            let announcement = manager.generate_focus_announcement(focused);
            assert!(announcement.is_none());
        }

        // Test verbose verbosity (detailed announcements)
        manager.config_mut().verbosity = A11yVerbosity::Verbose;

        {
            let focused = manager.focused_element().unwrap();
            let announcement_verbose = manager.generate_focus_announcement(focused);
            assert!(announcement_verbose.is_some());
            assert!(announcement_verbose.unwrap().text.contains("type"));
        }
    }
}
