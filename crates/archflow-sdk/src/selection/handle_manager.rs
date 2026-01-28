//! Selection Handle Manager - Handle rendering and hit testing with caching

use crate::canvas::Shape;
use crate::tools::CursorType;
use archflow_core::{EntityId, Rect, Vec2};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Handle type for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleType {
    ResizeNorthWest,
    ResizeNorth,
    ResizeNorthEast,
    ResizeEast,
    ResizeSouthEast,
    ResizeSouth,
    ResizeSouthWest,
    ResizeWest,
    Rotate,
}

impl HandleType {
    pub fn cursor(&self) -> CursorType {
        match self {
            HandleType::ResizeNorthWest | HandleType::ResizeSouthEast => CursorType::ResizeNWSE,
            HandleType::ResizeNorthEast | HandleType::ResizeSouthWest => CursorType::ResizeNESW,
            HandleType::ResizeNorth | HandleType::ResizeSouth => CursorType::ResizeNS,
            HandleType::ResizeEast | HandleType::ResizeWest => CursorType::ResizeEW,
            HandleType::Rotate => CursorType::Grab,
        }
    }
}

/// A single selection handle
#[derive(Debug, Clone)]
pub struct SelectionHandle {
    pub handle_type: HandleType,
    pub position: Vec2,
    pub size: f32,
    pub cursor: CursorType,
}

impl SelectionHandle {
    pub fn new(handle_type: HandleType, position: Vec2, size: f32) -> Self {
        Self {
            handle_type,
            position,
            size,
            cursor: handle_type.cursor(),
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_center_size(self.position, Vec2::splat(self.size))
    }
}

/// Unified bounding box for multi-entity selection
#[derive(Debug, Clone, Copy)]
pub struct UnifiedBounds {
    pub min: Vec2,
    pub max: Vec2,
    pub center: Vec2,
    pub width: f32,
    pub height: f32,
}

impl UnifiedBounds {
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
}

/// Cached handle data for hit testing
#[derive(Debug, Clone)]
pub struct HandleCache {
    handle_bounds: HashMap<HandleType, Rect>,
    last_update: Instant,
    dirty: bool,
}

impl HandleCache {
    pub fn new() -> Self {
        Self {
            handle_bounds: HashMap::new(),
            last_update: Instant::now(),
            dirty: true,
        }
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
        self.handle_bounds.clear();
    }

    pub fn update(&mut self, handles: &[SelectionHandle]) {
        self.handle_bounds.clear();
        for handle in handles {
            self.handle_bounds
                .insert(handle.handle_type, handle.bounds());
        }
        self.last_update = Instant::now();
        self.dirty = false;
    }

    pub fn is_valid(&self, max_age: Duration) -> bool {
        !self.dirty && self.last_update.elapsed() < max_age
    }

    pub fn hit_test(&self, point: Vec2) -> Option<HandleType> {
        for (handle_type, bounds) in &self.handle_bounds {
            if bounds.contains(point) {
                return Some(*handle_type);
            }
        }
        None
    }
}

impl Default for HandleCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for selection handles
#[derive(Debug, Clone)]
pub struct SelectionHandleManager {
    handle_size: f32,
    cache: HandleCache,
    current_handles: Vec<SelectionHandle>,
}

impl SelectionHandleManager {
    pub fn new() -> Self {
        Self {
            handle_size: 8.0,
            cache: HandleCache::new(),
            current_handles: Vec::new(),
        }
    }

    pub fn set_handle_size(&mut self, size: f32) {
        self.handle_size = size;
        self.cache.invalidate();
    }

    pub fn calculate_handles_from_bounds(&self, bounds: UnifiedBounds) -> Vec<SelectionHandle> {
        let min = bounds.min;
        let max = bounds.max;
        let center = bounds.center;

        let rotation_pos = Vec2::new(center.x, min.y - 20.0);

        vec![
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
            SelectionHandle::new(HandleType::Rotate, rotation_pos, self.handle_size),
        ]
    }

    pub fn calculate_handles_for_entities(
        &self,
        entities: &[(EntityId, Rect)],
    ) -> Vec<SelectionHandle> {
        if entities.is_empty() {
            return Vec::new();
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

        let unified_bounds = UnifiedBounds {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
            center,
            width,
            height,
        };

        self.calculate_handles_from_bounds(unified_bounds)
    }

    pub fn hit_test(&self, point: Vec2) -> Option<HandleType> {
        self.cache.hit_test(point)
    }

    pub fn handles(&self) -> &[SelectionHandle] {
        &self.current_handles
    }
}

impl Default for SelectionHandleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform operation for resize/rotate
#[derive(Debug, Clone)]
pub struct TransformOperation {
    pub entity_id: EntityId,
    pub original_bounds: (Vec2, Vec2),
    pub current_bounds: (Vec2, Vec2),
    pub handle: HandleType,
}

impl TransformOperation {
    pub fn new_resize(entity_id: EntityId, bounds: (Vec2, Vec2), handle: HandleType) -> Self {
        Self {
            entity_id,
            original_bounds: bounds,
            current_bounds: bounds,
            handle,
        }
    }

    pub fn update_resize(&mut self, current_pos: Vec2) -> (Vec2, Vec2) {
        let (mut min, mut max) = self.original_bounds;

        match self.handle {
            HandleType::ResizeSouthEast => max = current_pos,
            HandleType::ResizeNorthWest => min = current_pos,
            HandleType::ResizeNorth => min.y = current_pos.y,
            HandleType::ResizeSouth => max.y = current_pos.y,
            HandleType::ResizeEast => max.x = current_pos.x,
            HandleType::ResizeWest => min.x = current_pos.x,
            _ => {}
        }

        if min.x > max.x {
            std::mem::swap(&mut min.x, &mut max.x);
        }
        if min.y > max.y {
            std::mem::swap(&mut min.y, &mut max.y);
        }

        self.current_bounds = (min, max);
        (min, max)
    }

    pub fn current_bounds(&self) -> (Vec2, Vec2) {
        self.current_bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_handles_single_entity() {
        let manager = SelectionHandleManager::new();
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 50.0);
        let unified_bounds = UnifiedBounds::from_shapes(&[shape]).unwrap();
        let handles = manager.calculate_handles_from_bounds(unified_bounds);
        assert_eq!(handles.len(), 9);
    }

    #[test]
    fn test_hit_test_returns_correct_handle() {
        let mut manager = SelectionHandleManager::new();
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);
        let unified_bounds = UnifiedBounds::from_shapes(&[shape]).unwrap();
        let handles = manager.calculate_handles_from_bounds(unified_bounds);
        manager.cache.update(&handles);

        // NorthWest handle is at min (0,0) with size 8.0, so bounds are (-4,-4) to (4,4)
        // Point (2, 2) is inside the NorthWest handle bounds
        assert_eq!(
            manager.hit_test(Vec2::new(2.0, 2.0)),
            Some(HandleType::ResizeNorthWest)
        );
    }

    #[test]
    fn test_resize_operation() {
        let entity_id = EntityId::new();
        let bounds = (Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let mut operation =
            TransformOperation::new_resize(entity_id, bounds, HandleType::ResizeSouthEast);
        let (min, max) = operation.update_resize(Vec2::new(150.0, 150.0));
        assert_eq!(max, Vec2::new(150.0, 150.0));
    }
}
