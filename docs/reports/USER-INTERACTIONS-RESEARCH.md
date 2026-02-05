# ArchFlow User Interactions Research Report

## Análisis Comparativo de Patrones de Interacción en Herramientas de Canvas

**Fecha**: 2026-02-02  
**Autor**: Architecture Team  
**Versión**: 1.0  
**Estado**: Completed

---

## 1. Resumen Ejecutivo

Este reporte documenta la investigación exhaustiva de patrones de interacción usuario en aplicaciones de canvas de referencia: **Figma**, **tldraw**, **draw.io** y **Excalidraw**. El objetivo es identificar los patrones comunes y únicos para informar el diseño del SDK de ArchFlow con Logic Bricks.

### Hallazgos Clave

| Patrón | Herramientas | Observaciones |
|--------|--------------|---------------|
| **State Machine para tools** | tldraw | Cada tool es un StateNode con child states |
| **Selection por marquee** | Figma, tldraw, draw.io | Shift/Ctrl para modificar comportamiento |
| **Transform handles** | Figma, draw.io | 8 handles + rotate handle |
| **Snap to grid/guides** | draw.io, Figma | Guides visuales durante drag |
| **Undo/Redo colaborativo** | Excalidraw | CRDT para conflictos |
| **Gesture-based panning** | tldraw, Excalidraw | Space+drag, middle click |

---

## 2. Figma: Análisis de Interacciones

### 2.1 Sistema de Selección

Figma implementa uno de los sistemas de selección más sofisticados de la industria.

#### 2.1.1 Métodos de Selección

```
┌────────────────────────────────────────────────────────────────────┐
│                      MÉTODOS DE SELECCIÓN                          │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Click Simple                                                      │
│  ─────────────                                                     │
│  • Click en objeto → Selecciona objeto, deselecciona otros        │
│  • Click en área vacía → Deselecciona todo                        │
│                                                                    │
│  Multi-Select (Add)                                                │
│  ───────────────────                                               │
│  • Shift + Click → Añade a selección actual                       │
│  • Cmd/Ctrl + Click → Alterna selección (Figma usa Cmd/Ctrl)      │
│                                                                    │
│  Multi-Select (Remove)                                             │
│  ──────────────────────                                            │
│  • Cmd/Ctrl + Click en seleccionado → Quita de selección          │
│                                                                    │
│  Selection Anidada                                                 │
│  ───────────────────                                               │
│  • Cmd/Ctrl + Click + Drag → Atraviesa grupos para seleccionar    │
│    elementos anidados sin desagrupar                              │
│                                                                    │
│  Marquee Selection                                                  │
│  ──────────────────                                                │
│  • Click + Drag → Crea rectángulo de selección                    │
│  • Shift + Click + Drag → Añade al marquee a selección existente  │
│  • Cmd/Ctrl + Click + Drag → Quita del marquee de selección       │
│                                                                    │
│  Select All                                                        │
│  ──────────                                                        │
│  • Cmd/Ctrl + A → Selecciona todos los objetos visibles           │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

#### 2.1.2 Modificadores de Selección

| Modificador | Mac | Windows | Efecto |
|-------------|-----|---------|--------|
| Add to selection | ⌘ | Ctrl | Alterna selección del objeto |
| Remove from selection | ⌥ + ⌘ | Alt + Ctrl | Quita de selección |
| Select nested | N/A | Ctrl | Atraviesa grupos |
| Marquee add | Shift | Shift | Añade al marquee |
| Marquee remove | ⌘ | Ctrl | Quita del marquee |

### 2.2 Sistema de Transformación

#### 2.2.1 Handles de Transformación

```
                    ┌───────────────────────────┐
                    │         RESIZE            │
                    │    ┌─────────────────┐    │
                    │    │        ▲        │    │
                    │    │   NW──┼──NE    │    │
                    │    │ ◄──┼──┼──►    │    │
                    │    │   SW──┼──SE    │    │
                    │    │        ▼        │    │
                    │    └─────────────────┘    │
                    │                           │
                    │      ROTATE HANDLE        │
                    │           ↻              │
                    │    ┌─────────────────┐    │
                    │    │                 │    │
                    │    │   OBJETO        │    │
                    │    │                 │    │
                    │    │                 │    │
                    │    └─────────────────┘    │
                    └───────────────────────────┘
