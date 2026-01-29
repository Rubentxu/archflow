# ArchFlow Engine 2D: Análisis Técnico Completo

**Fecha:** 2026-01-23  
**Versión:** 1.6  
**Estado:** Correcciones Finales Aplicadas  
**Objetivo:** Definir la arquitectura definitiva para un motor 2D en Rust reutilizable, potente y con API accesible para developers.

---

## Notas de la Versión 1.6 (Correcciones Finales)

Esta versión incorpora las mejoras de `ENGINE-2D-FINAL-REVIEW.md`:

### Cambios Realizados:

1. ✅ **Sistema de Transformadas Jerárquicas:**
   - Añadido `GlobalTransform` (coordenadas de mundo calculadas)
   - Añadido `Parent` y `Children` para jerarquía C4
   - Sistema de propagación: `Global(Child) = Global(Parent) * Local(Child)`

2. ✅ **Ciclo de Acción Store vs ECS:**
   - Definido modelo de 3 fases: Input → Commit → Store
   - ECS mutable en caliente para feedback instantáneo (60fps)
   - Store inmutable hasta commit para persistencia

3. ✅ **Routing Ortogonal Simplificado:**
   - Eliminado A* sobre grilla (demasiado costoso)
   - Añadido "Doorway Routing" con heurística L-shape/U-shape
   - Mucho más rápido para tiempo real

4. ✅ **Sistema de Ports Mejorado:**
   - Añadido `PortBinding` con `Fixed`, `AutoClosest`, `AutoCompass`
   - Cálculo dinámico de puntos de anclaje

### Pivote Final:

> **El ECS es el estado vivo (mutable). El Store es el estado persistido (inmutable hasta commit).**

Este documento presenta un estudio exhaustivo de las tecnologías, patrones arquitectónicos y mejores prácticas necesarias para construir un motor 2D de alto rendimiento en Rust. La investigación se basa en:

- Análisis de código fuente de **tldraw**, **Excalidraw** y **ReactFlow** (via Repomix)
- Investigación de crates de Rust para gráficos 2D (lyon, kurbo, wgpu, bevy_ecs)
- Búsqueda de algoritmos de rendering hand-drawn (RoughJS)
- Evaluación de arquitecturas ECS y sistemas de animación

**Conclusión principal:** Un motor 2D efectivo debe combinar **ECS para el estado**, **lyon/kurbo para geometría**, **Canvas 2D/WebGPU para rendering**, y un **sistema de primitivas extensible** inspirado en tldraw.

---

## 2. Análisis de Sistemas Existentes

### 2.1 Tldraw: Arquitectura de Shapes Extensibles

`tldraw` implementa un patrón de **ShapeUtil** que es el gold standard para extensibilidad de formas:

```typescript
// De tldraw.dev/docs/shapes
abstract class ShapeUtil<Shape extends ShapeProp> {
  // Geometría y bounds
  getDefaultProps(): Shape['props']
  getGeometry(shape: Shape): GeometryResult
  
  // Rendering
  render(shape: Shape): ReactNode
  renderAsync(shape: Shape): Promise<ReactNode>
  
  // Hit testing
  hitTest(shape: Shape, point: Vec): boolean
  
  // Interacciones
  onHandleChange(shape: Shape, handle: Handle): Shape
  onTranslate(shape: Shape, delta: Vec): Shape
}
```

**Patrón clave:** Cada tipo de forma tiene su propio `ShapeUtil` registrado en un mapa. Esto permite:
- Extensibilidad total: anyone puede añadir formas nuevas
- Separación clara: lógica de render vs lógica de negocio
- Lazy loading: las formas pueden cargarse asíncronamente

### 2.2 Excalidraw: Hand-Drawn Rendering con RoughJS

Excalidraw usa **RoughJS** para generar graphics con apariencia "sketchy":

```typescript
//粗糙度控制 (roughness): 0 = preciso, 2+ = muy sketchy
//弓形控制 (bowing): curvatura de líneas rectas
const rc = rough.canvas(canvas);
rc.rectangle(x, y, width, height, {
  roughness: 1.5,
  bowing: 1.0,
  fill: '#fff',
  fillStyle: 'hachure', // 'hachure' | 'solid' | 'zigzag'
  stroke: '#000',
  strokeWidth: 1,
});
```

**Algoritmos de RoughJS** (shihn.ca/posts/2020/roughjs-algorithms):

1. **Doble trazo:** Cada línea se dibuja 2 veces con ligero offset para efecto "lápiz"
2. **Punto medio desplazado:** Para rectángulos, los corners se desplazan aleatoriamente
3. **Curvas de Bézier con ruido:** Los puntos de control se perturban estadísticamente
4. **Hachure fill:** Líneas paralelas espaciadas para fills (en lugar de sólido)

### 2.3 ReactFlow: Sistema de Nodos y Aristas

ReactFlow estructura el grafo con separación estricta entre **Nodes** y **Edges**:

```typescript
// Tipos base de ReactFlow
interface NodeBase {
  id: string;
  type?: string;
  position: XYPosition;
  data?: Record<string, unknown>;
  selected?: boolean;
  dragging?: boolean;
}

interface EdgeBase {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string;
  targetHandle?: string;
  animated?: boolean;
  markerEnd?: EdgeMarker;
}
```

**Tipos de aristas soportados:**
- `bezier`: Curvas de Bézier con control automático
- `straight`: Líneas rectas
- `step`: Escalones ortogonales
- `smoothstep`: Escalones con esquinas redondeadas
- `simplebezier`: Bézier simplificada para conexiones cortas

---

## 3. Arquitectura Propuesta para ArchFlow Engine

### 3.1 Diagrama de Arquitectura General

```
┌─────────────────────────────────────────────────────────────────┐
│                    ArchFlow Engine 2D                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   Renderer   │  │    Store     │  │    Spatial Index     │  │
│  │  (Canvas2D)  │  │   (Delta)    │  │     (RTree)         │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                      │              │
│         └────────────┬────┴──────────────────────┘              │
│                      ▼                                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  ECS Core (bevy_ecs)                     │  │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────────────────────┐ │  │
│  │  │Entities │  │Components│  │    Schedules & Systems  │ │  │
│  │  └─────────┘  └──────────┘  └─────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                      │                                          │
│         ┌────────────┼────────────┐                            │
│         ▼            ▼            ▼                            │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐                     │
│  │ ShapeDefs │ │ Animations│ │  Events   │                     │
│  └───────────┘ └───────────┘ └───────────┘                     │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Estructura de Crates Propuesta (Corregida)

**Corrección de la crítica:** Eliminar ambigüedad sobre ECS y separar claramente geometría de rendering.

```
crates/
├── archflow-core/            # Meta-crate (re-exports) y Domain Types
├── archflow-ecs/             # Wrapper thin sobre bevy_ecs + Componentes/Sistemas del dominio
├── archflow-geometry/        # Wrappers sobre euclid + kurbo (puro cálculo geométrico)
├── archflow-renderer/        # Traits de rendering y backend Canvas 2D
│   ├── canvas/               # Implementación WebSys Canvas
│   └── rough/                # Rough rendering simplificado (opcional)
├── archflow-workspace/       # Gestión del documento, undo/redo, selección
└── archflow-wasm/            # Bindings y lógica de navegador
```

**No crear:**
- ❌ `archflow-ecs-propio` - usar `bevy_ecs` directamente
- ❌ `archflow-lyon` - el navegador ya tesela paths para Canvas 2D

**Sí crear:**
- ✅ `archflow-geometry` - kurbo para cálculos, no para rendering

---

## 4. Sistema de Primitivas Detallado

### 4.1 Enum Principal de Primitivas

```rust
// archflow-primitives/src/lib.rs

