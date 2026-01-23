//! ArchFlow ECS - Bevy ECS integration

mod systems;

pub use systems::{
    Scale, SpatialDirty, Text, Transform, TransformDirty, ZIndex, mark_spatial_dirty,
    mark_transform_dirty, spatial_sync_system, spawn_shape, spawn_text, transform_update_system,
};

pub use bevy_ecs::world::World;

use archflow_core::geometry::Vec2;
use archflow_core::records::Record;
use bevy_ecs::prelude::*;
use indexmap::IndexMap;
use std::marker::PhantomData;

/// Maximum number of entities for ECS
pub const ARCFLOW_ECS_MAX_ENTITIES: usize = 10_000;

/// ECS Resources for managing Store ↔ ECS synchronization
#[derive(Resource, Debug)]
pub struct EcsSyncState<R: Record> {
    /// Map from RecordId to Entity for quick lookup
    pub id_to_entity: IndexMap<String, Entity>,
    _phantom: PhantomData<R>,
}

impl<R: Record> Default for EcsSyncState<R> {
    fn default() -> Self {
        Self {
            id_to_entity: IndexMap::new(),
            _phantom: PhantomData,
        }
    }
}

impl<R: Record> EcsSyncState<R> {
    /// Creates a new sync state
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the entity for a record ID
    pub fn entity_for(&self, id: &str) -> Option<Entity> {
        self.id_to_entity.get(id).copied()
    }

    /// Gets the number of synced records
    pub fn len(&self) -> usize {
        self.id_to_entity.len()
    }

    /// Checks if there are any synced records
    pub fn is_empty(&self) -> bool {
        self.id_to_entity.is_empty()
    }
}

/// Position component for ECS entities
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Position(pub Vec2);

impl Position {
    /// Creates a new position
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    /// Gets the x coordinate
    pub fn x(&self) -> f32 {
        self.0.x()
    }

    /// Gets the y coordinate
    pub fn y(&self) -> f32 {
        self.0.y()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

/// Shape component for ECS entities
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Shape {
    /// Type of shape
    pub shape_type: ShapeType,
    /// Width of the shape
    pub width: f32,
    /// Height of the shape
    pub height: f32,
    /// Rotation in radians
    pub rotation: f32,
}

impl Shape {
    /// Creates a new rectangle shape
    pub fn rect(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Rect,
            width,
            height,
            rotation: 0.0,
        }
    }

    /// Creates a new ellipse shape
    pub fn ellipse(radius_x: f32, radius_y: f32) -> Self {
        Self {
            shape_type: ShapeType::Ellipse,
            width: radius_x * 2.0,
            height: radius_y * 2.0,
            rotation: 0.0,
        }
    }

    /// Sets the rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::rect(100.0, 100.0)
    }
}

/// Type of shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeType {
    /// Rectangle/Square
    Rect,
    /// Ellipse/Circle
    Ellipse,
    /// Line
    Line,
    /// Arrow
    Arrow,
    /// Text
    Text,
    /// Image
    Image,
}

/// Color component for ECS entities
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0-1)
    pub r: f32,
    /// Green component (0-1)
    pub g: f32,
    /// Blue component (0-1)
    pub b: f32,
    /// Alpha component (0-1)
    pub a: f32,
}

impl Color {
    /// Creates a new color from RGBA components
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Creates a color from hex string
    pub fn hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
        } else {
            1.0
        };
        Self::new(r, g, b, a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

/// Stroke component for outlines
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Stroke width in pixels
    pub width: f32,
    /// Stroke color
    pub color: Color,
}

impl Stroke {
    /// Creates a new stroke
    pub fn new(width: f32, color: Color) -> Self {
        Self { width, color }
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self::new(2.0, Color::hex("#000000"))
    }
}

/// Fill component for shapes
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    /// Fill color
    pub color: Color,
    /// Fill opacity (0-1)
    pub opacity: f32,
}

impl Fill {
    /// Creates a new fill
    pub fn new(color: Color, opacity: f32) -> Self {
        Self {
            color,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Self::new(Color::hex("#000000"), 0.0)
    }
}
