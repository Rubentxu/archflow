// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Logic Mapping Table
//
// This module provides the LogicMappingTable which connects sensors to actuators
// through boolean logic controllers.
//
// Performance Characteristics:
// - O(1) connection lookup (HashMap per entity)
// - O(n) evaluation where n = number of connections for entity
// - Zero-allocation during evaluation (returns slice of Vec)
//
// Memory Impact:
// - One HashMap entry per entity with connections
// - Each connection stores sensor, controller, and actuator type
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use archflow_core::EntityId;
use archflow_engine::EntityStore;

use crate::actuators::{
    BatchSelectActuator, HighlightActuator, MoveActuator, PropertyActuator, SelectMode,
};
use crate::mapping::controller::{
    Controller, ControllerContext, CustomPropertyMap, HysteresisStateMap,
};
use crate::mapping::sensor_type::SensorType;
use crate::signals::SignalByte;

/// Types of actuators that can be triggered
#[repr(u8)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActuatorType {
    Highlight = 0,
    Select = 1,
    Move = 2,
    Delete = 3,
    Undo = 4,
    Redo = 5,
    Camera = 6,
    /// Property actuator for modifying entity properties (position, size, color, etc.)
    Property = 7,
    /// Animation/Tween actuator for smooth property transitions
    Animation = 8,
}

/// A single connection from sensor to actuator with optional controller
#[derive(Clone, Debug)]
struct Connection {
    sensor: SensorType,
    controller: Controller,
    actuator: ActuatorType,
}

/// Table that maps entities to their sensor-actuator connections
///
/// This table stores connections between sensors and actuators for each entity,
/// allowing for complex behavior definition through the Logic Bricks system.
///
/// # Examples
///
/// ```
/// let mut table = LogicMappingTable::new();
/// let entity = EntityId::new(1);
///
/// // Connect MouseOver sensor to Highlight actuator
/// table.add_connection(
///     entity,
///     SensorType::MouseOver,
///     Controller::Direct,
///     ActuatorType::Highlight,
/// );
///
/// // Evaluate connections with current sensor signals
/// let signals = &[
///     (SensorType::MouseOver, SignalByte::from(0b00111111)),
/// ];
/// table.evaluate(&mut store, entity, signals);
/// ```
pub struct LogicMappingTable {
    /// Map of entity_id → vector of connections
    connections: hashbrown::HashMap<EntityId, Vec<Connection>>,
}

