# ArchFlow: Arquitectura Final y Plan de Implementación MVP

**Versión:** 2.0  
**Fecha:** 2026-01-23  
**Estado:** Especificación Técnica Final  
**Base:** `docs/prd.md`, `docs/PRD-CRITICA.md`, `docs/WGPU-VS-GLOW.md`

---

## 1. Decisión Arquitectónica Final

### 1.1 Stack Tecnológico Definitivo

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ARCHFLOW TECH STACK                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    LEPTOS (Frontend UI)                             │    │
│  │  - Framework reactivo en Rust                                       │    │
│  │  - Compila a WASM                                                   │    │
│  │  - Signals para estado reactivo                                     │    │
│  │  - CSR + SSR opcional                                               │    │
│  │  - Bundle: ~500KB (UI only)                                        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                  WGPU (Rendering Engine)                            │    │
│  │  - API gráfica segura multiplataforma                               │    │
│  │  - Backends: WebGPU/WebGL2/Vulkan/Metal/D3D12                      │    │
│  │  - WGSL shaders (type-safe)                                         │    │
│  │  - Instanced rendering para 10k+ nodos                              │    │
│  │  - Bundle: ~2.8 MB (con WebGL2 backend)                            │    │
│  │                                                                     │    │
│  │  Feature Flags:                                                     │    │
│  │  - default: WebGL2 (compatibilidad máxima)                         │    │
│  │  - webgpu: WebGPU (cuando esté disponible)                         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    │                                         │
│                                    ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                  CORE DOMAIN (Rust)                                 │    │
│  │  - DDD con aggregates                                               │    │
│  │  - Event sourcing foundation                                        │    │
│  │  - AUF format (YAML storage)                                        │    │
│  │  - Component registry (AWS, Azure, GCP)                            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                  STORAGE (IndexedDB)                                │    │
│  │  - rust-indexeddb para persistencia local                          │    │
│  │  - AUF file export/import                                           │    │
│  │  - Undo/redo history                                                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Métricas de Rendimiento Objetivo

| Métrica | Objetivo | Implementación |
|---------|----------|----------------|
| **Carga inicial** | < 5 segundos | Code splitting, lazy loading |
| **Canvas FPS** | 60 fps constante | wgpu instanced rendering |
| **Nodos soportados** | 10,000+ | Instancing + level of detail |
| **Memoria** | < 200 MB | wgpu memory management |
| **Bundle size** | < 5 MB | Feature flags, tree shaking |
| **Latencia UI** | < 16ms | Leptos signals, no React-like re-renders |

---

## 2. Estructura del Proyecto

### 2.1 Workspace Final

