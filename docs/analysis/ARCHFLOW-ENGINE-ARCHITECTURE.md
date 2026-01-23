# ArchFlow Graphics Engine: Análisis de Arquitectura Integral (v2.0)

**Versión:** 2.0.0  
**Fecha:** 2026-01-23  
**Inspiración Principal:** tldraw, Bevy, críticas de revisión  
**Estado:** Actualizado post-revisión arquitectónica

---

## 1. Cambios Post-Revisión

La revisión identificó **problemas críticos** que requieren cambios arquitectónicos:

| Aspecto | Antes (v1.0) | Después (v2.0) | Justificación |
|---------|--------------|----------------|---------------|
| **ECS** | `legion` | `bevy_ecs` | legion está en mantenimiento; bevy_ecs es el estándar actual |
| **Spatial Index** | Custom Quadtree | `rstar` (R-Tree) | 10k+ nodos necesitan queries O(log n), no O(n) |
| **Undo/Redo** | Full snapshots | **Delta-based** | 10k nodos × 100 estados = GBs de RAM |
| **Texto** | Custom | `cosmic-text` | Layout de texto multilínea + BiDi es extremadamente complejo |
| **WASM Bridge** | JSON/serde | SharedArrayBuffer | Zero-copy para 60fps |

---

## 2. Stack Tecnológico v2.0

### 2.1 Dependencias Actualizadas

```toml
[dependencies]
# Mathematics (unchanged - excellent choice)
glam = "0.27"           # SIMD-optimized Vec2
euclid = "0.22"         # Typed geometry Box2D

# ECS (CHANGED: legion → bevy_ecs)
bevy_ecs = "0.13"       # Standard de facto, mejor ergonomía

# Curves (unchanged)
kurbo = "0.13"          # Bezier curves maduro

# Spatial Indexing (NEW)
rstar = "0.17"          # R-Tree para queries espaciales O(log n)

# Text Rendering (NEW)
cosmic-text = "0.11"    # Layout multilínea, emojis, BiDi

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Delta encoding for undo/redo (NEW)
serde_diff = "2.0"      # Para patches/deltas

# History management
indexmap = "2.0"
ahash = "0.8"
```

---

## 3. Arquitectura v2.0

### 3.1 Capas Refactorizadas

```
┌─────────────────────────────────────────────────────────────────────┐
│                      WASM BOUNDARY (Zero-Copy)                       │
│  ┌───────────────────┐  ┌───────────────────┐  ┌─────────────────┐  │
│  │ SharedArrayBuffer │  │  Event Proxy      │  │  Render Target  │  │
│  └───────────────────┘  └───────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│                      PRESENTATION LAYER                              │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Renderer Trait (impl: Canvas2D, WebGPU)                    │    │
│  │  • lyon_tessellation para geometry → triangles              │    │
│  │  • cosmic-text para text layout                             │    │
│  │  • instanced rendering para batch de shapes                 │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                     SPATIAL INDEX LAYER (NEW)                        │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  RTree<SpatialObject>                                       │    │
│  │  • Insert/Remove en O(log n)                                │    │
│  │  • Frustum culling (no renderizar lo fuera de pantalla)    │    │
│  │  • Hit testing (queries por área)                           │    │
│  │  • Nearest neighbor queries                                 │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                     APPLICATION LAYER (ECS)                          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  bevy_ecs::World                                             │    │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────────────────┐  │    │
│  │  │ Transform  │ │ Renderable │ │ Selection              │  │    │
│  │  │ Component  │ │ Component  │ │ Component              │  │    │
│  │  └────────────┘ └────────────┘ └────────────────────────┘  │    │
│  │                                                              │    │
│  │  ┌──────────────────────────────────────────────────────┐   │    │
│  │  │  Systems (auto-scheduled by bevy_ecs)                │   │    │
│  │  │  • transform_update_system                           │   │    │
│  │  │  • spatial_index_update_system                       │   │    │
│  │  │  • render_prepare_system                             │   │    │
│  │  └──────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                      DOMAIN LAYER (Records)                          │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Store<R> with Delta-Based History                          │    │
│  │  ┌─────────────┐ ┌─────────────────┐ ┌──────────────────┐  │    │
│  │  │ Records     │ │ ChangeHistory   │ │ CRDT-Ready       │  │    │
│  │  │ IndexMap    │ │ (Deltas only!)  │ │ (Loro optional)  │  │    │
│  │  └─────────────┘ └─────────────────┘ └──────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                    INFRASTRUCTURE LAYER                              │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────┐   │
│  │ glam      │ │ euclid    │ │ rstar     │ │ cosmic-text       │   │
│  └───────────┘ └───────────┘ └───────────┘ └───────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. Sistema de Records v2.0: Delta-Based History

### 4.1 El Problema de los Snapshots

```rust
// ANTES: Snapshots completos (PROBLEMÁTICO)
struct Store<R: Record> {
    records: IndexMap<RecordId, R>,
    undo_history: VecDeque<IndexMap<RecordId, R>>, // 💀 100MB+ con 10k registros
}
```

**Con 10,000 registros:**
- Un snapshot = ~1-2MB (dependiendo del tamaño del registro)
- 100 estados = 100-200MB de RAM
- Latencia de undo = cloning completo

### 4.2 Solución: Deltas (Diff-based)

```rust
// DESPUÉS: Deltas (OPTIMIZADO)
enum RecordChange {
    Created { id: RecordId, record: R },
    Updated { id: RecordId, old_values: SerdeDiff, new_values: SerdeDiff },
    Deleted { id: RecordId, record: R },
}

