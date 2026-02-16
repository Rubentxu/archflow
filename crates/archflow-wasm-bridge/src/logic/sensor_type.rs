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

    /// Always sensor - constantly active every frame
    Always = 9,

    /// Property sensor - detects changes in entity properties
    Property = 10,

    /// Ray sensor - line of sight detection
    Ray = 11,

    /// Timer sensor - activates after a delay
    Timer = 12,

    /// Channel sensor - listens for messages on a channel
    Channel = 13,
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
            CoreSensorType::Always => SensorType::Always,
            CoreSensorType::Property => SensorType::Property,
            CoreSensorType::Ray => SensorType::Ray,
            CoreSensorType::Timer => SensorType::Timer,
            CoreSensorType::Channel => SensorType::Channel,
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
            SensorType::Always => CoreSensorType::Always,
            SensorType::Property => CoreSensorType::Property,
            SensorType::Ray => CoreSensorType::Ray,
            SensorType::Timer => CoreSensorType::Timer,
            SensorType::Channel => CoreSensorType::Channel,
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
        assert_eq!(SensorType::Always as u8, 9);
        assert_eq!(SensorType::Property as u8, 10);
        assert_eq!(SensorType::Ray as u8, 11);
        assert_eq!(SensorType::Timer as u8, 12);
        assert_eq!(SensorType::Channel as u8, 13);
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

        // Test Always sensor
        let always_wasm = SensorType::Always;
        let always_core: CoreSensorType = always_wasm.into();
        assert_eq!(always_core, CoreSensorType::Always);

        // Test Property sensor
        let property_wasm = SensorType::Property;
        let property_core: CoreSensorType = property_wasm.into();
        assert_eq!(property_core, CoreSensorType::Property);

        // Test Ray sensor
        let ray_wasm = SensorType::Ray;
        let ray_core: CoreSensorType = ray_wasm.into();
        assert_eq!(ray_core, CoreSensorType::Ray);

        // Test Timer sensor
        let timer_wasm = SensorType::Timer;
        let timer_core: CoreSensorType = timer_wasm.into();
        assert_eq!(timer_core, CoreSensorType::Timer);

        // Test Channel sensor
        let channel_wasm = SensorType::Channel;
        let channel_core: CoreSensorType = channel_wasm.into();
        assert_eq!(channel_core, CoreSensorType::Channel);
    }

    #[test]
    fn test_new_sensor_types_from_core() {
        // Test Always sensor conversion from core
        let always_core = CoreSensorType::Always;
        let always_wasm: SensorType = always_core.into();
        assert_eq!(always_wasm, SensorType::Always);

        // Test Property sensor conversion from core
        let property_core = CoreSensorType::Property;
        let property_wasm: SensorType = property_core.into();
        assert_eq!(property_wasm, SensorType::Property);

        // Test Ray sensor conversion from core
        let ray_core = CoreSensorType::Ray;
        let ray_wasm: SensorType = ray_core.into();
        assert_eq!(ray_wasm, SensorType::Ray);

        // Test Timer sensor conversion from core
        let timer_core = CoreSensorType::Timer;
        let timer_wasm: SensorType = timer_core.into();
        assert_eq!(timer_wasm, SensorType::Timer);

        // Test Channel sensor conversion from core
        let channel_core = CoreSensorType::Channel;
        let channel_wasm: SensorType = channel_core.into();
        assert_eq!(channel_wasm, SensorType::Channel);
    }
}
