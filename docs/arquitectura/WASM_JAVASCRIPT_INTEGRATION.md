# Integración WASM-JavaScript: Arquitectura de Whiteboard Real
## Guía Completa de Implementación para Aplicación Interactiva estilo Figma

**Versión**: 1.0  
**Fecha**: 2025-01-31  
**Estado**: En Desarrollo  
**Referencias**: ARQUITECTURA_FINAL_V3.md, LOGIC_BRICKS_FEASIBILITY_STUDY.md, INTERACTION_PATTERNS.md

---

## 📋 Tabla de Contenidos

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Arquitectura de la Integración](#2-arquitectura-de-la-integración)
3. [Estado Actual de la Implementación](#3-estado-actual-de-la-implementación)
4. [Fases de Integración Completadas](#4-fases-de-integración-completadas)
5. [Roadmap de Implementación](#5-roadmap-de-implementación)
6. [Especificaciones Técnicas Detalladas](#6-especificaciones-técnicas-detalladas)
7. [Patrones de Interacción de Usuario](#7-patrones-de-interacción-de-usuario)
8. [Sistema de Herramientas (Tools)](#8-sistema-de-herramientas-tools)
9. [Gestión de Estado Sincronizado](#9-gestión-de-estado-sincronizado)
10. [Sistema de Eventos Bidireccional](#10-sistema-de-eventos-bidireccional)
11. [Rendering Pipeline](#11-rendering-pipeline)
12. [Métricas de Éxito](#12-métricas-de-éxito)

---

## 1. Resumen Ejecutivo

### 1.1 Objetivo

Crear una aplicación whiteboard colaborativa real estilo Figma donde **WASM (Rust) maneja toda la lógica de negocio y estado**, mientras JavaScript proporciona la UI y rendering de Canvas 2D (con migración planificada a WebGPU).

### 1.2 Principios de Diseño

| Principio | Descripción | Implementación |
|-----------|-------------|----------------|
| **Single Source of Truth** | Todo el estado vive en WASM | EntityStore, CommandQueue en Rust |
| **Unidirectional Data Flow** | JS → WASM (comandos), WASM → JS (estado) | SharedArrayBuffer para input |
| **Reactive UI** | React se actualiza cuando cambia WASM | polling o eventos de WASM |
| **Lock-Free Communication** | Sin muteces entre JS y WASM | Atómicos en SharedArrayBuffer |
| **Production Ready** | No stubs, no fallbacks | Todo implementado |

### 1.3 Comparativa con Figma

| Característica | Figma | ArchFlow |
|---------------|-------|----------|
| Rendering Engine | Canvas 2D + WebGL | Canvas 2D → WebGPU (WASM) |
| State Management | C++ + JavaScript Bridge | Rust WASM + SharedArrayBuffer |
| Real-time Collaboration | Operational Transforms | CRDT (Loro) - planificado |
| Extensibility | Plugin System | Logic Bricks + Plugins |
| Performance | Native | Near-native (WASM) |

---

## 2. Arquitectura de la Integración

### 2.1 Diagrama de Capas

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 1: USER INTERFACE                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │   Toolbar    │  │   Sidebar    │  │ Properties   │  │   Header     │   │
│  │  (Tools)     │  │  (Library)   │  │  (Logic)     │  │  (Actions)   │   │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘   │
│         │                 │                 │                 │             │
│  ┌──────▼─────────────────▼─────────────────▼─────────────────▼───────┐   │
│  │                        React App Components                     │   │
│  │  • Canvas.tsx (Rendering + Event Forwarding)                   │   │
│  │  • App.tsx (State Coordination)                                │   │
│  │  • useArchFlowWasm.ts (WASM Loader & API)                       │   │
│  └────────────────────────────────┬────────────────────────────────┘   │
└───────────────────────────────────┼─────────────────────────────────────┘
                                    │
                    ┌───────────────▼────────────────┐
                    │     BOUNDARY LAYER (JS)       │
                    │  ┌─────────────────────────┐  │
                    │  │   push_input_event()    │  │
                    │  │   get_entity_*()        │  │
                    │  │   spawn_entity()        │  │
                    │  │   set_*()               │  │
                    │  └─────────────────────────┘  │
                    └───────────────┬────────────────┘
                                    │ SharedArrayBuffer
                    ┌───────────────▼────────────────┐
                    │   BOUNDARY LAYER (WASM/Rust)   │
                    │  ┌─────────────────────────┐  │
                    │  │   WasmBridge            │  │
                    │  │   • Entity accessors    │  │
                    │  │   • Command queue       │  │
                    │  │   • Input processor     │  │
                    │  └─────────────────────────┘  │
                    └───────────────┬────────────────┘
                                    │
┌───────────────────────────────────▼─────────────────────────────────────┐
│                           LAYER 2: ENGINE CORE (WASM)                   │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     ArchFlowEngine                               │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │   │
│  │  │ EntityStore  │  │  Camera      │  │ SpatialHash  │          │   │
│  │  │ (SoA Layout) │  │  (Zoom/Pan)  │  │ (O(1) Query) │          │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘          │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │   │
│  │  │CommandQueue  │  │ConnectionStr. │  │ LogicMapping │          │   │
│  │  │(Undo/Redo)   │  │(Anchors)     │  │(Sensors)     │          │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘          │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼─────────────────────────────────────┐
│                        LAYER 3: RENDERING (Future)                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                   GpuRenderer (WebGPU)                          │   │
│  │  • Multi-phase instancing                                      │   │
│  │  • MTSDF text rendering                                        │   │
│  │  • 100k entities @ 60FPS                                       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Flujo de Datos

```
USER INTERACTION
       │
       ▼
┌──────────────┐
│ JavaScript   │
│ Event Handler│
└──────┬───────┘
       │
       │ 1. push_input_event(type, x, y, buttons, modifiers)
       ▼
┌──────────────────────────────────────┐
│  SharedArrayBuffer (Lock-Free)       │
│  • InputRingBuffer                   │
│  • Atomic head/tail pointers         │
│  • Zero-copy data transfer           │
└──────┬───────────────────────────────┘
       │
       │ 2. WASM drains events each tick
       ▼
┌──────────────────────────────────────┐
│  ArchFlowEngine::tick()             │
│  • Process input events             │
│  • Execute commands                 │
│  • Update entity state              │
│  • Update spatial index             │
│  • Evaluate Logic Bricks sensors    │
└──────┬───────────────────────────────┘
       │
       │ 3. JavaScript polls for state changes
       ▼
┌──────────────────────────────────────┐
│  get_entity_position_screen()       │
│  get_entity_size_screen()           │
│  get_entity_color_hex()             │
│  get_entity_label()                 │
│  get_alive_entities()               │
└──────┬───────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────┐
│  React State Update                 │
│  • Re-render Canvas                  │
│  • Update Properties Panel          │
│  • Update Sidebar                   │
└──────────────────────────────────────┘
```

---

## 3. Estado Actual de la Implementación

### 3.1 ✅ Componentes Implementados

| Componente | Archivo | Estado | Descripción |
|------------|---------|--------|-------------|
| **WasmBridge** | `crates/archflow-web/src/bridge.rs` | ✅ Completo | 40+ funciones WASM exportadas |
| **Entity Accessors** | `bridge.rs:380-520` | ✅ Completo | get_entity_*, set_* |
| **Input System** | `crates/archflow-web/src/input.rs` | ✅ Completo | SharedArrayBuffer lock-free |
| **Canvas Component** | `Canvas.tsx` | ✅ Parcial | Rendering + eventos básicos |
| **Logic Bricks** | `LogicMappingTableWasm` | ✅ Completo | Sensor-actuator connections |
| **useArchFlowWasm** | `useArchFlowWasm.ts` | ✅ Completo | Hook de carga WASM |

### 3.2 🔨 Componentes en Desarrollo

| Componente | Archivo | Estado | Pendiente |
|------------|---------|--------|-----------|
| **Tool System** | `Toolbar.tsx` | 🔶 UI only | Conectar a WASM ToolManager |
| **Properties Sync** | `PropertiesPanel.tsx` | 🔶 UI only | Sincronizar con EntityStore |
| **Sidebar Sync** | `Sidebar.tsx` | 🔶 Demo data | Mostrar entidades reales |
| **Undo/Redo** | `Toolbar.tsx` | 🔶 Buttons only | Conectar a HistoryManager |
| **Selection** | `Canvas.tsx` | 🔶 Partial | Multi-selección, rect selection |
| **Zoom/Pan** | `Canvas.tsx` | 🔶 Partial | Zoom-to-cursor |

### 3.3 📋 Componentes Planeados

| Componente | Prioridad | Complejidad |
|------------|-----------|-------------|
| **WebGPU Rendering** | Alta | Alta |
| **Multi-touch Gestures** | Media | Media |
| **Text Editing** | Alta | Media |
| **Connection Drawing** | Alta | Media |
| **Drag & Drop Library** | Media | Baja |
| **Real-time Collaboration** | Baja | Alta |

---

## 4. Fases de Integración Completadas

### 4.1 Fase 1: Entity Accessors ✅

**Objetivo**: Exponer datos de entidades desde WASM a JavaScript

**Implementación**:
```rust
// crates/archflow-web/src/bridge.rs

#[wasm_bindgen]
pub fn get_alive_entities() -> Result<Vec<u32>, JsValue> {
    unsafe {
        if let Some(engine) = &ENGINE {
            Ok(engine.store.draw_order[..engine.store.alive_count()].to_vec())
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }
}

#[wasm_bindgen]
pub fn get_entity_position_screen(entity_index: u32) -> Result<js_sys::Array, JsValue> {
    unsafe {
        if let Some(engine) = &ENGINE {
            let idx = entity_index as usize;
            let world_pos = engine.store.pos(idx);
            let (screen_x, screen_y) = engine.world_to_screen(world_pos);
            // ...
        }
    }
}
```

**Uso en JavaScript**:
```typescript
// Canvas.tsx
const fetchEntitiesFromWasm = (): EntityData[] => {
    const aliveEntities = wasm.WasmBridge.get_alive_entities();
    return aliveEntities.map(entityId => ({
        id: entityId,
        x: wasm.WasmBridge.get_entity_position_screen(entityId)[0],
        y: wasm.WasmBridge.get_entity_position_screen(entityId)[1],
        color: wasm.WasmBridge.get_entity_color_hex(entityId),
        // ...
    }));
};
```

### 4.2 Fase 2: Input Event Forwarding ✅

**Objetivo**: Enviar eventos de input desde JavaScript a WASM vía SharedArrayBuffer

**Implementación**:
```rust
// crates/archflow-web/src/bridge.rs

#[wasm_bindgen]
pub fn push_input_event(
    event_type: u8,
    x: f32,
    y: f32,
    buttons: u8,
    modifiers: u8,
) -> Result<(), JsValue> {
    use crate::input::{Buttons, InputEventType, Modifiers, RawInputEvent};
    
    unsafe {
        if let Some(processor) = &mut INPUT_PROCESSOR {
            let input_event_type = match event_type {
                0 => InputEventType::Down,
                1 => InputEventType::Move,
                2 => InputEventType::Up,
                3 => InputEventType::Wheel,
                _ => InputEventType::Move,
            };
            
            let event = RawInputEvent::new(
                0, 0, x, y,
                input_event_type,
                Buttons(buttons),
                Modifiers(modifiers),
            );
            
            processor.buffer().push_event(event);
            Ok(())
        } else {
            Err(JsError::new("Input processor not initialized").into())
        }
    }
}
```

**Uso en JavaScript**:
```typescript
// Canvas.tsx
const handlePointerDown = (e: React.PointerEvent) => {
    const wasm = window.ArchFlowWasm;
    if (wasm) {
        const buttons = (e.buttons & 1) | ((e.buttons & 2) << 1);
        const modifiers = (e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0);
        wasm.WasmBridge.push_input_event(0, x, y, buttons, modifiers); // 0 = Down
    }
    // ... hit testing y selección
};
```

### 4.3 Fase 3: Logic Bricks Integration ✅

**Objetivo**: Conectar el editor de Logic Bricks con LogicMappingTableWasm

**Implementación**:
```typescript
// useArchFlowWasm.ts
export function useArchFlowWasm() {
    // ...
    const createLogicMappingTable = () => {
        if (!window.ArchFlowWasm) {
            throw new Error("WASM not loaded yet");
        }
        return new window.ArchFlowWasm.LogicMappingTableWasm();
    };
    
    return {
        createLogicMappingTable,
        getLogicMappingTable: () => window.ArchFlowWasm?.LogicMappingTableWasm,
        getSensorType: () => window.ArchFlowWasm?.SensorType,
        getActuatorType: () => window.ArchFlowWasm?.ActuatorType,
        getController: () => window.ArchFlowWasm?.Controller,
    };
}
```

---

## 5. Roadmap de Implementación

### 5.1 Timeline de Desarrollo

```
FASE 1: Fundamentos ✅ (Completado)
├─ Entity accessors
├─ Input event forwarding
└─ Logic Bricks básicos

FASE 2: Herramientas Interactivas 🔶 (En Progreso)
├─ Select Tool (drag, resize, rotate)
├─ Pan/Zoom Tool
├─ Shape Tool (rect, circle, diamond)
└─ Connection Tool

FASE 3: Edición Avanzada 📋 (Pendiente)
├─ Text editing in-place
├─ Multi-selection
├─ Copy/Paste/Duplicate
└─ Keyboard shortcuts

FASE 4: Colaboración 📋 (Pendiente)
├─ CRDT integration
├─ Remote cursors
├─ Conflict resolution
└─ Presence indicators

FASE 5: Rendering WebGPU 📋 (Pendiente)
├─ Migrate Canvas 2D → WebGPU
├─ MTSDF text rendering
├─ Icon atlas
└─ 60FPS @ 100k entities
```

### 5.2 Priorización por Valor de Usuario

| Característica | Impacto | Esfuerzo | Prioridad |
|----------------|---------|----------|-----------|
| Drag & Drop entidades | Alta | Media | P0 |
| Zoom-to-cursor | Alta | Baja | P0 |
| Multi-selección | Alta | Media | P0 |
| Undo/Redo funcional | Alta | Media | P0 |
| Editar texto | Alta | Alta | P1 |
| Conexiones magnéticas | Alta | Alta | P1 |
| Copy/Paste | Media | Media | P1 |
| Colaboración real-time | Media | Alta | P2 |
| WebGPU rendering | Baja | Alta | P2 |

---

## 6. Especificaciones Técnicas Detalladas

### 6.1 Sistema de Herramientas (Tool System)

#### 6.1.1 Arquitectura del Tool Manager

```rust
// crates/archflow-interaction/src/tool_manager.rs

pub trait Tool {
    fn on_pointer_down(&mut self, ctx: &mut ToolContext, event: &PointerEvent) -> ToolTransition;
    fn on_pointer_move(&mut self, ctx: &mut ToolContext, event: &PointerEvent) -> ToolTransition;
    fn on_pointer_up(&mut self, ctx: &mut ToolContext, event: &PointerEvent) -> ToolTransition;
    fn on_key_down(&mut self, ctx: &mut ToolContext, key: KeyCode) -> ToolTransition;
    fn on_key_up(&mut self, ctx: &mut ToolContext, key: KeyCode) -> ToolTransition;
}

pub enum ToolTransition {
    None,
    PushState(Box<dyn Tool>),
    ReplaceState(Box<dyn Tool>),
    PopState,
    Complete,
}

pub struct ToolContext {
    pub camera: &mut Camera,
    pub store: &mut EntityStore,
    pub spatial: &mut SpatialHash,
    pub selection: &mut Vec<EntityId>,
    pub command_queue: &mut CommandQueue,
    pub gizmo_renderer: &mut GizmoRenderer,
}

pub struct ToolManager {
    current_tool: Option<Box<dyn Tool>>,
    tool_stack: Vec<Box<dyn Tool>>,
    modifier_keys: KeyboardModifiers,
    pointer_state: PointerState,
}

impl ToolManager {
    pub fn set_tool(&mut self, tool: Box<dyn Tool>) {
        self.current_tool = Some(tool);
    }
    
    pub fn handle_pointer_event(&mut self, ctx: &mut ToolContext, event: &PointerEvent) {
        if let Some(tool) = &mut self.current_tool {
            let transition = match event.event_type {
                InputEventType::Down => tool.on_pointer_down(ctx, event),
                InputEventType::Move => tool.on_pointer_move(ctx, event),
                InputEventType::Up => tool.on_pointer_up(ctx, event),
                _ => ToolTransition::None,
            };
            
            self.apply_transition(ctx, transition);
        }
    }
}
```

#### 6.1.2 Select Tool Implementation

```rust
// crates/archflow-interaction/src/tools/select.rs

pub struct SelectTool {
    state: SelectToolState,
    drag_start: Vec2,
    selection_box_start: Option<Vec2>,
    hovered_entity: Option<EntityId>,
}

pub enum SelectToolState {
    Idle,
    Hovering { entity: EntityId },
    Dragging { start: Vec2, entities: Vec<EntityId> },
    Panning { start: Vec2 },
    Selecting { start: Vec2 },
    Resizing { handle: ResizeHandle, entity: EntityId },
}

impl Tool for SelectTool {
    fn on_pointer_down(&mut self, ctx: &mut ToolContext, event: &PointerEvent) -> ToolTransition {
        let world_pos = ctx.camera.screen_to_world(event.x, event.y);
        
        // Check for resize handle hit
        if let Some(entity) = ctx.selection.last() {
            if let Some(handle) = self.hit_resize_handle(ctx, *entity, world_pos) {
                self.state = SelectToolState::Resizing { handle, entity: *entity };
                return ToolTransition::None;
            }
        }
        
        // Check for entity hit
        let hit = ctx.spatial.query_point(world_pos);
        
        if event.modifiers.is_ctrl_pressed() {
            // Toggle selection
            if let Some(entity) = hit {
                self.toggle_selection(ctx, entity);
            }
        } else {
            // New selection
            ctx.selection.clear();
            if let Some(entity) = hit {
                ctx.selection.push(entity);
                self.state = SelectToolState::Dragging {
                    start: world_pos,
                    entities: vec![entity],
                };
            } else {
                // Start selection box or pan
                if event.modifiers.is_space_pressed() {
                    self.state = SelectToolState::Panning { start: world_pos };
                } else {
                    self.state = SelectToolState::Selecting { start: world_pos };
                    self.selection_box_start = Some(world_pos);
                }
            }
        }
        
        ToolTransition::None
    }
    
    fn on_pointer_move(&mut self, ctx: &mut ToolContext, event: &PointerEvent) -> ToolTransition {
        let world_pos = ctx.camera.screen_to_world(event.x, event.y);
        
        match &mut self.state {
            SelectToolState::Idle => {
                // Update hover
                let hit = ctx.spatial.query_point(world_pos);
                self.hovered_entity = hit;
                
                if let Some(entity) = hit {
                    ctx.gizmo_renderer.draw_resize_hints(entity);
                }
            }
            SelectToolState::Dragging { start, entities } => {
                let delta = world_pos - *start;
                
                for entity in entities {
                    ctx.command_queue.push(Command::Move {
                        id: *entity,
                        delta,
                    });
                }
                
                *start = world_pos;
            }
            SelectToolState::Panning { start } => {
                let delta = world_pos - *start;
                ctx.camera.pan(delta);
                *start = world_pos;
            }
            SelectToolState::Resizing { handle, entity } => {
                let new_size = self.calculate_resize(ctx, *entity, *handle, world_pos);
                ctx.command_queue.push(Command::Resize {
                    id: *entity,
                    size: new_size,
                });
            }
            _ => {}
        }
        
        ToolTransition::None
    }
}
```

#### 6.1.3 Integración JavaScript con Tool System

```typescript
// Canvas.tsx

type ToolType = 'select' | 'pan' | 'draw' | 'shape';

interface ToolState {
    currentTool: ToolType;
    toolStack: ToolType[];
    modifiers: {
        shift: boolean;
        ctrl: boolean;
        alt: boolean;
        space: boolean;
    };
}

const [toolState, setToolState] = useState<ToolState>({
    currentTool: 'select',
    toolStack: [],
    modifiers: { shift: false, ctrl: false, alt: false, space: false }
});

// Tool switching
const switchTool = (tool: ToolType) => {
    setToolState(prev => ({
        ...prev,
        currentTool: tool,
        toolStack: [...prev.toolStack, tool]
    }));
    
    // Notify WASM of tool change
    const wasm = window.ArchFlowWasm;
    if (wasm) {
        wasm.WasmBridge.set_tool(tool);
    }
};

// Keyboard shortcuts for tools
useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
        // Tool shortcuts
        switch (e.key.toLowerCase()) {
            case 'v':
                switchTool('select');
                break;
            case 'h':
            case 'space':
                switchTool('pan');
                break;
            case 'r':
                switchTool('draw');
                break;
            case 'u':
                switchTool('shape');
                break;
        }
        
        // Update modifiers
        setToolState(prev => ({
            ...prev,
            modifiers: {
                shift: e.shiftKey,
                ctrl: e.ctrlKey || e.metaKey,
                alt: e.altKey,
                space: prev.modifiers.space || e.code === 'Space'
            }
        }));
    };
    
    const handleKeyUp = (e: KeyboardEvent) => {
        if (e.code === 'Space' && toolState.currentTool === 'pan') {
            switchTool(toolState.toolStack[toolState.toolStack.length - 2] || 'select');
        }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
        window.removeEventListener('keydown', handleKeyDown);
        window.removeEventListener('keyup', handleKeyUp);
    };
}, [toolState]);
```

### 6.2 Sistema de Selección

#### 6.2.1 Single Entity Selection

```typescript
const handlePointerDown = (e: React.PointerEvent) => {
    const wasm = window.ArchFlowWasm;
    if (!wasm) return;
    
    // Push input event to WASM
    wasm.WasmBridge.push_input_event(0, x, y, buttons, modifiers);
    
    // Hit testing (could be done in WASM)
    const clickedEntity = hitTest(x, y);
    
    if (clickedEntity !== null) {
        if (modifiers.ctrl) {
            // Toggle selection
            if (selectedEntities.includes(clickedEntity)) {
                setSelectedEntities(prev => prev.filter(id => id !== clickedEntity));
                wasm.WasmBridge.set_entity_selected(clickedEntity, false);
            } else {
                setSelectedEntities(prev => [...prev, clickedEntity]);
                wasm.WasmBridge.set_entity_selected(clickedEntity, true);
            }
        } else {
            // New single selection
            setSelectedEntities([clickedEntity]);
            wasm.WasmBridge.clear_selection();
            wasm.WasmBridge.set_entity_selected(clickedEntity, true);
        }
    } else {
        // Clicked on empty space
        if (!modifiers.ctrl) {
            setSelectedEntities([]);
            wasm.WasmBridge.clear_selection();
        }
    }
};
```

#### 6.2.2 Rectangle Selection (Marquee)

```typescript
const [selectionBox, setSelectionBox] = useState<{start: {x: number, y: number}, end: {x: number, y: number}} | null>(null);

const handlePointerMove = (e: React.PointerEvent) => {
    if (toolState.currentTool === 'select' && isDragging && selectionBoxStart) {
        const currentX = e.clientX - rect.left;
        const currentY = e.clientY - rect.top;
        
        setSelectionBox({
            start: selectionBoxStart,
            end: { x: currentX, y: currentY }
        });
        
        // Query entities in selection box
        const wasm = window.ArchFlowWasm;
        if (wasm) {
            const boxEntities = wasm.WasmBridge.query_entities_in_rect(
                selectionBoxStart.x,
                selectionBoxStart.y,
                currentX - selectionBoxStart.x,
                currentY - selectionBoxStart.y
            );
            
            setHoveredEntities(boxEntities);
        }
    }
};

const handlePointerUp = (e: React.PointerEvent) => {
    if (selectionBox && hoveredEntities.length > 0) {
        setSelectedEntities(hoveredEntities);
        
        // Sync to WASM
        const wasm = window.ArchFlowWasm;
        if (wasm) {
            wasm.WasmBridge.clear_selection();
            hoveredEntities.forEach(id => {
                wasm.WasmBridge.set_entity_selected(id, true);
            });
        }
    }
    
    setSelectionBox(null);
    setHoveredEntities([]);
};
```

#### 6.2.3 Rendering Selection Visuals

```typescript
const renderSelection = (ctx: CanvasRenderingContext2D) => {
    selectedEntities.forEach(entityId => {
        const pos = wasm.WasmBridge.get_entity_position_screen(entityId);
        const size = wasm.WasmBridge.get_entity_size_screen(entityId);
        
        // Selection border
        ctx.strokeStyle = "#13b6ec";
        ctx.lineWidth = 2;
        ctx.strokeRect(pos[0] - 2, pos[1] - 2, size[0] + 4, size[1] + 4);
        
        // Resize handles
        const handles = getResizeHandles(pos[0], pos[1], size[0], size[1]);
        handles.forEach(handle => {
            ctx.fillStyle = "#ffffff";
            ctx.fillRect(handle.x - 4, handle.y - 4, 8, 8);
            ctx.strokeStyle = "#13b6ec";
            ctx.strokeRect(handle.x - 4, handle.y - 4, 8, 8);
        });
    });
    
    // Selection box (marquee)
    if (selectionBox) {
        const { start, end } = selectionBox;
        const x = Math.min(start.x, end.x);
        const y = Math.min(start.y, end.y);
        const w = Math.abs(end.x - start.x);
        const h = Math.abs(end.y - start.y);
        
        ctx.fillStyle = "rgba(19, 182, 236, 0.1)";
        ctx.fillRect(x, y, w, h);
        ctx.strokeStyle = "#13b6ec";
        ctx.setLineDash([5, 5]);
        ctx.strokeRect(x, y, w, h);
        ctx.setLineDash([]);
    }
};
```

### 6.3 Sistema de Zoom y Pan

#### 6.3.1 Zoom-to-Cursor (como Figma)

```typescript
const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    
    const wasm = window.ArchFlowWasm;
    if (wasm) {
        // Get current zoom
        const currentZoom = wasm.WasmBridge.get_zoom();
        
        // Calculate new zoom
        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        const newZoom = Math.max(0.25, Math.min(4, currentZoom * zoomFactor));
        
        // Get camera center before zoom
        const cameraCenter = wasm.WasmBridge.get_camera_center();
        
        // Calculate world position of mouse before zoom
        const worldBefore = screenToWorld(mouseX, mouseY, cameraCenter, currentZoom);
        
        // Apply zoom
        wasm.WasmBridge.set_zoom(newZoom);
        
        // Adjust camera to keep mouse over same world position
        const worldAfter = screenToWorld(mouseX, mouseY, cameraCenter, newZoom);
        const adjustment = {
            x: worldBefore.x - worldAfter.x,
            y: worldBefore.y - worldAfter.y
        };
        
        wasm.WasmBridge.set_camera_center(
            cameraCenter[0] + adjustment.x,
            cameraCenter[1] + adjustment.y
        );
        
        setScale(newZoom);
    }
};

function screenToWorld(screenX: number, screenY: number, cameraCenter: number[], zoom: number) {
    const width = canvas.width;
    const height = canvas.height;
    const aspectRatio = width / height;
    const worldWidth = 2 * aspectRatio / zoom;
    const worldHeight = 2 / zoom;
    
    const ndcX = (screenX / width) * 2 - 1;
    const ndcY = 1 - (screenY / height) * 2;
    
    return {
        x: cameraCenter[0] + ndcX * worldWidth / 2,
        y: cameraCenter[1] + ndcY * worldHeight / 2
    };
}
```

#### 6.3.2 Pan con Middle Mouse o Space+Drag

```typescript
const handlePointerMove = (e: React.PointerEvent) => {
    // Pan with middle mouse or Space+drag
    const isPanning = e.buttons === 4 || (e.buttons === 1 && modifiers.space);
    
    if (isPanning && panStart) {
        const dx = e.clientX - panStart.x;
        const dy = e.clientY - panStart.y;
        
        const wasm = window.ArchFlowWasm;
        if (wasm) {
            // Convert screen delta to world delta
            const worldDelta = screenDeltaToWorld(dx, dy);
            
            const cameraCenter = wasm.WasmBridge.get_camera_center();
            wasm.WasmBridge.set_camera_center(
                cameraCenter[0] - worldDelta.x,
                cameraCenter[1] - worldDelta.y
            );
        }
        
        setPanStart({ x: e.clientX, y: e.clientY });
    }
};
```

### 6.4 Sistema de Undo/Redo

#### 6.4.1 HistoryManager en WASM

```rust
// crates/archflow-engine/src/history.rs

pub struct HistoryManager {
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
    max_depth: usize,
}

struct UndoEntry {
    commands: Vec<Command>,
    inverse_commands: Vec<Command>,
}

impl HistoryManager {
    pub fn record(&mut self, commands: Vec<Command>) {
        let inverse_commands: Vec<Command> = commands.iter()
            .map(|cmd| cmd.inverse())
            .collect();
        
        self.undo_stack.push(UndoEntry {
            commands,
            inverse_commands,
        });
        
        self.redo_stack.clear();
        
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }
    
    pub fn undo(&mut self, store: &mut EntityStore) -> Result<(), HistoryError> {
        if let Some(entry) = self.undo_stack.pop() {
            // Apply inverse commands
            for cmd in entry.inverse_commands {
                cmd.execute(store);
            }
            
            self.redo_stack.push(entry);
            Ok(())
        } else {
            Err(HistoryError::NothingToUndo)
        }
    }
    
    pub fn redo(&mut self, store: &mut EntityStore) -> Result<(), HistoryError> {
        if let Some(entry) = self.redo_stack.pop() {
            // Apply original commands
            for cmd in entry.commands {
                cmd.execute(store);
            }
            
            self.undo_stack.push(entry);
            Ok(())
        } else {
            Err(HistoryError::NothingToRedo)
        }
    }
}
```

#### 6.4.2 Integración JavaScript con Undo/Redo

```typescript
// Toolbar.tsx

const [canUndo, setCanUndo] = useState(false);
const [canRedo, setCanRedo] = useState(false);

// Check undo/redo availability
useEffect(() => {
    const checkHistoryAvailability = () => {
        const wasm = window.ArchFlowWasm;
        if (wasm) {
            setCanUndo(wasm.WasmBridge.can_undo());
            setCanRedo(wasm.WasmBridge.can_redo());
        }
    };
    
    const interval = setInterval(checkHistoryAvailability, 100);
    return () => clearInterval(interval);
}, []);

const handleUndo = () => {
    const wasm = window.ArchFlowWasm;
    if (wasm && canUndo) {
        wasm.WasmBridge.undo();
        
        // Refresh entities from WASM
        const entities = fetchEntitiesFromWasm();
        setEntities(entities);
    }
};

const handleRedo = () => {
    const wasm = window.ArchFlowWasm;
    if (wasm && canRedo) {
        wasm.WasmBridge.redo();
        
        // Refresh entities from WASM
        const entities = fetchEntitiesFromWasm();
        setEntities(entities);
    }
};
```

---

## 7. Patrones de Interacción de Usuario

### 7.1 Patrones de Selección

| Patrón | Activación | Comportamiento | WASM API |
|--------|-----------|----------------|----------|
| **Click** | Pointer Down + Up sin mover | Selecciona una entidad | `set_entity_selected(id, true)` |
| **Ctrl+Click** | Click con Ctrl | Toggle selección (add/remove) | `set_entity_selected(id, true/false)` |
| **Drag** | Pointer Down + Move | Mueve entidades seleccionadas | `move_entity(id, dx, dy)` |
| **Shift+Click** | Click con Shift | Selección rango (first-last) | `select_range(first, last)` |
| **Marquee** | Drag en espacio vacío | Selección rectangular | `query_entities_in_rect(x,y,w,h)` |
| **Ctrl+A** | Ctrl + A | Seleccionar todo | `select_all_entities()` |

### 7.2 Patrones de Edición

| Operación | Shortcut | Comportamiento | WASM API |
|-----------|----------|----------------|----------|
| **Duplicar** | Ctrl+D | Copia selección | `duplicate_entities(ids)` |
| **Eliminar** | Delete/Borrar | Borra selección | `despawn_entities(ids)` |
| **Copiar** | Ctrl+C | Copia al clipboard | `copy_entities(ids)` |
| **Pegar** | Ctrl+V | Pega desde clipboard | `paste_entities()` |
| **Cortar** | Ctrl+X | Corta al clipboard | `cut_entities(ids)` |

### 7.3 Patrones de Navegación

| Operación | Activación | Comportamiento | WASM API |
|-----------|-----------|----------------|----------|
| **Zoom In** | Wheel Up / Ctrl++ | Zoom hacia adentro | `set_zoom(zoom * 1.1)` |
| **Zoom Out** | Wheel Down / Ctrl+- | Zoom hacia afuera | `set_zoom(zoom * 0.9)` |
| **Zoom to Fit** | Ctrl+0 / Ctrl+1 | Ajustar a vista | `zoom_to_fit()` |
| **Pan** | Middle Drag / Space+Drag | Mover cámara | `set_camera_center(x, y)` |
| **Center View** | Double Click en fondo | Centrar en punto | `center_view_at(x, y)` |

---

## 8. Sistema de Herramientas (Tools)

### 8.1 Definición de Herramientas

```typescript
// types/tools.ts

export enum ToolType {
    Select = 'select',
    Pan = 'pan',
    Draw = 'draw',
    Shape = 'shape',
    Text = 'text',
    Connection = 'connection',
    Eraser = 'eraser',
}

export interface Tool {
    id: ToolType;
    name: string;
    icon: string;
    cursor: string;
    canEdit: boolean;
    shortcuts: string[];
}

export const TOOLS: Record<ToolType, Tool> = {
    [ToolType.Select]: {
        id: ToolType.Select,
        name: 'Select',
        icon: 'near_me',
        cursor: 'default',
        canEdit: true,
        shortcuts: ['v'],
    },
    [ToolType.Pan]: {
        id: ToolType.Pan,
        name: 'Pan',
        icon: 'pan_tool',
        cursor: 'grab',
        canEdit: false,
        shortcuts: ['h', 'space'],
    },
    [ToolType.Draw]: {
        id: ToolType.Draw,
        name: 'Draw',
        icon: 'edit',
        cursor: 'crosshair',
        canEdit: true,
        shortcuts: ['r'],
    },
    [ToolType.Shape]: {
        id: ToolType.Shape,
        name: 'Shape',
        icon: 'crop_square',
        cursor: 'crosshair',
        canEdit: true,
        shortcuts: ['u'],
    },
    // ...
};
```

### 8.2 Tool Manager en JavaScript

```typescript
// hooks/useToolManager.ts

interface ToolManagerState {
    currentTool: ToolType;
    previousTool?: ToolType;
    toolStack: ToolType[];
    isToolActive: boolean;
}

export function useToolManager() {
    const [toolState, setToolState] = useState<ToolManagerState>({
        currentTool: ToolType.Select,
        toolStack: [ToolType.Select],
        isToolActive: false,
    });
    
    const switchTool = (tool: ToolType) => {
        setToolState(prev => ({
            currentTool: tool,
            previousTool: prev.currentTool,
            toolStack: [...prev.toolStack, tool],
            isToolActive: true,
        }));
        
        // Notify WASM
        const wasm = window.ArchFlowWasm;
        if (wasm) {
            wasm.WasmBridge.set_current_tool(tool);
        }
    };
    
    const pushTool = (tool: ToolType) => {
        setToolState(prev => ({
            ...prev,
            toolStack: [...prev.toolStack, tool],
            currentTool: tool,
        }));
    };
    
    const popTool = () => {
        setToolState(prev => {
            const newStack = [...prev.toolStack];
            newStack.pop();
            const previousTool = newStack[newStack.length - 1] || ToolType.Select;
            
            return {
                ...prev,
                toolStack: newStack,
                currentTool: previousTool,
            };
        });
    };
    
    const temporaryTool = (tool: ToolType) => {
        const originalTool = toolState.currentTool;
        pushTool(tool);
        
        return () => {
            setToolState(prev => {
                const newStack = prev.toolStack.filter(t => t !== tool);
                return {
                    ...prev,
                    toolStack: newStack,
                    currentTool: originalTool,
                };
            });
        };
    };
    
    return {
        currentTool: toolState.currentTool,
        toolStack: toolState.toolStack,
        switchTool,
        pushTool,
        popTool,
        temporaryTool,
    };
}
```

### 8.3 Integración con Canvas

```typescript
// Canvas.tsx

const { currentTool, switchTool, temporaryTool } = useToolManager();

useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
        // Space for temporary pan
        if (e.code === 'Space' && !toolState.space) {
            e.preventDefault();
            const restorePan = temporaryTool(ToolType.Pan);
            setToolState(prev => ({ ...prev, space: true }));
            
            const handleKeyUp = () => {
                restorePan();
                setToolState(prev => ({ ...prev, space: false }));
                window.removeEventListener('keyup', handleKeyUp);
            };
            window.addEventListener('keyup', handleKeyUp);
        }
        
        // Tool shortcuts
        if (!e.repeat && !e.ctrlKey && !e.metaKey && !e.altKey) {
            switch (e.key.toLowerCase()) {
                case 'v':
                    switchTool(ToolType.Select);
                    break;
                case 'h':
                    switchTool(ToolType.Pan);
                    break;
                case 'r':
                    switchTool(ToolType.Draw);
                    break;
                case 'u':
                    switchTool(ToolType.Shape);
                    break;
            }
        }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
}, [temporaryTool]);

const handlePointerDown = (e: React.PointerEvent) => {
    switch (currentTool) {
        case ToolType.Select:
            handleSelectToolDown(e);
            break;
        case ToolType.Pan:
            handlePanToolDown(e);
            break;
        case ToolType.Draw:
            handleDrawToolDown(e);
            break;
        case ToolType.Shape:
            handleShapeToolDown(e);
            break;
    }
};
```

---

## 9. Gestión de Estado Sincronizado

### 9.1 Patrón de Sincronización

```typescript
// hooks/useWasmEntities.ts

export function useWasmEntities() {
    const [entities, setEntities] = useState<EntityData[]>([]);
    const [lastSync, setLastSync] = useState<number>(0);
    
    const syncFromWasm = useCallback(() => {
        const wasm = window.ArchFlowWasm;
        if (!wasm) return;
        
        const newEntities = fetchEntitiesFromWasm();
        setEntities(newEntities);
        setLastSync(Date.now());
    }, []);
    
    // Auto-sync on interval
    useEffect(() => {
        const interval = setInterval(syncFromWasm, 100); // 10 FPS sync
        return () => clearInterval(interval);
    }, [syncFromWasm]);
    
    // Manual sync after commands
    const executeCommand = useCallback((command: () => void) => {
        command();
        syncFromWasm();
    }, [syncFromWasm]);
    
    return {
        entities,
        syncFromWasm,
        executeCommand,
        lastSync,
    };
}
```

### 9.2 Detección de Cambios Optimizada

```rust
// crates/archflow-web/src/bridge.rs

#[wasm_bindgen]
pub fn get_dirty_entities() -> Vec<u32> {
    unsafe {
        if let Some(engine) = &ENGINE {
            let mut dirty = Vec::new();
            
            for &idx in &engine.store.dirty_render.iter().collect::<Vec<_>>() {
                let entity_id = engine.store.draw_order[idx];
                dirty.push(entity_id);
            }
            
            dirty
        } else {
            Vec::new()
        }
    }
}

#[wasm_bindgen]
pub fn clear_dirty_flags() {
    unsafe {
        if let Some(engine) = &mut ENGINE {
            engine.store.clear_dirty_flags();
        }
    }
}
```

```typescript
// Optimized sync - only fetch changed entities
const syncDirtyEntities = useCallback(() => {
    const wasm = window.ArchFlowWasm;
    if (!wasm) return;
    
    const dirtyIds = wasm.WasmBridge.get_dirty_entities();
    
    if (dirtyIds.length === 0) return;
    
    setEntities(prev => {
        const updated = [...prev];
        dirtyIds.forEach(id => {
            const idx = updated.findIndex(e => e.id === id);
            if (idx !== -1) {
                updated[idx] = fetchEntityData(id);
            }
        });
        return updated;
    });
    
    wasm.WasmBridge.clear_dirty_flags();
}, []);
```

---

## 10. Sistema de Eventos Bidireccional

### 10.1 Arquitectura de Eventos

```typescript
// types/events.ts

export enum WasmEventType {
    // From WASM to JS
    EntitySpawned = 'entity_spawned',
    EntityDespawned = 'entity_despawned',
    EntityMoved = 'entity_moved',
    EntitySelected = 'entity_selected',
    EntityDeselected = 'entity_deselected',
    ColorChanged = 'color_changed',
    TextChanged = 'text_changed',
    ConnectionCreated = 'connection_created',
    ConnectionDeleted = 'connection_deleted',
    
    // From JS to WASM (commands)
    SpawnEntity = 'spawn_entity',
    DespawnEntity = 'despawn_entity',
    MoveEntity = 'move_entity',
    SetColor = 'set_color',
    SetText = 'set_text',
    CreateConnection = 'create_connection',
    DeleteConnection = 'delete_connection',
}

export interface WasmEvent {
    type: WasmEventType;
    entityId?: number;
    data?: any;
    timestamp: number;
}

// Event emitter for WASM → JS communication
class WasmEventEmitter {
    private listeners: Map<WasmEventType, Set<(event: WasmEvent) => void>> = new Map();
    
    on(eventType: WasmEventType, callback: (event: WasmEvent) => void): () => void {
        if (!this.listeners.has(eventType)) {
            this.listeners.set(eventType, new Set());
        }
        this.listeners.get(eventType)!.add(callback);
        
        return () => {
            this.listeners.get(eventType)?.delete(callback);
        };
    }
    
    emit(event: WasmEvent) {
        this.listeners.get(event.type)?.forEach(callback => callback(event));
    }
}

export const wasmEventEmitter = new WasmEventEmitter();
```

### 10.2 Polling de Eventos desde WASM

```rust
// crates/archflow-web/src/bridge.rs

pub struct EventQueue {
    events: Vec<WasmEvent>,
}

impl EventQueue {
    pub fn push(&mut self, event: WasmEvent) {
        self.events.push(event);
    }
    
    pub fn drain(&mut self) -> Vec<WasmEvent> {
        core::mem::take(&mut self.events)
    }
}

static mut EVENT_QUEUE: Option<EventQueue> = None;

#[wasm_bindgen]
pub fn drain_events() -> JsValue {
    unsafe {
        if let Some(queue) = &mut EVENT_QUEUE {
            let events = queue.drain();
            
            // Convert to JavaScript array
            let array = js_sys::Array::new();
            for event in events {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"type".into(), &event.type.into()).unwrap();
                js_sys::Reflect::set(&obj, &"entityId".into(), &event.entity_id.into()).unwrap();
                js_sys::Reflect::set(&obj, &"data".into(), &event.data.into()).unwrap();
                js_sys::Reflect::set(&obj, &"timestamp".into(), &event.timestamp.into()).unwrap();
                array.push(&obj);
            }
            
            array.into()
        } else {
            js_sys::Array::new().into()
        }
    }
}
```

```typescript
// hooks/useWasmEvents.ts

export function useWasmEvents() {
    const [events, setEvents] = useState<WasmEvent[]>([]);
    
    useEffect(() => {
        const interval = setInterval(() => {
            const wasm = window.ArchFlowWasm;
            if (!wasm) return;
            
            const rawEvents = wasm.WasmBridge.drain_events();
            const newEvents: WasmEvent[] = [];
            
            for (let i = 0; i < rawEvents.length; i++) {
                newEvents.push({
                    type: rawEvents[i].type,
                    entityId: rawEvents[i].entityId,
                    data: rawEvents[i].data,
                    timestamp: rawEvents[i].timestamp,
                });
            }
            
            if (newEvents.length > 0) {
                setEvents(prev => [...prev, ...newEvents]);
                newEvents.forEach(event => wasmEventEmitter.emit(event));
            }
        }, 16); // 60 FPS polling
        
        return () => clearInterval(interval);
    }, []);
    
    return { events };
}
```

---

## 11. Rendering Pipeline

### 11.1 Canvas 2D Rendering (Actual)

```typescript
// Canvas.tsx - Current rendering

const render = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // Clear
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Grid background
    renderGrid(ctx);
    
    // Connections
    renderConnections(ctx);
    
    // Entities
    entities.forEach(entity => {
        renderEntity(ctx, entity);
    });
    
    // Selection
    renderSelection(ctx);
    
    // Gizmos
    renderGizmos(ctx);
}, [entities, selectedEntities, selectionBox]);

useEffect(() => {
    const animationFrame = requestAnimationFrame(render);
    return () => cancelAnimationFrame(animationFrame);
}, [render]);
```

### 11.2 WebGPU Rendering (Futuro)

```rust
// crates/archflow-render/src/lib.rs

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    shape_pipeline: wgpu::RenderPipeline,
    entity_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
}

impl GpuRenderer {
    pub fn render_frame(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.surface.get_current_texture().unwrap().view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.95,
                        g: 0.95,
                        b: 0.95,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
        });
        
        render_pass.set_pipeline(&self.shape_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.entity_buffer.slice(..));
        render_pass.draw(0..6, 0..instance_count);
        
        drop(render_pass);
        
        self.queue.submit(Some(encoder.finish()));
    }
}
```

```typescript
// Future: WebGPU rendering from JavaScript
const renderWithWebGPU = async () => {
    if (!navigator.gpu) {
        console.error('WebGPU not supported');
        return;
    }
    
    const adapter = await navigator.gpu.requestAdapter();
    const device = await adapter.requestDevice();
    const context = canvasRef.current.getContext('webgpu');
    
    // In the future, rendering will be done entirely in WASM
    // JavaScript will only manage the surface
    const wasm = window.ArchFlowWasm;
    if (wasm && wasm.WasmBridge.render_webgpu) {
        wasm.WasmBridge.render_webgpu(context);
    }
};
```

---

## 12. Métricas de Éxito

### 12.1 Métricas Técnicas

| Métrica | Objetivo | Medición |
|---------|-----------|----------|
| **Tiempo de carga WASM** | < 500ms | Performance API |
| **FPS a 1000 entidades** | > 60 FPS | requestAnimationFrame |
| **Latencia input→render** | < 16ms | Timestamp comparación |
| **Memory footprint** | < 50 MB | performance.memory |
| **WASM binary size** | < 200 KB | File size |
| **Compilación WASM** | < 60s | build time |

### 12.2 Métricas de Usuario

| Métrica | Objetivo | Medición |
|---------|-----------|----------|
| **Time to first interaction** | < 2s | User testing |
| **Selección visual feedback** | < 50ms | Perceptual |
| **Zoom smoothness** | 60 FPS | Frame drops |
| **Undo/Redo response** | < 100ms | Perceptual |
| **Drag precision** | < 1px | User testing |

### 12.3 Checklist de Completitud

- [ ] Entity accessors funcionales
- [ ] Input event forwarding implementado
- [ ] Selección de entidades funcional
- [ ] Drag & drop de entidades
- [ ] Zoom-to-cursor
- [ ] Pan con middle mouse
- [ ] Undo/Redo conectado
- [ ] Properties panel sincronizado
- [ ] Sidebar mostrando entidades reales
- [ ] Logic Bricks editor funcional
- [ ] Multi-selección
- [ ] Marquee selection
- [ ] Resize handles
- [ ] Text editing básico
- [ ] Connection drawing
- [ ] Library drag & drop

---

## Apéndice A: Referencias de API WASM

### A.1 Funciones de Entity

```rust
// Creación
spawn_entity(x: f32, y: f32, width: f32, height: f32) -> u32

// Modificación
move_entity(entity_id: u32, dx: f32, dy: f32) -> Result<()>
set_color(entity_id: u32, r: u8, g: u8, b: u8, a: u8) -> Result<()>
set_shape(entity_id: u32, shape: u8) -> Result<()>
set_label(entity_id: u32, label: &str) -> Result<()>
set_size(entity_id: u32, width: f32, height: f32) -> Result<()>
set_position(entity_id: u32, x: f32, y: f32) -> Result<()>

// Consulta
get_alive_entities() -> Result<Vec<u32>>
get_entity_position_screen(entity_id: u32) -> Result<Array>
get_entity_size_screen(entity_id: u32) -> Result<Array>
get_entity_color_hex(entity_id: u32) -> Result<String>
get_entity_shape(entity_id: u32) -> Result<u8>
get_entity_label(entity_id: u32) -> Result<String>
is_entity_visible(entity_id: u32) -> Result<bool>
is_entity_selected(entity_id: u32) -> Result<bool>

// Eliminación
despawn_entity(entity_id: u32) -> Result<()>
clear() -> Result<()>

// Selección
clear_selection() -> Result<()>
set_entity_selected(entity_id: u32, selected: bool) -> Result<()>
```

### A.2 Funciones de Cámara

```rust
// Zoom
set_zoom(zoom: f32) -> Result<()>
get_zoom() -> Result<f32>

// Pan
set_camera_center(x: f32, y: f32) -> Result<()>
get_camera_center() -> Result<Array>

// Viewport
resize(width: f32, height: f32) -> Result<()>
zoom_to_fit() -> Result<()>
center_view_on(entity_id: u32) -> Result<()>
```

### A.3 Funciones de Input

```rust
// Eventos
push_input_event(event_type: u8, x: f32, y: f32, buttons: u8, modifiers: u8) -> Result<()>

// SharedArrayBuffer (alternativa directa)
get_input_buffer_ptr() -> *mut InputRingBuffer
get_input_buffer_size() -> usize
```

### A.4 Funciones de Historia

```rust
undo() -> Result<()>
redo() -> Result<()>
can_undo() -> bool
can_redo() -> bool
```

### A.5 Funciones de Logic Bricks

```rust
// LogicMappingTable
LogicMappingTableWasm.new() -> LogicMappingTableWasm
add_highlight(entity_id: u32, sensor: SensorType, controller: Controller) -> ()
add_select(entity_id: u32, sensor: SensorType, controller: Controller) -> ()
add_move(entity_id: u32, sensor: SensorType, controller: Controller) -> ()
remove_connection(entity_id: u32, sensor: SensorType) -> ()
has_connection(entity_id: u32, sensor: SensorType) -> bool
connection_count(entity_id: u32) -> usize
clear_entity(entity_id: u32) -> ()
clear() -> ()
```

---

## Conclusión

Este documento establece la hoja de ruta completa para la integración real entre JavaScript y WASM en la aplicación whiteboard estilo Figma. La arquitectura propuesta asegura que:

1. **WASM maneja toda la lógica de negocio** - EntityStore, CommandQueue, SpatialHash, Logic Bricks
2. **JavaScript es una capa delgada** - UI, rendering temporal, event forwarding
3. **Comunicación lock-free** - SharedArrayBuffer con atómicos
4. **Migración incremental** - Canvas 2D → WebGPU sin romper la API
5. **Production ready** - Sin stubs ni fallbacks

La implementación actual (enero 2025) tiene completadas las fases 1-3 (fundamentos, entity accessors, input forwarding, Logic Bricks básicos). Las siguientes fases se enfocarán en herramientas interactivas completas, edición avanzada y finalmente migración a WebGPU.

---

**Documento mantenido por**: ArchFlow Team  
**Última actualización**: 2025-01-31  
**Próxima revisión**: Al completar Fase 4 (Herramientas Interactivas)