struct Store<R: Record> {
    records: IndexMap<RecordId, R>,
    undo_stack: VecDeque<Vec<RecordChange>>, // Solo cambios, no todo el estado
    redo_stack: VecDeque<Vec<RecordChange>>,
}

impl<R: Record> Store<R> {
    /// Apply change and record delta for undo
    pub fn put(&mut self, record: R) -> Vec<RecordChange> {
        let changes = match self.records.get(record.id()) {
            None => {
                // Nuevo registro
                vec![RecordChange::Created { 
                    id: record.id().clone(), 
                    record: record.clone() 
                }]
            }
            Some(old) => {
                // Actualización - calcular diff
                let diff = serde_diff::diff(old, &record);
                vec![RecordChange::Updated {
                    id: record.id().clone(),
                    old_values: diff.old,
                    new_values: diff.new,
                }]
            }
        };
        
        // Guardar delta (no snapshot completo)
        self.undo_stack.push_back(changes.clone());
        self.redo_stack.clear();
        
        // Aplicar cambio
        self.records.insert(record.id().clone(), record);
        
        changes
    }
    
    /// Undo: Apply inverse deltas
    pub fn undo(&mut self) -> bool {
        if let Some(changes) = self.undo_stack.pop_back() {
            // Apply inverse of each change
            for change in changes.into_iter().rev() {
                match change {
                    RecordChange::Created { id, .. } => {
                        self.records.shift_remove(&id);
                    }
                    RecordChange::Updated { id, old_values, .. } => {
                        if let Some(record) = self.records.get_mut(&id) {
                            serde_diff::patch(record, &old_values);
                        }
                    }
                    RecordChange::Deleted { id, record, .. } => {
                        self.records.insert(id, record);
                    }
                }
            }
            true
        } else {
            false
        }
    }
}
```

### 4.3 Memoria Comparada

| Configuración | Memoria por estado | 100 estados |
|--------------|-------------------|-------------|
| **Snapshots** (v1.0) | 1-2 MB | 100-200 MB |
| **Deltas** (v2.0) | 1-10 KB | 100 KB - 1 MB |
| **Ahorro** | 100-1000x | 100-1000x |

---

## 5. Spatial Indexing con R-Tree (rstar)

### 5.1 Por qué no un Quadtree custom

| Aspecto | Quadtree Custom | R-Tree (rstar) |
|---------|-----------------|----------------|
| Queries | O(√n) | O(log n) |
| Memoria | Fragmentada | Compacto |
| Mantenimiento | Lo mantienes tú | Comunidad |
| Border cases | Complejo | Resuelto |

### 5.2 Implementación

```rust
use rstar::{RTree, RTreeObject, Point};

#[derive(Clone)]
struct SpatialRecord {
    id: RecordId,
    bounds: euclid::Box2D<f32>,
    // ... otros campos
}

impl RTreeObject for SpatialRecord {
    type Envelope = AABB<[f32; 4]>;
    
    fn envelope(&self) -> Self::Envelope {
        AABB::from([
            self.bounds.min_x(),
            self.bounds.min_y(),
            self.bounds.max_x(),
            self.bounds.max_y(),
        ])
    }
}

