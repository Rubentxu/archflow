# ArchFlow MVP Demo - Implementation Plan

## Executive Summary

**Goal**: Create a fully functional web demo that showcases ArchFlow's capabilities with Rust/WASM rendering, real-time collaboration features, and interactive UI elements.

**Approach**: Use Canvas 2D API (via web-sys) as the primary rendering backend for maximum compatibility, with the option to upgrade to WebGPU later.

---

## Current Architecture Analysis

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ArchFlow Crate Structure                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────┐  ┌──────────────────────┐  ┌───────────────┐  │
│  │  archflow-wasm-collab │  │  archflow-renderers  │  │ archflow-     │  │
│  │  - SharedBuffer       │  │  - BatchRenderer2D   │  │  primitives   │  │
│  │  - BinaryDeltaCodec   │  │  - RenderContext     │  │  - Rectangle  │  │
│  │  - WasmBridge         │  │  - Renderable trait  │  │  - Ellipse    │  │
│  └──────────────────────┘  └──────────────────────┘  │  - Selection  │  │
│                                                      └───────────────┘  │
│  ┌──────────────────────┐  ┌──────────────────────┐                     │
│  │  archflow-records     │  │  archflow-spatial    │                     │
│  │  - RecordStore        │  │  - RTree spatial     │                     │
│  │  - ChangeSet          │  │    indexing          │                     │
│  │  - Record trait       │  │  - ViewportManager   │                     │
│  └──────────────────────┘  └──────────────────────┘                     │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## MVP Demo Requirements

### Core Features (MVP 100%)

1. **Interactive Canvas**
   - Render rectangles, ellipses, lines
   - Pan and zoom controls
   - Shape selection via click

2. **Shape Manipulation**
   - Drag and drop shapes
   - Resize handles on selection
   - Color picker for fill/stroke

3. **WASM Integration**
   - Load ArchFlow WASM module
   - Zero-copy SharedBuffer for shape data
   - Binary delta sync for updates

4. **Real-time Collaboration (Demo)**
   - Simulated multi-user cursors
   - Change notifications via deltas

### Nice-to-Have (Post-MVP)

- Text rendering
- Path/pen tool
- Layer management
- Undo/redo
- Undo/redo with delta sync

---

## Task Breakdown

### Phase 1: Project Setup (1 hour)

#### Task 1.1: Create Demo Web Crate

**File**: `crates/demo-web/Cargo.toml`

```toml
[package]
name = "archflow-demo-web"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Internal
archflow-wasm-collab = { path = "../archflow-wasm-collab" }
archflow-records = { path = "../archflow-records" }
archflow-primitives = { path = "../archflow-primitives" }
archflow-renderers = { path = "../archflow-renderers" }
archflow-spatial = { path = "../archflow-spatial" }
archflow-core = { path = "../archflow-core" }

# WASM bindings
wasm-bindgen = "0.2"
js-sys = "0.3"

[dependencies.web-sys]
version = "0.3"
features = [
    "console",
    "Window",
    "Document",
    "Element",
    "HtmlCanvasElement",
    "HtmlElement",
    "MouseEvent",
    "KeyboardEvent",
    "WheelEvent",
    "CanvasRenderingContext2d",
    "CanvasGradient",
    "CanvasPattern",
    "ImageData",
    "Path2d",
    "TextMetrics",
    "CssStyleDeclaration",
    "EventTarget",
    "Performance",
    "DomRect",
    "KeyboardEventInit",
    "MouseEventInit",
]

[dependencies.wgpu]
version = "0.19"
features = ["webgl"]

[profile.release]
opt-level = "s"
lto = true
```

**Status**: ⏳ Pending

---

#### Task 1.2: Create Demo HTML Structure

