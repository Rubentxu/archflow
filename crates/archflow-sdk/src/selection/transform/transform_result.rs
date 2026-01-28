//! TransformResult - Unified result type for all transformation operations
//!
//! This module provides a unified result type that can represent the outcome
//! of any transformation operation, making it easier to handle different
//! types of transforms in a uniform way.

use crate::selection::transform::{
    MultiTransformResult, ResizeResult, RotationAngle, RotationResult,
};
use crate::selection::{HandleType, UnifiedBounds};
use archflow_core::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of transformation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransformType {
    /// Resize operation
    Resize,
    /// Rotation operation
    Rotate,
    /// Combined resize and rotation
    Combined,
    /// Multi-entity transform
    Multi,
}

/// Unified result type for all transformation operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformResult {
    /// Type of transformation
    pub transform_type: TransformType,
    /// Handle that initiated the transform (if applicable)
    pub handle: Option<HandleType>,
    /// Entity IDs affected
    pub entity_ids: Vec<EntityId>,
    /// Original unified bounds (before transform)
    pub original_bounds: Option<UnifiedBounds>,
    /// New bounds for each entity
    pub entity_bounds: HashMap<EntityId, (archflow_core::Vec2, archflow_core::Vec2)>,
    /// New rotation for each entity
    pub entity_rotations: HashMap<EntityId, f32>,
    /// Whether any operation was clamped/snapped/modified
    pub was_modified: bool,
    /// Whether the transform was completed (vs preview)
    pub is_completed: bool,
    /// Additional metadata
    pub metadata: TransformMetadata,
}

/// Additional metadata about the transform
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransformMetadata {
    /// Total rotation applied in degrees
    pub total_rotation: f32,
    /// Whether rotation was snapped
    pub rotation_snapped: bool,
    /// Snap increment used (if any)
    pub snap_increment: f32,
    /// Whether resize maintained aspect ratio
    pub aspect_ratio_preserved: bool,
    /// Whether resize was from center
    pub from_center: bool,
    /// Whether any entity was clamped to minimum size
    pub size_clamped: bool,
    /// Delta from original center (for undo)
    pub center_delta: Option<archflow_core::Vec2>,
}

impl TransformResult {
    /// Create an empty result
    #[inline]
    pub fn new() -> Self {
        Self {
            transform_type: TransformType::Multi,
            handle: None,
            entity_ids: Vec::new(),
            original_bounds: None,
            entity_bounds: HashMap::new(),
            entity_rotations: HashMap::new(),
            was_modified: false,
            is_completed: false,
            metadata: TransformMetadata::default(),
        }
    }

    /// Create from resize result
    pub fn from_resize(
        entity_id: EntityId,
        handle: HandleType,
        original_bounds: UnifiedBounds,
        resize_result: ResizeResult,
        rotation: f32,
    ) -> Self {
        let mut entity_bounds = HashMap::new();
        entity_bounds.insert(entity_id, (resize_result.min, resize_result.max));

        let mut entity_rotations = HashMap::new();
        entity_rotations.insert(entity_id, rotation);

        Self {
            transform_type: TransformType::Resize,
            handle: Some(handle),
            entity_ids: vec![entity_id],
            original_bounds: Some(original_bounds),
            entity_bounds,
            entity_rotations,
            was_modified: resize_result.was_clamped,
            is_completed: true,
            metadata: TransformMetadata {
                total_rotation: 0.0,
                rotation_snapped: false,
                snap_increment: 0.0,
                aspect_ratio_preserved: false,
                from_center: false,
                size_clamped: resize_result.was_clamped,
                center_delta: Some(resize_result.delta),
            },
        }
    }

    /// Create from rotation result
    pub fn from_rotation(
        entity_id: EntityId,
        original_bounds: UnifiedBounds,
        rotation_result: RotationResult,
        _original_rotation: f32,
    ) -> Self {
        let mut entity_bounds = HashMap::new();
        entity_bounds.insert(entity_id, (original_bounds.min, original_bounds.max));

        let mut entity_rotations = HashMap::new();
        entity_rotations.insert(entity_id, rotation_result.angle.to_degrees());

        Self {
            transform_type: TransformType::Rotate,
            handle: Some(HandleType::Rotate),
            entity_ids: vec![entity_id],
            original_bounds: Some(original_bounds),
            entity_bounds,
            entity_rotations,
            was_modified: rotation_result.was_snapped,
            is_completed: true,
            metadata: TransformMetadata {
                total_rotation: rotation_result.delta,
                rotation_snapped: rotation_result.was_snapped,
                snap_increment: if rotation_result.was_snapped {
                    DEFAULT_SNAP_INCREMENT
                } else {
                    0.0
                },
                aspect_ratio_preserved: false,
                from_center: false,
                size_clamped: false,
                center_delta: None,
            },
        }
    }

    /// Create from multi-transform result
    pub fn from_multi(
        entity_ids: Vec<EntityId>,
        original_bounds: UnifiedBounds,
        multi_result: MultiTransformResult,
    ) -> Self {
        Self {
            transform_type: TransformType::Multi,
            handle: None,
            entity_ids,
            original_bounds: Some(original_bounds),
            entity_bounds: multi_result.entity_bounds,
            entity_rotations: multi_result.entity_rotations,
            was_modified: multi_result.was_modified,
            is_completed: true,
            metadata: TransformMetadata::default(),
        }
    }

    /// Add an entity to the result
    pub fn add_entity(
        &mut self,
        entity_id: EntityId,
        bounds: (archflow_core::Vec2, archflow_core::Vec2),
        rotation: f32,
    ) {
        self.entity_ids.push(entity_id);
        self.entity_bounds.insert(entity_id, bounds);
        self.entity_rotations.insert(entity_id, rotation);
        self.was_modified = true;
    }

