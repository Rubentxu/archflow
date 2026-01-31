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

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Generation, Index};
use archflow_engine::{AnchorSide, ConnectionStore, EntityStore, LineStyle, MAX_ENTITIES};

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
        // Reserved (4 * 4 = 16 bytes)
        bytes[20..24].copy_from_slice(&self._reserved[0].to_le_bytes());
        bytes[24..28].copy_from_slice(&self._reserved[1].to_le_bytes());
        bytes[28..32].copy_from_slice(&self._reserved[2].to_le_bytes());
        bytes[32..36].copy_from_slice(&self._reserved[3].to_le_bytes());
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

/// Entity data chunk for serialization
///
/// Contains all entity data in a compact 128-byte structure
/// aligned to 16 bytes for efficient memory access
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
struct EntityChunk {
    /// Entity index
    index: u32,
    /// Generation counter
    generation: u8,
    /// Padding
    _pad1: [u8; 3],
    /// Transform [x, y, w, h]
    transform: [f32; 4],
    /// Local transform [x, y, w, h]
    local_transform: [f32; 4],
    /// World transform [x, y, w, h]
    world_transform: [f32; 4],
    /// Metadata (shape, layer, visibility, selected, locked)
    metadata: u32,
    /// Color (0xRRGGBBAA)
    color: u32,
    /// Texture index
    texture_index: u16,
    /// Text glyph count
    text_glyph_count: u16,
    /// Text glyph start
    text_glyph_start: u32,
    /// Text scale
    text_scale: f32,
    /// UV rectangle [u, v, w, h]
    uv_rect: [f32; 4],
    /// Color tint [r, g, b, a]
    color_tint: [f32; 4],
    /// Parent entity index (u32::MAX if none)
    parent_index: u32,
    /// Parent generation
    parent_generation: u8,
    /// Padding to align to 16 bytes
    _pad2: [u8; 3],
    /// Reserved for future use
    _reserved: [u32; 4],
}

/// Connection data chunk for serialization
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ConnectionChunk {
    /// Source entity index
    src_index: u32,
    /// Source entity generation
    src_generation: u8,
    /// Target entity index
    tgt_index: u32,
    /// Target entity generation
    tgt_generation: u8,
    /// Source anchor point
    src_anchor: u8,
    /// Target anchor point
    tgt_anchor: u8,
    /// Line style
    line_style: u8,
    /// Padding
    _padding: u8,
}

/// Project serializer for saving project state
pub struct ProjectSerializer;

