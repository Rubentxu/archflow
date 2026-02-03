# archflow-render

> **High-Performance GPU Rendering** - WebGPU-based 2D rendering with multi-phase instancing, infinite canvas, and advanced texture management.

## Overview

`archflow-render` provides a high-performance WebGPU-based rendering system for ArchFlow architecture diagrams. It features multi-phase instanced rendering for optimal GPU utilization, an infinite 2D canvas with professional zoom-to-cursor navigation, and efficient texture atlas management.

**Key Capabilities:**
- **Multi-phase rendering** - Specialized pipelines for shapes, icons, images, and text
- **Instanced rendering** - Single draw call for thousands of entities
- **Infinite canvas** - Unlimited pan/zoom with viewport culling
- **Texture atlases** - Efficient packing for icons and glyphs
- **WebGPU** - Modern GPU API for cross-platform performance

## Architecture

The crate follows a **GPU-Centric Architecture** with specialized render phases:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Application Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │EntityStore   │  │CameraControl │  │RenderConfig  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                    GpuRenderer Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │Instance Sync │  │Viewport Cull│  │Batch Sorting │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                    Render Phases                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Shapes (SDF) │  │Icons (Atlas) │  │Text (MTSDF)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      WebGPU Layer                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │RenderPipelines│  │GpuResources │  │Shaders (WGSL)│          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Render Phases

The renderer uses four specialized render phases instead of a single pipeline:

```rust
pub enum RenderPhase {
    Shapes = 0,   // SDF-based shape rendering
    Icons = 1,    // Texture atlas lookup
    Images = 2,   // Texture2D array
    Text = 3,     // MTSDF text rendering
}
```

**Why Multi-Phase?**

| Approach | Draw Calls | SIMD Divergence | Cache Efficiency |
|----------|-----------|-----------------|------------------|
| Single Pipeline | 1 | High | Low |
| Multi-Phase | 4 | None | High |

Each phase is optimized for its content type:
- **Shapes**: SDF anti-aliasing for smooth edges
- **Icons**: Texture atlas with bilinear filtering
- **Images**: Texture2D array for multiple images
- **Text**: MTSDF for crisp text at any size

### GPU Instance Data

Entities are rendered as GPU-friendly instances:

```rust
#[repr(C, align(16))]
pub struct GpuInstance {
    pos: [f32; 2],                    // World position [x, y]
    size: [f32; 2],                   // World size [width, height]
    color: u32,                       // Packed RGBA color
    shape_type_or_texture_index: u32,  // Shape type or texture layer
    _padding: [u32; 2],               // 16-byte alignment
    uv_rect: [f32; 4],                // Texture UV [x, y, w, h]
}
```

**Memory Layout:**
- **Size**: 48 bytes per instance
- **Max Instances**: 100,000 entities = ~4.8MB
- **Alignment**: 16-byte for GPU efficiency

### Camera System

Professional infinite 2D camera with zoom-to-cursor:

```rust
use archflow_render::Camera;

let mut camera = Camera::new(1920.0, 1080.0);

// Set viewport size
camera.set_viewport_size(1920.0, 1080.0);

// Zoom to cursor (professional pattern)
camera.zoom_to_cursor(1.1, mouse_pos);  // 10% zoom in
camera.zoom_to_cursor(0.9, mouse_pos);  // 10% zoom out

// Pan with delta
camera.pan_with_delta(delta_x, delta_y);

// Get view-projection matrix
let matrix = camera.build_view_projection_matrix();

// Coordinate conversion
let world_pos = camera.screen_to_world(screen_pos, screen_size);
let screen_pos = camera.world_to_screen(world_pos, screen_size);
```

**Zoom Range:** 0.01x to 100x (10,000:1 ratio)

**Professional vs Amateur Zoom:**

```
Amateur: Canvas center zoom         Professional: Cursor position zoom
┌──────────────┐                    ┌──────────────┐
│      [*]     │                    │      [*]     │
│   Zoom here  │                    │   Zoom here  │
└──────────────┘                    └──────────────┘
```

### Texture Atlas Packing

Efficient shelf-packing for texture atlases:

