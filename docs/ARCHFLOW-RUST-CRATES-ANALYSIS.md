# ArchFlow: Análisis de Crates Rust Reutilizables

**Fecha:** 2026-01-23
**Versión:** 1.0
**Estado:** Análisis de Dependencias

---

## 1. Executive Summary

Analizamos el ecosistema de Rust para encontrar crates probados y maduros que podamos reutilizar en ArchFlow. Encontramos 3 categorías principales:

1. **Geometría 2D**: Para posición, dimensiones, colisiones, hit testing
2. **Estructuras de Grafo**: Para representar conexiones y dependencias
3. **Graphics/Rendering**: Para Canvas2D y WebGPU rendering

### Crates Recomendados (Prioridad Alta)

| Crate | Propósito | Descargas | Uso en ArchFlow |
|-------|-----------|-----------|-----------------|
| **euclid** | Geometry primitives | 1.1M | Posiciones, rectángulos |
| **kurbo** | 2D curves (Bézier) | 958K | Conexiones curvas |
| **parry2d** | Collision detection | 38K | Hit testing, intersecciones |
| **petgraph** | Graph data structure | 12.9M | Conexiones DAG |
| **wgpu** | WebGPU bindings | - | Rendering (post-MVP) |

---

## 2. Crates de Geometría y Matemáticas

### 2.1 euclid - Geometry Primitives

**Fuente:** https://lib.rs/crates/euclid
**Descargas:** 1.1M (muy popular)
**Licencia:** MIT/Apache-2.0

#### Características
```rust
use euclid::{Point2D, Rect, Size2D, Vector2D, Angle};

// Tipos de datos principales
Point2D<f32>    // Posición en 2D
Vector2D<f32>  // Dirección/magnitud
Rect<f32>        // Rectángulo (para bounding boxes)
Size2D<f32>      // Dimensiones
Angle<f32>        // Ángulos (para rotaciones)
```

#### Ventajas para ArchFlow
1. **Tipos inmutables por defecto** - Perfecto para el core
2. **Operaciones matemáticas integradas**: `rect.translate()`, `rect.intersection()`
3. **Soporte para f32/f64** - Elige precisión según necesidad
4. **Serialización con `serde`** - Compatible con nuestro formato AUF
5. **Cero dependencias externas** - Ligero y rápido

#### Código de Ejemplo

```rust
use euclid::{Point2D, Rect, Size2D};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub position: Point2D<f64>,  // World position
    pub dimensions: Size2D<f64>,
}

impl Component {
    pub fn bounding_rect(&self) -> Rect<f64> {
        Rect::new(
            self.position,
            self.dimensions,
        )
    }

    pub fn intersects(&self, other: &Component) -> bool {
        self.bounding_rect().intersects(&other.bounding_rect())
    }

    pub fn contains_point(&self, point: &Point2D<f64>) -> bool {
        self.bounding_rect().contains(point)
    }
}
```

#### Operaciones Necesarias para ArchFlow
```rust
// Viewport culling (Excalidraw pattern)
fn is_visible(viewport: &Rect<f64>, component: &Component) -> bool {
    viewport.intersects(&component.bounding_rect())
}

// Hit testing
fn hit_test(point: &Point2D<f64>, components: &[Component]) -> Option<&Component> {
    components
        .iter()
        .find(|comp| comp.contains_point(point))
}

// Snap to grid
fn snap_to_grid(point: &Point2D<f64>, grid_size: f64) -> Point2D<f64> {
    let x = (point.x / grid_size).round() * grid_size;
    let y = (point.y / grid_size).round() * grid_size;
    Point2D::new(x, y)
}
```

**Recomendación:** ✅ USAR euclid para todas las operaciones de geometría 2D.

---

### 2.2 kurbo - 2D Curves Library

**Fuente:** https://lib.rs/crates/kurbo
**Descargas:** 958K
**Licencia:** MIT/Apache-2.0

#### Características
```rust
use kurbo::{BezierPath, StrokeParams, BezPath, Shape};

// Tipos principales
BezierPath     // Paths con Bézier curves
StrokeParams    // Parámetros de stroke (ancho, joins, caps)
BezPath       // Path builder
Shape          // Abstract shape (para hit testing)
```

#### Ventajas para ArchFlow
1. **Renderizado de conexiones**: Bézier curves smoothstep/straight
2. **Stroke con estilos**: Líneas sólidas, dashed, animadas
3. **Bounding boxes**: `Shape::bounding_box()` para culling
4. **Hit testing**: `Shape::winding()` para detectar clicks en curvas

