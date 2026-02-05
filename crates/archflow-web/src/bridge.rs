// ArchFlow Web - WASM Bridge
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
//
// WASM bridge for JavaScript/WebAssembly communication:
// - Exposes engine functions to JavaScript via wasm-bindgen
// - Handles SharedArrayBuffer for lock-free input
// - Provides requestAnimationFrame loop integration
// - Manages canvas and WebGPU context
//

#![allow(missing_docs)]

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::engine::ArchFlowEngine;
use crate::input::{InputProcessor, InputRingBuffer};

use archflow_engine::store::MAX_ENTITIES;
use archflow_render::Renderer;

#[cfg(target_arch = "wasm32")]
use archflow_render::WebGL2Renderer;

// Tracing support (conditionally compiled)
#[cfg(feature = "tracing-logging")]
use tracing::{debug, error, info, trace, warn};

/// Initialize tracing for WASM
///
/// This function sets up the tracing subscriber with wasm-tracing
/// for console output and performance timing.
///
/// Configures the subscriber to capture TRACE level and above from:
/// - archflow::wasm (bridge layer)
/// - archflow::engine (entity store, spatial hash)
/// - archflow::logic (sensors, actuators)
/// - archflow::render (rendering)
/// - archflow::interaction (user input handling)
#[cfg(feature = "tracing-logging")]
fn init_tracing() {
    use console_error_panic_hook::set_once;

    static mut TRACING_INIT: bool = false;
    unsafe {
        if TRACING_INIT {
            return;
        }
        TRACING_INIT = true;
    }

    // Set up panic hook to get better error messages
    set_once();

    // Initialize wasm-tracing as the global default subscriber (only on WASM)
    // This sends all tracing events to the browser console
    // Note: wasm-tracing doesn't support filters like tracing-subscriber,
    // so it will capture ALL events regardless of level
    #[cfg(target_arch = "wasm32")]
    wasm_tracing::set_as_global_default();

    info!(target: "archflow::wasm", "ArchFlow WASM tracing initialized");
    debug!(target: "archflow::wasm", "Debug mode: detailed traces enabled");
    trace!(target: "archflow::wasm", "Trace level: all events from all crates will be logged");
}

/// No-op tracing stub when tracing feature is disabled
#[cfg(not(feature = "tracing-logging"))]
fn init_tracing() {
    // Tracing disabled, do nothing
}

/// Convert RGBA color format to ABGR format
///
/// WebGL expects colors in ABGR format (little-endian), but Color::rgba creates RGBA.
/// This function performs the conversion.
#[inline]
fn rgba_to_abgr(rgba: u32) -> u32 {
    let r = (rgba >> 24) & 0xFF;
    let g = (rgba >> 16) & 0xFF;
    let b = (rgba >> 8) & 0xFF;
    let a = rgba & 0xFF;

    // ABGR format: A=highest byte, B, G, R=lowest byte
    (a << 24) | (b << 16) | (g << 8) | r
}

// WASM Bridge for JavaScript/WebAssembly communication
//
// This struct provides the interface between JavaScript and the Rust engine.
// It manages the engine lifecycle and exposes functions that can be called from JS.
#[wasm_bindgen]
pub struct WasmBridge {
    // Store engine and input processor in the struct for safe access
    engine: RefCell<Option<ArchFlowEngine>>,
    input_processor: RefCell<Option<InputProcessor>>,
    // Context loss handlers (HU-RENDER-009)
    #[cfg(target_arch = "wasm32")]
    on_context_lost: Cell<Option<Closure<dyn FnMut(web_sys::Event)>>>,
    #[cfg(target_arch = "wasm32")]
    on_context_restored: Cell<Option<Closure<dyn FnMut(web_sys::Event)>>>,
    #[cfg(target_arch = "wasm32")]
    is_recovering: Cell<bool>,
    #[cfg(target_arch = "wasm32")]
    #[cfg(target_arch = "wasm32")]
    pending_canvas: Cell<Option<web_sys::HtmlCanvasElement>>,
    #[cfg(target_arch = "wasm32")]
    canvas: RefCell<Option<web_sys::HtmlCanvasElement>>,
}

#[wasm_bindgen]
impl WasmBridge {
    /// Create a new WASM bridge
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize tracing on first bridge creation
        init_tracing();

        #[cfg(feature = "tracing-logging")]
        debug!(target: "archflow::wasm", "WasmBridge created");

