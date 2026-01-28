//! WASM bindings for ArchFlow SDK
//!
//! This module provides WebAssembly bindings for the ArchFlow SDK,
//! enabling JavaScript/TypeScript integration.

use crate::background::{GridConfig, GridType};
use crate::canvas::{Canvas, Shape, ShapeChanges, ShapeType};
use crate::events::{EventBuilder, EventStore, UndoManager};
use crate::layers::{C4Level, Layer, LayerManager};
use crate::plugin::{PluginContext, PluginHost, PluginRegistry, SimplePluginHost};
use crate::viewport::{Viewport, ViewportManager};
use archflow_core::{Color, EntityId, Vec2};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// JavaScript-friendly error type
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct JsError {
    message: String,
    #[source]
    source: Option<Box<dyn std::error::Error + 'static>>,
}

impl JsError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<wasm_bindgen::JsValue> for JsError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::new(format!("{:?}", value))
    }
}

/// Result type for WASM operations
pub type JsResult<T> = Result<T, JsError>;

/// Represents a point in 2D space for JS interop
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for JsColor {
    fn from(c: Color) -> Self {
        let (r, g, b, a) = c.to_rgba();
        Self { r, g, b, a }
    }
}

impl From<JsColor> for Color {
    fn from(c: JsColor) -> Self {
        Color::from_rgba(c.r, c.g, c.b, c.a)
    }
}

/// Represents a shape for JS interop
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

impl From<Shape> for JsShape {
    fn from(s: Shape) -> Self {
        Self {
            id: s.id.to_string(),
            shape_type: format!("{:?}", s.shape_type),
            x: s.x,
            y: s.y,
            width: s.width,
            height: s.height,
            rotation: s.rotation,
            fill_color: s.fill_color.into(),
            stroke_color: s.stroke_color.map(|c| c.into()),
            stroke_width: s.stroke_width,
            opacity: s.opacity,
            layer_id: s.layer_id.to_string(),
            selected: s.selected,
        }
    }
}

/// Represents selection for JS interop
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsSelection {
    pub shape_ids: Vec<String>,
    pub bounds: JsRect,
    pub is_box: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    viewport_manager: RefCell<ViewportManager>,
    event_store: RefCell<EventStore>,
    undo_manager: RefCell<UndoManager>,
    plugin_host: RefCell<SimplePluginHost>,
    context: RefCell<PluginContext>,
}