```rust
use archflow_render::AtlasPacker;

let mut packer = AtlasPacker::new(2048, 2048);

// Allocate textures
let rect1 = packer.allocate(64, 64);
let rect2 = packer.allocate(32, 32);
let rect3 = packer.allocate(128, 128);

// Check efficiency
println!("Utilization: {:.1}%", packer.utilization() * 100.0);
println!("Shelves: {}", packer.shelves.len());
println!("Free: {} pixels²", packer.free_area());
```

**Algorithm:** Horizontal shelf packing
- **Complexity:** O(shelves) per insertion
- **Reorganization:** Never (stable allocation)
- **Best for:** Uniform texture sizes

## Usage Examples

### Basic Rendering Setup

```rust
use archflow_render::{GpuRenderer, Camera, RenderPhase};

// Initialize renderer
let mut renderer = GpuRenderer::new();
let mut camera = Camera::new(1920.0, 1080.0);

// Sync from EntityStore
let visible_count = renderer.sync_from_store(&entity_store, &camera);

// Get batch information
let shape_count = renderer.batch_count(RenderPhase::Shapes);
let icon_count = renderer.batch_count(RenderPhase::Icons);
let text_count = renderer.batch_count(RenderPhase::Text);

// Render
renderer.render(&camera);
```

### WebGPU Context

```rust
use archflow_render::{WebGpuContext, GpuResources};

// Create WebGPU context
let context = WebGpuContext::new()?;
context.set_surface(surface);
context.configure_surface(1920, 1080);

// Create GPU resources
let resources = GpuResources::new(&context)?;

// Upload data
resources.write_instances(&context.queue, &instances)?;
resources.write_uniforms(&context.queue, &camera_uniforms)?;
```

### Complete Render Loop

```rust
fn render_loop(
    renderer: &mut GpuRenderer,
    camera: &Camera,
    entity_store: &EntityStore,
) {
    // Sync visible entities
    renderer.sync_from_store(entity_store, camera);

    // Clear viewport
    renderer.clear();

    // Render all phases
    renderer.render(camera);

    // Present
    renderer.present();
}
```

### Camera Controls

```rust
// Mouse wheel zoom
if let Some(scroll) = input.scroll_delta() {
    let zoom_factor = 1.0 + scroll.y * 0.001;
    camera.zoom_to_cursor(zoom_factor, input.mouse_pos());
}

// Middle-click pan
if input.mouse_down(MouseButton::Middle) {
    let delta = input.mouse_delta();
    camera.pan_with_delta(delta.x, delta.y);
}

// Keyboard zoom
if input.key_down(KeyCode::Equals) {  // Plus key
    camera.zoom_to_cursor(1.1, camera.center());
}
if input.key_down(KeyCode::Minus) {
    camera.zoom_to_cursor(0.9, camera.center());
}
```

### Atlas Management

```rust
use archflow_render::AtlasPacker;

// Create atlas for icons
let mut icon_atlas = AtlasPacker::new(2048, 2048);
icon_atlas.padding = 2;

// Pack icons
let mut icon_uvs = Vec::new();
for icon in &icons {
    if let Some(rect) = icon_atlas.allocate(icon.width, icon.height) {
        let uv = rect.to_uv_coords(2048.0, 2048.0);
        icon_uvs.push((icon.id.clone(), uv));
    }
}

// Upload to GPU
let atlas_texture = create_gpu_texture(&icon_atlas);
```

## Performance Characteristics

### Frame Breakdown

| Stage | Time | Notes |
|-------|------|-------|
| Entity Sync | ~2ms | 100k entities CPU → GPU |
| Culling | ~0.5ms | Viewport-based |
| GPU Upload | ~1ms | write_buffer |
| Draw Calls | ~0.5ms | 4 draw calls |
| Total | ~4ms | 250 FPS potential |

### Memory Usage

| Resource | Size | Entities |
|----------|------|----------|
| Instance Buffer | 4.8MB | 100k entities |
| Uniform Buffer | 64 bytes | Camera matrix |
| Shape Atlas | 16MB | 2048×2048 RGBA |
| Icon Atlas | 16MB | 2048×2048 RGBA |
| Text Atlas | 16MB | 2048×2048 RGBA |

