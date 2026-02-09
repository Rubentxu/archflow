// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Actuator WASM Bindings
//
// Epic 5.5: Expose Actuators to JavaScript/TypeScript
//
// Provides JavaScript-accessible wrappers for all actuator types
// that can be triggered by sensor signals through controllers.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::string::String;
use archflow_logic::actuators::{
    BatchSelectActuator, HighlightActuator, MoveActuator, PropertyActuator,
    SelectMode as CoreSelectMode, StateActuator,
};
use wasm_bindgen::prelude::*;

/// Configuration for highlight actuator
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct HighlightConfig {
    color: u32,
    restore_color: u32,
    opacity: f32,
}

#[wasm_bindgen]
impl HighlightConfig {
    /// Creates a new highlight configuration
    #[wasm_bindgen(constructor)]
    pub fn new(color: u32, restore_color: u32, opacity: f32) -> Self {
        Self {
            color,
            restore_color,
            opacity,
        }
    }

    /// Get the highlight color (ARGB)
    #[wasm_bindgen]
    pub fn color(&self) -> u32 {
        self.color
    }

    /// Get the restore color (ARGB)
    #[wasm_bindgen]
    pub fn restore_color(&self) -> u32 {
        self.restore_color
    }

    /// Get the opacity (0.0 - 1.0)
    #[wasm_bindgen]
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
}

/// Select mode for selection actuator (matches core SelectMode)
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectModeWasm {
    /// Single selection (replaces current selection)
    Single = 0,

    /// Multi selection (adds to current selection)
    Multi = 1,

    /// Replace selection (clears and selects new)
    Replace = 2,

    /// Toggle selection (inverts selection state)
    Toggle = 3,

    /// Add to selection (ensure selected)
    Add = 4,

    /// Subtract from selection (ensure deselected)
    Subtract = 5,
}

impl From<SelectModeWasm> for CoreSelectMode {
    fn from(wasm: SelectModeWasm) -> Self {
        match wasm {
            SelectModeWasm::Single => CoreSelectMode::Single,
            SelectModeWasm::Multi => CoreSelectMode::Multi,
            SelectModeWasm::Replace => CoreSelectMode::Replace,
            SelectModeWasm::Toggle => CoreSelectMode::Toggle,
            SelectModeWasm::Add => CoreSelectMode::Add,
            SelectModeWasm::Subtract => CoreSelectMode::Subtract,
        }
    }
}

impl From<CoreSelectMode> for SelectModeWasm {
    fn from(core: CoreSelectMode) -> Self {
        match core {
            CoreSelectMode::Single => SelectModeWasm::Single,
            CoreSelectMode::Multi => SelectModeWasm::Multi,
            CoreSelectMode::Replace => SelectModeWasm::Replace,
            CoreSelectMode::Toggle => SelectModeWasm::Toggle,
            CoreSelectMode::Add => SelectModeWasm::Add,
            CoreSelectMode::Subtract => SelectModeWasm::Subtract,
        }
    }
}

/// Configuration for move actuator
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct MoveConfig {
    snap: f32,
    constrain_x: bool,
    constrain_y: bool,
}

#[wasm_bindgen]
impl MoveConfig {
    /// Creates a new move configuration
    #[wasm_bindgen(constructor)]
    pub fn new(snap: f32, constrain_x: bool, constrain_y: bool) -> Self {
        Self {
            snap,
            constrain_x,
            constrain_y,
        }
    }

    /// Get snap value in pixels
    #[wasm_bindgen]
    pub fn snap(&self) -> f32 {
        self.snap
    }

    /// Whether X axis is constrained
    #[wasm_bindgen]
    pub fn constrain_x(&self) -> bool {
        self.constrain_x
    }

    /// Whether Y axis is constrained
    #[wasm_bindgen]
    pub fn constrain_y(&self) -> bool {
        self.constrain_y
    }
}

/// Configuration for camera actuator
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct CameraConfig {
    target_x: f32,
    target_y: f32,
    zoom: f32,
    duration_ms: u32,
    smooth: f32,
}

#[wasm_bindgen]
impl CameraConfig {
    /// Creates a new camera configuration
    #[wasm_bindgen(constructor)]
    pub fn new(target_x: f32, target_y: f32, zoom: f32, duration_ms: u32, smooth: f32) -> Self {
        Self {
            target_x,
            target_y,
            zoom,
            duration_ms,
            smooth,
        }
    }

    /// Get target X position
    #[wasm_bindgen]
    pub fn target_x(&self) -> f32 {
        self.target_x
    }

    /// Get target Y position
    #[wasm_bindgen]
    pub fn target_y(&self) -> f32 {
        self.target_y
    }

