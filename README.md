# ArchFlow

<p align="center">
  <strong>The Living Architecture Platform</strong>
</p>

<p align="center">
  <em>Design, Simulate, Deploy, and Evolve Your Cloud Architecture</em>
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

---

## 🎯 What is ArchFlow?

ArchFlow is a **Living Architecture Platform** that transforms how organizations design, collaborate, simulate, and deploy cloud-native and hybrid architectures. We bridge the gap between visual design tools (Figma, draw.io) and infrastructure as code (Terraform, Pulumi) by making the architectural diagram the **single source of truth** that is both visual and executable.

### The Problem We're Solving

| Challenge | Impact |
|-----------|--------|
| **Architecture Drift** | Diagrams become outdated as soon as they're created |
| **Tool Fragmentation** | Architects use 5+ tools (diagramming, IaC, documentation, collaboration) |
| **Implementation Gaps** | Beautiful diagrams don't translate to deployable infrastructure |
| **Cost Surprises** | Architecture decisions made without cost visibility |

### The ArchFlow Solution

```
┌─────────────────────────────────────────────────────────────────┐
│                    ArchFlow Platform                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   🎨 DESIGN          👥 COLLABORATE          🔬 SIMULATE       │
│   Visual Editor      Real-time Editing       Cost & Performance │
│   Component Library  Git-Native Versioning   Failure Scenarios  │
│   Smart Connections  Context-Aware Comments  Security Analysis  │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   🚀 DEPLOY          📊 ANALYZE             🤖 AI ASSIST       │
│   IaC Generation     Compliance Reports      Smart Suggestions │
│   Cloud Sync         Cost Optimization       Pattern Detection │
│   Drift Detection    Evolution Tracking      Auto-Documentation│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Core Capabilities

### 1. Visual Architecture Design

The foundation of ArchFlow is a high-performance, Rust-powered rendering engine that enables:

- **Infinite Canvas** with nested architectural frames
- **C4 Model Integration**: Seamless transitions between Context, Container, Component, and Code levels
- **Smart Component Library**: AWS, Azure, GCP, and custom enterprise components
- **Semantic Connections**: Ports and relationships that maintain architectural meaning

### 2. Architecture as Code (AaC)

Define your architecture in code, export to any IaC format:

| Export Target | Status | Use Case |
|---------------|--------|----------|
| Terraform | ✅ MVP | Infrastructure provisioning |
| Kubernetes | ✅ MVP | Container orchestration |
| Pulumi | 🔜 Phase 2 | Multi-language IaC |
| AWS CDK | 🔜 Phase 2 | AWS-specific constructs |
| CloudFormation | 🔜 Phase 2 | AWS templates |
| Crossplane | 🔜 Phase 3 | Kubernetes-native IaC |

### 3. Real-Time Collaboration

- **Multi-user Editing**: See changes as your team works
- **Git-Native Workflow**: Architecture Pull Requests (APRs) with visual diffs
- **Context-Aware Comments**: Discuss components directly on the diagram
- **Approval Workflows**: Visual sign-off with full audit trail

### 4. Simulation & What-If Analysis

Before deploying, validate your architecture:

- **Cost Simulation**: Real-time cost estimation with Infracost integration
- **Performance Analysis**: Latency modeling, throughput planning, bottleneck identification
- **Failure Simulation**: Chaos engineering scenarios, dependency impact analysis
- **Security Scanning**: Attack path analysis, compliance gap detection

### 5. AI-Powered Design Assistant

Generate, optimize, and document your architecture with AI:

- **Architecture Generation**: "Create a serverless API with auth and database"
- **Optimization Suggestions**: "Reduce cost by 40% using spot instances"
- **Pattern Recognition**: Identify anti-patterns and recommend improvements
- **Documentation**: Auto-generate Architecture Decision Records (ADRs)

---

## 🛠️ Technical Architecture

### The Rust + WebAssembly Foundation

ArchFlow is built on a high-performance Rust engine that compiles to WebAssembly:

```
┌────────────────────────────────────────────────────────────┐
│                    Frontend (Browser)                       │
├────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Rust/WASM Core Engine                         │  │
│  │  ┌─────────┐ ┌──────────┐ ┌─────────────────────┐   │  │
│  │  │Graphics │ │Geometry  │ │Collaboration Engine │   │  │
│  │  │(WebGPU) │ │Engine    │ │                     │   │  │
│  │  └─────────┘ └──────────┘ └─────────────────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
│           ↑                    ↑                           │
│     archflow-geometry     archflow-primitives              │
│           ↑                                              │
│     archflow-core                                        │
└────────────────────────────────────────────────────────────┘
              ↓ (optional)
