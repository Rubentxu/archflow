// ═══════════════════════════════════════════════════════════════════════════════
// JSON Serialization - Human-readable document format
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use archflow_core::{EntityId, Generation, Index};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::vec::Vec;

use crate::{
    ArchitectureData, Document, DocumentMeta, EntityData, Migration, PersistenceError,
    PersistenceResult, PropValue, Schema, SchemaVersion, ShapeTypeDef, SpatialIndexData,
    StoreSnapshot, TextData,
};

use crate::logic::SerializableWiring;

// ═══════════════════════════════════════════════════════════════════════════════
// JSON REPRESENTATION TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// JSON document wrapper (root object)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonDocument {
    version: u32,
    schema: JsonSchema,
    meta: JsonDocumentMeta,
    store: JsonStore,
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_index: Option<JsonSpatialIndex>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logic_wiring: Option<serde_json::Value>,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonSchema {
    version: u32,
    name: String,
    shape_types: Vec<JsonShapeType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    migrations: Vec<JsonMigration>,
}

/// Shape type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonShapeType {
    name: String,
    type_id: u8,
    supports_children: bool,
    supports_text: bool,
    supports_connections: bool,
}

/// Migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonMigration {
    from_version: u32,
    to_version: u32,
    description: String,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonDocumentMeta {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    created_at: String,
    modified_at: String,
    app_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom: Option<serde_json::Value>,
}

/// Store snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonStore {
    version: u32,
    entity_count: u32,
    entities: Vec<JsonEntity>,
}

/// Entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonEntity {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
    transform: [f32; 4],
    world_transform: [f32; 4],
    metadata: u32,
    color: u32,
    #[serde(skip_serializing_if = "is_zero_u16", default)]
    texture_index: u16,
    #[serde(skip_serializing_if = "is_default_tint", default)]
    color_tint: [f32; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<JsonTextData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arch_data: Option<JsonArchitectureData>,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty", default)]
    props: serde_json::Map<String, serde_json::Value>,
}

/// Text data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonTextData {
    content: String,
    scale: f32,
    glyph_count: u16,
}

/// Architecture data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonArchitectureData {
    name: String,
    c4_level: u8,
    entity_type: u8,
    cloud_provider: u8,
    technology: String,
    description: String,
}

