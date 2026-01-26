//! ArchFlow WASM v2.0 - Bindings para JavaScript
//!
//! Este crate expone el engine completo a JavaScript/WebAssembly

use archflow_core::{
    AnimatedProperty, AnimationConfig, AnimationManager, Color, ColorPalette, EasingFunction,
    EntityId, FloatAnimation, FloatKeyframe, PositionAnimation, PositionKeyframe, Scene,
    SnapHelper, Uuid, Vec2, ZoomLevel, ZoomManager,
};
use std::str::FromStr;
use std::time::Duration;
use wasm_bindgen::prelude::*;

// Console logging macros
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Shared state for the WASM module
#[wasm_bindgen]
pub struct ArchFlowEngine {
    scene: Scene,
    zoom_manager: ZoomManager,
    canvas_width: f32,
    canvas_height: f32,
    color_palette: ColorPalette,
    shapes: Vec<WasmShape>,
    animation_manager: AnimationManager,
    snap_helper: SnapHelper,
}

// Internal shape structure for WASM engine (not exposed to JS directly)
pub struct WasmShape {
    pub id: String,
    pub shape_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: String,
    pub rotation: f32,
    pub opacity: f32,
}

#[wasm_bindgen]
impl ArchFlowEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ArchFlowEngine, JsValue> {
        console_error_panic_hook::set_once();
        console_log!("ArchFlow Engine v2.0 initialized");
        Ok(Self {
            scene: Scene::new(),
            zoom_manager: ZoomManager::new(800.0, 600.0),
            canvas_width: 800.0,
            canvas_height: 600.0,
            color_palette: ColorPalette::default(),
            shapes: Vec::new(),
            animation_manager: AnimationManager::new(),
            snap_helper: SnapHelper::new().enable().with_grid_size(10.0),
        })
    }

    // ===== Canvas Configuration =====

    #[wasm_bindgen]
    pub fn configure_canvas(&mut self, width: f32, height: f32, _background_color: &str) {
        self.canvas_width = width;
        self.canvas_height = height;
        self.zoom_manager.set_size(width, height);
        console_log!("Canvas configured: {}x{}", width, height);
    }

    #[wasm_bindgen]
    pub fn get_canvas_width(&self) -> f32 {
        self.canvas_width
    }

    #[wasm_bindgen]
    pub fn get_canvas_height(&self) -> f32 {
        self.canvas_height
    }

    // ===== Shape Creation =====

    #[wasm_bindgen]
    pub fn add_rectangle(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &str,
    ) -> String {
        let shape_id = self.scene.add_rectangle(x, y, width, height);

        let shape = WasmShape {
            id: shape_id.to_string(),
            shape_type: "rectangle".to_string(),
            x,
            y,
            width,
            height,
            color: color.to_string(),
            rotation: 0.0,
            opacity: 1.0,
        };
        self.shapes.push(shape);

        console_log!("Added rectangle at ({}, {})", x, y);
        shape_id.to_string()
    }

    #[wasm_bindgen]
    pub fn add_ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        radius_x: f32,
        radius_y: f32,
        color: &str,
    ) -> String {
        let shape_id = self.scene.add_ellipse(cx, cy, radius_x, radius_y);

        let shape = WasmShape {
            id: shape_id.to_string(),
            shape_type: "ellipse".to_string(),
            x: cx,
            y: cy,
            width: radius_x * 2.0,
            height: radius_y * 2.0,
            color: color.to_string(),
            rotation: 0.0,
            opacity: 1.0,
        };
        self.shapes.push(shape);

        console_log!("Added ellipse at ({}, {})", cx, cy);
        shape_id.to_string()
    }

    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &str) -> String {
        let shape_id = self.scene.add_line(x1, y1, x2, y2);

        let shape = WasmShape {
            id: shape_id.to_string(),
            shape_type: "line".to_string(),
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
            color: color.to_string(),
            rotation: 0.0,
            opacity: 1.0,
        };
        self.shapes.push(shape);

        console_log!("Added line from ({}, {}) to ({}, {})", x1, y1, x2, y2);
        shape_id.to_string()
    }

    #[wasm_bindgen]
    pub fn update_shape(&mut self, id: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
        if let Some(shape) = self.shapes.iter_mut().find(|s| s.id == id) {
            shape.x = x;
            shape.y = y;
            shape.width = width;
            shape.height = height;
            return true;
        }
        false
    }

    // ===== Shape Access =====

    #[wasm_bindgen]
    pub fn get_shape_count(&self) -> usize {
        self.shapes.len()
    }

    #[wasm_bindgen]
    pub fn get_all_shapes_json(&self) -> String {
        let mut result = String::from("[");
        for (i, shape) in self.shapes.iter().enumerate() {
            if i > 0 {
                result.push(',');
            }
            result.push_str(&format!(
                r#"{{"id":"{}","shape_type":"{}","x":{},"y":{},"width":{},"height":{},"color":"{}"}}"#,
                shape.id, shape.shape_type, shape.x, shape.y, shape.width, shape.height, shape.color
            ));
        }
        result.push(']');
        result
    }

    #[wasm_bindgen]
    pub fn clear_shapes(&mut self) {
        self.scene = Scene::new();
        self.shapes.clear();
        self.animation_manager = AnimationManager::new();
        console_log!("Cleared all shapes");
    }

    #[wasm_bindgen]
    pub fn get_shape_json(&self, index: usize) -> String {
        if let Some(shape) = self.shapes.get(index) {
            format!(
                r#"{{"id":"{}","shape_type":"{}","x":{},"y":{},"width":{},"height":{},"color":"{}"}}"#,
                shape.id,
                shape.shape_type,
                shape.x,
                shape.y,
                shape.width,
                shape.height,
                shape.color
            )
        } else {
            "null".to_string()
        }
    }

    // ===== Zoom System =====

    #[wasm_bindgen]
    pub fn get_zoom_level(&self) -> String {
        format!("{:?}", self.zoom_manager.zoom_level())
    }

    #[wasm_bindgen]
    pub fn get_zoom_scale(&self) -> f32 {
        self.zoom_manager.scale()
    }

    #[wasm_bindgen]
    pub fn zoom_in(&mut self) {
        self.zoom_manager.zoom_in();
        console_log!("Zoomed in to {:?}", self.zoom_manager.zoom_level());
    }

    #[wasm_bindgen]
    pub fn zoom_out(&mut self) {
        self.zoom_manager.zoom_out();
        console_log!("Zoomed out to {:?}", self.zoom_manager.zoom_level());
    }

    #[wasm_bindgen]
    pub fn zoom_to(&mut self, level: &str) {
        let zoom_level = match level {
            "system" | "context" => ZoomLevel::System,
            "container" => ZoomLevel::Container,
            "component" => ZoomLevel::Component,
            "code" => ZoomLevel::Code,
            _ => ZoomLevel::Container,
        };

        self.zoom_manager.zoom_to_level(zoom_level);
        console_log!("Zoomed to {}", level);
    }

    // ===== Animation System =====

    #[wasm_bindgen]
    pub fn update_animations(&mut self, delta_ms: f64) {
        let delta = Duration::from_secs_f64(delta_ms / 1000.0);

        // Update animations
        self.animation_manager.update(delta);

        // Check for completed animations or update shape values
        // Note: In a real ECS system, this would happen automatically.
        // Here we need to map animation values back to shapes for the demo.
        let mut updates = Vec::new();

        for (entity_id, value) in self.animation_manager.get_active_positions() {
            updates.push((entity_id, value));
        }

        // Apply position updates to WasmShapes
        for (id_str, pos) in updates {
            for shape in &mut self.shapes {
                if shape.id == id_str.as_string() {
                    shape.x = pos.0;
                    shape.y = pos.1;
                }
            }
        }

        let mut float_updates = Vec::new();
        for (entity_id, property, value) in self.animation_manager.get_active_floats() {
            if property == AnimatedProperty::Opacity {
                float_updates.push((entity_id, value));
            }
        }

        // Apply float updates
        for (id_str, val) in float_updates {
            for shape in &mut self.shapes {
                if shape.id == id_str.as_string() {
                    shape.opacity = val;
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn animate_position(
        &mut self,
        id_str: &str,
        target_x: f32,
        target_y: f32,
        duration_ms: f64,
        easing_str: &str,
    ) {
        if let Ok(uuid) = Uuid::from_str(id_str) {
            let entity_id = EntityId::from(uuid);

            // Find current pos
            let mut start_pos = (0.0, 0.0);
            if let Some(shape) = self.shapes.iter().find(|s| s.id == id_str) {
                start_pos = (shape.x, shape.y);
            }

            let easing = match easing_str {
                "ease-in" => EasingFunction::EaseIn,
                "ease-out" => EasingFunction::EaseOut,
                "ease-in-out" => EasingFunction::EaseInOut,
                "linear" => EasingFunction::Linear,
                "elastic" => EasingFunction::Elastic,
                "bounce" => EasingFunction::Bounce,
                _ => EasingFunction::EaseInOut,
            };

            let anim = PositionAnimation::new(
                entity_id,
                vec![
                    PositionKeyframe::new(0.0, start_pos, easing),
                    PositionKeyframe::new(1.0, (target_x, target_y), easing),
                ],
            )
            .with_config(AnimationConfig {
                duration: Duration::from_millis(duration_ms as u64),
                ..Default::default()
            });

            self.animation_manager.add_position_animation(anim);
        }
    }

    #[wasm_bindgen]
    pub fn animate_opacity(&mut self, id_str: &str, target_opacity: f32, duration_ms: f64) {
        if let Ok(uuid) = Uuid::from_str(id_str) {
            let entity_id = EntityId::from(uuid);

            // Find current opacity
            let mut start_opacity = 1.0;
            if let Some(shape) = self.shapes.iter().find(|s| s.id == id_str) {
                start_opacity = shape.opacity;
            }

            let anim = FloatAnimation::new(
                entity_id,
                AnimatedProperty::Opacity,
                vec![
                    FloatKeyframe::new(0.0, start_opacity, EasingFunction::EaseInOut),
                    FloatKeyframe::new(1.0, target_opacity, EasingFunction::EaseInOut),
                ],
            )
            .with_config(AnimationConfig {
                duration: Duration::from_millis(duration_ms as u64),
                ..Default::default()
            });

            self.animation_manager.add_float_animation(anim);
        }
    }

    #[wasm_bindgen]
    pub fn get_animation_count(&self) -> usize {
        self.animation_manager.len()
    }

    // ===== Zoom System =====

    #[wasm_bindgen]
    pub fn update_zoom(&mut self, delta_ms: f64) -> bool {
        let delta = Duration::from_secs_f64(delta_ms / 1000.0);
        self.zoom_manager.update(delta)
    }

    // ===== Grid/Snap =====

    #[wasm_bindgen]
    pub fn snap_to_grid(&self, x: f32, y: f32) -> Box<[f32]> {
        let snapped = self.snap_helper.snap_to_grid(Vec2::new(x, y));
        vec![snapped.x, snapped.y].into_boxed_slice()
    }

    #[wasm_bindgen]
    pub fn snap_to_grid_json(&self, x: f32, y: f32, grid_size: f32) -> String {
        let snapped_x = (x / grid_size).round() * grid_size;
        let snapped_y = (y / grid_size).round() * grid_size;
        format!("[{},{}]", snapped_x, snapped_y)
    }

    #[wasm_bindgen]
    pub fn get_grid_size(&self) -> f32 {
        self.snap_helper.grid_size()
    }

    // ===== Color Palette =====

    #[wasm_bindgen]
    pub fn get_primary_color(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.color_palette.primary.r * 255.0) as u8,
            (self.color_palette.primary.g * 255.0) as u8,
            (self.color_palette.primary.b * 255.0) as u8
        )
    }

    #[wasm_bindgen]
    pub fn get_accent_color(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.color_palette.accent.r * 255.0) as u8,
            (self.color_palette.accent.g * 255.0) as u8,
            (self.color_palette.accent.b * 255.0) as u8
        )
    }

    // ===== Utility =====

    #[wasm_bindgen]
    pub fn version(&self) -> String {
        "2.0.0".to_string()
    }

    #[wasm_bindgen]
    pub fn log_info(&self, message: &str) {
        console_log!("{}", message);
    }
}

// Helper functions for JS interop
#[wasm_bindgen]
pub fn hex_to_rgb(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return String::from("[0,0,0]");
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    format!("[{},{},{}]", r, g, b)
}

#[wasm_bindgen]
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = t * 2.0 - 2.0;
        0.5 * t * t * t + 1.0
    }
}

#[wasm_bindgen]
pub fn ease_elastic(t: f32) -> f32 {
    let p = 0.3;
    let s = p / 4.0;
    (2.0_f32).powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin() + 1.0
}

#[wasm_bindgen]
pub fn ease_bounce(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}
