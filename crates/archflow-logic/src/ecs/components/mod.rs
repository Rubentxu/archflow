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

use crate::ecs::{Component, VecStorage};
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
