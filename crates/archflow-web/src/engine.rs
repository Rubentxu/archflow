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

use std::vec;
use std::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, CommandQueue, ConnectionStore, EntityStore, SpatialHash};
use archflow_interaction::{
    CameraController, CrdtManager, GizmoRenderer, HistoryManager, InputProcessor,
};
use archflow_render::{Camera, GpuRenderer};

/// Main ArchFlow Engine combining all systems
///
/// This is the central orchestrator that coordinates all engine subsystems
/// in a single tick() function called from requestAnimationFrame.
pub struct ArchFlowEngine {
    /// Entity component system with SoA layout
    pub store: EntityStore,

    /// Spatial hash for O(1) spatial queries
    pub spatial_hash: SpatialHash,

    /// GPU renderer with multi-phase instancing
    pub renderer: GpuRenderer,

    /// Immediate mode gizmo renderer
    pub gizmo_renderer: GizmoRenderer,

    /// Input processor with event ring buffer
    pub input_processor: InputProcessor,

    /// Command queue for deferred execution
    pub command_queue: CommandQueue,

    /// Undo/Redo history manager
    pub history: HistoryManager,

    /// 2D infinite camera with zoom-to-cursor
    pub camera: Camera,

    /// Camera controller for pan/zoom
    pub camera_controller: CameraController,

    /// Connection store with magnetic anchors
    pub connection_store: ConnectionStore,

    /// CRDT manager for real-time collaboration
    pub crdt: CrdtManager,

    /// Current user ID for CRDT (default: 0 for local)
    pub user_id: u32,
}

impl ArchFlowEngine {
    /// Create a new engine instance
    pub fn new(canvas_width: f32, canvas_height: f32) -> Self {
        Self {
            store: EntityStore::new(),
            spatial_hash: SpatialHash::new(archflow_engine::MAX_ENTITIES as usize),
            renderer: GpuRenderer::new(),
            gizmo_renderer: GizmoRenderer::new(),
            input_processor: InputProcessor::new(),
            command_queue: CommandQueue::new(),
            history: HistoryManager::new(100),
            camera: Camera::new(canvas_width, canvas_height),
            camera_controller: CameraController::new(),
            connection_store: ConnectionStore::new(),
            crdt: CrdtManager::new(0),
            user_id: 0,
        }
    }

