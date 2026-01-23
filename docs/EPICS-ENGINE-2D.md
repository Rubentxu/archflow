# ArchFlow Engine 2D: Épicas de Implementación

**Versión:** 1.9.0  
**Fecha:** 2026-01-23  
**Basado en:** `docs/PRD-pipeline-dsl.md` (Living Architecture Platform Vision)  
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
EPIC-003: Rendering Engine         ██████████████░░░░░░░  55%
EPIC-004: Interactivity            ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-005: Connection Routing       ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-006: Spatial Indexing         ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-007: Event Sourcing           ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-008: Animations               ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-009: External Resources       ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-010: Incremental Zoom         ░░░░░░░░░░░░░░░░░░░░   0%
EPIC-011: Developer APIs           ░░░░░░░░░░░░░░░░░░░░   0%
```

---

## Versiones de Crates (Enero 2026)

| Crate | Versión | Estado |
|-------|---------|--------|
| **archflow-core** | 0.1.0 | ✅ Completo |
| **archflow-ecs** | 0.1.0 | ✅ Completo |
| **archflow-geometry** | 0.1.0 | ✅ Completo |
| **archflow-primitives** | 0.1.0 | ✅ Completo |
| **archflow-renderer** | 0.1.0 | ✅ Completo |
| **archflow-renderer-canvas** | 0.1.0 | ✅ Completo |
| **archflow-renderer-rough** | 0.1.0 | ✅ Completo |
| **archflow-workspace** | 0.1.0 | ✅ Completo |
| **archflow-wasm** | 0.1.0 | ✅ Completo |
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
├── archflow-workspace/      # ✅ Gestión de documentos
└── archflow-wasm/           # ✅ Bindings WebAssembly
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

**Objetivo:** Implementar el sistema de rendering con arquitectura separable.

**Duración:** 4 semanas  
**Estado:** EN PROGRESO (35%)

---

### ✅ US-020: Geometry Engine (kurbo)

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] GeometryEngine con kurbo para Bézier
- [x] Cálculo de bounding boxes
- [x] Detección de intersecciones (SAT, ray casting)
- [x] Conversión PathCommand → kurbo::BezPath
- [x] Path simplification (Ramer-Douglas-Peucker)

**Implementación:**
- `archflow-geometry/src/geometry.rs`:
  - `GeometryEngine`: Operaciones geométricas básicas
  - `DiscretizeConfig`: Configuración para discretización
  - Métodos: `distance()`, `angle()`, `quadratic_bezier()`, `cubic_bezier()`, `ellipse_points()`
  - `bounds_of_points()`, `point_in_rect()`, `rects_intersect()`

- `archflow-geometry/src/path.rs`:
  - `PathEngine`: Operaciones sobre paths
  - `PathElement`: MoveTo, LineTo, QuadTo, CurveTo, Close
  - `SimplifyConfig`: Configuración para simplificación
  - Métodos: `from_elements()`, `to_elements()`, `simplify()`, `bounds()`, `length()`
  - `rect_path()`, `ellipse_path()`, `line_path()`, `arc_path()`

- `archflow-geometry/src/intersection.rs`:
  - `IntersectionEngine`: Detección de intersecciones
  - `IntersectionType`: None, Point, Line, Segment, Area
  - `HitTestConfig`: Configuración para hit testing
  - Métodos: `rect_rect()`, `point_in_rect()`, `point_in_polygon()`, `segment_segment()`, `line_circle()`, `path_path()`, `polygons_intersect()`

**Tests:** 26 tests pasando en `archflow-geometry`

---

### ✅ US-021: Renderer Trait y Canvas 2D

**Estado:** COMPLETADO ✅

**Criterios de Aceptación:**
- [x] trait Renderer con métodos base
- [x] Canvas2DRenderer implementado con web-sys
- [x] Path trait abstracto para diferentes backends
- [x] StrokeStyle y FillStyle configurables
- [x] FontStyle para texto
- [x] RoughRenderer actualizado para implementar trait completo
- [x] Tests pasando (26 tests en geometry + todos los crates)

**Investigación Realizada:**
- web-sys CanvasRenderingContext2D features requeridas
- wasm-bindgen JsCast para type casting
- APIs deprecated de set_fill_style/set_stroke_style

**Implementación:**
- `archflow-renderer/src/lib.rs`:
  - `Renderer` trait: clear, save, restore, translate, rotate, scale, reset_transform
  - `draw_rect`, `draw_ellipse`, `draw_path`, `fill_path`, `stroke_path`
  - `draw_text` con FontStyle
  - `draw_image`, `draw_image_slice`
  - `Path` trait: to_svg_path, bounds, is_empty, length
  - `Image` trait: width, height, data, pixel_format
  - `StrokeStyle`: color, width, line_cap, line_join, dash_pattern, miter_limit
  - `FillStyle`, `FontStyle`, `FontFamily`, `FontWeight`, `LineCap`, `LineJoin`
  - `CompositeOperation`, `RendererConfig`

- `archflow-renderer-canvas/src/lib.rs`:
  - `CanvasRenderer`: Implementación usando CanvasRenderingContext2d
  - `CanvasImageBitmap`: Wrapper para imágenes
  - `CanvasRendererBuilder`: Builder pattern
  - Conversión color → JsValue para web-sys

- `archflow-renderer-rough/src/lib.rs`:
  - `RoughRenderer<R>`: Wrapper decorator que implementa Renderer

**Tests:** 26 tests pasando en `archflow-geometry` + 0 failures en todos los crates

---

### US-022: Rendering Optimizado

**Estado:** PENDIENTE ⏳

**Criterios de Aceptación:**
- [ ] Culling espacial
- [ ] Batch rendering por tipo
- [ ] FPS counter para debugging

**Tasks:**
- [ ] Implementar RenderContext con dirty rect tracking
- [ ] Implementar spatial culling con archflow-geometry
- [ ] Implementar batch rendering por tipo de primitiva
- [ ] Añadir FPS counter

---

# ⏳ ÉPICA 004: Interactivity & Selection

**Objetivo:** Implementar interacción del usuario (drag, resize, select).

**Duración:** 3 semanas  
**Estado:** PENDIENTE

---

### US-030: Sistema de Selección

**Criterios de Aceptación:**
- [ ] Click para seleccionar
- [ ] Shift+Click selección múltiple
- [ ] Drag selection
- [ ] Visual feedback

### US-031: Sistema de Drag & Drop

**Criterios de Aceptación:**
- [ ] Draggable component
- [ ] Feedback visual instantáneo
- [ ] Snap to grid
- [ ] Multi-drag

### US-032: Sistema de Resize

**Criterios de Aceptación:**
- [ ] Resizable component
- [ ] Handles en corners y edges
- [ ] Aspect ratio lock
- [ ] Min/max constraints

### US-033: Hit Testing Optimizado

**Criterios de Aceptación:**
- [ ] Hit testing por punto
- [ ] Z-order correcto
- [ ] Menos de 1ms para 1000 objetos

---

# ⏳ ÉPICA 005: Connection Routing

**Objetivo:** Implementar conexiones inteligentes entre formas.

**Duración:** 2 semanas  
**Estado:** PENDIENTE

---

### US-040: Routing Ortogonal (Doorway)

**Criterios de Aceptación:**
- [ ] RoutingMode::Straight
- [ ] RoutingMode::Orthogonal heurística L-shape
- [ ] RoutingMode::Curved (Bézier)
- [ ] Evitar obstáculos

### US-041: Marcadores de Flecha

**Criterios de Aceptación:**
- [ ] MarkerType: None, Arrow, Circle, Diamond
- [ ] Custom markers
- [ ] Tamaño y color configurable

---

# ⏳ ÉPICA 006: Spatial Indexing

**Objetivo:** Implementar R-Tree para queries espaciales eficientes.

**Duración:** 2 semanas  
**Estado:** PENDIENTE

---

### US-050: R-Tree Implementation

**Criterios de Aceptación:**
- [ ] SpatialIndex con rstar
- [ ] insert() y remove()
- [ ] query_viewport, query_point, query_area

### US-051: Sincronización ECS → R-Tree

**Criterios de Aceptación:**
- [ ] SpatialSyncSet system
- [ ] Dirty tracking
- [ ] Sync en Changed<Transform>

---

# ⏳ ÉPICA 007: Event Sourcing & Undo/Redo

**Objetivo:** Implementar sistema Git-like de eventos para undo/redo y colaboración.

**Duración:** 3 semanas  
**Estado:** PENDIENTE

---

### US-060: Document y EventJournal

### US-061: Domain Events

### US-062: Undo/Redo Manager

### US-063: Snapshots y Persistencia

---

# ⏳ ÉPICA 008: Sistema de Animaciones

**Objetivo:** Implementar sistema de animaciones keyframe.

**Duración:** 2 semanas  
**Estado:** PENDIENTE

---

### US-070: Arquitectura de Animaciones

### US-071: Easing Functions

---

# ⏳ ÉPICA 009: Recursos Externos

**Objetivo:** Implementar soporte para imágenes, videos y HTML embebido.

**Duración:** 2 semanas  
**Estado:** PENDIENTE

---

### US-080: Sistema de Recursos

### US-081: HTML Overlays

---

# ⏳ ÉPICA 010: Zoom de Detalle Incremental

**Objetivo:** Implementar el sistema de niveles de detalle (modelo C4 interactivo).

**Duración:** 3 semanas  
**Estado:** PENDIENTE

---

### US-090: Sistema de Niveles

### US-091: Jerarquía y Detalle

---

# ⏳ ÉPICA 011: APIs para Desarrolladores

**Objetivo:** Crear APIs accesibles para que otros desarrolladores usen el motor.

**Duración:** 2 semanas  
**Estado:** PENDIENTE

---

### US-100: API de Alto Nivel

### US-101: API de Formas Personalizadas

---

## 📦 Dependencias del Proyecto

```toml
[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
thiserror = "2.0"
anyhow = "1.0"

# Graphics & Geometry
kurbo = "0.13"           # 2D curves and paths
glam = "0.31"            # SIMD math library

# WebAssembly
wasm-bindgen = "0.2.108"
web-sys = { version = "0.3", features = [
    "CanvasRenderingContext2d",
    "HtmlCanvasElement",
    "Window",
    "Element",
    "MouseEvent",
] }

[dev-dependencies]
criterion = "0.5"
```

---

## 🚀 Próximos Pasos Inmediatos

1. **Mejorar Renderer Canvas 2D** (US-021)
   - Implementar drawing de paths con kurbo::BezPath
   - Añadir stroke y fill reales
   - Implementar Image drawing

2. **Añadir Tests de Rendering**
   - Tests para cada método del Renderer
   - Tests de integración con archflow-primitives

3. **Comenzar US-022: Rendering Optimizado**
   - Implementar RenderContext
   - Añadir spatial culling

---

## 📚 Documentación de Referencia

| Documento | Descripción |
|-----------|-------------|
| `README.md` | Visión del producto y estado actual |
| `README-ES.md` | Documentación en español |
| `docs/prd.md` | Product Requirements Document completo |
| `docs/ARCHITECTURE-DESIGN.md` | Decisiones de arquitectura técnica |

---

**Documento preparado por:** ArchFlow Development Team  
**Última actualización:** 2026-01-23  
**Versión:** 1.9.0
