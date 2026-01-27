//! Transform - Transformación 2D simplificada para APIs de alto nivel
//!
//! Esta es una versión simplificada para Scene y ShapeFactory.
//! Para transformaciones completas en ECS, usar archflow_ecs_hybrid::Transform.

use crate::{Mat3, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Transformación 2D simplificada para Scene y APIs de alto nivel
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Transform {
    /// Posición
    pub translation: Vec2,
    /// Rotación en radianes
    pub rotation: f32,
    /// Escala
    pub scale: Vec2,
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
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Convertir a matriz 3x3
    pub fn to_mat3(&self) -> Mat3 {
        let trans = Mat3::from_translation(self.translation);
        let rot = Mat3::from_rotation(self.rotation);
        let scale = Mat3::from_scale(self.scale);
        trans.mul_mat(rot).mul_mat(scale)
    }
}

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
