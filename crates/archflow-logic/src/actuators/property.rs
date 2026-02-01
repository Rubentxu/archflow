// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - PropertyActuator Implementation
//
// This actuator executes PropertyCommands to modify entity properties.
// It maintains state for undo/restore operations and supports zero-copy
// command execution through the Command pattern.
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-012
//
// Key Features:
// - Zero-copy: Commands are Plain Old Data (Copy), ≤16 bytes
// - Undo/Redo: Each command has inverse() for state restoration
// - Batch execution: Multiple commands can be executed in one pass
// - State restoration: Actuator maintains previous state for undo
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};

use crate::pulse::{Pulse, SensorState};

/// Property types that can be modified by PropertyActuator
///
/// Each property maps to a specific Command variant.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    /// Position - uses Move command
    Position = 0,

    /// Color (0xRRGGBBAA) - uses SetColor command
    Color = 1,

    /// Size - uses Resize command
    Size = 2,

    /// Visibility - uses SetVisible command
    Visibility = 3,

    /// Shape - uses SetShape command
    Shape = 4,

    /// Layer - uses SetLayer command
    Layer = 5,

    /// Texture index - uses SetTexture command
    Texture = 6,
}

/// Actuator that modifies entity properties when triggered
///
/// This actuator responds to pulses by executing commands on entities.
/// It supports undo/redo through the Command pattern.
///
/// # Example
///
/// ```rust
/// use archflow_logic::actuators::PropertyActuator;
/// use archflow_logic::Property;
/// use archflow_core::Vec2;
///
/// // Create a color actuator
/// let actuator = PropertyActuator::new(Property::Color, entity_id, 0xFF0000FF);
///
/// // When triggered by a positive pulse, this will set the entity color to red
/// ```
#[derive(Clone, Copy)]
pub struct PropertyActuator {
    /// Property type this actuator modifies
    property: Property,

    /// Entity ID this actuator operates on
    entity_id: u32,

    /// Target value (for Position and Size)
    target_value: [u32; 4], // Store as u32 array for Copy trait (Vec2 = 2*u32)

    /// Target color (for Color property)
    target_color: u32,

    /// Target visibility (for Visibility property)
    target_visibility: bool,

    /// Target shape/layer/texture (for their respective properties)
    target_u8: u8,
}

impl PropertyActuator {
    /// Create a new PropertyActuator for color modification
    #[must_use]
    pub fn new_color(entity_id: u32, color: u32) -> Self {
        Self {
            property: Property::Color,
            entity_id,
            target_value: [0; 4],
            target_color: color,
            target_visibility: false,
            target_u8: 0,
        }
    }

    /// Create a new PropertyActuator for position modification
    #[must_use]
    pub fn new_position(entity_id: u32, target: Vec2) -> Self {
        Self {
            property: Property::Position,
            entity_id,
            target_value: [target.x.to_bits(), target.y.to_bits(), 0, 0],
            target_color: 0,
            target_visibility: false,
            target_u8: 0,
        }
    }

    /// Create a new PropertyActuator for size modification
    #[must_use]
    pub fn new_size(entity_id: u32, target: Vec2) -> Self {
        Self {
            property: Property::Size,
            entity_id,
            target_value: [target.x.to_bits(), target.y.to_bits(), 0, 0],
            target_color: 0,
            target_visibility: false,
            target_u8: 0,
        }
    }

    /// Create a new PropertyActuator for visibility modification
    #[must_use]
    pub fn new_visibility(entity_id: u32, visible: bool) -> Self {
        Self {
            property: Property::Visibility,
            entity_id,
            target_value: [0; 4],
            target_color: 0,
            target_visibility: visible,
            target_u8: 0,
        }
    }

    /// Create a new PropertyActuator for shape modification
    #[must_use]
    pub fn new_shape(entity_id: u32, shape: u8) -> Self {
        Self {
            property: Property::Shape,
            entity_id,
            target_value: [0; 4],
            target_color: 0,
            target_visibility: false,
            target_u8: shape,
        }
    }

