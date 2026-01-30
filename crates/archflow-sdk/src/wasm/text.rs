//! WASM bindings for text module
//!
//! Provides WebAssembly bindings for text tool operations

use crate::text::TextManager;
use wasm_bindgen::prelude::*;

/// WASM-exposed text manager
#[wasm_bindgen]
pub struct JsTextManager {
    inner: TextManager,
}

#[wasm_bindgen]
impl JsTextManager {
    /// Creates a new text manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: TextManager::new(),
        }
    }

    /// Returns true if any text is being edited
    #[wasm_bindgen(getter = isEditing)]
    pub fn is_editing(&self) -> bool {
        self.inner.is_editing()
    }

    /// Gets the number of text entities
    #[wasm_bindgen(getter = textCount)]
    pub fn text_count(&self) -> usize {
        self.inner.text_count()
    }
}

/// WASM-exposed text style
#[wasm_bindgen]
#[derive(Clone)]
pub struct JsTextStyle {
    pub font_size: f32,
    pub font_weight: u16,
    pub italic: bool,
    font_family: String,
    alignment: String,
}

#[wasm_bindgen]
impl JsTextStyle {
    /// Creates a new text style
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: 400,
            italic: false,
            alignment: "left".to_string(),
        }
    }

    /// Gets the font family
    #[wasm_bindgen(getter)]
    pub fn font_family(&self) -> String {
        self.font_family.clone()
    }

    /// Sets the font family
    #[wasm_bindgen(setter)]
    pub fn set_font_family(&mut self, family: &str) {
        self.font_family = family.to_string();
    }

    /// Gets the alignment
    #[wasm_bindgen(getter)]
    pub fn alignment(&self) -> String {
        self.alignment.clone()
    }

    /// Sets the alignment
    #[wasm_bindgen(setter)]
    pub fn set_alignment(&mut self, alignment: &str) {
        self.alignment = alignment.to_string();
    }
}

/// TypeScript definitions
#[wasm_bindgen]
pub fn get_text_typescript_definitions() -> String {
    r#"
// Text Manager
export class JsTextManager {
    constructor();
    readonly isEditing: boolean;
    readonly textCount: number;
}

export class JsTextStyle {
    constructor();
    fontFamily: string;
    fontSize: number;
    fontWeight: number;
    italic: boolean;
    alignment: TextAlignment;
}

export type TextAlignment = 'left' | 'center' | 'right' | 'justify';

export const TextAlignments: {
    readonly LEFT: 'left';
    readonly CENTER: 'center';
    readonly RIGHT: 'right';
    readonly JUSTIFY: 'justify';
};
"#
    .to_string()
}
