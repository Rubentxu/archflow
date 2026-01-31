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

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, CommandQueue, ConnectionStore, EntityStore};
use archflow_render::{Camera, GpuRenderer};

/// Main ArchFlow Engine combining all systems
///
/// This is the central orchestrator that coordinates all engine subsystems
/// in a single tick() function called from requestAnimationFrame.
pub struct ArchFlowEngine {
    /// Entity component system with SoA layout
    pub store: EntityStore,

    /// GPU renderer with multi-phase instancing
    pub renderer: GpuRenderer,

    /// Command queue for deferred execution
    pub command_queue: CommandQueue,

    /// 2D infinite camera with zoom-to-cursor
    pub camera: Camera,

    /// Connection store with magnetic anchors
    pub connection_store: ConnectionStore,

    /// Currently selected entities (for drag operations)
    pub selected_entities: Vec<EntityId>,

    /// Canvas width in pixels
    pub canvas_width: f32,

    /// Canvas height in pixels
    pub canvas_height: f32,
}

impl ArchFlowEngine {
    /// Create a new engine instance
    pub fn new(canvas_width: f32, canvas_height: f32) -> Self {
        let mut camera = Camera::new(canvas_width, canvas_height);
        camera.set_viewport_size(canvas_width, canvas_height);

        Self {
            store: EntityStore::new(),
            renderer: GpuRenderer::new(),
            command_queue: CommandQueue::new(),
            camera,
            connection_store: ConnectionStore::new(),
            selected_entities: Vec::new(),
            canvas_width,
            canvas_height,
        }
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
    pub fn tick(&mut self, _timestamp: f64) {
        // ═════════════════════════════════════════════════════════════════════
        // PHASE 1: COMMAND EXECUTION
        // ═════════════════════════════════════════════════════════════════════
        self.execute_commands();

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 2: CONNECTION UPDATE
        // ═════════════════════════════════════════════════════════════════════
        self.connection_store.update_dirty(&self.store);

        // ═════════════════════════════════════════════════════════════════════
        // PHASE 3: RENDER PREPARATION
        // ═════════════════════════════════════════════════════════════════════
        self.prepare_render();
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// PHASE IMPLEMENTATIONS
    /// ═══════════════════════════════════════════════════════════════════════════

    fn execute_commands(&mut self) {
        // Drain all commands from the queue
        let commands = self.command_queue.drain();

        for cmd in commands {
            // Execute the command
            cmd.execute(&mut self.store);
        }
    }

    fn prepare_render(&mut self) {
        // Sync renderer with entity store
        self.renderer.sync_from_store(&self.store, &self.camera);
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// COORDINATE CONVERSION HELPERS
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Vec2 {
        // Get canvas dimensions
        let width = self.canvas_width;
        let height = self.canvas_height;

        // Convert screen pixel to normalized device coordinates (-1 to +1)
        let ndc_x = (screen_x / width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / height) * 2.0; // Flip Y

        // Convert NDC to world coordinates using camera
        let aspect_ratio = width / height;
        let world_width = 2.0 * aspect_ratio / self.camera.zoom;
        let world_height = 2.0 / self.camera.zoom;

        let world_x = self.camera.center.x + ndc_x * world_width / 2.0;
        let world_y = self.camera.center.y + ndc_y * world_height / 2.0;

        Vec2::new(world_x, world_y)
    }

    /// Convert screen delta to world delta
    pub fn screen_delta_to_world(&self, screen_dx: f32, screen_dy: f32) -> Vec2 {
        let width = self.canvas_width;
        let height = self.canvas_height;

        let aspect_ratio = width / height;
        let world_width = 2.0 * aspect_ratio / self.camera.zoom;
        let world_height = 2.0 / self.camera.zoom;

        Vec2::new(
            (screen_dx / width) * world_width,
            (screen_dy / height) * world_height,
        )
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> (f32, f32) {
        let width = self.canvas_width;
        let height = self.canvas_height;

        let aspect_ratio = width / height;
        let world_width = 2.0 * aspect_ratio / self.camera.zoom;
        let world_height = 2.0 / self.camera.zoom;

        let rel_x = world_pos.x - self.camera.center.x;
        let rel_y = world_pos.y - self.camera.center.y;

        let ndc_x = rel_x / (world_width / 2.0);
        let ndc_y = rel_y / (world_height / 2.0);

        let screen_x = (ndc_x + 1.0) * width / 2.0;
        let screen_y = (1.0 - ndc_y) * height / 2.0;

        (screen_x, screen_y)
    }

    /// ═══════════════════════════════════════════════════════════════════════════
    /// UNDO/REDO HELPERS
    /// ═══════════════════════════════════════════════════════════════════════════

    /// Undo the last command
    pub fn undo(&mut self) {
        // Simplified undo - in production this would use HistoryManager
        // For now, just a placeholder
    }

    /// Redo the last undone command
    pub fn redo(&mut self) {
        // Simplified redo - in production this would use HistoryManager
        // For now, just a placeholder
    }

    /// Get camera dimensions
    pub fn camera_dimensions(&self) -> (f32, f32) {
        (self.canvas_width, self.canvas_height)
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
        // At 2x zoom, 100px on screen covers less world distance
        // Just verify it's less than at 1x zoom (which would be ~0.333)
        assert!(world_100px.x < 0.3);
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
}