#### Código de Ejemplo - Conexiones AWS

```rust
use kurbo::{BezPath, StrokeParams, Point, Vec2};

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub source: ComponentId,
    pub target: ComponentId,
    pub connection_type: ConnectionType,  // Straight, Bezier, Step
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Straight,
    Bezier { curvature: f64 },
    Step { offset: f64 },
}

impl Connection {
    pub fn to_bez_path(&self, start: Point, end: Point) -> BezPath {
        match self.connection_type {
            ConnectionType::Straight => {
                let mut path = BezPath::new();
                path.move_to(start);
                path.line_to(end);
                path
            }
            ConnectionType::Bezier { curvature } => {
                let control = self.control_point(start, end, curvature);
                let mut path = BezPath::new();
                path.move_to(start);
                path.quad_to(control, end);
                path
            }
            ConnectionType::Step { offset } => {
                let mid = self.step_midpoint(start, end, offset);
                let mut path = BezPath::new();
                path.move_to(start);
                path.line_to(mid);
                path.line_to(end);
                path
            }
        }
    }

    pub fn stroke(&self, path: &BezPath, stroke: &StrokeParams) -> BezPath {
        path.stroke(stroke)
    }

    fn control_point(&self, start: Point, end: Point, curvature: f64) -> Point {
        let mid = (start + end.to_vec2()) * 0.5;
        let normal = (end - start).to_vec2().perp();
        mid + normal * curvature
    }

    fn step_midpoint(&self, start: Point, end: Point, offset: f64) -> Point {
        let mid = (start + end.to_vec2()) * 0.5;
        Point::new(mid.x, start.y + offset)
    }
}
```

#### Parámetros de Stroke (para diferentes estilos)

```rust
use kurbo::StrokeParams;

// Conexión sólida
let solid = StrokeParams::default();

// Conexión dashed
let dashed = StrokeParams {
    dash_pattern: DashPattern::Dashed,
    dash_offset: 0.0,
    ..Default::default()
};

// Conexión animada (flujo de datos)
let animated = StrokeParams {
    dash_pattern: DashPattern::Animated,
    dash_offset: time_offset,
    ..Default::default()
};
```

**Recomendación:** ✅ USAR kurbo para renderizar conexiones con Bézier curves y strokes.

---

### 2.3 parry2d - Collision Detection

**Fuente:** https://lib.rs/crates/parry2d
**Descargas:** 38K
**Licencia:** Apache-2.0

#### Características
```rust
use parry2d::{naive, shape::Cuboid, math::Point};

// Hit testing rápido
let point = Point::new(x, y);
let cuboid = Cuboid::new(half_extents);
let contains = cuboid.contains_local_point(&point);

// Intersección entre rectángulos
let intersects = cuboid1.intersects(&cuboid2);
```

#### Ventajas para ArchFlow
1. **Algoritmos optimizados**: GJK (Gilbert-Johnson-Keerthi) para colisiones
2. **Bounding boxes**: Para viewport culling rápido
3. **Spatial queries**: `intersects`, `contains`, `distance`
4. **Soporte para formas complejas**: No solo rectángulos, sino también polígonos

#### Código de Ejemplo - Hit Testing Avanzado

```rust
use parry2d::{naive::QueryDispatcher, shape::Cuboid, math::Point};
use std::collections::HashMap;

pub struct HitTester {
    components: HashMap<ComponentId, Cuboid>,
}

impl HitTester {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, id: ComponentId, rect: Rect<f64>) {
        let half_extents = rect.size() * 0.5;
        let cuboid = Cuboid::new(half_extents);
        self.components.insert(id, cuboid);
    }

    pub fn hit_test(&self, point: Point<f64>) -> Option<ComponentId> {
        let p = Point::new(point.x, point.y);

        // Buscar en orden inverso (Z-order: componentes dibujados después al frente)
        for (id, cuboid) in self.components.iter().rev() {
            if cuboid.contains_local_point(&p) {
                return Some(id.clone());
            }
        }

        None
    }

    pub fn intersects(&self, a: ComponentId, b: ComponentId) -> bool {
        let (c_a, c_b) = self.components.get_many(&[a, b]);
        match (c_a, c_b) {
            (Some(a), Some(b)) => a.intersects(b),
            _ => false,
        }
    }
}
```

**Recomendación:** ✅ USAR parry2d para hit testing y colisiones avanzadas (opcional para MVP).

---

### 2.4 Otros Crates de Geometría (Opcionales)