```
archflow/
├── Cargo.toml                          # Workspace root
├── Cargo.lock
│
├── packages/
│   ├── core/                           # Domain layer (DDD)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── architecture/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── architecture.rs
│   │   │   │   ├── layer.rs
│   │   │   │   └── policy.rs
│   │   │   ├── component/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── component.rs
│   │   │   │   ├── registry.rs    # 10 tipos AWS
│   │   │   │   └── types.rs
│   │   │   ├── events/
│   │   │   │   ├── mod.rs
│   │   │   │   └── architecture_events.rs
│   │   │   ├── value_objects/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── position.rs
│   │   │   │   ├── version.rs
│   │   │   │   └── component_id.rs
│   │   │   ├── errors/
│   │   │   │   └── mod.rs
│   │   │   └── auformat/
│   │   │       ├── mod.rs
│   │   │       ├── schema.rs     # JSON Schema
│   │   │       └── serializer.rs
│   │   └── tests/
│   │
│   ├── canvas/                         # WGPU Rendering Engine
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── renderer/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── wgpu_renderer.rs
│   │   │   │   ├── pipeline.rs
│   │   │   │   └── shaders/
│   │   │   │       ├── mod.rs
│   │   │   │       ├── component.wgsl
│   │   │   │       ├── connection.wgsl
│   │   │   │       └── grid.wgsl
│   │   │   ├── state/
│   │   │   │   ├── mod.rs
│   │   │   │   └── canvas_state.rs
│   │   │   ├── views/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── view_2d.rs
│   │   │   │   └── view_types.rs
│   │   │   ├── animation/
│   │   │   │   ├── mod.rs
│   │   │   │   └── animation_system.rs
│   │   │   ├── input/
│   │   │   │   ├── mod.rs
│   │   │   │   └── event_handler.rs
│   │   │   └── geometry/
│   │   │       ├── mod.rs
│   │   │       └── shapes.rs
│   │   └── tests/
│   │
│   ├── storage/                        # Persistence Layer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── indexeddb/
│   │   │   │   ├── mod.rs
│   │   │   │   └── repository.rs
│   │   │   ├── file/
│   │   │   │   ├── mod.rs
│   │   │   │   └── aufile_io.rs
│   │   │   └── history/
│   │   │       └── undo_redo.rs
│   │   └── tests/
│   │
│   ├── export/                         # IaC Exporters
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── terraform/
│   │   │   │   ├── mod.rs
│   │   │   │   └── hcl_generator.rs
│   │   │   └── kubernetes/
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   ├── app/                            # Leptos WASM Application
│   │   ├── Cargo.toml
│   │   ├── index.html
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── app.rs
│   │   │   ├── components/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── canvas/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── canvas_view.rs
│   │   │   │   │   └── mini_map.rs
│   │   │   │   ├── palette/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── component_item.rs
│   │   │   │   ├── properties/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── property_field.rs
│   │   │   │   ├── toolbar/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── tools.rs
│   │   │   │   ├── layers/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── layer_item.rs
│   │   │   │   └── common/
│   │   │   │       ├── modal.rs
│   │   │   │       └── toast.rs
│   │   │   ├── pages/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── home.rs
│   │   │   │   └── editor.rs
│   │   │   ├── state/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── store.rs
│   │   │   │   └── signals.rs
│   │   │   ├── services/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── export_service.rs
│   │   │   │   └── storage_service.rs
│   │   │   ├── router/
│   │   │   │   └── mod.rs
│   │   │   └── styles/
│   │   │       └── main.css
│   │   ├── assets/
│   │   │   ├── fonts/
│   │   │   └── icons/
│   │   └── tests/
│   │
│   └── shared/                         # Shared Utilities
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config/
│       │   │   └── mod.rs
│       │   └── telemetry/
│       │       └── mod.rs
│       └── tests/
│
├── tools/
│   └── cli/                            # CLI Tool (futuro)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── api/                                # Serverless Functions (futuro)
│   └── src/
│       └── main.rs
│
├── .github/workflows/
│   └── ci.yml
│
├── Cargo.toml                          # Workspace config
├── rust-toolchain.toml
├── README.md
├── LICENSE
└── .gitignore
```

### 2.2 Dependencias del Workspace

```toml
# Cargo.toml (workspace root)

[workspace]
members = [
    "packages/core",
    "packages/canvas",
    "packages/storage",
    "packages/export",
    "packages/app",
    "packages/shared",
    "tools/cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[workspace.dependencies]
# Core
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
thiserror = "2.0"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }
bytemuck = { version = "1.14", features = ["derive"] }
regex = "1.10"

# Leptos
leptos = { version = "0.6", features = ["wasm-bind"] }
leptos_meta = "0.6"
leptos_router = "0.6"

# WGPU
wgpu = { version = "0.18", features = ["spirv", "glsl"] }
naga = { version = "0.13", features = ["wgsl-in", "glsl-in", "spv-in"] }
wgpu-text = "0.2"
lyon = "1.0"

# WASM
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Document",
    "Element",
    "HtmlElement",
    "HtmlCanvasElement",
    "WebGl2RenderingContext",
    "Performance",
    "Storage",
    "EventTarget",
    "KeyboardEvent",
    "MouseEvent",
    "WheelEvent",
] }

# Storage
indexeddb = "0.5"
gloo-storage = "0.2"

# Tracing
tracing = "0.1"
tracing-wasm = "0.2"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Testing
rstest = "0.18"
proptest = "1.4"
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
debug = true
opt-level = 0

[patch.crates-io]
wgpu = { git = "https://github.com/gfx-rs/wgpu", branch = "master" }
```

