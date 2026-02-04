# EPIC-RENDER-001: Multi-Backend Rendering with Degradation Strategy

## Metadata

| Campo | Valor |
|-------|-------|
| **ID** | EPIC-RENDER-001 |
| **Título** | Sistema de Rendering Multi-Backend con Degradación Automática |
| **Prioridad** | Alta |
| **Estimación** | XL (8+ sprints) |
| **Estado** | Definición |
| **Fecha creación** | 2026-02-03 |
| **Product Owner** | @team |
| **Arquitecto responsable** | @team |

## Executive Summary

Implementar un sistema de rendering que degrada automáticamente entre WebGPU → WebGL2 → Canvas 2D según la disponibilidad del navegador, priorizando rendimiento óptimo sin perder funcionalidad. La arquitectura se integra con el código existente en `archflow-render`, `archflow-web`, y `archflow-engine`.

### Contexto del Problema

- **Actual**: Rendering solo con WebGPU (no disponible en Linux hasta 2026)
- **Impacto**: Aplicación no funciona en ~40% de usuarios
- **Presupuesto**: +500KB para WebGL2 (total ~1.3MB WASM)
- **Restricción**: No se puede perder NINGUNA feature existente

### Arquitectura Existente

```
crates/
├── archflow-core/          # Tipos base: Vec2, Color, EntityId, Rect
├── archflow-engine/         # EntityStore, lógica de negocio
│   └── store.rs            # EntityStore con MAX_ENTITIES = 100k
├── archflow-render/         # WebGPU Rendering (EXISTENTE)
│   ├── lib.rs              # Reexports: GpuRenderer, Camera, Atlas, etc.
│   ├── renderer.rs         # GpuRenderer con Multi-Phase Instancing
│   ├── pipelines.rs        # RenderPipelines (4 specialized pipelines)
│   ├── gpu_resources.rs    # GpuResources (buffers, textures)
│   ├── webgpu_context.rs   # WebGpuContext wrapper
│   ├── camera.rs           # 2D infinite camera
│   ├── atlas.rs            # Texture atlas con shelf packing
│   └── shaders/            # WGSL shaders
│       ├── sdf_shapes.wgsl
│       ├── icon_texture.wgsl
│       ├── image_array.wgsl
│       └── mtsdf_text.wgsl
└── archflow-web/            # WASM Bridge (EXISTENTE)
    └── bridge.rs           # WasmBridge con initialize_graphics()
```

## Arquitectura Propuesta (DDD)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TypeScript/React Layer                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  Canvas.tsx (crates/archflow-web-ui/src/components/)           │    │
│  │  - Llama a bridge.initialize_graphics(canvas)                  │    │
│  │  - NO modifica (delegación completa a WASM)                    │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      WASM Interface Layer (archflow-web)                │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │           WasmBridge (crates/archflow-web/src/bridge.rs)       │    │
│  │  CAMBIO: engine: ArchFlowEngine                                  │    │
│  │          .renderer: Box<dyn Renderer>  (NUEVO CAMPO)             │    │
│  │                                                                 │    │
│  │  fn initialize_graphics(&mut self, canvas) -> Result<()> {     │    │
│  │      // NUEVO: Usar RendererSelector                            │    │
│  │      let renderer = RendererSelector::detect_and_create()?;     │    │
│  │      self.engine.set_renderer(renderer);                        │    │
│  │  }                                                             │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                   Application Layer (archflow-engine)                    │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │              ArchFlowEngine (MODIFICADO)                         │    │
│  │  CAMBIOS:                                                       │    │
│  │  - renderer: Option<Box<dyn Renderer>>  (NUEVO)                │    │
│  │  - fn set_renderer(&mut self, renderer: Box<dyn Renderer>)     │    │
│  │  - fn tick(&mut self) {                                        │    │
│  │        self.renderer.render(&self.store)?;                     │    │
│  │    }                                                           │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  EntityStore (SIN CAMBIOS - ya está bien desacoplado)                   │
│  - Contiene: pos, size, colors, shape_type, texture_index, uv_rects   │
│  - Métodos: spawn(), despawn(), is_visible(), etc.                     │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Domain Layer (archflow-render)                       │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                   Renderer Trait (NUEVO - Port)                 │    │
│  │  pub trait Renderer {                                           │    │
│  │      fn sync_from_store(&mut self, store: &EntityStore)        │    │
│  │          -> usize;  // Retorna visible entities count          │    │
│  │      fn render(&mut self) -> Result<(), RenderError>;          │    │
│  │      fn resize(&mut self, width: u32, height: u32);            │    │
│  │      fn backend_name(&self) -> &'static str;                   │    │
│  │  }                                                              │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                 Infrastructure Layer (archflow-render)                   │
│                                                                         │
│  ESTRUCTURA DE CRATES:                                                  │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │  archflow-render/ (EXISTENTE - refactorizado)                │      │
│  │  ├── lib.rs                                                  │      │
│  │  ├── renderer.rs  → GpuRenderer (renombrar a WebGPURenderer) │      │
│  │  ├── pipelines.rs    (MANTIENE - 4 specialized pipelines)    │      │
│  │  ├── gpu_resources.rs (MANTIENE - Buffers, textures)         │      │
│  │  ├── webgpu_context.rs (MANTIENE)                            │      │
│  │  ├── camera.rs        (MANTIENE - compartido por todos)      │      │
│  │  ├── atlas.rs         (MANTIENE - compartido por todos)      │      │
│  │  ├── shaders.rs       (MANTIENE - WGSL sources)              │      │
│  │  └── webgl2/          (NUEVO SUB-MÓDULO)                     │      │
│  │      ├── mod.rs        # WebGL2Renderer                       │      │
│  │      ├── context.rs    # WebGL2Context                        │      │
│  │      ├── pipelines.rs  # WebGL2RenderPipelines                │      │
│  │      └── shaders/      # GLSL shaders (generados por Naga)    │      │
│  │                                                             │      │
│  │  Y FUTURO:                                                   │      │
│  │  └── canvas2d/         (Canvas2DRenderer - fallback final)   │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                         │
│  IMPLEMENTACIÓN DEL TRAIT:                                              │
│  ┌──────────────────┐  ┌──────────────────┐  ┌─────────────────────┐   │
│  │  WebGPURenderer  │  │  WebGL2Renderer  │  │  Canvas2DRenderer   │   │
│  │  (Adapter)       │  │  (Adapter)       │  │  (Adapter)          │   │
│  │                  │  │                  │  │                     │   │
│  │  - GpuRenderer   │  │  - NUEVO         │  │  - FUTURO           │   │
│  │    refactorizado │  │                  │  │                     │   │
│  │  - Impl Renderer │  │  - Impl Renderer │  │  - Impl Renderer    │   │
│  └──────────────────┘  └──────────────────┘  └─────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Principios SOLID Aplicados

