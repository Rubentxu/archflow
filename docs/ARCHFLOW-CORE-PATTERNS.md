# ArchFlow Core: Análisis de Patrones de React Flow, tldraw y Excalidraw

**Fecha:** 2026-01-23
**Versión:** 1.0
**Estado:** Análisis Técnico

---

## 1. Executive Summary

Tras analizar los repositorios de **React Flow**, **tldraw** y **Excalidraw**, identificamos patrones arquitectónicos comunes que podemos adaptar a Rust para construir el core de ArchFlow.

### Decisiones Arquitectónicas Clave

1. **Core-First Approach**: El dominio (modelos, reglas, validación) debe ser independiente de rendering
2. **Change Pattern System**: Similar a React Flow's `NodeChange`/`EdgeChange`
3. **Renderer Abstraction**: Patrón Strategy para soportar Canvas2D/WebGPU sin cambiar core
4. **Immutable Data**: Los shapes/records son inmutables (tldraw pattern)
5. **Viewport-Based Rendering**: Solo renderizar lo visible (Excalidraw optimization)

---

## 2. Análisis de Herramientas Existentes

### 2.1 React Flow: Node-Based Editor Pattern

**Fuente:** `packages/system/src/types/changes.ts`, `types/nodes.ts`

#### Patrones Clave

**1. Union de Changes (Change Pattern)**
```typescript
export type NodeChange<NodeType extends NodeBase = NodeBase> =
  | NodeDimensionChange
  | NodePositionChange
  | NodeSelectionChange
  | NodeRemoveChange
  | NodeAddChange<NodeType>
  | NodeReplaceChange<NodeType>;

export type EdgeChange<EdgeType extends EdgeBase = EdgeBase> =
  | EdgeSelectionChange
  | EdgeRemoveChange
  | EdgeAddChange<EdgeType>
  | EdgeReplaceChange<EdgeType>;
```

**Ventajas para ArchFlow:**
- Cada operación tiene un tipo específico
- Fácil implementar undo/redo (stack de cambios)
- Batch updates: aplicar múltiples cambios en una transacción

**Adaptación a Rust:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeChange {
    Add { id: NodeId, node: Node },
    Remove { id: NodeId },
    UpdatePosition { id: NodeId, position: Position },
    UpdateDimensions { id: NodeId, dimensions: Dimensions },
    Select { id: NodeId, selected: bool },
    Replace { id: NodeId, node: Node },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeChange {
    Add { edge: Edge },
    Remove { id: EdgeId },
    UpdateSource { id: EdgeId, source: NodeId },
    UpdateTarget { id: EdgeId, target: NodeId },
    Select { id: EdgeId, selected: bool },
}
```

**2. NodeBase con Data Genérico**
```typescript
export type NodeBase<
  NodeData extends Record<string, unknown> = Record<string, unknown>,
  NodeType extends string | undefined = string | undefined
> = {
  id: string;
  type?: NodeType;
  position: XYPosition;
  data: NodeData;
  // ... más propiedades
};
```

**Adaptación a Rust con Componentes AWS:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub resource_type: CloudResourceType,
    pub position: WorldPosition,
    pub dimensions: Dimensions,
    pub data: ResourceData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudResourceType {
    Ec2Instance,
    LambdaFunction,
    S3Bucket,
    RdsInstance,
    Vpc,
    LoadBalancer,
    IamRole,
    CloudfrontDistribution,
    DynamoDbTable,
    Waf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub properties: HashMap<String, PropertyValue>,
    pub tags: Vec<Tag>,
    pub terraform_name: Option<String>,
}
```

**3. InternalNodeBase - Separación de concerns**
```typescript
export type InternalNodeBase<NodeType extends NodeBase = NodeBase> =
  Omit<NodeType, 'measured'> & {
    measured: { width?: number; height?: number; };
    internals: { /* ... */ };
  };
```

**Insight:** Separar datos del usuario (`NodeBase`) de datos internos del renderer (`InternalNodeBase`).

**Adaptación a Rust:**
```rust
// Datos del usuario (serializables a AUF)
pub struct Node {
    pub id: NodeId,
    pub resource_type: CloudResourceType,
    pub position: WorldPosition,
    pub data: ResourceData,
}

// Datos internos del renderer (no serializados)
pub struct RenderableNode {
    pub node: Node,
    pub measured: Option<MeasuredDimensions>,
    pub screen_position: Option<ScreenPosition>,
    pub is_visible: bool,
}
```

---

### 2.2 tldraw: Immutable Records Pattern

**Fuente:** `packages/utils/src/lib/object.ts`, `media.ts`

#### Patrones Clave

**1. Operaciones Inmutables en Objetos**
```typescript
import { deepCopy } from '@tldraw/utils';

export function updateRecord<T extends Record<string, any>>(
  record: T,
  updates: Partial<T>
): T {
  return { ...record, ...updates };
}
```

**Ventajas para Rust:**
- Rust ya tiene ownership y borrowing
- Los tipos son inmutables por defecto
- `#[derive(Clone)]` para copies explícitas

**Adaptación a Rust:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    pub id: ComponentId,
    pub resource_type: CloudResourceType,
    pub position: WorldPosition,
    pub properties: HashMap<String, PropertyValue>,
}

