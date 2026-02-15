//! EntityCommandBuffer for JS-WASM Bridge
//!
//! Provides deferred execution of commands from JavaScript to minimize
//! JS↔WASM round-trips. Commands are batched and executed in a single
//! playback call.
//!
//! This is a pure JS-side buffer that collects commands and allows
//! the bridge to execute them in batch.

#![allow(unused)]

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

/// Command type for the ECB
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum EcbCommandType {
    Nop = 0,
    Spawn = 1,
    Despawn = 2,
    Teleport = 3,
    Resize = 4,
    SetColor = 5,
    SetShape = 6,
    SetVisible = 7,
    SetVelocity = 8,
    SetLayer = 9,
    SetSelection = 10,
}

/// A single command in the buffer - 16 bytes fixed size
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct EcbCommand {
    pub cmd_type: u8,
    pub _pad1: u8,
    pub _pad2: u8,
    pub _pad3: u8,
    pub entity: u32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
    pub param4: f32,
}

/// EntityCommandBuffer - Deferred command execution for JS-WASM
///
/// Use this to batch multiple commands and execute them efficiently.
#[wasm_bindgen]
pub struct JsEntityCommandBuffer {
    commands: Vec<EcbCommand>,
    capacity: usize,
    next_entity_id: u32,
    // Results tracking
    spawned_ids: Vec<u32>,
}

#[wasm_bindgen]
impl JsEntityCommandBuffer {
    /// Create a new ECB
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Self {
            commands: Vec::with_capacity(capacity),
            capacity,
            next_entity_id: 0,
            spawned_ids: Vec::new(),
        }
    }

    /// Clear the buffer
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.commands.clear();
        self.next_entity_id = 0;
        self.spawned_ids.clear();
    }

    /// Get command count
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get capacity
    #[wasm_bindgen]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // ================================================================================
    // COMMAND REGISTRATION
    // ================================================================================

    /// Spawn a new entity (returns temp ID for use within ECB)
    #[wasm_bindgen]
    pub fn spawn(&mut self, x: f32, y: f32, width: f32, height: f32) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        self.spawned_ids.push(id);

        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::Spawn as u8,
            entity: id,
            param1: x,
            param2: y,
            param3: width,
            param4: height,
            ..Default::default()
        });

        id
    }

    /// Despawn an entity
    #[wasm_bindgen]
    pub fn despawn(&mut self, entity: u32) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::Despawn as u8,
            entity,
            ..Default::default()
        });
    }

    /// Teleport entity to position
    #[wasm_bindgen]
    pub fn teleport(&mut self, entity: u32, x: f32, y: f32) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::Teleport as u8,
            entity,
            param1: x,
            param2: y,
            ..Default::default()
        });
    }

    /// Resize entity
    #[wasm_bindgen]
    pub fn resize(&mut self, entity: u32, width: f32, height: f32) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::Resize as u8,
            entity,
            param1: width,
            param2: height,
            ..Default::default()
        });
    }

    /// Set entity color
    #[wasm_bindgen]
    pub fn set_color(&mut self, entity: u32, color: u32) {
        let color_bits = color as f32;
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::SetColor as u8,
            entity,
            param1: color_bits,
            ..Default::default()
        });
    }

    /// Set entity shape (0 = rect, 1 = circle)
    #[wasm_bindgen]
    pub fn set_shape(&mut self, entity: u32, shape: u8) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::SetShape as u8,
            entity,
            param1: shape as f32,
            ..Default::default()
        });
    }

    /// Set entity visibility
    #[wasm_bindgen]
    pub fn set_visible(&mut self, entity: u32, visible: bool) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::SetVisible as u8,
            entity,
            param1: if visible { 1.0 } else { 0.0 },
            ..Default::default()
        });
    }

    /// Set entity velocity
    #[wasm_bindgen]
    pub fn set_velocity(&mut self, entity: u32, vx: f32, vy: f32) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::SetVelocity as u8,
            entity,
            param1: vx,
            param2: vy,
            ..Default::default()
        });
    }

    /// Set entity layer
    #[wasm_bindgen]
    pub fn set_layer(&mut self, entity: u32, layer: i32) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::SetLayer as u8,
            entity,
            param1: layer as f32,
            ..Default::default()
        });
    }

    /// Set selection state
    #[wasm_bindgen]
    pub fn set_selection(&mut self, entity: u32, selected: bool) {
        self.push_command(EcbCommand {
            cmd_type: EcbCommandType::SetSelection as u8,
            entity,
            param1: if selected { 1.0 } else { 0.0 },
            ..Default::default()
        });
    }

    // ================================================================================
    // DATA ACCESS FOR BRIDGE
    // ================================================================================

    /// Get commands pointer
    #[wasm_bindgen]
    pub fn commands_ptr(&self) -> *const EcbCommand {
        self.commands.as_ptr()
    }

    /// Get command count
    #[wasm_bindgen]
    pub fn commands_count(&self) -> usize {
        self.commands.len()
    }

    /// Get spawned entity IDs
    #[wasm_bindgen]
    pub fn spawned_ids_ptr(&self) -> *const u32 {
        self.spawned_ids.as_ptr()
    }

    /// Get spawned count
    #[wasm_bindgen]
    pub fn spawned_count(&self) -> usize {
        self.spawned_ids.len()
    }

    // ================================================================================
    // INTERNAL
    // ================================================================================

    #[inline]
    fn push_command(&mut self, cmd: EcbCommand) {
        if self.commands.len() < self.capacity {
            self.commands.push(cmd);
        }
    }
}

// ================================================================================
// ZERO-COPY BUFFER
// ================================================================================

/// Zero-copy buffer for direct memory access
#[wasm_bindgen]
pub struct ZeroCopyCommandBuffer {
    commands: Vec<EcbCommand>,
    write_index: usize,
    capacity: usize,
}

#[wasm_bindgen]
impl ZeroCopyCommandBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Self {
            commands: vec![EcbCommand::default(); capacity],
            write_index: 0,
            capacity,
        }
    }

    #[wasm_bindgen]
    pub fn data_ptr(&mut self) -> *mut EcbCommand {
        self.commands.as_mut_ptr()
    }

    #[wasm_bindgen]
    pub fn set_count(&mut self, count: usize) {
        self.write_index = count.min(self.capacity);
    }

    #[wasm_bindgen]
    pub fn count(&self) -> usize {
        self.write_index
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.write_index = 0;
    }
}
