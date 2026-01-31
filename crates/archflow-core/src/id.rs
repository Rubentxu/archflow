// ═══════════════════════════════════════════════════════════════════════════════
// EntityId - Generational Index (Dense Entity Storage)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 4
//
// EntityId is a packed u32 containing:
// - Index (24 bits): Which slot in the dense arrays
// - Generation (8 bits): Validation counter to detect dangling references
//
// Benefits:
// - O(1) entity access via direct array indexing
// - Cache-friendly (dense storage, no gaps)
// - Detects use-after-free bugs via generation check
// - Only 4 bytes per handle (vs 16+ for UUID/GUID)
// ═══════════════════════════════════════════════════════════════════════════════

use static_assertions::const_assert;

/// Entity identifier with generational validation
///
/// Layout (bits 0-31):
/// ```text
/// | Generation (8) | Index (24) |
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EntityId(u32);

// Ensure EntityId is exactly 4 bytes
const_assert!(std::mem::size_of::<EntityId>() == 4);

impl EntityId {
    /// Maximum index value (2^24 - 1 = 16,777,215)
    pub const MAX_INDEX: u32 = (1 << 24) - 1;

    /// Maximum generation value (2^8 - 1 = 255)
    pub const MAX_GENERATION: u8 = 255;

    /// Bit mask for extracting the index portion
    const INDEX_MASK: u32 = 0x00FF_FFFF;

    /// Bit shift for extracting the generation portion
    const GENERATION_SHIFT: u32 = 24;

    /// Create a new EntityId from raw value
    #[inline(always)]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Create an EntityId from index and generation
    #[inline(always)]
    pub const fn from_parts(index: Index, generation: Generation) -> Self {
        Self((generation.0 as u32) << Self::GENERATION_SHIFT | (index.0 & Self::INDEX_MASK))
    }

    /// Extract the index portion
    #[inline(always)]
    pub const fn index(self) -> Index {
        Index(self.0 & Self::INDEX_MASK)
    }

    /// Extract the generation portion
    #[inline(always)]
    pub const fn generation(self) -> Generation {
        Generation((self.0 >> Self::GENERATION_SHIFT) as u8)
    }

    /// Check if this EntityId is valid (not the invalid sentinel)
    #[inline(always)]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }

    /// Get the raw u32 value
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Create an invalid EntityId (sentinel value)
    #[inline(always)]
    pub const fn invalid() -> Self {
        Self(u32::MAX)
    }
}

impl core::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EntityId({}:{})", self.index().0, self.generation().0)
    }
}

/// Index portion of EntityId (which slot in the dense arrays)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Index(pub u32);

/// Generation portion of EntityId (validation counter)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Generation(pub u8);

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entityid_size() {
        assert_eq!(std::mem::size_of::<EntityId>(), 4);
    }

    #[test]
    fn test_entityid_from_parts() {
        let id = EntityId::from_parts(Index(100), Generation(5));
        assert_eq!(id.index().0, 100);
        assert_eq!(id.generation().0, 5);
    }

    #[test]
    fn test_entityid_invalid() {
        let invalid = EntityId::invalid();
        assert!(!invalid.is_valid());
        assert_eq!(invalid.as_u32(), u32::MAX);
    }

    #[test]
    fn test_entityid_max_index() {
        assert_eq!(EntityId::MAX_INDEX, 0x00FF_FFFF);
    }

    #[test]
    fn test_entityid_max_generation() {
        assert_eq!(EntityId::MAX_GENERATION, 255);
    }

    #[test]
    fn test_entityid_roundtrip() {
        let original = EntityId::from_parts(Index(12345), Generation(42));
        let reconstructed = EntityId::new(original.as_u32());
        assert_eq!(original, reconstructed);
    }
}