**File**: `crates/demo-web/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ArchFlow Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: system-ui, sans-serif; overflow: hidden; }
        
        #app {
            display: grid;
            grid-template-rows: 48px 1fr 24px;
            height: 100vh;
        }
        
        #toolbar {
            background: #1e1e1e;
            color: #fff;
            display: flex;
            align-items: center;
            padding: 0 16px;
            gap: 8px;
            border-bottom: 1px solid #333;
        }
        
        #toolbar button {
            background: #333;
            border: none;
            color: #fff;
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
        }
        
        #toolbar button:hover { background: #444; }
        #toolbar button.active { background: #0066cc; }
        
        #canvas-container {
            position: relative;
            overflow: hidden;
        }
        
        #canvas {
            position: absolute;
            top: 0;
            left: 0;
        }
        
        #statusbar {
            background: #1e1e1e;
            color: #888;
            display: flex;
            align-items: center;
            padding: 0 16px;
            font-size: 12px;
            gap: 16px;
        }
        
        #cursors {
            position: absolute;
            top: 0;
            left: 0;
            pointer-events: none;
        }
        
        .cursor {
            position: absolute;
            width: 20px;
            height: 20px;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 10px;
            font-weight: bold;
            color: white;
            transform: translate(-50%, -50%);
            transition: left 0.1s, top 0.1s;
        }
    </style>
</head>
<body>
    <div id="app">
        <div id="toolbar">
            <span style="font-weight: bold; margin-right: 16px;">🎨 ArchFlow</span>
            <button data-tool="select" class="active">Select</button>
            <button data-tool="rect">Rectangle</button>
            <button data-tool="ellipse">Ellipse</button>
            <button data-tool="line">Line</button>
            <div style="flex: 1;"></div>
            <button id="btn-clear">Clear</button>
        </div>
        <div id="canvas-container">
            <canvas id="canvas"></canvas>
            <div id="cursors"></div>
        </div>
        <div id="statusbar">
            <span id="status-shape-count">Shapes: 0</span>
            <span id="status-pos">Position: 0, 0</span>
            <span id="status-sync">Sync: -</span>
        </div>
    </div>
    
    <script type="module">
        import init, { ArchFlowDemo } from './pkg/archflow_demo_web.js';
        
        async function main() {
            await init();
            
            const canvas = document.getElementById('canvas');
            const container = document.getElementById('canvas-container');
            
            // Resize canvas to container
            function resizeCanvas() {
                canvas.width = container.clientWidth;
                canvas.height = container.clientHeight;
            }
            resizeCanvas();
            window.addEventListener('resize', resizeCanvas);
            
            // Create demo instance
            const demo = new ArchFlowDemo(canvas);
            
            // Tool selection
            document.querySelectorAll('[data-tool]').forEach(btn => {
                btn.addEventListener('click', () => {
                    document.querySelectorAll('[data-tool]').forEach(b => b.classList.remove('active'));
                    btn.classList.add('active');
                    demo.set_tool(btn.dataset.tool);
                });
            });
            
            // Clear button
            document.getElementById('btn-clear').addEventListener('click', () => {
                demo.clear();
            });
            
            // Mouse events
            canvas.addEventListener('mousedown', e => {
                const rect = canvas.getBoundingClientRect();
                demo.on_mousedown(e.clientX - rect.left, e.clientY - rect.top, e.button);
            });
            
            canvas.addEventListener('mousemove', e => {
                const rect = canvas.getBoundingClientRect();
                demo.on_mousemove(e.clientX - rect.left, e.clientY - rect.top);
                
                // Update status bar
                document.getElementById('status-pos').textContent = 
                    `Position: ${Math.round(e.clientX - rect.left)}, ${Math.round(e.clientY - rect.top)}`;
            });
            
            canvas.addEventListener('mouseup', e => {
                const rect = canvas.getBoundingClientRect();
                demo.on_mouseup(e.clientX - rect.left, e.clientY - rect.top);
            });
            
            // Wheel for zoom
            canvas.addEventListener('wheel', e => {
                e.preventDefault();
                const rect = canvas.getBoundingClientRect();
                demo.on_wheel(e.clientX - rect.left, e.clientY - rect.top, e.deltaY > 0);
            }, { passive: false });
            
            // Animation loop
            function render() {
                demo.render();
                document.getElementById('status-shape-count').textContent = 
                    `Shapes: ${demo.shape_count()}`;
                requestAnimationFrame(render);
            }
            render();
            
            // Simulate remote cursor
            setInterval(() => {
                const x = Math.random() * canvas.width;
                const y = Math.random() * canvas.height;
                demo.simulate_remote_cursor(x, y, 'Alice');
            }, 3000);
            
            console.log('ArchFlow Demo initialized');
        }
        
        main().catch(console.error);
    </script>
</body>
</html>
```