impl Component {
    pub fn with_position(mut self, position: WorldPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_property(mut self, key: &str, value: PropertyValue) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }
}

// Uso:
let component = Component::new()
    .with_position(Position { x: 100.0, y: 200.0 })
    .with_property("instance_type", PropertyValue::String("t3.micro".into()));
```

**2. Utils Separados del Core**
`tldraw` separa utilities (`packages/utils`) del core editor.

**Adaptación a Rust:**
```toml
[workspace]
members = [
    "archflow-core",      # Dominio puro (sin WASM)
    "archflow-utils",     # Geometría, collision detection
    "archflow-renderer",  # Rendering abstraction
    "archflow-wasm",     # WASM bindings
]
```

---

### 2.3 Excalidraw: Viewport Culling & Double Canvas

**Fuente:** `packages/excalidraw/scene/Renderer.ts`, `types.ts`

#### Patrones Clave

**1. getVisibleCanvasElements - Viewport Culling**
```typescript
const getVisibleCanvasElements = ({
  elementsMap,
  zoom,
  offsetLeft,
  offsetTop,
  scrollX,
  scrollY,
  // ...
}: {
  elementsMap: RenderableElementsMap;
  zoom: number;
  offsetLeft: number;
  offsetTop: number;
  scrollX: number;
  scrollY: number;
  // ...
}) => {
  const viewport = {
    x: -scrollX / zoom - offsetLeft,
    y: -scrollY / zoom - offsetTop,
    w: width / zoom,
    h: height / zoom,
  };

  return Array.from(elementsMap.values()).filter((element) => {
    const isInsideViewport = !isOutsideViewPort(appState, [
      element.x,
      element.y,
      element.width,
      element.height,
    ]);
    return isInsideViewport;
  });
};
```

**Adaptación a Rust:**
```rust
pub struct Viewport {
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub zoom: f64,
    pub width: f64,
    pub height: f64,
}

impl Viewport {
    pub fn world_to_screen(&self, world_pos: WorldPosition) -> ScreenPosition {
        ScreenPosition {
            x: (world_pos.x - self.scroll_x) * self.zoom,
            y: (world_pos.y - self.scroll_y) * self.zoom,
        }
    }

    pub fn is_visible(&self, rect: WorldRect) -> bool {
        let world_viewport = WorldRect {
            x: -self.scroll_x / self.zoom,
            y: -self.scroll_y / self.zoom,
            width: self.width / self.zoom,
            height: self.height / self.zoom,
        };

        world_viewport.intersects(&rect)
    }
}

pub struct Scene<T> {
    components: HashMap<ComponentId, T>,
}

impl<T> Scene<T> {
    pub fn visible_components(&self, viewport: &Viewport) -> Vec<&T> {
        self.components.values()
            .filter(|comp| viewport.is_visible(comp.bounds()))
            .collect()
    }
}
```

**2. AppState Separado de Elements**
```typescript
import type { AppState, BinaryFiles } from "../types";

