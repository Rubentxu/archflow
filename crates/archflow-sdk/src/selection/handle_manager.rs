//! Selection Handle Manager - Handle rendering and hit testing with caching
//!
//! This module provides:
//! - HandleType: Enum for all 9 handle types (8 resize + 1 rotation)
//! - SelectionHandle: Individual handle with position and size
//! - UnifiedBounds: Bounding box for multi-entity selection
//! - HandleCache: Optimized hit testing with caching
//! - SelectionHandleManager: Main manager for handle operations

use crate::canvas::Shape;
use crate::tools::CursorType;
use archflow_core::{EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Handle type for selection
///
/// Represents all 9 handle types available for selection:
/// - 8 resize handles positioned at corners and edges
/// - 1 rotation handle positioned above the selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleType {
    /// Top-left corner resize handle
    ResizeNorthWest,
    /// Top edge resize handle
    ResizeNorth,
    /// Top-right corner resize handle
    ResizeNorthEast,
    /// Right edge resize handle
    ResizeEast,
    /// Bottom-right corner resize handle
    ResizeSouthEast,
    /// Bottom edge resize handle
    ResizeSouth,
    /// Bottom-left corner resize handle
    ResizeSouthWest,
    /// Left edge resize handle
    ResizeWest,
    /// Rotation handle (positioned above the selection)
    Rotate,
}

impl HandleType {
    /// Get the cursor type to display when hovering over this handle
    #[inline]
    pub fn cursor(&self) -> CursorType {
        match self {
            HandleType::ResizeNorthWest | HandleType::ResizeSouthEast => CursorType::ResizeNWSE,
            HandleType::ResizeNorthEast | HandleType::ResizeSouthWest => CursorType::ResizeNESW,
            HandleType::ResizeNorth | HandleType::ResizeSouth => CursorType::ResizeNS,
            HandleType::ResizeEast | HandleType::ResizeWest => CursorType::ResizeEW,
            HandleType::Rotate => CursorType::Grab,
        }
    }

    /// Check if this is a corner handle (diagonal resize)
    #[inline]
    pub fn is_corner(&self) -> bool {
        matches!(
            self,
            HandleType::ResizeNorthWest
                | HandleType::ResizeNorthEast
                | HandleType::ResizeSouthEast
                | HandleType::ResizeSouthWest
        )
    }

    /// Check if this is an edge handle (single-axis resize)
    #[inline]
    pub fn is_edge(&self) -> bool {
        matches!(
            self,
            HandleType::ResizeNorth
                | HandleType::ResizeSouth
                | HandleType::ResizeEast
                | HandleType::ResizeWest
        )
    }

    /// Check if this is the rotation handle
    #[inline]
    pub fn is_rotation(&self) -> bool {
        *self == HandleType::Rotate
    }

    /// Get the axis for edge handles (None for corners and rotation)
    #[inline]
    pub fn axis(&self) -> Option<Axis> {
        match self {
            HandleType::ResizeNorth | HandleType::ResizeSouth => Some(Axis::Y),
            HandleType::ResizeEast | HandleType::ResizeWest => Some(Axis::X),
            _ => None,
        }
    }
}

/// Axis for edge handle operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// X-axis (horizontal)
    X,
    /// Y-axis (vertical)
    Y,
}

/// A single selection handle
///
/// Contains all information needed to render and hit-test a handle.
#[derive(Debug, Clone)]
pub struct SelectionHandle {
    /// Type of handle
    pub handle_type: HandleType,
    /// Position of handle center in canvas coordinates
    pub position: Vec2,
    /// Size of handle in pixels (square)
    pub size: f32,
    /// Cursor type to display
    pub cursor: CursorType,
}

impl SelectionHandle {
    /// Create a new selection handle
    #[inline]
    pub fn new(handle_type: HandleType, position: Vec2, size: f32) -> Self {
        Self {
            handle_type,
            position,
            size,
            cursor: handle_type.cursor(),
        }
    }

    /// Get the bounds of this handle for hit testing
    ///
    /// Returns a rectangle centered on the handle position
    /// with the handle size as both width and height.
    #[inline]
    pub fn bounds(&self) -> Rect {
        Rect::from_center_size(self.position, Vec2::splat(self.size))
    }