**Status**: ⏳ Pending

---

### Phase 2: Core WASM Module (3 hours)

#### Task 2.1: Implement Demo WASM Struct

**File**: `crates/demo-web/src/lib.rs`

```rust
//! ArchFlow Demo Web - WASM module for browser demo
//!
//! This module provides the main WASM interface for the interactive demo.

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

mod shapes;
mod state;

use shapes::{Shape, ShapeType, ShapeId};
use state::DemoState;

/// Main WASM struct exposed to JavaScript
#[wasm_bindgen]
pub struct ArchFlowDemo {
    state: DemoState,
    context: CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ArchFlowDemo {
    /// Creates a new demo instance
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<ArchFlowDemo, JsValue> {
        console_error_panic_hook::set_once();
        
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        let width = canvas.width();
        let height = canvas.height();
        
        Ok(ArchFlowDemo {
            state: DemoState::new(),
            context,
            width,
            height,
        })
    }
    
    // === Tool Management ===
    
    /// Sets the current tool
    #[wasm_bindgen]
    pub fn set_tool(&mut self, tool: &str) {
        self.state.set_tool(tool);
    }
    
    // === Input Handling ===
    
    #[wasm_bindgen]
    pub fn on_mousedown(&mut self, x: f64, y: f64, button: u16) {
        self.state.on_mousedown(x, y, button);
    }
    
    #[wasm_bindgen]
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        self.state.on_mousemove(x, y);
    }
    
    #[wasm_bindgen]
    pub fn on_mouseup(&mut self, x: f64, y: f64) {
        self.state.on_mouseup(x, y);
    }
    
    #[wasm_bindgen]
    pub fn on_wheel(&mut self, x: f64, y: f64, zoom_out: bool) {
        self.state.on_wheel(x, y, zoom_out);
    }
    
    // === Shape Operations ===
    
    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.state.add_rect(x, y, width, height);
    }
    
    #[wasm_bindgen]
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) {
        self.state.add_ellipse(x, y, radius_x, radius_y);
    }
    
    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.state.add_line(x1, y1, x2, y2);
    }
    
    #[wasm_bindgen]
    pub fn delete_selected(&mut self) {
        self.state.delete_selected();
    }
    
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.state.clear();
    }
    
    // === Rendering ===
    
    #[wasm_bindgen]
    pub fn render(&mut self) {
        // Clear canvas
        self.context.set_fill_style(&JsValue::from_str("#1e1e1e"));
        self.context.fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
        
        // Render grid
        self.render_grid();
        
        // Render all shapes
        for shape in self.state.shapes() {
            self.render_shape(shape);
        }
        
        // Render selection
        if let Some(selection) = self.state.selection() {
            self.render_selection(selection);
        }
    }
    
    // === Query Methods ===
    
    #[wasm_bindgen]
    pub fn shape_count(&self) -> usize {
        self.state.shape_count()
    }
    
    // === Collaboration Simulation ===
    
    #[wasm_bindgen]
    pub fn simulate_remote_cursor(&mut self, x: f64, y: f64, name: &str) {
        self.state.add_remote_cursor(x, y, name);
    }
    
    #[wasm_bindgen]
    pub fn get_delta(&self) -> Vec<u8> {
        self.state.serialize_delta()
    }
}

// Private rendering methods
impl ArchFlowDemo {
    fn render_grid(&self) {
        let grid_size = 20.0;
        self.context.set_stroke_style(&JsValue::from_str("#2a2a2a"));
        self.context.set_line_width(1.0);
        
        // Vertical lines
        let mut path = web_sys::Path2d::new().unwrap();
        let mut x = 0.0;
        while x < self.width as f64 {
            path.move_to(x, 0.0);
            path.line_to(x, self.height as f64);
            x += grid_size;
        }
        // Horizontal lines
        let mut y = 0.0;
        while y < self.height as f64 {
            path.move_to(0.0, y);
            path.line_to(self.width as f64, y);
            y += grid_size;
        }
        self.context.stroke_with_path(&path);
    }
    
    fn render_shape(&self, shape: &Shape) {
        match shape.shape_type {
            ShapeType::Rectangle => {
                self.render_rectangle(shape);
            }
            ShapeType::Ellipse => {
                self.render_ellipse(shape);
            }
            ShapeType::Line => {
                self.render_line(shape);
            }
        }
    }
    
    fn render_rectangle(&self, shape: &Shape) {
        let color = shape.color_as_css();
        self.context.set_fill_style(&JsValue::from_str(&color));
        self.context.set_stroke_style(&JsValue::from_str("#fff"));
        self.context.set_line_width(1.0);
        
        let path = web_sys::Path2d::new().unwrap();
        path.rect(shape.x, shape.y, shape.width, shape.height);
        self.context.fill_with_path_2d(&path);
        self.context.stroke_with_path_2d(&path);
    }
    
    fn render_ellipse(&self, shape: &Shape) {
        let color = shape.color_as_css();
        self.context.set_fill_style(&JsValue::from_str(&color));
        self.context.set_stroke_style(&JsValue::from_str("#fff"));
        self.context.set_line_width(1.0);
        
        let rx = shape.width / 2.0;
        let ry = shape.height / 2.0;
        let cx = shape.x + rx;
        let cy = shape.y + ry;
        
        let path = web_sys::Path2d::new().unwrap();
        path.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::PI * 2.0);
        self.context.fill_with_path_2d(&path);
        self.context.stroke_with_path_2d(&path);
    }
    
    fn render_line(&self, shape: &Shape) {
        self.context.set_stroke_style(&JsValue::from_str(&shape.color_as_css()));
        self.context.set_line_width(2.0);
        
        let path = web_sys::Path2d::new().unwrap();
        path.move_to(shape.x, shape.y);
        path.line_to(shape.x + shape.width, shape.y + shape.height);
        self.context.stroke_with_path_2d(&path);
    }
    
    fn render_selection(&self, selection: &crate::shapes::Rect) {
        self.context.set_stroke_style(&JsValue::from_str("#0066cc"));
        self.context.set_line_width(2.0);
        self.context.set_line_dash(&JsValue::from_str("5,5")).unwrap();
        
        let path = web_sys::Path2d::new().unwrap();
        path.rect(selection.x, selection.y, selection.width, selection.height);
        self.context.stroke_with_path_2d(&path);
        
        self.context.set_line_dash(&JsValue::from_str("")).unwrap();
        
        // Draw resize handles
        self.render_handle(selection.x, selection.y);
        self.render_handle(selection.x + selection.width, selection.y);
        self.render_handle(selection.x, selection.y + selection.height);
        self.render_handle(selection.x + selection.width, selection.y + selection.height);
    }
    
    fn render_handle(&self, x: f64, y: f64) {
        self.context.set_fill_style(&JsValue::from_str("#0066cc"));
        let path = web_sys::Path2d::new().unwrap();
        path.rect(x - 4.0, y - 4.0, 8.0, 8.0);
        self.context.fill_with_path_2d(&path);
    }
}
```