| Crate | Uso | Prioridad |
|-------|-----|-----------|
| **planar_geo** | Algoritmos geométricos avanzados (Delaunay, Voronoi) | Baja (post-MVP) |
| **robust** | Predicados geométricos robustos (floating-point) | Baja |
| **pathfinder_geometry** | SIMD-accelerated geometry | Baja (optimización) |

---

## 3. Crates de Estructuras de Grafo

### 3.1 petgraph - Graph Data Structure Library

**Fuente:** https://lib.rs/crates/petgraph
**Descargas:** 12.9M (EL ESTÁNDAR)
**Licencia:** MIT/Apache-2.0

#### Características
```rust
use petgraph::{Graph, Directed, NodeIndex};

// Grafo dirigido con pesos en edges
type FlowGraph = Graph<ComponentId, ConnectionId, Directed>;

// Algoritmos disponibles
use petgraph::algo::{
    dijkstra,      // Shortest path
    topos,          // Topological sort (DAG validation)
    kosaraju_sharir,  // Cycle detection
    is_cyclic_undirected, // Cycle detection
};
```

#### Ventajas para ArchFlow
1. **Validación de DAG**: `is_cyclic_directed()` para detectar ciclos en infra
2. **Topological sort**: `toposort()` para ordenar despliegue
3. **Algoritmos de pathfinding**: `dijkstra()` para encontrar rutas entre componentes
4. **Vecinos de entrada/salida**: `neighbors()` para descubrir conexiones

#### Código de Ejemplo - Validación de Infraestructura

```rust
use petgraph::{Graph, Directed, Direction};
use petgraph::algo::{is_cyclic_directed, toposort, DfsSpace};
use petgraph::visit::Dfs;

#[derive(Debug, Clone)]
pub struct Component {
    pub id: ComponentId,
    pub resource_type: CloudResourceType,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub source: ComponentId,
    pub target: ComponentId,
}

pub struct InfrastructureGraph {
    graph: Graph<Component, Connection, Directed>,
}

impl InfrastructureGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    pub fn add_component(&mut self, component: Component) -> NodeIndex {
        self.graph.add_node(component)
    }

    pub fn add_connection(&mut self, connection: Connection) {
        let source_idx = self.graph
            .node_indices()
            .find(|idx| self.graph[idx].id == connection.source);

        let target_idx = self.graph
            .node_indices()
            .find(|idx| self.graph[idx].id == connection.target);

        if let (Some(src), Some(tgt)) = (source_idx, target_idx) {
            self.graph.add_edge(src, tgt, connection);
        }
    }

    /// Detectar ciclos (error en infraestructura AWS)
    pub fn has_cycles(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Ordenar componentes para deployment (dependencias primero)
    pub fn deployment_order(&self) -> Result<Vec<Component>, CyclicError> {
        match toposort(&self.graph, None) {
            Ok(order) => {
                let components = order
                    .iter()
                    .map(|idx| self.graph[*idx].clone())
                    .collect();
                Ok(components)
            }
            Err(cycle) => Err(CyclicError::Cycle(cycle)),
        }
    }

    /// Encontrar componentes downstream de uno dado
    pub fn downstream_components(&self, component_id: ComponentId) -> Vec<Component> {
        let idx = self.graph
            .node_indices()
            .find(|i| self.graph[*i].id == component_id);

        idx.map_or(Vec::new(), |i| {
            self.graph
                .neighbors_directed(i)
                .map(|neighbor| self.graph[neighbor].clone())
                .collect()
        })
    }

    /// Calcular número de dependencias (para validación de complejidad)
    pub fn dependency_count(&self, component_id: ComponentId) -> usize {
        let idx = self.graph
            .node_indices()
            .find(|i| self.graph[*i].id == component_id);

        idx.map_or(0, |i| self.graph.edges_directed(i).count())
    }
}

#[derive(Debug)]
pub enum CyclicError {
    Cycle(Vec<Component>),
}
```

#### Validación de Terraform Deployment Order