export type RenderInteractiveSceneCallback = {
  app: AppClassProperties,
  canvas: HTMLCanvasElement | null,
  elementsMap: RenderableElementsMap,
  // ...
};
```

**Insight:** `AppState` (viewport, zoom, scroll) está separado de `elementsMap` (componentes).

**Adaptación a Rust:**
```rust
pub struct EditorState {
    pub scene: Scene<Component>,
    pub viewport: Viewport,
    pub selection: HashSet<ComponentId>,
    pub undo_stack: Vec<Vec<Change>>,
    pub redo_stack: Vec<Vec<Change>>,
}

impl EditorState {
    pub fn apply_change(&mut self, change: Change) -> Result<()> {
        // 1. Crear snapshot (si es un grupo de cambios)
        // 2. Aplicar cambio a scene
        // 3. Actualizar viewport si es necesario
        // 4. Añadir a undo_stack
        Ok(())
    }
}
```

---

## 3. Arquitectura Core de ArchFlow (Rust)

### 3.1 Estructura de Crates

```
archflow/
├── archflow-core/           # Dominio puro, sin dependencias de renderizado
│   ├── src/
│   │   ├── domain/         # Tipos de dominio
│   │   │   ├── component.rs
│   │   │   ├── connection.rs
│   │   │   ├── position.rs
│   │   │   └── change.rs
│   │   ├── scene/           # Gestión de escena
│   │   │   ├── mod.rs
│   │   │   ├── scene.rs
│   │   │   └── selection.rs
│   │   ├── validation/       # Validación de recursos AWS
│   │   │   ├── mod.rs
│   │   │   ├── validator.rs
│   │   │   └── rules.rs
│   │   └── export/          # Export a Terraform (post-MVP)
│   │       ├── mod.rs
│   │       └── hcl_generator.rs
│   └── Cargo.toml
│
├── archflow-utils/          # Utilidades comunes (geometría, etc.)
│   ├── src/
│   │   ├── geometry/
│   │   │   ├── rect.rs
│   │   │   ├── point.rs
│   │   │   └── intersection.rs
│   │   └── grid/
│   │       └── snap.rs
│   └── Cargo.toml
│
├── archflow-renderer/      # Abstracción de rendering
│   ├── src/
│   │   ├── traits.rs       # Renderer trait
│   │   ├── canvas2d/      # Canvas 2D implementation
│   │   │   ├── mod.rs
│   │   │   └── renderer.rs
│   │   └── webgpu/        # WebGPU implementation (post-MVP)
│   │       ├── mod.rs
│   │       └── renderer.rs
│   └── Cargo.toml
│
└── archflow-wasm/           # WASM bindings (Leptos integration)
    ├── src/
    │   ├── lib.rs
    │   ├── editor.rs
    │   └── bindings.rs
    └── Cargo.toml
```

---

### 3.2 archflow-core: Dominio Puro

#### 3.2.1 Tipos de Dominio

```rust
// src/domain/component.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ComponentId(pub String);

