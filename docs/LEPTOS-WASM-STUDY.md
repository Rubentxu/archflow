# Estudio de Arquitectura: Enfoque Leptos/WASM (Figma-like)

**Versión:** 1.0  
**Fecha:** 2026-01-23  
**Estado:** Análisis Revisado  
**Referencia:** `docs/ARCHITECTURE-STUDY.md`, `docs/PRD-CRITICA.md`  
**Enfoque:** Leptos + WebAssembly para Canvas de Alto Rendimiento

---

## 1. Figma Architecture Analysis

### 1.1 How Figma Works

Figma achieves 60fps with complex diagrams using:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          FIGMA ARCHITECTURE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Browser (Chrome/Firefox/Safari)                                            │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  UI Layer (JavaScript/TypeScript)                                   │    │
│  │  - React for UI components (toolbars, panels)                      │    │
│  │  - Collaboration features                                           │    │
│  │  - User interactions (clicks, typing)                              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Core Engine (C++ → WebAssembly)                                    │    │
│  │  - Real-time rendering (WebGL)                                     │    │
│  │  - Vector graphics processing                                      │    │
│  │  - Layout engine                                                    │    │
│  │  - Collision detection                                              │    │
│  │  - Memory management                                                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  WebGL Renderer                                                      │    │
│  │  - GPU-accelerated rendering                                       │    │
│  │  - 10,000+ elements at 60fps                                       │    │
│  │  -抗锯齿 (Anti-aliasing)                                           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  Backend (Distributed Systems)                                               │
│  - WebSocket servers for real-time sync                                     │
│  - CRDT (Conflict-free Replicated Data Types)                              │
│  - Operational Transformation                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Key Insight

**Figma runs C++ in the browser via WebAssembly for performance-critical code.**

Leptos + Rust → WebAssembly es el equivalente moderno y type-safe.

---

## 2. Enfoque Corregido: Leptos/WASM-First

### 2.1 Arquitectura Propuesta

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ARCHFLOW ARCHITECTURE (LEPTOS/WASM)                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                   Web Browser (WASM Target)                          │    │
│  │                                                                       │    │
│  │  ┌─────────────────────────────────────────────────────────────┐    │    │
│  │  │  Leptos Components (Rust compiled to WASM)                  │    │    │
│  │  │  - Canvas rendering                                          │    │    │
│  │  │  - Component palette                                         │    │    │
│  │  │  - Properties panel                                          │    │    │
│  │  │  - State management (Signals)                                │    │    │
│  │  │  - Event handling                                            │    │    │
│  │  └─────────────────────────────────────────────────────────────┘    │    │
│  │                                                                       │    │
│  │  ┌─────────────────────────────────────────────────────────────┐    │    │
│  │  │  Canvas Renderer (WebGPU/WebGL via Rust)                    │    │    │
│  │  │  - wgpu or glow                                             │    │    │
│  │  │  - 10,000+ nodes at 60fps                                   │    │    │
│  │  │  - GPU-accelerated drawing                                  │    │    │
│  │  │  - Smooth pan/zoom                                          │    │    │
│  │  └─────────────────────────────────────────────────────────────┘    │    │
│  │                                                                       │    │
│  │  ┌─────────────────────────────────────────────────────────────┐    │    │
│  │  │  Local Storage (IndexedDB via Rust)                         │    │    │
│  │  │  - AUF file storage                                         │    │    │
│  │  │  - Component cache                                          │    │    │
│  │  │  - Undo/redo history                                        │    │    │
│  │  └─────────────────────────────────────────────────────────────┘    │    │
│  │                                                                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Backend (Serverless - Optional for MVP)                            │    │
│  │                                                                       │    │
│  │  ┌───────────────────┐  ┌───────────────────┐                      │    │
│  │  │  Export Service   │  │  Storage Service  │                      │    │
│  │  │  (Terraform HCL)  │  │  (AUF files)      │                      │    │
│  │  └───────────────────┘  └───────────────────┘                      │    │
│  │                                                                       │    │
│  │  Technology: Rust binaries compiled to WASM + Vercel Functions      │    │
│  │                                                                       │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Stack Tecnológico

| Capa | Tecnología | Justificación |
|------|------------|---------------|
| Framework UI | Leptos | Rust → WASM, fine-grained reactivity, rendimiento óptimo |
| Canvas Renderer | wgpu + glow | GPU-accelerated, WebGPU compatible |
| State Management | Leptos Signals | Type-safe, zero-cost abstractions |
| Storage | IndexedDB via Rust (rust-indexeddb) | Persistencia local robusta |
| Build | trunk / wasm-pack | Standard WASM toolchain |
| Backend | Rust WASM + Vercel Serverless | Mismo lenguaje, máximo rendimiento |

