# ArchFlow

<p align="center">
  <strong>A Production-Ready 2D Graphics Engine Built in Rust</strong>
</p>

<p align="center">
  <a href="https://github.com/Rubentxu/archflow/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/Rubentxu/archflow/ci.yml?branch=main" alt="CI Status">
  </a>
  <a href="https://github.com/Rubentxu/archflow/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
  <a href="https://github.com/Rubentxu/archflow">
    <img src="https://img.shields.io/badge/Rust-1.80+-orange.svg" alt="Rust Version">
  </a>
</p>

<p align="center">
  <em>Enterprise-grade 2D graphics engine with Zero Trust security principles</em>
</p>

---

## 🎯 Purpose

ArchFlow is a **production-ready 2D graphics engine** designed from the ground up for building professional diagramming tools, node-based editors, and interactive visual applications. Built entirely in Rust, it provides a robust foundation for applications requiring:

- **Diagramming & Flowchart Tools**: Create node-based editors with ports and connections
- **Vector Graphics Rendering**: High-quality Bézier curve rendering with multiple backends
- **Interactive Canvas Applications**: Drag, resize, select, and manipulate graphical elements
- **WebAssembly Deployment**: Run your graphics applications in browsers with native performance

### Key Design Principles

1. **Zero Trust Security**: Every component is designed with security in mind, from input validation to secure defaults
2. **Production Ready**: Comprehensive testing, documentation, and clean code practices
3. **Extensible Architecture**: Hexagonal architecture (Ports & Adapters) for maximum flexibility
4. **Type Safety**: Leverage Rust's type system to prevent runtime errors at compile time

---

## 🚀 Features

### Core Infrastructure

- **Custom Type System**: `Vec2`, `Mat3`, `Rect`, `Color`, `EntityId` with full serialization support
- **Entity Component System**: Clean abstraction for managing graphical entities
- **Transformations**: Translation, rotation, and scale operations with matrix support

### Primitives System

- **Shape Primitives**: Rectangle, Ellipse, Line, Polyline with full geometric properties
- **Style System**: Comprehensive styling with Fill, Stroke, Text, and Effect support
- **Ports & Connections**: Full connectivity system for node-based diagrams with smart routing

### Geometry Engine

- **Bézier Curves**: Quadratic and cubic Bézier curve support via kurbo
- **Path Operations**: Path creation, transformation, and simplification (Ramer-Douglas-Peucker)
- **Intersection Detection**: SAT algorithm, ray casting, and precise hit testing
- **Spatial Indexing**: Efficient spatial queries for large scenes

### Renderer

- **Abstract Renderer Trait**: Backend-agnostic rendering interface
- **Canvas 2D Backend**: Web-compatible rendering via web-sys
- **Rough Renderer**: Hand-drawn style rendering for sketches

---

## 📦 Crate Structure

```
crates/
├── archflow-core/           # Core types and domain primitives
├── archflow-ecs/            # Entity Component System
├── archflow-geometry/       # Geometry engine with kurbo
├── archflow-primitives/     # Shapes, styles, ports & connections
├── archflow-renderer/       # Abstract renderer traits
├── archflow-renderer-canvas/ # Canvas 2D implementation
├── archflow-renderer-rough/  # Rough/hand-drawn style renderer
├── archflow-workspace/      # Document and workspace management
└── archflow-wasm/           # WebAssembly bindings
```

---

## 🛠️ Getting Started

### Prerequisites

- **Rust**: 1.80 or later
- **Cargo**: Latest stable version
- **Git**: For version control

### Installation

```bash
# Clone the repository
git clone https://github.com/Rubentxu/archflow.git
cd archflow

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run benchmarks
cargo bench --workspace
```

### Example Usage

```rust
use archflow_core::{Vec2, EntityId};
use archflow_primitives::{Rectangle, FillStyle, StrokeStyle};
use archflow_geometry::GeometryEngine;

// Create a rectangle primitive
let rect = Rectangle::new(
    Vec2::new(0.0, 0.0),
    Vec2::new(100.0, 50.0),
);

// Apply styles
let styled_rect = rect
    .with_fill(FillStyle::solid("#FF5733"))
    .with_stroke(StrokeStyle::new("#333333", 2.0));

// Use geometry engine for calculations
let engine = GeometryEngine::default();
let center = engine.rect_center(rect.global_bounds());
let area = rect.local_bounds().area();
```

---

## 📚 Documentation

- **Architecture**: See `docs/ARCHITECTURE-DESIGN.md`
- **EPICS & User Stories**: See `docs/EPICS-ENGINE-2D.md`
- **API Documentation**: Run `cargo doc --open` to generate local documentation

---

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p archflow-geometry

# Run tests with coverage
cargo tarpaulin --workspace
```

---

## 📦 Dependencies

ArchFlow uses carefully selected dependencies:

| Crate | Version | Purpose |
|-------|---------|---------|
| `kurbo` | 0.13 | 2D geometry and Bézier curves |
| `glam` | 0.31 | SIMD-accelerated math |
| `serde` | 1.0 | Serialization |
| `uuid` | 1.11 | Entity identification |
| `web-sys` | 0.3 | WebAssembly DOM bindings |

---

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [kurbo](https://github.com/linebender/kurbo) - Excellent 2D geometry library
- [glam](https://github.com/bitshifter/glam-rs) - High-performance math library
- [tldraw](https://github.com/tldraw/tldraw) - Inspiration for the primitives design
- [React Flow](https://github.com/xyflow/react-flow) - Reference for node-based diagrams

---

<p align="center">
  Built with ❤️ using Rust
</p>
