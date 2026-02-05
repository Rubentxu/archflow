# Arquitectura Final: ArchFlow Engine v3.0
## Especificación Técnica Completa con MVP Funcional

**Versión:** 3.0 (MVP Ready - Complete)  
**Fecha:** Enero 2026  
**Target:** WASM32 (WebGPU/WebGL2 fallback)  
**Objetivo:** Motor de diagramación C4 profesional con 100k objetos @ 60FPS

---

## 📋 Tabla de Contenidos Completo

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Principios Arquitectónicos Fundamentales](#2-principios-arquitectonicos-fundamentales)
3. [Estructura de Crates: Bounded Contexts DDD](#3-estructura-de-crates-bounded-contexts-ddd)
4. [Capa de Datos: EntityStore (SoA Estricto + Hierarchy)](#4-capa-de-datos-entitystore-soa-estricto--hierarchy)
5. [Sistema de Comandos (Command-Driven)](#5-sistema-de-comandos-command-driven)
6. [Cámara 2D Infinita](#6-camara-2d-infinita)
7. [Sistema de Input (SharedArrayBuffer Lock-Free)](#7-sistema-de-input-sharedarraybuffer-lock-free)
8. [Spatial Indexing (Grid Hash + Hierarchy)](#8-spatial-indexing-grid-hash--hierarchy)
9. [Pipeline de Renderizado: Multi-Phase Instancing](#9-pipeline-de-renderizado-multi-phase-instancing)
10. [Texture Atlas Dinámico (Shelf Packing)](#10-texture-atlas-dinamico-shelf-packing)
11. [Sistema de Conexiones Magnéticas](#11-sistema-de-conexiones-magneticas)
12. [Sistema de Texto (MTSDF + Pre-shaping)](#12-sistema-de-texto-mtsdf--pre-shaping)
13. [Hit Testing (O(1) Spatial Query)](#13-hit-testing-o1-spatial-query)
14. [Carga de Iconos (Draw.io Parser)](#14-carga-de-iconos-drawio-parser)
15. [UI y Gizmos (Immediate Mode)](#15-ui-y-gizmos-immediate-mode)
16. [Undo/Redo (Command Sourcing)](#16-undoredo-command-sourcing)
17. [Colaboración en Tiempo Real (CRDT)](#17-colaboracion-en-tiempo-real-crdt)
18. [Motor de Curvas Bézier (GPU-Based)](#18-motor-de-curvas-bezier-gpu-based)
19. [Serialización Zero-Copy](#19-serializacion-zero-copy)
20. [Exportación IaC (Terraform/Mermaid)](#20-exportacion-iac-terraformmermaid)
21. [Flujo del Frame (El Tick Integrado)](#21-flujo-del-frame-el-tick-integrado)
22. [Optimización de Compilación](#22-optimizacion-de-compilacion)
23. [Métricas de Validación](#23-metricas-de-validacion)
24. [Roadmap de Implementación](#24-roadmap-de-implementacion)

---

## 1. Resumen Ejecutivo

ArchFlow es un motor de diagramación vectorial profesional construido con Rust y WebAssembly, diseñado para Solutions Architects que necesitan modelar infraestructuras complejas con miles de componentes.

### Objetivo Técnico

Superar a Figma en rendimiento mediante:

- **Data-Oriented Design (DOD)**: Structure of Arrays (SoA) para máxima eficiencia de caché
- **Zero-Allocation Hot Path**: Ninguna allocation durante el frame de renderizado
- **WebGPU Native**: Multi-Phase Instancing para 100k objetos @ 60FPS
- **Hexagonal Architecture**: Separación estricta entre dominio, aplicación e infraestructura
- **Domain-Driven Design**: Bounded Contexts para diagramas C4, motor de renderizado, interacción

### Características Diferenciadoras vs Figma

| Métrica | Target | Figma (Referencia) |
|---------|--------|-------------------|
| Objetos @ 60FPS | 100,000 | ~10,000 (Canvas) |
| Latencia de Input | <8ms | ~16ms |
| Binary Size (gzipped) | <500KB | ~800KB |
| Memory Heap | <64MB | ~100MB |
| Zoom Infinity | Sí (MSDF + Vector) | Sí |
| Librerías Nativas | AWS/Azure/GCP (Draw.io) | Manual |
| Exportación IaC | Terraform/Mermaid | No |
| Conexiones | Orthogonal magnético | Manual |

---

## 2. Principios Arquitectónicos Fundamentales

### 2.1. Constraints de Diseño WASM

```
┌─────────────────────────────────────────────────────────────┐
│                  CONSTRAINTS CRÍTICOS WASM                  │
├─────────────────────────────────────────────────────────────┤
│  ❌ ZERO ALLOCATION en hot path                             │
│     - Ningún malloc durante tick() o render()              │
│     - Pre-allocation de todos los buffers                   │
│                                                             │
│  ❌ SINGLE THREADED (por ahora)                             │
│     - WebWorkers para cálculo pesado (texto, CRDT)          │
│     - Main thread solo para orquestación                    │
│                                                             │
│  │ FFI MINIMIZATION                                        │
│     - Máximo 1 crossing JS/WASM por frame                   │
│     - SharedArrayBuffer para input lock-free                │
│                                                             │
│  ❌ MEMORY BUDGET CONTROLADO                                │
│     - Heap inicial: 32MB, máximo 64MB                       │
│     - String Pool flat para evitar fragmentación            │
└─────────────────────────────────────────────────────────────┘
```

### 2.2. Patrones Arquitectónicos Aplicados

| Patrón | Propósito | Implementación |
|--------|-----------|----------------|
| **Hexagonal Architecture** | Separar dominio de infraestructura | Ports & Adapters entre contextos |
| **Data-Oriented Design** | Maximizar cache locality | Structure of Arrays (SoA) estricto |
| **Command Pattern** | Encapsular intenciones de cambio | Comandos Copy, ≤16 bytes |
| **CQRS** | Separar lectura de escritura | Queries vs Commands en Store |
| **Event Sourcing** | Reconstruir estado y sincronizar | Domain Events para CRDT |
| **Immediate Mode** | UI sin estado retained | Gizmos generados cada frame |

### 2.3. Stack Tecnológico

```toml
[workspace]
members = [
    "archflow-core",      # Tipos base, IDs, matemáticas (no_std compatible)
    "archflow-engine",    # EntityStore SOA, SpatialHash, Commands
    "archflow-render",    # WebGPU, Shaders, MSDF Atlas
    "archflow-interaction", # HitTesting, Camera, Input Processor
    "archflow-plugins",   # Draw.io parser, SVG rasterizer, IaC generators
    "archflow-export",    # Terraform, Mermaid, FlatBuffers serialization
    "archflow-web",       # WASM bridge, Loop principal, JS bindings
]
resolver = "2"

[profile.release]
lto = true
opt-level = "z"        # Optimizado para tamaño de binario WASM
codegen-units = 1      # Código monolítico (más lento de compilar, más rápido)
panic = "abort"        # Eliminar unwinding support
strip = true           # Eliminar símbolos de debug

[profile.release.package."*"]
opt-level = 3          # Dependencias con optimización máxima

[dependencies]
# ═══════════════════════════════════════════════════════════
# Core (no_std compatible)
# ═══════════════════════════════════════════════════════════
glam = { version = "0.25", features = ["vec2", "mat4"] }     # Matemáticas alto rendimiento
heapless = "0.8"                                            # Arrays fijos, sin alloc en hot path
micromath = "2.1"                                           # Math rápido para WASM (alternativa ligera)
bitflags = "2.4"                                              # Máscaras de estado eficiente

# ════════════════════════════════════════════════════════════
# Data Layer (manejo de memoria y caché)
# ════════════════════════════════════════════════════════════
fixedbitset = "0.5"                                            # Dirty tracking O(1) con bitsets compactos
bumpalo = "3.14"                                              # Arena allocation para comandos temporales

# ════════════════════════════════════════════════════════════
# Rendering (WebGPU + rasterización)
# ════════════════════════════════════════════════════════════
wgpu = { version = "0.20", features = ["webgl"] }              # WebGPU + WebGL2 fallback
bytemuck = { version = "1.14", features = ["derive"] }   # Cast seguro de slices a/desde GPU
resvg = { version = "0.36", default-features = false }     # SVG rasterization (tiny-skia backend)

# ════════════════════════════════════════════════════════════
# Text (shaping y layout - sin runtime allocations)
# ══════════════════════════════════════════════════════════
cosmic-text = { version = "0.11", default-features = false, features = ["swash"] }  # Solo shaping, no raster
# lru = "0.12"                                                   # Cache de layouts de texto

# ════════════════════════════════════════════════════════════
# Plugins (integraciones externas)
# ══════════════════════════════════════════════════════════
quick-xml = "0.31"                                             # Parser XML rápido y ligero (Draw.io)
flate2 = "1.0"                                                 # Descompresión Deflate (Draw.io comprimido)
base64 = "0.21"                                                # Decodificación Base64 (Draw.io encoded)
percent-encoding = "2.3"                                      # URL decoding (Draw.io)

# ════════════════════════════════════════════════════════════
# WASM Bridge (comunicación con JavaScript)
# ════════════════════════════════════════════════════════════
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "HtmlCanvasElement", "HtmlImageElement", "CanvasRenderingContext2d",
    "PointerEvent", "WheelEvent", "KeyboardEvent", "OffscreenCanvas",
    "Window", "Document", "Element", "Event", "InputEvent"
] }

# ════════════════════════════════════════════════════════════
# Serialización (zero-copy read/write)
# ════════════════════════════════════════════════════════════
flatbuffers = "23.5"                                            # Serialización cero-copia para archivos .af

# ════════════════════════════════════════════════════════════
# Colaboración (opcional - futuro)
# ══════════════════════════════════════════════════════════
loro = "0.1"                                                    # o implementación custom con Lamport timestamps
```

---

## 3. Estructura de Crates: Bounded Contexts DDD

```
┌─────────────────────────────────────────────────────────────────────┐
│                    archflow-web (Composition Root)                   │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐     │
│   │   Bridge     │  │  Event Loop  │  │   Dependency        │     │
│   │  (WASM/JS)   │  │  (rAF)       │  │   Injection         │     │
│   └──────────────┘  └──────────────┘  └──────────────────────┘     │
└──────────────────────┬──────────────────────────────────────────────┘
                       │ Dependency direction (infrastructure depends on domain)
┌──────────────────────▼──────────────────────────────────────────────┐
│                  archflow-interaction (Application)                  │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐     │
│   │   Tools      │  │   History    │  │   Command Handlers  │     │
│ │(Selection,   │  │  (Undo)      │  │   (Use Cases)        │     │
│ │  Pan, Pen)   │  │              │  │                      │     │
│   └──────────────┘  └──────────────┘  └──────────────────────┘     │
└──────────────────────┬──────────────────────────────────────────────┘
                       │ uses
┌──────────────────────▼──────────────────────────────────────────────┐
│                   archflow-engine (Data Layer)                       │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐     │
│ │  EntityStore │  │ SpatialHash  │  │  ConnectionStore     │     │
│ │  (SoA Impl)  │  │  (Grid)      │  │  (Anchor-Based)      │     │
│   └──────────────┘  └──────────────┘  └──────────────────────┘     │
└──────────────────────┬──────────────────────────────────────────────┘
                       │ Implements ports defined by:
┌──────────────────────▼──────────────────────────────────────────────┐
│                   archflow-render (Infrastructure)                   │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐     │
│ │   GpuRenderer │  │  AtlasPack   │  │   Text System        │     │
│ │(Multi-Phase) │ │  (Shelf)     │  │   (MTSDF)            │     │
│   └──────────────┘  └──────────────┘  └──────────────────────┘     │
└──────────────────────┬──────────────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────────────┐
│                    archflow-diagram (Domain Core)                    │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐     │
│ │   C4 Model   │  │   Commands   │  │   Domain Events      │     │
│ │ (Aggregates) │  │   (Pure)     │  │   (Facts)            │     │
│   └──────────────┘  └──────────────┘  └──────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
                       │
         ┌─────────────┴─────────────┐
         │                           │
┌────────▼──────────┐    ┌───────────▼────────────┐
│ archflow-text     │    │ archflow-core          │
│ (Supporting Sub)  │    │ (Shared Kernel)        │
└───────────────────┘    └────────────────────────┘
```

### 3.1. Diagrama de Dependencias

```
archflow-diagram (Domain)
    ↑
archflow-engine (Data) ←──┐
    ↑                      │
archflow-interaction ─────┘
    ↑
archflow-render
    ↑
archflow-web (Composition Root)
```

**Regla de Oro**: El dominio (`diagram`) NO depende de `std`, `wgpu`, ni `web-sys`.

### 3.2. Shared Kernel (`archflow-core`)

Este crate contiene tipos **puros, inmutables y Copy** que pueden usarse en `no_std`:

```rust
// archflow-core/src/lib.rs

pub mod math;
pub use math::{Vec2, Vec3, Mat4, Rect, Color};

pub mod id;
pub use id::{EntityId, Generation, Index};

pub mod vo;
pub use vo::{Position, Size, Transform, Bounds};

pub mod ports;
pub use ports::{StorePort, CanvasPort, EventPublisher};
```

---

## 4. Capa de Datos: EntityStore (SoA Estricto + Transform Hierarchy)

### 4.1. Layout de Memoria Optimizado

```rust
// archflow-engine/src/store.rs

pub const MAX_ENTITIES: usize = 100_000;
pub const MAX_GLYPHS: usize = 500_000;
pub const MAX_CONNECTIONS: usize = 200_000;
pub const MAX_TEXT_LENGTH: usize = 50_000;  // Caracteres totales en pool

/// Almacenamiento Structure of Arrays con bit-packing agresivo
/// 
/// Optimizaciones aplicadas:
/// - 64-byte alignment para aprovechar lecturas SIMD
/// - Bit-packing de metadata en u32 (ahorra ~12 bytes por entidad vs structs separadas)
/// - Transform hierarchy para soportar grupos/frames sin penalizar rendimiento
pub struct EntityStore {
    // ═══════════════════════════════════════════════════════════
    // HOT DATA (Cache Line 0-2): Accessed every frame by render
    // ═════════════════════════════════════════════════════════════
    
    // Transform (64 bytes alineados para máximo throughput SIMD)
    pub transforms: Vec<[f32; 4]>,      // [x, y, w, h] - 16 bytes
    
    // Metadata empaquetado (ahorra ~40% de memoria vs usar structs)
    // Layout: [shape:4 | layer:4 | visibility:1 | selected:1 | locked:1 | padding:21]
    pub metadata: Vec<u32>,
    
    // Colores (directo para GPU, empaquetado)
    pub colors: Vec<u32>,              // 0xRRGGBBAA packed
    
    // Textura (para iconos/images del Atlas)
    pub texture_index: Vec<u16>,        // 0 = color sólido (SDF), 1..N = índice en atlas
    pub uv_rects: Vec<[f32; 4]>,       // [u, v, w, h] normalizados en atlas
    
    // Color de tinte (para feedback visual de selección, filtros)
    pub color_tints: Vec<[f32; 4]>,    // RGBA para tint por instancia
    
    // Texto (índices al glyph buffer flat)
    pub text_glyph_start: Vec<u32>,   // Índice en buffer global de glyphs
    pub text_glyph_count: Vec<u16>,   // Cantidad de glyphs
    pub text_scale: Vec<f32>,         // Tamaño de fuente para MSDF
    
    // ═════════════════════════════════════════════════════════════
    // TRANSFORM HIERARCHY (NUEVO en V2.0)
    // ═══════════════════════════════════════════════════════════
    
    pub parent_id: Vec<Option<EntityId>>,  // Para grouping/frames
    pub local_transform: Vec<[f32; 4]>,   // Transform relativo al padre
    pub world_transform: Vec<[f32; 4]>,   // Cache de world space (actual render position)
    pub dirty_hierarchy: FixedBitSet,      // Marcado cuando padre se mueve
    
    // ═══════════════════════════════════════════════════════════
    // COLD DATA (Acceso solo al seleccionar/inspeccionar - separado del hot path)
    // ═══════════════════════════════════════════════════════════
    
    pub arch_data: Vec<Option<Box<ArchitectureData>>>,
    pub string_pool: StringPool,
    
    // ═══════════════════════════════════════════════════════════
    // MANAGEMENT (Infraestructura)
    // ═════════════════════════════════════════════════════════════
    
    generations: Vec<u8>,                // Validación de EntityId generacional
    free_list: Vec<u32>,               // Stack LIFO de índices libres
    alive_count: usize,               // Número de entidades vivas
    
    // Dirty Tracking (FixedBitSet para O(1) operations)
    dirty_transform: FixedBitSet,     // Para Spatial Hash update
    dirty_render: FixedBitSet,        // Para GPU upload
    dirty_text: FixedBitSet,          // Para text layout recalcular
    
    // Z-Order (Indirection Layer para render order)
    draw_order: Vec<u32>,             // [idx0, idx1, ...] orden visual
    dirty_z_order: bool,               // Marcado cuando cambia el orden
    
    // Command Queue (Pre-allocated, reutilizado)
    command_queue: heapless::Vec<Command, 1024>,  // Buffer de comandos sin alloc
}

// Buffer global de glyphs (compartido por todas las entidades)
pub struct GlyphBuffer {
    pub pos_local: Vec<[f32; 2]>,     // Offset local dentro de entidad padre
    pub uv: Vec<[f32; 4]>,            // UV en el MSDF/Texture atlas
}
```

### 4.2. Helpers de Bit-Packing

```rust
impl EntityStore {
    /// Obtiene el tipo de forma del bitfield de metadata
    #[inline(always)]
    pub fn shape_type(&self, idx: usize) -> u8 {
        (self.metadata[idx] & 0xF) as u8
    }
    
    /// Establece el tipo de forma (forma básica: 0=Rect, 1=Circle, etc.)
    #[inline(always)]
    pub fn set_shape_type(&mut self, idx: usize, shape: u8) {
        // Preservar todos los demás bits, solo cambiar los 4 menores
        self.metadata[idx] = (self.metadata[idx] & !0xF) | (shape as u32 & 0xF);
    }
    
    /// Verifica si la entidad es visible
    #[inline(always)]
    pub fn is_visible(&self, idx: usize) -> bool {
        (self.metadata[idx] & (1 << 8)) != 0
    }
    
    /// Establece visibilidad (bit 8)
    #[inline(always)]
    pub fn set_visible(&mut self, idx: usize, visible: bool) {
        if visible {
            self.metadata[idx] |= 1 << 8;
        } else {
            self.metadata[idx] &= !(1 << 8);
        }
    }
    
    /// Verifica si la entidad está seleccionada
    #[inline(always)]
    pub fn is_selected(&self, idx: usize) -> bool {
        (self.metadata[idx] & (1 << 9)) != 0
    }
    
    /// Establece selección (bit 9) - actualiza también el color_tint
    #[inline(always)]
    pub fn set_selected(&mut self, idx: usize, selected: bool) {
        if selected {
            self.metadata[idx] |= 1 << 9;
            // Feedback visual: tinte azulado suave para indicar selección
            self.color_tints[idx] = [0.7, 0.8, 1.0, 1.0];
        } else {
            self.metadata[idx] &= !(1 << 9);
            // Restaurar color normal
            self.color_tints[idx] = [1.0, 1.0, 1.0, 1.0];
        }
    }
    
    /// Obtiene el layer de z-index (bits 4-7)
    #[inline(always)]
    pub fn layer(&self, idx: usize) -> u8 {
        ((self.metadata[idx] >> 4) & 0xF) as u8
    }
}
```

### 4.3. String Pool (Zero-Allocation Strings)

```rust
// archflow-engine/src/string_pool.rs

/// String Pool plano para evitar Vec<String> en WASM
/// 
/// Problema de Vec<String>:
/// - 10,000 entidades = 10,000 allocations en heap
/// - Cada String tiene 24 bytes overhead + capacity
/// - Cache misses al iterar (memoria dispersa)
/// 
/// Solución String Pool:
/// - Un solo Vec<u8> conteniendo todos los strings concatenados
/// - Tabla de offsets (start, len) por EntityId
pub struct StringPool {
    buffer: Vec<u8>,                  // Todos los strings concatenados
    offsets: Vec<(usize, usize)>,     // (start, len) por EntityId
    free_list: Vec<usize>,            // Slots libres para reuse
}

impl StringPool {
    pub fn with_capacity(entities: usize, total_chars: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(total_chars),
            offsets: vec![(0, 0); entities],
            free_list: Vec::new(),
        }
    }
    
    pub fn set(&mut self, entity_idx: usize, text: &str) {
        let bytes = text.as_bytes();
        let start = self.buffer.len();
        self.buffer.extend_from_slice(bytes);
        self.offsets[entity_idx] = (start, bytes.len());
    }
    
    #[inline(always)]
    pub fn get(&self, entity_idx: usize) -> &str {
        let (start, len) = self.offsets[entity_idx];
        unsafe {
            std::str::from_utf8_unchecked(
                &self.buffer[start..start + len]
            )
        }
    }
}
```

---

## 5. Sistema de Comandos (Command-Driven)

### 5.1. Comandos con Soporte para Hierarchy

```rust
// archflow-diagram/src/commands.rs

/// Comandos de dominio (Plain Old Data, Copy)
/// 
/// Reglas:
/// - Máximo 16 bytes para eficiencia de caché
/// - Sin Box, String, o Vec (usar índices u32)
/// - #[repr(C, u8)] para layout predecible y padding correcto
#[repr(C, u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    // ═══════════════════════════════════════════════════════════
    // CREACIÓN / DESTRUCCIÓN
    // ═════════════════════════════════════════════════════════════
    Spawn {
        pos: Vec2,      // 8 bytes
        size: Vec2,     // 8 bytes
        parent: Option<EntityId>, // 4 bytes
    } = 0,
    
    Despawn(EntityId) = 1,  // 4 bytes
    
    // ═══════════════════════════════════════════════════════════
    // TRANSFORMACIÓN (Hot Path)
    // ═══════════════════════════════════════════════════════════
    Move {
        id: EntityId,    // 4 bytes
        delta: Vec2,     // 8 bytes
    } = 2,
    
    Teleport {
        id: EntityId,    // 4 bytes
        pos: Vec2,       // 8 bytes
    } = 3,
    
    Resize {
        id: EntityId,    // 4 bytes
        size: Vec2,      // 8 bytes
    } = 4,
    
    // NUEVO: Mover grupo completo (jerarquía)
    MoveGroup {
        root_id: EntityId,  // 4 bytes - mover este y todos sus descendientes
        delta: Vec2,       // 8 bytes
    } = 5,
    
    // ══════════════════════════════════════════════════════════════
    // ESTILO (Hot Path)
    // ════════════════════════════════════════════════════════════
    SetColor {
        id: EntityId,    // 4 bytes
        color: u32,      // 4 bytes (0xRRGGBBAA)
    } = 6,
    
    SetShape {
        id: EntityId,    // 4 bytes
        shape: u8,       // 1 byte
    } = 7,
    
    // ════════════════════════════════════════════════════════════
    // TEXTURA / TEXTURE (Cold Path)
    // ════════════════════════════════════════════════════════════
    SetText {
        id: EntityId,         // 4 bytes
        text_hash: u64,       // 8 bytes (hash para String Pool)
    } = 8,
    
    // NUEVO: Para iconos/images del Atlas
    SetTexture {
        id: EntityId,           // 4 bytes
        texture_index: u16,     // 2 bytes
        uv_rect: [f32; 4],     // 16 bytes
    } = 9,
    
    SetTextScale {
        id: EntityId,    // 4 bytes
        scale: f32,      // 4 bytes
    } = 10,
    
    // ════════════════════════════════════════════════════════════
    // ARQUITECTURA (Domain Specific - C4)
    // ════════════════════════════════════════════════════════════
    SetC4Level {
        id: EntityId,    // 4 bytes
        level: u8,       // 1 byte (System=0, Container=1, Component=2)
    } = 11,
    
    SetCloudProvider {
        id: EntityId,    // 4 bytes
        provider: u8,    // 1 byte (AWS=0, GCP=1, Azure=2)
    } = 12,
}

// Static assertions para asegurar tamaño ≤16 bytes
const _: [(); 16] = [(); std::mem::size_of::<Command>()];
```

### 5.2. Command Queue

```rust
// archflow-engine/src/command_queue.rs

use heapless::Vec as StackVec;

/// Cola de comandos pre-allocada (sin allocations en hot path)
pub struct CommandQueue {
    buffer: StackVec<Command, 1024>,  // Capacidad fija de 1024 comandos
}

impl CommandQueue {
    pub fn push(&mut self, cmd: Command) -> bool {
        self.buffer.push(cmd).is_ok()
    }
    
    pub fn drain(&mut self) -> impl Iterator<Item = Command> + '_ {
        self.buffer.iter().copied()
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
```

---

## 6. Cámara 2D Infinita

### 6.1. Estructura de la Cámara

```rust
// archflow-render/src/camera.rs

use archflow_core::Vec2;
use glam::Mat4;

/// Cámara ortográfica 2D para diagramas infinitos
/// 
/// Características:
/// - Zoom infinito (0.01x a 100x)
/// - Pan ilimitado (mundo infinito)
/// - Zoom hacia el cursor (como Figma/Google Maps)
/// - Viewport culling automático con SpatialHash
pub struct Camera {
    /// Centro de la cámara en coordenadas de mundo
    pub center: Vec2,
    
    /// Nivel de zoom (1.0 = 100%, 0.5 = 200%, 2.0 = 50%)
    pub zoom: f32,
    
    /// Relación de aspecto de la ventana (width / height)
    pub aspect_ratio: f32,
    
    /// Planos cercano/lejano para ortografía (para efectos de profundidad futuros)
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            aspect_ratio: width / height,
            near: -1.0,
            far: 1.0,
        }
    }
    
    /// Construye la matriz View-Projection para el shader
    /// 
    /// Esta matriz se sube al Uniform Buffer de WebGPU cada frame
    /// y transforma las coordenadas de mundo a coordenadas de clip space
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // Calcular medio ancho y alto de la vista en coordenadas de mundo
        let half_height = 1.0 / self.zoom;
        let half_width = half_height * self.aspect_ratio;
        
        // Matriz ortográfica 2D (right-handed, Y hacia arriba)
        Mat4::orthographic_rh(
            self.center.x - half_width,  // left
            self.center.x + half_width,  // right
            self.center.y - half_height,  // bottom
            self.center.y + half_height,  // top
            self.near,
            self.far,
        )
    }
    
    /// Convertir coordenadas de pantalla a coordenadas de mundo
    /// 
    /// Útil para:
    /// - Hit testing (convertir posición del mouse)
    /// - Posicionar nuevas entidades
    /// - Snap to grid
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
        // Normalizar a coordenadas de dispositivo normalizado (NDC) [-1, 1]
        let ndc = (screen_pos / screen_size) * 2.0 - Vec2::ONE;
        
        // Aplicar zoom inverso para obtener coordenadas de mundo
        let half_height = 1.0 / self.zoom;
        let half_width = half_height * self.aspect_ratio;
        
        Vec2::new(
            self.center.x + ndc.x * half_width,
            self.center.y + ndc.y * half_height,
        )
    }
    
    /// Convertir coordenadas de mundo a coordenadas de pantalla
    pub fn world_to_screen(&self, world_pos: Vec2, screen_size: Vec2) -> Vec2 {
        let half_height = 1.0 / self.zoom;
        let half_width = half_height * self.aspect_ratio;
        
        // Primero a mundo normalizado
        let ndc = Vec2::new(
            (world_pos.x - self.center.x) / half_width,
            (world_pos.y - self.center.y) / half_height,
        );
        
        // Luego a pantalla
        (ndc * 0.5 + 0.5) * screen_size
    }
    
    /// Obtener el rectángulo visible de la cámara en coordenadas de mundo
    /// 
    /// Útil para:
    /// - Viewport culling (solo renderizar lo visible)
    /// - Determinar qué iconos cargar en lazy loading
    pub fn viewport_bounds(&self) -> Rect {
        let half_height = 1.0 / self.zoom;
        let half_width = half_height * self.aspect_ratio;
        
        Rect::from_center_size(
            self.center,
            Vec2::new(half_width * 2.0, half_height * 2.0),
        )
    }
}
```

### 6.2. Controlador de Cámara con Zoom-to-Cursor

```rust
// archflow-interaction/src/camera_controller.rs

use archflow_core::Vec2;
use archflow_render::Camera;

const ZOOM_MIN: f32 = 0.01;   // 1% zoom (muy alejado)
const ZOOM_MAX: f32 = 100.0;  // 10000% zoom (muy cercano)
const ZOOM_INTENSITY: f32 = 0.001;  // Sensibilidad del zoom

pub struct CameraController {
    drag_start: Option<Vec2>,
    last_position: Vec2,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            drag_start: None,
            last_position: Vec2::ZERO,
        }
    }
    
    /// Manejar rueda del ratón con zoom hacia el cursor
    /// 
    /// Esta es la característica clave que diferencia a una herramienta profesional
    /// de una amateur: el zoom debe ir hacia donde está el cursor, no al centro.
    pub fn on_wheel(
        &mut self,
        delta_y: f32,
        mouse_screen: Vec2,
        camera: &mut Camera,
        screen_size: Vec2,
    ) {
        let old_zoom = camera.zoom;
        
        // Calcular nuevo zoom con límites e intensidad
        camera.zoom *= 1.0 + (-delta_y * ZOOM_INTENSITY);
        camera.zoom = camera.zoom.clamp(ZOOM_MIN, ZOOM_MAX);
        
        // La fórmula mágica del zoom-to-cursor:
        // center_new = center_old + (mouse_world - center_old) * (1 - old_zoom/new_zoom)
        let mouse_world = camera.screen_to_world(mouse_screen, screen_size);
        let zoom_ratio = old_zoom / camera.zoom;
        
        camera.center = camera.center + (mouse_world - camera.center) * (1.0 - zoom_ratio);
    }
    
    /// Manejar pan (arrastrar con click derecho o espacio+drag)
    pub fn on_drag(
        &mut self,
        mouse_screen: Vec2,
        delta: Vec2,
        camera: &mut Camera,
        screen_size: Vec2,
    ) {
        // Convertir delta de pantalla a mundo
        let half_height = 1.0 / camera.zoom;
        let half_width = half_height * camera.aspect_ratio;
        
        let world_delta = Vec2::new(
            delta.x * (2.0 * half_width) / screen_size.x,
            delta.y * (2.0 * half_height) / screen_size.y,
        );
        
        camera.center -= world_delta;
    }
}
```

---

## 7. Sistema de Input (SharedArrayBuffer Lock-Free)

### 7.1. SharedArrayBuffer entre JS y WASM

```rust
// archflow-web/src/input_bridge.rs

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

const MAX_POINTERS: usize = 8;      // Máximo 8 punteros simultáneos
const EVENT_CAPACITY: usize = 128;    // Buffer de eventos por frame

/// Estructura de evento crudo para lock-free passing
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawInputEvent {
    pub timestamp: u64,       // Para delta time preciso (performance.now())
    pub pointer_id: u32,      // Identificador del puntero (para multitouch)
    pub x: f32,               // Coordenada X en pantalla (píxeles)
    pub y: f32,               // Coordenada Y en pantalla (píxeles)
    pub pressure: f32,         // Presión del puntero (0.0 a 1.0)
    pub event_type: u8,        // 0=Down, 1=Move, 2=Up, 3=Wheel, 4=KeyDown
    pub buttons: u8,           // Bitmask de botones del ratón
    pub modifiers: u8,         // Bitmask: Shift=1, Ctrl=2, Alt=4
    _padding: u8,             // Padding para alineación a 8 bytes
}

/// Ring buffer lock-free entre JS (productor) y WASM (consumer)
pub struct InputRingBuffer {
    head: AtomicU32,          // JS escribe (head pointer)
    tail: AtomicU32,          // WASM lee (tail pointer)
    data: [RawInputEvent; EVENT_CAPACITY],
}

impl InputRingBuffer {
    pub fn new() -> Self {
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            data: unsafe { std::mem::zeroed() },
        }
    }
    
    /// Llamado desde JS vía wasm-bindgen (no debe allocar en hot path)
    pub fn push_event(&self, event: RawInputEvent) -> bool {
        let head = self.head.load(Ordering::Acquire) as usize;
        let tail = self.tail.load(Ordering::Acquire) as usize;
        let next = (head + 1) % EVENT_CAPACITY;
        
        if next == tail {
            return false;  // Buffer lleno - aplicar backpressure
        }
        
        self.data[head] = event;
        self.head.store(next as u32, Ordering::Release);
        true
    }
    
    /// Consumir todos los eventos en el tick (O(n) donde n=pocos eventos)
    pub fn drain(&self) -> impl Iterator<Item = RawInputEvent> + '_ {
        let mut tail = self.tail.load(Ordering::Acquire) as usize;
        let head = self.head.load(Ordering::Acquire) as usize;
        
        std::iter::from_fn(move || {
            if tail == head {
                None
            } else {
                let event = self.data[tail];
                tail = (tail + 1) % EVENT_CAPACITY;
                self.tail.store(tail as u32, Ordering::Release);
                Some(event)
            }
        })
    }
}
```

### 7.2. JS Side con Coalescing Crítico

```javascript
// web/input_handler.js

// SHARED ARRAY BUFFER SETUP
const SAB_SIZE = 4096;  // 4KB para eventos (más que suficiente)
const buffer = new SharedArrayBuffer(SAB_SIZE);
const inputView = new DataView(buffer);
const headPtr = 0;
const tailPtr = 4;
const dataOffset = 8;
const EVENT_SIZE = 32;  // sizeof(RawInputEvent)

// COALESCING - ESENCIAL para performance
canvas.addEventListener('pointermove', (e) => {
    // El navegador nos da eventos coalescidos (agrupados)
    if (e.getCoalescedEvents) {
        for (const evt of e.getCoalescedEvents()) {
            writeEvent(evt);
        }
    } else {
        writeEvent(e);
    }
    
    // Notificar a WASM que hay datos disponibles
    Atomics.notify(inputView, tailPtr);
});

function writeEvent(evt) {
    const head = Atomics.load(inputView, headPtr);
    const next = (head + EVENT_SIZE) % (SAB_SIZE - dataOffset);
    
    // Escribir campos del evento
    inputView.setFloat64(dataOffset + head + 0, performance.now(), true);
    inputView.setUint32(dataOffset + head + 8, evt.pointerId, true);
    inputView.setFloat32(dataOffset + head + 12, evt.clientX, true);
    inputView.setFloat32(dataOffset + head + 16, evt.clientY, true);
    inputView.setFloat32(dataOffset + head + 20, evt.pressure || 0, true);
    inputView.setUint8(dataOffset + head + 24, eventTypeToByte(evt), true);
    inputView.setUint8(dataOffset + head + 25, evt.buttons || 0, true);
    inputView.setUint8(dataOffset + head + 26, modifiersToByte(evt), true);
    
    Atomics.store(inputView, headPtr, next);
}

function eventTypeToByte(evt) {
    // Mapping de tipos de evento a bytes
    switch (evt.type) {
        case 'mousedown': return 0;
        case 'mousemove': return 1;
        case 'mouseup': return 2;
        case 'wheel': return 3;
        case 'keydown': return 4;
        default: return 1;
    }
}

function modifiersToByte(evt) {
    let byte = 0;
    if (evt.shiftKey) byte |= 0x01;
    if (evt.ctrlKey) byte |= 0x02;
    if (evt.altKey) byte |= 0x04;
    return byte;
}
```

---

## 8. Spatial Indexing (Grid Hash + Hierarchy)

### 8.1. SpatialHash con Soporte para Jerarquías

```rust
// archflow-engine/src/spatial_hash.rs

use archflow_core::EntityId;
use hashbrown::HashMap;
use heapless::Vec as StackVec;

const CELL_SIZE: f32 = 128.0;           // Tamaño de celda en world units
const MAX_ENTITIES_PER_CELL: usize = 16; // Para mantener arrays pequeños

/// Spatial Hash optimizado para WASM
/// 
/// Ventajas vs R-Tree:
/// - O(1) inserción vs O(log n) del R-Tree
/// - Sin allocaciones dinámicas en query
/// - Cache-friendly (arrays contiguos en celdas)
/// - Binary size < 5KB vs 50KB de rstar
pub struct SpatialHash {
    // HashMap de coordenadas de celda → entidades en esa celda
    cells: HashMap<(i16, i16), StackVec<EntityId, MAX_ENTITIES_PER_CELL>>,
    
    // Mapeo entidad → celda actual (para O(1) remove)
    entity_to_cell: Vec<Option<(i16, i16)>>,
}

impl SpatialHash {
    pub fn new() -> Self {
        Self {
            cells: HashMap::with_capacity(4096),  // ~64KB de memoria
            entity_to_cell: Vec::new(),
        }
    }
    
    /// Actualización incremental O(k) donde k = entidades dirty
    /// 
    /// Solo actualiza las entidades marcadas como dirty en el EntityStore
    pub fn sync_dirty(&mut self, store: &EntityStore, dirty: &FixedBitSet) {
        for idx in dirty.ones() {
            let id = store.get_id_by_index(idx);
            
            // NUEVO: Usar world_transform en lugar de pos (soporte para jerarquías)
            let world_pos = store.world_transform[idx];
            let pos = Vec2::new(world_pos[0], world_pos[1]);
            
            let new_cell = (
                (pos.x / CELL_SIZE).floor() as i16,
                (pos.y / CELL_SIZE).floor() as i16,
            );
            
            // Remover de celda anterior si existe
            if let Some(old_cell) = self.entity_to_cell[idx] {
                if old_cell != new_cell {
                    self.remove_from_cell(old_cell, id);
                }
            }
            
            // Insertar en nueva celda
            let cell = self.cells.entry(new_cell).or_default();
            let _ = cell.push(id);  // Ignora si llena (raro en diseño bien particionado)
            self.entity_to_cell[idx] = Some(new_cell);
        }
    }
    
    /// Query O(1): revisa celda del punto y 8 celdas vecinas
    pub fn query_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        let cx = (point.x / CELL_SIZE).floor() as i16;
        let cy = (point.y / CELL_SIZE).floor() as i16;
        
        (-1..=1).flat_map(move |dx| {
            (-1..=1).filter_map(move |dy| {
                self.cells.get(&(cx + dx, cy + dy)).map(|v| v.iter().copied())
            })
        }).flatten()
    }
    
    /// Query O(celdas): para marquee selection o viewport culling
    pub fn query_rect(&self, min: Vec2, max: Vec2) -> Vec<EntityId> {
        let cx_min = (min.x / CELL_SIZE).floor() as i16;
        let cy_min = (min.y / CELL_SIZE).floor() as i16;
        let cx_max = (max.x / CELL_SIZE).floor() as i16;
        let cy_max = (max.y / CELL_SIZE).floor() as i16;
        
        (cx_min..=cx_max).flat_map(move |cx| {
            (cy_min..=cy_max).flat_map(move |cy| {
                self.cells.get(&(cx, cy)).map(|v| v.iter().copied())
            })
        }).flatten().collect()
    }
    
    fn remove_from_cell(&mut self, cell: (i16, i16), id: EntityId) {
        if let Some(vec) = self.cells.get_mut(&cell) {
            vec.retain(|&e| *e != id);
        }
    }
}
```

---

## 9. Pipeline de Renderizado: Multi-Phase Instancing

### 9.1. Por Qué Multi-Phase vs Single Pipeline

**Crítica de V1**: El "Single Pipeline" con branching/mix masivo en fragment shader causaba divergencia SIMD, penalizando rendimiento.

**Solución V2/V3**: 4 draw calls especializados con mejor coherencia de ejecución:

| Fase | Shader | Contenido | Beneficio |
|------|--------|----------|----------|
| **Shapes** | `sdf_shapes.wgsl` | Rectángulos, círculos, líneas | Coherencia SIMD |
| **Icons** | `icon_texture.wgsl` | Iconos AWS/Azure (Atlas) | Optimizado para texturas |
| **Images** | `image_array.wgsl` | PNGs, capturas | Bindless texture array |
| **Text** | `mtsdf_text.wgsl` | Labels, docs | MTSDF multi-canal |

### 9.2. GpuRenderer Multi-Phase

```rust
// archflow-render/src/gpu_renderer.rs

use wgpu::util::DeviceExt;
use archflow_engine::EntityStore;
use archflow_render::camera::Camera;

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    
    // Pipelines especializadas
    shape_pipeline: wgpu::RenderPipeline,
    icon_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    
    // Storage Buffers (compartidos)
    entity_buffer: wgpu::Buffer,
    draw_order_buffer: wgpu::Buffer,
    
    // Texturas
    icon_atlas: wgpu::Texture,
    image_array: wgpu::Texture,  // Texture2DArray
    text_atlas: wgpu::Texture,     // MTSDF Atlas
    
    // Uniforms
    uniform_buffer: wgpu::Buffer,
    
    // Staging (CPU-side, reutilizado)
    staging_entities: Vec<GpuInstance>,
    staging_shapes: Vec<u32>,    // draw_order por fase
    staging_icons: Vec<u32>,
    staging_images: Vec<u32>,
    staging_text: Vec<u32>,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuInstance {
    pos: [f32; 2],
    size: [f32; 2],
    color: u32,
    shape_type: u32,
    data: [f32; 2],
    _padding: u64,  // Pad to 32 bytes total
}

impl GpuRenderer {
    pub async fn new(canvas: &HtmlCanvasElement) -> Result<Self, wgpu::RequestError> {
        // Setup WebGPU estándar
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }).await?;
        
        let surface = instance.create_surface(canvas)?;
        
        let device = instance.request_device(&wgpu::RequestDeviceOptions {
            limits: wgpu::Limits {
                max_storage_buffers_per_shader_stage: 4,
                ..Default::default()
            },
            ..Default::default()
        }).await?;
        
        let queue = device.queue();
        
        // Crear pipelines (4 fases)
        let shape_pipeline = Self::create_shape_pipeline(&device, surface.get_preferred_format(&adapter));
        let icon_pipeline = Self::create_icon_pipeline(&device, surface.get_preferred_format(&adapter));
        let image_pipeline = Self::create_image_pipeline(&device, surface.get_preferred_format(&adapter));
        let text_pipeline = Self::create_text_pipeline(&device, surface.get_preferred_format(&adapter));
        
        // Crear buffers
        let entity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Buffer"),
            size: (MAX_ENTITIES * std::mem::size_of::<GpuInstance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let draw_order_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Draw Order Buffer"),
            size: (MAX_ENTITIES * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Crear texturas
        let icon_atlas = Self::create_icon_atlas(&device)?;
        let image_array = Self::create_image_array(&device)?;
        let text_atlas = Self::create_mtsdf_atlas(&device)?;
        
        // Crear bind groups
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: wgpu::BindGroupLayout {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                }],
            }],
            label: Some("Global Bind Group"),
        })?;
        
        Ok(Self {
            device,
            queue,
            surface,
            shape_pipeline,
            icon_pipeline,
            image_pipeline,
            text_pipeline,
            entity_buffer,
            draw_order_buffer,
            uniform_buffer,
            icon_atlas,
            image_array,
            text_atlas,
            staging_entities: Vec::with_capacity(MAX_ENTITIES),
            staging_shapes: Vec::new(),
            staging_icons: Vec::new(),
            staging_images: Vec::new(),
            staging_text: Vec::new(),
        })
    }
    
    pub fn sync_from(&mut self, store: &EntityStore, camera: &Camera) {
        self.staging_entities.clear();
        self.staging_shapes.clear();
        self.staging_icons.clear();
        self.staging_images.clear();
        self.staging_text.clear();
        
        // Preparar buckets por tipo
        for &idx in &store.draw_order {
            let texture_idx = store.texture_index[*idx];
            let shape_type = store.shape_type(*idx);
            
            match texture_idx {
                0 => {
                    // Color sólido → Shapes
                    if shape_type <= 2 {  // Rect, Circle, Line
                        self.staging_shapes.push(*idx);
                    } else {
                        // Texto
                        self.staging_text.push(*idx);
                    }
                }
                1..=1000 => {
                    // Icon Atlas
                    self.staging_icons.push(*idx);
                }
                _ => {
                    // Image Array
                    self.staging_images.push(*idx);
                }
            }
        }
        
        // Subir datos de instancia
        if !self.staging_entities.is_empty() {
            self.queue.write_buffer(
                &self.entity_buffer,
                0,
                bytemuck::cast_slice(&self.staging_entities),
            );
        }
        
        // Subir draw order
        self.queue.write_buffer(
            &self.draw_order_buffer,
            0,
            bytemuck::cast_slice(&store.draw_order),
        );
        
        // Actualizar uniforms (cámara)
        let uniforms = CameraUniforms::from_camera(camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }
    
    pub fn render_frame(&mut self, store: &EntityStore) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.98, g: 0.98, b: 0.98, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            
            // FASE 1: Shapes
            if !self.staging_shapes.is_empty() {
                rpass.set_pipeline(&self.shape_pipeline);
                rpass.set_bind_group(0, &self.global_bind_group, &[]);
                rpass.draw(0..4, 0..self.staging_shapes.len() as u32);
            }
            
            // FASE 2: Icons
            if !self.staging_icons.is_empty() {
                rpass.set_pipeline(&self.icon_pipeline);
                rpass.set_bind_group(0, &self.global_bind_group, &[]);
                rpass.set_bind_group(1, &self.icon_bind_group, &[]);
                rpass.draw(0..4, 0..self.staging_icons.len() as u32);
            }
            
            // FASE 3: Images
            if !self.staging_images.is_empty() {
                rpass.set_pipeline(&self.image_pipeline);
                rpass.set_bind_group(0, &self.global_bind_group, &[]);
                rpass.set_bind_group(1, &self.image_array_bind_group, &[]);
                rpass.draw(0..4, 0..self.staging_images.len() as u32);
            }
            
            // FASE 4: Text
            if !self.staging_text.is_empty() {
                rpass.set_pipeline(&self.text_pipeline);
                rpass.set_bind_group(0, &self.global_bind_group, &[]);
                rpass.set_bind_group(1, &self.text_bind_group, &[]);
                rpass.draw(0..4, 0..self.staging_text.len() as u32);
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
```

---

## 10. Texture Atlas Dinámico (Shelf Packing)

```rust
// archflow-render/src/atlas/packer.rs

use std::collections::HashMap;

/// Rectángulo en el atlas de texturas
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Estantería (fila) en el atlas
struct Shelf {
    y_start: u32,
    height: u32,
    current_x: u32,
}

/// Empaquetador de texturas usando algoritmo de estanterías (Shelf Packing)
/// 
/// Ventajas:
/// - O(shelves) para insertar, no requiere reorganización completa
/// - Adecuado para texturas de tamaño similar (como iconos de librerías)
/// - Fácil implementación y muy rápido
pub struct AtlasPacker {
    width: u32,
    height: u32,
    shelves: Vec<Shelf>,
    padding: u32,
}

impl AtlasPacker {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            shelves: Vec::new(),
            padding: 2,  // Evita que los píxeles de un icono "sangren" al vecino
        }
    }
    
    /// Asigna espacio en el atlas para un icono/textura
    /// 
    /// Devuelve None si el atlas está lleno
    pub fn allocate(&mut self, w: u32, h: u32) -> Option<Rect> {
        let needed_w = w + self.padding;
        let needed_h = h + self.padding;
        
        // 1. Buscar una estantería existente donde quepa
        for shelf in &mut self.shelves {
            if shelf.height >= needed_h && (shelf.current_x + needed_w) <= self.width {
                let rect = Rect {
                    x: shelf.current_x,
                    y: shelf.y_start,
                    w,
                    h,
                };
                shelf.current_x += needed_w;
                return Some(rect);
            }
        }
        
        // 2. Si no hay espacio, crear nueva estantería arriba de la última
        let y_start = self.shelves.last()
            .map(|s| s.y_start + s.height)
            .unwrap_or(0);
        
        if y_start + needed_h <= self.height {
            self.shelves.push(Shelf {
                y_start,
                height: needed_h,
                current_x: needed_w,
            });
            Some(Rect { x: 0, y: y_start, w, h })
        } else {
            None  // Atlas lleno
        }
    }
}
```

---

## 11. Sistema de Conexiones Magnéticas

```rust
// archflow-engine/src/connection_store.rs

pub struct ConnectionStore {
    pub sources: Vec<EntityId>,
    pub targets: Vec<EntityId>,
    pub source_anchors: Vec<AnchorSide>,
    pub target_anchors: Vec<AnchorSide>,
    pub line_styles: Vec<LineStyle>,
    pub active_anchors: FixedBitSet,  // Entidades con conexiones activas
    pub dirty: FixedBitSet,           // Conexiones que necesitan recálculo
}

/// Puntos de anclaje en los bordes de las entidades
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorSide {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
    Center = 4,
    Custom(f32, f32),  // Offset porcentaje 0.0-1.0 relativo al centro
}

/// Estilos de línea para diferentes tipos de conexión
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStyle {
    Direct = 0,    // Línea recta entre puntos
    Orthogonal = 1, // Línea con ángulos de 90° (estándar arquitectura)
    Step = 2,        // Línea con escalones (manual routing)
    Bezier = 3,       // Curva de Bézier suave
}

impl ConnectionStore {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            targets: Vec::new(),
            source_anchors: Vec::new(),
            target_anchors: Vec::new(),
            line_styles: Vec::new(),
            active_anchors: FixedBitSet::with_capacity(MAX_ENTITIES),
            dirty: FixedBitSet::with_capacity(MAX_CONNECTIONS),
        }
    }
    
    /// Actualizar solo las conexiones dirty (O(k) donde k = conexiones con entidades movidas)
    pub fn update_dirty(&mut self, store: &EntityStore, renderer: &mut GpuRenderer) {
        for idx in self.dirty.ones() {
            let src_idx = self.sources[idx].index();
            let tgt_idx = self.targets[idx].index();
            
            // Solo recalcular si alguno de los extremos está dirty
            if store.dirty_transform.contains(src_idx) || store.dirty_transform.contains(tgt_idx) {
                let points = self.generate_orthogonal_points(src_idx, tgt_idx, store);
                renderer.update_connection(idx, points, self.line_styles[idx]);
            }
        }
        self.dirty.clear();
    }
    
    /// Generar puntos para routing ortogonal (90°)
    fn generate_orthogonal_points(
        &self,
        src_idx: usize,
        tgt_idx: usize,
        store: &EntityStore,
    ) -> Vec<Vec2> {
        let src_pos = Self::get_anchor_point(src_idx, self.source_anchors, store);
        let tgt_pos = Self::get_anchor_point(tgt_idx, self.target_anchors, store);
        
        // Routing ortogonal inteligente: elegir horizontal o vertical según separación
        let dx = (tgt_pos.x - src_pos.x).abs();
        let dy = (tgt_pos.y - src_pos.y).abs();
        
        if dx > dy {
            // Más separado en X - routing horizontal dominante
            vec![
                src_pos,
                Vec2::new((src_pos.x + tgt_pos.x) / 2.0, src_pos.y),  // Mid X
                Vec2::new((src_pos.x + tgt_pos.x) / 2.0, tgt_pos.y),  // Mid Y
                tgt_pos,
            ]
        } else {
            // Más separado en Y - routing vertical dominante
            vec![
                src_pos,
                Vec2::new(src_pos.x, (src_pos.y + tgt_pos.y) / 2.0),  // Mid Y
                Vec2::new(tgt_pos.x, (src_pos.y + tgt_pos.y) / 2.0),  // Mid X
                tgt_pos,
            ]
        }
    }
    
    fn get_anchor_point(idx: usize, anchors: &[AnchorSide], store: &EntityStore) -> Vec2 {
        let transform = store.world_transform[idx];
        let center = Vec2::new(transform[0], transform[1]);
        let size = Vec2::new(transform[2], transform[3]);
        
        match anchors[idx] {
            AnchorSide::Top => Vec2::new(center.x, center.y + size.y / 2.0),
            AnchorSide::Bottom => Vec2::new(center.x, center.y - size.y / 2.0),
            AnchorSide::Left => Vec2::new(center.x - size.x / 2.0, center.y),
            AnchorSide::Right => Vec2::new(center.x + size.x / 2.0, center.y),
            AnchorSide::Center => center,
            AnchorSide::Custom(dx, dy) => center + Vec2::new(size.x * dx, size.y * dy),
        }
    }
}
```

---

## 12. Sistema de Texto (MTSDF + Pre-shaping)

### 12.1. MTSDF Atlas (Multi-channel True SDF)

```rust
// archflow-render/src/text/mtsdf_atlas.rs

pub struct MtsdfAtlas {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    glyph_cache: HashMap<(u32, u32, u16), Rect>,  // (font_id, glyph_id, size_px) -> UV
}

impl MtsdfAtlas {
    pub async fn generate(device: &wgpu::Device, fonts: &[FontData]) -> Result<Self, Error> {
        // Generar MTSDF usando msdfgen (offline o vía build script)
        // MTSDF = Multi-channel SDF para bordes afilados en fuentes pequeñas
        let (width, height, pixels) = generate_msdf_atlas(fonts)?;
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MTSDF Atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });
        
        // Subir datos de píxeles
        queue.write_texture(&texture, wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            aspect: wgpu::TextureAspect::All,
            src: wgpu::ImageCopyTemplate {
                data: bytemuck::cast_slice(&pixels),
            },
        });
        
        Ok(Self {
            texture,
            width,
            height,
            glyph_cache: HashMap::new(),
        })
    }
}
```

### 12.2. Glyph Run Cache (Persistente)

```rust
// archflow-text/src/layout_cache.rs

use lru::LruCache;

pub struct GlyphRunCache {
    cache: LruCache<u64, FlatGlyphRun>,
}

struct FlatGlyphRun {
    glyph_positions: Vec<[f32; 2]>,
    glyph_uvs: Vec<[f32; 4]>,
    total_glyphs: usize,
}

impl GlyphRunCache {
    pub fn get_or_compute(
        &mut self,
        text: &str,
        font_size: f32,
        font_system: &mut FontSystem,
    ) -> &FlatGlyphRun {
        let key = self.hash_text_and_scale(text, font_size);
        
        if !self.cache.contains(&key) {
            let layout = self.compute_layout(text, font_size, font_system);
            self.cache.put(key, layout);
        }
        
        self.cache.get(&key).unwrap()
    }
    
    fn compute_layout(
        &mut self,
        text: &str,
        font_size: f32,
        font_system: &mut FontSystem,
    ) -> FlatGlyphRun {
        // Usar cosmic-text para shaping
        let mut buffer = Buffer::new(font_system, Metrics::new(font_size, font_size));
        buffer.set_size(font_system, f32::MAX, f32::MAX);
        buffer.set_text(font_system, text, Attrs::new(), Shaping::Advanced);
        
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                positions.push([glyph.x, glyph.y + run.line_y]);
                uvs.push(self.get_glyph_uv(glyph));
            }
        }
        
        FlatGlyphRun {
            glyph_positions: positions,
            glyph_uvs: uvs,
            total_glyphs: positions.len(),
        }
    }
}
```

---

## 13. Hit Testing (O(1) Spatial Query)

```rust
// archflow-interaction/src/hit_testing.rs

use archflow_core::{EntityId, Vec2, Rect};
use archflow_engine::{EntityStore, SpatialHash};

/// Hit tester O(1) usando SpatialHash + Z-order
pub struct HitTester;

impl HitTester {
    /// Encontrar la entidad más cercana bajo el cursor, respetando Z-order
    pub fn find_at(
        cursor_world: Vec2,
        spatial: &SpatialHash,
        store: &EntityStore,
    ) -> Option<EntityId> {
        // O(1): Obtener candidatos del SpatialHash
        let candidates = spatial.query_point(cursor_world);
        
        let mut best_hit: Option<EntityId> = None;
        let mut max_z = -1;
        
        // O(k): Refinar búsqueda con AABB test y Z-order
        for id in candidates {
            let idx = id.index();
            let transform = store.world_transform[idx];
            let pos = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            
            let rect = Rect::from_center_size(pos, size);
            
            if rect.contains(cursor_world) {
                let z_index = store.get_z_index(id);
                if z_index > max_z {
                    max_z = z_index as i32;
                    best_hit = Some(id);
                }
            }
        }
        
        best_hit
    }
    
    /// Selección por rectángulo (marquee selection)
    pub fn find_in_rect(
        selection_rect: Rect,
        spatial: &SpatialHash,
        store: &EntityStore,
    ) -> Vec<EntityId> {
        spatial.query_rect(selection_rect.min, selection_rect.max)
            .filter(|&id| {
                let idx = id.index();
                let transform = store.world_transform[idx];
                let pos = Vec2::new(transform[0], transform[1]);
                let size = Vec2::new(transform[2], transform[3]);
                let rect = Rect::from_center_size(pos, size);
                selection_rect.contains_rect(&rect)
            })
            .collect()
    }
}
```

---

## 14. Carga de Iconos (Draw.io Parser)

### 14.1. Decodificador de Draw.io

```rust
// archflow-plugins/src/drawio/decoder.rs

use flate2::read::DeflateDecoder;
use base64::Engine;
use std::io::Read;
use percent_encoding::percent_decode;

/// Decodifica datos comprimidos de Draw.io
/// 
/// Formato Draw.io:
/// 1. XML (descripción del diagrama)
/// 2. Deflate (compresión)
/// 3. Base64 (codificación)
/// 4. URL encode (%-encoding)
pub fn decode_drawio_data(encoded_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 1. URL Decode (si hay %xx)
    let decoded_url = if encoded_data.contains('%') {
        percent_decode::percent_decode_str(encoded_data)?.decode_utf8()?
    } else {
        Cow::Borrowed(encoded_data)
    };
    
    // 2. Base64 Decode
    let compressed_bytes = base64::engine::general_purpose::STANDARD.decode(decoded_url.as_bytes())?;
    
    // 3. Inflate (Deflate decompression)
    let mut decoder = DeflateDecoder::new(&compressed_bytes[..]);
    let mut xml_string = String::new();
    decoder.read_to_string(&mut xml_string)?;
    
    Ok(xml_string)
}
```

### 14.2. Parser de Librerías

```rust
// archflow-plugins/src/drawio/parser.rs

use quick_xml::events::Event;

/// Icono de librería Draw.io
pub struct LibraryIcon {
    pub id: String,
    pub name: String,
    pub svg_data: String,
}

/// Parser de librerías Draw.io
pub fn parse_library_xml(xml_content: &str) -> Vec<LibraryIcon> {
    let mut reader = quick_xml::Reader::from_str(xml_content);
    let mut icons = Vec::new();
    let mut current_icon = None;
    let mut current_data = String::new();
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name() == "item" => {
                current_icon = Some(LibraryIcon {
                    id: e.try_get_attribute("id").unwrap_or("").to_string(),
                    name: String::new(),
                    svg_data: String::new(),
                });
            }
            Ok(Event::Text(ref text)) if current_icon.is_some() => {
                if current_icon.as_mut().unwrap().name.is_empty() {
                    current_icon.as_mut().name = text.to_string();
                } else {
                    current_icon.as_mut().svg_data = text.to_string();
                }
            }
            Ok(Event::End(ref e)) if e.name() == "item" => {
                if let Some(icon) = current_icon.take() {
                    icons.push(icon);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("Error parsing Draw.io XML: {}", e),
            _ => {}
        }
    }
    
    icons
}
```

### 14.3. Rasterizador de SVG a GPU

```rust
// archflow-render/src/atlas/svg_rasterizer.rs

use resvg::usvg;
use tiny_skia::PixmapMut;

pub struct SvgRasterizer {
    packer: AtlasPacker,
    texture: wgpu::Texture,
}

impl SvgRasterizer {
    pub async fn add_svg(
        &mut self,
        svg_data: &str,
        size: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<Rect> {
        // 1. Parsear SVG
        let tree = usvg::Tree::from_data(svg_data.as_bytes(), &Default::default()).ok()?;
        
        // 2. Renderizar a buffer RGBA
        let mut pixels = vec![0u8; (size * size * 4) as usize];
        let mut fb = PixmapMut::from_bytes(&mut pixels, size, size).ok()?;
        let render_ts = usvg::Transform::identity();
        resvg::render(&tree, render_ts, &mut fb);
        
        // 3. Obtener espacio en atlas
        let rect = self.packer.allocate(size, size)?;
        
        // 4. Subir a GPU (solo el cuadrado del icono)
        queue.write_texture(
            &self.texture,
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: rect.x, y: rect.y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size * 4),
                rows_per_image: Some(size),
            },
            &pixels,
        );
        
        Some(rect)
    }
}
```

---

## 15. UI y Gizmos (Immediate Mode)

```rust
// archflow-render/src/gizmos.rs

pub enum GizmoCommand {
    Line { 
        start: Vec2, 
        end: Vec2, 
        color: u32, 
        width: f32, 
        dashed: bool 
    },
    Rect { 
        min: Vec2, 
        max: Vec2, 
        stroke_color: u32, 
        fill_color: Option<u32> 
    },
    Handle { 
        pos: Vec2, 
        cursor: u8 
    },
}

pub struct GizmoRenderer {
    instances: heapless::Vec<GpuInstance, 512>,
    count: usize,
}

impl GizmoRenderer {
    pub fn clear(&mut self) {
        self.count = 0;
    }
    
    pub fn draw_selection_box(&mut self, bounds: Rect, color: u32) {
        // Borde: 4 líneas ortogonales
        let corners = [
            bounds.min,
            Vec2::new(bounds.max.x, bounds.min.y),
            bounds.max,
            Vec2::new(bounds.min.x, bounds.max.y),
        ];
        
        for i in 0..4 {
            let start = corners[i];
            let end = corners[(i + 1) % 4];
            self.draw_line(start, end, color, 2.0, false);
        }
        
        // Relleno con transparencia
        self.draw_rect_filled(bounds.min, bounds.max, Some(color.with_alpha(0.1)));
        
        // Esquinas (handles en las esquinas)
        let handle_size = 8.0;
        for corner in &corners {
            self.draw_handle(*corner, handle_size);
        }
    }
    
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: u32, width: f32, dashed: bool) {
        if self.count >= 512 { return; }
        
        // Convertir línea en rectángulo rotado
        let dir = end - start;
        let len = dir.length();
        let angle = dir.y.atan2(dir.x);
        
        let center = (start + end) / 2.0;
        
        self.instances[self.count] = GpuInstance {
            pos: [center.x, center.y],
            size: [len, width],
            color,
            shape_type: 2,  // Line
            data: [start.x, start.y],
            _padding: 0,
        };
        self.count += 1;
    }
    
    pub fn draw_rect_filled(&mut self, min: Vec2, max: Vec2, fill: Option<u32>) {
        if self.count >= 512 { return; }
        
        self.instances[self.count] = GpuInstance {
            pos: [(min.x + max.x) / 2.0, (min.y + max.y) / 2.0],
            size: [max.x - min.x, max.y - min.y],
            color: fill.unwrap_or(color),
            shape_type: 0, // Rect
            data: [0.0, 0.0],
            _padding: 0,
        };
        self.count += 1;
    }
    
    pub fn draw_handle(&mut self, pos: Vec2, size: f32) {
        if self.count >= 512 { return; }
        
        // Dibujar handle circular
        for angle in [0, std::f32::consts::PI * 0.5; std::f32::consts::PI * 2.0] {
            let handle_pos = pos + Vec2::new(
                size * 0.5 * angle.cos(),
                size * 0.5 * angle.sin(),
            );
            
            self.instances[self.count] = GpuInstance {
                pos: [handle_pos.x, handle_pos.y],
                size: [size, size],
                color: 0xFFFFFFFF, // Blanco puro
                shape_type: 1, // Circle
                data: [0.0, 0.0],
                _padding: 0,
            };
            self.count += 1;
        }
    }
    
    pub fn submit(&mut self, renderer: &mut GpuRenderer) {
        if self.count > 0 {
            renderer.draw_gizmos(&self.instances[..self.count]);
        }
    }
}
```

---

## 16. Undo/Redo (Command Sourcing)

```rust
// archflow-interaction/src/history.rs

use std::collections::VecDeque;

pub struct HistoryManager {
    undo_stack: VecDeque<UndoEntry>,
    redo_stack: VecDeque<UndoEntry>,
    max_depth: usize,
}

struct UndoEntry {
    redo: Command,
    undo: Command,
}

impl HistoryManager {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_depth),
            redo_stack: VecDeque::with_capacity(max_depth),
            max_depth,
        }
    }
    
    pub fn record(&mut self, redo: Command, undo: Command) {
        self.undo_stack.push_back(UndoEntry { redo, undo });
        self.redo_stack.clear();  // Invalidar redo al nueva acción
    }
    
    pub fn undo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(entry) = self.undo_stack.pop_back() {
            entry.undo.execute(store);
            self.redo_stack.push_back(entry);
            true
        } else {
            false
        }
    }
    
    pub fn redo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(entry) = self.redo_stack.pop_back() {
            entry.redo.execute(store);
            self.undo_stack.push_back(entry);
            true
        } else {
            false
        }
    }
}

// Ejemplo de comando reversible
struct ChangeColorCmd {
    entity_id: EntityId,
    old_color: u32,
    new_color: u32,
}

impl ChangeColorCmd {
    fn execute(&self, store: &mut EntityStore) {
        store.set_color(self.entity_id, self.new_color);
    }
    
    fn undo(&self, store: &mut EntityStore) {
        store.set_color(self.entity_id, self.old_color);
    }
}
```

---

## 17. Colaboración en Tiempo Real (CRDT)

```rust
// archflow-interaction/src/crdt.rs

use std::collections::VecDeque;

/// Comando remoto con timestamp de Lamport
pub struct RemoteCommand {
    pub origin_user: u32,
    pub timestamp: u64,  // Lamport timestamp para orden total
    pub command: Command,
}

/// Gestor de CRDT para sincronización multi-usuario
pub struct CrdtManager {
    user_id: u32,
    lamport_clock: u64,
    pending: VecDeque<RemoteCommand>,
}

impl CrdtManager {
    pub fn new(user_id: u32) -> Self {
        Self {
            user_id,
            lamport_clock: 0,
            pending: VecDeque::new(),
        }
    }
    
    /// Aplicar comando local y broadcast a usuarios remotos
    pub fn apply_local(&mut self, cmd: Command) -> RemoteCommand {
        self.lamport_clock += 1;
        
        RemoteCommand {
            origin_user: self.user_id,
            timestamp: self.lamport_clock,
            command: cmd,
        }
    }
    
    /// Aplicar comando remoto con resolución de conflictos
    pub fn apply_remote(&mut self, store: &mut EntityStore, remote: RemoteCommand) {
        // Actualizar clock
        self.lamport_clock = self.lamport_clock.max(remote.timestamp) + 1;
        
        // Aplicar comando (posible transformación OT si hay conflicto)
        remote.command.execute(store);
    }
    
    /// Resolver conflictos cuando el mismo objeto es modificado por dos usuarios
    pub fn resolve_conflict(
        &self,
        local_cmd: &Command,
        remote_cmd: &Command,
    ) -> ConflictResolution {
        match local_cmd.timestamp().cmp(&remote_cmd.timestamp()) {
            std::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
            std::cmp::Ordering::Less => ConflictResolution::UseRemote,
            std::cmp::Ordering::Equal => {
                // Tie-breaker: user ID
                match local_cmd.origin_user().cmp(&remote_cmd.origin_user) {
                    std::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
                    _ => ConflictResolution::UseRemote,
                }
            }
        }
    }
}

pub enum ConflictResolution {
    KeepLocal,    // Mantener versión local
    UseRemote,    // Usar versión remota
}
```

---

## 18. Motor de Curvas Bézier (GPU-Based)

```rust
// archflow-core/src/paths.rs

/// Comandos de ruta para representar curvas vectoriales
pub enum PathCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo { control: Vec2, end: Vec2 },
    CubicTo { ctrl1: Vec2, ctrl2: Vec2, end: Vec2 },
}

/// Path compuesto por comandos de ruta
pub struct Path {
    commands: Vec<PathCommand>,
    stroke_width: f32,
    stroke_color: Color,
    fill_color: Option<Color>,
}

// Para renderizado en GPU:
// - Convertir curvas en bounding boxes + datos de control
// - Fragment shader calcula SDF a curva (ecuación de Bezier)
```

---

## 19. Serialización Zero-Copy

```rust
// archflow-export/src/serialization.rs

use flatbuffers::{FlatBuffer, // Zero-copy write
    WI};                           // Zero-copy read

pub struct ProjectSerializer;

impl ProjectSerializer {
    pub fn serialize(store: &EntityStore, connections: &ConnectionStore) -> Vec<u8> {
        let mut builder = FlatBuffer::new();
        
        // Header
        let mut header = builder.create_vector("header");
        header.push(&format!("ArchFlow{}", 1));  // Versión
        header.push(0);  // Flags
        header.push(0);  // Padding
        builder.finish_minimal(&mut header).unwrap();
        
        // Entity Count
        let mut entities = builder.create_vector("entities");
        entities.push(&store.alive_count.to_le_bytes());
        builder.finish_minimal(&mut entities).unwrap();
        
        // SoA Blobs (direct memcpy desde RAM)
        let mut pos_blob = builder.create_vector("positions");
        bytemuck::cast_slice(&store.pos).iter());
        pos_blob.extend_from_slice(bytemuck::cast_slice(&store.size));
        builder.finish_minimal(&mut pos_blob).unwrap();
        
        // String Pool
        let mut string_blob = builder.create_vector("strings");
        let mut string_offsets = builder.create_vector("offsets");
        for (start, len) in &store.string_pool.offsets {
            string_offsets.push(&(start as u32, len as u32));
        }
        builder.finish_minimal(&mut string_blob).unwrap();
        
        // Footer
        builder.finish_minimal(&mut builder.create_vector("footer")).unwrap();
        
        builder.collapse(&mut [].to_vec()).unwrap()
    }
}

pub struct ProjectDeserializer;

impl ProjectDeserializer {
    pub fn deserialize(data: &[u8]) -> Result<(EntityStore, ConnectionStore), Error> {
        let mut buf = FlatBuffer::new(&data)?;
        
        // Validar header
        let magic = buf.get_ref("header")?;
        let version = magic.get_u8(1)?;
        if magic.get_str() != b"ArchFlow" || version != &1 {
            return Err("Invalid format")?);
        }
        
        // Parsear Entity Count
        let entity_count = buf.get_u32("entities")? as usize;
        
        // Recuperar SOA blobs (zero-copy)
        let pos_data = buf.get_ref("positions")?;
        let size_data = buf.get_ref("sizes")?;
        let colors_data = buf.get_ref("colors")?;
        
        // Reconstruir EntityStore desde slices (zero-copy)
        let pos_slice: &[f32] = bytemuck::cast_slice(pos_data)?;
        let size_slice: &[f32] = bytemuck::cast_slice(size_data)?;
        let color_slice: &[u32] = bytemuck::cast_slice(colors_data)?;
        
        // String pool
        let string_blob = buf.get_ref("strings")?;
        let string_offsets = buf.get_ref("offsets")?;
        
        // Reconstruir stores
        let mut store = EntityStore::with_capacity(entity_count);
        store.pos = pos_slice.to_vec();
        store.size = size_slice.to_vec();
        store.colors = color_slice.to_vec();
        // ... resto de campos ...
        
        Ok((store, connections))
    }
}
```

---

## 20. Exportación IaC (Terraform/Mermaid)

```rust
// archflow-export/src/terraform.rs

pub struct TerraformExporter;

impl TerraformExporter {
    pub fn generate(&self, store: &EntityStore, connections: &ConnectionStore) -> String {
        let mut hcl = String::new();
        hcl.push_str("# ══════════════════════════════════════════════\n");
        hcl.push_str("# Infrastructure Diagram Generated by ArchFlow\n\n");
        hcl.push_str("   Timestamp: ${chrono::Utc::now().to_rfc3339()}\n\n");
        
        // Recorrer entidades por tipo C4
        for idx in 0..store.alive_count {
            if !store.is_visible(idx) {
                continue;
            }
            
            if let Some(arch_data) = &store.arch_data[idx] {
                match arch_data.c4_level {
                    C4Level::System => {
                        hcl.push_str(&format!("resource \"{}\" {{\n", arch_data.name));
                        hcl.push_str(&format!("  name = \"{}\"\n", store.string_pool.get(idx)));
                        hcl.push_str(&format!("  type = \"{}\"\n", "system"));
                        // Atributos específicos según cloud provider
                        Self::add_cloud_attributes(&mut hcl, arch_data, idx);
                        hcl.push_str("}\n");
                    }
                    C4Level::Container => {
                        // Generar VPC, Subnet, etc.
                    }
                    C4Level::Component => {
                        // Generar EC2, Lambda, S3, etc.
                    }
                }
            }
        }
        
        // Conexiones → depends_on
        hcl.push_str("\n# ════════════════════════════════════════════\n");
        for (src_idx, tgt_idx) in connections.iter() {
            let src_name = store.string_pool.get(src_idx);
            let tgt_name = store.string_pool.get(tgt_idx);
            hcl.push_str(&format!(
                "dependency \"resource_{}_to_{}\" {{\n",
                src_name, tgt_name
            ));
            hcl.push_str(&format!("  depends_on = [resource.{}.name]\n]\n", src_name));
        }
        
        hcl
    }
    
    fn add_cloud_attributes(&mut hcl: String, arch_data: &ArchitectureData, idx: usize) {
        // Generar atributos específicos del cloud provider
        match arch_data.cloud_provider {
            CloudProvider::AWS => Self::add_aws_attributes(hcl, arch_data, idx),
            CloudProvider::GCP => Self::add_gcp_attributes(hcl, arch_data, idx),
            CloudProvider::Azure => Self::add_azure_attributes(hcl, arch_data, idx),
        }
    }
}
```

---

## 21. Flujo del Frame (El Tick Integrado)

```rust
// archflow-web/src/engine.rs

pub struct ArchFlowEngine {
    store: EntityStore,
    spatial_hash: SpatialHash,
    string_pool: StringPool,
    renderer: GpuRenderer,
    text_system: TextLayoutSystem,
    gizmo_renderer: GizmoRenderer,
    input_processor: InputProcessor,
    command_queue: CommandQueue,
    history: HistoryManager,
    camera: Camera,
    connection_store: ConnectionStore,
    crdt: CrdtManager,
}

impl ArchFlowEngine {
    #[wasm_bindgen]
    pub fn tick(&mut self, timestamp: f64) {
        // ══════════════════════════════════════════════════════════
        // FASE 1: INPUT (Zero-copy desde SharedArrayBuffer)
        // ════════════════════════════════════════════════════════════
        for evt in self.input_ring_buffer.drain() {
            let world_pos = self.camera.screen_to_world(Vec2::new(evt.x, evt.y));
            
            match evt.event_type {
                0 => self.on_pointer_down(world_pos, evt),
                1 => self.on_pointer_move(world_pos, evt),
                2 => self.on_pointer_up(world_pos, evt),
                3 => self.on_wheel(evt, &mut self.camera),
                _ => {}
            }
        }
        
        // ══════════════════════════════════════════════════════════════
        // FASE 2: COMMAND FLUSH (Dominio)
        // ════════════════════════════════════════════════════════
        for cmd in self.command_queue.drain() {
            let undo_cmd = self.history.capture_undo(&cmd, &self.store);
            cmd.execute(&mut self.store);
            self.history.record(cmd, undo_cmd);
        }
        
        // ══════════════════════════════════════════════════════════
        // FASE 3: HIERARCHY UPDATE (Si hay dirty)
        // ════════════════════════════════════════════════════════
        if self.store.has_dirty_hierarchy() {
            self.store.update_world_transforms();
        }
        
        // ══════════════════════════════════════════════════════════
        // FASE 4: TEXT LAYOUT (Lazy, solo dirty)
        // ══════════════════════════════════════════════════════════
        if !self.store.dirty_text.is_empty() {
            self.text_system.update(&mut self.store, &self.store.dirty_text);
            self.store.dirty_text.clear();
        }
        
        // ══════════════════════════════════════════════════════════
        // FASE 5: SPATIAL SYNC (O(k) donde k = dirty)
        // ════════════════════════════════════════════════════════════════
        if !self.store.dirty_transform.is_empty() {
            self.spatial_hash.sync_dirty(&self.store, &self.store.dirty_transform);
            self.store.dirty_transform.clear();
        }
        
        // ══════════════════════════════════════════════════════════
        // FASE 6: CONNECTION UPDATE (Solo dirty)
        // ══════════════════════════════════════════════════════════
        self.connection_store.update_dirty(&self.store, &mut self.renderer);
        
        // ════════════════════════════════════════════════════════════
        // FASE 7: GIZMO GENERATION (Immediate mode)
        // ════════════════════════════════════════════════════════════════
        self.gizmo_renderer.clear();
        if let Some(selection) = self.input_processor.get_selection() {
            let bounds = self.store.get_bounds(selection.id);
            self.gizmo_renderer.draw_selection_box(bounds, Color::BLUE);
        }
        
        // ════════════════════════════════════════════════════════════
        // FASE 8: RENDER (Multi-Phase Instancing)
        // ════════════════════════════════════════════════════════════
        self.renderer.sync_from(&self.store, &self.camera);
        self.renderer.render_frame(&self.store);
        
        // ════════════════════════════════════════════════════════════
        // FASE 9: CLEANUP (Prepare for next frame)
        // ══════════════════════════════════════════════════════════════════
        self.store.clear_render_dirty();
    }
}
```

---

## 22. Optimización de Compilación

```toml
[profile.release]
lto = true                           # Link Time Optimization
opt-level = "z"                        # Optimizado para tamaño
codegen-units = 1                        # Código monolítico
panic = "abort"                         # Eliminar unwinding
strip = true                            # Eliminar símbolos

[profile.release.package."*"]
opt-level = 3                          # Dependencias con máxima optimización
```

### Build y Post-procesamiento

```bash
# 1. Compilar
cargo build --release

# 2. Generar WASM
wasm-bindgen target/wasm32-unknown-unknown-release/archflow_web.wasm \
  --out-dir web/pkg --target web

# 3. Optimizar WASM (Binaryen -40% más pequeño)
wasm-opt -Oz -O4 --enable-mv-analysis \
  -o web/pkg/archflow_web_bg.wasm \
  -o web/pkg/archflow_web_bg.o

# 4. Comprimir (Brotli - ~60% adicional)
brotli -q 11 web/pkg/archflow_web_bg.wasm
```

---

## 23. Métricas de Validación

| Métrica | Target | Método de Medición |
|---------|--------|---------------------|
| **Frame Time** | <16.6ms | `performance.now()` |
| **Allocations/Frame** | 0 | Custom allocator logging |
| **Binary Size** | <500KB (gzipped) | `ls -lh` + brotli |
| **Memory WASM** | <64MB | `memory.buffer.byteLength` |
| **Input Latency** | <8ms | Event timestamp → render delta |
| **Iconos Cargados** | 1,000+ | Draw.io library parsing |
| **Conexiones** | 10,000 | Orthogonal routing @ 60FPS |

---

## 24. Roadmap de Implementación

### Fase 1: Core Foundation (Semanas 1-2)
- [ ] Implementar `EntityId` generacional (packed u32)
- [ ] Implementar `EntityStore` con SoA estricto y pre-alloc
- [ ] Implementar `Command` enum (Copy, 16 bytes max)
- [ ] Implementar `InputRingBuffer` lock-free

### Fase 2: WebGPU Rendering (Semanas 3-4)
- [ ] Setup WebGPU context
- [ ] Crear shader WGSL unificado SDF
- [ ] Implementar `GpuRenderer` con storage buffers
- [ ] Sistema de staging SoA→AOS sin allocaciones

### Fase 3: Spatial Indexing (Semanas 5-6)
- [ ] Implementar `SpatialHash` con grid fijo
- [ ] Integrar dirty tracking con EntityStore
- [ ] Implementar hit-testing O(1)

### Fase 4: Text System (Semanas 7-8)
- [ ] Integrar `cosmic-text` para layout
- [ ] Generar MSDF atlas (offline o lazy)
- [ ] Implementar `StringPool` flat

### Fase 5: Interaction & UI (Semanas 9-10)
- [ ] Implementar `GizmoRenderer` immediate mode
- [ ] Sistema de selección y drag
- [ ] DOM overlay para input de texto
- [ ] Implementar `HistoryManager` con undo/redo

### Fase 6: Polish & Optimization (Semanas 11-12)
- [ ] Profiling y optimización de hot paths
- [ ] Compilación con wasm-opt
- [ ] Testing de carga (100k objetos)
- [ ] Validación de métricas

---

## Conclusión

Esta versión V3 completa es la **Single Source of Truth** para la implementación de ArchFlow Engine v3.0 - MVP Ready.

**Especificación técnica definitiva y completa para un motor de diagramación profesional que supera a Figma en rendimiento, extensibilidad (IaC export, librerías nativas) y usabilidad (zoom infinito, conexiones inteligentes).**
