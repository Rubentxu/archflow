// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - 2D Infinite Camera with Zoom-to-Cursor
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 6
//
// Camera implementation for infinite canvas:
// - Zoom range: 0.01x to 100x
// - Unlimited panning (infinite world)
// - Zoom-to-cursor (like Figma/Google Maps)
// - Automatic viewport culling with SpatialHash
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]
#![warn(clippy::all)]

use archflow_core::{Rect, Vec2, Vec2f64};

/// Minimum zoom level (0.01% - very zoomed out, needed for 1:1 pixel mapping on large screens)
pub const ZOOM_MIN: f32 = 0.0001;

/// Maximum zoom level (10000% - very zoomed in)
pub const ZOOM_MAX: f32 = 100.0;

/// Zoom intensity for mouse wheel
pub const ZOOM_INTENSITY: f32 = 0.001;

/// Pixels Per Unit - Defines the relationship between screen pixels and world units.
///
/// With PPU = 1.0 (1:1 pixels):
/// - A 100px button = 100 world units
/// - An 800x600 canvas = 800 x 600 world units
/// - Zoom 1.0 = pixel-perfect rendering
///
/// Camera-Relative Rendering handles precision at large coordinates.
pub const PIXELS_PER_UNIT: f32 = 1.0;

/// 2D Orthographic camera for infinite canvas diagrams
///
/// Features:
/// - Infinite zoom range (0.01x to 100x)
/// - Unlimited panning (infinite world)
/// - Aspect ratio awareness for proper rendering
/// - Near/far planes for future depth effects
/// - Double-precision position (Vec2f64) for jitter-free zoom
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    /// Center of the camera in world coordinates (f64 for precision at extreme zoom)
    pub center: Vec2f64,

    /// Zoom level (1.0 = 100%, 0.5 = 200%, 2.0 = 50%)
    pub zoom: f32,

    /// Aspect ratio of the viewport (width / height)
    pub aspect_ratio: f32,

    /// Near plane for orthographic projection
    pub near: f32,

    /// Far plane for orthographic projection
    pub far: f32,
}