**Status**: ⏳ Pending

---

#### Task 2.2: Define Shape Types

**File**: `crates/demo-web/src/shapes.rs`

```rust
//! Shape definitions for the demo

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Clone, Debug)]
pub struct Shape {
    pub id: ShapeId,
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: [u8; 4],
    pub rotation: f32,
}

impl Shape {
    pub fn color_as_css(&self) -> String {
        format!(
            "rgba({},{},{},{})",
            self.color[0], self.color[1], self.color[2], self.color[3]
        )
    }
    
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width
            && y >= self.y && y <= self.y + self.height
    }
    
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeId(pub u64);

impl ShapeId {
    pub fn next() -> Self {
        ShapeId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Clone, Debug)]
pub struct RemoteCursor {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub color: [u8; 4],
}

impl RemoteCursor {
    pub fn new(x: f64, y: f64, name: &str) -> Self {
        let hash = name.bytes().fold(0u64, |acc, b| acc * 31 + b as u64);
        Self {
            x,
            y,
            name: name.to_string(),
            color: [
                ((hash >> 24) & 0xFF) as u8,
                ((hash >> 16) & 0xFF) as u8,
                ((hash >> 8) & 0xFF) as u8,
                200,
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShapeStore {
    shapes: HashMap<ShapeId, Shape>,
    order: Vec<ShapeId>,
}

impl ShapeStore {
    pub fn new() -> Self {
        Self {
            shapes: HashMap::new(),
            order: Vec::new(),
        }
    }
    
    pub fn add(&mut self, shape: Shape) -> ShapeId {
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.order.push(id);
        id
    }
    
    pub fn remove(&mut self, id: ShapeId) -> Option<Shape> {
        self.order.retain(|&oid| oid != id);
        self.shapes.remove(&id)
    }
    
    pub fn get(&self, id: ShapeId) -> Option<&Shape> {
        self.shapes.get(&id)
    }
    
    pub fn get_mut(&mut self, id: ShapeId) -> Option<&mut Shape> {
        self.shapes.get_mut(&id)
    }
    
    pub fn contains(&self, id: ShapeId) -> bool {
        self.shapes.contains_key(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Shape> {
        self.order.iter().filter_map(|id| self.shapes.get(id))
    }
    
    pub fn count(&self) -> usize {
        self.shapes.len()
    }
    
    pub fn find_at_point(&self, x: f64, y: f64) -> Option<ShapeId> {
        // Search in reverse order (top to bottom)
        for id in self.order.iter().rev() {
            if let Some(shape) = self.shapes.get(id) {
                if shape.contains(x, y) {
                    return Some(*id);
                }
            }
        }
        None
    }
    
    pub fn clear(&mut self) {
        self.shapes.clear();
        self.order.clear();
    }
}
```

