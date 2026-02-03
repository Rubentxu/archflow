# ArchFlow Behaviors SDK

> **Fluent TypeScript API** - High-level shape builder with Logic Bricks integration for interactive canvas elements.

## Overview

The ArchFlow Behaviors SDK provides a fluent, chainable API for creating interactive canvas shapes. It abstracts away the complexity of the Logic Bricks system, allowing developers to declaratively define shapes and their behaviors using intuitive TypeScript.

**Key Capabilities:**
- **Fluent API** - Chainable methods for all shape properties
- **Behavior composition** - Declarative interaction patterns
- **Type-safe** - Full TypeScript type definitions
- **Logic Bricks mapping** - Automatic sensor-actuator wiring
- **WASM integration** - Seamless bridge to Rust backend

## Architecture

The SDK follows **Domain-Driven Design** with clear layer separation:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Developer API Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ShapeBuilder  │  │BehaviorConfig│  │Type Definitions│         │
│  │(Fluent Chain)│  │(Declarative) │  │(TypeScript)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                    Logic Bricks Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │BehaviorRegistry│ │CustomController│ │LogicSDK      │          │
│  │(Templates)   │  │Registry       │  │(High-Level) │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      WASM Bridge Layer                          │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │WASM Module   │  │EntityStore   │                            │
│  │(Rust Backend)│  │(State)       │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### ShapeBuilder - Fluent API

The `ShapeBuilder` class provides a chainable API for creating shapes:

```typescript
import { ShapeBuilder } from '@archflow/sdk';

const shape = new ShapeBuilder('rectangle')
  .position(100, 200)
  .size(200, 150)
  .fillColor(0xFFFF00)
  .cornerRadius(8)
  .onHover()
  .onClick(() => console.log('clicked!'))
  .draggable()
  .build();
```

**Method Categories:**
- **Geometry** - `position()`, `size()`, `rotation()`, `scale()`
- **Render** - `fillColor()`, `border()`, `shadow()`
- **Behaviors** - `onHover()`, `onClick()`, `draggable()`
- **Metadata** - `tag()`, `data()`, `externalId()`

### Behavior Configuration

Behaviors are configured declaratively and map to Logic Bricks:

```typescript
// Hover behavior
.onHover({ color: 0xFF0000, opacity: 0.5 })

// Maps to Logic Bricks:
// Sensor: MouseOver
// Controller: Direct
// Actuator: Highlight
```

**Available Behaviors:**

| Behavior | Sensor | Actuator | Purpose |
|----------|--------|----------|---------|
| `onHover()` | MouseOver | Highlight | Visual feedback |
| `onClick()` | MouseClick | Select | Selection |
| `onDoubleClick()` | MouseClick | Move | Custom action |
| `draggable()` | MouseDrag | Move | Position change |
| `resizable()` | MouseDrag | Resize | Size change |
| `selectable()` | MouseClick | Select | Click selection |
| `tooltip()` | MouseOver | ShowTooltip | Contextual help |

### Type System

Comprehensive TypeScript types for type safety:

```typescript
// Shape types
type ShapeType = 'rectangle' | 'circle' | 'ellipse' | 'path' 
              | 'text' | 'image' | 'group' | 'connector';

// Shape interface
interface Shape {
  id: ShapeId;
  type: ShapeType;
  position: Point2D;
  size: Size2D;
  rotation: number;
  scale: number;
  opacity: number;
  zIndex: number;
  visible: boolean;
  locked: boolean;
  // ... render properties
  // ... behavior metadata
}
```

## Usage Examples

### Basic Rectangle

```typescript
import { ArchFlow } from '@archflow/sdk';

const rect = ArchFlow.createShape('rectangle')
  .position(100, 100)
  .size(200, 150)
  .fillColor(0x3B82F6)  // Blue
  .cornerRadius(8)
  .border(2, 0x1E40AF)  // Dark blue border
  .build();
```

### Interactive Button

```typescript
const button = ArchFlow.createShape('rectangle')
  .position(300, 200)
  .size(120, 40)
  .fillColor(0x10B981)  // Green
  .cornerRadius(4)
  .onHover({ color: 0x059669, opacity: 0.8 })
  .onClick(() => handleClick())
  .build();
```

### Draggable Card

```typescript
const card = ArchFlow.createShape('rectangle')
  .position(50, 50)
  .size(300, 200)
  .fillColor(0xFFFFFF)
  .shadow('lg')
  .cornerRadius(12)
  .onHover()
  .draggable({ axis: 'both', snap: 8 })
  .selectable()
  .build();
```

### Text Label

```typescript
const label = ArchFlow.createShape('text')
  .position(100, 300)
  .size(200, 30)
  .textContent('Hello, World!')
  .textColor(0x000000)
  .fontSize(16)
  .fontWeight('bold')
  .textAlign('center')
  .build();
```

### Connector

```typescript
const connector = ArchFlow.createShape('connector')
  .connectTo(sourceShape.id)
  .connectorStyle(2, 0x6B7280)
  .endMarker('arrow')
  .build();
```

### Custom Data

```typescript
const annotated = ArchFlow.createShape('rectangle')
  .position(100, 100)
  .size(200, 150)
  .data({ 
    title: 'My Component',
    description: 'This is important',
    metadata: { id: 123, category: 'ui' }
  })
  .tag('component')
  .tag('interactive')
  .externalId('ext-ref-abc')
  .build();
```

### Group Creation

```typescript
const group = ShapeBuilder.group(
  shape1,
  shape2,
  shape3
);
```

### Lifecycle Hooks

