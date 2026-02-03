# ArchFlow

> **Visual Programming Platform** - Professional architecture diagramming with real-time collaboration, powered by Rust and WebAssembly.

[![Version](https://img.shields.io/badge/version-0.36.1-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Rust](https://img.shields.io/badge/rust-1.85+-orange)
![WebAssembly](https://img.shields.io/badge/wasm-yes-brightgreen)

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Crate Documentation](#crate-documentation)
  - [Core Libraries](#core-libraries)
  - [Engine & Logic](#engine--logic)
  - [Rendering & Text](#rendering--text)
  - [Data & Export](#data--export)
  - [Web Application](#web-application)
- [Integration Guide](#integration-guide)
- [Development Guide](#development-guide)
- [Contributing](#contributing)

## Overview

**ArchFlow** is a professional-grade visual programming platform for creating, editing, and collaborating on architecture diagrams. It combines the performance of Rust with the flexibility of WebAssembly to deliver a real-time collaborative diagramming experience.

**Key Features:**
- **Real-time Collaboration** - CRDT-based multi-user editing with conflict resolution
- **Professional Canvas** - Infinite pan/zoom with professional zoom-to-cursor navigation
- **High Performance** - 100,000+ entities at 60 FPS using WebGPU rendering
- **Logic Bricks System** - Declarative behavior composition inspired by Blender
- **Multi-Format Export** - Mermaid diagrams, Terraform configurations, and more
- **C4 Model Support** - Built-in support for software architecture diagrams

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/hodei-archFlow.git
cd hodei-archFlow

# Build the project
cargo build --workspace --release

# Run tests
cargo test --workspace
```

### Basic Usage

```rust
use archflow_core::{Vec2, EntityId};
use archflow_engine::EntityStore;
use archflow_logic::{LogicSystem, LogicMappingTable};
use archflow_render::GpuRenderer;

// Create entity store
let mut store = EntityStore::new();

// Spawn an entity
let entity_id = store.spawn(
    Vec2::new(100.0, 200.0),
    Vec2::new(50.0, 30.0),
    None
);

// Create logic system for behaviors
let mut logic = LogicSystem::new();
let mapping = LogicMappingTable::default();
mapping.connect(
    SensorType::MouseOver,
    Controller::Direct,
    ActuatorType::Highlight
);

// Update and render
logic.update(&mut store, &mapping);
renderer.render(&camera);
```

### TypeScript/JavaScript Usage

```typescript
import { ArchFlow } from '@archflow/sdk';

// Create an interactive shape
const shape = ArchFlow.createShape('rectangle')
  .position(100, 200)
  .size(200, 150)
  .fillColor(0x3B82F6)
  .cornerRadius(8)
  .onHover({ color: 0xFFFF00, opacity: 0.5 })
  .onClick(() => console.log('Clicked!'))
  .draggable()
  .build();
```

## Architecture

ArchFlow follows **Hexagonal Architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                    WEB APPLICATION (TypeScript/React)           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │Canvas UI     │  │Properties UI │  │Toolbar UI    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │ WASM Bridge
┌─────────────────────────────┴───────────────────────────────────┐
│                      CORE LAYER (Rust)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │archflow-core │  │archflow-engine│  │archflow-logic│          │
│  │(Math/Types) │  │(Entity Mgmt) │  │(Behaviors)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │archflow-text │  │archflow-render│  │archflow-     │          │
│  │(Text Render) │  │(GPU/WebGPU)  │  │interaction   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │archflow-     │  │archflow-     │                            │
│  │diagram       │  │persistence   │                            │
│  │(C4 Model)    │  │(Save/Load)    │                            │
│  └──────────────┘  └──────────────┘                            │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │archflow-     │  │archflow-     │                            │
│  │export        │  │plugins       │                            │
│  │(Multi-Format) │  │(Importers)    │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Documentation

### Core Libraries

#### [archflow-core](crates/archflow-core/README.md)

**Foundation types and math primitives for the entire system.**

- **EntityId** - Generational indexing (24-bit index + 8-bit generation)
- **Vec2** - 2D vector math with comprehensive operations
- **Rect** - Rectangle operations for spatial queries
- **Transform** - 2D affine transformations
- **Animation** - 75+ easing functions (polynomial, trigonometric, elastic)
- **Zoom System** - 4-level zoom with progressive disclosure

**Key Types:**
```rust
pub struct EntityId(u32);  // Packed: [index:24, generation:8]
pub struct Vec2(f32, f32);
pub struct Rect { x, y, width, height }
```

**Integration:** Used by all other crates as the foundation.

---

#### [archflow-diagram](crates/archflow-diagram/README.md)

**C4 model entities for software architecture diagrams.**

- **Person** - Human actors in the system
- **SoftwareSystem** - Major software systems
- **Container** - Applications/services
- **Database** - Data storage systems
- **MessageQueue** - Async messaging systems
- **Cloud Providers** - AWS, GCP, Azure entities

**Key Features:**
- C4 model hierarchy (System → Container → Component → Code)
- Cloud provider integration for IaC generation
- Export to Mermaid, Terraform, DrawIO

**Integration:** Uses archflow-core for geometry, archflow-export for serialization.

---

### Engine & Logic

#### [archflow-engine](crates/archflow-engine/README.md)

**High-performance entity management with Structure of Arrays (SoA) memory layout.**

- **EntityStore** - 100K entities with hot/cold data separation
- **Command Pattern** - Transactional operations (Spawn, Despawn, Move, etc.)
- **Spatial Hashing** - O(1) spatial queries via grid-based indexing
- **Security Layer** - RBAC, rate limiting, HMAC signing, audit logging
- **Network Optimization** - Multi-strategy compression (70-90% reduction)
- **Undo/Redo** - Command sourcing with branching history

**Performance:**
- Max 100,000 entities, 200,000 connections
- O(1) average spatial queries
- ~128 bytes per entity memory usage

**Integration:** Core domain logic used by interaction, render, and web-ui.

---

#### [archflow-logic](crates/archflow-logic/README.md)

**Logic Bricks system - event-driven reactive architecture inspired by Blender's Game Engine.**

- **Sensors** - Input detection (Mouse, Keyboard, Spatial)
- **Controllers** - Boolean logic (AND, OR, NOT) and advanced (Debounce, Hysteresis)
- **Actuators** - Command generation (Select, Move, Highlight, Camera)
- **SignalByte** - 1 byte per entity per sensor for 6-tick history

**Memory Efficiency:**
- 1 byte/entity/sensor (400KB for 100K entities × 4 sensors)
- <0.1μs per sensor evaluation

**Integration:** Maps to archflow-interaction for input, archflow-engine for state changes.

---

#### [archflow-interaction](crates/archflow-interaction/README.md)

**Professional interaction layer - sensory and motor system for the application.**

- **Input Processing** - Lock-free SharedArrayBuffer for 32-byte events
- **Hit Testing** - O(1) spatial queries via hash grid
- **Camera Controller** - Zoom-to-cursor navigation (Figma/Google Maps style)
- **Gizmo Rendering** - Immediate-mode UI for visual feedback
- **History Manager** - Command sourcing for undo/redo
- **CRDT Manager** - Real-time collaboration with Lamport timestamps

**Performance:**
- 32-byte event structures for cache efficiency
- 512 gizmo instances pre-allocated buffer
- 90%+ culling effectiveness with viewport testing

**Integration:** Bridges UI input to engine commands, handles collaboration synchronization.

---

### Rendering & Text

#### [archflow-render](crates/archflow-render/README.md)

**High-performance WebGPU-based 2D rendering system.**

- **Multi-phase Rendering** - 4 specialized pipelines (Shapes, Icons, Images, Text)
- **Instanced Rendering** - Single draw call for thousands of entities
- **Infinite Camera** - 0.01x to 100x zoom range with pan/zoom
- **Texture Atlases** - Shelf-packing algorithm for efficient GPU utilization
- **WGSL Shaders** - Custom shaders for each render phase

**Performance:**
- 4ms total frame time for 100K entities
- 4.8MB GPU instance buffer
- 90%+ reduction with viewport culling

**Integration:** Receives entity data from archflow-engine, textures from archflow-text.

---

#### [archflow-text](crates/archflow-text/README.md)

**SDF/MTSDF-based text rendering with subpixel accuracy.**

- **SDF Generation** - Crisp text at any scale
- **MTSDF Enhancement** - Better diagonal edge rendering
- **Glyph Caching** - 90%+ cache hit ratio
- **Text Layout** - Shaping, line breaking, alignment
- **`no_std` Compatible** - Works in embedded/WASM environments

**Performance:**
- ~5ms per glyph (32px) for SDF generation
- <0.1ms cache lookup time
- Subpixel-accurate positioning

**Integration:** Provides texture atlases to archflow-render for GPU text rendering.

---

### Data & Export

#### [archflow-persistence](crates/archflow-persistence/README.md)

**Document serialization and persistence layer.**

- **Multiple Formats** - JSON (human-readable) and Binary (optimized)
- **Compression** - Optional Gzip for 60-80% size reduction
- **Spatial Indexing** - Pre-built hash grids for O(1) queries
- **Logic Wiring** - Sensor-Controller-Actuator connection persistence
- **Schema Versioning** - Backward compatibility support

**Performance:**
- 1ms/1K entities (binary), 2ms/1K (JSON)
- 60-80% compression with Gzip
- Auto-detection of format and compression

**Integration:** Saves/loads archflow-engine state, integrates with export formats.

---

#### [archflow-export](crates/archflow-export/README.md)

**Multi-format export system for diagrams and infrastructure.**

- **Mermaid Export** - Flowchart, Sequence, State, ER, Class diagrams
- **Terraform Export** - AWS, GCP, Azure infrastructure generation
- **Binary Serialization** - Compact project persistence
- **Format-Specific Modules** - Pluggable export architecture

**Supported Formats:**
- Mermaid (5 diagram types)
- Terraform (3 cloud providers)
- Binary (with integrity validation)

**Integration:** Uses archflow-diagram for C4 model mapping, archflow-persistence for data.

---

#### [archflow-plugins](crates/archflow-plugins/README.md)

**External integrations for interoperability.**

- **Draw.io Import** - 3-layer decode (URL → Base64 → Deflate → XML)
- **SVG Parsing** - Icon library extraction
- **SVG Rasterization** - GPU texture conversion
- **Atlas Packing** - Shelf-packing for textures

**Decoding Pipeline:**
```
Draw.io Data → URL Decode → Base64 → Inflate → XML → Icons
```

**Integration:** Provides textures to archflow-render, icons to archflow-diagram.

---

### Web Application

#### [archflow-web-ui](crates/archflow-web-ui/README.md)

**React-based web application with WASM backend.**

- **Canvas Component** - Main rendering interface
- **Properties Panel** - Entity property editing
- **Toolbar** - Tool palette and actions
- **State Management** - Zustand stores (Entity, Canvas, Collaboration)
- **WASM Bridge** - Seamless Rust backend integration
- **Real-time Collaboration** - CRDT-based multi-user editing

**Performance:**
- 60 FPS with 1000+ entities
- <2s initial load including WASM
- <100ms collaboration latency

**Sub-modules:**
- **SDK** ([`src/sdk/README.md`](crates/archflow-web-ui/src/sdk/README.md)) - Fluent TypeScript API for shape creation

---

## Integration Guide

### Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                    archflow-web-ui                         │
│                    (TypeScript/React)                      │
└─────────────────────────────┬───────────────────────────────┘
                              │ WASM
┌─────────────────────────────┴───────────────────────────────────┐
│                   Core Libraries Layer                      │
│                                                              │
│  archflow-core ──────────────────────────────────────┐   │
│       │                                                    │   │
│       ├─► archflow-diagram ──────────────┬────────────┘   │
│       │                                │                  │   │
│       ├─► archflow-engine ────────┬──────┴──────────┐   │
│       │                           │                  │   │
│       ├─► archflow-logic ─────────┴─────────┐        │   │
│       │                                     │        │   │
│       ├─► archflow-interaction ──────────┐││        │   │
│       │                              │ ││        │   │
│       ├─► archflow-text ───────────────┐│││        │   │
│       │                              ││││        │   │
│       └─► archflow-render ────────────┘│││        │   │
│                                     ││││        │   │
│  Data Layer                          │││        │   │
│  ├─► archflow-persistence ──────────┘││        │   │
│  │                                     ││        │   │
│  └─► archflow-export ─────────────────┘│        │   │
│                                        │        │   │
│  External Integrations                     │        │   │
│  └─► archflow-plugins ───────────────────┘        │   │
│                                                   │   │
└───────────────────────────────────────────┴───┴───┘
                                                    │
                                            Architecture Models
```

### Common Patterns

#### Creating an Entity

```rust
use archflow_core::Vec2;
use archflow_engine::EntityStore;

let mut store = EntityStore::new();
let entity_id = store.spawn(
    Vec2::new(100.0, 200.0),
    Vec2::new(50.0, 30.0),
    None
);
```

#### Adding Behaviors

```rust
use archflow_logic::{LogicMappingTable, SensorType, Controller, ActuatorType};

let mut mapping = LogicMappingTable::default();
mapping.connect(
    SensorType::MouseOver,
    Controller::Direct,
    ActuatorType::Highlight
);
```

#### Spatial Query

```rust
use archflow_interaction::hit_testing::HitTester;

let hit_tester = HitTester;
let entities = hit_tester.find_at_point(
    Vec2::new(150.0, 200.0),
    &spatial_hash,
    &entity_store
);
```

#### Export to Mermaid

```rust
use archflow_export::MermaidExporter;

let exporter = MermaidExporter::new()
    .with_diagram_type(MermaidDiagramType::Flowchart);

let mermaid = exporter.export(&entity_store, &connection_store);
```

## Development Guide

### Building

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --workspace --release

# Single crate
cargo build -p archflow-engine
```

### Testing

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p archflow-engine

# With output
cargo test --workspace -- --nocapture
```

### Code Quality

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Check (faster than build)
cargo check --all-targets
```

### Documentation

```bash
# Build docs
cargo doc --no-deps --open

# Build crate docs
cargo doc -p archflow-engine --open
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Ensure all checks pass (`cargo fmt`, `cargo clippy`, `cargo test`)
6. Submit a pull request

### Code Standards

- **Conventional Commits** - Follow conventional commit format
- **TDD** - Write tests before implementation
- **Documentation** - Document public APIs with KDoc (Rust) or TSDoc (TypeScript)
- **SOLID Principles** - Maintain clean architecture
- **No Placeholders** - All implementations must be functional

## License

MIT License - See [LICENSE](LICENSE) for details.

## Acknowledgments

- **Blender Game Engine** - Inspiration for Logic Bricks system
- **Figma** - Professional zoom-to-cursor navigation pattern
- **Mermaid.js** - Diagram generation
- **WebGPU** - High-performance rendering
- **Rust Community** - Excellent language and tooling

---

**Version:** 0.36.1  
**Documentation Last Updated:** 2026-02-03