┌────────────────────────────────────────────────────────────┐
│                  Backend Services                           │
│  Sync Service │ AI Engine │ Agent Service │ Storage        │
└────────────────────────────────────────────────────────────┘
```

### Current Implementation (Foundation Layer)

Our current implementation provides the **core foundation** for the full platform:

| Crate | Purpose | Status |
|-------|---------|--------|
| `archflow-core` | Core types (Vec2, Mat3, Rect, Color, EntityId) | ✅ Complete |
| `archflow-geometry` | Geometry engine with kurbo, Bézier curves, intersection detection | ✅ Complete |
| `archflow-primitives` | Shapes, styles, ports & connections | ✅ Complete |
| `archflow-renderer` | Abstract renderer traits | ✅ Complete |
| `archflow-renderer-canvas` | Canvas 2D backend | ✅ Complete |
| `archflow-ecs` | Entity Component System | ✅ Complete |
| `archflow-workspace` | Document & workspace management | ✅ Complete |
| `archflow-wasm` | WebAssembly bindings | ✅ Complete |

### Performance Targets

| Metric | Target | Implementation |
|--------|--------|----------------|
| Load 10k nodes | <2s | Rust/WASM |
| 60fps pan/zoom | 1k animated elements | WebGPU |
| Collaboration latency | <100ms | CRDT-based sync |

---

## 📦 Getting Started

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
```

### Quick Example

```rust
use archflow_core::{Vec2, EntityId};
use archflow_primitives::{Rectangle, FillStyle, StrokeStyle, Port};
use archflow_geometry::GeometryEngine;

// Create a cloud component with ports
let component = Rectangle::new(
    Vec2::new(0.0, 0.0),
    Vec2::new(200.0, 150.0),
).with_fill(FillStyle::solid("#FF5733"))
 .with_stroke(StrokeStyle::new("#333333", 2.0))
 .with_port(Port::output("api", Vec2::new(200.0, 75.0)))
 .with_port(Port::input("data", Vec2::new(0.0, 75.0)));

// Calculate geometric properties
let engine = GeometryEngine::default();
let bounds = component.global_bounds();
let center = engine.rect_center(bounds);

// Export to Infrastructure as Code
let terraform = component.to_terraform();
let kubernetes = component.to_kubernetes();
```

---

## 🗺️ Roadmap

```
Phase 1: Foundation (Months 1-6) - YOU ARE HERE
├── ✅ Core Engine (Geometry, Rendering, Primitives)
├── 🔄 Component System Design
└── ⏳ AUF (Architecture Universal Format) Specification

Phase 2: MVP (Months 7-12)
├── Visual Editor with drag-drop
├── Terraform & Kubernetes Export
├── Basic Cost Simulation
└── Component Library (AWS, Azure, GCP)

Phase 3: Collaboration (Months 13-18)
├── Real-time multi-user editing
├── Git integration (commit/pull/push)
├── Architecture Pull Requests (APRs)
└── Comments and review workflows

Phase 4: Intelligence (Months 19-24)
├── AI-assisted design generation
├── Advanced simulations (performance, security)
├── Optimization recommendations
└── Plugin SDK

Phase 5: Platform (Months 25-30)
├── Component Marketplace
├── Enterprise features (SSO, audit, on-prem)
├── Partner integrations
└── Community ecosystem
```

---

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [PRD](docs/prd.md) | Product Requirements Document - Full vision and specification |
| [Architecture Design](docs/ARCHITECTURE-DESIGN.md) | Technical architecture decisions |
| [EPICS-ENGINE-2D](docs/EPICS-ENGINE-2D.md) | Implementation roadmap and user stories |
| [API Documentation](docs/) | Generated Rust documentation |

---

## 🤝 Contributing

ArchFlow is in its early stages, and we need contributors to build the foundation!

### How to Contribute

1. **Explore the Crates**: Start with `archflow-core` to understand the type system
2. **Pick a Feature**: Check the [EPICS](docs/EPICS-ENGINE-2D.md) for available work
3. **Follow TDD**: Write tests first, then implement
4. **Submit PR**: Open a pull request with clear description

### Current Priorities

- Component system design and implementation
- Architecture Universal Format (AUF) specification
- Rendering optimizations (WebGPU support)
- Cloud provider component libraries
- CI/CD and testing infrastructure

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [kurbo](https://github.com/linebender/kurbo) - 2D geometry library
- [glam](https://github.com/bitshifter/glam-rs) - SIMD math library
- [tldraw](https://github.com/tldraw/tldraw) - Inspiration for canvas interaction
- [Terraform](https://www.terraform.io/) - IaC format inspiration
- [C4 Model](https://c4model.com/) - Architecture visualization methodology

---

<p align="center">
  <strong>Architecture as a Living System</strong><br>
  Where diagrams become deployable infrastructure.
</p>
