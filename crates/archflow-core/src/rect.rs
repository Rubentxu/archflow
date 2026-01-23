//! Rect - Rectángulo 2D

use crate::Vec2;
use kurbo::Rect as KurboRect;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Rectángulo 2D definido por posición y tamaño
///
/// Compatible con euclid::Box2D pero más simple para nuestro caso de uso.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Posición de la esquina superior izquierda
    pub min: Vec2,

    /// Posición de la esquina inferior derecha
    pub max: Vec2,
}

impl Rect {
    /// Crear rectángulo desde min y max
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Crear rectángulo desde posición y tamaño
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    /// Crear rectángulo desde centro y tamaño
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size / 2.0;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Crear rectángulo infinito
    pub fn infinite() -> Self {
        Self {
            min: Vec2::splat(f32::MIN),
            max: Vec2::splat(f32::MAX),
        }
    }

    /// Obtener posición (esquina superior izquierda)
    pub fn min(&self) -> Vec2 {
        self.min
    }

    /// Obtener tamaño
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Obtener centro
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Ancho
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Alto
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Verificar si contiene un punto
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Verificar intersección con otro rectángulo
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Obtener intersección con otro rectángulo
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if self.intersects(other) {
            Some(Rect::from_min_max(
                Vec2::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
                Vec2::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
            ))
        } else {
            None
        }
    }

    /// Expandir para contener un punto
    pub fn expand_to_contain(&mut self, point: Vec2) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Crear un margen alrededor del rectángulo
    pub fn inflate(&self, margin: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(margin),
            max: self.max + Vec2::splat(margin),
        }
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::from_pos_size(Vec2::ZERO, Vec2::ZERO)
    }
}

/// Conversión de Rect a KurboRect
impl From<Rect> for KurboRect {
    fn from(rect: Rect) -> Self {
        KurboRect::new(
            rect.min.x as f64,
            rect.min.y as f64,
            rect.max.x as f64,
            rect.max.y as f64,
        )
    }
}

/// Conversión de KurboRect a Rect
impl From<KurboRect> for Rect {
    fn from(kurbo: KurboRect) -> Self {
        Self::from_min_max(
            Vec2::new(kurbo.x0 as f32, kurbo.y0 as f32),
            Vec2::new(kurbo.x1 as f32, kurbo.y1 as f32),
        )
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rect(min={}, max={}, size={}x{})",
            self.min,
            self.max,
            self.width(),
            self.height()
        )
    }
}

/// Alias para Rect en 2D
pub type Rect2D = Rect;
