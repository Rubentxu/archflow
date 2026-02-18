// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - Brick Chain Builder
//
// Epic: EPIC-LOGIC-002 - API Fluida y Conectividad
// Implements fluent builder pattern for creating Logic Bricks chains in JavaScript
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

use super::{
    ActuatorType, BrickHandle, Controller, LogicMappingTableWasm, SelectModeWasm, SensorType,
};

/// Brick Chain Builder - API Fluida implementada en Rust/WASM
///
/// # JavaScript Example
/// ```javascript
/// const handle = bridge
///   .sensor(Sensor.Mouse.Click('Left'))
///   .controller(Controller.And())
///   .actuator(Actuator.Select.Single())
///   .connect();
/// ```
#[wasm_bindgen]
pub struct BrickChainBuilder {
    /// Entity ID this builder is creating logic for
    entity_id: u32,
    /// Sensors to chain
    sensors: Vec<SensorType>,
    /// Controllers to filter signals
    controllers: Vec<Controller>,
    /// Actuators to trigger
    actuators: Vec<(u8, u8)>,
    /// Optional mapping table
    mapping_table: Option<LogicMappingTableWasm>,
    /// Optional name for this logic chain (for debugging)
    name: Option<String>,
    /// Track the last node type added (for implicit controller)
    last_node_type: Option<NodeType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeType {
    Sensor,
    Controller,
    Actuator,
}

#[wasm_bindgen]
impl BrickChainBuilder {
    /// Creates a new BrickChainBuilder for an entity
    #[wasm_bindgen(constructor)]
    pub fn new(entity_id: u32) -> Self {
        Self {
            entity_id,
            sensors: Vec::new(),
            controllers: Vec::new(),
            actuators: Vec::new(),
            mapping_table: None,
            name: None,
            last_node_type: None,
        }
    }

    /// Creates a new BrickChainBuilder with a mapping table
    #[wasm_bindgen]
    pub fn with_mapping_table(entity_id: u32, mapping_table: LogicMappingTableWasm) -> Self {
        Self {
            entity_id,
            sensors: Vec::new(),
            controllers: Vec::new(),
            actuators: Vec::new(),
            mapping_table: Some(mapping_table),
            name: None,
            last_node_type: None,
        }
    }

    /// Add a sensor to the brick chain
    #[wasm_bindgen]
    pub fn sensor(mut self, sensor: SensorType) -> Self {
        self.sensors.push(sensor);
        self.last_node_type = Some(NodeType::Sensor);
        self
    }

    /// Add a keyboard key sensor (convenience)
    #[wasm_bindgen]
    pub fn sensor_key(mut self, _key_code: u32) -> Self {
        // Would create KeyShortcut sensor
        self.sensors.push(SensorType::KeyShortcut);
        self.last_node_type = Some(NodeType::Sensor);
        self
    }

    /// Add a controller to the brick chain
    #[wasm_bindgen]
    pub fn controller(mut self, controller: Controller) -> Self {
        self.controllers.push(controller);
        self.last_node_type = Some(NodeType::Controller);
        self
    }

    /// Add an AND controller (shortcut)
    #[wasm_bindgen]
    pub fn and(mut self) -> Self {
        let sensor = self
            .sensors
            .last()
            .copied()
            .unwrap_or(SensorType::MouseOver);
        self.controllers.push(Controller::and(sensor));
        self.last_node_type = Some(NodeType::Controller);
        self
    }

    /// Add an OR controller (shortcut)
    #[wasm_bindgen]
    pub fn or(mut self) -> Self {
        let sensor = self
            .sensors
            .last()
            .copied()
            .unwrap_or(SensorType::MouseOver);
        self.controllers.push(Controller::or(sensor));
        self.last_node_type = Some(NodeType::Controller);
        self
    }

    /// Add a NOT controller (invert signal)
    #[wasm_bindgen]
    pub fn invert(mut self) -> Self {
        self.controllers.push(Controller::not());
        self.last_node_type = Some(NodeType::Controller);
        self
    }

    /// Start a new chain (clears current sensors/controllers, keeps entity_id)
    #[wasm_bindgen]
    pub fn new_chain(mut self) -> Self {
        self.sensors.clear();
        self.controllers.clear();
        self.actuators.clear();
        self.last_node_type = None;
        self
    }

