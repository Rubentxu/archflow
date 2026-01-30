//! Generational Entity ID for Type-Safe Entity References
//!
//! EntityId uses a 32-bit value combining:
//! - **Index** (24 bits): Position in SOA arrays (0..16,777,216)
//! - **Generation** (8 bits): Version to detect stale references
//!
//! ## Layout
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │ Generation (8 bits) │     Index (24 bits)    │
//! │   0xFF            │   0x00FFFFFF          │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust
//! use soa_entity::EntityId;
//!
//! let id1 = EntityId::new(100, 1);
//! assert_eq!(id1.index(), 100);
//! assert_eq!(id1.generation(), 1);
//!
//! let id2 = EntityId::new(100, 2);
//! assert_eq!(id2.index(), 100); // Same index
//! assert_ne!(id1, id2);           // Different generation
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

/// Maximum number of entities supported (2^24 - 1)
pub const MAX_ENTITIES: usize = 0x00FF_FFFF;

/// Index mask for extracting index from EntityId
const INDEX_MASK: u32 = 0x00FF_FFFF;

/// Generational entity identifier.
///
/// Combines an index (position in SOA arrays) with a generation counter
/// to prevent stale pointer bugs. When an entity is despawned and its index
/// is reused, the generation is incremented, making all old EntityIds invalid.
///
/// # Examples
///
/// ```
/// use soa_entity::EntityId;
///
/// let id1 = EntityId::new(5, 1);
/// assert_eq!(id1.index(), 5);
/// assert_eq!(id1.generation(), 1);
/// assert!(id1.is_valid());
///
/// let id2 = EntityId::new(5, 2);
/// assert_eq!(id2.index(), 5); // Reuses index
/// assert_eq!(id2.generation(), 2); // New generation
/// assert!(!id1.is_valid()); // Old ID now stale
/// assert!(id2.is_valid()); // New ID valid
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct EntityId(pub u32);

impl EntityId {
    /// Maximum number of entities supported (2^24 - 1)
    pub const MAX_ENTITIES: usize = MAX_ENTITIES;

    /// Creates a new EntityId from index and generation.
    ///
    /// # Arguments
    ///
    /// * `index` - Position in SOA arrays (0..16,777,216)
    /// * `generation` - Generation counter (0..255)
    ///
    /// # Panics
    ///
    /// Panics if index >= MAX_ENTITIES
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityId;
    ///
    /// let id = EntityId::new(100, 1);
    /// assert_eq!(id.index(), 100);
    /// assert_eq!(id.generation(), 1);
    /// ```
    #[inline]
    pub const fn new(index: usize, generation: u32) -> Self {
        assert!(index < MAX_ENTITIES, "EntityId index too large");
        Self((index as u32) | (generation << 24))
    }

    /// Extracts the index (position in SOA arrays).
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityId;
    ///
    /// let id = EntityId::new(100, 1);
    /// assert_eq!(id.index(), 100);
    /// ```
    #[inline]
    pub fn index(&self) -> usize {
        (self.0 & INDEX_MASK) as usize
    }

    /// Extracts the generation counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityId;
    ///
    /// let id = EntityId::new(100, 1);
    /// assert_eq!(id.generation(), 1);
    /// ```
    #[inline]
    pub fn generation(&self) -> u32 {
        self.0 >> 24
    }

    /// Checks if this EntityId is valid (not the NULL sentinel).
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityId;
    ///
    /// assert!(EntityId::new(100, 1).is_valid());
    /// assert!(!EntityId::NULL.is_valid());
    /// ```
    #[inline]
    pub fn is_valid(&self) -> bool {
        *self != Self::NULL
    }

    /// Null/sentinel EntityId representing no entity.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityId;
    ///
    /// let id = EntityId::NULL;
    /// assert!(!id.is_valid());
    /// assert_eq!(id.index(), 0);
    /// assert_eq!(id.generation(), 0);
    /// ```
    pub const NULL: Self = Self(0);

    /// Creates a new EntityId with the next generation for the same index.
    ///
    /// This is used internally when reusing an index after despawning.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityId;
    ///
    /// let id1 = EntityId::new(5, 1);
    /// let id2 = id1.next_generation();
    /// assert_eq!(id2.index(), 5);
    /// assert_eq!(id2.generation(), 2);
    /// ```
    #[inline]
    pub fn next_generation(&self) -> Self {
        Self::new(self.index(), self.generation() + 1)
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({}:{})", self.index(), self.generation())
    }
}

impl Default for EntityId {
    #[inline]
    fn default() -> Self {
        Self::NULL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_creation() {
        let id = EntityId::new(100, 1);
        assert_eq!(id.index(), 100);
        assert_eq!(id.generation(), 1);
    }

    #[test]
    fn test_entity_id_max_entities() {
        let id = EntityId::new(MAX_ENTITIES - 1, 1);
        assert_eq!(id.index(), MAX_ENTITIES - 1);
    }

    #[test]
    #[should_panic]
    fn test_entity_id_too_large_panics() {
        let _ = EntityId::new(MAX_ENTITIES, 1);
    }

    #[test]
    fn test_entity_id_null() {
        assert_eq!(EntityId::NULL.0, 0);
        assert!(!EntityId::NULL.is_valid());
    }

    #[test]
    fn test_entity_id_equality() {
        let id1 = EntityId::new(5, 1);
        let id2 = EntityId::new(5, 1);
        assert_eq!(id1, id2);

        let id3 = EntityId::new(5, 2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_entity_id_next_generation() {
        let id1 = EntityId::new(5, 1);
        let id2 = id1.next_generation();
        assert_eq!(id2.index(), 5);
        assert_eq!(id2.generation(), 2);
    }

    #[test]
    fn test_entity_id_hash() {
        use std::collections::HashSet;

        let id1 = EntityId::new(5, 1);
        let id2 = EntityId::new(5, 2);
        let id3 = EntityId::new(100, 1);

        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id3);

        assert_eq!(set.len(), 3);
    }
}