impl ComponentId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn from(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudResourceType {
    // Tier 1 (MVP - 10 componentes)
    Ec2Instance,
    LambdaFunction,
    S3Bucket,
    RdsInstance,
    Vpc,
    LoadBalancer,
    IamRole,
    CloudfrontDistribution,
    DynamoDbTable,
    Waf,

    // Tier 2 (Post-MVP - más componentes)
    // CloudWatch,
    // SnsTopic,
    // SqsQueue,
    // ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PropertyValue {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    List(Vec<String>),
    Map(HashMap<String, PropertyValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub properties: HashMap<String, PropertyValue>,
    pub tags: Vec<(String, String)>,
    pub terraform_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub resource_type: CloudResourceType,
    pub position: WorldPosition,
    pub dimensions: Dimensions,
    pub data: ResourceData,
}

impl Component {
    pub fn new(resource_type: CloudResourceType) -> Self {
        let dimensions = Dimensions::default_for(&resource_type);

        Self {
            id: ComponentId::new(),
            resource_type,
            position: WorldPosition { x: 0.0, y: 0.0 },
            dimensions,
            data: ResourceData {
                properties: HashMap::new(),
                tags: Vec::new(),
                terraform_name: None,
            },
        }
    }

    pub fn with_position(mut self, position: WorldPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_property(mut self, key: &str, value: PropertyValue) -> Self {
        self.data.properties.insert(key.to_string(), value);
        self
    }
}
```

```rust
// src/domain/connection.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: ConnectionId,
    pub source: ComponentId,
    pub target: ComponentId,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
}

impl Connection {
    pub fn new(source: ComponentId, target: ComponentId) -> Self {
        Self {
            id: ConnectionId(uuid::Uuid::new_v4().to_string()),
            source,
            target,
            source_port: None,
            target_port: None,
        }
    }
}
```

```rust
// src/domain/position.rs

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WorldPosition {
    pub x: f64,
    pub y: f64,
}

impl WorldPosition {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl WorldRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn contains(&self, point: &WorldPosition) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}
```

#### 3.2.2 Change Pattern (React Flow style)

```rust
// src/domain/change.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Change {
    AddComponent { component: Component },
    RemoveComponent { id: ComponentId },
    UpdateComponentPosition { id: ComponentId, position: WorldPosition },
    UpdateComponentDimensions { id: ComponentId, dimensions: Dimensions },
    SelectComponent { id: ComponentId, selected: bool },
    UpdateComponentProperty {
        id: ComponentId,
        key: String,
        value: PropertyValue,
    },
    AddConnection { connection: Connection },
    RemoveConnection { id: ConnectionId },
    Batch { changes: Vec<Change> },
}

impl Change {
    pub fn apply(&self, scene: &mut Scene) -> Result<(), ChangeError> {
        match self {
            Change::AddComponent { component } => {
                scene.add_component(component.clone());
            }
            Change::RemoveComponent { id } => {
                scene.remove_component(id)?;
            }
            Change::UpdateComponentPosition { id, position } => {
                scene.update_component_position(id, *position)?;
            }
            // ... otros cambios
            Change::Batch { changes } => {
                for change in changes {
                    change.apply(scene)?;
                }
            }
        }
        Ok(())
    }

    pub fn undo(&self, scene: &mut Scene) -> Result<(), ChangeError> {
        // Crear el cambio inverso y aplicarlo
        let inverse = self.inverse(scene)?;
        inverse.apply(scene)
    }

    fn inverse(&self, scene: &Scene) -> Result<Self, ChangeError> {
        match self {
            Change::AddComponent { component } => {
                Ok(Change::RemoveComponent {
                    id: component.id.clone(),
                })
            }
            Change::RemoveComponent { id } => {
                // Recuperar el componente del estado anterior
                let component = scene.get_component(id)
                    .ok_or(ChangeError::ComponentNotFound(id.clone()))?;
                Ok(Change::AddComponent {
                    component: component.clone(),
                })
            }
            Change::UpdateComponentPosition { id, position } => {
                let component = scene.get_component(id)
                    .ok_or(ChangeError::ComponentNotFound(id.clone()))?;
                Ok(Change::UpdateComponentPosition {
                    id: id.clone(),
                    position: component.position,
                })
            }
            // ... otros casos
            Change::Batch { changes } => {
                let inverse_changes: Vec<Change> = changes
                    .iter()
                    .map(|c| c.inverse(scene))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Change::Batch {
                    changes: inverse_changes.into_iter().rev().collect(),
                })
            }
            _ => todo!(),
        }
    }
}
```

#### 3.2.3 Scene Management

```rust
// src/scene/scene.rs

use crate::domain::{Component, ComponentId, Connection, ConnectionId, WorldRect};
use std::collections::{HashMap, HashSet};

