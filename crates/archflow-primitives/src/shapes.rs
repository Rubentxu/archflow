//! ArchFlow Primitives - Primitivas geométricas del engine
//!
//! Este crate define las primitivas fundamentales para el sistema de dibujo

use archflow_core::{EntityId, Rect, Transform, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enum de todos los tipos de primitivas soportadas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Rectangle,
    Ellipse,
    Line,
    Polyline,
    Path,
    BezierCurve,
    Arc,
    Text,
    Image,
    Group,
}

/// Trait común para todas las primitivas
pub trait Primitive: Send + Sync {
    fn primitive_type(&self) -> PrimitiveType;
    fn id(&self) -> EntityId;
    fn transform(&self) -> Transform;
    fn set_transform(&mut self, transform: Transform);
    fn local_bounds(&self) -> Rect;
    fn global_bounds(&self) -> Rect;
    fn contains_point(&self, point: Vec2) -> bool;
}

/// Propiedades comunes a todas las primitivas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveProperties {
    pub id: EntityId,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub visible: bool,
    pub opacity: f32,
    pub layer: i32,
    pub locked: bool,
}

impl Default for PrimitiveProperties {
    fn default() -> Self {
        Self {
            id: EntityId::new(),
            name: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            visible: true,
            opacity: 1.0,
            layer: 0,
            locked: false,
        }
    }
}

/// Primitiva rectangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub props: PrimitiveProperties,
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
}

impl Rectangle {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            props: PrimitiveProperties::default(),
            width,
            height,
            corner_radius: 0.0,
        }
    }
}

impl Primitive for Rectangle {
    fn primitive_type(&self) -> PrimitiveType {
        PrimitiveType::Rectangle
    }

    fn id(&self) -> EntityId {
        self.props.id
    }

    fn transform(&self) -> Transform {
        Transform::identity()
    }

    fn set_transform(&mut self, _transform: Transform) {}

    fn local_bounds(&self) -> Rect {
        Rect::from_pos_size(Vec2::ZERO, Vec2::new(self.width, self.height))
    }

    fn global_bounds(&self) -> Rect {
        self.local_bounds()
    }

    fn contains_point(&self, point: Vec2) -> bool {
        self.local_bounds().contains(point)
    }
}

/// Primitiva elipse/círculo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ellipse {
    pub props: PrimitiveProperties,
    pub radius_x: f32,
    pub radius_y: f32,
}

impl Ellipse {
    pub fn new(radius_x: f32, radius_y: f32) -> Self {
        Self {
            props: PrimitiveProperties::default(),
            radius_x,
            radius_y,
        }
    }

    pub fn circle(radius: f32) -> Self {
        Self::new(radius, radius)
    }
}

impl Primitive for Ellipse {
    fn primitive_type(&self) -> PrimitiveType {
        PrimitiveType::Ellipse
    }

    fn id(&self) -> EntityId {
        self.props.id
    }

    fn transform(&self) -> Transform {
        Transform::identity()
    }

    fn set_transform(&mut self, _transform: Transform) {}

    fn local_bounds(&self) -> Rect {
        Rect::from_center_size(
            Vec2::ZERO,
            Vec2::new(self.radius_x * 2.0, self.radius_y * 2.0),
        )
    }

    fn global_bounds(&self) -> Rect {
        self.local_bounds()
    }

    fn contains_point(&self, point: Vec2) -> bool {
        let nx = point.x / self.radius_x;
        let ny = point.y / self.radius_y;
        nx * nx + ny * ny <= 1.0
    }
}

/// Primitiva línea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub props: PrimitiveProperties,
    pub start: Vec2,
    pub end: Vec2,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self {
            props: PrimitiveProperties::default(),
            start,
            end,
        }
    }
}

impl Primitive for Line {
    fn primitive_type(&self) -> PrimitiveType {
        PrimitiveType::Line
    }

    fn id(&self) -> EntityId {
        self.props.id
    }

    fn transform(&self) -> Transform {
        Transform::identity()
    }

    fn set_transform(&mut self, _transform: Transform) {}

    fn local_bounds(&self) -> Rect {
        let start_min = self.start.min(self.end);
        let start_max = self.start.max(self.end);
        Rect::from_min_max(start_min, start_max)
    }

    fn global_bounds(&self) -> Rect {
        self.local_bounds()
    }

    fn contains_point(&self, point: Vec2) -> bool {
        let bounds = self.local_bounds();
        point.x >= bounds.min.x
            && point.x <= bounds.max.x
            && point.y >= bounds.min.y
            && point.y <= bounds.max.y
    }
}

/// Primitiva polilínea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline {
    pub props: PrimitiveProperties,
    pub points: Vec<Vec2>,
    pub closed: bool,
}

impl Polyline {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self {
            props: PrimitiveProperties::default(),
            points,
            closed: false,
        }
    }
}

impl Primitive for Polyline {
    fn primitive_type(&self) -> PrimitiveType {
        PrimitiveType::Polyline
    }

    fn id(&self) -> EntityId {
        self.props.id
    }

    fn transform(&self) -> Transform {
        Transform::identity()
    }

    fn set_transform(&mut self, _transform: Transform) {}

    fn local_bounds(&self) -> Rect {
        if self.points.is_empty() {
            return Rect::default();
        }
        let min = self
            .points
            .iter()
            .fold(Vec2::new(f32::MAX, f32::MAX), |a, b| a.min(*b));
        let max = self
            .points
            .iter()
            .fold(Vec2::new(f32::MIN, f32::MIN), |a, b| a.max(*b));
        Rect::from_min_max(min, max)
    }

    fn global_bounds(&self) -> Rect {
        self.local_bounds()
    }

    fn contains_point(&self, point: Vec2) -> bool {
        self.local_bounds().contains(point)
    }
}
