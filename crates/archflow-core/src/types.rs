//! Types - Tipos base con soporte serde
//!
//! Este módulo define tipos base como Vec2 y Mat3 que incluyen serialización serde.
//!
//! # Arquitectura
//!
//! Los tipos están diseñados siguiendo principios de:
//! - **Zero-cost abstractions**: Trait implementations son `#[inline]`
//! - **Cache-friendly**: Alignment de 8 bytes para Vec2
//! - **Type safety**: Sin type aliases confusos como `f32` para vectores

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, MulAssign, Neg, Sub, SubAssign};

/// Vector 2D con soporte serde.
///
/// # Memory Layout
///
/// ```ignore
/// Vec2 { x: f32, y: f32 }  // 8 bytes, 4-byte aligned
/// ```
///
/// # Ejemplo
///
/// ```
/// use archflow_core::types::Vec2;
///
/// let v = Vec2::new(1.0, 2.0);
/// assert_eq!(v.dot(v), 5.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec2 {
    /// X component
    pub x: f32,
    /// Y component
    pub y: f32,
}

impl Vec2 {
    /// Vector cero
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    /// Vector uno
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };

    /// Eje X unitario
    pub const X: Vec2 = Vec2 { x: 1.0, y: 0.0 };

    /// Eje Y unitario
    pub const Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };

    /// Creates a new vector from components.
    ///
    /// # Arguments
    ///
    /// * `x` - X component
    /// * `y` - Y component
    ///
    /// # Returns
    ///
    /// A new `Vec2` with the given components.
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Creates a vector with all components set to the same value.
    ///
    /// # Arguments
    ///
    /// * `v` - Value for all components
    ///
    /// # Returns
    ///
    /// A vector with `x = v` and `y = v`.
    #[inline]
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// Calculates the squared length of the vector.
    ///
    /// This is faster than `length()` as it avoids the square root.
    ///
    /// # Returns
    ///
    /// The squared length `x² + y²`.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Calculates the length (magnitude) of the vector.
    ///
    /// # Returns
    ///
    /// The length `sqrt(x² + y²)`.
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns a normalized vector with length 1.0.
    ///
    /// If the vector has zero length, returns `ZERO`.
    ///
    /// # Returns
    ///
    /// A normalized copy of this vector.
    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::ZERO
        }
    }

    /// Calculates the dot product with another vector.
    ///
    /// # Arguments
    ///
    /// * `other` - The other vector
    ///
    /// # Returns
    ///
    /// The dot product `x₁·x₂ + y₁·y₂`.
    #[inline]
    pub fn dot(&self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Calculates the 2D cross product (perp-dot product).
    ///
    /// Returns `x₁·y₂ - y₁·x₂`, equivalent to the z-component
    /// of the 3D cross product.
    ///
    /// # Arguments
    ///
    /// * `other` - The other vector
    ///
    /// # Returns
    ///
    /// The cross product value.
    #[inline]
    pub fn cross(&self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Returns a perpendicular vector (rotated 90° counter-clockwise).
    ///
    /// # Returns
    ///
    /// A perpendicular vector `(-y, x)`.
    #[inline]
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Linear interpolation between this and another vector.
    ///
    /// # Arguments
    ///
    /// * `other` - The destination vector
    /// * `t` - Interpolation factor in range [0, 1]
    ///
    /// # Returns
    ///
    /// A linearly interpolated vector.
    #[inline]
    pub fn lerp(&self, other: Vec2, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// Component-wise minimum with another vector.
    ///
    /// # Arguments
    ///
    /// * `other` - The other vector
    ///
    /// # Returns
    ///
    /// A vector with `min(x₁, x₂)` and `min(y₁, y₂)`.
    #[inline]
    pub fn min(&self, other: Vec2) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// Component-wise maximum with another vector.
    ///
    /// # Arguments
    ///
    /// * `other` - The other vector
    ///
    /// # Returns
    ///
    /// A vector with `max(x₁, x₂)` and `max(y₁, y₂)`.
    #[inline]
    pub fn max(&self, other: Vec2) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, s: f32) -> Self {
        Self::new(self.x / s, self.y / s)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec2({:.2}, {:.2})", self.x, self.y)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Vec2f64 - Vector 2D de doble precisión para coordenadas de cámara
//
// Usado para:
// - Posición de cámara (evita jittering en zoom extremo)
// - Coordenadas de mundo (antes de conversión a f32 para GPU)
// - Conversión precisa a f32 para shaders
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Vector 2D de doble precisión para coordenadas de cámara.
///
/// Problem: En zoom extremo (1000x+), coordenadas como 10_000_000.0
/// pierden precisión cuando se convierten a f32 (~7 dígitos significativos).
///
/// Solution: Usar f64 para posición de cámara, convertir a f32
/// SOLO después de restar la posición de cámara (coordinates relativas).
///
/// # Memory Layout
///
/// ```ignore
/// Vec2f64 { x: f64, y: f64 }  // 16 bytes, 8-byte aligned
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vec2f64 {
    /// X component in double precision
    pub x: f64,
    /// Y component in double precision
    pub y: f64,
}

impl Vec2f64 {
    /// Vector cero
    pub const ZERO: Vec2f64 = Vec2f64 { x: 0.0, y: 0.0 };

    /// Creates a new vector from components.
    ///
    /// # Arguments
    ///
    /// * `x` - X component
    /// * `y` - Y component
    ///
    /// # Returns
    ///
    /// A new `Vec2f64` with the given components.
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Creates a vector with all components set to the same value.
    ///
    /// # Arguments
    ///
    /// * `v` - Value for all components
    ///
    /// # Returns
    ///
    /// A vector with `x = v` and `y = v`.
    #[inline]
    pub fn splat(v: f64) -> Self {
        Self { x: v, y: v }
    }

    /// Calculates the length (magnitude) of the vector.
    ///
    /// # Returns
    ///
    /// The length `sqrt(x² + y²)`.
    #[inline]
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Subtracts a Vec2 (returns Vec2).
    ///
    /// Useful when the target precision is f32.
    ///
    /// # Arguments
    ///
    /// * `other` - The Vec2 to subtract
    ///
    /// # Returns
    ///
    /// A `Vec2` with the difference.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub_f32(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x as f32 - other.x, self.y as f32 - other.y)
    }

    /// Converts to Vec2 (truncating/converting).
    ///
    /// Warning: May lose precision for large values.
    ///
    /// # Returns
    ///
    /// A `Vec2` with converted components.
    #[inline]
    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    /// Converts to Vec2 with safe relative conversion.
    ///
    /// Useful for coordinates relative to a reference point (near 0).
    ///
    /// # Arguments
    ///
    /// * `reference` - The reference point to subtract first
    ///
    /// # Returns
    ///
    /// A `Vec2` with components relative to the reference.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_core::types::{Vec2, Vec2f64};
    ///
    /// let world_pos = Vec2f64::new(10_000_000.0, 10_000_000.0);
    /// let camera_pos = Vec2f64::new(9_999_990.0, 9_999_990.0);
    ///
    /// // This would lose precision if done directly:
    /// // let _bad = world_pos.to_vec2(); // May lose precision
    ///
    /// // This preserves precision:
    /// let relative = world_pos.to_relative_vec2(camera_pos);
    /// ```
    #[inline]
    pub fn to_relative_vec2(self, reference: Self) -> Vec2 {
        let relative = self.sub(reference);
        // Now relative.x and relative.y are small (~tens or hundreds)
        // so f32 conversion preserves precision
        Vec2::new(relative.x as f32, relative.y as f32)
    }
}

impl Default for Vec2f64 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Vec2f64 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2f64 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for Vec2f64 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f64> for Vec2f64 {
    type Output = Self;
    #[inline]
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s)
    }
}

