// ═══════════════════════════════════════════════════════════════════════════════
// Document Type - Main document structure for persistence
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use archflow_core::EntityId;
use std::collections::BTreeMap;

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Schema version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Get the combined version number
    #[must_use]
    pub const fn as_u32(&self) -> u32 {
        (self.major << 16) | (self.minor << 8) | self.patch
    }

    /// Create from a combined u32 version number
    #[must_use]
    pub const fn from_u32(version: u32) -> Self {
        Self {
            major: (version >> 16) & 0xFF,
            minor: (version >> 8) & 0xFF,
            patch: version & 0xFF,
        }
    }

    /// Check if this version is compatible with another
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Same major version is required for compatibility
        self.major == other.major
    }
}

/// Document schema definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    /// Schema version
    pub version: SchemaVersion,
    /// Schema name
    pub name: String,
    /// Shape type definitions
    pub shape_types: BTreeMap<String, ShapeTypeDef>,
    /// Migration history
    pub migrations: Vec<Migration>,
}

impl Schema {
    /// Create the current schema
    #[must_use]
    pub fn current() -> Self {
        Self {
            version: SchemaVersion::from_u32(CURRENT_SCHEMA_VERSION),
            name: "ArchFlow Document Schema".into(),
            shape_types: ShapeTypeDef::all_built_in(),
            migrations: Vec::new(),
        }
    }

    /// Check if a document with this schema can be loaded
    #[must_use]
    pub fn is_compatible(&self, document_version: u32) -> bool {
        let doc_schema = SchemaVersion::from_u32(document_version);
        self.version.is_compatible_with(&doc_schema)
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::current()
    }
}

/// Shape type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeTypeDef {
    /// Type name
    pub name: String,
    /// Type identifier (0-15 for bit-packing)
    pub type_id: u8,
    /// Whether this type supports children (grouping)
    pub supports_children: bool,
    /// Whether this type supports text content
    pub supports_text: bool,
    /// Whether this type supports connections
    pub supports_connections: bool,
}

impl ShapeTypeDef {
    /// Create a new shape type definition
    #[must_use]
    pub fn new(
        name: &'static str,
        type_id: u8,
        supports_children: bool,
        supports_text: bool,
        supports_connections: bool,
    ) -> Self {
        Self {
            name: String::from(name),
            type_id,
            supports_children,
            supports_text,
            supports_connections,
        }
    }

    /// Get all built-in shape types
    fn all_built_in() -> BTreeMap<String, ShapeTypeDef> {
        let mut types = BTreeMap::new();

        types.insert(
            "rectangle".into(),
            Self::new("Rectangle", 0, false, false, false),
        );
        types.insert("circle".into(), Self::new("Circle", 1, false, false, false));
        types.insert(
            "ellipse".into(),
            Self::new("Ellipse", 2, false, false, false),
        );
        types.insert("line".into(), Self::new("Line", 3, false, false, false));
        types.insert(
            "triangle".into(),
            Self::new("Triangle", 4, false, false, false),
        );
        types.insert(
            "diamond".into(),
            Self::new("Diamond", 5, false, false, false),
        );
        types.insert(
            "cylinder".into(),
            Self::new("Cylinder", 6, false, false, false),
        );
        types.insert("person".into(), Self::new("Person", 7, false, false, false));
        types.insert(
            "rounded_rect".into(),
            Self::new("RoundedRect", 8, false, false, false),
        );
        types.insert(
            "dashed_rect".into(),
            Self::new("DashedRect", 9, false, false, false),
        );
        types.insert("group".into(), Self::new("Group", 10, true, false, false));
        types.insert("text".into(), Self::new("Text", 11, false, true, false));
        types.insert(
            "connector".into(),
            Self::new("Connector", 12, false, false, true),
        );

        types
    }
}

/// Migration record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// Source version
    pub from_version: u32,
    /// Target version
    pub to_version: u32,
    /// Migration description
    pub description: String,
}

/// Document metadata
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentMeta {
    /// Document title
    pub title: String,
    /// Document description
    pub description: String,
    /// Author name
    pub author: Option<String>,
    /// Creation timestamp (RFC3339)
    pub created_at: String,
    /// Last modification timestamp (RFC3339)
    pub modified_at: String,
    /// Application version that created this document
    pub app_version: String,
    /// Custom metadata
    pub custom: BTreeMap<String, String>,
}

