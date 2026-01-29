# ArchFlow SDK - Análisis y Justificación de Crates

## Resumen Ejecutivo

Este documento proporciona una justificación detallada de cada crate en el SDK de ArchFlow, evaluando su propósito, utilidad y encaje en la arquitectura con una capa mínima de JavaScript, considerando los requisitos del PRD: **colaboración en tiempo real** y **diagramas con animaciones multicapa estilo C4**.

---

## 1. Crates del Núcleo (Core)

### 1.1 `archflow-core` ✅ **MANTENER - ESENCIAL**

**Propósito**: Meta-crate con tipos base del dominio.

**Justificación**:
- ✅ **Vec2, Mat3, Rect**: Tipos geométricos fundamentales usados por TODOS los demás crates
- ✅ **EntityId**: Identificadores tipados para todas las entidades del sistema
- ✅ **Color**: Sistema de color (Rgba, Hsla) para renderizado y estilos
- ✅ **Animation**: Keyframes y easing para animaciones
- ✅ **Transform**: Matrices de transformación 2D

**Connascence Transformada**: `Connascence of Position` → `Connascence of Name` mediante tipos compartidos centralizados.

**Veredicto**: **IMPRESCINDIBLE**. Sin este crate no existe consistencia de tipos.

---

### 1.2 `archflow-geometry` ✅ **MANTENER - ALTO RENDIMIENTO**

**Propósito**: Cálculos geométricos usando kurbo.

**Justificación**:
- ✅ **GeometryEngine**: Operaciones geométricas básicas (área, perímetro, centroid)
- ✅ **IntersectionEngine**: Detección de intersecciones O(n) optimizada
- ✅ **PathEngine**: Manipulación de curvas de Bézier y paths
- ✅ **Kurbo Integration**: Usa kurbo (excelente librería de 2D geometry)

**Casos de Uso**:
- Hit-testing preciso en formas complejas
- Cálculo de bounds para renderizado
- Operaciones booleanas sobre shapes

**Veredicto**: **MANTENER**. Necesario para geometría avanzada.

---

### 1.3 `archflow-spatial` ✅ **MANTENER - ACTIVACIÓN ADAPTATIVA**

**Propósito**: Indexación espacial con R-Tree.

#### 1.3.1 Análisis de Overhead

| Métrica | Sin R-Tree (O(n)) | Con R-Tree (O(log n)) | Overhead |
|---------|-------------------|----------------------|----------|
| 10 shapes | 10 checks | ~14 checks (tree ops) | +40% |
| 100 shapes | 100 checks | ~14 checks | **-86%** |
| 1,000 shapes | 1,000 checks | ~14 checks | **-99%** |
| 10,000 shapes | 10,000 checks | ~14 checks | **-99.9%** |

#### 1.3.2 Estrategia de Activación Adaptativa

```rust
// El SDK decide automáticamente cuándo usar R-Tree
pub struct SpatialIndexManager {
    rtree: Option<RTreeIndex>,
    use_rtree_threshold: usize, // Default: 100 shapes
    shapes_count: usize,
}

impl SpatialIndexManager {
    pub fn insert(&mut self, id: RecordId, bounds: Bounds) {
        self.shapes_count += 1;
        
        // Activar R-Tree automáticamente al cruzar el threshold
        if self.shapes_count >= self.use_rtree_threshold && self.rtree.is_none() {
            self.rtree = Some(RTreeIndex::new(16));
            // Migrar shapes existentes al R-Tree
            self.rebuild_index();
        }
        
        if let Some(ref mut tree) = self.rtree {
            tree.insert(id, bounds);
        }
        // Si no hay R-Tree, no hacer nada (búsqueda lineal en registros)
    }
    
    pub fn point_query(&self, point: [f64; 2]) -> Vec<RecordId> {
        match &self.rtree {
            Some(tree) => tree.point_query(point),
            None => {
                // Búsqueda lineal - aceptable para <100 shapes
                self.linear_point_query(point)
            }
        }
    }
}
```

#### 1.3.3 Costo de Memoria

