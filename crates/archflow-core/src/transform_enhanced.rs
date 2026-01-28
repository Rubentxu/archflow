//! Enhanced Transform - Full 2D transformation system with composition and decomposition
//!
//! This module provides a comprehensive 2D transformation system including:
//! - **Transform**: Full 3x3 matrix transform with composition
//! - **TransformDecomposition**: Decomposition into translation, rotation, scale, skew
//! - **CompactTransform**: Memory-efficient representation for storage
//!
//! # Composition Order
//!
//! Transformations are composed in the order: Scale -> Rotate -> Translate
//! This means `transform.compose(b)` applies `b` first, then `a`.

use crate::types::{Mat3, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Mul, MulAssign};

/// 2D Transform with full 3x3 matrix support
///
/// Represents a 2D affine transform that can be composed, inverted, and decomposed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// The 3x3 transformation matrix
    pub matrix: Mat3,
}

impl Transform {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            matrix: Mat3::IDENTITY,
        }
    }

    /// Create from translation
    pub fn from_translation(x: f32, y: f32) -> Self {
        Self {
            matrix: Mat3::from_translation(Vec2::new(x, y)),
        }
    }

    /// Create from translation vector
    pub fn from_translation_vec(v: Vec2) -> Self {
        Self {
            matrix: Mat3::from_translation(v),
        }
    }

    /// Create from rotation (angle in radians)
    pub fn from_rotation(angle: f32) -> Self {
        Self {
            matrix: Mat3::from_rotation(angle),
        }
    }

    /// Create from uniform scale
    pub fn from_scale(s: f32) -> Self {
        Self {
            matrix: Mat3::from_scale(Vec2::new(s, s)),
        }
    }

    /// Create from non-uniform scale
    pub fn from_scale_vec(v: Vec2) -> Self {
        Self {
            matrix: Mat3::from_scale(v),
        }
    }

    /// Compose this transform with another
    ///
    /// The other transform is applied first, then this one.
    /// Equivalent to `self * other` in matrix multiplication order.
    pub fn compose(self, other: Transform) -> Self {
        Self {
            matrix: self.matrix.mul_mat(other.matrix),
        }
    }

    /// Apply transform to a point
    pub fn transform_point(self, point: Vec2) -> Vec2 {
        self.matrix.transform_point2(point)
    }

    /// Apply transform to a vector (direction, no translation)
    pub fn transform_vector(self, vector: Vec2) -> Vec2 {
        self.matrix.transform_vector2(vector)
    }

    /// Get the inverse transform
    pub fn inverse(self) -> Option<Self> {
        self.matrix.inverse().map(|m| Self { matrix: m })
    }

    /// Decompose into translation, rotation, scale, and skew
    pub fn decompose(self) -> TransformDecomposition {
        TransformDecomposition::from_matrix(self.matrix)
    }

    /// Check if transform is identity
    pub fn is_identity(&self) -> bool {
        self.matrix == Mat3::IDENTITY
    }

    /// Check if transform is only translation
    pub fn is_translation_only(&self) -> bool {
        let m = &self.matrix;
        m.m00 == 1.0 && m.m11 == 1.0 && m.m01 == 0.0 && m.m10 == 0.0
    }

    /// Check if transform is only uniform scale
    pub fn is_uniform_scale_only(&self) -> bool {
        let m = &self.matrix;
        let translation = m.m02 == 0.0 && m.m12 == 0.0;
        let rotation = m.m01 == 0.0 && m.m10 == 0.0;
        let uniform_scale = (m.m00 - m.m11).abs() < 1e-6;
        translation && rotation && uniform_scale
    }

    /// Get translation component
    pub fn translation(&self) -> Vec2 {
        Vec2::new(self.matrix.m02, self.matrix.m12)
    }

    /// Get scale components
    pub fn scale(&self) -> Vec2 {
        let sx = Vec2::new(self.matrix.m00, self.matrix.m10).length();
        let sy = Vec2::new(self.matrix.m01, self.matrix.m11).length();
        Vec2::new(sx, sy)
    }

    /// Get rotation angle in radians
    pub fn rotation(&self) -> f32 {
        Vec2::new(self.matrix.m00, self.matrix.m10)
            .normalize()
            .y
            .atan2(Vec2::new(self.matrix.m00, self.matrix.m10).normalize().x)
    }

    /// Convert to 3x3 matrix
    pub fn to_mat3(self) -> Mat3 {
        self.matrix
    }

    /// Create from 3x3 matrix
    pub fn from_mat3(matrix: Mat3) -> Self {
        Self { matrix }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.compose(other)
    }
}