/// Spatial index
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonSpatialIndex {
    cell_size: f32,
    cell_count: usize,
    cells: Vec<Vec<String>>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

fn is_zero_u16(v: &u16) -> bool {
    *v == 0
}

fn is_default_tint(v: &[f32; 4]) -> bool {
    v[0] == 1.0 && v[1] == 1.0 && v[2] == 1.0 && v[3] == 1.0
}

/// Convert EntityId to string representation
fn entity_id_to_string(id: EntityId) -> String {
    format!("{}:{}", id.index().0, id.generation().0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialize document to JSON string (compact)
pub fn to_json(document: &Document) -> PersistenceResult<String> {
    let json_doc = to_json_document(document)?;
    serde_json::to_string(&json_doc).map_err(|e| PersistenceError::Serialization(e.to_string()))
}

/// Serialize document to pretty JSON string
pub fn to_json_pretty(document: &Document) -> PersistenceResult<String> {
    let json_doc = to_json_document(document)?;
    serde_json::to_string_pretty(&json_doc)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))
}

fn to_json_document(document: &Document) -> PersistenceResult<JsonDocument> {
    let logic_wiring = document
        .logic_wiring
        .as_ref()
        .map(to_json_logic_wiring)
        .transpose()?;

    Ok(JsonDocument {
        version: document.schema.version.as_u32(),
        schema: to_json_schema(&document.schema)?,
        meta: to_json_meta(&document.meta)?,
        store: to_json_store(&document.store)?,
        spatial_index: document.spatial_index.as_ref().map(to_json_spatial),
        logic_wiring,
    })
}

fn to_json_schema(schema: &Schema) -> PersistenceResult<JsonSchema> {
    let shape_types = schema
        .shape_types
        .values()
        .map(|t| JsonShapeType {
            name: t.name.clone(),
            type_id: t.type_id,
            supports_children: t.supports_children,
            supports_text: t.supports_text,
            supports_connections: t.supports_connections,
        })
        .collect();

    let migrations = schema
        .migrations
        .iter()
        .map(|m| JsonMigration {
            from_version: m.from_version,
            to_version: m.to_version,
            description: m.description.clone(),
        })
        .collect();

    Ok(JsonSchema {
        version: schema.version.as_u32(),
        name: schema.name.clone(),
        shape_types,
        migrations,
    })
}

fn to_json_meta(meta: &DocumentMeta) -> PersistenceResult<JsonDocumentMeta> {
    let custom = if meta.custom.is_empty() {
        None
    } else {
        Some(serde_json::to_value(&meta.custom).unwrap_or(serde_json::Value::Null))
    };

    Ok(JsonDocumentMeta {
        title: meta.title.clone(),
        description: if meta.description.is_empty() {
            None
        } else {
            Some(meta.description.clone())
        },
        author: meta.author.clone(),
        created_at: meta.created_at.clone(),
        modified_at: meta.modified_at.clone(),
        app_version: meta.app_version.clone(),
        custom,
    })
}

fn to_json_store(store: &StoreSnapshot) -> PersistenceResult<JsonStore> {
    let entities = store
        .entities
        .iter()
        .map(|e| to_json_entity(e))
        .collect::<Result<_, _>>()?;

    Ok(JsonStore {
        version: store.version,
        entity_count: store.entity_count,
        entities,
    })
}

fn to_json_entity(entity: &EntityData) -> PersistenceResult<JsonEntity> {
    let props = entity
        .props
        .iter()
        .map(|(k, v)| Ok((k.clone(), prop_value_to_json(v)?)))
        .collect::<Result<_, PersistenceError>>()?;

    Ok(JsonEntity {
        id: entity_id_to_string(entity.id),
        parent_id: entity.parent_id.map(entity_id_to_string),
        transform: entity.transform,
        world_transform: entity.world_transform,
        metadata: entity.metadata,
        color: entity.color,
        texture_index: entity.texture_index,
        color_tint: entity.color_tint,
        text: entity.text.as_ref().map(to_json_text),
        arch_data: entity.arch_data.as_ref().map(to_json_arch),
        props,
    })
}

fn to_json_text(text: &TextData) -> JsonTextData {
    JsonTextData {
        content: text.content.clone(),
        scale: text.scale,
        glyph_count: text.glyph_count,
    }
}

fn to_json_arch(arch: &ArchitectureData) -> JsonArchitectureData {
    JsonArchitectureData {
        name: arch.name.clone(),
        c4_level: arch.c4_level,
        entity_type: arch.entity_type,
        cloud_provider: arch.cloud_provider,
        technology: arch.technology.clone(),
        description: arch.description.clone(),
    }
}

fn to_json_spatial(spatial: &SpatialIndexData) -> JsonSpatialIndex {
    JsonSpatialIndex {
        cell_size: spatial.cell_size,
        cell_count: spatial.cell_count,
        cells: spatial
            .cells
            .iter()
            .map(|cell| cell.iter().map(|id| entity_id_to_string(*id)).collect())
            .collect(),
    }
}

fn prop_value_to_json(value: &PropValue) -> PersistenceResult<serde_json::Value> {
    Ok(match value {
        PropValue::String(s) => serde_json::Value::String(s.clone()),
        PropValue::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n)
                .ok_or_else(|| PersistenceError::Serialization("Invalid number".into()))?,
        ),
        PropValue::Boolean(b) => serde_json::Value::Bool(*b),
        PropValue::Array(arr) => {
            let vals: Result<Vec<_>, _> = arr.iter().map(prop_value_to_json).collect();
            serde_json::Value::Array(vals?)
        }
        PropValue::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k.clone(), prop_value_to_json(v)?);
            }
            serde_json::Value::Object(map)
        }
        PropValue::Null => serde_json::Value::Null,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// DESERIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Deserialize document from JSON string
pub fn from_json(json: &str) -> PersistenceResult<Document> {
    let json_doc: JsonDocument =
        serde_json::from_str(json).map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
    from_json_document(json_doc)
}

