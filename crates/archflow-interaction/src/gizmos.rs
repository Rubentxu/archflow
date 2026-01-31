// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Gizmo Renderer (Immediate Mode UI)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 15
//
// Immediate Mode UI Rendering for debug visualization:
// - Selection boxes with handles
// - Lines and shapes
// - Debug overlays
// - Zero retained state - rebuilt every frame
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{Color, Rect, Vec2};

/// Maximum number of gizmo instances that can be drawn per frame
pub const MAX_GIZMO_INSTANCES: usize = 512;

/// Shape type for gizmo rendering
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoShape {
    /// Rectangle (filled or outline)
    Rectangle = 0,
    /// Circle/Ellipse
    Circle = 1,
    /// Line (can be dashed)
    Line = 2,
    /// Diamond (for handles)
    Diamond = 3,
    /// Cross (for rotation handles)
    Cross = 4,
}

/// Cursor type for handles
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoCursor {
    /// Default pointer
    Arrow = 0,
    /// Move cursor (4-directional arrows)
    Move = 1,
    /// Resize cursors (8 directions)
    ResizeNW = 2,
    ResizeN = 3,
    ResizeNE = 4,
    ResizeE = 5,
    ResizeSE = 6,
    ResizeS = 7,
    ResizeSW = 8,
    ResizeW = 9,
    /// Rotation cursor (circular arrow)
    Rotate = 10,
    /// Pointer finger (for clickable handles)
    Pointer = 11,
    /// Crosshair (for precision tools)
    Crosshair = 12,
    /// Not-allowed (prohibited operation)
    NotAllowed = 13,
}

/// Gizmo instance data (32 bytes, matching GpuInstance layout)
///
/// This structure mirrors the GpuInstance layout for efficient
/// direct upload to the GPU without reformatting.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct GizmoInstance {
    /// Position [x, y] in world coordinates
    pub pos: [f32; 2],

    /// Size [w, h] in world coordinates
    pub size: [f32; 2],

    /// Packed color as 0xRRGGBBAA
    pub color: u32,

    /// Shape type for rendering
    pub shape_type: u8,

    /// Cursor type when hovering over this gizmo
    pub cursor: u8,

    /// Data field (angle for rotation, dash pattern for lines, etc.)
    pub data: [f32; 2],
}

impl GizmoInstance {
    /// Create a new gizmo instance
    #[inline(always)]
    pub const fn new(pos: Vec2, size: Vec2, color: u32, shape_type: GizmoShape) -> Self {
        Self {
            pos: [pos.x, pos.y],
            size: [size.x, size.y],
            color,
            shape_type: shape_type as u8,
            cursor: GizmoCursor::Arrow as u8,
            data: [0.0, 0.0],
        }
    }

    /// Set the cursor type for this gizmo
    #[inline(always)]
    pub const fn with_cursor(mut self, cursor: GizmoCursor) -> Self {
        self.cursor = cursor as u8;
        self
    }

    /// Set the data field (for rotation angle, dash pattern, etc.)
    #[inline(always)]
    pub const fn with_data(mut self, data: [f32; 2]) -> Self {
        self.data = data;
        self
    }
}

/// Immediate Mode Gizmo Renderer
///
/// **Immediate Mode** means:
/// - No retained state between frames
/// - All gizmos are rebuilt every frame based on current state
/// - Zero allocation hot path (pre-allocated buffer)
/// - Simple API: draw commands → submit → render
///
/// **Performance:**
/// - Pre-allocated buffer of 512 instances
/// - Zero heap allocations during drawing
/// - Direct GPU upload via slice
pub struct GizmoRenderer {
    /// Pre-allocated instance buffer
    instances: Vec<GizmoInstance>,

    /// Current number of active instances
    count: usize,
}