```typescript
const shape = ArchFlow.createShape('rectangle')
  .position(100, 100)
  .size(200, 150)
  .onCreate(() => console.log('Created!'))
  .onUpdate(() => console.log('Modified!'))
  .onDestroy(() => console.log('Removed!'))
  .onSelect(() => console.log('Selected!'))
  .onDeselect(() => console.log('Deselected!'))
  .build();
```

### Keyboard Shortcuts

```typescript
const shape = ArchFlow.createShape('rectangle')
  .position(100, 100)
  .size(200, 150)
  .onDelete(() => deleteShape())
  .onEscape(() => cancelAction())
  .onCopy(() => copyToClipboard())
  .onPaste(() => pasteFromClipboard())
  .onShortcut('Ctrl+S', () => saveShape())
  .build();
```

## Complete API Reference

### Geometry Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `position(x, y)` | x, y: number | Set position |
| `x(value)` | value: number | Set X coordinate |
| `y(value)` | value: number | Set Y coordinate |
| `size(w, h)` | w, h: number | Set size |
| `width(value)` | value: number | Set width |
| `height(value)` | value: number | Set height |
| `bounds(x, y, w, h)` | x, y, w, h: number | Set bounds |
| `center(x, y)` | x, y: number | Center at point |
| `rotation(degrees)` | degrees: number | Set rotation |
| `scale(factor)` | factor: number | Set scale |
| `opacity(value)` | value: 0-1 | Set opacity |
| `zIndex(value)` | value: number | Set z-index |
| `visible(value)` | value: boolean | Set visibility |
| `locked(value)` | value: boolean | Set locked |

### Render Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `fillColor(hex)` | hex: number | Set fill color |
| `noFill()` | - | Remove fill |
| `border(w, c, s?)` | w, c, s: number, string | Set border |
| `noBorder()` | - | Remove border |
| `cornerRadius(r)` | r: number | Set corner radius |
| `shadow(size)` | size: ShadowSize | Set shadow preset |
| `customShadow(c)` | c: ShadowConfig | Set custom shadow |
| `stroke(w, c)` | w, c: number | Set stroke |
| `lineCap(style)` | style: LineCap | Set line cap |
| `lineJoin(style)` | style: LineJoin | Set line join |
| `dashArray(dash)` | dash: number[] | Set dash array |

### Behavior Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `onHover(c?)` | c?: config | Add hover behavior |
| `onClick(cb)` | cb: function | Add click handler |
| `onDoubleClick(cb)` | cb: function | Add double-click |
| `onRightClick(cb)` | cb: function | Add right-click |
| `draggable(c?)` | c?: config | Make draggable |
| `resizable(c?)` | c?: config | Make resizable |
| `selectable(m?)` | m?: mode | Make selectable |
| `multiSelectable(m?)` | m?: modifier | Multi-select |
| `tooltip(c, d?)` | c, d: string, number | Add tooltip |
| `onDelete()` | - | Delete on Delete key |
| `onEscape()` | - | Handle Escape |
| `onCopy()` | - | Copy on Ctrl+C |
| `onPaste()` | - | Paste on Ctrl+V |

## Type Definitions

### Behavior Configuration

```typescript
interface BehaviorConfig {
  sensor?: any;           // Sensor type
  controller?: any;       // Controller type
  actuator?: any;         // Actuator type
  config?: Record<string, unknown>;
}
```

### Drag/Resize Options

```typescript
interface DraggableOptions {
  axis?: 'x' | 'y' | 'both';
  snap?: number;
}

interface ResizableOptions {
  handles?: ResizeHandle[];
  snap?: number;
}
```

### Selection Modes

```typescript
type SelectionMode = 'single' | 'additive' | 'range';

type ResizeHandle = 'nw' | 'n' | 'ne' | 'w' | 'e' | 'sw' | 's' | 'se';
```

## Integration with Logic Bricks

The SDK automatically maps to the Logic Bricks system:

```
TypeScript API → Behavior Registry → WASM Bridge → Rust Logic Bricks
```

**Example Mapping:**

```typescript
// TypeScript
.onHover({ color: 0xFF0000 })

// Behavior Registry
behaviorConfig = {
  sensor: SensorType.MouseOver,
  controller: Controller.direct(),
  actuator: ActuatorType.Highlight,
  config: { color: 0xFF0000, opacity: 0.5 }
}

// WASM Bridge
archflow_wasm.register_behavior(entity_id, behaviorConfig)

// Rust Logic Bricks
Sensor: MouseOverSensor
Controller: DirectController
Actuator: HighlightActuator
```

## Best Practices

### Method Chaining

1. **Set geometry first**: Position and size
2. **Then appearance**: Colors, borders, shadows
3. **Finally behaviors**: Interactions
4. **Always call build()**: Finalize the shape

### Performance

1. **Reuse builders**: Create template builders
2. **Batch operations**: Build multiple shapes together
3. **Lazy behaviors**: Only add needed interactions
4. **Optimize rendering**: Use `visible()` for culling

### Type Safety

1. **Use type assertions**: For custom data
2. **Leverage enums**: For shape types and modes
3. **Avoid `any`**: Use proper type definitions
4. **Enable strict mode**: Catch errors early

## Constraints and Limitations

### Current Limitations

- **Single-threaded**: Main thread execution
- **No animations**: Static properties only
- **Fixed behaviors**: Limited customization
- **TypeScript only**: No JavaScript flow types

### Future Enhancements

- **Animation API**: Transitions and keyframes
- **Plugin system**: Custom behaviors
- **Validation**: Schema-based shape validation
- **Serialization**: Save/load shape definitions

## References

- **EPIC-WEB-011**: Behaviors SDK specification
- **archflow-logic**: Logic Bricks system
- **archflow-web-ui**: React integration
- **ShapeBuilder.ts**: Implementation source

## License

MIT License - See LICENSE file for details.
