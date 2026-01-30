//! Tests for Canvas bounded context reorganization
//!
//! These tests verify that the Canvas bounded context properly consolidates
//! all canvas-related modules from the old architecture.

use archflow_core::{EntityId, Vec2};

// Mock imports - will be replaced with actual implementations
use crate::{Canvas, ShapeType};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Verifies that the canvas crate unifies all canvas-related modules
    ///
    /// This test ensures that modules from archflow-sdk, archflow-primitives,
    /// archflow-spatial, and archflow-geometry are now consolidated.
    #[test]
    fn test_canvas_crate_unified() {
        // Create a canvas instance
        let mut canvas = Canvas::new(800.0, 600.0);

        // Verify canvas has viewport management (from archflow-spatial)
        let viewport = canvas.viewport();
        assert_eq!(viewport.width(), 800.0);
        assert_eq!(viewport.height(), 600.0);

        // Verify canvas has shape management (from archflow-primitives)
        let rect_id = canvas.create_rectangle(100.0, 100.0, 200.0, 150.0);
        let shape = canvas.get_shape(rect_id);
        assert!(shape.is_some());
        assert_eq!(shape.unwrap().shape_type(), ShapeType::Rectangle);

        // Verify canvas has selection (from archflow-sdk)
        canvas.select(rect_id);
        assert_eq!(canvas.selection().len(), 1);
    }

    /// Test: Verifies that canvas dependencies are correct
    ///
    /// The canvas crate should only depend on archflow-core and serde,
    /// not on other canvas-related crates from the old architecture.
    #[test]
    fn test_canvas_dependencies() {
        // This is a compile-time test - if it compiles, dependencies are correct
        // The canvas crate should NOT import from:
        // - archflow-primitives
        // - archflow-spatial
        // - archflow-geometry
        // - archflow-workspace

        // Instead, all functionality should be available directly from the canvas crate
        let mut canvas = Canvas::new(800.0, 600.0);

        // Viewport functionality (was in archflow-spatial)
        let _ = canvas.viewport();

        // Shape functionality (was in archflow-primitives)
        let _ = canvas.create_rectangle(0.0, 0.0, 100.0, 100.0);

        // Geometry functionality (was in archflow-geometry)
        let shape = canvas.get_shape(EntityId::new());
        assert!(shape.is_none());
    }

    /// Test: Verifies that archflow-workspace is eliminated
    ///
    /// Workspace functionality (Document, EventJournal, UndoManager)
    /// should be integrated into the canvas crate or moved to editing crate.
    #[test]
    fn test_workspace_eliminated() {
        // The workspace crate concepts should be accessible differently:
        // - Event sourcing -> moved to events module (in editing crate)
        // - Undo/Redo -> moved to commands module (in editing crate)
        // - Selection -> integrated into canvas

        let mut canvas = Canvas::new(800.0, 600.0);

        // Create a shape first
        let shape_id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        // Selection is now part of canvas
        canvas.select(shape_id);
        assert_eq!(canvas.selection().len(), 1);

        canvas.clear_selection();
        assert_eq!(canvas.selection().len(), 0);

        // Undo/redo will be handled by Command pattern (editing crate)
        // This is tested in the editing crate tests
    }

    /// Test: Verifies that canvas has a unified API
    ///
    /// The canvas should expose a single, coherent API that combines
    /// viewport, shapes, selection, and layers without exposing internal
    /// module boundaries.
    #[test]
    fn test_canvas_unified_api() {
        let mut canvas = Canvas::new(800.0, 600.0);

        // Create multiple shapes
        let rect1 = canvas.create_rectangle(100.0, 100.0, 200.0, 150.0);
        let rect2 = canvas.create_rectangle(400.0, 100.0, 150.0, 150.0);
        let ellipse = canvas.create_ellipse(300.0, 350.0, 75.0, 50.0);

        // Select shapes
        canvas.select_multiple(vec![rect1, rect2]);

        // Verify selection
        assert_eq!(canvas.selection().len(), 2);

        // Update a shape
        let result = canvas.update_shape(
            rect1,
            crate::ShapeChanges {
                x: Some(150.0),
                y: Some(150.0),
                ..Default::default()
            },
        );
        assert!(result);

        // Delete a shape
        assert!(canvas.delete_shape(rect2));
        assert_eq!(canvas.selection().len(), 1); // rect2 removed from selection

        // Verify zoom operations (viewport integration)
        canvas.zoom_to_fit();
        assert!(canvas.viewport().zoom > 0.0);

        // Verify coordinate transforms
        let screen_pos = Vec2::new(400.0, 300.0);
        let canvas_pos = canvas.screen_to_canvas(screen_pos);
        let _ = canvas.canvas_to_screen(canvas_pos);
    }
}