impl GizmoRenderer {
    /// Create a new gizmo renderer
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(MAX_GIZMO_INSTANCES),
            count: 0,
        }
    }

    /// Clear all gizmos (call at start of each frame)
    pub fn clear(&mut self) {
        self.count = 0;
        // Note: we don't clear the vector to avoid reallocation
        // The count field determines what gets rendered
    }

    /// Draw a selection box with handles
    ///
    /// Renders a rectangular selection box with:
    /// - Border (4 orthogonal lines)
    /// - Semi-transparent fill
    /// - Corner handles for resizing
    ///
    /// # Arguments
    /// * `bounds` - Rectangle to draw
    /// * `color` - Border color (0xRRGGBBAA format)
    pub fn draw_selection_box(&mut self, bounds: Rect, color: u32) {
        // Calculate corners of the rectangle
        let min = bounds.min;
        let max = bounds.max;
        let corners = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];

        // Draw border (4 lines)
        for i in 0..4 {
            let start = corners[i];
            let end = corners[(i + 1) % 4];
            self.draw_line(start, end, color, 2.0, false);
        }

        // Draw semi-transparent fill
        let fill_color = Color(color).with_alpha(0x20); // ~12% alpha
        self.draw_rect_filled(min, max, Some(fill_color.as_u32()));

        // Draw corner handles
        let handle_size = 8.0;
        for corner in &corners {
            self.draw_handle(*corner, handle_size);
        }
    }

    /// Draw a line from start to end
    ///
    /// # Arguments
    /// * `start` - Start position
    /// * `end` - End position
    /// * `color` - Line color (0xRRGGBBAA)
    /// * `width` - Line width in pixels
    /// * `dashed` - Whether to draw a dashed line
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: u32, width: f32, dashed: bool) {
        if self.count >= MAX_GIZMO_INSTANCES {
            return; // Buffer full
        }

        let center = (start + end) / 2.0;
        let length = start.distance(end);

        let dash_data = if dashed { [10.0, 5.0] } else { [0.0, 0.0] };

        let instance = GizmoInstance {
            pos: [center.x, center.y],
            size: [length, width],
            color,
            shape_type: GizmoShape::Line as u8,
            cursor: GizmoCursor::Arrow as u8,
            data: dash_data,
        };

        // Ensure capacity before pushing
        if self.instances.len() <= self.count {
            self.instances.push(instance);
        } else {
            self.instances[self.count] = instance;
        }
        self.count += 1;
    }

    /// Draw a filled rectangle
    ///
    /// # Arguments
    /// * `min` - Minimum corner position
    /// * `max` - Maximum corner position
    /// * `fill` - Fill color, or None for transparent
    pub fn draw_rect_filled(&mut self, min: Vec2, max: Vec2, fill: Option<u32>) {
        if self.count >= MAX_GIZMO_INSTANCES {
            return;
        }

        let center = (min + max) / 2.0;
        let size = max - min;

        let instance = GizmoInstance {
            pos: [center.x, center.y],
            size: [size.x, size.y],
            color: fill.unwrap_or(0x00000000),
            shape_type: GizmoShape::Rectangle as u8,
            cursor: GizmoCursor::Arrow as u8,
            data: [0.0, 0.0],
        };

        // Ensure capacity before pushing
        if self.instances.len() <= self.count {
            self.instances.push(instance);
        } else {
            self.instances[self.count] = instance;
        }
        self.count += 1;
    }

    /// Draw a circular handle
    ///
    /// # Arguments
    /// * `pos` - Center position of the handle
    /// * `size` - Diameter of the handle
    pub fn draw_handle(&mut self, pos: Vec2, size: f32) {
        if self.count >= MAX_GIZMO_INSTANCES {
            return;
        }

        let instance = GizmoInstance {
            pos: [pos.x, pos.y],
            size: [size, size],
            color: 0xFFFFFFFF, // White
            shape_type: GizmoShape::Circle as u8,
            cursor: GizmoCursor::Move as u8,
            data: [0.0, 0.0],
        };

        // Ensure capacity before pushing
        if self.instances.len() <= self.count {
            self.instances.push(instance);
        } else {
            self.instances[self.count] = instance;
        }
        self.count += 1;
    }

    /// Draw a circle
    ///
    /// # Arguments
    /// * `center` - Center position
    /// * `radius` - Circle radius
    /// * `color` - Circle color
    pub fn draw_circle(&mut self, center: Vec2, radius: f32, color: u32) {
        if self.count >= MAX_GIZMO_INSTANCES {
            return;
        }

        let size = Vec2::splat(radius * 2.0);

        let instance = GizmoInstance {
            pos: [center.x, center.y],
            size: [size.x, size.y],
            color,
            shape_type: GizmoShape::Circle as u8,
            cursor: GizmoCursor::Arrow as u8,
            data: [0.0, 0.0],
        };

        // Ensure capacity before pushing
        if self.instances.len() <= self.count {
            self.instances.push(instance);
        } else {
            self.instances[self.count] = instance;
        }
        self.count += 1;
    }

    /// Draw a diamond shape (for rotation handles, etc.)
    ///
    /// # Arguments
    /// * `center` - Center position
    /// * `size` - Width/height of the diamond
    /// * `color` - Fill color
    pub fn draw_diamond(&mut self, center: Vec2, size: f32, color: u32) {
        if self.count >= MAX_GIZMO_INSTANCES {
            return;
        }

        let size_vec = Vec2::splat(size);

        let instance = GizmoInstance {
            pos: [center.x, center.y],
            size: [size_vec.x, size_vec.y],
            color,
            shape_type: GizmoShape::Diamond as u8,
            cursor: GizmoCursor::Arrow as u8,
            data: [0.0, 0.0],
        };

        // Ensure capacity before pushing
        if self.instances.len() <= self.count {
            self.instances.push(instance);
        } else {
            self.instances[self.count] = instance;
        }
        self.count += 1;
    }

    /// Draw a crosshair (for rotation centers, etc.)
    ///
    /// # Arguments
    /// * `center` - Center position
    /// * `size` - Size of the crosshair
    /// * `color` - Line color
    pub fn draw_crosshair(&mut self, center: Vec2, size: f32, color: u32) {
        // Horizontal line
        let h_start = center - Vec2::new(size / 2.0, 0.0);
        let h_end = center + Vec2::new(size / 2.0, 0.0);
        self.draw_line(h_start, h_end, color, 1.0, false);

        // Vertical line
        let v_start = center - Vec2::new(0.0, size / 2.0);
        let v_end = center + Vec2::new(0.0, size / 2.0);
        self.draw_line(v_start, v_end, color, 1.0, false);
    }

    /// Draw a dashed line
    ///
    /// # Arguments
    /// * `start` - Start position
    /// * `end` - End position
    /// * `color` - Line color
    /// * `width` - Line width
    pub fn draw_dashed_line(&mut self, start: Vec2, end: Vec2, color: u32, width: f32) {
        self.draw_line(start, end, color, width, true);
    }

    /// Draw grid lines for reference
    ///
    /// # Arguments
    /// * `bounds` - Area to cover with grid
    /// * `spacing` - Distance between grid lines
    /// * `color` - Grid line color
    pub fn draw_grid(&mut self, bounds: Rect, spacing: f32, color: u32) {
        let min = bounds.min;
        let max = bounds.max;

        // Vertical lines
        let mut x = (min.x / spacing).floor() * spacing;
        while x <= max.x {
            let start = Vec2::new(x, min.y);
            let end = Vec2::new(x, max.y);
            self.draw_line(start, end, color, 1.0, true);
            x += spacing;
        }

        // Horizontal lines
        let mut y = (min.y / spacing).floor() * spacing;
        while y <= max.y {
            let start = Vec2::new(min.x, y);
            let end = Vec2::new(max.x, y);
            self.draw_line(start, end, color, 1.0, true);
            y += spacing;
        }
    }

    /// Draw an arrow (for indicating direction)
    ///
    /// # Arguments
    /// * `from` - Start position
    /// * `to` - End position (arrowhead points here)
    /// * `color` - Arrow color
    /// * `width` - Line width
    pub fn draw_arrow(&mut self, from: Vec2, to: Vec2, color: u32, width: f32) {
        // Draw main line
        self.draw_line(from, to, color, width, false);

        // Draw arrowhead
        let direction = (to - from).normalize();
        let arrow_size: f32 = 10.0;
        let arrow_angle: f32 = 0.5; // ~30 degrees

        let cos_a = (arrow_angle).cos();
        let sin_a = (arrow_angle).sin();

        // Left wing
        let left_dir = archflow_core::Vec2::new(
            direction.x * cos_a - direction.y * sin_a,
            direction.x * sin_a + direction.y * cos_a,
        );
        let left_wing = to - left_dir * arrow_size;
        self.draw_line(to, left_wing, color, width, false);

        // Right wing
        let right_dir = archflow_core::Vec2::new(
            direction.x * cos_a + direction.y * sin_a,
            -direction.x * sin_a + direction.y * cos_a,
        );
        let right_wing = to - right_dir * arrow_size;
        self.draw_line(to, right_wing, color, width, false);
    }

    /// Get the current number of gizmo instances
    #[inline(always)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Check if there are any gizmos to render
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get a slice of all gizmo instances
    pub fn instances(&self) -> &[GizmoInstance] {
        &self.instances[..self.count]
    }

    /// Submit gizmos for rendering
    ///
    /// This is a placeholder - actual implementation would
    /// upload instances to the GPU for rendering.
    ///
    /// In the full implementation, this would be integrated
    /// with the GpuRenderer to draw all gizmos as an overlay.
    pub fn submit(&mut self) -> &[GizmoInstance] {
        self.instances()
    }
}

