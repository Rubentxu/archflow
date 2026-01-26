//! Synchronization utilities for Records to ECS
//!
//! This module provides extension traits and utility functions
//! for synchronizing Record changes to ECS entities.

use archflow_records::{Record, RecordId};
use bevy_ecs::prelude::*;

/// Extension trait for converting RecordId to ECS Entity.
///
/// This provides a convenient way to create Entity references
/// from RecordId for ECS operations.
pub trait RecordIdEntityExt {
    /// Converts RecordId to Entity.
    ///
    /// This assumes RecordId was created from a u64 Entity index.
    /// Returns Entity with same index.
    fn to_entity(&self) -> Entity;
}

impl RecordIdEntityExt for RecordId {
    fn to_entity(&self) -> Entity {
        Entity::from_bits(self.as_u64())
    }
}

/// Extension trait for extracting u64 from RecordId.
pub trait RecordIdU64Ext {
    fn as_u64(&self) -> u64;
}

impl RecordIdU64Ext for RecordId {
    fn as_u64(&self) -> u64 {
        let s = self.as_str();
        if let Some(hex_str) = s.strip_prefix("id_") {
            u64::from_str_radix(hex_str.trim_start_matches('0'), 10).unwrap_or_else(|_| {
                use std::hash::{Hash, Hasher};
                let mut hash = std::collections::hash_map::DefaultHasher::new();
                s.hash(&mut hash);
                use std::hash::Hasher;
                hash.finish()
            })
        } else {
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            s.hash(&mut hash);
            use std::hash::Hasher;
            hash.finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_id_to_entity() {
        let id = RecordId::from_u64(12345);
        let entity = id.to_entity();
        assert_eq!(entity.to_bits(), 12345);
    }

    #[test]
    fn test_record_id_as_u64() {
        let id1 = RecordId::from_u64(12345);
        let u64_val = id1.as_u64();
        assert_eq!(u64_val, 12345);

        let id2 = RecordId::from_str("custom_id_1234567890").unwrap();
        let u64_val2 = id2.as_u64();
        assert!(u64_val2 > 0);
    }

    #[test]
    fn test_record_id_to_entity_from_u64() {
        let id = RecordId::from_u64(12345);
        let entity = id.to_entity();
        assert_eq!(entity.to_bits(), 12345);
    }
}
