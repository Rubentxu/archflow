# archflow-engine

> **High-Performance Entity Management System** - Domain layer for collaborative architecture diagrams with real-time synchronization, security, and spatial indexing.

## Overview

`archflow-engine` provides the core domain logic for managing entities in architectural diagrams. It implements a **Component Entity System (CES)** with **Structure of Arrays (SoA)** memory layout for optimal cache performance, designed to handle 100,000+ entities with sub-millisecond query times.

**Key Capabilities:**
- **O(1) spatial queries** via grid-based hashing
- **Command pattern** for transactional operations
- **Event sourcing** with full audit trail
- **Security layer** with RBAC, rate limiting, and HMAC signing
- **Network optimization** with multi-strategy compression
- **Undo/Redo system** with branching history support

## Architecture

The engine follows **Domain-Driven Design (DDD)** principles with hexagonal architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Application Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ HistoryService│  │SecurityService│  │CompressionService│     │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                      Domain Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ EntityStore  │  │ SpatialHash  │  │CommandQueue  │          │
│  │ (SoA Layout) │  │ (O(1) Queries)│  │ (Transactional)│       │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │ConnectionStore│  │StringPool   │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Entity Management

Entities are the fundamental domain objects, managed through the `EntityStore`:

```rust
use archflow_engine::EntityStore;

let mut store = EntityStore::new();

// Spawn entity at position
let id = store.spawn(Vec2::new(100.0, 200.0), Vec2::new(50.0, 30.0), None);

// Entity properties
let entity = store.get(id);
assert_eq!(entity.position, Vec2::new(100.0, 200.0));
assert_eq!(entity.size, Vec2::new(50.0, 30.0));
```

### Command Pattern

All mutations go through the `Command` enum for transactional processing:

```rust
use archflow_engine::Command;

let command = Command::Move {
    id: EntityId::new(1),
    delta: Vec2::new(10.0, 5.0),
};

// Execute through command queue
queue.execute(command);
```

**Available Commands:**
- `Spawn` - Create new entity
- `Despawn` - Remove entity
- `Move` - Translate entity
- `Scale` - Resize entity
- `Rotate` - Change rotation
- `SetColor` - Update color
- `SetTexture` - Change texture
- `SetParent` - Establish hierarchy
- `AddChild` - Add to parent
- `RemoveChild` - Remove from parent
- `AddConnection` - Create relationship
- `RemoveConnection` - Remove relationship
- `SetUserData` - Store custom data

### Spatial Hashing

Grid-based spatial indexing provides O(1) queries for collision detection and region searches:

```rust
use archflow_engine::SpatialHash;

let mut spatial = SpatialHash::new(64.0, 100_000);

// Insert entity
spatial.insert(entity_id, bounds);

// Query point
let results = spatial.query_point(Vec2::new(150.0, 200.0));

// Query rectangle
let results = spatial.query_rect(Rect::new(100.0, 100.0, 200.0, 200.0));

// Query named region
spatial.insert_region("canvas", Rect::new(0.0, 0.0, 800.0, 600.0));
let results = spatial.query_region("canvas");
```

**Algorithm Details:**
- **Cell Size**: Configurable (default 64px)
- **Hash Function**: `(floor(x / cell_size), floor(y / cell_size))`
- **Complexity**: O(1) average, O(n) worst case (all entities in same cell)
- **Memory**: O(n) where n = number of entities

## Security Layer

The engine provides a comprehensive security framework for multi-user collaboration:

### Role-Based Access Control (RBAC)

```rust
use archflow_engine::security::{SecurityService, Role, Permission};

let security = SecurityService::new();

// Define roles
security.add_role(Role::Viewer);
security.add_role(Role::Editor);
security.add_role(Role::Admin);

// Grant permissions
security.grant_permission(Role::Editor, Permission::CreateEntity);
security.grant_permission(Role::Editor, Permission::UpdateEntity);

// Check permissions
security.check_permission(user_id, Permission::DeleteEntity)?;
```

### Rate Limiting (Token Bucket)

```rust
use archflow_engine::security::TokenBucket;

let mut limiter = TokenBucket::new(100, 10); // 100 tokens, 10 refill/sec

// Consume tokens
limiter.consume(5)?; // Success
limiter.consume(150)?; // Error: Rate limit exceeded
```

**Algorithm:**
```
tokens = min(capacity, tokens + (now - last_refill) * refill_rate)
if tokens >= requested:
    tokens -= requested
    return Success
return RateLimitExceeded
```