| Configuración | Memoria Adicional | Justificación |
|---------------|-------------------|---------------|
| R-Tree inactivo | 0 bytes | Sin estructura adicional |
| R-Tree activo | ~32 bytes/shape | HashMap + TreeNode overhead |
| 10,000 shapes | ~320 KB | Aceptable para el rendimiento ganado |

**Veredicto**: **MANTENER CON ACTIVACIÓN ADAPTATIVA**. El overhead es insignificante (<100 shapes) y el beneficio es crítico para documentos grandes.

---

## 2. Crates de Renderizado

### 2.1 `archflow-renderers` ✅ **MANTENER - RENDIMIENTO GPU**

**Propósito**: WebGPU batch rendering con instancing.

**Justificación**:
- ✅ **BatchRenderer2D**: Agrupa objetos por material para minimizar state changes GPU
- ✅ **Instancing**: Un solo draw call para miles de objetos del mismo tipo
- ✅ **Zero-Copy**: `bytemuck` para transferencia directa a GPU
- ✅ **O(C) Complexity**: Solo procesa registros visibles/modificados

**Arquitectura**:
```
Renderable (Trait)
    ↓
BatchRenderer2D (organiza por material)
    ↓
RenderContext (WebGPU)
    ↓
GPU (instanced draw calls)
```

**Veredicto**: **MANTENER**. Diferenciador clave de rendimiento.

---

## 3. Crates de Estado y Colaboración

### 3.1 `archflow-records` ✅ **MANTENER - FUNDACIÓN**

**Propósito**: Sistema de registros con fractional indexing y CRDT light.

**Justificación**:
- ✅ **RecordId**: IDs tipados y validados
- ✅ **FractionalIndex**: Ordenamiento conflict-free (como tldraw/Figma)
- ✅ **DeltaManager**: Sistema O(1) de undo/redo
- ✅ **RecordStore**: Change tracking con FixedBitSet
- ✅ **ChangeSet**: Solo marca qué cambió, no duplica estado

**Casos de Uso**:
- Shapes ordenados por z-index
- Undo/redo sin memory leaks
- Colaboración parcial (solo cambios)

**Veredicto**: **IMPRESCINDIBLE**. Base del sistema de estado.

---

### 3.2 `archflow-collab` ✅ **MANTENER - COLABORACIÓN REQUERIDA**

**Propósito**: Sistema CRDT completo para colaboración en tiempo real.

#### 3.2.1 Requisito del PRD

El PRD sección 3.4 Collaboration System especifica:
- **Multi-user Editing**: Live cursors, selection, changes
- **Real-time Features**: Live cursors, selection sync
- **Git Integration**: Branch strategy con merge workflows
- **Collaboration Latency Target**: <100ms

#### 3.2.2 Arquitectura CRDT Integrada

```rust
// El SDK expone CRDT como componente central, no opcional
pub struct CollabManager<R: Record> {
    crdt: CRDT<R>,
    sync_client: Option<SyncClient>,
    conflict_pipeline: ConflictResolutionPipeline<R>,
}

impl<R: Record> CollabManager<R> {
    /// Inicializar colaboración local (sin servidor)
    pub fn new_local(site_id: SiteId) -> Self {
        Self {
            crdt: CRDT::new(site_id),
            sync_client: None,
            conflict_pipeline: ConflictResolutionPipeline::new(),
        }
    }
    
    /// Conectar a servidor de colaboración
    pub async fn connect(&mut self, server_url: &str) -> Result<(), CollabError> {
        self.sync_client = Some(SyncClient::new(server_url).await?);
        Ok(())
    }
    
    /// Obtener cambios para sincronización
    pub fn get_pending_changes(&mut self) -> Vec<RecordChange<R>> {
        self.crdt.get_changes()
    }
    
    /// Aplicar cambios remotos con resolución de conflictos
    pub fn apply_remote_changes(
        &mut self, 
        remote_clock: &VectorClock, 
        remote_records: Vec<R>
    ) -> Result<(), ApplyError> {
        self.crdt.merge(remote_clock, remote_records)
    }
}
```

#### 3.2.3 Justificación Técnica