### 2.3 Por qué Leptos/WASM

**Rendimiento comparable a código nativo:**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PERFORMANCE COMPARISON                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Language      │ Execution Model    │ Memory    │ Startup  │ Type Safety   │
│  ─────────────────────────────────────────────────────────────────────────  │
│  JavaScript    │ JIT compilation    │ GC        │ ~100ms   │ Low           │
│  TypeScript    │ Transpiled to JS   │ GC        │ ~100ms   │ Medium        │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Rust/WASM     │ AOT compilation    │ Manual    │ ~50ms    │ High          │
│  ─────────────────────────────────────────────────────────────────────────  │
│  C++/WASM      │ AOT compilation    │ Manual    │ ~30ms    │ Medium        │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Conclusion: Rust/WASM ofrece el mejor balance entre rendimiento y safety  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Arquitectura Detallada Leptos

### 3.1 Estructura del Proyecto

```
archflow/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
│
├── packages/
│   ├── core/                     # Core domain (shared between WASM and CLI)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── architecture/     # Architecture aggregate
│   │   │   │   ├── mod.rs
│   │   │   │   ├── architecture.rs
│   │   │   │   └── state.rs
│   │   │   ├── component/        # Component aggregate
│   │   │   │   ├── mod.rs
│   │   │   │   ├── component.rs
│   │   │   │   └── registry.rs
│   │   │   ├── value_objects/    # Value objects
│   │   │   │   ├── mod.rs
│   │   │   │   ├── position.rs
│   │   │   │   └── version.rs
│   │   │   ├── events/           # Domain events
│   │   │   │   ├── mod.rs
│   │   │   │   └── architecture_events.rs
│   │   │   ├── errors/           # Error types
│   │   │   │   └── mod.rs
│   │   │   └── auformat/         # AUF format
│   │   │       ├── mod.rs
│   │   │       ├── schema.rs
│   │   │       └── serializer.rs
│   │   └── tests/
│   │
│   ├── canvas/                   # Canvas rendering engine
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── renderer/         # WebGPU/WebGL renderer
│   │   │   │   ├── mod.rs
│   │   │   │   ├── wgpu_renderer.rs
│   │   │   │   └── shapes.rs
│   │   │   ├── layout/           # Auto-layout engine
│   │   │   │   ├── mod.rs
│   │   │   │   └── auto_layout.rs
│   │   │   ├── input/            # Mouse/keyboard handling
│   │   │   │   ├── mod.rs
│   │   │   │   └── event_handler.rs
│   │   │   └── selection/        # Selection logic
│   │   │       ├── mod.rs
│   │   │       └── selection_manager.rs
│   │   └── tests/
│   │
│   ├── storage/                  # Persistence layer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── indexeddb/        # IndexedDB adapter
│   │   │   │   ├── mod.rs
│   │   │   │   └── repository.rs
│   │   │   ├── file/             # AUF file I/O
│   │   │   │   ├── mod.rs
│   │   │   │   └── aufile_io.rs
│   │   │   └── repository/       # Repository pattern
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   ├── export/                   # IaC exporters
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── terraform/        # Terraform HCL generator
│   │   │   │   ├── mod.rs
│   │   │   │   └── hcl_generator.rs
│   │   │   └── kubernetes/       # Kubernetes YAML (Phase 2)
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   └── app/                      # Leptos WASM application
│       ├── Cargo.toml
│       ├── index.html
│       ├── src/
│       │   ├── lib.rs
│       │   ├── main.rs
│       │   ├── app.rs            # Root component
│       │   ├── components/       # Leptos components
│       │   │   ├── mod.rs
│       │   │   ├── canvas.rs     # Canvas wrapper
│       │   │   ├── palette.rs    # Component palette
│       │   │   ├── properties.rs # Properties panel
│       │   │   ├── toolbar.rs    # Top toolbar
│       │   │   └── layers.rs     # Layer panel
│       │   ├── pages/            # Pages
│       │   │   ├── mod.rs
│       │   │   ├── home.rs
│       │   │   └── editor.rs
│       │   ├── state/            # Global state (signals)
│       │   │   ├── mod.rs
│       │   │   └── store.rs
│       │   ├── services/         # Application services
│       │   │   ├── mod.rs
│       │   │   └── export_service.rs
│       │   └── styles/           # CSS
│       │       └── main.css
│       ├── assets/
│       │   └── fonts/
│       └── tests/
│
├── tools/
│   ├── cli/                      # CLI tool (optional)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── build.sh
│   │
│   └── wasm-bindgen/             # WASM bindings helper
│
├── api/                          # Serverless functions (Rust → WASM)
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
│
├── Cargo.toml                    # Workspace configuration
│
├── .github/workflows/
│   └── ci.yml
│
├── README.md
├── LICENSE
└── .gitignore
```

