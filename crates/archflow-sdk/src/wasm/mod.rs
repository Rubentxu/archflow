//! WASM bindings for ArchFlow SDK
//!
//! This module provides WebAssembly bindings for the ArchFlow SDK,
//! enabling JavaScript/TypeScript integration.

pub mod alignment;
pub mod animation;
pub mod group;
pub mod keyboard;
pub mod layers;
pub mod library;
pub mod properties;
pub mod text;

use crate::background::{GridConfig, GridType};
use crate::canvas::{Canvas, Shape, ShapeChanges};
use crate::layers::C4Level;
use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use ts_rs::TS;
use wasm_bindgen::prelude::*;

/// Represents a point in 2D space for JS interop
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsVec2")]
pub struct JsVec2 {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for JsVec2 {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<JsVec2> for Vec2 {
    fn from(v: JsVec2) -> Self {
        Vec2::new(v.x, v.y)
    }
}

/// Represents a color for JS interop
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsColor")]
pub struct JsColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Represents a shape for JS interop
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsShape")]
pub struct JsShape {
    pub id: String,
    pub shape_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub fill_color: JsColor,
    pub stroke_color: Option<JsColor>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub layer_id: String,
    pub selected: bool,
}

impl From<&Shape> for JsShape {
    fn from(s: &Shape) -> Self {
        Self {
            id: s.id.to_string(),
            shape_type: format!("{:?}", s.shape_type),
            x: s.x,
            y: s.y,
            width: s.width,
            height: s.height,
            rotation: s.rotation,
            fill_color: JsColor {
                r: s.fill_color.r,
                g: s.fill_color.g,
                b: s.fill_color.b,
                a: s.fill_color.a,
            },
            stroke_color: s.stroke_color.map(|c| JsColor {
                r: c.r,
                g: c.g,
                b: c.b,
                a: c.a,
            }),
            stroke_width: s.stroke_width,
            opacity: s.opacity,
            layer_id: s.layer_id.to_string(),
            selected: s.selected,
        }
    }
}

/// Represents selection for JS interop
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsSelection")]
pub struct JsSelection {
    pub shape_ids: Vec<String>,
    pub bounds: JsRect,
    pub is_box: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsRect")]
pub struct JsRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Main editor instance exposed to JavaScript
#[wasm_bindgen]
pub struct ArchFlowEditor {
    canvas: RefCell<Canvas>,
}

#[wasm_bindgen]
impl ArchFlowEditor {
    /// Creates a new editor instance
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            canvas: RefCell::new(Canvas::new(width, height)),
        }
    }

    // === Viewport Operations ===

    /// Pans the viewport
    #[wasm_bindgen]
    pub fn pan(&self, dx: f32, dy: f32) {
        self.canvas.borrow_mut().pan(Vec2::new(dx, dy));
    }

    /// Zooms at a point
    #[wasm_bindgen]
    pub fn zoom_at(&self, screen_x: f32, screen_y: f32, factor: f32) {
        self.canvas
            .borrow_mut()
            .zoom_at(Vec2::new(screen_x, screen_y), factor);
    }

    /// Zooms to fit all content
    #[wasm_bindgen]
    pub fn zoom_to_fit(&self) {
        self.canvas.borrow_mut().zoom_to_fit();
    }

    // === Shape Operations ===

    /// Creates a rectangle
    #[wasm_bindgen]
    pub fn create_rectangle(&self, x: f32, y: f32, width: f32, height: f32) -> String {
        let id = self
            .canvas
            .borrow_mut()
            .create_rectangle(x, y, width, height);
        id.to_string()
    }

    /// Creates an ellipse
    #[wasm_bindgen]
    pub fn create_ellipse(&self, x: f32, y: f32, radius_x: f32, radius_y: f32) -> String {
        let id = self
            .canvas
            .borrow_mut()
            .create_ellipse(x, y, radius_x, radius_y);
        id.to_string()
    }