| Feature CRDT | Uso en ArchFlow |
|--------------|-----------------|
| **Vector Clock** | Tracking de causalidad entre usuarios |
| **Last-Writer-Wins** | Resolución automática de conflictos |
| **Causal Relations** | Detectar ediciones concurrentes |
| **SiteId** | Identificar origen de cambios |
| **Sync Protocol** | Sincronización incremental |

**Veredicto**: **IMPRESCINDIBLE según PRD**. La colaboración en tiempo real es un requisito core del producto (Phase 2: Collaboration).

---

### 3.3 `archflow-workspace` ✅ **MANTENER - DOCUMENTOS**

**Propósito**: Gestión de documentos con event sourcing.

**Justificación**:
- ✅ **Document**: Contenedor principal con event journal
- ✅ **EventJournal**: Registro de eventos para replay/debug
- ✅ **UndoManager**: Gestión undo/redo a nivel documento
- ✅ **SelectionState**: Estado de selección multi-shape

**Casos de Uso**:
- Documentos múltiples (tabs)
- Persistencia de estado
- Debug/replay de acciones
- **Collaboration Sync**: Event journal como source of truth para sync

**Veredicto**: **MANTENER**. Integración con CRDT para sync de eventos.

---

### 3.4 `archflow-ecs-hybrid` ✅ **MANTENER - ANIMACIONES Y CAPAS C4**

**Propósito**: Sincronización Records ↔ ECS (bevy_ecs) para animaciones y lógica de juego.

#### 3.4.1 Justificación según PRD

El PRD especifica requisitos que ECS maneja excelentemente:

| Requisito PRD | Por qué ECS es Ideal |
|---------------|---------------------|
| **Semantic Zoom (C4 Levels)** | Sistema de entidades anidadas para Context→Container→Component→Code |
| **Animations** | Systems de Bevy para tweening, interpolación, keyframes |
| **Multi-layer System** | Componentes por capa (security, cost, compliance) |
| **Interactive Simulations** | Game loop para physics, collision detection |

#### 3.4.2 Arquitectura Híbrida Records ↔ ECS

```rust
// Records son la fuente de verdad (persistencia)
// ECS es el motor de renderizado y animaciones

pub struct EcsEngine {
    world: World,
    record_store: RecordStore<ComponentRecord>,
    sync_systems: Schedule,
    animation_systems: Schedule,
}

impl EcsEngine {
    /// Sincronización Records → ECS (draw calls, animaciones)
    pub fn sync_records_to_ecs(&mut self) {
        let changeset = self.record_store.drain_changes();
        
        for change in changeset.created_or_updated() {
            // Crear o actualizar entidad ECS对应的
            self.upsert_entity(change.record());
        }
        
        for id in changeset.deleted() {
            self.despawn_entity(id);
        }
    }
    
    /// Ejecutar sistemas de animación (interpolación, easing)
    pub fn run_animations(&mut self, delta_time: f32) {
        self.animation_systems.run(&mut self.world);
    }
    
    /// Obtener datos de renderizado desde ECS
    pub fn get_render_data(&self) -> RenderData {
        // Query a ECS para posiciones, escalas, colores actuales
        // Esto ya incluye todas las animaciones en progreso
    }
}

/// Sistema de animación usando bevy_ecs
fn animate_position(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &AnimationTarget)>
) {
    for (mut transform, target) in &mut query {
        if let Some((start, end, duration, started_at)) = target.current_animation {
            let elapsed = time.now() - started_at;
            let t = (elapsed.as_secs_f32() / duration).clamp(0.0, 1.0);
            
            // Interpolación con easing
            let eased_t = easing::ease_in_out(t);
            transform.translation = start.lerp(end, eased_t);
        }
    }
}
```

#### 3.4.3 Configuración Condicional (Feature Flags)

```toml
# Cargo.toml
[features]
default = ["animations", "layers"]

# Desactivar si solo se necesita editor estático
no-animations = ["bevy_ecs", "archflow_core/animation"]
animations = ["bevy_ecs", "archflow_core/animation"]

# Sistema de capas C4
layers = []  # Siempre disponible, sin dependencias adicionales
```

#### 3.4.4 Uso Recomendado por Escenario