---

## 3. WGPU Renderer Architecture

### 3.1 Renderer Structure

```rust
// packages/canvas/src/renderer/wgpu_renderer.rs

use wgpu::*;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder};
use lyon::tessellation;
use std::sync::Arc;
use crate::state::CanvasState;
use crate::animation::AnimationFrame;

/// WGPU Renderer para el canvas de ArchFlow
pub struct WgpuRenderer {
    // WGPU Core
    instance: Instance,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface_config: SurfaceConfiguration,
    
    // Pipelines
    component_pipeline: RenderPipeline,
    connection_pipeline: RenderPipeline,
    grid_pipeline: RenderPipeline,
    selection_pipeline: RenderPipeline,
    text_pipeline: GlyphBrush<'static>,
    
    // Buffers
    component_vertex_buffer: Buffer,
    component_instance_buffer: Buffer,
    connection_vertex_buffer: Buffer,
    grid_vertex_buffer: Buffer,
    
    // Texture Atlas
    texture_atlas: TextureAtlas,
    
    // State
    width: u32,
    height: u32,
    scale_factor: f64,
}

impl WgpuRenderer {
    /// Inicializa el renderer con un canvas HTML
    pub async fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, RendererError> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        let surface = instance.create_surface_from_canvas(canvas)?;
        
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await
            .ok_or(RendererError::NoAdapter)?;
        
        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: Some("ArchFlow Device"),
            features: Features::empty(),
            limits: Limits::downlevel_webgl2_defaults()
                .using_resolution(1280, 720),
        }, None).await?;
        
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        
        let width = canvas.width();
        let height = canvas.height();
        let scale_factor = canvas.owner_document()
            .and_then(|d| d.window())
            .map(|w| w.device_pixel_ratio())
            .unwrap_or(1.0);
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &surface_config);
        
        // Create pipelines
        let component_pipeline = Self::create_component_pipeline(
            &device, 
            surface_format,
            include_str!("shaders/component.wgsl")
        )?;
        
        let connection_pipeline = Self::create_connection_pipeline(
            &device,
            surface_format,
            include_str!("shaders/connection.wgsl")
        )?;
        
        let grid_pipeline = Self::create_grid_pipeline(
            &device,
            surface_format,
            include_str!("shaders/grid.wgsl")
        )?;
        
        let selection_pipeline = Self::create_selection_pipeline(
            &device,
            surface_format,
            include_str!("shaders/selection.wgsl")
        )?;
        
        // Create buffers
        let component_vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Component Vertices"),
            size: 64 * 1024, // 64KB for vertex data
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let component_instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Component Instances"),
            size: 64 * 1024 * 1024, // 64MB for 10k instances
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Initialize text renderer
        let text_pipeline = GlyphBrushBuilder::using_fonts_bytes(vec![
            include_bytes!("../../assets/fonts/inter-regular.ttf"),
        ]).build(&device, surface_format);
        
        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_config,
            component_pipeline,
            connection_pipeline,
            grid_pipeline,
            selection_pipeline,
            text_pipeline,
            component_vertex_buffer,
            component_instance_buffer,
            connection_vertex_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Connection Vertices"),
                size: 128 * 1024,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            grid_vertex_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Grid Vertices"),
                size: 256 * 1024,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            texture_atlas: TextureAtlas::new(&device, &queue)?,
            width,
            height,
            scale_factor,
        })
    }
    
    /// Renderiza el estado del canvas
    pub fn render(&mut self, state: &CanvasState, animations: &[AnimationFrame]) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                tracing::error!("Failed to acquire frame: {:?}", e);
                return;
            }
        };
        
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ArchFlow Encoder"),
        });
        
        // 1. Renderizar grid
        if state.show_grid {
            self.render_grid(state, &mut encoder, &view);
        }
        
        // 2. Renderizar conexiones
        self.render_connections(state, &mut encoder, &view);
        
        // 3. Renderizar componentes (instanced)
        self.render_components(state, &mut encoder, &view);
        
        // 4. Renderizar selección
        self.render_selection(state, &mut encoder, &view);
        
        // 5. Renderizar animaciones
        self.render_animations(animations, &mut encoder, &view);
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
    
    fn render_grid(&self, state: &CanvasState, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Grid Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        
        render_pass.set_pipeline(&self.grid_pipeline);
        render_pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
        render_pass.draw(0..self.grid_vertex_count, 0..1);
    }
    
    fn render_components(&self, state: &CanvasState, encoder: &mut CommandEncoder, view: &TextureView) {
        // Update instance buffer with component data
        let instance_data: Vec<InstanceData> = state.components.values()
            .map(|c| InstanceData {
                transform: Self::calculate_transform(c, state),
                color: Self::component_color(c),
            })
            .collect();
        
        self.queue.write_buffer(
            &self.component_instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data)
        );
        
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Components Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        
        render_pass.set_pipeline(&self.component_pipeline);
        render_pass.set_vertex_buffer(0, self.component_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.component_instance_buffer.slice(..));
        render_pass.draw(0..6, 0..instance_data.len() as u32);
    }
    
    fn create_component_pipeline(
        device: &Device,
        format: TextureFormat,
        shader_code: &str,
    ) -> Result<RenderPipeline, RendererError> {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Component Shader"),
            source: ShaderSource::Wgsl(shader_code.into()),
            compilation_options: Default::default(),
        });
        
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Component Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        Ok(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Component Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    VertexBufferLayout {
                        array_stride: size_of::<InstanceData>() as BufferAddress,
                        step_mode: VertexStepMode::Instance,
                        attributes: &vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4],
                    },
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        }))
    }
    
    fn calculate_transform(component: &ComponentRenderData, state: &CanvasState) -> [[f32; 4]; 4] {
        let scale_x = component.size.width;
        let scale_y = component.size.height;
        let trans_x = component.position.x;
        let trans_y = component.position.y;
        let scale = state.transform.scale;
        
        [
            [scale as f32, 0.0, 0.0, 0.0],
            [0.0, scale as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [trans_x as f32, trans_y as f32, 0.0, 1.0],
        ]
    }
    
    fn component_color(component: &ComponentRenderData) -> [f32; 4] {
        let c = component.color;
        [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, 1.0]
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct InstanceData {
    transform: [[f32; 4]; 4],
    color: [f32; 4],
}

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("No suitable GPU adapter found")]
    NoAdapter,
    #[error("Failed to create surface: {0}")]
    SurfaceError(String),
    #[error("Shader compilation failed: {0}")]
    ShaderError(String),
}
```