    /// Get bounds for an entity
    #[inline]
    pub fn get_bounds(
        &self,
        entity_id: &EntityId,
    ) -> Option<(archflow_core::Vec2, archflow_core::Vec2)> {
        self.entity_bounds.get(entity_id).copied()
    }

    /// Get rotation for an entity
    #[inline]
    pub fn get_rotation(&self, entity_id: &EntityId) -> Option<f32> {
        self.entity_rotations.get(entity_id).copied()
    }

    /// Check if result contains an entity
    #[inline]
    pub fn contains(&self, entity_id: &EntityId) -> bool {
        self.entity_ids.contains(entity_id)
    }

    /// Get the number of affected entities
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_ids.is_empty()
    }

    /// Check if transform is completed (vs preview)
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    /// Mark as preview (not completed)
    pub fn as_preview(&mut self) {
        self.is_completed = false;
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.is_completed = true;
    }

    /// Merge with another result
    pub fn merge(&mut self, other: TransformResult) {
        for entity_id in other.entity_ids {
            if !self.entity_ids.contains(&entity_id) {
                self.entity_ids.push(entity_id);
            }
            if let Some(bounds) = other.entity_bounds.get(&entity_id) {
                self.entity_bounds.insert(entity_id, *bounds);
            }
            if let Some(rotation) = other.entity_rotations.get(&entity_id) {
                self.entity_rotations.insert(entity_id, *rotation);
            }
        }

        self.was_modified = self.was_modified || other.was_modified;
        self.is_completed = self.is_completed && other.is_completed;
    }
}

impl Default for TransformResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Default snap increment for rotation
const DEFAULT_SNAP_INCREMENT: f32 = 15.0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::{HandleType, UnifiedBounds};
    use archflow_core::{EntityId, Vec2};

    fn create_test_bounds() -> UnifiedBounds {
        UnifiedBounds {
            min: Vec2::new(100.0, 100.0),
            max: Vec2::new(200.0, 200.0),
            center: Vec2::new(150.0, 150.0),
            width: 100.0,
            height: 100.0,
        }
    }

    #[test]
    fn test_new_result() {
        let result = TransformResult::new();

        assert!(result.is_empty());
        assert!(!result.is_completed());
        assert!(!result.was_modified);
    }

    #[test]
    fn test_from_resize() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let resize_result = ResizeResult {
            min: Vec2::new(100.0, 100.0),
            max: Vec2::new(250.0, 250.0),
            delta: Vec2::new(50.0, 50.0),
            was_clamped: false,
        };

        let result = TransformResult::from_resize(
            entity_id,
            HandleType::ResizeSouthEast,
            bounds,
            resize_result,
            0.0,
        );

        assert_eq!(result.transform_type, TransformType::Resize);
        assert_eq!(result.handle, Some(HandleType::ResizeSouthEast));
        assert_eq!(result.len(), 1);
        assert!(result.contains(&entity_id));

        let new_bounds = result.get_bounds(&entity_id).unwrap();
        assert_eq!(new_bounds.1.x, 250.0);
    }

    #[test]
    fn test_from_rotation() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let rotation_result = RotationResult {
            angle: RotationAngle::new(45.0),
            delta: 45.0,
            was_snapped: true,
            guide_point: Vec2::new(175.0, 120.0),
        };

        let result = TransformResult::from_rotation(entity_id, bounds, rotation_result, 0.0);

        assert_eq!(result.transform_type, TransformType::Rotate);
        assert_eq!(result.handle, Some(HandleType::Rotate));
        assert!(result.metadata.rotation_snapped);

        let rotation = result.get_rotation(&entity_id).unwrap();
        assert!((rotation - 45.0).abs() < 0.1);
    }

    #[test]
    fn test_add_entity() {
        let mut result = TransformResult::new();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        result.add_entity(id1, (Vec2::ZERO, Vec2::new(100.0, 100.0)), 0.0);
        result.add_entity(
            id2,
            (Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0)),
            45.0,
        );

        assert_eq!(result.len(), 2);
        assert!(result.contains(&id1));
        assert!(result.contains(&id2));
    }

    #[test]
    fn test_merge_results() {
        let mut result1 = TransformResult::new();
        let id1 = EntityId::new();
        result1.add_entity(id1, (Vec2::ZERO, Vec2::new(100.0, 100.0)), 0.0);

        let mut result2 = TransformResult::new();
        let id2 = EntityId::new();
        result2.add_entity(
            id2,
            (Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0)),
            45.0,
        );

        result1.merge(result2);

        assert_eq!(result1.len(), 2);
        assert!(result1.contains(&id1));
        assert!(result1.contains(&id2));
    }

    #[test]
    fn test_preview_vs_completed() {
        let mut result = TransformResult::new();
        assert!(!result.is_completed);

        result.as_preview();
        assert!(!result.is_completed);

        result.complete();
        assert!(result.is_completed);
    }

    #[test]
    fn test_metadata_fields() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let resize_result = ResizeResult {
            min: Vec2::new(100.0, 100.0),
            max: Vec2::new(250.0, 250.0),
            delta: Vec2::new(50.0, 50.0),
            was_clamped: true,
        };

        let result = TransformResult::from_resize(
            entity_id,
            HandleType::ResizeSouthEast,
            bounds,
            resize_result,
            0.0,
        );

        assert!(result.metadata.size_clamped);
        assert!(result.metadata.center_delta.is_some());
    }
}