impl DocumentMeta {
    /// Create new document metadata
    #[must_use]
    pub fn new(title: String) -> Self {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| String::from(""));

        Self {
            title,
            description: String::new(),
            author: None,
            created_at: now.clone(),
            modified_at: now,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            custom: BTreeMap::new(),
        }
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.modified_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| String::from(""));
    }
}

impl Default for DocumentMeta {
    fn default() -> Self {
        Self::new(String::from("Untitled"))
    }
}

/// Main document structure
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Document schema
    pub schema: Schema,
    /// Document metadata
    pub meta: DocumentMeta,
    /// Entity store snapshot
    pub store: StoreSnapshot,
    /// Spatial hash data (optional, for pre-built index)
    pub spatial_index: Option<SpatialIndexData>,
    /// Logic Bricks wiring (optional)
    pub logic_wiring: Option<crate::logic::SerializableWiring>,
}

/// Spatial index data for pre-built queries
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialIndexData {
    /// Cell size used for the index
    pub cell_size: f32,
    /// Number of cells in the index
    pub cell_count: usize,
    /// Entity IDs per cell (cell_index -> entity_ids)
    pub cells: Vec<Vec<EntityId>>,
}

/// Store snapshot - simplified entity data for serialization
///
/// This is a simplified representation of EntityStore's SoA layout
/// that can be easily serialized and deserialized.
#[derive(Debug, Clone, PartialEq)]
pub struct StoreSnapshot {
    /// Version of the store format
    pub version: u32,
    /// Number of entities in the store
    pub entity_count: u32,
    /// Entity data (one entry per entity)
    pub entities: Vec<EntityData>,
}

/// Single entity data record
#[derive(Debug, Clone, PartialEq)]
pub struct EntityData {
    /// Entity ID
    pub id: EntityId,
    /// Parent entity ID (if any)
    pub parent_id: Option<EntityId>,
    /// Transform [x, y, width, height]
    pub transform: [f32; 4],
    /// World transform [x, y, width, height]
    pub world_transform: [f32; 4],
    /// Metadata (bit-packed)
    pub metadata: u32,
    /// Color (0xRRGGBBAA)
    pub color: u32,
    /// Texture index
    pub texture_index: u16,
    /// Color tint [r, g, b, a]
    pub color_tint: [f32; 4],
    /// Text data (if text entity)
    pub text: Option<TextData>,
    /// Architecture data (if C4 entity)
    pub arch_data: Option<ArchitectureData>,
    /// Custom properties
    pub props: BTreeMap<String, PropValue>,
}

/// Text data for text entities
#[derive(Debug, Clone, PartialEq)]
pub struct TextData {
    /// Text content
    pub content: String,
    /// Font scale
    pub scale: f32,
    /// Number of glyphs
    pub glyph_count: u16,
}

/// Architecture data for C4 diagram entities
#[derive(Debug, Clone, PartialEq)]
pub struct ArchitectureData {
    /// Entity name
    pub name: String,
    /// C4 level (0=Person, 1=System, 2=Container, 3=Component)
    pub c4_level: u8,
    /// Entity type (custom)
    pub entity_type: u8,
    /// Cloud provider (0=None, 1=AWS, 2=GCP, 3=Azure)
    pub cloud_provider: u8,
    /// Technology name
    pub technology: String,
    /// Description
    pub description: String,
}

/// Property value (dynamic type)
#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    /// String value
    String(String),
    /// Number value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<PropValue>),
    /// Object (map of values)
    Object(BTreeMap<String, PropValue>),
    /// Null value
    Null,
}