### 3.2 WGSL Shaders

```wgsl
// packages/canvas/src/renderer/shaders/component.wgsl

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) transform: mat4x4<f32>,
    @location(6) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(
    input: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = instance.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.position = world_pos;
    output.color = instance.color;
    output.uv = input.uv;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple rounded rectangle
    let uv = input.uv - 0.5;
    let dist = length(uv);
    
    if dist > 0.45 {
        // Rounded corner effect
        let alpha = 1.0 - smoothstep(0.45, 0.5, dist);
        return vec4<f32>(input.color.rgb, input.color.a * alpha);
    }
    
    // Add subtle gradient
    let gradient = 0.9 + 0.1 * (input.uv.x + input.uv.y);
    return vec4<f32>(input.color.rgb * gradient, input.color.a);
}
```

```wgsl
// packages/canvas/src/renderer/shaders/connection.wgsl

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) t: f32,  // Position along curve (0-1)
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) t: f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = vec4<f32>(0.5, 0.5, 0.5, 1.0);  // Gray by default
    output.t = input.t;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Animated dash effect
    let dash = sin(input.t * 50.0 + f32(global_time)) * 0.5 + 0.5;
    let color = mix(vec4<f32>(0.5, 0.5, 0.5, 1.0), vec4<f32>(0.3, 0.6, 0.9, 1.0), dash);
    return color;
}

@group(0) @binding(0) var<uniform> global_time: f32;
```

---

## 4. Canvas State con DDD