impl Div<f64> for Vec2f64 {
    type Output = Self;
    #[inline]
    fn div(self, s: f64) -> Self {
        Self::new(self.x / s, self.y / s)
    }
}

impl fmt::Display for Vec2f64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec2f64({:.4}, {:.4})", self.x, self.y)
    }
}

/// Matriz 3x3 para transformaciones 2D.
///
/// Column-major order representation:
/// ```ignore
/// [[m00, m01, m02],
///  [m10, m11, m12],
///  [m20, m21, m22]]
/// ```
///
/// Used for:
/// - 2D affine transformations (translation, rotation, scale)
/// - World-to-screen projection
/// - Hierarchical transforms (parent-child relationships)
///
/// # Memory Layout
///
/// ```ignore
/// Mat3 { m00..m22 }  // 36 bytes, 4-byte aligned
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Mat3 {
    /// Column 0, row 0
    pub m00: f32,
    /// Column 1, row 0
    pub m01: f32,
    /// Column 2, row 0 (translation x)
    pub m02: f32,
    /// Column 0, row 1
    pub m10: f32,
    /// Column 1, row 1
    pub m11: f32,
    /// Column 2, row 1 (translation y)
    pub m12: f32,
    /// Column 0, row 2
    pub m20: f32,
    /// Column 1, row 2
    pub m21: f32,
    /// Column 2, row 2
    pub m22: f32,
}

impl Mat3 {
    /// Identity matrix
    pub const IDENTITY: Mat3 = Mat3 {
        m00: 1.0,
        m01: 0.0,
        m02: 0.0,
        m10: 0.0,
        m11: 1.0,
        m12: 0.0,
        m20: 0.0,
        m21: 0.0,
        m22: 1.0,
    };

    /// Creates a matrix from column vectors.
    ///
    /// # Arguments
    ///
    /// * `c0` - First column (basis vector x)
    /// * `c1` - Second column (basis vector y)
    /// * `c2` - Third column (translation)
    ///
    /// # Returns
    ///
    /// A new matrix with the given columns.
    #[inline]
    pub fn from_cols(c0: Vec2, c1: Vec2, c2: Vec2) -> Self {
        Self {
            m00: c0.x,
            m01: c1.x,
            m02: c2.x,
            m10: c0.y,
            m11: c1.y,
            m12: c2.y,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
        }
    }

