//! ECS Systems Module
//!
//! This module provides systems for synchronizing Records with ECS entities
//! and tracking changes for bidirectional sync.

pub mod dirty_tracking_system;
pub mod sync_record_to_ecs;

pub use dirty_tracking_system::{
    cleanup_dirty_system, clear_dirty_flags_system, dirty_tracking_system,
};
pub use sync_record_to_ecs::sync_records_to_ecs_system;