### S - Single Responsibility Principle
- `RendererSelector`: Solo decide qué backend usar
- `WebGPURenderer`: Solo rendering WebGPU
- `WebGL2Renderer`: Solo rendering WebGL2
- `EntityStore`: Solo gestión de entidades (no cambios)

### O - Open/Closed Principle
```rust
// Abierto para extensión, cerrado para modificación
pub trait Renderer {
    fn sync_from_store(&mut self, store: &EntityStore) -> usize;
    fn render(&mut self) -> Result<(), RenderError>;
}

// Nuevos backends sin modificar código existente
impl Renderer for WebGPURenderer { /* refactorizado de GpuRenderer */ }
impl Renderer for WebGL2Renderer { /* nuevo */ }
impl Renderer for Canvas2DRenderer { /* futuro */ }
```

### L - Liskov Substitution Principle
```rust
// Cualquier Renderer puede sustituir a otro sin romper el sistema
pub struct ArchFlowEngine {
    renderer: Option<Box<dyn Renderer>>,
}

impl ArchFlowEngine {
    pub fn set_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderer = Some(renderer);
    }
    
    pub fn tick(&mut self, timestamp: f64) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.sync_from_store(&self.store, &self.camera);
            renderer.render().ok();
        }
    }
}
```

#### Tests de Integración Implementados

```rust
// RED - Test de integración: verificar que renderer se inyecta correctamente
#[test]
fn test_set_renderer() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);
    let original_backend = engine.renderer.backend_name();
    assert_eq!(original_backend, "cpu");  // GpuRenderer es CPU-side stub

    // Crear mock renderer
    let mock_renderer = MockRenderer::new();
    engine.set_renderer(Box::new(mock_renderer));

    // Verificar que el renderer cambió
    assert_eq!(engine.renderer.backend_name(), "mock");
}

// GREEN - Implementar MockRenderer
pub struct MockRenderer {
    backend: &'static str,
}

impl MockRenderer {
    pub fn new() -> Self {
        Self {
            backend: "mock",
        }
    }
}

impl Renderer for MockRenderer {
    fn sync_from_store(&mut self, _store: &EntityStore, _camera: &Camera) -> usize {
        0
    }

    fn batch_count(&self, _phase: RenderPhase) -> usize {
        0
    }

    fn instances(&self) -> &[GpuInstance] {
        &[]
    }

    fn camera_uniforms(&self) -> &CameraUniforms {
        static UNIFORMS: CameraUniforms = CameraUniforms::default();
        &UNIFORMS
    }

    fn batch_indices(&self, _phase: RenderPhase) -> &[u32] {
        &[]
    }

    fn total_draw_calls(&self) -> u32 {
        0
    }

    fn resize(&mut self, _width: u32, _height: u32) {}

    fn backend_name(&self) -> &'static str {
        self.backend
    }
}
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "Rust WASM WebGPU WebGL2 detection navigator.gpu browser capabilities 2025"

**Qué buscar:**
- [ ] Cómo detectar WebGPU desde Rust WASM (wasm-bindgen)
- [ ] Diferencias en detección entre navegadores (Chrome, Firefox, Safari)
- [ ] Cómo hacer fallback gracefully cuando un backend falla
- [ ] Patrones para lazy initialization de renderers

**TDD Approach:**
```rust
// RED - Test primero
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backend_detection_order() {
        // Mock navigator with WebGPU enabled
        // Should prefer WebGPU → WebGL2 → Canvas2D
    }
    
    #[test]
    fn test_renderer_selector_fallback() {
        // When WebGPU fails, should fall back to WebGL2
        // When WebGL2 fails, should fall back to Canvas2D
    }
}

// GREEN - Implementación mínima
impl RendererSelector {
    pub fn detect_and_create() -> Result<Box<dyn Renderer>, RenderError> {
        // Try backends in order
    }
}

// REFACTOR - Extraer lógica de detección
```

#### Trazas de Debug (Development Mode)
```rust
#[cfg(debug_assertions)]
{
    tracing::info!(target: "archflow::render::selector",
        webgpu_available = has_webgpu,
        webgl2_available = has_webgl2,
        canvas_available = true,
        selected_backend = ?backend,
        "Backend detection completed");
}

#[cfg(debug_assertions)]
if selected_backend == Backend::Canvas2D {
    tracing::warn!(target: "archflow::render::selector",
        "Using Canvas2D fallback - performance will be significantly reduced");
}
```

#### Estimación
- Investigación: 4 horas
- Implementación trait Renderer: 4 horas
- Implementación RendererSelector: 6 horas
- Refactorización GpuRenderer: 4 horas
- Tests: 6 horas
- **Total: 24 horas (~3 días)**

---

### HU-RENDER-002: Implementación de WebGL2 Renderer

**Como** desarrollador de ArchFlow  
**Quiero** implementar un renderer completo usando WebGL2  
**Para** dar soporte a ~95% de navegadores con rendimiento óptimo

#### Criterios de Aceptación
- [ ] `WebGL2Renderer` implementa trait `Renderer`
- [ ] Soporta las 4 fases de rendering (Shapes, Icons, Images, Text)
- [ ] Instanced rendering para 100k entities
- [ ] Compatible con EntityStore existente
- [ ] Tests de integración

#### Tareas Técnicas

**1. Crear estructura de módulos en `archflow-render/src/webgl2/`:**
```
archflow-render/src/webgl2/
├── mod.rs              # WebGL2Renderer main implementation
├── context.rs          # WebGL2Context (similar a WebGpuContext)
├── pipelines.rs        # WebGL2RenderPipelines (shader programs)
├── buffers.rs          # GPU buffer management
└── shaders/            # GLSL shaders (generados desde WGSL)
    ├── shapes.glsl
    ├── icons.glsl
    ├── images.glsl
    └── text.glsl