/// Primitiva atómica de renderizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Primitive {
    // Geometría básica
    Rect(RectPrimitive),
    Ellipse(EllipsePrimitive),
    Line(LinePrimitive),
    Polyline(PolylinePrimitive),
    Path(PathPrimitive),
    Text(TextPrimitive),
    Image(ImagePrimitive),
    
    // Diagramación
    Arrow(ArrowPrimitive),
    Connector(ConnectorPrimitive),
    Cloud(CloudPrimitive),
    Cylinder(CylinderPrimitive),
    Cube(CubePrimitive),
    Diamond(DiamondPrimitive),
    
    // Composición
    Group(GroupPrimitive),
    Clip(ClipPrimitive),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectPrimitive {
    pub bounds: Rect,
    pub corner_radii: [f32; 4], // top-left, top-right, bottom-right, bottom-left
    pub style: StyleId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPrimitive {
    pub commands: Vec<PathCommand>,
    pub fill: Option<FillStyle>,
    pub stroke: Option<StrokeStyle>,
    pub winding_rule: WindingRule, // NonZero | EvenOdd
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo { control: Vec2, end: Vec2 },
    CubicTo { control1: Vec2, control2: Vec2, end: Vec2 },
    Arc { 
        center: Vec2, 
        radii: Vec2, 
        start_angle: Rad, 
        end_angle: Rad,
        counter_clockwise: bool 
    },
    Close,
}
```

### 4.2 Sistema de Estilos

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    // Stroke
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub stroke_style: StrokeStyle, // Solid, Dashed, Dotted, DashDot
    pub stroke_cap: LineCap,       // Butt, Round, Square
    pub stroke_join: LineJoin,     // Miter, Round, Bevel
    
    // Fill
    pub fill_color: Option<Color>,
    pub fill_opacity: f32,
    pub fill_pattern: Option<FillPattern>,
    
    // Efectos
    pub shadow: Option<Shadow>,
    pub blur: Option<f32>,
    pub blend_mode: BlendMode,
    pub opacity: f32,
    
    // Hand-drawn mode
    pub roughness: f32,      // 0.0 = exacto, 2.0 = muy sketchy
    pub bowing: f32,         // curvatura de líneas
    pub seed: u64,           // para reproducibilidad
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowPrimitive {
    pub start: Vec2,
    pub end: Vec2,
    
    // Puntos de control para el routing (calculados por el routing system)
    pub waypoints: Vec<Vec2>, 
    
    // Configuración
    pub routing_mode: RoutingMode, // Straight, Orthogonal, Curved
    pub start_marker: MarkerType,  // Arrow, Circle, None, Diamond
    pub end_marker: MarkerType,
    
    pub style: StyleId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingMode {
    Straight,
    Orthogonal,      // Manhattan routing (codos 90 grados)
    Curved(f32),     // Bézier con curvatura
}
```

### 4.2 Puertos y Conexiones (Actualizado v1.6)

Para diagramas de arquitectura, las conexiones no van al centro, sino a "Puertos".

**NUEVO (v1.6): PortBinding para cálculo dinámico de puntos de anclaje**

```rust
#[derive(Component, Debug, Clone)]
pub struct Ports {
    pub items: Vec<Port>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: String,         // "top", "bottom", "input-1"
    pub offset: Vec2,       // Posición relativa al centro
    pub direction: Vec2,    // Vector normal (hacia donde sale la flecha)
    pub type_: PortType,    // Logical, Physical
    pub binding: PortBinding,  // NUEVO v1.6: Cómo calcular posición
}

/// Cómo se calcula la posición del puerto dinámicamente
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortBinding {
    /// Punto fijo relativo al centro (ej: (0.5, 0.0) es derecha centro)
    Fixed(Vec2),
    
    /// Punto dinámico que se mueve al borde más cercano a la otra entidad
    AutoClosest,
    
    /// Punto dinámico alineado a la brújula (North, South, East, West)
    AutoCompass,
}

/// Calcular posición real de un puerto considerando su binding
pub fn get_port_position(
    port: &Port,
    entity_bounds: Rect,
    target_position: Vec2,
) -> Vec2 {
    match &port.binding {
        PortBinding::Fixed(offset) => {
            entity_bounds.min + *offset
        }
        PortBinding::AutoClosest => {
            // Encontrar el punto del bounding box más cercano al target
            find_closest_point_on_rect(entity_bounds, target_position)
        }
        PortBinding::AutoCompass => {
            // Encontrar punto alineado a los 4 cardinales
            find_compass_point(entity_bounds, target_position)
        }
    }
}

/// Encontrar punto más cercano del rectángulo a un punto exterior
fn find_closest_point_on_rect(rect: Rect, point: Vec2) -> Vec2 {
    Vec2::new(
        point.x.clamp(rect.min.x, rect.max.x),
        point.y.clamp(rect.min.y, rect.max.y)
    )
}

/// Encontrar punto en el borde más cercano alineado a cardinales
fn find_compass_point(rect: Rect, target: Vec2) -> Vec2 {
    let center = rect.center();
    let dx = target.x - center.x;
    let dy = target.y - center.y;
    
    // Determinar cardinalidad basada en la dirección al target
    if dx.abs() > dy.abs() {
        // Más horizontal → East o West
        if dx > 0.0 {
            Vec2::new(rect.max.x, center.y)  // East
        } else {
            Vec2::new(rect.min.x, center.y)  // West
        }
    } else {
        // Más vertical → North o South
        if dy > 0.0 {
            Vec2::new(center.x, rect.max.y)  // South (Y+ en Canvas)
        } else {
            Vec2::new(center.x, rect.min.y)  // North (Y- en Canvas)
        }
    }
}
```

---

## 5. Sistema de Rendering

### 5.1 Arquitectura de Rendering: Separación Geometría vs. Drawing

**Corrección de la crítica:** NO usar `lyon` para Canvas 2D (el navegador ya tesela). Usar:
- **`kurbo`** para cálculos de geometría (intersecciones, Bézier, bounding boxes)
- **`CanvasRenderingContext2d`** para el dibujado real

```rust
// archflow-geometry/src/lib.rs
// Solo cálculos - NADA de rendering aquí

use kurbo::{BezPath, Rect as KurboRect, Point, Shape as KurboShape};

/// Cálculos geométricos puros (sin efectos visuales)
pub struct GeometryEngine;

impl GeometryEngine {
    /// Calcular bounding box de una curva Bézier
    pub fn bezier_bounds(commands: &[PathCommand]) -> Rect {
        let path = Self::commands_to_kurbo(commands);
        path.bounding_box()
    }
    
    /// Encontrar intersección entre dos paths
    pub fn intersect_paths(path1: &[PathCommand], path2: &[PathCommand]) -> Vec<Vec2> {
        let kpath1 = Self::commands_to_kurbo(path1);
        let kpath2 = Self::commands_to_kurbo(path2);
        // Usar kurbo para intersecciones
        todo!()
    }
    
    fn commands_to_kurbo(commands: &[PathCommand]) -> BezPath {
        let mut path = BezPath::new();
        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(p) => path.move_to(Point::new(p.x, p.y)),
                PathCommand::LineTo(p) => path.line_to(Point::new(p.x, p.y)),
                // ... convertir a kurbo
            }
        }
        path
    }
}

// archflow-renderer/src/lib.rs
// Solo drawing - NADA de cálculos aquí

/// Renderer de Canvas 2D (navegador o headless)
pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn save(&mut self);
    fn restore(&mut self);
    
    // Transformaciones
    fn translate(&mut self, x: f32, y: f32);
    fn rotate(&mut self, angle: Rad);
    fn scale(&mut self, sx: f32, sy: f32);
    
    // Rendering de primitivas (usa kurbo para bounds, Canvas para draw)
    fn draw_rect(&mut self, rect: &RectPrimitive);
    fn draw_ellipse(&mut self, center: Vec2, radii: Vec2);
    fn draw_path(&mut self, path: &PathPrimitive);
    fn draw_text(&mut self, text: &TextPrimitive);
    fn draw_image(&mut self, image: &ImagePrimitive);
    fn draw_arrow(&mut self, arrow: &ArrowPrimitive);
}
```

### 5.2 Rough Rendering Simplificado

**Corrección de la crítica:** No implementar toda la lógica de RoughJS. Opciones:
1. **SVG Filters** (displacement map) - más simple, menos control
2. **Ruido en vértices** - línea con jitter, sin hachure fill complejo

```rust
/// Simplificación: solo doble trazo con jitter en vértices
/// NO hachure fill completo (demasiado complejo para MVP)
pub struct RoughRenderer;

impl RoughRenderer {
    pub fn draw_rough_rect(
        &self,
        ctx: &mut dyn Renderer,
        rect: &RectPrimitive,
        rough: &RoughParams,
    ) {
        // Dos pasadas con ligero offset
        for i in 0..2 {
            let offset = if i == 1 {
                Vec2::new(rough.seed_offsets[0], rough.seed_offsets[1])
            } else {
                Vec2::ZERO
            };
            
            // Usar kurbo para generar path con jitter
            let jittered_path = self.jitter_rect(rect, rough.seed + i as u64);
            ctx.draw_path(&jittered_path);
        }
    }
    
    fn jitter_rect(&self, rect: &RectPrimitive, seed: u64) -> PathPrimitive {
        let mut rng = SeededRng::new(seed);
        let jitter_amount = rough.roughness;
        
        // Los 4 corners con jitter
        let corners = [
            rect.bounds.min + Vec2::new(rng.next_f32() * jitter_amount, rng.next_f32() * jitter_amount),
            rect.bounds.max + Vec2::new(rng.next_f32() * jitter_amount, -rng.next_f32() * jitter_amount),
            rect.bounds.max - Vec2::new(rng.next_f32() * jitter_amount, rng.next_f32() * jitter_amount),
            rect.bounds.min - Vec2::new(rng.next_f32() * jitter_amount, -rng.next_f32() * jitter_amount),
        ];
        
        PathPrimitive {
            commands: vec![
                PathCommand::MoveTo(corners[0]),
                PathCommand::LineTo(corners[1]),
                PathCommand::LineTo(corners[2]),
                PathCommand::LineTo(corners[3]),
                PathCommand::Close,
            ],
            // ... fill/stroke
        }
    }
}
```

### 5.3 Optimización de Rendering: Culling y Layers

```rust
pub struct RenderContext {
    pub viewport: Rect,
    pub zoom: f32,
    pub visible_layers: Vec<LayerId>,
}

impl CanvasRenderer {
    pub fn render_frame(
        &mut self,
        store: &Store,
        context: RenderContext,
    ) -> Result<(), RenderError> {
        // 1. Culling espacial con RTree
        let visible = self.spatial_index.query(&context.viewport);
        
        // 2. Layer sorting
        let mut sorted = visible
            .into_iter()
            .sorted_by_key(|e| store.get_layer_z_index(e.layer_id()));
        
        // 3. Batch rendering por tipo
        for group in sorted.group_by(|e| e.primitive_type()) {
            self.begin_batch();
            for entity in group {
                self.draw_primitive(&entity.primitive);
            }
            self.end_batch();
        }
        
        // 4. Overlay para selección/hover
        self.draw_selection_overlay(visible);
        
        Ok(())
    }
}
```

### 5.3 Comparativa: Canvas 2D vs WebGPU

| Aspecto | Canvas 2D | WebGPU |
|---------|-----------|--------|
| **Curva aprendizaje** | Baja | Alta |
| **Compatibilidad** | Universal | Moderada (Chrome/FF modernos) |
| **Rendimiento** | Bueno (<10k objetos) | Excelente (50k+ objetos) |
| **Shaders** | No | Sí |
| **Para MVP** | ✅ Recomendado | ⚠️ Fase 2+ |

**Recomendación:** Comenzar con Canvas 2D optimizado, migrar a WebGPU solo si profiling lo justifica.

---

## 6. Sistema de Animaciones

### 6.1 Arquitectura de Animaciones

```rust
// archflow-animation/src/lib.rs

pub trait AnimationCurve {
    fn sample(&self, t: f64) -> f64;  // t in [0, 1]
}

pub struct KeyframeAnimation<T: Interpolatable> {
    pub keyframes: Vec<Keyframe<T>>,
    pub curve: Box<dyn AnimationCurve>,
    pub repeat: RepeatMode,
}

pub enum RepeatMode {
    Once,
    Loop,
    PingPong,
    Clamp,
}

pub struct Animation {
    pub id: AnimationId,
    pub target: EntityId,
    pub property: AnimatedProperty,
    pub keyframes: Vec<Keyframe<f64>>,
    pub easing: EasingFunction,
    pub duration: Duration,
    pub state: AnimationState,
}

pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f64, f64, f64, f64),  // x1, y1, x2, y2
    Elastic,
    Bounce,
    Spring { stiffness: f32, damping: f32 },
}

// Easing functions predefinidas
impl EasingFunction {
    pub const EASE_OUT_BACK: Self = Self::CubicBezier(0.34, 1.56, 0.64, 1.0);
    pub const EASE_IN_OUT_QUART: Self = Self::CubicBezier(0.76, 0.0, 0.24, 1.0);
}
```

### 6.2 Sistema de Propiedades Animables

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimatedProperty {
    Position,
    Rotation,
    Scale,
    Opacity,
    StrokeWidth,
    FillColor,
    PathOffset,      // Para animaciones sobre paths
    Custom(&'static str),
}

pub trait Interpolatable {
    fn lerp(&self, other: &Self, t: f64) -> Self;
    fn ease(&self, other: &Self, curve: &dyn AnimationCurve) -> Self;
}

// Implementaciones
impl Interpolatable for Vec2 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        self * (1.0 - t as f32) + other * t as f32
    }
}

impl Interpolatable for Color {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            r: self.r.lerp(&other.r, t),
            g: self.g.lerp(&other.g, t),
            b: self.b.lerp(&other.b, t),
            a: self.a.lerp(&other.a, t),
        }
    }
}
```

### 6.3 Partículas para Flows Animados

```rust
pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub config: ParticleConfig,
}

pub struct Particle {
    pub position: Vec2,
    pub progress: f32,  // 0.0 a 1.0 sobre el path
    pub lifetime: f32,
    pub size: f32,
    pub color: Color,
}

impl ParticleSystem {
    pub fn update(&mut self, delta_time: Duration) {
        for particle in &mut self.particles {
            particle.progress += delta_time.as_secs_f32() * self.config.speed;
            particle.lifetime -= delta_time.as_secs_f32();
            
            // Calcular posición en el path
            particle.position = self.config.path.sample(particle.progress);
        }
        
        // Eliminar partículas expiradas
        self.particles.retain(|p| p.lifetime > 0.0);
        
        // Emitir nuevas partículas
        self.emit_particles();
    }
}
```

---

## 7. Connection Routing & Smart Flows (Prioridad Alta)

El verdadero diferenciador para diagramas de arquitectura es el **Smart Routing**.

### 7.1 Algoritmos de Routing (Simplificado v1.6)

**NUEVO:** A* sobre grilla es costoso para 60fps. Usamos heurística más simple:

```rust
/// Routing ortogonal con heurística de "puertas" (Doorway Routing)
/// Mucho más rápido que A* para MVP
pub enum RoutingMode {
    Straight,
    Orthogonal(OrthogonalConfig),
    Curved(f32),  // Bézier con curvatura
}

pub struct OrthogonalConfig {
    /// Distancia mínima del "puerto" al "codo"
    pub stub_length: f32,
    /// Cómo seleccionar la dirección del path
    pub direction_rule: DirectionRule,
}

pub enum DirectionRule {
    /// Solo horizontal o vertical (L-shapes)
    LShape,
    /// Intenta minimizar el número de bends
    MinBends,
    /// Alineado a los 4 puntos cardinales del source
    Compass,
}

/// Doorway Routing: encuentra "puertas" en el borde de cada shape
/// y conecta las puertas más cercanas con path ortogonal
pub fn calculate_orthogonal_route(
    start: ConnectionPoint,
    end: ConnectionPoint,
    obstacles: &[Rect],
    config: &OrthogonalConfig,
) -> Vec<Vec2> {
    // 1. Encontrar las "puertas" (puntos en el borde del shape)
    let start_doors = find_doors(start.entity, start.side);
    let end_doors = find_doors(end.entity, end.side);
    
    // 2. Encontrar el path más corto (no A* completo, sino greedy con backtracking)
    let (path, _) = find_orthogonal_path(
        start_doors,
        end_doors,
        obstacles,
        config.stub_length,
    );
    
    path
}

/// Encontrar "puertas" en el borde de un shape
fn find_doors(entity: EntityId, preferred_side: Option<Direction>) -> Vec<Door> {
    let bounds = get_bounds(entity);
    let ports = get_ports(entity);
    
    // Las puertas pueden ser:
    // 1. Ports definidos explícitamente
    // 2. Puntos cardinales del bounding box
    // 3. Centro de cada lado
    
    let mut doors = Vec::new();
    
    for port in ports {
        doors.push(Door {
            position: bounds.min + port.offset,
            direction: port.direction,
            entity,
        });
    }
    
    // Si no hay ports, usar lados del bounding box
    if doors.is_empty() {
        doors.extend([
            Door { position: bounds.center(), direction: Direction::North, entity },
            Door { position: bounds.center(), direction: Direction::South, entity },
            Door { position: bounds.center(), direction: Direction::East, entity },
            Door { position: bounds.center(), direction: Direction::West, entity },
        ]);
    }
    
    doors
}

/// Greedy path finding con backtracking mínimo
fn find_orthogonal_path(
    start_doors: Vec<Door>,
    end_doors: Vec<Door>,
    obstacles: &[Rect],
    stub_length: f32,
) -> (Vec<Vec2>, usize) {
    let mut best_path = Vec::new();
    let mut min_cost = f32::MAX;
    
    for start_door in &start_doors {
        for end_door in &end_doors {
            // Intentar path L-shape primero (muy común, rápido de calcular)
            if let Some(l_path) = try_l_shape(start_door, end_door, obstacles, stub_length) {
                let cost = path_cost(&l_path, obstacles);
                if cost < min_cost {
                    min_cost = cost;
                    best_path = l_path;
                }
            }
        }
    }
    
    // Si L-shape no funciona, intentar U-shape o Z-shape
    if best_path.is_empty() {
        best_path = try_z_shape(start_doors.first().unwrap(), end_doors.first().unwrap(), obstacles);
    }
    
    (best_path, 0)
}
```

### 7.2 Integración con Kurbo (Bézier sin cambios)

Usaremos `kurbo` para curvas Bézier:

```rust
// archflow-geometry/src/routing.rs

pub fn calculate_bezier_route(start: Vec2, end: Vec2, control_offset: f32) -> BezPath {
    let mut path = BezPath::new();
    path.move_to(start.to_point());
    
    // Calcular puntos de control para suavidad
    let c1 = start + Vec2::new(control_offset, 0.0);
    let c2 = end - Vec2::new(control_offset, 0.0);
    
    path.curve_to(c1.to_point(), c2.to_point(), end.to_point());
    path
}
```

### 7.3 Animación de Flujo

En lugar de partículas complejas, usaremos inicialmente `stroke-dashoffset` animado en el shader/renderer para simular flujo de datos a través de los cables.

---

## 8. Spatial Indexing

### 8.1 R-Tree con rstar

**Advertencia de la crítica:** El R-Tree NO es la fuente de verdad de posición. Debe sincronizarse con el ECS. Si un sistema mueve una entidad en el ECS, el R-Tree queda obsoleto sin un sistema de sync.

```rust
// archflow-spatial/src/rtree.rs