pub struct Scene {
    components: HashMap<ComponentId, Component>,
    connections: HashMap<ConnectionId, Connection>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.insert(component.id.clone(), component);
    }

    pub fn remove_component(&mut self, id: &ComponentId) -> Result<(), SceneError> {
        // Primero eliminar todas las conexiones que usan este componente
        let to_remove: Vec<ConnectionId> = self.connections
            .values()
            .filter(|c| c.source == *id || c.target == *id)
            .map(|c| c.id.clone())
            .collect();

        for conn_id in to_remove {
            self.connections.remove(&conn_id);
        }

        self.components.remove(id)
            .ok_or(SceneError::ComponentNotFound(id.clone()))?;

        Ok(())
    }

    pub fn get_component(&self, id: &ComponentId) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn get_component_mut(&mut self, id: &ComponentId) -> Option<&mut Component> {
        self.components.get_mut(id)
    }

    pub fn components_in_rect(&self, rect: &WorldRect) -> Vec<&Component> {
        self.components
            .values()
            .filter(|comp| {
                let comp_rect = WorldRect {
                    x: comp.position.x,
                    y: comp.position.y,
                    width: comp.dimensions.width,
                    height: comp.dimensions.height,
                };
                rect.intersects(&comp_rect)
            })
            .collect()
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.insert(connection.id.clone(), connection);
    }

    pub fn remove_connection(&mut self, id: &ConnectionId) -> Result<(), SceneError> {
        self.connections.remove(id)
            .ok_or(SceneError::ConnectionNotFound(id.clone()))?;

        Ok(())
    }
}
```

#### 3.2.4 Selection System

```rust
// src/scene/selection.rs

use crate::domain::ComponentId;
use std::collections::HashSet;

pub struct Selection {
    selected: HashSet<ComponentId>,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
        }
    }

    pub fn select(&mut self, id: ComponentId) {
        self.selected.insert(id);
    }

    pub fn deselect(&mut self, id: &ComponentId) {
        self.selected.remove(id);
    }

    pub fn toggle(&mut self, id: ComponentId) {
        if self.selected.contains(&id) {
            self.selected.remove(&id);
        } else {
            self.selected.insert(id);
        }
    }

    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn is_selected(&self, id: &ComponentId) -> bool {
        self.selected.contains(id)
    }

    pub fn selected_ids(&self) -> impl Iterator<Item = &ComponentId> {
        self.selected.iter()
    }
}
```

---

### 3.3 archflow-renderer: Abstracción de Rendering

```rust
// src/traits.rs

use crate::domain::{Component, Connection, ComponentId, WorldPosition};

pub trait Renderer {
    fn render(&mut self, state: &RenderState);

    fn hit_test(&self, point: ScreenPosition) -> Option<HitResult>;

    fn set_viewport(&mut self, viewport: Viewport);
}

#[derive(Debug, Clone)]
pub enum HitResult {
    Component { id: ComponentId, position: WorldPosition },
    Connection { id: ConnectionId },
    Background,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub components: Vec<(Component, RenderInfo)>,
    pub connections: Vec<Connection>,
    pub viewport: Viewport,
}

#[derive(Debug, Clone)]
pub struct RenderInfo {
    pub screen_position: ScreenPosition,
    pub is_visible: bool,
    pub is_selected: bool,
}
```

```rust
// src/canvas2d/renderer.rs

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::JsCast;
use crate::traits::{Renderer, RenderState};
use crate::domain::{CloudResourceType};

pub struct Canvas2DRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    texture_cache: HashMap<CloudResourceType, ImageBitmap>,
}

impl Canvas2DRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, RendererError> {
        let context = canvas
            .get_context("2d")
            .map(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>())
            .transpose()
            .map_err(|_| RendererError::CanvasNotSupported)?;

        Ok(Self {
            canvas,
            context,
            texture_cache: HashMap::new(),
        })
    }

    fn draw_component(&self, component: &Component, render_info: &RenderInfo) {
        if !render_info.is_visible {
            return;
        }

        let x = render_info.screen_position.x;
        let y = render_info.screen_position.y;

        // Dibujar rectángulo
        self.context.set_fill_style(&JsValue::from_str("#ffffff"));
        self.context.fill_rect(x, y, component.dimensions.width, component.dimensions.height);

        // Dibujar icono
        if let Some(icon) = self.texture_cache.get(&component.resource_type) {
            self.context.draw_image_with_html_image_element(
                icon,
                x,
                y,
                component.dimensions.width,
                component.dimensions.height,
            );
        }

        // Dibujar selección
        if render_info.is_selected {
            self.context.set_stroke_style(&JsValue::from_str("#3b82f6"));
            self.context.set_line_width(2.0);
            self.context.stroke_rect(x, y, component.dimensions.width, component.dimensions.height);
        }
    }
}