```

**2. Implementar `WebGL2Context` en `context.rs`:**
```rust
use glow;

/// WebGL2 context wrapper
pub struct WebGL2Context {
    pub gl: glow::Context,
    pub canvas: web_sys::HtmlCanvasElement,
}

impl WebGL2Context {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, String> {
        // Get WebGL2 context from canvas
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        // Use glow for WebGL2 bindings
        let gl = unsafe {
            // Initialize glow context from canvas
            // This requires setup with wasm-bindgen
            todo!("Initialize glow from canvas")
        };
        
        Ok(Self { gl, canvas })
    }
}
```

**3. Implementar `WebGL2Renderer` en `mod.rs`:**
```rust
use crate::{
    Renderer, RenderError,
    camera::Camera,
    engine::EntityStore,
};
use super::{context::WebGL2Context, pipelines::WebGL2RenderPipelines};

pub struct WebGL2Renderer {
    context: WebGL2Context,
    pipelines: WebGL2RenderPipelines,
    instance_buffer: Vec<f32>,  // Flat buffer for instanced rendering
    batch_counts: [usize; 4],   // One per render phase
}

impl WebGL2Renderer {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, RenderError> {
        let context = WebGL2Context::new(canvas).await?;
        let pipelines = WebGL2RenderPipelines::new(&context)?;
        
        Ok(Self {
            context,
            pipelines,
            instance_buffer: Vec::with_capacity(100_000 * 16), // 16 floats per instance
            batch_counts: [0; 4],
        })
    }
}

impl Renderer for WebGL2Renderer {
    fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize {
        // Similar logic to GpuRenderer::sync_from_store
        // 1. Clear previous frame data
        // 2. Iterate visible entities
        // 3. Sort into render phases
        // 4. Build instance buffer
        todo!("Implement sync_from_store for WebGL2")
    }
    
    fn render(&mut self) -> Result<(), RenderError> {
        // 1. Upload instance buffer to GPU
        // 2. Render each phase with instanced draw calls
        // 3. Use gl.draw_arrays_instanced()
        todo!("Implement render for WebGL2")
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.context.gl.viewport(0, 0, width as i32, height as i32);
    }
    
    fn backend_name(&self) -> &'static str {
        "WebGL2"
    }
}
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "Rust WASM WebGL2 glow crate instanced rendering best practices 2025"

**Qué buscar:**
- [ ] Cómo configurar `glow` para WASM
- [ ] Instanced rendering API en WebGL2 (ANGLE_instanced_arrays)
- [ ] Cómo estructurar vertex data para instanced rendering
- [ ] Diferencias en shader language (WGSL vs GLSL ES 3.0)
- [ ] Performance patterns para WebGL2 (buffer updates, state changes)

**Recursos específicos:**
- [ ] Glow documentation: https://github.com/grovesNL/glow
- [ ] WebGL2 instanced rendering examples
- [ ] Three.js WebGL2 renderer code (para reference)

**TDD Approach:**
```rust
// RED - Test de integración
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_webgl2_renderer_creation() {
    let canvas = create_test_canvas();
    let renderer = WebGL2Renderer::new(canvas).await.unwrap();
    assert_eq!(renderer.backend_name(), "WebGL2");
}

// GREEN - Implementar creación
impl WebGL2Renderer {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, RenderError> {
        // Create context, pipelines, etc.
    }
}

// REFACTOR - Extraer inicialización de shaders
```

#### Trazas de Debug (Development Mode)
```rust
#[cfg(debug_assertions)]
{
    tracing::info!(target: "archflow::render::webgl2",
        entities = entity_count,
        draw_calls = draw_call_count,
        batch_counts = ?self.batch_counts,
        frame_time_ms = frame_time,
        "WebGL2 frame completed");
    
    if draw_call_count > 10 {
        tracing::warn!(target: "archflow::render::webgl2",
            draw_calls = draw_call_count,
            "High draw call count - should use instanced rendering");
    }
}
```

#### Estado
✅ **COMPLETADO** - 2026-02-04
- Commit: feat(render): implement WebGL2 texture alignment utility
- Commit: docs(epic): update HU-RENDER-002 with texture alignment utility
- Texture Padding Utility implementada para WebGL2
  - `texture_layout.rs` module (432 lines)
  - `PixelFormat` enum: 12 formats (R8, RG8, RGB8, RGBA8, 16F, 32F variants)
  - `Alignment` enum: 1, 2, 4, 8 bytes
  - `TextureLayout` struct with row padding calculations
  - Functions: `calculate_aligned_row_size()`, `calculate_optimal_alignment()`, `pad_texture_data()`
  - **14 tests passing** (texture_layout module)
- WebGL2 fallas silenciosas o errores crípticos si el alineamiento no es exacto
  - ✅ Resuelto con la utilidad de alineamiento de texturas
  - WebGPU requiere alineamiento de 256 bytes
  - WebGL2 requiere alineamiento de 1/2/4/8 bytes (via GL_UNPACK_ALIGNMENT)
- Nota: HU-RENDER-002 tiene el shader compilation pendiente (HU-RENDER-004)

#### Estimación
- Investigación: 8 horas
- Setup de glow/OpenGL bindings: 8 horas
- Implementación pipelines: 12 horas
- Implementación renderer: 16 horas
- Tests: 8 horas
- **Total: 52 horas (~1.5 semanas)**

---

### HU-RENDER-003: Integración de Renderer en ArchFlowEngine

**Como** desarrollador de ArchFlow  
**Quiero** integrar el trait `Renderer` en `ArchFlowEngine`  
**Para** que el motor use el renderer seleccionado dinámicamente

