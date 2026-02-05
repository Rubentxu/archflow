# Patrones de Interacción de Usuario - ArchFlow Engine

**Versión:** 1.0  
**Fecha:** 2026-01-31  
**Referencias:** Figma, tldraw, Excalidraw, draw.io, Pointer Events API

---

## Tabla de Contenidos

1. [Resumen Ejecutivo](#resumen-ejecutivo)
2. [Patrones de Interacción con Mouse](#patrones-de-interaccion-con-mouse)
3. [Patrones de Interacción con Teclado](#patrones-de-interaccion-con-teclado)
4. [Patrones de Interacción Táctil (Tablet/Touch)](#patrones-de-interaccion-tactil)
5. [State Machine para Herramientas](#state-machine-para-herramientas)
6. [SDK API Design para Developers](#sdk-api-design-para-developers)
7. [Implementación WASM/Rust](#implementacion-wasmrust)

---

## Resumen Ejecutivo

Este documento define los patrones de interacción de usuario para herramientas de diagramación vectorial tipo Figma/tldraw, enfocado en crear un SDK fácil de usar que oculte la complejidad del backend WASM/Rust.

**Principios clave:**
- **State Machine Pattern**: Cada herramienta tiene estados bien definidos (idle, active, dragging, etc.)
- **Pointer Events API**: Unificación de mouse/touch/stylus
- **Gestures Composition**: Combinación de eventos primitivos en gestos complejos
- **Progressive Enhancement**: Fallbacks para dispositivos sin soporte completo
- **Developer Experience First**: API simple y expresiva

---

## Patrones de Interacción con Mouse

### 1.1 Estados del puntero (Pointer States)

```
┌─────────────────────────────────────────────────────────────────┐
│                    POINTER STATE MACHINE                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────┐    pointerdown    ┌──────────┐                  │
│   │  IDLE    │ ─────────────────►│  ACTIVE   │                  │
│   └─────────┘                   └──────────┘                  │
│        ▲                            │                            │
│        │                            ▼                            │
│        │                     pointerup /                         │
│        │                     pointercancel                         │
│        │                            │                            │
│        │                    ┌──────────────┐                   │
│        └───────────────────│  DRAGGING    │◄───────────────┤
│                             └──────────────┘  pointermove   │
│                                                    while down  │
└─────────────────────────────────────────────────────────────────┘
```

**Transiciones:**
- `IDLE → ACTIVE`: `pointerdown` sobre un elemento interactivo
- `ACTIVE → IDLE`: `pointerup` o `pointercancel`
- `ACTIVE → DRAGGING`: `pointermove` con `pointerdown` activo + distancia > threshold
- `DRAGGING → ACTIVE`: `pointermove` vuelve bajo threshold
- `DRAGGING → IDLE`: `pointerup` o `pointercancel`

### 1.2 Patrones de Herramientas Mouse

#### Tool: Select Tool

**Estado: Idle**
```
┌─────────────────────────────────────────────────────────────────┐
│ SELECT TOOL - IDLE                                              │
├─────────────────────────────────────────────────────────────────┤
│  Click en objeto vacío → Start selection box                    │
│  Click en objeto → Select object                               │
│  Shift+Click → Toggle selection                                 │
│  Ctrl+Click → Add to selection                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Estado: Dragging**
```
┌─────────────────────────────────────────────────────────────────┐
│ SELECT TOOL - DRAGGING                                          │
├─────────────────────────────────────────────────────────────────┤
│  Drag sobre objetos → Move selection                            │
│  Drag sobre handle → Resize/Rotate (handle-specific)            │
│  Drag sobre vacío → Pan camera                                  │
│  Alt+Drag → Duplicate selection                                 │
│  Shift+Drag → Constrain movement (horizontal/vertical/45°)      │
└─────────────────────────────────────────────────────────────────┘
```

#### Tool: Draw Tool

**Estado: Idle**
```
┌─────────────────────────────────────────────────────────────────┐
│ DRAW TOOL - IDLE                                                │
├─────────────────────────────────────────────────────────────────┤
│  Click+Drag → Draw freehand shape                              │
│  Double-click → Create text node                                │
│  Right-click → Context menu (shape properties)                  │
└─────────────────────────────────────────────────────────────────┘
```

**Estado: Drawing**
```
┌─────────────────────────────────────────────────────────────────┐
│ DRAW TOOL - DRAWING                                             │
├─────────────────────────────────────────────────────────────────┤
│  pointermove → Add point to current stroke                      │
│  pointerup → Finalize shape                                    │
│  Escape → Cancel current stroke                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Tool: Shape Tool

**Patrón de creación de formas:**
```
┌─────────────────────────────────────────────────────────────────┐
│ SHAPE CREATION SEQUENCE                                        │
├─────────────────────────────────────────────────────────────────┤
│  1. Click en canvas → Set origin point                         │
│  2. Drag → Preview shape with rubber-band                       │
│  3. Release → Finalize shape                                  │
│  4. Shift+Drag → Constrain proportions (1:1, golden ratio)      │
│  5. Alt+Drag → Draw from center                                │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 Modifiers de Mouse (Button States)

**Bitmask de botones:**
```rust
pub struct PointerButtons(u8);

impl PointerButtons {
    pub const PRIMARY: u8   = 0b00000001; // Left click
    pub const SECONDARY: u8 = 0b00000010; // Right click
    pub const AUXILIARY: u8 = 0b00000100; // Middle click
    pub const BACK: u8       = 0b00001000; // Back button
    pub const FORWARD: u8    = 0b00010000; // Forward button
    
    pub fn is_primary(&self) -> bool { self.0 & PRIMARY != 0 }
    pub fn is_secondary(&self) -> bool { self.0 & SECONDARY != 0 }
    pub fn is_middle(&self) -> bool { self.0 & AUXILIARY != 0 }
}
```

**Patrones de uso:**
- **Primary (Left)**: Selección, creación, arrastre
- **Secondary (Right)**: Context menu, pan alternativo
- **Auxiliary (Middle)**: Pan, quick zoom
- **Back/Forward**: Navigate history

### 1.4 Wheel Events

**Patrones de zoom con rueda del mouse:**
```rust
enum WheelMode {
    Zoom { center: Vec2 },
    Pan { delta: Vec2 },
    None,
}

// Zoom hacia el cursor (Figma-style)
fn handle_wheel_zoom(event: WheelEvent, camera: &mut Camera) {
    let zoom_factor = if event.delta_y > 0 { 0.9 } else { 1.1 };
    camera.zoom_to_screen_point(event.position, zoom_factor);
}
```

**Modificadores:**
- `Ctrl+Wheel`: Zoom in/out
- `Shift+Wheel`: Pan horizontal
- `Alt+Wheel`: Zoom horizontal (rare)
- `Wheel` solo: Pan vertical o zoom (configurable)

---

## Patrones de Interacción con Teclado

### 2.1 Shortcuts de Herramientas

**Shortcut numéricos (Figma-style):**
```rust
// Ctrl/Cmd + 1-9 para cambiar de herramienta
const TOOL_SHORTCUTS: &[Tool] = &[
    SELECT,      // 1
    FRAME,        // 2
    SHAPE,        // 3 - Rectangle
    PEN,          // 4 - Freehand
    TEXT,         // 5
    HAND,         // 6 - Pan
    COMMENT,      // 7
    CONNECTOR,    // 8
    EYEDROPPER,   // 9
];
```

### 2.2 Shortcuts de Manipulación

**Transformaciones con teclado:**
```
┌─────────────────────────────────────────────────────────────────┐
│ KEYBOARD MANIPULATION SHORTCUTS                                 │
├─────────────────────────────────────────────────────────────────┤
│  Arrow keys → Nudge selection 1px                              │
│  Shift+Arrows → Nudge 10px                                     │
│  Ctrl+D → Duplicate selection                                  │
│  Ctrl+K → Fill color                                            │
│  Ctrl+B → Border color/width                                    │
│  Ctrl+Shift+M → Merge selected shapes                           │
│  Delete/Backspace → Delete selection                             │
│  Ctrl+Z → Undo                                                  │
│  Ctrl+Shift+Z → Redo                                            │
│  Ctrl+A → Select all in current frame                          │
│  Ctrl+C → Copy                                                 │
│  Ctrl+V → Paste                                                │
│  Ctrl+Shift+V → Paste in place                                 │
│  Ctrl+G → Group selection                                       │
│  Ctrl+Shift+G → Ungroup                                         │
│  Ctrl+R → Rotate selection                                     │
│  Ctrl+Shift+R → Reset rotation                                  │
│  Ctrl+] → Bring forward                                        │
│  Ctrl+[ → Send backward                                       │
│  Ctrl+Shift+] → Bring to front                                 │
│  Ctrl+Shift+[ → Send to back                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Modifiers de Teclado

**Key State Tracking:**
```rust
pub struct KeyboardModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Windows key / Cmd on Mac
    pub caps_lock: bool,
}

impl KeyboardModifiers {
    pub fn from_pointer_event(event: &PointerEvent) -> Self {
        Self {
            shift: event.shiftKey,
            ctrl: event.ctrlKey,
            alt: event.altKey,
            meta: event.metaKey,
            caps_lock: event.getModifierState("CapsLock"),
        }
    }
    
    // Combinaciones útiles
    pub fn is_platform_shortcut(&self) -> bool {
        #[cfg(target_os = "macos")]
        { self.ctrl && !self.alt && !self.shift }
        
        #[cfg(not(target_os = "macos"))]
        { self.ctrl && !self.alt }
    }
}
```

### 2.4 Text Input Shortcuts

**Durante edición de texto:**
```
┌─────────────────────────────────────────────────────────────────┐
│ TEXT EDITING SHORTCUTS                                         │
├─────────────────────────────────────────────────────────────────┤
│  Escape → Finish editing / deselect                             │
│  Enter → Confirm text / add line break                          │
│  Tab → Next text object                                          │
│  Shift+Tab → Previous text object                               │
│  Arrow keys → Move cursor within text                            │
│  Ctrl+A → Select all text                                       │
│  Ctrl+C/V/X → Standard clipboard                                │
│  Ctrl+B → Bold                                                 │
│  Ctrl+I → Italic                                               │
│  Ctrl+U → Underline                                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Patrones de Interacción Táctil (Tablet/Touch)

### 3.1 Pointer Events para Touch

**Unified Pointer API:**
```rust
// Pointer Events unifica mouse, touch, pen
pub enum PointerType {
    Mouse,
    Touch,
    Pen,
}

pub struct PointerEvent {
    pub pointer_id: u32,
    pub pointer_type: PointerType,
    pub pressure: f32,    // 0.0 - 1.0 (pen pressure)
    pub tilt_x: f32,      // -90° to 90° (pen tilt)
    pub tilt_y: f32,      // -90° to 90° (pen tilt)
    pub twist: f32,      // 0° to 359° (pen rotation)
    pub is_primary: bool,
    pub buttons: PointerButtons,
}
```

### 3.2 Multi-Touch Gestures

**State machine para multi-touch:**
```rust
pub struct TouchStateMachine {
    // Track active pointers by pointer_id
    active_pointers: HashMap<u32, TouchPoint>,
    
    // Gesture detection
    gesture_detector: GestureDetector,
}

pub struct TouchPoint {
    pub pointer_id: u32,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
    pub start_time: u64,
}

pub enum DetectedGesture {
    None,
    OneFingerTap,
    TwoFingerTap,
    Pinch { center: Vec2, scale: f32 },
    Pan { delta: Vec2 },
    Rotate { angle: f32 },
}
```

### 3.3 Patrones de Gestos Táctiles

#### Pinch to Zoom (Two-Finger)

**Algoritmo de detección:**
```rust
fn detect_pinch(points: &TouchPoint, touch_point_history: &HashMap<u32, Vec<Vec2>>) -> Option<PinchGesture> {
    if points.len() != 2 {
        return None;
    }
    
    let (p1, p2) = (points[0], points[1]);
    
    // Calcular distancia actual
    let current_distance = p1.current_pos.distance(p2.current_pos);
    
    // Obtener distancias anteriores
    let prev_distances = touch_point_history.get(&p1.pointer_id)
        .zip(touch_point_history.get(&p2.pointer_id))
        .map(|(h1, h2)| h1.last().zip(h2.last())
            .map(|(pos1, pos2)| pos1.distance(pos2)));
    
    if let Some(Some(prev_distance)) = prev_distances {
        let scale = current_distance / prev_distance;
        let center = (p1.current_pos + p2.current_pos) / 2.0;
        
        Some(PinchGesture { center, scale })
    } else {
        None
    }
}
```

#### Pan con dos dedos

**Pan vs Zoom:**
```rust
// Distinguir entre pinch y pan basado en movimiento relativo
fn classify_two_finger_gesture(
    initial_distance: f32,
    current_distance: f32,
    initial_center: Vec2,
    current_center: Vec2,
    threshold: f32,
) -> GestureType {
    let distance_change = (current_distance - initial_distance).abs();
    let center_change = (current_center - initial_center).length();
    
    if center_change > distance_change * threshold {
        GestureType::Pan
    } else {
        GestureType::Pinch
    }
}
```

#### Panning con un dedo (Two-Finger Pan alternative)

**Pan con Space+Drag:**
```rust
// En iPad, Space+Drag simula pan como con mouse middle button
if modifier_keys.space && pointer_type == PointerType::Touch {
    state.transition_to(State::Panning);
}
```

### 3.4 Apple Pencil / Stylus Support

**Pressure sensitivity para drawing:**
```rust
pub struct StylusState {
    pub pressure: f32,        // 0.0 - 1.0
    pub altitude_angle: f32, // 0° - 90° (elevation from surface)
    pub azimuth_angle: f32,  // 0° - 360° (rotation around tip)
}

// Aplicar presión al stroke width
fn compute_stroke_width(base_width: f32, pressure: f32) -> f32 {
    let min_width = base_width * 0.2;
    let max_width = base_width * 2.0;
    min_width + (max_width - min_width) * pressure
}

// Altitude para eraser shortcut
fn is_eraser_mode(stylus: &StylusState) -> bool {
    stylus.altitude_angle < 45.0 // Pencil invertado
}
```

### 3.5 Haptic Feedback

**Patrones de feedback háptico:**
```rust
pub enum HapticPattern {
    LightTap,
    MediumTap,
    HeavyTap,
    Success,
    Failure,
    SelectionChanged,
    SnapToGrid,
}

// Trigger haptic feedback en momentos clave
async fn trigger_haptic(pattern: HapticPattern) {
    match pattern {
        HapticPattern::LightTap => {
            navigator_vibrate(10).await;
        }
        HapticPattern::SnapToGrid => {
            navigator_vibrate(5).await; // Snaps
            sleep(Duration::from_millis(50)).await;
            navigator_vibrate(15).await; // Confirmation
        }
        // ... other patterns
    }
}
```

---

## State Machine para Herramientas

### 4.1 Tool State Machine

**Arquitectura del state machine:**
```rust
pub trait Tool {
    fn on_pointer_down(&mut self, event: &PointerEvent, context: &mut ToolContext) -> ToolTransition;
    fn on_pointer_move(&mut self, event: &PointerEvent, context: &mut ToolContext) -> ToolTransition;
    fn on_pointer_up(&mut self, event: &PointerEvent, context: &mut ToolContext) -> ToolTransition;
    fn on_key_down(&mut self, key: Key, context: &mut ToolContext) -> ToolTransition;
    fn on_key_up(&mut self, key: Key, context: &mut ToolContext) -> ToolTransition;
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
    pub selection: &mut SelectionManager,
    pub command_queue: &mut CommandQueue,
    pub gizmo_renderer: &mut GizmoRenderer,
}
```

### 4.2 Select Tool Implementation

**State machine del Select Tool:**
```rust
pub struct SelectTool {
    state: SelectToolState,
    drag_start: Option<Vec2>,
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
    Rotating { center: Vec2, start_angle: f32, entity: EntityId },
}

impl SelectTool {
    fn on_pointer_down(&mut self, event: &PointerEvent, ctx: &mut ToolContext) -> ToolTransition {
        match self.state {
            SelectToolState::Idle => {
                if let Some(entity) = ctx.hit_test(event.position) {
                    ToolTransition::PushState(Box::new(SelectToolState::Hovering { entity }))
                } else if ctx.modifiers.space {
                    ToolTransition::PushState(Box::new(SelectToolState::Panning { start: event.position }))
                } else {
                    ToolTransition::PushState(Box::new(SelectToolState::Selecting { start: event.position }))
                }
            }
            SelectToolState::Hovering { entity } => {
                if ctx.modifiers.alt {
                    // Duplicate
                    let duplicate_cmd = Command::Duplicate { entities: vec![entity] };
                    ctx.command_queue.push(duplicate_cmd);
                    ToolTransition::PushState(Box::new(SelectToolState::Dragging { 
                        start: event.position, 
                        entities: vec![duplicate_id] 
                    }))
                } else {
                    ToolTransition::PushState(Box::new(SelectToolState::Dragging { 
                        start: event.position, 
                        entities: vec![entity] 
                    }))
                }
            }
            // ... other state transitions
        }
    }
}
```

### 4.3 Tool Manager

**Central tool management:**
```rust
pub struct ToolManager {
    current_tool: Box<dyn Tool>,
    tool_stack: Vec<Box<dyn Tool>>,
    modifier_keys: KeyboardModifiers,
    pointer_state: PointerStateMachine,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            current_tool: Box::new(SelectTool::new()),
            tool_stack: Vec::new(),
            modifier_keys: KeyboardModifiers::empty(),
            pointer_state: PointerStateMachine::Idle,
        }
    }
    
    pub fn set_tool(&mut self, tool: Box<dyn Tool>) {
        // Finalizar tool actual
        self.current_tool.on_cancel(&mut ToolContext::default());
        
        // Activar nueva herramienta
        self.current_tool = tool;
    }
    
    pub fn push_tool(&mut self, tool: Box<dyn Tool>) {
        self.tool_stack.push(self.current_tool);
        self.current_tool = tool;
    }
    
    pub fn pop_tool(&mut self) {
        if let Some(tool) = self.tool_stack.pop() {
            self.current_tool = tool;
        }
    }
}
```

---

## SDK API Design para Developers

### 5.1 Principios de Diseño del SDK

**Objetivos del SDK:**
1. **Simplicidad**: API minimalista que oculta complejidad
2. **Expresividad**: Permite casos de uso avanzados
3. **Type Safety**: Aprovecha el sistema de tipos de Rust
4. **Performance**: Zero overhead abstraction
5. **Ergonomía**: Fluent methods y builders

### 5.2 Arquitectura del SDK

**Diagrama de capas:**
```
┌─────────────────────────────────────────────────────────────────┐
│                     DEVELOPER APPLICATION                       │
│  (React, Vue, Svelte, vanilla JS - cualquier framework)          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                    ┌────────▼─────────┐
                    │   ARCHFLOW SDK    │  ← API fácil de usar
                    └────────┬─────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
   ┌────▼────┐          ┌─────▼────┐        ┌────▼─────┐
   │Pointer  │          │  State    │        │ Render   │
   │ Events  │          │  Machine  │        │ Pipeline │
   └────┬────┘          └─────┬────┘        └────┬─────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             │
                    ┌───────▼───────────┐
                    │   ARCHFLOW WASM    │  ← Core engine
                    │   (Rust Backend)   │
                    └────────────────────┘
```

### 5.3 API Core del SDK

**Inicialización:**
```typescript
// SDK API (TypeScript definitions)
import { ArchFlowEngine, CanvasConfig, ToolType } from '@archflow/sdk';

class ArchFlowSDK {
    private engine: ArchFlowEngine;
    
    constructor(canvas: HTMLCanvasElement, config?: Partial<CanvasConfig>) {
        this.engine = new ArchFlowEngine(canvas, {
            enableGizmos: true,
            enableTouchGestures: true,
            enableKeyboardShortcuts: true,
            ...config
        });
    }
    
    // API de alto nivel
    async loadLibrary(url: string): Promise<Library> {
        return await this.engine.loadLibrary(url);
    }
    
    addShape(shape: ShapeDefinition): EntityId {
        return this.engine.addShape(shape);
    }
    
    getSelection(): Entity[] {
        return this.engine.getSelection();
    }
    
    // Event handling
    onPointerDown(callback: (event: PointerEvent) => void): () => void {
        return this.engine.on('pointerdown', callback);
    }
    
    onSelectionChange(callback: (selection: Entity[]) => void): () => void {
        return this.engine.on('selectionchange', callback);
    }
    
    // Viewport control
    zoomTo(center: Point, zoom: number): void {
        this.engine.setCamera({ center, zoom });
    }
    
    fitToView(entities?: Entity[]): void {
        this.engine.fitToView(entities);
    }
    
    // Serialization
    serialize(): Uint8Array {
        return this.engine.serializeProject();
    }
    
    deserialize(data: Uint8Array): void {
        this.engine.loadProject(data);
    }
    
    // Lifecycle
    start(): void {
        this.engine.start();
    }
    
    stop(): void {
        this.engine.stop();
    }
    
    destroy(): void {
        this.engine.destroy();
    }
}
```

### 5.4 Fluent API para Operaciones Comunes

**Builder patterns para shapes:**
```typescript
// Fluent builder para crear shapes
const entityId = sdk.createShape({
    type: 'rectangle',
    position: { x: 100, y: 100 },
    size: { width: 200, height: 100 },
    style: {
        fill: '#FF0000',
        stroke: '#000000',
        strokeWidth: 2,
    },
    text: 'My Shape',
    cornerRadius: 8,
    rotation: 45,
});

// Alternative fluent API
sdk.createShape()
    .rectangle()
    .at(100, 100)
    .size(200, 100)
    .fill('#FF0000')
    .stroke('#000000', 2)
    .text('My Shape')
    .cornerRadius(8)
    .rotation(45)
    .build();
```

### 5.5 Event System

**Tipos de eventos:**
```typescript
interface ArchFlowEvents {
    // Pointer events
    'pointerdown': PointerEvent;
    'pointermove': PointerEvent;
    'pointerup': PointerEvent;
    'pointercancel': PointerEvent;
    
    // Selection events
    'selectionchange': SelectionChangeEvent;
    'hoverchange': HoverChangeEvent;
    
    // Entity events
    'entityadd': EntityEvent;
    'entitychange': EntityChangeEvent;
    'entityremove': EntityEvent;
    
    // Viewport events
    'viewportchange': ViewportChangeEvent;
    'zoom': ZoomEvent;
    
    // Tool events
    'toolchange': ToolChangeEvent;
    'gesturestart': GestureEvent;
    'gestureend': GestureEvent;
    
    // Keyboard events
    'keydown': KeyboardEvent;
    'keyup': KeyboardEvent;
    
    // Lifecycle events
    'ready': () => void;
    'beforeframe': () => void;
    'afterframe': () => void;
    'destroy': () => void;
}
```

**Ejemplo de uso:**
```typescript
const sdk = new ArchFlowSDK(canvas);

// Suscribir a eventos
sdk.on('pointerdown', (event) => {
    console.log('Pointer down at:', event.position);
});

sdk.on('selectionchange', (event) => {
    console.log('Selection changed:', event.selection);
});

const unsubscribe = sdk.on('zoom', (event) => {
    console.log('Zoom changed to:', event.zoom);
});

// Cleanup
unsubscribe();
```

### 5.6 Plugin System

**Arquitectura de plugins:**
```typescript
interface Plugin {
    name: string;
    version: string;
    
    install(sdk: ArchFlowSDK): void;
    uninstall(sdk: ArchFlowSDK): void;
    
    // Hooks
    onPointerDown?(event: PointerEvent, ctx: PluginContext): void;
    onPointerMove?(event: PointerEvent, ctx: PluginContext): void;
    onPointerUp?(event: PointerEvent, ctx: PluginContext): void;
    
    // Custom tools
    tools?: ToolDefinition[];
    
    // Custom shapes
    shapeRenderers?: ShapeRendererMap;
}

// Ejemplo: Plugin de diagramas C4
class C4DiagramPlugin implements Plugin {
    name = 'c4-diagram';
    version = '1.0.0';
    
    install(sdk: ArchFlowSDK) {
        // Registrar herramientas custom
        sdk.registerTool('person', new PersonTool());
        sdk.registerTool('database', new DatabaseTool());
        sdk.registerTool('system', new SystemTool());
        
        // Registrar renderers custom
        sdk.registerShapeRenderer('person', renderPersonIcon);
    }
}

// Usar plugin
sdk.use(new C4DiagramPlugin());
```

---

## Implementación WASM/Rust

### 6.1 Boundary Layer (JavaScript ↔ WASM)

**SharedArrayBuffer Layout:**
```rust
// Memory layout para comunicación lock-free
#[repr(C)]
pub struct SharedBufferLayout {
    // Header (read-write)
    pub head: AtomicU32,     // JS write → WASM read
    pub tail: AtomicU32,     // WASM write → JS read
    
    // Event buffer (ring buffer)
    pub events: [RawInputEvent; EVENT_CAPACITY],
}

// JavaScript side
const SHARED_BUFFER = new SharedArrayBuffer(
    4 + 4 + (EVENT_SIZE * EVENT_CAPACITY)
);

const inputView = new Int32Array(SHARED_BUFFER);
const eventView = new Uint8Array(SHARED_BUFFER, 8, EVENT_SIZE * EVENT_CAPACITY);

// JS escribe eventos
function writeEvent(event) {
    const head = inputView[0];
    const next = (head + 1) % EVENT_CAPACITY;
    
    // Write event data
    const offset = 8 + (head * EVENT_SIZE);
    eventView.set(encodeEvent(event), offset);
    
    // Update head
    inputView[0] = next;
}

// WASM lee eventos
pub fn drain_input_events(&self) -> Vec<RawInputEvent> {
    let head = self.buffer.head.load(Ordering::Acquire);
    let tail = self.buffer.tail.load(Ordering::Acquire);
    
    let mut events = Vec::new();
    
    while head != tail {
        let event = unsafe {
            (self.buffer.events.as_ptr().add(tail as usize) as *const RawInputEvent)
                .read()
        };
        
        events.push(event);
        tail = (tail + 1) % EVENT_CAPACITY;
    }
    
    self.buffer.tail.store(tail, Ordering::Release);
    events
}
```

### 6.2 Exposing SDK API

**WASM bindings con wasm-bindgen:**
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ArchFlowEngine {
    camera: Camera,
    store: EntityStore,
    renderer: GpuRenderer,
    tool_manager: ToolManager,
    // ... other fields
}

#[wasm_bindgen]
impl ArchFlowEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, config: JsValue) -> Result<ArchFlowEngine, JsValue> {
        // Initialize engine
        Ok(ArchFlowEngine {
            // ... initialize components
        })
    }
    
    #[wasm_bindgen(method, getter)]
    pub fn selection(&self) -> JsValue {
        // Convert EntityId Vec to JS array
        serde_wasm_bindgen::to_value(&self.get_selection())
            .unwrap_or_else(|_| JsValue::undefined())
    }
    
    #[wasm_bindgen]
    pub fn add_shape(&mut self, shape: JsValue) -> Result<u32, JsValue> {
        // Parse shape from JS
        let shape_def: ShapeDefinition = serde_wasm_bindgen::from_value(shape)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Create entity
        let id = self.store.spawn(shape_def.position, shape_def.size);
        
        Ok(id.index().0) // Return entity index
    }
    
    #[wasm_bindgen]
    pub fn on_pointer_event(&mut self, event: JsValue) {
        let input_event: RawInputEvent = serde_wasm_bindgen::from_value(event)
            .expect("Invalid pointer event");
        
        // Process through tool manager
        self.tool_manager.handle_input(input_event, &mut self.context);
    }
    
    #[wasm_bindgen]
    pub fn start(&mut self) {
        // Start render loop
    }
    
    #[wasm_bindgen]
    pub fn stop(&mut self) {
        // Stop render loop
    }
}
```

### 6.3 Event Writer Pattern

**Output events from WASM to JS:**
```rust
// Event queue for WASM → JS communication
pub struct EventWriter {
    callback_id: Option<u32>,
    pending_events: VecDeque<OutputEvent>,
}

impl EventWriter {
    pub fn emit(&mut self, event: OutputEvent) {
        self.pending_events.push_back(event);
    }
    
    pub fn flush(&mut self) -> Option<Vec<u8>> {
        if self.pending_events.is_empty() {
            return None;
        }
        
        // Serialize events
        let serialized = bincode::serialize(&self.pending_events)
            .expect("Failed to serialize events");
        
        self.pending_events.clear();
        
        Some(serialized)
    }
}

// JavaScript side
sdk.setEventHandler((events) => {
    for (const [id, data]) of events) {
        const handler = eventHandlers.get(id);
        handler?.(data);
    }
});
```

---

## Checklist de Implementación

### Fase 1: Fundamentos (Week 1-2)
- [ ] Implementar `PointerType` y `PointerButtons`
- [ ] Implementar `KeyboardModifiers` tracking
- [ ] Implementar `PointerStateMachine` básico
- [ ] Implementar `Tool` trait
- [ ] Crear `ToolManager`

### Fase 2: Mouse Interactions (Week 2-3)
- [ ] Implementar `SelectTool` con state machine
- [ ] Implementar `DrawTool` con stroke capture
- [ ] Implementar `ShapeTool` con rubber-band preview
- [ ] Implementar wheel events (zoom/pan)
- [ ] Implementar drag con modifiers

### Fase 3: Keyboard Interactions (Week 3-4)
- [ ] Implementar keyboard shortcuts
- [ ] Implementar tool switching (Ctrl+1-9)
- [ ] Implementar text editing shortcuts
- [ ] Implementar undo/redo (Ctrl+Z/Y)

### Fase 4: Touch/Multi-Touch (Week 4-5)
- [ ] Implementar pinch-to-zoom
- [ ] Implementar two-finger pan
- [ ] Implementar stylus support (pressure, tilt)
- [ ] Implementar haptic feedback

### Fase 5: SDK API (Week 5-6)
- [ ] Diseñar TypeScript API
- [ ] Implementar `ArchFlowSDK` class
- [ ] Implementar fluent builders
- [ ] Implementar event system
- [ ] Implementar plugin system

### Fase 6: WASM Integration (Week 6-7)
- [ ] Implementar SharedArrayBuffer communication
- [ ] Implementar `wasm-bindgen` bindings
- [ ] Implementar `EventWriter` para output
- [ ] Optimizar serialización

### Fase 7: Testing & Docs (Week 7-8)
- [ ] Tests unitarios de cada tool
- [ ] Tests de integración del SDK
- [ ] Documentación de API
- [ ] Ejemplos y tutoriales

---

## Referencias

### Fuentes Consultadas

1. **[Pinch zoom gestures - MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events/Pinch_zoom_gestures)**
   - Documentación oficial de Pointer Events API para gestos de pinch zoom

2. **[Drawing on HTML5 Canvas - StackOverflow](https://stackoverflow.com/questions/45108732/drawing-on-html5-canvas-with-support-for-multitouch-pinch-pan-and-zoom)**
   - Discusión sobre implementación de drawing con multitouch

3. **[Understanding the State Machine Pattern - Medium](https://medium.com/kotlin-android-chronicle/understanding-the-state-machine-pattern-and-how-to-use-it-in-android-development-64e4d9fe3397)**
   - Patrones de state machine para UI interactiva

4. **[The Role of State Machines in Software Development - OnyxGS](https://www.onyxgs.com/blog/role-state-machines-software-development)**
   - Rol de state machines en desarrollo de software

5. **[Rive State Machine Guide](https://rive.app/blog/how-state-machines-work-in-rive)**
   - State machines para animaciones interactivas

6. **[Keyboard Shortcuts - draw.io](https://drawio-app.com/tutorials/shortcuts/)**
   - Referencia de shortcuts en herramientas de diagramación

7. **[Diagram faster using mouse + keyboard - draw.io](https://www.drawio.com/blog/modifier-shortcuts-in-diagrams)**
   - Patrones de uso de mouse + teclado combinados

### Herramientas Analizadas

- **Figma**: Referencia principal para UX de herramientas de diseño
- **tldraw**: Referencia para state machines en herramientas de dibujo
- **Excalidraw**: Referencia para hand-drawing style
- **draw.io**: Referencia para shortcuts de diagramación

---

*Documento generado para ArchFlow Engine v3.7.0*
*Autor: Claude Code - Research & Analysis*
*Fecha: 2026-01-31*