impl MulAssign for Transform {
    fn mul_assign(&mut self, other: Self) {
        *self = self.compose(other);
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transform(matrix=[{:.2}, {:.2}, {:.2}], [{:.2}, {:.2}, {:.2}], [{:.2}, {:.2}, {:.2}])",
            self.matrix.m00,
            self.matrix.m01,
            self.matrix.m02,
            self.matrix.m10,
            self.matrix.m11,
            self.matrix.m12,
            self.matrix.m20,
            self.matrix.m21,
            self.matrix.m22
        )
    }
}

/// Decomposition of a 2D transform into its components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransformDecomposition {
    /// Translation component
    pub translation: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale in X direction
    pub scale_x: f32,
    /// Scale in Y direction
    pub scale_y: f32,
    /// Skew in X direction (radians)
    pub skew_x: f32,
    /// Skew in Y direction (radians)
    pub skew_y: f32,
}

impl TransformDecomposition {
    /// Decompose a matrix into its components
    pub fn from_matrix(matrix: Mat3) -> Self {
        let m = &matrix;

        // Extract scale and rotation using polar decomposition
        let a = Vec2::new(m.m00, m.m10);
        let b = Vec2::new(m.m01, m.m11);

        let scale_x = a.length();
        let scale_y = b.length();

        // Normalize rotation matrix
        let normalized_a = if scale_x > 0.0 { a / scale_x } else { Vec2::X };
        let normalized_b = if scale_y > 0.0 { b / scale_y } else { Vec2::Y };

        // Extract rotation (determinant check for reflection)
        let det = normalized_a.x * normalized_b.y - normalized_a.y * normalized_b.x;
        let reflection = if det < 0.0 { -1.0 } else { 1.0 };

        let rotation = normalized_a.y.atan2(normalized_a.x);

        // Calculate skew (dot product unused but kept for potential future use)
        let _dot = normalized_a.dot(normalized_b);
        let skew_x = (normalized_b.y * reflection).atan2(normalized_b.x * reflection)
            - std::f32::consts::FRAC_PI_2;
        let skew_y = normalized_a.y.atan2(normalized_a.x);

        Self {
            translation: Vec2::new(m.m02, m.m12),
            rotation,
            scale_x,
            scale_y: scale_y * reflection,
            skew_x,
            skew_y: if reflection < 0.0 { -skew_y } else { skew_y },
        }
    }

    /// Create a transform from decomposition
    pub fn to_transform(self) -> Transform {
        let (sin, cos) = self.rotation.sin_cos();
        let (_skew_sin_x, skew_cos_x) = self.skew_x.sin_cos();
        let (_skew_sin_y, skew_cos_y) = self.skew_y.sin_cos();

        // Apply: Translate * Rotate * Scale * Skew
        // Skew X: [1, 0, 0; tan(skew_x), 1, 0; 0, 0, 1]
        // Skew Y: [1, tan(skew_y), 0; 0, 1, 0; 0, 0, 1]
        // Scale: [scale_x, 0, 0; 0, scale_y, 0; 0, 0, 1]
        // Rotate: [cos, -sin, 0; sin, cos, 0; 0, 0, 1]
        // Translate: [1, 0, tx; 0, 1, ty; 0, 0, 1]

        let m00 = self.scale_x * cos * skew_cos_x;
        let m01 = -self.scale_x * sin * skew_cos_y;
        let m02 = self.translation.x;

        let m10 = self.scale_y * sin * skew_cos_x;
        let m11 = self.scale_y * cos * skew_cos_y;
        let m12 = self.translation.y;

        Transform {
            matrix: Mat3 {
                m00,
                m01,
                m02,
                m10,
                m11,
                m12,
                m20: 0.0,
                m21: 0.0,
                m22: 1.0,
            },
        }
    }