### HMAC Command Signing

```rust
use archflow_engine::security::HmacSigner;

let signer = HmacSigner::new(secret_key);
let signature = signer.sign(&command);

let verifier = HmacSigner::new(secret_key);
assert!(verifier.verify(&command, &signature));
```

## Network Optimization

### Command Compression

Multi-strategy compression reduces network payload by 70-90%:

```rust
use archflow_engine::compression::{CompressedBatch, BatchBuilder};

let mut builder = BatchBuilder::new();

// Add commands - applies automatic compression
builder.add(Command::Move { id, delta: Vec2::new(1.0, 0.0) });
builder.add(Command::Move { id, delta: Vec2::new(2.0, 0.0) });
builder.add(Command::Move { id, delta: Vec2::new(3.0, 0.0) });

// Build compressed batch
let batch: CompressedBatch = builder.build();

// Decompress on receiver
let commands = batch.decompress()?;
```

**Compression Strategies:**
1. **Deduplication**: Remove consecutive duplicate commands
2. **Run-Length Encoding**: Compress repetitive patterns
3. **Delta Encoding**: Store differences instead of absolute values

## Undo/Redo System

Versioned command history supports linear and branching timelines:

```rust
use archflow_engine::history::{CommandHistory, CommandGroup};

let mut history = CommandHistory::new();

// Execute command group
let mut group = CommandGroup::new("Resize entity");
group.add(Command::Scale { id, scale: Vec2::new(2.0, 2.0) });
history.execute_group(group);

// Undo
history.undo()?;

// Redo
history.redo()?;
```

## Data Structures

### EntityStore (SoA Layout)

The Structure of Arrays layout maximizes cache locality:

```rust
pub struct EntityStore {
    // Hot data - frequently accessed (cache line 0)
    transforms: Vec<[f32; 4]>,      // [x, y, width, height]
    metadata: Vec<u32>,              // Bit-packed: [visible:1, locked:1, gen:8, type:6, reserved:16]
    colors: Vec<u32>,                // RGBA colors
    texture_index: Vec<u32>,        // Texture references
    
    // Cold data - less frequently accessed
    parents: Vec<Option<EntityId>>,
    children: Vec<Vec<EntityId>>,
    connections: Vec<Vec<Connection>>,
    user_data: Vec<DataType>,
    
    // String interning
    string_pool: StringPool,
}
```

**Benefits:**
- **Cache Efficiency**: Contiguous access patterns
- **SIMD Friendly**: Same-type data enables vectorization
- **Memory Efficiency**: Hot data fits in L1 cache

### ConnectionStore

Optimized for 200,000 connections:

```rust
pub struct ConnectionStore {
    from: Vec<EntityId>,
    to: Vec<EntityId>,
    anchor_from: Vec<Anchor>,
    anchor_to: Vec<Anchor>,
    style: Vec<LineStyle>,
}
```

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Max Entities | 100,000 | Configurable via feature |
| Max Connections | 200,000 | 2:1 connection ratio |
| Spatial Query | O(1) avg | O(n) worst case |
| Command Execution | O(1) | Constant time |
| Memory per Entity | ~128 bytes | Hot + cold data |
| Compression Ratio | 70-90% | Depends on command pattern |
| Rate Limiting | O(1) | Token bucket algorithm |

## Port Interfaces

The engine exposes port interfaces following hexagonal architecture:

### EntityStorePort

```rust
pub trait EntityStorePort {
    fn spawn(&mut self, pos: Vec2, size: Vec2, parent: Option<EntityId>) -> EntityId;
    fn despawn(&mut self, id: EntityId) -> Result<()>;
    fn get(&self, id: EntityId) -> Option<&Entity>;
    fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity>;
}
```

### SpatialQueryPort

```rust
pub trait SpatialQueryPort {
    fn query_point(&self, point: Vec2) -> Vec<EntityId>;
    fn query_rect(&self, rect: Rect) -> Vec<EntityId>;
    fn query_region(&self, name: &str) -> Vec<EntityId>;
}
```

### CommandExecutorPort

```rust
pub trait CommandExecutorPort {
    fn execute(&mut self, command: Command) -> Result<()>;
    fn execute_batch(&mut self, commands: Vec<Command>) -> Result<Vec<CommandResult>>;
}
```

## Constraints and Limits

### Hard Limits

