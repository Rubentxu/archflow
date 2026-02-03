# archflow-persistence

> **Document Serialization & Persistence** - Comprehensive document storage with multiple formats, spatial indexing, and Logic Bricks wiring persistence.

## Overview

`archflow-persistence` provides a robust document serialization and persistence layer for ArchFlow architecture diagrams. It handles complex document structures with multiple serialization formats, automatic compression, spatial indexing for O(1) queries, and Logic Bricks connection persistence.

**Key Capabilities:**
- **Multiple formats** - JSON (human-readable) and Binary (optimized)
- **Compression** - Optional Gzip compression for large documents
- **Spatial indexing** - Pre-built hash grids for instant spatial queries
- **Logic wiring** - Sensor-Controller-Actuator connection persistence
- **Schema versioning** - Backward compatibility support

## Architecture

The crate follows a **Layered Architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                   PersistenceEngine                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Document I/O │  │Format Detection│ │Options Mgmt │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      Format Layer                               │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │ JSON Format  │  │Binary Format │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      Data Layer                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │EntityMapper  │  │SpatialHash  │  │LogicWiring   │          │
│  │(SoA↔AoS)     │  │Builder      │  │Serializer    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Document Structure

The core `Document` type contains all persistent data:

```rust
use archflow_persistence::{Document, PersistenceEngine};

let document = Document::with_title("Architecture Diagram".to_string());

// Document contains:
// - schema: Version and type definitions
// - meta: Metadata (title, author, timestamps)
// - store: EntityStore snapshot (all entities)
// - spatial_index: Optional pre-built hash grid
// - logic_wiring: Optional sensor-controller-actuator connections
```

### Entity Serialization

Individual entities are serialized as `EntityData`:

```rust
EntityData {
    id: EntityId,           // Unique identifier
    parent_id: Option<EntityId>, // Hierarchy
    transform: [f32; 4],    // [x, y, width, height]
    world_transform: [f32; 4], // World-space coords
    metadata: u32,          // Bit-packed flags
    color: u32,            // RGBA color
    texture_index: u16,    // Texture atlas reference
    color_tint: [f32; 4],  // Color multiplier
    text: Option<TextData>, // Text content
    arch_data: Option<ArchitectureData>, // C4 model data
    props: BTreeMap<String, PropValue>, // Custom properties
}
```

**SoA to AoS Transformation:**

The `EntityMapper` converts between the engine's Structure of Arrays (SoA) layout and the serialization-friendly Array of Structures (AoS) format:

```
EntityStore (SoA):
transforms: Vec<[f32; 4]>  →  EntityData (AoS)
colors: Vec<u32>            →  Individual records
metadata: Vec<u32>          →  Self-contained
...
```

### Format Selection

Choose the appropriate format for your use case:

| Format | Size | Speed | Human Readable | Use Case |
|--------|------|-------|----------------|----------|
| **JSON** | 2-3x | 2ms/1k | ✅ Yes | Version control, debugging |
| **Binary** | 1x | 1ms/1k | ❌ No | Production, performance |
| **+Gzip** | 0.2-0.4x | +5ms | ❌ No | Large documents, network |

### Spatial Indexing

Pre-build spatial indexes for O(1) queries:

```rust
use archflow_persistence::spatial::{SpatialHashBuilder, SpatialIndexData};

// Build spatial index from entity data
let builder = SpatialHashBuilder::new();
let index_data = builder.build_engine_hash(&store_snapshot)?;

// Include in document
document.spatial_index = Some(index_data);

// Load and use
if let Some(index) = document.spatial_index {
    let entities = index.query_point(Vec2::new(100.0, 200.0));
}
```

**Cell Sizes:**
- **64px** - Default for rendering operations
- **40px** - Optimized for Logic Bricks queries

### Logic Wiring Persistence

Sensor-Controller-Actuator connections are serialized:

```rust
SerializableWiring {
    sensors: Vec<SerializableSensor>,       // Input definitions
    controllers: Vec<SerializableController>, // Logic definitions
    actuators: Vec<SerializableActuator>,   // Output definitions
    connections: Vec<SerializableConnection>, // Wiring between them
}
```

**Pre-built Templates:**
- `highlight_wiring()` - Mouse hover highlighting
- `select_wiring()` - Click selection behavior
- `drag_wiring()` - Drag-and-drop interactions

## Usage Examples

### Basic Save/Load

```rust
use archflow_persistence::{PersistenceEngine, SerializationFormat};

let engine = PersistenceEngine::new();

// Save to JSON
let json_bytes = engine.export_bytes(&document)?;

// Load from JSON
let loaded_doc = engine.import_bytes(&json_bytes)?;
```

### Binary with Compression

```rust
use archflow_persistence::{PersistenceOptions, SerializationFormat, CompressionOption};

let options = PersistenceOptions::new()
    .with_format(SerializationFormat::Binary)
    .with_compression(CompressionOption::Gzip);

let engine = PersistenceEngine::with_options(options);

// Compressed binary output
let bytes = engine.export_bytes(&document)?;
```

### Complete Document Creation

