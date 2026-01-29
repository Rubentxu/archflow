# ArchFlow Engine 2D: Épicas de Implementación

**Versión:** 1.12.0
**Fecha:** 2026-01-24
**Basado en:** `docs/prd.md` (Living Architecture Platform Vision)
**Filosofía:** TDD + Investigación Profunda + Rust + WebAssembly

---

## ⚠️ IMPORTANTE: Flujo de Desarrollo Obligatorio

**ANTES de implementar cualquier feature, DEBES:**

1. ✅ Ejecutar las queries de investigación indicadas en cada historia
2. ✅ Documentar los hallazgos y decisiones de arquitectura
3. ✅ Verificar que las APIs de terceros están actualizadas
4. ✅ Solo entonces proceder con la implementación

---

## 📊 Estado General del Proyecto

```
EPIC-001: Core Infrastructure      ████████████████████ 100%
EPIC-002: Base Primitives          ████████████████████ 100%
EPIC-003: Rendering Engine         ████████████████████ 100%
EPIC-004: Interactivity            ████████████████████ 100%
EPIC-005: Connection Routing       ████████████████████ 100%
EPIC-006: Spatial Indexing         ████████████████████ 100%
EPIC-007: Event Sourcing           ████████████████████ 100% ✅
EPIC-008: Animations               ████████████████████ 100% ✅
EPIC-009: External Resources       ████████████████████ 100% ✅
EPIC-010: Incremental Zoom         ████████████████████ 100% ✅
EPIC-011: Developer APIs           ████████████████████ 100% ✅
```

---

## Versiones de Crates (Enero 2026)

| Crate | Versión | Estado |
|-------|---------|--------|
| **archflow-core** | 0.4.0 | ✅ Completo |
| **archflow-ecs** | 0.4.0 | ✅ Completo |
| **archflow-geometry** | 0.4.0 | ✅ Completo |
| **archflow-primitives** | 0.4.0 | ✅ Completo |
| **archflow-renderer** | 0.4.0 | ✅ Completo |
| **archflow-renderer-canvas** | 0.4.0 | ✅ Completo |
| **archflow-renderer-rough** | 0.4.0 | ✅ Completo |
| **archflow-wasm** | 0.4.0 | ✅ Completo |
| **archflow-workspace** | 0.4.0 | ✅ Completo |
| **archflow-demo** | 0.1.0 | ✅ Demo |
| **archflow-demo-server** | 0.1.0 | ✅ Server |
| **kurbo** | 0.13.0 | Dependencia |
| **glam** | 0.31.0 | Dependencia |

---

# ✅ ÉPICA 001: Core Infrastructure

**Objetivo:** Establecer la base del proyecto con estructura de crates, tipos base y configuración de desarrollo.

**Duración:** 2 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-001: Estructura de Crates Base

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Workspace Cargo.toml con todos los crates
- [x] `archflow-core/` con re-exports
- [x] `archflow-ecs/` con traits ECS
- [x] `archflow-geometry/` con kurbo wrappers
- [x] `archflow-renderer/` con traits base
- [x] `archflow-wasm/` para bindings

**Implementación:**
```bash
crates/
├── archflow-core/           # ✅ Tipos centrales y domain primitives
├── archflow-ecs/            # ✅ Sistema de Entidades y Componentes
├── archflow-geometry/       # ✅ Motor de geometría con kurbo
├── archflow-primitives/     # ✅ Formas, estilos, puertos y conexiones
├── archflow-renderer/       # ✅ Traits de renderizador abstractos
├── archflow-renderer-canvas/ # ✅ Backend Canvas 2D
├── archflow-renderer-rough/  # ✅ Renderizador estilo boceto
├── archflow-wasm/           # ✅ Bindings WebAssembly
├── archflow-workspace/      # ✅ Gestión de documentos
└── archflow-demo/           # ✅ Demo interactiva
```

**Commit:** `feat: initial commit - ArchFlow 2D graphics engine`

---

### ✅ US-002: Tipos Base del Dominio

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Vec2 wrapper personalizado con operaciones
- [x] Rect con métodos de geometría
- [x] Color con espacio RGBA
- [x] EntityId como tipo opaco UUID-based
- [x] Transform con translation/rotation/scale

**Implementación:**
- `archflow-core/src/types.rs`: Vec2, Mat3 (custom implementation)
- `archflow-core/src/rect.rs`: Rect con métodos geométricos
- `archflow-core/src/color.rs`: Color RGBA con serde
- `archflow-core/src/entity_id.rs`: EntityId como newtype de UUID
- `archflow-core/src/transform.rs`: Transform con matrices