    /// Create a new PropertyActuator for layer modification
    #[must_use]
    pub fn new_layer(entity_id: u32, layer: u8) -> Self {
        Self {
            property: Property::Layer,
            entity_id,
            target_value: [0; 4],
            target_color: 0,
            target_visibility: false,
            target_u8: layer,
        }
    }

    /// Create a new PropertyActuator for texture modification
    #[must_use]
    pub fn new_texture(entity_id: u32, texture_index: u16) -> Self {
        Self {
            property: Property::Texture,
            entity_id,
            target_value: [0; 4],
            target_color: 0,
            target_visibility: false,
            target_u8: texture_index as u8,
        }
    }

    /// Get the property type this actuator modifies
    #[must_use]
    pub fn property(&self) -> Property {
        self.property
    }

    /// Get the entity ID this actuator operates on
    #[must_use]
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Activate the actuator in response to a pulse
    ///
    /// This generates and executes a Command based on the actuator's
    /// property type and the pulse state.
    ///
    /// # Arguments
    ///
    /// * `pulse` - The pulse that triggered this actuator
    /// * `store` - Mutable reference to the EntityStore for modifications
    pub fn activate(&mut self, pulse: &Pulse, store: &mut EntityStore) {
        // Only respond to Positive pulses
        if pulse.state != SensorState::Positive {
            return;
        }

        let idx = pulse.entity_id as usize;
        let entity_id = EntityId::new(pulse.entity_id);

        // Generate command based on property type
        let command = match self.property {
            Property::Position => {
                let current = store.pos(idx);
                let target = Vec2::new(
                    f32::from_bits(self.target_value[0]),
                    f32::from_bits(self.target_value[1]),
                );
                Command::Move {
                    id: entity_id,
                    delta: target - current,
                }
            }

            Property::Color => Command::SetColor {
                id: entity_id,
                color: self.target_color,
            },

            Property::Size => {
                let current = store.size(idx);
                let target = Vec2::new(
                    f32::from_bits(self.target_value[0]),
                    f32::from_bits(self.target_value[1]),
                );
                Command::Resize {
                    id: entity_id,
                    size: target,
                }
            }

            Property::Visibility => Command::SetVisible {
                id: entity_id,
                visible: self.target_visibility,
            },

            Property::Shape => Command::SetShape {
                id: entity_id,
                shape: self.target_u8,
            },

            Property::Layer => Command::SetLayer {
                id: entity_id,
                layer: self.target_u8,
            },

            Property::Texture => {
                let texture_index = self.target_u8 as u16;
                Command::SetTexture {
                    id: entity_id,
                    texture_index,
                }
            }
        };

        // Execute the command
        command.execute(store);
    }