    /// Get the memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Compact representation of a 2D transform for efficient storage
///
/// Uses different variants based on the transform type to save memory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompactTransform {
    /// Only translation
    Translation { x: f32, y: f32 },
    /// Translation + uniform scale
    TranslationScale { x: f32, y: f32, scale: f32 },
    /// Translation + non-uniform scale
    Scale {
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
    },
    /// Translation + rotation
    Rotation { x: f32, y: f32, rotation: f32 },
    /// Full transform (all components)
    Full { matrix: [f32; 9] },
}

impl CompactTransform {
    /// Create from a full transform
    pub fn from_transform(transform: Transform) -> Self {
        let decomp = transform.decompose();
        let mat = transform.to_mat3();

        // Check if it's just translation
        if decomp.scale_x == 1.0
            && decomp.scale_y == 1.0
            && decomp.rotation == 0.0
            && decomp.skew_x == 0.0
            && decomp.skew_y == 0.0
        {
            return CompactTransform::Translation {
                x: decomp.translation.x,
                y: decomp.translation.y,
            };
        }

        // Check if it's translation + uniform scale
        if decomp.scale_x == decomp.scale_y
            && decomp.rotation == 0.0
            && decomp.skew_x == 0.0
            && decomp.skew_y == 0.0
        {
            return CompactTransform::TranslationScale {
                x: decomp.translation.x,
                y: decomp.translation.y,
                scale: decomp.scale_x,
            };
        }

        // Check if it's translation + rotation only (no scale)
        if decomp.scale_x == 1.0
            && decomp.scale_y == 1.0
            && decomp.skew_x == 0.0
            && decomp.skew_y == 0.0
        {
            return CompactTransform::Rotation {
                x: decomp.translation.x,
                y: decomp.translation.y,
                rotation: decomp.rotation,
            };
        }

        // Check if it's translation + non-uniform scale (no rotation)
        if decomp.rotation == 0.0 && decomp.skew_x == 0.0 && decomp.skew_y == 0.0 {
            return CompactTransform::Scale {
                x: decomp.translation.x,
                y: decomp.translation.y,
                scale_x: decomp.scale_x,
                scale_y: decomp.scale_y,
            };
        }

        // Fall back to full matrix
        CompactTransform::Full {
            matrix: [
                mat.m00, mat.m01, mat.m02, mat.m10, mat.m11, mat.m12, mat.m20, mat.m21, mat.m22,
            ],
        }
    }

    /// Convert to full transform
    pub fn to_transform(self) -> Transform {
        match self {
            CompactTransform::Translation { x, y } => Transform::from_translation(x, y),
            CompactTransform::TranslationScale { x, y, scale } => {
                Transform::from_translation(x, y) * Transform::from_scale(scale)
            }
            CompactTransform::Scale {
                x,
                y,
                scale_x,
                scale_y,
            } => {
                Transform::from_translation(x, y)
                    * Transform::from_scale_vec(Vec2::new(scale_x, scale_y))
            }
            CompactTransform::Rotation { x, y, rotation } => {
                Transform::from_translation(x, y) * Transform::from_rotation(rotation)
            }
            CompactTransform::Full { matrix } => Transform::from_mat3(Mat3 {
                m00: matrix[0],
                m01: matrix[1],
                m02: matrix[2],
                m10: matrix[3],
                m11: matrix[4],
                m12: matrix[5],
                m20: matrix[6],
                m21: matrix[7],
                m22: matrix[8],
            }),
        }
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        match self {
            CompactTransform::Translation { .. } => std::mem::size_of::<f32>() * 2,
            CompactTransform::TranslationScale { .. } => std::mem::size_of::<f32>() * 3,
            CompactTransform::Scale { .. } => std::mem::size_of::<f32>() * 4,
            CompactTransform::Rotation { .. } => std::mem::size_of::<f32>() * 3,
            CompactTransform::Full { .. } => std::mem::size_of::<f32>() * 9,
        }
    }