```

#### 2.2.2 Comportamiento de Resize

| Modificador | Efecto | Comentario |
|-------------|--------|------------|
| **Sin modificador** | Proportional resize | Mantiene aspect ratio |
| **Shift** | Free resize | Sin constraints |
| **Alt/Option** | Centered resize | Expande desde centro |
| **Cmd/Ctrl** | Resize from center | Similar a Alt |
| **Shift + Alt** | Proportional + Centered | Ambas transformaciones |

#### 2.2.3 Comportamiento de Rotate

| Modificador | Efecto | Paso |
|-------------|--------|------|
| **Sin modificador** | Rotación libre | Continua |
| **Shift** | Rotación constrained | 15° increments |
| **Escape** | Cancela operación | Regresa a estado original |

### 2.3 Herramientas de Creación

| Herramienta | Atajo | Descripción |
|-------------|-------|-------------|
| Frame | F | Crea frames con dimensiones preset |
| Frame (arbitrary) | A | Frame con dimensiones arbitrarias |
| Rectangle | R | Rectángulos con rounded corners |
| Ellipse | O | Elipses y círculos perfectos (Shift) |
| Line | L | Líneas y conectores |
| Arrow | P | Flechas con punta configurable |
| Pen | P (switch) | Drawing vectorial |
| Text | T | Tipografías con estilos |
| Hand | H | Pan del canvas |
| Comment | C | Comentarios anclados |

### 2.4 Auto Layout y Constraints

```
CONSTRAINTS (Herencia):
┌─────────────────────────────────────────────────────────┐
│  Parent Frame                                            │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Child Element                    ┌───────────┐  │    │
│  │  ───────────────────────         │  Resize   │  │    │
│  │  Left    = Anchored to left      │  behavior │  │    │
│  │  Right   = Anchored to right     │  when     │  │    │
│  │  Top     = Anchored to top       │  parent   │  │    │
│  │  Bottom  = Anchored to bottom    │  resizes  │  │    │
│  │  Center  = Centered              └───────────┘  │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘

AUTO LAYOUT (Flexbox-like):
┌─────────────────────────────────────────────────────────┐
│  Frame (Auto Layout)                                     │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Item 1  [gap]  Item 2  [gap]  Item 3          │    │
│  │  Direction: Horizontal | Vertical              │    │
│  │  Alignment: Start | Center | End | Stretch     │    │
│  │  Padding: 16px                                     │    │
│  │  Resize: Hug contents | Fill container | Fixed  │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### 2.5 Atajos de Teclado Esenciales

```
SELECCIÓN Y NAVEGACIÓN:
┌─────────────────────────────────────────────────────────┐
│  Cmd/Ctrl + A         │  Select All                     │
│  Cmd/Ctrl + Click     │  Toggle selection               │
│  Shift + Click        │  Add to selection               │
│  Cmd/Ctrl + D         │  Duplicate                      │
│  Cmd/Ctrl + G         │  Group                          │
│  Cmd/Ctrl + Shift + G │  Ungroup                        │
│  Escape               │  Deselect / Cancel              │
│  Enter                │  Edit text / Enter group        │
└─────────────────────────────────────────────────────────┘

TRANSFORMACIÓN:
┌─────────────────────────────────────────────────────────┐
│  Arrow keys           │  Move 1px                       │
│  Shift + Arrow        │  Move 10px                      │
│  Alt + Drag           │  Duplicate while moving         │
│  R                    │  Resize tool                    │
│  T                    │  Rotate tool                    │
└─────────────────────────────────────────────────────────┘

HERRAMIENTAS:
┌─────────────────────────────────────────────────────────┐
│  V                    │  Move / Select tool             │
│  H                    │  Hand tool (pan)                │
│  F                    │  Frame tool                     │
│  R                    │  Rectangle tool                 │
│  O                    │  Ellipse tool                   │
│  L                    │  Line / Arrow tool              │
│  P                    │  Pen tool                       │
│  T                    │  Text tool                      │
│  C                    │  Comment tool                   │
│  I                    │  Color picker                   │
└─────────────────────────────────────────────────────────┘
```

