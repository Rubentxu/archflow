// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow SDK - Public Actuator API
//
// This module defines the public Actuator trait that SDK developers use
// to create custom actuators.
//
// Reference: docs/epics/EPIC-SDK-PUBLIC-API.md - Section "API de Actuadores"
// ═══════════════════════════════════════════════════════════════════════════════

use crate::sensors::SensorState;
use archflow_core::EntityId;

/// Pulse emitted by a sensor when triggered
///
/// A pulse carries information about which sensor fired,
/// which entity it applies to, the resulting state, and when.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pulse {
    /// ID of the sensor that emitted this pulse
    pub sensor_id: u32,

    /// Entity this pulse applies to
    pub entity_id: EntityId,

    /// Sensor state that triggered this pulse
    pub state: SensorState,

    /// Timestamp when pulse was emitted (engine ticks)
    pub timestamp: u64,
}

/// Configuration for an actuator
///
/// This struct contains metadata about an actuator that's useful
/// for debugging, UI display, and serialization.
#[derive(Clone, Debug)]
pub struct ActuatorConfig {
    /// Human-readable name for this actuator
    pub name: String,

    /// Actuator type identifier
    pub actuator_type: ActuatorType,

    /// Whether this actuator is enabled
    pub enabled: bool,
}

/// Types of actuators supported by the SDK
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActuatorType {
    /// Modifies entity properties (position, color, etc.)
    Property,

    /// Controls visibility (show/hide entities)
    Visibility,

    /// Plays animations
    Animation,

    /// Sends messages to other entities
    Message,

    /// Camera control (pan, zoom, follow)
    Camera,

    /// Custom actuator type
    Custom(u8),
}

/// Trait that all custom actuators must implement
///
/// This is the PRIMARY way SDK developers extend the action system.
/// Actuators respond to pulses from sensors by executing actions.
///
/// # Example
///
/// ```rust
/// use archflow_sdk::actuators::{Actuator, ActuatorConfig, ActuatorType, Pulse};
/// use archflow_sdk::sensors::SensorState;
///
/// struct HighlightActuator {
///     highlight_color: u32, // 0xRRGGBBAA
///     normal_color: u32,
///     config: ActuatorConfig,
/// }
///
/// impl Actuator for HighlightActuator {
///     fn activate(&mut self, pulse: &Pulse, store: &mut archflow_engine::EntityStore) {
///         let idx = pulse.entity_id.index().0 as usize;
///         if pulse.state == SensorState::Positive {
///             // Highlight the entity when sensor is positive
///             store.set_color(idx, self.highlight_color);
///         } else {
///             // Reset to normal color when sensor is negative
///             store.set_color(idx, self.normal_color);
///         }
///     }
///
///     fn config(&self) -> &ActuatorConfig {
///         &self.config
///     }
/// }
/// ```
pub trait Actuator {
    /// Activate the actuator in response to a pulse
    ///
    /// This method is called when a sensor emits a pulse that's connected
    /// to this actuator. The actuator should execute its action based on
    /// the pulse state and entity.
    ///
    /// # Arguments
    ///
    /// * `pulse` - The pulse that triggered this actuator
    /// * `store` - Mutable reference to the EntityStore for modifications
    ///
    /// # Performance Notes
    ///
    /// - This is a HOT PATH - called whenever a connected sensor fires
    /// - Avoid allocations (use stack or pre-allocated buffers)
    /// - Batch multiple entity updates if possible
    /// - Consider using `#[inline]` for simple implementations
    ///
    /// # Thread Safety
    ///
    /// This method may be called from multiple threads if the SDK
    /// uses parallel system execution. Ensure thread-safety if needed.
    fn activate(&mut self, pulse: &Pulse, store: &mut archflow_engine::EntityStore);

    /// Return the actuator's configuration
    ///
    /// This provides metadata about the actuator for debugging and UI.
    fn config(&self) -> &ActuatorConfig;

    /// Optional: Reset actuator state
    ///
    /// Called when the actuator should reset its internal state.
    /// Default implementation does nothing.
    #[allow(unused_variables)]
    fn reset(&mut self) {
        // Default: no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;
    use archflow_engine::EntityStore;

    struct TestActuator {
        config: ActuatorConfig,
        activated_count: usize,
    }

    impl Actuator for TestActuator {
        fn activate(&mut self, pulse: &Pulse, store: &mut EntityStore) {
            self.activated_count += 1;
            // Simple test: move entity by (1, 1)
            // EntityStore methods use usize index, not EntityId directly
            let idx = pulse.entity_id.index().0 as usize;
            let current_pos = store.pos(idx);
            store.set_pos(idx, current_pos + Vec2::new(1.0, 1.0));
        }

        fn config(&self) -> &ActuatorConfig {
            &self.config
        }
    }

    #[test]
    fn test_actuator_activate() {
        let mut store = EntityStore::new();
        let entity_id = store.spawn(Vec2::new(10.0, 20.0), Vec2::new(50.0, 50.0));

        let mut actuator = TestActuator {
            config: ActuatorConfig {
                name: "Test Actuator".to_string(),
                actuator_type: ActuatorType::Property,
                enabled: true,
            },
            activated_count: 0,
        };

        let pulse = Pulse {
            sensor_id: 0,
            entity_id,
            state: SensorState::Positive,
            timestamp: 0,
        };

        let idx = entity_id.index().0 as usize;
        let initial_pos = store.pos(idx);
        actuator.activate(&pulse, &mut store);
        let final_pos = store.pos(idx);

        assert_eq!(actuator.activated_count, 1);
        assert_eq!(final_pos.x, initial_pos.x + 1.0);
        assert_eq!(final_pos.y, initial_pos.y + 1.0);
    }

    #[test]
    fn test_actuator_config() {
        let config = ActuatorConfig {
            name: "Test Actuator".to_string(),
            actuator_type: ActuatorType::Property,
            enabled: true,
        };

        assert_eq!(config.name, "Test Actuator");
        assert!(config.enabled);
        assert_eq!(config.actuator_type, ActuatorType::Property);
    }
}