    /// Creates a translation matrix.
    ///
    /// # Arguments
    ///
    /// * `v` - Translation vector
    ///
    /// # Returns
    ///
    /// A matrix representing the translation.
    #[inline]
    pub fn from_translation(v: Vec2) -> Self {
        Self {
            m00: 1.0,
            m01: 0.0,
            m02: v.x,
            m10: 0.0,
            m11: 1.0,
            m12: v.y,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
        }
    }

    /// Creates a rotation matrix.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians
    ///
    /// # Returns
    ///
    /// A matrix representing the rotation.
    #[inline]
    pub fn from_rotation(angle: f32) -> Self {
        let s = angle.sin();
        let c = angle.cos();
        Self {
            m00: c,
            m01: -s,
            m02: 0.0,
            m10: s,
            m11: c,
            m12: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
        }
    }

    /// Creates a scale matrix.
    ///
    /// # Arguments
    ///
    /// * `v` - Scale factors for x and y
    ///
    /// # Returns
    ///
    /// A matrix representing the scale.
    #[inline]
    pub fn from_scale(v: Vec2) -> Self {
        Self {
            m00: v.x,
            m01: 0.0,
            m02: 0.0,
            m10: 0.0,
            m11: v.y,
            m12: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
        }
    }

    /// Multiplies this matrix by another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other matrix to multiply with
    ///
    /// # Returns
    ///
    /// A new matrix representing the composition.
    #[inline]
    pub fn mul_mat(&self, other: Mat3) -> Self {
        Self {
            m00: self.m00 * other.m00 + self.m01 * other.m10 + self.m02 * other.m20,
            m01: self.m00 * other.m01 + self.m01 * other.m11 + self.m02 * other.m21,
            m02: self.m00 * other.m02 + self.m01 * other.m12 + self.m02 * other.m22,
            m10: self.m10 * other.m00 + self.m11 * other.m10 + self.m12 * other.m20,
            m11: self.m10 * other.m01 + self.m11 * other.m11 + self.m12 * other.m21,
            m12: self.m10 * other.m02 + self.m11 * other.m12 + self.m12 * other.m22,
            m20: self.m20 * other.m00 + self.m21 * other.m10 + self.m22 * other.m20,
            m21: self.m20 * other.m01 + self.m21 * other.m11 + self.m22 * other.m21,
            m22: self.m20 * other.m02 + self.m21 * other.m12 + self.m22 * other.m22,
        }
    }

    /// Transforms a 2D point (applies translation).
    ///
    /// # Arguments
    ///
    /// * `v` - The point to transform
    ///
    /// # Returns
    ///
    /// The transformed point.
    #[inline]
    pub fn transform_point2(&self, v: Vec2) -> Vec2 {
        let x = self.m00 * v.x + self.m01 * v.y + self.m02;
        let y = self.m10 * v.x + self.m11 * v.y + self.m12;
        Vec2::new(x, y)
    }

    /// Transforms a 2D direction (no translation).
    ///
    /// # Arguments
    ///
    /// * `v` - The direction to transform
    ///
    /// # Returns
    ///
    /// The transformed direction.
    #[inline]
    pub fn transform_vector2(&self, v: Vec2) -> Vec2 {
        let x = self.m00 * v.x + self.m01 * v.y;
        let y = self.m10 * v.x + self.m11 * v.y;
        Vec2::new(x, y)
    }

    /// Calculates the inverse matrix.
    ///
    /// For 2D affine matrices, this uses the optimized formula.
    ///
    /// # Returns
    ///
    /// `Some(inverse)` if the matrix is invertible, `None` if singular.
    #[inline]
    pub fn inverse(self) -> Option<Self> {
        let det = self.m00 * self.m11 - self.m01 * self.m10;

        // Check for singular matrix
        if det.abs() < 1e-10 {
            return None;
        }

        let inv_det = 1.0 / det;

        Some(Self {
            m00: self.m11 * inv_det,
            m01: -self.m01 * inv_det,
            m02: (self.m01 * self.m12 - self.m11 * self.m02) * inv_det,
            m10: -self.m10 * inv_det,
            m11: self.m00 * inv_det,
            m12: (self.m10 * self.m02 - self.m00 * self.m12) * inv_det,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
        })
    }

    /// Calculates the determinant.
    ///
    /// # Returns
    ///
    /// The determinant value.
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.m00 * self.m11 - self.m01 * self.m10
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl MulAssign for Mat3 {
    fn mul_assign(&mut self, other: Self) {
        *self = self.mul_mat(other);
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mat3([{:.2}, {:.2}, {:.2}], [{:.2}, {:.2}, {:.2}], [{:.2}, {:.2}, {:.2}])",
            self.m00,
            self.m01,
            self.m02,
            self.m10,
            self.m11,
            self.m12,
            self.m20,
            self.m21,
            self.m22
        )
    }
}