**Status**: ⏳ Pending

---

#### Task 2.3: Implement Demo State Machine

**File**: `crates/demo-web/src/state.rs`

```rust
//! Demo state management

use crate::shapes::{Shape, ShapeId, ShapeStore, ShapeType, RemoteCursor};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tool {
    Select,
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionState {
    Idle,
    Dragging {
        shape_id: ShapeId,
        start_x: f64,
        start_y: f64,
        original_x: f64,
        original_y: f64,
    },
    Creating {
        shape_type: ShapeType,
        start_x: f64,
        start_y: f64,
        current_x: f64,
        current_y: f64,
    },
    Resizing {
        shape_id: ShapeId,
        handle: ResizeHandle,
        start_x: f64,
        start_y: f64,
        original_width: f64,
        original_height: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug)]
pub struct DemoState {
    tool: Tool,
    interaction: InteractionState,
    shapes: ShapeStore,
    selection: Option<ShapeId>,
    cursors: Vec<RemoteCursor>,
    pan_offset: (f64, f64),
    zoom: f32,
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            tool: Tool::Select,
            interaction: InteractionState::Idle,
            shapes: ShapeStore::new(),
            selection: None,
            cursors: Vec::new(),
            pan_offset: (0.0, 0.0),
            zoom: 1.0,
        }
    }
    
    // === Tool Management ===
    
    pub fn set_tool(&mut self, tool: &str) {
        self.tool = match tool {
            "rect" | "rectangle" => Tool::Rectangle,
            "ellipse" => Tool::Ellipse,
            "line" => Tool::Line,
            _ => Tool::Select,
        };
        self.interaction = InteractionState::Idle;
    }
    
    // === Input Handling ===
    
    pub fn on_mousedown(&mut self, x: f64, y: f64, _button: u16) {
        let world_x = (x - self.pan_offset.0) / self.zoom as f64;
        let world_y = (y - self.pan_offset.1) / self.zoom as f64;
        
        match self.tool {
            Tool::Select => {
                if let Some(id) = self.shapes.find_at_point(world_x, world_y) {
                    self.selection = Some(id);
                    self.interaction = InteractionState::Dragging {
                        shape_id: id,
                        start_x: world_x,
                        start_y: world_y,
                        original_x: self.shapes.get(id).map(|s| s.x).unwrap_or(0.0),
                        original_y: self.shapes.get(id).map(|s| s.y).unwrap_or(0.0),
                    };
                } else {
                    self.selection = None;
                }
            }
            Tool::Rectangle | Tool::Ellipse | Tool::Line => {
                self.interaction = InteractionState::Creating {
                    shape_type: match self.tool {
                        Tool::Rectangle => ShapeType::Rectangle,
                        Tool::Ellipse => ShapeType::Ellipse,
                        Tool::Line => ShapeType::Line,
                        _ => unreachable!(),
                    },
                    start_x: world_x,
                    start_y: world_y,
                    current_x: world_x,
                    current_y: world_y,
                };
            }
        }
    }
    
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        let world_x = (x - self.pan_offset.0) / self.zoom as f64;
        let world_y = (y - self.pan_offset.1) / self.zoom as f64;
        
        match &mut self.interaction {
            InteractionState::Dragging { shape_id, start_x, start_y, original_x, original_y } => {
                if let Some(shape) = self.shapes.get_mut(*shape_id) {
                    let dx = world_x - start_x;
                    let dy = world_y - start_y;
                    shape.x = original_x + dx;
                    shape.y = original_y + dy;
                }
            }
            InteractionState::Creating { current_x, current_y, .. } => {
                *current_x = world_x;
                *current_y = world_y;
            }
            _ => {}
        }
    }
    
    pub fn on_mouseup(&mut self, x: f64, y: f64) {
        let world_x = (x - self.pan_offset.0) / self.zoom as f64;
        let world_y = (y - self.pan_offset.1) / self.zoom as f64;
        
        match std::mem::replace(&mut self.interaction, InteractionState::Idle) {
            InteractionState::Creating { shape_type, start_x, start_y, .. } => {
                let min_x = start_x.min(world_x);
                let min_y = start_y.min(world_y);
                let width = (world_x - start_x).abs();
                let height = (world_y - start_y).abs();
                
                if width > 5.0 && height > 5.0 {
                    let shape = Shape {
                        id: ShapeId::next(),
                        shape_type,
                        x: min_x,
                        y: min_y,
                        width,
                        height,
                        color: [70, 130, 180, 255],
                        rotation: 0.0,
                    };
                    self.shapes.add(shape);
                    self.selection = Some(shape.id);
                }
            }
            _ => {}
        }
    }
    
    pub fn on_wheel(&mut self, _x: f64, _y: f64, zoom_out: bool) {
        let delta = if zoom_out { -0.1 } else { 0.1 };
        self.zoom = (self.zoom + delta as f32).clamp(0.1, 5.0);
    }
    
    // === Shape Operations ===
    
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let shape = Shape {
            id: ShapeId::next(),
            shape_type: ShapeType::Rectangle,
            x, y, width, height,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        };
        self.shapes.add(shape);
    }
    
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) {
        let shape = Shape {
            id: ShapeId::next(),
            shape_type: ShapeType::Ellipse,
            x: x - radius_x,
            y: y - radius_y,
            width: radius_x * 2.0,
            height: radius_y * 2.0,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        };
        self.shapes.add(shape);
    }
    
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let shape = Shape {
            id: ShapeId::next(),
            shape_type: ShapeType::Line,
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        };
        self.shapes.add(shape);
    }
    
    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selection {
            self.shapes.remove(id);
            self.selection = None;
        }
    }
    
    pub fn clear(&mut self) {
        self.shapes.clear();
        self.selection = None;
    }
    
    // === Query Methods ===
    
    pub fn shapes(&self) -> impl Iterator<Item = &Shape> {
        self.shapes.iter()
    }
    
    pub fn shape_count(&self) -> usize {
        self.shapes.count()
    }
    
    pub fn selection(&self) -> Option<Rect> {
        self.selection.and_then(|id| {
            self.shapes.get(id).map(|s| Rect {
                x: s.x,
                y: s.y,
                width: s.width,
                height: s.height,
            })
        })
    }
    
    // === Collaboration ===
    
    pub fn add_remote_cursor(&mut self, x: f64, y: f64, name: &str) {
        // Remove old cursor for same user
        self.cursors.retain(|c| c.name != name);
        self.cursors.push(RemoteCursor::new(x, y, name));
    }
    
    pub fn serialize_delta(&self) -> Vec<u8> {
        // Simple binary format: [type, id_size, id, data...]
        let mut result = Vec::new();
        for shape in self.shapes.iter() {
            result.push(shape.shape_type as u8);
            result.extend_from_slice(&shape.id.0.to_le_bytes());
            result.extend_from_slice(&shape.x.to_le_bytes());
            result.extend_from_slice(&shape.y.to_le_bytes());
            result.extend_from_slice(&shape.width.to_le_bytes());
            result.extend_from_slice(&shape.height.to_le_bytes());
        }
        result
    }
}
```

