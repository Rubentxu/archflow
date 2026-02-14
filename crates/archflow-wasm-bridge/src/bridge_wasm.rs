// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - WASM Bridge
//
// Architecture Reference: docs/analysis/ARCHITECTURE-CLEAN-BRIDGE.md
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// SECTION 1: IMPORTS AND UTILITIES
// - Standard library imports (alloc, core)
// - WASM bindings
// - Engine and domain types
// - Helper functions (rgba_to_abgr)
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::engine::ArchFlowEngine;
use crate::input::{InputEventType, InputProcessor, InputRingBuffer};

use archflow_engine::store::MAX_ENTITIES;
use archflow_engine::{Command, DeltaMask};
use archflow_logic::mapping::{ActuatorType, Controller, LogicMappingTable, SensorType};
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

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// SECTION 2: STRUCT DEFINITION - WasmBridge
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
//
// Main entry point for WASM bridge. Provides:
// - Engine lifecycle (initialize, resize, graphics setup)
// - Input handling (SharedArrayBuffer, push_input_event)
// - Frame loop (tick, poll_events)
// - Entity operations
// - Selection management
// - Camera control
// - History (undo/redo)
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
pub struct WasmBridge {
    /// Core engine instance - wrapped in RefCell for interior mutability
    engine: RefCell<Option<ArchFlowEngine>>,
    /// Input processor for handling user input events
    input_processor: RefCell<Option<InputProcessor>>,
    /// Context loss handlers (HU-RENDER-009)
    #[cfg(target_arch = "wasm32")]
    on_context_lost: Cell<Option<Closure<dyn FnMut(web_sys::Event)>>>,
    #[cfg(target_arch = "wasm32")]
    on_context_restored: Cell<Option<Closure<dyn FnMut(web_sys::Event)>>>,
    #[cfg(target_arch = "wasm32")]
    is_recovering: Cell<bool>,
    #[cfg(target_arch = "wasm32")]
    pending_canvas: Cell<Option<web_sys::HtmlCanvasElement>>,
    #[cfg(target_arch = "wasm32")]
    canvas: RefCell<Option<web_sys::HtmlCanvasElement>>,
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// SECTION 3: WASM-EXPOSED METHODS - Initialization & Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════

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

        // Set the renderer in the engine
        match self.engine.try_borrow_mut() {
            Ok(mut engine_borrow) => {
                if let Some(engine) = engine_borrow.as_mut() {
                    engine.set_renderer(Box::new(renderer));
                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm", "WebGL2 renderer initialized successfully");
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

            #[cfg(target_arch = "wasm32")]
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

        #[cfg(target_arch = "wasm32")]
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
    #[cfg(target_arch = "wasm32")]
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

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 4: FRAME LOOP - tick() and poll_events()
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Run one frame of the engine
    ///
    /// This should be called from requestAnimationFrame.
    /// Uses the fluent API: sample_input() → tick() → poll_events()
    #[wasm_bindgen]
    pub fn tick(&self, timestamp: f64) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            // Get current input from SharedArrayBuffer
            if let Some(processor) = self.input_processor.borrow_mut().as_mut() {
                let events = processor.process_events();

                // Process raw input events
                for event in events {
                    Self::process_input_event(engine, &event);
                }
            }

            // Delegate to the main engine tick which handles:
            // 1. Logic Bricks tick (Sensors -> Controllers -> Actuators)
            // 2. Command execution (Apply changes to EntityStore)
            // 3. Connection updates
            // 4. Rendering (Prepare -> Draw)
            engine.tick(timestamp);
        }
        Ok(())
    }

    /// Poll all events from the logic system
    ///
    /// Returns a JavaScript array of events emitted by the logic system
    /// during the current frame. Call this once per frame after `tick()`.
    ///
    /// # Returns
    ///
    /// Number of events generated during the current frame.
    /// Call this once per frame after `tick()`.
    ///
    /// # Example
    ///
    /// ```javascript
    /// // In your JavaScript/TypeScript code
    /// const eventCount = bridge.poll_events();
    /// if (eventCount > 0) {
    ///     console.log('Events generated:', eventCount);
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn poll_events(&self) -> usize {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.logic_bricks.poll_events()
        } else {
            0
        }
    }

