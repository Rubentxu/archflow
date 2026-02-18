// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - JS Entity Builder (Simplified)
//
// Epic: EPIC-ECS-005 - WASM Integration
// Provides fluent JavaScript API: world.spawn().insert().behavior().build()
// Uses primitive types for WASM compatibility
//
// Updated: EPIC-ECS-009 - Migrated to use ECS ShapeComponents
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

// Shape type constants for JS API (matching ShapeType enum)
#[wasm_bindgen]
pub struct ShapeTypes;

#[wasm_bindgen]
impl ShapeTypes {
    #[wasm_bindgen]
    pub fn rectangle() -> u8 {
        0
    }
    #[wasm_bindgen]
    pub fn circle() -> u8 {
        1
    }
    #[wasm_bindgen]
    pub fn ellipse() -> u8 {
        2
    }
    #[wasm_bindgen]
    pub fn triangle() -> u8 {
        3
    }
    #[wasm_bindgen]
    pub fn diamond() -> u8 {
        4
    }
    #[wasm_bindgen]
    pub fn cylinder() -> u8 {
        5
    }
    #[wasm_bindgen]
    pub fn line() -> u8 {
        6
    }
    #[wasm_bindgen]
    pub fn arc() -> u8 {
        7
    }
}

/// JS Entity Builder - Fluent API para JavaScript
///
/// Permite encadenar llamadas en JavaScript:
/// ```javascript
/// const entity = await bridge.world.spawn()
///     .insert(0, 0, 50, 50)  // x, y, width, height
///     .behavior('move')
///         .sensor(1, 25)      // sensor_type, key_code
///         .actuator(2, 0, 100) // actuator_type, x, y
///     .build();
/// ```
#[wasm_bindgen]
pub struct JsEntityBuilder {
    /// Entity ID being built
    entity_id: u32,
    /// Behavior blocks added
    behavior_blocks: Vec<JsBehaviorBlock>,
    /// Current behavior block being built
    current_behavior: Option<JsBehaviorBlock>,
}

/// Single behavior block in JS API
#[wasm_bindgen]
#[derive(Clone)]
pub struct JsBehaviorBlock {
    /// Name of the behavior
    name: String,
    /// Sensor type (0=MouseOver, 1=MouseClick, 2=KeyShortcut, etc.)
    sensor_type: u8,
    /// Key code for keyboard sensors
    key_code: u32,
    /// Controller type (0=Direct, 1=AND, 2=OR, etc.)
    controller_type: u8,
    /// Controller parameter
    controller_param: u32,
    /// Actuator type (0=Highlight, 1=Select, 2=Move, etc.)
    actuator_type: u8,
    /// Actuator X value
    actuator_x: f32,
    /// Actuator Y value
    actuator_y: f32,
}

#[wasm_bindgen]
impl JsBehaviorBlock {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sensor_type: 0,
            key_code: 0,
            controller_type: 0,
            controller_param: 0,
            actuator_type: 0,
            actuator_x: 0.0,
            actuator_y: 0.0,
        }
    }

    /// Get sensor type
    #[wasm_bindgen]
    pub fn sensor_type(&self) -> u8 {
        self.sensor_type
    }

    /// Get key code
    #[wasm_bindgen]
    pub fn key_code(&self) -> u32 {
        self.key_code
    }

    /// Get controller type
    #[wasm_bindgen]
    pub fn controller_type(&self) -> u8 {
        self.controller_type
    }

    /// Get actuator type
    #[wasm_bindgen]
    pub fn actuator_type(&self) -> u8 {
        self.actuator_type
    }
}

#[wasm_bindgen]
impl JsEntityBuilder {
    /// Create a new JSEntityBuilder
    #[wasm_bindgen(constructor)]
    pub fn new(entity_id: u32) -> Self {
        Self {
            entity_id,
            behavior_blocks: Vec::new(),
            current_behavior: None,
        }
    }