    /// Get zoom level
    #[wasm_bindgen]
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Get duration in milliseconds
    #[wasm_bindgen]
    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    /// Get smoothing factor (0.0 - 1.0)
    #[wasm_bindgen]
    pub fn smooth(&self) -> f32 {
        self.smooth
    }
}

/// Property value wrapper for WASM
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PropertyValue {
    inner: String,
}

#[wasm_bindgen]
impl PropertyValue {
    /// Create a string property value
    #[wasm_bindgen]
    pub fn from_string(value: String) -> Self {
        Self { inner: value }
    }

    /// Create a number property value
    #[wasm_bindgen]
    pub fn from_number(value: f64) -> Self {
        Self {
            inner: alloc::format!("{}", value),
        }
    }

    /// Create a boolean property value
    #[wasm_bindgen]
    pub fn from_bool(value: bool) -> Self {
        Self {
            inner: alloc::format!("{}", value),
        }
    }

    /// Get the raw value string
    #[wasm_bindgen]
    pub fn value(&self) -> String {
        self.inner.clone()
    }
}

/// Configuration for property actuator
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct PropertyConfig {
    property_name: String,
    value: PropertyValue,
}

#[wasm_bindgen]
impl PropertyConfig {
    /// Creates a new property configuration
    #[wasm_bindgen(constructor)]
    pub fn new(property_name: String, value: PropertyValue) -> Self {
        Self {
            property_name,
            value,
        }
    }

    /// Get property name
    #[wasm_bindgen]
    pub fn property_name(&self) -> String {
        self.property_name.clone()
    }

    /// Get property value
    #[wasm_bindgen]
    pub fn value(&self) -> PropertyValue {
        PropertyValue {
            inner: self.value.inner.clone(),
        }
    }
}

/// Extended actuator types for the Logic Bricks system
///
/// # JavaScript Example
/// ```javascript
/// import { ExtendedActuatorType } from '@archflow/sdk';
///
/// const highlight = ExtendedActuatorType.Highlight;
/// const camera = ExtendedActuatorType.Camera;
/// ```
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtendedActuatorType {
    /// Highlight actuator - changes entity color
    Highlight = 0,

    /// Select actuator - marks entity as selected
    Select = 1,

    /// Move actuator - moves entity (drag operation)
    Move = 2,

    /// Camera actuator - moves camera
    Camera = 3,

    /// Property actuator - sets entity property
    Property = 4,

    /// State actuator - changes entity state
    State = 5,
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_logic::SelectMode;

    #[test]
    fn test_highlight_config() {
        let config = HighlightConfig::new(0xFF00FF00, 0xFF0000FF, 0.5);
        assert_eq!(config.color(), 0xFF00FF00);
        assert_eq!(config.restore_color(), 0xFF0000FF);
        assert_eq!(config.opacity(), 0.5);
    }

    #[test]
    fn test_select_mode_conversion() {
        assert_eq!(SelectMode::Single, SelectModeWasm::Single.into());
        assert_eq!(SelectMode::Multi, SelectModeWasm::Multi.into());
        assert_eq!(SelectMode::Replace, SelectModeWasm::Replace.into());
        assert_eq!(SelectMode::Toggle, SelectModeWasm::Toggle.into());

        assert_eq!(SelectModeWasm::Single, SelectMode::Single.into());
        assert_eq!(SelectModeWasm::Multi, SelectMode::Multi.into());
        assert_eq!(SelectModeWasm::Replace, SelectMode::Replace.into());
        assert_eq!(SelectModeWasm::Toggle, SelectMode::Toggle.into());
    }

    #[test]
    fn test_move_config() {
        let config = MoveConfig::new(8.0, true, false);
        assert_eq!(config.snap(), 8.0);
        assert!(config.constrain_x());
        assert!(!config.constrain_y());
    }

    #[test]
    fn test_camera_config() {
        let config = CameraConfig::new(100.0, 200.0, 1.5, 300, 0.8);
        assert_eq!(config.target_x(), 100.0);
        assert_eq!(config.target_y(), 200.0);
        assert_eq!(config.zoom(), 1.5);
        assert_eq!(config.duration_ms(), 300);
        assert_eq!(config.smooth(), 0.8);
    }

    #[test]
    fn test_property_value() {
        let string_val = PropertyValue::from_string(alloc::format!("test"));
        assert_eq!(string_val.value(), "test");

        let number_val = PropertyValue::from_number(42.5);
        assert_eq!(number_val.value(), "42.5");

        let bool_val = PropertyValue::from_bool(true);
        assert_eq!(bool_val.value(), "true");
    }

    #[test]
    fn test_property_config() {
        let value = PropertyValue::from_string(alloc::format!("value"));
        let config = PropertyConfig::new(alloc::format!("prop"), value);
        assert_eq!(config.property_name(), "prop");
        assert_eq!(config.value().value(), "value");
    }
}
