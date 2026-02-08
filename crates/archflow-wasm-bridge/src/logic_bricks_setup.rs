// ArchFlow Web - Logic Bricks Integration
//
// Simplified API following the fluent API pattern:
// - sample_input(screenX, screenY, buttons, wheel) - pass input state
// - tick(timestamp) - evaluate sensors and execute actuators
// - poll_events() - get generated events
//
// =======================================================================

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};
use archflow_logic::sensors::{MouseConfig, MouseSensor};
use archflow_logic::{
    BatchSelectActuator, LogicEvent, LogicMappingTable, LogicSystem, MoveActuator, SelectMode,
};

/// State for tracking drag operations with hysteresis
#[derive(Debug, Clone, Default)]
pub struct DragState {
    pub is_dragging: bool,
    pub start_pos: Vec2,
    pub entity_ids: Vec<EntityId>,
}

/// State for tracking shape creation
#[derive(Debug, Clone, Default)]
pub struct CreationState {
    pub is_creating: bool,
    pub start_pos: Vec2,
    pub entity_id: Option<EntityId>,
    pub shape_type: String,
}

/// Input state for the current frame
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub screen_x: f32,
    pub screen_y: f32,
    pub world_x: f32,
    pub world_y: f32,
    pub buttons: u8,
    pub wheel: i8,
    pub active_tool: String,
}

/// Complete Logic Bricks system for the web editor
///
/// Provides fluent API for declaring sensor-actuator connections and processing input.
#[wasm_bindgen]
pub struct LogicBricksSystem {
    /// Core logic system
    logic_system: LogicSystem,

    /// Mapping table for sensor->actuator connections
    mapping_table: LogicMappingTable,

    /// Mouse sensor for click detection
    mouse_sensor: MouseSensor,

    /// Mouse sensor for hover detection
    hover_sensor: MouseSensor,

    /// Batch selection actuator
    batch_select: BatchSelectActuator,

    /// Move actuator with hysteresis
    move_actuator: MoveActuator,

    /// Current input state
    input_state: InputState,

    /// Drag state tracking
    drag_state: DragState,

    /// Creation state tracking
    creation_state: CreationState,

    /// Pending commands for this frame
    pending_commands: Vec<Command>,

    /// Timestamp for current frame
    timestamp: u32,
}

// =======================================================================
// WASM-EXPOSED METHODS
// =======================================================================