| Escenario | ECS Requerido | Uso |
|-----------|---------------|-----|
| **Editor básico** | ❌ | Records directo |
| **Editor con animaciones** | ✅ | Sistema de tweening |
| **Diagrams C4 multicapa** | ✅ | Entity hierarchy |
| **Simulaciones interactivas** | ✅ | Game loop, physics |
| **Preview estático** | ❌ | Records + Renderer directo |

**Veredicto**: **MANTENER DISPONIBLE**. Esencial para:
1. **Animaciones fluidas** (smooth transitions, dragging feedback)
2. **Capas C4** (jerarquía de componentes)
3. **Simulaciones** (game loop para physics/collision)

---

## 4. Crates de Primitivas UI

### 4.1 `archflow-primitives` ✅ **MANTENER - INTERACCIÓN**

**Propósito**: Primitivas de interacción UI (drag-drop, resize, routing).

**Justificación**:
- ✅ **DragDrop**: Sistema completo de drag con snap guides
- ✅ **Resize**: Handles, aspect ratio, constraints
- ✅ **Routing**: Conexiones entre shapes (como draw.io)
- ✅ **Selection**: Box selection, multi-select

**Componentes**:
```
DragManager ──→ DragState, Draggable, SnapGuides
ResizeManager ──→ ResizeState, HandleType, SizeConstraints
ConnectionRouter ──→ ControlPointMode, MarkerType, Obstacle
```

**Veredicto**: **MANTENER**. Proporciona comportamiento "de editor" estándar.

---

## 5. Crates de Integración WASM

### 5.1 `archflow-wasm-collab` ✅ **MANTENER - BRIDGE JS**

**Propósito**: Zero-copy WASM bridge para JavaScript.

**Justificación**:
- ✅ **SharedBuffer**: SharedArrayBuffer para zero-copy entre Rust/JS
- ✅ **WasmBridge**: API mínima para JS
- ✅ **BinaryDeltaCodec**: Delta encoding para updates
- ✅ **Cross-Origin Isolation**: Verificación de seguridad

**Arquitectura Zero-Copy**:
```
┌─────────────────┐     SharedArrayBuffer     ┌─────────────────┐
│   JavaScript    │ ◄──────────────────────► │      Rust       │
│                 │      Zero-copy read       │   (WASM)        │
│  Canvas.read()  │                          │  Records, ECS   │
└─────────────────┘                          └─────────────────┘
```

**Veredicto**: **IMPRESCINDIBLE**. Única forma de comunicar Rust con JS eficientemente.

---

## 6. Crates de Demo/Test

### 6.1 `demo-web` ⚠️ **REFERENCIA - NO SDK**

**Propósito**: Demo funcional de todas las features.

**Justificación**:
- ✅ **DemoState**: Estado completo del demo (shapes, selección, undo/redo)
- ✅ **Handlers**: Event handlers para mouse/teclado
- ✅ **Render Loop**: Canvas rendering integration

**Para SDK**: Este crate es **referencia de implementación**, no parte del SDK.

**Veredicto**: **MANTENER como referencia**, no como API pública.

---

### 6.2 `archflow-tests` ✅ **MANTENER - CONFIANZA**

**Propósito**: Integration tests y stress tests.

**Justificación**:
- ✅ **Full Workflow Tests**: Records → SharedBuffer → Render
- ✅ **Binary Delta Tests**: Encoding/decoding
- ✅ **ChangeSet Tests**: Optimización de memoria
- ✅ **Stress Tests**: 20,000 records, memory usage, throughput

**Veredicto**: **MANTENER**. Esencial para regression testing.

---

## 7. Matriz de Decisión SDK Final

| Crate | Esencial | Activación | Feature Flag | Justificación PRD |
|-------|----------|------------|--------------|-------------------|
| `archflow-core` | ✅ | Always | - | Tipos base del dominio |
| `archflow-geometry` | ✅ | Always | - | Geometría avanzada (hit-testing) |
| `archflow-spatial` | ⚠️ | Adaptative | - | Activo si >100 shapes |
| `archflow-renderers` | ✅ | Always | - | WebGPU batch rendering |
| `archflow-records` | ✅ | Always | - | Estado y CRDT base |
| `archflow-collab` | ✅ | Always | - | **3.4 Collaboration System** |
| `archflow-workspace` | ✅ | Always | - | Documentos + event sourcing |
| `archflow-ecs-hybrid` | ⚠️ | Conditional | `animations` | **3.1.3 Animations**, **3.5 Simulations** |
| `archflow-primitives` | ✅ | Always | - | UI interactions |
| `archflow-wasm-collab` | ✅ | Always | - | Bridge JS/Rust |

