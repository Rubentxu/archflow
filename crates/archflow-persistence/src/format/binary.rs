// ═══════════════════════════════════════════════════════════════════════════════
// Binary Serialization - Optimized format for large documents
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use archflow_core::EntityId;
use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
use std::vec::Vec;

use crate::{
    ArchitectureData, Document, EntityData, PersistenceError, PersistenceResult, PropValue,
    SpatialIndexData, StoreSnapshot, TextData,
};

// ═══════════════════════════════════════════════════════════════════════════════
// BINARY FORMAT HEADER
// ═══════════════════════════════════════════════════════════════════════════════

/// ArchFlow binary format magic number
pub const MAGIC_NUMBER: u32 = 0xAF01_0001;

/// Maximum supported binary format version
pub const MAX_FORMAT_VERSION: u32 = 1;

#[repr(u32)]
enum BinaryFormatType {
    Raw = MAGIC_NUMBER,
    GzipCompressed = 0xAF02_0001,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BINARY HEADER
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BinaryHeader {
    magic: u32,
    version: u32,
    flags: u32,
}

impl BinaryHeader {
    fn new() -> Self {
        Self {
            magic: MAGIC_NUMBER,
            version: 1,
            flags: 0,
        }
    }

    fn with_compression(mut self) -> Self {
        self.magic = BinaryFormatType::GzipCompressed as u32;
        self.flags |= 0x01; // Set compression flag
        self
    }

    fn is_compressed(&self) -> bool {
        (self.flags & 0x01) != 0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialize document to binary format
pub fn to_binary(document: &Document) -> PersistenceResult<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write header
    let header = BinaryHeader::new();
    buffer
        .write_u32::<LE>(header.magic)
        .map_err(bincode_io_err)?;
    buffer
        .write_u32::<LE>(header.version)
        .map_err(bincode_io_err)?;
    buffer
        .write_u32::<LE>(header.flags)
        .map_err(bincode_io_err)?;

    // Write schema version
    buffer
        .write_u32::<LE>(document.schema.version.as_u32())
        .map_err(bincode_io_err)?;

    // Write document metadata
    write_string(&mut buffer, &document.meta.title).map_err(bincode_io_err)?;
    write_string(&mut buffer, &document.meta.description).map_err(bincode_io_err)?;
    write_optional_string(&mut buffer, &document.meta.author).map_err(bincode_io_err)?;
    write_string(&mut buffer, &document.meta.created_at).map_err(bincode_io_err)?;
    write_string(&mut buffer, &document.meta.modified_at).map_err(bincode_io_err)?;
    write_string(&mut buffer, &document.meta.app_version).map_err(bincode_io_err)?;

    // Write custom metadata count and entries
    let custom_count = document.meta.custom.len() as u32;
    buffer
        .write_u32::<LE>(custom_count)
        .map_err(bincode_io_err)?;
    for (key, value) in &document.meta.custom {
        write_string(&mut buffer, key).map_err(bincode_io_err)?;
        write_string(&mut buffer, value).map_err(bincode_io_err)?;
    }

    // Write store snapshot
    write_store(&mut buffer, &document.store)?;

    // Write spatial index (optional)
    let has_spatial = document.spatial_index.is_some() as u8;
    buffer.write_u8(has_spatial).map_err(bincode_io_err)?;
    if let Some(ref spatial) = document.spatial_index {
        write_spatial(&mut buffer, spatial)?;
    }

    // Logic wiring not yet implemented for binary format
    buffer.write_u8(0).map_err(bincode_io_err)?;

    Ok(buffer)
}

fn write_store<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    store: &StoreSnapshot,
) -> Result<(), PersistenceError> {
    buffer
        .write_u32::<LE>(store.version)
        .map_err(bincode_io_err)?;
    buffer
        .write_u32::<LE>(store.entity_count)
        .map_err(bincode_io_err)?;

    // Write entity count
    buffer
        .write_u32::<LE>(store.entities.len() as u32)
        .map_err(bincode_io_err)?;

    // Write each entity
    for entity in &store.entities {
        write_entity(buffer, entity)?;
    }

    Ok(())
}

fn write_entity<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    entity: &EntityData,
) -> Result<(), PersistenceError> {
    // Entity ID as u32
    buffer
        .write_u32::<LE>(entity.id.as_u32())
        .map_err(bincode_io_err)?;

    // Parent ID (0xFFFFFFFF if None)
    match entity.parent_id {
        Some(id) => buffer
            .write_u32::<LE>(id.as_u32())
            .map_err(bincode_io_err)?,
        None => buffer.write_u32::<LE>(0xFFFFFFFF).map_err(bincode_io_err)?,
    }

    // Transform
    for v in &entity.transform {
        buffer.write_f32::<LE>(*v).map_err(bincode_io_err)?;
    }

    // World transform
    for v in &entity.world_transform {
        buffer.write_f32::<LE>(*v).map_err(bincode_io_err)?;
    }

    // Metadata
    buffer
        .write_u32::<LE>(entity.metadata)
        .map_err(bincode_io_err)?;

    // Color
    buffer
        .write_u32::<LE>(entity.color)
        .map_err(bincode_io_err)?;

    // Texture index
    buffer
        .write_u16::<LE>(entity.texture_index)
        .map_err(bincode_io_err)?;

    // Color tint
    for v in &entity.color_tint {
        buffer.write_f32::<LE>(*v).map_err(bincode_io_err)?;
    }

    // Text (optional)
    let has_text = entity.text.is_some() as u8;
    buffer.write_u8(has_text).map_err(bincode_io_err)?;
    if let Some(ref text) = entity.text {
        write_text(buffer, text)?;
    }

    // Arch data (optional)
    let has_arch = entity.arch_data.is_some() as u8;
    buffer.write_u8(has_arch).map_err(bincode_io_err)?;
    if let Some(ref arch) = entity.arch_data {
        write_arch(buffer, arch)?;
    }

    // Properties
    write_props(buffer, &entity.props)?;

    Ok(())
}