---

## 3. tldraw: Análisis de Arquitectura

### 3.1 Arquitectura de State Machine

tldraw implementa una arquitectura basada en **StateNode** que es referencia directa para nuestro diseño de Controllers.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ARQUITECTURA DE ESTADOS                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Editor (Root StateNode)                                           │
│  ├── SelectTool                                                     │
│  │   ├── idle                                                       │
│  │   ├── pointing         (hover + ready to interact)              │
│  │   ├── brushing         (marquee selection)                      │
│  │   ├── translating      (moving objects)                         │
│  │   ├── resizing         (resizing objects)                       │
│  │   ├── rotating         (rotating objects)                       │
│  │   ├── stretching       (stretching shapes)                      │
│  │   └── cropping         (cropping images)                        │
│  │                                                                   │
│  ├── DrawTool                                                       │
│  │   ├── idle                                                       │
│  │   ├── drawing                                                    │
│  │   └── rendering                                                  │
│  │                                                                   │
│  ├── ArrowTool                                                      │
│  │   ├── idle                                                       │
│  │   ├── pointing                                                    │
│  │   ├── dragging                                                    │
│  │   └── complete                                                    │
│  │                                                                   │
│  ├── HandTool                                                       │
│  │   └── dragging                                                    │
│  │                                                                   │
│  ├── EraserTool                                                     │
│  │   └── erasing                                                     │
│  │                                                                   │
│  └── ZoomTool                                                       │
│      └── zooming                                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Ciclo de Vida de un StateNode

```typescript
// tldraw StateNode lifecycle (simplificado)
interface StateNode<T = string> {
  // Identificación
  readonly id: string;
  readonly type: T;
  readonly parent: StateNode<T> | null;
  readonly children: Map<string, StateNode<T>>;
  
  // Lifecycle hooks
  onEnter?(prevState: T): void;
  onExit?(nextState: T): void;
  onTransition?(nextState: T): void;
  
  // Event handlers (override estos)
  onPointerDown?(event: PointerEvent): void;
  onPointerMove?(event: PointerEvent): void;
  onPointerUp?(event: PointerEvent): void;
  onKeyDown?(event: KeyboardEvent): void;
  onKeyUp?(event: KeyboardEvent): void;
  onWheel?(event: WheelEvent): void;
  
  // Special events
  onDoubleClick?(event: PointerEvent): void;
  onLongPress?(event: PointerEvent): void;
  onInterrupt?(): void;
  onCancel?(): void;
  onComplete?(): void;
}
```

### 3.3 InputsManager

```
┌─────────────────────────────────────────────────────────────────────┐
│                      INPUTS MANAGER                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Pointer State                                                      │
│  ├── currentScreenPoint: Vec2                                      │
│  ├── currentPagePoint: Vec2                                        │
│  ├── previousScreenPoint: Vec2                                     │
│  ├── previousPagePoint: Vec2                                       │
│  ├── isDown: boolean                                               │
│  ├── downPoint: Vec2                                               │
│  ├── keys: Set<string>                                             │
│  ├── modifiers: { ctrl: bool, shift: bool, alt: bool, meta: bool } │
│  └── type: 'mouse' | 'touch' | 'pen'                              │
│                                                                     │
│  Keyboard State                                                     │
│  ├── keysPressed: Set<string>                                      │
│  ├── keysRecentlyPressed: Set<string>                              │
│  └── lastKeyCode: string                                           │
│                                                                     │
│  Gesture State                                                      │
│  ├── isPinching: boolean                                           │
│  ├── initialPinchCenter: Vec2                                      │
│  ├── pinchScale: number                                            │
│  └── pinchRotation: number                                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.4 Behaviors en tldraw

| Behavior | Descripción | Estados Involucrados |
|----------|-------------|---------------------|
| **Pointing** | Detecta hover y clicks sobre shapes | pointing |
| **Dragging** | Detecta inicio de drag | translating, stretching |
| **Brushing** | Marquee selection | brushing |
| **Resizing** | Resize via handles | resizing |
| **Rotating** | Rotate via handle | rotating |
| **Zooming** | Zoom via wheel/pinch | zooming |
| **Panning** | Pan via middle-click/space | dragging (hand tool) |
| **Erasing** | Click to delete | erasing |

### 3.5 Herramientas Únicas de tldraw

| Herramienta | Atajo | Caso de Uso |
|-------------|-------|-------------|
| **Hand** | H | Pan del canvas infinito |
| **Laser** | L | Presentations, atención visual |
| **Scribble** | S | Dibujo estilo whiteboard |
| **Sticky** | N | Notas adhesivas |
| **Arrow** | A | Conexiones inteligentes |
| **Clip** | C | Recorte de imágenes |
| **Eraser** | E | Eliminación de shapes |
| **Text** | T | Texto inline |

### 3.6 Patrón de Transiciones

```
Ejemplo: SelectTool State Machine
─────────────────────────────────────────────────────────────