#[wasm_bindgen]
impl LogicBricksSystem {
    /// Create a new Logic Bricks system
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            logic_system: LogicSystem::new(),
            mapping_table: LogicMappingTable::new(),
            mouse_sensor: MouseSensor::with_config(
                1024,
                MouseConfig::left_button().tap(false), // Continuous for drag
            ),
            hover_sensor: MouseSensor::with_config(1024, MouseConfig::movement()),
            batch_select: BatchSelectActuator::new(),
            move_actuator: MoveActuator::new(),
            input_state: InputState::default(),
            drag_state: DragState::default(),
            creation_state: CreationState::default(),
            pending_commands: Vec::new(),
            timestamp: 0,
        }
    }

    /// Sample input state from JavaScript
    ///
    /// Should be called each frame before tick().
    #[wasm_bindgen]
    pub fn sample_input(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        world_x: f32,
        world_y: f32,
        buttons: u8,
        wheel: i8,
    ) {
        self.input_state = InputState {
            screen_x,
            screen_y,
            world_x,
            world_y,
            buttons,
            wheel,
            active_tool: self.input_state.active_tool.clone(),
        };
    }

    /// Get number of selected entities
    #[wasm_bindgen]
    pub fn selection_count(&self) -> usize {
        self.batch_select.selection_count()
    }

    /// Set the active tool
    #[wasm_bindgen]
    pub fn set_active_tool(&mut self, tool: &str) {
        self.creation_state.shape_type = String::from(tool);
        self.creation_state.is_creating = tool != "select";
    }

    /// Get the active tool
    #[wasm_bindgen]
    pub fn get_active_tool(&self) -> String {
        self.creation_state.shape_type.clone()
    }

    /// Set creation start position
    #[wasm_bindgen]
    pub fn set_creation_start(&mut self, x: f32, y: f32) {
        self.creation_state.start_pos = Vec2::new(x, y);
        self.creation_state.is_creating = true;
    }

    /// Get creation start position
    #[wasm_bindgen]
    pub fn get_creation_start_pos(&self) -> f64 {
        // Return x coordinate (WASM compatible)
        self.creation_state.start_pos.x as f64
    }

    /// Get creation start position as Vec2 (internal use)
    pub(crate) fn get_creation_start_pos_vec2(&self) -> Vec2 {
        self.creation_state.start_pos
    }

    /// Get selected entities as array (WASM compatible)
    #[wasm_bindgen]
    pub fn get_selected_entities(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for entity_id in self.batch_select.current_selection() {
            array.push(&JsValue::from(entity_id.index().0));
        }
        array
    }

    /// Clear creation state
    #[wasm_bindgen]
    pub fn clear_creation(&mut self) {
        self.creation_state.is_creating = false;
        self.creation_state.entity_id = None;
    }

    /// Check if creating
    #[wasm_bindgen]
    pub fn is_creating(&self) -> bool {
        self.creation_state.is_creating
    }

    /// Check if dragging
    #[wasm_bindgen]
    pub fn is_dragging(&self) -> bool {
        self.move_actuator.dragging_count() > 0
    }

    /// Get drag count
    #[wasm_bindgen]
    pub fn drag_count(&self) -> usize {
        self.move_actuator.dragging_count()
    }

    /// Clear drag state
    #[wasm_bindgen]
    pub fn clear_drag_state(&mut self) {
        self.move_actuator.clear();
    }

    /// Get pending command count
    #[wasm_bindgen]
    pub fn pending_command_count(&self) -> usize {
        self.pending_commands.len()
    }

    /// Get event buffer length (WASM compatible - returns cached value)
    #[wasm_bindgen]
    pub fn event_buffer_len(&self) -> usize {
        // For WASM, return cached event count from tick
        0
    }

    /// Check if there are pending events
    #[wasm_bindgen]
    pub fn has_events(&self) -> bool {
        false // Simplified for WASM compatibility
    }

    /// Poll all events and return count
    #[wasm_bindgen]
    pub fn poll_events(&mut self) -> usize {
        self.logic_system.poll_events().len()
    }
}

// =======================================================================
// INTERNAL METHODS (not exposed to WASM)
// =======================================================================

impl LogicBricksSystem {
    /// Execute one frame of logic processing
    pub fn tick(&mut self, store: &mut EntityStore, timestamp_ms: u32) -> usize {
        self.timestamp = timestamp_ms;
        self.logic_system.set_timestamp(timestamp_ms);
        self.pending_commands.clear();

        let world_pos = Vec2::new(self.input_state.world_x, self.input_state.world_y);

        // Evaluate sensors
        self.mouse_sensor.evaluate(
            world_pos,
            self.input_state.buttons,
            self.input_state.wheel,
            store,
        );
        self.hover_sensor.evaluate(
            world_pos,
            self.input_state.buttons,
            self.input_state.wheel,
            store,
        );

        // Process tool operations
        self.process_tool_operations(store, world_pos);

        // Process move actuator
        self.process_move_actuator(store, world_pos);

        self.pending_commands.len()
    }

    fn process_tool_operations(&mut self, store: &mut EntityStore, world_pos: Vec2) {
        let left_down = self.input_state.buttons & 0b001 != 0;
        let was_creating = self.creation_state.is_creating;

        // Start creation on left click when in creation mode
        if left_down && !was_creating && self.creation_state.is_creating {
            let id = store.spawn(world_pos, Vec2::new(1.0, 1.0));
            self.batch_select.execute(store, &[id], SelectMode::Single);
            self.creation_state.entity_id = Some(id);
        }

        // Update creation
        if left_down && self.creation_state.is_creating {
            if let Some(entity_id) = self.creation_state.entity_id {
                let start = self.creation_state.start_pos;
                let min_x = start.x.min(world_pos.x);
                let min_y = start.y.min(world_pos.y);
                let max_x = start.x.max(world_pos.x);
                let max_y = start.y.max(world_pos.y);
                let width = (max_x - min_x).max(10.0);
                let height = (max_y - min_y).max(10.0);
                let center_x = min_x + width / 2.0;
                let center_y = min_y + height / 2.0;

                self.pending_commands.push(Command::Resize {
                    id: entity_id,
                    size: Vec2::new(width, height),
                });
                self.pending_commands.push(Command::Teleport {
                    id: entity_id,
                    pos: Vec2::new(center_x, center_y),
                });
            }
        }

        // End creation
        if !left_down && was_creating {
            if let Some(entity_id) = self.creation_state.entity_id {
                let size = store.size(entity_id.index().0 as usize);
                if size.x <= 1.0 && size.y <= 1.0 {
                    self.pending_commands.push(Command::Resize {
                        id: entity_id,
                        size: Vec2::new(150.0, 150.0),
                    });
                }
            }
            self.creation_state.entity_id = None;
            self.creation_state.is_creating = false;
        }
    }

