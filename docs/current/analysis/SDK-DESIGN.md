# ArchFlow SDK - Análisis y Diseño

## Índice

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Análisis del Engine Existente](#2-análisis-del-engine-existente)
3. [Estudio de APIs de Referencia](#3-estudio-de-apis-de-referencia)
4. [Diseño de Arquitectura del SDK](#4-diseño-de-arquitectura-del-sdk)
5. [API Design - Capa Rust](#5-api-design---capa-rust)
6. [API Design - Capa WASM/JS](#6-api-design---capa-wasmjs)
7. [Patrones de Integración](#7-patrones-de-integración)
8. [Roadmap de Implementación](#8-roadmap-de-implementación)

---

## 1. Resumen Ejecutivo

Este documento presenta el diseño técnico del **ArchFlow SDK**, una capa de abstracción profesional que expone las capacidades del engine de renderizado de alto rendimiento a desarrolladores externos. El SDK sigue patrones establecidos por herramientas líderes como **tldraw** y **Figma**, adaptados a la arquitectura única de ArchFlow basada en Rust y WebGPU.

### Objetivos Principales

| Objetivo | Descripción | Prioridad |
|----------|-------------|-----------|
| **Accesibilidad** | API intuitiva para desarrolladores web y Rust | Alta |
| **Performance** | Zero-overhead abstraction sobre el engine | Alta |
| **Extensibilidad** | Sistema de plugins y extensiones | Media |
| **Type Safety** | Tipado completo en Rust y TypeScript | Alta |
| **Documentación** | Docs comprehensivos con ejemplos | Alta |

### Stack Tecnológico

```
┌─────────────────────────────────────────────────────────────────┐
│                    ARCHFLOW SDK LAYERS                          │
├─────────────────────────────────────────────────────────────────┤
│  @archflow/sdk (npm)        │  TypeScript / JavaScript API     │
│  ├── React components       │  UI Components                    │
│  ├── Canvas wrapper         │  <ArchFlowCanvas />               │
│  └── Hooks                  │  useEditor(), useSelection()      │
├─────────────────────────────────────────────────────────────────┤
│  archflow-sdk (Rust)        │  Rust API para aplicaciones Rust  │
│  ├── Editor trait           │  Trait principal                  │
│  ├── Commands               │  Sistema de comandos              │
│  └── Events                 │  Sistema de eventos               │
├─────────────────────────────────────────────────────────────────┤
│  archflow-wasm              │  WASM bindings                    │
│  └── Core                   │  Engine bindings                  │
├─────────────────────────────────────────────────────────────────┤
│  ARCHFLOW ENGINE (Rust)     │  Core engine                      │
│  ├── archflow-core          │  Tipos base, geometría            │
│  ├── archflow-renderers     │  WebGPU batch rendering           │
│  ├── archflow-geometry      │  Algoritmos geométricos           │
│  ├── archflow-spatial       │  Índices espaciales (R-tree)      │
│  ├── archflow-collab        │  Colaboración en tiempo real      │
│  └── archflow-records       │  CRDT store                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Análisis del Engine Existente

### 2.1 Crates Disponibles

El engine de ArchFlow ya cuenta con una arquitectura modular robusta:

#### **archflow-core** - Tipos Fundamentales
```rust
// Tipos base disponibles
Vec2, Vec3, Mat3          // Álgebra lineal
Rect, Rect2D              // Rectángulos y bounds
Color, Rgba, Hsla         // Sistema de colores
EntityId                  // Identificadores únicos
Transform                 // Matrices de transformación
Animation, Keyframes      // Sistema de animaciones
```

#### **archflow-renderers** - Rendering de Alto Rendimiento
```rust
// Sistema de batch rendering con WebGPU
BatchRenderer2D           // Renderizado instanciado
RenderContext             // Contexto WebGPU
Renderable trait          // Trait para objetos renderizables
MaterialId                // Identificadores de material
Bounds                    // Límites para culling
```

#### **archflow-geometry** - Algoritmos Geométricos
```rust
// Algoritmos especializados
Intersection              // Detección de intersecciones
Polygon operations        // Operaciones booleanas
Convex hull               // Cálculo de convex hull
SDF                      // Signed Distance Fields
```

#### **archflow-spatial** - Índices Espaciales
```rust
// Estructuras para optimización espacial
RTree                    // Índice R-tree
ViewportManager          // Gestión de viewport visible
SpatialHash              // Hash espacial para colisiones
```

#### **archflow-records** - Store Persistente
```rust
// Sistema de almacenamiento
RecordStore              // Store con versionado
FractionalIndex          // Índices ordenados
Delta encoding           // Diferencias para sync
```

#### **archflow-collab** - Colaboración
```rust
// Protocolo de colaboración
Network                  // WebSocket client/server
SharedBuffer             // Buffer compartido (SharedArrayBuffer)
Sync protocol            // Sincronización CRDT
```

### 2.2 Demo Web Actual

El demo existente (`crates/demo-web`) ya implementa:

| Componente | Estado | Descripción |
|------------|--------|-------------|
| `shapes.rs` | ✅ | ShapeId, ShapeType, Shape, ShapeStore |
| `state.rs` | ✅ | DemoState con Tool, InteractionState, Command pattern |
| `lib.rs` | ✅ | WASM bindings con renderizado |

### 2.3 Brecha entre Engine y SDK

```
FUNCIONALIDAD              ENGINE        SDK    PRIORIDAD
────────────────────────────────────────────────────────────────
Shape creation             ████████     ░░░░░   Alta
Shape manipulation         ████████     ░░░░░   Alta
Selection system           ████░░░░     ░░░░░   Alta
Viewport/Zoom/Pan          ████████     ░░░░░   Alta
Undo/Redo                  ████████     ░░░░░   Alta
Keyboard shortcuts         ████████     ░░░░░   Alta
Event handling             █████░░░     ░░░░░   Media
Plugin system              █████░░░     ░░░░░   Baja
React integration          ░░░░░░░░     ░░░░░   Media
TypeScript types           ░░░░░░░░     ░░░░░   Alta
```

---

## 3. Estudio de APIs de Referencia

### 3.1 Figma Plugin API

Figma expone su funcionalidad a través de un **plugin system** basado en JavaScript:

```javascript
// Figma Plugin API
figma.showUI(__html__, { width: 300, height: 400 });

// Acceso al documento
const nodes = figma.currentPage.findAll(n => n.type === 'RECTANGLE');

// Manipulación de nodos
node.x = 100;
node.y = 200;
node.resize(300, 400);

// Eventos
figma.on('selectionchange', () => {
  console.log('New selection:', figma.currentPage.selection);
});

// Comandos
figma.group([node1, node2], figma.currentPage);
figma.ungroup(groupNode);
```

**Patrones observados:**
- API imperativa con getters/setters
- Eventos mediante callbacks
- Sistema de undo/redo integrado
- Sandbox con limitaciones de seguridad

### 3.2 tldraw SDK

tldraw expone su editor a través de **React components** y un **store** observable:

```typescript
// tldraw React integration
import { Tldraw, useEditor, useSelection } from '@tldraw/tldraw';

function MyComponent() {
  const editor = useEditor();
  const selection = useSelection();

  // API del editor
  editor.createShape({ type: 'rectangle', x: 100, y: 100 });
  editor.deleteShapes(selection);

  // Transiciones
  editor.transition('select.idle', 'select.dragging');
}

// Store observable (yjs)
const store = new TLStore();
store.listen((update) => {
  console.log('Store changed:', update);
});
```

**Patrones observados:**
- Store observable basado en Yjs (CRDT)
- Comandos inmutables para modificaciones
- Estado serializable completamente
- Sistema de migraciones para versiones

### 3.3 Comparativa de Enfoques

| Aspecto | Figma | tldraw | ArchFlow (propuesta) |
|---------|-------|--------|----------------------|
| **Paradigma** | Imperativo | React + Store | Rust-first + bindings |
| **Estado** | Mutable | Yjs CRDT | Event sourcing |
| **Eventos** | Callbacks | Observables | Rust traits |
| **Extensibilidad** | Plugins sandboxed | React components | Traits + plugins |
| **UI** | HTML/CSS | React | Framework-agnostic |
| **Lenguaje** | JavaScript | TypeScript | Rust + TS |

---

## 4. Diseño de Arquitectura del SDK

### 4.1 Principios de Diseño

1. **Zero-Cost Abstraction**: El SDK no debe añadir overhead en Rust
2. **Type Safety**: Tipado completo en ambos lenguajes
3. **Ergonomic API**: API intuitiva que sigue convenciones establecidas
4. **Framework Agnostic**: No зависи del framework UI del usuario
5. **Observable**: Estado observable para sincronización

### 4.2 Capas del SDK

```
┌─────────────────────────────────────────────────────────────────┐
│                     APPLICATION LAYER                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  React Adapter  │  │  Vue Adapter    │  │  Vanilla JS     │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     SDK TYPES LAYER                             │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  TypeScript Definitions (@archflow/sdk)                     ││
│  │  • Editor interface    • Shape interfaces    • Events       ││
│  │  • Commands            • Hooks               • Components  ││
│  └─────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────┤
│                     RUST SDK LAYER                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  archflow-sdk (Rust crate)                                  ││
│  │  • Editor trait          • Command pattern    • Events      ││
│  │  • Selection manager     • History            • Plugins     ││
│  └─────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────┤
│                     ENGINE LAYER                                │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  archflow-core + renderers + geometry + spatial + ...       ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### 4.3 Module Structure

```
archflow-sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Main entry point
│   ├── editor/
│   │   ├── mod.rs
│   │   ├── editor.rs            # Editor trait
│   │   ├── selection.rs         # Selection management
│   │   └── viewport.rs          # Viewport/zoom/pan
│   ├── shapes/
│   │   ├── mod.rs
│   │   ├── shape.rs             # Shape abstraction
│   │   ├── builder.rs           # Shape builder API
│   │   └── custom.rs            # Custom shape registration
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── command.rs           # Command trait
│   │   ├── registry.rs          # Command registry
│   │   └── builtins.rs          # Built-in commands
│   ├── events/
│   │   ├── mod.rs
│   │   ├── event.rs             # Event types
│   │   └── handler.rs           # Event handling
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── plugin.rs            # Plugin trait
│   │   └── manager.rs           # Plugin lifecycle
│   └── input/
│       ├── mod.rs
│       ├── mouse.rs             # Mouse input handling
│       ├── keyboard.rs          # Keyboard shortcuts
│       └── touch.rs             # Touch gestures
└── examples/
    └── basic.rs
```

---

## 5. API Design - Capa Rust

### 5.1 Editor Trait

El trait principal que define la interfaz del editor:

```rust
use archflow_core::{EntityId, Transform, Vec2, Color};
use archflow_geometry::Rect;

/// Trait principal del editor de ArchFlow
///
/// Implementado por el engine y expuesto a través de WASM.
/// Sigue el patrón de diseño "trait as API" para máxima flexibilidad.
#[wasm_bindgen]
pub trait Editor {
    // === Shape Operations ===

    /// Crea una nueva forma en el editor
    fn create_shape(&mut self, shape: &CreateShapeParams) -> Result<EntityId, EditorError>;

    /// Obtiene una forma por su ID
    fn get_shape(&self, id: EntityId) -> Option<&dyn Shape>;

    /// Obtiene todas las formas
    fn get_shapes(&self) -> Vec<&dyn Shape>;

    /// Actualiza una forma existente
    fn update_shape(&mut self, id: EntityId, params: &UpdateShapeParams);

    /// Elimina una forma
    fn delete_shape(&mut self, id: EntityId);

    // === Selection Operations ===

    /// Obtiene los IDs de las formas seleccionadas
    fn get_selection(&self) -> Vec<EntityId>;

    /// Selecciona una forma (reemplaza selección actual)
    fn select(&mut self, id: EntityId);

    /// Añade a la selección actual
    fn select_add(&mut self, id: EntityId);

    /// Quita de la selección
    fn select_remove(&mut self, id: EntityId);

    /// Selecciona todas las formas
    fn select_all(&mut self);

    /// Limpia la selección
    fn clear_selection(&mut self);

    // === Viewport Operations ===

    /// Obtiene el viewport actual
    fn get_viewport(&self) -> Viewport;

    /// Configura el nivel de zoom
    fn set_zoom(&mut self, zoom: f32);

    /// Configura la posición del viewport
    fn set_viewport_center(&mut self, x: f64, y: f64);

    /// Ajusta el zoom para mostrar todas las formas
    fn zoom_to_fit(&mut self);

    /// Ajusta el zoom para mostrar la selección
    fn zoom_to_selection(&mut self);

    // === History Operations ===

    /// Deshace la última acción
    fn undo(&mut self) -> bool;

    /// Rehace la última acción deshecha
    fn redo(&mut self) -> bool;

    /// Indica si hay acciones para deshacer
    fn can_undo(&self) -> bool;

    /// Indica si hay acciones para rehacer
    fn can_redo(&self) -> bool;

    // === Input Handling ===

    /// Procesa un evento de mouse
    fn on_mouse_down(&mut self, x: f64, y: f64, button: u16);

    /// Procesa movimiento de mouse
    fn on_mouse_move(&mut self, x: f64, y: f64);

    /// Procesa liberación de mouse
    fn on_mouse_up(&mut self, x: f64, y: f64);

    /// Procesa evento de wheel
    fn on_wheel(&mut self, x: f64, y: f64, delta_y: f64);

    /// Procesa evento de teclado
    fn on_key_down(&mut self, key: &str, modifiers: &KeyModifiers) -> bool;

    /// Procesa liberación de teclado
    fn on_key_up(&mut self, key: &str, modifiers: &KeyModifiers);

    // === Events ===

    /// Suscribe a eventos del editor
    fn subscribe(&mut self, event_type: EventType, callback: EventCallback);

    /// Cancela una suscripción
    fn unsubscribe(&mut self, subscription_id: SubscriptionId);
}
```

### 5.2 Shape Abstraction

```rust
/// Representa una forma en el editor.
///
/// Este trait define las operaciones comunes a todas las formas.
/// Las formas concretas implementan este trait.
pub trait Shape {
    /// Obtiene el ID único de la forma
    fn id(&self) -> EntityId;

    /// Obtiene el tipo de forma
    fn shape_type(&self) -> ShapeType;

    /// Obtiene los bounds de la forma
    fn bounds(&self) -> Rect;

    /// Obtiene la posición (top-left)
    fn position(&self) -> Vec2;

    /// Configura la posición
    fn set_position(&mut self, x: f64, y: f64);

    /// Obtiene el tamaño
    fn size(&self) -> (f64, f64);

    /// Configura el tamaño
    fn set_size(&mut self, width: f64, height: f64);

    /// Obtiene el color de relleno
    fn fill_color(&self) -> Color;

    /// Configura el color de relleno
    fn set_fill_color(&mut self, color: Color);

    /// Obtiene la rotación en grados
    fn rotation(&self) -> f32;

    /// Configura la rotación en grados
    fn set_rotation(&mut self, degrees: f32);

    /// Verifica si un punto está dentro de la forma
    fn contains_point(&self, x: f64, y: f64) -> bool;

    /// Verifica si intersecta con un rectángulo
    fn intersects_rect(&self, rect: &Rect) -> bool;

    /// Serializa la forma a JSON
    fn to_json(&self) -> serde_json::Value;

    /// Deserializa desde JSON
    fn from_json(json: &serde_json::Value) -> Result<Self, EditorError>
    where
        Self: Sized;
}

/// Tipos de formas soportadas
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Path,
    Text,
    Image,
    Group,
    Custom(u32), // Para extensiones
}
```

### 5.3 Command Pattern

```rust
/// Trait base para comandos ejecutables.
///
/// Los comandos encapsulan una acción y su inverso para implementar
/// undo/redo. Siguiendo el patrón Command de Gang of Four.
pub trait Command: Send + Sync {
    /// Ejecuta el comando y retorna el comando inverso para undo
    fn execute(&mut self, editor: &mut dyn Editor) -> Result<Box<dyn Command>, EditorError>;

    /// Descripción del comando (para UI)
    fn description(&self) -> String;

    /// Indica si el comando puede ejecutarse
    fn can_execute(&self, editor: &dyn Editor) -> bool {
        true
    }
}

/// Registro de comandos disponibles
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn CommandConstructor>>,
}

impl CommandRegistry {
    /// Registra un nuevo comando
    pub fn register<C: CommandConstructor + 'static>(&mut self, name: &str) {
        self.commands.insert(name.to_string(), Box::new(C::create));
    }

    /// Crea una instancia de comando
    pub fn create(&self, name: &str) -> Option<Box<dyn Command>> {
        self.commands.get(name).and_then(|c| c())
    }
}

/// Trait para constructores de comandos
pub trait CommandConstructor: Send + Sync {
    fn create() -> Option<Box<dyn Command>>;
}

// === Built-in Commands ===

/// Comando para crear una nueva forma
pub struct CreateShapeCommand {
    shape_type: ShapeType,
    position: Vec2,
    size: (f64, f64),
    color: Option<Color>,
    created_id: Option<EntityId>,
}

impl Command for CreateShapeCommand {
    fn execute(&mut self, editor: &mut dyn Editor) -> Result<Box<dyn Command>, EditorError> {
        let params = CreateShapeParams {
            shape_type: self.shape_type,
            x: self.position.x,
            y: self.position.y,
            width: self.size.0,
            height: self.size.1,
            fill_color: self.color,
            ..Default::default()
        };

        let id = editor.create_shape(&params)?;
        let reverse = DeleteShapeCommand { id, ..Default::default() };
        self.created_id = Some(id);
        Ok(Box::new(reverse))
    }

    fn description(&self) -> String {
        format!("Create {}", self.shape_type)
    }
}

/// Comando para mover formas
pub struct MoveShapesCommand {
    shape_ids: Vec<EntityId>,
    delta: Vec2,
    original_positions: HashMap<EntityId, Vec2>,
}

impl Command for MoveShapesCommand {
    fn execute(&mut self, editor: &mut dyn Editor) -> Result<Box<dyn Command>, EditorError> {
        // Guardar posiciones originales
        for id in &self.shape_ids {
            if let Some(shape) = editor.get_shape(*id) {
                self.original_positions.insert(*id, shape.position());
            }
        }

        // Mover formas
        for id in &self.shape_ids {
            if let Some(shape) = editor.get_shape(*id) {
                let new_pos = shape.position() + self.delta;
                editor.update_shape(*id, &UpdateShapeParams {
                    x: Some(new_pos.x),
                    y: Some(new_pos.y),
                    ..Default::default()
                });
            }
        }

        // Comando inverso
        let mut reverse = MoveShapesCommand {
            shape_ids: self.shape_ids.clone(),
            delta: -self.delta,
            original_positions: HashMap::new(),
        };
        std::mem::swap(&mut reverse.original_positions, &mut self.original_positions);
        Ok(Box::new(reverse))
    }

    fn description(&self) -> String {
        format!("Move {} shapes", self.shape_ids.len())
    }
}
```

---

## 6. API Design - Capa WASM/JS

### 6.1 TypeScript Interfaces

```typescript
// @archflow/sdk - TypeScript definitions

export interface EditorOptions {
  /** Canvas element to attach to */
  canvas: HTMLCanvasElement;
  /** Initial viewport settings */
  viewport?: ViewportOptions;
  /** Rendering backend */
  renderer?: 'canvas2d' | 'webgpu';
}

export interface ViewportOptions {
  x?: number;
  y?: number;
  zoom?: number;
  minZoom?: number;
  maxZoom?: number;
}

export interface ShapeData {
  id: string;
  type: ShapeType;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  fillColor: string;
  strokeColor?: string;
  strokeWidth?: number;
  opacity: number;
}

export type ShapeType = 
  | 'rectangle' 
  | 'ellipse' 
  | 'line' 
  | 'path' 
  | 'text' 
  | 'image'
  | 'group';

export interface Selection {
  shapes: string[];
  bounds: Bounds;
}

export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type EventType = 
  | 'selectionchange'
  | 'shapecreate'
  | 'shapedelete'
  | 'shapeupdate'
  | 'viewportchange'
  | 'toolchange'
  | 'undo'
  | 'redo';

export interface EditorEventMap {
  selectionchange: (selection: Selection) => void;
  shapecreate: (shape: ShapeData) => void;
  shapedelete: (id: string) => void;
  shapeupdate: (id: string, changes: Partial<ShapeData>) => void;
  viewportchange: (viewport: Viewport) => void;
  toolchange: (tool: Tool) => void;
  undo: () => void;
  redo: () => void;
}
```

### 6.2 JavaScript API

```typescript
// @archflow/sdk - JavaScript API

import { init, Editor } from '@archflow/sdk';

class ArchFlowEditor {
  private editor: Editor;
  private eventListeners: Map<string, Function[]> = new Map();

  constructor(options: EditorOptions) {
    this.editor = new Editor(options.canvas);
    this.setupEventListeners();
  }

  // === Shape Operations ===

  createRectangle(x: number, y: number, width: number, height: number): string {
    return this.editor.createShape({
      type: 'rectangle',
      x, y, width, height
    });
  }

  createEllipse(x: number, y: number, radiusX: number, radiusY: number): string {
    return this.editor.createShape({
      type: 'ellipse',
      x: x - radiusX,
      y: y - radiusY,
      width: radiusX * 2,
      height: radiusY * 2
    });
  }

  createLine(x1: number, y1: number, x2: number, y2: number): string {
    return this.editor.createShape({
      type: 'line',
      x: x1,
      y: y1,
      width: x2 - x1,
      height: y2 - y1
    });
  }

  getShape(id: string): ShapeData | null {
    return this.editor.getShape(id);
  }

  getAllShapes(): ShapeData[] {
    return this.editor.getShapes();
  }

  updateShape(id: string, changes: Partial<ShapeData>): void {
    this.editor.updateShape(id, changes);
  }

  deleteShape(id: string): void {
    this.editor.deleteShape(id);
  }

  // === Selection Operations ===

  getSelection(): Selection {
    const ids = this.editor.getSelection();
    const shapes = ids.map(id => this.editor.getShape(id)).filter(Boolean);
    return {
      shapes: ids,
      bounds: this.calculateSelectionBounds(shapes)
    };
  }

  select(id: string): void {
    this.editor.select(id);
  }

  selectMultiple(ids: string[]): void {
    this.editor.selectClear();
    ids.forEach(id => this.editor.selectAdd(id));
  }

  selectAll(): void {
    this.editor.selectAll();
  }

  clearSelection(): void {
    this.editor.clearSelection();
  }

  // === Viewport Operations ===

  getViewport(): Viewport {
    return this.editor.getViewport();
  }

  setZoom(zoom: number): void {
    this.editor.setZoom(Math.max(0.1, Math.min(10, zoom)));
  }

  zoomIn(amount: number = 0.1): void {
    const current = this.editor.getViewport().zoom;
    this.setZoom(current + amount);
  }

  zoomOut(amount: number = 0.1): void {
    const current = this.editor.getViewport().zoom;
    this.setZoom(current - amount);
  }

  zoomToFit(): void {
    this.editor.zoomToFit();
  }

  pan(dx: number, dy: number): void {
    const viewport = this.editor.getViewport();
    this.editor.setViewportCenter(
      viewport.x + dx,
      viewport.y + dy
    );
  }

  // === History Operations ===

  undo(): boolean {
    return this.editor.undo();
  }

  redo(): boolean {
    return this.editor.redo();
  }

  canUndo(): boolean {
    return this.editor.canUndo();
  }

  canRedo(): boolean {
    return this.editor.canRedo();
  }

  // === Event System ===

  on<K extends EventType>(event: K, callback: EditorEventMap[K]): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(callback);

    // Return unsubscribe function
    return () => {
      const listeners = this.eventListeners.get(event);
      const index = listeners?.indexOf(callback);
      if (index !== undefined && index > -1) {
        listeners?.splice(index, 1);
      }
    };
  }

  off<K extends EventType>(event: K, callback?: EditorEventMap[K]): void {
    if (callback) {
      const listeners = this.eventListeners.get(event);
      const index = listeners?.indexOf(callback);
      if (index !== undefined && index > -1) {
        listeners?.splice(index, 1);
      }
    } else {
      this.eventListeners.delete(event);
    }
  }

  // === Input Handling ===

  private setupEventListeners(): void {
    const canvas = this.editor.getCanvas();
    
    canvas.addEventListener('mousedown', (e: MouseEvent) => {
      this.editor.onMouseDown(e.offsetX, e.offsetY, e.button);
    });

    canvas.addEventListener('mousemove', (e: MouseEvent) => {
      this.editor.onMouseMove(e.offsetX, e.offsetY);
    });

    canvas.addEventListener('mouseup', (e: MouseEvent) => {
      this.editor.onMouseUp(e.offsetX, e.offsetY);
    });

    canvas.addEventListener('wheel', (e: WheelEvent) => {
      e.preventDefault();
      this.editor.onWheel(e.offsetX, e.offsetY, e.deltaY);
    }, { passive: false });

    document.addEventListener('keydown', (e: KeyboardEvent) => {
      this.editor.onKeyDown(e.key, {
        ctrl: e.ctrlKey || e.metaKey,
        shift: e.shiftKey,
        alt: e.altKey
      });
    });
  }

  // === Utilities ===

  render(): void {
    this.editor.render();
  }

  destroy(): void {
    this.editor.destroy();
    this.eventListeners.clear();
  }
}

// Factory function
export async function createEditor(options: EditorOptions): Promise<ArchFlowEditor> {
  await init(); // Initialize WASM
  return new ArchFlowEditor(options);
}

// React hook
export function useArchFlowEditor(options: EditorOptions) {
  const [editor, setEditor] = useState<ArchFlowEditor | null>(null);

  useEffect(() => {
    createEditor(options).then(setEditor);
    return () => editor?.destroy();
  }, []);

  return editor;
}
```

### 6.3 React Component

```typescript
// @archflow/sdk/react - React integration

import React, { useRef, useEffect, useCallback } from 'react';
import { createEditor, ArchFlowEditor } from '@archflow/sdk';

interface ArchFlowCanvasProps {
  width?: number | string;
  height?: number | string;
  onSelectionChange?: (selection: Selection) => void;
  onShapeCreate?: (shape: ShapeData) => void;
  initialShapes?: ShapeData[];
}

export const ArchFlowCanvas: React.FC<ArchFlowCanvasProps> = ({
  width = '100%',
  height = '100%',
  onSelectionChange,
  onShapeCreate,
  initialShapes = []
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const editorRef = useRef<ArchFlowEditor | null>(null);

  // Initialize editor
  useEffect(() => {
    if (!canvasRef.current) return;

    createEditor({ canvas: canvasRef.current }).then(editor => {
      editorRef.current = editor;

      // Subscribe to events
      editor.on('selectionchange', (selection) => {
        onSelectionChange?.(selection);
      });

      editor.on('shapecreate', (shape) => {
        onShapeCreate?.(shape);
      });

      // Add initial shapes
      initialShapes.forEach(shape => {
        editor.createShape(shape);
      });
    });

    return () => {
      editorRef.current?.destroy();
    };
  }, []);

  // Expose editor methods
  const createRectangle = useCallback((x: number, y: number, w: number, h: number) => {
    editorRef.current?.createRectangle(x, y, w, h);
  }, []);

  const createEllipse = useCallback((x: number, y: number, rx: number, ry: number) => {
    editorRef.current?.createEllipse(x, y, rx, ry);
  }, []);

  const zoomIn = useCallback(() => {
    editorRef.current?.zoomIn();
  }, []);

  const zoomOut = useCallback(() => {
    editorRef.current?.zoomOut();
  }, []);

  // Render props for custom controls
  return (
    <div className="archflow-canvas-container">
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        style={{ width, height }}
      />
      {/* Control panel can access editor methods */}
      <ControlPanel
        onZoomIn={zoomIn}
        onZoomOut={zoomOut}
        onCreateRectangle={createRectangle}
        onCreateEllipse={createEllipse}
      />
    </div>
  );
};
```

---

## 7. Patrones de Integración

### 7.1 Plugin System

```rust
/// Trait para plugins del editor.
///
/// Los plugins pueden extiende la funcionalidad del editor
/// sin modificar el código base. Siguiendo el patrón Strategy.
pub trait Plugin: Send + Sync {
    /// Nombre del plugin
    fn name(&self) -> &'static str;

    /// Versión del plugin
    fn version(&self) -> Version;

    /// Inicializa el plugin
    fn init(&mut self, editor: &mut dyn Editor) -> Result<(), PluginError>;

    /// Se ejecuta cuando el editor se destruye
    fn shutdown(&self);

    /// Hook llamado en cada frame de render
    fn on_render(&self, _editor: &dyn Editor) {}

    /// Hook llamado cuando cambia la selección
    fn on_selection_change(&self, _editor: &dyn Editor, _selection: &[EntityId]) {}
}

/// Gestor de plugins
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    editor: Option<*mut dyn Editor>,
}

impl PluginManager {
    /// Instala un nuevo plugin
    pub fn install(&mut self, plugin: Box<dyn Plugin>) -> Result<PluginId, PluginError> {
        // Validate plugin
        plugin.validate()?;

        // Initialize with editor access
        if let Some(editor_ptr) = self.editor {
            plugin.init(unsafe { &mut *editor_ptr })?;
        }

        let id = PluginId::new();
        self.plugins.push(plugin);
        Ok(id)
    }

    /// Desinstala un plugin
    pub fn uninstall(&mut self, id: PluginId) -> Result<(), PluginError> {
        self.plugins.retain(|p| p.id() != id);
        Ok(())
    }
}
```

### 7.2 Custom Shapes

```rust
/// Trait para formas personalizadas.
///
/// Permite a los usuarios del SDK definir sus propias formas
/// que se integran completamente con el editor.
pub trait CustomShape: Send + Sync {
    /// Identificador único del tipo de forma
    const TYPE: &'static str;

    /// Nombre legible de la forma
    const DISPLAY_NAME: &'static str;

    /// Crea una instancia de la forma
    fn create(&self, id: EntityId) -> Box<dyn Shape>;

    /// Configuración de renderizado
    fn render_options(&self) -> ShapeRenderOptions {
        ShapeRenderOptions::default()
    }
}

/// Registro de formas personalizadas
pub struct ShapeRegistry {
    shapes: HashMap<&'static str, Box<dyn CustomShapeFactory>>,
}

impl ShapeRegistry {
    /// Registra una forma personalizada
    pub fn register<T: CustomShape + Default + 'static>(&mut self) {
        let factory = Box::new(T::default());
        self.shapes.insert(T::TYPE, factory);
    }

    /// Crea una forma por tipo
    pub fn create(&self, type_name: &str, id: EntityId) -> Option<Box<dyn Shape>> {
        self.shapes.get(type_name).and_then(|f| f.create(id))
    }
}
```

---

## 8. Roadmap de Implementación

### 8.1 Fase 1: Core SDK (Semanas 1-3)

```
PRIORIDAD ALTA - Funcionalidad básica
──────────────────────────────────────────────────────────────
□ Editor trait básico en Rust
□ Implementación WASM del trait
□ Tipos TypeScript correspondientes
□ Shape abstraction (Rectangle, Ellipse, Line)
□ Selection system básico
□ Viewport/Zoom/Pan
□ Command pattern para undo/redo
□ Keyboard shortcuts (parcial)
□ Tests unitarios core
□ Documentación API
```

### 8.2 Fase 2: Input & Events (Semanas 4-5)

```
PRIORIDAD ALTA - Sistema de eventos
──────────────────────────────────────────────────────────────
□ Mouse input handling completo
□ Keyboard shortcuts completos
□ Touch gestures básicos
□ Event system observable
□ Plugin architecture básica
□ Selection manager avanzado (multi-select, box select)
□ Tests de integración input
□ Ejemplos básicos
```

### 8.3 Fase 3: React Integration (Semanas 6-7)

```
PRIORIDAD MEDIA - Integración UI
──────────────────────────────────────────────────────────────
□ React component <ArchFlowCanvas />
□ React hooks (useEditor, useSelection, useShapes)
□ TypeScript definitions completas
□ Ejemplos React
□ Documentación con Storybook
□ Performance benchmarks
```

### 8.4 Fase 4: Advanced Features (Semanas 8-10)

```
PRIORIDAD BAJA - Funcionalidades avanzadas
──────────────────────────────────────────────────────────────
□ Custom shapes registration
□ Plugin system completo
□ Transform handles (resize, rotate)
□ Grid y snap-to-grid
□ Alignment tools
□ Collaboration client bindings
□ CRDT store integration
```

---

## 9. Métricas de Éxito

### 9.1 Criterios Técnicos

| Métrica | Target | Medición |
|---------|--------|----------|
| API surface | < 50 methods | Count |
| Type coverage | 100% TS types | Coverage report |
| Bundle size | < 50KB gzipped | Build output |
| WASM size | < 200KB | Build output |
| Render FPS | 60 @ 1080p | Chrome devtools |
| Memory usage | < 50MB idle | Performance tab |

### 9.2 Criterios de UX Developer

| Métrica | Target | Medición |
|---------|--------|----------|
| Time to "hello world" | < 5 min | User testing |
| API discoverability | "Intuitive" > 80% | Survey |
| Documentation clarity | 5/5 avg rating | Review |
| Error messages | Actionable > 90% | Code review |

---

## 10. Referencias

- [Figma Plugin API Documentation](https://www.figma.com/plugin-docs/)
- [tldraw SDK Source Code](https://github.com/tldraw/tldraw)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
- [Yjs CRDT](https://github.com/yjs/yjs)

---

*Documento creado como parte del análisis de arquitectura SDK*
*Versión: 0.1.0*
*Última actualización: 2024*
