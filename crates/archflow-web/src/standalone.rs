//! Standalone Implementation
//!
//! This module provides a minimal but functional standalone implementation
//! for the web editor. It uses the existing shapes and state modules.

use crate::JsColor;
use crate::shapes::{Shape, ShapeType};
use crate::state::InteractionState;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

// ============ Simple Types ============

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsVec2Simple {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsSelectionSimple {
    pub shape_ids: Vec<String>,
    pub bounds: JsRect,
    pub is_box: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsShapeSimple {
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
    pub selected: bool,
}

// ============ Main Editor ============

#[wasm_bindgen]
pub struct ArchFlowEditor {
    context: CanvasRenderingContext2d,
    width: u32,
    height: u32,
    shapes: RefCell<HashMap<String, Shape>>,
    selected_shapes: RefCell<Vec<String>>,
    interaction: RefCell<InteractionState>,
    zoom: RefCell<f32>,
    pan_x: RefCell<f32>,
    pan_y: RefCell<f32>,
}

#[wasm_bindgen]
impl ArchFlowEditor {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<ArchFlowEditor, JsValue> {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Info).map_err(|_| "Failed to init logging")?;

        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("Failed to cast to CanvasRenderingContext2d"))?;

        let width = canvas.width();
        let height = canvas.height();

        log::info!("ArchFlowEditor initialized: {}x{}", width, height);

        Ok(ArchFlowEditor {
            context,
            width,
            height,
            shapes: RefCell::new(HashMap::new()),
            selected_shapes: RefCell::new(Vec::new()),
            interaction: RefCell::new(InteractionState::default()),
            zoom: RefCell::new(1.0),
            pan_x: RefCell::new(0.0),
            pan_y: RefCell::new(0.0),
        })
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    // ============ Shape Operations ============

    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> String {
        let id = Uuid::new_v4().to_string();
        let shape = Shape {
            id: id.clone(),
            shape_type: ShapeType::Rectangle,
            x: x as f32,
            y: y as f32,
            width: width as f32,
            height: height as f32,
            rotation: 0.0,
            fill_color: JsColor {
                r: 0.2,
                g: 0.4,
                b: 0.8,
                a: 1.0,
            },
            stroke_color: Some(JsColor {
                r: 0.1,
                g: 0.2,
                b: 0.4,
                a: 1.0,
            }),
            stroke_width: 2.0,
            opacity: 1.0,
            metadata: HashMap::new(),
        };
        self.shapes.borrow_mut().insert(id.clone(), shape);
        id
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> String {
        let id = Uuid::new_v4().to_string();
        let shape = Shape {
            id: id.clone(),
            shape_type: ShapeType::Ellipse,
            x: x as f32,
            y: y as f32,
            width: (radius_x * 2.0) as f32,
            height: (radius_y * 2.0) as f32,
            rotation: 0.0,
            fill_color: JsColor {
                r: 0.3,
                g: 0.6,
                b: 0.3,
                a: 1.0,
            },
            stroke_color: Some(JsColor {
                r: 0.15,
                g: 0.3,
                b: 0.15,
                a: 1.0,
            }),
            stroke_width: 2.0,
            opacity: 1.0,
            metadata: HashMap::new(),
        };
        self.shapes.borrow_mut().insert(id.clone(), shape);
        id
    }

    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> String {
        let id = Uuid::new_v4().to_string();
        let shape = Shape {
            id: id.clone(),
            shape_type: ShapeType::Line,
            x: x1 as f32,
            y: y1 as f32,
            width: (x2 - x1) as f32,
            height: (y2 - y1) as f32,
            rotation: 0.0,
            fill_color: JsColor {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            },
            stroke_color: Some(JsColor {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            }),
            stroke_width: 2.0,
            opacity: 1.0,
            metadata: HashMap::new(),
        };
        self.shapes.borrow_mut().insert(id.clone(), shape);
        id
    }

    #[wasm_bindgen]
    pub fn add_text(&mut self, _x: f64, _y: f64, _text: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let shape = Shape {
            id: id.clone(),
            shape_type: ShapeType::Rectangle,
            x: _x as f32,
            y: _y as f32,
            width: 100.0,
            height: 30.0,
            rotation: 0.0,
            fill_color: JsColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            stroke_color: None,
            stroke_width: 0.0,
            opacity: 1.0,
            metadata: HashMap::new(),
        };
        self.shapes.borrow_mut().insert(id.clone(), shape);
        id
    }

    #[wasm_bindgen]
    pub fn create_shape(
        &mut self,
        shape_type: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        _color: &str,
    ) -> String {
        match shape_type {
            "rect" | "rectangle" => self.add_rect(x, y, width, height),
            "ellipse" | "circle" => self.add_ellipse(x, y, width / 2.0, height / 2.0),
            "line" => self.add_line(x, y, x + width, y + height),
            "text" => self.add_text(x, y, "Text"),
            _ => self.add_rect(x, y, width, height),
        }
    }

    #[wasm_bindgen]
    pub fn delete_shape(&mut self, id: &str) -> bool {
        if self.shapes.borrow_mut().remove(id).is_some() {
            self.selected_shapes.borrow_mut().retain(|s| s != id);
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn get_shape(&self, id: &str) -> JsValue {
        if let Some(shape) = self.shapes.borrow().get(id) {
            let js_shape = JsShapeSimple {
                id: shape.id.clone(),
                shape_type: format!("{:?}", shape.shape_type),
                x: shape.x,
                y: shape.y,
                width: shape.width,
                height: shape.height,
                rotation: shape.rotation,
                fill_color: shape.fill_color.clone(),
                stroke_color: shape.stroke_color.clone(),
                stroke_width: shape.stroke_width,
                opacity: shape.opacity,
                selected: self.selected_shapes.borrow().contains(&shape.id),
            };
            serde_wasm_bindgen::to_value(&js_shape).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    #[wasm_bindgen]
    pub fn get_all_shapes(&self) -> JsValue {
        let js_shapes: Vec<JsShapeSimple> = self
            .shapes
            .borrow()
            .values()
            .map(|s| JsShapeSimple {
                id: s.id.clone(),
                shape_type: format!("{:?}", s.shape_type),
                x: s.x,
                y: s.y,
                width: s.width,
                height: s.height,
                rotation: s.rotation,
                fill_color: s.fill_color.clone(),
                stroke_color: s.stroke_color.clone(),
                stroke_width: s.stroke_width,
                opacity: s.opacity,
                selected: self.selected_shapes.borrow().contains(&s.id),
            })
            .collect();
        serde_wasm_bindgen::to_value(&js_shapes).unwrap_or(JsValue::NULL)
    }

    // ============ Selection Operations ============

    #[wasm_bindgen]
    pub fn select(&mut self, id: &str) {
        if self.shapes.borrow().contains_key(id) {
            self.selected_shapes.borrow_mut().clear();
            self.selected_shapes.borrow_mut().push(id.to_string());
        }
    }

    #[wasm_bindgen]
    pub fn select_multiple(&mut self, ids: Vec<String>) {
        let available: Vec<String> = ids
            .into_iter()
            .filter(|id| self.shapes.borrow().contains_key(id))
            .collect();
        *self.selected_shapes.borrow_mut() = available;
    }

    #[wasm_bindgen]
    pub fn clear_selection(&mut self) {
        self.selected_shapes.borrow_mut().clear();
    }

    #[wasm_bindgen]
    pub fn get_selection(&self) -> JsValue {
        let selected = self.selected_shapes.borrow();
        if selected.is_empty() {
            return serde_wasm_bindgen::to_value(&JsSelectionSimple {
                shape_ids: vec![],
                bounds: JsRect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
                is_box: false,
            })
            .unwrap_or(JsValue::NULL);
        }

        let shapes = self.shapes.borrow();
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for shape_id in selected.iter() {
            if let Some(shape) = shapes.get(shape_id) {
                min_x = min_x.min(shape.x);
                min_y = min_y.min(shape.y);
                max_x = max_x.max(shape.x + shape.width);
                max_y = max_y.max(shape.y + shape.height);
            }
        }

        serde_wasm_bindgen::to_value(&JsSelectionSimple {
            shape_ids: selected.clone(),
            bounds: JsRect {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            },
            is_box: false,
        })
        .unwrap_or(JsValue::NULL)
    }

    // ============ Viewport Operations ============

    #[wasm_bindgen]
    pub fn pan(&mut self, dx: f64, dy: f64) {
        *self.pan_x.borrow_mut() += dx as f32;
        *self.pan_y.borrow_mut() += dy as f32;
    }

    #[wasm_bindgen]
    pub fn zoom_at(&mut self, _x: f64, _y: f64, factor: f64) {
        let new_zoom = (*self.zoom.borrow() * factor as f32).clamp(0.1, 5.0);
        *self.zoom.borrow_mut() = new_zoom;
    }

    #[wasm_bindgen]
    pub fn zoom_in(&mut self) {
        *self.zoom.borrow_mut() = (*self.zoom.borrow() * 1.2).clamp(0.1, 5.0);
    }

    #[wasm_bindgen]
    pub fn zoom_out(&mut self) {
        *self.zoom.borrow_mut() = (*self.zoom.borrow() * 0.8).clamp(0.1, 5.0);
    }

    #[wasm_bindgen]
    pub fn zoom_to_fit(&mut self) {
        *self.zoom.borrow_mut() = 1.0;
        *self.pan_x.borrow_mut() = 0.0;
        *self.pan_y.borrow_mut() = 0.0;
    }

    #[wasm_bindgen]
    pub fn get_zoom(&self) -> f32 {
        *self.zoom.borrow()
    }

    #[wasm_bindgen]
    pub fn set_zoom(&mut self, factor: f64) {
        *self.zoom.borrow_mut() = factor as f32;
    }

    // ============ Coordinate Conversion ============

    #[wasm_bindgen]
    pub fn screen_to_canvas(&self, x: f64, y: f64) -> JsValue {
        let canvas_x = (x - *self.pan_x.borrow() as f64) / *self.zoom.borrow() as f64;
        let canvas_y = (y - *self.pan_y.borrow() as f64) / *self.zoom.borrow() as f64;
        serde_wasm_bindgen::to_value(&JsVec2Simple {
            x: canvas_x as f32,
            y: canvas_y as f32,
        })
        .unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen]
    pub fn canvas_to_screen(&self, x: f64, y: f64) -> JsValue {
        let screen_x = x * *self.zoom.borrow() as f64 + *self.pan_x.borrow() as f64;
        let screen_y = y * *self.zoom.borrow() as f64 + *self.pan_y.borrow() as f64;
        serde_wasm_bindgen::to_value(&JsVec2Simple {
            x: screen_x as f32,
            y: screen_y as f32,
        })
        .unwrap_or(JsValue::NULL)
    }

    // ============ C4 Layer Operations ============

    #[wasm_bindgen]
    pub fn get_c4_level(&self) -> String {
        "Context".to_string()
    }

    #[wasm_bindgen]
    pub fn set_c4_level(&self, _level: &str) {
        // Not implemented in standalone mode
    }

    // ============ Grid Operations ============

    #[wasm_bindgen]
    pub fn get_grid_config(&self) -> JsValue {
        JsValue::NULL
    }

    #[wasm_bindgen]
    pub fn set_grid_config(&self, _config: JsValue) {
        // Not implemented in standalone mode
    }

    // ============ Input Handling ============

    #[wasm_bindgen]
    pub fn on_mousedown(&mut self, x: f64, y: f64, button: u16) {
        let canvas_x = (x - *self.pan_x.borrow() as f64) / *self.zoom.borrow() as f64;
        let canvas_y = (y - *self.pan_y.borrow() as f64) / *self.zoom.borrow() as f64;

        let mut interaction = self.interaction.borrow_mut();
        interaction.is_dragging = true;
        interaction.drag_start_x = canvas_x;
        interaction.drag_start_y = canvas_y;
        interaction.current_x = canvas_x;
        interaction.current_y = canvas_y;

        if button == 0 {
            let shapes = self.shapes.borrow();
            let mut clicked: Vec<String> = Vec::new();
            for (id, shape) in shapes.iter() {
                if canvas_x >= shape.x as f64
                    && canvas_x <= (shape.x + shape.width) as f64
                    && canvas_y >= shape.y as f64
                    && canvas_y <= (shape.y + shape.height) as f64
                {
                    clicked.push(id.clone());
                }
            }
            if !clicked.is_empty() {
                *self.selected_shapes.borrow_mut() = clicked;
            }
        }
    }

    #[wasm_bindgen]
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        let canvas_x = (x - *self.pan_x.borrow() as f64) / *self.zoom.borrow() as f64;
        let canvas_y = (y - *self.pan_y.borrow() as f64) / *self.zoom.borrow() as f64;

        let mut interaction = self.interaction.borrow_mut();
        interaction.current_x = canvas_x;
        interaction.current_y = canvas_y;
    }

    #[wasm_bindgen]
    pub fn on_mouseup(&mut self, _x: f64, _y: f64) {
        let mut interaction = self.interaction.borrow_mut();
        interaction.is_dragging = false;
    }

    #[wasm_bindgen]
    pub fn on_wheel(&mut self, x: f64, y: f64, delta_y: f64) {
        let factor = if delta_y > 0.0 { 0.9 } else { 1.1 };
        self.zoom_at(x, y, factor);
    }

    #[wasm_bindgen]
    pub fn on_keydown(&mut self, key: &str, _shift: bool, ctrl: bool) {
        match key {
            "Delete" | "Backspace" => {
                self.delete_selected();
            }
            "a" if ctrl => {
                let all_ids: Vec<String> = self.shapes.borrow().keys().cloned().collect();
                *self.selected_shapes.borrow_mut() = all_ids;
            }
            _ => {}
        }
    }

    #[wasm_bindgen]
    pub fn on_keyup(&mut self, _key: &str) {}

    // ============ Query Methods ============

    #[wasm_bindgen]
    pub fn shape_count(&self) -> usize {
        self.shapes.borrow().len()
    }

    #[wasm_bindgen]
    pub fn selection_count(&self) -> usize {
        self.selected_shapes.borrow().len()
    }

    // ============ Commands ============

    #[wasm_bindgen]
    pub fn undo(&mut self) -> bool {
        false
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) -> bool {
        false
    }

    #[wasm_bindgen]
    pub fn can_undo(&self) -> bool {
        false
    }

    #[wasm_bindgen]
    pub fn can_redo(&self) -> bool {
        false
    }

    // ============ Simulation ============

    #[wasm_bindgen]
    pub fn start_simulation(&mut self) {
        log::info!("Starting simulation...");
    }

    #[wasm_bindgen]
    pub fn stop_simulation(&mut self) {
        log::info!("Stopping simulation...");
    }

    // ============ Deployment ============

    #[wasm_bindgen]
    pub fn deploy_architecture(&mut self) {
        log::info!("Deploying architecture...");
    }

    // ============ Rendering ============

    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.context.set_fill_style_str("#ffffff");
        let _ = self
            .context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        self.context.save();

        let zoom = *self.zoom.borrow();
        let pan_x = *self.pan_x.borrow();
        let pan_y = *self.pan_y.borrow();

        self.context.translate(pan_x as f64, pan_y as f64).ok();
        self.context.scale(zoom as f64, zoom as f64).ok();

        let shapes = self.shapes.borrow();
        let selected = self.selected_shapes.borrow();
        for (_, shape) in shapes.iter() {
            self.render_shape(shape, selected.contains(&shape.id));
        }

        self.context.restore();
    }

    fn render_shape(&self, shape: &Shape, is_selected: bool) {
        let fill_color = format!(
            "rgba({},{},{},{})",
            (shape.fill_color.r * 255.0) as u8,
            (shape.fill_color.g * 255.0) as u8,
            (shape.fill_color.b * 255.0) as u8,
            shape.fill_color.a * shape.opacity
        );

        self.context.set_fill_style_str(&fill_color);

        if let Some(stroke) = &shape.stroke_color {
            let stroke_color = format!(
                "rgba({},{},{},{})",
                (stroke.r * 255.0) as u8,
                (stroke.g * 255.0) as u8,
                (stroke.b * 255.0) as u8,
                stroke.a * shape.opacity
            );
            self.context.set_stroke_style_str(&stroke_color);
            self.context.set_line_width(shape.stroke_width as f64);
        }

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        match shape.shape_type {
            ShapeType::Rectangle => {
                let _ = path.rect(
                    shape.x as f64,
                    shape.y as f64,
                    shape.width as f64,
                    shape.height as f64,
                );
            }
            ShapeType::Ellipse => {
                let rx = shape.width as f64 / 2.0;
                let ry = shape.height as f64 / 2.0;
                let cx = shape.x as f64 + rx;
                let cy = shape.y as f64 + ry;
                let _ = path.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::PI * 2.0);
            }
            ShapeType::Line => {
                let _ = path.move_to(shape.x as f64, shape.y as f64);
                let _ = path.line_to(
                    (shape.x + shape.width) as f64,
                    (shape.y + shape.height) as f64,
                );
            }
            ShapeType::AwsResource => {
                let _ = path.rect(
                    shape.x as f64,
                    shape.y as f64,
                    shape.width as f64,
                    shape.height as f64,
                );
            }
        }

        let _ = self.context.fill_with_path_2d(&path);
        if shape.stroke_color.is_some() {
            let _ = self.context.stroke_with_path(&path);
        }

        if is_selected {
            self.render_selection_outline(
                shape.x as f64,
                shape.y as f64,
                shape.width as f64,
                shape.height as f64,
            );
        }
    }

    fn render_selection_outline(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.set_stroke_style_str("#0066cc");
        self.context.set_line_width(2.0);
        let _ = self.context.set_line_dash(&JsValue::from_str("5,5"));

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.rect(x, y, width, height);
        let _ = self.context.stroke_with_path(&path);
        let _ = self.context.set_line_dash(&JsValue::from_str(""));
    }

    // ============ Manager Accessors ============

    #[wasm_bindgen]
    pub fn get_library_manager(&self) -> JsValue {
        JsValue::NULL
    }

    #[wasm_bindgen]
    pub fn get_properties_manager(&self) -> JsValue {
        JsValue::NULL
    }

    #[wasm_bindgen]
    pub fn get_alignment_manager(&self) -> JsValue {
        JsValue::NULL
    }

    #[wasm_bindgen]
    pub fn get_group_manager(&self) -> JsValue {
        JsValue::NULL
    }

    // ============ Helper Methods ============

    fn delete_selected(&mut self) -> usize {
        let mut count = 0;
        let mut shapes = self.shapes.borrow_mut();
        let selected = self.selected_shapes.borrow().clone();
        for id in &selected {
            if shapes.remove(id).is_some() {
                count += 1;
            }
        }
        self.selected_shapes.borrow_mut().clear();
        count
    }
}

// ============ Backward Compatibility ============

#[wasm_bindgen]
pub struct ArchFlowDemo(ArchFlowEditor);

#[wasm_bindgen]
impl ArchFlowDemo {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<ArchFlowDemo, JsValue> {
        Ok(ArchFlowDemo(ArchFlowEditor::new(canvas)?))
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.0.resize(width, height);
    }

    // Forward all other methods to inner editor
    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> String {
        self.0.add_rect(x, y, width, height)
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> String {
        self.0.add_ellipse(x, y, radius_x, radius_y)
    }

    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> String {
        self.0.add_line(x1, y1, x2, y2)
    }

    #[wasm_bindgen]
    pub fn add_text(&mut self, x: f64, y: f64, text: &str) -> String {
        self.0.add_text(x, y, text)
    }

    #[wasm_bindgen]
    pub fn create_shape(
        &mut self,
        shape_type: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: &str,
    ) -> String {
        self.0.create_shape(shape_type, x, y, width, height, color)
    }

    #[wasm_bindgen]
    pub fn delete_shape(&mut self, id: &str) -> bool {
        self.0.delete_shape(id)
    }

    #[wasm_bindgen]
    pub fn get_shape(&self, id: &str) -> JsValue {
        self.0.get_shape(id)
    }

    #[wasm_bindgen]
    pub fn get_all_shapes(&self) -> JsValue {
        self.0.get_all_shapes()
    }

    #[wasm_bindgen]
    pub fn select(&mut self, id: &str) {
        self.0.select(id)
    }

    #[wasm_bindgen]
    pub fn select_multiple(&mut self, ids: Vec<String>) {
        self.0.select_multiple(ids)
    }

    #[wasm_bindgen]
    pub fn clear_selection(&mut self) {
        self.0.clear_selection()
    }

    #[wasm_bindgen]
    pub fn get_selection(&self) -> JsValue {
        self.0.get_selection()
    }

    #[wasm_bindgen]
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.0.pan(dx, dy)
    }

    #[wasm_bindgen]
    pub fn zoom_at(&mut self, x: f64, y: f64, factor: f64) {
        self.0.zoom_at(x, y, factor)
    }

    #[wasm_bindgen]
    pub fn zoom_in(&mut self) {
        self.0.zoom_in()
    }

    #[wasm_bindgen]
    pub fn zoom_out(&mut self) {
        self.0.zoom_out()
    }

    #[wasm_bindgen]
    pub fn zoom_to_fit(&mut self) {
        self.0.zoom_to_fit()
    }

    #[wasm_bindgen]
    pub fn get_zoom(&self) -> f32 {
        self.0.get_zoom()
    }

    #[wasm_bindgen]
    pub fn set_zoom(&mut self, factor: f64) {
        self.0.set_zoom(factor)
    }

    #[wasm_bindgen]
    pub fn screen_to_canvas(&self, x: f64, y: f64) -> JsValue {
        self.0.screen_to_canvas(x, y)
    }

    #[wasm_bindgen]
    pub fn canvas_to_screen(&self, x: f64, y: f64) -> JsValue {
        self.0.canvas_to_screen(x, y)
    }

    #[wasm_bindgen]
    pub fn get_c4_level(&self) -> String {
        self.0.get_c4_level()
    }

    #[wasm_bindgen]
    pub fn set_c4_level(&self, level: &str) {
        self.0.set_c4_level(level)
    }

    #[wasm_bindgen]
    pub fn get_grid_config(&self) -> JsValue {
        self.0.get_grid_config()
    }

    #[wasm_bindgen]
    pub fn set_grid_config(&self, config: JsValue) {
        self.0.set_grid_config(config)
    }

    #[wasm_bindgen]
    pub fn on_mousedown(&mut self, x: f64, y: f64, button: u16) {
        self.0.on_mousedown(x, y, button)
    }

    #[wasm_bindgen]
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        self.0.on_mousemove(x, y)
    }

    #[wasm_bindgen]
    pub fn on_mouseup(&mut self, x: f64, y: f64) {
        self.0.on_mouseup(x, y)
    }

    #[wasm_bindgen]
    pub fn on_wheel(&mut self, x: f64, y: f64, delta_y: f64) {
        self.0.on_wheel(x, y, delta_y)
    }

    #[wasm_bindgen]
    pub fn on_keydown(&mut self, key: &str, shift: bool, ctrl: bool) {
        self.0.on_keydown(key, shift, ctrl)
    }

    #[wasm_bindgen]
    pub fn on_keyup(&mut self, key: &str) {
        self.0.on_keyup(key)
    }

    #[wasm_bindgen]
    pub fn shape_count(&self) -> usize {
        self.0.shape_count()
    }

    #[wasm_bindgen]
    pub fn selection_count(&self) -> usize {
        self.0.selection_count()
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) -> bool {
        self.0.undo()
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) -> bool {
        self.0.redo()
    }

    #[wasm_bindgen]
    pub fn can_undo(&self) -> bool {
        self.0.can_undo()
    }

    #[wasm_bindgen]
    pub fn can_redo(&self) -> bool {
        self.0.can_redo()
    }

    #[wasm_bindgen]
    pub fn start_simulation(&mut self) {
        self.0.start_simulation()
    }

    #[wasm_bindgen]
    pub fn stop_simulation(&mut self) {
        self.0.stop_simulation()
    }

    #[wasm_bindgen]
    pub fn deploy_architecture(&mut self) {
        self.0.deploy_architecture()
    }

    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.0.render()
    }
}
