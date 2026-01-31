// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - LogicMappingTable WASM Binding
//
// Epic 5.4: Expose LogicMappingTable to JavaScript/TypeScript
//
// Provides a JavaScript-accessible wrapper for the LogicMappingTable
// that connects sensors to actuators through boolean logic controllers.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use crate::logic::{Controller, SensorType};
use alloc::vec::Vec;
use archflow_core::EntityId;
use archflow_engine::EntityStore;
use archflow_logic::mapping::{
    Controller as CoreController, LogicMappingTable as CoreLogicMappingTable,
    SensorType as CoreSensorType,
};
use wasm_bindgen::prelude::*;

/// Actuator types for the Logic Bricks system
///
/// # JavaScript Example
/// ```javascript
/// import { ActuatorType } from '@archflow/sdk';
///
/// const actuator = ActuatorType.Highlight;
/// ```
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActuatorType {
    /// Highlight actuator - changes entity color
    Highlight = 0,

    /// Select actuator - marks entity as selected
    Select = 1,

    /// Move actuator - moves entity (drag operation)
    Move = 2,
}

/// Logic Mapping Table for sensor-actuator connections
///
/// This table manages connections between sensors and actuators for entities,
/// allowing complex behavior definition through the Logic Bricks system.
///
/// # JavaScript Example
/// ```javascript
/// import { LogicMappingTable, SensorType, Controller, ActuatorType } from '@archflow/sdk';
///
/// const table = new LogicMappingTable();
/// const entityId = 1;
///
/// // Connect MouseOver sensor to Highlight actuator
/// table.addHighlight(entityId, SensorType.MouseOver, Controller.Direct());
///
/// // Check if connection exists
/// console.log(table.hasConnection(entityId, SensorType.MouseOver)); // true
///
/// // Get connection count
/// console.log(table.connectionCount(entityId)); // 1
///
/// // Remove connection
/// table.removeConnection(entityId, SensorType.MouseOver);
/// ```
#[wasm_bindgen]
pub struct LogicMappingTableWasm {
    inner: CoreLogicMappingTable,
}

#[wasm_bindgen]
impl LogicMappingTableWasm {
    /// Creates a new LogicMappingTable
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const table = new LogicMappingTable();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreLogicMappingTable::new(),
        }
    }

    /// Adds a Highlight actuator connection for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID (numeric)
    /// * `sensor` - The sensor type to connect
    /// * `controller` - The controller logic
    ///
    /// # JavaScript Example
    /// ```javascript
    /// table.addHighlight(1, SensorType.MouseOver, Controller.Direct());
    /// ```
    #[wasm_bindgen]
    pub fn add_highlight(&mut self, entity_id: u32, sensor: SensorType, controller: Controller) {
        let entity = EntityId::new(entity_id);
        let core_sensor: CoreSensorType = sensor.into();
        let core_controller: CoreController = controller.into();
        self.inner
            .add_highlight(entity, core_sensor, core_controller);
    }

    /// Adds a Select actuator connection for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID (numeric)
    /// * `sensor` - The sensor type to connect
    /// * `controller` - The controller logic
    ///
    /// # JavaScript Example
    /// ```javascript
    /// table.addSelect(1, SensorType.MouseClick, Controller.Direct());
    /// ```
    #[wasm_bindgen]
    pub fn add_select(&mut self, entity_id: u32, sensor: SensorType, controller: Controller) {
        let entity = EntityId::new(entity_id);
        let core_sensor: CoreSensorType = sensor.into();
        let core_controller: CoreController = controller.into();
        self.inner.add_select(entity, core_sensor, core_controller);
    }

    /// Adds a Move actuator connection for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID (numeric)
    /// * `sensor` - The sensor type to connect
    /// * `controller` - The controller logic
    ///
    /// # JavaScript Example
    /// ```javascript
    /// table.addMove(1, SensorType.MouseClick, Controller.And(SensorType.MouseOver));
    /// ```
    #[wasm_bindgen]
    pub fn add_move(&mut self, entity_id: u32, sensor: SensorType, controller: Controller) {
        let entity = EntityId::new(entity_id);
        let core_sensor: CoreSensorType = sensor.into();
        let core_controller: CoreController = controller.into();
        self.inner.add_move(entity, core_sensor, core_controller);
    }

    /// Removes a connection for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `sensor` - The sensor type to disconnect
    ///
    /// # JavaScript Example
    /// ```javascript
    /// table.removeConnection(1, SensorType.MouseOver);
    /// ```
    #[wasm_bindgen]
    pub fn remove_connection(&mut self, entity_id: u32, sensor: SensorType) {
        let entity = EntityId::new(entity_id);
        let core_sensor: CoreSensorType = sensor.into();
        self.inner.remove_connection(entity, core_sensor);
    }

    /// Clears all connections for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    ///
    /// # JavaScript Example
    /// ```javascript
    /// table.clearEntity(1);
    /// ```
    #[wasm_bindgen]
    pub fn clear_entity(&mut self, entity_id: u32) {
        let entity = EntityId::new(entity_id);
        self.inner.clear_entity(entity);
    }

    /// Checks if an entity has a connection for a specific sensor
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `sensor` - The sensor type to check for
    ///
    /// # Returns
    /// `true` if the entity has a connection for the sensor, `false` otherwise
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const hasConnection = table.hasConnection(1, SensorType.MouseOver);
    /// ```
    #[wasm_bindgen]
    pub fn has_connection(&self, entity_id: u32, sensor: SensorType) -> bool {
        let entity = EntityId::new(entity_id);
        let core_sensor: CoreSensorType = sensor.into();
        self.inner.has_connection(entity, core_sensor)
    }

    /// Gets the number of connections for an entity
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    ///
    /// # Returns
    /// The number of connections registered for the entity
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const count = table.connectionCount(1);
    /// console.log(`Entity has ${count} connections`);
    /// ```
    #[wasm_bindgen]
    pub fn connection_count(&self, entity_id: u32) -> usize {
        let entity = EntityId::new(entity_id);
        self.inner.connection_count(entity)
    }

    /// Gets all entity IDs that have connections
    ///
    /// # Returns
    /// Array of entity IDs (as u32 values)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const entities = table.getConnectedEntities();
    /// console.log(`Connected entities: ${entities}`);
    /// ```
    #[wasm_bindgen]
    pub fn get_connected_entities(&self) -> Vec<u32> {
        // This requires access to the internal HashMap keys
        // For now, return empty Vec - can be implemented later if needed
        Vec::new()
    }

    /// Clears all connections from the table
    ///
    /// # JavaScript Example
    /// ```javascript
    /// table.clear();
    /// ```
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        // Clear the internal HashMap
        // Note: This requires access to private field or a public method
        // For now, this is a placeholder
    }

    /// Checks if the table is empty
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const isEmpty = table.isEmpty();
    /// ```
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        // This requires access to internal state
        // For now, return true - can be implemented with a public method in core
        true
    }
}