### 3.2 Dependencias del Workspace

```toml
# Cargo.toml (workspace root)

[workspace]
members = [
    "packages/core",
    "packages/canvas",
    "packages/storage",
    "packages/export",
    "packages/app",
    "tools/cli",
    "api",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["ArchFlow Team"]

[workspace.dependencies]
# Core
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
thiserror = "2.0"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "1.0"
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Leptos
leptos = { version = "0.6", features = ["wasm-bind"] }
leptos_meta = "0.6"
leptos_router = "0.6"

# WASM
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Document",
    "Element",
    "HtmlElement",
    "HtmlCanvasElement",
    "WebGl2RenderingContext",
    "WebGlRenderingContext",
    "Performance",
    "Storage",
    "EventTarget",
    "KeyboardEvent",
    "MouseEvent",
    "WheelEvent",
    "TouchEvent",
] }
js-sys = "0.3"

# Graphics
wgpu = { version = "0.18", optional = true }
glow = { version = "0.13", optional = true }

# Storage
indexeddb = { version = "0.5", optional = true }
gloo-storage = { version = "0.2", optional = true }

# State management
store = { version = "0.4", optional = true }

# Tracing
tracing = "0.1"
tracing-wasm = "0.2"

# Testing
rstest = "0.18"
proptest = "1.4"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"

[patch.crates-io]
# Patch dependencies if needed
```

### 3.3 App Package (Leptos/WASM)

```toml
# packages/app/Cargo.toml

[package]
name = "archflow-app"
version.workspace = true
edition.workspace = true
publish = false

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = ["hydrate"]
ssr = ["leptos_use"]

[dependencies]
# Core domain
archflow-core = { path = "../core" }
archflow-canvas = { path = "../canvas" }
archflow-storage = { path = "../storage" }
archflow-export = { path = "../export" }

# Leptos
leptos.workspace = true
leptos_meta.workspace = true
leptos_router.workspace = true

# WASM
wasm-bindgen.workspace = true
wasm-bindgen-futures.workspace = true
js-sys.workspace = true

# Graphics
wgpu = { workspace = true, optional = true }
glow = { workspace = true, optional = true }

# Graphics features
canvas = ["dep:wgpu", "dep:glow"]
default = ["canvas"]

# State
gloo-storage = { workspace = true, optional = true }
store = { workspace = true, optional = true }
default = ["storage"]

[dev-dependencies]
wasm-bindgen-test = "0.3"

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O4"]
```

---

## 4. Canvas Rendering Engine (Leptos + WebGPU)

### 4.1 Canvas State

```rust
// packages/canvas/src/lib.rs

use leptos::*;
use std::collections::{HashMap, HashSet};
use archflow_core::architecture::*;
use archflow_core::component::*;

pub struct CanvasState {
    /// All components on the canvas
    pub components: HashMap<ComponentId, ComponentRenderData>,
    
    /// All relationships (connections)
    pub relationships: Vec<RelationshipRenderData>,
    
    /// Selected component IDs
    pub selected: HashSet<ComponentId>,
    
    /// Current view transform (pan/zoom)
    pub transform: ViewTransform,
    
    /// Hovered component
    pub hovered: Option<ComponentId>,
    
    /// Drag state
    pub drag: Option<DragState>,
    
    /// Selection box (for multi-select)
    pub selection_box: Option<SelectionBox>,
}

#[derive(Clone, Debug)]
pub struct ComponentRenderData {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub position: Position,
    pub size: Size,
    pub label: String,
    pub icon_type: IconType,
    pub color: Color,
    pub z_index: u32,
}

#[derive(Clone, Debug)]
pub struct RelationshipRenderData {
    pub id: uuid::Uuid,
    pub from: ComponentId,
    pub to: ComponentId,
    pub relationship_type: RelationshipType,
    pub path: Vec<Position>,  // Bézier control points
    pub color: Color,
    pub animated: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ViewTransform {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

#[derive(Clone, Debug)]
pub struct DragState {
    pub component_id: ComponentId,
    pub start_position: Position,
    pub current_offset: Position,
}

#[derive(Clone, Debug, Default)]
pub struct SelectionBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum IconType {
    Server,
    Database,
    Cloud,
    Network,
    Shield,
    Bucket,
    Function,
    LoadBalancer,
    Custom(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            components: HashMap::new(),
            relationships: Vec::new(),
            selected: HashSet::new(),
            transform: ViewTransform { x: 0.0, y: 0.0, scale: 1.0 },
            hovered: None,
            drag: None,
            selection_box: None,
        }
    }
}

impl CanvasState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> Position {
        Position::new(
            (screen_x - self.transform.x) / self.transform.scale,
            (screen_y - self.transform.y) / self.transform.scale,
        )
    }
    
    pub fn world_to_screen(&self, world_x: f64, world_y: f64) -> (f64, f64) {
        (
            world_x * self.transform.scale + self.transform.x,
            world_y * self.transform.scale + self.transform.y,
        )
    }
    
    pub fn hit_test(&self, x: f64, y: f64) -> Option<ComponentId> {
        let world_pos = self.screen_to_world(x, y);
        
        // Check in reverse z-order (top to bottom)
        for (_, component) in self.components.iter().rev() {
            let half_width = component.size.width / 2.0;
            let half_height = component.size.height / 2.0;
            
            if world_pos.x >= component.position.x - half_width
                && world_pos.x <= component.position.x + half_width
                && world_pos.y >= component.position.y - half_height
                && world_pos.y <= component.position.y + half_height
            {
                return Some(component.id);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}
```

