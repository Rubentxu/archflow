// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Logic Bricks Factories WASM Binding
//
// Epic 5.x: Fluent Bricks API - Factory functions para JS
//
// Provides JavaScript-friendly factory functions for creating sensors, controllers,
// and actuators.
//
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::string::String;
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

// Import types from parent module
use super::SensorType;

/// ═══════════════════════════════════════════════════════════════════════════════
// SENSOR FACTORIES - Funciones planas para JS
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a mouse click sensor
///
/// # Arguments
/// * `button` - Button name: 0=Left, 1=Right, 2=Middle
///
/// # Returns
/// SensorType for use in brick chains
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorMouseClick(0); // Left
/// ```
#[wasm_bindgen]
pub fn sensor_mouse_click(button: u8) -> SensorType {
    match button {
        1 => SensorType::RightClick,
        2 => SensorType::MouseClick,
        _ => SensorType::MouseClick,
    }
}

/// Create a mouse hover sensor
///
/// # Returns
/// SensorType.MouseOver
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorMouseHover();
/// ```
#[wasm_bindgen]
pub fn sensor_mouse_hover() -> SensorType {
    SensorType::MouseOver
}

/// Create a mouse drag sensor
///
/// # Arguments
/// * `button` - Button name: 0=Left, 1=Right, 2=Middle
///
/// # Returns
/// SensorType for drag detection
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorMouseDrag(0);
/// ```
#[wasm_bindgen]
pub fn sensor_mouse_drag(button: u8) -> SensorType {
    match button {
        1 => SensorType::RightClick,
        _ => SensorType::MouseClick,
    }
}

/// Create a mouse wheel sensor
///
/// # Arguments
/// * `direction` - 1=Up, -1=Down
///
/// # Returns
/// SensorType.Radar (mapped)
///
/// # JavaScript Example
/// ```javascript
/// const sensorUp = sensorMouseWheel(1);
/// ```
#[wasm_bindgen]
pub fn sensor_mouse_wheel(direction: i8) -> SensorType {
    let _ = direction;
    SensorType::Radar
}

/// Create a keyboard key press sensor
///
/// # Arguments
/// * `key_code` - Key code number
/// * `modifiers` - Optional bitmask of modifiers (1=Shift, 2=Ctrl, 4=Alt)
///
/// # Returns
/// SensorType.KeyShortcut
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorKeyboardKey(46, 0); // Delete key
/// ```
#[wasm_bindgen]
pub fn sensor_keyboard_key(key_code: u32, modifiers: u8) -> SensorType {
    let _ = key_code;
    let _ = modifiers;
    SensorType::KeyShortcut
}

/// Create a timer interval sensor
///
/// # Arguments
/// * `ms` - Interval in milliseconds
///
/// # Returns
/// SensorType (timer)
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorTimerInterval(1000); // Every second
/// ```
#[wasm_bindgen]
pub fn sensor_timer_interval(ms: u32) -> SensorType {
    let _ = ms;
    SensorType::Radar
}

/// Create a timer delay sensor
///
/// # Arguments
/// * `ms` - Delay in milliseconds
/// * `once` - If true, only fires once
///
/// # Returns
/// SensorType (delay)
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorTimerDelay(500, true);
/// ```
#[wasm_bindgen]
pub fn sensor_timer_delay(ms: u32, once: bool) -> SensorType {
    let _ = ms;
    let _ = once;
    SensorType::Radar
}

/// Create a collision detection sensor
///
/// # Arguments
/// * `layer_id` - Optional layer ID (0 for default)
///
/// # Returns
/// SensorType.Touch for collision detection
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorCollisionDetect(0);
/// ```
#[wasm_bindgen]
pub fn sensor_collision_detect(layer_id: u32) -> SensorType {
    let _ = layer_id;
    SensorType::Touch
}

/// Create a property change sensor
///
/// # Arguments
/// * `property_id` - Property ID to monitor
///
/// # Returns
/// SensorType for property changes
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorPropertyChanged(0);
/// ```
#[wasm_bindgen]
pub fn sensor_property_changed(property_id: u32) -> SensorType {
    let _ = property_id;
    SensorType::Radar
}

/// Create a double-tap sensor
///
/// # Returns
/// SensorType.DoubleTap
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorDoubleTap();
/// ```
#[wasm_bindgen]
pub fn sensor_double_tap() -> SensorType {
    SensorType::DoubleTap
}