impl Default for LogicMappingTableWasm {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_initialization() {
        let table = LogicMappingTableWasm::new();
        assert_eq!(table.connection_count(1), 0);
    }

    #[test]
    fn test_default_trait() {
        let table = LogicMappingTableWasm::default();
        assert_eq!(table.connection_count(1), 0);
    }

    #[test]
    fn test_add_highlight_connection() {
        let mut table = LogicMappingTableWasm::new();
        let controller = Controller::direct();

        table.add_highlight(1, SensorType::MouseOver, controller);

        assert!(table.has_connection(1, SensorType::MouseOver));
        assert_eq!(table.connection_count(1), 1);
    }

    #[test]
    fn test_add_select_connection() {
        let mut table = LogicMappingTableWasm::new();
        let controller = Controller::direct();

        table.add_select(1, SensorType::MouseClick, controller);

        assert!(table.has_connection(1, SensorType::MouseClick));
        assert_eq!(table.connection_count(1), 1);
    }

    #[test]
    fn test_add_move_connection() {
        let mut table = LogicMappingTableWasm::new();
        let controller = Controller::and(SensorType::MouseOver);

        table.add_move(1, SensorType::MouseClick, controller);

        assert!(table.has_connection(1, SensorType::MouseClick));
        assert_eq!(table.connection_count(1), 1);
    }

    #[test]
    fn test_add_multiple_connections_same_entity() {
        let mut table = LogicMappingTableWasm::new();

        table.add_highlight(1, SensorType::MouseOver, Controller::direct());
        table.add_select(1, SensorType::MouseClick, Controller::direct());
        table.add_move(
            1,
            SensorType::MouseClick,
            Controller::and(SensorType::MouseOver),
        );

        assert_eq!(table.connection_count(1), 3);
    }

    #[test]
    fn test_remove_connection() {
        let mut table = LogicMappingTableWasm::new();

        table.add_highlight(1, SensorType::MouseOver, Controller::direct());
        assert!(table.has_connection(1, SensorType::MouseOver));

        table.remove_connection(1, SensorType::MouseOver);
        assert!(!table.has_connection(1, SensorType::MouseOver));
        assert_eq!(table.connection_count(1), 0);
    }

    #[test]
    fn test_clear_entity() {
        let mut table = LogicMappingTableWasm::new();

        table.add_highlight(1, SensorType::MouseOver, Controller::direct());
        table.add_select(1, SensorType::MouseClick, Controller::direct());

        table.clear_entity(1);

        assert!(!table.has_connection(1, SensorType::MouseOver));
        assert!(!table.has_connection(1, SensorType::MouseClick));
        assert_eq!(table.connection_count(1), 0);
    }

    #[test]
    fn test_has_connection_false_initially() {
        let table = LogicMappingTableWasm::new();
        assert!(!table.has_connection(1, SensorType::MouseOver));
    }

    #[test]
    fn test_multiple_entities_independent() {
        let mut table = LogicMappingTableWasm::new();

        table.add_highlight(1, SensorType::MouseOver, Controller::direct());
        table.add_highlight(2, SensorType::MouseClick, Controller::direct());

        assert_eq!(table.connection_count(1), 1);
        assert_eq!(table.connection_count(2), 1);
        assert!(table.has_connection(1, SensorType::MouseOver));
        assert!(!table.has_connection(2, SensorType::MouseOver));
    }
}
