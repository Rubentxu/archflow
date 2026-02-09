// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Main Engine Tick Loop
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 21
//
// Integrated tick loop combining all engine systems:
// - Input processing (SharedArrayBuffer)
// - Command execution (Command Sourcing)
// - Spatial indexing
// - Multi-phase GPU rendering
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use alloc::boxed::Box;
use archflow_core::{EntityId, Vec2, Vec2f64};
use archflow_engine::{Command, CommandQueue, ConnectionStore, EntityStore, MAX_ENTITIES};
use archflow_interaction::HistoryManager;
use archflow_logic::{EventRingBuffer, LogicSystem, SelectMode};
use archflow_render::{Camera, GpuRenderer, Renderer};
use wasm_bindgen::JsValue;

/// Converts RGBA color format to ABGR for WebGL compatibility.
///
/// WebGL's UNSIGNED_BYTE normalized attributes read bytes in little-endian order,
/// so we need to swap R and B channels.
///
/// Note: This function is defined in bridge.rs for WASM compatibility.
/// See `bridge::rgba_to_abgr` for the actual implementation and examples.
#[inline]
const fn rgba_to_abgr(rgba: u32) -> u32 {
    let r = (rgba >> 24) & 0xFF;
    let g = (rgba >> 16) & 0xFF;
    let b = (rgba >> 8) & 0xFF;
    let a = rgba & 0xFF;

    // ABGR format: A=highest byte, B, G, R=lowest byte
    (a << 24) | (b << 16) | (g << 8) | r
}

/// Main ArchFlow Engine combining all systems
///
/// This is the central orchestrator that coordinates all engine subsystems
/// in a single tick() function called from requestAnimationFrame.
pub struct ArchFlowEngine {
    /// Entity component system with SoA layout
    pub store: EntityStore,

    /// GPU renderer with multi-phase instancing (polymorphic)
    pub renderer: Box<dyn Renderer>,

    /// Command queue for deferred execution
    pub command_queue: CommandQueue,

    /// 2D infinite camera with zoom-to-cursor
    pub camera: Camera,

    /// Connection store with magnetic anchors
    pub connection_store: ConnectionStore,

    /// Logic Bricks system (sensors, actuators, controllers, event ring-buffer)
    /// Replaces the old logic_system and integrates BatchSelectActuator
    pub logic_bricks: crate::logic_bricks_setup::LogicBricksSystem,

    /// Canvas width in pixels
    pub canvas_width: f32,

    /// Canvas height in pixels
    pub canvas_height: f32,

    /// History manager for undo/redo functionality
    pub history: HistoryManager,

    /// Current active tool (select, rectangle, circle, etc.)
    pub active_tool: alloc::string::String,

    /// Flag indicating if we are currently creating a new entity (drag-to-create)
    /// TODO: Migrate to creation logic in Logic Bricks
    pub is_creating: bool,

    /// Starting world position of the drag operation
    /// TODO: Migrate to MoveActuator in LogicSystem
    pub drag_start: Option<Vec2>,

    /// Flag indicating if we are currently dragging selected entities
    /// TODO: Migrate to MoveActuator in LogicSystem
    pub is_dragging: bool,

    /// Last mouse screen position for calculating deltas during drag
    /// TODO: Migrate to MoveActuator in LogicSystem
    pub last_mouse_screen_pos: Option<Vec2>,

    /// Active fill color for new shapes (RGBA packed)
    pub active_color: u32,

    /// Active stroke color for new shapes (RGBA packed)
    pub active_stroke_color: u32,

    /// Active stroke width for new shapes
    pub active_stroke_width: f32,

    /// Event output buffer for JavaScript (HU-LOGIC-EVENTS-002)
    pub events: EventRingBuffer,
}

