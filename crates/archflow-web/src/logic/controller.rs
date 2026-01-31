// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Controller WASM Binding
//
// Epic 5.3: Expose Controller to JavaScript/TypeScript
//
// Provides a JavaScript-accessible enum for boolean logic controllers
// that combine sensor signals using AND, OR, NOT, or Direct logic.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use super::SensorType;
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
}

/// Controller for boolean logic operations on sensor signals
///
/// This struct wraps the ControllerType enum with optional secondary sensor
/// for AND/OR operations.
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
    /// The type of controller (Direct, AND, OR, NOT)
    controller_type: ControllerType,

    /// The secondary sensor for AND/OR operations (optional)
    secondary_sensor: Option<SensorType>,
}

#[wasm_bindgen]
impl Controller {
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
        }
    }

    /// Returns the controller type
    #[wasm_bindgen]
    pub fn controller_type(&self) -> ControllerType {
        self.controller_type
    }

    /// Returns the secondary sensor (if any)
    ///
    /// Returns `null` if there is no secondary sensor (Direct or NOT controllers).
    #[wasm_bindgen]
    pub fn secondary_sensor(&self) -> Option<SensorType> {
        self.secondary_sensor
    }

    /// Checks if this controller has a secondary sensor
    #[wasm_bindgen]
    pub fn has_secondary_sensor(&self) -> bool {
        self.secondary_sensor.is_some()
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
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(core, CoreController::Direct);
    }

    #[test]
    fn test_to_core_controller_not() {
        let wasm = Controller::not();
        let core: CoreController = wasm.into();
        assert_eq!(core, CoreController::NOT);
    }
}
