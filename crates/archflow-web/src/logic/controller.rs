// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Controller WASM Binding
//
// Epic 5.3: Expose Controller to JavaScript/TypeScript
//
// Provides a JavaScript-accessible enum for boolean logic controllers
// that combine sensor signals using AND, OR, NOT, or Direct logic.
// Also supports advanced controllers: Blinky, Debounce, Hysteresis, Threshold, Pattern.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use super::SensorType;
use alloc::string::String;
use archflow_logic::mapping::{Controller as CoreController, SensorType as CoreSensorType};
use wasm_bindgen::prelude::*;

/// Controller type enumeration
///
/// Defines the type of boolean logic to apply when combining sensor signals.
///
/// # JavaScript Example
/// ```javascript
/// import { Controller, SensorType } from '@archflow/sdk';
///
/// // Direct: pass through the primary sensor
/// const direct = Controller.Direct();
///
/// // AND: require both MouseOver AND MouseClick
/// const and = Controller.And(SensorType.MouseClick);
///
/// // OR: require MouseOver OR MouseClick
/// const or = Controller.Or(SensorType.MouseClick);
///
/// // NOT: invert the primary sensor
/// const not = Controller.Not();
///
/// // Blinky: toggle every N ticks
/// const blinky = Controller.Blinky(4);
///
/// // Debounce: require N stable ticks
/// const debounce = Controller.Debounce(6);
///
/// // Hysteresis: different on/off thresholds
/// const hyst = Controller.Hysteresis(0.8, 0.3);
/// ```
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerType {
    /// Pass through the primary sensor signal
    Direct = 0,

    /// AND logic: primary AND other sensor must both be active
    And = 1,

    /// OR logic: primary OR other sensor must be active
    Or = 2,

    /// NOT logic: invert the primary sensor signal
    Not = 3,

    /// Blinky: Toggles active/inactive at regular intervals
    Blinky = 4,

    /// Debounce: Requires signal to be stable for N ticks
    Debounce = 5,

    /// Hysteresis: Different activation/deactivation thresholds
    Hysteresis = 6,

    /// Threshold: Requires minimum stability percentage
    Threshold = 7,

    /// Pattern: Matches specific binary pattern in history
    Pattern = 8,

    /// Custom: JavaScript sandbox evaluation
    Custom = 9,
}

/// Controller for boolean logic operations on sensor signals
///
/// This struct wraps the ControllerType enum with optional parameters:
/// - secondary_sensor: for AND/OR operations
/// - numeric_params: for Blinky, Debounce, Threshold, Pattern controllers
/// - float_params: for Hysteresis controller
/// - custom_data: for Custom controller (name, code)
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Controller {
    /// The type of controller
    controller_type: ControllerType,

    /// The secondary sensor for AND/OR operations (optional)
    secondary_sensor: Option<SensorType>,

    /// Numeric parameter (for Blinky interval, Debounce ticks, Pattern mask)
    numeric_param: u8,

    /// Float parameters (for Hysteresis high/low, Threshold value)
    float_param1: f32,
    float_param2: f32,

    /// Custom controller name (for Custom type)
    custom_name: Option<String>,

    /// Custom controller code (for Custom type)
    custom_code: Option<String>,
}