impl From<String> for PropValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<f64> for PropValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for PropValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl Document {
    /// Create a new empty document
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: Schema::current(),
            meta: DocumentMeta::default(),
            store: StoreSnapshot {
                version: 1,
                entity_count: 0,
                entities: Vec::new(),
            },
            spatial_index: None,
            logic_wiring: None,
        }
    }

    /// Create a new document with a title
    #[must_use]
    pub fn with_title(title: String) -> Self {
        Self {
            schema: Schema::current(),
            meta: DocumentMeta::new(title),
            store: StoreSnapshot {
                version: 1,
                entity_count: 0,
                entities: Vec::new(),
            },
            spatial_index: None,
            logic_wiring: None,
        }
    }

    /// Get the number of entities in the document
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.store.entities.len()
    }

    /// Check if the document is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.store.entities.is_empty()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERDE SUPPORT
// ═══════════════════════════════════════════════════════════════════════════════

// Implement serde for our types

use serde::{Deserialize, Serialize};

impl Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_u32().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = u32::deserialize(deserializer)?;
        Ok(Self::from_u32(v))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_new() {
        let v = SchemaVersion::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_schema_version_as_u32() {
        let v = SchemaVersion::new(1, 2, 3);
        assert_eq!(v.as_u32(), (1 << 16) | (2 << 8) | 3);
    }

    #[test]
    fn test_schema_version_from_u32() {
        let v = SchemaVersion::from_u32((1 << 16) | (2 << 8) | 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_schema_version_compatible() {
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = SchemaVersion::new(1, 5, 0);
        let v3 = SchemaVersion::new(2, 0, 0);

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_schema_current() {
        let schema = Schema::current();
        assert_eq!(schema.version.as_u32(), CURRENT_SCHEMA_VERSION);
        assert!(!schema.shape_types.is_empty());
    }

    #[test]
    fn test_document_new() {
        let doc = Document::new();
        assert!(doc.is_empty());
        assert_eq!(doc.entity_count(), 0);
    }

    #[test]
    fn test_document_with_title() {
        let doc = Document::with_title("Test Document".into());
        assert_eq!(doc.meta.title, "Test Document");
        assert!(doc.is_empty());
    }

    #[test]
    fn test_document_meta_new() {
        let meta = DocumentMeta::new("Test".into());
        assert_eq!(meta.title, "Test");
        assert!(!meta.created_at.is_empty());
        assert!(!meta.modified_at.is_empty());
    }

    #[test]
    fn test_shape_type_def_built_ins() {
        let types = ShapeTypeDef::all_built_in();
        assert!(types.contains_key("rectangle"));
        assert!(types.contains_key("circle"));
        assert!(types.contains_key("group"));
        assert!(types.contains_key("text"));
        assert!(types.contains_key("connector"));
    }

    #[test]
    fn test_prop_value_from() {
        let s: PropValue = "hello".to_string().into();
        assert!(matches!(s, PropValue::String(_)));

        let n: PropValue = 42.0.into();
        assert!(matches!(n, PropValue::Number(_)));

        let b: PropValue = true.into();
        assert!(matches!(b, PropValue::Boolean(_)));
    }

    #[test]
    fn test_entity_data_default_transform() {
        let entity = EntityData {
            id: EntityId::new(1),
            parent_id: None,
            transform: [0.0, 0.0, 100.0, 60.0],
            world_transform: [0.0, 0.0, 100.0, 60.0],
            metadata: 0,
            color: 0xFFCCDDEE,
            texture_index: 0,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            text: None,
            arch_data: None,
            props: BTreeMap::new(),
        };

        assert_eq!(entity.transform[0], 0.0);
        assert_eq!(entity.transform[1], 0.0);
        assert_eq!(entity.transform[2], 100.0);
        assert_eq!(entity.transform[3], 60.0);
    }

    #[test]
    fn test_arch_data() {
        let arch = ArchitectureData {
            name: "UserService".into(),
            c4_level: 2, // Container
            entity_type: 0,
            cloud_provider: 1, // AWS
            technology: "Rust".into(),
            description: "User management service".into(),
        };

        assert_eq!(arch.name, "UserService");
        assert_eq!(arch.c4_level, 2);
        assert_eq!(arch.cloud_provider, 1);
    }

    #[test]
    fn test_text_data() {
        let text = TextData {
            content: "Hello World".into(),
            scale: 16.0,
            glyph_count: 11,
        };

        assert_eq!(text.content, "Hello World");
        assert_eq!(text.scale, 16.0);
        assert_eq!(text.glyph_count, 11);
    }
}
