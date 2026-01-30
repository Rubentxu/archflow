//! WASM bindings for alignment module
//!
//! Provides WebAssembly bindings for alignment and distribution operations

use crate::canvas::Canvas;
use archflow_core::EntityId;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// WASM-exposed alignment manager
#[wasm_bindgen]
pub struct JsAlignmentManager {
    canvas: Option<Canvas>,
}

#[wasm_bindgen]
impl JsAlignmentManager {
    /// Creates a new alignment manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { canvas: None }
    }

    /// Sets the canvas reference for alignment operations
    #[wasm_bindgen]
    pub fn set_canvas(&mut self, _canvas_ptr: usize) {
        // Pointer-based canvas reference for WASM
    }

    /// Aligns multiple shapes to the given alignment type
    #[wasm_bindgen]
    pub fn align(&self, shape_ids: Vec<String>, _alignment_type: String) -> bool {
        if shape_ids.len() < 2 {
            return false;
        }

        let entity_ids: Vec<EntityId> = shape_ids
            .into_iter()
            .filter_map(|id| EntityId::from_str(&id))
            .collect();

        if entity_ids.len() < 2 {
            return false;
        }

        // Alignment would be performed here using the canvas
        // For now, this is a placeholder implementation
        true
    }

    /// Distributes shapes evenly
    #[wasm_bindgen]
    pub fn distribute(&self, shape_ids: Vec<String>, _axis: String) -> bool {
        if shape_ids.len() < 3 {
            return false;
        }

        let entity_ids: Vec<EntityId> = shape_ids
            .into_iter()
            .filter_map(|id| EntityId::from_str(&id))
            .collect();

        if entity_ids.len() < 3 {
            return false;
        }

        // Distribution would be performed here
        true
    }

    /// Aligns shapes to the left edge of the selection bounds
    #[wasm_bindgen]
    pub fn align_left(&self, shape_ids: Vec<String>) -> bool {
        self.align(shape_ids, "left".to_string())
    }

    /// Aligns shapes to the center horizontally
    #[wasm_bindgen]
    pub fn align_center_horizontal(&self, shape_ids: Vec<String>) -> bool {
        self.align(shape_ids, "centerHorizontal".to_string())
    }

    /// Aligns shapes to the right edge of the selection bounds
    #[wasm_bindgen]
    pub fn align_right(&self, shape_ids: Vec<String>) -> bool {
        self.align(shape_ids, "right".to_string())
    }

    /// Aligns shapes to the top edge of the selection bounds
    #[wasm_bindgen]
    pub fn align_top(&self, shape_ids: Vec<String>) -> bool {
        self.align(shape_ids, "top".to_string())
    }

    /// Aligns shapes to the center vertically
    #[wasm_bindgen]
    pub fn align_center_vertical(&self, shape_ids: Vec<String>) -> bool {
        self.align(shape_ids, "centerVertical".to_string())
    }

    /// Aligns shapes to the bottom edge of the selection bounds
    #[wasm_bindgen]
    pub fn align_bottom(&self, shape_ids: Vec<String>) -> bool {
        self.align(shape_ids, "bottom".to_string())
    }
}

/// TypeScript definitions for alignment module
pub const ALIGNMENT_TYPES: &str = r#"
/**
 * Alignment Manager for WASM
 */
export class JsAlignmentManager {
    constructor();
    setCanvas(canvasPtr: number): void;
    align(shapeIds: string[], alignmentType: string): boolean;
    distribute(shapeIds: string[], axis: string): boolean;
    alignLeft(shapeIds: string[]): boolean;
    alignCenterHorizontal(shapeIds: string[]): boolean;
    alignRight(shapeIds: string[]): boolean;
    alignTop(shapeIds: string[]): boolean;
    alignCenterVertical(shapeIds: string[]): boolean;
    alignBottom(shapeIds: string[]): boolean;
}

export type AlignmentType =
    | 'left'
    | 'centerHorizontal'
    | 'right'
    | 'top'
    | 'centerVertical'
    | 'bottom';

export type DistributionAxis = 'horizontal' | 'vertical';
"#;

/// Get TypeScript definitions for alignment
#[wasm_bindgen]
pub fn get_alignment_typescript_definitions() -> String {
    ALIGNMENT_TYPES.to_string()
}