**Status**: ⏳ Pending

---

### Phase 3: Integration with WASM Bridge (1 hour)

#### Task 3.1: Export to SharedBuffer

**File**: `crates/demo-web/src/lib.rs` (add method)

```rust
impl ArchFlowDemo {
    /// Exports current state to SharedBuffer for JS access
    pub fn export_to_shared_buffer(&self) -> *const archflow_wasm_collab::RenderAttribute {
        // Convert shapes to RenderAttributes
        // This enables zero-copy access from JS
        0 as *const archflow_wasm_collab::RenderAttribute
    }
    
    /// Gets the current change set as binary delta
    pub fn get_changes(&self) -> Vec<u8> {
        self.state.serialize_delta()
    }
}
```

**Status**: ⏳ Pending

---

### Phase 4: Build Configuration (30 min)

#### Task 4.1: Create Build Scripts

**File**: `crates/demo-web/build.sh`

```bash
#!/bin/bash
set -e

echo "Building ArchFlow Demo WASM module..."

# Build with wasm-pack
wasm-pack build --target web --out-dir ../demo/pkg

echo "Build complete! Files in ../demo/pkg/"
echo ""
echo "To serve the demo:"
echo "  cd ../demo && python3 -m http.server 8080"
echo ""
echo "Server must have COOP/COEP headers for SharedArrayBuffer:"
echo "  python3 -m http.server 8080 \\"
echo "    --header 'Cross-Origin-Opener-Policy: same-origin' \\"
echo "    --header 'Cross-Origin-Embedder-Policy: require-corp'"
```

