// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Hybrid BGE Integration Module
//
// This module provides integration components for Blender Game Engine (BGE) logic bricks,
// enabling seamless interoperability between BGE's sensor-controller-actuator paradigm
// and the ArchFlow ECS architecture.
//
// Architecture:
// - SensorComponent: Event generators (mouse, keyboard, proximity)
// - ControllerComponent: Logic evaluators (AND, OR, NOT, pulse, toggle)
// - ActuatorComponent: Action executors (highlight, select, move, etc.)
//
// All components integrate with the existing ECS Query API and System trait,
// allowing BGE logic to run within the standard system scheduling framework.
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::any::TypeId;
use core::hash::{Hash, Hasher};

use crate::ecs::component::Component;
use crate::ecs::component::ComponentId;

// Module declarations
pub mod bge_logic_system;

// Re-exports
use crate::ecs::component::VecStorage;
pub use bge_logic_system::{BgeLogicConfig, BgeLogicStats, BgeLogicSystem, SensorEvaluation};

/// Represents the type of sensor event to detect.
///
/// Sensors are the input layer of BGE logic, generating events based on
/// user interactions or world state changes. Each variant captures a specific
/// type of interaction with its associated configuration.
#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SensorComponent {
    /// Triggers when mouse cursor hovers over the entity.
    ///
    /// Configuration:
    /// - `distance`: Maximum distance for hover detection (default: 100.0)
    MouseHover {
        /// Maximum distance from ray to entity for hover detection.
        #[deprecated(
            since = "0.50.0",
            note = "Use SensorConfig::distance instead. This field will be removed."
        )]
        distance: f32,
    },

    /// Triggers on mouse click events.
    ///
    /// Configuration:
    /// - `button`: Mouse button to listen for (0=left, 1=right, 2=middle)
    /// - `click_type`: Single click, double click, or long press
    MouseClick {
        /// Mouse button index (0 = left, 1 = right, 2 = middle).
        button: u8,
        /// Type of click event to detect.
        click_type: ClickType,
    },

    /// Triggers when another entity enters a proximity zone.
    ///
    /// Configuration:
    /// - `radius`: Radius of the proximity detection zone
    /// - `entity_type`: Optional filter for entity type to detect
    Proximity {
        /// Radius of the spherical detection zone.
        radius: f32,
        /// Optional entity type ID to filter detections.
        #[doc(hidden)]
        _entity_type_id: Option<TypeId>,
    },

    /// Triggers on keyboard shortcut activation.
    ///
    /// Configuration:
    /// - `key`: Key code to listen for
    /// - `modifiers`: Optional modifier keys (Ctrl, Shift, Alt)
    KeyShortcut {
        /// Virtual key code.
        key: u32,
        /// Bitmask of modifier keys.
        #[deprecated(since = "0.50.0", note = "Use SensorConfig::modifiers instead")]
        modifiers: u32,
    },

    /// Triggers on rapid double-click detection.
    ///
    /// Configuration:
    /// - `time_window`: Maximum time between clicks in milliseconds
    DoubleTap {
        /// Maximum time window between clicks (default: 300ms).
        time_window_ms: u32,
    },

    /// Triggers after sustained press duration.
    ///
    /// Configuration:
    /// - `duration_ms`: Minimum press duration in milliseconds
    LongPress {
        /// Minimum duration to trigger (default: 500ms).
        duration_ms: u32,
    },

    /// Triggers on right-click events.
    ///
    /// This is a convenience variant for right-click detection.
    /// Equivalent to `MouseClick { button: 1, click_type: Single }`.
    RightClick,
    /// Always triggers (always active).
    ///
    /// Configuration:
    /// - No configuration needed - this sensor is always active
    Always,
    /// Triggers when an entity property matches a target value.
    ///
    /// Configuration:
    /// - `property_name`: Name of the property to check
    /// - `comparator`: Comparison operator to use
    /// - `target_value`: Value to compare against
    Property {
        /// Name of the property to evaluate.
        property_name: String,
        /// Comparison operator.
        comparator: PropertyComparator,
        /// Target value to compare against.
        target_value: f32,
    },
    /// Triggers when a ray intersects with the entity.
    ///
    /// Configuration:
    /// - `origin`: Ray origin point in world coordinates
    /// - `direction`: Ray direction vector (normalized)
    /// - `max_distance`: Maximum ray distance for detection
    Ray {
        /// Ray origin point [x, y, z].
        origin: [f32; 3],
        /// Ray direction vector [x, y, z] (should be normalized).
        direction: [f32; 3],
        /// Maximum detection distance.
        max_distance: f32,
    },
    /// Triggers after a specified duration in ticks.
    ///
    /// Configuration:
    /// - `duration_ticks`: Number of ticks to wait before triggering
    Timer {
        /// Number of ticks to wait before activating.
        duration_ticks: u32,
    },
    /// Triggers when a specific channel receives a message.
    ///
    /// Configuration:
    /// - `channel_id`: Channel identifier to listen on
    Channel {
        /// Channel identifier to listen for messages.
        channel_id: u32,
    },
}

