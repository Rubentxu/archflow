//! # archflow-spatial
//!
//! Spatial indexing system with R-Tree for efficient spatial queries.

pub mod queries;
pub mod rtree;
pub mod trait_spatial_index;
pub mod viewport_manager;

pub use queries::{HitTestOptions, HitTestResult, SpatialQueries};
pub use rtree::{RTreeIndex, SpatialObject};
pub use trait_spatial_index::{Frustum, SpatialIndex};
pub use viewport_manager::ViewportManager;