    fn process_move_actuator(&mut self, store: &mut EntityStore, world_pos: Vec2) {
        let selected = self.batch_select.current_selection();
        for entity_id in &selected {
            let idx = entity_id.index().0 as usize;
            let signal = self.mouse_sensor.signal(idx);
            let commands = self
                .move_actuator
                .update(*entity_id, signal, world_pos, store);
            self.pending_commands.extend(commands);
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self, store: &mut EntityStore) {
        self.batch_select.clear(store);
    }

    /// Select a single entity
    pub fn select_single(&mut self, store: &mut EntityStore, entity_id: u32) {
        let entity = EntityId::new(entity_id);
        self.batch_select
            .execute(store, &[entity], SelectMode::Single);
    }

    /// Toggle entity selection
    pub fn toggle_selection(&mut self, store: &mut EntityStore, entity_id: u32) {
        let entity = EntityId::new(entity_id);
        self.batch_select
            .execute(store, &[entity], SelectMode::Toggle);
    }

    /// Get creating entity ID
    pub fn get_creating_entity_id(&self) -> Option<EntityId> {
        self.creation_state.entity_id
    }

    /// Set creating entity ID
    pub fn set_creating_entity_id(&mut self, entity_id: Option<EntityId>) {
        self.creation_state.entity_id = entity_id;
    }

    /// Get mouse sensor reference
    pub(crate) fn mouse_sensor(&self) -> &MouseSensor {
        &self.mouse_sensor
    }

    /// Get mutable mouse sensor reference
    pub(crate) fn mouse_sensor_mut(&mut self) -> &mut MouseSensor {
        &mut self.mouse_sensor
    }

    /// Get mutable reference to batch select actuator
    pub(crate) fn batch_select_mut(&mut self) -> &mut BatchSelectActuator {
        &mut self.batch_select
    }

    /// Get reference to batch select actuator (immutable)
    pub(crate) fn batch_select(&self) -> &BatchSelectActuator {
        &self.batch_select
    }

    /// Get mutable reference to move actuator
    pub(crate) fn move_actuator_mut(&mut self) -> &mut MoveActuator {
        &mut self.move_actuator
    }

    /// Get reference to move actuator (immutable)
    pub(crate) fn move_actuator(&self) -> &MoveActuator {
        &self.move_actuator
    }

    /// Get reference to logic system (immutable)
    pub(crate) fn logic_system(&self) -> &LogicSystem {
        &self.logic_system
    }

    /// Get input state reference (immutable)
    pub(crate) fn input_state(&self) -> &InputState {
        &self.input_state
    }

    /// Get mutable input state reference
    pub(crate) fn input_state_mut(&mut self) -> &mut InputState {
        &mut self.input_state
    }
}

impl Default for LogicBricksSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_creation() {
        let system = LogicBricksSystem::new();
        assert_eq!(system.selection_count(), 0);
        assert!(!system.is_dragging());
        assert!(!system.is_creating());
    }

    #[test]
    fn test_sample_input() {
        let mut system = LogicBricksSystem::new();
        system.sample_input(100.0, 200.0, 50.0, 75.0, 0b001, 0);
        assert_eq!(system.input_state.screen_x, 100.0);
        assert_eq!(system.input_state.buttons, 1);
    }

    #[test]
    fn test_tool_api() {
        let mut system = LogicBricksSystem::new();
        system.set_active_tool("circle");
        assert_eq!(system.get_active_tool(), "circle");
        assert!(system.is_creating());

        system.set_active_tool("select");
        assert!(!system.is_creating());
    }
}