impl Default for SensorComponent {
    fn default() -> Self {
        Self::MouseHover { distance: 100.0 }
    }
}

/// Comparison operators for property-based sensors.
///
/// Used by Property sensors to compare entity properties against target values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PropertyComparator {
    /// Equal to target value.
    Equal = 0,
    /// Not equal to target value.
    NotEqual = 1,
    /// Greater than target value.
    GreaterThan = 2,
    /// Less than target value.
    LessThan = 3,
    /// Greater than or equal to target value.
    GreaterThanOrEqual = 4,
    /// Less than or equal to target value.
    LessThanOrEqual = 5,
}

impl Default for PropertyComparator {
    fn default() -> Self {
        Self::Equal
    }
}

/// Types of click events that can be detected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClickType {
    /// Trigger on single click.
    Single = 0,
    /// Trigger on double-click (rapid succession).
    Double = 1,
    /// Trigger on sustained press.
    Long = 2,
}

impl Default for ClickType {
    fn default() -> Self {
        Self::Single
    }
}

/// Represents the type of logic controller to evaluate.
///
/// Controllers form the decision layer of BGE logic, combining sensor outputs
/// using boolean logic or state machines. They determine when actuators fire.
#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ControllerComponent {
    /// Passes through sensor output unchanged.
    ///
    /// Used for simple direct connections between sensor and actuator.
    Direct,

    /// Fires only when ALL conditions are true.
    ///
    /// Configuration:
    /// - `conditions`: List of nested controllers to combine
    And {
        /// List of controller conditions (maximum 8 for stack safety).
        conditions: Vec<ControllerComponent>,
    },

    /// Fires when ANY condition is true.
    ///
    /// Configuration:
    /// - `conditions`: List of nested controllers to combine
    Or {
        /// List of controller conditions (maximum 8 for stack safety).
        conditions: Vec<ControllerComponent>,
    },

    /// Inverts the result of the nested controller.
    ///
    /// Configuration:
    /// - `condition`: Controller to invert
    Not {
        /// Controller whose output to invert.
        condition: Box<ControllerComponent>,
    },

    /// Pulses output on each tick for a specified duration.
    ///
    /// Configuration:
    /// - `tick_count`: Number of ticks to pulse
    /// - `controller`: Underlying controller to trigger pulse
    Pulse {
        /// Number of ticks to remain active after trigger.
        tick_count: u32,
        /// Controller that triggers the pulse.
        controller: Box<ControllerComponent>,
    },

    /// Toggles state on each trigger.
    ///
    /// Configuration:
    /// - `controller`: Underlying controller that toggles state
    Toggle {
        /// Controller that triggers the toggle.
        controller: Box<ControllerComponent>,
    },

    /// Delays activation by specified number of ticks.
    ///
    /// Configuration:
    /// - `delay_ticks`: Number of ticks to delay
    /// - `controller`: Controller to delay
    Delay {
        /// Number of ticks to wait before passing through.
        delay_ticks: u32,
        /// Controller whose output to delay.
        controller: Box<ControllerComponent>,
    },

    /// Fires once when condition becomes true (edge-triggered).
    ///
    /// Configuration:
    /// - `controller`: Controller to monitor for rising edge
    OneShot {
        /// Controller to monitor.
        controller: Box<ControllerComponent>,
    },
}

impl Default for ControllerComponent {
    fn default() -> Self {
        Self::Direct
    }
}