    /// Estimate serialization size in bytes
    pub fn serialized_size(&self) -> usize {
        // Rough estimate for JSON serialization
        match self {
            CompactTransform::Translation { .. } => 32,
            CompactTransform::TranslationScale { .. } => 48,
            CompactTransform::Scale { .. } => 64,
            CompactTransform::Rotation { .. } => 48,
            CompactTransform::Full { .. } => 144,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let t = Transform::identity();
        assert!(t.is_identity());
        assert!(t.is_translation_only());
        assert!(t.is_uniform_scale_only());
    }

    #[test]
    fn test_translation_transform() {
        let t = Transform::from_translation(100.0, 200.0);

        assert!(t.is_translation_only());
        assert_eq!(t.translation(), Vec2::new(100.0, 200.0));
        assert_eq!(t.scale(), Vec2::new(1.0, 1.0));
        assert_eq!(t.rotation(), 0.0);
    }

    #[test]
    fn test_rotation_transform() {
        let t = Transform::from_rotation(std::f32::consts::FRAC_PI_2);

        let point = Vec2::new(1.0, 0.0);
        let transformed = t.transform_point(point);

        assert!((transformed.x - 0.0).abs() < 1e-6);
        assert!((transformed.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_transform() {
        let t = Transform::from_scale(2.0);

        assert!(t.is_uniform_scale_only());

        let point = Vec2::new(1.0, 1.0);
        let transformed = t.transform_point(point);

        assert_eq!(transformed, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn test_non_uniform_scale() {
        let t = Transform::from_scale_vec(Vec2::new(2.0, 3.0));

        let point = Vec2::new(1.0, 1.0);
        let transformed = t.transform_point(point);

        assert_eq!(transformed, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_transform_composition() {
        let t1 = Transform::from_translation(100.0, 0.0);
        let t2 = Transform::from_rotation(std::f32::consts::FRAC_PI_2);

        // Compose: t1 * t2 means apply t2 first, then t1
        let composed = t1.compose(t2);

        // Apply to origin - should translate after rotation
        let point = Vec2::ZERO;
        let transformed = composed.transform_point(point);

        assert!((transformed.x - 100.0).abs() < 1e-6);
        assert!((transformed.y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_composition_order() {
        let translation = Transform::from_translation(100.0, 0.0);
        let rotation = Transform::from_rotation(std::f32::consts::FRAC_PI_2);

        // Translate then rotate
        let t_then_r = translation.compose(rotation);

        // Rotate then translate
        let r_then_t = rotation.compose(translation);

        // Apply to a point
        let point = Vec2::new(0.0, 50.0);

        let t_then_r_result = t_then_r.transform_point(point);
        let r_then_t_result = r_then_t.transform_point(point);

        // Results should be different
        assert_ne!(t_then_r_result, r_then_t_result);
    }

    #[test]
    fn test_inverse_translation() {
        let t = Transform::from_translation(100.0, 200.0);
        let inv = t.inverse().unwrap();

        let point = Vec2::new(150.0, 250.0);
        let transformed = t.transform_point(point);
        let back = inv.transform_point(transformed);

        assert!((back.x - point.x).abs() < 1e-6);
        assert!((back.y - point.y).abs() < 1e-6);
    }

    #[test]
    fn test_inverse_rotation() {
        let angle = std::f32::consts::FRAC_PI_4;
        let t = Transform::from_rotation(angle);
        let inv = t.inverse().unwrap();

        let point = Vec2::new(1.0, 0.0);
        let transformed = t.transform_point(point);
        let back = inv.transform_point(transformed);

        assert!(
            (back.x - point.x).abs() < 1e-5,
            "x: {} vs {}",
            back.x,
            point.x
        );
        assert!(
            (back.y - point.y).abs() < 1e-5,
            "y: {} vs {}",
            back.y,
            point.y
        );
    }

    #[test]
    fn test_inverse_scale() {
        let t = Transform::from_scale(2.0);
        let inv = t.inverse().unwrap();

        let point = Vec2::new(4.0, 6.0);
        let transformed = t.transform_point(point);
        let back = inv.transform_point(transformed);

        assert!((back.x - point.x).abs() < 1e-6);
        assert!((back.y - point.y).abs() < 1e-6);
    }

    #[test]
    fn test_decomposition_identity() {
        let t = Transform::identity();
        let decomp = t.decompose();

        assert_eq!(decomp.translation, Vec2::ZERO);
        assert_eq!(decomp.rotation, 0.0);
        assert!((decomp.scale_x - 1.0).abs() < 1e-6);
        assert!((decomp.scale_y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decomposition_translation() {
        let t = Transform::from_translation(50.0, 100.0);
        let decomp = t.decompose();

        assert_eq!(decomp.translation, Vec2::new(50.0, 100.0));
        assert!((decomp.scale_x - 1.0).abs() < 1e-6);
        assert!((decomp.scale_y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decomposition_rotation() {
        let t = Transform::from_rotation(std::f32::consts::FRAC_PI_4);
        let decomp = t.decompose();

        assert!((decomp.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
        assert!((decomp.scale_x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decomposition_scale() {
        let t = Transform::from_scale(2.0);
        let decomp = t.decompose();

        assert!((decomp.scale_x - 2.0).abs() < 1e-6);
        assert!((decomp.scale_y - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_compact_translation() {
        let t = Transform::from_translation(100.0, 200.0);
        let compact = CompactTransform::from_transform(t);

        match compact {
            CompactTransform::Translation { x, y } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            }
            _ => panic!("Expected Translation variant"),
        }
    }

    #[test]
    fn test_compact_translation_scale() {
        let t = Transform::from_translation(50.0, 50.0) * Transform::from_scale(2.0);
        let compact = CompactTransform::from_transform(t);

        match compact {
            CompactTransform::TranslationScale { x, y, scale } => {
                assert_eq!(x, 50.0);
                assert_eq!(y, 50.0);
                assert_eq!(scale, 2.0);
            }
            _ => panic!("Expected TranslationScale variant"),
        }
    }

    #[test]
    fn test_compact_to_matrix_identity() {
        let compact = CompactTransform::Translation { x: 0.0, y: 0.0 };
        let t = compact.to_transform();
        assert!(t.is_identity());
    }

    #[test]
    fn test_compact_to_matrix_translation() {
        let compact = CompactTransform::Translation { x: 100.0, y: 200.0 };
        let t = compact.to_transform();

        let point = Vec2::ZERO;
        let transformed = t.transform_point(point);

        assert_eq!(transformed, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn test_compact_roundtrip() {
        let original = Transform::from_translation(50.0, 100.0)
            * Transform::from_rotation(std::f32::consts::FRAC_PI_4);

        let compact = CompactTransform::from_transform(original);
        let restored = compact.to_transform();

        // Check a few points
        let test_points = [Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)];

        for point in test_points {
            let orig = original.transform_point(point);
            let restored_result = restored.transform_point(point);

            assert!((orig.x - restored_result.x).abs() < 1e-4);
            assert!((orig.y - restored_result.y).abs() < 1e-4);
        }
    }

    #[test]
    fn test_memory_savings() {
        let full_size = std::mem::size_of::<Mat3>();
        let compact_translation = CompactTransform::Translation { x: 0.0, y: 0.0 };
        let compact_full = CompactTransform::Full { matrix: [0.0; 9] };

        // Translation uses 2 floats instead of 9
        assert!(compact_translation.memory_usage() < full_size);

        // Full still uses 9 floats
        assert_eq!(compact_full.memory_usage(), full_size);
    }

    #[test]
    fn test_serialize_deserialize_compact() {
        let original = CompactTransform::Translation { x: 42.0, y: 84.0 };

        let json = serde_json::to_string(&original).unwrap();
        let restored: CompactTransform = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }
}