```rust
impl InfrastructureGraph {
    pub fn terraform_deployment_plan(&self) -> Result<DeploymentPlan, GraphError> {
        // 1. Validar que es un DAG (sin ciclos)
        if self.has_cycles() {
            return Err(GraphError::CyclicDependency);
        }

        // 2. Obtener orden topológico
        let order = self.deployment_order()?;

        // 3. Agrupar por niveles de dependencia
        let mut levels: Vec<Vec<Component>> = Vec::new();
        let mut current_level: Vec<Component> = Vec::new();
        let mut processed: HashSet<ComponentId> = HashSet::new();

        for component in order {
            let deps = self.get_dependencies(&component.id);

            // Solo añadir al nivel actual si todas las dependencias están procesadas
            if deps.iter().all(|dep| processed.contains(dep)) {
                current_level.push(component);
                processed.insert(component.id);
            } else {
                // Cambiar de nivel
                if !current_level.is_empty() {
                    levels.push(std::mem::take(&mut current_level));
                }
                current_level.push(component);
                processed.insert(component.id);
            }
        }

        if !current_level.is_empty() {
            levels.push(current_level);
        }

        Ok(DeploymentPlan { levels })
    }
}
```

**Recomendación:** ✅ USAR petgraph para validar DAG y calcular deployment order.

---

### 3.2 daggy - DAG Data Structure

**Fuente:** https://lib.rs/crates/daggy
**Descargas:** 33K
**Licencia:** MIT

#### Características
```rust
use daggy::Dag;

// DAG simple y directo
type DependencyDag = Dag<ComponentId>;

// Operaciones
dag.add_node(component_id);
dag.add_dependency(parent_id, child_id);
dag.topological_sort()?;  // Retorna error si hay ciclo
```

#### Ventajas para ArchFlow
1. **API más simple** que petgraph (para casos básicos)
2. **Detecta ciclos automáticamente** al intentar añadir dependencias
3. **Topological sort** incluido

**Comparación: petgraph vs daggy**

| Aspecto | petgraph | daggy |
|----------|----------|--------|
| **Complejidad** | Alta (más flexible) | Baja (DAG-only) |
| **Algoritmos** | 20+ algoritmos | Solo topological sort |
| **Uso en ArchFlow** | DAG validation + pathfinding | Solo DAG validation |

**Recomendación:** Si solo necesitas validación DAG, usa **daggy**. Si necesitas algoritmos adicionales, usa **petgraph**.

---

## 4. Crates de Graphics/Rendering

### 4.1 wgpu - WebGPU Bindings

**Fuente:** https://lib.rs/crates/wgpu
**Descargas:** ~ (sin contar)
**Licencia:** MIT/Apache-2.0

#### Características
```rust
use wgpu::{Instance, Device, Queue, Surface, ShaderModule};

// Setup básico
let instance = Instance::new(wgpu::Backends::all());
let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default());
let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
    label: Some("ArchFlow Renderer"),
    features: wgpu::Features::default(),
    limits: wgpu::Limits::default(),
});

// Rendering pipeline
let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: Some("ArchFlow Shader"),
    source: include_str!("shader.wgsl"),
});
```

#### Ventajas para ArchFlow (Post-MVP)
1. **GPU acceleration** para 10k+ componentes
2. **Instanced rendering**: Dibujar miles de componentes con un solo draw call
3. **Compute shaders**: Para cálculos geométricos en GPU (intersecciones, etc.)

#### Código de Ejemplo - Instanced Rendering

```rust
struct InstanceData {
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    selected: f32,  // 0.0 o 1.0
}

pub struct WebGPURenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    instance_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
}

impl WebGPURenderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        // Instance buffer (para 10k+ componentes)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of::<InstanceData>() as u64 * MAX_COMPONENTS,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            instance_buffer,
            render_pipeline,
        }
    }

    pub fn render(&mut self, components: &[Component]) {
        let instances: Vec<InstanceData> = components
            .iter()
            .map(|comp| InstanceData {
                position: [comp.position.x as f32, comp.position.y as f32],
                size: [comp.dimensions.width as f32, comp.dimensions.height as f32],
                color: self.color_for_type(&comp.resource_type),
                selected: if comp.selected { 1.0 } else { 0.0 },
            })
            .collect();

        // Subir a GPU
        self.queue.write_buffer(&self.instance_buffer, 0, &instances);

        // Single draw call para todos los componentes del mismo tipo
        // En producción: separar por tipo de recurso (EC2, Lambda, etc.)
    }
}
```

**Recomendación:** ⏳ RESERVAR para post-MVP. Canvas2D es suficiente para MVP (500-1000 componentes).

---

### 4.2 lyon - 2D Vector Graphics Renderer

**Fuente:** https://lib.rs/crates/lyon
**Descargas:** 100K
**Licencia:** MIT/Apache-2.0

#### Características
```rust
use lyon::{tessellation::FillTessellator, path::Path, math::Point};

// Tesselación de paths complejos
let mut tess = FillTessellator::new();
tess.tessellate_path(&Path::builder().with_point(start).with_line(end));
let geometry = tess.build();
```

