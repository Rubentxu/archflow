// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - Camera Controller with Zoom-to-Cursor
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 6.2
//
// Camera controller implementing professional zoom-to-cursor behavior:
// - Zoom towards mouse cursor position (like Figma/Google Maps)
// - Pan with mouse drag or space+drag
// - Smooth zoom within 0.01x to 100x range
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(unused_imports)]

extern crate alloc;

use archflow_core::{Vec2, Vec2f64};
use archflow_render::{Camera, ZOOM_INTENSITY, ZOOM_MAX, ZOOM_MIN};

/// Camera controller state for tracking drag operations
#[derive(Clone, Debug, PartialEq, Default)]
pub struct CameraController {
    /// Start position of the current drag operation
    drag_start: Option<Vec2>,

    /// Last recorded mouse position
    last_position: Vec2,

    /// Whether we're currently panning
    is_panning: bool,
}

impl CameraController {
    /// Create a new camera controller
    pub fn new() -> Self {
        Self {
            drag_start: None,
            last_position: Vec2::ZERO,
            is_panning: false,
        }
    }

    /// Handle mouse wheel input with zoom-to-cursor
    ///
    /// This is the key feature that distinguishes professional tools
    /// from amateur ones: zoom should go towards the cursor, not the center.
    ///
    /// # Arguments
    /// * `delta_y` - Wheel delta (positive = scroll down/zoom out, negative = zoom in)
    /// * `mouse_screen` - Mouse position in screen coordinates
    /// * `camera` - Mutable reference to the camera
    /// * `screen_size` - Current viewport size in pixels
    pub fn on_wheel(
        &mut self,
        delta_y: f32,
        mouse_screen: Vec2,
        camera: &mut Camera,
        screen_size: Vec2,
    ) {
        let old_zoom = camera.zoom;

        // IMPORTANT: Calculate mouse world position BEFORE changing zoom
        // This is the key to zoom-to-cursor functionality
        let mouse_world = camera.screen_to_world(mouse_screen, screen_size);

        // Calculate new zoom with limits and intensity
        let zoom_factor = 1.0 + (-delta_y * ZOOM_INTENSITY);
        camera.zoom *= zoom_factor;
        camera.zoom = camera.zoom.clamp(ZOOM_MIN, ZOOM_MAX);

        // The magic formula for zoom-to-cursor:
        // center_new = center_old + (mouse_world - center_old) * (1 - old_zoom/new_zoom)
        //
        // This ensures that the point under the mouse cursor remains stationary
        let zoom_ratio = old_zoom / camera.zoom;

        camera.center = camera.center + (mouse_world - camera.center) * ((1.0 - zoom_ratio) as f64);
    }

    /// Start a pan drag operation
    pub fn start_drag(&mut self, start_pos: Vec2) {
        self.drag_start = Some(start_pos);
        self.last_position = start_pos;
        self.is_panning = true;
    }

    /// End a pan drag operation
    pub fn end_drag(&mut self) {
        self.drag_start = None;
        self.is_panning = false;
    }

    /// Handle pan drag input
    ///
    /// # Arguments
    /// * `mouse_screen` - Current mouse position in screen coordinates
    /// * `camera` - Mutable reference to the camera
    /// * `screen_size` - Current viewport size in pixels
    pub fn on_drag(&mut self, mouse_screen: Vec2, camera: &mut Camera, screen_size: Vec2) {
        // Calculate delta from last position
        let delta = mouse_screen - self.last_position;
        self.last_position = mouse_screen;

        // Convert screen delta to world delta
        let half_height = 1.0 / camera.zoom;
        let half_width = half_height * camera.aspect_ratio;

        let world_delta = Vec2::new(
            delta.x * (2.0 * half_width) / screen_size.x,
            delta.y * (2.0 * half_height) / screen_size.y,
        );

        // Move camera in opposite direction (dragging right moves camera left)
        // Convert to Vec2f64 for camera.center
        camera.center -= Vec2f64::new(world_delta.x as f64, world_delta.y as f64);
    }