### 4.2 Leptos Canvas Component

```rust
// packages/app/src/components/canvas.rs

use leptos::*;
use archflow_canvas::*;
use crate::state::store::use_store;

#[component]
pub fn CanvasComponent(
    #[prop(default = 800.0)] width: f64,
    #[prop(default = 600.0)] height: f64,
) -> impl IntoView {
    let store = use_store();
    let canvas_ref = create_node_ref::<html::Canvas>();
    
    // Initialize renderer when canvas is mounted
    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            // Initialize WebGPU/WebGL renderer
            let renderer = CanvasRenderer::new(canvas, width, height);
            store.set_renderer(renderer);
        }
    });
    
    // Handle mouse events
    let on_mousedown = move |event: web_sys::MouseEvent| {
        let rect = canvas_ref.get()
            .expect("Canvas ref should exist")
            .get_bounding_client_rect();
        
        let x = event.client_x() as f64 - rect.left();
        let y = event.client_y() as f64 - rect.top();
        
        if event.shift_key() {
            // Multi-select with selection box
            store.start_selection(x, y);
        } else {
            // Single select or drag
            if let Some(component_id) = store.canvas_state.hit_test(x, y) {
                store.select_component(Some(component_id));
                
                if event.button() == 0 {
                    // Start dragging
                    let pos = store.canvas_state.screen_to_world(x, y);
                    store.start_drag(component_id, pos);
                }
            } else {
                // Deselect
                store.select_component(None);
            }
        }
    };
    
    let on_mousemove = move |event: web_sys::MouseEvent| {
        let rect = canvas_ref.get()
            .expect("Canvas ref should exist")
            .get_bounding_client_rect();
        
        let x = event.client_x() as f64 - rect.left();
        let y = event.client_y() as f64 - rect.top();
        
        if let Some(drag_state) = &store.canvas_state.drag {
            let pos = store.canvas_state.screen_to_world(x, y);
            store.update_drag(drag_state.component_id, pos);
        } else if let Some(selection) = &store.canvas_state.selection_box {
            store.update_selection(x, y);
        } else {
            // Update hover state
            let component_id = store.canvas_state.hit_test(x, y);
            store.set_hovered(component_id);
        }
    };
    
    let on_mouseup = move |event: web_sys::MouseEvent| {
        if store.canvas_state.drag.is_some() {
            store.end_drag();
        }
        if store.canvas_state.selection_box.is_some() {
            store.end_selection();
        }
    };
    
    let on_wheel = move |event: web_sys::WheelEvent| {
        event.prevent_default();
        
        let rect = canvas_ref.get()
            .expect("Canvas ref should exist")
            .get_bounding_client_rect();
        
        let mouse_x = event.client_x() as f64 - rect.left();
        let mouse_y = event.client_y() as f64 - rect.top();
        
        let delta = if event.delta_mode() == 0 {
            event.delta_y() * 0.001
        } else {
            event.delta_y() * -0.1
        };
        
        store.zoom_at(mouse_x, mouse_y, delta);
    };
    
    // Render using store's canvas state
    create_effect(move |_| {
        if let Some(renderer) = &store.renderer {
            renderer.render(&store.canvas_state);
        }
    });
    
    view! {
        <canvas
            ref=canvas_ref
            width={width as u32}
            height={height as u32}
            class="canvas"
            on:mousedown={on_mousedown}
            on:mousemove={on_mousemove}
            on:mouseup={on_mouseup}
            on:wheel={on_wheel}
            on:contextmenu=|e| e.prevent_default()
        />
    }
}
```

### 4.3 Global Store with Signals