        Self {
            engine: RefCell::new(None),
            input_processor: RefCell::new(None),
            #[cfg(target_arch = "wasm32")]
            on_context_lost: Cell::new(None),
            #[cfg(target_arch = "wasm32")]
            on_context_restored: Cell::new(None),
            #[cfg(target_arch = "wasm32")]
            is_recovering: Cell::new(false),
            #[cfg(target_arch = "wasm32")]
            pending_canvas: Cell::new(None),
            #[cfg(target_arch = "wasm32")]
            canvas: RefCell::new(None),
        }
    }

    /// Initialize the engine
    ///
    /// This should be called once when the application starts.
    #[wasm_bindgen]
    pub fn initialize(&self, canvas_width: f32, canvas_height: f32) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm", canvas_width = canvas_width, canvas_height = canvas_height, "Initializing engine");

        self.engine
            .borrow_mut()
            .replace(ArchFlowEngine::new(canvas_width, canvas_height));
        self.input_processor
            .borrow_mut()
            .replace(InputProcessor::new());

        #[cfg(feature = "tracing-logging")]
        debug!(target: "archflow::wasm", "Engine initialized successfully");

        Ok(())
    }

    /// Resize the engine and renderer
    #[wasm_bindgen]
    pub fn resize(&self, width: f32, height: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.resize(width, height);

            // Renderer resize is handled via engine since renderer is owned by engine?
            // Wait, engine.renderer is Box<dyn Renderer>.
            // Renderer trait has resize method.
            engine.renderer.resize(width as u32, height as u32);

            #[cfg(feature = "tracing-logging")]
            debug!(target: "archflow::wasm", width, height, "Resized engine and renderer");
        }
        Ok(())
    }

    /// Initialize graphics (uses WebGL2/Canvas 2D by default)
    ///
    /// This should be called after `initialize()` and after the canvas is mounted.
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn initialize_graphics(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        self.initialize_graphics_with_backend(canvas, "auto")
    }

    /// Detect available graphics backends
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn detect_available_backends(&self) -> Result<js_sys::Object, JsValue> {
        let result = js_sys::Object::new();

        // WebGL2: check browser support
        let webgl2_available = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.create_element("canvas").ok())
            .and_then(|canvas| {
                canvas
                    .dyn_ref::<web_sys::HtmlCanvasElement>()
                    .and_then(|c| c.get_context("webgl2").ok().map(|ctx| ctx.is_some()))
            })
            .unwrap_or(false);

        js_sys::Reflect::set(
            &result,
            &JsValue::from_str("webgl2"),
            &JsValue::from_bool(webgl2_available),
        )
        .map_err(|_| JsValue::from_str("Failed to set webgl2 property"))?;

        // WebGPU: check browser support - simplified check
        let webgpu_available = false; // WebGPU detection requires additional web-sys features

        js_sys::Reflect::set(
            &result,
            &JsValue::from_str("webgpu"),
            &JsValue::from_bool(webgpu_available),
        )
        .map_err(|_| JsValue::from_str("Failed to set webgpu property"))?;

        // Canvas 2D is always available in browsers
        js_sys::Reflect::set(
            &result,
            &JsValue::from_str("canvas2d"),
            &JsValue::from_bool(true),
        )
        .map_err(|_| JsValue::from_str("Failed to set canvas2d property"))?;

        // Preferred backend: WebGL2 > WebGPU > Canvas 2D
        let preferred = if webgl2_available {
            "webgl2"
        } else if webgpu_available {
            "webgpu"
        } else {
            "canvas2d"
        };
        js_sys::Reflect::set(
            &result,
            &JsValue::from_str("preferred"),
            &JsValue::from_str(preferred),
        )
        .map_err(|_| JsValue::from_str("Failed to set preferred property"))?;

        // Performance info
        let perf = js_sys::Object::new();
        js_sys::Reflect::set(
            &perf,
            &JsValue::from_str("webgl2"),
            &JsValue::from_str("60fps @ 50k entities"),
        )
        .ok();
        js_sys::Reflect::set(
            &perf,
            &JsValue::from_str("webgpu"),
            &JsValue::from_str("60fps @ 100k entities"),
        )
        .ok();
        js_sys::Reflect::set(
            &perf,
            &JsValue::from_str("canvas2d"),
            &JsValue::from_str("30fps @ 5k entities"),
        )
        .ok();
        js_sys::Reflect::set(&result, &JsValue::from_str("performance"), &perf)
            .map_err(|_| JsValue::from_str("Failed to set performance property"))?;

        Ok(result)
    }

    /// Initialize graphics with a specific backend
    ///
    /// Supported backends: "webgl2", "webgpu", "canvas2d", "auto"
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn initialize_graphics_with_backend(
        &self,
        canvas: web_sys::HtmlCanvasElement,
        backend: &str,
    ) -> Result<(), JsValue> {
        // Store canvas reference for DPI scaling
        self.canvas.borrow_mut().replace(canvas.clone());

        #[cfg(feature = "tracing-logging")]
        info!(
            target: "archflow::wasm",
            backend = %backend,
            "Initializing graphics with backend"
        );

        match backend {
            "webgl2" => self.try_initialize_webgl2(&canvas),
            "webgpu" => {
                // Fall back to WebGL2 for now
                self.try_initialize_webgl2(&canvas)
            }
            "canvas2d" => self.try_initialize_webgl2(&canvas),
            "auto" => {
                // Try WebGL2 first
                if let Ok(()) = self.try_initialize_webgl2(&canvas) {
                    return Ok(());
                }
                // Fall back to canvas2d (WebGL2 with 2D context)
                self.try_initialize_webgl2(&canvas)
            }
            _ => Err(JsValue::from_str(&alloc::format!(
                "Unknown backend: {}. Supported: webgl2, webgpu, canvas2d, auto",
                backend
            ))),
        }
    }

    /// Try to initialize WebGL2/Canvas 2D rendering
    #[cfg(target_arch = "wasm32")]
    fn try_initialize_webgl2(&self, canvas: &web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm", "Initializing WebGL2/Canvas 2D renderer");

        let width = canvas.width();
        let height = canvas.height();

        web_sys::console::log_1(&JsValue::from_str(&alloc::format!(
            "Creating WebGL2 context, canvas size: {}x{}",
            width,
            height
        )));

        // Create WebGL2 renderer directly from canvas
        let mut renderer = match archflow_render::WebGL2Renderer::new(canvas.clone()) {
            Ok(renderer) => renderer,
            Err(e) => {
                web_sys::console::error_1(&JsValue::from_str(&alloc::format!(
                    "Renderer creation error: {:?}",
                    e
                )));
                return Err(JsValue::from_str(&alloc::format!(
                    "Failed to create WebGL2 renderer: {:?}",
                    e
                )));
            }
        };
        renderer.resize(width, height);

        web_sys::console::log_1(&JsValue::from_str("Renderer created, setting in engine..."));

        // Set the renderer in the engine
        match self.engine.try_borrow_mut() {
            Ok(mut engine_borrow) => {
                if let Some(engine) = engine_borrow.as_mut() {
                    engine.set_renderer(Box::new(renderer));
                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm", "WebGL2 renderer initialized successfully");
                    web_sys::console::log_1(&JsValue::from_str(
                        "WebGL2 renderer initialized successfully",
                    ));
                } else {
                    web_sys::console::error_1(&JsValue::from_str("Engine not initialized"));
                    return Err(JsError::new("Engine not initialized").into());
                }
            }
            Err(_) => {
                web_sys::console::error_1(&JsValue::from_str("RefCell already borrowed"));
                return Err(JsError::new("RefCell already borrowed").into());
            }
        }

        // Register context loss handlers (HU-RENDER-009)
        self.register_context_handlers(canvas);

        Ok(())
    }

    /// Stub for non-WASM targets (should never be called)
    #[cfg(not(target_arch = "wasm32"))]
    fn try_initialize_webgl2(&self, _canvas: &()) -> Result<(), JsValue> {
        Err(JsError::new("Graphics initialization only available on WASM").into())
    }

    // ════════════════════════════════════════════════════════════════════════════════
    // CONTEXT LOSS HANDLERS (HU-RENDER-009)
    // ════════════════════════════════════════════════════════════════════════════════

    /// Register context loss and restoration event handlers
    #[cfg(target_arch = "wasm32")]
    fn register_context_handlers(&self, canvas: &web_sys::HtmlCanvasElement) {
        use wasm_bindgen::closure::Closure;

        // Store canvas for recovery
        self.pending_canvas.set(Some(canvas.clone()));

        // Clone canvas for use in closure (avoiding lifetime issues)
        let canvas_for_recovery = canvas.clone();

        // Context lost handler
        let on_lost = Closure::wrap(Box::new(move |event: web_sys::Event| {
            event.prevent_default();

            #[cfg(feature = "tracing-logging")]
            warn!(target: "archflow::web", "WebGL context lost - scheduling recovery");

            web_sys::console::warn_1(&JsValue::from_str(
                "WebGL context lost - attempting recovery",
            ));

            // Schedule recovery asynchronously
            let canvas = canvas_for_recovery.clone();
            let closure = Closure::wrap(Box::new(move || {
                let _ = Self::recover_context_internal(&canvas);
            }) as Box<dyn FnMut()>);

            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    100,
                );

            closure.forget();
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Context restored handler
        let on_restored = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            #[cfg(feature = "tracing-logging")]
            info!(target: "archflow::web", "WebGL context restored");

            web_sys::console::log_1(&JsValue::from_str("WebGL context restored"));
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Register listeners
        if let Err(e) = canvas
            .add_event_listener_with_callback("webglcontextlost", on_lost.as_ref().unchecked_ref())
        {
            #[cfg(feature = "tracing-logging")]
            error!(target: "archflow::web", error = ?e, "Failed to register contextlost listener");
        }

        if let Err(e) = canvas.add_event_listener_with_callback(
            "webglcontextrestored",
            on_restored.as_ref().unchecked_ref(),
        ) {
            #[cfg(feature = "tracing-logging")]
            error!(target: "archflow::web", error = ?e, "Failed to register contextrestored listener");
        }

        // Store handlers to prevent GC
        self.on_context_lost.set(Some(on_lost));
        self.on_context_restored.set(Some(on_restored));

        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::web", "Context loss handlers registered");
    }

    /// Internal recovery function (called via setTimeout)
    #[cfg(target_arch = "wasm32")]
    fn recover_context_internal(canvas: &web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        // Re-initialize WebGL2
        // Note: This is a simplified recovery - full recovery would need to
        // re-create all textures, buffers, etc. from the EntityStore
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::web", "Attempting WebGL context recovery");

        web_sys::console::log_1(&JsValue::from_str("Attempting WebGL context recovery..."));

        // For now, we just log - full recovery requires re-initializing
        // the entire rendering pipeline which is complex
        #[cfg(feature = "tracing-logging")]
        warn!(target: "archflow::web",
            "Full context recovery requires re-creating all GPU resources - consider re-initializing");

        Ok(())
    }

    /// Check if context recovery is in progress
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn is_recovering(&self) -> bool {
        self.is_recovering.get()
    }

    /// Get a pointer to the SharedArrayBuffer for input events
    ///
    /// This returns a pointer to the InputRingBuffer that JavaScript can
    /// write to directly via SharedArrayBuffer.
    #[wasm_bindgen]
    pub fn get_input_buffer_ptr(&self) -> *mut InputRingBuffer {
        if let Some(mut processor) = self.input_processor.borrow_mut().take() {
            processor.buffer() as *mut InputRingBuffer
        } else {
            core::ptr::null_mut()
        }
    }

    /// Get the size of the input buffer in bytes
    #[wasm_bindgen]
    pub fn get_input_buffer_size() -> usize {
        core::mem::size_of::<InputRingBuffer>()
    }

    /// Push an input event from JavaScript
    ///
    /// This is a higher-level alternative to directly writing to SharedArrayBuffer.
    /// JavaScript can call this function to push input events.
    #[wasm_bindgen]
    pub fn push_input_event(
        &self,
        event_type: u8,
        x: f32,
        y: f32,
        buttons: u8,
        modifiers: u8,
    ) -> Result<(), JsValue> {
        use crate::input::{Buttons, InputEventType, Modifiers, RawInputEvent};

        let input_event_type = match event_type {
            0 => InputEventType::Down,
            1 => InputEventType::Move,
            2 => InputEventType::Up,
            3 => InputEventType::Wheel,
            4 => InputEventType::KeyDown,
            5 => InputEventType::KeyUp,
            _ => InputEventType::Move,
        };

        let button_flags = Buttons(buttons);
        let modifier_flags = Modifiers(modifiers);

        if self.canvas.borrow().is_some() {
            // IMPORTANT: x, y are ALREADY canvas-relative coordinates from JS
            // getCanvasPosition() in Canvas.tsx already computed (clientX - rect.left) * dpr
            // So we must NOT subtract rect.left/top again or we get wrong coordinates!
            let canvas_x = x;
            let canvas_y = y;

            let event = RawInputEvent::new(
                0,
                0,
                canvas_x,
                canvas_y,
                input_event_type,
                button_flags,
                modifier_flags,
            );

            if let Some(processor) = self.input_processor.borrow_mut().as_mut() {
                if processor.buffer().push_event(event) {
                    Ok(())
                } else {
                    // Logic full warning removed to reduce noise as per user request
                    Err(JsError::new("Input buffer full").into())
                }
            } else {
                Err(JsError::new("Input processor not initialized").into())
            }
        } else {
            Err(JsError::new("Canvas not initialized").into())
        }
    }

    /// Run one frame of the engine
    ///
    /// This should be called from requestAnimationFrame.
    #[wasm_bindgen]
    pub fn tick(&self, timestamp: f64) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            if let Some(processor) = self.input_processor.borrow_mut().as_mut() {
                let events = processor.process_events();
                for event in events {
                    Self::process_input_event(engine, &event);
                }
            }
            engine.tick(timestamp);
        }
        Ok(())
    }

    /// Process a single input event and update the engine
    fn process_input_event(engine: &mut ArchFlowEngine, event: &crate::input::RawInputEvent) {
        use crate::input::InputEventType;
        use archflow_core::Vec2;
        use archflow_engine::store::ShapeType;

        let event_type: InputEventType = match event.event_type {
            0 => InputEventType::Down,
            1 => InputEventType::Move,
            2 => InputEventType::Up,
            3 => InputEventType::Wheel,
            4 => InputEventType::KeyDown,
            5 => InputEventType::KeyUp,
            _ => InputEventType::Move,
        };

        match event_type {
            InputEventType::Down => {
                let tool = engine.active_tool.clone();
                let world_pos = engine.screen_to_world(event.x, event.y);

                #[cfg(feature = "tracing-logging")]
                info!(target: "archflow::wasm::input",
                    screen_pos = ?(event.x, event.y),
                    world_pos = ?world_pos,
                    camera_center = ?engine.camera.center,
                    camera_zoom = engine.camera.zoom,
                    canvas_size = ?(engine.canvas_width, engine.canvas_height),
                    tool = %tool,
                    "Input: Down");

                if tool == "select" {
                    // Standard selection logic
                    for &entity_idx in &engine.store.draw_order[..engine.store.alive_count()] {
                        let idx = entity_idx as usize;
                        if !engine.store.is_visible(idx) {
                            continue;
                        }
                        let pos = engine.store.pos(idx);
                        let size = engine.store.size(idx);
                        let half_size = size / 2.0;
                        let min = pos - half_size;
                        let max = pos + half_size;
                        if world_pos.x >= min.x
                            && world_pos.x <= max.x
                            && world_pos.y >= min.y
                            && world_pos.y <= max.y
                        {
                            let entity_id = archflow_core::EntityId::new(entity_idx);
                            engine.selected_entities.clear();
                            engine.selected_entities.push(entity_id);
                            engine.store.set_selected(idx, true);
                            #[cfg(feature = "tracing-logging")]
                            debug!(target: "archflow::wasm::input", entity_idx, "Selected entity");
                            break;
                        }
                    }
                } else if tool == "rectangle" || tool == "circle" || tool == "square" {
                    // Creation logic
                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm::input", tool = %tool, "Creating shape");

                    // Start with a small size that will be updated during drag
                    let default_size = Vec2::new(1.0, 1.0);
                    let id = engine.store.spawn(world_pos, default_size);
                    let idx = id.index().0 as usize;

                    // Set shape type
                    let shape = if tool == "circle" {
                        ShapeType::Circle
                    } else {
                        ShapeType::Rectangle
                    };
                    engine.store.set_shape_type(idx, shape as u8);

                    // Set active colors
                    web_sys::console::log_1(
                        &format!(
                            "🔵 Creating shape with active_color=0x{:08x}",
                            engine.active_color
                        )
                        .into(),
                    );

                    engine.store.colors[idx] = engine.active_color;
                    engine.store.stroke_colors[idx] = engine.active_stroke_color;
                    engine.store.stroke_widths[idx] = engine.active_stroke_width;

                    // Select and start creating
                    engine.selected_entities.clear();
                    engine.selected_entities.push(id);
                    engine.store.set_selected(idx, true);
                    engine.is_creating = true;
                    engine.drag_start = Some(world_pos);

                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm::input", entity_idx = idx, shape = ?shape, "Spawned entity");
                }
            }
            InputEventType::Move => {
                if engine.is_creating {
                    // Resize interactively while creating
                    if !engine.selected_entities.is_empty() {
                        if let Some(start_pos) = engine.drag_start {
                            let current_pos = engine.screen_to_world(event.x, event.y);
                            let id = engine.selected_entities[0];
                            let idx = id.index().0 as usize;

                            let min_x = start_pos.x.min(current_pos.x);
                            let min_y = start_pos.y.min(current_pos.y);
                            let max_x = start_pos.x.max(current_pos.x);
                            let max_y = start_pos.y.max(current_pos.y);

                            let width = (max_x - min_x).max(10.0);
                            let height = (max_y - min_y).max(10.0);
                            let center_x = min_x + width / 2.0;
                            let center_y = min_y + height / 2.0;

                            engine.store.set_pos(idx, Vec2::new(center_x, center_y));
                            engine.store.set_size(idx, Vec2::new(width, height));
                        }
                    }
                } else if !engine.selected_entities.is_empty() {
                    // Standard move logic
                    let world_delta = engine.screen_delta_to_world(event.x, event.y);
                    for entity_id in &engine.selected_entities {
                        let idx = archflow_core::EntityId::index(*entity_id);
                        let current_pos = engine.store.pos(idx.0 as usize);
                        engine
                            .store
                            .set_pos(idx.0 as usize, current_pos + world_delta);
                    }
                }
            }
            InputEventType::Up => {
                if engine.is_creating {
                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm::input", "Finished creating shape");

                    // If the shape was created with a single click (no drag), apply default size
                    if !engine.selected_entities.is_empty() {
                        let id = engine.selected_entities[0];
                        let idx = id.index().0 as usize;
                        let current_size = engine.store.size(idx);

                        // If size is still 1x1 (no drag occurred), apply a reasonable default
                        if current_size.x <= 1.0 && current_size.y <= 1.0 {
                            let default_shape_size = Vec2::new(150.0, 150.0);
                            engine.store.set_size(idx, default_shape_size);

                            #[cfg(feature = "tracing-logging")]
                            info!(target: "archflow::wasm::input",
                                entity_idx = idx,
                                size = ?default_shape_size,
                                "Applied default size for click-created shape");
                        }
                    }

                    engine.is_creating = false;
                    engine.drag_start = None;
                    // Reset tool to select after creation?
                    // engine.active_tool = alloc::string::String::from("select");
                    // Actually, keep it active for multiple shapes is better for "tool" paradigm
                }
            }
            _ => {}
        }
    }

    /// Spawn a new entity at the given position
    #[wasm_bindgen]
    pub fn spawn_entity(&self, x: f32, y: f32, width: f32, height: f32) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let id = engine.store.spawn(
                archflow_core::Vec2::new(x, y),
                archflow_core::Vec2::new(width, height),
            );
            let color = archflow_core::Color::rgb(
                (js_sys::Math::random() * 255.0) as u8,
                (js_sys::Math::random() * 255.0) as u8,
                (js_sys::Math::random() * 255.0) as u8,
            );
            let idx = id.index().0 as usize;
            engine.store.colors[idx] = color.0;
            Ok(id.index().0)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Move an entity by the given delta
    #[wasm_bindgen]
    pub fn move_entity(&self, entity_index: u32, dx: f32, dy: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            use archflow_engine::Command;
            let id = EntityId::new(entity_index);
            let cmd = Command::Move {
                id,
                delta: archflow_core::Vec2::new(dx, dy),
            };
            engine.command_queue.push(cmd);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the color of an entity
    #[wasm_bindgen]
    pub fn set_color(&self, entity_index: u32, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::{Color, EntityId};
            use archflow_engine::Command;
            let id = EntityId::new(entity_index);
            let rgba = Color::rgba(r, g, b, a).0;
            let color = rgba_to_abgr(rgba);
            let cmd = Command::SetColor { id, color };
            engine.command_queue.push(cmd);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the stroke color of an entity
    #[wasm_bindgen]
    pub fn set_stroke_color(
        &self,
        entity_index: u32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::{Color, EntityId};
            let id = EntityId::new(entity_index);
            let rgba = Color::rgba(r, g, b, a).0;
            let color = rgba_to_abgr(rgba);
            // TODO: Use Command for undo/redo
            engine.store.set_stroke_color(id.index().0 as usize, color);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the stroke width of an entity
    #[wasm_bindgen]
    pub fn set_stroke_width(&self, entity_index: u32, width: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            let id = EntityId::new(entity_index);
            // TODO: Use Command for undo/redo
            engine.store.set_stroke_width(id.index().0 as usize, width);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the active fill color for new shapes
    #[wasm_bindgen]
    pub fn set_active_color(&self, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let rgba = archflow_core::Color::rgba(r, g, b, a).0;
            let abgr = rgba_to_abgr(rgba);

            web_sys::console::log_1(
                &format!(
                    "🎨 set_active_color: R={} G={} B={} A={} | RGBA=0x{:08x} | ABGR=0x{:08x}",
                    r, g, b, a, rgba, abgr
                )
                .into(),
            );

            engine.active_color = abgr;
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the active stroke color for new shapes
    #[wasm_bindgen]
    pub fn set_active_stroke_color(&self, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let rgba = archflow_core::Color::rgba(r, g, b, a).0;
            engine.active_stroke_color = rgba_to_abgr(rgba);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the active stroke width for new shapes
    #[wasm_bindgen]
    pub fn set_active_stroke_width(&self, width: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.active_stroke_width = width;
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the active fill color (returns RGBA as hex string)
    #[wasm_bindgen]
    pub fn get_active_color(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let abgr = engine.active_color;
            // Convert ABGR back to RGBA for JavaScript
            let r = abgr & 0xFF;
            let g = (abgr >> 8) & 0xFF;
            let b = (abgr >> 16) & 0xFF;
            let a = (abgr >> 24) & 0xFF;
            Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the active stroke color (returns RGBA as hex string)
    #[wasm_bindgen]
    pub fn get_active_stroke_color(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let abgr = engine.active_stroke_color;
            // Convert ABGR back to RGBA for JavaScript
            let r = abgr & 0xFF;
            let g = (abgr >> 8) & 0xFF;
            let b = (abgr >> 16) & 0xFF;
            let a = (abgr >> 24) & 0xFF;
            Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the active stroke width
    #[wasm_bindgen]
    pub fn get_active_stroke_width(&self) -> Result<f32, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.active_stroke_width)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the number of alive entities
    #[wasm_bindgen]
    pub fn entity_count(&self) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.store.alive_count() as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Undo the last action
    #[wasm_bindgen]
    pub fn undo(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.undo();
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Redo the last undone action
    #[wasm_bindgen]
    pub fn redo(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.redo();
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the camera zoom level
    #[wasm_bindgen]
    pub fn set_zoom(&self, zoom: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.camera.zoom = zoom.clamp(archflow_render::ZOOM_MIN, archflow_render::ZOOM_MAX);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the current camera zoom level
    #[wasm_bindgen]
    pub fn get_zoom(&self) -> Result<f32, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.camera.zoom)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the camera center position
    #[wasm_bindgen]
    pub fn set_camera_center(&self, x: f32, y: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.camera.center = archflow_core::Vec2f64::new(x as f64, y as f64);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the camera center position
    #[wasm_bindgen]
    pub fn get_camera_center(&self) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let array = js_sys::Array::new();
            array.push(&JsValue::from(engine.camera.center.x));
            array.push(&JsValue::from(engine.camera.center.y));
            Ok(array)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Serialize the current project
    #[wasm_bindgen]
    pub fn serialize_project(&self) -> Result<js_sys::Uint8Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            use archflow_export::ProjectSerializer;
            let data = ProjectSerializer::serialize(&engine.store, &engine.connection_store);
            let array = unsafe { js_sys::Uint8Array::view(&data) };
            Ok(array)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Clear all entities
    #[wasm_bindgen]
    pub fn clear(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.store = archflow_engine::EntityStore::new();
            engine.selected_entities.clear();
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the shape type of an entity
    #[wasm_bindgen]
    pub fn set_shape(&self, entity_index: u32, shape: u8) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            use archflow_engine::Command;
            let id = EntityId::new(entity_index);
            let cmd = Command::SetShape { id, shape };
            engine.command_queue.push(cmd);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the label of an entity
    #[wasm_bindgen]
    pub fn set_label(&self, entity_index: u32, label: &str) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Invalid entity index").into());
            }
            engine.store.string_pool.set(idx, label);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get list of alive entity indices
    #[wasm_bindgen]
    pub fn get_alive_entities(&self) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.store.draw_order[..engine.store.alive_count()].to_vec())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get entity position in screen coordinates
    #[wasm_bindgen]
    pub fn get_entity_position_screen(&self, entity_index: u32) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            let world_pos = engine.store.pos(idx);
            let (screen_x, screen_y) = engine.world_to_screen(world_pos);
            let array = js_sys::Array::new();
            array.push(&JsValue::from(screen_x));
            array.push(&JsValue::from(screen_y));
            Ok(array)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get entity size in screen coordinates
    #[wasm_bindgen]
    pub fn get_entity_size_screen(&self, entity_index: u32) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            let size = engine.store.size(idx);
            let screen_width = size.x * engine.camera.zoom * engine.canvas_width / 800.0;
            let screen_height = size.y * engine.camera.zoom * engine.canvas_height / 600.0;
            let array = js_sys::Array::new();
            array.push(&JsValue::from(screen_width));
            array.push(&JsValue::from(screen_height));
            Ok(array)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get entity color as hex string
    #[wasm_bindgen]
    pub fn get_entity_color_hex(&self, entity_index: u32) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            let color = engine.store.colors[idx];
            let r = (color >> 24) & 0xFF;
            let g = (color >> 16) & 0xFF;
            let b = (color >> 8) & 0xFF;
            Ok(format!("#{:02X}{:02X}{:02X}", r, g, b))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get entity shape type
    #[wasm_bindgen]
    pub fn get_entity_shape(&self, entity_index: u32) -> Result<u8, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            Ok(engine.store.shape_type(idx))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get entity label from string pool
    #[wasm_bindgen]
    pub fn get_entity_label(&self, entity_index: u32) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            Ok(engine.store.string_pool.get(idx).to_string())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Check if entity is visible
    #[wasm_bindgen]
    pub fn is_entity_visible(&self, entity_index: u32) -> Result<bool, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            Ok(engine.store.is_visible(idx))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Check if entity is selected
    #[wasm_bindgen]
    pub fn is_entity_selected(&self, entity_index: u32) -> Result<bool, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            Ok(engine.store.is_selected(idx))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set entity visibility
    #[wasm_bindgen]
    pub fn set_entity_visible(&self, entity_index: u32, visible: bool) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            engine.store.set_visible(idx, visible);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the current tool type
    #[wasm_bindgen]
    pub fn set_tool(&self, tool: &str) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm", tool = %tool, "Setting tool");

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.active_tool = alloc::string::String::from(tool);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the current tool type
    #[wasm_bindgen]
    pub fn get_tool(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.active_tool.clone())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Clear all selected entities
    #[wasm_bindgen]
    pub fn clear_selection(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            for &entity_id in &engine.selected_entities {
                let idx = entity_id.index().0 as usize;
                engine.store.set_selected(idx, false);
            }
            engine.selected_entities.clear();
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Add an entity to the selection
    #[wasm_bindgen]
    pub fn select_entity(&self, entity_index: u32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            let id = EntityId::new(entity_index);
            engine.selected_entities.push(id);
            let idx = id.index().0 as usize;
            if idx < MAX_ENTITIES {
                engine.store.set_selected(idx, true);
            }
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the list of selected entity IDs
    #[wasm_bindgen]
    pub fn get_selection(&self) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let array = js_sys::Array::new();
            for &entity_id in &engine.selected_entities {
                array.push(&JsValue::from(entity_id.index().0));
            }
            Ok(array)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the selection state of an entity directly
    #[wasm_bindgen]
    pub fn set_entity_selected(&self, entity_index: u32, selected: bool) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Invalid entity index").into());
            }
            engine.store.set_selected(idx, selected);
            if selected {
                use archflow_core::EntityId;
                let id = EntityId::new(entity_index);
                if !engine.selected_entities.contains(&id) {
                    engine.selected_entities.push(id);
                }
            } else {
                engine
                    .selected_entities
                    .retain(|id| id.index().0 != entity_index as u32);
            }
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the size of an entity
    #[wasm_bindgen]
    pub fn set_size(&self, entity_index: u32, width: f32, height: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            use archflow_engine::Command;
            let id = EntityId::new(entity_index);
            let cmd = Command::Resize {
                id,
                size: archflow_core::Vec2::new(width, height),
            };
            engine.command_queue.push(cmd);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the position of an entity
    #[wasm_bindgen]
    pub fn set_position(&self, entity_index: u32, x: f32, y: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            use archflow_engine::Command;
            let id = EntityId::new(entity_index);
            let cmd = Command::Teleport {
                id,
                pos: archflow_core::Vec2::new(x, y),
            };
            engine.command_queue.push(cmd);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Duplicate an entity (create a copy at a slight offset)
    #[wasm_bindgen]
    pub fn duplicate_entity(&self, entity_index: u32) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::Vec2;
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            let pos = engine.store.pos(idx);
            let size = engine.store.size(idx);
            let color = engine.store.colors[idx];
            let shape = engine.store.shape_type(idx);
            let new_id = engine.store.spawn(pos + Vec2::new(20.0, 20.0), size);
            let new_idx = new_id.index().0 as usize;
            engine.store.colors[new_idx] = color;
            engine.store.set_shape_type(new_idx, shape);
            Ok(new_id.index().0)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Delete all selected entities
    #[wasm_bindgen]
    pub fn delete_selected(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_engine::Command;
            let entities_to_delete: alloc::vec::Vec<_> =
                engine.selected_entities.iter().copied().collect();
            for id in entities_to_delete {
                let cmd = Command::Despawn(id);
                engine.command_queue.push(cmd);
            }
            engine.selected_entities.clear();
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Check if undo is available
    #[wasm_bindgen]
    pub fn can_undo(&self) -> Result<bool, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.can_undo())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Check if redo is available
    #[wasm_bindgen]
    pub fn can_redo(&self) -> Result<bool, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.can_redo())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get history state for UI feedback
    #[wasm_bindgen]
    pub fn get_history_state(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let undo_count = engine.history.undo_count();
            let redo_count = engine.history.redo_count();
            Ok(alloc::format!("undo:{},redo:{}", undo_count, redo_count))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }
}

/// Custom error type for JavaScript
#[wasm_bindgen]
pub struct JsError {
    message: String,
}

#[wasm_bindgen]
impl JsError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Self {
        Self {
            message: alloc::format!("{}", message),
        }
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

// UNIT TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = WasmBridge::new();
        // Should not panic
    }

    #[test]
    fn test_js_error() {
        let error = JsError::new("Test error");
        assert_eq!(error.message(), "Test error");
    }

    #[test]
    fn test_input_buffer_size() {
        let expected_size = core::mem::size_of::<InputRingBuffer>();
        assert!(expected_size > 0);
    }

    #[test]
    fn test_bridge_with_engine() {
        let bridge = WasmBridge::new();
        // Can initialize
        assert!(bridge.initialize(800.0, 600.0).is_ok());
        // Can get entity count
        assert!(bridge.entity_count().is_ok());
    }
}

    #[test]
    fn test_color_conversion_rgba_to_abgr() {
        // Test con rojo puro
        let red_rgba = archflow_core::Color::rgba(255, 0, 0, 255).0;
        let red_abgr = rgba_to_abgr(red_rgba);
        // RGBA: 0xFF0000FF
        // ABGR: 0xFF0000FF (mismo porque R y A están en posiciones simétricas)
        assert_eq!(red_rgba, 0xFF0000FF, "Red RGBA should be 0xFF0000FF");
        assert_eq!(red_abgr, 0xFF0000FF, "Red ABGR should be 0xFF0000FF");
        
        // Test con verde puro
        let green_rgba = archflow_core::Color::rgba(0, 255, 0, 255).0;
        let green_abgr = rgba_to_abgr(green_rgba);
        // RGBA: 0x00FF00FF
        // ABGR: 0xFF00FF00
        assert_eq!(green_rgba, 0x00FF00FF, "Green RGBA should be 0x00FF00FF");
        assert_eq!(green_abgr, 0xFF00FF00, "Green ABGR should be 0xFF00FF00");
        
        // Test con azul por defecto
        let blue_rgba = archflow_core::Color::rgba(59, 130, 246, 255).0;
        let blue_abgr = rgba_to_abgr(blue_rgba);
        // RGBA: 0x3B82F6FF
        // ABGR: 0xFFF6823B
        assert_eq!(blue_rgba, 0x3B82F6FF, "Blue RGBA should be 0x3B82F6FF");
        assert_eq!(blue_abgr, 0xFFF6823B, "Blue ABGR should be 0xFFF6823B");
        
        println!("✅ Conversión RGBA → ABGR funciona correctamente");
    }
