# ArchFlow Engine 2D: Épicas de Implementación

**Versión:** 1.7.0  
**Fecha:** 2026-01-23  
**Basado en:** `docs/analysis/ENGINE-2D-ANALYSIS.md` (v1.6)  
**Filosofía:** TDD + Investigación Profunda + bevy_ecs 0.18

---

## ⚠️ IMPORTANTE: Flujo de Desarrollo Obligatorio

**ANTES de implementar cualquier feature, DEBES:**

1. ✅ Ejecutar las queries de Perplexity indicadas en cada historia
2. ✅ Documentar los hallazgos y decisiones de arquitectura
3. ✅ Verificar que las APIs de terceros están actualizadas
4. ✅ Solo entonces proceder con la implementación

---

## Versiones de Crates (Enero 2025)

| Crate | Versión | Notas |
|-------|---------|-------|
| **bevy_ecs** | 0.18.0 | 5 versiones nuevas vs 0.13 |
| **glam** | 0.31.0 | SIMD, zerocopy support |
| **kurbo** | 0.13.0 | 2D curves |
| **rstar** | 0.12.2 | R*-tree spatial index |
| **bincode** | 3.0.0 | ⚠️ Breaking changes |
| **zstd** | 0.13.3 | Compresión |
| **wasm-bindgen** | 0.2.108 | JS interop |

---

# ÉPICA 001: Core Infrastructure

**Objetivo:** Establecer la base del proyecto con estructura de crates, tipos base y configuración de desarrollo.

**Duración:** 2 semanas

---

### US-001: Estructura de Crates Base

**Como** desarrollador quiero una estructura de crates bien definida para que el proyecto sea mantenible y escalable.

**Criterios de Aceptación:**
- [ ] Workspace Cargo.toml con todos los crates
- [ ] `archflow-core/` con re-exports
- [ ] `archflow-ecs/` con bevy_ecs
- [ ] `archflow-geometry/` con euclid/kurbo wrappers
- [ ] `archflow-renderer/` con traits base
- [ ] `archflow-wasm/` para bindings

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "bevy_ecs 0.18 workspace Rust multi-crate structure best practices 2025"
- [ ] **Query 2:** "Rust Cargo.toml workspace features conditional compilation examples"
- [ ] **Query 3:** "GitHub Actions Rust CI/CD pipeline cargo test clippy fmt 2025"
- [ ] **Output:** Documento de arquitectura de crates con decisiones

**Tasks:**
- [ ] Crear estructura de directorios
- [ ] Configurar Cargo.toml workspace
- [ ] Crear crates individuales
- [ ] Configurar feature flags
- [ ] Configurar rust-analyzer
- [ ] Configurar CI/CD

---

### US-002: Tipos Base del Dominio

**Como** desarrollador quiero tipos base bien definidos (Vec2, Rect, Color, EntityId) para que todas las capas usen los mismos tipos.

**Criterios de Aceptación:**
- [ ] Vec2 wrapper sobre glam con operaciones
- [ ] Rect con métodos de geometría
- [ ] Color con espacio RGBA
- [ ] EntityId como tipo opaco
- [ ] Transform con translation/rotation/scale

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "glam 0.31 Vec2 Vec3 Mat3 best practices Rust game development"
- [ ] **Query 2:** "Rust newtype wrapper vs type alias performance benchmarks"
- [ ] **Query 3:** "euclid 0.22 Box2D Rect2D comparison with glam for 2D graphics"
- [ ] **Query 4:** "Rust UUID EntityId serialization performance bincode 3.0"
- [ ] **Output:** Documento de decisiones de tipos con benchmarks

**Tasks:**
- [ ] Implementar Vec2 (glam wrapper)
- [ ] Implementar Rect (euclid wrapper)
- [ ] Implementar Color
- [ ] Implementar EntityId (UUID-based)
- [ ] Implementar Transform
- [ ] Tests unitarios

---

### US-003: Configuración de Desarrollo

