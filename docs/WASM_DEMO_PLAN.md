# ArchFlow WASM Demo - Implementation Plan

## Executive Summary

**Current State**: The web demo at `crates/demo/index.html` is pure JavaScript simulation. The Rust/WASM renderer exists but does nothing.

**Goal**: Create a functional web demo where Rust code actually renders shapes, text, and UI elements via WASM.

**Approach**: Use Canvas 2D API via `web-sys` for rendering. This is the fastest path to a working demo while demonstrating the architecture.

---

## Current Architecture Analysis

```
┌─────────────────────────────────────────────────────────────────┐
│                    crates/demo/index.html                        │
│                    (Pure JavaScript)                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              CanvasRenderingContext2d                     │   │
│  │         (All rendering happens in JS)                     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

                        VS

┌─────────────────────────────────────────────────────────────────┐
│                    crates/demo/index.html                        │
│                    (WASM Integration)                            │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              JavaScript Bridge                            │   │
│  │     instantiateWasm() → archflowWasm.render()             │   │
│  └────────────────────────┬──────────────────────────────────┘   │
│                           │                                      │
│                           ▼                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              archflow-wasm (WASM Module)                  │   │
│  │  ┌────────────────────────────────────────────────────┐   │   │
│  │  │              ArchFlowWasm                          │   │   │
│  │  │  - canvas: HtmlCanvasElement                       │   │   │
│  │  │  - context: CanvasRenderingContext2d               │   │   │
│  │  │  - render() → clear → draw_shapes → draw_text      │   │   │
│  │  └────────────────────────────────────────────────────┘   │   │
│  └────────────────────────┬──────────────────────────────────┘   │
│                           │                                      │
│                           ▼                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              archflow-renderer                            │   │
│  │  - PathTessellator: rect, ellipse, paths                 │   │
│  │  - TextRenderer: cosmic-text integration                 │   │
│  │  - Shape storage: Entity list from ECS                   │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Task 1: Update WASM Dependencies

**File**: `crates/wasm/Cargo.toml`

**Required web-sys features** (add to existing):
```toml
[dependencies.web-sys]
version = "0.3"
features = [
    # Existing
    "console",
    "Window",
    "Document",
    "Element",
    "HtmlCanvasElement",
    "MouseEvent",
    "KeyboardEvent",
    "Performance",
    "TextMetrics",
    "CanvasRenderingContext2d",
    "CanvasGradient",
    # NEW - Needed for proper 2D rendering
    "CanvasPattern",
    "ImageData",
    "Path2d",
]
```

**Analysis**:
- `CanvasRenderingContext2d`: Core 2D drawing API - ALREADY PRESENT
- `Path2d`: For path-based drawing - MISSING
- `ImageData`: For pixel manipulation - MISSING

`✶ Insight ─────────────────────────────────────`
Web-sys provides Rust bindings to browser APIs. The `features` array in Cargo.toml is critical - each feature enables specific DOM APIs. Without them, you get compile-time errors when trying to use the API.
`─────────────────────────────────────────────────`

---

## Task 2: Enhance Renderer with Canvas 2D Bridge

**File**: `crates/renderer/src/lib.rs`

**New structure**:
```rust
pub struct Renderer {
    size: (u32, u32),
    // NEW: Canvas context for actual rendering
    context: Option<CanvasRenderingContext2d>,
    // NEW: Shape queue for batched rendering
    shape_queue: Vec<ShapeBatchItem>,
}

pub struct ShapeBatchItem {
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
    pub rotation: f32,
}

pub enum ShapeType {
    Rectangle,
    Ellipse,
    Text { text: String, font_size: f64 },
}
```

**Key changes to `render()` method**:
```rust
pub fn render(&mut self) -> RendererResult<()> {
    // Check if we have a canvas context
    let Some(ref ctx) = self.context else {
        return Ok(()); // No canvas, skip rendering
    };

    // Clear canvas
    let js_val = ctx.clear_rect(0.0, 0.0, self.size.0 as f64, self.size.1 as f64);

    // Draw all queued shapes
    for item in &self.shape_queue {
        match item.shape_type {
            ShapeType::Rectangle => self.draw_rect(ctx, item)?,
            ShapeType::Ellipse => self.draw_ellipse(ctx, item)?,
            ShapeType::Text { ref text, font_size } => {
                self.draw_text(ctx, item, text, font_size)?
            }
        }
    }

    // Clear queue after rendering
    self.shape_queue.clear();

    Ok(())
}
```

**Helper methods to add**:
```rust
impl Renderer {
    fn draw_rect(&self, ctx: &CanvasRenderingContext2d, item: &ShapeBatchItem) -> Result<(), RendererError> {
        let _ = ctx.set_fill_style(&JsValue::from_str(&format!(
            "rgba({},{},{},{})",
            item.color.r, item.color.g, item.color.b, item.color.a
        )));

        // Use Path2d for rounded rectangles
        let path = Path2d::new().map_err(|e| RendererError::WgpuError(e))?;
        // ... path operations ...

        ctx.fill_with_path_2d(&path);
        Ok(())
    }

