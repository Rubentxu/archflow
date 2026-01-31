// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - SensorType WASM Binding
//
// Epic 5.2: Expose SensorType to JavaScript/TypeScript
//
// Provides a JavaScript-accessible enum for sensor types in the Logic Bricks system.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use archflow_logic::mapping::SensorType as CoreSensorType;
use wasm_bindgen::prelude::*;

/// Sensor types for the Logic Bricks system
///
/// # JavaScript Example
/// ```javascript
/// import { SensorType } from '@archflow/sdk';
///
/// const sensor = SensorType.MouseOver;
/// ```
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorType {
    /// Mouse is hovering over an entity
    MouseOver = 0,

    /// Mouse button was clicked on an entity
    MouseClick = 1,

    /// Another entity is within proximity radius
    Proximity = 2,

    /// Keyboard shortcut was pressed
    KeyShortcut = 3,
}

impl From<CoreSensorType> for SensorType {
    fn from(core: CoreSensorType) -> Self {
        match core {
            CoreSensorType::MouseOver => SensorType::MouseOver,
            CoreSensorType::MouseClick => SensorType::MouseClick,
            CoreSensorType::Proximity => SensorType::Proximity,
            CoreSensorType::KeyShortcut => SensorType::KeyShortcut,
        }
    }
}

impl From<SensorType> for CoreSensorType {
    fn from(wasm: SensorType) -> Self {
        match wasm {
            SensorType::MouseOver => CoreSensorType::MouseOver,
            SensorType::MouseClick => CoreSensorType::MouseClick,
            SensorType::Proximity => CoreSensorType::Proximity,
            SensorType::KeyShortcut => CoreSensorType::KeyShortcut,
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
    fn test_sensor_type_values() {
        assert_eq!(SensorType::MouseOver as u8, 0);
        assert_eq!(SensorType::MouseClick as u8, 1);
        assert_eq!(SensorType::Proximity as u8, 2);
        assert_eq!(SensorType::KeyShortcut as u8, 3);
    }

    #[test]
    fn test_sensor_type_copy() {
        let sensor1 = SensorType::MouseOver;
        let sensor2 = sensor1;
        assert_eq!(sensor1, sensor2);
    }

    #[test]
    fn test_sensor_type_equality() {
        assert_eq!(SensorType::MouseOver, SensorType::MouseOver);
        assert_ne!(SensorType::MouseOver, SensorType::MouseClick);
    }

    #[test]
    fn test_from_core_sensor_type() {
        let core = CoreSensorType::MouseOver;
        let wasm: SensorType = core.into();
        assert_eq!(wasm, SensorType::MouseOver);
    }

    #[test]
    fn test_to_core_sensor_type() {
        let wasm = SensorType::Proximity;
        let core: CoreSensorType = wasm.into();
        assert_eq!(core, CoreSensorType::Proximity);
    }
}
