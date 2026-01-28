//! ArchFlow Core - Meta-crate con tipos base del dominio
//!
//! Este crate contiene los tipos fundamentales usados por todo el engine:
//! - Tipos geométricos (Vec2, Mat3, Rect)
//! - Identificadores (EntityId)
//! - Tipos de error y resultado
//! - Animaciones (Keyframes, Easing Functions)
//! - Recursos Externos (Imágenes, Videos, HTML Overlays)
//! - Zoom Incremental (Level of Detail para C4)
//! - APIs para Desarrolladores (CanvasBuilder, ShapeFactory, Scene)

// Módulos
mod animation;
mod api;
mod color;
mod entity_id;
mod error;
mod rect;
mod resources;
mod transform;
mod transform_enhanced;
mod types;
mod zoom;

// Re-exports
pub use animation::*;
pub use api::*;
pub use color::{Color, Hsla, Rgba};
pub use entity_id::{EntityId, EntityIdGenerator};
pub use error::{CoreError, CoreResult};
pub use rect::{Rect, Rect2D};
pub use resources::*;
pub use transform::Transform;
pub use transform_enhanced::{
    CompactTransform, Transform as TransformEnhanced, TransformDecomposition,
};
pub use types::{Mat3, Vec2};
pub use zoom::*;

// Re-export de uuid
pub use uuid::Uuid;
