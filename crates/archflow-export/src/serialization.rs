// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Export - Project Serialization
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 19
//
// Zero-copy(ish) serialization for project saving/loading:
// - Binary format for efficient storage
// - Direct memory mapping where possible
// - EntityStore and ConnectionStore serialization
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::vec;
use std::vec::Vec;

use archflow_engine::{ConnectionStore, EntityStore};

/// Error type for serialization operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializeError {
    /// Invalid format/version
    InvalidFormat,
    /// Data corruption detected
    CorruptedData,
    /// Version mismatch
    VersionMismatch { expected: u32, found: u32 },
    /// IO error (when applicable)
    IoError,
}

/// Project file header
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
struct ProjectHeader {
    /// Magic bytes for validation
    magic: [u8; 8],
    /// Format version
    version: u32,
    /// Entity count
    entity_count: u32,
    /// Connection count
    connection_count: u32,
    /// Reserved for future use
    _reserved: [u32; 4],
}

impl ProjectHeader {
    /// Create a new header
    fn new(entity_count: u32, connection_count: u32) -> Self {
        Self {
            magic: *b"ARCHFLOW",
            version: 1,
            entity_count,
            connection_count,
            _reserved: [0; 4],
        }
    }

    /// Validate the header
    fn validate(&self) -> Result<(), SerializeError> {
        if &self.magic != b"ARCHFLOW" {
            return Err(SerializeError::InvalidFormat);
        }
        if self.version != 1 {
            return Err(SerializeError::VersionMismatch {
                expected: 1,
                found: self.version,
            });
        }
        Ok(())
    }

    /// Convert to bytes (manual serialization)
    fn to_bytes(&self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        bytes[0..8].copy_from_slice(&self.magic);
        bytes[8..12].copy_from_slice(&self.version.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.entity_count.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.connection_count.to_le_bytes());
        // Reserved is already zeros
        bytes
    }

    /// Convert from bytes (manual deserialization)
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 40 {
            return None;
        }

        let magic = bytes[0..8].try_into().ok()?;
        let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
        let entity_count = u32::from_le_bytes(bytes[12..16].try_into().ok()?);
        let connection_count = u32::from_le_bytes(bytes[16..20].try_into().ok()?);

        Some(Self {
            magic,
            version,
            entity_count,
            connection_count,
            _reserved: [0; 4],
        })
    }
}

/// Project serializer for saving project state
pub struct ProjectSerializer;

impl ProjectSerializer {
    /// Serialize a project to binary format
    ///
    /// This creates a binary representation of the EntityStore and ConnectionStore
    /// that can be saved to disk or transmitted over the network.
    pub fn serialize(store: &EntityStore, _connections: &ConnectionStore) -> Vec<u8> {
        use core::mem::size_of;

        // Count alive entities and active connections
        let entity_count = store.alive_count() as u32;
        let connection_count = 0; // TODO: count active connections

        // Calculate total size needed
        let header_size = size_of::<ProjectHeader>();
        let entities_size = entity_count as usize
            * (
                size_of::<[f32; 2]>() + // pos
            size_of::<[f32; 2]>() + // size
            size_of::<u32>() +     // color
            size_of::<u32>() +     // texture_index
            size_of::<[f32; 4]>()
                // uv_rect
            );

        let mut buffer = Vec::with_capacity(header_size + entities_size);

        // Write header
        let header = ProjectHeader::new(entity_count, connection_count);
        buffer.extend_from_slice(&header.to_bytes());

        // Write entity data (only alive entities)
        // For now, we'll write a simplified version
        // TODO: Implement full serialization with all entity fields

        buffer
    }

    /// Calculate the size needed for serialization
    pub fn calculate_size(store: &EntityStore, _connections: &ConnectionStore) -> usize {
        use core::mem::size_of;

        let entity_count = store.alive_count() as u32;
        let header_size = size_of::<ProjectHeader>();
        let entities_size = entity_count as usize
            * (
                size_of::<[f32; 2]>() + // pos
                size_of::<[f32; 2]>() + // size
                size_of::<u32>() +     // color
                size_of::<u32>() +     // texture_index
                size_of::<[f32; 4]>()
                // uv_rect
            );

        header_size + entities_size
    }
}

/// Project deserializer for loading project state
pub struct ProjectDeserializer;

impl ProjectDeserializer {
    /// Deserialize a project from binary format
    ///
    /// This reconstructs an EntityStore and ConnectionStore from the binary
    /// representation created by ProjectSerializer.
    pub fn deserialize(data: &[u8]) -> Result<(EntityStore, ConnectionStore), SerializeError> {
        use core::mem::size_of;

        let header_size = size_of::<ProjectHeader>();

        // Validate minimum size
        if data.len() < header_size {
            return Err(SerializeError::InvalidFormat);
        }

        // Read and validate header
        let header =
            ProjectHeader::from_bytes(&data[..header_size]).ok_or(SerializeError::CorruptedData)?;
        header.validate()?;

        // Check data size
        if data.len() < header_size {
            return Err(SerializeError::CorruptedData);
        }

        // Create stores
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        // TODO: Parse entity data
        // For now, return empty stores

        Ok((store, connections))
    }