impl ArchFlowEngine {
    /// Create a new engine instance
    pub fn new(canvas_width: f32, canvas_height: f32) -> Self {
        let mut camera = Camera::new(canvas_width, canvas_height);

        // Initialize zoom for PPU=1.0 (1:1 pixels)
        // At zoom=1.0: viewport height = canvas_height world units
        // For example: 600px height = 600 world units (spans -300 to +300)
        if canvas_height > 0.0 {
            camera.zoom = 1.0;
        }

        camera.set_viewport_size(canvas_width, canvas_height);

        Self {
            store: EntityStore::new(),
            renderer: Box::new(GpuRenderer::new()),
            command_queue: CommandQueue::new(),
            camera,
            connection_store: ConnectionStore::new(),
            logic_bricks: crate::logic_bricks_setup::LogicBricksSystem::new(),
            canvas_width,
            canvas_height,
            history: HistoryManager::with_default_depth(),
            active_tool: alloc::string::String::from("select"),
            is_creating: false,
            drag_start: None,
            is_dragging: false,
            last_mouse_screen_pos: None,
            active_color: archflow_core::Color::rgba(0x3b, 0x82, 0xf6, 0xff).0,
            active_stroke_color: archflow_core::Color::rgba(0x00, 0x00, 0x00, 0xff).0,
            active_stroke_width: 2.0,
            events: EventRingBuffer::new(1024),
        }
    }

    /// Set the renderer (for backend switching)
    pub fn set_renderer(&mut self, new_renderer: Box<dyn Renderer>) {
        self.renderer = new_renderer;
    }