**Como** desarrollador quiero un entorno de desarrollo configurado para poder escribir código eficientemente.

**Criterios de Aceptación:**
- [ ] cargo fmt configurado
- [ ] cargo clippy sin warnings
- [ ] CI con cargo check
- [ ] Documentación con cargo doc

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "rustfmt.toml configuration best practices 2025"
- [ ] **Query 2:** "cargo clippy restrictive warnings for library development"
- [ ] **Query 3:** "Rust GitHub Actions cache cargo target directory optimization"
- [ ] **Output:** Archivos de configuración optimizados

**Tasks:**
- [ ] Configurar rustfmt.toml
- [ ] Configurar clippy.toml
- [ ] Configurar .cargo/config.toml
- [ ] Configurar GitHub Actions

---

# ÉPICA 002: Primitivas Base

**Objetivo:** Implementar todas las primitivas gráficas básicas del motor.

**Duración:** 3 semanas

---

### US-010: Sistema de Primitivas

**Como** desarrollador quiero un enum `Primitive` con todas las formas para que el renderer pueda dibujar cualquier forma.

**Criterios de Aceptación:**
- [ ] enum Primitive con Rect, Ellipse, Line, Polyline, Path, Text, Image
- [ ] Formas de diagramación: Arrow, Connector, Cloud, Cylinder, Cube, Diamond
- [ ] GroupPrimitive y ClipPrimitive
- [ ] Trait PrimitiveRenderer

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "tldraw ShapeUtil architecture pattern Rust implementation 2025"
- [ ] **Query 2:** "Rust enum vs trait object performance serialization bevy_ecs"
- [ ] **Query 3:** "SVG path command specification Arc CubicBezier Rust kurbo"
- [ ] **Query 4:** "Rust serialization strategy for recursive enum with trait objects"
- [ ] **Output:** Documento de arquitectura de primitivas con benchmarks

**Tasks:**
- [ ] Definir enum Primitive
- [ ] Implementar RectPrimitive
- [ ] Implementar EllipsePrimitive
- [ ] Implementar PathPrimitive
- [ ] Implementar formas de diagramación
- [ ] Tests de serialización

---

### US-011: Sistema de Estilos

**Como** desarrollador quiero un sistema de estilos extensible para controlar la apariencia de las primitivas.

**Criterios de Aceptación:**
- [ ] struct Style con stroke y fill
- [ ] StrokeStyle (color, width, cap, join, dash)
- [ ] FillStyle (color, pattern, gradient)
- [ ] Shadow y efectos

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust style system design patterns for graphics engine"
- [ ] **Query 2:** "CSS stroke-dasharray equivalent Rust Canvas API"
- [ ] **Query 3:** "SVG filter effects Rust implementation shadow blur"
- [ ] **Query 4:** "Rust serde serialization backwards compatibility schema evolution"
- [ ] **Output:** Documento de diseño de sistema de estilos

**Tasks:**
- [ ] Implementar Style struct
- [ ] Implementar StrokeStyle
- [ ] Implementar FillStyle
- [ ] Implementar Shadow
- [ ] StyleRegistry

---

### US-012: Puertos y Conexiones

**Como** arquitecto quiero conectar formas en puntos específicos (puertos) para crear diagramas claros.

**Criterios de Aceptación:**
- [ ] Ports component en entidades
- [ ] PortBinding: Fixed, AutoClosest, AutoCompass
- [ ] Cálculo dinámico de posición de puerto

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "React Flow handles ports implementation pattern architecture"
- [ ] **Query 2:** "Node connection points calculation algorithm bounding box"
- [ ] **Query 3:** "tldraw ports handles implementation source code analysis"
- [ ] **Output:** Documento de diseño de sistema de puertos

**Tasks:**
- [ ] Implementar Ports component
- [ ] Implementar Port enum
- [ ] Implementar PortBinding
- [ ] Implementar get_port_position()

---

# ÉPICA 003: Sistema de Rendering