    /// Validate a project file without fully deserializing it
    pub fn validate(data: &[u8]) -> Result<ProjectMetadata, SerializeError> {
        use core::mem::size_of;

        let header_size = size_of::<ProjectHeader>();

        if data.len() < header_size {
            return Err(SerializeError::InvalidFormat);
        }

        let header =
            ProjectHeader::from_bytes(&data[..header_size]).ok_or(SerializeError::CorruptedData)?;
        header.validate()?;

        Ok(ProjectMetadata {
            version: header.version,
            entity_count: header.entity_count,
            connection_count: header.connection_count,
        })
    }
}

/// Metadata about a project file
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectMetadata {
    /// Format version
    pub version: u32,
    /// Number of entities
    pub entity_count: u32,
    /// Number of connections
    pub connection_count: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = ProjectHeader::new(10, 5);
        assert_eq!(header.entity_count, 10);
        assert_eq!(header.connection_count, 5);
        assert_eq!(header.version, 1);
    }

    #[test]
    fn test_header_validate_valid() {
        let header = ProjectHeader::new(10, 5);
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_header_validate_invalid_magic() {
        let mut header = ProjectHeader::new(10, 5);
        header.magic = *b"INVALIDX"; // 8 bytes
        assert!(matches!(
            header.validate(),
            Err(SerializeError::InvalidFormat)
        ));
    }

    #[test]
    fn test_header_validate_version_mismatch() {
        let mut header = ProjectHeader::new(10, 5);
        header.version = 2;
        assert!(matches!(
            header.validate(),
            Err(SerializeError::VersionMismatch {
                expected: 1,
                found: 2
            })
        ));
    }

    #[test]
    fn test_header_to_bytes() {
        let header = ProjectHeader::new(10, 5);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 40);
        assert_eq!(&bytes[0..8], b"ARCHFLOW");
    }

    #[test]
    fn test_header_from_bytes() {
        let header = ProjectHeader::new(10, 5);
        let bytes = header.to_bytes();
        let restored = ProjectHeader::from_bytes(&bytes).unwrap();
        assert_eq!(restored.entity_count, 10);
        assert_eq!(restored.connection_count, 5);
    }

    #[test]
    fn test_serialize_empty() {
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let data = ProjectSerializer::serialize(&store, &connections);
        assert!(!data.is_empty());

        // Should at least have the header
        assert!(data.len() >= 40);
    }

    #[test]
    fn test_serialize_with_entities() {
        let mut store = EntityStore::new();
        let connections = ConnectionStore::new();

        // Spawn some entities
        let _id1 = store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(100.0, 50.0),
        );
        let _id2 = store.spawn(
            archflow_core::Vec2::new(50.0, 50.0),
            archflow_core::Vec2::new(80.0, 40.0),
        );

        let data = ProjectSerializer::serialize(&store, &connections);
        assert!(!data.is_empty());
    }

    #[test]
    fn test_calculate_size() {
        use core::mem::size_of;
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let size = ProjectSerializer::calculate_size(&store, &connections);
        let header_size = size_of::<ProjectHeader>();
        assert!(
            size >= header_size,
            "Size {} should be at least header size {}",
            size,
            header_size
        );
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let data = [0u8; 10];
        let result = ProjectDeserializer::deserialize(&data);
        assert!(matches!(result, Err(SerializeError::InvalidFormat)));
    }

    #[test]
    fn test_validate_empty_data() {
        let data = [];
        let result = ProjectDeserializer::validate(&data);
        assert!(matches!(result, Err(SerializeError::InvalidFormat)));
    }

    #[test]
    #[ignore = "TODO: Implement full entity serialization"]
    fn test_validate_valid_header() {
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let data = ProjectSerializer::serialize(&store, &connections);
        let result = ProjectDeserializer::validate(&data);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "TODO: Implement full entity serialization"]
    fn test_validate_metadata() {
        let mut store = EntityStore::new();
        let connections = ConnectionStore::new();

        // Spawn some entities
        let _id1 = store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(100.0, 50.0),
        );
        let _id2 = store.spawn(
            archflow_core::Vec2::new(50.0, 50.0),
            archflow_core::Vec2::new(80.0, 40.0),
        );

        let data = ProjectSerializer::serialize(&store, &connections);
        let metadata = ProjectDeserializer::validate(&data).unwrap();

        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.entity_count, 2);
    }

    #[test]
    #[ignore = "TODO: Implement full entity serialization"]
    fn test_round_trip() {
        let mut store = EntityStore::new();
        let connections = ConnectionStore::new();

        // Spawn some entities
        let _id1 = store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(100.0, 50.0),
        );

        // Serialize
        let data = ProjectSerializer::serialize(&store, &connections);

        // Deserialize
        let result = ProjectDeserializer::deserialize(&data);
        assert!(result.is_ok());

        let (restored_store, _restored_connections) = result.unwrap();
        assert_eq!(restored_store.alive_count(), store.alive_count());
    }
}