```rust
// packages/canvas/src/state/canvas_state.rs

use leptos::*;
use std::collections::{HashMap, HashSet};
use archflow_core::component::*;
use crate::geometry::*;

/// Estado completo del canvas
#[derive(Clone, Debug)]
pub struct CanvasState {
    /// Componentes en el canvas
    pub components: HashMap<ComponentId, ComponentRenderData>,
    
    /// Conexiones entre componentes
    pub relationships: Vec<RelationshipRenderData>,
    
    /// Selección actual
    pub selected: HashSet<ComponentId>,
    
    /// Transformación de vista
    pub transform: ViewTransform,
    
    /// Componente bajo el cursor
    pub hovered: Option<ComponentId>,
    
    /// Estado de arrastre
    pub drag: Option<DragState>,
    
    /// Caja de selección (multi-select)
    pub selection_box: Option<SelectionBox>,
    
    /// Herramienta activa
    pub active_tool: Tool,
    
    /// Configuración visual
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub grid_size: f64,
    
    /// Capas
    pub layer_system: LayerSystem,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            components: HashMap::new(),
            relationships: Vec::new(),
            selected: HashSet::new(),
            transform: ViewTransform::default(),
            hovered: None,
            drag: None,
            selection_box: None,
            active_tool: Tool::Select,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            layer_system: LayerSystem::default(),
        }
    }
}

impl CanvasState {
    /// Convierte coordenadas de pantalla a coordenadas del mundo
    pub fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> Position {
        Position::new(
            (screen_x - self.transform.x) / self.transform.scale,
            (screen_y - self.transform.y) / self.transform.scale,
        )
    }
    
    /// Convierte coordenadas del mundo a coordenadas de pantalla
    pub fn world_to_screen(&self, world_x: f64, world_y: f64) -> (f64, f64) {
        (
            world_x * self.transform.scale + self.transform.x,
            world_y * self.transform.scale + self.transform.y,
        )
    }
    
    /// Test de colisión - retorna el componente bajo el cursor
    pub fn hit_test(&self, screen_x: f64, screen_y: f64) -> Option<ComponentId> {
        let world_pos = self.screen_to_world(screen_x, screen_y);
        
        for (_, component) in self.components.iter().rev() {
            let bounds = component.bounding_box();
            if bounds.contains(&world_pos) {
                return Some(component.id);
            }
        }
        None
    }
    
    /// Añade un componente al canvas
    pub fn add_component(&mut self, component: Component) {
        self.components.insert(
            *component.id(),
            ComponentRenderData::from_component(&component),
        );
    }
    
    /// Mueve un componente
    pub fn move_component(&mut self, id: &ComponentId, new_position: Position) {
        let position = if self.snap_to_grid {
            Position::new(
                (new_position.x / self.grid_size).round() * self.grid_size,
                (new_position.y / self.grid_size).round() * self.grid_size,
            )
        } else {
            new_position
        };
        
        if let Some(component) = self.components.get_mut(id) {
            component.position = position;
        }
    }
    
    /// Zoom centrado en un punto
    pub fn zoom_at(&mut self, mouse_x: f64, mouse_y: f64, delta: f64) {
        let zoom_factor = 1.0 + delta;
        let new_scale = (self.transform.scale * zoom_factor).clamp(0.1, 5.0);
        
        let world_before = self.screen_to_world(mouse_x, mouse_y);
        self.transform.scale = new_scale;
        let world_after = self.screen_to_world(mouse_x, mouse_y);
        
        // Adjust pan to maintain mouse position
        self.transform.x += (world_after.x - world_before.x) * new_scale;
        self.transform.y += (world_after.y - world_before.y) * new_scale;
    }
    
    /// Selecciona un componente
    pub fn select(&mut self, id: Option<ComponentId>, add_to_selection: bool) {
        if !add_to_selection {
            self.selected.clear();
        }
        if let Some(id) = id {
            self.selected.insert(id);
        }
    }
    
    /// Selecciona todos los componentes en el rectángulo
    pub fn select_in_rect(&mut self, rect: &SelectionBox) {
        let left = rect.x.min(rect.x + rect.width);
        let right = rect.x.max(rect.x + rect.width);
        let top = rect.y.min(rect.y + rect.height);
        let bottom = rect.y.max(rect.y + rect.height);
        
        for (_, component) in self.components.iter() {
            let bounds = component.bounding_box();
            if bounds.intersects(left, right, top, bottom) {
                self.selected.insert(component.id);
            }
        }
    }
}

/// Datos de renderizado de un componente
#[derive(Debug, Clone)]
pub struct ComponentRenderData {
    pub id: ComponentId,
    pub component_type: ComponentType,
    pub position: Position,
    pub size: Size,
    pub label: String,
    pub color: Color,
    pub z_index: u32,
    pub locked: bool,
    pub layer_id: LayerId,
}

impl ComponentRenderData {
    pub fn from_component(component: &Component) -> Self {
        let (icon, color) = Self::component_type_to_style(component.component_type());
        
        Self {
            id: *component.id(),
            component_type: component.component_type().clone(),
            position: *component.position(),
            size: *component.size(),
            label: component.name().to_string(),
            color,
            z_index: component.position().z as u32,
            locked: false,
            layer_id: component.layer_id(),
        }
    }
    
    fn component_type_to_style(component_type: &ComponentType) -> (&'static str, Color) {
        match component_type.category() {
            ComponentCategory::Compute => ("server", Color::blue()),
            ComponentCategory::Storage => ("database", Color::green()),
            ComponentCategory::Network => ("network", Color::orange()),
            ComponentCategory::Security => ("shield", Color::red()),
            ComponentCategory::Custom => ("custom", Color::gray()),
        }
    }
    
    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::from_center_size(
            self.position,
            self.size,
        )
    }
}

/// Transformación de vista
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewTransform {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

/// Herramienta activa
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Select,
    Pan,
    Connection,
    Zoom,
    Text,
}
```