**Objetivo:** Implementar el sistema de rendering con arquitectura separable.

**Duración:** 4 semanas

---

### US-020: Geometry Engine (kurbo)

**Como** desarrollador quiero un motor de geometría basado en kurbo para calcular intersecciones, bounds y paths.

**Criterios de Aceptación:**
- [ ] GeometryEngine con kurbo para Bézier
- [ ] Cálculo de bounding boxes
- [ ] Detección de intersecciones
- [ ] Conversión PathCommand → kurbo::BezPath

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "kurbo 0.13 BezPath performance benchmarks Rust 2025"
- [ ] **Query 2:** "curve intersection algorithm Bézier Rust implementation"
- [ ] **Query 3:** "kurbo vs lyon for 2D geometry Rust Canvas rendering"
- [ ] **Query 4:** "path tessellation algorithms Canvas 2D performance"
- [ ] **Output:** Documento de arquitectura de geometría

**Tasks:**
- [ ] Configurar kurbo dependencia
- [ ] Implementar GeometryEngine
- [ ] Implementar bezier_bounds()
- [ ] Implementar intersect_paths()
- [ ] Tests de geometría

---

### US-021: Renderer Trait y Canvas 2D

**Como** desarrollador quiero un trait Renderer abstracto para poder implementar diferentes backends.

**Criterios de Aceptación:**
- [ ] trait Renderer con métodos base
- [ ] Canvas2DRenderer implementado
- [ ] Soporte WebSys
- [ ] Tests de rendering

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust trait based renderer architecture pattern Canvas WebGPU"
- [ ] **Query 2:** "web-sys CanvasRenderingContext2D Rust wasm performance 2025"
- [ ] **Query 3:** "immediate mode vs retained mode rendering Rust ECS"
- [ ] **Query 4:** "Canvas 2D state stack save restore pattern Rust"
- [ ] **Output:** Documento de arquitectura de rendering

**Tasks:**
- [ ] Definir trait Renderer
- [ ] Implementar Canvas2DRenderer
- [ ] Implementar métodos de drawing
- [ ] Tests de rendering

---

### US-022: Rendering Optimizado

**Como** usuario quiero que el rendering sea rápido para tener 60fps incluso con muchos objetos.

**Criterios de Aceptación:**
- [ ] Culling espacial
- [ ] Batch rendering por tipo
- [ ] FPS counter para debugging

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "spatial culling algorithm R-Tree view frustum optimization"
- [ ] **Query 2:** "Canvas 2D batch rendering performance Rust wasm 2025"
- [ ] **Query 3:** "dirty rect tracking optimization Canvas rendering"
- [ ] **Query 4:** "Path2D caching strategy for static shapes Canvas"
- [ ] **Output:** Documento de optimizaciones

**Tasks:**
- [ ] Implementar RenderContext
- [ ] Implementar culling
- [ ] Implementar batch rendering
- [ ] FPS counter

---

# ÉPICA 004: Interactivity & Selection

**Objetivo:** Implementar interacción del usuario (drag, resize, select).

**Duración:** 3 semanas

---

### US-030: Sistema de Selección

**Como** usuario quiero seleccionar formas con el mouse para poder modificarlas después.

**Criterios de Aceptación:**
- [ ] Click para seleccionar
- [ ] Shift+Click selección múltiple
- [ ] Drag selection
- [ ] Visual feedback

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "tldraw selection system implementation hit testing source code"
- [ ] **Query 2:** "bevy_ecs selection state management pattern UI interaction"
- [ ] **Query 3:** "Canvas drag selection rectangle algorithm Rust"
- [ ] **Query 4:** "selection overlay rendering pattern Canvas 2D"
- [ ] **Output:** Documento de diseño de selección

**Tasks:**
- [ ] Implementar Selectable component
- [ ] Implementar selection state
- [ ] Implementar pointer handlers
- [ ] Selection overlay

---

### US-031: Sistema de Drag & Drop

**Como** usuario quiero arrastrar formas con el mouse para repositionar elementos.