---

## 8. Propuesta API SDK Final

### 8.1 Capa Rust (Core SDK)

```rust
// crates/archflow-sdk/src/lib.rs

// ==================== TIPOS FUNDAMENTALES ====================
pub use archflow_core::{Vec2, Rect, EntityId, Color, Transform, Animation};

// ==================== ESTADO Y DOCUMENTOS ===================
pub use archflow_records::{Record, RecordId, RecordStore, FractionalIndex, ChangeSet};
pub use archflow_workspace::{Document, UndoManager, SelectionState};

// ==================== COLABORACIÓN (REQUERIDA) ===============
pub use archflow_collab::{
    CRDT, SiteId, VectorClock, CausalRelation,
    SyncClient, SyncServerBackend,
    ConflictDetector, ConflictResolutionPipeline,
};

// ==================== PRIMITIVAS DE INTERACCIÓN ==============
pub use archflow_primitives::{
    Draggable, Resizable, 
    DragManager, ResizeManager,
    DragSelectionBox, SelectionConfig,
    ConnectionRouter, RoutingResult,
};

// ==================== RENDERIZADO ============================
pub use archflow_renderers::{Renderable, BatchRenderer2D, RenderContext, Bounds};

// ==================== GEOMETRÍA Y ESPACIO ====================
pub use archflow_geometry::{GeometryEngine, IntersectionEngine};
pub use archflow_spatial::{RTreeIndex, SpatialIndexManager, SpatialQueries};

// ==================== ECS HÍBRIDO (OPCIONAL) =================
#[cfg(feature = "animations")]
pub use archflow_ecs_hybrid::{
    RecordRef, Transform, RenderableEcs,
    sync_records_to_ecs_system, dirty_tracking_system,
    TransformBundle, RenderableBundle,
};

// ==================== WASM BRIDGE (INterno) ==================
pub use archflow_wasm_collab::{WasmBridge, SharedBuffer, BinaryDeltaCodec};
```

### 8.2 Capa JS (Minimal)

```javascript
// @archflow/sdk

import { init, Editor } from '@archflow/sdk';

class ArchFlowEditor {
    constructor(canvas, options = {}) {
        // Configuración de features
        this.options = {
            collaboration: true,      // PRD 3.4
            animations: true,         // PRD 3.1.3
            layers: ['context', 'container', 'component', 'code'], // PRD C4
            spatialIndexThreshold: 100,
            ...options
        };
        
        this.wasm = init();
        this.editor = new Editor(this.wasm, this.options);
        
        this.bindEvents(canvas);
        this.renderLoop();
    }
    
    bindEvents(canvas) {
        // Solo traduce eventos DOM → WASM calls
        canvas.addEventListener('pointerdown', (e) => 
            this.editor.onPointerDown(e.clientX, e.clientY));
        
        canvas.addEventListener('keydown', (e) =>
            this.editor.onKeyDown(e.key, e.ctrlKey));
    }
    
    renderLoop() {
        // Lee de SharedBuffer (zero-copy)
        const data = this.wasm.getRenderData();
        
        // Aplica animaciones si están activas
        if (this.options.animations) {
            data.shapes = data.shapes.map(shape => 
                this.applyAnimations(shape));
        }
        
        this.canvas.draw(data); // Solo draw calls
        requestAnimationFrame(() => this.renderLoop());
    }
    
    // === COLABORACIÓN (PRD 3.4) ===
    async connect(url) {
        return this.editor.collab.connect(url);
    }
    
    onRemoteChanges(callback) {
        this.editor.collab.onRemoteChanges(callback);
    }
}
```

---

