//! SOA Entity Store - Type-Safe, Cache-Friendly Entity Component System
//!
//! This crate provides a Structure of Arrays (SOA) entity store optimized for
//! WASM compilation and cache-friendly memory layout. Uses generational indices
//! to prevent stale pointer bugs and automatic compaction to maintain locality.
//!
//! ## Architecture
//!
//! - **Generational IDs**: EntityId with (index: 24-bit, generation: 8-bit)
//! - **Auto-compaction**: Triggers when fragmentation >30%
//! - **Type-safe access**: All getters return Option<&T>, never panic
//! - **Dirty tracking**: Bitset-based tracking for optimized GPU upload
//! - **Zero-copy rendering**: RenderBatch for efficient WebGPU uploads
//!
//! ## Basic Usage
//!
//! ```rust
//! use soa_entity::EntityStore;
//! use archflow_core::Vec2;
//!
//! let mut store = EntityStore::new(100000);
//! let id = store.spawn().unwrap();
//! store.set_position(id, Vec2::new(100.0, 200.0)).unwrap();
//! ```
//!
//! ## Dirty Tracking for GPU Upload
//!
//! The store automatically tracks which components have been modified, enabling
//! efficient partial GPU uploads instead of transferring the entire buffer:
//!
//! ```rust
//! use soa_entity::EntityStore;
//! use archflow_core::Vec2;
//!
//! let mut store = EntityStore::new(1000);
//!
//! // Spawn some entities
//! let id1 = store.spawn().unwrap();
//! let id2 = store.spawn().unwrap();
//! let id3 = store.spawn().unwrap();
//!
//! // Modify only id1 and id2
//! store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();
//! store.set_position(id2, Vec2::new(30.0, 40.0)).unwrap();
//!
//! // Calculate dirty ranges for efficient GPU upload
//! let dirty_ranges = store.calculate_dirty_ranges(store.dirty_positions());
//! // Returns: [(0, 2)] - entities 0 and 1 are dirty (contiguous range)
//!
//! // Upload only the dirty range to GPU
//! for (start, length) in &dirty_ranges {
//!     let offset = start * 8; // 2 floats × 4 bytes per float
//!     let size = length * 8;
//!     // gpu_queue.write_buffer(&position_buffer, offset, &data[offset..offset+size]);
//! }
//!
//! // Mark as clean after successful upload
//! store.mark_positions_clean(&dirty_ranges);
//!
//! // Next frame: only newly modified entities will be dirty
//! ```
//!
//! ## Zero-Copy WebGPU Upload
//!
//! The RenderBatch struct provides zero-copy access to WASM memory for WebGPU:
//!
//! ```rust,no_run
//! use soa_entity::{EntityStore, RenderBatch};
//! use archflow_core::Vec2;
//!
//! let mut store = EntityStore::new(1000);
//! let id = store.spawn().unwrap();
//! store.set_position(id, Vec2::new(10.0, 20.0)).unwrap();
//!
//! // Create render batch with captured dirty state
//! let batch = RenderBatch::from_store(&store);
//!
//! // Get zero-copy view of dirty positions for WebGPU upload
//! let dirty_positions = batch.positions_dirty_slice();
//! let offset = batch.position_dirty_byte_offset();
//! let size = batch.position_dirty_byte_size();
//!
//! // In JavaScript:
//! // device.queue.writeBuffer(positionBuffer, offset, dirty_positions, 0, size);
//! ```

#![warn(missing_docs, rust_2018_idioms)]

pub mod entity_id;
pub mod render_batch;
pub mod store;

// Re-export commonly used types
pub use entity_id::EntityId;
pub use render_batch::RenderBatch;
pub use store::EntityStore;

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Color, Vec2};

    #[test]
    fn test_soa_basic_usage() {
        let mut store = EntityStore::new(100);

        // Spawn entities
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        assert_ne!(id1, id2);
        assert_eq!(store.count(), 2);

        // Set position
        store.set_position(id1, Vec2::new(100.0, 200.0)).unwrap();
        let pos = store.position(id1).unwrap();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);

        // Set color
        store
            .set_color(id1, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();
        let col = store.color(id1).unwrap();
        assert_eq!(col.r, 1.0);
    }

    #[test]
    fn test_entity_id_generation() {
        let id1 = EntityId::new(0, 1);
        assert_eq!(id1.index(), 0);
        assert_eq!(id1.generation(), 1);

        let id2 = EntityId::new(0, 2);
        assert_eq!(id2.index(), 0);
        assert_eq!(id2.generation(), 2);
        assert_ne!(id1, id2); // Mismo índice, diferente generación
    }

    #[test]
    fn test_is_valid_stale_detection() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        assert!(store.is_valid(id));

        store.despawn(id).unwrap();
        assert!(!store.is_valid(id));
    }

    #[test]
    fn test_generational_spawn_despawn() {
        let mut store = EntityStore::new(100);

        let id1 = store.spawn().unwrap(); // (0, 1)
        store.despawn(id1).unwrap();
        let id2 = store.spawn().unwrap(); // (0, 2) - reutiliza índice

        assert_eq!(id2.index(), 0);
        assert_eq!(id2.generation(), 2);
        assert!(!store.is_valid(id1)); // Stale
        assert!(store.is_valid(id2)); // Válido
    }

    #[test]
    fn test_compaction_reduces_fragmentation() {
        let mut store = EntityStore::new(100);

        // Crear 100 entidades
        let ids: Vec<_> = (0..50).map(|_| store.spawn().unwrap()).collect();

        // Borrar 25 (fragmentar)
        for id in ids.iter().take(25) {
            store.despawn(*id).unwrap();
        }

        let frag_before = store.fragmentation();
        assert!(frag_before > 0.2); // >20% huecos

        store.compact();

        let frag_after = store.fragmentation();
        assert!(frag_after < frag_before); // Menos huecos
    }

    #[test]
    fn test_compaction_preserves_values() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(100.0, 200.0)).unwrap();

        let before = store.position(id).unwrap();

        store.compact();

        let after = store.position(id).unwrap();
        assert_eq!(before, after); // Valor preservado
    }

    #[test]
    fn test_accessors_return_option() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        // Getter
        let pos = store.position(id);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), Vec2::new(0.0, 0.0));

        // Setter
        let result = store.set_position(id, Vec2::new(50.0, 50.0));
        assert!(result.is_ok());

        // Verificar cambio
        let pos = store.position(id);
        assert_eq!(pos.unwrap(), Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_access_invalid_entity_returns_none() {
        let store = EntityStore::new(100);
        let fake_id = EntityId::new(999, 1);

        assert!(store.position(fake_id).is_none());
    }
}