fn from_json_document(json_doc: JsonDocument) -> PersistenceResult<Document> {
    let logic_wiring = json_doc
        .logic_wiring
        .as_ref()
        .map(from_json_logic_wiring)
        .transpose()?;

    Ok(Document {
        schema: from_json_schema(json_doc.schema)?,
        meta: from_json_meta(json_doc.meta)?,
        store: from_json_store(json_doc.store)?,
        spatial_index: json_doc.spatial_index.map(from_json_spatial).transpose()?,
        logic_wiring,
    })
}

fn from_json_schema(schema: JsonSchema) -> PersistenceResult<Schema> {
    let mut shape_types = std::collections::BTreeMap::new();
    for t in schema.shape_types {
        shape_types.insert(
            t.name.clone(),
            ShapeTypeDef {
                name: t.name,
                type_id: t.type_id,
                supports_children: t.supports_children,
                supports_text: t.supports_text,
                supports_connections: t.supports_connections,
            },
        );
    }

    let migrations = schema
        .migrations
        .into_iter()
        .map(|m| Migration {
            from_version: m.from_version,
            to_version: m.to_version,
            description: m.description,
        })
        .collect();

    Ok(Schema {
        version: SchemaVersion::from_u32(schema.version),
        name: schema.name,
        shape_types,
        migrations,
    })
}

fn from_json_meta(meta: JsonDocumentMeta) -> PersistenceResult<DocumentMeta> {
    let custom = if let Some(val) = meta.custom {
        serde_json::from_value(val).unwrap_or_default()
    } else {
        std::collections::BTreeMap::new()
    };

    Ok(DocumentMeta {
        title: meta.title,
        description: meta.description.unwrap_or_default(),
        author: meta.author,
        created_at: meta.created_at,
        modified_at: meta.modified_at,
        app_version: meta.app_version,
        custom,
    })
}

fn from_json_store(store: JsonStore) -> PersistenceResult<StoreSnapshot> {
    let entities = store
        .entities
        .into_iter()
        .map(from_json_entity)
        .collect::<Result<_, _>>()?;

    Ok(StoreSnapshot {
        version: store.version,
        entity_count: store.entity_count,
        entities,
    })
}

fn from_json_entity(entity: JsonEntity) -> PersistenceResult<EntityData> {
    let props = entity
        .props
        .into_iter()
        .map(|(k, v)| Ok((k, json_to_prop_value(v)?)))
        .collect::<Result<_, PersistenceError>>()?;

    Ok(EntityData {
        id: parse_entity_id(&entity.id)?,
        parent_id: entity
            .parent_id
            .as_deref()
            .map(parse_entity_id)
            .transpose()?,
        transform: entity.transform,
        world_transform: entity.world_transform,
        metadata: entity.metadata,
        color: entity.color,
        texture_index: entity.texture_index,
        color_tint: entity.color_tint,
        text: entity.text.map(from_json_text).transpose()?,
        arch_data: entity.arch_data.map(from_json_arch).transpose()?,
        props,
    })
}

fn from_json_text(text: JsonTextData) -> PersistenceResult<TextData> {
    Ok(TextData {
        content: text.content,
        scale: text.scale,
        glyph_count: text.glyph_count,
    })
}

fn from_json_arch(arch: JsonArchitectureData) -> PersistenceResult<ArchitectureData> {
    Ok(ArchitectureData {
        name: arch.name,
        c4_level: arch.c4_level,
        entity_type: arch.entity_type,
        cloud_provider: arch.cloud_provider,
        technology: arch.technology,
        description: arch.description,
    })
}

fn from_json_spatial(spatial: JsonSpatialIndex) -> PersistenceResult<SpatialIndexData> {
    let cells = spatial
        .cells
        .into_iter()
        .map(|cell| {
            cell.into_iter()
                .map(|s| parse_entity_id(&s))
                .collect::<Result<_, _>>()
        })
        .collect::<Result<_, _>>()?;

    Ok(SpatialIndexData {
        cell_size: spatial.cell_size,
        cell_count: spatial.cell_count,
        cells,
    })
}