    /// Resize the canvas
    pub fn resize(&mut self, width: f32, height: f32) {
        self.canvas_width = width;
        self.canvas_height = height;
        self.camera.set_viewport_size(width, height);
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// MAIN TICK LOOP - Called from requestAnimationFrame
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Execute one frame of the engine
    ///
    /// This is the main entry point called from JavaScript via requestAnimationFrame.
    /// It processes all subsystems in the correct order to maintain consistency.
    pub fn tick(&mut self, timestamp: f64) {
        // ═════════════════════════════════════════════════════════════════════
        // PHASE 0: LOGIC BRICKS (Sensors → Controllers → Actuators)
        // ═════════════════════════════════════════════════════════════════════
        // Execute the 4-phase Logic Bricks pipeline:
        // 1. SAMPLE: Sensors read EntityStore (immutable)
        // 2. LOGIC: Controllers filter pulses
        // 3. ACTUATE: Actuators write commands to CommandQueue
        // 4. COMMIT: Commands will be executed in Phase 1 below
        // 4. COMMIT: Commands will be executed in Phase 1 below

        // VERSION STAMP: Confirm WASM is loaded
        // VERSION STAMP: Only log on first tick
        static mut FIRST_TICK: bool = true;
        unsafe {
            #[cfg(target_arch = "wasm32")]
            if FIRST_TICK {
                web_sys::console::log_1(&JsValue::from_str(
                    "🚀 WASM ENGINE LOADED - Version: 2026-02-08-20:40 - LogicBricks enabled",
                ));
                FIRST_TICK = false;
            }
        }

        let timestamp_ms = (timestamp * 100.0) as u32;
        self.logic_bricks.tick(
            &mut self.store,
            timestamp_ms,
            self.active_color,
            self.active_stroke_color,
            self.active_stroke_width,
        );

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 1: COMMAND EXECUTION (includes Logic Bricks commands)
        // ═════════════════════════════════════════════════════════════════════

        // Transfer commands from creation/logic logic to main queue
        let logic_cmds = self.logic_bricks.drain_commands();
        for cmd in logic_cmds {
            self.command_queue.push(cmd);
        }

        self.execute_commands();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 2: CONNECTION UPDATE
        // ═════════════════════════════════════════════════════════════════════
        self.connection_store.update_dirty(&self.store);

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 3: RENDER PREPARATION AND DRAW
        // ═════════════════════════════════════════════════════════════════════
        self.prepare_render();

        // Execute the draw calls
        if let Err(e) = self.renderer.render() {
            // We can't use tracing/log easily here without checking features/imports,
            // but we should at least not panic.
            // In a real engine, we might want to log this once or flag it.
            // For now, GpuRenderer (default) returns error, so we expect this to fail
            // until WebGL2Renderer is injected.
            let _ = e;
        }
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// PHASE IMPLEMENTATIONS
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Flush all pending commands from the queue
    ///
    /// This is called automatically by `tick()`, but can be called manually
    /// if you need immediate command execution.
    pub fn flush_commands(&mut self) {
        self.execute_commands();
    }

    fn execute_commands(&mut self) {
        // Drain all commands from the queue
        let commands = self.command_queue.drain();

        for cmd in commands {
            // Capture current state for undo BEFORE executing
            // For reversible commands, store the undo command
            if let Some(undo_cmd) = cmd.inverse(&self.store) {
                // Record both the redo (original cmd) and undo in history
                // Clone cmd since we need to execute it after recording
                self.history.record(cmd.clone(), undo_cmd);
            }

            // Execute the command
            cmd.execute(&mut self.store);
        }
    }

    fn prepare_render(&mut self) {
        // Sync renderer with entity store using trait
        self.renderer.sync_from_store(&self.store, &self.camera);
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// SELECTION HELPERS (Logic Bricks Integration)
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Get the current selection as a Vec (for backward compatibility)
    ///
    /// Note: This creates a Vec from the BatchSelectActuator's BitVec.
    /// For performance-critical code, use `logic_bricks.batch_select()` directly.
    pub fn get_selected_entities(&self) -> Vec<EntityId> {
        self.logic_bricks.batch_select().current_selection()
    }

    /// Check if an entity is selected
    pub fn is_entity_selected(&self, entity_id: EntityId) -> bool {
        self.logic_bricks.batch_select().is_selected(entity_id)
    }

    /// Get the number of selected entities
    pub fn selection_count(&self) -> usize {
        self.logic_bricks.batch_select().selection_count()
    }

    /// Check if selection is empty
    pub fn is_selection_empty(&self) -> bool {
        self.logic_bricks.batch_select().selection_count() == 0
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.logic_bricks.batch_select_mut().clear(&mut self.store);
    }

    /// Select a single entity (clears previous selection)
    pub fn select_entity(&mut self, entity_id: EntityId) {
        self.logic_bricks.batch_select_mut().clear(&mut self.store);
        self.logic_bricks.batch_select_mut().execute(
            &mut self.store,
            &[entity_id],
            SelectMode::Single,
        );
    }

    /// Toggle entity selection (multi-select)
    pub fn toggle_entity_selection(&mut self, entity_id: EntityId) {
        self.logic_bricks.batch_select_mut().execute(
            &mut self.store,
            &[entity_id],
            SelectMode::Multi,
        );
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// COORDINATE CONVERSION HELPERS
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Vec2 {
        let screen_pos = Vec2::new(screen_x, screen_y);
        let screen_size = Vec2::new(self.canvas_width, self.canvas_height);

        // Use camera's screen_to_world which handles Vec2f64 internally
        let world_pos = self.camera.screen_to_world(screen_pos, screen_size);

        // Convert from Vec2f64 to Vec2 for API compatibility
        Vec2::new(world_pos.x as f32, world_pos.y as f32)
    }

    /// Convert screen delta to world delta
    pub fn screen_delta_to_world(&self, screen_dx: f32, screen_dy: f32) -> Vec2 {
        let zoom = self.camera.zoom;

        // Calculate world dimensions based on canvas aspect ratio
        // World width = 2 * (canvas_aspect) / zoom
        // World height = 2.0 / zoom
        // screen_dx / canvas_width = fraction of screen width
        // fraction * world_width = world delta
        let canvas_aspect = self.canvas_width / self.canvas_height;
        let world_width = 2.0 * canvas_aspect / zoom;

        // screen_dy is inverted because screen +Y is down, world +Y is up
        Vec2::new(
            (screen_dx / self.canvas_width) * world_width,
            -(screen_dy / self.canvas_height) * 2.0 / zoom,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> (f32, f32) {
        let width = self.canvas_width;
        let height = self.canvas_height;
        let screen_size = Vec2::new(width, height);

        // Convert to Vec2f64 and use camera's world_to_screen method
        let world_pos_f64 = Vec2f64::new(world_pos.x as f64, world_pos.y as f64);
        let screen_pos = self.camera.world_to_screen(world_pos_f64, screen_size);
        (screen_pos.x, screen_pos.y)
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// UNDO/REDO HELPERS
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Undo the last command
    pub fn undo(&mut self) -> bool {
        self.history.undo(&mut self.store)
    }

    /// Redo the last undone command
    pub fn redo(&mut self) -> bool {
        self.history.redo(&mut self.store)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Get camera dimensions
    pub fn camera_dimensions(&self) -> (f32, f32) {
        (self.canvas_width, self.canvas_height)
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // DRAG OPERATIONS (using MoveActuator with 6-tick hysteresis)
    // ═══════════════════════════════════════════════════════════════════════════════════

    /// Check if any entity is currently being dragged
    pub fn is_dragging(&self, entity_id: EntityId) -> bool {
        self.logic_bricks.move_actuator().is_dragging(entity_id)
    }

    /// Get number of dragging entities
    pub fn dragging_count(&self) -> usize {
        self.logic_bricks.move_actuator().dragging_count()
    }

    /// Try to start dragging an entity
    /// Returns commands to execute if drag should start (after 6-tick hysteresis)
    pub fn try_start_drag(&mut self, entity_id: EntityId, mouse_pos: Vec2) -> Vec<Command> {
        let idx = entity_id.index().0 as usize;
        let signal = self.logic_bricks.mouse_sensor().signal(idx);
        self.logic_bricks
            .move_actuator_mut()
            .update(entity_id, signal, mouse_pos, &self.store)
    }

    /// Update drag for an entity
    pub fn update_drag(&mut self, entity_id: EntityId, mouse_pos: Vec2) -> Vec<Command> {
        let idx = entity_id.index().0 as usize;
        let signal = self.logic_bricks.mouse_sensor().signal(idx);
        self.logic_bricks
            .move_actuator_mut()
            .update(entity_id, signal, mouse_pos, &self.store)
    }

    /// Clear all drag state
    pub fn clear_drag_state(&mut self) {
        self.logic_bricks.move_actuator_mut().clear();
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // CREATION OPERATIONS (tool-based entity creation)
    // ═══════════════════════════════════════════════════════════════════════════════════

    /// Get current active tool
    pub fn active_tool(&self) -> alloc::string::String {
        self.logic_bricks.get_active_tool()
    }

    /// Set active tool
    pub fn set_active_tool(&mut self, tool: &str) {
        self.logic_bricks.set_active_tool(tool);
    }

    /// Start creating a new shape at the given position
    pub fn start_creation(&mut self, pos: Vec2) -> EntityId {
        // Spawn the entity
        let id = self.store.spawn(pos, Vec2::new(1.0, 1.0));
        self.logic_bricks
            .batch_select_mut()
            .execute(&mut self.store, &[id], SelectMode::Single);

        // Track creation state via public API
        self.logic_bricks.set_creation_start(pos.x, pos.y);

        id
    }

    /// Update shape creation (resize during drag-to-create)
    pub fn update_creation(&mut self, current_pos: Vec2, entity_id: EntityId) {
        let start_pos = self.logic_bricks.get_creation_start_pos_vec2();
        let idx = entity_id.index().0 as usize;

        let min_x = start_pos.x.min(current_pos.x);
        let min_y = start_pos.y.min(current_pos.y);
        let max_x = start_pos.x.max(current_pos.x);
        let max_y = start_pos.y.max(current_pos.y);

        let width = (max_x - min_x).max(10.0);
        let height = (max_y - min_y).max(10.0);
        let center_x = min_x + width / 2.0;
        let center_y = min_y + height / 2.0;

        // Push resize command
        let resize_cmd = Command::Resize {
            id: entity_id,
            size: Vec2::new(width, height),
        };
        self.command_queue.push(resize_cmd);

        // Push move command
        let move_cmd = Command::Teleport {
            id: entity_id,
            pos: Vec2::new(center_x, center_y),
        };
        self.command_queue.push(move_cmd);
    }

    /// End creation - apply default size if no drag occurred
    pub fn end_creation(&mut self, entity_id: EntityId) {
        let idx = entity_id.index().0 as usize;
        let current_size = self.store.size(idx);

        // If size is still minimal (no drag occurred), apply default size
        if current_size.x <= 1.0 && current_size.y <= 1.0 {
            let cmd = Command::Resize {
                id: entity_id,
                size: Vec2::new(150.0, 150.0),
            };
            self.command_queue.push(cmd);
        }

        // Reset creation state via public API
        self.logic_bricks.clear_creation();
    }
}

impl Default for ArchFlowEngine {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = ArchFlowEngine::new(800.0, 600.0);
        assert_eq!(engine.store.alive_count(), 0);
    }

    #[test]
    fn test_engine_default() {
        let engine = ArchFlowEngine::default();
        assert_eq!(engine.store.alive_count(), 0);
    }

    #[test]
    fn test_tick_empty() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);
        engine.tick(0.0);
        // Should not panic
        assert_eq!(engine.store.alive_count(), 0);
    }

    #[test]
    fn test_execute_command() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let cmd = Command::Spawn {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(100.0, 50.0),
            parent: None,
        };
        engine.command_queue.push(cmd);

        engine.execute_commands();
        assert_eq!(engine.store.alive_count(), 1);
    }

    #[test]
    fn test_coordinate_conversion() {
        let engine = ArchFlowEngine::new(800.0, 600.0);

        // Center of screen should be center of world (0, 0)
        let world = engine.screen_to_world(400.0, 300.0);
        assert!((world.x - 0.0).abs() < 0.01);
        assert!((world.y - 0.0).abs() < 0.01);

        // World (0, 0) should map to screen center
        let (screen_x, screen_y) = engine.world_to_screen(Vec2::ZERO);
        assert!((screen_x - 400.0).abs() < 0.5);
        assert!((screen_y - 300.0).abs() < 0.5);
    }

    #[test]
    fn test_coordinate_conversion_with_zoom() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);
        engine.camera.zoom = 2.0;

        // With 2x zoom, screen center should still be world center
        let world = engine.screen_to_world(400.0, 300.0);
        assert!((world.x - 0.0).abs() < 0.01);
        assert!((world.y - 0.0).abs() < 0.01);

        // World coordinates should be more "zoomed in" at 2x zoom
        let world_100px = engine.screen_to_world(500.0, 300.0);
        // At 2x zoom, 100px on screen covers less world distance than 1x
        // With zoom=2.0, viewport height = 300 units (not 600), so 100px = 50 units
        assert!(
            (world_100px.x - 50.0).abs() < 0.1,
            "At 2x zoom, 100px should equal ~50 world units, got {}",
            world_100px.x
        );
    }

    #[test]
    fn test_prepare_render() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);
        let _id = engine
            .store
            .spawn(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0));

        engine.prepare_render();
        // Check that renderer synced
        assert_eq!(engine.renderer.instances().len(), 1);
    }

    #[test]
    fn test_camera_dimensions() {
        let engine = ArchFlowEngine::new(1920.0, 1080.0);
        let (w, h) = engine.camera_dimensions();
        assert_eq!(w, 1920.0);
        assert_eq!(h, 1080.0);
    }

    #[test]
    fn test_screen_delta_to_world() {
        let engine = ArchFlowEngine::new(800.0, 600.0);

        // 100px screen delta at 1x zoom
        let delta = engine.screen_delta_to_world(100.0, 0.0);
        // World aspect ratio = 800/600 = 4/3
        // World width = 2 * (4/3) / 1 = 8/3 ≈ 2.667
        // 100px = 100/800 of screen width = 0.125
        // 0.125 * 2.667 ≈ 0.333 world units
        assert!((delta.x - 0.333).abs() < 0.01);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // UNDO/REDO TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_history_initially_empty() {
        let engine = ArchFlowEngine::new(800.0, 600.0);
        assert!(!engine.can_undo());
        assert!(!engine.can_redo());
        assert_eq!(engine.history.undo_count(), 0);
        assert_eq!(engine.history.redo_count(), 0);
    }

    #[test]
    fn test_move_command_records_history() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        // Spawn an entity first
        let id = engine
            .store
            .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Execute a Move command via queue
        let cmd = Command::Move {
            id,
            delta: Vec2::new(10.0, 20.0),
        };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        // Should be recorded in history
        assert!(engine.can_undo());
        assert!(!engine.can_redo());
        assert_eq!(engine.history.undo_count(), 1);
    }