impl Renderer for Canvas2DRenderer {
    fn render(&mut self, state: &RenderState) {
        // Limpiar canvas
        let width = state.viewport.width as i32;
        let height = state.viewport.height as i32;
        self.context.clear_rect(0.0, 0.0, width as f64, height as f64);

        // Dibujar grid (si está activado)
        self.draw_grid(&state.viewport);

        // Dibujar conexiones
        for connection in &state.connections {
            self.draw_connection(connection, state);
        }

        // Dibujar componentes
        for (component, render_info) in &state.components {
            self.draw_component(component, render_info);
        }
    }

    fn hit_test(&self, point: ScreenPosition) -> Option<HitResult> {
        // Test de hit en orden inverso (Z-order)
        for (component, render_info) in state.components.iter().rev() {
            if !render_info.is_visible {
                continue;
            }

            let rect = ScreenRect {
                x: render_info.screen_position.x,
                y: render_info.screen_position.y,
                width: component.dimensions.width,
                height: component.dimensions.height,
            };

            if rect.contains(&point) {
                return Some(HitResult::Component {
                    id: component.id.clone(),
                    position: component.position,
                });
            }
        }

        None
    }

    fn set_viewport(&mut self, viewport: Viewport) {
        // Actualizar transform de canvas
        self.context.set_transform(1.0, 0.0, 0.0, 1.0, -viewport.scroll_x, -viewport.scroll_y);
    }
}
```

---

### 3.4 archflow-wasm: WASM Bindings

```rust
// src/lib.rs

use wasm_bindgen::prelude::*;
use archflow_core::{EditorState, Change, Component, CloudResourceType};

#[wasm_bindgen]
pub struct ArchFlowEditor {
    state: EditorState,
}

#[wasm_bindgen]
impl ArchFlowEditor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
        }
    }

    #[wasm_bindgen]
    pub fn add_component(&mut self, resource_type: &str) -> String {
        let resource_type = match resource_type {
            "ec2-instance" => CloudResourceType::Ec2Instance,
            "lambda-function" => CloudResourceType::LambdaFunction,
            // ... más tipos
            _ => panic!("Unknown resource type: {}", resource_type),
        };

        let component = Component::new(resource_type);
        let id = component.id.0.clone();

        let change = Change::AddComponent { component };
        self.state.apply_change(change).unwrap();

        id
    }

    #[wasm_bindgen]
    pub fn apply_change(&mut self, change_json: &str) -> Result<(), JsValue> {
        let change: Change = serde_json::from_str(change_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.state.apply_change(change)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_state_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap()
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) -> Result<(), JsValue> {
        self.state.undo()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) -> Result<(), JsValue> {
        self.state.redo()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }
}
```

---

## 4. Testing Strategy (TDD)

### 4.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CloudResourceType;

    #[test]
    fn test_create_component() {
        let component = Component::new(CloudResourceType::Ec2Instance);

        assert_eq!(component.resource_type, CloudResourceType::Ec2Instance);
        assert_eq!(component.position, WorldPosition { x: 0.0, y: 0.0 });
    }

    #[test]
    fn test_component_with_position() {
        let component = Component::new(CloudResourceType::LambdaFunction)
            .with_position(WorldPosition { x: 100.0, y: 200.0 });

        assert_eq!(component.position, WorldPosition { x: 100.0, y: 200.0 });
    }

    #[test]
    fn test_change_add_component() {
        let mut scene = Scene::new();
        let component = Component::new(CloudResourceType::S3Bucket);
        let change = Change::AddComponent { component: component.clone() };

        change.apply(&mut scene).unwrap();

        assert_eq!(scene.get_component(&component.id), Some(&component));
    }

    #[test]
    fn test_change_undo() {
        let mut scene = Scene::new();
        let component = Component::new(CloudResourceType::Ec2Instance);
        let change = Change::AddComponent { component: component.clone() };

        change.apply(&mut scene).unwrap();
        assert!(scene.get_component(&component.id).is_some());

        change.undo(&mut scene).unwrap();
        assert!(scene.get_component(&component.id).is_none());
    }

    #[test]
    fn test_selection_toggle() {
        let mut selection = Selection::new();
        let id = ComponentId::from("test-id");

        selection.toggle(id.clone());
        assert!(selection.is_selected(&id));

        selection.toggle(id.clone());
        assert!(!selection.is_selected(&id));
    }
}
```