idle ─────pointerDown────► pointing ───pointerMove──► brushing
  │                             │                             │
  │                             │                             │
  │                             │                             │
  │                    ┌────────┴────────┐                   │
  │                    │                 │                   │
  │                    ▼                 ▼                   │
  │            ┌──────────────► translating ◄───────────────┘
  │            │                      │
  │            │                      │ pointerUp
  │            │                      ▼
  │            │                   complete
  │            │                      │
  │            │                      ▼
  │            │                   idle
  │            │
  │            │         ┌──────────────────────────┐
  │            │         │                          │
  │            └────────►│ resizing/rotating/stretch│
  │                      │                          │
  │                      │    pointerMove           │
  │                      ▼                          │
  │                   complete ─────────────────────┘
  │                      │
  │                      ▼
  │                   idle
  │
  │  pointerUp (no drag)
  │         │
  │         ▼
  └────► complete
            │
            ▼
          idle
```

---

## 4. draw.io: Análisis de Diagramación

### 4.1 Modificadores de Teclado para Diagramación

```
┌─────────────────────────────────────────────────────────────────────┐
│                    MODIFICADORES PRINCIPALES                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ALT                                                               │
│  ───                                                               │
│  • Alt + Scroll → Zoom in/out                                       │
│  • Alt + Click → Múltiples selecciones no adyacentes               │
│  • Alt + Drag → Resize manteniendo aspect ratio                    │
│                                                                     │
│  CTRL + SHIFT                                                      │
│  ────────────                                                      │
│  • Ctrl + Shift + Scroll → Zoom preciso                            │
│  • Ctrl + Shift + Click → Context menu                             │
│  • Ctrl + Shift + Drag → Resize con grid size                      │
│                                                                     │
│  SHIFT                                                             │
│  ─────                                                             │
│  • Shift + Drag → Proportional resize                              │
│  • Shift + Click + Drag → Marquee selection                        │
│  • Shift + Resize → Mantener proporciones                          │
│                                                                     │
│  CTRL                                                              │
│  ────                                                              │
│  • Ctrl + Drag → Resize desde el centro                            │
│  • Ctrl + Click → Selection toggle                                 │
│  • Ctrl + Scroll → Zoom                                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 Atajos de Teclado Completos