fn parse_entity_id(s: &str) -> PersistenceResult<EntityId> {
    // EntityId format: "index:generation" (matching entity_id_to_string format)
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() >= 2 {
        let index = parts[0]
            .parse::<u32>()
            .map_err(|_| PersistenceError::Deserialization(format!("Invalid entity ID: {s}")))?;
        let generation = parts[1]
            .parse::<u8>()
            .map_err(|_| PersistenceError::Deserialization(format!("Invalid entity ID: {s}")))?;
        return Ok(EntityId::from_parts(Index(index), Generation(generation)));
    }

    // Try parsing as plain u32 (backward compatibility)
    let id_val = s
        .parse::<u32>()
        .map_err(|_| PersistenceError::Deserialization(format!("Invalid entity ID: {s}")))?;
    Ok(EntityId::new(id_val))
}

fn json_to_prop_value(value: serde_json::Value) -> PersistenceResult<PropValue> {
    Ok(match value {
        serde_json::Value::Null => PropValue::Null,
        serde_json::Value::Bool(b) => PropValue::Boolean(b),
        serde_json::Value::Number(n) => PropValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => PropValue::String(s),
        serde_json::Value::Array(arr) => {
            let vals: Result<Vec<_>, _> = arr.into_iter().map(json_to_prop_value).collect();
            PropValue::Array(vals?)
        }
        serde_json::Value::Object(obj) => {
            let mut map = BTreeMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_prop_value(v)?);
            }
            PropValue::Object(map)
        }
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// LOGIC WIRING SERIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert SerializableWiring to JSON value
fn to_json_logic_wiring(wiring: &SerializableWiring) -> PersistenceResult<serde_json::Value> {
    crate::logic::LogicWiringSerializer::to_json(wiring)
        .map(|s| serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!(null)))
        .map_err(|e| PersistenceError::Serialization(e.to_string()))
}

/// Deserialize JSON value to SerializableWiring
fn from_json_logic_wiring(value: &serde_json::Value) -> PersistenceResult<SerializableWiring> {
    if value.is_null() {
        return Ok(SerializableWiring::new());
    }

    let json_str = serde_json::to_string(value)
        .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;

    crate::logic::LogicWiringSerializer::from_json(&json_str)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_serialize_empty_document() {
        let doc = Document::new();
        let json = to_json(&doc).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["version"], 1);
    }

    #[test]
    fn test_serialize_document_with_title() {
        let doc = Document::with_title("Test Document".into());
        let json = to_json(&doc).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["title"], "Test Document");
    }

    #[test]
    fn test_deserialize_empty_document() {
        let json = r#"{"version":1,"schema":{"version":1,"name":"ArchFlow Document Schema","shape_types":[],"migrations":[]},"meta":{"title":"Untitled","created_at":"2024-01-01T00:00:00Z","modified_at":"2024-01-01T00:00:00Z","app_version":"0.36.0"},"store":{"version":1,"entity_count":0,"entities":[]}}"#;
        let doc = from_json(json).unwrap();
        assert_eq!(doc.meta.title, "Untitled");
        assert_eq!(doc.entity_count(), 0);
    }

    #[test]
    fn test_serialize_round_trip() {
        let mut doc = Document::with_title("Test".into());

        // Add an entity
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

        let json = to_json(&doc).unwrap();
        let doc2 = from_json(&json).unwrap();

        assert_eq!(doc2.meta.title, "Test");
        assert_eq!(doc2.entity_count(), 1);
        assert_eq!(
            doc2.store.entities[0].transform,
            [100.0, 200.0, 150.0, 80.0]
        );
    }

    #[test]
    fn test_pretty_json() {
        let doc = Document::with_title("Test".into());
        let json = to_json_pretty(&doc).unwrap();

        // Pretty JSON should contain newlines and indentation
        assert!(json.contains('\n'));
        assert!(json.contains("  "));
    }

    #[test]
    fn test_invalid_json() {
        let json = "not valid json";
        let result = from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_prop_value_serialization() {
        let mut props: BTreeMap<String, PropValue> = BTreeMap::new();
        props.insert("string".into(), PropValue::String("hello".into()));
        props.insert("number".into(), PropValue::Number(42.0));
        props.insert("bool".into(), PropValue::Boolean(true));
        props.insert("null".into(), PropValue::Null);

        for (_k, v) in &props {
            let json = prop_value_to_json(v).unwrap();
            let round_trip = json_to_prop_value(json).unwrap();
            assert_eq!(v, &round_trip);
        }
    }
}
