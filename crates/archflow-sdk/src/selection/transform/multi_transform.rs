//! MultiTransform - Multi-entity transformation with preserved relative positions
//!
//! This module provides transformation of multiple entities while preserving
//! their relative positions. Essential for group transformations.

use crate::selection::transform::{
    ResizeOperation, ResizeResult, RotationAngle, RotationOperation, RotationResult,
};
use crate::selection::{HandleType, UnifiedBounds};
use archflow_core::{EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Relative transform data for an entity within a multi-entity selection
#[derive(Debug, Clone)]
pub struct EntityTransform {
    /// Entity ID
    pub entity_id: EntityId,
    /// Original bounds
    pub bounds: (Vec2, Vec2),
    /// Center of the entity
    pub center: Vec2,
    /// Offset from unified center (preserved during transform)
    pub offset: Vec2,
    /// Original rotation angle
    pub rotation: f32,
}

impl EntityTransform {
    /// Calculate the offset from unified center
    #[inline]
    pub fn from_unified(unified_center: Vec2, bounds: (Vec2, Vec2), rotation: f32) -> Self {
        let center = Vec2::new(
            (bounds.0.x + bounds.1.x) / 2.0,
            (bounds.0.y + bounds.1.y) / 2.0,
        );

        Self {
            entity_id: EntityId::new(), // Will be set by caller
            bounds,
            center,
            offset: center - unified_center,
            rotation,
        }
    }
}

/// Result of a multi-entity transform operation
#[derive(Debug, Clone, Default)]
pub struct MultiTransformResult {
    /// Mapping of entity ID to new bounds
    pub entity_bounds: HashMap<EntityId, (Vec2, Vec2)>,
    /// Mapping of entity ID to new rotation
    pub entity_rotations: HashMap<EntityId, f32>,
    /// Whether any entity was clamped or snapped
    pub was_modified: bool,
    /// Unified center after transform
    pub unified_center: Vec2,
}

impl MultiTransformResult {
    /// Create an empty result
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transformed entity
    pub fn add_entity(&mut self, entity_id: EntityId, bounds: (Vec2, Vec2), rotation: f32) {
        self.entity_bounds.insert(entity_id, bounds);
        self.entity_rotations.insert(entity_id, rotation);
        self.was_modified = true;
    }

    /// Get new bounds for an entity
    #[inline]
    pub fn get_bounds(&self, entity_id: &EntityId) -> Option<(Vec2, Vec2)> {
        self.entity_bounds.get(entity_id).copied()
    }

    /// Get new rotation for an entity
    #[inline]
    pub fn get_rotation(&self, entity_id: &EntityId) -> Option<f32> {
        self.entity_rotations.get(entity_id).copied()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_bounds.is_empty()
    }

    /// Get the count of transformed entities
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_bounds.len()
    }
}

/// Multi-entity transformation manager
#[derive(Debug, Clone)]
pub struct MultiTransform {
    /// Entity transforms with their offsets
    transforms: Vec<EntityTransform>,
    /// Unified center of all entities
    unified_center: Vec2,
    /// Original unified bounds
    unified_bounds: UnifiedBounds,
}

impl MultiTransform {
    /// Create from entity data
    ///
    /// # Arguments
    ///
    /// * `entities` - Mapping of EntityId to (bounds, rotation)
    pub fn from_entities(entities: &HashMap<EntityId, ((Vec2, Vec2), f32)>) -> Option<Self> {
        if entities.is_empty() {
            return None;
        }

        // Calculate unified bounds
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for ((min, max), _) in entities.values() {
            min_x = min_x.min(min.x);
            min_y = min_y.min(min.y);
            max_x = max_x.max(max.x);
            max_y = max_y.max(max.y);
        }

        let unified_center = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);

        let unified_bounds = UnifiedBounds {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
            center: unified_center,
            width: max_x - min_x,
            height: max_y - min_y,
        };