**Criterios de Aceptación:**
- [ ] Draggable component
- [ ] Feedback visual instantáneo
- [ ] Snap to grid
- [ ] Multi-drag

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "bevy_ecs drag and drop system implementation tutorial 2025"
- [ ] **Query 2:** "snap to grid algorithm incremental update performance"
- [ ] **Query 3:** "multiple objects drag transformation ECS system"
- [ ] **Output:** Documento de diseño de drag & drop

**Tasks:**
- [ ] Implementar Draggable component
- [ ] Implementar pointer handlers
- [ ] Implementar snap system
- [ ] Multi-drag support

---

### US-032: Sistema de Resize

**Como** usuario quiero redimensionar formas con handles para cambiar el tamaño.

**Criterios de Aceptación:**
- [ ] Resizable component
- [ ] Handles en corners y edges
- [ ] Aspect ratio lock
- [ ] Min/max constraints

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "tldraw resize handles implementation source code analysis"
- [ ] **Query 2:** "resize handle corner edge algorithm bounding box transformation"
- [ ] **Query 3:** "aspect ratio preserve resize algorithm math"
- [ ] **Output:** Documento de diseño de resize

**Tasks:**
- [ ] Implementar Resizable component
- [ ] Implementar handles
- [ ] Implementar resize logic
- [ ] Constraints

---

### US-033: Hit Testing Optimizado

**Como** usuario quiero que las formas respondan inmediatamente al mouse para una experiencia fluida.

**Criterios de Aceptación:**
- [ ] Hit testing por punto
- [ ] Z-order correcto
- [ ] Menos de 1ms para 1000 objetos

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "R-Tree point location query algorithm spatial index"
- [ ] **Query 2:** "hierarchical hit testing z-order ECS bevy_ecs"
- [ ] **Query 3:** "point in polygon algorithm Bézier curve Rust"
- [ ] **Output:** Documento de optimización de hit testing

**Tasks:**
- [ ] Implementar query_point
- [ ] Implementar query_area
- [ ] Optimizar rendimiento
- [ ] Benchmarks

---

# ÉPICA 005: Connection Routing

**Objetivo:** Implementar conexiones inteligentes entre formas.

**Duración:** 2 semanas

---

### US-040: Routing Ortogonal (Doorway)

**Como** arquitecto quiero conexiones con codos de 90 grados para diagramas limpios.

**Criterios de Aceptación:**
- [ ] RoutingMode::Straight
- [ ] RoutingMode::Orthogonal heurística L-shape
- [ ] RoutingMode::Curved (Bézier)
- [ ] Evitar obstáculos

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "orthogonal routing algorithm A* obstacle avoidance diagram"
- [ ] **Query 2:** "tldraw connection routing implementation source code"
- [ ] **Query 3:** "Manhattan routing path finding algorithm 2025"
- [ ] **Query 4:** "doorway routing heuristic L-shape optimization"
- [ ] **Output:** Documento de algoritmos de routing

**Tasks:**
- [ ] Implementar OrthogonalConfig
- [ ] Implementar find_doors()
- [ ] Implementar find_orthogonal_path()
- [ ] Tests de routing

---

### US-041: Marcadores de Flecha

**Como** usuario quiero diferentes estilos de flechas para indicar dirección del flujo.

**Criterios de Aceptación:**
- [ ] MarkerType: None, Arrow, Circle, Diamond
- [ ] Custom markers
- [ ] Tamaño y color configurable

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "SVG marker definition arrow circle diamond Rust Canvas"
- [ ] **Query 2:** "arrowhead geometry calculation angle direction Rust"
- [ ] **Output:** Documento de marcadores

**Tasks:**
- [ ] Definir MarkerType
- [ ] Implementar marker rendering
- [ ] Custom markers

---

# ÉPICA 006: Spatial Indexing

**Objetivo:** Implementar R-Tree para queries espaciales eficientes.

**Duración:** 2 semanas

---

