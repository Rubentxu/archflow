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

use archflow_core::{EntityId, Generation, Index, Vec2};
use archflow_engine::{Command, EntityStore, ShapeType};
use archflow_logic::mapping::{Controller, LogicMappingTable, SensorType};
use archflow_logic::sensors::{MouseConfig, MouseSensor};
use archflow_logic::signals::SignalByte;
use archflow_logic::{BatchSelectActuator, LogicSystem, MoveActuator, SelectMode};

/// Map tool name to ShapeType enum value
fn tool_to_shape_type(tool: &str) -> u8 {
    match tool {
        "rectangle" => ShapeType::Rectangle as u8,
        "circle" => ShapeType::Circle as u8,
        "ellipse" => ShapeType::Ellipse as u8,
        "triangle" => ShapeType::Triangle as u8,
        "diamond" => ShapeType::Diamond as u8,
        "cylinder" => ShapeType::Cylinder as u8,
        "person" => ShapeType::Person as u8,
        _ => ShapeType::Rectangle as u8, // Default to rectangle
    }
}

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
    pub modifiers: u8,
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

    /// Creation state tracking
    creation_state: CreationState,

    /// Pending commands for this frame
    pending_commands: Vec<Command>,

    /// Timestamp for current frame
    timestamp: u32,

    /// Previous button state for edge detection
    prev_buttons: u8,

    /// Start position of marquee selection
    marquee_start: Option<Vec2>,
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
            creation_state: CreationState::default(),
            pending_commands: Vec::new(),
            timestamp: 0,
            prev_buttons: 0,
            marquee_start: None,
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
        modifiers: u8,
    ) {
        self.input_state = InputState {
            screen_x,
            screen_y,
            world_x,
            world_y,
            buttons,
            wheel,
            modifiers,
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
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "🔧 LogicBricks: set_active_tool({})",
            tool
        )));

        self.creation_state.shape_type = String::from(tool);
        // Don't set is_creating here - it should only be set when actually creating
        // is_creating will be set to true when mouse down happens in process_tool_operations
        self.creation_state.is_creating = false;
        self.creation_state.entity_id = None;
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
    /// Drain pending commands
    pub fn drain_commands(&mut self) -> Vec<Command> {
        self.pending_commands.drain(..).collect()
    }

    /// Execute one frame of logic processing
    pub fn tick(
        &mut self,
        store: &mut EntityStore,
        timestamp_ms: u32,
        active_color: u32,
        active_stroke_color: u32,
        active_stroke_width: f32,
    ) {
        self.timestamp = timestamp_ms;
        self.logic_system.set_timestamp(timestamp_ms);
        self.pending_commands.clear();

        // Resize components to match entity store capacity
        // This prevents panics when new entities are created beyond initial capacity
        let capacity = store.transforms.capacity();
        self.mouse_sensor.resize(capacity);
        self.hover_sensor.resize(capacity);
        self.batch_select.resize(capacity);

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
        self.process_tool_operations(
            store,
            world_pos,
            active_color,
            active_stroke_color,
            active_stroke_width,
        );

        // Create a local copy of draw order to avoid borrowing store during the loop
        let draw_order: Vec<u32> = store.draw_order[..store.alive_count()].to_vec();

        let mut any_entity_hit = false;

        // LOGIC BRICKS EVALUATION
        // Evaluate logic bricks for all visible entities (selection, etc.)
        for idx_u32 in draw_order {
            let idx = idx_u32 as usize;
            let generation_val = store.generation(idx);
            let entity_id = EntityId::from_parts(Index(idx_u32), Generation(generation_val));

            // Track if mouse is over this entity (hit testing)
            if self.mouse_sensor.is_positive(idx) {
                any_entity_hit = true;
            }

            let mut signals: Vec<(SensorType, SignalByte)> = Vec::new();

            // Collect sensor signals for this entity
            // Check Mouse Click Sensor
            let click_signal = self.mouse_sensor.signal(idx);
            if click_signal.as_u8() != 0 {
                signals.push((SensorType::MouseClick, click_signal));
            }

            // Check Mouse Hover Sensor
            let hover_signal = self.hover_sensor.signal(idx);
            if hover_signal.as_u8() != 0 {
                signals.push((SensorType::MouseOver, hover_signal));
            }

            if !signals.is_empty() {
                // Evaluate Logic Bricks connections
                // We don't need to track just_selected because MoveActuator handles hysteresis
                self.mapping_table.evaluate(
                    store,
                    entity_id,
                    &signals,
                    &mut self.batch_select,
                    self.input_state.modifiers,
                );
            }
        }

        // Process move actuator - Hysteresis logic inside MoveActuator prevents immediate jump
        self.process_move_actuator(store, world_pos);

        // Handle Marquee / Deselection
        let left_down = self.input_state.buttons & 0b001 != 0;
        let right_down = self.input_state.buttons & 0b010 != 0;

        if left_down && !right_down {
            if let Some(start) = self.marquee_start {
                // Dragging marquee
                self.marquee_select(store, start, world_pos);
            } else {
                // Start marquee only if not dragging an entity
                // Check if any entity is being dragged (simplified check)
                let any_dragging = self
                    .move_actuator
                    .is_dragging(archflow_core::EntityId::new(0));
                if !any_dragging {
                    self.marquee_start = Some(world_pos);
                }
            }
        } else {
            self.marquee_start = None;
        }

        // Poll for events from logic system
        self.logic_system.poll_events();
    }

    /// Region selection using SpatialHash
    fn marquee_select(&mut self, store: &mut EntityStore, start: Vec2, end: Vec2) {
        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);

        // Avoid tiny selection boxes
        if (max_x - min_x).abs() < 1.0 && (max_y - min_y).abs() < 1.0 {
            // It's a click on empty space -> clear selection
            // Only clear if no modifiers (shift/ctrl) are pressed
            if self.input_state.modifiers == 0 {
                self.batch_select.clear(store);
            }
            return;
        }

        // Region selection using SpatialHash (O(log N))
        let query_rect = archflow_core::Rect::new(min_x, min_y, max_x, max_y);
        let in_area = self.logic_system.spatial_hash.query_rect(query_rect);

        // Mode based on modifiers
        if (self.input_state.modifiers & 0x01) != 0 {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "➕ ADD Mode: entities={}, current_count={}",
                in_area.len(),
                self.batch_select.selection_count()
            )));
            self.batch_select
                .execute_with_events(store, &in_area, SelectMode::Add, |_, _| {});
        } else if (self.input_state.modifiers & 0x02) != 0
            || (self.input_state.modifiers & 0x08) != 0
        {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "➖ SUBTRACT Mode: entities={}, current_count={}",
                in_area.len(),
                self.batch_select.selection_count()
            )));
            self.batch_select
                .execute_with_events(store, &in_area, SelectMode::Subtract, |_, _| {});
        } else {
            // Replace mode: clear first, then add all
            self.batch_select.clear(store);
            self.batch_select
                .execute_with_events(store, &in_area, SelectMode::Add, |_, _| {});
        }
    }

    fn process_tool_operations(
        &mut self,
        store: &mut EntityStore,
        world_pos: Vec2,
        active_color: u32,
        active_stroke_color: u32,
        active_stroke_width: f32,
    ) {
        let left_down = self.input_state.buttons & 0b001 != 0;
        let is_in_creation_mode =
            self.creation_state.shape_type != "select" && self.creation_state.shape_type != "";
        let has_entity = self.creation_state.entity_id.is_some();

        // Start creation on left click when in creation mode and not already creating
        if left_down && is_in_creation_mode && !has_entity {
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "🎯 Starting creation: tool={}, pos=({:.1}, {:.1})",
                self.creation_state.shape_type, world_pos.x, world_pos.y
            )));

            let id = store.spawn(world_pos, Vec2::new(1.0, 1.0));

            // Apply shape type based on active tool
            let shape_type = tool_to_shape_type(&self.creation_state.shape_type);
            let idx = id.index().0 as usize;
            store.set_shape_type(idx, shape_type);

            // Apply active colors and stroke
            store.set_color(idx, active_color);
            store.set_stroke_color(idx, active_stroke_color);
            store.set_stroke_width(idx, active_stroke_width);

            // Add default Logic Bricks connections
            // 1. Click -> Select
            self.mapping_table
                .add_select(id, SensorType::MouseClick, Controller::Direct);
            // 2. Click + Hover -> Move
            self.mapping_table.add_move(
                id,
                SensorType::MouseClick,
                Controller::AND(SensorType::MouseOver),
            );

            self.batch_select.execute(store, &[id], SelectMode::Single);
            self.creation_state.entity_id = Some(id);
            self.creation_state.start_pos = world_pos;
            self.creation_state.is_creating = true;

            web_sys::console::log_1(&JsValue::from_str(&format!(
                "🎨 Created {} shape with color=0x{:08x}, stroke=0x{:08x}, width={:.1}, id={:?}",
                self.creation_state.shape_type,
                active_color,
                active_stroke_color,
                active_stroke_width,
                id
            )));
        }

        // Update creation - resize the shape as user drags
        if left_down && has_entity {
            if let Some(entity_id) = self.creation_state.entity_id {
                let start = self.creation_state.start_pos;
                let min_x = start.x.min(world_pos.x);
                let max_x = start.x.max(world_pos.x);
                let min_y = start.y.min(world_pos.y);
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

        // End creation on left button release
        if !left_down && has_entity {
            web_sys::console::log_1(&JsValue::from_str("✅ Finished creation"));
            self.creation_state.entity_id = None;
            self.creation_state.is_creating = false;
        }
    }

    fn process_move_actuator(&mut self, store: &mut EntityStore, world_pos: Vec2) {
        // Collect all entities that need update: currently selected + currently dragging
        // This ensures deselected entities can still "release" their drag state
        let mut to_update = self.batch_select.current_selection();
        for id in self.move_actuator.dragging_entities() {
            if !to_update.contains(&id) {
                to_update.push(id);
            }
        }

        for id in to_update {
            let idx = id.index().0 as usize;
            let signal = self.mouse_sensor.signal(idx);
            let cmds = self.move_actuator.update(id, signal, world_pos, store);
            for cmd in cmds {
                self.pending_commands.push(cmd);
            }
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
        system.sample_input(100.0, 200.0, 50.0, 75.0, 0b001, 0, 0);
        assert_eq!(system.input_state.screen_x, 100.0);
        assert_eq!(system.input_state.buttons, 1);
    }

    #[test]
    fn test_tool_api() {
        let mut system = LogicBricksSystem::new();
        system.set_active_tool("circle");
        assert_eq!(system.get_active_tool(), "circle");
        assert!(!system.is_creating()); // set_active_tool sets is_creating to false

        system.set_creation_start(10.0, 20.0);
        assert!(system.is_creating());

        system.set_active_tool("select");
        assert!(!system.is_creating());
    }
}
