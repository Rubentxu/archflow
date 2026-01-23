//! Types - Tipos base con soporte serde
//!
//! Este módulo define tipos base como Vec2 y Mat3 que incluyen serialización serde.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Div, Mul, MulAssign, Neg, Sub};

/// Vector 2D con soporte serde
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
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

    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len)
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(&self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(&self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    #[inline]
    pub fn lerp(&self, other: Vec2, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    #[inline]
    pub fn min(&self, other: Vec2) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

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

/// Matriz 3x3 para transformaciones 2D
/// [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat3 {
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
}

impl Mat3 {
    /// Matriz identidad
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

    /// Crear desde array
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

    /// Crear matriz de traslación
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

    /// Crear matriz de rotación
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

    /// Crear matriz de escala
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

    /// Multiplicar por otra matriz
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

    /// Multiplicar por vector 2D (transformar punto)
    #[inline]
    pub fn transform_point2(&self, v: Vec2) -> Vec2 {
        let x = self.m00 * v.x + self.m01 * v.y + self.m02;
        let y = self.m10 * v.x + self.m11 * v.y + self.m12;
        Vec2::new(x, y)
    }

    /// Multiplicar por vector 2D (transformar dirección, sin traslación)
    #[inline]
    pub fn transform_vector2(&self, v: Vec2) -> Vec2 {
        let x = self.m00 * v.x + self.m01 * v.y;
        let y = self.m10 * v.x + self.m11 * v.y;
        Vec2::new(x, y)
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