### US-050: R-Tree Implementation

**Como** desarrollador quiero un R-Tree con rstar para queries O(log n) en lugar de O(n).

**Criterios de Aceptación:**
- [ ] SpatialIndex con rstar
- [ ] insert() y remove()
- [ ] query_viewport, query_point, query_area

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "rstar 0.12 R*-tree Rust documentation examples 2025"
- [ ] **Query 2:** "R-Tree vs Quadtree spatial index performance comparison"
- [ ] **Query 3:** "bulk load R-Tree algorithm incremental insert"
- [ ] **Output:** Documento de arquitectura de spatial index

**Tasks:**
- [ ] Configurar rstar
- [ ] Implementar SpatialEntity
- [ ] Implementar SpatialIndex
- [ ] Tests de R-Tree

---

### US-051: Sincronización ECS → R-Tree

**Como** desarrollador quiero que el R-Tree se sincronice con el ECS para que los queries siempre estén actualizados.

**Criterios de Aceptación:**
- [ ] SpatialSyncSet system
- [ ] Dirty tracking
- [ ] Sync en Changed<Transform>

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "bevy_ecs Changed filter trigger every frame optimization"
- [ ] **Query 2:** "dirty flag pattern ECS system update synchronization"
- [ ] **Query 3:** "R-Tree incremental update vs rebuild benchmark"
- [ ] **Output:** Documento de sincronización

**Tasks:**
- [ ] Implementar mark_dirty()
- [ ] Implementar sync_dirty()
- [ ] Implementar spatial system

---

# ÉPICA 007: Event Sourcing & Undo/Redo

**Objetivo:** Implementar sistema Git-like de eventos para undo/redo y colaboración.

**Duración:** 3 semanas

---

### US-060: Document y EventJournal

**Como** desarrollador quiero un Document struct con EventJournal para implementar el modelo Git-like.

**Criterios de Aceptación:**
- [ ] Document struct con commits
- [ ] EventJournal con commits indexados
- [ ] Branch management

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "event sourcing Rust implementation patterns 2025"
- [ ] **Query 2:** "bincode 3.0 serialization configuration options"
- [ ] **Query 3:** "Git-like document model delta encoding optimization"
- [ ] **Output:** Documento de event sourcing

**Tasks:**
- [ ] Implementar Document struct
- [ ] Implementar EventJournal
- [ ] Implementar Commit struct
- [ ] Branch management

---

### US-061: Domain Events

**Como** desarrollador quiero un enum DomainEvent completo para representar todos los cambios posibles.

**Criterios de Aceptación:**
- [ ] EntityCreated/Updated/Deleted
- [ ] EntityMoved/Rotated/Scaled
- [ ] Connector events
- [ ] Selection events

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust enum serialization versioning backwards compatible"
- [ ] **Query 2:** "event sourcing domain event design patterns DDD"
- [ ] **Query 3:** "tldraw history store implementation source code"
- [ ] **Output:** Documento de eventos de dominio

**Tasks:**
- [ ] Definir DomainEvent enum
- [ ] Implementar eventos
- [ ] Tests de eventos

---

### US-062: Undo/Redo Manager

**Como** usuario quiero hacer undo y redo de mis acciones para corregir errores fácilmente.

**Criterios de Aceptación:**
- [ ] UndoManager struct
- [ ] Undo stack y redo stack
- [ ] Keyboard shortcuts
- [ ] Undo grouping

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust undo redo implementation pattern ECS state"
- [ ] **Query 2:** "transaction grouping undo multiple events optimization"
- [ ] **Query 3:** "keyboard shortcut Rust winit event handling"
- [ ] **Output:** Documento de undo/redo

**Tasks:**
- [ ] Implementar UndoManager
- [ ] Implementar undo/redo
- [ ] Keyboard handlers
- [ ] Tests

---

### US-063: Snapshots y Persistencia

**Como** usuario quiero guardar y cargar documentos para poder retomar mi trabajo.