#[wasm_bindgen]
impl ArchFlowEditor {
    /// Creates a new editor instance
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            canvas: RefCell::new(Canvas::new(width, height)),
            viewport_manager: RefCell::new(ViewportManager::new(width, height)),
            event_store: RefCell::new(EventStore::default()),
            undo_manager: RefCell::new(UndoManager::default()),
            plugin_host: RefCell::new(SimplePluginHost::new()),
            context: RefCell::new(PluginContext::default()),
        }
    }

    // === Viewport Operations ===

    /// Gets the current viewport
    #[wasm_bindgen]
    pub fn get_viewport(&self) -> JsValue {
        let viewport = self.canvas.borrow().viewport();
        to_value(&viewport).unwrap_or(JsValue::NULL)
    }

    /// Sets the viewport
    #[wasm_bindgen]
    pub fn set_viewport(&self, viewport: JsValue) -> JsResult<()> {
        let v: Viewport = from_value(viewport).map_err(JsError::from)?;
        self.canvas.borrow_mut().set_viewport(v);
        Ok(())
    }

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
        let entity_id = EntityId::new_with(id);
        if let Some(shape) = self.canvas.borrow().get_shape(entity_id) {
            to_value(&JsShape::from(shape.clone())).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    /// Updates a shape
    #[wasm_bindgen]
    pub fn update_shape(&self, id: &str, changes: JsValue) -> bool {
        let entity_id = EntityId::new_with(id);
        let shape_changes: ShapeChanges = from_value(changes).unwrap_or_default();
        self.canvas
            .borrow_mut()
            .update_shape(entity_id, shape_changes)
    }

    /// Deletes a shape
    #[wasm_bindgen]
    pub fn delete_shape(&self, id: &str) -> bool {
        let entity_id = EntityId::new_with(id);
        self.canvas.borrow_mut().delete_shape(entity_id)
    }

    /// Gets all shapes
    #[wasm_bindgen]
    pub fn get_all_shapes(&self) -> JsValue {
        let shapes: Vec<JsShape> = self
            .canvas
            .borrow()
            .all_shapes()
            .iter()
            .map(|s| JsShape::from(s.clone()))
            .collect();
        to_value(&shapes).unwrap_or(JsValue::NULL)
    }

    // === Selection Operations ===

    /// Selects a shape
    #[wasm_bindgen]
    pub fn select(&self, id: &str) {
        let entity_id = EntityId::new_with(id);
        self.canvas.borrow_mut().select(entity_id);
    }

    /// Selects multiple shapes
    #[wasm_bindgen]
    pub fn select_multiple(&self, ids: Vec<String>) {
        let entity_ids: Vec<EntityId> = ids.into_iter().map(EntityId::new_with).collect();
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
        let selection = self.canvas.borrow().selection();
        to_value(&selection).unwrap_or(JsValue::NULL)
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
        if let Ok(c4_level) = level.parse::<C4Level>() {
            self.canvas.borrow_mut().set_c4_level(c4_level);
        }
    }

    /// Gets layer manager
    #[wasm_bindgen]
    pub fn get_layer_manager(&self) -> JsValue {
        let lm = self.canvas.borrow().layer_manager();
        to_value(&lm).unwrap_or(JsValue::NULL)
    }

    // === Grid Operations ===

    /// Gets grid configuration
    #[wasm_bindgen]
    pub fn get_grid_config(&self) -> JsValue {
        let config = self.canvas.borrow().background_renderer().grid_config();
        to_value(&config).unwrap_or(JsValue::NULL)
    }

    /// Sets grid configuration
    #[wasm_bindgen]
    pub fn set_grid_config(&self, config: JsValue) {
        let grid_config: GridConfig = from_value(config).unwrap_or_default();
        self.canvas.borrow_mut().set_grid_config(grid_config);
    }

    // === Event/Undo Operations ===

    /// Gets event store
    #[wasm_bindgen]
    pub fn get_event_store(&self) -> JsValue {
        to_value(&*self.event_store.borrow()).unwrap_or(JsValue::NULL)
    }

    /// Gets undo manager
    #[wasm_bindgen]
    pub fn get_undo_manager(&self) -> JsValue {
        to_value(&*self.undo_manager.borrow()).unwrap_or(JsValue::NULL)
    }

    /// Checks if undo is available
    #[wasm_bindgen]
    pub fn can_undo(&self) -> bool {
        self.undo_manager.borrow().can_undo()
    }

    /// Checks if redo is available
    #[wasm_bindgen]
    pub fn can_redo(&self) -> bool {
        self.undo_manager.borrow().can_redo()
    }

    // === Plugin Operations ===

    /// Gets plugin registry
    #[wasm_bindgen]
    pub fn get_plugin_registry(&self) -> JsValue {
        to_value(&*self.plugin_host.borrow().registry).unwrap_or(JsValue::NULL)
    }

    // === Coordinate Conversion ===

    /// Converts screen to canvas coordinates
    #[wasm_bindgen]
    pub fn screen_to_canvas(&self, x: f32, y: f32) -> JsValue {
        let point = self.canvas.borrow().screen_to_canvas(Vec2::new(x, y));
        to_value(&JsVec2::from(point)).unwrap_or(JsValue::NULL)
    }

    /// Converts canvas to screen coordinates
    #[wasm_bindgen]
    pub fn canvas_to_screen(&self, x: f32, y: f32) -> JsValue {
        let point = self.canvas.borrow().canvas_to_screen(Vec2::new(x, y));
        to_value(&JsVec2::from(point)).unwrap_or(JsValue::NULL)
    }

    // === Canvas Dimensions ===

    /// Gets canvas dimensions
    #[wasm_bindgen]
    pub fn get_dimensions(&self) -> JsValue {
        serde_json::json!({
            "width": 800.0,
            "height": 600.0
        })
        .into()
    }
}