    /// Check if a point is within this handle's hit area
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }
}

/// Unified bounding box for multi-entity selection
///
/// Represents the combined bounds of all selected entities,
/// used for calculating handle positions and multi-entity transforms.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnifiedBounds {
    /// Minimum corner (top-left in screen coordinates)
    pub min: Vec2,
    /// Maximum corner (bottom-right in screen coordinates)
    pub max: Vec2,
    /// Center point
    pub center: Vec2,
    /// Width of bounds
    pub width: f32,
    /// Height of bounds
    pub height: f32,
}

impl UnifiedBounds {
    /// Create unified bounds from a collection of shapes
    #[inline]
    pub fn from_shapes(shapes: &[Shape]) -> Option<Self> {
        if shapes.is_empty() {
            return None;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for shape in shapes {
            let bounds = shape.bounds();
            min_x = min_x.min(bounds.min.x);
            min_y = min_y.min(bounds.min.y);
            max_x = max_x.max(bounds.max.x);
            max_y = max_y.max(bounds.max.y);
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let center = Vec2::new(min_x + width / 2.0, min_y + height / 2.0);

        Some(Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
            center,
            width,
            height,
        })
    }

    /// Create unified bounds from entity bounds
    #[inline]
    pub fn from_entity_bounds(entities: &[(EntityId, Rect)]) -> Option<Self> {
        if entities.is_empty() {
            return None;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for (_, bounds) in entities {
            min_x = min_x.min(bounds.min.x);
            min_y = min_y.min(bounds.min.y);
            max_x = max_x.max(bounds.max.x);
            max_y = max_y.max(bounds.max.y);
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let center = Vec2::new(min_x + width / 2.0, min_y + height / 2.0);

        Some(Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
            center,
            width,
            height,
        })
    }

    /// Calculate the aspect ratio (width / height)
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        if self.height > 0.0 {
            self.width / self.height
        } else {
            1.0
        }
    }

    /// Check if bounds are valid (positive width and height)
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

/// Cached handle data for optimized hit testing
///
/// Maintains a hash map of handle bounds for quick lookup
/// during pointer events. Includes invalidation logic for
/// when handles need to be recalculated.
#[derive(Debug, Clone)]
pub struct HandleCache {
    handle_bounds: HashMap<HandleType, Rect>,
    last_update: Instant,
    dirty: bool,
}

impl HandleCache {
    /// Create a new empty handle cache
    #[inline]
    pub fn new() -> Self {
        Self {
            handle_bounds: HashMap::new(),
            last_update: Instant::now(),
            dirty: true,
        }
    }

    /// Invalidate the cache, forcing recalculation on next access
    #[inline]
    pub fn invalidate(&mut self) {
        self.dirty = true;
        self.handle_bounds.clear();
    }

    /// Update the cache with new handle data
    #[inline]
    pub fn update(&mut self, handles: &[SelectionHandle]) {
        self.handle_bounds.clear();
        for handle in handles {
            self.handle_bounds
                .insert(handle.handle_type, handle.bounds());
        }
        self.last_update = Instant::now();
        self.dirty = false;
    }

    /// Check if the cache is still valid
    ///
    /// # Arguments
    ///
    /// * `max_age` - Maximum age before cache is considered stale
    #[inline]
    pub fn is_valid(&self, max_age: Duration) -> bool {
        !self.dirty && self.last_update.elapsed() < max_age
    }

    /// Test a point against all cached handles
    ///
    /// Returns the handle type if the point is within any handle,
    /// otherwise returns None.
    #[inline]
    pub fn hit_test(&self, point: Vec2) -> Option<HandleType> {
        for (handle_type, bounds) in &self.handle_bounds {
            if bounds.contains(point) {
                return Some(*handle_type);
            }
        }
        None
    }

    /// Get the bounds for a specific handle type
    #[inline]
    pub fn get_bounds(&self, handle_type: HandleType) -> Option<&Rect> {
        self.handle_bounds.get(&handle_type)
    }
}

impl Default for HandleCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for selection handles
///
/// Provides methods for calculating handle positions, hit testing,
/// and cache management.
#[derive(Debug, Clone)]
pub struct SelectionHandleManager {
    /// Default handle size in pixels
    handle_size: f32,
    /// Cache for handle hit testing
    cache: HandleCache,
    /// Current handles (updated when selection changes)
    current_handles: Vec<SelectionHandle>,
    /// Position of rotation handle above selection
    rotation_offset: f32,
}

impl SelectionHandleManager {
    /// Create a new selection handle manager with default settings
    #[inline]
    pub fn new() -> Self {
        Self {
            handle_size: 8.0,
            cache: HandleCache::new(),
            current_handles: Vec::new(),
            rotation_offset: 20.0,
        }
    }

    /// Set the handle size
    ///
    /// Changes will take effect on next handle calculation.
    /// Also invalidates the cache.
    #[inline]
    pub fn set_handle_size(&mut self, size: f32) {
        self.handle_size = size;
        self.cache.invalidate();
    }

    /// Set the rotation handle offset from selection bounds
    #[inline]
    pub fn set_rotation_offset(&mut self, offset: f32) {
        self.rotation_offset = offset;
        self.cache.invalidate();
    }

    /// Calculate handles for given unified bounds
    ///
    /// Returns 9 handles: 8 for resize (corners + edges) and 1 for rotation.
    #[inline]
    pub fn calculate_handles_from_bounds(&self, bounds: UnifiedBounds) -> Vec<SelectionHandle> {
        let min = bounds.min;
        let max = bounds.max;
        let center = bounds.center;

        // Position rotation handle above the selection bounds
        let rotation_pos = Vec2::new(center.x, min.y - self.rotation_offset);

        vec![
            // Corner handles
            SelectionHandle::new(HandleType::ResizeNorthWest, min, self.handle_size),
            SelectionHandle::new(
                HandleType::ResizeNorthEast,
                Vec2::new(max.x, min.y),
                self.handle_size,
            ),
            SelectionHandle::new(HandleType::ResizeSouthEast, max, self.handle_size),
            SelectionHandle::new(
                HandleType::ResizeSouthWest,
                Vec2::new(min.x, max.y),
                self.handle_size,
            ),
            // Edge handles
            SelectionHandle::new(
                HandleType::ResizeNorth,
                Vec2::new(center.x, min.y),
                self.handle_size,
            ),
            SelectionHandle::new(
                HandleType::ResizeEast,
                Vec2::new(max.x, center.y),
                self.handle_size,
            ),
            SelectionHandle::new(
                HandleType::ResizeSouth,
                Vec2::new(center.x, max.y),
                self.handle_size,
            ),
            SelectionHandle::new(
                HandleType::ResizeWest,
                Vec2::new(min.x, center.y),
                self.handle_size,
            ),
            // Rotation handle
            SelectionHandle::new(HandleType::Rotate, rotation_pos, self.handle_size),
        ]
    }

    /// Calculate handles for a collection of entities
    ///
    /// Computes the unified bounds first, then calculates handles.
    /// Returns empty vector if no entities provided.
    #[inline]
    pub fn calculate_handles_for_entities(
        &self,
        entities: &[(EntityId, Rect)],
    ) -> Vec<SelectionHandle> {
        if entities.is_empty() {
            return Vec::new();
        }

        let unified_bounds = match UnifiedBounds::from_entity_bounds(entities) {
            Some(bounds) => bounds,
            None => return Vec::new(),
        };

        self.calculate_handles_from_bounds(unified_bounds)
    }

    /// Update the current handles and cache
    #[inline]
    pub fn update_handles(&mut self, handles: &[SelectionHandle]) {
        self.current_handles = handles.to_vec();
        self.cache.update(handles);
    }

    /// Get the current handles
    #[inline]
    pub fn handles(&self) -> &[SelectionHandle] {
        &self.current_handles
    }

    /// Get the cache reference
    #[inline]
    pub fn cache(&self) -> &HandleCache {
        &self.cache
    }

    /// Get mutable cache reference
    #[inline]
    pub fn cache_mut(&mut self) -> &mut HandleCache {
        &mut self.cache
    }

    /// Hit test a point against cached handles
    ///
    /// Returns the handle type if found, otherwise None.
    #[inline]
    pub fn hit_test(&self, point: Vec2) -> Option<HandleType> {
        self.cache.hit_test(point)
    }

    /// Get the handle size
    #[inline]
    pub fn handle_size(&self) -> f32 {
        self.handle_size
    }

    /// Get the rotation handle offset
    #[inline]
    pub fn rotation_offset(&self) -> f32 {
        self.rotation_offset
    }
}

impl Default for SelectionHandleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_type_cursor() {
        assert_eq!(HandleType::ResizeNorthWest.cursor(), CursorType::ResizeNWSE);
        assert_eq!(HandleType::ResizeNorth.cursor(), CursorType::ResizeNS);
        assert_eq!(HandleType::ResizeEast.cursor(), CursorType::ResizeEW);
        assert_eq!(HandleType::Rotate.cursor(), CursorType::Grab);
    }

    #[test]
    fn test_handle_type_is_corner() {
        assert!(HandleType::ResizeNorthWest.is_corner());
        assert!(!HandleType::ResizeNorth.is_corner());
        assert!(!HandleType::Rotate.is_corner());
    }

    #[test]
    fn test_handle_type_is_edge() {
        assert!(!HandleType::ResizeNorthWest.is_edge());
        assert!(HandleType::ResizeNorth.is_edge());
        assert!(!HandleType::Rotate.is_edge());
    }

    #[test]
    fn test_handle_type_axis() {
        assert_eq!(HandleType::ResizeNorth.axis(), Some(Axis::Y));
        assert_eq!(HandleType::ResizeEast.axis(), Some(Axis::X));
        assert_eq!(HandleType::Rotate.axis(), None);
    }

    #[test]
    fn test_selection_handle_bounds() {
        let handle =
            SelectionHandle::new(HandleType::ResizeNorthWest, Vec2::new(100.0, 100.0), 10.0);
        let bounds = handle.bounds();

        // Center at (100, 100) with size 10 means bounds from (95, 95) to (105, 105)
        assert!((bounds.min.x - 95.0).abs() < 0.001);
        assert!((bounds.min.y - 95.0).abs() < 0.001);
        assert!((bounds.max.x - 105.0).abs() < 0.001);
        assert!((bounds.max.y - 105.0).abs() < 0.001);
    }

    #[test]
    fn test_selection_handle_contains() {
        let handle =
            SelectionHandle::new(HandleType::ResizeNorthWest, Vec2::new(100.0, 100.0), 10.0);

        assert!(handle.contains(Vec2::new(100.0, 100.0))); // Center
        assert!(handle.contains(Vec2::new(99.0, 99.0))); // Near center
        assert!(!handle.contains(Vec2::new(50.0, 50.0))); // Far away
    }

    #[test]
    fn test_unified_bounds_from_shapes() {
        let shape1 = Shape::new_rectangle(0.0, 0.0, 100.0, 50.0);
        let shape2 = Shape::new_rectangle(50.0, 25.0, 150.0, 75.0);

        let bounds = UnifiedBounds::from_shapes(&[shape1, shape2]).unwrap();

        assert!((bounds.min.x - 0.0).abs() < 0.001);
        assert!((bounds.min.y - 0.0).abs() < 0.001);
        assert!((bounds.max.x - 200.0).abs() < 0.001);
        assert!((bounds.max.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_unified_bounds_from_empty() {
        let bounds = UnifiedBounds::from_shapes(&[]);
        assert!(bounds.is_none());
    }

    #[test]
    fn test_unified_bounds_aspect_ratio() {
        let bounds = UnifiedBounds {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(200.0, 100.0),
            center: Vec2::new(100.0, 50.0),
            width: 200.0,
            height: 100.0,
        };

        assert!((bounds.aspect_ratio() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_handle_cache_hit_test() {
        let mut cache = HandleCache::new();

        let handles = vec![
            SelectionHandle::new(HandleType::ResizeNorthWest, Vec2::new(0.0, 0.0), 10.0),
            SelectionHandle::new(HandleType::ResizeSouthEast, Vec2::new(100.0, 100.0), 10.0),
        ];

        cache.update(&handles);

        assert_eq!(
            cache.hit_test(Vec2::new(0.0, 0.0)),
            Some(HandleType::ResizeNorthWest)
        );
        assert_eq!(
            cache.hit_test(Vec2::new(100.0, 100.0)),
            Some(HandleType::ResizeSouthEast)
        );
        assert_eq!(cache.hit_test(Vec2::new(50.0, 50.0)), None);
    }

    #[test]
    fn test_handle_cache_invalidation() {
        let mut cache = HandleCache::new();
        let handles = vec![SelectionHandle::new(
            HandleType::ResizeNorthWest,
            Vec2::new(0.0, 0.0),
            10.0,
        )];

        cache.update(&handles);
        assert!(cache.is_valid(Duration::from_secs(1)));

        cache.invalidate();
        assert!(!cache.is_valid(Duration::from_secs(1)));
    }

    #[test]
    fn test_calculate_handles_single_entity() {
        let manager = SelectionHandleManager::new();
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 50.0);
        let unified_bounds = UnifiedBounds::from_shapes(&[shape]).unwrap();
        let handles = manager.calculate_handles_from_bounds(unified_bounds);

        assert_eq!(handles.len(), 9); // 8 resize + 1 rotation

        // Check corner handles
        assert_eq!(handles[0].handle_type, HandleType::ResizeNorthWest);
        assert_eq!(handles[1].handle_type, HandleType::ResizeNorthEast);
        assert_eq!(handles[2].handle_type, HandleType::ResizeSouthEast);
        assert_eq!(handles[3].handle_type, HandleType::ResizeSouthWest);

        // Check edge handles
        assert_eq!(handles[4].handle_type, HandleType::ResizeNorth);
        assert_eq!(handles[5].handle_type, HandleType::ResizeEast);
        assert_eq!(handles[6].handle_type, HandleType::ResizeSouth);
        assert_eq!(handles[7].handle_type, HandleType::ResizeWest);

        // Check rotation handle
        assert_eq!(handles[8].handle_type, HandleType::Rotate);
    }

    #[test]
    fn test_hit_test_returns_correct_handle() {
        let mut manager = SelectionHandleManager::new();
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);
        let unified_bounds = UnifiedBounds::from_shapes(&[shape]).unwrap();
        let handles = manager.calculate_handles_from_bounds(unified_bounds);
        manager.update_handles(&handles);

        // NorthWest handle is at (0,0) with size 8.0, so bounds are (-4,-4) to (4,4)
        // Point (2, 2) is inside the NorthWest handle bounds
        assert_eq!(
            manager.hit_test(Vec2::new(2.0, 2.0)),
            Some(HandleType::ResizeNorthWest)
        );
    }

    #[test]
    fn test_rotation_handle_position() {
        let manager = SelectionHandleManager::new();
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);
        let unified_bounds = UnifiedBounds::from_shapes(&[shape]).unwrap();
        let handles = manager.calculate_handles_from_bounds(unified_bounds);

        let rotation_handle = handles
            .iter()
            .find(|h| h.handle_type == HandleType::Rotate)
            .unwrap();

        // Rotation handle should be above the selection (y < 0)
        assert!(rotation_handle.position.y < 0.0);
        // X should be at center (50)
        assert!((rotation_handle.position.x - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_manager_set_handle_size() {
        let mut manager = SelectionHandleManager::new();
        manager.set_handle_size(12.0);
        assert_eq!(manager.handle_size(), 12.0);
    }

    #[test]
    fn test_manager_update_handles() {
        let mut manager = SelectionHandleManager::new();
        let handles = vec![
            SelectionHandle::new(HandleType::ResizeNorthWest, Vec2::new(0.0, 0.0), 8.0),
            SelectionHandle::new(HandleType::Rotate, Vec2::new(50.0, -20.0), 8.0),
        ];

        manager.update_handles(&handles);

        assert_eq!(manager.handles().len(), 2);
        assert_eq!(
            manager.hit_test(Vec2::new(0.0, 0.0)),
            Some(HandleType::ResizeNorthWest)
        );
    }

    #[test]
    fn test_empty_entities() {
        let manager = SelectionHandleManager::new();
        let handles = manager.calculate_handles_for_entities(&[]);
        assert!(handles.is_empty());
    }
}