    fn draw_ellipse(&self, ctx: &CanvasRenderingContext2d, item: &ShapeBatchItem) -> Result<(), RendererError> {
        // ctx.ellipse(x, y, radiusX, radiusY, rotation, startAngle, endAngle)
        let _ = ctx.ellipse(
            item.x, item.y,
            item.width / 2.0, item.height / 2.0,
            0.0, 0.0, std::f64::consts::PI * 2.0
        );
        ctx.fill();
        Ok(())
    }

    fn draw_text(&self, ctx: &CanvasRenderingContext2d, item: &ShapeBatchItem, text: &str, font_size: f64) -> Result<(), RendererError> {
        let _ = ctx.set_font(&format!("{}px system-ui", font_size));
        let _ = ctx.set_fill_style(&JsValue::from_str(&format!(
            "rgba({},{},{},{})",
            item.color.r, item.color.g, item.color.b, item.color.a
        )));
        ctx.fill_text(text, item.x, item.y)
            .map_err(|_| RendererError::WgpuError("Text render failed".into()))?;
        Ok(())
    }
}
```

`✶ Insight ─────────────────────────────────────`
Using `CanvasRenderingContext2d` is simpler than WebGL for 2D graphics. The API is imperative (like JS), which maps well to Rust's method calls. `Path2d` allows building complex shapes and rendering them in one call.
`─────────────────────────────────────────────────`

---

## Task 3: Add Canvas Context Setter to Renderer

**File**: `crates/renderer/src/lib.rs`

**New public method**:
```rust
impl Renderer {
    /// Set the canvas context for rendering (used by WASM layer)
    pub fn set_context(&mut self, context: CanvasRenderingContext2d) {
        self.context = Some(context);
    }

    /// Queue a shape for rendering
    pub fn queue_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.shape_queue.push(ShapeBatchItem {
            shape_type: ShapeType::Rectangle,
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
            color,
            rotation: 0.0,
        });
    }

    /// Queue text for rendering
    pub fn queue_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        self.shape_queue.push(ShapeBatchItem {
            shape_type: ShapeType::Text {
                text: text.to_string(),
                font_size: font_size as f64,
            },
            x: x as f64,
            y: y as f64,
            width: 0.0,
            height: 0.0,
            color,
            rotation: 0.0,
        });
    }
}
```

---

## Task 4: Update WASM Bindings

**File**: `crates/wasm/src/lib.rs`

**Updated implementation**:
```rust
#[wasm_bindgen]
pub struct ArchFlowWasm {
    renderer: Rc<RefCell<Renderer>>,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,  // NEW: Store context
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ArchFlowWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<ArchFlowWasm, JsValue> {
        // Get canvas context
        let context = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2D context")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Context is not 2D")?;

        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;

        let config = RendererBuilder::new()
            .with_size(width, height)
            .with_samples(1)  // Canvas 2D doesn't need MSAA
            .with_vsync(false);

        let mut renderer = Renderer::new(config);
        renderer.set_context(context.clone());  // NEW: Set context