### 4.2 Integration Tests (Kotest-like en Rust)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_drag_and_drop_workflow() {
        let mut editor = ArchFlowEditor::new();

        // Crear componente
        let id = editor.add_component("ec2-instance");

        // Aplicar cambio de posición
        let change_json = r#"{
            "type": "UpdateComponentPosition",
            "id": "PLACEHOLDER_ID",
            "position": { "x": 100.0, "y": 200.0 }
        }"#.replace("PLACEHOLDER_ID", &id);

        editor.apply_change(&change_json).unwrap();

        // Verificar posición
        let state = editor.get_state_json();
        assert!(state.contains(r#""x": 100.0"#));
        assert!(state.contains(r#""y": 200.0"#));

        // Verificar undo
        editor.undo().unwrap();
        let undone_state = editor.get_state_json();
        assert!(!undone_state.contains(r#""x": 100.0"#));
    }
}
```

---

## 5. Conclusiones y Recomendaciones

### 5.1 Patrones a Implementar (Prioridad Alta)

1. ✅ **Change Pattern (React Flow)**: Union de tipos de cambios con undo/redo
2. ✅ **Immutable Records (tldraw)**: Builder pattern para componentes
3. ✅ **Viewport Culling (Excalidraw)**: Solo renderizar lo visible
4. ✅ **Core-First**: Dominio independiente de rendering
5. ✅ **Renderer Trait**: Strategy pattern para Canvas2D/WebGPU

### 5.2 Patrones a Implementar (Prioridad Media - Post-MVP)

6. ⏳ **Texture Caching**: Cache de iconos renderizados (tldraw)
7. ⏳ **Double Canvas**: Static + overlay (Excalidraw)
8. ⏳ **Scene Snapshots**: Para persistencia eficiente

### 5.3 Arquitectura de MVP

```
MVP Scope (3-4 meses):
├── archflow-core          ✅ DONE (dominio + cambios)
├── archflow-utils         ✅ DONE (geometría básica)
├── archflow-renderer      ✅ DONE (Canvas2D only)
├── archflow-wasm         ✅ DONE (bindings básicos)
└── archflow-leptos       🚧 IN PROGRESS (UI en Leptos)
    └── 10 componentes AWS
    └── Drag & drop
    └── Undo/redo (20 estados)
    └── Export a Terraform (básico)
```

### 5.4 Riesgos y Mitigación

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|----------|------------|
| **WASM bundle size > 5MB** | Media | Alto | Feature flags, tree-shaking |
| **Canvas2D performance limit** | Media | Alto | Ya tenemos WebGPU en roadmap |
| **Memory leaks en Rust** | Baja | Alto | Rust ownership previene esto |
| **Learning curve de equipo** | Alta | Medio | Documentación extensa |

---

## 6. Next Steps

### Week 1-2: Foundation
- [ ] Crear workspace Cargo con 4 crates
- [ ] Implementar tipos de dominio (Component, Connection, Position)
- [ ] Implementar Change pattern (Apply/Undo)
- [ ] Tests unitarios básicos

### Week 3-4: Scene Management
- [ ] Implementar Scene con HashMaps
- [ ] Implementar Selection system
- [ ] Implementar Viewport con culling
- [ ] Tests de integración

### Week 5-6: Rendering Abstraction
- [ ] Definir Renderer trait
- [ ] Implementar Canvas2DRenderer
- [ ] Implementar hit testing
- [ ] Tests de rendering

### Week 7-8: WASM Bindings
- [ ] Exponer API a JavaScript
- [ ] Integrar con Leptos
- [ ] Tests E2E en navegador
- [ ] Performance benchmarks

---

## 7. Referencias

- **React Flow**: https://reactflow.dev (NodeChange pattern)
- **tldraw**: https://github.com/tldraw/tldraw (Immutable records)
- **Excalidraw**: https://github.com/excalidraw/excalidraw (Viewport culling)
- **MVP Roadmap**: `/home/rubentxu/Proyectos/rust/hodei-archFlow/docs/MVP-ROADMAP-CONSOLIDATED.md`
- **PRD Crítica**: `/home/rubentxu/Proyectos/rust/hodei-archFlow/docs/PRD-CRITICA.md`
