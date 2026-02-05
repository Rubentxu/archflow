# EPIC-WEB-011: ArchFlow Behaviors SDK [L1-17XX]

## API Fluida para User Interactions en Canvas Whiteboard (WASM)

---

## 1. Resumen Ejecutivo

Esta épica define una **capa de abstracción completamente transparente** sobre Logic Bricks para un **canvas whiteboard interactivo en WASM**. El objetivo es que un developer pueda crear entidades interactivas (rectángulos, círculos, path drawing, sticky notes) **sin conocer nada** del sistema Sensor→Controller→Actuator subyacente.

### 1.1 Contexto: Canvas Whiteboard WASM

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// ESTAMOS EN UN CANVAS, NO EN HTML DOM
// ═══════════════════════════════════════════════════════════════════════════════

// ❌ INCORRECTO (pensando en HTML DOM):
entity.onClick();  // No hay click event en DOM

// ✅ CORRECTO (canvas whiteboard):
entity.onClick();  // El canvas detecta click → proyecta a entidad → trigger

// Las entidades son figuras en el canvas:
// - RectShape(x, y, w, h)
// - CircleShape(x, y, radius)  
// - PathShape(points[])
// - TextShape(x, y, text, font)
// - GroupShape(children[])
```

### 1.2 Principio Fundamental: API 100% Abstracta

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// LO QUE EL DEVELOPER ESCRIBE:
// ═══════════════════════════════════════════════════════════════════════════════

// Crear un sticky note interactivo
const note = ArchFlow.createShape('rectangle')
    .position(100, 200)
    .size(200, 150)
    .fillColor(0xFFFF00)  // Amarillo sticky note
    .border(2, 0xCCCCCC)
    .cornerRadius(8)
    .onHover()
    .onClick(() => selectNote())
    .draggable()
    .build();


// ═══════════════════════════════════════════════════════════════════════════════
// LO QUE EL DEVELOPER NUNCA VE (interno, escondido):
// ═══════════════════════════════════════════════════════════════════════════════

// Traducción automática interna:
LogicMappingTable.addConnection(
    note.id,
    SensorType.MouseOver,
    Controller.Direct,
    ActuatorType.Highlight
);
LogicMappingTable.addConnection(
    note.id,
    SensorType.MouseClick,
    Controller.Debounce({ ticks: 2 }),
    ActuatorType.Select
);
HighlightActuator.configure(note.id, { color: 0xFFFF00 });
```

### 1.3 Regla de Oro

> **"Si el developer necesita entender Logic Bricks para usar la API, estamos fallando."**

### 1.4 Comparación de APIs

| Aspecto | API Original | API Fluida Canvas |
|---------|-------------|-------------------|
| **Visibilidad de arquitectura** | Completa | Ninguna |
| **Referencias a LogicMappingTable/SensorType** | Sí | Nunca |
| **Líneas para hover+click en shape** | ~30 | 5 |
| **Curva de aprendizaje** | Media-Alta | Ninguna |
| **Entidades** | Genéricas | Shape-specific |

### 1.5 Objetivos de la Épica

- ✅ API 100% abstracta para canvas whiteboard
- ✅ Shapes específicos (rectangle, circle, path, text, group)
- ✅ Method chaining fluido (`.onHover().onClick().draggable()`)
- ✅ Integración automática con eventos del canvas
- ✅ Templates predefinidos (sticky note, card, node, edge)

---

## 2. Filosofía de Diseño

### 2.1 Niveles de Abstracción

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    NIVELES DE ABSTRACCIÓN CANVAS                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   NIVEL 1: SHAPE BÁSICO                                                     │
│   ═══════════════════════════════════                                       │
│   ArchFlow.createShape('rectangle').position(100, 200).build();             │
│                                                                             │
│   NIVEL 2: SHAPE CON EVENTOS                                                │
│   ════════════════════════════════════════                                  │
│   .onHover().onClick().draggable().build();                                 │
│                                                                             │
│   NIVEL 3: SHAPES PREDEFINIDOS                                              │
│   ═══════════════════════════════════════                                   │
│   .stickyNote().draggable().selectable().build();                           │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   TODOS LOS NIVELES ESCONDEN:                                       │   │
│   │   • SensorType.MouseOver → onHover()                                │   │
│   │   • Controller.Direct → implícito                                   │   │
│   │   • ActuatorType.Highlight → implícito                              │   │
│   │   • Canvas render → shape.fillColor(), shape.border()               │   │
│   │   • Event handling → canvas → entity projection                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Shapes del Canvas

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// SHAPES DISPONIBLES EN EL CANVAS
// ═══════════════════════════════════════════════════════════════════════════════

type ShapeType =
    | 'rectangle'
    | 'circle'
    | 'ellipse'
    | 'path'
    | 'text'
    | 'image'
    | 'group'
    | 'connector';