        // Calculate individual transforms with offsets
        let transforms: Vec<EntityTransform> = entities
            .iter()
            .map(|(id, ((min, max), rotation))| {
                let center = Vec2::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0);

                EntityTransform {
                    entity_id: *id,
                    bounds: (*min, *max),
                    center,
                    offset: center - unified_center,
                    rotation: *rotation,
                }
            })
            .collect();

        Some(Self {
            transforms,
            unified_center,
            unified_bounds,
        })
    }

    /// Get unified bounds
    #[inline]
    pub fn unified_bounds(&self) -> UnifiedBounds {
        self.unified_bounds
    }

    /// Get unified center
    #[inline]
    pub fn unified_center(&self) -> Vec2 {
        self.unified_center
    }

    /// Get number of entities
    #[inline]
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Apply resize to all entities
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle being dragged
    /// * `resize_result` - Result from ResizeOperation
    ///
    /// # Returns
    ///
    /// New bounds and rotations for all entities
    pub fn apply_resize(
        &self,
        _handle: HandleType,
        resize_result: ResizeResult,
    ) -> MultiTransformResult {
        let mut result = MultiTransformResult::new();
        result.unified_center = resize_result.center();

        // Calculate scale factors
        let original_width = self.unified_bounds.width;
        let original_height = self.unified_bounds.height;
        let new_width = resize_result.width();
        let new_height = resize_result.height();

        let scale_x = if original_width > 0.0 {
            new_width / original_width
        } else {
            1.0
        };
        let scale_y = if original_height > 0.0 {
            new_height / original_height
        } else {
            1.0
        };

        for transform in &self.transforms {
            // Calculate new center maintaining relative offset
            let scaled_offset =
                Vec2::new(transform.offset.x * scale_x, transform.offset.y * scale_y);
            let new_center = resize_result.center() + scaled_offset;

            // Calculate new bounds
            let entity_width = transform.bounds.1.x - transform.bounds.0.x;
            let entity_height = transform.bounds.1.y - transform.bounds.0.y;

            let new_min = Vec2::new(
                new_center.x - (entity_width * scale_x) / 2.0,
                new_center.y - (entity_height * scale_y) / 2.0,
            );
            let new_max = Vec2::new(
                new_center.x + (entity_width * scale_x) / 2.0,
                new_center.y + (entity_height * scale_y) / 2.0,
            );

            result.add_entity(transform.entity_id, (new_min, new_max), transform.rotation);
        }

        result
    }

    /// Apply rotation to all entities
    ///
    /// # Arguments
    ///
    /// * `rotation_result` - Result from RotationOperation
    ///
    /// # Returns
    ///
    /// New bounds and rotations for all entities
    pub fn apply_rotation(&self, rotation_result: RotationResult) -> MultiTransformResult {
        let mut result = MultiTransformResult::new();
        result.unified_center = self.unified_center;

        let rotation_angle = rotation_result.angle;

        for transform in &self.transforms {
            // Rotate the offset around unified center
            let rotated_offset = rotate_offset(transform.offset, rotation_angle);

            // Calculate new center
            let new_center = self.unified_center + rotated_offset;

            // Calculate new bounds (rotated around new center)
            let (new_min, new_max) =
                rotate_entity_bounds(transform.bounds, new_center, rotation_angle);

            // New rotation is cumulative
            let new_rotation = transform.rotation + rotation_result.delta;

            result.add_entity(transform.entity_id, (new_min, new_max), new_rotation);
        }

        result
    }

    /// Apply combined resize and rotation
    pub fn apply_combined(
        &self,
        resize_result: Option<ResizeResult>,
        rotation_result: Option<RotationResult>,
    ) -> MultiTransformResult {
        // First apply rotation, then resize (or vice versa depending on order)
        // For simplicity, we apply resize first, then rotation

        let mut result = if let Some(resize) = resize_result {
            self.apply_resize(HandleType::ResizeSouthEast, resize)
        } else {
            MultiTransformResult::new()
        };

        if let Some(rotation) = rotation_result {
            // For combined transforms, we need to re-calculate from original
            // This is a simplified implementation
            result = self.apply_rotation(rotation);

            // If resize was also applied, we need to apply it too
            if let Some(resize) = resize_result {
                result = self.apply_resize(HandleType::ResizeSouthEast, resize);
            }
        }

        result
    }
}

/// Rotate an offset vector by an angle
#[inline]
fn rotate_offset(offset: Vec2, angle: RotationAngle) -> Vec2 {
    let cos = (-angle.to_radians()).cos();
    let sin = (-angle.to_radians()).sin();

    Vec2::new(
        offset.x * cos - offset.y * sin,
        offset.x * sin + offset.y * cos,
    )
}

