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

    /// AABB collision between entities
    Touch = 4,

    /// Entity in directional cone (radar)
    Radar = 5,

    /// Rapid double-click pattern detected
    DoubleTap = 6,

    /// Mouse button held down (long press)
    LongPress = 7,

    /// Right mouse button click
    RightClick = 8,
}

impl From<CoreSensorType> for SensorType {
    fn from(core: CoreSensorType) -> Self {
        match core {
            CoreSensorType::MouseOver => SensorType::MouseOver,
            CoreSensorType::MouseClick => SensorType::MouseClick,
            CoreSensorType::Proximity => SensorType::Proximity,
            CoreSensorType::KeyShortcut => SensorType::KeyShortcut,
            CoreSensorType::Touch => SensorType::Touch,
            CoreSensorType::Radar => SensorType::Radar,
            CoreSensorType::DoubleTap => SensorType::DoubleTap,
            CoreSensorType::LongPress => SensorType::LongPress,
            CoreSensorType::RightClick => SensorType::RightClick,
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
            SensorType::Touch => CoreSensorType::Touch,
            SensorType::Radar => CoreSensorType::Radar,
            SensorType::DoubleTap => CoreSensorType::DoubleTap,
            SensorType::LongPress => CoreSensorType::LongPress,
            SensorType::RightClick => CoreSensorType::RightClick,
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
        assert_eq!(SensorType::Touch as u8, 4);
        assert_eq!(SensorType::Radar as u8, 5);
        assert_eq!(SensorType::DoubleTap as u8, 6);
        assert_eq!(SensorType::LongPress as u8, 7);
        assert_eq!(SensorType::RightClick as u8, 8);
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

    #[test]
    fn test_new_sensor_types() {
        // Test the new sensor types can be converted
        let touch_wasm = SensorType::Touch;
        let touch_core: CoreSensorType = touch_wasm.into();
        assert_eq!(touch_core, CoreSensorType::Touch);

        let radar_wasm = SensorType::Radar;
        let radar_core: CoreSensorType = radar_wasm.into();
        assert_eq!(radar_core, CoreSensorType::Radar);

        let double_tap_wasm = SensorType::DoubleTap;
        let double_tap_core: CoreSensorType = double_tap_wasm.into();
        assert_eq!(double_tap_core, CoreSensorType::DoubleTap);

        let long_press_wasm = SensorType::LongPress;
        let long_press_core: CoreSensorType = long_press_wasm.into();
        assert_eq!(long_press_core, CoreSensorType::LongPress);

        let right_click_wasm = SensorType::RightClick;
        let right_click_core: CoreSensorType = right_click_wasm.into();
        assert_eq!(right_click_core, CoreSensorType::RightClick);
    }
}
