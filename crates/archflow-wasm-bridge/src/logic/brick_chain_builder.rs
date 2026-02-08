// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - BrickChainBuilder WASM Binding
//
// Epic 5.x: Fluent Bricks API - API fluida para declarar brick chains
//
// Provides a fluent builder pattern for declaring sensor-controller-actuator
// connections.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::string::ToString;
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
    /// Entity ID this brick chain belongs to
    entity_id: u32,

    /// Sensors to evaluate
    sensors: Vec<SensorType>,

    /// Controllers to filter signals
    controllers: Vec<Controller>,

    /// Actuators with their modes
    actuators: Vec<(u8, u8)>, // (actuator_type, mode)

    /// Reference to the mapping table for registration
    mapping_table: Option<LogicMappingTableWasm>,
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
        }
    }

    /// Add a sensor to the brick chain
    #[wasm_bindgen]
    pub fn sensor(mut self, sensor: SensorType) -> Self {
        self.sensors.push(sensor);
        self
    }

    /// Add a keyboard key sensor (convenience)
    #[wasm_bindgen]
    pub fn sensor_key(mut self, key_code: u32) -> Self {
        // Placeholder - would create KeyShortcut sensor
        self.sensors.push(SensorType::KeyShortcut);
        self
    }

    /// Add a controller to the brick chain
    #[wasm_bindgen]
    pub fn controller(mut self, controller: Controller) -> Self {
        self.controllers.push(controller);
        self
    }

    /// Add a Select actuator
    ///
    /// # Arguments
    /// * `mode` - 0=Single, 1=Multi, 2=Replace
    #[wasm_bindgen]
    pub fn actuator_select(mut self, mode: u8) -> Self {
        self.actuators.push((1, mode)); // 1 = Select actuator type
        self
    }

    /// Add a Highlight actuator
    ///
    /// # Arguments
    /// * `color_argb` - Color in ARGB format
    /// * `opacity` - Opacity value
    #[wasm_bindgen]
    pub fn actuator_highlight(mut self, color_argb: u32, opacity: f32) -> Self {
        let _ = color_argb;
        let _ = opacity;
        self.actuators.push((0, 0)); // 0 = Highlight actuator type
        self
    }

    /// Add a Move actuator
    ///
    /// # Arguments
    /// * `mode` - 0=To, 1=By, 2=Drag
    /// * `x` - X value or offset
    /// * `y` - Y value or offset
    #[wasm_bindgen]
    pub fn actuator_move(mut self, mode: u8, x: f32, y: f32) -> Self {
        let _ = mode;
        let _ = x;
        let _ = y;
        self.actuators.push((2, mode)); // 2 = Move actuator type
        self
    }

    /// Connect and register the brick chain
    #[wasm_bindgen]
    pub fn connect(self) -> BrickHandle {
        // Generate unique ID
        let id = alloc::format!("brick-{}-{}", self.entity_id, self.sensors.len());

        // Return handle with ID
        BrickHandle::new(id)
    }

    /// Get the entity ID
    #[wasm_bindgen]
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Get the number of sensors
    #[wasm_bindgen]
    pub fn sensor_count(&self) -> usize {
        self.sensors.len()
    }

    /// Get the number of controllers
    #[wasm_bindgen]
    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }

    /// Get the number of actuators
    #[wasm_bindgen]
    pub fn actuator_count(&self) -> usize {
        self.actuators.len()
    }
}

impl Default for BrickChainBuilder {
    fn default() -> Self {
        Self::new(0)
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
    fn test_builder_full_chain() {
        let handle = BrickChainBuilder::new(1)
            .sensor(SensorType::MouseClick)
            .controller(Controller::direct())
            .actuator_select(0)
            .connect();

        assert!(!handle.id().is_empty());
    }
}