/// Represents the type of actuator action to perform.
///
/// Actuators form the output layer of BGE logic, executing actions in response
/// to controller evaluations. Each variant represents a specific action type.
#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ActuatorComponent {
    /// Highlights the entity with a visual effect.
    ///
    /// Configuration:
    /// - `color`: Highlight color (RGB)
    /// - `pulse`: Whether to pulse the highlight
    Highlight {
        /// RGB color components (0.0 to 1.0).
        color: [f32; 3],
        /// Whether to pulse the highlight effect.
        pulse: bool,
    },

    /// Marks the entity as selected.
    ///
    /// Configuration:
    /// - `exclusive`: Whether to deselect other entities
    Select {
        /// If true, deselects other entities when this is selected.
        exclusive: bool,
    },

    /// Applies movement to the entity.
    ///
    /// Configuration:
    /// - `velocity`: Movement velocity vector
    /// - `local_space`: Apply in local or world space
    Move {
        /// Velocity vector (units per second).
        velocity: [f32; 3],
        /// If true, apply in entity's local coordinate system.
        local_space: bool,
    },

    /// Applies rotation to the entity.
    ///
    /// Configuration:
    /// - `rotation`: Rotation axis and angle
    /// - `local_space`: Apply in local or world space
    Rotate {
        /// Axis-angle rotation (axis vector, angle in radians).
        rotation: ([f32; 3], f32),
        /// If true, apply in entity's local coordinate system.
        local_space: bool,
    },

    /// Applies scale to the entity.
    ///
    /// Configuration:
    /// - `scale`: Scale factors for X, Y, Z axes
    Scale {
        /// Scale factors (1.0 = original size).
        scale: [f32; 3],
    },

    /// Plays a sound effect.
    ///
    /// Configuration:
    /// - `sound_id`: Identifier of the sound to play
    /// - `volume`: Playback volume (0.0 to 1.0)
    Sound {
        /// Sound resource identifier.
        sound_id: String,
        /// Playback volume (0.0 to 1.0).
        volume: f32,
    },

    /// Triggers an animation.
    ///
    /// Configuration:
    /// - `animation_id`: Animation clip to play
    /// - `loop`: Whether to loop the animation
    Animation {
        /// Animation clip identifier.
        animation_id: String,
        /// If true, loop the animation continuously.
        loop_animation: bool,
    },

    /// Custom actuator for user-defined behaviors.
    ///
    /// Configuration:
    /// - `action_type`: Custom action identifier
    /// - `params`: Action-specific parameters
    Custom {
        /// Custom action type identifier.
        action_type: String,
        /// Action-specific parameters as key-value pairs.
        params: Vec<(String, String)>,
    },
}

impl Default for ActuatorComponent {
    fn default() -> Self {
        Self::Highlight {
            color: [1.0, 1.0, 0.0],
            pulse: false,
        }
    }
}

/// Reference to a sensor component on another entity.
///
/// Used by controllers to reference sensors on different entities,
/// enabling complex multi-entity logic chains.
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct SensorRef {
    /// Entity ID that owns the referenced sensor.
    pub entity: crate::ecs::EntityId,
    /// Type of sensor to reference (for validation).
    #[doc(hidden)]
    pub sensor_type: SensorComponentType,
}

/// Discriminates sensor component variants for type-safe references.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SensorComponentType {
    MouseHover = 0,
    MouseClick = 1,
    Proximity = 2,
    KeyShortcut = 3,
    DoubleTap = 4,
    LongPress = 5,
    RightClick = 6,
    Always = 7,
    Property = 8,
    Ray = 9,
    Timer = 10,
    Channel = 11,
}

impl From<&SensorComponent> for SensorComponentType {
    fn from(sensor: &SensorComponent) -> Self {
        match sensor {
            SensorComponent::MouseHover { .. } => Self::MouseHover,
            SensorComponent::MouseClick { .. } => Self::MouseClick,
            SensorComponent::Proximity { .. } => Self::Proximity,
            SensorComponent::KeyShortcut { .. } => Self::KeyShortcut,
            SensorComponent::DoubleTap { .. } => Self::DoubleTap,
            SensorComponent::LongPress { .. } => Self::LongPress,
            SensorComponent::RightClick => Self::RightClick,
            SensorComponent::Always => Self::Always,
            SensorComponent::Property { .. } => Self::Property,
            SensorComponent::Ray { .. } => Self::Ray,
            SensorComponent::Timer { .. } => Self::Timer,
            SensorComponent::Channel { .. } => Self::Channel,
        }
    }
}