fn write_text<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    text: &TextData,
) -> Result<(), PersistenceError> {
    write_string(buffer, &text.content).map_err(bincode_io_err)?;
    buffer.write_f32::<LE>(text.scale).map_err(bincode_io_err)?;
    buffer
        .write_u16::<LE>(text.glyph_count)
        .map_err(bincode_io_err)?;
    Ok(())
}

fn write_arch<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    arch: &ArchitectureData,
) -> Result<(), PersistenceError> {
    write_string(buffer, &arch.name).map_err(bincode_io_err)?;
    buffer.write_u8(arch.c4_level).map_err(bincode_io_err)?;
    buffer.write_u8(arch.entity_type).map_err(bincode_io_err)?;
    buffer
        .write_u8(arch.cloud_provider)
        .map_err(bincode_io_err)?;
    write_string(buffer, &arch.technology).map_err(bincode_io_err)?;
    write_string(buffer, &arch.description).map_err(bincode_io_err)?;
    Ok(())
}

fn write_props<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    props: &std::collections::BTreeMap<String, PropValue>,
) -> Result<(), PersistenceError> {
    buffer
        .write_u32::<LE>(props.len() as u32)
        .map_err(bincode_io_err)?;

    for (key, value) in props {
        write_string(buffer, key).map_err(bincode_io_err)?;
        write_prop_value(buffer, value)?;
    }

    Ok(())
}