    /// Set the name for this logic block (for debugging)
    #[wasm_bindgen]
    pub fn logic_bricks(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Get the name of this logic block
    #[wasm_bindgen]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Add a Select actuator
    ///
    /// # Arguments
    /// * `mode` - 0=Single, 1=Multi, 2=Replace
    #[wasm_bindgen]
    pub fn actuator_select(mut self, mode: u8) -> Self {
        // Implicit Controller: If the last node added was a sensor (no controller),
        // automatically inject an AND controller
        if self.last_node_type == Some(NodeType::Sensor) && self.controllers.is_empty() {
            let sensor = self
                .sensors
                .last()
                .copied()
                .unwrap_or(SensorType::MouseOver);
            self.controllers.push(Controller::and(sensor));
        }
        self.actuators.push((1, mode)); // 1 = Select actuator type
        self.last_node_type = Some(NodeType::Actuator);
        self
    }

    /// Add a Highlight actuator
    ///
    /// # Arguments
    /// * `color_argb` - Color in ARGB format
    /// * `opacity` - Opacity from 0.0 to 1.0
    #[wasm_bindgen]
    pub fn actuator_highlight(mut self, color_argb: u32, opacity: f32) -> Self {
        // Implicit Controller: If the last node added was a sensor (no controller),
        // automatically inject an AND controller
        if self.last_node_type == Some(NodeType::Sensor) && self.controllers.is_empty() {
            let sensor = self
                .sensors
                .last()
                .copied()
                .unwrap_or(SensorType::MouseOver);
            self.controllers.push(Controller::and(sensor));
        }
        let _ = color_argb;
        let _ = opacity;
        self.actuators.push((0, 0)); // 0 = Highlight actuator type
        self.last_node_type = Some(NodeType::Actuator);
        self
    }

    /// Add a Move actuator
    ///
    /// # Arguments
    /// * `mode` - Movement mode
    /// * `x` - X velocity or position
    /// * `y` - Y velocity or position
    #[wasm_bindgen]
    pub fn actuator_move(mut self, mode: u8, x: f32, y: f32) -> Self {
        // Implicit Controller: If the last node added was a sensor (no controller),
        // automatically inject an AND controller
        if self.last_node_type == Some(NodeType::Sensor) && self.controllers.is_empty() {
            let sensor = self
                .sensors
                .last()
                .copied()
                .unwrap_or(SensorType::MouseOver);
            self.controllers.push(Controller::and(sensor));
        }
        let _ = mode;
        let _ = x;
        let _ = y;
        self.actuators.push((2, mode)); // 2 = Move actuator type
        self.last_node_type = Some(NodeType::Actuator);
        self
    }

    /// Connect the chain to the entity
    #[wasm_bindgen]
    pub fn connect(self) -> BrickHandle {
        // For now, return a handle (actual connection logic would be in the bridge)
        BrickHandle::new(self.entity_id.to_string())
    }

    /// Get the entity ID
    #[wasm_bindgen]
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Get the number of sensors added
    #[wasm_bindgen]
    pub fn sensor_count(&self) -> u32 {
        self.sensors.len() as u32
    }

    /// Get the number of controllers added
    #[wasm_bindgen]
    pub fn controller_count(&self) -> u32 {
        self.controllers.len() as u32
    }

    /// Get the number of actuators added
    #[wasm_bindgen]
    pub fn actuator_count(&self) -> u32 {
        self.actuators.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = BrickChainBuilder::new(42);
        assert_eq!(builder.entity_id(), 42);
        assert_eq!(builder.sensor_count(), 0);
    }

    #[test]
    fn test_builder_sensor_chaining() {
        let builder = BrickChainBuilder::new(1)
            .sensor(SensorType::MouseOver)
            .sensor(SensorType::MouseClick);
        assert_eq!(builder.sensor_count(), 2);
    }

    #[test]
    fn test_builder_controller_chaining() {
        let builder = BrickChainBuilder::new(1)
            .sensor(SensorType::MouseOver)
            .and()
            .sensor(SensorType::MouseClick)
            .or();
        assert_eq!(builder.sensor_count(), 2);
        assert_eq!(builder.controller_count(), 2);
    }

    #[test]
    fn test_implicit_controller() {
        // When adding actuator directly after sensor, implicit controller should be added
        let builder = BrickChainBuilder::new(1)
            .sensor(SensorType::MouseOver)
            .actuator_select(0);

        // Should have 1 sensor and 1 controller (implicit AND)
        assert_eq!(builder.sensor_count(), 1);
        assert_eq!(builder.controller_count(), 1);
    }

    #[test]
    fn test_logic_bricks_naming() {
        let builder = BrickChainBuilder::new(1)
            .logic_bricks("Movement")
            .sensor(SensorType::KeyShortcut)
            .actuator_move(0, 1.0, 0.0);

        assert_eq!(builder.name(), Some("Movement".to_string()));
    }

    #[test]
    fn test_new_chain() {
        let builder = BrickChainBuilder::new(1)
            .sensor(SensorType::MouseOver)
            .actuator_select(0)
            .new_chain()
            .sensor(SensorType::MouseClick);

        // After new_chain, only the second sensor should remain
        assert_eq!(builder.sensor_count(), 1);
        assert_eq!(builder.actuator_count(), 0);
    }
}