/// Configuration data shared across sensor types.
///
/// Provides common configuration fields that can be used across different
/// sensor variants for consistent behavior.
#[derive(Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct SensorConfig {
    /// Debounce time in milliseconds to prevent rapid re-triggering.
    pub debounce_ms: u32,
    /// Whether the sensor is initially active.
    pub initially_active: bool,
    /// Minimum time between triggers in milliseconds.
    pub min_interval_ms: u32,
}

impl SensorConfig {
    /// Creates a new default configuration.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the debounce time.
    #[inline]
    #[must_use]
    pub fn with_debounce_ms(mut self, debounce_ms: u32) -> Self {
        self.debounce_ms = debounce_ms;
        self
    }

    /// Sets whether the sensor starts active.
    #[inline]
    #[must_use]
    pub fn with_initially_active(mut self, active: bool) -> Self {
        self.initially_active = active;
        self
    }

    /// Sets the minimum interval between triggers.
    #[inline]
    #[must_use]
    pub fn with_min_interval_ms(mut self, interval_ms: u32) -> Self {
        self.min_interval_ms = interval_ms;
        self
    }
}

// ============================================================================
// Component Integration
// ============================================================================

impl Component for SensorComponent {
    type Storage = VecStorage<SensorComponent>;
}

impl Component for ControllerComponent {
    type Storage = VecStorage<ControllerComponent>;
}

impl Component for ActuatorComponent {
    type Storage = VecStorage<ActuatorComponent>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_component_default() {
        let sensor = SensorComponent::default();
        matches!(sensor, SensorComponent::MouseHover { distance: 100.0 });
    }

    #[test]
    fn test_click_type_defaults() {
        assert_eq!(ClickType::default(), ClickType::Single);
    }

    #[test]
    fn test_controller_component_default() {
        let controller = ControllerComponent::default();
        matches!(controller, ControllerComponent::Direct);
    }

    #[test]
    fn test_actuator_component_default() {
        let actuator = ActuatorComponent::default();
        matches!(
            actuator,
            ActuatorComponent::Highlight {
                color: [1.0, 1.0, 0.0],
                pulse: false
            }
        );
    }

    #[test]
    fn test_sensor_config_builder() {
        let config = SensorConfig::new()
            .with_debounce_ms(50)
            .with_initially_active(true)
            .with_min_interval_ms(100);

        assert_eq!(config.debounce_ms, 50);
        assert!(config.initially_active);
        assert_eq!(config.min_interval_ms, 100);
    }

    #[test]
    fn test_sensor_component_type_conversion() {
        let sensor = SensorComponent::MouseClick {
            button: 0,
            click_type: ClickType::Double,
        };
        let sensor_type = SensorComponentType::from(&sensor);
        assert_eq!(sensor_type, SensorComponentType::MouseClick);
    }

    #[test]
    fn test_controller_and_nesting() {
        let controller = ControllerComponent::And {
            conditions: vec![
                ControllerComponent::Direct,
                ControllerComponent::Not {
                    condition: Box::new(ControllerComponent::Direct),
                },
            ],
        };

        match &controller {
            ControllerComponent::And { conditions } => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected AND controller"),
        }
    }

    #[test]
    fn test_actuator_highlight_config() {
        let actuator = ActuatorComponent::Highlight {
            color: [0.0, 0.5, 1.0],
            pulse: true,
        };

        match actuator {
            ActuatorComponent::Highlight { color, pulse } => {
                assert_eq!(color, [0.0, 0.5, 1.0]);
                assert!(pulse);
            }
            _ => panic!("Expected Highlight actuator"),
        }
    }

    #[test]
    fn test_sensor_ref_creation() {
        let entity_id = crate::ecs::EntityId::from_usize(42);
        let sensor_type = SensorComponentType::MouseHover;

        let sensor_ref = SensorRef {
            entity: entity_id,
            sensor_type,
        };

        assert_eq!(sensor_ref.entity.index(), 42);
        assert_eq!(sensor_ref.sensor_type, SensorComponentType::MouseHover);
    }

    #[test]
    fn test_actuator_sound_config() {
        let sound_id = alloc::string::ToString::to_string("click.wav");
        let actuator = ActuatorComponent::Sound {
            sound_id,
            volume: 0.75,
        };

        match actuator {
            ActuatorComponent::Sound { sound_id, volume } => {
                assert_eq!(sound_id, "click.wav");
                assert!((volume - 0.75).abs() < 0.001);
            }
            _ => panic!("Expected Sound actuator"),
        }
    }
}