// Ejemplos:
ArchFlow.createShape('rectangle')   // Rectángulo
ArchFlow.createShape('circle')      // Círculo
ArchFlow.createShape('path')        // Path drawing (pen tool)
ArchFlow.createShape('text')        // Texto (sticky note)
ArchFlow.createShape('image')       // Imagen
ArchFlow.createShape('connector')   // Línea conectora (entre nodos)
ArchFlow.createShape('group')       // Grupo de shapes
```

### 2.3 Principios de Diseño

1. **Ninguna referencia a Logic Bricks** en la API pública
2. **Shapes específicos** para el dominio whiteboard
3. **Propiedades de renderizado** (fillColor, border, shadow), no CSS
4. **Eventos del canvas** proyectados a entidades
5. **Type-safe**: TypeScript con tipos bien definidos

---

## 3. ArchFlow API Principal

### 3.1 Entry Point

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// ARCHFLOW - API PÚBLICA DEL DEVELOPER (CANVAS WHITEBOARD)
// ═══════════════════════════════════════════════════════════════════════════════

export const ArchFlow = {
    // ═══════════════════════════════════════════════════════════════════════════════
    // SHAPE CREATION
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Crear shape con API fluida
    createShape(type: ShapeType): ShapeBuilder {
        return new ShapeBuilder(type);
    },

    /// Crear rectángulo (alias común)
    createRectangle(): ShapeBuilder {
        return new ShapeBuilder('rectangle');
    },

    /// Crear círculo
    createCircle(): ShapeBuilder {
        return new ShapeBuilder('circle');
    },

    /// Crear sticky note (rectángulo amarillo con texto)
    createStickyNote(): ShapeBuilder {
        return new ShapeBuilder('rectangle')
            .fillColor(0xFFFF00)
            .border(1, 0xCCCCCC)
            .cornerRadius(4)
            .shadow('sm')
            .interactive();
    },

    /// Crear nodo de diagrama
    createNode(): ShapeBuilder {
        return new ShapeBuilder('rectangle')
            .fillColor(0xFFFFFF)
            .border(2, 0x2196F3)
            .cornerRadius(8)
            .shadow('md')
            .interactive()
            .selectable();
    },

    /// Crear conector entre nodos
    createConnector(): ShapeBuilder {
        return new ShapeBuilder('connector')
            .stroke(2, 0x666666)
            .lineCap('round');
    },

    /// Crear texto
    createText(content: string): ShapeBuilder {
        return new ShapeBuilder('text')
            .textContent(content)
            .fontFamily('Inter')
            .fontSize(14)
            .fillColor(0x333333);
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // CLONE & MODIFY
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Clonar shape existente
    clone(shape: Shape): ShapeBuilder {
        return new ShapeBuilder(shape.type, shape.clone());
    },

    /// Modificar shape existente
    modify(shape: Shape): ShapeBuilder {
        return new ShapeBuilder(shape.type, shape);
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // BEHAVIORS (Templates Predefinidos)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Obtener template de behavior
    behavior(name: string): BehaviorDefinition | undefined {
        return BehaviorRegistry.get(name);
    },

    /// Listar behaviors disponibles
    behaviors(category?: BehaviorCategory): string[] {
        return category
            ? BehaviorRegistry.listByCategory(category)
            : BehaviorRegistry.list();
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // SCENE MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Escena activa
    scene: CanvasScene,

    /// Obtener shape por ID
    getShape(id: number): Shape | undefined {
        return this.scene.getShape(id);
    },

    /// Obtener shapes por tag
    getShapesByTag(tag: string): Shape[] {
        return this.scene.getShapesByTag(tag);
    },

    /// Obtener shapes por behavior
    getShapesWithBehavior(name: string): Shape[] {
        return this.scene.getShapes().filter(s => s.behavior.has(name));
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // INITIALIZATION
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Inicializar con canvas
    initialize(canvas: HTMLCanvasElement): void {
        this.scene = new CanvasScene(canvas);
        BehaviorRegistry.initDefaults();
        EventBridge.setup(canvas, this.scene);
    },

    /// Iniciar loop de render
    start(): void {
        this.scene.startRenderLoop();
    },

    /// Detener loop
    stop(): void {
        this.scene.stopRenderLoop();
    },
};
```

### 3.2 ShapeBuilder API

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE BUILDER - API principal para crear shapes interactivos
// ═══════════════════════════════════════════════════════════════════════════════

class ShapeBuilder {
    private shape: Shape;
    private behaviorConfigs: Array<[string, object?]> = [];
    private eventHandlers: Map<string, Function[]> = new Map();