#### Ventajas para ArchFlow
1. **Tesselación automática**: Paths curvos → triángulos para GPU
2. **Anti-aliasing**: Renderizado suave
3. **Path building**: Builder pattern para crear formas complejas

**Comparación: lyon vs kurbo**

| Aspecto | lyon | kurbo |
|----------|-------|--------|
| **Foco** | Tesselación y GPU rendering | Bézier curves matemáticas |
| **Uso en ArchFlow** | WebGPU rendering (post-MVP) | Conexiones Bézier (MVP) |

**Recomendación:** Usar **kurbo** para MVP (más simple), **lyon** para WebGPU (post-MVP).

---

### 4.3 femtovg - Antialiased 2D Vector Drawing

**Fuente:** https://lib.rs/crates/femtovg
**Descargas:** 72K
**Licencia:** MIT/Apache-2.0

#### Características
```rust
use femtovg::{Paint, PathBuilder, Renderer};

// Canvas-like API
let mut renderer = Renderer::new();
let mut path = PathBuilder::new();

path.move_to(x, y);
path.line_to(x + 100, y);
path.close();

renderer.fill(&mut ctx, &Paint::default(), &path.built());
renderer.stroke(&mut ctx, &Paint::default(), &path.built());
```

#### Ventajas para ArchFlow
1. **API simple** como Canvas 2D
2. **Anti-aliasing** incluido
3. **Backend agnostic**: WebGL, Metal, Vulkan

**Recomendación:** Opcional. Si Canvas 2D API de Web es suficiente, no necesitamos esta librería.

---

## 5. Workspace de Cargo Propuesto

```toml
[workspace]
members = [
    "crates/archflow-core",       # Dominio puro
    "crates/archflow-geometry",   # Wrapper sobre euclid + parry2d
    "crates/archflow-graph",      # Wrapper sobre petgraph
    "crates/archflow-renderer",  # Canvas2D renderer (custom)
    "crates/archflow-wasm",        # WASM bindings
]

[workspace.dependencies]
# Dependencias compartidas
euclid = "0.23"
kurbo = "0.13"
petgraph = "0.8"
parry2d = "0.26"
serde = { version = "1.0", features = ["derive"] }
```

### 5.1 archflow-geometry

```toml
[package]
name = "archflow-geometry"
version = "0.1.0"

[dependencies]
euclid = "0.23"       # Geometry primitives
kurbo = "0.13"        # Bézier curves
parry2d = "0.26"      # Collision detection (opcional)
serde = { version = "1.0", features = ["derive"] }
```

### 5.2 archflow-graph

```toml
[package]
name = "archflow-graph"
version = "0.1.0"

[dependencies]
petgraph = "0.8"       # Graph data structure
thiserror = "1.0"       # Error handling
```

### 5.3 archflow-renderer

```toml
[package]
name = "archflow-renderer"
version = "0.1.0"

[dependencies]
# MVP: Canvas 2D puro (sin librerías externas para renderizado)
wgpu = { version = "0.20", optional = true }  # Post-MVP
euclid = "0.23"       # Reutilizar tipos
kurbo = "0.13"        # Para conexiones

[features]
default = ["canvas2d"]
webgpu = ["wgpu", "dep:archflow-geometry/webgpu"]
```

---

## 6. Roadmap de Adopción de Crates

### Fase 1: MVP (Canvas 2D - 4 semanas)

```
✅ Semana 1-2: Geometría
├── Añadir euclid al workspace
├── Reemplazar tipos propios (Position, Rect) con euclid
├── Implementar hit testing con euclid
└── Tests de geometría básica

✅ Semana 3-4: Grafo DAG
├── Añadir petgraph al workspace
├── Implementar validación de DAG (sin ciclos)
├── Implementar topological sort (deployment order)
└── Tests de grafos

✅ Semana 5-6: Rendering Canvas2D
├── Implementar Canvas 2D renderer
├── Integrar euclid para transformadas
└── Tests de rendering básico
```

### Fase 2: MVP+ (Conexiones Bézier - 2 semanas)

```
⏳ Semana 7-8: Conexiones Avanzadas
├── Añadir kurbo al workspace
├── Implementar tipos de conexión (Straight, Bezier, Step)
├── Renderizar conexiones con kurbo
└── Tests de conexiones curvas
```

### Fase 3: Post-MVP (WebGPU - 6-8 semanas)