```rust
// packages/app/src/state/store.rs

use leptos::*;
use std::collections::{HashMap, HashSet};
use archflow_core::architecture::*;
use archflow_core::component::*;
use archflow_canvas::*;
use archflow_storage::repository::ArchitectureRepository;

#[derive(Clone)]
pub struct AppStore {
    // Architecture state
    pub architecture_id: RwSignal<Option<String>>,
    pub architecture_name: RwSignal<String>,
    pub architecture: RwSignal<Option<Architecture>>,
    
    // Canvas state
    pub canvas_state: RwSignal<CanvasState>,
    pub renderer: RwSignal<Option<CanvasRenderer>>,
    
    // Selection
    pub selected_component_id: RwSignal<Option<ComponentId>>,
    
    // UI state
    pub is_loading: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub notification: RwSignal<Option<Notification>>,
    
    // History for undo/redo
    pub history: RwSignal<Vec<ArchitectureSnapshot>>,
    pub history_index: RwSignal<usize>,
    
    // Repository
    pub repository: Arc<dyn ArchitectureRepository>,
}

#[derive(Clone, Debug)]
pub struct ArchitectureSnapshot {
    pub components: HashMap<ComponentId, Component>,
    pub relationships: Vec<Relationship>,
    pub version: Version,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl AppStore {
    pub fn new(repository: Arc<dyn ArchitectureRepository>) -> Self {
        Self {
            architecture_id: RwSignal::new(None),
            architecture_name: RwSignal::new("Untitled Architecture".to_string()),
            architecture: RwSignal::new(None),
            canvas_state: RwSignal::new(CanvasState::new()),
            renderer: RwSignal::new(None),
            selected_component_id: RwSignal::new(None),
            is_loading: RwSignal::new(false),
            error_message: RwSignal::new(None),
            notification: RwSignal::new(None),
            history: RwSignal::new(Vec::new()),
            history_index: RwSignal::new(0),
            repository,
        }
    }
    
    pub fn create_architecture(&mut self, name: String, description: String) {
        let mut arch = Architecture::new(name, description).unwrap();
        self.architecture_id.set(Some(arch.id().to_string()));
        self.architecture_name.set(arch.name().to_string());
        self.architecture.set(Some(arch.clone()));
        self.canvas_state.update(|state| {
            state.components.clear();
        });
        self.save_to_history();
    }
    
    pub fn select_component(&mut self, id: Option<ComponentId>) {
        self.selected_component_id.set(id);
        
        self.canvas_state.update(|state| {
            state.selected.clear();
            if let Some(id) = id {
                state.selected.insert(id);
            }
        });
    }
    
    pub fn add_component(&mut self, component: Component) {
        self.canvas_state.update(|state| {
            state.components.insert(
                *component.id(),
                ComponentRenderData::from_component(&component),
            );
        });
        
        self.architecture.update(|arch| {
            if let Some(arch) = arch {
                arch.add_component(component).ok();
            }
        });
        
        self.save_to_history();
    }
    
    pub fn move_component(&mut self, id: ComponentId, new_position: Position) {
        self.canvas_state.update(|state| {
            if let Some(component) = state.components.get_mut(&id) {
                component.position = new_position;
            }
        });
        
        self.architecture.update(|arch| {
            if let Some(arch) = arch {
                arch.update_component(&id, HashMap::new()).ok();
            }
        });
    }
    
    pub fn start_drag(&mut self, component_id: ComponentId, position: Position) {
        self.canvas_state.update(|state| {
            state.drag = Some(DragState {
                component_id,
                start_position: position,
                current_offset: Position::new(0.0, 0.0),
            });
        });
    }
    
    pub fn update_drag(&mut self, component_id: ComponentId, position: Position) {
        if let Some(drag) = &self.canvas_state.drag {
            if drag.component_id == component_id {
                self.canvas_state.update(|state| {
                    if let Some(drag) = &mut state.drag {
                        drag.current_offset = Position::new(
                            position.x - drag.start_position.x,
                            position.y - drag.start_position.y,
                        );
                        
                        if let Some(component) = state.components.get_mut(&component_id) {
                            component.position = Position::new(
                                component.position.x + drag.current_offset.x,
                                component.position.y + drag.current_offset.y,
                            );
                        }
                    }
                });
            }
        }
    }
    
    pub fn end_drag(&mut self) {
        self.canvas_state.update(|state| {
            state.drag = None;
        });
        self.save_to_history();
    }
    
    pub fn zoom_at(&mut self, mouse_x: f64, mouse_y: f64, delta: f64) {
        self.canvas_state.update(|state| {
            let zoom_factor = 1.0 + delta;
            let new_scale = (state.transform.scale * zoom_factor).clamp(0.1, 5.0);
            
            // Zoom towards mouse position
            let world_before = state.screen_to_world(mouse_x, mouse_y);
            state.transform.scale = new_scale;
            let world_after = state.screen_to_world(mouse_x, mouse_y);
            
            // Adjust pan to maintain mouse position
            state.transform.x += (world_after.x - world_before.x) * new_scale;
            state.transform.y += (world_after.y - world_before.y) * new_scale;
        });
    }
    
    pub fn set_renderer(&mut self, renderer: CanvasRenderer) {
        self.renderer.set(Some(renderer));
    }
    
    fn save_to_history(&mut self) {
        if let Some(arch) = &self.architecture.get() {
            let snapshot = ArchitectureSnapshot {
                components: arch.components().map(|c| (*c.id(), c.clone())).collect(),
                relationships: arch.relationships().to_vec(),
                version: arch.version().clone(),
            };
            
            self.history.update(|history| {
                // Remove any redo states
                history.truncate(self.history_index.get() + 1);
                history.push(snapshot);
                
                // Limit history size
                if history.len() > 50 {
                    history.remove(0);
                }
            });
            
            self.history_index.update(|idx| {
                *idx = std::cmp::min(*idx + 1, history.len() - 1);
            });
        }
    }
}

impl ComponentRenderData {
    pub fn from_component(component: &Component) -> Self {
        Self {
            id: *component.id(),
            component_type: component.component_type().clone(),
            position: *component.position(),
            size: Size::new(100.0, 60.0), // Default size
            label: component.name().to_string(),
            icon_type: Self::component_type_to_icon(component.component_type()),
            color: Self::component_type_to_color(component.component_type()),
            z_index: component.position().z as u32,
        }
    }
    
    fn component_type_to_icon(component_type: &ComponentType) -> IconType {
        match component_type {
            ComponentType::Ec2Instance => IconType::Server,
            ComponentType::LambdaFunction => IconType::Function,
            ComponentType::S3Bucket => IconType::Bucket,
            ComponentType::RdsInstance => IconType::Database,
            ComponentType::Vpc | ComponentType::LoadBalancer => IconType::Network,
            ComponentType::IamRole => IconType::Shield,
            _ => IconType::Cloud,
        }
    }
    
    fn component_type_to_color(component_type: &ComponentType) -> Color {
        match component_type.category() {
            ComponentCategory::Compute => Color { r: 66, g: 133, b: 244, a: 255 }, // Blue
            ComponentCategory::Storage => Color { r: 52, g: 168, b: 83, a: 255 },  // Green
            ComponentCategory::Network => Color { r: 251, g: 188, b: 5, a: 255 },  // Yellow
            ComponentCategory::Security => Color { r: 234, g: 67, b: 53, a: 255 }, // Red
            ComponentCategory::Custom => Color { r: 156, g: 39, b: 176, a: 255 }, // Purple
        }
    }
}
```