use rstar::{RTree, RTreeObject, PointDistance};

#[derive(Debug, Clone)]
pub struct SpatialEntity {
    pub id: EntityId,
    pub bounds: AABB,
    pub layer: LayerId,
}

impl RTreeObject for SpatialEntity {
    type Envelope = AABB<2>;
    
    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

pub struct SpatialIndex {
    tree: RTree<SpatialEntity>,
    /// Dirty entities que necesitan sync
    dirty: HashSet<EntityId>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
            dirty: HashSet::new(),
        }
    }
    
    pub fn insert(&mut self, entity: SpatialEntity) {
        self.tree.insert(entity);
    }
    
    pub fn remove(&mut self, id: EntityId) {
        if let Some(entity) = self.tree.iter()
            .find(|e| e.id == id)
            .cloned() 
        {
            self.tree.remove(&entity);
        }
    }
    
    /// Marcar entidad como modificada (para sync posterior)
    pub fn mark_dirty(&mut self, id: EntityId) {
        self.dirty.insert(id);
    }
    
    /// Sync de entidades modificadas desde el ECS
    pub fn sync_dirty(&mut self, world: &World) {
        let mut to_remove = Vec::new();
        let mut to_insert = Vec::new();
        
        for id in self.dirty.drain() {
            if let Some((transform, bounds)) = world.get::<(Transform, Bounds)>(id) {
                to_remove.push(id);
                to_insert(SpatialEntity {
                    id,
                    bounds: AABB::from_point_and_size(
                        [bounds.world.x, bounds.world.y],
                        [bounds.world.width, bounds.world.height]
                    ),
                    layer: world.get_layer_id(id).unwrap_or_default(),
                });
            }
        }
        
        for id in to_remove {
            self.remove(id);
        }
        for entity in to_insert {
            self.insert(entity);
        }
    }
    
    /// Query por viewport (culling)
    pub fn query_viewport(&self, viewport: Rect) -> Vec<EntityId> {
        let envelope = AABB::from_point_and_size(
            [viewport.x, viewport.y],
            [viewport.width, viewport.height]
        );
        
        self.tree.locate_in_envelope_intersecting(&envelope)
            .map(|e| e.id)
            .collect()
    }
    
    /// Query por punto (hit testing)
    pub fn query_point(&self, point: Vec2) -> Vec<EntityId> {
        let envelope = AABB::from_point_and_size(
            [point.x, point.y],
            [0.0, 0.0]
        );
        
        self.tree.locate_in_envelope(&envelope)
            .filter(|e| e.bounds.contains(point))
            .map(|e| e.id)
            .collect()
    }
    
    /// Query por área (selección múltiples)
    pub fn query_area(&self, area: Rect) -> Vec<EntityId> {
        let envelope = AABB::from_point_and_size(
            [area.x, area.y],
            [area.width, area.height]
        );
        
        self.tree.locate_in_envelope_intersecting(&envelope)
            .map(|e| e.id)
            .collect()
    }
    
    /// KNN nearest neighbor para snapping
    pub fn nearest_neighbors(&self, point: Vec2, k: usize) -> Vec<(EntityId, f32)> {
        self.tree.nearest_neighbor_iter(&point)
            .take(k)
            .map(|e| (e.id, e.bounds.distance_to_point(point)))
            .collect()
    }
}
```

### 8.2 Sistema de Sincronización ECS → R-Tree

```rust
/// Sistema que sincroniza cambios del ECS al R-Tree
/// Se ejecuta DÉSPUÉS de todos los sistemas que modifican Transform
#[derive(SystemSet)]
struct SpatialSyncSet;

fn spatial_index_sync_system(
    mut spatial: ResMut<SpatialIndex>,
    changed_transforms: Query<(EntityId, &Bounds), Changed<Transform>>,
) {
    // Marcar todas las entidades con Transform modificado
    for (id, _) in &changed_transforms {
        spatial.mark_dirty(id);
    }
    
    // Sync real
    spatial.sync_dirty(&world);
}
```

### 8.3 Optimización de Updates Espaciales

```rust
impl SpatialIndex {
    /// Rebuild eficiente: solo reindexar lo que cambió
    pub fn update(&mut self, changes: &[EntityChange]) {
        for change in changes {
            match change {
                EntityChange::Added(e) => self.insert(e.clone()),
                EntityChange::Modified(id, new_bounds) => {
                    self.remove(*id);
                    if let Some(entity) = self.tree.iter()
                        .find(|e| e.id == *id)
                        .map(|e| {
                            let mut e = e.clone();
                            e.bounds = *new_bounds;
                            e
                        })
                    {
                        self.insert(entity);
                    }
                }
                EntityChange::Removed(id) => self.remove(*id),
            }
        }
    }
    
    /// Bulk insert para carga inicial
    pub fn bulk_insert(&mut self, entities: Vec<SpatialEntity>) {
        self.tree = RTree::bulk_load(entities);
    }
}
```

---

## 9. Sistema ECS para el Motor

### 9.1 Integración con bevy_ecs

**Corrección crítica:** NO crear un ECS propio. Usar `bevy_ecs` directamente. Es el estándar de facto, tiene ergonomía excelente y query filters potentes. Reinventar un ECS consumiría meses de debugging innecesario.

```rust
// archflow-ecs/src/lib.rs

use bevy_ecs::prelude::*;

// Re-export de bevy_ecs para uso en el engine
pub use bevy_ecs::prelude::{
    Component, Entity, Query, System, World,
    IntoSystem, Commands, Resource,
    Added, Changed, Or, With, Without,
};

// Componentes del dominio ArchFlow
pub use crate::components::{
    Transform, Bounds, Visual, Selectable,
    Draggable, Resizable, Animated, Connector,
};

// Sistemas del engine
pub use crate::systems::{
    transform_propagation_system,
    spatial_index_sync_system,
    animation_system,
    render_system,
};
```

### 9.2 Componentes del Motor

```rust
// Componentes base - Transform Local (actualizado v1.6)
#[derive(Component)]
pub struct Transform {
    pub translation: Vec2,  // Position local (relativa al padre)
    pub rotation: Rad,
    pub scale: Vec2,
}

#[derive(Component)]
pub struct Bounds {
    pub local: Rect,
    pub world: Rect,
}

#[derive(Component)]
pub struct Visual {
    pub primitive: PrimitiveType,
    pub style_id: StyleId,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

**NUEVO (v1.6): Sistema de Transformadas Jerárquicas**

Para soportar el modelo C4 y el Zoom de Detalle, necesitamos:
- `Transform`: coordenadas locales (relativas al padre)
- `GlobalTransform`: coordenadas de mundo (calculadas frame a frame)
- `Parent`/`Children`: vínculos jerárquicos

```rust
// Transform Local (relativa al padre)
#[derive(Component)]
pub struct Transform {
    pub translation: Vec2,  // Position local
    pub rotation: Rad,
    pub scale: Vec2,
}

/// Transformada Global (calculada frame a frame)
/// Se recalcula automáticamente desde Transform + Parent
#[derive(Component)]
pub struct GlobalTransform {
    pub matrix: Mat3, // Matriz 3x3 de transformación afín 2D
    pub z_index: i32, // Profundidad calculada acumulada
}

/// Componentes de Jerarquía (para el modelo C4)
#[derive(Component)]
pub struct Parent(pub EntityId);

#[derive(Component)]
pub struct Children(pub Vec<EntityId>);

/// Sistema de Propagación de Transformadas
/// Multiplica matrices: Global(Child) = Global(Parent) * Local(Child)
fn transform_propagation_system(
    mut query: Query<(Entity, &Transform, Option<&Parent>, &mut GlobalTransform)>,
) {
    // CRÍTICO: El orden de procesamiento importa (padres antes que hijos)
    // En bevy_ecs, usamos un approach de 2 fases:
    // 1. Calcular GlobalTransform para entidades sin padre
    // 2. Propagar a hijos recursivamente
    todo!("Implementar propagacion topologica con bevy_ecs");
}
```

// Componentes de interacción
#[derive(Component)]
pub struct Selectable {
    pub selected: bool,
    pub hover: bool,
}

#[derive(Component)]
pub struct Draggable {
    pub dragging: bool,
    pub handle: Option<HandleId>,
}

#[derive(Component)]
pub struct Resizable {
    pub min_width: f32,
    pub min_height: f32,
    pub keep_aspect: bool,
}

// Componentes de animación
#[derive(Component)]
pub struct Animated {
    pub animation_id: AnimationId,
    pub from_value: f64,
    pub to_value: f64,
    pub progress: f64,
    pub property: AnimatedProperty,
}

// Componentes de conectores
#[derive(Component)]
pub struct Connector {
    pub source_entity: EntityId,
    pub target_entity: EntityId,
    pub source_handle: Option<HandleId>,
    pub target_handle: Option<HandleId>,
    pub routing: RoutingType,
}

#[derive(Component)]
pub struct ArrowHead {
    pub style: ArrowHeadStyle,
    pub position: ArrowHeadPosition,  // Start | End | Both
}
```

### 9.3 Sistemas del Motor

```rust
// Sistema de rendering
pub struct RenderSystem;
impl System for RenderSystem {
    fn update(&mut self, world: &mut World) {
        let ctx = world.resource::<RenderContext>();
        let renderer = world.resource::<mut dyn Renderer>();
        
        // Query: entidades visibles y no bloqueadas
        let visible: Vec<_> = world.query::<(
            &Transform, 
            &Visual, 
            &Bounds,
            !&Locked
        )>()
            .iter()
            .filter(|(_, bounds)| bounds.world.intersects(&ctx.viewport))
            .collect();
        
        renderer.render_frame(visible, ctx);
    }
}

// Sistema de spatial index
pub struct SpatialIndexSystem;
impl System for SpatialIndexSystem {
    fn update(&mut self, world: &mut World) {
        let spatial = world.resource_mut::<SpatialIndex>();
        let changes = world.resource::<EntityChanges>();
        
        spatial.update(&changes.modified);
    }
}

// Sistema de animaciones
pub struct AnimationSystem;
impl System for AnimationSystem {
    fn update(&mut self, world: &mut World, delta: Duration) {
        world.query::<&mut Animated>()
            .for_each(|(_, anim)| {
                anim.progress += delta.as_secs_f64();
                if anim.progress >= 1.0 {
                    // Animación completada
                    world.remove_component::<Animated>(anim.entity_id);
                }
            });
    }
}
```

---

## 10. Herramientas y Algoritmos de Referencia

### 10.1 Crates de Rust Recomendados

| Crate | Uso | Categoría |
|-------|-----|-----------|
| **lyon** | Tessellation de paths para GPU | Graphics |
| **kurbo** | Curvas y paths 2D | Geometry |
| **rstar** | R-Tree spatial indexing | Data Structures |
| **glam** | Matemáticas 2D/3D SIMD | Math |
| **euclid** | Tipos geométricos | Math |
| **wgpu** | Rendering GPU (opcional) | Graphics |
| **winit** | Window management | Platform |
| **image** | Carga de imágenes | Media |
| **fontdue** | Rendering de texto | Text |
| **hashbrown** | HashMap más rápido | Collections |

### 10.2 Librerías JS/TS de Referencia

| Librería | Patrón/Aprendizaje |
|----------|-------------------|
| **RoughJS** | Algoritmos hand-drawn |
| **tldraw** | Sistema ShapeUtil extensible |
| **Excalidraw** | Integración RoughJS + UI |
| **ReactFlow** | Nodos, handles, aristas |
| **Paper.js** | Operaciones booleanas 2D |
| **Fabric.js** | Canvas object model |
| **Konva.js** | Canvas reactivo |

### 10.3 Recursos de Algoritmos

- **Tessellation:** nical.github.io/posts/lyon-intro.html
- **RoughJS:** shihn.ca/posts/2020/roughjs-algorithms/
- **R-Tree:** docs.rs/rstar/latest/rstar/
- **ECS:** github.com/SanderMertens/ecs-faq

---

## 11. APIs Propuestas para Desarrolladores

### 11.1 API de Alto Nivel

```rust
use archflow_engine::{prelude::*, *};

fn main() -> Result<()> {
    // 1. Crear el engine
    let engine = Engine::new(EngineConfig {
        width: 1024,
        height: 768,
        renderer: RendererType::Canvas2D,
    })?;
    
    // 2. Crear un documento
    let doc = engine.new_document();
    
    // 3. Añadir formas
    let rect = doc.add_shape(Shape::rect()
        .position([100.0, 100.0])
        .size([200.0, 100.0])
        .fill_color("#3B82F6")
        .stroke_color("#1E40AF")
        .stroke_width(2.0)
        .corner_radius(8.0)
        .roughness(1.5)  // Modo sketchy
    )?;
    
    // 4. Crear un conector
    let arrow = doc.add_arrow(ArrowConfig {
        from: rect.id(),
        to: [400.0, 200.0],
        routing: RoutingType::Curved,
        start_head: ArrowHead::Triangle,
        end_head: ArrowHead::Circle { radius: 8.0 },
        animated: true,  // Partículas de flujo
    })?;
    
    // 5. Animar
    doc.animate(rect.id(), Animation::scale()
        .to([1.2, 1.2])
        .duration(Duration::from_millis(300))
        .easing(EasingFunction::EaseOutBack)
    )?;
    
    // 6. Render loop
    engine.run(|frame| {
        doc.render(frame);
    })?;
    
    Ok(())
}
```

### 11.2 API de Formas Personalizadas

```rust
// Definir una forma custom
struct CustomShape {
    config: CustomShapeConfig,
}

impl ShapeDef for CustomShape {
    type Config = CustomShapeConfig;
    type Props = CustomShapeProps;
    