    /// Process a single input event and update the engine
    fn process_input_event(engine: &mut ArchFlowEngine, event: &crate::input::RawInputEvent) {
        use crate::input::InputEventType;
        use archflow_core::Vec2;

        // Calculate world position
        let world_pos = engine.screen_to_world(event.x, event.y);

        // Update Logic Bricks input state with current event data
        // This is CRITICAL for creation logic which runs in tick() and needs valid world coordinates
        // to calculate the shape size correctly.
        engine.logic_bricks.sample_input(
            event.x,
            event.y,
            world_pos.x,
            world_pos.y,
            event.buttons,
            0, // Wheel delta
            event.modifiers,
        );

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
                    // Logic Bricks handles selection via MappingTable and filters pulses.
                    // We only need to track last mouse position for coordinate conversion if needed,
                    // but the actual selection/move logic is now in logic_bricks.tick()
                    engine.last_mouse_screen_pos = Some(Vec2::new(event.x, event.y));
                } else if tool == "rectangle" || tool == "circle" || tool == "square" {
                    // ═══════════════════════════════════════════════════════════════════
                    // CREATION (using Logic Bricks for state management)
                    // ═══════════════════════════════════════════════════════════════════

                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm::input", tool = %tool, "Creating shape via Logic Bricks");

                    // Store creation start position
                    engine
                        .logic_bricks
                        .set_creation_start(world_pos.x, world_pos.y);

                    // Start creation via engine helper (spawns entity + selects it)
                    let id = engine.start_creation(world_pos);
                    let idx = id.index().0 as usize;

                    // Set shape type
                    let shape = if tool == "circle" {
                        archflow_engine::store::ShapeType::Circle
                    } else {
                        archflow_engine::store::ShapeType::Rectangle
                    };
                    engine.store.set_shape_type(idx, shape as u8);

                    // Set active colors
                    engine.store.colors[idx] = engine.active_color;
                    engine.store.stroke_colors[idx] = engine.active_stroke_color;
                    engine.store.stroke_widths[idx] = engine.active_stroke_width;

                    #[cfg(feature = "tracing-logging")]
                    info!(target: "archflow::wasm::input", entity_idx = idx, shape = ?shape, "Spawned entity via Logic Bricks");
                }
            }
            InputEventType::Move => {
                let world_pos = engine.screen_to_world(event.x, event.y);

                // ═══════════════════════════════════════════════════════════════════
                // DRAG or CREATION update (using Logic Bricks)
                // ═══════════════════════════════════════════════════════════════════

                if engine.logic_bricks.is_creating() {
                    // Creation mode - resize via Logic Bricks
                    // The update is handled in tick() via process_tool_operations
                } else {
                    // Check if any entity is being dragged via MoveActuator
                    let selected = engine.logic_bricks.get_selected_entities();
                    let length = selected.length();
                    if length > 0 {
                        // Use MoveActuator update (handles hysteresis internally)
                        for i in 0..length {
                            let entity_id_value =
                                js_sys::Reflect::get(&selected, &JsValue::from(i))
                                    .unwrap_or(JsValue::NULL);
                            if let Some(id) = entity_id_value.as_f64() {
                                let entity_id = archflow_core::EntityId::new(id as u32);
                                let _ = engine.update_drag(entity_id, world_pos);
                            }
                        }
                    }
                }

                engine.last_mouse_screen_pos = Some(Vec2::new(event.x, event.y));
            }
            InputEventType::Up => {
                // ═══════════════════════════════════════════════════════════════════
                // END DRAG or CREATION (using Logic Bricks)
                // ═══════════════════════════════════════════════════════════════════

                if let Some(entity_id) = engine.logic_bricks.get_creating_entity_id() {
                    // End creation
                    engine.end_creation(entity_id);
                    engine.logic_bricks.set_creating_entity_id(None);
                } else {
                    // Clear drag state
                    engine.clear_drag_state();
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

    /// Bulk spawn multiple entities in a single call - ZERO-COPY
    ///
    /// This is the MOST EFFICIENT way to spawn entities:
    /// - positions: flat array of [x0, y0, x1, y1, ...] (2 * count floats)
    /// - sizes: flat array of [w0, h0, w1, h1, ...] (2 * count floats)
    /// - colors: flat array of [r0, g0, b0, a0, r1, g1, b1, a1, ...] (4 * count u8s)
    ///   Pass empty Uint8Array() for random colors
    ///
    /// Returns: array of spawned entity indices
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const positions = new Float32Array([100, 100, 200, 200, 300, 300]);
    /// const sizes = new Float32Array([50, 50, 60, 60, 70, 70]);
    /// const colors = new Uint8Array([255, 0, 0, 255, 0, 255, 0, 255]); // or empty for random
    /// const ids = bridge.bulk_spawn(positions, sizes, colors);
    /// ```
    #[wasm_bindgen]
    pub fn bulk_spawn(
        &self,
        positions: &[f32],
        sizes: &[f32],
        colors: &[u8],
    ) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let count = positions.len() / 2;
            if sizes.len() / 2 != count {
                return Err(JsError::new("positions and sizes length mismatch").into());
            }
            let has_colors = colors.len() >= count * 4;

            let mut result = Vec::with_capacity(count);

            for i in 0..count {
                let x = positions[i * 2];
                let y = positions[i * 2 + 1];
                let w = sizes[i * 2];
                let h = sizes[i * 2 + 1];

                let id = engine.store.spawn(
                    archflow_core::Vec2::new(x, y),
                    archflow_core::Vec2::new(w, h),
                );

                // Set color if provided
                if has_colors {
                    let r = colors[i * 4];
                    let g = colors[i * 4 + 1];
                    let b = colors[i * 4 + 2];
                    let a = colors[i * 4 + 3];
                    let color = archflow_core::Color::rgba(r, g, b, a);
                    let idx = id.index().0 as usize;
                    engine.store.colors[idx] = color.0;
                } else {
                    // Random color
                    let color = archflow_core::Color::rgb(
                        (js_sys::Math::random() * 255.0) as u8,
                        (js_sys::Math::random() * 255.0) as u8,
                        (js_sys::Math::random() * 255.0) as u8,
                    );
                    let idx = id.index().0 as usize;
                    engine.store.colors[idx] = color.0;
                }

                result.push(id.index().0);
            }

            Ok(result)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Spawn a pool of pre-allocated entities for optimal performance
    ///
    /// Use this to pre-allocate entities at startup, then use set_visible()
    /// to show/hide them instead of spawning/despawning.
    ///
    /// Returns: number of entities spawned
    #[wasm_bindgen]
    pub fn spawn_pool(&self, count: u32) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            for _ in 0..count {
                engine.store.spawn(
                    archflow_core::Vec2::new(-1000.0, -1000.0), // Off-screen
                    archflow_core::Vec2::new(0.0, 0.0),
                );
            }
            Ok(count)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PHYSICS METHODS (EPIC-AFRAME-006)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Set velocity for physics simulation
    /// vx, vy = velocity in units/second
    #[wasm_bindgen]
    pub fn set_velocity(&self, entity_id: u32, vx: f32, vy: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_id as usize;
            if idx >= archflow_engine::MAX_ENTITIES {
                return Err(JsError::new("Entity ID out of bounds").into());
            }
            engine.store.set_velocity(idx, vx, vy);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get velocity of an entity
    /// Returns [vx, vy]
    #[wasm_bindgen]
    pub fn get_velocity(&self, entity_id: u32) -> Result<Vec<f32>, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_id as usize;
            if idx >= archflow_engine::MAX_ENTITIES {
                return Err(JsError::new("Entity ID out of bounds").into());
            }
            let vel = engine.store.velocity(idx);
            Ok(vec![vel.x, vel.y])
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set acceleration for physics simulation
    /// ax, ay = acceleration in units/second^2
    #[wasm_bindgen]
    pub fn set_acceleration(&self, entity_id: u32, ax: f32, ay: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_id as usize;
            if idx >= archflow_engine::MAX_ENTITIES {
                return Err(JsError::new("Entity ID out of bounds").into());
            }
            engine.store.set_acceleration(idx, ax, ay);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set physics material properties
    /// restitution: 0.0 = no bounce, 1.0 = full bounce
    /// friction: 0.0 = no friction, 1.0 = high friction
    /// mass: 0.0 = infinite/static, >0 = dynamic
    #[wasm_bindgen]
    pub fn set_physics_material(
        &self,
        entity_id: u32,
        restitution: f32,
        friction: f32,
        mass: f32,
    ) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_id as usize;
            if idx >= archflow_engine::MAX_ENTITIES {
                return Err(JsError::new("Entity ID out of bounds").into());
            }
            engine
                .store
                .set_physics_material(idx, restitution, friction, mass);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Batch set physics material for multiple entities
    /// This is more efficient than calling set_physics_material for each entity
    #[wasm_bindgen]
    pub fn batch_set_physics_materials(
        &self,
        ids: &[u32],
        restitution: f32,
        friction: f32,
        mass: f32,
    ) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            for &id in ids {
                let idx = id as usize;
                if idx < archflow_engine::MAX_ENTITIES {
                    engine
                        .store
                        .set_physics_material(idx, restitution, friction, mass);
                }
            }
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Integrate physics for all entities
    /// This should be called every frame for physics to work
    /// Returns number of entities processed
    #[wasm_bindgen]
    pub fn integrate_physics(
        &self,
        dt: f32,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    ) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            // Use batched integration for better performance with large entity counts
            let count = engine
                .store
                .integrate_all_physics_batched(dt, min_x, min_y, max_x, max_y);
            Ok(count as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Batch set velocities for multiple entities
    /// ids: array of entity IDs
    /// vx, vy: flat arrays of velocities
    #[wasm_bindgen]
    pub fn batch_set_velocities(&self, ids: &[u32], vx: &[f32], vy: &[f32]) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            if ids.len() != vx.len() || ids.len() != vy.len() {
                return Err(JsError::new("Array length mismatch").into());
            }
            for i in 0..ids.len() {
                let idx = ids[i] as usize;
                if idx < archflow_engine::MAX_ENTITIES {
                    engine.store.set_velocity(idx, vx[i], vy[i]);
                }
            }
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 6: LOGIC BRICKS - Sensors, Controllers, Actuators
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Add a sensor connection to an entity
    ///
    /// Creates a sensor-to-actuator connection using the LogicMappingTable.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity to add the sensor to
    /// * `sensor_type` - Type of sensor (0=MouseOver, 1=MouseClick, 2=Proximity, 3=KeyShortcut, 4=Touch, 5=Radar, 6=DoubleTap, 7=LongPress, 8=RightClick)
    /// * `controller_type` - Type of controller (0=Direct, 1=AND, 2=OR, 3=NOT)
    /// * `actuator_type` - Type of actuator (0=Highlight, 1=Select, 2=Move, 3=Sound, 4=Animation, 5=Custom, 6=Property, 7=Visibility)
    ///
    /// # Returns
    ///
    /// Ok(true) if connection was added successfully
    #[wasm_bindgen]
    pub fn add_sensor(
        &self,
        entity_id: u32,
        sensor_type: u8,
        controller_type: u8,
        actuator_type: u8,
    ) -> Result<bool, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let entity = archflow_core::EntityId::new(entity_id);

            // Convert sensor_type to SensorType
            let sensor = match SensorType::from_index(sensor_type) {
                Some(s) => s,
                None => {
                    return Err(JsError::new("Invalid sensor type").into());
                }
            };

            // Convert controller_type to Controller
            let controller = match controller_type {
                0 => Controller::Direct,
                1 => Controller::AND(SensorType::MouseOver), // Default AND sensor
                2 => Controller::OR(SensorType::MouseOver),  // Default OR sensor
                3 => Controller::NOT,
                4 => Controller::Blinky { interval: 30 },
                5 => Controller::Debounce { ticks: 5 },
                _ => Controller::Direct,
            };

            // Convert actuator_type to ActuatorType (only 7 types: Highlight, Select, Move, Delete, Undo, Redo, Camera)
            let actuator = match actuator_type {
                0 => ActuatorType::Highlight,
                1 => ActuatorType::Select,
                2 => ActuatorType::Move,
                3 => ActuatorType::Delete,
                4 => ActuatorType::Undo,
                5 => ActuatorType::Redo,
                6 => ActuatorType::Camera,
                _ => ActuatorType::Highlight,
            };

            // Add the connection to mapping table
            engine
                .logic_bricks
                .mapping_table_mut()
                .add_connection(entity, sensor, controller, actuator);

            Ok(true)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Remove a sensor connection from an entity
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity to remove the sensor from
    /// * `sensor_type` - Type of sensor to disconnect
    #[wasm_bindgen]
    pub fn remove_sensor(&self, entity_id: u32, sensor_type: u8) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let entity = archflow_core::EntityId::new(entity_id);

            if let Some(sensor) = SensorType::from_index(sensor_type) {
                engine
                    .logic_bricks
                    .mapping_table_mut()
                    .remove_connection(entity, sensor);
            }

            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get number of connections for an entity
    #[wasm_bindgen]
    pub fn connection_count(&self, entity_id: u32) -> Result<usize, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let entity = archflow_core::EntityId::new(entity_id);
            Ok(engine.logic_bricks.mapping_table().connection_count(entity))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Clear all logic connections for an entity
    #[wasm_bindgen]
    pub fn clear_entity_logic(&self, entity_id: u32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let entity = archflow_core::EntityId::new(entity_id);
            engine.logic_bricks.mapping_table_mut().clear_entity(entity);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Clear all logic connections for all entities
    #[wasm_bindgen]
    pub fn clear_all_logic(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            // Get all entities from the store and clear their connections
            // For simplicity, we create a new mapping table
            *engine.logic_bricks.mapping_table_mut() = LogicMappingTable::new();
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Configure mouse sensor for an entity
    ///
    /// # Arguments
    ///
    /// * `mode` - Mouse mode: 0=movement, 1=left_button, 2=right_button, 3=middle_button, 4=wheel_up
    /// * `tap` - Enable tap detection (true) or continuous (false)
    #[wasm_bindgen]
    pub fn configure_mouse_sensor(&self, mode: u8, tap: bool) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_logic::sensors::MouseConfig;

            let config = match mode {
                0 => MouseConfig::movement(),
                1 => MouseConfig::left_button().tap(tap),
                2 => MouseConfig::right_button().tap(tap),
                3 => MouseConfig::middle_button().tap(tap),
                4 => MouseConfig::wheel_up(),
                _ => MouseConfig::left_button().tap(tap),
            };

            engine
                .logic_bricks
                .logic_system_mut()
                .configure_mouse(config);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the current entity count
    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            Ok(engine.store.alive_count() as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the maximum entity capacity
    #[wasm_bindgen]
    pub fn get_max_entities(&self) -> u32 {
        archflow_engine::MAX_ENTITIES as u32
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

    /// Set highlight tint color (for visual feedback on hover/selection)
    #[wasm_bindgen]
    pub fn set_color_tint(
        &self,
        entity_index: u32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            engine.store.set_color_tint(idx, [r, g, b, a]);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Clear highlight tint (reset to default)
    #[wasm_bindgen]
    pub fn clear_color_tint(&self, entity_index: u32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            engine.store.set_color_tint(idx, [0.0, 0.0, 0.0, 0.0]);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set selection state of an entity
    #[wasm_bindgen]
    pub fn set_selected(&self, entity_index: u32, selected: bool) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            engine.store.set_selected(idx, selected);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get selection state of an entity
    #[wasm_bindgen]
    pub fn is_selected(&self, entity_index: u32) -> Result<bool, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            Ok(engine.store.is_selected(idx))
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Move entity by delta (direct position update, not command queue)
    #[wasm_bindgen]
    pub fn move_entity_by(&self, entity_index: u32, dx: f32, dy: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            // Direct position update - moves entity immediately
            engine.store.transforms[idx][0] += dx;
            engine.store.transforms[idx][1] += dy;
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set velocity directly (for physics integration)
    #[wasm_bindgen]
    pub fn set_entity_velocity(&self, entity_index: u32, vx: f32, vy: f32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            engine.store.set_velocity(idx, vx, vy);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get current velocity of an entity
    #[wasm_bindgen]
    pub fn get_entity_velocity(&self, entity_index: u32) -> Result<Vec<f32>, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Entity index out of bounds").into());
            }
            let vel = engine.store.velocity(idx);
            Ok(vec![vel.x, vel.y])
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

    /// Get the color of an entity (returns hex string)
    #[wasm_bindgen]
    pub fn get_color(&self, entity_index: u32) -> Result<String, JsValue> {
        match self.engine.try_borrow() {
            Ok(engine_guard) => {
                if let Some(engine) = engine_guard.as_ref() {
                    let idx = entity_index as usize;
                    if engine.store.is_alive_index(idx) {
                        let abgr = engine.store.colors[idx];
                        let r = abgr & 0xFF;
                        let g = (abgr >> 8) & 0xFF;
                        let b = (abgr >> 16) & 0xFF;
                        Ok(format!("#{0:02x}{1:02x}{2:02x}", r, g, b))
                    } else {
                        Err(JsError::new("Entity not found").into())
                    }
                } else {
                    Err(JsError::new("Engine not initialized").into())
                }
            }
            Err(_) => Err(JsError::new("Engine is busy (borrowed)").into()),
        }
    }

    /// Get the stroke color of an entity (returns hex string)
    #[wasm_bindgen]
    pub fn get_stroke_color(&self, entity_index: u32) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if engine.store.is_alive_index(idx) {
                let abgr = engine.store.stroke_colors[idx];
                let r = abgr & 0xFF;
                let g = (abgr >> 8) & 0xFF;
                let b = (abgr >> 16) & 0xFF;
                Ok(format!("#{0:02x}{1:02x}{2:02x}", r, g, b))
            } else {
                Err(JsError::new("Entity not found").into())
            }
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the stroke width of an entity
    #[wasm_bindgen]
    pub fn get_stroke_width(&self, entity_index: u32) -> Result<f32, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if engine.store.is_alive_index(idx) {
                Ok(engine.store.stroke_widths[idx])
            } else {
                Err(JsError::new("Entity not found").into())
            }
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the active fill color for new shapes
    #[wasm_bindgen]
    pub fn set_active_color(&self, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let rgba = archflow_core::Color::rgba(r, g, b, a).0;
            // Store directly in RGBA format (WebGL expects RGBA, not ABGR)
            engine.active_color = rgba;
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
            // Store directly in RGBA format (WebGL expects RGBA, not ABGR)
            engine.active_stroke_color = rgba;
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
        match self.engine.try_borrow() {
            Ok(engine_guard) => {
                if let Some(engine) = engine_guard.as_ref() {
                    let rgba = engine.active_color;
                    // Extract RGB using Color methods (handles ABGR internal format)
                    let color = archflow_core::Color(rgba);
                    Ok(format!(
                        "#{0:02x}{1:02x}{2:02x}",
                        color.r(),
                        color.g(),
                        color.b()
                    ))
                } else {
                    Err(JsError::new("Engine not initialized").into())
                }
            }
            Err(_) => Err(JsError::new("Engine is busy (borrowed)").into()),
        }
    }

    /// Get the active stroke color (returns RGBA as hex string)
    #[wasm_bindgen]
    pub fn get_active_stroke_color(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let rgba = engine.active_stroke_color;
            // Extract RGB using Color methods (handles ABGR internal format)
            let color = archflow_core::Color(rgba);
            Ok(format!(
                "#{:02x}{:02x}{:02x}",
                color.r(),
                color.g(),
                color.b()
            ))
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

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 5: ENTITY OPERATIONS - CRUD, properties, bounds
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Get the number of alive entities
    #[wasm_bindgen]
    pub fn entity_count(&self) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.store.alive_count() as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // ANEXA-002: SCENE LOADING - JSON Scene serialization/deserialization
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Load a scene from JSON string
    ///
    /// Expects JSON with format (component-based):
    /// ```json
    /// {
    ///   "entities": [
    ///     { "id": "entity1", "components": { "Position": {"x": 100, "y": 200}, "Size": {"width": 50, "height": 50}, "Shape": {"shape": 0}, "Color": {"color": 4294967295} } }
    ///   ]
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn load_scene(&self, json: &str) -> Result<u32, JsValue> {
        use archflow_core::Vec2;
        use archflow_logic::api::json::{ComponentDefinition, Scene};

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let scene: Scene = serde_json::from_str(json)
                .map_err(|e| JsError::new(&alloc::format!("Invalid JSON: {}", e)))?;

            let mut count = 0u32;
            for entity_def in &scene.entities {
                let mut pos = Vec2::new(0.0, 0.0);
                let mut size = Vec2::new(32.0, 32.0);

                for comp in &entity_def.components {
                    match comp.component_type.as_str() {
                        "Position" => {
                            if let Some(x) = comp.data.get("x").and_then(|v| v.as_f64()) {
                                pos.x = x as f32;
                            }
                            if let Some(y) = comp.data.get("y").and_then(|v| v.as_f64()) {
                                pos.y = y as f32;
                            }
                        }
                        "Size" => {
                            if let Some(w) = comp.data.get("width").and_then(|v| v.as_f64()) {
                                size.x = w as f32;
                            }
                            if let Some(h) = comp.data.get("height").and_then(|v| v.as_f64()) {
                                size.y = h as f32;
                            }
                        }
                        _ => {}
                    }
                }

                let entity_id = engine.store.spawn(pos, size);

                for comp in &entity_def.components {
                    match comp.component_type.as_str() {
                        "Shape" => {
                            if let Some(shape) = comp.data.get("shape").and_then(|v| v.as_u64()) {
                                let idx = entity_id.index().0 as usize;
                                engine.store.set_shape_type(idx, shape as u8);
                            }
                        }
                        "Color" => {
                            if let Some(color) = comp.data.get("color").and_then(|v| v.as_u64()) {
                                let idx = entity_id.index().0 as usize;
                                engine.store.set_color(idx, color as u32);
                            }
                        }
                        "Visibility" => {
                            if let Some(visible) =
                                comp.data.get("visible").and_then(|v| v.as_bool())
                            {
                                let idx = entity_id.index().0 as usize;
                                engine.store.set_visible(idx, visible);
                            }
                        }
                        _ => {}
                    }
                }

                count += 1;
            }

            Ok(count)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Serialize current scene to JSON string
    #[wasm_bindgen]
    pub fn serialize_scene(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            use archflow_core::Vec2;
            use archflow_logic::api::json::{ComponentDefinition, Scene, SceneMetadata};

            let mut entities = Vec::new();

            let alive = engine.store.alive_count();
            for i in 0..alive {
                if engine.store.is_alive_index(i) {
                    let pos = engine.store.pos(i);
                    let size = engine.store.size(i);
                    let shape = engine.store.shape_type(i);
                    let colors = engine.store.colors_ref();
                    let color = if i < colors.len() {
                        colors[i]
                    } else {
                        0xFFFFFFFF
                    };
                    let visible = engine.store.is_visible(i);

                    let components = alloc::vec![
                        ComponentDefinition {
                            component_type: alloc::string::String::from("Position"),
                            data: serde_json::json!({ "x": pos.x, "y": pos.y }),
                        },
                        ComponentDefinition {
                            component_type: alloc::string::String::from("Size"),
                            data: serde_json::json!({ "width": size.x, "height": size.y }),
                        },
                        ComponentDefinition {
                            component_type: alloc::string::String::from("Shape"),
                            data: serde_json::json!({ "shape": shape }),
                        },
                        ComponentDefinition {
                            component_type: alloc::string::String::from("Color"),
                            data: serde_json::json!({ "color": color }),
                        },
                        ComponentDefinition {
                            component_type: alloc::string::String::from("Visibility"),
                            data: serde_json::json!({ "visible": visible }),
                        },
                    ];

                    entities.push(archflow_logic::api::json::EntityDefinition {
                        id: alloc::format!("entity_{}", i),
                        name: None,
                        components,
                        behaviors: Vec::new(),
                        children: Vec::new(),
                    });
                }
            }

            let scene = Scene {
                id: alloc::string::String::new(),
                name: Some(alloc::string::String::from("serialized")),
                version: alloc::string::String::from("1.0"),
                metadata: SceneMetadata {
                    author: None,
                    description: None,
                    gravity: [0.0, -9.81, 0.0],
                    ambient_light: [0.5, 0.5, 0.5],
                    fog: None,
                },
                entities,
                behaviors: Vec::new(),
            };

            serde_json::to_string(&scene)
                .map_err(|e| JsError::new(&alloc::format!("Serialization error: {}", e)).into())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // ANEXA-003: ENTITY QUERIES - Query entities by various criteria
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Query entities by shape type
    ///
    /// shape: 0=rectangle, 1=circle, 2=triangle, etc.
    #[wasm_bindgen]
    pub fn query_by_shape(&self, shape: u8) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            for i in 0..alive {
                if engine.store.is_alive_index(i) && engine.store.shape_type(i) == shape {
                    results.push(i as u32);
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query entities by visibility
    #[wasm_bindgen]
    pub fn query_by_visibility(&self, visible: bool) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            for i in 0..alive {
                if engine.store.is_alive_index(i) && engine.store.is_visible(i) == visible {
                    results.push(i as u32);
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query entities by selection state
    #[wasm_bindgen]
    pub fn query_by_selection(&self, selected: bool) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            for i in 0..alive {
                if engine.store.is_alive_index(i) && engine.store.is_selected(i) == selected {
                    results.push(i as u32);
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query entities within bounds (AABB query)
    #[wasm_bindgen]
    pub fn query_in_bounds(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            let min_x = x;
            let max_x = x + width;
            let min_y = y;
            let max_y = y + height;

            for i in 0..alive {
                if engine.store.is_alive_index(i) {
                    let pos = engine.store.pos(i);
                    let size = engine.store.size(i);

                    let entity_min_x = pos.x - size.x / 2.0;
                    let entity_max_x = pos.x + size.x / 2.0;
                    let entity_min_y = pos.y - size.y / 2.0;
                    let entity_max_y = pos.y + size.y / 2.0;

                    if entity_min_x < max_x
                        && entity_max_x > min_x
                        && entity_min_y < max_y
                        && entity_max_y > min_y
                    {
                        results.push(i as u32);
                    }
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query entities with minimum size
    #[wasm_bindgen]
    pub fn query_by_min_size(&self, min_width: f32, min_height: f32) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            for i in 0..alive {
                if engine.store.is_alive_index(i) {
                    let size = engine.store.size(i);
                    if size.x >= min_width && size.y >= min_height {
                        results.push(i as u32);
                    }
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query all alive entities (returns all entity IDs)
    #[wasm_bindgen]
    pub fn query_all(&self) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::with_capacity(alive);

            for i in 0..alive {
                if engine.store.is_alive_index(i) {
                    results.push(i as u32);
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query entities by layer
    #[wasm_bindgen]
    pub fn query_by_layer(&self, layer: u8) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            for i in 0..alive {
                if engine.store.is_alive_index(i) && engine.store.layer(i) == layer {
                    results.push(i as u32);
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Query entities that have velocity (moving entities)
    #[wasm_bindgen]
    pub fn query_with_velocity(&self) -> Result<Vec<u32>, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let alive = engine.store.alive_count();
            let mut results = Vec::new();

            for i in 0..alive {
                if engine.store.is_alive_index(i) {
                    let vel = engine.store.velocity(i);
                    // Consider entity has velocity if either component is non-zero
                    if vel.x != 0.0 || vel.y != 0.0 {
                        results.push(i as u32);
                    }
                }
            }

            Ok(results)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // ANEXA-004: AUDIO SYSTEM - Web Audio API integration
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Initialize audio context (must be called after user interaction)
    #[wasm_bindgen]
    pub fn init_audio(&self) -> Result<bool, JsValue> {
        if let Some(_engine) = self.engine.borrow().as_ref() {
            // Audio is handled via JavaScript Web Audio API
            // This is a placeholder that returns true - actual audio is managed in JS
            Ok(true)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Play a beep sound using Web Audio API oscillator
    ///
    /// # Arguments
    /// * `frequency` - Frequency in Hz (220.0 to 2000.0)
    /// * `duration` - Duration in seconds (0.1 to 2.0)
    /// * `volume` - Volume/gain from 0.0 to 1.0
    #[wasm_bindgen]
    pub fn play_beep(&self, frequency: f32, _duration: f32, volume: f32) -> Result<(), JsValue> {
        if self.engine.borrow().is_none() {
            return Err(JsError::new("Engine not initialized").into());
        }

        // Validate frequency and volume
        let freq_val = if frequency < 220.0 {
            220.0
        } else if frequency > 2000.0 {
            2000.0
        } else {
            frequency
        };
        let vol_val = if volume < 0.0 {
            0.0
        } else if volume > 1.0 {
            1.0
        } else {
            volume
        };

        let freq = (freq_val as i32).to_string();
        let vol = (vol_val * 100.0_f32) as i32;

        // Create JavaScript code to play beep using Web Audio API
        let js_code = alloc::format!(
            r#"(function() {{
                var ctx = window.audioContext;
                if (!ctx) return;
                var osc = ctx.createOscillator();
                var gain = ctx.createGain();
                osc.frequency.value = {};
                gain.gain.value = {} / 100;
                osc.connect(gain);
                gain.connect(ctx.destination);
                osc.start();
                setTimeout(function() {{ osc.stop(); }}, 200);
            }})()"#,
            freq,
            vol
        );

        // Evaluate JavaScript - ignore result as it just plays sound
        let _result = js_sys::eval(&js_code);

        Ok(())
    }

    /// Set master volume
    ///
    /// volume: 0.0-1.0
    #[wasm_bindgen]
    pub fn set_master_volume(&self, volume: f32) -> Result<(), JsValue> {
        if self.engine.borrow().is_some() {
            let clamped = volume.clamp(0.0, 1.0);
            let _ = clamped;
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // ANEXA-005: BATCH OPERATIONS - Efficient multi-entity operations
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Batch set positions for multiple entities
    ///
    /// ids: array of entity indices
    /// xs: array of x positions (same length as ids)
    /// ys: array of y positions (same length as ids)
    #[wasm_bindgen]
    pub fn batch_set_positions(&self, ids: &[u32], xs: &[f32], ys: &[f32]) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            if ids.len() != xs.len() || ids.len() != ys.len() {
                return Err(JsError::new("Array lengths must match").into());
            }

            let count = ids.len();
            for i in 0..count {
                let idx = ids[i] as usize;
                let pos = archflow_core::Vec2::new(xs[i], ys[i]);
                engine.store.set_pos(idx, pos);
            }

            Ok(count as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Batch set sizes for multiple entities
    ///
    /// ids: array of entity indices
    /// widths: array of widths
    /// heights: array of heights
    #[wasm_bindgen]
    pub fn batch_set_sizes(
        &self,
        ids: &[u32],
        widths: &[f32],
        heights: &[f32],
    ) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            if ids.len() != widths.len() || ids.len() != heights.len() {
                return Err(JsError::new("Array lengths must match").into());
            }

            let count = ids.len();
            for i in 0..count {
                let idx = ids[i] as usize;
                let size = archflow_core::Vec2::new(widths[i], heights[i]);
                engine.store.set_size(idx, size);
            }

            Ok(count as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Batch set colors for multiple entities
    ///
    /// ids: array of entity indices
    /// colors: array of RGBA colors (u32)
    #[wasm_bindgen]
    pub fn batch_set_colors(&self, ids: &[u32], colors: &[u32]) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            if ids.len() != colors.len() {
                return Err(JsError::new("Array lengths must match").into());
            }

            let count = ids.len();
            for i in 0..count {
                let idx = ids[i] as usize;
                engine.store.set_color(idx, colors[i]);
            }

            Ok(count as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Batch set visibility for multiple entities
    ///
    /// ids: array of entity indices
    /// visible: visibility state to apply to all
    #[wasm_bindgen]
    pub fn batch_set_visibility(&self, ids: &[u32], visible: bool) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let count = ids.len();
            for i in 0..count {
                let idx = ids[i] as usize;
                engine.store.set_visible(idx, visible);
            }

            Ok(count as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Batch despawn multiple entities
    ///
    /// ids: array of entity indices to remove
    #[wasm_bindgen]
    pub fn batch_despawn(&self, ids: &[u32]) -> Result<u32, JsValue> {
        use archflow_core::{EntityId, Generation, Index};

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let mut count = 0u32;
            for i in 0..ids.len() {
                let idx = ids[i] as usize;
                if engine.store.is_alive_index(idx) {
                    let gen_val = engine.store.generation(idx);
                    let entity_id = EntityId::from_parts(Index(idx as u32), Generation(gen_val));
                    engine.store.despawn(entity_id);
                    count += 1;
                }
            }

            Ok(count)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 6: HISTORY - Undo/Redo operations
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 7: CAMERA - Viewport control
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

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
            engine
                .logic_bricks
                .batch_select_mut()
                .clear(&mut engine.store);
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

    /// Batch set shapes for multiple entities (optimized)
    #[wasm_bindgen]
    pub fn batch_set_shapes(&self, ids: &[u32], shapes: &[u8]) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            use archflow_core::EntityId;
            use archflow_engine::Command;
            let count = ids.len().min(shapes.len());
            for i in 0..count {
                let id = EntityId::new(ids[i]);
                let cmd = Command::SetShape {
                    id,
                    shape: shapes[i],
                };
                engine.command_queue.push(cmd);
            }
            Ok(count as u32)
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

    /// Get entity position in world coordinates
    #[wasm_bindgen]
    pub fn get_entity_position_world(&self, entity_index: u32) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            let world_pos = engine.store.pos(idx);
            let array = js_sys::Array::new();
            array.push(&JsValue::from(world_pos.x));
            array.push(&JsValue::from(world_pos.y));
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

    /// Get entity size in world coordinates
    #[wasm_bindgen]
    pub fn get_entity_size_world(&self, entity_index: u32) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                return Err(JsError::new("Invalid entity index").into());
            }
            let size = engine.store.size(idx);
            let array = js_sys::Array::new();
            array.push(&JsValue::from(size.x));
            array.push(&JsValue::from(size.y));
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
            let raw_color = engine.store.colors[idx];
            // Extract RGB using Color methods (handles ABGR internal format)
            let color = archflow_core::Color(raw_color);
            Ok(format!(
                "#{:02X}{:02X}{:02X}",
                color.r(),
                color.g(),
                color.b()
            ))
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

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 9: TOOLS - Active tool management
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Set the current tool type
    #[wasm_bindgen]
    pub fn set_tool(&self, tool: &str) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm", tool = %tool, "Setting tool");

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.active_tool = alloc::string::String::from(tool);
            // CRITICAL: Also update the logic_bricks system which handles shape creation
            engine.logic_bricks.set_active_tool(tool);
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

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 8: SELECTION - Entity selection management
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Clear all selections (deselect all entities)
    #[wasm_bindgen]
    pub fn clear_selection(&self) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine
                .logic_bricks
                .batch_select_mut()
                .clear(&mut engine.store);

            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Add an entity to the selection (toggle mode)
    #[wasm_bindgen]
    pub fn select_entity(&self, entity_index: u32) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine
                .logic_bricks
                .toggle_selection(&mut engine.store, entity_index);
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get the list of selected entity IDs
    #[wasm_bindgen]
    pub fn get_selection(&self) -> Result<js_sys::Array, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.logic_bricks.get_selected_entities())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Set the selection state of an entity directly
    ///
    /// Uses DeltaMask for memory-efficient undo/redo via command queue.
    #[wasm_bindgen]
    pub fn set_entity_selected(&self, entity_index: u32, selected: bool) -> Result<(), JsValue> {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let idx = entity_index as usize;
            if idx >= MAX_ENTITIES {
                return Err(JsError::new("Invalid entity index").into());
            }

            let currently_selected = engine
                .logic_bricks
                .batch_select()
                .is_selected(archflow_core::EntityId::new(entity_index));

            // Only create command if state needs to change
            if currently_selected != selected {
                engine
                    .logic_bricks
                    .toggle_selection(&mut engine.store, entity_index);
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
            let entities_to_delete = engine.logic_bricks.batch_select().current_selection();
            for id in entities_to_delete {
                let cmd = Command::Despawn(id);
                engine.command_queue.push(cmd);
            }
            engine
                .logic_bricks
                .batch_select_mut()
                .clear(&mut engine.store);
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

    // ═══════════════════════════════════════════════════════════════════════════════
    // INPUT HANDLERS - Logic Bricks Integration
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Report mouse move event to Logic Bricks sensors
    ///
    /// This should be called from JavaScript's mousemove event handler.
    /// The engine will convert screen coordinates to world coordinates
    /// and feed them to the appropriate sensors.
    ///
    /// # Arguments
    /// * `screen_x` - Mouse X position in screen pixels
    /// * `screen_y` - Mouse Y position in screen pixels
    /// * `buttons` - Bitmask of pressed buttons (1=left, 2=right, 4=middle)
    #[wasm_bindgen]
    pub fn on_mouse_move(&self, screen_x: f32, screen_y: f32, buttons: u8, modifiers: u8) {
        #[cfg(feature = "tracing-logging")]
        trace!(target: "archflow::wasm", screen_x, screen_y, buttons, modifiers, "on_mouse_move");

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let world_pos = engine.screen_to_world(screen_x, screen_y);
            engine.logic_bricks.sample_input(
                screen_x,
                screen_y,
                world_pos.x,
                world_pos.y,
                buttons,
                0,
                modifiers,
            );
        }
    }

    /// Report mouse down event to Logic Bricks sensors
    ///
    /// This should be called from JavaScript's mousedown event handler.
    /// Triggers mouse click sensors for left/right/middle buttons.
    ///
    /// # Arguments
    /// * `screen_x` - Mouse X position in screen pixels
    /// * `screen_y` - Mouse Y position in screen pixels
    /// * `button` - Mouse button (0=left, 1=right, 2=middle)
    /// * `modifiers` - Bitmask of modifiers (1=shift, 2=ctrl, 4=alt)
    #[wasm_bindgen]
    pub fn on_mouse_down(&self, screen_x: f32, screen_y: f32, button: u8, modifiers: u8) {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm", screen_x, screen_y, button, modifiers, "🖱️ on_mouse_down");

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let world_pos = engine.screen_to_world(screen_x, screen_y);
            // Convert button to button bitmask
            let buttons = match button {
                0 => 1, // Left
                1 => 2, // Right
                2 => 4, // Middle
                _ => 0,
            };

            #[cfg(feature = "tracing-logging")]
            debug!(target: "archflow::wasm", world_x = world_pos.x, world_y = world_pos.y, buttons, "Converted to world coords and button mask");

            engine.logic_bricks.sample_input(
                screen_x,
                screen_y,
                world_pos.x,
                world_pos.y,
                buttons,
                0,
                modifiers,
            );
        }
    }

    /// Report mouse up event to Logic Bricks sensors
    ///
    /// This should be called from JavaScript's mouseup event handler.
    /// Clears the button state in sensors.
    ///
    /// # Arguments
    /// * `screen_x` - Mouse X position in screen pixels
    /// * `screen_y` - Mouse Y position in screen pixels
    /// * `button` - Mouse button that was released
    #[wasm_bindgen]
    pub fn on_mouse_up(&self, screen_x: f32, screen_y: f32, button: u8, modifiers: u8) {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm", screen_x, screen_y, button, modifiers, "🖱️ on_mouse_up");

        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let world_pos = engine.screen_to_world(screen_x, screen_y);
            // Button released means it's no longer pressed (buttons = 0)
            engine.logic_bricks.sample_input(
                screen_x,
                screen_y,
                world_pos.x,
                world_pos.y,
                0,
                0,
                modifiers,
            );
        }
    }

    /// Report mouse wheel event to Logic Bricks sensors
    ///
    /// This should be called from JavaScript's wheel event handler.
    /// Triggers zoom camera actuators when applicable.
    ///
    /// # Arguments
    /// * `screen_x` - Mouse X position in screen pixels
    /// * `screen_y` - Mouse Y position in screen pixels
    /// * `delta_y` - Scroll delta (positive=up, negative=down)
    /// * `modifiers` - Bitmask of modifiers
    #[wasm_bindgen]
    pub fn on_wheel(&self, screen_x: f32, screen_y: f32, delta_y: f32, modifiers: u8) {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            let world_pos = engine.screen_to_world(screen_x, screen_y);
            // Convert delta_y to wheel value (clamped to i8 range)
            let wheel = (delta_y.clamp(-128.0, 127.0) as i8).min(127).max(-128) as i8;
            engine.logic_bricks.sample_input(
                screen_x,
                screen_y,
                world_pos.x,
                world_pos.y,
                0,
                wheel,
                modifiers,
            );
        }
    }

    /// Report keyboard event to Logic Bricks sensors
    ///
    /// This should be called from JavaScript's keydown/keyup event handlers.
    /// Triggers keyboard shortcut sensors.
    ///
    /// # Arguments
    /// * `key_code` - DOM keyCode value
    /// * `is_down` - true for keydown, false for keyup
    /// * `modifiers` - Bitmask of modifiers (1=shift, 2=ctrl, 4=alt)
    #[wasm_bindgen]
    pub fn on_key(&self, key_code: u32, is_down: bool, modifiers: u8) {
        // Keyboard events are processed through the LogicMappingTable
        // The actual handling depends on the configured keyboard sensors
        #[cfg(feature = "tracing-logging")]
        {
            if is_down {
                trace!(target: "archflow::input", "Key down: code={}, modifiers={}", key_code, modifiers);
            } else {
                trace!(target: "archflow::input", "Key up: code={}, modifiers={}", key_code, modifiers);
            }
        }
        // Future: Store key state and process in tick()
        let _ = is_down;
        let _ = key_code;
        let _ = modifiers;
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════
    // SECTION 10: INPUT - Mouse, keyboard, and input state
    // ═══════════════════════════════════════════════════════════════════════════════════════════════════════

    /// Process all pending input events
    ///
    /// This drains the input ring buffer and feeds events to Logic Bricks sensors.
    /// Called automatically by tick(), but can be called manually if needed.
    #[wasm_bindgen]
    pub fn process_input_events(&self) {
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            // Process events from the input processor's ring buffer
            if let Some(input_processor) = self.input_processor.borrow_mut().as_mut() {
                let events = input_processor.process_events();
                for event in events {
                    let buttons = event.buttons;
                    let wheel = if event.event_type == InputEventType::Wheel as u8 {
                        (event.pressure.clamp(-128.0, 127.0) as i8) as i8
                    } else {
                        0
                    };
                    let world_pos = engine.screen_to_world(event.x, event.y);
                    engine.logic_bricks.sample_input(
                        event.x,
                        event.y,
                        world_pos.x,
                        world_pos.y,
                        buttons,
                        wheel,
                        event.modifiers,
                    );
                }
            }
        }
    }

    /// Get current mouse position in screen coordinates
    ///
    /// Returns tuple of (x, y) or null if engine not initialized.
    #[wasm_bindgen]
    pub fn get_mouse_position(&self) -> Result<String, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            if let Some(input_processor) = self.input_processor.borrow().as_ref() {
                let pos = input_processor.mouse_pos();
                Ok(alloc::format!("{},{}", pos.x, pos.y))
            } else {
                Err(JsError::new("Input processor not initialized").into())
            }
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get current mouse button state
    ///
    /// Returns bitmask of pressed buttons (1=left, 2=right, 4=middle)
    #[wasm_bindgen]
    pub fn get_mouse_buttons(&self) -> Result<u8, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            if let Some(input_processor) = self.input_processor.borrow().as_ref() {
                Ok(input_processor.mouse_buttons().0)
            } else {
                Err(JsError::new("Input processor not initialized").into())
            }
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    /// Get current keyboard modifiers
    ///
    /// Returns bitmask of pressed modifiers (1=shift, 2=ctrl, 4=alt)
    #[wasm_bindgen]
    pub fn get_modifiers(&self) -> Result<u8, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            if let Some(input_processor) = self.input_processor.borrow().as_ref() {
                Ok(input_processor.modifiers().0)
            } else {
                Err(JsError::new("Input processor not initialized").into())
            }
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════
// SECTION 11: JS ERROR - Custom error type for JavaScript
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════
// SECTION 12: UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════

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

#[cfg(target_arch = "wasm32")]
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
}
