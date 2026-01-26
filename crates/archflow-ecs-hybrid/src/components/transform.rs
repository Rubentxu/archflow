//! ECS Transform Component
//!
//! This module provides the Transform component for ECS entities,
//! representing position, rotation, and scale in 2D space.

use archflow_records::{Bounds, Record, RecordId};
use bevy_ecs::prelude::*;
use glam::Vec2;

/// Transform component for ECS entities.
///
/// Represents the position, rotation, and scale of an entity
/// in 2D space. Used for rendering and spatial queries.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position in 2D space
    pub position: Vec2,

    /// Rotation in radians
    pub rotation: f32,

    /// Scale factors for X and Y axes
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    /// Creates a new Transform with the given position.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    #[inline]
    pub fn from_translation(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            ..Default::default()
        }
    }

    /// Creates a Transform from a Record.
    ///
    /// Extracts position from the Record's bounds if available.
    ///
    /// # Arguments
    ///
    /// * `record` - Reference to the Record
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_ecs_hybrid::Transform;
    /// use archflow_records::{Record, RecordId, Bounds};
    /// use std::str::FromStr;
    ///
    /// #[derive(Debug, Clone)]
    /// struct TestRecord {
    ///     id: RecordId,
    ///     bounds: Option<Bounds>,
    /// }
    ///
    /// impl Record for TestRecord {
    ///     fn id(&self) -> &RecordId { &self.id }
    ///     fn type_name(&self) -> &'static str { "TestRecord" }
    ///     fn bounds(&self) -> Option<Bounds> { self.bounds.clone() }
    /// }
    ///
    /// let id = RecordId::from_str("test_1234567890").unwrap();
    /// let bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);
    /// let record = TestRecord { id, bounds: Some(bounds) };
    ///
    /// let transform = Transform::from_record(&record);
    /// assert_eq!(transform.position.x, 150.0);
    /// assert_eq!(transform.position.y, 150.0);
    /// ```
    pub fn from_record<R: Record>(record: &R) -> Self {
        let position = record
            .bounds()
            .map(|b| {
                let center = b.center();
                Vec2::new(center.0 as f32, center.1 as f32)
            })
            .unwrap_or(Vec2::ZERO);

        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Converts this Transform to a 4x4 homogeneous transformation matrix.
    ///
    /// Useful for passing to rendering systems.
    ///
    /// # Returns
    ///
    /// A `glam::Mat4` representing this transform
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_ecs_hybrid::Transform;
    ///
    /// let transform = Transform::from_translation(100.0, 200.0);
    /// let matrix = transform.to_mat4();
    ///
    /// // Check translation is applied (via w_axis())
    /// let translation = matrix.w_axis();
    /// assert_eq!(translation.x, 100.0);
    /// assert_eq!(translation.y, 200.0);
    /// ```
    pub fn to_mat4(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(glam::Vec3::new(self.position.x, self.position.y, 0.0))
            * glam::Mat4::from_rotation_z(self.rotation)
            * glam::Mat4::from_scale(glam::Vec3::new(self.scale.x, self.scale.y, 1.0))
    }

    /// Translates this transform by the given offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Translation vector
    #[inline]
    pub fn translate(&mut self, offset: Vec2) {
        self.position += offset;
    }

    /// Rotates this transform by the given angle (in radians).
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// Scales this transform by the given factors.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale factors
    #[inline]
    pub fn scale_by(&mut self, scale: Vec2) {
        self.scale *= scale;
    }
}

/// Bundle for spawning entities with Transform component.
#[derive(Bundle, Clone, Debug)]
pub struct TransformBundle {
    /// Transform component
    pub transform: Transform,
}

impl Default for TransformBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

impl TransformBundle {
    /// Creates a TransformBundle from a Record.
    ///
    /// # Arguments
    ///
    /// * `record` - Reference to the Record
    pub fn from_record<R: Record>(record: &R) -> Self {
        Self {
            transform: Transform::from_record(record),
        }
    }

    /// Creates a TransformBundle with the given position.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    #[inline]
    pub fn from_translation(x: f32, y: f32) -> Self {
        Self {
            transform: Transform::from_translation(x, y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_records::{Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq)]
    struct MockRecord {
        id: RecordId,
        bounds: Option<Bounds>,
    }

    impl Record for MockRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }
        fn type_name(&self) -> &'static str {
            "MockRecord"
        }
        fn bounds(&self) -> Option<Bounds> {
            self.bounds.clone()
        }
    }

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, Vec2::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_from_translation() {
        let transform = Transform::from_translation(100.0, 200.0);
        assert_eq!(transform.position, Vec2::new(100.0, 200.0));
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_from_record() {
        let id = RecordId::from_str("record_test_001").unwrap();
        let bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);
        let record = MockRecord {
            id,
            bounds: Some(bounds),
        };

        let transform = Transform::from_record(&record);
        assert_eq!(transform.position.x, 150.0);
        assert_eq!(transform.position.y, 150.0);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_from_record_no_bounds() {
        let id = RecordId::from_str("record_test_002").unwrap();
        let record = MockRecord { id, bounds: None };

        let transform = Transform::from_record(&record);
        assert_eq!(transform.position, Vec2::ZERO);
    }

    #[test]
    fn test_transform_translate() {
        let mut transform = Transform::from_translation(100.0, 200.0);
        transform.translate(Vec2::new(10.0, 20.0));
        assert_eq!(transform.position, Vec2::new(110.0, 220.0));
    }

    #[test]
    fn test_transform_rotate() {
        let mut transform = Transform::default();
        transform.rotate(std::f32::consts::PI / 2.0);
        assert!((transform.rotation - std::f32::consts::PI / 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_transform_scale() {
        let mut transform = Transform::default();
        transform.scale_by(Vec2::new(2.0, 3.0));
        assert_eq!(transform.scale, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_transform_bundle_default() {
        let bundle = TransformBundle::default();
        assert_eq!(bundle.transform, Transform::default());
    }

    #[test]
    fn test_transform_bundle_from_record() {
        let id = RecordId::from_str("bundle_test_001").unwrap();
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let record = MockRecord {
            id,
            bounds: Some(bounds),
        };

        let bundle = TransformBundle::from_record(&record);
        assert_eq!(bundle.transform.position, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_transform_bundle_from_translation() {
        let bundle = TransformBundle::from_translation(150.0, 250.0);
        assert_eq!(bundle.transform.position, Vec2::new(150.0, 250.0));
    }

    #[test]
    fn test_transform_equality() {
        let t1 = Transform::from_translation(100.0, 200.0);
        let t2 = Transform::from_translation(100.0, 200.0);
        let t3 = Transform::from_translation(100.0, 201.0);

        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_transform_clone() {
        let t1 = Transform {
            position: Vec2::new(100.0, 200.0),
            rotation: std::f32::consts::PI,
            scale: Vec2::new(2.0, 3.0),
        };
        let t2 = t1.clone();

        assert_eq!(t1.position, t2.position);
        assert!((t1.rotation - t2.rotation).abs() < 1e-6);
        assert_eq!(t1.scale, t2.scale);
    }
}