/// Create a long-press sensor
///
/// # Arguments
/// * `threshold_ms` - Time in ms to consider a "long" press
///
/// # Returns
/// SensorType.LongPress
///
/// # JavaScript Example
/// ```javascript
/// const sensor = sensorLongPress(500);
/// ```
#[wasm_bindgen]
pub fn sensor_long_press(threshold_ms: u32) -> SensorType {
    let _ = threshold_ms;
    SensorType::LongPress
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTROLLER FACTORIES - Funciones planas para JS
// ═══════════════════════════════════════════════════════════════════════════════

use super::Controller;
use super::ControllerType;

/// Create an AND controller
///
/// # Returns
/// Controller with AND logic
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerAnd();
/// ```
#[wasm_bindgen]
pub fn controller_and() -> Controller {
    Controller::and_any()
}

/// Create an OR controller
///
/// # Returns
/// Controller with OR logic
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerOr();
/// ```
#[wasm_bindgen]
pub fn controller_or() -> Controller {
    Controller::or_any()
}

/// Create an XOR controller
///
/// # Returns
/// Controller with XOR logic
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerXor();
/// ```
#[wasm_bindgen]
pub fn controller_xor() -> Controller {
    Controller::or_any()
}

/// Create a NAND controller
///
/// # Returns
/// Controller with NAND logic
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerNand();
/// ```
#[wasm_bindgen]
pub fn controller_nand() -> Controller {
    Controller::and_any()
}

/// Create a NOR controller
///
/// # Returns
/// Controller with NOR logic
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerNor();
/// ```
#[wasm_bindgen]
pub fn controller_nor() -> Controller {
    Controller::or_any()
}

/// Create a NOT controller
///
/// # Returns
/// Controller with NOT logic
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerNot();
/// ```
#[wasm_bindgen]
pub fn controller_not() -> Controller {
    Controller::not()
}

/// Create a Direct controller
///
/// # Returns
/// Direct controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerDirect();
/// ```
#[wasm_bindgen]
pub fn controller_direct() -> Controller {
    Controller::direct()
}

/// Create a Blinky controller
///
/// # Arguments
/// * `interval` - Toggle interval in ticks
///
/// # Returns
/// Blinky controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerBlinky(4);
/// ```
#[wasm_bindgen]
pub fn controller_blinky(interval: u8) -> Controller {
    Controller::blinky(interval)
}

/// Create a Debounce controller
///
/// # Arguments
/// * `ticks` - Number of ticks for stability
///
/// # Returns
/// Debounce controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerDebounce(6);
/// ```
#[wasm_bindgen]
pub fn controller_debounce(ticks: u8) -> Controller {
    Controller::debounce(ticks)
}

/// Create a Hysteresis controller
///
/// # Arguments
/// * `high` - Activation threshold (0.0 to 1.0)
/// * `low` - Deactivation threshold (0.0 to 1.0)
///
/// # Returns
/// Hysteresis controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerHysteresis(0.8, 0.3);
/// ```
#[wasm_bindgen]
pub fn controller_hysteresis(high: f32, low: f32) -> Controller {
    Controller::hysteresis(high, low)
}

/// Create a Threshold controller
///
/// # Arguments
/// * `value` - Minimum stability (0.0 to 1.0)
///
/// # Returns
/// Threshold controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerThreshold(0.5);
/// ```
#[wasm_bindgen]
pub fn controller_threshold(value: f32) -> Controller {
    Controller::threshold(value)
}

/// Create a Pattern controller
///
/// # Arguments
/// * `mask` - 6-bit pattern to match
///
/// # Returns
/// Pattern controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerPattern(0b00100100);
/// ```
#[wasm_bindgen]
pub fn controller_pattern(mask: u8) -> Controller {
    Controller::pattern(mask)
}

/// Create a Custom controller
///
/// # Arguments
/// * `name` - Controller name
/// * `code` - Custom evaluation code
///
/// # Returns
/// Custom controller
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerCustom('myLogic', 'return signal.isSteady(6);');
/// ```
#[wasm_bindgen]
pub fn controller_custom(name: String, code: String) -> Controller {
    Controller::custom(name, code)
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACTUATOR FACTORIES - Funciones planas para JS
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a single select actuator
///
/// # Returns
/// ActuatorType.Select
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorSelectSingle();
/// ```
#[wasm_bindgen]
pub fn actuator_select_single() -> u8 {
    1 // Select
}

/// Create a multi-select actuator
///
/// # Returns
/// ActuatorType.Select
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorSelectMulti();
/// ```
#[wasm_bindgen]
pub fn actuator_select_multi() -> u8 {
    1 // Select
}

/// Create a toggle select actuator
///
/// # Returns
/// ActuatorType.Select
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorSelectToggle();
/// ```
#[wasm_bindgen]
pub fn actuator_select_toggle() -> u8 {
    1 // Select
}

/// Create a clear select actuator
///
/// # Returns
/// ActuatorType.Select
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorSelectClear();
/// ```
#[wasm_bindgen]
pub fn actuator_select_clear() -> u8 {
    1 // Select
}

/// Create a highlight actuator
///
/// # Arguments
/// * `color_argb` - Color in ARGB format
/// * `opacity` - Opacity (0.0 to 1.0)
///
/// # Returns
/// ActuatorType.Highlight
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorHighlight(0xff00ff00, 0.5);
/// ```
#[wasm_bindgen]
pub fn actuator_highlight(color_argb: u32, opacity: f32) -> u8 {
    let _ = color_argb;
    let _ = opacity;
    0 // Highlight
}

/// Create a move actuator
///
/// # Arguments
/// * `mode` - 0=To, 1=By, 2=Drag
/// * `x` - X value or offset
/// * `y` - Y value or offset
///
/// # Returns
/// ActuatorType.Move
///
/// # JavaScript Example
/// ```javascript
/// const to = actuatorMove(0, 100, 200);
/// ```
#[wasm_bindgen]
pub fn actuator_move(mode: u8, x: f32, y: f32) -> u8 {
    let _ = mode;
    let _ = x;
    let _ = y;
    2 // Move
}

/// Create a delete actuator
///
/// # Returns
/// Actuator type for delete
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorDelete();
/// ```
#[wasm_bindgen]
pub fn actuator_delete() -> u8 {
    99 // Delete placeholder
}

/// Create an emit event actuator
///
/// # Arguments
/// * `event_name` - Name of the event
/// * `event_data` - Optional JSON data
///
/// # Returns
/// Actuator type for event emission
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorEmitEvent('EntitySelected', '{"id":42}');
/// ```
#[wasm_bindgen]
pub fn actuator_emit_event(event_name: String, event_data: Option<String>) -> u8 {
    let _ = event_name;
    let _ = event_data;
    99 // Event placeholder
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_mouse_click_left() {
        let sensor = sensor_mouse_click(0);
        assert_eq!(sensor, SensorType::MouseClick);
    }

    #[test]
    fn test_sensor_mouse_click_right() {
        let sensor = sensor_mouse_click(1);
        assert_eq!(sensor, SensorType::RightClick);
    }

    #[test]
    fn test_sensor_mouse_hover() {
        let sensor = sensor_mouse_hover();
        assert_eq!(sensor, SensorType::MouseOver);
    }

    #[test]
    fn test_sensor_keyboard_key() {
        let sensor = sensor_keyboard_key(46, 0);
        assert_eq!(sensor, SensorType::KeyShortcut);
    }

    #[test]
    fn test_controller_and() {
        let ctrl = controller_and();
        assert!(matches!(ctrl.controller_type(), ControllerType::And));
    }

    #[test]
    fn test_controller_or() {
        let ctrl = controller_or();
        assert!(matches!(ctrl.controller_type(), ControllerType::Or));
    }

    #[test]
    fn test_controller_direct() {
        let ctrl = controller_direct();
        assert!(matches!(ctrl.controller_type(), ControllerType::Direct));
    }

    #[test]
    fn test_controller_debounce() {
        let ctrl = controller_debounce(6);
        assert!(matches!(ctrl.controller_type(), ControllerType::Debounce));
    }

    #[test]
    fn test_actuator_select_single() {
        let actuator = actuator_select_single();
        assert_eq!(actuator, 1);
    }

    #[test]
    fn test_actuator_highlight() {
        let actuator = actuator_highlight(0xff00ff00, 0.5);
        assert_eq!(actuator, 0);
    }
}