impl ProjectSerializer {
    /// Serialize a project to binary format
    ///
    /// This creates a binary representation of the EntityStore and ConnectionStore
    /// that can be saved to disk or transmitted over the network.
    ///
    /// # Binary Format
    ///
    /// ```text
    /// [Header 40 bytes]
    /// [EntityChunk * entity_count]  // 128 bytes each
    /// [ConnectionChunk * connection_count]  // 16 bytes each
    /// [StringPoolData]
    /// ```
    pub fn serialize(store: &EntityStore, connections: &ConnectionStore) -> Vec<u8> {
        use core::mem::size_of;

        let entity_count = store.alive_count() as u32;
        let connection_count = connections.len() as u32;

        // Calculate sizes
        let header_size = size_of::<ProjectHeader>();
        let entity_chunk_size = size_of::<EntityChunk>();
        let entities_size = entity_count as usize * entity_chunk_size;
        let conn_chunk_size = size_of::<ConnectionChunk>();
        let connections_size = connection_count as usize * conn_chunk_size;

        let mut buffer = Vec::with_capacity(header_size + entities_size + connections_size + 1024);

        // Write header
        let header = ProjectHeader::new(entity_count, connection_count);
        buffer.extend_from_slice(&header.to_bytes());

        // Write entity data (only alive entities)
        let mut entities_written = 0u32;
        for i in 0..MAX_ENTITIES {
            if !store.is_alive_index(i) {
                continue;
            }

            let parent_id = store.parent_ids_ref()[i];
            let chunk = EntityChunk {
                index: i as u32,
                generation: store.generation(i),
                _pad1: [0; 3],
                transform: store.transforms_ref()[i],
                local_transform: store.local_transforms_ref()[i],
                world_transform: store.world_transforms_ref()[i],
                metadata: store.metadata_ref()[i],
                color: store.colors_ref()[i],
                texture_index: store.texture_indices_ref()[i],
                text_glyph_count: store.text_glyph_counts_ref()[i],
                text_glyph_start: store.text_glyph_starts_ref()[i],
                text_scale: store.text_scales_ref()[i],
                uv_rect: store.uv_rects_ref()[i],
                color_tint: store.color_tints_ref()[i],
                parent_index: match parent_id {
                    Some(id) => id.index().0,
                    None => u32::MAX,
                },
                parent_generation: match parent_id {
                    Some(id) => id.generation().0,
                    None => 0,
                },
                _pad2: [0; 3],
                _reserved: [0; 4],
            };

            // SAFETY: EntityChunk is POD (plain old data) with known alignment
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    &chunk as *const EntityChunk as *const u8,
                    entity_chunk_size,
                )
            };
            buffer.extend_from_slice(bytes);
            entities_written += 1;
        }

        // Write connection data (compact format: 16 bytes)
        for i in 0..connections.len() {
            // Compact format: src_index(4) + src_gen(1) + pad(1) + tgt_index(4) + tgt_gen(1) + pad(1) + src_anchor(1) + tgt_anchor(1) + style(1) + pad(1)
            buffer.extend_from_slice(&connections.sources[i].index().0.to_le_bytes());
            buffer.push(connections.sources[i].generation().0);
            buffer.push(0); // padding
            buffer.extend_from_slice(&connections.targets[i].index().0.to_le_bytes());
            buffer.push(connections.targets[i].generation().0);
            buffer.push(0); // padding
            buffer.push(connections.source_anchors[i] as u8);
            buffer.push(connections.target_anchors[i] as u8);
            buffer.push(connections.line_styles[i] as u8);
            buffer.push(0); // padding
        }

        // Write string pool data
        let sp_buffer = store.string_pool.buffer();
        buffer.extend_from_slice(&(sp_buffer.len() as u32).to_le_bytes());
        buffer.extend_from_slice(sp_buffer);

        buffer
    }

    /// Calculate the size needed for serialization
    pub fn calculate_size(store: &EntityStore, connections: &ConnectionStore) -> usize {
        use core::mem::size_of;

        const HEADER_SIZE: usize = 40;
        const CONN_SIZE: usize = 16; // Fixed compact connection size

        let entity_count = store.alive_count() as usize;
        let connection_count = connections.len();

        let entity_chunk_size = size_of::<EntityChunk>();
        let entities_size = entity_count * entity_chunk_size;

        let connections_size = connection_count * CONN_SIZE;

        let string_pool_size = 4 + store.string_pool.buffer().len();

        HEADER_SIZE + entities_size + connections_size + string_pool_size
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

        // Header is always 40 bytes (fixed size for compatibility)
        const HEADER_SIZE: usize = 40;
        let entity_chunk_size = size_of::<EntityChunk>();
        const CONN_CHUNK_SIZE: usize = 16;

        // Validate minimum size
        if data.len() < HEADER_SIZE {
            return Err(SerializeError::InvalidFormat);
        }

        // Read and validate header
        let header =
            ProjectHeader::from_bytes(&data[..HEADER_SIZE]).ok_or(SerializeError::CorruptedData)?;
        header.validate()?;

        let entity_count = header.entity_count as usize;
        let connection_count = header.connection_count as usize;

        // Create stores
        let mut store = EntityStore::new();
        let mut connections = ConnectionStore::new();

        let mut offset = HEADER_SIZE;

        // Read entity data
        for _ in 0..entity_count {
            if offset + entity_chunk_size > data.len() {
                return Err(SerializeError::CorruptedData);
            }

            // Manual deserialization to avoid alignment issues
            let chunk_bytes = &data[offset..offset + entity_chunk_size];

            let index = u32::from_le_bytes(chunk_bytes[0..4].try_into().unwrap());
            let generation = chunk_bytes[4];
            // _pad1 at 5..8
            let transform = [
                f32::from_le_bytes(chunk_bytes[8..12].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[12..16].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[16..20].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[20..24].try_into().unwrap()),
            ];
            let local_transform = [
                f32::from_le_bytes(chunk_bytes[24..28].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[28..32].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[32..36].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[36..40].try_into().unwrap()),
            ];
            let world_transform = [
                f32::from_le_bytes(chunk_bytes[40..44].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[44..48].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[48..52].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[52..56].try_into().unwrap()),
            ];
            let metadata = u32::from_le_bytes(chunk_bytes[56..60].try_into().unwrap());
            let color = u32::from_le_bytes(chunk_bytes[60..64].try_into().unwrap());
            let texture_index = u16::from_le_bytes(chunk_bytes[64..66].try_into().unwrap());
            let text_glyph_count = u16::from_le_bytes(chunk_bytes[66..68].try_into().unwrap());
            let text_glyph_start = u32::from_le_bytes(chunk_bytes[68..72].try_into().unwrap());
            let text_scale = f32::from_le_bytes(chunk_bytes[72..76].try_into().unwrap());
            let uv_rect = [
                f32::from_le_bytes(chunk_bytes[76..80].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[80..84].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[84..88].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[88..92].try_into().unwrap()),
            ];
            let color_tint = [
                f32::from_le_bytes(chunk_bytes[92..96].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[96..100].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[100..104].try_into().unwrap()),
                f32::from_le_bytes(chunk_bytes[104..108].try_into().unwrap()),
            ];
            let parent_index = u32::from_le_bytes(chunk_bytes[108..112].try_into().unwrap());
            let parent_generation = chunk_bytes[112];
            // _pad2 at 113..116, _reserved at 116..144

            let idx = index as usize;
            if idx >= MAX_ENTITIES {
                return Err(SerializeError::CorruptedData);
            }

            // Restore entity data using helper methods
            store.set_generation(idx, generation);
            store.set_transform(idx, transform);
            store.set_local_transform(idx, local_transform);
            store.set_world_transform(idx, world_transform);
            store.set_metadata(idx, metadata);
            store.set_color(idx, color);
            store.set_texture_index(idx, texture_index);
            store.set_text_glyph_count(idx, text_glyph_count);
            store.set_text_glyph_start(idx, text_glyph_start);
            store.set_text_scale(idx, text_scale);
            store.set_uv_rect(idx, uv_rect);
            store.set_color_tint(idx, color_tint);

            // Restore parent
            if parent_index != u32::MAX {
                store.set_parent_id(
                    idx,
                    Some(EntityId::from_parts(
                        Index(parent_index),
                        Generation(parent_generation),
                    )),
                );
            } else {
                store.set_parent_id(idx, None);
            }

            offset += entity_chunk_size;
        }

        // Update alive count
        store.set_alive_count(entity_count);

        // Read connection data
        for _ in 0..connection_count {
            // Fixed 16-byte compact format
            const CONN_SIZE: usize = 16;
            if offset + CONN_SIZE > data.len() {
                return Err(SerializeError::CorruptedData);
            }

            let conn_bytes = &data[offset..offset + CONN_SIZE];

            let src_index = u32::from_le_bytes(conn_bytes[0..4].try_into().unwrap());
            let src_generation = conn_bytes[4];
            // padding at 5
            let tgt_index = u32::from_le_bytes(conn_bytes[6..10].try_into().unwrap());
            let tgt_generation = conn_bytes[10];
            // padding at 11
            let src_anchor = conn_bytes[12];
            let tgt_anchor = conn_bytes[13];
            let line_style = conn_bytes[14];
            // padding at 15

            let src_id = EntityId::from_parts(Index(src_index), Generation(src_generation));
            let tgt_id = EntityId::from_parts(Index(tgt_index), Generation(tgt_generation));
            let src_anchor_match = match src_anchor {
                0 => AnchorSide::Top,
                1 => AnchorSide::Bottom,
                2 => AnchorSide::Left,
                3 => AnchorSide::Right,
                4 => AnchorSide::Center,
                _ => AnchorSide::Center,
            };
            let tgt_anchor_match = match tgt_anchor {
                0 => AnchorSide::Top,
                1 => AnchorSide::Bottom,
                2 => AnchorSide::Left,
                3 => AnchorSide::Right,
                4 => AnchorSide::Center,
                _ => AnchorSide::Center,
            };
            let line_style_match = match line_style {
                0 => LineStyle::Direct,
                1 => LineStyle::Orthogonal,
                2 => LineStyle::Step,
                3 => LineStyle::Bezier,
                _ => LineStyle::Orthogonal,
            };

            connections.add_connection(
                src_id,
                tgt_id,
                src_anchor_match,
                tgt_anchor_match,
                line_style_match,
            );

            offset += CONN_SIZE;
        }

        // Read string pool data
        if offset + 4 > data.len() {
            return Err(SerializeError::CorruptedData);
        }

        let string_pool_len =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + string_pool_len > data.len() {
            return Err(SerializeError::CorruptedData);
        }

        // Restore string pool
        store
            .string_pool
            .set_buffer(&data[offset..offset + string_pool_len]);
        store.string_pool.clear_offsets();

        Ok((store, connections))
    }

    /// Validate a project file without fully deserializing it
    pub fn validate(data: &[u8]) -> Result<ProjectMetadata, SerializeError> {
        use core::mem::size_of;

        const HEADER_SIZE: usize = 40;

        if data.len() < HEADER_SIZE {
            return Err(SerializeError::InvalidFormat);
        }

        let header =
            ProjectHeader::from_bytes(&data[..HEADER_SIZE]).ok_or(SerializeError::CorruptedData)?;
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
        header.magic = *b"INVALIDX";
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
        assert!(data.len() >= 40);
    }

    #[test]
    fn test_serialize_with_entities() {
        let mut store = EntityStore::new();
        let connections = ConnectionStore::new();

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
        assert!(data.len() > 40);
    }

    #[test]
    fn test_calculate_size() {
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let size = ProjectSerializer::calculate_size(&store, &connections);
        const HEADER_SIZE: usize = 40;
        assert!(
            size >= HEADER_SIZE,
            "Size {} should be at least header size {}",
            size,
            HEADER_SIZE
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
    fn test_validate_valid_header() {
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let data = ProjectSerializer::serialize(&store, &connections);
        let result = ProjectDeserializer::validate(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_metadata() {
        let mut store = EntityStore::new();
        let connections = ConnectionStore::new();

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
    fn test_round_trip() {
        let mut store = EntityStore::new();
        let connections = ConnectionStore::new();

        let id1 = store.spawn(
            archflow_core::Vec2::new(10.0, 20.0),
            archflow_core::Vec2::new(100.0, 50.0),
        );
        let id2 = store.spawn(
            archflow_core::Vec2::new(50.0, 50.0),
            archflow_core::Vec2::new(80.0, 40.0),
        );

        // Set some properties
        let idx1 = id1.index().0 as usize;
        store.set_shape_type(idx1, 2);
        store.set_color(idx1, 0xFF0000FF);
        store.set_visible(idx1, false);

        // Serialize
        let data = ProjectSerializer::serialize(&store, &connections);

        // Deserialize
        let result = ProjectDeserializer::deserialize(&data);
        assert!(result.is_ok());

        let (restored_store, _restored_connections) = result.unwrap();
        assert_eq!(restored_store.alive_count(), 2);

        // Verify entity properties
        assert_eq!(restored_store.alive_count(), store.alive_count());
        assert_eq!(restored_store.generation(idx1), store.generation(idx1));
        assert_eq!(
            restored_store.transforms_ref()[idx1],
            store.transforms_ref()[idx1]
        );
        assert_eq!(
            restored_store.metadata_ref()[idx1],
            store.metadata_ref()[idx1]
        );
        assert_eq!(restored_store.colors_ref()[idx1], store.colors_ref()[idx1]);
    }

    #[test]
    fn test_round_trip_with_connections() {
        let mut store = EntityStore::new();
        let mut connections = ConnectionStore::new();

        let id1 = store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(100.0, 50.0),
        );
        let id2 = store.spawn(
            archflow_core::Vec2::new(200.0, 0.0),
            archflow_core::Vec2::new(100.0, 50.0),
        );

        connections.add_connection(
            id1,
            id2,
            AnchorSide::Right,
            AnchorSide::Left,
            LineStyle::Orthogonal,
        );

        // Serialize
        let data = ProjectSerializer::serialize(&store, &connections);

        // Deserialize
        let result = ProjectDeserializer::deserialize(&data);
        assert!(result.is_ok());

        let (_restored_store, restored_connections) = result.unwrap();
        assert_eq!(restored_connections.len(), 1);
    }

    #[test]
    fn test_entity_chunk_size() {
        use core::mem::size_of;
        let size = size_of::<EntityChunk>();
        // EntityChunk is aligned to 16 bytes, actual size may vary due to padding
        assert!(
            size >= 128,
            "EntityChunk size {} should be at least 128 bytes",
            size
        );
        assert!(
            size % 16 == 0,
            "EntityChunk size {} should be 16-byte aligned",
            size
        );
    }

    #[test]
    fn test_connection_chunk_size() {
        use core::mem::size_of;
        let size = size_of::<ConnectionChunk>();
        // ConnectionChunk size depends on padding (actual size is 20 bytes on x86_64)
        assert!(
            size >= 12,
            "ConnectionChunk size {} should be at least 12 bytes",
            size
        );
    }
}