```
SELECCIÓN Y ORGANIZACIÓN:
┌─────────────────────────────────────────────────────────┐
│  Ctrl + A              │  Select All                    │
│  Ctrl + Shift + A      │  Select Vertices               │
│  Ctrl + Shift + I      │  Select Invert                 │
│  Ctrl + Shift + L      │  Select Vertices               │
│  Ctrl + Shift + J      │  Select Edges                  │
│  Ctrl + Click          │  Toggle selection              │
│  Shift + Click         │  Add to selection              │
└─────────────────────────────────────────────────────────┘

ALINEACIÓN Y DISTRIBUCIÓN:
┌─────────────────────────────────────────────────────────┐
│  Ctrl + Shift + L        │  Align Left                  │
│  Ctrl + Shift + R        │  Align Right                 │
│  Ctrl + Shift + T        │  Align Top                   │
│  Ctrl + Shift + B        │  Align Bottom                │
│  Ctrl + Shift + H        │  Align Horizontal Center     │
│  Ctrl + Shift + V        │  Align Vertical Center       │
│  Ctrl + Shift + D        │  Distribute Horizontally     │
│  Ctrl + Shift + I        │  Distribute Vertically       │
└─────────────────────────────────────────────────────────┘

TRANSFORMACIÓN:
┌─────────────────────────────────────────────────────────┐
│  Delete                │  Delete                        │
│  Backspace             │  Delete                        │
│  Ctrl + D              │  Duplicate                     │
│  Ctrl + Shift + D      │  Set Default Style             │
│  Ctrl + Shift + U      │  Ungroup                       │
│  Ctrl + G              │  Group                         │
│  Ctrl + M              │  To Back                       │
│  Ctrl + Shift + M      │  To Front                      │
└─────────────────────────────────────────────────────────┘

HERRAMIENTAS:
┌─────────────────────────────────────────────────────────┐
│  Ctrl + Shift + F        │  Search Shapes               │
│  Ctrl + Shift + K        │  Connectors                  │
│  Ctrl + Shift + P        │  Personal Templates          │
│  Ctrl + 1-9              │  Recent Shapes               │
└─────────────────────────────────────────────────────────┘
```

### 4.3 Gestión de Formas

```
BÚSQUEDA RÁPIDA:
┌─────────────────────────────────────────────────────────┐
│  ┌─────────────────────────────────────────────────┐    │
│  │  Ctrl + Shift + F                              │    │
│  │  ┌───────────────────────────────────────────┐  │    │
│  │  │  🔍 Search shapes...                    │  │    │
│  │  │  ┌─────────────────────────────────────┐  │    │
│  │  │  │ rectangle    │  ×                   │  │    │
│  │  │  │ ellipse      │  ×                   │  │    │
│  │  │  │ cloud        │  ×                   │  │    │
│  │  │  │ database     │  ×                   │  │    │
│  │  │  │ flowchart    │  ×                   │  │    │
│  │  │  └─────────────────────────────────────┘  │    │
│  │  └───────────────────────────────────────────┘    │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 4.4 Alineación y Distribución

```
ALINEACIÓN (6 posiciones):
┌─────────────────────────────────────────────────────────┐
│                    ┌───────────────────┐                │
│                    │   ┌───┐           │                │
│                    │   │ 1 │           │  1. Top       │
│                    │   └───┘           │                │
│                    │                   │                │
│                    │ ┌───┐ ┌───┐       │  2. Middle    │
│                    │ │ 2 │ │   │       │                │
│                    │ └───┘ └───┘       │                │
│                    │                   │                │
│                    │   ┌───┐           │  3. Bottom    │
│                    │   │   │           │                │
│                    │   └───┘           │                │
│                    └───────────────────┘                │
│                                                     ▲   │
│  DISTRIBUCIÓN (Horizontal/Vertical):                │   │
│                                                     └───┘
│  ┌───┐     ┌───┐     ┌───┐     ┌───┐     ┌───┐       │
│  │   │     │   │     │   │     │   │     │   │       │
│  └───┘     └───┘     └───┘     └───┘     └───┘       │
│   ↓         ↓         ↓         ↓         ↓           │
│  Equal spacing (horizontal or vertical)              │
└─────────────────────────────────────────────────────────┘
```

### 4.5 Plantillas de Atajos por Tipo de Diagrama

| Tipo de Diagrama | Atajos Principales |
|------------------|-------------------|
| **Flujo** | A (arrow), R (rectangle), D (diamond) |
| **UML** | Shift + 1-9 (clases, interfaces) |
| **Red** | Server, cloud, database shapes |
| **Mind Map** | N ( sticky), T (text), arrows |

---

## 5. Excalidraw: Análisis de Whiteboard

### 5.1 Herramientas de Dibujo

| Herramienta | Atajo | Características |
|-------------|-------|-----------------|
| **Selection** | V | Selección de elementos |
| **Rectangle** | R | Rectángulos |
| **Ellipse** | O | Elipses y círculos |
| **Arrow** | A | Flechas inteligentes |
| **Line** | S | Líneas rectas |
| **Freehand** | P | Drawing estilo lápiz |
| **Text** | T | Texto editable |
| **Sticky** | N | Notas adhesivas |
| **Hand** | H | Pan del canvas |
| **Zoom** | Z | Zoom in/out |

### 5.2 Pipeline de Stroke Smoothing

```
┌─────────────────────────────────────────────────────────────────────┐
│                  STROKE SMOOTHING PIPELINE                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  RAW INPUT                                                          │
│  (pointer events cada ~8ms)                                         │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  1. SAMPLING & DOWNSAMPLING                                  │   │
│  │     • Elimina puntos redundantes                             │   │
│  │     • Mantiene curvatura relevante                           │   │
│  │     • Threshold configurable                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  2. CURVE FITTING (Catmull-Rom Splines)                      │   │
│  │     • Convierte puntos a curvas suaves                       │   │
│  │     • Control de tensión                                    │   │
│  │     • Eliminación de jitter                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  3. RENDERING                                                │   │
│  │     • Canvas 2D API                                          │   │
│  │     • SVG para export                                        │   │
│  │     • Configuración de estilo (cap, join)                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.3 Configuración de Estilos