    fn type_name() -> &'static str {
        "custom_shape"
    }
    
    fn default_config() -> Self::Config {
        CustomShapeConfig {
            points: vec![],
            color: Color::BLACK,
        }
    }
    
    fn compute_bounds(props: &Self::Props) -> Rect {
        // Calcular bounding box
        todo!()
    }
    
    fn hit_test(props: &Self::Props, point: Vec2) -> bool {
        // Hit testing custom
        todo!()
    }
}

// Registrar en el engine
engine.register_shape::<CustomShape>("custom_shape");
```

---

## 12. Roadmap de Implementación

### Fase 1: Core Engine (Semanas 1-4)
- [x] Setup de crates y estructura mono-repo
- [x] Tipos geométricos básicos (Vec2, Rect, Transform)
- [x] ECS básico (Entity, Component, Query)
- [x] Renderer Canvas 2D inicial
- [ ] Primitivas básicas (Rect, Ellipse, Line, Path)

### Fase 2: Primitivas y Estilos (Semanas 5-8)
- [ ] Sistema de estilos extensible
- [ ] Text primitive con font rendering
- [ ] Image primitive con filtros
- [ ] Arrow/Connector primitive con routing
- [ ] Group y Clip primitives

### Fase 3: Interactivity (Semanas 9-12)
- [ ] Spatial index (R-Tree)
- [ ] Hit testing optimizado
- [ ] Drag & drop
- [ ] Resize handles
- [ ] Selection/transform box

### Fase 4: Animaciones (Semanas 13-16)
- [ ] Keyframe animation system
- [ ] Easing functions
- [ ] Particle system para flows
- [ ] Property interpolation

### Fase 5: Hand-Drawn Rendering (Semanas 17-20)
- [ ] Rough-style algorithms
- [ ] Hachure fill generator
- [ ] Seeded RNG determinista
- [ ] Integración con primitives

### Fase 6: Optimización (Semanas 21-24)
- [ ] Culling spatial
- [ ] Batch rendering
- [ ] Level of detail (LOD)
- [ ] Performance profiling

---

## 13. Event Sourcing: Sistema de Journal Git-like

### 13.1 Filosofía Git para el Motor 2D

El motor debe comportarse como un **repositorio Git**: cada acción es un "commit", el estado actual es el resultado de aplicar todos los commits en secuencia, y podemos hacer checkout a cualquier punto anterior (undo/redo, time-travel debugging, branching para colaboración).

```rust
// El documento es un repositorio de eventos
pub struct Document {
    /// Journal de todos los eventos (commit history)
    events: EventJournal,
    
    /// Estado actual (cache derivado de eventos)
    snapshot: DocumentSnapshot,
    
    /// Head actual (puntero al commit actual)
    head: CommitId,
}

pub struct EventJournal {
    /// Todos los commits ordenados por secuencia
    commits: Vec<Commit>,
    
    /// Índice por Branch para navegación
    branches: HashMap<BranchName, Branch>,
    
    /// Commit actual por rama
    heads: HashMap<BranchName, CommitId>,
}

pub struct Commit {
    pub id: CommitId,           // SHA-1 hash del contenido
    pub parent_ids: Vec<CommitId>, // Múltiples padres para merge
    pub branch: BranchName,
    pub author: Author,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub events: Vec<DomainEvent>,
    pub schema_version: u32,     // Para migraciones
}
```

**NUEVO (v1.6): Ciclo de Acción - Store vs ECS**

```rust
/// Ciclo de Acción para evitar desincronización:
/// 1. INPUT → ECS (feedback visual instantáneo, mutable directo)
/// 2. COMMIT → Command Queue (acumular cambios)
/// 3. STORE → Event Journal (persistencia, undo/redo)
pub enum ActionPhase {
    /// Modo interactivo: dragging, resizing
    /// Direct ECS mutations para 60fps feedback
    Interactive,
    
    /// Modo commit: mouse up, finalize cambios
    /// Convierte mutations en DomainEvents
    Commit,
    
    /// Modo persistido: evento registrado
    /// Store actualizado
    Persisted,
}

// INPUT: Dragging muda el ECS directamente (feedback instantáneo)
fn on_pointer_move(
    pointer: &PointerState,
    mut transforms: Query<&mut Transform>,
) {
    if pointer.is_dragging() {
        if let Some((entity, delta)) = pointer.current_drag() {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                // DIRECT MUTATION: feedback visual instantáneo
                transform.translation += delta;
            }
        }
    }
}

// COMMIT: Al soltar, emitir Command
fn on_pointer_up(
    pointer: &PointerState,
    commands: &mut CommandQueue,
    transforms: Query<&Transform>,
) {
    if let Some(drag) = pointer.end_drag() {
        let final_transform = transforms.get(drag.entity).unwrap();
        
        commands.push(Command::MoveEntity {
            entity_id: drag.entity,
            from: drag.start_position,
            to: final_transform.translation,
        });
    }
}

// STORE: Command se convierte en DomainEvent
impl Document {
    pub fn execute_command(&mut self, command: Command) {
        match command {
            Command::MoveEntity { entity_id, from, to } => {
                self.current_transaction.add_event(
                    DomainEvent::EntityMoved { entity_id, delta: to - from, from, to }
                );
            }
        }
    }
    
    /// Al finalizar la transacción
    pub fn commit_transaction(&mut self) {
        let events = self.current_transaction.events();
        let commit = Commit::new_from_events(events, self.current_author.clone(), "".into());
        
        self.append_commit(commit);
        self.current_transaction.clear();
    }
}
```

### 13.2 Eventos de Dominio (Domain Events)

```rust
/// Eventos atómicos que representan cambios en el documento
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainEvent {
    // Eventos de entidad
    EntityCreated {
        entity_id: EntityId,
        entity_type: EntityType,
        initial_data: Value,  // JSON del estado inicial
    },
    EntityUpdated {
        entity_id: EntityId,
        property_path: String,  // e.g., "transform.position"
        old_value: Value,
        new_value: Value,
    },
    EntityDeleted {
        entity_id: EntityId,
        tombstone: Value,  // Último estado para undo
    },
    EntityRestored {
        entity_id: EntityId,
        from_tombstone: Value,
    },
    
    // Eventos de transformación
    EntityMoved {
        entity_id: EntityId,
        delta: Vec2,
        from: Vec2,
        to: Vec2,
    },
    EntityRotated {
        entity_id: EntityId,
        delta: Rad,
        around: Vec2,
    },
    EntityScaled {
        entity_id: EntityId,
        from_scale: Vec2,
        to_scale: Vec2,
        center: Vec2,
    },
    
    // Eventos de connectores
    ConnectorCreated {
        connector_id: EntityId,
        source: ConnectionEndpoint,
        target: ConnectionEndpoint,
    },
    ConnectorRerouted {
        connector_id: EntityId,
        old_waypoints: Vec<Vec2>,
        new_waypoints: Vec<Vec2>,
    },
    
    // Eventos de selección
    SelectionChanged {
        added: Vec<EntityId>,
        removed: Vec<EntityId>,
    },
    
    // Eventos de layer
    LayerCreated {
        layer_id: LayerId,
        parent_id: Option<LayerId>,
        name: String,
    },
    LayerOrderChanged {
        layer_id: LayerId,
        from_index: usize,
        to_index: usize,
    },
    EntityLayerChanged {
        entity_id: EntityId,
        from_layer: LayerId,
        to_layer: LayerId,
    },
    
    // Eventos de documento
    DocumentMetadataUpdated {
        field: String,
        old_value: Value,
        new_value: Value,
    },
}

impl DomainEvent {
    /// Determina si el evento puede comprimirse con el siguiente
    pub fn can_merge_with(&self, next: &Self) -> bool {
        match (self, next) {
            // Dos movimientos del mismo entity pueden merge
            (Self::EntityMoved { entity_id: id1, .. }, 
             Self::EntityMoved { entity_id: id2, .. }) => id1 == id2,
            
            // Dos updates del mismo entity pueden merge
            (Self::EntityUpdated { entity_id: id1, .. },
             Self::EntityUpdated { entity_id: id2, .. }) => id1 == id2,
            
            _ => false,
        }
    }
}
```

### 13.3 Sistema de Undo/Redo

```rust
pub struct UndoManager {
    /// Cola de commits undo (antes del HEAD actual)
    undo_stack: Vec<CommitId>,
    
    /// Cola de commits redo (después del HEAD, se limpia en nuevo commit)
    redo_stack: Vec<CommitId>,
    
    /// Máximo número de estados en undo
    max_undo_depth: usize,
}

impl UndoManager {
    pub fn undo(&mut self, doc: &mut Document) -> Result<(), UndoError> {
        let Some(commit_id) = self.undo_stack.pop() else {
            return Err(UndoError::NoMoreUndos);
        };
        
        // Obtener el commit padre para ir hacia atrás
        let commit = doc.events.get_commit(commit_id);
        let target_commit = commit.parent_ids.first()
            .ok_or(UndoError::InitialCommit)?;
        
        doc.checkout(target_commit)?;
        self.redo_stack.push(commit_id);
        
        Ok(())
    }
    
    pub fn redo(&mut self, doc: &mut Document) -> Result<(), RedoError> {
        let Some(commit_id) = self.redo_stack.pop() else {
            return Err(RedoError::NoMoreRedos);
        };
        
        doc.checkout(commit_id)?;
        self.undo_stack.push(commit_id);
        
        Ok(())
    }
    
    /// Registrar un nuevo commit (limpia redo stack)
    pub fn on_new_commit(&mut self, commit_id: CommitId) {
        self.redo_stack.clear();
        self.undo_stack.push(commit_id);
        
        // Limitar profundidad
        if self.undo_stack.len() > self.max_undo_depth {
            self.undo_stack.remove(0);
        }
    }
}
```

### 13.4 Branching y Colaboración

```rust
pub struct Branch {
    pub name: BranchName,
    pub head: CommitId,
    pub created_at: DateTime<Utc>,
    pub created_by: Author,
    pub description: String,
}

impl Document {
    /// Crear una nueva rama desde el commit actual
    pub fn branch(&mut self, name: BranchName, author: Author) -> Branch {
        let branch = Branch {
            name: name.clone(),
            head: self.head,
            created_at: Utc::now(),
            created_by: author,
            description: String::new(),
        };
        
        self.events.branches.insert(name.clone(), branch.clone());
        self.events.heads.insert(name, self.head);
        
        branch
    }
    
    /// Cambiar a otra rama (switch branch)
    pub fn checkout_branch(&mut self, name: &BranchName) -> Result<(), CheckoutError> {
        let head = self.events.heads.get(name)
            .ok_or(CheckoutError::BranchNotFound)?;
        
        self.checkout(head)
    }
    
    /// Mergear una rama en otra
    pub fn merge(
        &mut self, 
        source: &BranchName, 
        target: &BranchName,
        strategy: MergeStrategy,
    ) -> Result<MergeResult, MergeError> {
        let source_head = self.events.heads.get(source)
            .ok_or(MergeError::BranchNotFound)?;
        let target_head = self.events.heads.get(target)
            .ok_or(MergeError::BranchNotFound)?;
        
        // Los dos commits base y sus ancestros comunes
        let base = self.events.find_common_ancestor(source_head, target_head);
        let source_commits = self.events.get_commits_since(base, source_head);
        let target_commits = self.events.get_commits_since(base, target_head);
        
        // Three-way merge
        let merge_result = self.three_way_merge(
            self.events.get_commit(*base),
            source_commits,
            target_commits,
            strategy,
        )?;
        
        // Crear merge commit
        let merge_commit = Commit::new_merge(
            self.head,
            *source_head,
            self.events.current_author.clone(),
            format!("Merge branch '{}' into '{}'", source, target),
            merge_result.events,
        );
        
        self.append_commit(merge_commit)?;
        
        Ok(merge_result)
    }
}
```

### 13.5 Reconstrucción de Estado desde Cero

```rust
impl Document {
    /// Reconstruir el estado completo aplicando todos los eventos
    pub fn rebuild_state(&mut self) -> Result<(), RebuildError> {
        let mut world = World::new();
        let mut spatial_index = SpatialIndex::new();
        
        // Obtener todos los commits hasta el HEAD actual
        let commits = self.events.get_commits_until(self.head);
        
        for commit in commits {
            // Aplicar cada evento del commit
            for event in &commit.events {
                self.apply_event(&mut world, &mut spatial_index, event)?;
            }
            
            // Notificar sistemas del commit aplicado
            self.on_commit_applied(commit.id, &world);
        }
        
        // Actualizar snapshot
        self.snapshot = DocumentSnapshot {
            world,
            spatial_index,
            timestamp: Utc::now(),
            commit_id: self.head,
        };
        
        Ok(())
    }
    