    /// Handle pan with explicit delta (for touch/trackpad)
    ///
    /// # Arguments
    /// * `delta` - Movement delta in screen coordinates
    /// * `camera` - Mutable reference to the camera
    /// * `screen_size` - Current viewport size in pixels
    pub fn on_pan_delta(&mut self, delta: Vec2, camera: &mut Camera, screen_size: Vec2) {
        // Convert screen delta to world delta
        let half_height = 1.0 / camera.zoom;
        let half_width = half_height * camera.aspect_ratio;

        let world_delta = Vec2::new(
            delta.x * (2.0 * half_width) / screen_size.x,
            delta.y * (2.0 * half_height) / screen_size.y,
        );

        camera.center -= Vec2f64::new(world_delta.x as f64, world_delta.y as f64);
    }

    /// Check if currently panning
    pub fn is_panning(&self) -> bool {
        self.is_panning
    }

    /// Get the current drag start position (if any)
    pub fn drag_start(&self) -> Option<Vec2> {
        self.drag_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = CameraController::new();
        assert!(!controller.is_panning());
        assert_eq!(controller.drag_start(), None);
    }

    #[test]
    fn test_drag_state() {
        let mut controller = CameraController::new();
        assert!(!controller.is_panning());

        controller.start_drag(Vec2::new(100.0, 100.0));
        assert!(controller.is_panning());
        assert_eq!(controller.drag_start(), Some(Vec2::new(100.0, 100.0)));

        controller.end_drag();
        assert!(!controller.is_panning());
        assert_eq!(controller.drag_start(), None);
    }

    #[test]
    fn test_zoom_changes() {
        let mut controller = CameraController::new();
        let mut camera = Camera::new(100.0, 100.0);
        let initial_zoom = camera.zoom;

        // Zoom in (negative delta)
        controller.on_wheel(
            -100.0,
            Vec2::new(50.0, 50.0),
            &mut camera,
            Vec2::new(100.0, 100.0),
        );
        assert!(camera.zoom > initial_zoom);

        // Zoom out (positive delta)
        let zoom_before = camera.zoom;
        controller.on_wheel(
            100.0,
            Vec2::new(50.0, 50.0),
            &mut camera,
            Vec2::new(100.0, 100.0),
        );
        assert!(camera.zoom < zoom_before);
    }

    #[test]
    fn test_zoom_limits() {
        let mut controller = CameraController::new();
        let mut camera = Camera::new(100.0, 100.0);

        // Try to zoom beyond limits
        camera.zoom = ZOOM_MAX;
        controller.on_wheel(
            -1000.0,
            Vec2::new(50.0, 50.0),
            &mut camera,
            Vec2::new(100.0, 100.0),
        );
        assert_eq!(camera.zoom, ZOOM_MAX);

        camera.zoom = ZOOM_MIN;
        controller.on_wheel(
            1000.0,
            Vec2::new(50.0, 50.0),
            &mut camera,
            Vec2::new(100.0, 100.0),
        );
        assert_eq!(camera.zoom, ZOOM_MIN);
    }

    #[test]
    fn test_pan_moves_camera() {
        let mut controller = CameraController::new();
        let mut camera = Camera::new(100.0, 100.0);
        let initial_center = camera.center;

        // Pan right
        controller.on_drag(Vec2::new(60.0, 50.0), &mut camera, Vec2::new(100.0, 100.0));
        assert!(camera.center.x < initial_center.x); // Camera moves left (opposite to drag)
    }

    #[test]
    fn test_zoom_to_cursor() {
        let mut controller = CameraController::new();
        let mut camera = Camera::new(100.0, 100.0);

        // Position mouse at specific screen location
        let mouse_screen = Vec2::new(75.0, 50.0); // Right side of screen
        let mouse_world_before = camera.screen_to_world(mouse_screen, Vec2::new(100.0, 100.0));

        // Zoom in
        controller.on_wheel(-100.0, mouse_screen, &mut camera, Vec2::new(100.0, 100.0));

        // Mouse position in world should remain approximately the same
        let mouse_world_after = camera.screen_to_world(mouse_screen, Vec2::new(100.0, 100.0));
        let diff = (mouse_world_after - mouse_world_before).length();

        // The point under the cursor should remain stable (within reasonable precision)
        assert!(diff < 0.1, "Zoom-to-cursor failed: point moved by {}", diff);
    }
}