    /// Insert a component (x, y, width, height)
    #[wasm_bindgen]
    pub fn insert(self, _x: f32, _y: f32, _width: f32, _height: f32) -> Self {
        // This would interface with the engine to set components
        // For now, just return self for chaining
        self
    }

    /// Set shape type (0=Rectangle, 1=Circle, 2=Ellipse, etc.)
    /// Uses ShapeTypes constants: ShapeTypes.circle(), ShapeTypes.rectangle(), etc.
    #[wasm_bindgen]
    pub fn shape(self, _shape_type: u8) -> Self {
        // This will use ShapeComponent ECS in the engine
        // shape_type: 0=Rectangle, 1=Circle, 2=Ellipse, 3=Triangle, 4=Diamond, 5=Cylinder, 6=Line, 7=Arc
        self
    }

    /// Set fill color (r, g, b)
    #[wasm_bindgen]
    pub fn color(self, _r: u8, _g: u8, _b: u8) -> Self {
        // This will use ColorComponent ECS in the engine
        self
    }

    /// Set position (x, y) - uses Transform component
    #[wasm_bindgen]
    pub fn position(self, _x: f32, _y: f32) -> Self {
        // This will use Transform ECS component
        self
    }

    /// Set size (width, height) - uses RenderProperties component
    #[wasm_bindgen]
    pub fn size(self, _width: f32, _height: f32) -> Self {
        // This will use RenderProperties ECS component
        self
    }

    /// Set layer for rendering order - uses RenderProperties component
    #[wasm_bindgen]
    pub fn layer(self, _layer: i32) -> Self {
        // This will use RenderProperties ECS component
        self
    }

    /// Set visibility (true=visible, false=hidden) - uses VisibilityComponent
    #[wasm_bindgen]
    pub fn visible(self, _is_visible: bool) -> Self {
        // This will use VisibilityComponent ECS
        self
    }

    /// Set stroke color (r, g, b)
    #[wasm_bindgen]
    pub fn stroke(self, _r: u8, _g: u8, _b: u8) -> Self {
        // This will use ColorComponent ECS stroke field
        self
    }

    /// Set stroke width
    #[wasm_bindgen]
    pub fn stroke_width(self, _width: f32) -> Self {
        // This will use ColorComponent ECS stroke_width field
        self
    }

    /// Start a behavior block
    #[wasm_bindgen]
    pub fn behavior(mut self, name: &str) -> Self {
        let block = JsBehaviorBlock::new(name);
        self.current_behavior = Some(block);
        self
    }

    /// Add a sensor to current behavior
    /// sensor_type: 0=MouseOver, 1=MouseClick, 2=KeyShortcut, etc.
    /// key_code: key code for keyboard sensors
    #[wasm_bindgen]
    pub fn sensor(mut self, sensor_type: u8, key_code: u32) -> Self {
        if let Some(ref mut behavior) = self.current_behavior {
            behavior.sensor_type = sensor_type;
            behavior.key_code = key_code;
        }
        self
    }

    /// Add a controller to current behavior
    /// controller_type: 0=Direct, 1=AND, 2=OR, 3=NOT, etc.
    /// param: parameter for the controller
    #[wasm_bindgen]
    pub fn controller(mut self, controller_type: u8, param: u32) -> Self {
        if let Some(ref mut behavior) = self.current_behavior {
            behavior.controller_type = controller_type;
            behavior.controller_param = param;
        }
        self
    }

    /// Add an actuator to current behavior
    /// actuator_type: 0=Highlight, 1=Select, 2=Move, etc.
    /// x, y: coordinates or values
    #[wasm_bindgen]
    pub fn actuator(mut self, actuator_type: u8, x: f32, y: f32) -> Self {
        if let Some(ref mut behavior) = self.current_behavior {
            behavior.actuator_type = actuator_type;
            behavior.actuator_x = x;
            behavior.actuator_y = y;
        }
        self
    }

    /// End current behavior block
    #[wasm_bindgen]
    pub fn end_behavior(mut self) -> Self {
        if let Some(behavior) = self.current_behavior.take() {
            self.behavior_blocks.push(behavior);
        }
        self
    }

