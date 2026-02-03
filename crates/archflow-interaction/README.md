# archflow-interaction

> **Professional Interaction Layer** - High-performance input processing, hit testing, camera controls, and real-time collaboration through CRDT synchronization.

## Overview

`archflow-interaction` provides the sensory and motor system for the ArchFlow visual programming platform. It implements professional-grade interaction patterns including zoom-to-cursor navigation, multi-pointer input handling, gizmo rendering, undo/redo functionality, and real-time collaboration through CRDT-based synchronization.

**Key Capabilities:**
- **O(1) hit testing** via spatial hash grid
- **Zoom-to-cursor navigation** (professional pattern like Figma/Google Maps)
- **Lock-free input sharing** via SharedArrayBuffer
- **Immediate-mode gizmo rendering** for visual feedback
- **Command sourcing** for undo/redo operations
- **CRDT synchronization** for real-time collaboration

## Architecture

The crate follows **Domain-Driven Design (DDD)** with clear separation between sensory input, motor output, and coordination logic:

```
┌─────────────────────────────────────────────────────────────────┐
│                      SENSORY LAYER                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │InputProcessor│  │ HitTester    │  │CameraController│        │
│  │(Multi-pointer)│  │(Spatial Hash)│  │(Zoom-to-cursor)│        │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                   COORDINATION LAYER                            │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │HistoryManager│  │ CrdtManager  │                            │
│  │(Undo/Redo)   │  │(Collaboration)│                           │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      MOTOR LAYER                                │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │GizmoRenderer │  │CommandExecutor│                            │
│  │(Visual Feedback)│(State Changes)│                           │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Input Processing

High-performance input handling with lock-free cross-thread communication:

```rust
use archflow_interaction::input::{InputProcessor, InputEventType, Buttons, Modifiers};

let mut input = InputProcessor::new();

// Process raw event (32-byte structure)
let event = RawInputEvent::new(
    InputEventType::PointerDown,
    Buttons::PRIMARY,
    Modifiers::empty(),
    Vec2::new(100.0, 200.0),
);

input.push_event(event);

// Drain and process events
for event in input.drain_events() {
    // Handle event
}
```

**Performance Features:**
- **32-byte event structures** for cache line efficiency
- **Lock-free ring buffer** for zero-copy event sharing between JS and WASM
- **MAX_POINTERS: 10** simultaneous inputs (multi-touch support)
- **EVENT_CAPACITY: 256** events before overflow

### Hit Testing

O(1) spatial queries using the spatial hash grid from archflow-engine:

```rust
use archflow_interaction::hit_testing::HitTester;

let hit_tester = HitTester;

// Find topmost entity at point
if let Some(entity_id) = hit_tester.find_at_point(
    Vec2::new(150.0, 200.0),
    &spatial_hash,
    &entity_store,
) {
    // Entity found at cursor position
}

// Find entities in selection rectangle
let selected = hit_tester.find_in_rect(
    Rect::new(100.0, 100.0, 200.0, 150.0),
    &spatial_hash,
    &entity_store,
);

// Find entities within proximity radius
let nearby = hit_tester.find_nearby(
    Vec2::new(150.0, 200.0),
    50.0,  // radius
    &spatial_hash,
    &entity_store,
);
```

**Query Types:**
| Method | Complexity | Use Case |
|--------|------------|----------|
| `find_at_point()` | O(1) avg | Click selection |
| `find_in_rect()` | O(c) where c = cells | Marquee selection |
| `find_contained()` | O(c) | Full containment test |
| `find_nearby()` | O(c) | Proximity-based selection |
| `hit_entity()` | O(1) | Direct entity hit test |

### Camera Controller

Professional 2D infinite canvas navigation with zoom-to-cursor:

```rust
use archflow_interaction::camera_controller::CameraController;

let mut camera = CameraController::new();

// Zoom to cursor (professional pattern)
camera.zoom_to_cursor(1.1, cursor_pos);  // 10% zoom in toward cursor
camera.zoom_to_cursor(0.9, cursor_pos);  // 10% zoom out toward cursor