// Spatial index para todo el documento
struct SpatialIndex {
    tree: RTree<SpatialRecord>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
        }
    }
    
    pub fn insert(&mut self, record: SpatialRecord) {
        self.tree.insert(record);
    }
    
    pub fn remove(&mut self, id: &RecordId) -> Option<SpatialRecord> {
        self.tree.remove(|r| &r.id == id)
    }
    
    /// Hit test: encontrar todos los elementos en un punto
    pub fn point_query(&self, point: Vec2) -> Vec<&SpatialRecord> {
        self.tree.locate_point_at_point([point.x, point.y])
    }
    
    /// Frustum culling: obtener solo elementos visibles
    pub fn frustum_query(&self, viewport: euclid::Box2D<f32>) -> Vec<&SpatialRecord> {
        self.tree.locate_in_envelope(&AABB::from([
            viewport.min_x(),
            viewport.min_y(),
            viewport.max_x(),
            viewport.max_y(),
        ]))
    }
    
    /// Nearest neighbor: encontrar elemento más cercano
    pub fn nearest_to(&self, point: Vec2) -> Option<&SpatialRecord> {
        self.tree.nearest_neighbor(&[point.x, point.y]).map(|r| &*r)
    }
}
```

### 5.3 Queries de Rendimiento

| Query Type | Con R-Tree | Sin Spatial Index |
|-----------|------------|-------------------|
| Hit test (1 punto) | O(log n) + k | O(n) |
| Frustum culling | O(log n) + k | O(n) |
| Selection box | O(log n) + k | O(n) |
| Nearest neighbor | O(log n) | O(n) |

Donde `k` = número de resultados.

---

## 6. Text Rendering: cosmic-text

### 6.1 Por qué no hacerlo desde cero

`cosmic-text` proporciona:
- Layout multilínea automático
- Soporte de emojis (noto-fonts-emoji)
- Bidirectional text (BiDi) - necesario para árabe/hebreo
- Kerning y ligaduras
- Hyphenation

```rust
use cosmic_text::{FontSystem, SwashCache, TextArea, Layout};

struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl TextRenderer {
    pub fn render_text(
        &mut self,
        text: &str,
        position: Vec2,
        font_size: f32,
        color: Color,
    ) -> Vec<Shape> {
        let mut text_buffer = cosmic_text::Buffer::new(
            &mut self.font_system,
            cosmic_text::Metrics::new(font_size, font_size),
        );
        
        text_buffer.set_text(
            &mut self.font_system,
            text,
            cosmic_text::Shaping::Advanced,
        );
        
        text_buffer.layout_as_rich_text(
            &mut self.font_system,
            500.0, // max_width
            cosmic_text::Wrap::Word,
            cosmic_text::Align::Left,
        );
        
        // Convertir a shapes para rendering
        self.buffer_to_shapes(&text_buffer, position, color)
    }
}
```

---

## 7. WASM Zero-Copy Bridge

### 7.1 El Problema

```rust
// LENTO: Serialización en cada frame
fn render_loop() {
    let records = store.get_all_records(); // Clone completo
    let json = serde_json::to_string(&records).unwrap(); // Serialización
    js_sys::JSON::parse(&json); // JS parsing
    canvas.draw(&parsed); // Drawing
}
```

### 7.2 Solución: SharedArrayBuffer + Direct Memory Access

```rust
// RÁPIDO: Acceso directo a memoria
#[wasm_bindgen]
pub struct RenderBuffer {
    ptr: *mut f32,
    capacity: usize,
}

#[wasm_bindgen]
impl ArchFlowEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> ArchFlowEngine {
        // Crear SharedArrayBuffer
        let buffer = js_sys::ArrayBuffer::new(
            (width * height * 4 * std::mem::size_of::<f32>()) as u32
        );
        let ptr = buffer.as_ref().as_ref() as *const _ as *mut f32;
        
        Self {
            ptr,
            buffer,
            width,
            height,
        }
    }
    
    /// JS puede acceder directamente a la memoria sin cloning
    pub fn get_vertex_buffer_ptr(&self) -> *const f32 {
        self.ptr
    }
    
    pub fn render_to_buffer(&mut self) {
        // Escribir directamente en el buffer compartido
        // GPU lee directamente desde JS
    }
}
```

### 7.3 JavaScript Usage

```javascript
// JS: Acceso directo a vértices
const engine = new ArchFlowEngine(800, 600);
const ptr = engine.get_vertex_buffer_ptr();
const floatArray = new Float32Array(memory.buffer, ptr, vertexCount * 7);