**File**: `crates/demo-web/serve.sh`

```bash
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEMO_DIR="$PROJECT_ROOT/demo"

echo "Serving ArchFlow Demo at http://localhost:8080"
echo "Press Ctrl+C to stop"
echo ""

# Build and run the Rust server with COOP/COEP headers
cd "$PROJECT_ROOT"
cargo run -p archflow-demo-server -- --dir "$DEMO_DIR" --port 8080
```

**File**: `crates/demo-server/Cargo.toml`

```toml
[package]
name = "archflow-demo-server"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "serve"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["full"] }
http = "1.0"
```

**File**: `crates/demo-server/src/main.rs`

```rust
//! Simple HTTP server with COOP/ SharedArrayBuffer support

use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
struct Args {
    dir: PathBuf,
    port: u16,
}

async fn handle_request(
    stream: &mut TcpStream,
    dir: &PathBuf,
    method: &str,
    path: &str,
) -> std::io::Result<()> {
    let file_path = if path == "/" || path.is_empty() {
        dir.join("index.html")
    } else {
        dir.join(&path[1..]) // Remove leading slash
    };

    let response = if file_path.exists() && file_path.is_file() {
        let mut file = File::open(&file_path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        let content_type = match file_path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html",
            Some("js") => "application/javascript",
            Some("wasm") => "application/wasm",
            Some("css") => "text/css",
            Some("json") => "application/json",
            _ => "application/octet-stream",
        };

        format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Cross-Origin-Opener-Policy: same-origin\r\n\
             Cross-Origin-Embedder-Policy: require-corp\r\n\
             \r\n",
            content_type,
            contents.len()
        )
    } else {
        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
    };

    stream.write_all(response.as_bytes()).await?;
    if !response.ends_with("\r\n\r\n") {
        stream.write_all(b"\r\n").await?;
    }
    if response.starts_with("HTTP/1.1 200 OK") {
        stream.write_all(&contents).await?;
    }

    Ok(())
}

async fn handle_client(mut stream: TcpStream, dir: PathBuf) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer).await;

    let request = String::from_utf8_lossy(&buffer);
    let parts: Vec<&str> = request.split("\r\n").collect();

    if parts.is_empty() {
        return;
    }

    let request_line = parts[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if let Err(e) = handle_request(&mut stream, &dir, method, path).await {
        eprintln!("Error handling request: {}", e);
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut dir = PathBuf::from(".");
    let mut port = 8080;

    for i in (0..args.len()).step_by(2) {
        if i + 1 < args.len() {
            match args[i].as_str() {
                "--dir" => dir = PathBuf::from(&args[i + 1]),
                "--port" => port = args[i + 1].parse().unwrap_or(8080),
                _ => {}
            }
        }
    }

    if !dir.exists() {
        eprintln!("Directory not found: {:?}", dir);
        std::process::exit(1);
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let dir = Arc::new(dir);

    println!("ArchFlow Demo Server");
    println!("====================");
    println!("Serving: {:?}", dir);
    println!("Address: http://localhost:{}", port);
    println!("");
    println!("Press Ctrl+C to stop");
    println!("");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("[{}] Connected", addr);
        let dir = Arc::clone(&dir);
        tokio::spawn(async move {
            handle_client(stream, dir).await;
        });
    }
}
```