fn write_prop_value<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    value: &PropValue,
) -> Result<(), PersistenceError> {
    match value {
        PropValue::Null => {
            buffer.write_u8(0).map_err(bincode_io_err)?;
        }
        PropValue::Boolean(b) => {
            buffer.write_u8(1).map_err(bincode_io_err)?;
            buffer.write_u8(*b as u8).map_err(bincode_io_err)?;
        }
        PropValue::Number(n) => {
            buffer.write_u8(2).map_err(bincode_io_err)?;
            buffer.write_f64::<LE>(*n).map_err(bincode_io_err)?;
        }
        PropValue::String(s) => {
            buffer.write_u8(3).map_err(bincode_io_err)?;
            write_string(buffer, s).map_err(bincode_io_err)?;
        }
        PropValue::Array(arr) => {
            buffer.write_u8(4).map_err(bincode_io_err)?;
            buffer
                .write_u32::<LE>(arr.len() as u32)
                .map_err(bincode_io_err)?;
            for v in arr {
                write_prop_value(buffer, v)?;
            }
        }
        PropValue::Object(obj) => {
            buffer.write_u8(5).map_err(bincode_io_err)?;
            buffer
                .write_u32::<LE>(obj.len() as u32)
                .map_err(bincode_io_err)?;
            for (k, v) in obj {
                write_string(buffer, k).map_err(bincode_io_err)?;
                write_prop_value(buffer, v)?;
            }
        }
    }
    Ok(())
}

fn write_spatial<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    spatial: &SpatialIndexData,
) -> Result<(), PersistenceError> {
    buffer
        .write_f32::<LE>(spatial.cell_size)
        .map_err(bincode_io_err)?;
    buffer
        .write_usize::<LE>(spatial.cell_count)
        .map_err(bincode_io_err)?;

    buffer
        .write_u32::<LE>(spatial.cells.len() as u32)
        .map_err(bincode_io_err)?;
    for cell in &spatial.cells {
        buffer
            .write_u32::<LE>(cell.len() as u32)
            .map_err(bincode_io_err)?;
        for entity_id in cell {
            buffer
                .write_u32::<LE>(entity_id.as_u32())
                .map_err(bincode_io_err)?;
        }
    }

    Ok(())
}

fn write_string<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    s: &str,
) -> Result<(), std::io::Error> {
    let bytes = s.as_bytes();
    buffer.write_u32::<LE>(bytes.len() as u32)?;
    buffer.write_all(bytes)
}