// Rendering directo sin parsing
gpu.drawArraysInstanced(..., vertexCount);
```

---

## 8. Comparación de Rendimiento Objetivo

### 8.1 Budget de 16.6ms por frame (60fps)

| Sistema | v1.0 (estimado) | v2.0 (objetivo) |
|---------|-----------------|----------------|
| ECS Query | 2-5ms | 0.5-1ms |
| Spatial Index | N/A | 0.1-0.5ms |
| Hit Testing | O(n) = 10ms | O(log n) = 0.1ms |
| Frustum Culling | N/A | 0.5-1ms |
| Tessellation (Lyon) | 2-5ms | 2-5ms |
| Text Layout | N/A | 1-2ms |
| **Total** | **>20ms** | **<10ms** ✅ |

### 8.2 Memoria

| Aspecto | v1.0 | v2.0 |
|---------|------|------|
| Store (10k registros) | 10-20 MB | 10-20 MB |
| Undo/Redo (100 estados) | 1-2 GB 💀 | 100-500 KB ✅ |
| Spatial Index | N/A | 5-10 MB |
| Render Buffers | 50 MB | 50 MB |
| **Total** | **>1 GB** | **<100 MB** ✅ |

---

## 9. Plan de Implementación v2.0

### Fase 1: Core + Spatial (Semanas 1-2)

```
1.1 Records System v2.0
    ├── RecordId, FractionalIndex (listo)
    ├── Delta-based Store (NUEVO)
    └── CRDT-compatibility layer

1.2 Spatial Indexing
    ├── Integración con rstar
    ├── SpatialRecord wrapper
    └── Queries: point, box, nearest

1.3 Tests de rendimiento
    └── Benchmark: 10k inserts + spatial queries
```

### Fase 2: ECS + Rendering Foundation (Semanas 3-4)

```
2.1 bevy_ecs Integration
    ├── Transform, Renderable components
    ├── Systems auto-scheduling
    └── World serialization

2.2 Rendering Foundation
    ├── Renderer trait abstraction
    ├── Lyon tessellation integration
    └── Canvas 2D backend (MVP)

2.3 Text Rendering Setup
    └── cosmic-text integration
```

### Fase 3: WASM Bridge + Optimization (Semanas 5-6)

```
3.1 WASM Zero-Copy
    ├── SharedArrayBuffer setup
    ├── Direct memory access from JS
    └── Event proxy system

3.2 Performance Tuning
    ├── Profile con criterion
    └── Optimize hot paths
```

### Fase 4: Polish + Collaboration (Semanas 7-8)

```
4.1 Collaborative Ready
    ├── Loro CRDT integration (opcional)
    └── Sync protocol design

4.2 Polish
    ├── Text rendering completo
    └── Export (SVG/PNG)
```

---

## 10. Connascence Analysis v2.0

| # | Elementos | Tipo | Severidad | Mitigación |
|---|-----------|------|-----------|------------|
| 1 | `Delta.apply()` inverse | CoC | Alta | Tests unitarios para cada tipo de cambio |
| 2 | `RTree` synchronization | CoT | Media | Sistema de eventos para mantener synced |
| 3 | `WASM` memory layout | CoP | Alta | Documentación explícita del buffer format |
| 4 | `cosmic-text` ↔ renderer | CoL | Baja | Renderer trait abstraction |

---

## 11. Conclusiones

La crítica fue acertada. Los cambios principales son:

1. ✅ **bevy_ecs** en lugar de legion
2. ✅ **rstar** para spatial indexing
3. ✅ **Delta-based undo/redo** en lugar de snapshots
4. ✅ **cosmic-text** para texto
5. ✅ **SharedArrayBuffer** para WASM zero-copy

El resultado es una arquitectura que:
- **Cabe en memoria**: <100 MB vs >1 GB
- **Mantiene 60fps**: <10ms/frame vs >20ms
- **Es mantenible**: Crates bien kurados vs código custom
- **Es collaboration-ready**: CRDT desde el diseño

---

## Referencias

### Crates Actualizados
- [bevy_ecs](https://docs.rs/bevy_ecs/latest/bevy_ecs/) - ECS moderno
- [rstar](https://docs.rs/rstar/latest/rstar/) - R-Tree spatial index
- [cosmic-text](https://docs.rs/cosmic-text/latest/cosmic-text/) - Text rendering
- [serde_diff](https://docs.rs/serde_diff/latest/serde_diff/) - Delta encoding

### Referencias Originales
- [glam](https://docs.rs/glam/latest/glam/) - Linear algebra
- [euclid](https://docs.rs/euclid/latest/euclid/) - Geometry
- [kurbo](https://docs.rs/kurbo/latest/kurbo/) - Curves
- [lyon](https://docs.rs/lyon/latest/lyon/) - Tessellation

---

*Documento v2.0 generado el 2026-01-23 post-revisión arquitectónica.*