    constructor(type: ShapeType, existingShape?: Shape) {
        this.shape = existingShape || ArchFlow.scene.createShape(type);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GEOMETRY - Propiedades geométricas
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Posición (x, y)
    position(x: number, y: number): this {
        this.shape.setPosition(x, y);
        return this;
    }

    x(value: number): this {
        this.shape.setX(value);
        return this;
    }

    y(value: number): this {
        this.shape.setY(value);
        return this;
    }

    /// Tamaño (width, height)
    size(width: number, height: number): this {
        this.shape.setSize(width, height);
        return this;
    }

    width(value: number): this {
        this.shape.setWidth(value);
        return this;
    }

    height(value: number): this {
        this.shape.setHeight(value);
        return this;
    }

    /// Posición y tamaño juntos
    bounds(x: number, y: number, width: number, height: number): this {
        this.shape.setBounds(x, y, width, height);
        return this;
    }

    /// Centro (x, y) - reposiciona para que el centro sea este punto
    center(x: number, y: number): this {
        const w = this.shape.getWidth();
        const h = this.shape.getHeight();
        this.shape.setPosition(x - w / 2, y - h / 2);
        return this;
    }

    /// Rotation (degrees)
    rotation(degrees: number): this {
        this.shape.setRotation(degrees);
        return this;
    }

    /// Escala
    scale(factor: number): this {
        this.shape.setScale(factor);
        return this;
    }

    /// Opacidad (0-1)
    opacity(value: number): this {
        this.shape.setOpacity(value);
        return this;
    }

    /// Z-index (orden de renderizado)
    zIndex(value: number): this {
        this.shape.setZIndex(value);
        return this;
    }

    /// Visible/invisible
    visible(value: boolean): this {
        this.shape.setVisible(value);
        return this;
    }

    /// Locked (no interactivo)
    locked(): this;
    locked(value: boolean): this;

    locked(value?: boolean): this {
        this.shape.setLocked(value !== false);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // RENDER - Propiedades de renderizado (no CSS, son del canvas)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Color de relleno
    fillColor(hex: number): this {
        this.shape.setFillColor(hex);
        return this;
    }

    /// Sin relleno (transparente)
    noFill(): this {
        this.shape.setFillColor(null);
        return this;
    }

    /// Borde
    border(width: number, color: number, style?: string): this {
        this.shape.setBorder(width, color);
        return this;
    }

    /// Sin borde
    noBorder(): this {
        this.shape.setBorder(0, null);
        return this;
    }

    /// Corner radius (solo para rectángulos)
    cornerRadius(radius: number): this {
        this.shape.setCornerRadius(radius);
        return this;
    }

    /// Sombra
    shadow(size: 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl'): this {
        this.shape.setShadow(size);
        return this;
    }

    /// Sombra personalizada
    customShadow(config: ShadowConfig): this {
        this.shape.setCustomShadow(config);
        return this;
    }

    /// Stroke (para líneas y conectores)
    stroke(width: number, color: number): this {
        this.shape.setStroke(width, color);
        return this;
    }

    /// Estilo de línea
    lineCap(style: 'butt' | 'round' | 'square'): this {
        this.shape.setLineCap(style);
        return this;
    }

    lineJoin(style: 'miter' | 'round' | 'bevel'): this {
        this.shape.setLineJoin(style);
        return this;
    }

    dashArray(dash: number[]): this {
        this.shape.setDashArray(dash);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TEXT - Propiedades de texto (solo para TextShape)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Contenido de texto
    textContent(text: string): this {
        if (this.shape.type === 'text') {
            this.shape.setText(text);
        }
        return this;
    }

    /// Color de texto
    textColor(hex: number): this {
        if (this.shape.type === 'text') {
            this.shape.setTextColor(hex);
        }
        return this;
    }

    /// Fuente
    fontFamily(family: string): this {
        if (this.shape.type === 'text') {
            this.shape.setFontFamily(family);
        }
        return this;
    }

    /// Tamaño de fuente
    fontSize(size: number): this {
        if (this.shape.type === 'text') {
            this.shape.setFontSize(size);
        }
        return this;
    }

    /// Peso de fuente
    fontWeight(weight: number | 'normal' | 'bold'): this {
        if (this.shape.type === 'text') {
            this.shape.setFontWeight(weight);
        }
        return this;
    }

    /// Alineación de texto
    textAlign(align: 'left' | 'center' | 'right'): this {
        if (this.shape.type === 'text') {
            this.shape.setTextAlign(align);
        }
        return this;
    }

    /// Alineación vertical
    verticalAlign(align: 'top' | 'middle' | 'bottom'): this {
        if (this.shape.type === 'text') {
            this.shape.setVerticalAlign(align);
        }
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PATH - Propiedades de path (solo para PathShape)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Puntos del path
    points(points: Vec2[]): this {
        if (this.shape.type === 'path') {
            this.shape.setPoints(points);
        }
        return this;
    }

    /// Añadir punto al path
    addPoint(x: number, y: number): this {
        if (this.shape.type === 'path') {
            this.shape.addPoint(x, y);
        }
        return this;
    }

    /// Suavizar path (bezier)
    smooth(): this {
        if (this.shape.type === 'path') {
            this.shape.setSmooth(true);
        }
        return this;
    }

    /// Cerrar path
    closed(): this {
        if (this.shape.type === 'path') {
            this.shape.setClosed(true);
        }
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // CONNECTOR - Propiedades de conector (solo para ConnectorShape)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Conectar dos shapes
    connect(from: Shape, to: Shape): this {
        if (this.shape.type === 'connector') {
            this.shape.setConnection(from, to);
        }
        return this;
    }

    /// Estilo de conector
    connectorStyle(style: 'straight' | 'elbow' | 'curved'): this {
        if (this.shape.type === 'connector') {
            this.shape.setConnectorStyle(style);
        }
        return this;
    }

    /// Marker de inicio
    startMarker(type: 'none' | 'arrow' | 'dot' | 'diamond'): this {
        if (this.shape.type === 'connector') {
            this.shape.setStartMarker(type);
        }
        return this;
    }

    /// Marker de fin
    endMarker(type: 'none' | 'arrow' | 'dot' | 'diamond'): this {
        if (this.shape.type === 'connector') {
            this.shape.setEndMarker(type);
        }
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // MOUSE EVENTS - El más usado
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Highlight al hacer hover
    onHover(): this;
    onHover(config: HoverConfig): this;
    onHoverEnter(handler: (shape: Shape, position: Vec2) => void): this;
    onHoverExit(handler: (shape: Shape, position: Vec2) => void): this;

    onHover(config?: HoverConfig | ((shape: Shape, position: Vec2) => void)): this {
        const hoverConfig: HoverConfig = typeof config === 'object' && config !== null
            ? config
            : { color: 0xFFFF00, opacity: 0.3 };

        this.shape.behavior.add('hoverHighlight', hoverConfig);

        if (typeof config === 'function') {
            this.shape.on('hoverEnter', config);
        }

        return this;
    }

    /// Click izquierdo
    onClick(): this;
    onClick(handler: (shape: Shape, position: Vec2) => void): this;

    onClick(handler?: (shape: Shape, position: Vec2) => void): this {
        this.shape.behavior.add('clickSelect', { mode: 'single' });
        if (handler) {
            this.shape.on('click', handler);
        }
        return this;
    }

    /// Doble click
    onDoubleClick(): this;
    onDoubleClick(handler: (shape: Shape, position: Vec2) => void): this;

    onDoubleClick(handler?: (shape: Shape, position: Vec2) => void): this {
        this.shape.behavior.add('doubleClick');
        if (handler) {
            this.shape.on('doubleClick', handler);
        }
        return this;
    }

    /// Click derecho (context menu)
    onRightClick(): this;
    onRightClick(handler: (shape: Shape, position: Vec2) => void): this;

    onRightClick(handler?: (shape: Shape, position: Vec2) => void): this {
        this.shape.behavior.add('contextMenu');
        if (handler) {
            this.shape.on('rightClick', handler);
        }
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // DRAG & DROP
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Shape arrastrable
    draggable(): this;
    draggable(config: DraggableConfig): this;

    draggable(config?: DraggableConfig): this {
        this.shape.behavior.add('draggable', config);
        return this;
    }

    /// Callback al iniciar arrastre
    onDragStart(handler: (shape: Shape, position: Vec2) => void): this {
        this.shape.on('dragStart', handler);
        return this;
    }

    /// Callback durante arrastre
    onDragMove(handler: (shape: Shape, delta: Vec2) => void): this {
        this.shape.on('dragMove', handler);
        return this;
    }

    /// Callback al finalizar arrastre
    onDragEnd(handler: (shape: Shape, position: Vec2) => void): this {
        this.shape.on('dragEnd', handler);
        return this;
    }

    /// Resizable (cambiar tamaño)
    resizable(): this;
    resizable(config: ResizableConfig): this;

    resizable(config?: ResizableConfig): this {
        this.shape.behavior.add('resizable', config);
        return this;
    }

    /// Target de drop
    droppable(): this;
    droppable(config: DroppableConfig): this;

    droppable(config?: DroppableConfig): this {
        this.shape.behavior.add('droppable', config);
        return this;
    }

    /// Callback al recibir drop
    onDrop(handler: (shape: Shape, source: Shape, position: Vec2) => void): this {
        this.shape.on('drop', handler);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SELECTION
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Seleccionable
    selectable(): this;
    selectable(config: SelectableConfig): this;

    selectable(config?: SelectableConfig): this {
        this.shape.behavior.add('selectable', config);
        return this;
    }

    /// Multiseleccionable
    multiSelectable(): this {
        return this.selectable({ mode: 'multi' });
    }

    /// Selection box
    selectionBox(): this {
        this.shape.behavior.add('selectionBox');
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // KEYBOARD EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Presionar tecla (shape debe estar seleccionado)
    onKeyDown(key: string): this;
    onKeyDown(key: string, handler: (shape: Shape) => void): this;

    onKeyDown(key: string, handler?: (shape: Shape) => void): this {
        this.shape.behavior.add('keyDown', { key, requireSelection: true });
        if (handler) {
            this.shape.on(`keyDown:${key}`, handler);
        }
        return this;
    }

    /// Presionar Delete/Backspace (eliminar)
    onDelete(handler: (shape: Shape) => void): this {
        return this.onKeyDown('Delete', handler);
    }

    /// Presionar Escape (cancelar)
    onEscape(handler: (shape: Shape) => void): this {
        return this.onKeyDown('Escape', handler);
    }

    /// Copiar (Ctrl+C)
    onCopy(handler: (shape: Shape) => void): this {
        return this.onShortcut(['Ctrl', 'C'], handler);
    }

    /// Pegar (Ctrl+V)
    onPaste(handler: (shape: Shape) => void): this {
        return this.onShortcut(['Ctrl', 'V'], handler);
    }

    /// Shortcut (combinación)
    onShortcut(keys: string[], handler: (shape: Shape) => void): this {
        this.shape.behavior.add('shortcut', { keys, requireSelection: true });
        this.shape.on(`shortcut:${keys.join('+')}`, handler);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // TOOLTIP
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Tooltip al hacer hover
    tooltip(content: string): this;
    tooltip(config: TooltipConfig): this;

    tooltip(config?: TooltipConfig | string): this {
        const tooltipConfig = typeof config === 'string' ? { content } : config;
        this.shape.behavior.add('tooltip', tooltipConfig);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // CONNECTORS (Para conectar shapes)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Conectar este shape con otro
    connectTo(target: Shape): ShapeBuilder {
        const connector = ArchFlow.createConnector()
            .connect(this.shape, target)
            .stroke(2, 0x666666)
            .endMarker('arrow')
            .build();
        return new ShapeBuilder('connector', connector);
    }

    /// Crear conector desde este shape
    connectFrom(source: Shape): ShapeBuilder {
        const connector = ArchFlow.createConnector()
            .connect(source, this.shape)
            .stroke(2, 0x666666)
            .endMarker('arrow')
            .build();
        return new ShapeBuilder('connector', connector);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GROUP - Agrupación de shapes
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Añadir shape como hijo (para grupos)
    addChild(child: Shape): this {
        if (this.shape.type === 'group') {
            this.shape.addChild(child);
        }
        return this;
    }

    /// Crear grupo con hijos
    static group(children: Shape[]): ShapeBuilder {
        const group = ArchFlow.createShape('group');
        children.forEach(child => group.addChild(child));
        return group;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // LIFECYCLE
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Al crear el shape
    onCreate(handler: (shape: Shape) => void): this {
        this.shape.on('create', handler);
        return this;
    }

    /// Al destruir el shape
    onDestroy(handler: (shape: Shape) => void): this {
        this.shape.on('destroy', handler);
        return this;
    }

    /// Cada frame (animaciones)
    onUpdate(handler: (shape: Shape, deltaTime: number) => void): this {
        this.shape.on('update', handler);
        return this;
    }

    /// Al seleccionar
    onSelect(handler: (shape: Shape) => void): this {
        this.shape.on('select', handler);
        return this;
    }

    /// Al deseleccionar
    onDeselect(handler: (shape: Shape) => void): this {
        this.shape.on('deselect', handler);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // CUSTOM DATA
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Datos personalizados
    data(key: string, value: unknown): this {
        this.shape.setData(key, value);
        return this;
    }

    /// Tag para поиск
    tag(tag: string): this {
        this.shape.setTag(tag);
        return this;
    }

    /// ID externo (para sincronización)
    externalId(id: string): this {
        this.shape.setExternalId(id);
        return this;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // BUILDER FINAL
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Compilar y añadir a la escena
    build(): Shape {
        // Registrar behaviors
        for (const [name, config] of this.behaviorConfigs) {
            this.shape.behavior.add(name, config);
        }

        // Registrar event handlers
        for (const [event, handlers] of this.eventHandlers) {
            for (const handler of handlers) {
                this.shape.on(event, handler);
            }
        }

        // Añadir a la escena
        ArchFlow.scene.addShape(this.shape);

        // Disparar evento de creación
        this.shape.trigger('create', this.shape);

        return this.shape;
    }

    /// Obtener sin añadir a la escena
    getShape(): Shape {
        return this.shape;
    }
}
```

---

## 4. Configuraciones

### 4.1 Configuraciones de Render

```typescript
interface ShadowConfig {
    offsetX: number;
    offsetY: number;
    blur: number;
    color: number;
}

interface HoverConfig {
    color?: number;       // Color del highlight
    opacity?: number;     // Opacidad (0-1)
    transition?: number;  // Duración (ms)
}

interface StrokeConfig {
    width: number;
    color: number;
    dashArray?: number[];
}
```

### 4.2 Configuraciones de Behavior

```typescript
interface DraggableConfig {
    axis?: 'x' | 'y' | 'both';
    snap?: number | { x?: number; y?: number };
    bounds?: Boundary;
    cursor?: string;
}

interface ResizableConfig {
    minWidth?: number;
    minHeight?: number;
    maxWidth?: number;
    maxHeight?: number;
    handles?: ResizeHandle[];
}

interface SelectableConfig {
    mode?: 'single' | 'multi' | 'toggle';
    selectedColor?: number;
    selectedBorder?: boolean;
}

interface DroppableConfig {
    accept?: string[];
    highlightOnHover?: boolean;
}

interface TooltipConfig {
    content: string | (() => string);
    position?: 'top' | 'bottom' | 'left' | 'right' | 'mouse';
    delay?: number;
}
```

---

## 5. Ejemplos Completos de Uso

### 5.1 Sticky Note Interactivo

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// EJEMPLO 1: Sticky Note interactivo (rectángulo amarillo con texto)
// ═══════════════════════════════════════════════════════════════════════════════

const note = ArchFlow.createStickyNote()
    .position(100, 200)
    .size(200, 150)
    .fillColor(0xFFFF00)  // Amarillo sticky note
    .border(1, 0xCCCCCC)
    .cornerRadius(4)
    .shadow('sm')
    .textContent('Ideas here...')
    .textColor(0x333333)
    .fontFamily('Inter')
    .fontSize(14)
    .onHover({ color: 0xFFEB3B, transition: 100 })
    .onClick(() => editNote())
    .draggable({ snap: 8 })
    .resizable({ minWidth: 100, minHeight: 80 })
    .onDragEnd((shape) => {
        console.log('Note moved to:', shape.getPosition());
    })
    .build();
```

### 5.2 Nodo de Diagrama con Conectores

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// EJEMPLO 2: Nodo de diagrama con título y conector
// ═══════════════════════════════════════════════════════════════════════════════

// Nodo principal
const node = ArchFlow.createNode()
    .position(300, 100)
    .size(150, 80)
    .fillColor(0xFFFFFF)
    .border(2, 0x2196F3)
    .cornerRadius(8)
    .shadow('md')
    .textContent('Start')
    .textColor(0x2196F3)
    .fontFamily('Inter')
    .fontSize(16)
    .fontWeight('bold')
    .textAlign('center')
    .verticalAlign('middle')
    .interactive()
    .selectable({ mode: 'single' })
    .draggable({ snap: 10 })
    .build();

// Nodo de decisión
const decision = ArchFlow.createShape('diamond')
    .position(550, 100)
    .size(100, 100)
    .fillColor(0xFFF3E0)
    .border(2, 0xFF9800)
    .textContent('Check?')
    .textColor(0xFF9800)
    .fontSize(12)
    .textAlign('center')
    .interactive()
    .selectable()
    .build();

// Conectar nodos
const connector = ArchFlow.createConnector()
    .connect(node, decision)
    .stroke(2, 0x666666)
    .endMarker('arrow')
    .build();
```

### 5.3 Toolbar de Canvas

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// EJEMPLO 3: Toolbar con herramientas
// ═══════════════════════════════════════════════════════════════════════════════

// Herramienta Selection
const selectTool = ArchFlow.createRectangle()
    .position(10, 10)
    .size(40, 40)
    .fillColor(0xE3F2FD)
    .border(2, 0x2196F3)
    .cornerRadius(4)
    .interactive()
    .onClick(() => setTool('select'))
    .build();

// Herramienta Rectangle
const rectTool = ArchFlow.createRectangle()
    .position(10, 60)
    .size(40, 40)
    .fillColor(0xE3F2FD)
    .border(2, 0x2196F3)
    .cornerRadius(4)
    .onClick(() => setTool('rectangle'))
    .build();

// Herramienta Circle
const circleTool = ArchFlow.createCircle()
    .position(30, 150)
    .radius(20)
    .fillColor(0xE3F2FD)
    .border(2, 0x2196F3)
    .onClick(() => setTool('circle'))
    .build();

// Herramienta Pen (path)
const penTool = ArchFlow.createPath()
    .position(10, 220)
    .points([
        { x: 10, y: 240 },
        { x: 20, y: 230 },
        { x: 30, y: 240 }
    ])
    .stroke(2, 0x2196F3)
    .lineCap('round')
    .onClick(() => setTool('pen'))
    .build();

// Herramienta Text
const textTool = ArchFlow.createText('T')
    .position(20, 280)
    .fontSize(24)
    .fontWeight('bold')
    .fillColor(0x333333)
    .onClick(() => setTool('text'))
    .build();
```

### 5.4 Canvas Completo con Pan/Zoom

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// EJEMPLO 4: Canvas con pan/zoom y grid
// ═══════════════════════════════════════════════════════════════════════════════

// Inicializar canvas
const canvas = document.getElementById('canvas') as HTMLCanvasElement;
ArchFlow.initialize(canvas);
ArchFlow.start();

// Grid de fondo
const gridSize = 20;
const grid = ArchFlow.createShape('group');
for (let x = 0; x < 2000; x += gridSize) {
    for (let y = 0; y < 2000; y += gridSize) {
        if (x % 100 === 0 || y % 100 === 0) {
            ArchFlow.createCircle()
                .position(x, y)
                .radius(1)
                .fillColor(0xE0E0E0)
                .noBorder()
                .locked()
                .build();
        }
    }
}

//Viewport con pan/zoom
const viewport = {
    x: 0,
    y: 0,
    zoom: 1,

    pan(dx: number, dy: number) {
        this.x += dx;
        this.y += dy;
        updateViewport();
    },

    zoomAt(center: Vec2, delta: number) {
        const newZoom = Math.max(0.1, Math.min(5, this.zoom * delta));
        const worldPoint = screenToWorld(center);
        this.x = worldPoint.x - (worldPoint.x - this.x) * (newZoom / this.zoom);
        this.y = worldPoint.y - (worldPoint.y - this.y) * (newZoom / this.zoom);
        this.zoom = newZoom;
        updateViewport();
    }
};

// Eventos de viewport
canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    viewport.zoomAt({ x: e.clientX, y: e.clientY }, e.deltaY > 0 ? 0.9 : 1.1);
});

let isPanning = false;
canvas.addEventListener('mousedown', (e) => {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
        isPanning = true;
    }
});
canvas.addEventListener('mousemove', (e) => {
    if (isPanning) {
        viewport.pan(e.movementX / viewport.zoom, e.movementY / viewport.zoom);
    }
});
canvas.addEventListener('mouseup', () => {
    isPanning = false;
});
```

### 5.5 Board Colaborativo

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// EJEMPLO 5: Board con notas de múltiples usuarios
// ═══════════════════════════════════════════════════════════════════════════════

// Obtener notas del servidor
const notes = await fetchNotes();

notes.forEach(noteData => {
    const note = ArchFlow.createStickyNote()
        .position(noteData.x, noteData.y)
        .size(noteData.width, noteData.height)
        .fillColor(noteData.color || 0xFFFF00)
        .textContent(noteData.text)
        .textColor(0x333333)
        .fontFamily('Inter')
        .fontSize(14)
        .interactive()
        .selectable({ mode: 'multi' })
        .draggable({ snap: 8 })
        .data('noteId', noteData.id)
        .data('authorId', noteData.authorId)
        .data('authorColor', noteData.authorColor)
        .externalId(noteData.id)  // Para sincronización
        .onClick((shape) => {
            // Resaltar nota seleccionada
            highlightAuthorNotes(noteData.authorId);
        })
        .onDragEnd((shape) => {
            // Sincronizar posición
            syncNotePosition(shape.getData('noteId'), shape.getPosition());
        })
        .build();

    // Indicador de autor
    const authorIndicator = ArchFlow.createCircle()
        .position(noteData.x + 10, noteData.y + 10)
        .radius(8)
        .fillColor(noteData.authorColor)
        .noBorder()
        .locked()
        .build();

    // Agrupar nota con indicador
    const noteGroup = ArchFlow.createShape('group')
        .position(noteData.x, noteData.y)
        .addChild(note)
        .addChild(authorIndicator)
        .build();
});

// Cursor de otro usuario
function showRemoteCursor(userId: string, position: Vec2, color: number) {
    let cursor = ArchFlow.getShapesByTag(`cursor-${userId}`)[0];

    if (!cursor) {
        cursor = ArchFlow.createShape('group')
            .tag(`cursor-${userId}`)
            .addChild(
                ArchFlow.createPath()
                    .points([{ x: 0, y: 0 }, { x: 0, y: 15 }, { x: 5, y: 12 }, { x: 8, y: 18 }, { x: 12, y: 15 }, { x: 10, y: 10 }, { x: 15, y: 10 }])
                    .stroke(2, color)
                    .build()
            )
            .addChild(
                ArchFlow.createText(userId)
                    .position(15, 15)
                    .fontSize(10)
                    .fillColor(color)
                    .build()
            )
            .locked()
            .build();
    }

    cursor.setPosition(position.x, position.y);
}
```

---

## 6. Mapeo Interno (Cómo Funciona)

### 6.1 Traducción Automática

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// TRADUCCIÓN INTERNA - El developer nunca ve esto
// ═══════════════════════════════════════════════════════════════════════════════

const CANVAS_TRANSLATION_MAP: CanvasTranslationMap = {
    // Mouse Events
    'onHover': {
        sensor: 'MouseOver',
        controller: 'Direct',
        actuator: 'Highlight',
        config: { color: 0xFFFF00, opacity: 0.3 }
    },
    'onClick': {
        sensor: 'MouseClick',
        controller: 'Debounce',
        actuator: 'Select',
        config: { ticks: 2 }
    },
    'onDoubleClick': {
        sensor: 'DoubleTap',
        controller: 'Direct',
        actuator: 'Custom',
        config: { handler: 'doubleClick' }
    },
    'draggable': {
        sensors: ['MouseDown', 'MouseMove', 'MouseUp'],
        controller: 'Custom',
        actuator: 'Move',
        config: { axis: 'both', snap: 8 }
    },
    'resizable': {
        sensors: ['MouseDown', 'MouseMove', 'MouseUp'],
        controller: 'Custom',
        actuator: 'Resize',
        config: { handles: ['nw', 'ne', 'sw', 'se'] }
    },
    'selectable': {
        sensor: 'MouseClick',
        controller: 'Debounce',
        actuator: 'Select',
        config: { mode: 'single' }
    },
    'selectable-multi': {
        sensor: 'MouseClick',
        controller: 'Custom',
        actuator: 'Select',
        config: { mode: 'multi', modifier: 'Shift' }
    },
    'interactive': {
        sensor: 'MouseOver',
        controller: 'Direct',
        actuator: 'Highlight',
        config: { color: 0xFFFF00, opacity: 0.2 }
    },
    'tooltip': {
        sensor: 'MouseOver',
        controller: 'Debounce',
        actuator: 'Property',
        config: { ticks: 6, content: '' }
    },
    'contextMenu': {
        sensor: 'RightClick',
        controller: 'Direct',
        actuator: 'ContextMenu',
        config: {}
    }
};
```

### 6.2 Flujo de Eventos del Canvas

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// EVENT BRIDGE - Canvas events → Entity events
// ═══════════════════════════════════════════════════════════════════════════════

class EventBridge {
    private canvas: HTMLCanvasElement;
    private scene: CanvasScene;

    constructor(canvas: HTMLCanvasElement, scene: CanvasScene) {
        this.canvas = canvas;
        this.scene = scene;
        this.setupEventListeners();
    }

    private setupEventListeners(): void {
        // Mouse move - buscar shape bajo el cursor
        this.canvas.addEventListener('mousemove', (e) => {
            const pos = this.screenToCanvas(e.offsetX, e.offsetY);
            const shapes = this.scene.getShapesAtPoint(pos);

            // Notificar hover enter/exit
            shapes.forEach(shape => {
                if (!shape.lastHovered) {
                    shape.trigger('hoverEnter', pos);
                    shape.lastHovered = true;
                }
            });

            // Notificar hover exit a shapes que ya no están
            this.scene.getAllShapes().forEach(shape => {
                if (shape.lastHovered && !shapes.includes(shape)) {
                    shape.trigger('hoverExit', pos);
                    shape.lastHovered = false;
                }
            });
        });

        // Click
        this.canvas.addEventListener('click', (e) => {
            const pos = this.screenToCanvas(e.offsetX, e.offsetY);
            const shapes = this.scene.getShapesAtPoint(pos);

            // Only trigger on topmost shape
            if (shapes.length > 0) {
                const topShape = shapes[0];
                topShape.trigger('click', pos);
            }
        });

        // Doble click
        this.canvas.addEventListener('dblclick', (e) => {
            const pos = this.screenToCanvas(e.offsetX, e.offsetY);
            const shapes = this.scene.getShapesAtPoint(pos);

            if (shapes.length > 0) {
                const topShape = shapes[0];
                topShape.trigger('doubleClick', pos);
            }
        });

        // Right click
        this.canvas.addEventListener('contextmenu', (e) => {
            e.preventDefault();
            const pos = this.screenToCanvas(e.offsetX, e.offsetY);
            const shapes = this.scene.getShapesAtPoint(pos);

            if (shapes.length > 0) {
                const topShape = shapes[0];
                topShape.trigger('rightClick', pos);
            }
        });

        // Drag start
        this.canvas.addEventListener('mousedown', (e) => {
            if (e.button === 0) {
                const pos = this.screenToCanvas(e.offsetX, e.offsetY);
                const shapes = this.scene.getShapesAtPoint(pos);

                if (shapes.length > 0) {
                    this.draggingShape = shapes[0];
                    this.dragStartPos = pos;
                    this.draggingShape.trigger('dragStart', pos);
                }
            }
        });

        // Drag move
        this.canvas.addEventListener('mousemove', (e) => {
            if (this.draggingShape) {
                const pos = this.screenToCanvas(e.offsetX, e.offsetY);
                const delta = { x: pos.x - this.dragStartPos.x, y: pos.y - this.dragStartPos.y };
                this.draggingShape.trigger('dragMove', delta);
                this.dragStartPos = pos;
            }
        });

        // Drag end
        this.canvas.addEventListener('mouseup', () => {
            if (this.draggingShape) {
                this.draggingShape.trigger('dragEnd', this.draggingShape.getPosition());
                this.draggingShape = null;
            }
        });
    }

    private screenToCanvas(screenX: number, screenY: number): Vec2 {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: (screenX - rect.left) * (this.canvas.width / rect.width),
            y: (screenY - rect.top) * (this.canvas.height / rect.height)
        };
    }
}
```

---

## 7. BehaviorRegistry

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR REGISTRY - Templates predefinidos para canvas
// ═══════════════════════════════════════════════════════════════════════════════

class BehaviorRegistry {
    private templates = new Map<string, BehaviorDefinition>();

    initDefaults(): void {
        // Interactive (hover highlight)
        this.register('interactive', {
            methods: ['onHover'],
            config: { color: 0xFFFF00, opacity: 0.2 }
        });

        // Selectable
        this.register('selectable', {
            methods: ['selectable'],
            config: { mode: 'single' }
        });

        // Multi-selectable
        this.register('multiSelectable', {
            methods: ['selectable'],
            config: { mode: 'multi' }
        });

        // Draggable
        this.register('draggable', {
            methods: ['draggable'],
            config: { axis: 'both', snap: 8 }
        });

        // Resizable
        this.register('resizable', {
            methods: ['resizable'],
            config: { handles: ['nw', 'ne', 'sw', 'se'] }
        });

        // Sticky Note (composite)
        this.register('stickyNote', {
            methods: ['interactive', 'draggable', 'selectable'],
            config: {
                fillColor: 0xFFFF00,
                border: 1,
                cornerRadius: 4,
                shadow: 'sm'
            }
        });

        // Node (diagrama)
        this.register('node', {
            methods: ['interactive', 'selectable', 'draggable'],
            config: {
                fillColor: 0xFFFFFF,
                border: 2,
                cornerRadius: 8,
                shadow: 'md'
            }
        });

        // Connector
        this.register('connector', {
            methods: ['selectable'],
            config: { stroke: 2, endMarker: 'arrow' }
        });

        // Tooltip
        this.register('tooltip', {
            methods: ['tooltip'],
            config: { delay: 500 }
        });
    }

    register(name: string, template: BehaviorDefinition): void {
        this.templates.set(name, template);
    }

    get(name: string): BehaviorDefinition | undefined {
        return this.templates.get(name);
    }

    list(): string[] {
        return Array.from(this.templates.keys());
    }
}
```

---

## 8. Plan de Implementación

### Fase 1: ShapeBuilder Core (Semana 1)
- [ ] `ShapeBuilder` con geometry (position, size, bounds)
- [ ] `TranslationEngine` y `CANVAS_TRANSLATION_MAP`
- [ ] `EventBridge` para canvas → entities

### Fase 2: Render Properties (Semana 2)
- [ ] Fill, border, shadow, stroke
- [ ] Text properties (font, color, align)
- [ ] Path properties (points, smooth, closed)
- [ ] Connector properties (markers, style)

### Fase 3: Mouse Events (Semana 3)
- [ ] `onHover()`, `onClick()`, `onDoubleClick()`, `onRightClick()`
- [ ] `draggable()`, `resizable()`, `droppable()`
- [ ] Selection behaviors

### Fase 4: Behaviors Templates (Semana 4)
- [ ] `stickyNote()`, `node()`, `connector()`
- [ ] `BehaviorRegistry` con defaults
- [ ] Documentation y ejemplos

---

## 9. Comparación de APIs

### API Original (Logic Bricks) - 30+ líneas
```typescript
const note = scene.createShape('rectangle');
note.setPosition(100, 200);
note.setSize(200, 150);
note.setFillColor(0xFFFF00);

LogicMappingTable.addConnection(note.id, 'MouseOver', 'Direct', 'Highlight');
LogicMappingTable.addConnection(note.id, 'MouseClick', 'Debounce', 'Select');
// ... 25+ líneas más
```

### API Fluida Canvas - 5 líneas
```typescript
const note = ArchFlow.createStickyNote()
    .position(100, 200)
    .size(200, 150)
    .onHover()
    .onClick(() => select())
    .build();
```

---

## 10. Referencias

- **Canvas API**: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
- **Fluent Interface Pattern**: https://martinfowler.com/bliki/FluentInterface.html
- **Builder Pattern**: https://refactoring.guru/design-patterns/builder

---

## Estado de Implementación

**Estado**: ✅ **COMPLETADO** (2026-02-03)

### Componentes Implementados

| Componente | Archivo | Estado |
|------------|---------|--------|
| TypeScript Type Definitions | `crates/archflow-web-ui/src/sdk/types.ts` | ✅ Completado |
| Behavior Registry | `crates/archflow-web-ui/src/sdk/BehaviorRegistry.ts` | ✅ Completado |
| Shape Builder API | `crates/archflow-web-ui/src/sdk/ShapeBuilder.ts` | ✅ Completado |
| ArchFlow Entry Point | `crates/archflow-web-ui/src/sdk/index.ts` | ✅ Completado |
| Logic Bricks Integration | `crates/archflow-web-ui/src/sdk/logic-sdk.ts` | ✅ Completado |
| Usage Examples | `crates/archflow-web-ui/docs/BEHAVIORS-SDK-EXAMPLES.md` | ✅ Completado |

### Funcionalidades Implementadas

- ✅ **Shape Types**: rectangle, circle, ellipse, path, text, image, group, connector
- ✅ **Geometry Methods**: position, size, bounds, center, rotation, scale, opacity, zIndex
- ✅ **Render Properties**: fillColor, border, cornerRadius, shadow, stroke
- ✅ **Text Properties**: textContent, textColor, fontFamily, fontSize, fontWeight, textAlign, verticalAlign
- ✅ **Path Properties**: points, addPoint, smooth, closed
- ✅ **Connector Properties**: connectTo, connectorStyle, endMarker
- ✅ **Mouse Events**: onHover, onClick, onDoubleClick, onRightClick
- ✅ **Drag & Drop**: draggable, resizable, droppable
- ✅ **Selection**: selectable, multiSelectable, selectionBox
- ✅ **Keyboard Events**: onKeyDown, onDelete, onEscape, onCopy, onPaste, onShortcut
- ✅ **Lifecycle Hooks**: onCreate, onDestroy, onUpdate, onSelect, onDeselect
- ✅ **Metadata**: data, tag, externalId
- ✅ **Convenience Methods**: createRectangle, createCircle, createStickyNote, createNode, createConnector, etc.

### Traducción Automática a Logic Bricks

La API traduce automáticamente las llamadas de alto nivel al sistema Sensor→Controller→Actuator:

```typescript
// Alto nivel (API del developer)
shape.onHover().draggable().build();

// Traducción automática (interno)
// SensorType.MouseOver → Controller.Direct → ActuatorType.Highlight
// SensorType.MouseClick → Controller.Custom → ActuatorType.Move
```

### Próximos Pasos

1. Integración con Canvas WASM renderer
2. Testing de integración con whiteboard
3. Performance optimization para shapes complejos

---

*Documento creado: 2026-02-02*  
*Actualizado: 2026-02-03*  
*Basado en principios de API fluida, canvas whiteboard, y diseño developer-first*