impl Default for GizmoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Rect;

    #[test]
    fn test_gizmo_instance_creation() {
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(50.0, 30.0);
        let color = 0xFF0000FF;

        let instance = GizmoInstance::new(pos, size, color, GizmoShape::Rectangle);

        assert_eq!(instance.pos, [10.0, 20.0]);
        assert_eq!(instance.size, [50.0, 30.0]);
        assert_eq!(instance.color, color);
        assert_eq!(instance.shape_type, GizmoShape::Rectangle as u8);
    }

    #[test]
    fn test_gizmo_instance_with_cursor() {
        let instance = GizmoInstance::new(Vec2::ZERO, Vec2::ONE, 0xFFFFFFFF, GizmoShape::Circle)
            .with_cursor(GizmoCursor::Move);

        assert_eq!(instance.cursor, GizmoCursor::Move as u8);
    }

    #[test]
    fn test_gizmo_instance_with_data() {
        let instance = GizmoInstance::new(Vec2::ZERO, Vec2::ONE, 0xFFFFFFFF, GizmoShape::Line)
            .with_data([1.5, 2.5]);

        assert_eq!(instance.data, [1.5, 2.5]);
    }

    #[test]
    fn test_gizmo_renderer_creation() {
        let renderer = GizmoRenderer::new();
        assert_eq!(renderer.count(), 0);
        assert!(renderer.is_empty());
    }

    #[test]
    fn test_gizmo_renderer_default() {
        let renderer = GizmoRenderer::default();
        assert_eq!(renderer.count(), 0);
    }

    #[test]
    fn test_gizmo_renderer_clear() {
        let mut renderer = GizmoRenderer::new();

        // Draw some gizmos
        renderer.draw_rect_filled(Vec2::ZERO, Vec2::ONE, Some(0xFF0000FF));
        assert_eq!(renderer.count(), 1);

        // Clear
        renderer.clear();
        assert_eq!(renderer.count(), 0);
        assert!(renderer.is_empty());
    }

    #[test]
    fn test_draw_rect_filled() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_rect_filled(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 50.0),
            Some(0xFF0000FF),
        );

        assert_eq!(renderer.count(), 1);
        let instances = renderer.instances();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].shape_type, GizmoShape::Rectangle as u8);
    }

    #[test]
    fn test_draw_circle() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_circle(Vec2::new(50.0, 50.0), 25.0, 0x00FF00FF);

        assert_eq!(renderer.count(), 1);
        let instances = renderer.instances();
        assert_eq!(instances[0].shape_type, GizmoShape::Circle as u8);
        assert_eq!(instances[0].size, [50.0, 50.0]); // radius * 2
    }

    #[test]
    fn test_draw_line() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_line(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            0xFFFFFFFF,
            2.0,
            false,
        );

        assert_eq!(renderer.count(), 1);
        let instances = renderer.instances();
        assert_eq!(instances[0].shape_type, GizmoShape::Line as u8);
    }

    #[test]
    fn test_draw_dashed_line() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_dashed_line(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            0xFF0000FF,
            2.0,
        );

        assert_eq!(renderer.count(), 1);
        let instances = renderer.instances();
        // Dashed lines have non-zero data field
        assert!(instances[0].data[0] > 0.0 || instances[0].data[1] > 0.0);
    }

    #[test]
    fn test_draw_handle() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_handle(Vec2::new(100.0, 100.0), 10.0);

        assert_eq!(renderer.count(), 1);
        let instances = renderer.instances();
        assert_eq!(instances[0].shape_type, GizmoShape::Circle as u8);
        assert_eq!(instances[0].cursor, GizmoCursor::Move as u8);
    }

    #[test]
    fn test_draw_crosshair() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_crosshair(Vec2::new(50.0, 50.0), 20.0, 0xFFFFFFFF);

        // Crosshair draws 2 lines
        assert_eq!(renderer.count(), 2);
    }

    #[test]
    fn test_draw_diamond() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_diamond(Vec2::new(50.0, 50.0), 20.0, 0xFF00FFFF);

        assert_eq!(renderer.count(), 1);
        let instances = renderer.instances();
        assert_eq!(instances[0].shape_type, GizmoShape::Diamond as u8);
    }

    #[test]
    fn test_draw_arrow() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_arrow(Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 0xFFFFFFFF, 2.0);

        // Arrow draws 1 main line + 2 wings
        assert_eq!(renderer.count(), 3);
    }

    #[test]
    fn test_draw_selection_box() {
        let mut renderer = GizmoRenderer::new();

        let bounds = Rect::from_center_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        renderer.draw_selection_box(bounds, 0x4488FF);

        // Selection box: 4 border lines + 1 fill rect + 4 handles = 9 gizmos
        assert_eq!(renderer.count(), 9);
    }

    #[test]
    fn test_capacity_limit() {
        let mut renderer = GizmoRenderer::new();

        // Fill to capacity
        for _ in 0..MAX_GIZMO_INSTANCES {
            renderer.draw_rect_filled(Vec2::ZERO, Vec2::ONE, Some(0xFF0000FF));
        }
        assert_eq!(renderer.count(), MAX_GIZMO_INSTANCES);

        // This one should be ignored
        renderer.draw_rect_filled(Vec2::ZERO, Vec2::ONE, Some(0xFF0000FF));
        assert_eq!(renderer.count(), MAX_GIZMO_INSTANCES);
    }

    #[test]
    fn test_cursor_types() {
        assert_eq!(GizmoCursor::Arrow as u8, 0);
        assert_eq!(GizmoCursor::Move as u8, 1);
        assert_eq!(GizmoCursor::Rotate as u8, 10);
        assert_eq!(GizmoCursor::Pointer as u8, 11);
    }

    #[test]
    fn test_shape_types() {
        assert_eq!(GizmoShape::Rectangle as u8, 0);
        assert_eq!(GizmoShape::Circle as u8, 1);
        assert_eq!(GizmoShape::Line as u8, 2);
        assert_eq!(GizmoShape::Diamond as u8, 3);
        assert_eq!(GizmoShape::Cross as u8, 4);
    }

    #[test]
    fn test_instances_slice() {
        let mut renderer = GizmoRenderer::new();

        renderer.draw_circle(Vec2::new(10.0, 10.0), 5.0, 0xFFFFFFFF);
        renderer.draw_circle(Vec2::new(20.0, 20.0), 5.0, 0xFFFFFFFF);

        let instances = renderer.instances();
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].pos, [10.0, 10.0]);
        assert_eq!(instances[1].pos, [20.0, 20.0]);
    }
}