---

## 5. Component Registry (AWS)

```rust
// packages/core/src/component/registry.rs

use super::*;
use std::collections::HashMap;

/// Registro de componentes disponibles (10 tipos AWS)
pub struct ComponentRegistry {
    definitions: HashMap<ComponentType, ComponentDefinition>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
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
        // EC2 Instance
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
                        input_type: PropertyInputType::Select,
                        required: true,
                        default: Some(json!("t3.micro")),
                        options: Some(vec![
                            PropertyOption { label: "t3.micro".to_string(), value: json!("t3.micro") },
                            PropertyOption { label: "t3.small".to_string(), value: json!("t3.small") },
                            PropertyOption { label: "t3.medium".to_string(), value: json!("t3.medium") },
                            PropertyOption { label: "t3.large".to_string(), value: json!("t3.large") },
                            PropertyOption { label: "m5.large".to_string(), value: json!("m5.large") },
                            PropertyOption { label: "c5.xlarge".to_string(), value: json!("c5.xlarge") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "ami".to_string(),
                        name: "AMI ID".to_string(),
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: Some(json!("ami-0c55b159cbfafe1f0")),
                        options: None,
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "monitoring".to_string(),
                        name: "Detailed Monitoring".to_string(),
                        input_type: PropertyInputType::Boolean,
                        required: false,
                        default: Some(json!(false)),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // Lambda Function
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
                        input_type: PropertyInputType::Select,
                        required: true,
                        default: Some(json!("python3.11")),
                        options: Some(vec![
                            PropertyOption { label: "Python 3.11".to_string(), value: json!("python3.11") },
                            PropertyOption { label: "Node.js 20.x".to_string(), value: json!("nodejs20.x") },
                            PropertyOption { label: "Java 17".to_string(), value: json!("java17") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "timeout".to_string(),
                        name: "Timeout (seconds)".to_string(),
                        input_type: PropertyInputType::Number,
                        required: false,
                        default: Some(json!(30)),
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
                        input_type: PropertyInputType::Number,
                        required: false,
                        default: Some(json!(256)),
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
        
        // S3 Bucket
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
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: None,
                        options: None,
                        validation: Some(PropertyValidation {
                            pattern: Some(r"^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$".to_string()),
                            min: None,
                            max: Some(63.0),
                        }),
                    },
                    PropertyDefinition {
                        key: "versioning".to_string(),
                        name: "Versioning".to_string(),
                        input_type: PropertyInputType::Boolean,
                        required: false,
                        default: Some(json!(false)),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // RDS Instance
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
                        input_type: PropertyInputType::Select,
                        required: true,
                        default: Some(json!("postgres")),
                        options: Some(vec![
                            PropertyOption { label: "PostgreSQL".to_string(), value: json!("postgres") },
                            PropertyOption { label: "MySQL".to_string(), value: json!("mysql") },
                            PropertyOption { label: "Aurora PostgreSQL".to_string(), value: json!("aurora-postgresql") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "instance_class".to_string(),
                        name: "Instance Class".to_string(),
                        input_type: PropertyInputType::Select,
                        required: true,
                        default: Some(json!("db.t3.micro")),
                        options: Some(vec![
                            PropertyOption { label: "db.t3.micro".to_string(), value: json!("db.t3.micro") },
                            PropertyOption { label: "db.t3.small".to_string(), value: json!("db.t3.small") },
                            PropertyOption { label: "db.t3.medium".to_string(), value: json!("db.t3.medium") },
                        ]),
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "multi_az".to_string(),
                        name: "Multi-AZ".to_string(),
                        input_type: PropertyInputType::Boolean,
                        required: false,
                        default: Some(json!(false)),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // VPC
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
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: Some(json!("10.0.0.0/16")),
                        options: None,
                        validation: Some(PropertyValidation {
                            pattern: Some(r"^([0-9]{1,3}\.){3}[0-9]{1,3}/[0-9]{1,2}$".to_string()),
                            min: None,
                            max: None,
                        }),
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // Load Balancer
        self.definitions.insert(
            ComponentType::LoadBalancer,
            ComponentDefinition {
                type_: ComponentType::LoadBalancer,
                name: "Load Balancer".to_string(),
                category: ComponentCategory::Network,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "load-balancer",
                default_size: Size::new(100.0, 40.0),
                properties: vec![
                    PropertyDefinition {
                        key: "scheme".to_string(),
                        name: "Scheme".to_string(),
                        input_type: PropertyInputType::Select,
                        required: true,
                        default: Some(json!("internet-facing")),
                        options: Some(vec![
                            PropertyOption { label: "Internet-facing".to_string(), value: json!("internet-facing") },
                            PropertyOption { label: "Internal".to_string(), value: json!("internal") },
                        ]),
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // IAM Role
        self.definitions.insert(
            ComponentType::IamRole,
            ComponentDefinition {
                type_: ComponentType::IamRole,
                name: "IAM Role".to_string(),
                category: ComponentCategory::Security,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "shield",
                default_size: Size::new(100.0, 40.0),
                properties: vec![
                    PropertyDefinition {
                        key: "assume_role_policy".to_string(),
                        name: "Assume Role Policy".to_string(),
                        input_type: PropertyInputType::JsonEditor,
                        required: true,
                        default: Some(json!({
                            "Version": "2012-10-17",
                            "Statement": [{
                                "Effect": "Allow",
                                "Principal": {"Service": "lambda.amazonaws.com"},
                                "Action": "sts:AssumeRole"
                            }]
                        })),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // CloudFront Distribution
        self.definitions.insert(
            ComponentType::CloudFrontDistribution,
            ComponentDefinition {
                type_: ComponentType::CloudFrontDistribution,
                name: "CloudFront Distribution".to_string(),
                category: ComponentCategory::Network,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "cloud",
                default_size: Size::new(120.0, 50.0),
                properties: vec![
                    PropertyDefinition {
                        key: "origin_domain_name".to_string(),
                        name: "Origin Domain".to_string(),
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: None,
                        options: None,
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "viewer_protocol_policy".to_string(),
                        name: "Viewer Protocol Policy".to_string(),
                        input_type: PropertyInputType::Select,
                        required: false,
                        default: Some(json!("https-only")),
                        options: Some(vec![
                            PropertyOption { label: "HTTPS Only".to_string(), value: json!("https-only") },
                            PropertyOption { label: "Redirect HTTP to HTTPS".to_string(), value: json!("redirect-to-https") },
                        ]),
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // DynamoDB Table
        self.definitions.insert(
            ComponentType::DynamoTable,
            ComponentDefinition {
                type_: ComponentType::DynamoTable,
                name: "DynamoDB Table".to_string(),
                category: ComponentCategory::Storage,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "database",
                default_size: Size::new(100.0, 60.0),
                properties: vec![
                    PropertyDefinition {
                        key: "table_name".to_string(),
                        name: "Table Name".to_string(),
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: None,
                        options: None,
                        validation: None,
                    },
                    PropertyDefinition {
                        key: "hash_key".to_string(),
                        name: "Partition Key".to_string(),
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: Some(json!("id")),
                        options: None,
                        validation: None,
                    },
                ],
                default_properties: HashMap::new(),
            },
        );
        
        // WAF
        self.definitions.insert(
            ComponentType::Waf,
            ComponentDefinition {
                type_: ComponentType::Waf,
                name: "WAF".to_string(),
                category: ComponentCategory::Security,
                cloud_provider: Some(CloudProvider::Aws),
                icon: "shield",
                default_size: Size::new(80.0, 40.0),
                properties: vec![
                    PropertyDefinition {
                        key: "web_acl_name".to_string(),
                        name: "Web ACL Name".to_string(),
                        input_type: PropertyInputType::Text,
                        required: true,
                        default: None,
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
    
    pub fn create_component(
        &self,
        component_type: ComponentType,
        name: String,
        position: Position,
    ) -> Result<Component, &'static str> {
        let definition = self.get(&component_type)
            .ok_or("Component type not found in registry")?;
        
        Ok(Component::from_definition(name, component_type, position, definition))
    }
}
```

