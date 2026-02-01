// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - WASM Bridge
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
//
// WASM bridge for JavaScript/WebAssembly communication:
// - Exposes engine functions to JavaScript via wasm-bindgen
// - Handles SharedArrayBuffer for lock-free input
// - Provides requestAnimationFrame loop integration
// - Manages canvas and WebGPU context
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefCell;
use wasm_bindgen::prelude::*;

use crate::engine::ArchFlowEngine;
use crate::input::{InputProcessor, InputRingBuffer, MAX_POINTERS};

use archflow_engine::store::MAX_ENTITIES;

/// Global engine instance (using RefCell for interior mutability in WASM)
static mut ENGINE: Option<ArchFlowEngine> = None;
static mut INPUT_PROCESSOR: Option<InputProcessor> = None;

/// WASM Bridge for JavaScript/WebAssembly communication
///
/// This struct provides the interface between JavaScript and the Rust engine.
/// It manages the engine lifecycle and exposes functions that can be called from JS.
#[wasm_bindgen]
pub struct WasmBridge {
    _private: (),
}

#[wasm_bindgen]
impl WasmBridge {
    /// Create a new WASM bridge
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Initialize the engine
    ///
    /// This should be called once when the application starts.
    #[wasm_bindgen]
    pub fn initialize(canvas_width: f32, canvas_height: f32) -> Result<(), JsValue> {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(canvas_width, canvas_height));
            INPUT_PROCESSOR = Some(InputProcessor::new());
        }
        Ok(())
    }

    /// Get a pointer to the SharedArrayBuffer for input events
    ///
    /// This returns a pointer to the InputRingBuffer that JavaScript can
    /// write to directly via SharedArrayBuffer.
    #[wasm_bindgen]
    pub fn get_input_buffer_ptr() -> *mut InputRingBuffer {
        unsafe {
            if let Some(processor) = &mut INPUT_PROCESSOR {
                processor.buffer() as *mut InputRingBuffer
            } else {
                core::ptr::null_mut()
            }
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
    ///
    /// # Arguments
    /// * `event_type` - Event type (0=Down, 1=Move, 2=Up, 3=Wheel, 4=KeyDown, 5=KeyUp)
    /// * `x` - X coordinate in screen pixels
    /// * `y` - Y coordinate in screen pixels
    /// * `buttons` - Mouse button bitmask (Left=1, Right=2, Middle=4)
    /// * `modifiers` - Keyboard modifier bitmask (Shift=1, Ctrl=2, Alt=4)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// // Send pointer down event
    /// wasm.push_input_event(0, 100, 200, 1, 0);
    /// ```
    #[wasm_bindgen]
    pub fn push_input_event(
        event_type: u8,
        x: f32,
        y: f32,
        buttons: u8,
        modifiers: u8,
    ) -> Result<(), JsValue> {
        use crate::input::{Buttons, InputEventType, Modifiers, RawInputEvent};

        unsafe {
            if let Some(processor) = &mut INPUT_PROCESSOR {
                // Convert u8 to InputEventType
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

                let event = RawInputEvent::new(
                    0, // timestamp - can be added later if needed
                    0, // pointer_id
                    x,
                    y,
                    input_event_type,
                    button_flags,
                    modifier_flags,
                );

                if processor.buffer().push_event(event) {
                    Ok(())
                } else {
                    Err(JsError::new("Input buffer full").into())
                }
            } else {
                Err(JsError::new("Input processor not initialized").into())
            }
        }
    }

    /// Run one frame of the engine
    ///
    /// This should be called from requestAnimationFrame.
    #[wasm_bindgen]
    pub fn tick(timestamp: f64) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                // Process input events
                if let Some(processor) = &mut INPUT_PROCESSOR {
                    let events = processor.process_events();

                    for event in events {
                        Self::process_input_event(engine, &event);
                    }
                }

                // Update engine
                engine.tick(timestamp);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Process a single input event and update the engine
    fn process_input_event(engine: &mut ArchFlowEngine, event: &crate::input::RawInputEvent) {
        use archflow_core::Vec2;

        match event.event_type {
            0 => {
                // Pointer Down - simple hit test
                let world_pos = engine.screen_to_world(event.x, event.y);

                // Simple hit test: check if any entity contains the point
                for &entity_idx in &engine.store.draw_order[..engine.store.alive_count()] {
                    let idx = entity_idx as usize;
                    if !engine.store.is_visible(idx) {
                        continue;
                    }

                    let pos = engine.store.pos(idx);
                    let size = engine.store.size(idx);

                    // Check if point is inside entity bounds
                    let half_size = size / 2.0;
                    let min = pos - half_size;
                    let max = pos + half_size;

                    if world_pos.x >= min.x
                        && world_pos.x <= max.x
                        && world_pos.y >= min.y
                        && world_pos.y <= max.y
                    {
                        // Found entity at position - create EntityId from index
                        let entity_id = archflow_core::EntityId::new(entity_idx);
                        engine.selected_entities.clear();
                        engine.selected_entities.push(entity_id);
                        break;
                    }
                }
            }
            1 => {
                // Pointer Move - update drag if active
                if !engine.selected_entities.is_empty() {
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
            2 => {
                // Pointer Up - end drag
            }
            3 => {
                // Wheel - handled in JS side for camera zoom
            }
            _ => {}
        }
    }

    /// Spawn a new entity at the given position
    #[wasm_bindgen]
    pub fn spawn_entity(x: f32, y: f32, width: f32, height: f32) -> Result<u32, JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                let id = engine.store.spawn(
                    archflow_core::Vec2::new(x, y),
                    archflow_core::Vec2::new(width, height),
                );

                // Set a random color
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
    }

    /// Move an entity by the given delta
    #[wasm_bindgen]
    pub fn move_entity(entity_index: u32, dx: f32, dy: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    /// Set the color of an entity
    #[wasm_bindgen]
    pub fn set_color(entity_index: u32, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_core::Color;
                use archflow_core::EntityId;
                use archflow_engine::Command;

                let id = EntityId::new(entity_index);
                let color = Color::rgba(r, g, b, a);
                let cmd = Command::SetColor { id, color: color.0 };
                engine.command_queue.push(cmd);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the number of alive entities
    #[wasm_bindgen]
    pub fn entity_count() -> Result<u32, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                Ok(engine.store.alive_count() as u32)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Undo the last action
    #[wasm_bindgen]
    pub fn undo() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.undo();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Redo the last undone action
    #[wasm_bindgen]
    pub fn redo() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.redo();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the camera zoom level
    #[wasm_bindgen]
    pub fn set_zoom(zoom: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.camera.zoom =
                    zoom.clamp(archflow_render::ZOOM_MIN, archflow_render::ZOOM_MAX);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the current camera zoom level
    #[wasm_bindgen]
    pub fn get_zoom() -> Result<f32, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                Ok(engine.camera.zoom)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the camera center position
    #[wasm_bindgen]
    pub fn set_camera_center(x: f32, y: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.camera.center = archflow_core::Vec2::new(x, y);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the camera center position
    #[wasm_bindgen]
    pub fn get_camera_center() -> Result<js_sys::Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let array = js_sys::Array::new();
                array.push(&JsValue::from(engine.camera.center.x));
                array.push(&JsValue::from(engine.camera.center.y));
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Serialize the current project
    #[wasm_bindgen]
    pub fn serialize_project() -> Result<js_sys::Uint8Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                use archflow_export::ProjectSerializer;

                let data = ProjectSerializer::serialize(&engine.store, &engine.connection_store);
                let array = unsafe { js_sys::Uint8Array::view(&data) };
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Clear all entities
    #[wasm_bindgen]
    pub fn clear() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.store = archflow_engine::EntityStore::new();
                engine.selected_entities.clear();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the shape type of an entity
    #[wasm_bindgen]
    pub fn set_shape(entity_index: u32, shape: u8) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    /// Set the label of an entity
    #[wasm_bindgen]
    pub fn set_label(entity_index: u32, label: &str) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ENTITY ACCESSORS - For JavaScript rendering
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get list of alive entity indices
    #[wasm_bindgen]
    pub fn get_alive_entities() -> Result<Vec<u32>, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let alive = engine.store.draw_order[..engine.store.alive_count()].to_vec();
                Ok(alive)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get entity position in screen coordinates
    #[wasm_bindgen]
    pub fn get_entity_position_screen(entity_index: u32) -> Result<js_sys::Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
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
    }

    /// Get entity size in screen coordinates
    #[wasm_bindgen]
    pub fn get_entity_size_screen(entity_index: u32) -> Result<js_sys::Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                let size = engine.store.size(idx);
                // Size scales inversely with zoom
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
    }

    /// Get entity color as hex string
    #[wasm_bindgen]
    pub fn get_entity_color_hex(entity_index: u32) -> Result<String, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                let color = engine.store.colors[idx];
                let r = (color >> 24) & 0xFF;
                let g = (color >> 16) & 0xFF;
                let b = (color >> 8) & 0xFF;
                let a = color & 0xFF;

                Ok(format!("#{:02X}{:02X}{:02X}", r, g, b))
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get entity shape type
    #[wasm_bindgen]
    pub fn get_entity_shape(entity_index: u32) -> Result<u8, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                Ok(engine.store.shape_type(idx))
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get entity label from string pool
    #[wasm_bindgen]
    pub fn get_entity_label(entity_index: u32) -> Result<String, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                let label = engine.store.string_pool.get(idx);
                Ok(label.to_string())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Check if entity is visible
    #[wasm_bindgen]
    pub fn is_entity_visible(entity_index: u32) -> Result<bool, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                Ok(engine.store.is_visible(idx))
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Check if entity is selected
    #[wasm_bindgen]
    pub fn is_entity_selected(entity_index: u32) -> Result<bool, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                Ok(engine.store.is_selected(idx))
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set entity visibility
    ///
    /// # Arguments
    /// * `entity_index` - The entity index
    /// * `visible` - Whether the entity should be visible
    #[wasm_bindgen]
    pub fn set_entity_visible(entity_index: u32, visible: bool) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TOOL SYSTEM FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Set the current tool type
    ///
    /// # Arguments
    /// * `tool` - Tool identifier: "select", "pan", "draw", "shape", "text", "connection"
    #[wasm_bindgen]
    pub fn set_tool(tool: &str) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                // Store the current tool (for now just a placeholder)
                // In the full implementation, this would update a ToolManager
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the current tool type
    #[wasm_bindgen]
    pub fn get_tool() -> Result<String, JsValue> {
        unsafe {
            if let Some(_engine) = &ENGINE {
                // For now, return "select" as default
                // In the full implementation, this would query the ToolManager
                Ok(alloc::string::String::from("select"))
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SELECTION MANAGEMENT FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Clear all selected entities
    #[wasm_bindgen]
    pub fn clear_selection() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    /// Add an entity to the selection
    ///
    /// If Ctrl modifier is active, adds to selection (multi-select).
    /// Otherwise, replaces the current selection.
    #[wasm_bindgen]
    pub fn select_entity(entity_index: u32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_core::EntityId;
                let id = EntityId::new(entity_index);

                // For now, just add to selection
                // TODO: Handle Ctrl modifier for multi-select
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
    }

    /// Get the list of selected entity IDs
    #[wasm_bindgen]
    pub fn get_selection() -> Result<js_sys::Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let array = js_sys::Array::new();
                for &entity_id in &engine.selected_entities {
                    array.push(&JsValue::from(entity_id.index().0));
                }
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the selection state of an entity directly
    #[wasm_bindgen]
    pub fn set_entity_selected(entity_index: u32, selected: bool) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES {
                    return Err(JsError::new("Invalid entity index").into());
                }

                engine.store.set_selected(idx, selected);

                // Update selected_entities vector
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
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ENTITY MODIFICATION FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Set the size of an entity
    #[wasm_bindgen]
    pub fn set_size(entity_index: u32, width: f32, height: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    /// Set the position of an entity
    #[wasm_bindgen]
    pub fn set_position(entity_index: u32, x: f32, y: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
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
    }

    /// Duplicate an entity (create a copy at a slight offset)
    #[wasm_bindgen]
    pub fn duplicate_entity(entity_index: u32) -> Result<u32, JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_core::Vec2;

                let idx = entity_index as usize;
                if idx >= MAX_ENTITIES || !engine.store.is_alive_index(idx) {
                    return Err(JsError::new("Invalid entity index").into());
                }

                // Get current entity data
                let pos = engine.store.pos(idx);
                let size = engine.store.size(idx);
                let color = engine.store.colors[idx];
                let shape = engine.store.shape_type(idx);

                // Create duplicate at offset
                let new_id = engine.store.spawn(pos + Vec2::new(20.0, 20.0), size);
                let new_idx = new_id.index().0 as usize;

                // Copy properties
                engine.store.colors[new_idx] = color;
                engine.store.set_shape_type(new_idx, shape);

                Ok(new_id.index().0)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Delete all selected entities
    #[wasm_bindgen]
    pub fn delete_selected() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_engine::Command;

                // Collect entity IDs to despawn
                let entities_to_delete: alloc::vec::Vec<_> =
                    engine.selected_entities.iter().copied().collect();

                for id in entities_to_delete {
                    let cmd = Command::Despawn(id);
                    engine.command_queue.push(cmd);
                }

                // Clear selection
                engine.selected_entities.clear();

                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // HISTORY (UNDO/REDO) FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check if undo is available
    #[wasm_bindgen]
    pub fn can_undo() -> Result<bool, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                Ok(engine.can_undo())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Check if redo is available
    #[wasm_bindgen]
    pub fn can_redo() -> Result<bool, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                Ok(engine.can_redo())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get history state for UI feedback
    #[wasm_bindgen]
    pub fn get_history_state() -> Result<String, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let undo_count = engine.history.undo_count();
                let redo_count = engine.history.redo_count();
                Ok(alloc::format!("undo:{},redo:{}", undo_count, redo_count))
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
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

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

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
        // The InputRingBuffer should be properly sized
        let expected_size = core::mem::size_of::<InputRingBuffer>();
        assert!(expected_size > 0);
        assert_eq!(expected_size, 4 + 4 + (32 * 128)); // head + tail + data
    }

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_engine_not_initialized() {
        // When engine is not initialized, operations should fail
        let result = WasmBridge::entity_count();
        assert!(
            result.is_err(),
            "Should return error when engine not initialized"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Tool System Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_set_tool_and_get_tool() {
        // Initialize first
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        // Set tool
        assert!(WasmBridge::set_tool("select").is_ok());

        // Get tool should return the set tool
        let tool = WasmBridge::get_tool().unwrap();
        assert_eq!(tool, "select");

        // Change tool
        assert!(WasmBridge::set_tool("pan").is_ok());
        let tool = WasmBridge::get_tool().unwrap();
        assert_eq!(tool, "pan");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Selection Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_selection_clear_and_add() {
        // Initialize
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        // Clear selection (should not panic)
        assert!(WasmBridge::clear_selection().is_ok());

        // Add entity to selection
        assert!(WasmBridge::select_entity(1).is_ok());

        // Get selection should return array with entity
        let selection = WasmBridge::get_selection().unwrap();
        assert_eq!(selection.length(), 1);
        assert_eq!(selection.get(0).as_f64().unwrap(), 1.0);
    }

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_multi_selection() {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        // Add multiple entities
        assert!(WasmBridge::select_entity(1).is_ok());
        assert!(WasmBridge::select_entity(2).is_ok());
        assert!(WasmBridge::select_entity(3).is_ok());

        let selection = WasmBridge::get_selection().unwrap();
        assert_eq!(selection.length(), 3);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Entity Modification Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_entity_size_setter() {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        let id = WasmBridge::spawn_entity(100.0, 100.0, 50.0, 50.0).unwrap();

        // Set new size
        assert!(WasmBridge::set_size(id, 75.0, 80.0).is_ok());

        // Get size should return new values
        let size = WasmBridge::get_entity_size_screen(id).unwrap();
        assert!((size.get(0).as_f64().unwrap() - 75.0).abs() < 1.0); // Width ~75
        assert!((size.get(1).as_f64().unwrap() - 80.0).abs() < 1.0); // Height ~80
    }

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_entity_position_setter() {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        let id = WasmBridge::spawn_entity(100.0, 100.0, 50.0, 50.0).unwrap();

        // Set new position
        assert!(WasmBridge::set_position(id, 200.0, 150.0).is_ok());

        // Get position should return new values
        let pos = WasmBridge::get_entity_position_screen(id).unwrap();
        assert!((pos.get(0).as_f64().unwrap() - 200.0).abs() < 5.0); // Allow some margin for camera transform
    }

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_duplicate_entity() {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        let id1 = WasmBridge::spawn_entity(100.0, 100.0, 50.0, 50.0).unwrap();

        // Duplicate should create new entity at offset
        assert!(WasmBridge::duplicate_entity(id1).is_ok());

        // Should have 2 entities now
        assert_eq!(WasmBridge::entity_count().unwrap(), 2);
    }

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_delete_selected_entities() {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(800.0, 600.0));
        }

        let id1 = WasmBridge::spawn_entity(100.0, 100.0, 50.0, 50.0).unwrap();
        let id2 = WasmBridge::spawn_entity(200.0, 150.0, 75.0, 80.0).unwrap();

        // Select both
        WasmBridge::select_entity(id1).unwrap();
        WasmBridge::select_entity(id2).unwrap();

        // Delete selected
        assert!(WasmBridge::delete_selected().is_ok());

        // Should have 0 entities now
        assert_eq!(WasmBridge::entity_count().unwrap(), 0);
    }
}
