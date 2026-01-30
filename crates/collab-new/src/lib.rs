//! # ArchFlow Collaboration - Bounded Context for Real-time Collaboration

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub use archflow_core::EntityId;

/// Site ID for CRDT operations
pub type SiteId = u64;

/// CRDT operation type
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CrdtOp {
    Insert { id: EntityId, site_id: SiteId },
    Delete { id: EntityId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collab_crate_exists() {
        let site_id: SiteId = 1;
        assert_eq!(site_id, 1);
    }
}