fn write_optional_string<W: byteorder::WriteBytesExt>(
    buffer: &mut W,
    s: &Option<String>,
) -> Result<(), std::io::Error> {
    match s {
        Some(s) => {
            buffer.write_u8(1)?;
            write_string(buffer, s)
        }
        None => buffer.write_u8(0),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DESERIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Deserialize document from binary format
pub fn from_binary(data: &[u8]) -> PersistenceResult<Document> {
    if data.len() < 12 {
        return Err(PersistenceError::InvalidFormat(
            "Data too short for binary format".into(),
        ));
    }

    let mut cursor = Cursor::new(data);

    // Read header
    let magic = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    if magic != MAGIC_NUMBER {
        return Err(PersistenceError::InvalidFormat(format!(
            "Invalid magic number: 0x{magic:08X}"
        )));
    }

    let version = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    if version > MAX_FORMAT_VERSION {
        return Err(PersistenceError::VersionMismatch {
            expected: MAX_FORMAT_VERSION,
            found: version,
        });
    }

    let _flags = cursor.read_u32::<LE>().map_err(bincode_io_err)?;

    // Read schema version
    let _schema_version = cursor.read_u32::<LE>().map_err(bincode_io_err)?;

    // Read document metadata
    let title = read_string(&mut cursor)?;
    let description = read_string(&mut cursor)?;
    let author = read_optional_string(&mut cursor)?;
    let created_at = read_string(&mut cursor)?;
    let modified_at = read_string(&mut cursor)?;
    let app_version = read_string(&mut cursor)?;

    // Read custom metadata
    let custom_count = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let mut custom = std::collections::BTreeMap::new();
    for _ in 0..custom_count {
        let key = read_string(&mut cursor)?;
        let value = read_string(&mut cursor)?;
        custom.insert(key, value);
    }

    // Read store snapshot
    let store = read_store(&mut cursor)?;

    // Read spatial index
    let has_spatial = cursor.read_u8().map_err(bincode_io_err)? != 0;
    let spatial_index = if has_spatial {
        Some(read_spatial(&mut cursor)?)
    } else {
        None
    };

    // Logic wiring not yet implemented
    let _has_logic = cursor.read_u8().map_err(bincode_io_err)?;

    Ok(Document {
        schema: crate::document::Schema::current(),
        meta: crate::document::DocumentMeta {
            title,
            description,
            author,
            created_at,
            modified_at,
            app_version,
            custom,
        },
        store,
        spatial_index,
        logic_wiring: None,
    })
}

fn read_store<R: byteorder::ReadBytesExt>(
    cursor: &mut R,
) -> Result<StoreSnapshot, PersistenceError> {
    let version = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let entity_count = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let count = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;

    let mut entities = Vec::with_capacity(count);
    for _ in 0..count {
        entities.push(read_entity(cursor)?);
    }

    Ok(StoreSnapshot {
        version,
        entity_count,
        entities,
    })
}

fn read_entity<R: byteorder::ReadBytesExt>(cursor: &mut R) -> Result<EntityData, PersistenceError> {
    let id_raw = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let id = EntityId::new(id_raw);

    let parent_id_raw = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let parent_id = if parent_id_raw == 0xFFFFFFFF {
        None
    } else {
        Some(EntityId::new(parent_id_raw))
    };

    let mut transform = [0.0f32; 4];
    for v in &mut transform {
        *v = cursor.read_f32::<LE>().map_err(bincode_io_err)?;
    }

    let mut world_transform = [0.0f32; 4];
    for v in &mut world_transform {
        *v = cursor.read_f32::<LE>().map_err(bincode_io_err)?;
    }

    let metadata = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let color = cursor.read_u32::<LE>().map_err(bincode_io_err)?;
    let texture_index = cursor.read_u16::<LE>().map_err(bincode_io_err)?;

    let mut color_tint = [1.0f32; 4];
    for v in &mut color_tint {
        *v = cursor.read_f32::<LE>().map_err(bincode_io_err)?;
    }

    let has_text = cursor.read_u8().map_err(bincode_io_err)? != 0;
    let text = if has_text {
        Some(read_text(cursor)?)
    } else {
        None
    };

    let has_arch = cursor.read_u8().map_err(bincode_io_err)? != 0;
    let arch_data = if has_arch {
        Some(read_arch(cursor)?)
    } else {
        None
    };

    let props = read_props(cursor)?;

    Ok(EntityData {
        id,
        parent_id,
        transform,
        world_transform,
        metadata,
        color,
        texture_index,
        color_tint,
        text,
        arch_data,
        props,
    })
}

fn read_text<R: byteorder::ReadBytesExt>(cursor: &mut R) -> Result<TextData, PersistenceError> {
    let content = read_string(cursor)?;
    let scale = cursor.read_f32::<LE>().map_err(bincode_io_err)?;
    let glyph_count = cursor.read_u16::<LE>().map_err(bincode_io_err)?;
    Ok(TextData {
        content,
        scale,
        glyph_count,
    })
}

fn read_arch<R: byteorder::ReadBytesExt>(
    cursor: &mut R,
) -> Result<ArchitectureData, PersistenceError> {
    let name = read_string(cursor)?;
    let c4_level = cursor.read_u8().map_err(bincode_io_err)?;
    let entity_type = cursor.read_u8().map_err(bincode_io_err)?;
    let cloud_provider = cursor.read_u8().map_err(bincode_io_err)?;
    let technology = read_string(cursor)?;
    let description = read_string(cursor)?;
    Ok(ArchitectureData {
        name,
        c4_level,
        entity_type,
        cloud_provider,
        technology,
        description,
    })
}

fn read_props<R: byteorder::ReadBytesExt>(
    cursor: &mut R,
) -> Result<std::collections::BTreeMap<String, PropValue>, PersistenceError> {
    let count = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;
    let mut props = std::collections::BTreeMap::new();

    for _ in 0..count {
        let key = read_string(cursor)?;
        let value = read_prop_value(cursor)?;
        props.insert(key, value);
    }

    Ok(props)
}

fn read_prop_value<R: byteorder::ReadBytesExt>(
    cursor: &mut R,
) -> Result<PropValue, PersistenceError> {
    let type_tag = cursor.read_u8().map_err(bincode_io_err)?;

    Ok(match type_tag {
        0 => PropValue::Null,
        1 => PropValue::Boolean(cursor.read_u8().map_err(bincode_io_err)? != 0),
        2 => PropValue::Number(cursor.read_f64::<LE>().map_err(bincode_io_err)?),
        3 => PropValue::String(read_string(cursor)?),
        4 => {
            let count = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;
            let mut arr = Vec::with_capacity(count);
            for _ in 0..count {
                arr.push(read_prop_value(cursor)?);
            }
            PropValue::Array(arr)
        }
        5 => {
            let count = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;
            let mut obj = std::collections::BTreeMap::new();
            for _ in 0..count {
                let key = read_string(cursor)?;
                let value = read_prop_value(cursor)?;
                obj.insert(key, value);
            }
            PropValue::Object(obj)
        }
        _ => {
            return Err(PersistenceError::InvalidData(format!(
                "Unknown prop value type: {type_tag}"
            )));
        }
    })
}

fn read_spatial<R: byteorder::ReadBytesExt>(
    cursor: &mut R,
) -> Result<SpatialIndexData, PersistenceError> {
    let cell_size = cursor.read_f32::<LE>().map_err(bincode_io_err)?;
    let cell_count = cursor.read_usize::<LE>().map_err(bincode_io_err)?;
    let cell_count_raw = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;

    let mut cells = Vec::with_capacity(cell_count_raw);
    for _ in 0..cell_count_raw {
        let entity_count = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;
        let mut cell = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            cell.push(EntityId::new(
                cursor.read_u32::<LE>().map_err(bincode_io_err)?,
            ));
        }
        cells.push(cell);
    }

    Ok(SpatialIndexData {
        cell_size,
        cell_count,
        cells,
    })
}

fn read_string<R: byteorder::ReadBytesExt>(cursor: &mut R) -> Result<String, PersistenceError> {
    let len = cursor.read_u32::<LE>().map_err(bincode_io_err)? as usize;
    if len == 0 {
        return Ok(String::new());
    }

    if len > 10_000_000 {
        return Err(PersistenceError::InvalidData("String too large".into()));
    }

    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf).map_err(bincode_io_err)?;
    String::from_utf8(buf)
        .map_err(|_| PersistenceError::InvalidData("Invalid UTF-8 in string".into()))
}