**Status**: ⏳ Pending

---

## Implementation Order & Estimates

```
┌─────────────────────────────────────────────────────────────────────┐
│                         MVP Demo Timeline                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Week 1                                                              │
│  ├── Day 1 (2h):   Phase 1 - Project Setup                          │
│  │   ├── Task 1.1: Create demo-web crate                           │
│  │   └── Task 1.2: Create HTML structure                           │
│  │                                                                  │
│  ├── Day 2-3 (6h): Phase 2 - Core WASM Module                       │
│  │   ├── Task 2.1: Implement ArchFlowDemo struct                   │
│  │   ├── Task 2.2: Define Shape types                              │
│  │   └── Task 2.3: Implement state machine                         │
│  │                                                                  │
│  ├── Day 4 (2h):   Phase 3 - WASM Bridge Integration               │
│  └── Day 5 (1h):   Phase 4 - Build & Test                          │
│                                                                     │
│  Total: ~12 hours                                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Success Criteria

- [ ] Demo loads without errors
- [ ] Canvas renders with grid background
- [ ] Rectangle/Ellipse/Line tools work
- [ ] Shapes can be selected and dragged
- [ ] Zoom with mouse wheel works
- [ ] WASM module loads successfully
- [ ] 60 FPS render loop
- [ ] No JavaScript rendering fallback needed

---

## File Structure After Implementation

```
crates/
├── demo-web/
│   ├── Cargo.toml
│   ├── build.sh
│   ├── serve.py
│   ├── src/
│   │   ├── lib.rs          # Main WASM module
│   │   ├── shapes.rs       # Shape definitions
│   │   └── state.rs        # State machine
│   └── index.html          # Demo page (copied to demo/)
│
└── demo/                    # Output directory (gitignored)
    ├── index.html
    └── pkg/
        ├── archflow_demo_web.js
        ├── archflow_demo_web_bg.js
        ├── archflow_demo_web_bg.wasm
        └── package.json
```

---

## Future Enhancements (Post-MVP)

| Feature | Priority | Description |
|---------|----------|-------------|
| Text rendering | High | Add text tool with font selection |
| Undo/redo | High | RecordStore + ChangeSet for history |
| Layers | Medium | Z-index management UI |
| Snap to grid | Medium | Alignment guides during drag |
| Export PNG | Low | Canvas to image export |
| WebGPU renderer | Low | Upgrade to GPU rendering |
| Collaborative editing | Low | Real-time sync with deltas |

---

## Verification Checklist

```bash
# Build the WASM module
cd crates/demo-web
wasm-pack build --target web --out-dir ../demo/pkg

# Serve with correct headers
cd ..
python3 -m http.server 8080 \
    --header "Cross-Origin-Opener-Policy: same-origin" \
    --header "Cross-Origin-Embedder-Policy: require-corp"

# In browser, open http://localhost:8080/demo/
# Verify:
# 1. Canvas displays with grid
# 2. Click Rectangle tool, drag to create shape
# 3. Shape appears and can be selected
# 4. Drag shape to move it
# 5. Scroll wheel to zoom in/out
# 6. Check console for any errors
```

---

## Dependencies to Add

```toml
# In crates/demo-web/Cargo.toml

[dependencies]
# Existing...

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
```

---

## Related Documentation

- Previous: `docs/WASM_DEMO_PLAN.md` (reference)
- Architecture: `docs/ARCHITECTURE-DESIGN.md`
- WASM Bridge: `docs/WASM-BRIDGE-IMPLEMENTATION.md`
- Renderers: See `crates/archflow-renderers/src/lib.rs`