#### Criterios de Aceptación
- [x] `ArchFlowEngine` tiene campo `renderer: Box<dyn Renderer>` (con default GpuRenderer)
- [x] Método `set_renderer()` para inyectar el renderer
- [x] Método `tick()` usa el renderer para renderizar
- [x] Compatibilidad con código existente

#### Estado
✅ **COMPLETADO** - 2026-02-04
- Commit: feat(engine): implement HU-RENDER-003 renderer integration
- Todos los tests pasando (87 tests en archflow-web)
- Implementación actual usa `Box<dyn Renderer>` en lugar de `Option<Box<dyn Renderer>>`
  - Proporciona un renderer por defecto (GpuRenderer) para simplicidad
  - Permite cambio de backend en tiempo de ejecución via `set_renderer()`
  - WasmBridge inyecta WebGL2Renderer cuando se inicializa gráficos

#### Tareas Técnicas

**1. Modificar `archflow-engine/src/lib.rs` o donde esté `ArchFlowEngine`:**
```rust
use archflow_render::Renderer;

pub struct ArchFlowEngine {
    // Campos existentes...
    pub store: EntityStore,
    pub camera: Camera,

    // NUEVO: Renderer inyectable (con default)
    pub renderer: Box<dyn Renderer>,

    // Campos existentes...
    pub command_queue: Vec<Command>,
    pub history: History,
    pub selected_entities: Vec<EntityId>,
    // ... etc
}

impl ArchFlowEngine {
    pub fn new(canvas_width: f32, canvas_height: f32) -> Self {
        Self {
            store: EntityStore::new(),
            camera: Camera::new(canvas_width, canvas_height),
            renderer: Box::new(GpuRenderer::new()),  // Default renderer
            // ... resto de campos
        }
    }

    /// NUEVO: Inyectar renderer (para cambio de backend en runtime)
    pub fn set_renderer(&mut self, new_renderer: Box<dyn Renderer>) {
        self.renderer = new_renderer;
    }

    /// MODIFICADO: Usar renderer en tick
    pub fn tick(&mut self, _timestamp: f64) {
        // Procesar comandos (existente)
        self.process_commands();

        // NUEVO: Sincronizar renderer con EntityStore
        let visible_count = self.renderer.sync_from_store(&self.store, &self.camera);

        #[cfg(debug_assertions)]
        tracing::trace!(target: "archflow::engine",
            visible_entities = visible_count,
            total_entities = self.store.alive_count(),
            "Engine tick");

        // Nota: renderer.render() se llamará desde el bridge WASM
        // El renderer actual (GpuRenderer) es CPU-side y prepara datos,
        // pero el renderizado real lo hace WebGL2Renderer en el bridge
    }
}
```

**2. Actualizar `WasmBridge` en `archflow-web/src/bridge.rs`:**
```rust
use archflow_render::{Renderer, RendererSelector};

#[wasm_bindgen]
impl WasmBridge {
    #[wasm_bindgen]
    pub async fn initialize_graphics(
        &self,
        canvas: web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        tracing::info!(target: "archflow::wasm", "Initializing graphics with renderer selector");

        // NUEVO: Usar RendererSelector
        let renderer = RendererSelector::detect_and_create(canvas)
            .map_err(|e| JsError::new(&format!("Failed to create renderer: {:?}", e)))?;
        
        // Inyectar renderer en el engine
        if let Some(engine) = self.engine.borrow_mut().as_mut() {
            engine.set_renderer(renderer);
            
            #[cfg(feature = "tracing-logging")]
            tracing::info!(target: "archflow::wasm",
                backend = engine.renderer.as_ref().map(|r| r.backend_name()).unwrap_or("None"),
                "Graphics initialized successfully");
            
            Ok(())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }
}
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "Rust trait object lifetime Box dyn Renderer WASM ownership 2025"

**Qué buscar:**
- [ ] Cómo manejar lifetimes con trait objects en WASM
- [ ] Ownership patterns con `Box<dyn Renderer>`
- [ ] Cómo pasar renderer entre módulos

**TDD Approach:**
```rust
// RED - Test de integración
#[test]
fn test_engine_with_mock_renderer() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);
    let mock_renderer = MockRenderer::new();
    engine.set_renderer(Box::new(mock_renderer));
    
    // Spawn some entities
    engine.store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
    
    // Tick should call renderer
    engine.tick(0.0);
    
    assert!(mock_renderer.was_called());
}

// GREEN - Implementar set_renderer
impl ArchFlowEngine {
    pub fn set_renderer(&mut self, renderer: Box<dyn Renderer>) {
        self.renderer = Some(renderer);
    }
}

// REFACTOR - Limpiar interfaz
```

#### Trazas de Debug (Development Mode)
```rust
#[cfg(debug_assertions)]
{
    tracing::info!(target: "archflow::engine",
        backend = renderer.backend_name(),
        "Renderer set in engine");
}
```

#### Estimación
- Investigación: 2 horas
- Modificación ArchFlowEngine: 6 horas
- Modificación WasmBridge: 4 horas
- Tests: 4 horas
- **Total: 16 horas (~2 días)**

---

### HU-RENDER-004: Compilación de Shaders (WGSL → GLSL)

**Como** desarrollador de ArchFlow  
**Quiero** compilar shaders WGSL a GLSL automáticamente  
**Para** compartir lógica de shaders entre WebGPU y WebGL2

#### Criterios de Aceptación
- [ ] Shadores escritos en WGSL (source of truth)
- [ ] `build.rs` usa Naga para compilar a GLSL
- [ ] Shaders GLSL generados en `target/debug/build/`
- [ ] Integración con ambos renderers

#### Tareas Técnicas

**1. Crear `archflow-render/build.rs`:**
```rust
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=shaders");
    
    let shaders_dir = Path::new("shaders");
    let out_dir = Path::new(&std::env::var("OUT_DIR").unwrap());
    
    // List of WGSL shaders to compile
    let wgsl_shaders = [
        "sdf_shapes.wgsl",
        "icon_texture.wgsl",
        "image_array.wgsl",
        "mtsdf_text.wgsl",
    ];
    
    for shader_name in wgsl_shaders.iter() {
        let wgsl_path = shaders_dir.join(shader_name);
        let wgsl_content = fs::read_to_string(&wgsl_path)
            .expect(&format!("Failed to read {}", shader_name));
        
        // Use naga to compile WGSL to GLSL
        #[cfg(feature = "webgl2")]
        {
            let module = naga::wgsl::parse_str(&wgsl_content)
                .expect(&format!("Failed to parse {}", shader_name));
            
            let glsl = naga::back::glsl::write(
                &module,
                &naga::back::glsl::Options {
                    version: naga::back::glsl::Version::Embedded { version: 300 },
                    ..Default::default()
                },
            ).expect(&format!("Failed to compile {} to GLSL", shader_name));
            
            // Write GLSL output
            let glsl_name = shader_name.replace(".wgsl", ".glsl");
            let glsl_path = out_dir.join(&glsl_name);
            fs::write(&glsl_path, glsl)
                .expect(&format!("Failed to write {}", glsl_name));
        }
    }
}
```

**2. Actualizar `archflow-render/src/renderer.rs` para cargar shaders:**
```rust
// Include GLSL shaders if webgl2 feature is enabled
#[cfg(feature = "webgl2")]
const SHAPES_GLSL: &str = include_str!(concat!(env!("OUT_DIR"), "/sdf_shapes.glsl"));
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "Naga WGSL to GLSL build script Rust 2025 shader compilation"