```typescript
// Configuración de stroke (Excalidraw)
interface StrokeStyle {
  strokeColor: string;      // "#000000"
  backgroundColor: string;  // "transparent"
  strokeWidth: number;      // 2
  strokeStyle: 'solid' | 'dashed' | 'dotted';
  roughness: number;        // 0-2, estilo "hand-drawn"
  opacity: number;          // 0-1
  sloppiness: number;       // 0-1, variación natural
}

// Roughness levels
const ROUGHNESS_LEVELS = {
  exact: 0,      // Líneas precisas, sin variación
  rough: 1,      // Ligera variación natural
  scribble: 2,   // Estilo sketch/manual
};
```

### 5.4 Sistema de Colaboración

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COLABORACIÓN TIEMPO REAL                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  CRDT (Conflict-free Replicated Data Type)                         │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Elementos replicados:                                       │   │
│  │  • Shapes (rect, ellipse, line, text, arrow)                │   │
│  │  • Cursors (posición, nombre, color)                        │   │
│  │  • Selection (qué user seleccionó qué elementos)            │   │
│  │  • History (undo/redo scopes)                               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  CONFLICT RESOLUTION                                               │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  • Last-writer-wins para posiciones                         │   │
│  │  • Merge automático para propiedades                         │   │
│  │  • Cursor presence sin conflictos                            │   │
│  │  • History scope por sesión                                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.5 Undo/Redo Colaborativo

```
HISTORY SCOPES:
┌─────────────────────────────────────────────────────────┐
│  GLOBAL                                               │
│  • Undo afecta a todos los usuarios                   │
│  • Peligroso en colaboración activa                   │
│                                                        │
│  LOCAL                                                │
│  • Undo solo afecta al usuario actual                 │
│  • Seguro para colaboración                           │
│  • Necesita transformación de operaciones             │
│                                                        │
│  SINGLE_OPERATION                                     │
│  • Undo de operación específica                       │
│  • Requiere identificación única de operación         │
└─────────────────────────────────────────────────────────┘

TRANSFORMACIÓN DE CURSORES:
┌─────────────────────────────────────────────────────────┐
│  Cuando User A hace undo de "move":                    │
│  1. Identificar operaciones de move                    │
│  2. Calcular transformación inversa                    │
│  3. Aplicar a cursores de otros usuarios              │
│  4. Sincronizar posiciones                            │
└─────────────────────────────────────────────────────────┘
```

---

## 6. Análisis Comparativo Consolidado

### 6.1 Matriz de Features por Herramienta