    /// Build the entity and return its ID
    #[wasm_bindgen]
    pub fn build(self) -> u32 {
        self.entity_id
    }

    /// Get the entity ID
    #[wasm_bindgen]
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Get number of behavior blocks
    #[wasm_bindgen]
    pub fn behavior_count(&self) -> usize {
        self.behavior_blocks.len()
    }

    /// Get behavior at index
    #[wasm_bindgen]
    pub fn get_behavior(&self, index: usize) -> Option<JsBehaviorBlock> {
        self.behavior_blocks.get(index).cloned()
    }
}

/// Constants for sensor types (matching SensorType enum)
#[wasm_bindgen]
pub struct SensorTypes;

#[wasm_bindgen]
impl SensorTypes {
    #[wasm_bindgen]
    pub fn mouse_over() -> u8 {
        0
    }
    #[wasm_bindgen]
    pub fn mouse_click() -> u8 {
        1
    }
    #[wasm_bindgen]
    pub fn right_click() -> u8 {
        2
    }
    #[wasm_bindgen]
    pub fn key_shortcut() -> u8 {
        3
    }
    #[wasm_bindgen]
    pub fn proximity() -> u8 {
        4
    }
    #[wasm_bindgen]
    pub fn radar() -> u8 {
        5
    }
    #[wasm_bindgen]
    pub fn touch() -> u8 {
        6
    }
    #[wasm_bindgen]
    pub fn ray() -> u8 {
        7
    }
}

/// Constants for actuator types (matching ActuatorType enum)
#[wasm_bindgen]
pub struct ActuatorTypes;

#[wasm_bindgen]
impl ActuatorTypes {
    #[wasm_bindgen]
    pub fn highlight() -> u8 {
        0
    }
    #[wasm_bindgen]
    pub fn select() -> u8 {
        1
    }
    #[wasm_bindgen]
    pub fn move_actuator() -> u8 {
        2
    }
    #[wasm_bindgen]
    pub fn delete() -> u8 {
        3
    }
    #[wasm_bindgen]
    pub fn undo() -> u8 {
        4
    }
    #[wasm_bindgen]
    pub fn redo() -> u8 {
        5
    }
    #[wasm_bindgen]
    pub fn camera() -> u8 {
        6
    }
    #[wasm_bindgen]
    pub fn property() -> u8 {
        7
    }
    #[wasm_bindgen]
    pub fn animation() -> u8 {
        8
    }
}

/// Constants for controller types
#[wasm_bindgen]
pub struct ControllerTypes;

#[wasm_bindgen]
impl ControllerTypes {
    #[wasm_bindgen]
    pub fn direct() -> u8 {
        0
    }
    #[wasm_bindgen]
    pub fn and() -> u8 {
        1
    }
    #[wasm_bindgen]
    pub fn or() -> u8 {
        2
    }
    #[wasm_bindgen]
    pub fn not() -> u8 {
        3
    }
    #[wasm_bindgen]
    pub fn blinky() -> u8 {
        4
    }
    #[wasm_bindgen]
    pub fn debounce() -> u8 {
        5
    }
    #[wasm_bindgen]
    pub fn hysteresis() -> u8 {
        6
    }
    #[wasm_bindgen]
    pub fn threshold() -> u8 {
        7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_builder_creation() {
        let builder = JsEntityBuilder::new(1);
        assert_eq!(builder.entity_id(), 1);
    }

    #[test]
    fn test_behavior_block_creation() {
        let block = JsBehaviorBlock::new("test");
        assert_eq!(block.name, "test");
    }

    #[test]
    fn test_fluent_api() {
        let builder = JsEntityBuilder::new(1)
            .insert(0.0, 0.0, 50.0, 50.0)
            .behavior("move")
            .sensor(3, 25) // KeyShortcut, W key
            .controller(0, 0) // Direct
            .actuator(2, 0.0, 100.0); // Move

        assert_eq!(builder.behavior_count(), 0); // Not ended yet
    }
}