    fn apply_event(
        &self,
        world: &mut World,
        spatial: &mut SpatialIndex,
        event: &DomainEvent,
    ) -> Result<(), ApplyError> {
        match event {
            DomainEvent::EntityCreated { entity_id, entity_type, initial_data } => {
                let entity = world.create_entity(*entity_id);
                // Deserializar componentes desde initial_data
                for component in entity_type.components() {
                    let value = initial_data.get(&component.name());
                    let component_data = deserialize_component(value, component)?;
                    world.add_component(entity, component_data);
                }
                spatial.insert(entity.clone());
            }
            
            DomainEvent::EntityUpdated { entity_id, property_path, new_value, .. } => {
                if let Some(entity) = world.get_entity_mut(*entity_id) {
                    let component = parse_property_path(property_path)?;
                    let value = deserialize_value(new_value, &component.type_info())?;
                    world.set_component_value(entity, component, value);
                    
                    // Actualizar spatial index si es posición/size
                    if is_spatial_property(property_path) {
                        let bounds = world.get_bounds(entity);
                        spatial.update_bounds(entity.id, bounds);
                    }
                }
            }
            
            DomainEvent::EntityDeleted { entity_id, .. } => {
                world.delete_entity(*entity_id);
                spatial.remove(*entity_id);
            }
            
            DomainEvent::EntityMoved { entity_id, to, .. } => {
                if let Some(transform) = world.get_component_mut::<Transform>(*entity_id) {
                    transform.position = *to;
                    let bounds = world.get_bounds(*entity_id);
                    spatial.update_bounds(*entity_id, bounds);
                }
            }
            
            // ... otros eventos
        }
        
        Ok(())
    }
}
```

### 13.6 Time-Travel Debugging

```rust
/// Viajar en el tiempo a cualquier commit
pub struct TimeTravel {
    history: Vec<TimePoint>,
    bookmarks: HashMap<String, CommitId>,
}

pub struct TimePoint {
    commit_id: CommitId,
    message: String,
    timestamp: DateTime<Utc>,
    thumbnail: Option<ImageData>,
    state_hash: u64,  // Hash del estado para comparación rápida
}

impl TimeTravel {
    /// Viajar a un punto en el tiempo
    pub fn goto(
        &mut self, 
        doc: &mut Document, 
        target: &TimeTravelTarget,
    ) -> Result<(), TimeTravelError> {
        match target {
            TimeTravelTarget::Commit(id) => {
                doc.checkout(id)?;
            }
            TimeTravelTarget::Timestamp(ts) => {
                let commits = doc.events.get_commits_up_to(*ts);
                if let Some(last) = commits.last() {
                    doc.checkout(&last.id)?;
                }
            }
            TimeTravelTarget::Bookmark(name) => {
                let commit_id = self.bookmarks.get(name)
                    .ok_or(TimeTravelError::BookmarkNotFound)?;
                doc.checkout(commit_id)?;
            }
        }
        
        // Generar thumbnail del estado
        self.generate_thumbnail(doc);
        
        Ok(())
    }
    