| Feature | Figma | tldraw | draw.io | Excalidraw |
|---------|-------|--------|---------|------------|
| **Selección simple** | ✅ | ✅ | ✅ | ✅ |
| **Multi-selección** | ✅ | ✅ | ✅ | ✅ |
| **Marquee selection** | ✅ | ✅ | ✅ | ✅ |
| **Resize handles** | ✅ | ✅ | ✅ | ✅ |
| **Rotate handle** | ✅ | ✅ | ✅ | ⚠️ |
| **Snap to grid** | ✅ | ✅ | ✅ | ✅ |
| **Smart guides** | ✅ | ✅ | ✅ | ❌ |
| **Auto layout** | ✅ | ❌ | ❌ | ❌ |
| **Constraints** | ✅ | ❌ | ⚠️ | ❌ |
| **Keyboard shortcuts** | ✅ | ✅ | ✅ | ✅ |
| **Zoom/pan** | ✅ | ✅ | ✅ | ✅ |
| **Drawing libre** | ⚠️ | ✅ | ⚠️ | ✅ |
| **Shapes predefinidos** | ✅ | ✅ | ✅ | ✅ |
| **Conexiones** | ⚠️ | ✅ | ✅ | ✅ |
| **Sticky notes** | ❌ | ✅ | ✅ | ✅ |
| **Colaboración** | ✅ | ✅ | ⚠️ | ✅ |
| **Undo/Redo colaborativo** | ⚠️ | ✅ | ❌ | ✅ |

### 6.2 Patrones Comunes Identificados

```
PATRONES UNIVERSALES (todas las herramientas):
┌─────────────────────────────────────────────────────────────┐
│  1. SELECCIÓN                                              │
│     • Click = select                                       │
│     • Shift + Click = add/remove                           │
│     • Drag = marquee selection                             │
│     • Cmd/Ctrl + Click = toggle                            │
│                                                             │
│  2. TRANSFORMACIÓN                                         │
│     • Drag handles = resize                                │
│     • Rotate handle = rotation                             │
│     • Drag centro = move                                   │
│     • Shift = constrain axis                               │
│     • Alt = center from center                             │
│                                                             │
│  3. NAVEGACIÓN                                             │
│     • Wheel = zoom                                         │
│     • Space + Drag = pan                                   │
│     • Middle-click + Drag = pan                            │
│                                                             │
│  4. ACCIONES RÁPIDAS                                       │
│     • Delete = remove                                      │
│     • Escape = cancel/deselect                             │
│     • Cmd/Ctrl + Z = undo                                  │
│     • Cmd/Ctrl + Shift + Z = redo                          │
│     • Cmd/Ctrl + D = duplicate                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 Diferenciadores Únicos por Herramienta

| Herramienta | Diferenciador Principal | Impacto UX |
|-------------|------------------------|------------|
| **Figma** | Auto Layout + Constraints | Diseño UI profesional |
| **tldraw** | State machine architecture | Extensibilidad SDK |
| **draw.io** | Alineación/Distribución precisa | Diagramación técnica |
| **Excalidraw** | Stroke smoothing + Colaboración | Whiteboarding natural |

### 6.4 Modificadores de Teclado Estandarizados

```
MODIFICADORES RECOMENDADOS (CONSOLIDADO):
┌─────────────────────────────────────────────────────────┐
│  MODIFICADOR     │  MAC          │  WIN/LIN           │
├──────────────────┼───────────────┼────────────────────┤
│  Add selection   │  Shift        │  Shift             │
│  Toggle select   │  Cmd/Ctrl     │  Ctrl              │
│  Multi-select    │  Cmd/Ctrl     │  Ctrl              │
│  Clone           │  Alt          │  Alt               │
│  Clone + move    │  Alt + Drag   │  Alt + Drag        │
│  Constrain axis  │  Shift        │  Shift             │
│  Center resize   │  Cmd/Ctrl     │  Ctrl              │
│  Pan             │  Space + Drag │  Space + Drag      │
│  Quick zoom      │  Scroll       │  Scroll            │
│  Cancel          │  Escape       │  Escape            │
└─────────────────────────────────────────────────────────┘
```

---

## 7. Taxonomía de Interacciones para ArchFlow

### 7.1 Interacciones por Fase

| Fase | Interacciones | Descripción |
|------|---------------|-------------|
| **Detección** | hover, enter, leave, move | Detectar posición y estado del cursor |
| **Iniciación** | click, doubleClick, longPress, dragStart | Iniciar interacción |
| **Manipulación** | drag, pinch, wheel, gesture | Manipulación continua |
| **Transacción** | drop, release, complete | Finalizar interacción |
| **Feedback** | preview, snap, guide | Retroalimentación visual |

### 7.2 Interacciones por Objeto

| Objeto | Interacciones |
|--------|---------------|
| **Entity** | Select, Move, Resize, Rotate, Delete, Duplicate, Group |
| **Connection** | Create, Reroute, Delete, Reconnect, Style |
| **Group** | Select, Ungroup, Flatten, Lock, Unlock |
| **Canvas** | Pan, Zoom, Navigate, Fit |
| **Annotation** | Create, Edit, Delete, Pin |

### 7.3 Interacciones por Modificador

| Modificador | Efecto | Prioridad |
|-------------|--------|-----------|
| **Shift** | Add to selection / Constrain | Alta |
| **Ctrl/Cmd** | Toggle / Clone | Alta |
| **Alt/Option** | Duplicate / Free transform | Media |
| **Space** | Temporary pan / Hand tool | Alta |
| **Escape** | Cancel / Deselect | Alta |

---

## 8. Recomendaciones para ArchFlow SDK

### 8.1 Priorización de Behaviors

| Priority | Behavior | Justificación |
|----------|----------|---------------|
| **P0** | SelectBehavior | Fundamental, usado en todo |
| **P0** | TransformBehavior | Move/Resize/Rotate core |
| **P0** | PanBehavior | Navegación básica |
| **P0** | ZoomBehavior | Navegación básica |
| **P1** | CreateShapeBehavior | Creación de contenido |
| **P1** | ConnectBehavior | Diagramas |
| **P1** | KeyboardSensor | Accesos rápidos |
| **P2** | DrawBehavior | Whiteboarding |
| **P2** | AutoLayoutBehavior | Figma-style layouts |
| **P3** | CollaborationBehavior | Multi-user |

### 8.2 Estado de Implementación Sugerido

```
FASE 1 (Core):
  ✅ PointerSensor
  ✅ KeyboardSensor
  ✅ SelectBehavior
  ✅ TransformBehavior
  ✅ PanBehavior
  ✅ ZoomBehavior

