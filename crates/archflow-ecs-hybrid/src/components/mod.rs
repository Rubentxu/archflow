//! ECS Components Module
//!
//! This module provides components for linking ECS entities to Records
//! and managing synchronization state.

pub mod record_ref;
pub mod transform;

pub use record_ref::{Dirty, DirtyType, RecordRef};
pub use transform::{Transform, TransformBundle};
