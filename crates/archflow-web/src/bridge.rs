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

// WASM Bridge for JavaScript/WebAssembly communication
//
// This struct provides the interface between JavaScript and the Rust engine.
// It manages the engine lifecycle and exposes functions that can be called from JS.
#[wasm_bindgen]
pub struct WasmBridge {
    // Store engine and input processor in the struct for safe access
    engine: RefCell<Option<ArchFlowEngine>>,
    input_processor: RefCell<Option<InputProcessor>>,
}

#[wasm_bindgen]
impl WasmBridge {
    /// Create a new WASM bridge
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            engine: RefCell::new(None),
            input_processor: RefCell::new(None),
        }
    }

    /// Initialize the engine
    ///
    /// This should be called once when the application starts.
    #[wasm_bindgen]
    pub fn initialize(&self, canvas_width: f32, canvas_height: f32) -> Result<(), JsValue> {
        self.engine
            .borrow_mut()
            .replace(ArchFlowEngine::new(canvas_width, canvas_height));
        self.input_processor
            .borrow_mut()
            .replace(InputProcessor::new());
        Ok(())
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

        let event = RawInputEvent::new(0, 0, x, y, input_event_type, button_flags, modifier_flags);

        if let Some(processor) = self.input_processor.borrow_mut().as_mut() {
            if processor.buffer().push_event(event) {
                Ok(())
            } else {
                Err(JsError::new("Input buffer full").into())
            }
        } else {
            Err(JsError::new("Input processor not initialized").into())
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
                let world_pos = engine.screen_to_world(event.x, event.y);
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
                        break;
                    }
                }
            }
            InputEventType::Move => {
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
            let color = Color::rgba(r, g, b, a);
            let cmd = Command::SetColor { id, color: color.0 };
            engine.command_queue.push(cmd);
            Ok(())
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
            engine.camera.center = archflow_core::Vec2::new(x, y);
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
    pub fn set_tool(&self, _tool: &str) -> Result<(), JsValue> {
        Ok(())
    }

    /// Get the current tool type
    #[wasm_bindgen]
    pub fn get_tool(&self) -> Result<String, JsValue> {
        Ok(alloc::string::String::from("select"))
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