---

## 5. Component Registry (Leptos State)

```rust
// packages/core/src/component/registry.rs

use crate::component::*;
use serde::{Serialize, Deserialize};

pub struct ComponentRegistry {
    definitions: HashMap<ComponentType, ComponentDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentDefinition {
    pub type_: ComponentType,
    pub name: String,
    pub category: ComponentCategory,
    pub cloud_provider: Option<CloudProvider>,
    pub icon: &'static str,
    pub default_size: Size,
    pub properties: Vec<PropertyDefinition>,
    pub default_properties: HashMap<String, PropertyValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub key: String,
    pub name: String,
    pub type_: PropertyInputType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub options: Option<Vec<PropertyOption>>,
    pub validation: Option<PropertyValidation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyOption {
    pub label: String,
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyValidation {
    pub pattern: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropertyInputType {
    Text,
    Number,
    Boolean,
    Select,
    Multiline,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            definitions: HashMap::new(),
        };
        
        registry.register_aws_components();
        registry
    }
    
    fn register_aws_components(&mut self) {
        self.definitions.insert(
            ComponentType::Ec2Instance,
            ComponentDefinition {
                type_: ComponentType::Ec2Instance,
                name: "EC2 Instance".to_string(),
                category: ComponentCategory::Compute,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "server",
                default_size: Size::new(120.0, 60.0),
                properties: vec![
                    PropertyDefinition {
                        key: "instance_type".to_string(),
                        name: "Instance Type".to_string(),
                        type_: PropertyInputType::Select,
                        required: true,
                        default: Some(serde_json::json!("t3.micro")),
                        options: Some(vec![
                            PropertyOption { label: "t3.micro".to_string(), value: serde_json::json!("t3.micro") },
                            PropertyOption { label: "t3.small".to_string(), value: serde_json::json!("t3.small") },
                            PropertyOption { label: "t3.medium".to_string(), value: serde_json::json!("t3.medium") },
                            PropertyOption { label: "t3.large".to_string(), value: serde_json::json!("t3.large") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "ami".to_string(),
                        name: "AMI ID".to_string(),
                        type_: PropertyInputType::Text,
                        required: true,
                        default: Some(serde_json::json!("ami-0c55b159cbfafe1f0")),
                        options: None,
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "key_name".to_string(),
                        name: "Key Pair".to_string(),
                        type_: PropertyInputType::Text,
                        required: false,
                        default: None,
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        self.definitions.insert(
            ComponentType::LambdaFunction,
            ComponentDefinition {
                type_: ComponentType::LambdaFunction,
                name: "Lambda Function".to_string(),
                category: ComponentCategory::Compute,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "lambda",
                default_size: Size::new(100.0, 100.0),
                properties: vec![
                    PropertyDefinition {
                        key: "runtime".to_string(),
                        name: "Runtime".to_string(),
                        type_: PropertyInputType::Select,
                        required: true,
                        default: Some(serde_json::json!("python3.11")),
                        options: Some(vec![
                            PropertyOption { label: "Python 3.11".to_string(), value: serde_json::json!("python3.11") },
                            PropertyOption { label: "Node.js 20".to_string(), value: serde_json::json!("nodejs20.x") },
                            PropertyOption { label: "Java 17".to_string(), value: serde_json::json!("java17") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "handler".to_string(),
                        name: "Handler".to_string(),
                        type_: PropertyInputType::Text,
                        required: true,
                        default: Some(serde_json::json!("index.handler")),
                        options: None,
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "timeout".to_string(),
                        name: "Timeout (seconds)".to_string(),
                        type_: PropertyInputType::Number,
                        required: false,
                        default: Some(serde_json::json!(30)),
                        options: None,
                        validation: Some(PropertyValidation {
                            min: Some(1.0),
                            max: Some(900.0),
                            pattern: None,
                        }),
                    },
                    PropertyDefinition {
                        key: "memory_size".to_string(),
                        name: "Memory (MB)".to_string(),
                        type_: PropertyInputType::Number,
                        required: false,
                        default: Some(serde_json::json!(256)),
                        options: None,
                        validation: Some(PropertyValidation {
                            min: Some(128.0),
                            max: Some(10240.0),
                            pattern: None,
                        }),
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // Add more components...
        self.definitions.insert(
            ComponentType::S3Bucket,
            ComponentDefinition {
                type_: ComponentType::S3Bucket,
                name: "S3 Bucket".to_string(),
                category: ComponentCategory::Storage,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "bucket",
                default_size: Size::new(100.0, 80.0),
                properties: vec![
                    PropertyDefinition {
                        key: "bucket_name".to_string(),
                        name: "Bucket Name".to_string(),
                        type_: PropertyInputType::Text,
                        required: true,
                        default: None,
                        options: None,
                        validation: Some(PropertyValidation {
                            pattern: Some("^[a-z0-9-]{3,63}$".to_string()),
                            ..Default::default()
                        }),
                    },
                    PropertyDefinition {
                        key: "versioning".to_string(),
                        name: "Versioning".to_string(),
                        type_: PropertyInputType::Boolean,
                        required: false,
                        default: Some(serde_json::json!(false)),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        self.definitions.insert(
            ComponentType::Vpc,
            ComponentDefinition {
                type_: ComponentType::Vpc,
                name: "VPC".to_string(),
                category: ComponentCategory::Network,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "network",
                default_size: Size::new(150.0, 100.0),
                properties: vec![
                    PropertyDefinition {
                        key: "cidr_block".to_string(),
                        name: "CIDR Block".to_string(),
                        type_: PropertyInputType::Text,
                        required: true,
                        default: Some(serde_json::json!("10.0.0.0/16")),
                        options: None,
                        validation: Some(PropertyValidation {
                            pattern: Some(r"^([0-9]{1,3}\.){3}[0-9]{1,3}/[0-9]{1,2}$".to_string()),
                            ..Default::default()
                        }),
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        self.definitions.insert(
            ComponentType::RdsInstance,
            ComponentDefinition {
                type_: ComponentType::RdsInstance,
                name: "RDS Instance".to_string(),
                category: ComponentCategory::Storage,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "database",
                default_size: Size::new(120.0, 60.0),
                properties: vec![
                    PropertyDefinition {
                        key: "engine".to_string(),
                        name: "Engine".to_string(),
                        type_: PropertyInputType::Select,
                        required: true,
                        default: Some(serde_json::json!("postgres")),
                        options: Some(vec![
                            PropertyOption { label: "PostgreSQL".to_string(), value: serde_json::json!("postgres") },
                            PropertyOption { label: "MySQL".to_string(), value: serde_json::json!("mysql") },
                            PropertyOption { label: "Aurora PostgreSQL".to_string(), value: serde_json::json!("aurora-postgresql") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "instance_class".to_string(),
                        name: "Instance Class".to_string(),
                        type_: PropertyInputType::Select,
                        required: true,
                        default: Some(serde_json::json!("db.t3.micro")),
                        options: Some(vec![
                            PropertyOption { label: "db.t3.micro".to_string(), value: serde_json::json!("db.t3.micro") },
                            PropertyOption { label: "db.t3.small".to_string(), value: serde_json::json!("db.t3.small") },
                            PropertyOption { label: "db.t3.medium".to_string(), value: serde_json::json!("db.t3.medium") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "multi_az".to_string(),
                        name: "Multi-AZ".to_string(),
                        type_: PropertyInputType::Boolean,
                        required: false,
                        default: Some(serde_json::json!(false)),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
    }
    
    pub fn get(&self, component_type: &ComponentType) -> Option<&ComponentDefinition> {
        self.definitions.get(component_type)
    }
    
    pub fn get_all(&self) -> Vec<&ComponentDefinition> {
        self.definitions.values().collect()
    }
    
    pub fn get_by_category(&self, category: ComponentCategory) -> Vec<&ComponentDefinition> {
        self.definitions.values()
            .filter(|def| def.category == category)
            .collect()
    }
    
    pub fn search(&self, query: &str) -> Vec<&ComponentDefinition> {
        let lower_query = query.to_lowercase();
        self.definitions.values()
            .filter(|def| {
                def.name.to_lowercase().contains(&lower_query)
                    || format!("{:?}", def.type_).to_lowercase().contains(&lower_query)
            })
            .collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 6. Comparación Final

### 6.1 Lo que es Leptos/WASM vs TypeScript

| Aspecto | React + TypeScript | Leptos + WASM |
|---------|-------------------|---------------|
| Rendimiento canvas | Límite ~1000-2000 elementos流畅 | 10,000+ elementos流畅 |
| Type safety | TypeScript (build-time) | Rust (compile-time, absolute) |
| Memory | GC automático | Gestión manual (sin GC) |
| Bundle size | ~200KB React | ~1-2MB WASM (una vez) |
| Startup time | ~100-200ms | ~50-100ms |
| Herramientas | Maduras | En desarrollo |
| Hiring pool | Grande | Pequeña pero especializada |

### 6.2 Veredicto Final

**Para ArchFlow como "Figma para Architects":**

- **Canvas rendering**: Leptos/WASM es la elección correcta
- **Rendimiento**: Comparable a aplicaciones nativas
- **Diferenciación**: El PRD especifica "10,000+ elementos a 60fps"

**Trade-off:**
- TypeScript sería más rápido de desarrollar inicialmente
- Leptos/WASM entrega el rendimiento especificado en el PRD

**Recomendación:** Leptos/WASM para el canvas y lógica de dominio. Esta es exactamente la arquitectura que Figma usa (C++ en WebAssembly).

---

## 7. Roadmap Revisado (Leptos/WASM)

| Sprint | Duración | Entregable | Complejidad |
|--------|----------|------------|-------------|
| 0 | 1 semana | Setup Rust/WASM toolchain | Media |
| 1 | 2 semanas | Canvas renderer (WebGPU) | Alta |
| 2 | 2 semanas | Component registry + drag-drop | Media |
| 3 | 2 semanas | Properties panel + validation | Media |
| 4 | 2 semanas | Undo/Redo + Autosave (IndexedDB) | Media |
| 5 | 2 semanas | Export Terraform (Rust → WASM) | Media |
| 6 | 2 semanas | AUF Import/Export | Baja |
| 7 | 1 semana | Testing + Optimization | Media |
| **Total** | **12 semanas** | **MVP Completo** | |

---

## 8. Conclusión

El análisis revisado confirma que **Leptos/WASM es el enfoque correcto** para ArchFlow, replicando la arquitectura que hace que Figma funcione:

1. **Lenguaje compilado a WASM** (C++ en Figma, Rust/Leptos en ArchFlow)
2. **GPU-accelerated rendering** (WebGL/WebGPU)
3. **Type-safety extremo** (TypeScript vs Rust)
4. **Gestión de memoria manual** (sin GC pauses)

Esto entrega el rendimiento de 10,000+ nodos a 60fps especificado en el PRD.

¿Quieres que proceda con la implementación del proyecto Leptos/WASM?