fn read_optional_string<R: byteorder::ReadBytesExt>(
    cursor: &mut R,
) -> Result<Option<String>, PersistenceError> {
    let present = cursor.read_u8().map_err(bincode_io_err)? != 0;
    if present {
        Ok(Some(read_string(cursor)?))
    } else {
        Ok(None)
    }
}

// Helper trait extension
trait ReadBytesExtExt: byteorder::ReadBytesExt {
    fn read_usize<E: byteorder::ByteOrder>(&mut self) -> Result<usize, std::io::Error>;
}

impl<R: byteorder::ReadBytesExt + ?Sized> ReadBytesExtExt for R {
    #[cfg(target_pointer_width = "64")]
    fn read_usize<E: byteorder::ByteOrder>(&mut self) -> Result<usize, std::io::Error> {
        self.read_u64::<E>().map(|v| v as usize)
    }

    #[cfg(target_pointer_width = "32")]
    fn read_usize<E: byteorder::ByteOrder>(&mut self) -> Result<usize, std::io::Error> {
        self.read_u32::<E>().map(|v| v as usize)
    }
}

trait WriteBytesExtExt: byteorder::WriteBytesExt {
    fn write_usize<E: byteorder::ByteOrder>(&mut self, v: usize) -> Result<(), std::io::Error>;
}