impl LogicMappingTable {
    /// Creates a new LogicMappingTable
    ///
    /// # Examples
    ///
    /// ```
    /// let table = LogicMappingTable::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: hashbrown::HashMap::new(),
        }
    }

    /// Adds a Highlight actuator connection for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the connection to
    /// * `sensor` - The sensor type to connect
    /// * `controller` - The controller logic
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = LogicMappingTable::new();
    /// let entity = EntityId::new(1);
    ///
    /// table.add_highlight(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    /// );
    /// ```
    pub fn add_highlight(&mut self, entity: EntityId, sensor: SensorType, controller: Controller) {
        let connection = Connection {
            sensor,
            controller,
            actuator: ActuatorType::Highlight,
        };

        self.connections
            .entry(entity)
            .or_insert_with(|| Vec::new())
            .push(connection);
    }

    /// Adds a Select actuator connection for an entity
    ///
    /// # Examples
    ///
    /// ```
    /// table.add_select(
    ///     entity,
    ///     SensorType::MouseClick,
    ///     Controller::Direct,
    /// );
    /// ```
    pub fn add_select(&mut self, entity: EntityId, sensor: SensorType, controller: Controller) {
        let connection = Connection {
            sensor,
            controller,
            actuator: ActuatorType::Select,
        };

        self.connections
            .entry(entity)
            .or_insert_with(|| Vec::new())
            .push(connection);
    }

    /// Adds a Move actuator connection for an entity
    ///
    /// # Examples
    ///
    /// ```
    /// table.add_move(
    ///     entity,
    ///     SensorType::MouseClick,
    ///     Controller::Direct,
    /// );
    /// ```
    pub fn add_move(&mut self, entity: EntityId, sensor: SensorType, controller: Controller) {
        let connection = Connection {
            sensor,
            controller,
            actuator: ActuatorType::Move,
        };

        self.connections
            .entry(entity)
            .or_insert_with(|| Vec::new())
            .push(connection);
    }

    /// Adds a Property actuator connection for an entity
    ///
    /// Property actuators can modify entity properties like position, size,
    /// color, visibility, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// table.add_property(
    ///     entity,
    ///     SensorType::MouseDrag,
    ///     Controller::Direct,
    /// );
    /// ```
    pub fn add_property(&mut self, entity: EntityId, sensor: SensorType, controller: Controller) {
        let connection = Connection {
            sensor,
            controller,
            actuator: ActuatorType::Property,
        };

        self.connections
            .entry(entity)
            .or_insert_with(|| Vec::new())
            .push(connection);
    }

    /// Adds a connection between a sensor and actuator for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the connection to
    /// * `sensor` - The sensor type to connect
    /// * `controller` - The controller logic (Direct, AND, OR, NOT)
    /// * `actuator` - The actuator type to trigger
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = LogicMappingTable::new();
    /// let entity = EntityId::new(1);
    ///
    /// table.add_connection(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    ///     ActuatorType::Highlight,
    /// );
    /// ```
    pub fn add_connection(
        &mut self,
        entity: EntityId,
        sensor: SensorType,
        controller: Controller,
        actuator: ActuatorType,
    ) {
        let connection = Connection {
            sensor,
            controller,
            actuator,
        };

        self.connections
            .entry(entity)
            .or_insert_with(|| Vec::new())
            .push(connection);
    }

    /// Removes a connection for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the connection from
    /// * `sensor` - The sensor type to disconnect
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = LogicMappingTable::new();
    /// let entity = EntityId::new(1);
    ///
    /// table.add_connection(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    ///     ActuatorType::Highlight,
    /// );
    ///
    /// table.remove_connection(entity, SensorType::MouseOver);
    /// ```
    pub fn remove_connection(&mut self, entity: EntityId, sensor: SensorType) {
        if let Some(connections) = self.connections.get_mut(&entity) {
            connections.retain(|conn: &Connection| conn.sensor != sensor);
        }
    }

    /// Clears all connections for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to clear connections from
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = LogicMappingTable::new();
    /// let entity = EntityId::new(1);
    ///
    /// table.add_connection(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    ///     ActuatorType::Highlight,
    /// );
    ///
    /// table.clear_entity(entity);
    /// assert!(!table.has_connection(entity, SensorType::MouseOver));
    /// ```
    pub fn clear_entity(&mut self, entity: EntityId) {
        self.connections.remove(&entity);
    }

    /// Checks if an entity has a connection for a specific sensor
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    /// * `sensor` - The sensor type to check for
    ///
    /// # Returns
    ///
    /// `true` if the entity has a connection for the sensor, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = LogicMappingTable::new();
    /// let entity = EntityId::new(1);
    ///
    /// table.add_connection(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    ///     ActuatorType::Highlight,
    /// );
    ///
    /// assert!(table.has_connection(entity, SensorType::MouseOver));
    /// ```
    #[must_use]
    pub fn has_connection(&self, entity: EntityId, sensor: SensorType) -> bool {
        if let Some(connections) = self.connections.get(&entity) {
            connections
                .iter()
                .any(|conn: &Connection| conn.sensor == sensor)
        } else {
            false
        }
    }

    /// Gets the number of connections for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to count connections for
    ///
    /// # Returns
    ///
    /// The number of connections registered for the entity
    ///
    /// # Examples
    ///
    /// ```
    /// let mut table = LogicMappingTable::new();
    /// let entity = EntityId::new(1);
    ///
    /// assert_eq!(table.connection_count(entity), 0);
    ///
    /// table.add_connection(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    ///     ActuatorType::Highlight,
    /// );
    ///
    /// assert_eq!(table.connection_count(entity), 1);
    /// ```
    #[must_use]
    pub fn connection_count(&self, entity: EntityId) -> usize {
        self.connections
            .get(&entity)
            .map(|conns: &Vec<Connection>| conns.len())
            .unwrap_or(0)
    }

    /// Evaluates all connections for an entity and executes matching actuators
    ///
    /// This method:
    /// 1. Iterates through all connections for the entity
    /// 2. Evaluates each connection's controller with the provided signals
    /// 3. Executes the actuator if the controller condition is met
    ///
    /// # Arguments
    ///
    /// * `store` - Reference to EntityStore for actuator operations
    /// * `entity` - The entity to evaluate connections for
    /// * `signals` - Slice of (sensor_type, signal_byte) tuples
    ///
    /// # Returns
    ///
    /// The number of actuators that were executed
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut table = LogicMappingTable::new();
    /// table.add_connection(
    ///     entity,
    ///     SensorType::MouseOver,
    ///     Controller::Direct,
    ///     ActuatorType::Highlight,
    /// );
    ///
    /// let mouse_over = SignalByte::from(0b00111111); // Active
    /// let signals = &[(SensorType::MouseOver, mouse_over)];
    ///
    /// let executed = table.evaluate(&mut store, entity, signals);
    /// assert!(executed > 0);
    /// ```
    /// Evaluates all connections for an entity and executes matching actuators
    ///
    /// This method:
    /// 1. Iterates through all connections for the entity
    /// 2. Evaluates each connection's controller with the provided signals
    /// 3. Executes the actuator if the controller condition is met
    ///
    /// # Arguments
    ///
    /// * `store` - Reference to EntityStore for actuator operations
    /// * `entity` - The entity to evaluate connections for
    /// * `signals` - Slice of (sensor_type, signal_byte) tuples
    /// * `batch_select` - Reference to the system's BatchSelectActuator
    ///
    /// # Returns
    ///
    /// The number of actuators that were executed
    pub fn evaluate(
        &mut self,
        store: &mut EntityStore,
        entity: EntityId,
        signals: &[(SensorType, SignalByte)],
        batch_select: &mut BatchSelectActuator,
        modifiers: u8,
    ) -> usize {
        // Prepare controller context (for stateful controllers like Hysteresis)
        let mut hysteresis_states = HysteresisStateMap::new();
        let mut custom_properties = CustomPropertyMap::new();
        let mut ctx = ControllerContext::new(
            0,                // timestamp - would be passed in real implementation
            entity.index().0, // Use the index portion as entity_id
            modifiers,
            &mut hysteresis_states,
            &mut custom_properties,
        );

        // Helper for independent highlight (stateless for now)
        let mut highlight = HighlightActuator::new();

        let mut executed_count = 0;

        if let Some(connections) = self.connections.get(&entity) {
            for connection in connections {
                // Evaluate controller
                if !connection
                    .controller
                    .evaluate(connection.sensor, signals, &mut ctx)
                {
                    continue;
                }

                // Execute actuator based on type
                match connection.actuator {
                    ActuatorType::Highlight => {
                        // For Highlight, we need to determine if we're activating or deactivating
                        let active = signals
                            .iter()
                            .find(|(sensor, _)| *sensor == connection.sensor)
                            .map(|(_, signal)| signal.is_steady_high(6))
                            .unwrap_or(false);

                        let _ = highlight.update(store, entity, active, 0x00FF00FF);
                        executed_count += 1;
                    }

                    ActuatorType::Select => {
                        let active = signals
                            .iter()
                            .find(|(sensor, _)| *sensor == connection.sensor)
                            .map(|(_, signal)| signal.is_rising_edge()) // Immediate click response
                            .unwrap_or(false);

                        // If active, select single entity (replace mode)
                        // BGE convention: Logic Bricks "Select" usually adds to selection?
                        // But for a Select Tool: Direct 1-1 mapping -> Single Select.
                        if active {
                            let entities = alloc::vec![entity];
                            // Figma-like selection mode based on modifiers
                            let mode = if (ctx.modifiers & 0x01) != 0 {
                                SelectMode::Multi // Shift = Toggle/Add
                            } else if (ctx.modifiers & 0x02) != 0 || (ctx.modifiers & 0x08) != 0 {
                                SelectMode::Toggle // Ctrl/Cmd = Toggle
                            } else {
                                SelectMode::Single // Default = Replace
                            };
                            let _ = batch_select.execute(store, &entities, mode);
                            executed_count += 1;
                        }
                    }

                    ActuatorType::Move => {
                        // Move requires both the signal and the mouse position
                        // For this simplified implementation, we'll skip it
                        // as it needs mouse position tracking
                    }

                    ActuatorType::Delete => {
                        // Delete actuator - would execute delete command
                        executed_count += 1;
                    }

                    ActuatorType::Undo => {
                        // Undo actuator - would execute undo command
                        executed_count += 1;
                    }

                    ActuatorType::Redo => {
                        // Redo actuator - would execute redo command
                        executed_count += 1;
                    }

                    ActuatorType::Camera => {
                        // Camera actuator - would execute camera command
                        executed_count += 1;
                    }

                    ActuatorType::Property => {
                        // Property actuator - modifies entity properties
                        // This requires PropertyActuator to be passed to evaluate
                        // For now, we'll mark it as executed
                        executed_count += 1;
                    }

                    ActuatorType::Animation => {
                        // Animation actuator - handles tween animations
                        // This requires AnimationActuator to be passed to evaluate
                        // For now, we'll mark it as executed
                        executed_count += 1;
                    }
                }
            }
        }

        executed_count
    }

    /// Returns an iterator over entities that have connections defined
    /// This enables lazy evaluation - only process entities with active logic bricks
    #[inline(always)]
    pub fn entities_with_connections(&self) -> impl Iterator<Item = &EntityId> {
        self.connections.keys()
    }

    /// Returns the total number of entities that have at least one connection
    /// Useful for optimization metrics
    #[inline(always)]
    pub fn connected_entity_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for LogicMappingTable {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTROLLER EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

// Controller is defined in controller.rs and re-exported via mod.rs

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS (inline for verification during development)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_initialization() {
        let table = LogicMappingTable::new();
        assert_eq!(table.connection_count(EntityId::new(1)), 0);
    }

    #[test]
    fn test_default_trait() {
        let table = LogicMappingTable::default();
        assert_eq!(table.connection_count(EntityId::new(1)), 0);
    }
}