    /// Creates a line
    #[wasm_bindgen]
    pub fn create_line(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> String {
        let id = self.canvas.borrow_mut().create_line(x1, y1, x2, y2);
        id.to_string()
    }

    /// Gets a shape by ID
    #[wasm_bindgen]
    pub fn get_shape(&self, id: &str) -> JsValue {
        if let Some(entity_id) = EntityId::from_str(id) {
            if let Some(shape) = self.canvas.borrow().get_shape(entity_id) {
                match serde_wasm_bindgen::to_value(&JsShape::from(shape)) {
                    Ok(val) => return val,
                    Err(_) => return JsValue::NULL,
                }
            }
        }
        JsValue::NULL
    }

    /// Updates a shape
    #[wasm_bindgen]
    pub fn update_shape(&self, id: &str, changes: JsValue) -> bool {
        if let Some(entity_id) = EntityId::from_str(id) {
            if let Ok(shape_changes) = serde_wasm_bindgen::from_value::<ShapeChanges>(changes) {
                return self
                    .canvas
                    .borrow_mut()
                    .update_shape(entity_id, shape_changes);
            }
        }
        false
    }

    /// Deletes a shape
    #[wasm_bindgen]
    pub fn delete_shape(&self, id: &str) -> bool {
        if let Some(entity_id) = EntityId::from_str(id) {
            return self.canvas.borrow_mut().delete_shape(entity_id);
        }
        false
    }

    /// Gets all shapes
    #[wasm_bindgen]
    pub fn get_all_shapes(&self) -> JsValue {
        let shapes: Vec<JsShape> = self
            .canvas
            .borrow()
            .all_shapes()
            .into_iter()
            .map(|s| JsShape::from(s))
            .collect();
        match serde_wasm_bindgen::to_value(&shapes) {
            Ok(val) => val,
            Err(_) => JsValue::NULL,
        }
    }

    // === Selection Operations ===

    /// Selects a shape
    #[wasm_bindgen]
    pub fn select(&self, id: &str) {
        if let Some(entity_id) = EntityId::from_str(id) {
            self.canvas.borrow_mut().select(entity_id);
        }
    }

    /// Selects multiple shapes
    #[wasm_bindgen]
    pub fn select_multiple(&self, ids: Vec<String>) {
        let entity_ids: Vec<EntityId> = ids
            .into_iter()
            .filter_map(|s| EntityId::from_str(&s))
            .collect();
        self.canvas.borrow_mut().select_multiple(entity_ids);
    }

    /// Clears selection
    #[wasm_bindgen]
    pub fn clear_selection(&self) {
        self.canvas.borrow_mut().clear_selection();
    }

    /// Gets current selection
    #[wasm_bindgen]
    pub fn get_selection(&self) -> JsValue {
        let canvas = self.canvas.borrow();
        let selection = canvas.selection();
        match serde_wasm_bindgen::to_value(&selection) {
            Ok(val) => val,
            Err(_) => JsValue::NULL,
        }
    }

    // === Layer Operations ===

    /// Gets current C4 level
    #[wasm_bindgen]
    pub fn get_c4_level(&self) -> String {
        format!("{:?}", self.canvas.borrow().c4_level())
    }

    /// Sets current C4 level
    #[wasm_bindgen]
    pub fn set_c4_level(&self, level: &str) {
        match level {
            "Context" => self.canvas.borrow_mut().set_c4_level(C4Level::Context),
            "Container" => self.canvas.borrow_mut().set_c4_level(C4Level::Container),
            "Component" => self.canvas.borrow_mut().set_c4_level(C4Level::Component),
            "Code" => self.canvas.borrow_mut().set_c4_level(C4Level::Code),
            _ => {}
        }
    }

    // === Grid Operations ===

    /// Gets grid configuration
    #[wasm_bindgen]
    pub fn get_grid_config(&self) -> JsValue {
        let canvas = self.canvas.borrow();
        let config = canvas.background_renderer().grid_config();
        match serde_wasm_bindgen::to_value(&config) {
            Ok(val) => val,
            Err(_) => JsValue::NULL,
        }
    }

    /// Sets grid configuration
    #[wasm_bindgen]
    pub fn set_grid_config(&self, config: JsValue) {
        if let Ok(grid_config) = serde_wasm_bindgen::from_value::<GridConfig>(config) {
            self.canvas.borrow_mut().set_grid_config(grid_config);
        }
    }

    // === Coordinate Conversion ===

    /// Converts screen to canvas coordinates
    #[wasm_bindgen]
    pub fn screen_to_canvas(&self, x: f32, y: f32) -> JsValue {
        let point = self.canvas.borrow().screen_to_canvas(Vec2::new(x, y));
        match serde_wasm_bindgen::to_value(&JsVec2::from(point)) {
            Ok(val) => val,
            Err(_) => JsValue::NULL,
        }
    }

    /// Converts canvas to screen coordinates
    #[wasm_bindgen]
    pub fn canvas_to_screen(&self, x: f32, y: f32) -> JsValue {
        let point = self.canvas.borrow().canvas_to_screen(Vec2::new(x, y));
        match serde_wasm_bindgen::to_value(&JsVec2::from(point)) {
            Ok(val) => val,
            Err(_) => JsValue::NULL,
        }
    }

    // === Canvas Dimensions ===

    /// Gets canvas dimensions
    #[wasm_bindgen]
    pub fn get_dimensions(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"width".into(), &800.0.into()).unwrap();
        js_sys::Reflect::set(&obj, &"height".into(), &600.0.into()).unwrap();
        obj.into()
    }
}

/// TypeScript type definitions for the SDK
#[wasm_bindgen]
pub fn get_typescript_definitions() -> String {
    let mut definitions = String::new();

    // Add EPIC-006 module definitions
    definitions.push_str(&keyboard::get_keyboard_typescript_definitions());
    definitions.push_str(&group::get_group_typescript_definitions());
    definitions.push_str(&alignment::get_alignment_typescript_definitions());
    definitions.push_str(&properties::get_properties_typescript_definitions());
    definitions.push_str(&text::get_text_typescript_definitions());
    definitions.push_str(animation::ANIMATION_TYPES);
    definitions.push_str(CORE_TYPES);

    definitions
}

/// Core TypeScript type definitions
const CORE_TYPES: &str = r#"

/**
 * ArchFlow SDK TypeScript Definitions
 */

export class ArchFlowEditor {
    constructor(width: number, height: number);