// Pan gestures
camera.start_pan(drag_start_pos);
camera.pan_with_delta(delta);  // For touch/trackpad
camera.end_pan();

// Clamped zoom levels
// ZOOM_MIN: 0.01 (1% - zoomed out)
// ZOOM_MAX: 10.0 (1000% - zoomed in)
```

**Professional vs Amateur Zoom:**

```
Amateur (Canvas Center):        Professional (Cursor Position):
┌──────────────────┐            ┌──────────────────┐
│        [*]       │            │        [*]       │
│     Zoom here    │            │     Zoom here    │
│                  │            │                  │
└──────────────────┘            └──────────────────┘
Result: Zoom center             Result: Cursor position
```

The professional pattern maintains visual context and matches user expectations from tools like Figma, Google Maps, and CAD software.

### Gizmo Rendering

Immediate-mode UI elements for visual feedback:

```rust
use archflow_interaction::gizmos::{GizmoRenderer, Shape, Cursor};

let mut gizmos = GizmoRenderer::new();

// Draw selection rectangle
gizmos.draw_shape(Shape::Rect {
    position: Vec2::new(100.0, 100.0),
    size: Vec2::new(200.0, 150.0),
    color: 0x3B82F6,
    filled: false,
    line_width: 2.0,
});

// Draw resize handle
gizmos.draw_shape(Shape::Diamond {
    position: Vec2::new(300.0, 250.0),
    size: Vec2::new(8.0, 8.0),
    color: 0xFFFFFF,
    filled: true,
});

// Set cursor for feedback
gizmos.set_cursor(Cursor::ResizeNWSE);

// Submit to GPU
gizmos.submit();
```

**Shape Types:**
- `Rect` - Selection rectangles, bounds
- `Circle` / `Ellipse` - Radial handles
- `Line` - Guidelines, connections
- `Diamond` - Resize/rotation handles
- `Cross` - Center point markers

**Cursor Types:**
- `Move`, `ResizeNSEW` (8 directions), `Rotate`
- `Crosshair` for precision tools
- `Pointer`, `NotAllowed` for interactive feedback

### History Manager

Command sourcing pattern for undo/redo operations:

```rust
use archflow_interaction::history::{
    HistoryManager, 
    helpers::{move_entity, resize_entity},
};

let mut history = HistoryManager::new(100);  // 100 undo steps

// Execute reversible command
let cmd = move_entity(entity_id, Vec2::new(10.0, 5.0));
history.execute(&cmd, &mut entity_store);

// Undo
history.undo(&mut entity_store);

// Redo
history.redo(&mut entity_store);
```

**Built-in Commands:**
- `SetColor` - Color changes
- `Move` - Position translation
- `Resize` - Size changes
- `Teleport` - Absolute positioning
- `Spawn` / `Despawn` - Entity lifecycle

**Command Sourcing Pattern:**
```
Command → Redo Closure ─────┐
         └── Undo Closure ──┤
                            ▼
                    History Stack
                            │
                    Execution / Reversal
```

### CRDT Manager

Real-time collaboration with conflict resolution:

```rust
use archflow_interaction::crdt::CrdtManager;

let mut crdt = CrdtManager::new(user_id);

// Apply local command
let remote = crdt.apply_local(cmd, &mut entity_store);

// Broadcast to peers
broadcast_to_network(&remote);

// Receive remote command
crdt.apply_remote(&remote_cmd, &mut entity_store);
```

**Conflict Resolution:**
- **Lamport timestamps** for total ordering
- **Last-Write-Wins** for conflicting operations
- **Entity-aware** conflict detection
- **Command merging** for compatible operations

**Distributed System Features:**
- Causality tracking with happened-before relationships
- Concurrent operation detection
- Enhanced resolution with state verification

## Event Flow

```
User Input → SharedArrayBuffer → InputProcessor
                                      │
                                      ▼
                              HitTester (O(1) query)
                                      │
                                      ▼
                            Entity Selection
                                      │
                    ┌─────────────────┴─────────────────┐
                    ▼                                   ▼
            CameraController                      HistoryManager
            (Viewport Update)                      (State Change)
                    │                                   │
                    └─────────────────┬─────────────────┘
                                      ▼
                              GizmoRenderer
                              (Visual Feedback)
                                      │
                                      ▼
                              CrdtManager
                              (Network Sync)