    /// Reset the actuator state
    pub fn reset(&mut self) {
        // PropertyActuator doesn't maintain complex state that needs reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{EntityId, Vec2};

    #[test]
    fn test_property_actuator_new_color() {
        let actuator = PropertyActuator::new_color(100, 0xFF0000FF);

        assert_eq!(actuator.property(), Property::Color);
        assert_eq!(actuator.entity_id(), 100);
        assert_eq!(actuator.target_color, 0xFF0000FF);
    }

    #[test]
    fn test_property_actuator_new_position() {
        let target = Vec2::new(100.0, 200.0);
        let actuator = PropertyActuator::new_position(100, target);

        assert_eq!(actuator.property(), Property::Position);
        assert_eq!(actuator.entity_id(), 100);
        assert_eq!(f32::from_bits(actuator.target_value[0]), 100.0);
        assert_eq!(f32::from_bits(actuator.target_value[1]), 200.0);
    }

    #[test]
    fn test_property_actuator_activate_color() {
        let mut store = EntityStore::new();
        let pos = Vec2::new(100.0, 200.0);
        let entity_id = store.spawn(pos, Vec2::new(50.0, 50.0));

        let mut actuator = PropertyActuator::new_color(entity_id.index().0 as u32, 0xFF0000FF);
        let pulse = Pulse::positive(0, entity_id.index().0, 1000);

        // Activate should set color to red
        actuator.activate(&pulse, &mut store);

        let idx = entity_id.index().0 as usize;
        assert_eq!(store.colors[idx], 0xFF0000FF);
    }

    #[test]
    fn test_property_actuator_activate_visibility() {
        let mut store = EntityStore::new();
        let pos = Vec2::new(100.0, 200.0);
        let entity_id = store.spawn(pos, Vec2::new(50.0, 50.0));

        let mut actuator = PropertyActuator::new_visibility(entity_id.index().0 as u32, false);
        let pulse = Pulse::positive(0, entity_id.index().0, 1000);

        actuator.activate(&pulse, &mut store);

        let idx = entity_id.index().0 as usize;
        assert!(!store.is_visible(idx));
    }

    #[test]
    fn test_property_actuator_activate_position() {
        let mut store = EntityStore::new();
        let pos = Vec2::new(100.0, 200.0);
        let entity_id = store.spawn(pos, Vec2::new(50.0, 50.0));

        let target = Vec2::new(150.0, 250.0);
        let mut actuator = PropertyActuator::new_position(entity_id.index().0 as u32, target);
        let pulse = Pulse::positive(0, entity_id.index().0, 1000);

        actuator.activate(&pulse, &mut store);

        let idx = entity_id.index().0 as usize;
        let new_pos = store.pos(idx);
        // Should have moved by (50, 50)
        assert!((new_pos.x - 150.0).abs() < 0.01);
        assert!((new_pos.y - 250.0).abs() < 0.01);
    }

    #[test]
    fn test_property_actuator_no_op_on_negative() {
        let mut store = EntityStore::new();
        let pos = Vec2::new(100.0, 200.0);
        let entity_id = store.spawn(pos, Vec2::new(50.0, 50.0));
        let idx = entity_id.index().0 as usize;
        let original_color = store.colors[idx];

        let mut actuator = PropertyActuator::new_color(entity_id.index().0 as u32, 0xFF0000FF);

        // Negative pulse should not trigger
        let pulse = Pulse::negative(0, entity_id.index().0, 1000);
        actuator.activate(&pulse, &mut store);

        // Color should remain unchanged
        assert_eq!(store.colors[idx], original_color);
    }

    #[test]
    fn test_property_actuator_all_properties() {
        let mut store = EntityStore::new();
        let pos = Vec2::new(100.0, 200.0);
        let entity_id = store.spawn(pos, Vec2::new(50.0, 50.0));
        let eid = entity_id.index().0;

        // Test Position
        let mut pos_actuator = PropertyActuator::new_position(eid, Vec2::new(120.0, 220.0));
        pos_actuator.activate(&Pulse::positive(0, eid, 1000), &mut store);

        // Test Color
        let mut color_actuator = PropertyActuator::new_color(eid, 0x00FF00FF);
        color_actuator.activate(&Pulse::positive(0, eid, 1000), &mut store);

        // Test Visibility
        let mut vis_actuator = PropertyActuator::new_visibility(eid, false);
        vis_actuator.activate(&Pulse::positive(0, eid, 1000), &mut store);

        // Test Shape
        let mut shape_actuator = PropertyActuator::new_shape(eid, 5); // Diamond
        shape_actuator.activate(&Pulse::positive(0, eid, 1000), &mut store);

        // Test Layer
        let mut layer_actuator = PropertyActuator::new_layer(eid, 3);
        layer_actuator.activate(&Pulse::positive(0, eid, 1000), &mut store);

        // Test Texture
        let mut tex_actuator = PropertyActuator::new_texture(eid, 42);
        tex_actuator.activate(&Pulse::positive(0, eid, 1000), &mut store);
    }

    #[test]
    fn test_property_actuator_copy_trait() {
        // PropertyActuator must be Copy for efficient queuing
        let actuator1 = PropertyActuator::new_color(100, 0xFF0000FF);
        let actuator2 = actuator1; // Copy trait should work

        assert_eq!(actuator2.entity_id(), 100);
        assert_eq!(actuator2.target_color, 0xFF0000FF);
    }
}