**Tests:** Todos los tipos tienen tests unitarios.

---

### ✅ US-003: Configuración de Desarrollo

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] .gitignore configurado
- [x] Cargo fmt sin errores
- [x] CI/CD con GitHub Actions (push al repo)
- [x] Documentación inicial

**Archivos:**
- `.gitignore`: target/, .claude/, repo-analysis/
- `README.md`: Documentación principal en inglés
- `README-ES.md`: Documentación en español

---

# ✅ ÉPICA 002: Primitivas Base

**Objetivo:** Implementar todas las primitivas gráficas básicas del motor.

**Duración:** 3 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-010: Sistema de Primitivas

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Primitive trait con Rectangle, Ellipse, Line, Polyline
- [x] Trait Primitive para polimorfismo
- [x] PrimitiveProperties para metadatos
- [x] PrimitiveType enum

**Implementación:**
- `archflow-primitives/src/shapes.rs`: Rectangle, Ellipse, Line, Polyline
- Trait `Primitive` con métodos: `primitive_type()`, `id()`, `local_bounds()`, `global_bounds()`, `contains_point()`

---

### ✅ US-011: Sistema de Estilos

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Style trait unificado
- [x] FillStyle con color y patrón
- [x] StrokeStyle con width, cap, join, dash
- [x] TextStyle con alineación
- [x] EffectStyle para sombras

**Implementación:**
- `archflow-primitives/src/styles.rs`:
  - `FillStyle`: color, opacity, pattern
  - `StrokeStyle`: color, width, line_type (LineCap, LineJoin)
  - `TextStyle`: font, size, align, baseline
  - `EffectStyle`: shadow, blur
  - `ShapeStyle`: combinación de fill y stroke

---

### ✅ US-012: Puertos y Conexiones

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Port struct con id, node_id, name, position
- [x] PortType enum (Any, Number, String, etc.)
- [x] PortDirection (Input, Output, Bidirectional)
- [x] Connection struct con source/target ports
- [x] ConnectionManager para gestión centralizada
- [x] RoutingType (Straight, Orthogonal, Curved, Spline, Smart)

**Implementación:**
- `archflow-primitives/src/connectivity.rs`:
  - `Port`: Puerto de conexión entre nodos
  - `Connection`: Conexión entre dos puertos
  - `PortCollection`: Colección de puertos por nodo
  - `ConnectionManager`: Gestiona todas las conexiones

---

# 🔄 ÉPICA 003: Sistema de Rendering

**Objetivo:** Implementar sistema de rendering con múltiples backends.

**Duración:** 4 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-020: Geometry Engine (kurbo)

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Integración con kurbo para paths y curvas
- [x] BezPath wrapper con operaciones
- [x] ShapePath para formas básicas
- [x] Arc y other curve support

**Implementación:**
- `archflow-geometry/src/kurbo_ext.rs`: Extensions to kurbo
- `archflow-geometry/src/path.rs`: Path building and manipulation

---

### ✅ US-021: Renderer Trait y Canvas 2D

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Renderer trait abstracto
- [x] Canvas2D backend implementación
- [x] Drawing de primitives
- [x] Stroke y fill support

**Implementación:**
- `archflow-renderer-canvas/src/lib.rs`:
  - `CanvasRenderer`: Implementa Renderer trait
  - `render()`: Main rendering method
  - `draw_primitive()`: Polymorphic drawing

---

### ✅ US-022: Rendering Optimizado

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Batch rendering para múltiples shapes
- [x] Layer support
- [x] Viewport culling
- [x] Dirty rect tracking

**Implementación:**
- `archflow-renderer/src/batch.rs`: Batch processing
- `archflow-renderer/src/layers.rs`: Layer management

---

# ⏳ ÉPICA 004: Interactivity & Selection

**Objetivo:** Implementar selección, drag & drop y resize.

**Duración:** 3 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-030: Sistema de Selección

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] SelectionManager
- [x] Selection modes (single, multi, marquee)
- [x] Selection visualization

**Implementación:**
- `archflow-primitives/src/selection.rs`:
  - `SelectionManager`: Manages current selection
  - `SelectionMode`: Enum for selection type
  - `select()`: Add to selection

---

### ✅ US-031: Sistema de Drag & Drop

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] DragManager
- [x] Snap-to-grid
- [x] Constrained movement
- [x] Preview during drag

**Implementación:**
- `archflow-primitives/src/drag_drop.rs`:
  - `DragManager`: Handles drag operations
  - `SnapConfig`: Grid and guide configuration

---