```

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Hit Testing | O(1) avg | O(n) worst case |
| Event Processing | <1μs | Per event |
| Gizmo Capacity | 512 instances | Pre-allocated buffer |
| History Depth | 100 steps | Configurable |
| Event Capacity | 256 events | Ring buffer |
| Input Latency | <16ms | 60fps target |

## Memory Optimization

- **Zero-allocation hot path**: Pre-allocated buffers and pools
- **32-byte event structures**: Cache line efficiency
- **SOA (Structure of Arrays)**: GPU-friendly data layout
- **Lock-free sharing**: SharedArrayBuffer for cross-thread communication

## Usage Examples

### Complete Interaction Setup

```rust
use archflow_interaction::{
    input::InputProcessor,
    hit_testing::HitTester,
    camera_controller::CameraController,
    history::HistoryManager,
    crdt::CrdtManager,
    gizmos::GizmoRenderer,
};

// Initialize interaction systems
let mut input = InputProcessor::new();
let hit_tester = HitTester;
let mut camera = CameraController::new();
let mut history = HistoryManager::new(100);
let mut crdt = CrdtManager::new(user_id);
let mut gizmos = GizmoRenderer::new();

// Main loop
loop {
    // Process input events
    for event in input.drain_events() {
        match event.event_type {
            InputEventType::PointerDown => {
                if let Some(entity) = hit_tester.find_at_point(
                    event.position, &spatial_hash, &store
                ) {
                    // Start interaction
                }
            }
            InputEventType::PointerMove => {
                camera.pan_with_delta(event.delta);
            }
            InputEventType::Wheel => {
                camera.zoom_to_cursor(
                    1.0 + event.scroll_y * 0.001,
                    event.position
                );
            }
            _ => {}
        }
    }
    
    // Render gizmos
    gizmos.submit();
}
```

### Selection with Visual Feedback

```rust
// Handle pointer down
if let Some(entity_id) = hit_tester.find_at_point(
    cursor_pos, &spatial_hash, &store
) {
    // Draw selection highlight
    let entity = store.get(entity_id).unwrap();
    gizmos.draw_shape(Shape::Rect {
        position: entity.position,
        size: entity.size,
        color: 0x3B82F6,  // Blue selection
        filled: false,
        line_width: 2.0,
    });
    
    // Draw resize handles
    for handle in resize_handles(entity) {
        gizmos.draw_shape(Shape::Diamond {
            position: handle.position,
            size: Vec2::new(8.0, 8.0),
            color: 0xFFFFFF,
            filled: true,
        });
    }
}
```

## Integration with Logic Bricks

The interaction layer integrates with the Logic Bricks system (archflow-logic) through:

- **Sensors**: Mouse/Touch input events, spatial proximity detection
- **Actuators**: Entity transformation commands, gizmo visual feedback
- **Controllers**: Input processing pipeline, hit testing coordination

## Constraints and Limitations

### Current Constraints
- **Single-threaded coordination**: Main thread handles all UI logic
- **Fixed buffer sizes**: 512 gizmo instances, 256 event capacity
- **Limited conflict resolution**: Last-Write-Wins only (no merge yet)
- **2D camera only**: No 3D view support

### Scalability Considerations
- **Spatial hash grid** provides O(1) queries for reasonable entity counts
- **Ring buffer** may need overflow handling for bursty input
- **History depth** configurable but memory bounded
- **CRDT complexity** grows with concurrent users

## Architecture References

- **ARQUITECTURA_FINAL_V3.md**: Sections 6, 7, 13, 15, 16, 17
- **EPIC-WEB-010**: 2D Canvas rendering system
- **EPIC-WEB-011**: Behaviors SDK integration

## License

MIT License - See LICENSE file for details.