/// Rotate entity bounds around its center
fn rotate_entity_bounds(bounds: (Vec2, Vec2), center: Vec2, angle: RotationAngle) -> (Vec2, Vec2) {
    let corners = [
        bounds.0,                          // top-left
        Vec2::new(bounds.0.x, bounds.1.y), // bottom-left
        Vec2::new(bounds.1.x, bounds.0.y), // top-right
        bounds.1,                          // bottom-right
    ];

    let rotated: Vec<Vec2> = corners
        .into_iter()
        .map(|p| {
            let dx = p.x - center.x;
            let dy = p.y - center.y;
            let cos = (-angle.to_radians()).cos();
            let sin = (-angle.to_radians()).sin();

            Vec2::new(
                center.x + dx * cos - dy * sin,
                center.y + dx * sin + dy * cos,
            )
        })
        .collect();

    let min_x = rotated.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let min_y = rotated.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let max_x = rotated.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let max_y = rotated.iter().map(|p| p.y).fold(f32::MIN, f32::max);

    (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

/// Batch update shapes with transform result
pub fn apply_transform_to_canvas(
    canvas: &mut dyn CanvasAdapter,
    result: &MultiTransformResult,
) -> Result<(), Box<dyn std::error::Error>> {
    for (entity_id, new_bounds) in &result.entity_bounds {
        let new_rotation = result
            .entity_rotations
            .get(entity_id)
            .copied()
            .unwrap_or(0.0);

        let width = new_bounds.1.x - new_bounds.0.x;
        let height = new_bounds.1.y - new_bounds.0.y;
        let x = new_bounds.0.x;
        let y = new_bounds.0.y;

        canvas.update_shape(*entity_id, x, y, width, height, new_rotation)?;
    }

    Ok(())
}

/// Adapter trait for canvas operations
pub trait CanvasAdapter {
    fn update_shape(
        &mut self,
        entity_id: EntityId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::EntityId;

    fn create_test_entities() -> HashMap<EntityId, ((Vec2, Vec2), f32)> {
        let mut entities = HashMap::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        entities.insert(
            id1,
            ((Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0)), 0.0),
        );
        entities.insert(
            id2,
            ((Vec2::new(200.0, 200.0), Vec2::new(250.0, 250.0)), 0.0),
        );

        entities
    }

    #[test]
    fn test_from_entities() {
        let entities = create_test_entities();
        let multi = MultiTransform::from_entities(&entities).unwrap();

        assert_eq!(multi.len(), 2);
        assert!((multi.unified_center().x - 175.0).abs() < 0.1);
        assert!((multi.unified_center().y - 175.0).abs() < 0.1);
    }

    #[test]
    fn test_from_empty_entities() {
        let entities: HashMap<EntityId, ((Vec2, Vec2), f32)> = HashMap::new();
        let multi = MultiTransform::from_entities(&entities);

        assert!(multi.is_none());
    }

    #[test]
    fn test_apply_resize() {
        let entities = create_test_entities();
        let multi = MultiTransform::from_entities(&entities).unwrap();

        // Create a resize result (double the size)
        let resize_result = ResizeResult {
            min: Vec2::new(75.0, 75.0),
            max: Vec2::new(275.0, 275.0),
            delta: Vec2::new(50.0, 50.0),
            was_clamped: false,
        };

        let result = multi.apply_resize(HandleType::ResizeSouthEast, resize_result);

        assert_eq!(result.len(), 2);
        assert!(result.was_modified);

        // Entity 1 should be centered around new unified center
        let bounds1 = result.get_bounds(&entities.keys().next().unwrap()).unwrap();
        assert!((bounds1.0.x - 75.0).abs() < 10.0);
    }

    #[test]
    fn test_apply_rotation() {
        let entities = create_test_entities();
        let multi = MultiTransform::from_entities(&entities).unwrap();

        let rotation_result = RotationResult {
            angle: RotationAngle::new(90.0),
            delta: 90.0,
            was_snapped: false,
            guide_point: Vec2::new(175.0, 145.0),
        };

        let result = multi.apply_rotation(rotation_result);

        assert_eq!(result.len(), 2);
        assert!(result.was_modified);

        // Both entities should have 90° rotation
        for (id, _) in &entities {
            let rotation = result.get_rotation(id).unwrap();
            assert!((rotation - 90.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_preserved_relative_positions() {
        let entities = create_test_entities();
        let multi = MultiTransform::from_entities(&entities).unwrap();

        // Original positions relative to center
        let id1_offset = multi.transforms[0].offset;
        let id2_offset = multi.transforms[1].offset;

        // Apply rotation
        let rotation_result = RotationResult {
            angle: RotationAngle::new(90.0),
            delta: 90.0,
            was_snapped: false,
            guide_point: Vec2::new(175.0, 145.0),
        };

        let result = multi.apply_rotation(rotation_result);

        // Get new centers
        let bounds1 = result.get_bounds(&entities.keys().next().unwrap()).unwrap();
        let bounds2 = result.get_bounds(&entities.keys().nth(1).unwrap()).unwrap();

        let new_center1 = Vec2::new(
            (bounds1.0.x + bounds1.1.x) / 2.0,
            (bounds1.0.y + bounds1.1.y) / 2.0,
        );
        let new_center2 = Vec2::new(
            (bounds2.0.x + bounds2.1.x) / 2.0,
            (bounds2.0.y + bounds2.1.y) / 2.0,
        );

        // Distance between centers should be preserved
        let original_distance = (id2_offset - id1_offset).length();
        let new_distance = (new_center2 - new_center1).length();

        assert!((original_distance - new_distance).abs() < 0.1);
    }

    #[test]
    fn test_multi_transform_result() {
        let mut result = MultiTransformResult::new();

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);

        let id = EntityId::new();
        result.add_entity(id, (Vec2::ZERO, Vec2::new(100.0, 100.0)), 45.0);

        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert!(result.get_bounds(&id).is_some());
        assert!((result.get_rotation(&id).unwrap() - 45.0).abs() < 0.1);
    }

    #[test]
    fn test_rotate_offset() {
        let offset = Vec2::new(50.0, 0.0); // 50px to the right
        let angle = RotationAngle::new(90.0);

        let rotated = rotate_offset(offset, angle);

        // Should be 50px up
        assert!((rotated.x - 0.0).abs() < 0.1);
        assert!((rotated.y - 50.0).abs() < 0.1);
    }
}