## 9. Arquitectura SDK Final

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        JavaScript Layer (Minimal)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────────┐  │
│  │  Event DOM  │  │  Canvas 2D  │  │  React/Vue Component            │  │
│  │  Bindings   │  │  Render     │  │  (Wrapper UI, opciones)         │  │
│  └──────┬──────┘  └──────┬──────┘  └─────────────────────────────────┘  │
└─────────┼────────────────┼──────────────────────────────────────────────┘
          │                │
          ▼                ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Rust WASM Layer (Core SDK)                          │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │              archflow-wasm-collab (SharedBuffer)                  │    │
│  │              Zero-copy JS ↔ Rust communication                    │    │
│  └─────────────────────────────────┬───────────────────────────────┘    │
│                                    │                                    │
│  ┌─────────────────────────────┬───┴─────────────────────────────┐     │
│  │   archflow-collab           │    archflow-workspace           │     │
│  │   (CRDT + Sync)             │    (Event Sourcing)             │     │
│  │   ✅ REQUERIDO PRD 3.4      │    (Undo/Redo)                  │     │
│  └─────────────────────────────┴───────────────────────────────┘     │
│                                                                         │
│  ┌─────────────────────────┬────────────────────────────────────────┐  │
│  │   archflow-records      │    archflow-ecs-hybrid                │  │
│  │   (Store, ChangeSet)    │    (Animations + Layers)              │  │
│  │                         │    ⭐ OPCIONAL via feature flag        │  │
│  └─────────────────────────┴────────────────────────────────────────┘  │
│                                                                         │
│  ┌─────────────────────────┬────────────────────────────────────────┐  │
│  │   archflow-spatial      │    archflow-geometry                   │  │
│  │   (R-Tree Adaptativo)   │    (Kurbo Geometry)                    │  │
│  │   Activo si >100 shapes │    (Hit-testing)                       │  │
│  └─────────────────────────┴────────────────────────────────────────┘  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │              archflow-renderers (WebGPU Batch)                    │    │
│  │              + archflow-primitives (Drag, Resize, Routing)       │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 10. Resumen de Decisiones Finales

### ✅ Crates Esenciales (8)

| # | Crate | Justificación PRD | Activación |
|---|-------|-------------------|------------|
| 1 | `archflow-core` | Tipos base del dominio | Always |
| 2 | `archflow-records` | Estado, ChangeSet, FractionalIndex | Always |
| 3 | `archflow-collab` | **3.4 Collaboration System** - Real-time sync | Always |
| 4 | `archflow-workspace` | Documentos, undo/redo, event sourcing | Always |
| 5 | `archflow-primitives` | UI interactions (drag, resize, routing) | Always |
| 6 | `archflow-renderers` | WebGPU batch rendering | Always |
| 7 | `archflow-geometry` | Geometría avanzada, hit-testing | Always |
| 8 | `archflow-wasm-collab` | Zero-copy bridge JS/Rust | Always |

### ⚙️ Crates Adaptativos (2)

| # | Crate | Justificación | Activación |
|---|-------|---------------|------------|
| 9 | `archflow-spatial` | R-Tree con threshold adaptativo | Auto si >100 shapes |
| 10 | `archflow-ecs-hybrid` | Animaciones y capas C4 | `feature = "animations"` |

### 📚 Crates de Soporte (2)

| # | Crate | Uso |
|---|-------|-----|
| 11 | `demo-web` | Referencia de implementación |
| 12 | `archflow-tests` | Regression testing |

---

`✶ Insight ─────────────────────────────────────`
**Patrones de Diseño Aplicados:**

1. **Activación Adaptativa (Spatial)**: El R-Tree migra de O(n) a O(log n) automáticamente al cruzar el threshold, minimizando overhead inicial mientras escala.

2. **Feature Flags (ECS)**: Bevy_ecs es una dependencia pesada que se incluye solo cuando se necesitan animaciones o sistemas de juego, manteniendo el bundle pequeño para editores estáticos.

3. **CRDT como Core (Collab)**: Dado el requisito de colaboración del PRD, el sistema CRDT no es opcional - es la base del sync entre usuarios con resolución automática de conflictos mediante Vector Clocks.
─────────────────────────────────────────────────