**Qué buscar:**
- [ ] Cómo integrar Naga en `build.rs`
- [ ] Limitaciones de WGSL → GLSL (features no soportados)
- [ ] Cómo manejar errores de compilación de shaders
- [ ] Estrategias para versioning de shaders

**TDD Approach:**
```rust
// RED - Test de compilación
#[test]
fn test_shader_compilation() {
    let wgsl = std::fs::read_to_string("shaders/sdf_shapes.wgsl").unwrap();
    let module = naga::wgsl::parse_str(&wgsl).unwrap();
    let glsl = naga::back::glsl::write(&module, &Default::default()).unwrap();
    assert!(glsl.contains("void main"));
}

// GREEN - Integrar en build.rs
// (ver código arriba)

// REFACTOR - Extraer a función
```

#### Trazas de Debug (Development Mode)
```rust
#[cfg(debug_assertions)]
{
    tracing::info!(target: "archflow::render::shaders",
        shader_name = "sdf_shapes.wgsl",
        wgsl_size = wgsl_content.len(),
        glsl_size = glsl.len(),
        "Shader compiled");
}
```

#### Estimación
- Investigación: 6 horas
- Setup build.rs: 8 horas
- Integración con renderers: 8 horas
- Tests: 6 horas
- **Total: 28 horas (~3.5 días)**

---

### HU-RENDER-005: Optimización de Performance

**Como** usuario de ArchFlow  
**Quiero** que el rendering mantenga 60fps con 100k entities  
**Para** tener una experiencia fluida en escenarios complejos

#### Criterios de Aceptación
- [ ] 100,000 entities a 60fps en WebGPU
- [ ] 100,000 entities a 50fps+ en WebGL2
- [ ] Frustum culling ya existe en `GpuRenderer` (reutilizar)
- [ ] Benchmarks automatizados

#### Tareas Técnicas

**1. Reutilizar lógica de culling existente:**
```rust
// GpuRenderer ya tiene viewport culling en sync_from_store
// Reutilizar esta lógica para WebGL2Renderer
```

**2. Crear benchmarks en `archflow-render/benches/`:**
```rust
// benches/render_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use archflow_render::{WebGPURenderer, WebGL2Renderer};
use archflow_engine::EntityStore;

fn bench_render_100k(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_100k");
    
    // WebGPU
    group.bench_function("webgpu", |b| {
        let store = EntityStore::with_100k_entities();
        let mut renderer = WebGPURenderer::new().unwrap();
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
            renderer.render().unwrap();
        });
    });
    
    // WebGL2
    group.bench_function("webgl2", |b| {
        let store = EntityStore::with_100k_entities();
        let mut renderer = WebGL2Renderer::new().await.unwrap();
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
            renderer.render().unwrap();
        });
    });
}

criterion_group!(benches, bench_render_100k);
criterion_main!(benches);
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "WebGL2 instanced rendering performance optimization Rust WASM 2025 and 2026"

**Qué buscar:**
- [ ] Técnicas de batching para WebGL2
- [ ] Cómo minimizar state changes
- [ ] Buffer management (orphaning, streaming)

**TDD Approach:**
```rust
// RED - Test de performance
#[bench]
fn bench_webgl2_100k_entities(b: &mut Bencher) {
    let store = EntityStore::with_100k_entities();
    let mut renderer = WebGL2Renderer::new().await.unwrap();
    b.iter(|| {
        renderer.sync_from_store(&store, &camera);
        renderer.render().unwrap();
    });
}

// GREEN - Optimizar
impl WebGL2Renderer {
    fn render(&mut self) -> Result<(), RenderError> {
        // Use instanced rendering
        // Minimize state changes
        // Batch by texture
    }
}

// REFACTOR - Extraer a trait
```

#### Trazas de Debug (Development Mode)
```rust
#[cfg(debug_assertions)]
{
    tracing::info!(target: "archflow::render::perf",
        backend = self.backend_name(),
        total_entities = total_count,
        visible_after_culling = visible_count,
        frame_time_ms = frame_time,
        "Rendering performance");
}
```

#### Estimación
- Investigación: 8 horas
- Optimización WebGL2: 16 horas
- Benchmarks: 8 horas
- **Total: 32 horas (~4 días)**

---

### HU-RENDER-006: Verificación de Paridad de Features

**Como** equipo de desarrollo  
**Queremos** verificar que TODAS las features existentes funcionan en todos los backends  
**Para** asegurar que no perdemos funcionalidad con la degradación

#### Criterios de Aceptación
- [ ] Text rendering funciona en WebGL2
- [ ] Multi-phase rendering (Shapes, Icons, Images, Text)
- [ ] Camera transformations funcionan
- [ ] Colores y blending funcionan
- [ ] Test suite de regresión

#### Tareas Técnicas

**1. Crear checklist de features:**
- [x] Rendering de shapes (rect, circle, etc.)
- [x] Rendering de texturas (icons)
- [x] Rendering de imágenes
- [x] Rendering de texto (MTSDF)
- [x] Camera zoom/pan
- [x] Colores y transparencia
- [x] Viewport culling

**2. Tests de regresión:**
```rust
#[cfg(test)]
mod regression_tests {
    use super::*;
    
