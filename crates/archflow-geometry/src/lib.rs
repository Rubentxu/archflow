//! ArchFlow Geometry - Cálculos geométricos
//!
//! Este crate proporciona cálculos geométricos para el engine usando kurbo.
//! Incluye:
//! - GeometryEngine: operaciones geométricas básicas
//! - PathEngine: operaciones sobre paths y curvas de Bézier
//! - IntersectionEngine: detección de intersecciones

// Re-export de tipos de core
pub use archflow_core::{Rect, Vec2};

// Re-export de kurbo para uso externo
pub use kurbo::{BezPath, Point as KurboPoint, Rect as KurboRect, Shape};

// Módulos
mod geometry;
mod intersection;
mod path;
mod spatial;

pub use geometry::{DiscretizeConfig, GeometryEngine, IntersectionResult};
pub use intersection::{HitTestConfig, IntersectionEngine, IntersectionType};
pub use path::{ArcElement, ArcType, PathElement, PathEngine, SimplifyConfig};
pub use spatial::{
    DirtyChangeType, DirtyRecord, DirtyState, SpatialData, SpatialIndex, SpatialIndexConfig,
    SpatialItem, SpatialMetrics, SpatialQueryConfig, SpatialQueryOrder, SpatialQueryResult,
    SpatialQueryType,
};