FASE 2 (Content):
  ✅ CreateShapeBehavior
  ✅ ConnectBehavior
  ✅ TextBehavior
  ✅ DeleteBehavior

FASE 3 (Advanced):
  ✅ DrawBehavior
  ✅ MarqueeBehavior
  ✅ SnapBehavior
  ✅ GuideBehavior

FASE 4 (Collaboration):
  ✅ CursorBehavior
  ✅ PresenceBehavior
  ✅ CollaborationBehavior

FASE 5 (Professional):
  ✅ AutoLayoutBehavior
  ✅ ConstraintBehavior
  ✅ AlignDistributeBehavior
```

---

## 9. Referencias

### Documentación Oficial
- [Figma Help - Selection](https://help.figma.com/hc/en-us/articles/360040449873-Select-layers-and-objects)
- [Figma Help - Auto Layout](https://help.figma.com/hc/en-us/articles/360040451373-Guide-to-auto-layout)
- [tldraw SDK Docs](https://tldraw.dev/docs/editor)
- [tldraw Input Handling](https://tldraw.dev/sdk-features/input-handling)
- [draw.io Shortcuts](https://www.drawio.com/blog/shortcuts)
- [Excalidraw GitHub](https://github.com/excalidraw/excalidraw)

### Patrones Arquitectónicos
- [RxJS Drag and Drop Patterns](https://www.thisdot.co/blog/how-to-implement-drag-and-drop-using-rxjs)
- [Actor Model in Rust](https://tawandamunongo.dev/posts/2025/02/rust-actors/)
- [Event Bus with Tokio](https://blog.digital-horror.com/blog/event-bus-in-tokio/)

---

**End of Report**

*Este reporte sirve como base para el diseño de la EPIC-WEB-010: ArchFlow SDK con Logic Bricks.*