**Criterios de Aceptación:**
- [ ] create_snapshot() / load_snapshot()
- [ ] Zstd compression
- [ ] Checksum verification
- [ ] Formato "AFLW"

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "zstd Rust compression level performance benchmark 2025"
- [ ] **Query 2:** "Blake3 Rust checksum performance comparison"
- [ ] **Query 3:** "document file format design binary header versioning"
- [ ] **Output:** Documento de persistencia

**Tasks:**
- [ ] Implementar DocumentSnapshot
- [ ] Implementar save/load
- [ ] Compression
- [ ] Checksum

---

# ÉPICA 008: Sistema de Animaciones

**Objetivo:** Implementar sistema de animaciones keyframe.

**Duración:** 2 semanas

---

### US-070: Arquitectura de Animaciones

**Como** desarrollador quiero un sistema de animaciones basado en keyframes para crear animaciones suaves.

**Criterios de Aceptación:**
- [ ] AnimationCurve trait
- [ ] KeyframeAnimation struct
- [ ] RepeatMode (Once, Loop, PingPong)
- [ ] AnimationState

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust animation system ECS bevy_ecs pattern 2025"
- [ ] **Query 2:** "easing functions CSS equivalent Rust implementation"
- [ ] **Query 3:** "keyframe interpolation quaternion SLERP Vec2 lerp"
- [ ] **Output:** Documento de animaciones

**Tasks:**
- [ ] Definir AnimationCurve trait
- [ ] Implementar KeyframeAnimation
- [ ] Definir RepeatMode
- [ ] Animation struct

---

### US-071: Easing Functions

**Como** usuario quiero diferentes funciones de easing para animaciones más naturales.

**Criterios de Aceptación:**
- [ ] Linear, EaseIn, EaseOut, EaseInOut
- [ ] CubicBezier parametrizable
- [ ] Elastic, Bounce, Spring

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "CSS easing functions mathematical definition cubic-bezier"
- [ ] **Query 2:** "spring physics animation stiffness damping Rust"
- [ ] **Query 3:** "easing function performance Rust vs JavaScript"
- [ ] **Output:** Documento de easing

**Tasks:**
- [ ] Implementar EasingFunction
- [ ] Implementar funciones básicas
- [ ] Implementar Spring
- [ ] Const definitions

---

# ÉPICA 009: Recursos Externos

**Objetivo:** Implementar soporte para imágenes, videos y HTML embebido.

**Duración:** 2 semanas

---

### US-080: Sistema de Recursos

**Como** usuario quiero insertar imágenes en el canvas para enriquecer mis diagramas.

**Criterios de Aceptación:**
- [ ] ExternalResource enum
- [ ] ImageSource (RemoteUrl, Embedded, LocalPath)
- [ ] ImageFitMode (Fill, Contain, Cover)
- [ ] ResourceManager con caché LRU

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust LRU cache implementation performance 2025"
- [ ] **Query 2:** "Canvas image drawImage performance Rust wasm"
- [ ] **Query 3:** "tldraw image resource management source code"
- [ ] **Output:** Documento de recursos

**Tasks:**
- [ ] Definir ExternalResource
- [ ] Implementar ResourceManager
- [ ] LRU cache
- [ ] Carga asíncrona

---

### US-081: HTML Overlays

**Como** usuario quiero incrustar contenido HTML sobre el canvas para mostrar widgets interactivos.

**Criterios de Aceptación:**
- [ ] HtmlOverlayResource
- [ ] HtmlInteractionMode
- [ ] Iframe support
- [ ] Embed providers

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "tldraw embed HTML iframe implementation source code"
- [ ] **Query 2:** "Rust wasm HTML DOM overlay z-index Canvas"
- [ ] **Query 3:** "YouTube Vimeo embed URL patterns parsing Rust"
- [ ] **Output:** Documento de HTML overlays

**Tasks:**
- [ ] Implementar HtmlOverlayResource
- [ ] Implementar DOM overlay
- [ ] Implementar iframe support
- [ ] Embed providers