    // Viewport Operations
    pan(dx: number, dy: number): void;
    zoomAt(screenX: number, screenY: number, factor: number): void;
    zoomToFit(): void;

    // Shape Operations
    createRectangle(x: number, y: number, width: number, height: number): string;
    createEllipse(x: number, y: number, radiusX: number, radiusY: number): string;
    createLine(x1: number, y1: number, x2: number, y2: number): string;
    getShape(id: string): JsShape | null;
    updateShape(id: string, changes: ShapeChanges): boolean;
    deleteShape(id: string): boolean;
    getAllShapes(): JsShape[];

    // Selection Operations
    select(id: string): void;
    selectMultiple(ids: string[]): void;
    clearSelection(): void;
    getSelection(): Selection;

    // Layer Operations
    getC4Level(): string;
    setC4Level(level: string): void;

    // Grid Operations
    getGridConfig(): GridConfig;
    setGridConfig(config: GridConfig): void;

    // Coordinate Conversion
    screenToCanvas(x: number, y: number): Vec2;
    canvasToScreen(x: number, y: number): Vec2;

    // Canvas Dimensions
    getDimensions(): { width: number; height: number };
}

export interface Vec2 {
    x: number;
    y: number;
}

export interface JsShape {
    id: string;
    shapeType: string;
    x: number;
    y: number;
    width: number;
    height: number;
    rotation: number;
    fillColor: JsColor;
    strokeColor: JsColor | null;
    strokeWidth: number;
    opacity: number;
    layerId: string;
    selected: boolean;
}

export interface JsColor {
    r: number;
    g: number;
    b: number;
    a: number;
}

export interface Selection {
    shapeIds: string[];
    bounds: Rect;
    isBox: boolean;
}

export interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface ShapeChanges {
    x?: number;
    y?: number;
    width?: number;
    height?: number;
    rotation?: number;
    fillColor?: JsColor;
    strokeColor?: JsColor | null;
    strokeWidth?: number;
    opacity?: number;
}

export interface GridConfig {
    gridType: 'dots' | 'lines' | 'isometric';
    spacing: number;
    dotRadius: number;
    dotColor: JsColor;
    lineColor: JsColor;
    lineWidth: number;
    showGrid: boolean;
}
"#;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap();
}