    #[test]
    fn test_shapes_rendering() {
        // Test que shapes se renderizan igual en ambos backends
    }
    
    #[test]
    fn test_text_rendering() {
        // Test que texto se renderiza igual
    }
    
    #[test]
    fn test_camera_transforms() {
        // Test que transforms funcionan
    }
}
```

#### Estimación
- Tests: 16 horas
- Documentación: 4 horas
- **Total: 20 horas (~2.5 días)**

---

### HU-RENDER-007: Sistema de Dirty Checking y Buffering Persistente

**Como** desarrollador de ArchFlow  
**Quiero** implementar un sistema de mapeo de memoria persistente que solo actualice los buffers de entidades modificadas  
**Para** mantener 60fps con 100k entidades reduciendo el overhead de sincronización CPU→GPU

#### Criterios de Aceptación
- [ ] Sistema de dirty tracking integrado en `EntityStore`
- [ ] `GpuRenderer` solo actualiza buffers de entidades marcadas como "dirty"
- [ ] WebGL2Renderer usa `write_buffer` con offsets específicos
- [ ] Reducción del 80%+ en tiempo de `sync_from_store` para escenas estáticas
- [ ] Tests de performance con escenas mixtas (entidades dinámicas + estáticas)

#### Tareas Técnicas

**1. Implementar dirty tracking en EntityStore:**
```rust
// EN: archflow-engine/src/store.rs

/// Sistema de tracking de entidades modificadas
pub struct EntityStore {
    // ... campos existentes
    dirty_entities: EntityIdSet,  // NUEVO: entidades modificadas
    version_counter: u64,          // NUEVO: para detectar cambios globales
}

impl EntityStore {
    /// NUEVO: Marcar entidad como modificada
    pub fn mark_dirty(&mut self, id: EntityId) {
        self.dirty_entities.insert(id);
        self.version_counter += 1;
    }

    /// NUEVO: Obtener entidades dirty desde última sincronización
    pub fn take_dirty_entities(&mut self) -> Vec<EntityId> {
        self.dirty_entities.take_all()
    }

    /// NUEVO: Verificar si hay cambios globales (camera, viewport)
    pub fn version(&self) -> u64 {
        self.version_counter
    }

    /// Modificar setter existentes para marcar dirty
    pub fn set_position(&mut self, id: EntityId, pos: Vec2) {
        if let Some(entity) = self.get_mut(id) {
            entity.pos = pos;
            self.mark_dirty(id);
        }
    }
}
```

**2. Implementar buffering persistente en GpuResources:**
```rust
// EN: archflow-render/src/gpu_resources.rs

pub struct GpuResources {
    // Buffer persistente para instancias (mapeado una vez)
    instance_buffer: WgpuBuffer,

    // Rastrear qué rangos del buffer necesitan actualización
    dirty_ranges: Vec<Range<u32>>,
}

impl GpuResources {
    /// Escribir solo los datos dirty al buffer persistente
    pub fn write_dirty_instances(
        &mut self,
        device: &Device,
        queue: &Queue,
        store: &EntityStore,
    ) {
        let dirty_ids = store.take_dirty_entities();

        // Agrupar por región del buffer para minimizar writes
        for id in dirty_ids {
            let entity = store.get(id).unwrap();
            let offset = id.index() * INSTANCE_STRIDE;

            // Usar write_buffer con offset específico
            queue.write_buffer(
                &self.instance_buffer.buffer(),
                offset as u64,
                bytemuck::bytes_of(&entity.transform_matrix),
                std::mem::size_of::<Mat4>() as u64,
            );
        }
    }

    /// Forzar write completo (para frames críticos)
    pub fn write_all_instances(&mut self, queue: &Queue, instances: &[GpuInstance]) {
        queue.write_buffer(
            &self.instance_buffer.buffer(),
            0,
            bytemuck::cast_slice(instances),
        );
    }
}
```

**3. Implementar en WebGL2Renderer:**
```rust
// EN: archflow-render/src/webgl2/renderer.rs

impl WebGL2Renderer {
    /// Escribir solo entidades dirty al buffer de instancias
    fn write_dirty_instances(&mut self, store: &EntityStore) {
        let dirty_ids = store.take_dirty_entities();

        for id in dirty_ids {
            let entity = store.get(id).unwrap();
            let offset = id.index() * INSTANCE_STRIDE;

            // WebGL2: gl.bufferSubData con offset
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.instance_buffer));
            unsafe {
                self.gl.buffer_sub_data_u8_array(
                    glow::ARRAY_BUFFER,
                    offset as i32,
                    bytemuck::cast_slice(&[entity.instance_data]),
                );
            }
        }
    }
}
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "Rust WASM GPU buffer persistent mapping write_buffer subrange update performance 2025"

**Qué buscar:**
- [ ] Diferencias entre `write_buffer` completo vs parcial en wgpu
- [ ] Patrones de dirty tracking en ECS (Entity Component Systems)
- [ ] WebGL2 `bufferSubData` vs `bufferData` performance
- [ ] Estrategias de batching para updates incrementales

#### Estimación
- Investigación: 6 horas
- EntityStore dirty tracking: 8 horas
- GpuRenderer persistent buffering: 10 horas
- WebGL2Renderer partial updates: 8 horas
- Tests de performance: 6 horas
- **Total: 38 horas (~5 días)**

---

### HU-RENDER-008: Camera-Relative Rendering para Zoom Infinito

**Como** usuario de ArchFlow  
**Quiero** que las entidades no "tiemblen" cuando me alejo mucho del origen  
**Para** tener una experiencia fluida al hacer zoom extremo (como en Figma)