#[wasm_bindgen]
impl Controller {
    // ═══════════════════════════════════════════════════════════════════════════════
    // BASIC CONTROLLERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Creates a new Direct controller (pass-through)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const controller = Controller.Direct();
    /// ```
    #[wasm_bindgen]
    pub fn direct() -> Self {
        Self {
            controller_type: ControllerType::Direct,
            secondary_sensor: None,
            numeric_param: 0,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates an AND controller with a secondary sensor
    ///
    /// # Arguments
    /// * `sensor` - The secondary sensor type
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const controller = Controller.And(SensorType.MouseClick);
    /// // Requires both primary sensor AND MouseClick to be active
    /// ```
    #[wasm_bindgen]
    pub fn and(sensor: SensorType) -> Self {
        Self {
            controller_type: ControllerType::And,
            secondary_sensor: Some(sensor),
            numeric_param: 0,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates an OR controller with a secondary sensor
    ///
    /// # Arguments
    /// * `sensor` - The secondary sensor type
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const controller = Controller.Or(SensorType.MouseClick);
    /// // Requires primary sensor OR MouseClick to be active
    /// ```
    #[wasm_bindgen]
    pub fn or(sensor: SensorType) -> Self {
        Self {
            controller_type: ControllerType::Or,
            secondary_sensor: Some(sensor),
            numeric_param: 0,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates a NOT controller (inverts the signal)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const controller = Controller.Not();
    /// // Inverts the primary sensor signal
    /// ```
    #[wasm_bindgen]
    pub fn not() -> Self {
        Self {
            controller_type: ControllerType::Not,
            secondary_sensor: None,
            numeric_param: 0,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PREDEFINED CONTROLLERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Creates a Blinky controller that toggles at regular intervals
    ///
    /// # Arguments
    /// * `interval` - Toggle interval in ticks (16.67ms at 60 FPS)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// // Blink every 100ms (6 ticks at 60fps)
    /// const blinky = Controller.Blinky(6);
    /// ```
    #[wasm_bindgen]
    pub fn blinky(interval: u8) -> Self {
        Self {
            controller_type: ControllerType::Blinky,
            secondary_sensor: None,
            numeric_param: interval,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates a Debounce controller requiring N stable ticks
    ///
    /// # Arguments
    /// * `ticks` - Number of consecutive ticks signal must be HIGH
    ///
    /// # JavaScript Example
    /// ```javascript
    /// // Require 100ms of stable signal (6 ticks)
    /// const debounced = Controller.Debounce(6);
    /// ```
    #[wasm_bindgen]
    pub fn debounce(ticks: u8) -> Self {
        Self {
            controller_type: ControllerType::Debounce,
            secondary_sensor: None,
            numeric_param: ticks,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates a Hysteresis controller with different on/off thresholds
    ///
    /// # Arguments
    /// * `high` - Activation threshold (0.0 to 1.0)
    /// * `low` - Deactivation threshold (0.0 to 1.0)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// // Activate at 80%, deactivate at 30%
    /// const hyst = Controller.Hysteresis(0.8, 0.3);
    /// ```
    #[wasm_bindgen]
    pub fn hysteresis(high: f32, low: f32) -> Self {
        Self {
            controller_type: ControllerType::Hysteresis,
            secondary_sensor: None,
            numeric_param: 0,
            float_param1: high,
            float_param2: low,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates a Threshold controller with minimum stability
    ///
    /// # Arguments
    /// * `value` - Minimum stability threshold (0.0 to 1.0)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// // Require 50% stability (3 out of 6 ticks)
    /// const thresh = Controller.Threshold(0.5);
    /// ```
    #[wasm_bindgen]
    pub fn threshold(value: f32) -> Self {
        Self {
            controller_type: ControllerType::Threshold,
            secondary_sensor: None,
            numeric_param: 0,
            float_param1: value,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates a Pattern controller matching binary pattern
    ///
    /// # Arguments
    /// * `mask` - 6-bit pattern to match
    ///
    /// # JavaScript Example
    /// ```javascript
    /// // Match double-click pattern: 100100
    /// const pattern = Controller.Pattern(0b00100100);
    /// ```
    #[wasm_bindgen]
    pub fn pattern(mask: u8) -> Self {
        Self {
            controller_type: ControllerType::Pattern,
            secondary_sensor: None,
            numeric_param: mask,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: None,
            custom_code: None,
        }
    }

    /// Creates a Custom controller with JavaScript code
    ///
    /// # Arguments
    /// * `name` - Unique identifier for debugging
    /// * `code` - JavaScript code to evaluate
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const custom = Controller.Custom(
    ///   'tooltipOnCtrlHover',
    ///   'return signal.isSteady(6) && (context.modifiers & 2) !== 0;'
    /// );
    /// ```
    #[wasm_bindgen]
    pub fn custom(name: String, code: String) -> Self {
        Self {
            controller_type: ControllerType::Custom,
            secondary_sensor: None,
            numeric_param: 0,
            float_param1: 0.0,
            float_param2: 0.0,
            custom_name: Some(name),
            custom_code: Some(code),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GETTERS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Returns the controller type
    #[wasm_bindgen]
    pub fn controller_type(&self) -> ControllerType {
        self.controller_type
    }

    /// Returns the secondary sensor (if any)
    ///
    /// Returns `null` if there is no secondary sensor.
    #[wasm_bindgen]
    pub fn secondary_sensor(&self) -> Option<SensorType> {
        self.secondary_sensor
    }

    /// Returns the numeric parameter (for Blinky, Debounce, Pattern)
    #[wasm_bindgen]
    pub fn numeric_param(&self) -> u8 {
        self.numeric_param
    }

    /// Returns the first float parameter (for Hysteresis high, Threshold value)
    #[wasm_bindgen]
    pub fn float_param1(&self) -> f32 {
        self.float_param1
    }

    /// Returns the second float parameter (for Hysteresis low)
    #[wasm_bindgen]
    pub fn float_param2(&self) -> f32 {
        self.float_param2
    }

    /// Returns the custom name (for Custom controllers)
    #[wasm_bindgen]
    pub fn custom_name(&self) -> Option<String> {
        self.custom_name.clone()
    }

    /// Returns the custom code (for Custom controllers)
    #[wasm_bindgen]
    pub fn custom_code(&self) -> Option<String> {
        self.custom_code.clone()
    }

    /// Checks if this controller has a secondary sensor
    #[wasm_bindgen]
    pub fn has_secondary_sensor(&self) -> bool {
        self.secondary_sensor.is_some()
    }

    /// Checks if this controller is a Custom type
    #[wasm_bindgen]
    pub fn is_custom(&self) -> bool {
        self.controller_type == ControllerType::Custom
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::direct()
    }
}

// Conversions between WASM and Core types

impl From<Controller> for CoreController {
    fn from(wasm: Controller) -> Self {
        match wasm.controller_type {
            ControllerType::Direct => CoreController::Direct,
            ControllerType::And => {
                let sensor = wasm
                    .secondary_sensor
                    .expect("AND controller must have secondary sensor");
                CoreController::AND(sensor.into())
            }
            ControllerType::Or => {
                let sensor = wasm
                    .secondary_sensor
                    .expect("OR controller must have secondary sensor");
                CoreController::OR(sensor.into())
            }
            ControllerType::Not => CoreController::NOT,
            ControllerType::Blinky => CoreController::Blinky {
                interval: wasm.numeric_param,
            },
            ControllerType::Debounce => CoreController::Debounce {
                ticks: wasm.numeric_param,
            },
            ControllerType::Hysteresis => CoreController::Hysteresis {
                high: wasm.float_param1,
                low: wasm.float_param2,
            },
            ControllerType::Threshold => CoreController::Threshold {
                value: wasm.float_param1,
            },
            ControllerType::Pattern => CoreController::Pattern {
                mask: wasm.numeric_param,
            },
            ControllerType::Custom => {
                let name = wasm.custom_name.unwrap_or_default();
                let code = wasm.custom_code.unwrap_or_default();
                CoreController::Custom { name, code }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_direct_controller() {
        let controller = Controller::direct();
        assert_eq!(controller.controller_type(), ControllerType::Direct);
        assert!(!controller.has_secondary_sensor());
        assert!(controller.secondary_sensor().is_none());
    }

    #[test]
    fn test_and_controller() {
        let controller = Controller::and(SensorType::MouseClick);
        assert_eq!(controller.controller_type(), ControllerType::And);
        assert!(controller.has_secondary_sensor());
        assert_eq!(controller.secondary_sensor(), Some(SensorType::MouseClick));
    }

    #[test]
    fn test_or_controller() {
        let controller = Controller::or(SensorType::Proximity);
        assert_eq!(controller.controller_type(), ControllerType::Or);
        assert!(controller.has_secondary_sensor());
        assert_eq!(controller.secondary_sensor(), Some(SensorType::Proximity));
    }

    #[test]
    fn test_not_controller() {
        let controller = Controller::not();
        assert_eq!(controller.controller_type(), ControllerType::Not);
        assert!(!controller.has_secondary_sensor());
    }

    #[test]
    fn test_blinky_controller() {
        let controller = Controller::blinky(4);
        assert_eq!(controller.controller_type(), ControllerType::Blinky);
        assert_eq!(controller.numeric_param(), 4);
        assert!(!controller.has_secondary_sensor());
    }

    #[test]
    fn test_debounce_controller() {
        let controller = Controller::debounce(6);
        assert_eq!(controller.controller_type(), ControllerType::Debounce);
        assert_eq!(controller.numeric_param(), 6);
    }

    #[test]
    fn test_hysteresis_controller() {
        let controller = Controller::hysteresis(0.8, 0.3);
        assert_eq!(controller.controller_type(), ControllerType::Hysteresis);
        assert_eq!(controller.float_param1(), 0.8);
        assert_eq!(controller.float_param2(), 0.3);
    }

    #[test]
    fn test_threshold_controller() {
        let controller = Controller::threshold(0.5);
        assert_eq!(controller.controller_type(), ControllerType::Threshold);
        assert_eq!(controller.float_param1(), 0.5);
    }

    #[test]
    fn test_pattern_controller() {
        let controller = Controller::pattern(0b00100100);
        assert_eq!(controller.controller_type(), ControllerType::Pattern);
        assert_eq!(controller.numeric_param(), 0b00100100);
    }

    #[test]
    fn test_custom_controller() {
        let controller = Controller::custom("test".to_string(), "code".to_string());
        assert_eq!(controller.controller_type(), ControllerType::Custom);
        assert!(controller.is_custom());
        assert_eq!(controller.custom_name(), Some("test".to_string()));
        assert_eq!(controller.custom_code(), Some("code".to_string()));
    }

    #[test]
    fn test_controller_default() {
        let controller = Controller::default();
        assert_eq!(controller.controller_type(), ControllerType::Direct);
    }

    #[test]
    fn test_controller_clone() {
        let controller1 = Controller::and(SensorType::MouseOver);
        let controller2 = controller1.clone();
        assert_eq!(controller1, controller2);
    }

    #[test]
    fn test_to_core_controller_direct() {
        let wasm = Controller::direct();
        let core: CoreController = wasm.into();
        assert!(matches!(core, CoreController::Direct));
    }

    #[test]
    fn test_to_core_controller_not() {
        let wasm = Controller::not();
        let core: CoreController = wasm.into();
        assert!(matches!(core, CoreController::NOT));
    }

    #[test]
    fn test_to_core_controller_blinky() {
        let wasm = Controller::blinky(4);
        let core: CoreController = wasm.into();
        assert!(matches!(core, CoreController::Blinky { interval: 4 }));
    }

    #[test]
    fn test_to_core_controller_debounce() {
        let wasm = Controller::debounce(6);
        let core: CoreController = wasm.into();
        assert!(matches!(core, CoreController::Debounce { ticks: 6 }));
    }

    #[test]
    fn test_to_core_controller_hysteresis() {
        let wasm = Controller::hysteresis(0.8, 0.3);
        let core: CoreController = wasm.into();
        match core {
            CoreController::Hysteresis { high, low } => {
                assert_eq!(high, 0.8);
                assert_eq!(low, 0.3);
            }
            _ => panic!("Expected Hysteresis controller"),
        }
    }

    #[test]
    fn test_to_core_controller_threshold() {
        let wasm = Controller::threshold(0.5);
        let core: CoreController = wasm.into();
        match core {
            CoreController::Threshold { value } => {
                assert_eq!(value, 0.5);
            }
            _ => panic!("Expected Threshold controller"),
        }
    }

    #[test]
    fn test_to_core_controller_pattern() {
        let wasm = Controller::pattern(0b00100100);
        let core: CoreController = wasm.into();
        match core {
            CoreController::Pattern { mask } => {
                assert_eq!(mask, 0b00100100);
            }
            _ => panic!("Expected Pattern controller"),
        }
    }

    #[test]
    fn test_to_core_controller_custom() {
        let wasm = Controller::custom("test_name".to_string(), "test_code".to_string());
        let core: CoreController = wasm.into();
        match core {
            CoreController::Custom { name, code } => {
                assert_eq!(name, "test_name");
                assert_eq!(code, "test_code");
            }
            _ => panic!("Expected Custom controller"),
        }
    }
}