### ✅ US-032: Sistema de Resize

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] ResizeManager
- [x] Handle-based resizing (8 handles)
- [x] Aspect ratio preservation
- [x] Resize preview

**Implementación:**
- `archflow-primitives/src/resize.rs`:
  - `ResizeManager`: Handles resize operations
  - `HandleType`: 8 resize handles
  - `resize()`: Execute resize

---

### ✅ US-033: Hit Testing Optimizado

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] HitTestManager
- [x] Spatial index integration
- [x] Tolerance handling
- [x] Priority handling (topmost first)

**Implementación:**
- `archflow-primitives/src/hit_test.rs`:
  - `HitTestManager`: Performs hit testing
  - `hit_test()`: Returns topmost primitive at point

---

# ✅ ÉPICA 005: Connection Routing

**Objetivo:** Implementar sistema de routing de conexiones entre nodos.

**Duración:** 2 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-040: Routing Ortogonal (Doorway)

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] OrthogonalRouter
- [x] Obstacle avoidance
- [x] Manhattan-style paths

**Implementación:**
- `archflow-primitives/src/routing.rs`:
  - `OrthogonalRouter`: Generates orthogonal paths
  - `find_path()`: Pathfinding with obstacles

---

### ✅ US-041: Marcadores de Flecha

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] ArrowMarker struct
- [x] Line y filled styles
- [x] Auto-orientation

**Implementación:**
- `archflow-primitives/src/arrow.rs`:
  - `ArrowMarker`: Defines arrow head style
  - `orient()`: Rotates marker to path direction

---

# ✅ ÉPICA 006: Spatial Indexing

**Objetivo:** Implementar R-Tree para queries espaciales eficientes.

**Duración:** 2 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-050: R-Tree Implementation

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] rstar integration
- [x] SpatialIndex struct
- [x] insert(), remove(), query()

**Implementación:**
- `archflow-geometry/src/spatial.rs`:
  - `SpatialIndex`: R-Tree wrapper
  - `insert()`: Add object to index
  - `point_query()`: Find objects at point
  - `rect_query()`: Find objects in rect

---

### ✅ US-051: Sincronización ECS → R-Tree

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] SpatialBounds y GlobalBounds components
- [x] SpatialResource con SpatialIndex interno
- [x] spatial_sync_system para sincronización automática
- [x] calculate_global_aabb para AABB con rotación

**Implementación:**
- `archflow-ecs/src/spatial.rs`:
  - `SpatialBounds` y `GlobalBounds` components
  - `SpatialResource` con SpatialIndex interno
  - `spatial_sync_system` para sincronización automática
  - `calculate_global_aabb` para AABB con rotación

**Tests:** 6 tests pasando

---

# ✅ ÉPICA 007: Event Sourcing & Undo/Redo

**Objetivo:** Implementar sistema Git-like de eventos para undo/redo y colaboración.

**Duración:** 3 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-060: Document y EventJournal

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] EventJournal struct básico
- [x] EventStore trait
- [x] InMemoryEventStore implementación
- [x] FileEventStore implementación
- [x] Event sourcing queries

**Implementación:**
- `archflow-core/src/event_sourcing/`:
  - `event.rs`: Domain events definitions
  - `event_journal.rs`: EventJournal struct
  - `event_store.rs`: EventStore trait + implementations
  - `snapshot.rs`: Snapshot for performance
  - `undo_redo_stack.rs`: Undo/Redo management

---

### ✅ US-061: Domain Events

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] DomainEvent trait
- [x] Event implementations para todas las operaciones
- [x] Event sourcing queries
- [x] Event metadata (timestamp, author)

**Implementación:**
- `archflow-core/src/event_sourcing/event.rs`:
  - `DomainEvent`: Trait para todos los eventos
  - `ShapeCreated`, `ShapeModified`, `ShapeDeleted`
  - `SelectionChanged`, `ViewportChanged`
  - Event versioning para migraciones

---

### ✅ US-062: Undo/Redo Manager

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] UndoManager struct
- [x] UndoRedoStack implementación
- [x] Max undo depth limit
- [x] Batch operations support

**Implementación:**
- `archflow-workspace/src/lib.rs`:
  - `UndoManager`: Gestiona undo/redo
  - `undo()`: Revierte último cambio
  - `redo()`: Re-aplica cambio reversado
  - `clear()`: Limpia historial

---

### ✅ US-063: Snapshots y Persistencia

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Snapshot creation
- [x] Snapshot restoration
- [x] File-based persistence
- [x] Performance optimization (cada N eventos)