    #[test]
    fn test_undo_move_restores_position() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        // Spawn an entity
        let id = engine
            .store
            .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let original_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        // Move entity
        let cmd = Command::Move {
            id,
            delta: Vec2::new(10.0, 20.0),
        };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        let moved_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        // Undo should restore original position
        assert!(engine.undo());
        let undone_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        assert!((undone_pos.x - original_pos.x).abs() < 0.01);
        assert!((undone_pos.y - original_pos.y).abs() < 0.01);
    }

    #[test]
    fn test_redo_move_restores_moved_position() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        // Spawn an entity
        let id = engine
            .store
            .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Move entity
        let cmd = Command::Move {
            id,
            delta: Vec2::new(10.0, 20.0),
        };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        let moved_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        // Undo
        engine.undo();

        // Redo should restore moved position
        assert!(engine.redo());
        let redone_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        assert!((redone_pos.x - moved_pos.x).abs() < 0.01);
        assert!((redone_pos.y - moved_pos.y).abs() < 0.01);
    }

    #[test]
    fn test_set_color_records_history() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let id = engine
            .store
            .spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let original_color = engine.store.colors[id.index().0 as usize];

        // Set color
        let cmd = Command::SetColor {
            id,
            color: 0xFF0000FF,
        };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        assert!(engine.can_undo());
        assert_eq!(engine.history.undo_count(), 1);

        // Undo should restore original color
        engine.undo();
        assert_eq!(engine.store.colors[id.index().0 as usize], original_color);
    }

    #[test]
    fn test_new_action_clears_redo_stack() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let id = engine
            .store
            .spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        // First action
        let cmd1 = Command::Move {
            id,
            delta: Vec2::new(10.0, 0.0),
        };
        engine.command_queue.push(cmd1);
        engine.execute_commands();

        // Undo
        engine.undo();
        assert!(engine.can_redo());

        // New action should clear redo stack
        let cmd2 = Command::Move {
            id,
            delta: Vec2::new(0.0, 10.0),
        };
        engine.command_queue.push(cmd2);
        engine.execute_commands();

        assert!(!engine.can_redo());
    }

    #[test]
    fn test_resize_undo_redo() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let id = engine
            .store
            .spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let original_size = Vec2::new(
            engine.store.transforms[id.index().0 as usize][2],
            engine.store.transforms[id.index().0 as usize][3],
        );

        // Resize
        let new_size = Vec2::new(100.0, 80.0);
        let cmd = Command::Resize { id, size: new_size };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        // Undo should restore original size
        engine.undo();
        let undone_size = Vec2::new(
            engine.store.transforms[id.index().0 as usize][2],
            engine.store.transforms[id.index().0 as usize][3],
        );

        assert!((undone_size.x - original_size.x).abs() < 0.01);
        assert!((undone_size.y - original_size.y).abs() < 0.01);
    }

    #[test]
    fn test_teleport_undo_redo() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let id = engine
            .store
            .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let original_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        // Teleport
        let new_pos = Vec2::new(500.0, 600.0);
        let cmd = Command::Teleport { id, pos: new_pos };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        // Undo
        assert!(engine.undo());
        let undone_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        assert!((undone_pos.x - original_pos.x).abs() < 0.01);
        assert!((undone_pos.y - original_pos.y).abs() < 0.01);

        // Redo
        assert!(engine.redo());
        let redone_pos = Vec2::new(
            engine.store.transforms[id.index().0 as usize][0],
            engine.store.transforms[id.index().0 as usize][1],
        );

        assert!((redone_pos.x - new_pos.x).abs() < 0.01);
        assert!((redone_pos.y - new_pos.y).abs() < 0.01);
    }

    #[test]
    fn test_set_visible_undo_redo() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let id = engine
            .store
            .spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let idx = id.index().0 as usize;

        // Initially visible
        assert!(engine.store.is_visible(idx));

        // Hide
        let cmd = Command::SetVisible { id, visible: false };
        engine.command_queue.push(cmd);
        engine.execute_commands();

        assert!(!engine.store.is_visible(idx));

        // Undo should make visible again
        engine.undo();
        assert!(engine.store.is_visible(idx));

        // Redo should hide again
        engine.redo();
        assert!(!engine.store.is_visible(idx));
    }

    #[test]
    fn test_max_depth_enforcement() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);

        let id = engine
            .store
            .spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        // Record more actions than max depth (100)
        for i in 0..150 {
            let cmd = Command::Move {
                id,
                delta: Vec2::new(1.0, 0.0),
            };
            engine.command_queue.push(cmd);
            engine.execute_commands();
        }

        // Should be capped at max depth (100)
        assert_eq!(engine.history.undo_count(), 100);
    }
}