```
⏳ Semana 9-16: WebGPU Renderer
├── Añadir wgpu al workspace
├── Implementar Lyon tesselation (opcional)
├── Instanced rendering para 10k+ componentes
└── Performance benchmarks
```

---

## 7. Comparación de Estrategias

### 7.1 Estrategia A: Implementar Todo Propio

**Tiempo estimado:** 12-16 semanas

**Ventajas:**
- Control total sobre el código
- Sin dependencias externas
- Tamaño de bundle mínimo

**Desventajas:**
- Bugs en código geométrico
- Algoritmos no probados
- Mayor esfuerzo de mantenimiento

### 7.2 Estrategia B: Reutilizar Crates Probados (RECOMENDADO)

**Tiempo estimado:** 6-8 semanas

**Ventajas:**
- Código probado y battle-tested
- Bugs ya resueltos por la comunidad
- Actualizaciones y bug fixes automáticas
- Enfoque en dominio de negocio (AWS)

**Desventajas:**
- Dependencias externas (manejables con Cargo)
- Bundle size ligeramente mayor (no significativo para WASM)

### 7.3 Estrategia Híbrida (ÓPTIMO)

**Tiempo estimado:** 4-6 semanas

**Enfoque:**
- Usar euclid para geometría (core)
- Usar kurbo para conexiones (feature-specific)
- Usar petgraph para validación DAG (infra-specific)
- Implementar Canvas 2D propio (rendering específico para ArchFlow)

**Justificación:**
- Rendering tiene requisitos muy específicos (viewport, snapping, selection)
- Geometría y grafos son genéricos y bien resueltos
- Foco en código que diferencia ArchFlow de la competencia

---

## 8. Conclusiones y Recomendaciones

### 8.1 Crates a Usar (Prioridad CRÍTICA)

1. ✅ **euclid** - Para toda la geometría 2D
2. ✅ **kurbo** - Para conexiones Bézier (fase 2)
3. ✅ **petgraph** - Para validación DAG y topological sort
4. ✅ **wgpu** - Para WebGPU rendering (fase 3)

### 8.2 Crates a Evaluar (Opcionales)

5. ⏸ **parry2d** - Hit testing avanzado (si euclid no es suficiente)
6. ⏸ **daggy** - Si petgraph es demasiado complejo (unlikely)
7. ⏸ **lyon** - Para WebGPU tesselation (opcional)

### 8.3 Crates a NO Usar

8. ❌ **Ninguna crate de rendering 2D completo** (femtovg, skia-rs, etc.)
   - Razón: Rendering de ArchFlow es muy específico (viewport culling, snapping, selection handles)
   - Mejor implementar propio con Canvas 2D API

### 8.4 Next Steps Inmediatos

1. Crear workspace de Cargo con los crates propuestos
2. Implementar `archflow-geometry` como wrapper sobre euclid
3. Implementar `archflow-graph` como wrapper sobre petgraph
4. Actualizar documentación de core para usar estos crates

---

## 9. Appendix: Referencias

- **euclid**: https://lib.rs/crates/euclid
- **kurbo**: https://lib.rs/crates/kurbo
- **petgraph**: https://lib.rs/crates/petgraph
- **parry2d**: https://lib.rs/crates/parry2d
- **wgpu**: https://lib.rs/crates/wgpu
- **lyon**: https://lib.rs/crates/lyon
- **femtovg**: https://lib.rs/crates/femtovg

---

## 10. Connascence Analysis: Dependencias Externas

### 10.1 Connascence of Name (Media Severidad)

**Problema:** Dependemos de crates externos introduce nombres de tipos que pueden entrar en conflicto.

**Ejemplo:**
```rust
// euclid
use euclid::Point2D;

// ArchFlow
use archflow_core::domain::Point;  // Conflicto potencial
```

**Solución:** Wrapper types y `pub use` explícito
```rust
// archflow-geometry/src/lib.rs
pub use euclid::{Point2D, Rect, Size2D, Vector2D};

// Renombrar para evitar conflictos
pub use euclid::Point2D as WorldPoint;
pub use euclid::Point2D as ScreenPoint;
```

### 10.2 Connascence of Type (Baja Severidad)

**Problema:** Crates usan diferentes tipos de número (f32 vs f64).

**Solución:** Estandarizar en `f64` para geometría de precisión
```rust
// Configuración de workspace
[workspace.dependencies]
euclid = { version = "0.23", default-features = false }

// Default a f64
euclid = { version = "0.23", default-features = ["f64"] }
```

---

**Fin del documento**