- **MAX_ENTITIES**: 100,000 (EntityId uses 24-bit index)
- **MAX_CONNECTIONS**: 200,000
- **MAX_GLYPHS**: 500,000 (text rendering)
- **MAX_TEXT_LENGTH**: 50,000 characters
- **COMMAND_QUEUE_SIZE**: 1,024 commands
- **SPATIAL_CELL_SIZE**: 64 pixels (default)

### Memory Requirements

| Component | Memory | Notes |
|-----------|--------|-------|
| EntityStore | ~12.8 MB | 100K entities × 128 bytes |
| SpatialHash | ~2 MB | Depends on spatial distribution |
| CommandQueue | ~256 KB | 1024 commands × 256 bytes |
| ConnectionStore | ~9.6 MB | 200K connections × 48 bytes |

## Usage Examples

### Basic Entity Management

```rust
use archflow_engine::{EntityStore, Vec2};
use archflow_core::EntityId;

let mut store = EntityStore::new();

// Create entities
let parent = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(200.0, 150.0), None);
let child = store.spawn(Vec2::new(20.0, 20.0), Vec2::new(50.0, 30.0), Some(parent));

// Update properties
let entity = store.get_mut(parent).unwrap();
entity.color = 0xFF0000FF; // Red
entity.rotation = 45.0;
```

### Spatial Queries

```rust
use archflow_engine::SpatialHash;

let mut spatial = SpatialHash::new(64.0, 1000);

// Insert entities
spatial.insert(id1, Rect::new(10.0, 10.0, 50.0, 50.0));
spatial.insert(id2, Rect::new(60.0, 60.0, 50.0, 50.0));

// Find entities in region
let nearby = spatial.query_rect(Rect::new(0.0, 0.0, 100.0, 100.0));
assert_eq!(nearby.len(), 2);
```

### Command Processing

```rust
use archflow_engine::{Command, CommandQueue};

let mut queue = CommandQueue::new();

queue.execute(Command::Spawn { 
    pos: Vec2::new(0.0, 0.0), 
    size: Vec2::new(100.0, 100.0), 
    parent: None 
});

queue.execute(Command::Move { 
    id: EntityId::new(0), 
    delta: Vec2::new(10.0, 5.0) 
});
```

### Security Integration

```rust
use archflow_engine::security::{SecurityService, Role, Permission};

let mut security = SecurityService::new();

// Setup roles
security.add_role(Role::Viewer);
security.add_role(Role::Editor);
security.grant_permission(Role::Editor, Permission::CreateEntity);

// Assign role to user
security.assign_role(user_id, Role::Editor);

// Check permission
security.check_permission(user_id, Permission::CreateEntity)?;
```

## `no_std` Compatibility

This crate is `#![no_std]` compatible and uses `alloc` for dynamic collections:

```toml
[dependencies.archflow-engine]
version = "0.36"
features = ["std"] # Optional, for std::error::Error

# For no_std
default-features = false
```

## Architecture Decisions

### Why Structure of Arrays (SoA)?

**Traditional AoS (Array of Structures):**
```rust
struct Entity { x, y, w, h, color, ... }
Vec<Entity> // = [x,y,w,h,color, x,y,w,h,color, ...]
```

**SoA (Structure of Arrays):**
```rust
transforms: Vec<[f32; 4]>  // = [x,y,w,h, x,y,w,h, ...]
colors: Vec<u32>           // = [color, color, ...]
```

**Benefits:**
- Cache-friendly iteration (process all x positions, then all y)
- SIMD vectorization opportunities
- Better memory locality for hot data

### Why Command Pattern?

- **Audit Trail**: Every mutation is logged
- **Undo/Redo**: Commands can be reversed
- **Network Sync**: Commands serialize efficiently
- **Security**: Commands can be signed and verified
- **Transactions**: Multiple commands can be grouped atomically

### Why Spatial Hashing?

Alternatives considered:
- **Quadtree**: O(log n) but more complex, worse cache behavior
- **R-Tree**: Good for dynamic data, but higher constant factor
- **Brute Force**: O(n²) - unacceptable for large diagrams

Spatial hashing provides:
- **O(1)** average case for uniform distribution
- **Simple implementation** with predictable performance
- **Low memory overhead** compared to tree structures

## References

- **EPIC-WEB-011**: Behaviors SDK integration
- **EPIC-WEB-010**: 2D Canvas rendering
- **EPIC-WEB-012**: Logic Bricks system

## License

MIT License - See LICENSE file for details.