impl Camera {
    /// Create a new camera with the given viewport dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            center: Vec2f64::ZERO,
            zoom: 1.0,
            aspect_ratio: if height > 0.0 { width / height } else { 1.0 },
            near: -1.0,
            far: 1.0,
        }
    }

    /// Convert world position to camera-relative position (f32)
    ///
    /// This is the key method for jitter-free rendering at extreme zoom.
    /// By converting to f32 only after subtracting the camera position,
    /// we preserve sub-pixel precision even at coordinates like 10,000,000.
    ///
    /// # Arguments
    /// * `world_pos` - Position in world coordinates
    ///
    /// # Returns
    /// Position relative to camera, suitable for GPU upload
    #[inline(always)]
    pub fn world_to_camera(&self, world_pos: Vec2f64) -> Vec2 {
        // Subtract camera position (f64 precision)
        // Result is small (~hundreds or thousands) even for far-away points
        let relative = world_pos.sub(self.center);
        // Convert to f32 - now safe because values are small
        Vec2::new(relative.x as f32, relative.y as f32)
    }

    /// Convert world position to camera-relative with f32 input
    ///
    /// Convenience method for entities stored as f32.
    #[inline(always)]
    pub fn world_to_camera_f32(&self, world_pos: Vec2) -> Vec2 {
        // Convert entity pos to f64, subtract camera, convert back to f32
        let relative = Vec2f64::new(world_pos.x as f64, world_pos.y as f64).sub(self.center);
        // Convert f64 back to archflow_core::Vec2
        Vec2::new(relative.x as f32, relative.y as f32)
    }

    /// Set camera center (supports f64 for precision)
    #[inline(always)]
    pub fn set_center_f64(&mut self, center: Vec2f64) {
        self.center = center;
    }

    /// Get camera center as f64
    #[inline(always)]
    pub fn center_f64(&self) -> Vec2f64 {
        self.center
    }

    /// Update the aspect ratio when viewport is resized
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        if height > 0.0 {
            self.aspect_ratio = width / height;
        }
    }

    /// Build the view-projection matrix for the shader
    ///
    /// This matrix is uploaded to the WebGPU Uniform Buffer each frame
    /// and transforms world coordinates to clip space coordinates.
    ///
    /// NOTE: With Camera-Relative Rendering, this matrix effectively only handles
    /// Projection (Zoom + Aspect Ratio). The Translation (View) part is handled
    /// in the vertex shader by subtracting `camera.camera_pos` from vertex positions.
    ///
    /// With PPU=1.0 (1:1 pixels):
    /// - At zoom=1.0: viewport = canvas_height units
    /// - At zoom=2.0: viewport = canvas_height / 2 units
    pub fn build_view_projection_matrix(&self, canvas_height: f32) -> [[f32; 4]; 4] {
        // Calculate viewport height in world units
        // At zoom=1.0: viewport = canvas_height units
        let viewport_height = canvas_height / self.zoom;
        let half_height = viewport_height / 2.0;
        let half_width = half_height * self.aspect_ratio;

        // In relative rendering, the camera is always at (0,0) relative to itself
        // The vertex shader subtracts the actual world camera position
        let center_x = 0.0;
        let center_y = 0.0;

        let left = center_x - half_width;
        let right = center_x + half_width;
        let bottom = center_y - half_height;
        let top = center_y + half_height;

        // Orthographic projection matrix (column-major for WebGPU)
        // Maps: (left, bottom, near) -> (-1, -1, 0) and (right, top, far) -> (1, 1, 1)
        [
            [2.0 / (right - left), 0.0, 0.0, 0.0],
            [0.0, 2.0 / (top - bottom), 0.0, 0.0],
            [0.0, 0.0, 2.0 / (self.far - self.near), 0.0],
            [
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(self.far + self.near) / (self.far - self.near),
                1.0,
            ],
        ]
    }

    /// Convert screen coordinates to world coordinates
    /// Convert screen coordinates to world coordinates (returns f64)
    ///
    /// # Use cases
    /// - Positioning new entities
    /// - Snap to grid
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2f64 {
        // Normalize to device normalized coordinates (NDC) [-1, 1]
        // Screen Y=0 is top (DOM), World Y-up means:
        // - Screen top (Y=0) → NDC Y=1 → World +Y
        // - Screen bottom (Y=max) → NDC Y=-1 → World -Y
        let mut ndc = (screen_pos / screen_size) * 2.0 - Vec2::ONE;

        // Invert Y-axis: Screen increases downward, World increases upward
        ndc.y = -ndc.y;

        // Calculate viewport height in world units
        // At zoom=1.0: viewport = screen_height units
        let viewport_height = screen_size.y as f64 / self.zoom as f64;
        let half_height = viewport_height / 2.0;
        let half_width = half_height * self.aspect_ratio as f64;

        Vec2f64::new(
            self.center.x + ndc.x as f64 * half_width,
            self.center.y + ndc.y as f64 * half_height,
        )
    }

    /// Convert world coordinates to screen coordinates (returns f32)
    pub fn world_to_screen(&self, world_pos: Vec2f64, screen_size: Vec2) -> Vec2 {
        // Use same formula as screen_to_world for consistency
        let viewport_height = screen_size.y as f64 / self.zoom as f64;
        let half_height = viewport_height / 2.0;
        let half_width = half_height * self.aspect_ratio as f64;

        // World coordinates to NDC [-1, 1]
        // World +Y (top) → NDC Y=1 → Screen Y=0 (top)
        // World -Y (bottom) → NDC Y=-1 → Screen Y=height (bottom)
        let mut ndc = Vec2::new(
            ((world_pos.x - self.center.x) / half_width) as f32,
            ((world_pos.y - self.center.y) / half_height) as f32,
        );

        // Invert Y-axis: World increases upward, Screen increases downward
        ndc.y = -ndc.y;

        // NDC [-1, 1] to Screen [0, width/height]
        (ndc + Vec2::ONE) * 0.5 * screen_size
    }

    /// Get the visible rectangle of the camera in world coordinates
    ///
    /// Useful for:
    /// - Viewport culling (only render what's visible)
    /// - Determining which icons to lazy load
    pub fn viewport_bounds(&self, canvas_height: f32) -> Rect {
        // Calculate viewport height in world units (PPU=1.0 means no conversion needed)
        let viewport_height = canvas_height / self.zoom;
        let half_height = viewport_height / 2.0;
        let half_width = half_height * self.aspect_ratio;

        // Use center_f64() for compatibility
        let center = self.center_f64();
        Rect::from_center_size(
            Vec2::new(center.x as f32, center.y as f32),
            Vec2::new(half_width * 2.0, half_height * 2.0),
        )
    }

    /// Check if a world-space point is visible in the viewport
    pub fn is_visible(&self, world_pos: Vec2f64, canvas_height: f32) -> bool {
        let bounds = self.viewport_bounds(canvas_height);
        let pos_f32 = Vec2::new(world_pos.x as f32, world_pos.y as f32);
        bounds.contains(pos_f32)
    }

    /// Check if a world-space rectangle intersects the viewport
    pub fn is_rect_visible(&self, rect: Rect, canvas_height: f32) -> bool {
        self.viewport_bounds(canvas_height).intersects(&rect)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(1920.0, 1080.0);
        assert_eq!(camera.center, Vec2f64::ZERO);
        assert_eq!(camera.zoom, 1.0);
        assert!((camera.aspect_ratio - 16.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn test_screen_to_world() {
        let camera = Camera::new(100.0, 100.0);
        let screen_size = Vec2::new(100.0, 100.0);

        // Center of screen should be center of world (at zoom 1.0)
        let center_world = camera.screen_to_world(Vec2::new(50.0, 50.0), screen_size);
        assert_eq!(center_world, Vec2f64::ZERO);

        // Top-right corner of screen (0 is top, Y increases downward in screen space)
        // Should map to (+X, +Y) in world space (Y increases upward)
        let corner_world = camera.screen_to_world(Vec2::new(100.0, 0.0), screen_size);
        assert!(corner_world.x > 0.0 && corner_world.y > 0.0);

        // Bottom-right corner of screen (100, 100)
        // Should map to (+X, -Y) in world space
        let bottom_right = camera.screen_to_world(Vec2::new(100.0, 100.0), screen_size);
        assert!(bottom_right.x > 0.0 && bottom_right.y < 0.0);
    }

    /// E2E test: Verify mouse → screen_to_world → hit test flow
    ///
    /// This test validates the complete coordinate transformation pipeline:
    /// 1. Mouse click at screen position
    /// 2. Transform to world coordinates via screen_to_world
    /// 3. Perform AABB hit test
    /// 4. Verify correct entity is selected
    ///
    /// Coordinate system:
    /// - Screen: Y=0 at top, increases downward
    /// - World: Y increases upward (standard math convention)
    #[test]
    fn test_e2e_mouse_to_world_hit_test() {
        // Setup: 800x600 canvas, camera at origin, zoom=1.0
        let camera = Camera::new(800.0, 600.0);
        let screen_size = Vec2::new(800.0, 600.0);

        // Entity at world position (100, 100) with size 50x50
        let entity_pos = Vec2::new(100.0, 100.0);
        let entity_size = Vec2::new(50.0, 50.0);
        let entity_bounds = Rect::from_center_size(entity_pos, entity_size);

        // Test Case 1: Click at center of entity
        // For 800x600 camera, zoom=1.0:
        // - viewport: X[-400,400], Y[-300,300]
        // - world (100, 100) → ndc (0.25, 0.333) → screen (500, 200)
        // (Screen Y inverted: world +100 is above center, so screen Y < 300)
        let mouse_screen = Vec2::new(500.0, 200.0);
        let mouse_world = camera.screen_to_world(mouse_screen, screen_size);

        // Verify coordinate transformation
        assert!(
            (mouse_world.x - 100.0).abs() < 0.01,
            "X: {} != 100",
            mouse_world.x
        );
        assert!(
            (mouse_world.y - 100.0).abs() < 0.01,
            "Y: {} != 100",
            mouse_world.y
        );

        // Perform AABB hit test - convert f64 to f32
        let mouse_world_f32 = Vec2::new(mouse_world.x as f32, mouse_world.y as f32);
        let hit_test_result = entity_bounds.contains(mouse_world_f32);
        assert!(
            hit_test_result,
            "Click at screen {:?} should hit entity at {:?}",
            mouse_screen, entity_pos
        );

        // Test Case 2: Click outside entity (to the right)
        // world (200, 100) → screen (600, 200)
        let mouse_screen_outside = Vec2::new(600.0, 200.0);
        let mouse_world_outside = camera.screen_to_world(mouse_screen_outside, screen_size);
        assert!((mouse_world_outside.x - 200.0).abs() < 0.01);
        assert!((mouse_world_outside.y - 100.0).abs() < 0.01);

        let mouse_outside_f32 =
            Vec2::new(mouse_world_outside.x as f32, mouse_world_outside.y as f32);
        let hit_outside = entity_bounds.contains(mouse_outside_f32);
        assert!(!hit_outside, "Click outside entity should not hit");

        // Test Case 3: Camera panned away - entity should still be hittable
        let mut camera_panned = Camera::new(800.0, 600.0);
        camera_panned.set_center_f64(Vec2f64::new(1000.0, 1000.0)); // Camera far away

        // Click where entity WOULD be if we scrolled to it
        // When camera is at (1000,1000), world (100,100) is far from center
        // Entity at (100,100) is NOT visible - test culling
        let mouse_at_entity = Vec2::new(400.0, 500.0);
        let world_from_panned = camera_panned.screen_to_world(mouse_at_entity, screen_size);

        // Entity at (100,100) should NOT contain this world point (camera is far)
        let entity_still_at_original = Rect::from_center_size(entity_pos, entity_size);
        let world_panned_f32 = Vec2::new(world_from_panned.x as f32, world_from_panned.y as f32);
        assert!(
            !entity_still_at_original.contains(world_panned_f32),
            "With camera at (1000,1000), world (100,100) should be outside viewport"
        );

        // Test Case 4: Zoom affects hit testing correctly
        let mut camera_zoomed = Camera::new(800.0, 600.0);
        camera_zoomed.zoom = 2.0; // 2x zoom

        // At zoom=2.0, viewport is half the size (300 world units high instead of 600)
        // Same screen position maps to different world position
        let mouse_at_center = Vec2::new(400.0, 300.0);
        let world_at_zoom2 = camera_zoomed.screen_to_world(mouse_at_center, screen_size);

        // At zoom=2.0, center of screen (400,300) should still be world center (0,0)
        assert!((world_at_zoom2.x - 0.0).abs() < 0.01);
        assert!((world_at_zoom2.y - 0.0).abs() < 0.01);

        // But edge of screen maps closer to center at zoom 2.0
        let mouse_at_top_left = Vec2::new(0.0, 0.0);
        let world_top_left = camera_zoomed.screen_to_world(mouse_at_top_left, screen_size);

        // At zoom=2.0 with 800x600 canvas: viewport is 400x300 world units
        // Half: 200x150
        // Screen (0,0) is top-left, which maps to world (-200, +150) with Y inverted
        assert!((world_top_left.x - (-200.0)).abs() < 0.01);
        assert!((world_top_left.y - 150.0).abs() < 0.01);
    }

    /// Test roundtrip: world_to_screen(screen_to_world(x)) = x
    #[test]
    fn test_screen_world_roundtrip() {
        let camera = Camera::new(800.0, 600.0);
        let screen_size = Vec2::new(800.0, 600.0);

        // Test multiple screen positions
        let test_positions = [
            Vec2::new(0.0, 0.0),     // Top-left
            Vec2::new(400.0, 300.0), // Center
            Vec2::new(800.0, 600.0), // Bottom-right
            Vec2::new(123.0, 456.0), // Arbitrary point
        ];

        for screen_pos in test_positions {
            let world_pos = camera.screen_to_world(screen_pos, screen_size);
            let back_to_screen = camera.world_to_screen(world_pos, screen_size);

            assert!(
                (back_to_screen.x - screen_pos.x).abs() < 0.001,
                "Roundtrip X failed: {} -> {} -> {}",
                screen_pos.x,
                world_pos.x,
                back_to_screen.x
            );
            assert!(
                (back_to_screen.y - screen_pos.y).abs() < 0.001,
                "Roundtrip Y failed: {} -> {} -> {}",
                screen_pos.y,
                world_pos.y,
                back_to_screen.y
            );
        }
    }

    #[test]
    fn test_world_to_screen() {
        let camera = Camera::new(100.0, 100.0);
        let screen_size = Vec2::new(100.0, 100.0);

        // Center of world should be center of screen
        let center_screen = camera.world_to_screen(Vec2f64::ZERO, screen_size);
        assert_eq!(center_screen, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_world_to_camera_relative() {
        let mut camera = Camera::new(100.0, 100.0);
        camera.set_center_f64(Vec2f64::new(1_000_000.0, 1_000_000.0));

        // Entity very close to camera should have precise coordinates
        let entity_pos = Vec2f64::new(1_000_010.0, 1_000_005.0);
        let relative = camera.world_to_camera(entity_pos);

        // Should be ~10.0, ~5.0 (relative to camera)
        assert!((relative.x - 10.0).abs() < 0.001);
        assert!((relative.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_zoom_affects_viewport() {
        let mut camera = Camera::new(100.0, 100.0);
        let bounds_1x = camera.viewport_bounds(100.0);
        let size_1x = bounds_1x.size();

        camera.zoom = 2.0;
        let bounds_2x = camera.viewport_bounds(100.0);
        let size_2x = bounds_2x.size();

        // At 2x zoom, viewport should be half the size
        assert!((size_2x.x - size_1x.x * 0.5).abs() < 0.01);
        assert!((size_2x.y - size_1x.y * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_viewport_bounds() {
        // Create camera with zoom=1.0 (not engine-initialized zoom)
        let camera = Camera::new(100.0, 100.0);
        let bounds = camera.viewport_bounds(100.0);

        // Bounds should be centered at origin
        assert_eq!(bounds.center(), Vec2::ZERO);

        // At zoom=1.0 with PPU=1.0 and canvas 100x100:
        // viewport height = 100 / 1.0 = 100 world units
        let size = bounds.size();
        assert!((size.x - 100.0).abs() < 0.01);
        assert!((size.y - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_pixels_per_unit_consistency() {
        // Verify PPU constant is set correctly (1:1 pixels for Figma-like editing)
        assert_eq!(PIXELS_PER_UNIT, 1.0);

        // Test that screen_to_world correctly converts pixels to world units
        // With zoom=1.0 and PPU=1.0:
        // - canvas 800x600, viewport is 800x600 world units
        // - screen center (400, 300) should map to world center (0, 0)
        let camera = Camera::new(800.0, 600.0);

        // Debug: print values
        let world_center = camera.screen_to_world(
            Vec2::new(400.0, 300.0), // Canvas center
            Vec2::new(800.0, 600.0),
        );
        assert!(
            (world_center.x - 0.0).abs() < 0.01,
            "center.x = {}",
            world_center.x
        );
        assert!(
            (world_center.y - 0.0).abs() < 0.01,
            "center.y = {}",
            world_center.y
        );

        // Top-left of screen (0, 0) should map correctly
        let world_top_left = camera.screen_to_world(Vec2::new(0.0, 0.0), Vec2::new(800.0, 600.0));

        // At zoom=1.0 with PPU=1.0 and 800x600 canvas:
        // viewport is 800x600 world units (half: 400x300)
        // ndc = (0/800)*2-1 = -1 for X
        // ndc = (0/600)*2-1 = -1 for Y (before inversion, top in DOM)
        // After Y inversion: ndc.y = -(-1) = +1
        // world = center + ndc * half = 0 + (-1)*400, (+1)*300 = (-400, +300)
        assert!(
            (world_top_left.x - (-400.0)).abs() < 0.01,
            "top_left.x = {}",
            world_top_left.x
        );
        assert!(
            (world_top_left.y - 300.0).abs() < 0.01,
            "top_left.y = {}",
            world_top_left.y
        );
    }
}