#### Criterios de Aceptación
- [ ] Coordenadas de entidades calculadas relativas a la cámara en CPU
- [ ] Conversión automática de `f64` → `f32` sin pérdida de precisión
- [ ] Shaders adaptados para recibir coordenadas relativas
- [ ] Sin jittering visual en zoom 1000x+
- [ ] Compatibilidad con todos los backends (WebGPU, WebGL2)

#### Tareas Técnicas

**1. Implementar conversión de coordenadas en Camera:**
```rust
// EN: archflow-render/src/camera.rs

#[derive(Clone, Debug)]
pub struct Camera {
    /// Posición de la cámara en coordenadas del mundo (f64 para precisión)
    pub position: Vec2f64,

    /// Nivel de zoom (pixels por unidad)
    pub zoom: f32,

    /// Tamaño del viewport
    viewport_size: Vec2,
}

impl Camera {
    /// Convertir posición del mundo a coordenadas relativas a la cámara
    /// Usa f64 internamente, convierte a f32 al final
    pub fn world_to_camera(&self, world_pos: Vec2f64) -> Vec2f32 {
        let relative = world_pos - self.position;
        Vec2::new(relative.x as f32, relative.y as f32)
    }

    /// Generar uniform para shaders con coordenadas relativas
    pub fn to_uniform(&self) -> CameraUniforms {
        CameraUniforms {
            // Convertir posición a f32 (ahora es relativa, sin pérdida)
            position: self.world_to_camera(self.position),
            zoom: self.zoom,
            viewport_size: self.viewport_size,
        }
    }
}
```

**2. Modificar sync_from_store para coordenadas relativas:**
```rust
// EN: archflow-render/src/renderer.rs

impl<G: GraphicsApi> GpuRenderer<G> {
    pub fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize {
        // Pre-calcular coordenadas relativas de la cámara
        let camera_pos_f32 = camera.world_to_camera(camera.position);

        for entity in store.iter_visible(&view_rect) {
            // Convertir posición a coordenadas relativas a la cámara
            let relative_pos = entity.pos - camera.position;
            let pos_f32 = Vec2::new(relative_pos.x as f32, relative_pos.y as f32);

            // Crear instancia con coordenadas relativas
            let instance = GpuInstance {
                transform: Mat4::from_translation(Vec3::new(pos_f32.x, pos_f32.y, 0.0))
                    * Mat4::from_scale(Vec3::splat(entity.size.x * camera.zoom)),
                // ...
            };
            // ...
        }
    }
}
```

**3. Actualizar shaders para coordenadas relativas:**
```wgsl
// EN: archflow-render/shaders/sdf_shapes.wgsl

@vertex
fn vs_main(
    @location(1) instance_position: vec2<f32>,  // Ya es relativa a cámara
    // ...
) -> VertexOutput {
    // Ya no necesitamos restar camera.position
    let world_pos = instance_position + (position * instance_size);
    let screen_pos = (world_pos * camera.zoom) - (camera.viewport_size / 2.0);
    // ...
}
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "floating point precision jitter zoom WebGL GPU camera relative coordinates 2025"

#### Estimación
- Investigación: 8 horas
- Camera refactoring: 8 horas
- sync_from_store modification: 6 horas
- Shader updates: 6 horas
- Tests: 6 horas
- **Total: 34 horas (~4.5 días)**

---

### HU-RENDER-009: Recuperación Automática del Contexto (Context Loss)

**Como** usuario de ArchFlow  
**Quiero** que la aplicación se recupere automáticamente cuando WebGL pierde el contexto  
**Para** no perder mi trabajo al cambiar de pestaña o entrar en modo ahorro de energía

#### Criterios de Aceptación
- [ ] `WasmBridge` detecta eventos `webglcontextlost`
- [ ] Renderer se reinicia automáticamente sin perder estado
- [ ] Texturas y buffers se recargan desde el `EntityStore`
- [ ] Aplicación continúa funcionando tras recuperación
- [ ] Notificación al usuario en modo desarrollo

#### Tareas Técnicas

**1. Implementar ContextHandler en WasmBridge:**
```rust
// EN: archflow-web/src/bridge.rs

#[wasm_bindgen]
impl WasmBridge {
    pub fn register_context_handlers(&self, canvas: HtmlCanvasElement) {
        let on_lost = Closure::wrap(Box::new(move |event: Event| {
            tracing::warn!(target: "archflow::web", "WebGL context lost");
            event.prevent_default();
            Self::schedule_context_recovery(self.js_value());
        }) as Box<dyn FnMut(Event)>);

        canvas.add_event_listener_with_callback(
            "webglcontextlost",
            on_lost.as_ref().unchecked_ref(),
        ).expect("Failed to add contextlost listener");

        self.on_context_lost.set(Some(on_lost));
    }

    fn schedule_context_recovery(bridge: JsValue) {
        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
            Closure::wrap(Box::new(move || {
                let bridge: WasmBridge = bridge.unchecked_into();
                Self::recover_context(&bridge);
            }) as Box<dyn FnMut()>)
            .as_ref().unchecked_ref(),
            100,
        );
    }

    fn recover_context(bridge: &WasmBridge) {
        if let Some(canvas) = bridge.canvas.borrow().as_ref() {
            match RendererSelector::detect_and_create_async(canvas.clone()) {
                Ok(new_renderer) => {
                    if let Some(engine) = bridge.engine.borrow_mut().as_mut() {
                        engine.set_renderer(new_renderer);
                        tracing::info!(target: "archflow::web", "Recovery successful");
                    }
                }
                Err(e) => tracing::error!(target: "archflow::web", error = ?e),
            }
        }
    }
}
```

#### Investigación Previa Requerida (OBLIGATORIA)

**Query para Perplexity:**
> "WebGL context lost event handling preventDefault recovery best practices 2025"

#### Estimación
- Investigación: 6 horas
- Event listeners: 8 horas
- Recovery logic: 8 horas
- RendererSelector modification: 4 horas
- Tests: 6 horas
- **Total: 32 horas (~4 días)**

---

### HU-RENDER-010: Shader Specialization Constants para Multi-Backend

**Como** desarrollador de ArchFlow  
**Quiero** usar constantes de especialización en shaders para optimizar rendimiento por backend  
**Para** que WebGPU use features avanzados mientras WebGL2 usa un pipeline simplificado

#### Criterios de Aceptación
- [ ] WGSL shaders definen constantes con `override` para features
- [ ] WebGPURenderer habilita shadows, lights, effects avanzados
- [ ] WebGL2Renderer desactiva features costosos via PipelineCompilationOptions
- [ ] Naga compila correctamente constantes a GLSL
- [ ] Tests de paridad de rendering entre backends

#### Tareas Técnicas

**1. Definir constantes de especialización en shaders:**
```rust
// EN: archflow-render/src/shaders.rs