// Type aliases for easier JS interop
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ArchFlowEditor")]
    pub type TsArchFlowEditor;

    #[wasm_bindgen(typescript_type = "JsShape")]
    pub type TsJsShape;

    #[wasm_bindgen(typescript_type = "GridConfig")]
    pub type TsGridConfig;
}

/// TypeScript type definitions for the SDK
#[wasm_bindgen]
pub fn get_typescript_definitions() -> String {
    r#"
/**
 * ArchFlow SDK TypeScript Definitions
 */

export class ArchFlowEditor {
    constructor(width: number, height: number);

    // Viewport Operations
    getViewport(): Viewport;
    setViewport(viewport: Viewport): void;
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
    getLayerManager(): LayerManager;

    // Grid Operations
    getGridConfig(): GridConfig;
    setGridConfig(config: GridConfig): void;

    // Undo/Redo Operations
    getEventStore(): EventStore;
    getUndoManager(): UndoManager;
    canUndo(): boolean;
    canRedo(): boolean;

    // Plugin Operations
    getPluginRegistry(): PluginRegistry;

    // Coordinate Conversion
    screenToCanvas(x: number, y: number): Vec2;
    canvasToScreen(x: number, y: number): Vec2;

    // Canvas Dimensions
    getDimensions(): { width: number; height: number };
}

export interface Viewport {
    offset: Vec2;
    zoom: number;
    minZoom: number;
    maxZoom: number;
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

export interface LayerManager {
    layers: Layer[];
    currentLevel: string;
    createLayer(level: string, name: string): string;
    deleteLayer(id: string): boolean;
    setVisibility(id: string, visible: boolean): boolean;
    setOpacity(id: string, opacity: number): boolean;
}

export interface Layer {
    id: string;
    c4Level: string;
    name: string;
    visible: boolean;
    locked: boolean;
    opacity: number;
}

export interface EventStore {
    events: RecordedEvent[];
    append(event: RecordedEvent): void;
    replay(handler: EventHandler): void;
}

export interface RecordedEvent {
    metadata: EventMetadata;
    event: CanvasEvent;
}

export interface EventMetadata {
    id: string;
    timestamp: number;
    author: string;
    message: string;
}

export interface UndoManager {
    canUndo(): boolean;
    canRedo(): boolean;
    undoCount(): number;
    redoCount(): number;
    undo(): void;
    redo(): void;
    clear(): void;
}

export interface PluginRegistry {
    plugins: Map<string, Plugin>;
    register(plugin: Plugin): string;
    initializeAll(host: PluginHost): void;
    shutdownAll(host: PluginHost): void;
}

export interface Plugin {
    metadata: PluginMetadata;
    capabilities: string[];
    initialize(host: PluginHost): void;
    onEnable(host: PluginHost): void;
    onDisable(host: PluginHost): void;
    shutdown(host: PluginHost): void;
    onEvent(event: CanvasEvent, context: PluginContext): void;
    update(context: PluginContext, deltaTime: number): void;
}

export interface PluginMetadata {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
}

export interface PluginContext {
    viewport: Viewport;
    c4Level: string;
    mousePosition: Vec2 | null;
    selectedShapes: string[];
    canvasWidth: number;
    canvasHeight: number;
}

export type CanvasEvent =
    | { type: 'shapeCreated'; shapeId: string; shapeData: ShapeData }
    | { type: 'shapeUpdated'; shapeId: string; previous: ShapeData; current: ShapeData }
    | { type: 'shapeDeleted'; shapeId: string; shapeData: ShapeData }
    | { type: 'batch'; events: CanvasEvent[] };

export interface ShapeData {
    id: string;
    shapeType: string;
    x: number;
    y: number;
    width: number;
    height: number;
    fillColor: JsColor;
    strokeColor: JsColor | null;
    strokeWidth: number;
    opacity: number;
    rotation: number;
    layerId: string;
}
"#
    .to_string()
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap();
}