### Culling Effectiveness

| Scene Size | Without Culling | With Culling | Reduction |
|------------|-----------------|--------------|-----------|
| Small (1k) | 1,000 | 1,000 | 0% |
| Medium (10k) | 10,000 | ~2,000 | 80% |
| Large (100k) | 100,000 | ~10,000 | 90% |

## Integration with Other Crates

```toml
[dependencies]
archflow-render = "0.36"
archflow-engine = "0.36"   # EntityStore data source
archflow-core = "0.36"     # Core math types
archflow-plugins = "0.36"  # Texture atlas integration
```

### Data Flow

```
EntityStore → sync_from_store() → GpuInstance[]
                                          │
                                          ▼
                                    Instance Buffer
                                          │
                                          ▼
                                    Viewport Cull
                                          │
                                          ▼
                                    Batch by Phase
                                          │
                                          ▼
                            ┌─────────────┴─────────────┐
                            ▼                           ▼
                      Shape Phase               Icon Phase
                            │                           │
                            └─────────────┬─────────────┘
                                          ▼
                                    GPU Rendering
```

## Shaders

### Shapes (SDF-based)

```wgsl
// Signed distance field for smooth anti-aliasing
fn sdf_rect(p: vec2f, size: vec2f) -> f32 {
    d = max(abs(p) - size, vec2f(0.0));
    return length(d);
}

// Anti-aliased edge
fn aa_edge(distance: f32) -> f32 {
    return 1.0 - smoothstep(-0.5, 0.5, distance);
}
```

### Icons (Texture Atlas)

```wgsl
// Atlas texture lookup
let uv = instance.uv_rect;
let tex_coord = vec2f(
    uv.x + uv.z * local_coord.x,
    uv.y + uv.w * local_coord.y
);
let color = textureSample(icon_atlas, icon_sampler, tex_coord);
```

### Text (MTSDF)

```wgsl
// Multi-channel signed distance field
fn msdf_median(sample: vec3f) -> f32 {
    return max(min(sample.r, sample.g), min(max(sample.r, sample.g), sample.b));
}
```

## Constraints and Limitations

### Current Constraints

- **WebGPU Required**: Modern browser/GPU support needed
- **Entity Limit**: 100,000 entities per frame
- **Texture Size**: Power of 2 recommended
- **Memory Alignment**: 16-byte alignment required

### Platform Considerations

- **Chrome/Edge**: Full WebGPU support
- **Firefox**: WebGPU behind flag (as of 2024)
- **Safari**: Experimental WebGPU support
- **Desktop**: Native wgpu works on all platforms

## Best Practices

### Camera Handling

1. **Use zoom-to-cursor** for professional UX
2. **Clamp zoom levels** to prevent disorientation
3. **Maintain aspect ratio** when resizing viewport
4. **Smooth animations** for camera transitions

### Performance Optimization

1. **Enable culling** for large diagrams
2. **Sort entities** by render phase
3. **Batch updates** to minimize GPU sync
4. **Reuse buffers** instead of reallocating

### Texture Management

1. **Pack similar sizes** together for efficiency
2. **Use power-of-2** dimensions
3. **Add padding** to prevent bleeding
4. **Monitor utilization** and resize when needed

## Future Enhancements

### Planned Features

- **Compute Shaders**: Frustum culling on GPU
- **Texture Compression**: BC7/ETC2 support
- **Mipmaps**: Automatic mipmap generation
- **Bindless Textures**: Unlimited texture slots
- **Mesh Shaders**: Procedural geometry

### Performance Targets

- **1M entities**: Sub-16ms frame time
- **4K resolution**: 60 FPS sustained
- **Mobile devices**: 30 FPS minimum

## References

- **WebGPU Specification**: https://www.w3.org/TR/webgpu/
- **WGSL Language**: https://www.w3.org/TR/WGSL/
- **SDF Rendering**: https://github.com/mmalex/SDFMath
- **EPIC-WEB-010**: Canvas rendering system
- **archflow-engine**: EntityStore integration

## License

MIT License - See LICENSE file for details.
