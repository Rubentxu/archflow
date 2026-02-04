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
    /// We essentially generate a projection matrix assuming the camera is at (0,0).
    pub fn build_view_projection_matrix(&self) -> [[f32; 4]; 4] {
        // Calculate half-width and half-height of view in world coordinates
        let half_height = 1.0 / self.zoom;
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
    ///
    /// Useful for:
    /// - Hit testing (converting mouse position)
    /// - Positioning new entities
    /// - Snap to grid
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2f64 {
        // Normalize to device normalized coordinates (NDC) [-1, 1]
        // Screen Y=0 is top (DOM), World Y-up means we need:
        // - Screen top (Y=0) → NDC Y=1 → World +Y
        // - Screen bottom (Y=max) → NDC Y=-1 → World -Y
        // NDC formula: (pos / size) * 2 - 1
        // For Y: (0/height)*2-1 = 1 (top), (height/height)*2-1 = -1 (bottom)
        // This naturally maps Y-down to Y-up, NO extra flip needed!
        let ndc = (screen_pos / screen_size) * 2.0 - Vec2::ONE;

        // Apply inverse zoom to get world coordinates (f64 for precision)
        let half_height = 1.0 / self.zoom as f64;
        let half_width = half_height * self.aspect_ratio as f64;

        Vec2f64::new(
            self.center.x + ndc.x as f64 * half_width,
            self.center.y + ndc.y as f64 * half_height,
        )
    }

    /// Convert world coordinates to screen coordinates (returns f32)
    pub fn world_to_screen(&self, world_pos: Vec2f64, screen_size: Vec2) -> Vec2 {
        let half_height = 1.0 / self.zoom;
        let half_width = half_height * self.aspect_ratio;

        // Convert to f64 for precise computation
        let half_width_f64 = half_width as f64;
        let half_height_f64 = half_height as f64;

        // World coordinates to NDC [-1, 1]
        // World +Y (top) → NDC Y=1 → Screen Y=0 (top)
        // World -Y (bottom) → NDC Y=-1 → Screen Y=height (bottom)
        // The NDC conversion naturally handles the Y-flip, NO extra flip needed!
        let ndc = Vec2::new(
            ((world_pos.x - self.center.x) / half_width_f64) as f32,
            ((world_pos.y - self.center.y) / half_height_f64) as f32,
        );

        // NDC [-1, 1] to Screen [0, width/height]
        (ndc * 0.5 + 0.5) * screen_size
    }

    /// Get the visible rectangle of the camera in world coordinates
    ///
    /// Useful for:
    /// - Viewport culling (only render what's visible)
    /// - Determining which icons to lazy load
    pub fn viewport_bounds(&self) -> Rect {
        let half_height = 1.0 / self.zoom;
        let half_width = half_height * self.aspect_ratio;

        // Use center_f64() for compatibility
        let center = self.center_f64();
        Rect::from_center_size(
            Vec2::new(center.x as f32, center.y as f32),
            Vec2::new(half_width * 2.0, half_height * 2.0),
        )
    }

    /// Check if a world-space point is visible in the viewport
    pub fn is_visible(&self, world_pos: Vec2f64) -> bool {
        let bounds = self.viewport_bounds();
        let pos_f32 = Vec2::new(world_pos.x as f32, world_pos.y as f32);
        bounds.contains(pos_f32)
    }

    /// Check if a world-space rectangle intersects the viewport
    pub fn is_rect_visible(&self, rect: Rect) -> bool {
        self.viewport_bounds().intersects(&rect)
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

        // Top-right corner of screen
        let corner_world = camera.screen_to_world(Vec2::new(100.0, 100.0), screen_size);
        assert!(corner_world.x > 0.0 && corner_world.y > 0.0);
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
        let bounds_1x = camera.viewport_bounds();
        let size_1x = bounds_1x.size();

        camera.zoom = 2.0;
        let bounds_2x = camera.viewport_bounds();
        let size_2x = bounds_2x.size();

        // At 2x zoom, viewport should be half the size
        assert!((size_2x.x - size_1x.x * 0.5).abs() < 0.01);
        assert!((size_2x.y - size_1x.y * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_viewport_bounds() {
        let camera = Camera::new(100.0, 100.0);
        let bounds = camera.viewport_bounds();

        // Bounds should be centered at origin
        assert_eq!(bounds.center(), Vec2::ZERO);

        // At zoom 1.0 and 1:1 aspect, should be 2x2 units
        let size = bounds.size();
        assert!((size.x - 2.0).abs() < 0.01);
        assert!((size.y - 2.0).abs() < 0.01);
    }
}