impl<W: byteorder::WriteBytesExt + ?Sized> WriteBytesExtExt for W {
    #[cfg(target_pointer_width = "64")]
    fn write_usize<E: byteorder::ByteOrder>(&mut self, v: usize) -> Result<(), std::io::Error> {
        self.write_u64::<E>(v as u64)
    }

    #[cfg(target_pointer_width = "32")]
    fn write_usize<E: byteorder::ByteOrder>(&mut self, v: usize) -> Result<(), std::io::Error> {
        self.write_u32::<E>(v as u32)
    }
}

// Helper function
fn bincode_io_err<E: core::fmt::Display>(e: E) -> PersistenceError {
    PersistenceError::Serialization(e.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_magic_number() {
        assert_eq!(MAGIC_NUMBER, 0xAF01_0001);
    }

    #[test]
    fn test_serialize_empty_document() {
        let doc = Document::new();
        let binary = to_binary(&doc).unwrap();

        // Should contain magic number
        assert_eq!(binary[0], 0x01);
        assert_eq!(binary[1], 0x00);
        assert_eq!(binary[2], 0x01);
        assert_eq!(binary[3], 0xAF);
    }

    #[test]
    fn test_deserialize_empty_document() {
        let doc = Document::new();
        let binary = to_binary(&doc).unwrap();
        let doc2 = from_binary(&binary).unwrap();

        assert_eq!(doc2.meta.title, "Untitled");
        assert_eq!(doc2.entity_count(), 0);
    }

    #[test]
    fn test_serialize_round_trip() {
        let mut doc = Document::with_title("Test".into());

        doc.store.entities.push(EntityData {
            id: EntityId::new(1),
            parent_id: None,
            transform: [100.0, 200.0, 150.0, 80.0],
            world_transform: [100.0, 200.0, 150.0, 80.0],
            metadata: 0x0101,
            color: 0xFFCCDDEE,
            texture_index: 0,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            text: None,
            arch_data: None,
            props: BTreeMap::new(),
        });
        doc.store.entity_count = 1;

        let binary = to_binary(&doc).unwrap();
        let doc2 = from_binary(&binary).unwrap();

        assert_eq!(doc2.meta.title, "Test");
        assert_eq!(doc2.entity_count(), 1);
        assert_eq!(
            doc2.store.entities[0].transform,
            [100.0, 200.0, 150.0, 80.0]
        );
    }

    #[test]
    fn test_invalid_magic_number() {
        let invalid = [0x00, 0x00, 0x00, 0x00];
        let result = from_binary(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_too_short_data() {
        let short = [0x01, 0x00, 0x01, 0xAF];
        let result = from_binary(&short);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_smaller_than_json() {
        let mut doc = Document::with_title("Test".into());

        for i in 0..10 {
            doc.store.entities.push(EntityData {
                id: EntityId::new(i),
                parent_id: None,
                transform: [100.0, 200.0, 150.0, 80.0],
                world_transform: [100.0, 200.0, 150.0, 80.0],
                metadata: 0x0101,
                color: 0xFFCCDDEE,
                texture_index: 0,
                color_tint: [1.0, 1.0, 1.0, 1.0],
                text: None,
                arch_data: None,
                props: BTreeMap::new(),
            });
        }
        doc.store.entity_count = 10;

        let json = crate::format::json::to_json(&doc).unwrap();
        let binary = to_binary(&doc).unwrap();

        // Binary should be smaller than JSON
        assert!(binary.len() < json.len());
    }
}