    /// Comparar dos puntos en el tiempo
    pub fn diff(
        &self, 
        doc: &Document,
        from: CommitId, 
        to: CommitId,
    ) -> DiffResult {
        let from_state = doc.rebuild_state_at(from);
        let to_state = doc.rebuild_state_at(to);
        
        DiffResult {
            added: to_state.entities()
                .filter(|e| !from_state.has_entity(e.id))
                .collect(),
            removed: from_state.entities()
                .filter(|e| !to_state.has_entity(e.id))
                .collect(),
            modified: from_state.entities()
                .filter_map(|e| {
                    let to_entity = to_state.get_entity(e.id)?;
                    if e != to_entity {
                        Some((e.clone(), to_entity.clone()))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}
```

### 13.7 Colaboración en Tiempo Real (CRDT-ready)

```rust
/// Evento remote recibido del servidor
#[derive(Debug, Clone)]
pub struct RemoteEvent {
    pub peer_id: PeerId,
    pub events: Vec<DomainEvent>,
    pub vector_clock: VectorClock,
    pub base_commit: CommitId,
}

impl Document {
    /// Aplicar eventos remotos
    pub fn apply_remote_events(
        &mut self, 
        remote: RemoteEvent,
    ) -> Result<(), RemoteApplyError> {
        // Verificar causalidad con vector clock
        if !self.vector_clock.can_apply(&remote.vector_clock) {
            return Err(RemoteApplyError::CausalityViolation);
        }
        
        // Verificar que base_commit es ancestro del HEAD
        let is_ancestor = self.events.is_ancestor(
            remote.base_commit, 
            self.head
        );
        
        if !is_ancestor {
            // Conflict! Necesitamos rebasar o resolver merge
            return self.resolve_conflict(remote);
        }
        
        // Aplicar eventos
        let start_head = self.head;
        for event in remote.events {
            self.apply_single_event(&event)?;
        }
        
        // Crear commit de sync
        let sync_commit = Commit::new_sync(
            vec![start_head],  // Un solo padre
            remote.peer_id,
            format!("Sync with peer {}", remote.peer_id),
            remote.events,
        );
        
        self.append_commit(sync_commit)?;
        
        // Actualizar vector clock
        self.vector_clock.merge(&remote.vector_clock);
        
        Ok(())
    }
}

/// Vector Clock para ordenamiento causal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    clocks: HashMap<PeerId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self { clocks: HashMap::new() }
    }
    
    pub fn increment(&mut self, peer_id: PeerId) {
        let count = self.clocks.entry(peer_id).or_insert(0);
        *count += 1;
    }
    
    pub fn can_apply(&self, other: &VectorClock) -> bool {
        // Verificar causalidad: todos los eventos que preceden a `other`
        // ya están en `self`
        for (peer, &timestamp) in &other.clocks {
            let my_timestamp = self.clocks.get(peer).copied().unwrap_or(0);
            if timestamp <= my_timestamp + 1 {
                // Excepción para el peer que envía
                continue;
            }
        }
        true
    }
    
    pub fn merge(&mut self, other: &VectorClock) {
        for (peer, &timestamp) in &other.clocks {
            let my_timestamp = self.clocks.entry(*peer).or_insert(0);
            *my_timestamp = (*my_timestamp).max(timestamp);
        }
    }
}
```

### 13.8 Persistencia y Optimización

```rust
impl EventJournal {
    /// Compactar journal (squash de commits antiguos)
    pub fn compact(&mut self, keep_last_n: usize) -> Result<(), CompactError> {
        let commits_to_merge = &self.commits[..self.commits.len() - keep_last_n];
        if commits_to_merge.is_empty() {
            return Ok(());
        }
        
        // Reconstruir estado hasta el punto de compactación
        let mut world = World::new();
        let mut spatial = SpatialIndex::new();
        
        // Solo ejecutar hasta el punto de keep
        for commit in &self.commits[..commits.len() - keep_last_n] {
            for event in &commit.events {
                apply_event(&mut world, &mut spatial, event);
            }
        }
        
        // Crear un único "snapshot commit"
        let snapshot_event = DomainEvent::SnapshotCreated {
            snapshot_data: serialize_world(&world),
            original_commits: commits_to_merge.len(),
        };
        
        let snapshot_commit = Commit::new_snapshot(
            self.commits[0].id,  // Mantener ancestría
            snapshot_event,
        );
        
        // Reemplazar commits antiguos con el snapshot
        let remaining = self.commits.split_off(self.commits.len() - keep_last_n);
        self.commits = vec![snapshot_commit];
        self.commits.extend(remaining);
        
        // Actualizar índices
        self.rebuild_indices();
        
        Ok(())
    }
    
    /// Exportar documento como JSON (repo portable)
    pub fn export(&self) -> ExportData {
        ExportData {
            version: CURRENT_VERSION,
            events: self.commits.iter()
                .flat_map(|c| &c.events)
                .cloned()
                .collect(),
            branches: self.branches.clone(),
            heads: self.heads.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// 13.9 Sistema de Snapshots (Export/Import)
/// ==========================================
/// El patrón híbrido: Event Sourcing para edición, Snapshots para compartir.
/// Figma y tldraw usan esto: el documento en memoria es events + replay,
/// pero cuando guardas/exportas, creas un snapshot (estado serializado).
/// Esto reduce drásticamente el tamaño para compartir y mejora el load time.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    /// Hash del snapshot (verificación de integridad)
    pub checksum: Checksum,
    
    /// Versión del formato de snapshot
    pub version: u32,
    
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
    
    /// Estado completo del documento
    pub world: WorldSnapshot,
    
    /// Metadata adicional
    pub metadata: DocumentMetadata,
    
    /// Para differential sync
    pub parent_snapshot_id: Option<SnapshotId>,
    
    /// Compresión usada
    pub compression: CompressionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Entidades serializadas
    pub entities: Vec<EntitySnapshot>,
    
    /// Layers
    pub layers: Vec<LayerSnapshot>,
    
    /// Estilos
    pub styles: Vec<StyleSnapshot>,
    
    /// Recursos (imágenes, fuentes)
    pub resources: Vec<ResourceSnapshot>,
    
    /// Bounds del documento
    pub document_bounds: Rect,
    
    /// Viewport default
    pub default_viewport: Viewport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub entity_type: String,
    #[serde(default)]
    pub transform: Option<Transform>,
    #[serde(default)]
    pub bounds: Option<Bounds>,
    #[serde(default)]
    pub visual: Option<Visual>,
    #[serde(default)]
    pub props: HashMap<String, Value>,
    pub layer_id: LayerId,
    pub z_index: i32,
}

impl EventJournal {
    /// Crear un snapshot del estado actual
    pub fn create_snapshot(&self, world: &World) -> DocumentSnapshot {
        // 1. Reconstruir estado desde eventos
        let world_snapshot = self.serialize_world(world);
        
        // 2. Calcular checksum
        let data = bincode::serialize(&world_snapshot).unwrap();
        let checksum = blake3::hash(&data);
        
        // 3. Guardar referencia al snapshot anterior (para differential)
        let parent_id = self.latest_snapshot_id;
        
        DocumentSnapshot {
            checksum: Checksum::from_hash(checksum),
            version: SNAPSHOT_VERSION,
            created_at: Utc::now(),
            world: world_snapshot,
            metadata: self.metadata.clone(),
            parent_snapshot_id: parent_id,
            compression: CompressionType::Zstd,
        }
    }
    
    /// Guardar snapshot a archivo (para compartir)
    pub fn save_snapshot(
        &self, 
        world: &World,
        path: &Path,
    ) -> Result<SnapshotId, SaveError> {
        let snapshot = self.create_snapshot(world);
        
        // Serializar a bytes
        let mut bytes = Vec::new();
        bincode::serialize_into(&mut bytes, &snapshot)
            .map_err(SaveError::Serialize)?;
        
        // Comprimir (Zstd para mejor ratio que gzip, más rápido que lz4)
        let compressed = zstd::stream::encode_all(std::io::Cursor::new(bytes), 0)
            .map_err(SaveError::Compress)?;
        
        // Escribir a archivo
        std::fs::write(path, compressed)
            .map_err(SaveError::Io)?;
        
        // Registrar snapshot
        let snapshot_id = snapshot.id();
        self.snapshot_registry.insert(snapshot_id, path.to_path_buf());
        
        Ok(snapshot_id)
    }
    
    /// Cargar snapshot (para abrir archivos compartidos)
    pub fn load_snapshot(
        &mut self, 
        path: &Path,
    ) -> Result<World, LoadError> {
        // 1. Leer y descomprimir
        let compressed = std::fs::read(path)
            .map_err(LoadError::Io)?;
        let decompressed = zstd::stream::decode_all(std::io::Cursor::new(compressed))
            .map_err(LoadError::Decompress)?;
        
        // 2. Deserializar
        let snapshot: DocumentSnapshot = bincode::deserialize(&decompressed)
            .map_err(LoadError::Deserialize)?;
        
        // 3. Verificar integridad
        let data = bincode::serialize(&snapshot.world)
            .map_err(LoadError::Verify)?;
        let checksum = blake3::hash(&data);
        if checksum != snapshot.checksum.as_bytes() {
            return Err(LoadError::ChecksumMismatch);
        }
        
        // 4. Reconstruir World desde snapshot
        let world = self.deserialize_world(&snapshot.world);
        
        // 5. Inicializar journal con un único "snapshot commit"
        self.initialize_from_snapshot(&snapshot, world.id());
        
        Ok(world)
    }
    
    /// Cargar snapshot + aplicar eventos增量 (differential sync)
    pub fn load_snapshot_with_diff(
        &mut self,
        base_snapshot: &SnapshotId,
        events: &[DomainEvent],
    ) -> Result<World, LoadError> {
        // 1. Cargar snapshot base
        let (mut world, snapshot_commit_id) = self.load_snapshot_id(base_snapshot)?;
        
        // 2. Aplicar eventos增量 (deltas)
        for event in events {
            self.apply_event(&mut world, &mut self.spatial_index, event)?;
        }
        
        // 3. Crear nuevo commit
        let new_commit = Commit::new_delta(
            vec![snapshot_commit_id],
            self.current_author.clone(),
            format!("Differential sync from {}", base_snapshot),
            events.to_vec(),
        );
        
        self.append_commit(new_commit)?;
        
        Ok(world)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Snapshot completo (recomendado para compartir)
    Snapshot(SnapshotExport),
    
    /// Solo eventos (máxima edición, malo para compartir)
    Events(EventsExport),
    
    /// Diferencial desde snapshot
    Diff(DiffExport),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotExport {
    pub snapshot: DocumentSnapshot,
    pub include_resources: bool,  // Incluir imágenes embebidas
    pub compression: CompressionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsExport {
    pub events: Vec<DomainEvent>,
    pub from_commit: Option<CommitId>,
    pub to_commit: Option<CommitId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffExport {
    pub base_snapshot_id: SnapshotId,
    pub events: Vec<DomainEvent>,
    pub original_checksum: Checksum,
    pub new_checksum: Checksum,
}

impl Document {
    /// Exportar documento (user-facing API)
    pub fn export(
        &self, 
        format: ExportFormat,
    ) -> Result<Vec<u8>, ExportError> {
        match format {
            ExportFormat::Snapshot(snapshot_export) => {
                // Serializar y comprimir snapshot
                let mut bytes = Vec::new();
                bincode::serialize_into(&mut bytes, &snapshot_export.snapshot)
                    .map_err(ExportError::Serialize)?;
                
                let compressed = match snapshot_export.compression {
                    CompressionType::None => bytes,
                    CompressionType::Gzip => gzip_encode(&bytes),
                    CompressionType::Zstd => zstd_encode(&bytes, 3),
                    CompressionType::Lz4 => lz4_encode(&bytes),
                };
                
                // Añadir header (magic bytes + versión)
                Ok(self.encode_with_header(compressed))
            }
            
            ExportFormat::Events(events_export) => {
                // Exportar solo eventos
                let events: Vec<_> = match (events_export.from_commit, events_export.to_commit) {
                    (None, None) => self.events.get_all_events(),
                    (Some(from), Some(to)) => self.events.get_events_range(from, to),
                    _ => return Err(ExportError::InvalidRange),
                };
                
                bincode::serialize(&ExportFormat::Events(EventsExport {
                    events,
                    from_commit: events_export.from_commit,
                    to_commit: events_export.to_commit,
                })).map_err(ExportError::Serialize)
            }
            
            ExportFormat::Diff(diff_export) => {
                bincode::serialize(&diff_export)
                    .map_err(ExportError::Serialize)
            }
        }
    }
    
    /// Importar documento (user-facing API)
    pub fn import(
        &mut self,
        data: &[u8],
    ) -> Result<ImportResult, ImportError> {
        // Detectar formato por header
        if data.starts_with(SNAPSHOT_MAGIC) {
            self.import_snapshot(&data[SNAPSHOT_HEADER_SIZE..])
        } else {
            self.import_events(data)
        }
    }
    
    fn encode_with_header(&self, compressed: Vec<u8>) -> Vec<u8> {
        let mut header = Vec::new();
        header.extend_from_slice(SNAPSHOT_MAGIC);
        header.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
        header.extend_from_slice(compressed.as_slice());
        header
    }
    
    fn import_snapshot(&mut self, data: &[u8]) -> Result<ImportResult, ImportError> {
        let decompressed = zstd_decode(data)
            .map_err(ImportError::Decompress)?;
        
        let snapshot: DocumentSnapshot = bincode::deserialize(&decompressed)
            .map_err(ImportError::Deserialize)?;
        
        let world = self.deserialize_world(&snapshot.world);
        
        self.initialize_from_snapshot(&snapshot, world.id());
        
        Ok(ImportResult {
            format: ImportFormat::Snapshot,
            entities_created: snapshot.world.entities.len(),
            events_replayed: 0,  // Snapshot no replay
            file_size: data.len(),
        })
    }
    
    fn import_events(&mut self, data: &[u8]) -> Result<ImportResult, ImportError> {
        let events_export: EventsExport = bincode::deserialize(data)
            .map_err(ImportError::Deserialize)?;
        
        let events_count = events_export.events.len();
        
        // Crear commits desde eventos
        let mut commits = Vec::new();
        let mut current_batch = Vec::new();
        
        for event in events_export.events {
            current_batch.push(event);
            
            if current_batch.len() >= BATCH_SIZE {
                let commit = Commit::new_from_events(
                    current_batch.clone(),
                    self.current_author.clone(),
                    format!("Batch import"),
                );
                commits.push(commit);
                current_batch.clear();
            }
        }
        
        if !current_batch.is_empty() {
            let commit = Commit::new_from_events(
                current_batch,
                self.current_author.clone(),
                format!("Batch import (final)"),
            );
            commits.push(commit);
        }
        
        // Aplicar todos los commits
        for commit in commits {
            self.append_commit(commit)?;
        }
        
        Ok(ImportResult {
            format: ImportFormat::Events,
            entities_created: 0,  // Contar al replay
            events_replayed: events_count,
            file_size: data.len(),
        })
    }
}

/// Comparación de formatos
/// =======================
/// Formato          │ Tamaño  │ Editable │ Load Time │ Uso
/// ─────────────────┼─────────┼──────────┼───────────┼────────────────────────
/// Full Snapshot    │ Grande  │ ✅ Sí    │ Rápido    │ Compartir archivos
/// Events Only      │ Pequeño │ ✅ Sí    │ Lento     │ Version control interno
/// Differential     │ Muy peq │ ✅ Sí    │ Muy rápido│ Sync de red
/// Snapshot + Res   │ Muy grande│ ⚠️   │ Rápido    │ Archivos autocontenidos
```

`✶ Insight ─────────────────────────────────────`
**Event Sourcing + Git:** El patrón más poderoso para editores colaborativos. Figma usa internamente algo similar: el documento es una lista de operaciones que se aplican en orden. Esto permite:
- Undo/redo infinito sin crecer memoria exponencialmente
- Colaboración en tiempo real con resolución de conflictos
- Time-travel debugging (viajar a cualquier punto de la historia)
- Branching para experimentación
- Exportación portable (el archivo es solo la lista de eventos)
`─────────────────────────────────────────────────`

---

## 13.10 Recursos Externos: Imágenes, Videos y HTML Embebido

### 13.10.1 Cómo lo hacen Figma y Tldraw

**Tldraw** usa un sistema de **capas múltiples**:
1. **Canvas principal** para vectores y formas
2. **DOM overlays** para iframes y contenido HTML
3. **Iframes embebidos** para servicios externos (YouTube, Figma, etc.)

**Figma** maneja recursos de forma diferente:
1. **Imágenes:** Se rasterizan y almacenan como datos comprimidos
2. **Videos:** Se soportan como fills, no como elementos independientes
3. **Embebidos:** No soporta iframes, solo imágenes y videos

### 13.10.2 Arquitectura propuesta para ArchFlow

```rust
/// Tipos de recursos soportados
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExternalResource {
    Image(ImageResource),
    Video(VideoResource),
    Embed(EmbedResource),
    HtmlOverlay(HtmlOverlayResource),
}

/// Fuente de imagen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSource {
    RemoteUrl { url: String, cache_policy: CachePolicy },
    Embedded { mime_type: String, data: Vec<u8> },  // Zstd compressed
    LocalPath { path: std::path::PathBuf },
}

/// Modo de ajuste de imagen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFitMode {
    Fill,    // Crop para llenar
    Contain, // Letterbox
    Cover,   // Crop para cubrir
    None,    // Tamaño original
}

/// Recurso de video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResource {
    pub id: ResourceId,
    pub source: VideoSource,
    pub bounds: Rect,
    pub playback: VideoPlayback,
    pub volume: f32,
    pub loop_play: bool,
    pub auto_play: bool,
    pub thumbnail: Option<ResourceId>,
}

/// HTML Overlay - renderizado DOM sobre canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlOverlayResource {
    pub id: ResourceId,
    pub bounds: Rect,
    pub html_content: String,  // HTML directo
    pub iframe_url: Option<String>,  // O iframe externo
    pub interaction_mode: HtmlInteractionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HtmlInteractionMode {
    Interactive,   // Usuario puede interactuar
    Scrollable,    // Tiene scroll propio
    Passthrough,   // Click-through
    ViewOnly,      // Solo visualización
}
```

### 13.10.3 Sistema de Capas para Recursos

```
┌─────────────────────────────────────────┐
│  Capa UI (z=1000+)                      │  → Tooltips, handles, menús
├─────────────────────────────────────────┤
│  Capa HTML Overlay (z=201+)             │  → iframes, DOM embebido
├─────────────────────────────────────────┤
│  Capa Video (z=101-200)                 │  → HTML5 Video elements
├─────────────────────────────────────────┤
│  Capa Imágenes (z=1-100)                │  → Canvas ImageRendering
├─────────────────────────────────────────┤
│  Capa Vector (z=0)                      │  → Rect, Path, Text, etc.
└─────────────────────────────────────────┘
```

### 13.10.4 Gestor de Recursos con Caché LRU

```rust
pub struct ResourceManager {
    cache: HashMap<ResourceId, CachedResource>,
    download_queue: Vec<ResourceId>,
    max_cache_size: usize,  // e.g., 100MB
    image_loader: ImageLoader,
}

impl ResourceManager {
    pub async fn load_image(&self, source: &ImageSource) 
        -> Result<LoadedImage, ResourceError> 
    {
        match source {
            ImageSource::RemoteUrl { url, policy } => {
                self.fetch_remote(url, policy).await
            }
            ImageSource::Embedded { mime_type, data } => {
                self.decode_embedded(mime_type, data)
            }
            ImageSource::LocalPath { path } => {
                self.read_local(path)
            }
        }
    }
    
    /// Renderizar imagen en canvas
    pub fn render_image(
        &self, 
        img: &LoadedImage, 
        bounds: Rect,
        fit: ImageFitMode,
    ) -> RenderCommand {
        let source_rect = calculate_source_rect(img.size, bounds, fit);
        RenderCommand::DrawImage {
            texture: img.texture.clone(),
            source_rect,
            dest_rect: bounds,
        }
    }
}
```

### 13.10.5 Embed Providers (Tldraw-style)

```rust
pub struct EmbedProvider {
    pub name: String,
    pub pattern: Regex,  // URL matcher
    pub embed_renderer: Box<dyn EmbedRenderer>,
}

pub trait EmbedRenderer {
    fn create_iframe(&self, url: &str, bounds: Rect) 
        -> Result<web_sys::HtmlIframeElement, Error>;
}

// Providers por defecto
impl EmbedProvider {
    pub fn default_providers() -> Vec<Self> {
        vec![
            // YouTube
            Self {
                name: "YouTube".into(),
                pattern: Regex::new(r"youtube\.com/watch\?v=[\w-]+").unwrap(),
                embed_renderer: Box::new(YouTubeRenderer),
            },
            // Vimeo
            Self {
                name: "Vimeo".into(),
                pattern: Regex::new(r"vimeo\.com/\d+").unwrap(),
                embed_renderer: Box::new(VimeoRenderer),
            },
            // Figma
            Self {
                name: "Figma".into(),
                pattern: Regex::new(r"figma\.com/(file|design)/").unwrap(),
                embed_renderer: Box::new(FigmaRenderer),
            },
        ]
    }
}

pub struct YouTubeRenderer;

impl EmbedRenderer for YouTubeRenderer {
    fn create_iframe(&self, url: &str, bounds: Rect) 
        -> Result<web_sys::HtmlIframeElement, Error> 
    {
        let video_id = extract_video_id(url);
        let embed_url = format!(
            "https://www.youtube.com/embed/{}?autoplay=0&controls=1",
            video_id
        );
        create_iframe(embed_url, bounds)
    }
}
```

### 13.10.6 Documentos con Recursos Embebidos

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentWithResources {
    pub snapshot: DocumentSnapshot,
    pub resources: Vec<DocumentResource>,
    pub embed_threshold: usize,  // < 64KB inline, >64KB reference
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResource {
    pub id: ResourceId,
    pub embedded: bool,
    pub mime_type: String,
    pub size_bytes: usize,
    pub hash: Checksum,
    // Si embedded: datos inline
    // Si no: url externa
    pub source: ResourceDataSource,
}

impl DocumentWithResources {
    /// Guardar documento con recursos
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        let mut bytes = Vec::new();
        bincode::serialize_into(&mut bytes, self).map_err(SaveError::Serialize)?;
        
        // Comprimir con Zstd
        let compressed = zstd::encode_all(std::io::Cursor::new(bytes), 3)
            .map_err(SaveError::Compress)?;
        
        // Header: "AFLW" + versión + datos
        let mut file = Vec::new();
        file.extend_from_slice(b"AFLW");  // Magic bytes
        file.extend_from_slice(&1u32.to_le_bytes());  // Versión
        file.extend_from_slice(&compressed.len().to_le_bytes());
        file.extend(compressed);
        
        std::fs::write(path, file).map_err(SaveError::Io)
    }
    
    /// Cargar documento con recursos
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let data = std::fs::read(path).map_err(LoadError::Io)?;
        
        // Verificar header
        if &data[..4] != b"AFLW" {
            return Err(LoadError::InvalidFormat);
        }
        
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let data_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let compressed = &data[12..12 + data_len as usize];
        
        let decompressed = zstd::decode_all(std::io::Cursor::new(compressed))
            .map_err(LoadError::Decompress)?;
        
        bincode::deserialize(&decompressed).map_err(LoadError::Deserialize)
    }
}
```

### 13.10.7 Resumen de Recursos

| Tipo | Rendering | Formato Archivo | Notas |
|------|-----------|-----------------|-------|
| **Imagen** | Canvas drawImage | Inline (base64/zstd) o URL | Fit modes: Fill/Contain/Cover |
| **Video** | HTML5 Video overlay | Inline o URL | Auto-play, loop, thumbnail |
| **HTML** | DOM overlay/iframe | HTML string o URL | Interaction modes |
| **Embed** | Iframe con provider | URL externa | YouTube, Figma, Vimeo, etc. |

`✶ Insight ─────────────────────────────────────`
**Recursos externos:** El patrón de tldraw es superior para usabilidad:
- Imágenes pequeñas → inline (archivo autocontenido)
- Imágenes grandes → URL + hash (verificación de integridad)
- Videos/HTML → iframes con sandbox (seguridad)
- Capa de abstracción permite añadir providers nuevos sin cambiar core
`─────────────────────────────────────────────────`

---

## 13.11 Árbol de Profundidad: Zoom de Detalle Incremental

### 13.11.1 Filosofía del "Abanico de Detalle"

A diferencia del modelo C4 (donde cada nivel reemplaza completamente el anterior), ArchFlow implementa un **árbol de profundidad** donde:

- Los elementos **permanecen** en todos los niveles
- Cada nivel **añade más detalles** sobre los mismos elementos
- Es como **Google Maps**: Calles → Edificios → Tiendas → Interior

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ZOOM DE DETALLE INCREMENTAL                          │
│                                                                         │
│  Nivel 0: MACRO (visión superficial)                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │     ┌─────────┐              ┌─────────┐                        │   │
│  │     │ Service │──────────────│   DB    │                        │   │
│  │     └─────────┘              └─────────┘                        │   │
│  │      (solo rectángulos, sin detalles)                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Nivel 1: CONNECTIONS (conexiones visibles)                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │     ┌─────────┐═══════╗       ┌─────────┐                        │   │
│  │     │ Service │║ HTTP  ║──────│   DB    │                        │   │
│  │     └─────────┘╚═══════╝       └─────────┘                        │   │
│  │      (ahora ves protocolos, puertos, endpoints)                   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Nivel 2: INTERNALS (internos de cada componente)                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  ┌─────────────────────────┐    ┌─────────────────────────┐     │   │
│  │  │      Service            │    │      DB                 │     │   │
│  │  │  ┌─────┐ ┌─────┐ ┌─────┐│    │  ┌─────┐ ┌─────┐       │     │   │
│  │  │  │ Auth│ │ API │ │Logger││    │  │Read │ │Write│       │     │   │
│  │  │  └─────┘ └─────┘ └─────┘│    │  └─────┘ └─────┘       │     │   │
│  │  └─────────────────────────┘    └─────────────────────────┘     │   │
│  │      (ahora ves componentes internos)                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Nivel 3: DETAIL (parámetros, configs, métricas)                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  ┌─────────────────────────┐    ┌─────────────────────────┐     │   │
│  │  │      Service ░░░░░░░░░░ │    │      DB ░░░░░░░░░░░░░░░ │     │   │
│  │  │  ░░ Auth    ░ CPU: 50%  │    │  ░ Read   ░ Rows: 1.2M  │     │   │
│  │  │  ░ API      ░ RAM: 512MB│    │  ░ Write  ░ Conn: 45    │     │   │
│  │  │  ░ Logger   ░ Replicas:3│    │  ░ Backup ░ Size: 25GB  │     │   │
│  │  │  ░░░░░░░░░░░░░░░░░░░░░░░│    │  ░░░░░░░░░░░░░░░░░░░░░  │     │   │
│  │  └─────────────────────────┘    └─────────────────────────┘     │   │
│  │      (métricas, configs, logs en tiempo real)                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [Scroll wheel → cambiar nivel de detalle]                              │
│  [Los elementos nunca desaparecen, solo revelan más]                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 13.11.2 Sistema de Niveles de Profundidad

```rust
/// Niveles de profundidad (zoom de detalle)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DetailLevel {
    /// Nivel 0: Solo formas básicas
    Macro = 0,
    /// Nivel 1: Conexiones y protocolos
    Connections = 1,
    /// Nivel 2: Componentes internos
    Internals = 2,
    /// Nivel 3: Métricas, configs, detalles técnicos
    Detail = 3,
    /// Nivel 4: Código fuente, logs, trazas
    Code = 4,
}

impl DetailLevel {
    pub fn name(&self) -> &'static str {
        match self {
            DetailLevel::Macro => "Macro",
            DetailLevel::Connections => "Connections",
            DetailLevel::Internals => "Internals",
            DetailLevel::Detail => "Detail",
            DetailLevel::Code => "Code",
        }
    }
    
    pub fn next(&self) -> Option<Self> {
        match self {
            DetailLevel::Macro => Some(DetailLevel::Connections),
            DetailLevel::Connections => Some(DetailLevel::Internals),
            DetailLevel::Internals => Some(DetailLevel::Detail),
            DetailLevel::Detail => Some(DetailLevel::Code),
            DetailLevel::Code => None,
        }
    }
    
    pub fn prev(&self) -> Option<Self> {
        match self {
            DetailLevel::Macro => None,
            DetailLevel::Connections => Some(DetailLevel::Macro),
            DetailLevel::Internals => Some(DetailLevel::Connections),
            DetailLevel::Detail => Some(DetailLevel::Internals),
            DetailLevel::Code => Some(DetailLevel::Detail),
        }
    }
}

/// Configuración de visibilidad por nivel
#[derive(Component)]
pub struct DetailVisibility {
    pub level: DetailLevel,
    
    // Qué se muestra en cada nivel
    pub show_shape: bool,           // La forma base
    pub show_label: bool,           // Nombre
    pub show_icon: bool,            // Icono representativo
    pub show_connections: bool,     // Líneas de conexión
    pub show_protocols: bool,       // Protocolos en las líneas
    pub show_internals: bool,       // Sub-componentes
    pub show_metrics: bool,         // Métricas en tiempo real
    pub show_configs: bool,         // Configuraciones
    pub show_code: bool,            // Fragmentos de código
    pub show_logs: bool,            // Logs live
    
    // Estilo según nivel
    pub style_override: Option<StyleId>,
}

impl DetailVisibility {
    pub fn for_level(level: DetailLevel) -> Self {
        match level {
            DetailLevel::Macro => Self {
                level,
                show_shape: true,
                show_label: false,
                show_icon: false,
                show_connections: false,
                show_protocols: false,
                show_internals: false,
                show_metrics: false,
                show_configs: false,
                show_code: false,
                show_logs: false,
                style_override: None,
            },
            DetailLevel::Connections => Self {
                level,
                show_shape: true,
                show_label: true,
                show_icon: true,
                show_connections: true,
                show_protocols: true,
                show_internals: false,
                show_metrics: false,
                show_configs: false,
                show_code: false,
                show_logs: false,
                style_override: None,
            },
            DetailLevel::Internals => Self {
                level,
                show_shape: true,
                show_label: true,
                show_icon: true,
                show_connections: true,
                show_protocols: true,
                show_internals: true,
                show_metrics: false,
                show_configs: false,
                show_code: false,
                show_logs: false,
                style_override: None,
            },
            DetailLevel::Detail => Self {
                level,
                show_shape: true,
                show_label: true,
                show_icon: true,
                show_connections: true,
                show_protocols: true,
                show_internals: true,
                show_metrics: true,
                show_configs: true,
                show_code: false,
                show_logs: false,
                style_override: None,
            },
            DetailLevel::Code => Self {
                level,
                show_shape: true,
                show_label: true,
                show_icon: true,
                show_connections: true,
                show_protocols: true,
                show_internals: true,
                show_metrics: true,
                show_configs: true,
                show_code: true,
                show_logs: true,
                style_override: None,
            },
        }
    }
}

/// Entidad con sub-componentes (para niveles Internals+)
#[derive(Component)]
pub struct HierarchicalChildren {
    pub parent_id: EntityId,
    pub children: Vec<EntityId>,
    pub expanded: bool,
    pub expansion_progress: f64,  // 0.0 = collapsed, 1.0 = expanded
}
```

### 13.11.3 Sistema de Expansión de Árbol

```rust
/// Estado global de profundidad
pub struct DetailLevelState {
    pub current_level: DetailLevel,
    pub target_level: DetailLevel,
    pub transition_progress: f64,
    pub transition_start_time: DateTime<Utc>,
    
    // Para cada entidad: qué nivel reveló qué
    pub entity_reveals: HashMap<EntityId, EntityRevealState>,
}

pub struct EntityRevealState {
    pub entity_id: EntityId,
    pub reveals: Vec<DetailReveal>,  // Qué se reveló en cada nivel
    pub current_reveal: usize,        // Cuál está activo
}

pub struct DetailReveal {
    pub level: DetailLevel,
    pub element: RevealElement,
    pub position: Vec2,       // Dónde aparece
    pub animation: AnimationId,
}

pub enum RevealElement {
    Label { text: String },
    Icon { icon: IconType },
    Connection { target: EntityId, protocol: String },
    Internal { child_id: EntityId },
    Metric { name: String, value: String, unit: String },
    Config { key: String, value: String },
    Code { snippet: String, language: String },
    Log { message: String, level: LogLevel },
}

impl DetailLevelState {
    /// Cambiar nivel de detalle (con transición suave)
    pub fn set_level(&mut self, new_level: DetailLevel) {
        self.target_level = new_level;
        self.transition_progress = 0.0;
        self.transition_start_time = Utc::now();
        
        // Calcular qué entidades necesitan actualizar sus reveals
        for (entity_id, reveal_state) in &mut self.entity_reveals {
            let target_reveal = reveal_state.reveals
                .iter()
                .position(|r| r.level == new_level);
            
            if let Some(idx) = target_reveal {
                reveal_state.current_reveal = idx;
            }
        }
    }
    
    /// Obtener visibilidad actual para una entidad
    pub fn get_visibility(&self, entity_id: EntityId) -> DetailVisibility {
        // El estado efectivo es una interpolación entre levels
        let effective = self.effective_level();
        DetailVisibility::for_level(effective)
    }
    
    fn effective_level(&self) -> DetailLevel {
        // Interpolación suave entre current y target
        let t = self.transition_progress;
        if t >= 1.0 {
            self.target_level
        } else {
            // Mezcla de niveles durante transición
            let curr = self.current_level as u8 as f32;
            let tgt = self.target_level as u8 as f32;
            let mixed = curr + (tgt - curr) * t as f32;
            
            if mixed < 0.5 { DetailLevel::Macro }
            else if mixed < 1.5 { DetailLevel::Connections }
            else if mixed < 2.5 { DetailLevel::Internals }
            else if mixed < 3.5 { DetailLevel::Detail }
            else { DetailLevel::Code }
        }
    }
}
```

### 13.11.4 Renderizado por Nivel de Detalle

```rust
/// Sistema de renderizado por nivel de detalle
pub struct DetailLevelRenderer;

impl System for DetailLevelRenderer {
    fn render(
        &self,
        world: &World,
        state: &DetailLevelState,
        ctx: &mut RenderContext,
    ) {
        let visibility = state.get_visibility(EntityId::root());
        
        // Iterar entidades y renderizar según visibilidad
        for (entity_id, transform, visual, c4) in world.query::<(
            &Transform, &Visual, Option<&DetailVisibility>
        )>() {
            let entity_visibility = c4
                .map(|v| state.get_visibility(entity_id))
                .unwrap_or_else(|| DetailVisibility::for_level(state.effective_level()));
            
            // Renderizar forma base siempre visible
            if entity_visibility.show_shape {
                self.render_shape(entity_id, transform, visual, ctx);
            }
            
            // Label
            if entity_visibility.show_label {
                self.render_label(entity_id, transform, ctx);
            }
            
            // Icono
            if entity_visibility.show_icon {
                self.render_icon(entity_id, transform, ctx);
            }
        }
        
        // Renderizar conexiones
        if visibility.show_connections {
            self.render_connections(world, ctx);
        }
        
        // Renderizar protocolos en conexiones
        if visibility.show_protocols {
            self.render_connection_protocols(world, ctx);
        }
        
        // Renderizar internos (sub-componentes)
        if visibility.show_internals {
            self.render_internals(world, ctx);
        }
        
        // Renderizar métricas
        if visibility.show_metrics {
            self.render_metrics(world, ctx);
        }
        
        // Renderizar configs
        if visibility.show_configs {
            self.render_configs(world, ctx);
        }
        
        // Renderizar código
        if visibility.show_code {
            self.render_code_snippets(world, ctx);
        }
        
        // Renderizar logs
        if visibility.show_logs {
            self.render_logs(world, ctx);
        }
    }
    
    fn render_internals(
        &self,
        world: &World,
        ctx: &mut RenderContext,
    ) {
        // Buscar entidades con sub-componentes
        for (_, children) in world.query::<&HierarchicalChildren>() {
            if children.expanded {
                // Renderizar cada hijo
                for &child_id in &children.children {
                    if let Some((transform, visual)) = world.get::<(Transform, Visual)>(child_id) {
                        // Posición relativa al padre
                        let world_pos = transform.position;
                        ctx.draw_shape(visual.clone(), world_pos);
                    }
                }
                
                // Renderizar líneas de conexión padre → hijos
                self.render_hierarchy_lines(world, children.parent_id, &children.children, ctx);
            }
        }
    }
    
    fn render_metrics(
        &self,
        world: &World,
        ctx: &mut RenderContext,
    ) {
        for (entity_id, metrics) in world.query::<&EntityMetrics>() {
            let pos = world.get::<Transform>(entity_id)
                .map(|t| t.position + Vec2::new(0.0, -20.0))
                .unwrap_or_default();
            
            // Badge con métricas
            let badge_rect = Rect::new(pos.x - 40, pos.y - 25, 80, 20);
            ctx.draw_rect(badge_rect, Style::metric_badge());
            
            // Texto de métrica principal
            ctx.draw_text(
                &format!("{} {}", metrics.value, metrics.unit),
                Vec2::new(pos.x - 35, pos.y - 12),
                Color::WHITE,
            );
        }
    }
}
```

### 13.11.5 Animaciones de Transición

```rust
/// Animaciones suaves entre niveles de detalle
pub struct DetailTransitionAnimation {
    pub from_level: DetailLevel,
    pub to_level: DetailLevel,
    pub progress: f64,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
}

impl DetailTransitionAnimation {
    pub fn new(from: DetailLevel, to: DetailLevel) -> Self {
        Self {
            from_level: from,
            to_level: to,
            progress: 0.0,
            start_time: Utc::now(),
            duration: Duration::milliseconds(300),
        }
    }
    
    pub fn update(&mut self, now: DateTime<Utc>) {
        let elapsed = (now - self.start_time).num_milliseconds() as f64;
        self.progress = (elapsed / self.duration.num_milliseconds() as f64).clamp(0.0, 1.0);
    }
    
    /// Factor de interpolación para elementos appearing
    pub fn ease_in_out(&self) -> f64 {
        if self.progress < 0.5 {
            2.0 * self.progress * self.progress
        } else {
            1.0 - (-2.0 * self.progress + 2.0).powi(2) / 2.0
        }
    }
    
    /// Factor para elementos disappearing
    pub fn ease_out(&self) -> f64 {
        1.0 - (1.0 - self.progress).powi(2)
    }
}

/// Aplicar animación a opacidad
fn animate_opacity(current: f64, target: f64, animation: &DetailTransitionAnimation) -> f64 {
    let t = animation.ease_in_out();
    current + (target - current) * t
}

/// Aplicar animación a escala
fn animate_scale(current: Vec2, target: Vec2, animation: &DetailTransitionAnimation) -> Vec2 {
    let t = animation.ease_in_out();
    current.lerp(target, t as f32)
}
```

### 13.11.6 UI de Control de Nivel

```rust
/// Slider de nivel de detalle
pub struct DetailLevelSlider {
    pub position: Vec2,
    pub size: Vec2,
    pub levels: Vec<DetailLevel>,
    pub current: DetailLevel,
}

impl DetailLevelSlider {
    pub fn render(&self, ctx: &mut RenderContext) {
        // Track
        ctx.draw_rect(
            Rect::new(self.position.x, self.position.y, self.size.x, 4),
            Style::slider_track(),
        );
        
        // Niveles como ticks
        let step = self.size.x / (self.levels.len() - 1) as f32;
        for (i, level) in self.levels.iter().enumerate() {
            let x = self.position.x + i as f32 * step;
            
            // Tick
            let tick_rect = Rect::new(x - 6, self.position.y - 8, 12, 20);
            let is_active = *level == self.current;
            
            ctx.draw_rect(tick_rect, if is_active {
                Style::slider_active()
            } else {
                Style::slider_inactive()
            });
            
            // Label
            ctx.draw_text(
                level.name(),
                Vec2::new(x - 20, self.position.y + 15),
                if is_active { Color::WHITE } else { Color::GRAY_400 },
            );
        }
        
        // Thumb
        let thumb_x = self.position.x + 
            (self.current as u8 as f32) * step;
        ctx.draw_circle(
            Vec2::new(thumb_x, self.position.y + 2),
            10.0,
            Style::slider_thumb(),
        );
    }
}

/// Leyenda contextual según nivel
pub struct DetailLevelLegend {
    pub level: DetailLevel,
}

impl DetailLevelLegend {
    pub fn render(&self, ctx: &mut RenderContext) {
        let items = match self.level {
            DetailLevel::Macro => vec![
                ("Shapes", "Rectángulos y formas base"),
            ],
            DetailLevel::Connections => vec![
                ("Shapes", "Rectángulos y formas"),
                ("Labels", "Nombres de componentes"),
                ("Connections", "Líneas de conexión"),
                ("Protocols", "HTTP, gRPC, WebSocket"),
            ],
            DetailLevel::Internals => vec![
                ("All Previous", "Todo lo anterior"),
                ("Internals", "Sub-componentes"),
                ("Hierarchy", "Líneas padre-hijo"),
            ],
            DetailLevel::Detail => vec![
                ("All Previous", "Todo lo anterior"),
                ("Metrics", "CPU, RAM, requests/sec"),
                ("Configs", "Timeouts, límites, flags"),
            ],
            DetailLevel::Code => vec![
                ("All Previous", "Todo lo anterior"),
                ("Code Snippets", "Fragments de código"),
                ("Logs", "Logs en tiempo real"),
            ],
        };
        
        // Renderizar panel de leyenda
        let mut y = 50.0;
        for (name, desc) in items {
            ctx.draw_text(name, Vec2::new(20.0, y), Color::WHITE);
            ctx.draw_text(desc, Vec2::new(120.0, y), Color::GRAY_400);
            y += 24.0;
        }
    }
}
```

### 13.11.7 Resumen del Sistema de Detalle Incremental

| Nivel | Qué se ve | Ejemplo |
|-------|-----------|---------|
| **Macro** | Solo formas | `┌─────────┐` |
| **+Label** | Nombres | `┌ Service ┐` |
| **+Icon** | Iconos | `[icon] Service` |
| **+Connections** | Líneas | `Service ─── DB` |
| **+Protocols** | Protocolos | `Service ══HTTP══ DB` |
| **+Internals** | Sub-componentes | `Service: [Auth│API│Logger]` |
| **+Metrics** | KPIs live | `CPU: 50% RAM: 512MB` |
| **+Configs** | Parámetros | `timeout: 30s retries: 3` |
| **+Code** | Snippets | `fn handle() { ... }` |
| **+Logs** | Trazas live | `[INFO] Request processed` |

`✶ Insight ─────────────────────────────────────`
**Zoom de Detalle vs C4:**
- **C4:** Cada nivel es una página diferente (Context → Container → Component → Code)
- **Zoom de Detalle:** Es una página que se vuelve más densa (más capas de información sobre los mismos elementos)

El zoom de detalle es como Google Maps: de repente aparecen los nombres de las calles, luego los edificios, luego las tiendas...
`─────────────────────────────────────────────────`

---

## 14. Conclusiones y Recomendaciones Finales

### Puntos Clave

1. **Event Sourcing + Git:** El documento es un repositorio Git. Esto permite undo/redo infinito, time-travel, branching y colaboración en tiempo real.

2. **ECS + Store Delta:** Separar estado (ECS runtime) de serialización (Event Journal) es el patrón correcto para editores colaborativos.

3. **Primitivas Extensibles:** El patrón ShapeUtil de tldraw debe adaptarse para Rust, permitiendo que terceros crates añadan formas nuevas.

4. **⚠️ CORRECCIÓN: Rough Rendering Simplificado:** NO implementar toda la lógica de RoughJS (hachure fill es muy complejo). Usar solo:
   - Doble trazo con jitter en vértices
   - Opcional: SVG filters como alternativa más simple
   - Rough es **decoración**, no core del MVP

5. **⚠️ CORRECCIÓN: NO crear ECS propio:** Usar `bevy_ecs` directamente. Reinventar un ECS consume meses de debugging innecesario.

6. **⚠️ CORRECCIÓN: Geometría vs Rendering:** `kurbo` es para cálculos (intersecciones, Bézier), NO para teselar. Para Canvas 2D, el navegador ya tesela. `lyon` solo es necesario para WebGPU.

7. **⚠️ CORRECCIÓN: R-Tree NO es fuente de verdad:** El ECS es la fuente. El R-Tree es solo un índice que se sincroniza con `Changed<Transform>`.

8. **Spatial Indexing:** R-Tree con `rstar` es la opción más robusta para query performance. Combinar con layer-based filtering.

9. **Renderer Strategy:** Canvas 2D para MVP, WebGPU como upgrade futuro. La API debe abstraer el backend.

10. **Animaciones:** Sistema basado en keyframes con easing functions configurables. Partículas para flujos activos en conectores.

11. **Vector Clocks:** Para colaboración en tiempo real, necesitamos tracking causal de eventos.

12. **Recursos Externos:** Sistema de capas (Canvas + DOM overlay) permite imágenes, videos y HTML embebido sin perder rendimiento.

13. **Zoom de Detalle Incremental:** Árbol de profundidad donde los elementos nunca desaparecen, solo revelan más información (como Google Maps).

### Diferenciadores de ArchFlow

1. **Rust + WASM:** Rendimiento nativo en el navegador
2. **Event Sourcing:** True Git-like document model
3. **Recursos Híbridos:** Imágenes/videos/HTML embebidos con caché LRU
4. **Zoom de Detalle:** Los elementos permanecen y revelan más info progresivamente
5. **Architecture-Aware:** Semántica C4 integrada (no solo formas)
6. **IaC Preview:** Preview de Terraform/K8s mientras diseñas

### Recomendación Final (post-crítica)

**MVP "Architect" Priorities:**
1. ✅ Rendering Exacto (Rect, Ellipse, Text, Paths SVG básicos)
2. ✅ **Conexiones Inteligentes** (Orthogonal + Bézier routing) - **verdadero diferenciador**
3. ✅ Jerarquía (Groups y contenedores)
4. ⚠️ Rough Rendering - **bajar prioridad a Fase 3 o eliminar**

**PIVOTE PRINCIPAL:** De "motor de juego desde cero" a "integrar componentes maduros (bevy_ecs, kurbo) para diagramación profesional".

---

## 15. Referencias

### Documentos del Proyecto
- `docs/ARCHFLOW-EXPANDED-MVP-PROPOSAL.md`
- `docs/ENGINE-PRIMITIVES-SPEC.md`

### Recursos Externos
- tldraw.dev/docs/shapes
- tldraw.dev/examples/custom-embed
- c4model.com - C4 Model por Simon Brown
- roughjs.com
- github.com/nical/lyon
- docs.rs/rstar/latest/rstar/
- docs.rs/kurbo/latest/kurbo/
- github.com/SanderMertens/ecs-faq (ECS patterns)

---

**Documento preparado por:** ArchFlow Research  
**Última actualización:** 2026-01-23  
**Versión:** 1.4 (corregido: Zoom de Detalle Incremental取代 C4)  
**Versión:** 1.3 (añadida sección Navegación C4 y Drill-down Semántico)  
**Versión:** 1.2 (añadida sección de Recursos Externos)  
**Versión:** 1.1 (añadida sección Event Sourcing)

---

## 14. Conclusiones y Recomendaciones Finales

### Puntos Clave

1. **Event Sourcing + Git:** El documento es un repositorio Git. Esto permite undo/redo infinito, time-travel, branching y colaboración en tiempo real.

2. **ECS + Store Delta:** Separar estado (ECS runtime) de serialización (Event Journal) es el patrón correcto para editores colaborativos.

3. **Primitivas Extensibles:** El patrón ShapeUtil de tldraw debe adaptarse para Rust, permitiendo que terceros crates añadan formas nuevas.

4. **Hand-Drawn Rendering:** Implementar algoritmos de RoughJS en Rust es viable y diferenciador. La clave es el `SeededRng` para reproducibilidad.

5. **Spatial Indexing:** R-Tree con `rstar` es la opción más robusta para query performance. Combinar con layer-based filtering.

6. **Renderer Strategy:** Canvas 2D para MVP, WebGPU como upgrade futuro. La API debe abstraer el backend.

7. **Animaciones:** Sistema basado en keyframes con easing functions configurables. Partículas para flujos activos en conectores.

8. **Vector Clocks:** Para colaboración en tiempo real, necesitamos tracking causal de eventos.

9. **Recursos Externos:** Sistema de capas (Canvas + DOM overlay) permite imágenes, videos y HTML embebido sin perder rendimiento.

10. **Navegación C4:** Drill-down semántico permite explorar arquitectura desde Context hasta Code con animaciones smooth.

### Diferenciadores de ArchFlow

1. **Rust + WASM:** Rendimiento nativo en el navegador
2. **Hand-Drawn Built-in:** No como plugin, sino como primitiva de primera clase
3. **Event Sourcing:** True Git-like document model
4. **Recursos Híbridos:** Imágenes/videos/HTML embebidos con caché LRU
5. **Navegación C4:** Drill-down semántico desde Context hasta Code
6. **Architecture-Aware:** Semántica C4 integrada (no solo formas)
7. **IaC Preview:** Preview de Terraform/K8s mientras diseñas

---

## 15. Referencias

### Documentos del Proyecto
- `docs/ARCHFLOW-EXPANDED-MVP-PROPOSAL.md`
- `docs/ENGINE-PRIMITIVES-SPEC.md`

### Recursos Externos
- tldraw.dev/docs/shapes
- tldraw.dev/examples/custom-embed
- c4model.com - C4 Model por Simon Brown
- roughjs.com
- github.com/nical/lyon
- docs.rs/rstar/latest/rstar/
- docs.rs/kurbo/latest/kurbo/
- github.com/SanderMertens/ecs-faq (ECS patterns)

---

**Documento preparado por:** ArchFlow Research  
**Última actualización:** 2026-01-23  
**Versión:** 1.3 (añadida sección Navegación C4 y Drill-down Semántico)  
**Versión:** 1.2 (añadida sección de Recursos Externos)  
**Versión:** 1.1 (añadida sección Event Sourcing)