#[derive(Clone, Copy)]
pub struct ShaderConstants {
    pub max_lights: u32,
    pub enable_shadows: bool,
    pub enable_aa: bool,
    pub atlas_size: u32,
}

impl Default for ShaderConstants {
    fn default() -> Self {
        Self {
            max_lights: 0,
            enable_shadows: false,
            enable_aa: true,
            atlas_size: 1024,
        }
    }
}

pub const WEBGPU_CONSTANTS: ShaderConstants = ShaderConstants {
    max_lights: 4,
    enable_shadows: true,
    enable_aa: true,
    atlas_size: 2048,
};

pub const WEBGL2_CONSTANTS: ShaderConstants = ShaderConstants {
    max_lights: 0,
    enable_shadows: false,
    enable_aa: true,
    atlas_size: 1024,
};
```

**2. WGSL shaders con override:**
```wgsl
// EN: archflow-render/shaders/sdf_shapes.wgsl

override MAX_LIGHTS: u32 = 0;
override ENABLE_SHADOWS: bool = false;
override ENABLE_AA: bool = true;
```

#### Estimación
- Investigación: 8 horas
- Shader modifications: 10 horas
- PipelineCompilationOptions integration: 8 horas
- WebGL2 defines: 6 horas
- Tests de paridad: 6 horas
- **Total: 38 horas (~5 días)**

---

### HU-RENDER-011: Cola de Carga Asíncrona de Texturas

**Como** usuario de ArchFlow  
**Quiero** que la carga de nuevas texturas no bloqueque el renderizado  
**Para** tener una experiencia fluida mientras se cargan recursos

#### Criterios de Aceptación
- [ ] Sistema de cola asíncrona para carga de texturas
- [ ] Texturas en cola cargadas en background
- [ ] Renderizado continúa durante carga
- [ ] Callback cuando texturas están listas
- [ ] Manejo de errores de carga

#### Tareas Técnicas
Implementar `TextureLoader` con canal `mpsc` para carga en background thread.

#### Estimación
- Investigación: 4 horas
- Implementación cola: 8 horas
- Integración con atlas: 6 horas
- Tests: 4 horas
- **Total: 22 horas (~3 días)**

---

## Plan de Implementación Actualizado

### Fase 1: Foundation (Sprint 1) ✅ Completado
- HU-RENDER-001: Abstracción de Renderer
- HU-RENDER-003: Integración en ArchFlowEngine

### Fase 2: WebGL2 Renderer (Sprint 2-3) ⚠️ En Progreso
- HU-RENDER-002: Implementación WebGL2 (Texture alignment completado)
- HU-RENDER-004: Compilación de shaders (pendiente - marcado como PARTIAL)

### Fase 3: Estabilidad MVP (Sprint 4-5) 🆕 Nuevo
- HU-RENDER-007: Dirty Checking y Buffering Persistente
- HU-RENDER-008: Camera-Relative Rendering
- HU-RENDER-009: Context Loss Recovery

### Fase 4: Optimización (Sprint 6) 🆕 Nuevo
- HU-RENDER-010: Shader Specialization Constants
- HU-RENDER-005: Optimización de Performance (revisado)

### Fase 5: Quality (Sprint 7)
- HU-RENDER-006: Verificación de Paridad
- HU-RENDER-011: Cola Asíncrona de Texturas

## Métricas de Éxito

| Métrica | Target | Actual |
|---------|--------|--------|
| Browser support (WebGPU + WebGL2) | 95%+ | TBD |
| Frame rate @ 100k entities (WebGPU) | 60fps | TBD |
| Frame rate @ 100k entities (WebGL2) | 50fps+ | TBD |
| WASM size increase | < +500KB | TBD |
| Test coverage (render code) | 80%+ | TBD |
| Feature parity | 100% | TBD |

## Análisis de Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Naga no compile WGSL→GLSL correctamente | Alto | Media | Shaders GLSL manuales de respaldo |
| Performance WebGL2 inaceptable | Alto | Baja | Instanced rendering |
| WASM size excede presupuesto | Medio | Media | Profile dependencies |
| Feature drift entre backends | Alto | Baja | Tests de regresión |

## Deuda Técnica a Registrar

1. **Tests sin GPU**: Investigar cómo correr tests en CI/CD
2. **Canvas2D fallback**: Considerar si vale la pena implementar
3. **Profile tools**: Herramientas de profiling

## Dependencies Externas

| Crate | Versión | Uso |
|-------|---------|-----|
| `wgpu` | 0.20+ | WebGPU (existente) |
| `naga` | 22+ | WGSL → GLSL |
| `glow` | 0.15+ | WebGL2 bindings |

## Referencias

- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
- [WebGL2 Specification](https://www.khronos.org/webgl/)
- [Naga Shader Compiler](https://github.com/gfx-rs/naga)
- [Glow WebGL2](https://github.com/grovesNL/glow)

## Changelog

| Fecha | Cambio | Autor |
|-------|--------|-------|
| 2026-02-04 | Actualización: Añadidas HU-RENDER-007 a HU-RENDER-011 basadas en crítica técnica | @rubentxu |
| 2026-02-04 | Implementación completa de HU-RENDER-001 | @rubentxu |
| 2026-02-03 | Creación inicial con arquitectura real | @team |
| 2026-02-03 | Actualización con crates existentes | @team |
