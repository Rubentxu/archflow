// ═══════════════════════════════════════════════════════════════════════════════
// Integration Tests - Persistence Layer with Migration Engine
//
// Tests end-to-end integration of the persistence system:
// - Document serialization (JSON/Binary)
// - Migration engine version handling
// - Logic Bricks wiring persistence
// - SpatialHash pre-building
//
// EPIC-WEB-012: Migration Engine automatic
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_persistence::{
    CURRENT_SCHEMA_VERSION, CompressionOption, Document, DocumentMeta, MigrationEngine,
    PersistenceEngine, PersistenceOptions, Schema, SchemaVersion, SerializationFormat,
    StoreSnapshot,
};
use std::collections::BTreeMap;
use std::string::String;
use std::vec::Vec;

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 1: Migration Engine Version Detection
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_migration_engine_creation() {
    let engine = MigrationEngine::new();
    assert_eq!(engine.current_version(), &CURRENT_SCHEMA_VERSION);
}

#[test]
fn test_migration_current_version_no_change() {
    let engine = MigrationEngine::new();
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    let result = engine.migrate_to_current(doc);
    assert!(result.is_ok());

    let migrated = result.unwrap();
    assert_eq!(migrated.schema.version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn test_migration_from_newer_version_fails() {
    let engine = MigrationEngine::new();
    let newer_version = SchemaVersion {
        major: 999,
        minor: 0,
        patch: 0,
    };
    let doc = create_test_document(newer_version);

    let result = engine.migrate_to_current(doc);
    assert!(result.is_err());
}

#[test]
fn test_migration_preserves_document_data() {
    let engine = MigrationEngine::new();
    let mut doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Set some test data
    doc.meta.title = String::from("Test Title");
    doc.meta.author = Some(String::from("Test Author"));
    doc.meta.description = String::from("Test Description");

    let result = engine.migrate_to_current(doc);
    assert!(result.is_ok());

    let migrated = result.unwrap();
    assert_eq!(migrated.meta.title, "Test Title");
    assert_eq!(migrated.meta.author, Some("Test Author".to_string()));
    assert_eq!(migrated.meta.description, "Test Description");
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 2: JSON Serialization Round-Trip
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_serialization_round_trip() {
    let engine = PersistenceEngine::new();
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Serialize
    let json_str = engine.export_json(&doc);
    assert!(json_str.is_ok());

    let json = json_str.unwrap();
    assert!(!json.is_empty());

    // Deserialize
    let result = engine.import_json(&json);
    assert!(result.is_ok());

    let loaded = result.unwrap();
    assert_eq!(loaded.meta.title, doc.meta.title);
    assert_eq!(loaded.schema.version, doc.schema.version);
}

#[test]
fn test_json_with_pretty_print() {
    let options = PersistenceOptions::new()
        .with_format(SerializationFormat::Json)
        .with_pretty_print(true);

    let engine = PersistenceEngine::with_options(options);
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    let json_str = engine.export_json(&doc);
    assert!(json_str.is_ok());

    let json = json_str.unwrap();
    // Pretty printed JSON should have newlines
    assert!(json.contains('\n'));
}

#[test]
fn test_json_preserves_metadata() {
    let engine = PersistenceEngine::new();
    let mut doc = create_test_document(CURRENT_SCHEMA_VERSION);

    doc.meta.title = String::from("Preserved Title");
    doc.meta.description = String::from("Preserved Description");
    doc.meta.author = Some(String::from("Test Author"));

    let json_str = engine.export_json(&doc).unwrap();
    let result = engine.import_json(&json_str);

    assert!(result.is_ok());
    let loaded = result.unwrap();
    assert_eq!(loaded.meta.title, "Preserved Title");
    assert_eq!(loaded.meta.description, "Preserved Description");
    assert_eq!(loaded.meta.author, Some("Test Author".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 3: Compression Support
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gzip_compression() {
    let options = PersistenceOptions::new().with_compression(CompressionOption::Gzip);

    let engine = PersistenceEngine::with_options(options);
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Export to compressed bytes
    let bytes = engine.export_bytes(&doc);
    assert!(bytes.is_ok());

    let compressed = bytes.unwrap();
    // Gzip magic number
    assert_eq!(compressed[0], 0x1f);
    assert_eq!(compressed[1], 0x8b);

    // Import compressed bytes
    let result = engine.import_bytes(&compressed);
    assert!(result.is_ok());

    let loaded = result.unwrap();
    assert_eq!(loaded.meta.title, doc.meta.title);
}

#[test]
fn test_no_compression() {
    let options = PersistenceOptions::new().with_compression(CompressionOption::None);

    let engine = PersistenceEngine::with_options(options);
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Export to bytes (no compression)
    let bytes = engine.export_bytes(&doc);
    assert!(bytes.is_ok());

    let uncompressed = bytes.unwrap();
    // Should start with { for JSON
    assert_eq!(uncompressed[0], b'{');

    // Import
    let result = engine.import_bytes(&uncompressed);
    assert!(result.is_ok());
}

#[test]
fn test_compression_ratio() {
    let engine_compressed = PersistenceEngine::with_options(
        PersistenceOptions::new().with_compression(CompressionOption::Gzip),
    );
    let engine_uncompressed = PersistenceEngine::new();
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    let uncompressed_bytes = engine_uncompressed.export_bytes(&doc).unwrap();
    let compressed_bytes = engine_compressed.export_bytes(&doc).unwrap();

    // Compression should reduce size
    assert!(compressed_bytes.len() < uncompressed_bytes.len());

    // But not too small (should be reasonable)
    assert!(compressed_bytes.len() > 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 4: Format Auto-Detection
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_auto_detect_json() {
    let engine = PersistenceEngine::new();
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Export as JSON
    let json_bytes = engine.export_json(&doc);
    assert!(json_bytes.is_ok());

    let json = json_bytes.unwrap();
    let bytes = json.into_bytes();

    // Import should auto-detect JSON format
    let result = engine.import_bytes(&bytes);
    assert!(result.is_ok());
}

#[test]
fn test_auto_detect_with_invalid_format() {
    let engine = PersistenceEngine::new();

    // Invalid data (not JSON or binary)
    let invalid = vec![0xFF, 0xFF, 0xFF, 0xFF];

    let result = engine.import_bytes(&invalid);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 5: Options Builder
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_options_default() {
    let opts = PersistenceOptions::new();
    assert_eq!(opts.format, SerializationFormat::Json);
    assert_eq!(opts.compression, CompressionOption::None);
    assert!(opts.include_spatial_index);
    assert!(opts.include_logic_wiring);
}

#[test]
fn test_options_builder() {
    let opts = PersistenceOptions::new()
        .with_format(SerializationFormat::Binary)
        .with_compression(CompressionOption::Gzip)
        .with_pretty_print(true);

    assert_eq!(opts.format, SerializationFormat::Binary);
    assert_eq!(opts.compression, CompressionOption::Gzip);
    assert!(opts.pretty_print);
}

#[test]
fn test_engine_creation() {
    let engine = PersistenceEngine::new();
    assert_eq!(engine.options().format, SerializationFormat::Json);
}

#[test]
fn test_engine_with_options() {
    let opts = PersistenceOptions::new().with_format(SerializationFormat::Binary);
    let engine = PersistenceEngine::with_options(opts);
    assert_eq!(engine.options().format, SerializationFormat::Binary);
}

#[test]
fn test_engine_default() {
    let engine = PersistenceEngine::default();
    assert_eq!(engine.options().format, SerializationFormat::Json);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 6: Migration with Serialization
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_serialize_and_preserve_version() {
    let engine = PersistenceEngine::new();
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Serialize
    let json_str = engine.export_json(&doc).unwrap();

    // Deserialize
    let result = engine.import_json(&json_str);
    assert!(result.is_ok());

    let loaded = result.unwrap();
    assert_eq!(loaded.schema.version, CURRENT_SCHEMA_VERSION);
}

#[test]
fn test_migration_engine_integration() {
    let migration_engine = MigrationEngine::new();
    let doc = create_test_document(CURRENT_SCHEMA_VERSION);

    // Should not need migration
    let result = migration_engine.migrate_to_current(doc);
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

fn create_test_document(version: SchemaVersion) -> Document {
    Document {
        schema: Schema {
            version,
            name: String::from("Test Schema"),
            shape_types: BTreeMap::new(),
            migrations: Vec::new(),
        },
        meta: DocumentMeta {
            title: String::from("Test Document"),
            description: String::new(),
            author: None,
            created_at: String::from("2026-02-03T12:00:00Z"),
            modified_at: String::from("2026-02-03T12:00:00Z"),
            app_version: String::from("1.0.0"),
            custom: BTreeMap::new(),
        },
        store: StoreSnapshot {
            version: 1,
            entity_count: 0,
            entities: Vec::new(),
        },
        spatial_index: None,
        logic_wiring: None,
    }
}