**Implementación:**
- `archflow-core/src/event_sourcing/snapshot.rs`:
  - `Snapshot`: Serialized state representation
  - `create_snapshot()`: Generate from current state
  - `restore_snapshot()`: Apply snapshot
  - FileEventStore: Persists events to disk

---

# ✅ ÉPICA 008: Sistema de Animaciones

**Objetivo:** Implementar sistema de animaciones keyframe con easing.

**Duración:** 2 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-070: Arquitectura de Animaciones

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Animation trait
- [x] AnimationManager
- [x] Keyframe support
- [x] Duration y easing configuration

**Implementación:**
- `archflow-core/src/animation/`:
  - `Animation`: Trait para animaciones
  - `AnimationManager`: Gestiona animaciones activas
  - `Keyframe`: Frame en el tiempo con valores
  - `update()`: Actualiza estado de animación

---

### ✅ US-071: Easing Functions

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Easing trait
- [x] Standard easings (linear, ease_in, ease_out, ease_in_out)
- [x] Advanced easings (elastic, bounce, back)
- [x] Custom easing support

**Implementación:**
- `archflow-core/src/animation/easing.rs`:
  - `Easing` trait con `ease(t)` método
  - `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`
  - `Elastic`, `Bounce`, `Back` advanced functions
  - `EasingFunction` como función personalizada

**Easing Functions Disponibles:**
- `linear(t)` - Velocidad constante
- `ease_in(t)` - Aceleración gradual
- `ease_out(t)` - Desaceleración gradual
- `ease_in_out(t)` - Acelera y desacelera
- `elastic(t)` - Efecto elástico
- `bounce(t)` - Efecto rebote
- `back_in(t)` - Overshoot inicial
- `cubic_bezier(x1, y1, x2, y2)` - Curva personalizada

---

# ✅ ÉPICA 009: Recursos Externos

**Objetivo:** Implementar soporte para imágenes, videos y recursos externos.

**Duración:** 2 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-080: Sistema de Recursos

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Resource trait
- [x] ImageResource para imágenes
- [x] VideoResource para videos
- [x] ResourceManager con caching
- [x] Async loading

**Implementación:**
- `archflow-core/src/resources.rs`:
  - `Resource`: Trait para recursos
  - `ImageResource`: Carga imágenes (web-sys para WASM)
  - `VideoResource`: Carga y reproduce videos
  - `ResourceManager`: Cache y loading
  - `load()`: Carga asíncrona de recursos

---

### ✅ US-081: HTML Overlays

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] HtmlOverlay struct
- [x] Position y size configuration
- [x] HTML content rendering
- [x] CSS styling support

**Implementación:**
- `archflow-core/src/overlay.rs`:
  - `HtmlOverlay`: Elemento HTML embebido
  - `position()`: Ubicación en canvas
  - `size()`: Dimensiones del overlay
  - `render()`: Genera HTML para el overlay

---

# ✅ ÉPICA 010: Zoom de Detalle Incremental

**Objetivo:** Implementar el sistema de niveles de detalle (modelo C4 interactivo).

**Duración:** 3 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-090: Sistema de Niveles

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] ZoomLevel enum (System, Context, Container, Component, Code)
- [x] ZoomManager
- [x] Zoom transitions con animación
- [x] Level-based detail rendering

**Implementación:**
- `archflow-core/src/zoom.rs`:
  - `ZoomLevel`: Enum con 5 niveles de zoom
  - `ZoomManager`: Gestiona estado y transiciones
  - `zoom_to()`: Cambia a nivel específico
  - `zoom_in()` / `zoom_out()`: Navegación
  - `update()`: Actualiza zoom animado

**Niveles de Zoom:**
| Nivel | Escala | Descripción |
|-------|--------|-------------|
| System | 0.1x | Vista completa del sistema |
| Context | 0.25x | Contexto del subsistema |
| Container | 0.5x | Contenedor (servicio) |
| Component | 1.0x | Nivel de componentes |
| Code | 2.0x | Detalle de código |

---

### ✅ US-091: Jerarquía y Detalle

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] HierarchicalNode para estructura anidada
- [x] Level-of-detail rendering por zoom level
- [x] Progressive disclosure
- [x] Smooth transitions entre niveles

**Implementación:**
- `archflow-core/src/zoom.rs`:
  - `HierarchicalNode`: Nodo en jerarquía
  - `detail_level()`: Retorna nivel según zoom
  - `render_at_level()`: Renderiza con detalle apropiado

---

# ✅ ÉPICA 011: APIs para Desarrolladores

**Objetivo:** Crear APIs accesibles para que otros desarrolladores usen el motor.

**Duración:** 2 semanas  
**Estado:** COMPLETADA ✅

---

