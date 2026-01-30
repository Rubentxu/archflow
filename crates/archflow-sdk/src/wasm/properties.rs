//! WASM bindings for properties module
//!
//! Provides WebAssembly bindings for property editing

use crate::properties::PropertiesManager;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// WASM-exposed properties manager
#[wasm_bindgen]
pub struct JsPropertiesManager {
    inner: PropertiesManager,
}

#[wasm_bindgen]
impl JsPropertiesManager {
    /// Creates a new properties manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: PropertiesManager::new(),
        }
    }

    /// Returns true if a single shape is selected
    #[wasm_bindgen(getter = isSingleSelection)]
    pub fn is_single_selection(&self) -> bool {
        self.inner.is_single_selection()
    }

    /// Returns true if multiple shapes are selected
    #[wasm_bindgen(getter = isMultiSelection)]
    pub fn is_multi_selection(&self) -> bool {
        self.inner.is_multi_selection()
    }

    /// Updates a shape's properties
    #[wasm_bindgen]
    pub fn update_shape(&self, _shape_id: &str, _changes_json: String) -> bool {
        // TODO: Implement using PropertiesManager API
        false
    }

    /// Gets the current property values for a shape
    #[wasm_bindgen]
    pub fn get_properties(&self, _shape_id: &str) -> String {
        // TODO: Implement using PropertiesManager API
        "{}".to_string()
    }

    /// Sets the fill color of a shape
    #[wasm_bindgen]
    pub fn set_fill_color(&self, _shape_id: &str, _color: String) -> bool {
        // TODO: Implement using PropertiesManager API
        false
    }

    /// Sets the stroke color of a shape
    #[wasm_bindgen]
    pub fn set_stroke_color(&self, _shape_id: &str, _color: String) -> bool {
        // TODO: Implement using PropertiesManager API
        false
    }

    /// Sets the stroke width of a shape
    #[wasm_bindgen]
    pub fn set_stroke_width(&self, _shape_id: &str, _width: f32) -> bool {
        // TODO: Implement using PropertiesManager API
        false
    }

    /// Sets the opacity of a shape
    #[wasm_bindgen]
    pub fn set_opacity(&self, _shape_id: &str, _opacity: f32) -> bool {
        // TODO: Implement using PropertiesManager API
        false
    }

    /// Sets the lock aspect ratio flag
    #[wasm_bindgen]
    pub fn set_lock_aspect_ratio(&self, _locked: bool) {
        // TODO: Implement using PropertiesManager API
    }

    /// Resets properties to default values
    #[wasm_bindgen]
    pub fn reset_properties(&self, _shape_id: &str) -> bool {
        // TODO: Implement using PropertiesManager API
        false
    }
}

/// Color representation for WASM
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct JsColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[wasm_bindgen]
impl JsColor {
    /// Creates a new color from RGBA values
    #[wasm_bindgen(constructor)]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from hex string
    #[wasm_bindgen]
    pub fn from_hex(hex: &str) -> Result<JsColor, JsValue> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(JsValue::from_str("Invalid hex color"));
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red")? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green")? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue")? as f32 / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid alpha")? as f32 / 255.0
        } else {
            1.0
        };

        Ok(Self { r, g, b, a })
    }

    /// Converts to CSS rgba string
    #[wasm_bindgen]
    pub fn to_css(&self) -> String {
        format!(
            "rgba({},{},{},{})",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            self.a
        )
    }
}

/// Property change representation
#[derive(Clone, Debug)]
pub struct JsPropertyChange {
    pub property: String,
    pub old_value: String, // JSON string
    pub new_value: String, // JSON string
}

/// TypeScript definitions for properties module
pub const PROPERTIES_TYPES: &str = r#"
/**
 * Properties Manager for WASM
 */
export class JsPropertiesManager {
    constructor();
    readonly isSingleSelection: boolean;
    readonly isMultiSelection: boolean;
    updateShape(shapeId: string, changes: string): boolean;
    getProperties(shapeId: string): string;
    setFillColor(shapeId: string, color: string): boolean;
    setStrokeColor(shapeId: string, color: string): boolean;
    setStrokeWidth(shapeId: string, width: number): boolean;
    setOpacity(shapeId: string, opacity: number): boolean;
    setLockAspectRatio(locked: boolean): void;
    resetProperties(shapeId: string): boolean;
}

export class JsColor {
    constructor(r: number, g: number, b: number, a: number);
    static fromHex(hex: string): JsColor;
    toCss(): string;
    r: number;
    g: number;
    b: number;
    a: number;
}

export type PropertyType =
    | 'x'
    | 'y'
    | 'width'
    | 'height'
    | 'rotation'
    | 'fillColor'
    | 'strokeColor'
    | 'strokeWidth'
    | 'opacity';

export interface PropertyChange {
    property: PropertyType;
    oldValue: any;
    newValue: any;
}
"#;

/// Get TypeScript definitions for properties
#[wasm_bindgen]
pub fn get_properties_typescript_definitions() -> String {
    PROPERTIES_TYPES.to_string()
}