---

## 6. Roadmap de Implementación MVP

### 6.1 Sprint Breakdown (12 Semanas)

| Sprint | Duración | Entregable | Complejidad |
|--------|----------|------------|-------------|
| **0** | Semana 1 | Setup workspace, CI/CD, configuración | Baja |
| **1** | Semanas 2-3 | Core domain (Architecture, Component, Events) | Media |
| **2** | Semanas 4-5 | Canvas state, input handling, hit-test | Media |
| **3** | Semanas 6-7 | WGPU renderer básico (WebGL2) | Alta |
| **4** | Semanas 8-9 | Component registry, drag-drop, selección | Media |
| **5** | Semanas 10-11 | Properties panel, export Terraform | Media |
| **6** | Semana 12 | Testing, optimization, polish | Baja |

### 6.2 Milestones

| Milestone | Semana | Criterios |
|-----------|--------|-----------|
| M1 | 3 | Core domain compilando, tests pasando |
| M2 | 5 | Canvas state con hit-test funcional |
| M3 | 7 | WGPU renderer mostrando componentes |
| M4 | 9 | CRUD completo de componentes |
| M5 | 11 | Export Terraform funcionando |
| M6 | 12 | MVP Release (10k nodos @ 60fps) |

---

## 7. Resumen de Decisiones

### 7.1 Stack Confirmado

| Decisión | Valor |
|----------|-------|
| **Frontend UI** | Leptos + WASM |
| **Rendering** | wgpu (WebGL2 default, WebGPU futuro) |
| **State Management** | Leptos Signals + DDD aggregates |
| **Persistence** | IndexedDB (rust-indexeddb) |
| **Storage Format** | AUF (YAML) |
| **IaC Export** | Terraform HCL |
| **Components** | 10 tipos AWS iniciales |
| **Bundle Size Target** | < 5 MB |

### 7.2 Documentos de Referencia

| Documento | Contenido |
|-----------|-----------|
| `docs/ARCHITECTURE-STUDY.md` | Arquitectura general DDD |
| `docs/LEPTOS-VISUAL-IMPLEMENTATION.md` | Features visuales |
| `docs/BEVY-VS-WGPU-STUDY.md` | Análisis Bevy vs wgpu |
| `docs/WGPU-VS-GLOW.md` | Comparación wgpu vs glow |
| `docs/ARCHFLOW-MVP-IMPLEMENTATION.md` | **Este documento** |

---

**Documento preparado para inicio de implementación.**

**Próximo paso:** Iniciar Sprint 0 - Setup del workspace.