### ✅ US-100: API de Alto Nivel

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] Scene API para gestión de shapes
- [x] ShapeFactory para creación
- [x] CanvasBuilder para configuración
- [x] Documentación completa

**Implementación:**
- `archflow-core/src/scene.rs`:
  - `Scene`: Contenedor principal de shapes
  - `add_shape()`, `remove_shape()`, `get_shape()`
  - `clear()`: Limpia todos los shapes

- `archflow-core/src/shapes.rs`:
  - `ShapeFactory`: Creador de formas
  - `create_rectangle()`, `create_ellipse()`, `create_line()`

- `archflow-core/src/canvas.rs`:
  - `CanvasBuilder`: Configuración de canvas
  - `with_size()`, `with_background()`, `build()`

---

### ✅ US-101: API de Formas Personalizadas

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] CustomShape trait
- [x] Registration system
- [x] Serialization support
- [x] Ejemplos de uso

**Implementación:**
- `archflow-primitives/src/custom_shape.rs`:
  - `CustomShape`: Trait para formas personalizadas
  - `register_shape()`: Registra nueva forma
  - `ShapeRegistry`: Catálogo de formas disponibles

---

## 📦 Dependencias del Proyecto

```toml
[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde", "js"] }
thiserror = "2.0"
anyhow = "1.0"

# Graphics & Geometry
kurbo = "0.13"           # 2D curves and paths
glam = "0.31"            # SIMD math library
euclid = "0.22"          # Geometry primitives
rstar = "0.12"           # R-Tree spatial index

# ECS
bevy_ecs = "0.18"        # Entity Component System

# Rendering
cosmic-text = "0.16"     # Text rendering
lyon = "1.0"             # Path tessellation

# WebAssembly
wasm-bindgen = "0.2.108"
web-sys = { version = "0.3", features = [
    "CanvasRenderingContext2d",
    "HtmlCanvasElement",
    "HtmlImageElement",
    "HtmlVideoElement",
    "Window",
    "Element",
    "MouseEvent",
] }

[dev-dependencies]
criterion = "0.5"
```

---

## 🚀 Demo Interactiva WASM v2.0

La demo interactiva está operativa en `http://localhost:8080/`

### Componentes de la Demo

| Componente | Estado | Descripción |
|------------|--------|-------------|
| ArchFlowEngine WASM | ✅ | Engine bindeado a JS con 18+ métodos |
| Shapes API | ✅ | Rectángulos, elipses, líneas con JSON |
| Zoom System | ✅ | Niveles system/context/component/code |
| Grid Snapping | ✅ | Utilidad para alineación |
| Color Palette | ✅ | Colores primarios y de acento |
| Animation Easing | ✅ | ease_in_out, ease_elastic, ease_bounce |
| Event Sourcing | ✅ | API de eventos para domain model |
| Demo Server | ✅ | Servidor HTTP en puerto 8080 |
| Demo Page | ✅ | Interfaz HTML completa con canvas |

### Endpoints de la Demo

```
http://localhost:8080/              - Página interactiva
http://localhost:8080/pkg/archflow_wasm.js  - WASM bindings
http://localhost:8080/pkg/archflow_wasm_bg.wasm - Binary
http://localhost:8080/api/health    - Health check
http://localhost:8080/api/shapes    - Shapes JSON
```

### API de Alto Nivel (JavaScript)

```javascript
import init, { ArchFlowEngine } from './pkg/archflow_wasm.js';

const engine = new ArchFlowEngine();
engine.configure_canvas(800, 600, '#f0f0f0');

// Shapes
const rectId = engine.add_rectangle(100, 100, 120, 80, '#3498db');
const ellipseId = engine.add_ellipse(300, 150, 40, 40, '#9b59b6');
engine.add_line(500, 120, 600, 200, '#e74c3c');

// Zoom
engine.zoom_in();
engine.zoom_out();
engine.zoom_to('component');

// Shapes data
const shapes = JSON.parse(engine.get_all_shapes_json());
```

---

## 📚 Documentación de Referencia

| Documento | Descripción |
|-----------|-------------|
| `README.md` | Visión del producto y estado actual |
| `README-ES.md` | Documentación en español |
| `docs/prd.md` | Product Requirements Document completo |
| `docs/ARCHITECTURE-DESIGN.md` | Decisiones de arquitectura técnica |
| `docs/EPICS-ARCHFLOW-V2.md` | Épicas detalladas v2.0 |

---

**Documento preparado por:** ArchFlow Development Team  
**Última actualización:** 2026-01-24  
**Versión:** 1.12.0 (Demo WASM v2.0 operativa)
