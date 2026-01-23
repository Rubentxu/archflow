//! Transform - Transformación 2D (posición, rotación, escala)

use crate::{Mat3, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Transformación 2D para entidades
///
/// Representa la transformación local de una entidad relativa a su padre.
/// Para transformaciones globales, se calcula combinando con el padre.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Posición local (translation)
    pub translation: Vec2,

    /// Rotación en radianes
    pub rotation: f32,

    /// Escala (1.0 = sin escala)
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    /// Crear transform identidad
    pub fn identity() -> Self {
        Self::default()
    }

    /// Crear con posición
    pub fn from_translation(x: f32, y: f32) -> Self {
        Self {
            translation: Vec2::new(x, y),
            ..Default::default()
        }
    }

    /// Crear con rotación
    pub fn from_rotation(angle: f32) -> Self {
        Self {
            rotation: angle,
            ..Default::default()
        }
    }

    /// Crear con escala uniforme
    pub fn from_scale(s: f32) -> Self {
        Self {
            scale: Vec2::splat(s),
            ..Default::default()
        }
    }

    /// Convertir a matriz de transformación 2D
    pub fn to_matrix(&self) -> Mat3 {
        // Crear matrices de transformación
        let trans = Mat3::from_translation(self.translation);
        let rot = Mat3::from_rotation(self.rotation);
        let scale = Mat3::from_scale(self.scale);

        // Combinar: translation * rotation * scale
        trans.mul_mat(rot).mul_mat(scale)
    }

    /// Aplicar transformación a un punto
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        self.to_matrix().transform_point2(point)
    }

    /// Interpolación lineal
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// Transform2D - Alias para Transform
pub type Transform2D = Transform;

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transform(translation={}, rotation={}°, scale={})",
            self.translation,
            self.rotation.to_degrees(),
            self.scale
        )
    }
}