    /// Set the current user ID for CRDT operations
    pub fn set_user_id(&mut self, user_id: u32) {
        self.user_id = user_id;
        self.crdt = CrdtManager::new(user_id);
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// MAIN TICK LOOP - Called from requestAnimationFrame
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Execute one frame of the engine
    ///
    /// This is the main entry point called from JavaScript via requestAnimationFrame.
    /// It processes all subsystems in the correct order to maintain consistency.
    pub fn tick(&mut self, _timestamp: f64) {
        // ═════════════════════════════════════════════════════════════════════
        // PHASE 1: INPUT PROCESSING
        // ═════════════════════════════════════════════════════════════════════
        self.process_input();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 2: COMMAND EXECUTION
        // ═════════════════════════════════════════════════════════════════════
        self.execute_commands();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 3: SPATIAL SYNC
        // ═════════════════════════════════════════════════════════════════════
        self.sync_spatial();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 4: CONNECTION UPDATE
        // ═════════════════════════════════════════════════════════════════════
        self.update_connections();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 5: GIZMO GENERATION
        // ═════════════════════════════════════════════════════════════════════
        self.update_gizmos();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 6: RENDER PREPARATION
        // ═════════════════════════════════════════════════════════════════════
        self.prepare_render();
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// PHASE IMPLEMENTATIONS
    /// ═══════════════════════════════════════════════════════════════════════════

    fn process_input(&mut self) {
        // Drain all events from the input ring buffer
        let events = self.input_processor.drain_events();

        for evt in events {
            // Convert screen coordinates to world coordinates
            let screen_pos = Vec2::new(evt.x, evt.y);
            let screen_size = Vec2::new(self.camera.aspect_ratio * 2.0, 2.0);
            let world_pos = self.camera.screen_to_world(screen_pos, screen_size);

            // Process event based on type
            match evt.event_type_value() {
                archflow_interaction::InputEventType::PointerDown => {
                    self.on_pointer_down(world_pos, screen_size, evt);
                }
                archflow_interaction::InputEventType::PointerMove => {
                    self.on_pointer_move(world_pos, screen_size);
                }
                archflow_interaction::InputEventType::PointerUp => {
                    self.on_pointer_up();
                }
                archflow_interaction::InputEventType::Wheel => {
                    self.on_wheel(screen_pos, screen_size, evt);
                }
                _ => {}
            }
        }
    }

    fn execute_commands(&mut self) {
        // Drain all commands from the queue
        let commands = self.command_queue.drain();

        for cmd in commands {
            // Execute the command
            cmd.execute(&mut self.store);

            // Record in history (simplified - proper undo requires capturing state before)
            let _ = &self.history;
        }
    }

    fn sync_spatial(&mut self) {
        // Sync spatial hash with entity store
        // Note: This is a simplified version - full implementation would use dirty tracking
        let _ = &self.spatial_hash;
    }

    fn update_connections(&mut self) {
        // Update dirty connections
        self.connection_store.update_dirty(&self.store);
    }

    fn update_gizmos(&mut self) {
        // Clear previous frame gizmos
        self.gizmo_renderer.clear();

        // Draw selection box if there's a selection
        if let Some(_selection) = self.input_processor.get_selection() {
            // Gizmo rendering would go here
            // For now, we'll skip it since get_bounds doesn't exist on EntityStore
        }
    }

    fn prepare_render(&mut self) {
        // Sync renderer with entity store
        self.renderer.sync_from_store(&self.store, &self.camera);
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// INPUT HANDLERS
    /// ═══════════════════════════════════════════════════════════════════════════

    fn on_pointer_down(
        &mut self,
        world_pos: Vec2,
        screen_size: Vec2,
        evt: archflow_interaction::RawInputEvent,
    ) {
        // Check for hit test
        if let Some(_hit) =
            archflow_interaction::HitTester::find_at(world_pos, &self.spatial_hash, &self.store)
        {
            // Start entity drag
            self.input_processor.set_dragging(true);
        } else {
            // Start camera drag
            self.camera_controller.start_drag(world_pos);
            self.input_processor.set_dragging(true);
        }
    }

    fn on_pointer_move(&mut self, world_pos: Vec2, screen_size: Vec2) {
        if self.camera_controller.is_panning() {
            // Update camera drag
            self.camera_controller
                .on_drag(world_pos, &mut self.camera, screen_size);
        }
    }

    fn on_pointer_up(&mut self) {
        self.input_processor.end_selection();
        self.camera_controller.end_drag();
    }

    fn on_wheel(
        &mut self,
        screen_pos: Vec2,
        screen_size: Vec2,
        evt: archflow_interaction::RawInputEvent,
    ) {
        let delta_y = evt.pressure; // Reuse pressure field for wheel delta

        self.camera_controller
            .on_wheel(delta_y, screen_pos, &mut self.camera, screen_size);
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// UNDO/REDO HELPERS
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Undo the last command
    pub fn undo(&mut self) {
        self.history.undo(&mut self.store);
    }

    /// Redo the last undone command
    pub fn redo(&mut self) {
        self.history.redo(&mut self.store);
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
        assert_eq!(engine.user_id, 0);
    }

    #[test]
    fn test_engine_default() {
        let engine = ArchFlowEngine::default();
        assert_eq!(engine.store.alive_count(), 0);
    }

    #[test]
    fn test_set_user_id() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);
        engine.set_user_id(42);
        assert_eq!(engine.user_id, 42);
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
    fn test_camera_controller() {
        let mut engine = ArchFlowEngine::new(800.0, 600.0);
        let pos = Vec2::new(400.0, 300.0);
        let screen_size = Vec2::new(800.0, 600.0);

        engine
            .camera_controller
            .on_wheel(1.0, pos, &mut engine.camera, screen_size);
        // Should not panic
        assert!(engine.camera.zoom > 0.0);
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
}