        Ok(ArchFlowWasm {
            renderer: Rc::new(RefCell::new(renderer)),
            canvas,
            context,
            width,
            height,
        })
    }

    /// Add a rectangle to the render queue
    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: &[u8; 4]) {
        self.renderer.borrow_mut().queue_rect(
            x, y, width, height,
            Color::new(color[0], color[1], color[2], color[3])
        );
    }

    /// Add text to the render queue
    #[wasm_bindgen]
    pub fn add_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: &[u8; 4]) {
        self.renderer.borrow_mut().queue_text(
            text, x, y, font_size,
            Color::new(color[0], color[1], color[2], color[3])
        );
    }

    /// Clear all queued shapes
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        // Could add a clear() method to renderer
        self.renderer.borrow_mut().render().ok();
    }
}
```

---

## Task 5: Update HTML Demo to Use WASM

**File**: `crates/demo/index.html`

**Changes needed**:
1. Import WASM module
2. Instantiate `ArchFlowWasm`
3. Replace JavaScript rendering with WASM calls
4. Keep animation loop for driving the demo

```html
<script type="module">
    // Import WASM module
    import init, { ArchFlowWasm } from './pkg/archflow_wasm.js';

    async function main() {
        // Initialize WASM
        await init();

        // Get canvas and create engine
        const canvas = document.getElementById('canvas');
        const engine = new ArchFlowWasm(canvas);

        // Demo state
        let shapes = [];

        // Render function - calls WASM
        function render() {
            engine.clear();

            // Draw via WASM
            shapes.forEach(shape => {
                if (shape.type === 'rect') {
                    engine.add_rect(
                        shape.x - shape.width/2,
                        shape.y - shape.height/2,
                        shape.width,
                        shape.height,
                        [color.r, color.g, color.b, 255]
                    );
                }
            });

            // Trigger render
            engine.render();
        }
    }

    main();
</script>
```

`✶ Insight ─────────────────────────────────────`
The key is that the demo's "business logic" (shape positions, colors, animation) stays in JS or can be moved to Rust. The rendering layer moves to Rust/WASM. This hybrid approach is practical for incremental migration.
`─────────────────────────────────────────────────`

---

## Task 6: Build and Serve WASM

**Build command**:
```bash
# Build WASM package
cd crates/wasm
wasm-pack build --target web --out-dir ../demo/pkg

# Serve the demo
cd ../demo
python3 -m http.server 8080
```

**Required files in `crates/demo/`**:
```
crates/demo/
├── index.html          # Updated demo page
├── pkg/                # Generated by wasm-pack
│   ├── archflow_wasm.js
│   ├── archflow_wasm_bg.js
│   ├── archflow_wasm_bg.wasm
│   └── package.json
```

**Server requirements** (for SharedArrayBuffer if used):
```bash
# COOP/COEP headers required for SharedArrayBuffer
# With Python:
python3 -m http.server 8080 \
    --header "Cross-Origin-Opener-Policy: same-origin" \
    --header "Cross-Origin-Embedder-Policy: require-corp"
```

---

## Task 7: Add Tests for WASM Rendering

**File**: `crates/wasm/src/lib.rs`

```rust
#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_rect_queue() {
        // Test that add_rect doesn't panic
        // Test that shape is queued correctly
    }

    #[wasm_bindgen_test]
    fn test_text_queue() {
        // Test that add_text doesn't panic
        // Test that text is queued correctly
    }
}
```

---

## Implementation Order

```
Phase 1: Dependencies & Types (30 min)
├── Task 1: Update Cargo.toml with missing web-sys features
└── Task 2: Add CanvasRenderingContext2d to Renderer struct

Phase 2: Renderer Implementation (1-2 hours)
├── Task 3: Implement set_context() method
├── Task 4: Implement draw_rect() with Path2d
├── Task 5: Implement draw_ellipse()
└── Task 6: Implement draw_text()

Phase 3: WASM Bindings (1 hour)
├── Task 7: Update ArchFlowWasm constructor
├── Task 8: Add public add_rect(), add_text() methods
└── Task 9: Implement clear() and render() calls

Phase 4: HTML Demo Integration (30 min)
├── Task 10: Update index.html to import WASM
├── Task 11: Replace JS rendering with WASM calls
└── Task 12: Verify demo works

Phase 5: Build & Verify (30 min)
├── Task 13: Build with wasm-pack
├── Task 14: Test in browser
└── Task 15: Add unit tests
```

**Estimated Total Time**: 4-5 hours

---

## Success Criteria

- [ ] `crates/demo/index.html` calls `ArchFlowWasm` methods
- [ ] Shapes are rendered via Rust/WASM, not JavaScript
- [ ] No JavaScript simulation code in render loop
- [ ] 100% of existing tests still pass
- [ ] New tests for WASM rendering pass
- [ ] Demo runs at 60 FPS

---

## Future Enhancements (Post-MVP)

1. **WebGL Rendering**: Replace Canvas 2D with WebGL for GPU acceleration
2. **SharedArrayBuffer**: Use zero-copy memory sharing between JS and Rust
3. **Event Handling**: Pass mouse/keyboard events to Rust for processing
4. **Entity Management**: Move shape storage from JS to Rust ECS
5. **Text Rendering**: Integrate cosmic-text for proper text shaping
