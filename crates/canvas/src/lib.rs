//! # ArchFlow Canvas - Bounded Context for Diagramming
//!
//! This crate consolidates all canvas-related functionality from the old architecture:
//! - **Shapes** (from `archflow-primitives`): Rectangle, Ellipse, Line, Path, Text, etc.
//! - **Viewport** (from `archflow-spatial`): Pan, zoom, coordinate transforms
//! - **Spatial Index** (from `archflow-spatial`): R-Tree for efficient spatial queries
//! - **Geometry** (from `archflow-geometry`): Path operations, intersections, hit testing
//! - **Canvas State** (from `archflow-sdk`): Canvas, selection, layers, background
//!
//! # Architecture
//!
//! This bounded context follows the **Connascence of Meaning** principle:
//! - All concepts share the same domain language (Entity, Shape, Canvas, Selection)
//! - High cohesion: changes to canvas concepts stay localized
//! - Low coupling: depends only on `archflow-core` for shared types
//!
//! # Migration
//!
//! This crate replaces:
//! - `archflow-primitives` → `canvas::shapes`
//! - `archflow-spatial` → `canvas::spatial`
//! - `archflow-geometry` → `canvas::geometry`
//! - `archflow-workspace` → integrated into canvas (selection), moved rest to editing crate

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod canvas;
mod geometry;
mod shapes;
mod spatial;
mod viewport;

pub use canvas::{Canvas, CanvasError, CanvasOperation, Selection, ShapeChanges};
pub use geometry::{IntersectionResult, PathEngine};
pub use shapes::{Shape, ShapeGeometry, ShapeStyle, ShapeType, Stroke};
pub use spatial::{HitTestOptions, HitTestResult, SpatialIndex};
pub use viewport::Viewport;

/// Re-export core types for convenience
pub use archflow_core::{Color, EntityId, Rect, Transform, Vec2};

#[cfg(test)]
mod canvas_tests;