```rust
use archflow_persistence::{Document, DocumentMeta, Schema};
use archflow_core::Timestamp;

let mut document = Document::with_title("My Architecture".to_string());

// Set metadata
document.meta = DocumentMeta {
    title: "My Architecture".to_string(),
    author: "John Doe".to_string(),
    created_at: Timestamp::now(),
    modified_at: Timestamp::now(),
    version: "1.0.0".to_string(),
};

// Add spatial index
document.spatial_index = Some(build_spatial_index(&entity_store));

// Add logic wiring
document.logic_wiring = Some(build_logic_wiring());

// Save
let bytes = PersistenceEngine::new().export_bytes(&document)?;
```

### File I/O

```rust
use std::fs;

// Save to file
let bytes = engine.export_bytes(&document)?;
fs::write("diagram.archflow", bytes)?;

// Load from file
let bytes = fs::read("diagram.archflow")?;
let document = engine.import_bytes(&bytes)?;
```

### Auto-Detection

The engine automatically detects format and compression:

```rust
// Works with any combination:
// - Plain JSON
// - Gzipped JSON
// - Plain Binary
// - Gzipped Binary

let document = engine.import_bytes(&any_bytes)?;
```

**Detection Priority:**
1. Check for Gzip magic bytes
2. Check for JSON structure
3. Assume binary format

## Performance Characteristics

| Operation | 1K Entities | 10K Entities | 100K Entities |
|-----------|-------------|--------------|---------------|
| JSON Serialize | ~2ms | ~15ms | ~150ms |
| JSON Deserialize | ~3ms | ~25ms | ~280ms |
| Binary Serialize | ~1ms | ~8ms | ~80ms |
| Binary Deserialize | ~2ms | ~15ms | ~160ms |
| Spatial Index Build | ~0.5ms | ~3ms | ~30ms |
| Gzip Compression | ~2ms | ~8ms | ~50ms |

### Memory Usage

| Component | 1K Entities | 100K Entities |
|-----------|-------------|---------------|
| JSON Document | ~500KB | ~48MB |
| Binary Document | ~200KB | ~19MB |
| Spatial Index | ~16KB | ~1.6MB |
| Logic Wiring | ~1KB | ~10KB |

### Compression Ratios

Typical compression with Gzip:
- **JSON**: 60-80% reduction
- **Binary**: 70-85% reduction

## Integration with Other Crates

```toml
[dependencies]
archflow-persistence = "0.36"
archflow-engine = "0.36"   # For EntityStore
archflow-core = "0.36"     # For core types
archflow-logic = "0.36"    # For wiring types
```

### Data Flow

```
EntityStore → EntityMapper → StoreSnapshot → Serialization → Bytes
     ↓                                                        ↓
SpatialHash                                    Deserialization
     ↓                                                        ↓
SpatialIndexData                            StoreSnapshot → EntityMapper → EntityStore
```

## Schema Versioning

Documents include schema version information for backward compatibility:

```rust
Schema {
    version: u32,           // Schema version
    shape_types: Vec<String>, // Supported shape types
    custom_types: BTreeMap<String, String>, // Custom type definitions
}
```

**Migration Strategy:**
1. Read document schema version
2. Apply migration if needed
3. Load with current version

## Error Handling

```rust
use archflow_persistence::{PersistenceError, SerializeError, DeserializeError};

match engine.export_bytes(&document) {
    Ok(bytes) => { /* ... */ }
    Err(PersistenceError::Serialize(SerializeError::TooManyEntities(count))) => {
        eprintln!("Too many entities: {}", count);
    }
    Err(PersistenceError::Io(e)) => {
        eprintln!("I/O error: {}", e);
    }
    Err(e) => {
        eprintln!("Unknown error: {:?}", e);
    }
}
```

## Best Practices

### Choose the Right Format

- **Development/Debugging**: JSON without compression
- **Production**: Binary with compression
- **Version Control**: JSON (git-friendly)
- **Network Transfer**: Binary with Gzip

### Spatial Index Guidelines

- **Always include** for documents with 1000+ entities
- **Choose 64px** for rendering-heavy documents
- **Choose 40px** for logic-heavy documents
- **Omit** for simple documents (<100 entities)

### Logic Wiring Guidelines

- **Include** when behaviors are part of the document
- **Omit** for simple diagrams without interactions
- **Use templates** for common behavior patterns

## Constraints and Limitations

### Current Constraints

- **Maximum Entities**: Limited by available memory
- **Schema Compatibility**: Forward compatibility not guaranteed
- **Thread Safety**: Documents are `Send + Sync` but not concurrent
- **Binary Format**: May change between minor versions

### Performance Considerations

- **Large Documents**: Use binary + gzip
- **Frequent Saves**: Consider incremental updates
- **Network I/O**: Enable compression for transfer

## References

- **EPIC-WEB-010**: Canvas rendering integration
- **EPIC-WEB-011**: Logic Bricks system
- **archflow-engine**: EntityStore data structures
- **archflow-logic**: Sensor-Controller-Actuator types

## License

MIT License - See LICENSE file for details.