---

# ÉPICA 010: Zoom de Detalle Incremental

**Objetivo:** Implementar el sistema de niveles de detalle (modelo C4 interactivo).

**Duración:** 3 semanas

---

### US-090: Sistema de Niveles

**Como** arquitecto quiero ver más detalles según el nivel de zoom para explorar mi arquitectura progresivamente.

**Criterios de Aceptación:**
- [ ] DetailLevel enum (Macro, Connections, Internals, Detail, Code)
- [ ] DetailVisibility struct
- [ ] Transición suave entre niveles
- [ ] Slider UI

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "C4 model visualization tool architecture diagram"
- [ ] **Query 2:** "incremental detail zoom UI design pattern"
- [ ] **Query 3:** "tldraw detail level rendering implementation"
- [ ] **Output:** Documento de niveles

**Tasks:**
- [ ] Definir DetailLevel
- [ ] Implementar DetailVisibility
- [ ] Implementar transiciones
- [ ] UI slider

---

### US-091: Jerarquía y Detalle

**Como** arquitecto quiero expandir contenedores para ver componentes internos para hacer drill-down.

**Criterios de Aceptación:**
- [ ] HierarchicalChildren component
- [ ] Parent/Children links
- [ ] Expansion animation

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "hierarchical data visualization expanding collapsing nodes"
- [ ] **Query 2:** "tree view animation React vs Rust Canvas performance"
- [ ] **Output:** Documento de jerarquía

**Tasks:**
- [ ] Implementar HierarchicalChildren
- [ ] Implementar Parent/Children
- [ ] Sistema de expansión
- [ ] Animación

---

# ÉPICA 011: APIs para Desarrolladores

**Objetivo:** Crear APIs accesibles para que otros desarrolladores usen el motor.

**Duración:** 2 semanas

---

### US-100: API de Alto Nivel

**Como** desarrollador quiero una API simple y ergonómica para integrar ArchFlow en mi aplicación.

**Criterios de Aceptación:**
- [ ] Engine::new() factory
- [ ] Document creation
- [ ] fluent API para shapes
- [ ] animate() method

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "Rust API design patterns fluent builder method chaining"
- [ ] **Query 2:** "game engine API design Rust bevy ecs pattern"
- [ ] **Query 3:** "Rust crate public API ergonomics best practices 2025"
- [ ] **Output:** Documento de API

**Tasks:**
- [ ] Implementar Engine
- [ ] Implementar Document
- [ ] Fluent API
- [ ] Documentación

---

### US-101: API de Formas Personalizadas

**Como** desarrollador quiero definir mis propias formas para extender el motor.

**Criterios de Aceptación:**
- [ ] ShapeDef trait
- [ ] CustomShape example
- [ ] Registration system

**INVESTIGACIÓN (Obligatorio - Perplexity):**
- [ ] **Query 1:** "tldraw custom shape implementation guide plugin system"
- [ ] **Query 2:** "Rust trait object vs enum dispatch plugin architecture"
- [ ] **Output:** Documento de extensibilidad

**Tasks:**
- [ ] Definir ShapeDef trait
- [ ] Implementar CustomShape
- [ ] Registration system
- [ ] Documentación

---

## Apéndice: Dependencias

```toml
[dependencies]
bevy_ecs = "0.18"
glam = "0.31"
euclid = "0.22.13"
kurbo = "0.13"
rstar = "0.12"
serde = { version = "1.0", features = ["derive"] }
bincode = "3.0"
zstd = "0.13"
uuid = "1.11"
thiserror = "2.0"
anyhow = "1.0"
wasm-bindgen = "0.2.108"
web-sys = { version = "0.3", features = [...] }

[dev-dependencies]
test-case = "3.3"
proptest = "1.5"
criterion = "0.5"
```

---

**Documento preparado por:** ArchFlow Research  
**Última actualización:** 2026-01-23  
**Versión:** 1.7.0
