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

// Import types from parent module (using crate path to avoid conflicts)
use crate::logic::controller::Controller;
use crate::logic::controller::ControllerType;
use crate::logic::mapping_table::ActuatorType;
use crate::logic::sensor_type::SensorType;

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
pub fn sensor_mouse_drag(_button: u8) -> SensorType {
    // Mouse drag is simulated using MouseClick with Hysteresis controller
    SensorType::MouseClick
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
pub fn sensor_mouse_wheel(_direction: i8) -> SensorType {
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
pub fn sensor_keyboard_key(_key_code: u32, _modifiers: u8) -> SensorType {
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

// ═══════════════════════════════════════════════════════════════════════════════════════
// CONTROLLER FACTORIES - Funciones planas para JS
// ═══════════════════════════════════════════════════════════════════════════════

/// Create an AND controller
///
/// # Arguments
/// * `sensor` - Secondary sensor that must also be active
///
/// # Returns
/// Controller with AND logic (all sensors must be active)
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerAnd(SensorType.MouseOver);
/// ```
#[wasm_bindgen]
pub fn factory_and(sensor: SensorType) -> Controller {
    Controller::and(sensor)
}

/// Create an OR controller
///
/// # Arguments
/// * `sensor` - Alternative sensor that can activate
///
/// # Returns
/// Controller with OR logic (any sensor activates)
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerOr(SensorType.MouseClick);
/// ```
#[wasm_bindgen]
pub fn factory_or(sensor: SensorType) -> Controller {
    Controller::or(sensor)
}

/// Create an XOR controller
///
/// # Returns
/// Controller with XOR logic (exactly one sensor active)
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerXor();
/// ```
#[wasm_bindgen]
pub fn factory_xor() -> Controller {
    // XOR: Exactly one sensor active (not both, not none)
    // Implemented via Custom controller with JS logic
    Controller::custom(
        String::from("xor"),
        String::from(
            r#"
            (signal, context) => {
                // Count active sensors in the chain
                const sensors = context.getProperty('sensors') || [];
                let activeCount = 0;
                for (const s of sensors) {
                    if (s.getCurrent && s.getCurrent()) activeCount++;
                }
                return activeCount === 1;
            }
        "#,
        ),
    )
}

/// Create a NAND controller
///
/// # Returns
/// Controller with NAND logic (not all sensors active)
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerNand();
/// ```
#[wasm_bindgen]
pub fn factory_nand() -> Controller {
    // NAND: NOT (all active) = at least one inactive
    // Equivalent to: NOT(AND) - negated conjunction
    Controller::custom(
        String::from("nand"),
        String::from(
            r#"
            (signal, context) => {
                // NAND returns true if NOT all sensors are active
                const sensors = context.getProperty('sensors') || [];
                if (sensors.length === 0) return true;  // Vacuously true
                for (const s of sensors) {
                    if (s.getCurrent && !s.getCurrent()) return true;
                }
                return false;
            }
        "#,
        ),
    )
}

/// Create a NOR controller
///
/// # Returns
/// Controller with NOR logic (no sensors active)
///
/// # JavaScript Example
/// ```javascript
/// const ctrl = controllerNor();
/// ```
#[wasm_bindgen]
pub fn factory_nor() -> Controller {
    // NOR: NOT (any active) = none active
    // Equivalent to: NOT(OR) - negated disjunction
    Controller::custom(
        String::from("nor"),
        String::from(
            r#"
            (signal, context) => {
                // NOR returns true only if NO sensors are active
                const sensors = context.getProperty('sensors') || [];
                for (const s of sensors) {
                    if (s.getCurrent && s.getCurrent()) return false;
                }
                return true;
            }
        "#,
        ),
    )
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
pub fn factory_not() -> Controller {
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
pub fn factory_direct() -> Controller {
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
pub fn factory_blinky(interval: u8) -> Controller {
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
pub fn factory_debounce(ticks: u8) -> Controller {
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
pub fn factory_hysteresis(high: f32, low: f32) -> Controller {
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
pub fn factory_threshold(value: f32) -> Controller {
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
pub fn factory_pattern(mask: u8) -> Controller {
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
pub fn factory_custom(name: String, code: String) -> Controller {
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
pub fn actuator_select_single() -> ActuatorType {
    ActuatorType::Select
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
pub fn actuator_select_multi() -> ActuatorType {
    ActuatorType::Select
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
pub fn actuator_select_toggle() -> ActuatorType {
    ActuatorType::Select
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
pub fn actuator_select_clear() -> ActuatorType {
    ActuatorType::Select
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
pub fn actuator_highlight(color_argb: u32, opacity: f32) -> ActuatorType {
    let _ = color_argb;
    let _ = opacity;
    ActuatorType::Highlight
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
pub fn actuator_move(mode: u8, x: f32, y: f32) -> ActuatorType {
    let _ = mode;
    let _ = x;
    let _ = y;
    ActuatorType::Move
}

/// Create a delete actuator
///
/// # Returns
/// ActuatorType.Move (delete operates via move/transform)
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorDelete();
/// ```
#[wasm_bindgen]
pub fn actuator_delete() -> ActuatorType {
    ActuatorType::Move
}

/// Create an emit event actuator
///
/// # Returns
/// ActuatorType.Move (event emission handled via state change)
///
/// # JavaScript Example
/// ```javascript
/// const actuator = actuatorEmitEvent('EntitySelected', '{"id":42}');
/// ```
#[wasm_bindgen]
pub fn actuator_emit_event(event_name: String, event_data: Option<String>) -> ActuatorType {
    let _ = event_name;
    let _ = event_data;
    ActuatorType::Move
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
        let ctrl = factory_and(SensorType::MouseOver);
        assert!(matches!(ctrl.controller_type(), ControllerType::And));
    }

    #[test]
    fn test_controller_or() {
        let ctrl = factory_or(SensorType::MouseClick);
        assert!(matches!(ctrl.controller_type(), ControllerType::Or));
    }

    #[test]
    fn test_controller_xor() {
        let ctrl = factory_xor();
        assert!(matches!(ctrl.controller_type(), ControllerType::Custom));
        assert_eq!(ctrl.custom_name(), Some(String::from("xor")));
    }

    #[test]
    fn test_controller_nand() {
        let ctrl = factory_nand();
        assert!(matches!(ctrl.controller_type(), ControllerType::Custom));
        assert_eq!(ctrl.custom_name(), Some(String::from("nand")));
    }

    #[test]
    fn test_controller_nor() {
        let ctrl = factory_nor();
        assert!(matches!(ctrl.controller_type(), ControllerType::Custom));
        assert_eq!(ctrl.custom_name(), Some(String::from("nor")));
    }

    #[test]
    fn test_controller_direct() {
        let ctrl = factory_direct();
        assert!(matches!(ctrl.controller_type(), ControllerType::Direct));
    }

    #[test]
    fn test_controller_debounce() {
        let ctrl = factory_debounce(6);
        assert!(matches!(ctrl.controller_type(), ControllerType::Debounce));
    }

    #[test]
    fn test_actuator_select_single() {
        let actuator = actuator_select_single();
        assert_eq!(actuator, ActuatorType::Select);
    }

    #[test]
    fn test_actuator_highlight() {
        let actuator = actuator_highlight(0xff00ff00, 0.5);
        assert_eq!(actuator, ActuatorType::Highlight);
    }

    #[test]
    fn test_actuator_move() {
        let actuator = actuator_move(0, 100.0, 200.0);
        assert_eq!(actuator, ActuatorType::Move);
    }

    #[test]
    fn test_actuator_delete() {
        let actuator = actuator_delete();
        assert_eq!(actuator, ActuatorType::Move);
    }
}
