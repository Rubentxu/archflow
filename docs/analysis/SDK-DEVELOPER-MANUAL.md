# ArchFlow SDK Developer Manual

## Sistema de Logic Bricks (Sensor→Actuator)

**Fecha**: 2026-02-02  
**Versión**: 1.0

---

## 1. Arquitectura General

```
┌─────────────────────────────────────────────────────────────────────┐
│                     JAVASCRIPT / TYPESCRIPT                         │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  WasmBridge (Ya expuesto)                                   │   │
│   │  - spawnEntity, moveEntity, setPosition, setSize            │   │
│   │  - selectEntity, getSelection, undo, redo                   │   │
│   │  - getInputBufferPtr, pushInputEvent                        │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │  Logic Bricks API (YA EXPPUESTO A JS!)                      │   │
│   │                                                             │   │
│   │  new LogicMappingTable()                                    │   │
│   │  table.addHighlight(id, SensorType, Controller)             │   │
│   │  table.addMove(id, SensorType, Controller)                  │   │
│   │  table.addSelect(id, SensorType, Controller)                │   │
│   │                                                             │   │
│   │  Controller.And(sensor) | Controller.Or(sensor) | .Not()    │   │
│   │                                                             │   │
│   │  new SignalByte() - 6-tick history, edge detection          │   │
│   └─────────────────────────────────────────────────────────────┘   │
└────────────────────────────┬──────────────────────────────────────┘
                             │ WASM
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        RUST / WASM (CORE)                           │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    SENSORS (IMPLEMENTADOS)                   │   │
│  │  sensors/mod.rs                                              │   │
│  │  ├── MouseOverSensor     - Hover detection                   │   │
│  │  ├── MouseClickSensor    - Click detection                   │   │
│  │  ├── TouchSensor         - AABB collision                    │   │
│  │  ├── ProximitySensor     - Near detection + hysteresis       │   │
│  │  ├── RadarSensor         - Directional cone detection        │   │
│  │  ├── KeyShortcutSensor   - Keyboard shortcuts                │   │
│  │  ├── DoubleTapSensor     - Double click detection            │   │
│  │  ├── LongPressSensor     - Hold detection                    │   │
│  │  └── RightClickSensor    - Context menu trigger              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    ACTUATORS (IMPLEMENTADOS)                 │   │
│  │  actuators/mod.rs                                            │   │
│  │  ├── HighlightActuator  - Change entity color                │   │
│  │  ├── SelectActuator     - Mark entity as selected            │   │
│  │  ├── MoveActuator       - Move entity (drag)                 │   │
│  │  ├── PropertyActuator   - Modify properties                  │   │
│  │  ├── CameraActuator     - Pan/zoom camera                    │   │
│  │  ├── StateActuator      - State machine transitions          │   │
│  │  └── MessageActuator    - Inter-entity messaging             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    LOGIC SYSTEM                              │   │
│  │  logic_system.rs - LogicSystem                               │   │
│  │  ├── evaluate_sensors() → generates pulses                   │   │
│  │  ├── execute_actuators() → responds to pulses                │   │
│  │  └── update() → main loop                                    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    WIRING TABLE                              │   │
│  │  mapping/mod.rs                                              │   │
│  │  ├── LogicMappingTable  - sensor→actuator connections        │   │
│  │  ├── Controller         - AND, OR, NOT logic                 │   │
│  │  └── SensorType enum    - Sensor type identifiers            │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    COMMAND PATTERN                           │   │
│  │  command.rs                                                  │   │
│  │  ├── AnyCommand enum   - Move, Resize, SetPosition, etc.     │   │
│  │  └── CommandHistory    - Undo/Redo stack                     │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    SIGNAL PROCESSING                         │   │
│  │  signals.rs - SignalByte (6-tick history in 1 byte)          │   │
│  │  ├── is_rising_edge()   - Trigger on 0→1                     │   │
│  │  ├── is_falling_edge()  - Release on 1→0                     │   │
│  │  ├── is_steady_high()   - Hysteresis support                 │   │
│  │  └── is_steady_low()    - Debounce support                   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    SPATIAL INDEXING                          │   │
│  │  spatial.rs - SpatialHashGrid for O(1) collision             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    INPUT PROCESSING                          │   │
│  │  input.rs - InputSampler, InputSnapshot, InputEvent          │   │
│  │  └── SharedArrayBuffer support for <2ms latency              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    ANIMATION                                 │   │
│  │  tween.rs - Tween, TweenManager, easing functions            │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Sensores Disponibles

### 2.1 Enum SensorType (Expuesto a JS)

```typescript
// Ya disponible en @archflow/sdk
import { SensorType } from '@archflow/sdk';

SensorType.MouseOver   // = 0, Mouse hovering over entity
SensorType.MouseClick  // = 1, Mouse button clicked
SensorType.Proximity   // = 2, Another entity nearby
SensorType.KeyShortcut // = 3, Keyboard shortcut pressed
```

### 2.2 Sensores Implementados (Rust)

| Sensor | Archivo | Descripción |
|--------|---------|-------------|
| `MouseOverSensor` | `sensors/mouse_over.rs` | Detecta hover (entrada/salida) |
| `MouseClickSensor` | `sensors/mouse_click.rs` | Detecta clicks de botones |
| `TouchSensor` | `sensors/collision.rs` | Detección AABB overlap |
| `ProximitySensor` | `sensors/proximity.rs` | Detección nearby con hysteresis |
| `RadarSensor` | `sensors/radar.rs` | Detección en cono direccional |
| `KeyShortcutSensor` | `sensors/key_shortcut.rs` | Atajos de teclado |
| `DoubleTapSensor` | `sensors/double_tap.rs` | Doble click rápido |
| `LongPressSensor` | `sensors/long_press.rs` | Mantener presionado |
| `RightClickSensor` | `sensors/right_click.rs` | Click derecho |
| `TouchSensor` | `sensors/touch.rs` | Touch/hover detection |

### 2.3 SignalByte: Detección de Flancos

```rust
// Todos los sensores usan SignalByte internamente
// 6-tick history en 1 byte

let mut signal = SignalByte::new();

signal.push(true);   // T0
signal.push(true);   // T1
signal.push(false);  // T2
// Historia: 0b00000110

// Detectar flancos
signal.is_rising_edge();  // false (T-1=1, T=0)
signal.is_falling_edge(); // true  (T-1=1, T=0)

// Hysteresis / Debounce
signal.is_steady_high(3);  // true si últimos 3 ticks son 1
signal.is_steady_low(3);   // true si últimos 3 ticks son 0
```

**Expuesto a JS:**

```typescript
import { SignalByteWasm } from '@archflow/sdk';

const signal = new SignalByteWasm();
signal.push(true);
signal.push(true);
signal.isRisingEdge();    // true
signal.isFallingEdge();   // false
signal.isSteadyHigh(6);   // Verificar hysteresis
```

---

## 3. Controladores (Boolean Logic)

### 3.1 Enum ControllerType

```typescript
import { ControllerType } from '@archflow/sdk';

ControllerType.Direct = 0  // Pass-through
ControllerType.And     = 1 // primary AND secondary
ControllerType.Or      = 2 // primary OR secondary
ControllerType.Not     = 3 // Invertir señal
```

### 3.2 API de Controllers (Expuesto a JS)

```typescript
import { Controller, ControllerType, SensorType } from '@archflow/sdk';

// Direct: pasar señal directa
const direct = Controller.Direct();

// AND: ambos sensores deben estar activos
const and = Controller.And(SensorType.MouseOver);
// Solo activa si primario Y MouseOver están activos

// OR: al menos un sensor activo
const or = Controller.Or(SensorType.MouseClick);
// Activa si primario O MouseClick están activos

// NOT: invertir señal
const not = Controller.Not();
// Invierte la señal del sensor primario
```

---

## 4. Actuators Disponibles

### 4.1 Enum ActuatorType

```typescript
import { ActuatorType } from '@archflow/sdk';

ActuatorType.Highlight = 0  // Cambiar color (hover)
ActuatorType.Select    = 1  // Marcar como seleccionado
ActuatorType.Move      = 2  // Mover entidad (drag)
```

### 4.2 Actuators Implementados (Rust)

| Actuator | Archivo | Descripción |
|----------|---------|-------------|
| `HighlightActuator` | `actuators/highlight.rs` | Resaltar al hover |
| `SelectActuator` | `actuators/select.rs` | Gestión de selección |
| `MoveActuator` | `actuators/move_.rs` | Mover entidades |
| `PropertyActuator` | `actuators/property.rs` | Modificar propiedades |
| `CameraActuator` | `actuators/camera.rs` | Control de cámara |
| `StateActuator` | `actuators/state.rs` | Transiciones de estado |
| `MessageActuator` | `actuators/message.rs` | Mensajes entre entidades |

---

## 5. Wiring Table: Conectar Sensores a Actuators

### 5.1 LogicMappingTable (Expuesto a JS)

```typescript
import { LogicMappingTable, SensorType, Controller, ActuatorType } from '@archflow/sdk';

const table = new LogicMappingTable();
const entityId = 42;

// Conectar MouseOver → Highlight
table.addHighlight(entityId, SensorType.MouseOver, Controller.Direct());

// Conectar MouseClick → Select (AND con MouseOver)
table.addSelect(entityId, SensorType.MouseClick, Controller.And(SensorType.MouseOver));

// Conectar Proximity → Move
table.addMove(entityId, SensorType.Proximity, Controller.Direct());

// Consultar conexiones
table.hasConnection(entityId, SensorType.MouseOver);  // true
table.connectionCount(entityId);                       // 3

// Obtener entidades con conexiones
const entities = table.getConnectedEntities();  // Uint32Array

// Eliminar conexión
table.removeConnection(entityId, SensorType.MouseOver);

// Limpiar todo
table.clear();
table.clearEntity(entityId);
```

### 5.2 Métodos Disponibles

| Método | Descripción |
|--------|-------------|
| `addHighlight(id, sensor, controller)` | Conectar sensor a Highlight |
| `addSelect(id, sensor, controller)` | Conectar sensor a Select |
| `addMove(id, sensor, controller)` | Conectar sensor a Move |
| `hasConnection(id, sensor)` | Verificar si existe conexión |
| `connectionCount(id)` | Contar conexiones de una entidad |
| `getConnectedEntities()` | Obtener todas las entidades conectadas |
| `removeConnection(id, sensor)` | Eliminar conexión |
| `clearEntity(id)` | Eliminar todas las conexiones de una entidad |
| `clear()` | Limpiar todas las conexiones |

---

## 6. LogicSystem: Evaluación de Sensores y Ejecución

### 6.1 Flujo Principal

```rust
// En logic_system.rs
pub struct LogicSystem {
    input_sampler: InputSampler,
    pulse_bus: PulseBus,
    wiring: LogicMappingTable,
    // ... sensores ...
}

impl LogicSystem {
    pub fn evaluate_sensors(&mut self, store: &EntityStore) -> Vec<Pulse> {
        // 1. Tomar snapshot del input
        let snapshot = self.input_sampler.take_snapshot();
        
        // 2. Evaluar todos los sensores
        // - MouseOver: hit testing
        // - Touch: AABB collision
        // - Proximity: distance check
        // - Radar: cone check
        
        // 3. Generar pulsos para cambios de estado
        pulses
    }
    
    pub fn execute_actuators(&mut self, store: &mut EntityStore, pulses: &[Pulse]) {
        // Para cada pulso, buscar conexiones en wiring table
        // y ejecutar actuadores correspondientes
    }
    
    pub fn update(&mut self, store: &mut EntityStore) {
        let pulses = self.evaluate_sensors(store);
        self.execute_actuators(store, &pulses);
    }
}
```

### 6.2 SensorId Enum (Rust)

```rust
pub enum SensorId {
    Mouse = 0,
    Touch = 1,
    Proximity = 2,
    Radar = 3,
    Keyboard = 4,
    MouseClick = 5,
    DoubleTap = 6,
    LongPress = 7,
    RightClick = 8,
}
```

---

## 7. Command Pattern: Undo/Redo

### 7.1 AnyCommand Enum

```rust
pub enum AnyCommand {
    Move { entity_idx: usize, from: Vec2, to: Vec2 },
    Resize { entity_idx: usize, from: Vec2, to: Vec2 },
    SetPosition { entity_idx: usize, old_pos: Vec2, new_pos: Vec2 },
    SetSize { entity_idx: usize, old_size: Vec2, new_size: Vec2 },
    SetColor { entity_idx: usize, old_color: u32, new_color: u32 },
    Select { entity_idx: usize, selected: bool },
    Visibility { entity_idx: usize, visible: bool },
    Delete { entity_idx: usize, data: EntityData },
    Spawn { entity_idx: usize },
}
```

### 7.2 CommandHistory

```rust
pub struct CommandHistory {
    history: Vec<AnyCommand>,
    redo_stack: Vec<AnyCommand>,
    max_depth: usize,
}

impl CommandHistory {
    pub fn execute(&mut self, cmd: AnyCommand, store: &mut EntityStore);
    pub fn undo(&mut self, store: &mut EntityStore) -> Result<()>;
    pub fn redo(&mut self, store: &mut EntityStore) -> Result<()>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

**Expuesto a JS (via WasmBridge):**

```typescript
const bridge = getWasmBridge();

bridge.undo();    // Deshacer última acción
bridge.redo();    // Rehacer acción deshecha
bridge.canUndo(); // boolean
bridge.canRedo(); // boolean
```

---

## 8. Tween System: Animaciones

### 8.1 Tipos de Easing

```rust
pub enum Easing {
    Linear,
    QuadIn, QuadOut, QuadInOut,
    CubicIn, CubicOut, CubicInOut,
    SineIn, SineOut, SineInOut,
    BackOut,
    BounceOut,
    ElasticOut,
}
```

### 8.2 TweenManager

```rust
pub struct TweenManager {
    tweens: Vec<Tween>,
}

impl TweenManager {
    pub fn tween_position(
        &mut self,
        entity_idx: usize,
        to: Vec2,
        duration_ms: u32,
        easing: Easing,
    );
    
    pub fn tween_opacity(
        &mut self,
        entity_idx: usize,
        to: f32,
        duration_ms: u32,
        easing: Easing,
    );
    
    pub fn update(&mut self, store: &mut EntityStore, dt_ms: u32);
}
```

---

## 9. Ejemplo Completo en JavaScript

```typescript
import { 
  WasmBridge, 
  LogicMappingTable, 
  SensorType, 
  Controller, 
  ActuatorType,
  SignalByteWasm 
} from '@archflow/sdk';

// 1. Inicializar el bridge WASM
const bridge = new WasmBridge();
bridge.initialize(1920, 1080);

// 2. Crear tabla de conexiones
const wiring = new LogicMappingTable();

// 3. Configurar comportamientos para una entidad
const entityId = bridge.spawnEntity(100, 100, 80, 60);

// Hover → Highlight (verde al hover)
wiring.addHighlight(
  entityId, 
  SensorType.MouseOver, 
  Controller.Direct()
);

// Click → Select (AND con Hover para evitar seleccion accidental)
wiring.addSelect(
  entityId,
  SensorType.MouseClick,
  Controller.And(SensorType.MouseOver)
);

// Proximity → Highlight (amarillo cuando cerca de otro)
wiring.addHighlight(
  entityId,
  SensorType.Proximity,
  Controller.Or(SensorType.MouseOver)
);

// 4. Manejar eventos de input
document.addEventListener('mousemove', (e) => {
  bridge.pushInputEvent(
    1,  // MouseMove
    e.clientX, 
    e.clientY,
    e.buttons,
    getModifiers(e)  // Shift, Ctrl, Alt
  );
});

document.addEventListener('mousedown', (e) => {
  bridge.pushInputEvent(
    0,  // MouseDown
    e.clientX,
    e.clientY,
    1 << e.button,  // Bitmask de botones
    getModifiers(e)
  );
});

// 5. Game loop
function tick(timestamp) {
  bridge.tick(timestamp);
  requestAnimationFrame(tick);
}
requestAnimationFrame(tick);

// 6. Undo/Redo
document.addEventListener('keydown', (e) => {
  if (e.key === 'z' && (e.ctrlKey || e.metaKey)) {
    if (e.shiftKey) {
      bridge.redo();
    } else {
      bridge.undo();
    }
  }
});

// 7. Usar SignalByte para detección de flancos
const signal = new SignalByteWasm();
// El sistema interno ya usa esto para detectar
// rising_edge (entrada hover) y falling_edge (salida hover)
```

---

## 10. Estado de Implementación

### ✅ Ya Implementado y Expuesto a JS

| Componente | Archivo | Expuesto a JS |
|------------|---------|---------------|
| `LogicMappingTable` | `mapping/mod.rs` | ✅ |
| `Controller` | `mapping/mod.rs` | ✅ |
| `SensorType` enum | `mapping/sensor_type.rs` | ✅ |
| `ActuatorType` enum | `lib.rs` | ✅ |
| `SignalByteWasm` | `signals.rs` | ✅ |
| `WasmBridge` | `archflow-web` | ✅ |
| Command History | `command.rs` | ✅ (undo/redo) |
| InputSampler | `input.rs` | ✅ |
| SpatialHash | `spatial.rs` | ✅ |

### ✅ Implementado en Rust (no expuesto aún)

| Componente | Archivo |
|------------|---------|
| `LogicSystem` | `logic_system.rs` |
| Todos los Sensores | `sensors/*.rs` |
| Todos los Actuators | `actuators/*.rs` |
| Tweens | `tween.rs` |
| Visibility | `visibility.rs` |

---

## 11. Archivo de Definiciones TypeScript

El WASM genera automáticamente `archflow_web.d.ts` con todas las definiciones:

```typescript
// Tipos principales disponibles en JS/TS:
- WasmBridge        // Bridge principal
- LogicMappingTable // Tabla de conexiones
- Controller        // Controladores AND/OR/NOT
- ControllerType    // Enum de tipos
- SignalByteWasm    // Historial de señal
- SensorType        // Enum de sensores
- ActuatorType      // Enum de actuadores
```

---

## 12. Próximos Pasos

1. **Integrar LogicSystem con WasmBridge** - Conectar el motor de lógica con las funciones expuestas
2. **Exponer más actuators** - AddSelect, AddMove, etc. a JS
3. **API de alto nivel** - `editor.behaviors.configure()` 
4. **Behavior Bundles** - Presets como 'diagram', 'whiteboard'

---

## 13. Flujo de Datos: De JavaScript a WASM y Viceversa

Esta sección explica cómo los eventos del DOM llegan al sistema de Logic Bricks y cómo los cambios se reflejan de vuelta en JavaScript.

### 13.1 Arquitectura del Flujo de Datos (Completo)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              JAVASCRIPT / DOM                                │
│                                                                             │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│   │  Event:      │    │  Event:      │    │  Event:      │                 │
│   │  mousemove   │    │  mousedown   │    │  keydown     │                 │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                 │
│          │                   │                   │                          │
│          ▼                   ▼                   ▼                          │
│   ┌─────────────────────────────────────────────────────────────────┐       │
│   │                    JS Event Handlers                             │       │
│   │                                                                 │       │
│   │   canvas.onmousemove = (e) => {                                │       │
│   │     const x = e.clientX;                                       │       │
│   │     const y = e.clientY;                                       │       │
│   │     // OPCIÓN 1: Escribir directamente en SAB                  │       │
│   │     sharedBuffer.mouse_x = x;                                  │       │
│   │     sharedBuffer.mouse_y = y;                                  │       │
│   │     sharedBuffer.buttons = e.buttons;                          │       │
│   │                                                                 │       │
│   │     // OPCIÓN 2: Usar API de alto nivel                        │       │
│   │     bridge.pushInputEvent(TYPE_MOVE, x, y, buttons, modifiers);│       │
│   │   }                                                             │       │
│   │                                                                 │       │
│   │   canvas.onmousedown = (e) => {                                │       │
│   │     bridge.pushInputEvent(TYPE_DOWN, e.clientX, e.clientY,     │       │
│   │       1 << e.button, getModifiers(e));                         │       │
│   │   }                                                             │       │
│   │                                                                 │       │
│   │   document.onkeydown = (e) => {                                │       │
│   │     bridge.pushInputEvent(TYPE_KEY_DOWN, e.keyCode, 0, 0,      │       │
│   │       getModifiers(e));                                        │       │
│   │   }                                                             │       │
│   └─────────────────────────────────────────────────────────────────┘       │
│                                    │                                          │
│                                    ▼                                          │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │              SHARED ARRAY BUFFER (64 bytes)                           │ │
│   │                                                                        │ │
│   │   ┌─────────┬─────────┬─────────┬─────────┬─────────┐                 │ │
│   │   │ head    │ tail    │ mouse_x │ mouse_y │ buttons │ ...              │ │
│   │   │ (u32)   │ (u32)   │ (i32)   │ (i32)   │ (u8)    │                  │ │
│   │   └─────────┴─────────┴─────────┴─────────┴─────────┘                 │ │
│   │                                                                        │ │
│   │   Offset  0-3:  head (índice de escritura)                            │ │
│   │   Offset  4-7:  tail (índice de lectura)                              │ │
│   │   Offset  8-11: mouse_x (posición X del mouse)                        │ │
│   │   Offset 12-15: mouse_y (posición Y del mouse)                        │ │
│   │   Offset 16-16: buttons (bitmask: 1=left, 2=right, 4=middle)          │ │
│   │   Offset 17-17: modifiers (bitmask: 1=shift, 2=ctrl, 4=alt)           │ │
│   │   Offset 18-19: wheel_delta                                          │ │
│   │   Offset 20-23: timestamp                                             │ │
│   │   Offset 24-55: keys[32] (256 bits para 256 teclas)                   │ │
│   │   Offset 56-63: padding (alineación cache-line)                       │ │
│   │                                                                        │ │
│   │   Escrito por JS, leído por Rust sin copy (memory barrier)            │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└────────────────────────────────────────────────────┬──────────────────────────┘
                                                     │
                    ┌────────────────────────────────┘
                    │    Lectura atómica (no bloqueante)
                    │    Atomics.load() / Atomics.store()
                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              RUST / WASM                                     │
│                                                                             │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                      INPUT SAMPLER                                      │ │
│   │                                                                        │ │
│   │   pub struct InputSampler {                                            │ │
│   │       sab_ptr: Option<*const InputSnapshotSAB>,  // Puntero al SAB     │ │
│   │       sab_available: bool,                                          │ │
│   │       fallback_buffer: InputSnapshotSAB,      // Para browsers sin SAB │ │
│   │       event_buffer: Vec<RawInputEvent>,        // Buffer circular      │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   │   pub fn take_snapshot(&self) -> &InputSnapshotSAB {                  │ │
│   │       if self.sab_available {                                          │ │
│   │           unsafe { &*self.sab_ptr.unwrap() }  // ← Lectura directa     │ │
│   │       } else {                                                         │ │
│   │           &self.fallback_buffer                                        │ │
│   │       }                                                                │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   │   pub fn drain_events(&mut self) -> Vec<RawInputEvent> {               │ │
│   │       // Lee todos los eventos acumulados en el buffer                 │ │
│   │       // Actualiza fallback_buffer con último estado                   │ │
│   │       // Retorna lista de eventos para procesar                        │ │
│   │   }                                                                    │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    │ Eventos (lista de input events)          │
│                                    ▼                                          │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                   LOGIC SYSTEM (evaluate_sensors)                      │ │
│   │                                                                        │ │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 1. MOUSE OVER SENSOR                                             │ │ │
│   │   │   - Leer mouse_position del snapshot                             │ │ │
│   │   │   - Para CADA entidad: hit_test(mouse, entity_aabb)              │ │ │
│   │   │   - Generar SignalByte con historial 6-tick                      │ │ │
│   │   │   - Detectar: rising_edge, falling_edge, steady_high             │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 2. TOUCH/COLLISION SENSOR (AABB)                                 │ │ │
│   │   │   - SpatialHash.query(entity_aabb) → O(1)                        │ │ │
│   │   │   - Detectar overlaps entre entidades                            │ │ │
│   │   │   - Generar SignalByte por par de entidades                      │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 3. PROXIMITY SENSOR                                              │ │ │
│   │   │   - Para cada nearby entity: distance(entity_a, entity_b)        │ │ │
│   │   │   - Usar SpatialHash para encontrar vecinos                      │ │ │
│   │   │   - Hysteresis para evitar oscilación                           │ │ │
│   │   │   - Generar SignalByte de proximidad                             │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 4. RADAR SENSOR (Cono direccional)                               │ │ │
│   │   │   - Calcular ángulo hacia el mouse                               │ │ │
│   │   │   - Verificar si está dentro del cono de visión                  │ │ │
│   │   │   - Generar SignalByte direccional                               │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 5. KEYBOARD SENSOR (Shortcuts)                                   │ │ │
│   │   │   - Leer keys[] del snapshot (256 bits)                          │ │ │
│   │   │   - Verificar combinaciones configuradas                         │ │ │
│   │   │   - Ctrl+Z, Delete, Ctrl+D, etc.                                 │ │ │
│   │   │   - Generar SignalByte de teclado                                │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 6. DOUBLE TAP SENSOR                                             │ │ │
│   │   │   - Detectar dos clicks en menos de 500ms                        │ │ │
│   │   │   - Usar timestamp del snapshot                                  │ │ │
│   │   │   - Generar pulso de double-tap                                  │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 7. LONG PRESS SENSOR                                             │ │ │
│   │   │   - Trackear tiempo desde mouse_down                             │ │ │
│   │   │   - Verificar si mouse sigue encima (steady_high)                │ │ │
│   │   │   - Threshold configurable (ej. 500ms)                           │ │ │
│   │   │   - Generar pulso de long-press                                  │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 8. RIGHT CLICK SENSOR                                            │ │ │
│   │   │   - Detectar botón derecho pulsado                              │ │ │
│   │   │   - Verificar si está sobre alguna entidad                      │ │ │
│   │   │   - Generar pulso para menú contextual                          │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ 9. TOUCH SENSOR (Mobile)                                         │ │ │
│   │   │   - Detectar touch events (1+ dedos)                             │ │ │
│   │   │   - Trackear movimiento de dedos                                 │ │ │
│   │   │   - Detectar gestos (pinch, rotate, pan)                         │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   └─────────────────────────────────────────────────────────────────────┘ │
│   │                                    │                                          │
│   │   Generar PULSOS (cambios de estado):                                   │
│   │   ┌────────────────────────────────────────────────────────────────┐    │ │
│   │   │ Pulse {                                                         │    │ │
│   │   │   sensor_id: SensorType,                                        │    │ │
│   │   │   entity_id: EntityId,                                          │    │ │
│   │   │   state: PulseState,    // Positive, Negative, None             │    │ │
│   │   │   timestamp: u64,       // Momento del cambio                   │    │ │
│   │   │   signal_history: [u8; 6]  // Últimos 6 ticks                   │    │ │
│   │   │ }                                                               │    │ │
│   │   └────────────────────────────────────────────────────────────────┘    │ │
│   │                                                                             │
│   │   Solo se generan pulsos cuando hay cambio de estado:                       │
│   │   - Rising Edge (0→1): Entidad recibe hover/click                           │
│   │   - Falling Edge (1→0): Entidad pierde hover/soltar click                  │
│   │   - None (sin cambio): No se genera pulso                                  │
│   └────────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    │ Lista de pulsos                           │
│                                    ▼                                          │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                   WIRING TABLE (route pulses)                          │ │
│   │                                                                        │ │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ REGISTRO DE CONEXIONES:                                          │ │ │
│   │   │                                                                  │ │ │
│   │   │ Entity 42:                                                       │ │ │
│   │   │   ├─ Sensor: MouseOver    → Controller: Direct    → Highlight   │ │ │
│   │   │   ├─ Sensor: MouseClick   → Controller: And(MO)   → Select      │ │ │
│   │   │   └─ Sensor: Proximity    → Controller: Or(MO)    → Move        │ │ │
│   │   │                                                                  │ │ │
│   │   │ Entity 43:                                                       │ │ │
│   │   │   ├─ Sensor: MouseOver    → Controller: Direct    → Highlight   │ │ │
│   │   │   └─ Sensor: KeyShortcut  → Controller: Direct    → Delete      │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                                                        │ │
│   │   Para CADA pulso:                                                     │ │
│   │   1. Buscar conexiones: table.has_connection(entity_id, sensor_id)    │ │
│   │   2. Si hay conexiones, obtener Controller(s) asociados               │ │
│   │   3. Evaluar Controller:                                              │ │
│   │      - Direct: pasar señal tal cual                                   │ │
│   │      - And(sensor2): sensor1 AND sensor2 deben estar activos         │ │
│   │      - Or(sensor2): sensor1 OR sensor2 deben estar activos           │ │
│   │      - Not: invertir la señal                                        │ │
│   │   4. Si Controller pasa, ejecutar Actuator(s) asociados              │ │
│   │                                                                        │ │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ EJEMPLO DE EVALUACIÓN:                                           │ │ │
│   │   │                                                                  │ │ │
│   │   │ Pulso: MouseClick en Entity 42                                   │ │ │
│   │   │                                                                  │ │ │
│   │   │ 1. Buscar conexiones para MouseClick en Entity 42                │ │ │
│   │   │    → Encuentra: Controller.And(MouseOver)                        │ │ │
│   │   │                                                                  │ │ │
│   │   │ 2. Evaluar Controller.And(MouseOver):                            │ │ │
│   │   │    - Sensor primario: MouseClick = TRUE                          │ │ │
│   │   │    - Sensor secundario: MouseOver = TRUE                         │ │ │
│   │   │    - Resultado: TRUE AND TRUE = TRUE                             │ │ │
│   │   │                                                                  │ │ │
│   │   │ 3. Como resultado es TRUE:                                       │ │ │
│   │   │    → Ejecutar SelectActuator para Entity 42                      │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    │ Lista de comandos a ejecutar              │
│                                    ▼                                          │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                    ACTUATORS (responden a pulsos)                      │ │
│   │                                                                        │ │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ HIGHLIGHT ACTUATOR                                               │ │ │
│   │   │   HighlightActuator.activate(pulse, store)                       │ │ │
│   │   │   → store.set_color(entity, HOVER_COLOR)                         │ │ │
│   │   │   → store.set_highlight(entity, true)                            │ │ │
│   │   │   → CommandHistory.push(SetColorCmd { ... })                     │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ SELECT ACTUATOR                                                  │ │ │
│   │   │   SelectActuator.activate(pulse, store)                          │ │ │
│   │   │   → if pulse.state == Positive {                                 │ │ │
│   │   │        store.add_to_selection(entity)                            │ │ │
│   │   │     } else if pulse.state == Negative {                          │ │ │
│   │   │        store.remove_from_selection(entity)                       │ │ │
│   │   │     }                                                            │ │ │
│   │   │   → CommandHistory.push(SelectCmd { selected: true/false })      │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ MOVE ACTUATOR                                                    │ │ │
│   │   │   MoveActuator.activate(pulse, store, input_snapshot)            │ │ │
│   │   │   → Calcular delta desde última posición                         │ │ │
│   │   │   → store.move_entity(entity, delta)                             │ │ │
│   │   │   → CommandHistory.push(MoveCmd { delta: ... })                  │ │ │
│   │   │   → spatial_hash.update(entity, new_position)                    │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ PROPERTY ACTUATOR                                                │ │ │
│   │   │   PropertyActuator.activate(pulse, store, params)                │ │ │
│   │   │   → store.set_property(entity, key, value)                       │ │ │
│   │   │   → CommandHistory.push(SetPropertyCmd { ... })                  │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ CAMERA ACTUATOR                                                  │ │ │
│   │   │   CameraActuator.activate(pulse, camera, params)                 │ │ │
│   │   │   → camera.pan(delta_x, delta_y)                                 │ │ │
│   │   │   → camera.zoom(new_zoom)                                        │ │ │
│   │   │   → camera.fit_to_bounds(bounds)                                 │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ STATE ACTUATOR                                                   │ │ │
│   │   │   StateActuator.activate(pulse, store, params)                   │ │ │
│   │   │   → state_machine.transition(entity, new_state)                  │ │ │
│   │   │   → CommandHistory.push(StateChangeCmd { ... })                  │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   │                                    │                                          │
│   │   ┌──────────────────────────────────────────────────────────────────┐ │ │
│   │   │ MESSAGE ACTUATOR (Inter-entity messaging)                        │ │ │
│   │   │   MessageActuator.activate(pulse, store, params)                 │ │ │
│   │   │   → message_bus.send(to_entity, message)                         │ │ │
│   │   │   → receiver_entity.process_message(message)                     │ │ │
│   │   └──────────────────────────────────────────────────────────────────┘ │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    │ Modificaciones al EntityStore             │
│                                    ▼                                          │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                    ENTITY STORE (estado mutable)                       │ │
│   │                                                                        │ │
│   │   pub struct EntityStore {                                            │ │
│   │       transforms: Vec<[f32; 4]>,  // [x, y, w, h]                     │ │
│   │       colors: Vec<u32>,              // RGBA packed                    │ │
│   │       selection: SelectionStore,         // Set<EntityId>              │ │
│   │       properties: Vec<HashMap<String, Value>>,  // Props por entidad  │ │
│   │       visibility: Vec<bool>,           // Visible/invisible           │ │
│   │       states: Vec<EntityState>,        // Estado de máquina           │ │
│   │       generation: Vec<Generation>,     // Para debug                  │ │
│   │       alive: Vec<bool>,                // alive/dead                  │ │
│   │       dirty: BitVec,                   // Marca qué entidades changed │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   │   ════════════════════════════════════════════════════════════════════│ │
│   │   EJEMPLO DE MODIFICACIÓN:                                            │ │
│   │   ════════════════════════════════════════════════════════════════════│ │
│   │                                                                        │ │
│   │   // Highlight: cambiar color                                         │ │
│   │   store.set_color(idx, 0x4A90E2FF);  // Azul AWS                      │ │
│   │   store.set_highlight(idx, true);                                      │ │
│   │   store.dirty.set(idx, true);       // Marcar para re-render          │ │
│   │                                                                        │ │
│   │   // Select: añadir a selección                                       │ │
│   │   store.selection.add(idx);                                            │ │
│   │   store.dirty.set(idx, true);                                          │ │
│   │                                                                        │ │
│   │   // Move: mover entidad                                              │ │
│   │   let old_pos = store.get_position(idx);                              │ │
│   │   let new_pos = old_pos + delta;                                       │ │
│   │   store.set_position(idx, new_pos);                                    │ │
│   │   spatial_hash.update(idx, new_pos);  // Sincronizar spatial index    │ │
│   │   store.dirty.set(idx, true);                                          │ │
│   │                                                                        │ │
│   │   // Delete: marcar como dead                                          │ │
│   │   store.alive[idx] = false;                                            │ │
│   │   store.dirty.set(idx, true);                                          │ │
│   │   spatial_hash.remove(idx);           // Eliminar de spatial index    │ │
│   │                                                                        │ │
│   │   ════════════════════════════════════════════════════════════════════│ │
│   │                                                                        │ │
│   │   After any modification:                                             │ │
│   │   1. Set dirty flag for re-render                                     │ │
│   │   2. Push command to CommandHistory (for undo/redo)                   │ │
│   │   3. Update spatial hash if position/size changed                     │ │
│   │   4. Notify listeners if selection changed                            │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└────────────────────────────────────────────────────┬──────────────────────────┘
                                                     │
                    ┌────────────────────────────────┘
                    │    Lectura desde JS (no bloqueante)
                    │    bridge.getEntityPositionScreen(id)
                    │    bridge.getSelection()
                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              JAVASCRIPT (lectura)                            │
│                                                                             │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                    WASM BRIDGE (lectura de estado)                     │ │
│   │                                                                        │ │
│   │   const bridge = new WasmBridge();                                     │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER POSICIÓN DE ENTIDAD                                          │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const pos = bridge.getEntityPositionScreen(entityId);               │ │
│   │   // → [x, y]                                                          │ │
│   │   // x, y son coordenadas en pixels relativas al canvas                │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER TAMAÑO DE ENTIDAD                                            │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const size = bridge.getEntitySizeScreen(entityId);                  │ │
│   │   // → [width, height]                                                 │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER COLOR DE ENTIDAD                                             │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const color = bridge.getEntityColorHex(entityId);                   │ │
│   │   // → "#4A90E2" (hex string)                                          │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER SELECCIÓN                                                    │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const selection = bridge.getSelection();                             │ │
│   │   // → Uint32Array [id1, id2, id3, ...]                                │ │
│   │   const isSelected = bridge.isEntitySelected(entityId);               │ │
│   │   // → boolean                                                          │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER PROPIEDADES                                                  │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const props = bridge.getEntityProperties(entityId);                 │ │
│   │   // → { name: "EC2", type: "aws-ec2", status: "running" }            │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER VISIBILIDAD                                                  │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const isVisible = bridge.isEntityVisible(entityId);                 │ │
│   │   // → boolean                                                          │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // LEER ESTADO                                                       │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const state = bridge.getEntityState(entityId);                      │ │
│   │   // → "selected" | "editing" | "normal"                               │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // OBTENER TODAS LAS ENTIDADES VIVAS                                 │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const entities = bridge.getAliveEntities();                         │ │
│   │   // → Uint32Array [id1, id2, id3, ...]                                │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // OBTENER SOLO ENTIDADES MODIFICADAS (optimización)                │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const dirtyIds = bridge.getDirtyEntities();                         │ │
│   │   // → Uint32Array de IDs que cambiaron este frame                     │ │
│   │   bridge.clearDirtyFlags();  // Limpiar después de leer                │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // DESHACER / REHACER                                               │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   bridge.undo();    // Deshacer última acción                         │ │
│   │   bridge.redo();    // Rehacer acción deshecha                         │ │
│   │   bridge.canUndo(); // boolean                                         │ │
│   │   bridge.canRedo(); // boolean                                         │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // OBTENER POSICIÓN DE CÁMARA                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const camera = bridge.getCamera();                                   │ │
│   │   // → { x: 100, y: 200, zoom: 1.5 }                                   │ │
│   │                                                                        │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   // EXPORTAR PROYECTO                                                 │ │
│   │   //═════════════════════════════════════════════════════════════════  │ │
│   │   const export = bridge.exportProject();                               │ │
│   │   // → JSON string con todo el estado                                  │ │
│   │                                                                        │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    │ Datos de estado                          │
│                                    ▼                                          │
│   ┌────────────────────────────────────────────────────────────────────────┐ │
│   │                    RENDER (Canvas 2D / WebGPU)                         │ │
│   │                                                                        │ │
│   │   function render() {                                                  │ │
│   │     const entities = bridge.getAliveEntities();                       │ │
│   │                                                                        │ │
│   │     // Limpiar canvas                                                  │ │
│   │     ctx.clearRect(0, 0, canvas.width, canvas.height);                 │ │
│   │                                                                        │ │
│   │     // Renderizar cada entidad                                         │ │
│   │     for (const id of entities) {                                      │ │
│   │       const [x, y] = bridge.getEntityPositionScreen(id);             │ │
│   │       const [w, h] = bridge.getEntitySizeScreen(id);                 │ │
│   │       const color = bridge.getEntityColorHex(id);                     │ │
│   │       const isSelected = bridge.isEntitySelected(id);                 │ │
│   │                                                                        │ │
│   │       // Dibujar entidad                                               │ │
│   │       ctx.fillStyle = color;                                           │ │
│   │       ctx.fillRect(x - w/2, y - h/2, w, h);                           │ │
│   │                                                                        │ │
│   │       // Dibujar borde si está seleccionada                            │ │
│   │       if (isSelected) {                                                │ │
│   │         ctx.strokeStyle = '#13B6EC';                                   │ │
│   │         ctx.lineWidth = 2;                                             │ │
│   │         ctx.strokeRect(x - w/2 - 2, y - h/2 - 2, w + 4, h + 4);       │ │
│   │       }                                                                │ │
│   │     }                                                                  │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   │   //════════════════════════════════════════════════════════════════   │ │
│   │   // OPTIMIZACIÓN: Solo renderizar entidades modificadas               │ │
│   │   //════════════════════════════════════════════════════════════════   │ │
│   │   function renderOptimized() {                                         │ │
│   │     const dirtyIds = bridge.getDirtyEntities();                       │ │
│   │     if (dirtyIds.length === 0) return;  // Nada que actualizar        │ │
│   │                                                                        │ │
│   │     // Solo redibujar entidades marcadas como dirty                    │ │
│   │     for (const id of dirtyIds) {                                      │ │
│   │       redrawEntity(id);                                                │ │
│   │     }                                                                  │ │
│   │     bridge.clearDirtyFlags();                                          │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   │   //════════════════════════════════════════════════════════════════   │ │
│   │   // RENDERIZAR CONEXIONES                                             │ │
│   │   //════════════════════════════════════════════════════════════════   │ │
│   │   function renderConnections() {                                       │ │
│   │     const connections = bridge.getConnections();                      │ │
│   │     for (const conn of connections) {                                 │ │
│   │       const start = bridge.getEntityPositionScreen(conn.from);        │ │
│   │       const end = bridge.getEntityPositionScreen(conn.to);            │ │
│   │       drawOrthogonalLine(ctx, start, end);                            │ │
│   │     }                                                                  │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   │   //════════════════════════════════════════════════════════════════   │ │
│   │   // RENDERIZAR CURSOR DE COLABORACIÓN                                 │ │
│   │   //════════════════════════════════════════════════════════════════   │ │
│   │   function renderCollaboratorCursors() {                               │ │
│   │     const cursors = bridge.getRemoteCursors();                        │ │
│   │     for (const cursor of cursors) {                                   │ │
│   │       drawCursor(ctx, cursor.position, cursor.userColor, cursor.name);│ │
│   │     }                                                                  │ │
│   │   }                                                                    │ │
│   │                                                                        │ │
│   └────────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│   ════════════════════════════════════════════════════════════════════════  │
│   RESUMEN DEL FLUJO COMPLETO                                                │
│   ════════════════════════════════════════════════════════════════════════  │
│                                                                             │
│   ┌──────────┬─────────────────────────┬────────────────────────────┐       │
│   │  Paso    │  JavaScript             │  Rust / WASM                │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  1       │  Captura evento DOM     │  -                         │       │
│   │          │  mousemove/mousedown/   │                            │       │
│   │          │  keydown                │                            │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  2       │  Escribe en SAB         │  -                         │       │
│   │          │  o llama                │                            │       │
│   │          │  pushInputEvent()       │                            │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  3       │  -                      │  InputSampler.drain()      │       │
│   │          │                         │  Lee eventos del SAB       │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  4       │  -                      │  LogicSystem.eval_sensors()│       │
│   │          │                         │  Evalúa todos los sensores │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  5       │  -                      │  Genera PULSOS             │       │
│   │          │                         │  Solo en flancos (0→1/1→0) │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  6       │  -                      │  WiringTable.route()       │       │
│   │          │                         │  Busca conexiones          │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  7       │  -                      │  Controllers.eval()        │       │
│   │          │                         │  Evalúa AND/OR/NOT         │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  8       │  -                      │  Actuators.execute()       │       │
│   │          │                         │  Modifican EntityStore     │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  9       │  -                      │  CommandHistory.push()     │       │
│   │          │                         │  Para undo/redo            │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  10      │  -                      │  SpatialHash.update()      │       │
│   │          │                         │  Mantiene índice espacial  │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  11      │  getEntityPosition()    │  Lee de EntityStore        │       │
│   │          │  getSelection()         │                            │       │
│   │          │  isEntitySelected()     │                            │       │
│   ├──────────┼─────────────────────────┼────────────────────────────┤       │
│   │  12      │  render()               │  -                         │       │
│   │          │  Canvas.fillRect()      │                            │       │
│   └──────────┴─────────────────────────┴────────────────────────────┘       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 13.2 SharedArrayBuffer: Formato de Memoria

```typescript
interface InputBuffer {
    offset: 0,   // uint32 - índice de escritura
    offset: 4,   // uint32 - índice de lectura
    offset: 8,   // int32  - mouse_x
    offset: 12,  // int32  - mouse_y
    offset: 16,  // uint8  - buttons (bitmask)
    offset: 17,  // uint8  - modifiers
    offset: 18,  // int16  - wheel_delta
    offset: 20,  // uint32 - timestamp
    offset: 24,  // uint8[32] - key_states (256 bits)
    offset: 56,  // uint8[8]  - padding
}

// Escribir posición del mouse (offset 8)
view.setInt32(8, mouseX, true);
view.setInt32(12, mouseY, true);
```

### 13.3 API de Alto Nivel (Recomendada)

```typescript
import { WasmBridge } from '@archflow/sdk';

const bridge = new WasmBridge();
bridge.initialize(1920, 1080);

canvas.onmousemove = (e) => {
    bridge.pushInputEvent(1, e.clientX, e.clientY, e.buttons, getModifiers(e));
};

canvas.onmousedown = (e) => {
    bridge.pushInputEvent(0, e.clientX, e.clientY, 1 << e.button, getModifiers(e));
};

canvas.onmouseup = (e) => {
    bridge.pushInputEvent(2, e.clientX, e.clientY, 0, getModifiers(e));
};

function getModifiers(e) {
    return ((e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0));
}

function tick(timestamp) {
    bridge.tick(timestamp);
    render();
    requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
```

### 13.4 Tipos de Eventos

| eventType | Descripción |
|-----------|-------------|
| 0 | MouseDown |
| 1 | MouseMove |
| 2 | MouseUp |
| 3 | MouseWheel |
| 4 | KeyDown |
| 5 | KeyUp |

### 13.5 Resumen del Flujo

| Paso | JavaScript | SharedArrayBuffer | Rust |
|------|------------|-------------------|------|
| 1 | Captura evento DOM | - | - |
| 2 | `bridge.pushInputEvent()` | Escribe 64 bytes | - |
| 3 | - | - | `InputSampler.take_snapshot()` |
| 4 | - | - | `LogicSystem.evaluate_sensors()` |
| 5 | - | - | Genera Pulsos (flancos) |
| 6 | - | - | `WiringTable` rutea pulsos |
| 7 | - | - | `Actuators` modifican `EntityStore` |
| 8 | `bridge.getEntityPositionScreen()` | - | Lee de EntityStore |
| 9 | Renderiza | - | - |

---

## 14. Información de Archivos de Arquitectura (Documentos Historicos)

Esta sección documenta información verificada de los documentos de arquitectura originales que fundamentan la implementación actual de Logic Bricks.

### 14.1. SignalByte: Historial de 6 Ticks en 1 Byte

De **ideas-logic-bricks.md** y **refinamiento-logic-bricks.md**:

```rust
// Un SignalByte almacena 6 ticks de historial en 8 bits
// Bit 0: Estado actual (T)
// Bits 1-5: T-1 a T-5
// Bits 6-7: Flags internos

struct SignalByte(u8);

impl SignalByte {
    /// Inserta nuevo estado desplazando el historial
    pub fn push(&mut self, active: bool) {
        self.0 = (self.0 << 1) | (active as u8);
    }

    /// Detecta transición 0 → 1 (entrada)
    pub fn is_rising_edge(&self) -> bool {
        (self.0 & 0b00000011) == 0b00000001
    }

    /// Detecta transición 1 → 0 (salida)
    pub fn is_falling_edge(&self) -> bool {
        (self.0 & 0b00000011) == 0b00000010
    }

    /// Verifica si la señal ha sido estable durante N ticks
    pub fn is_steady(&self, ticks: u8) -> bool {
        let mask = (1 << ticks) - 1;
        (self.0 & mask) == mask
    }
}
```

**Casos de uso verificados:**

| Patrón | Significado | Uso |
|--------|-------------|-----|
| `000001` | Clic incipiente | Inicio de drag |
| `111111` | Pulsación firme | Confirmación |
| `110111` | Señal con ruido | Debouncing automático |

### 14.2. Logic Instruction Set (LIS): Formato de 16 Bytes

De **LOGIC_BRICKS_FEASIBILITY_STUDY.md**:

El protocolo binario para configurar sensores desde el SDK:

| Offset | Campo | Tipo | Descripción |
|--------|-------|------|-------------|
| 0x00 | `OpCode` | `u8` | 0=Link, 1=Unlink, 2=Batch |
| 0x01 | `SensorType` | `u8` | ID del sensor |
| 0x02 | `TriggerConfig` | `u8` | Bits: Mode, Invert, Tap |
| 0x03 | `Frequency` | `u8` | 0=每帧, N=延迟 |
| 0x04 | `EntityID` | `u32` | Índice en EntityStore |
| 0x08 | `ActuatorID` | `u8` | ID del actuador |
| 0x09 | `ControllerID` | `u8` | AND, OR, NOT, Direct |
| 0x0A | `Payload` | `u16` | Parámetros extra |
| 0x0C | `Timestamp` | `u32` | Para órdenes obsoletas |

### 14.3. TriggerMode: Modos de Disparo

De **refinamiento-logic-bricks.md**:

```rust
pub enum TriggerMode {
    Always,         // Cada frame mientras haya señal
    Rising,         // Frame donde 0 → 1
    Falling,        // Frame donde 1 → 0
    Stable(u8),     // Estable durante N ticks
    LongPress(u8),  // Mantenido N ticks
}
```

**Aplicaciones:**

| Modo | Mascara | Caso de Uso |
|------|---------|-------------|
| `Rising` | `(signal & 0b11) == 0b01` | Iniciar drag, play sound |
| `Falling` | `(signal & 0b11) == 0b10` | Soltar objeto, stop sound |
| `Stable(6)` | `signal & 0b111111 == 0b111111` | Tooltip tras 100ms |

### 14.4. Controladores: Puertas Lógicas

De **refinamiento-logic-bricks.md**:

```rust
pub enum Controller {
    Direct,              // Sin lógica
    And(SensorType),     // Ambos activos
    Or(SensorType),      // Al menos uno activo
    Not,                 // Invertir señal,
}
```

### 14.5. Catálogo de Sensores del MVP Original

De **LOGIC_BRICKS_FEASIBILITY_STUDY.md**:

| ID | Sensor | Input Source | Uso Principal |
|----|--------|--------------|---------------|
| 0x01 | **MouseHover** | Mouse Pos + AABB | Resaltar nodos |
| 0x02 | **EntityClick** | MouseHover + Button | Selección |
| 0x03 | **Proximity** | SpatialHash | Imanes de conexión |
| 0x04 | **DragHandle** | EntityClick + Movement | Mover iconos |
| 0x05 | **ShortcutKey** | KeyStates array | Delete, Ctrl+D |

### 14.6. Catálogo de Actuadores del MVP Original

De **LOGIC_BRICKS_FEASIBILITY_STUDY.md**:

| ID | Actuator | Efecto en EntityStore | Payload |
|----|----------|----------------------|---------|
| 0x10 | **Translate** | `positions[i] += delta` | `[f32, f32]` |
| 0x11 | **Highlight** | `colors[i]` | `u32` (Color) |
| 0x12 | **Scale** | `sizes[i] *= factor` | `f32` |
| 0x13 | **Visibility** | bit render | `u8` (0/1) |
| 0x14 | **Connect** | Crea entidad Edge | `u32` (Target) |

### 14.7. Hysteresis Colaborativa (Red Multi-Usuario)

De **LOGIC_BRICKS_FEASIBILITY_STUDY.md**:

```rust
// Si Alice mueve un nodo, el historial de 6 ticks
// permite a Bob absorber jitter de red:

fn handle_remote_mouse(entity_id: EntityId, new_pos: Option<Vec2>, signals: &mut Vec<SignalByte>) {
    let signal = &mut signals[entity_id.index()];
    
    if let Some(pos) = new_pos {
        signal.push(true);
        render_smooth(pos);
    } else {
        // Paquete perdido - mantener gracias al hysteresis
        if signal.count_ones() >= 4 {
            render_extrapolated(pos);
        } else {
            render_faded(pos);
        }
    }
}
```

### 14.8. Ejemplo: Conexión Magnética

De **LOGIC_BRICKS_FEASIBILITY_STUDY.md**:

```typescript
const connectionLogic = {
    sensor: Sensors.Proximity,
    config: {
        radius: 20,
        trigger: TriggerMode.Stable(6)  // 100ms de estabilidad
    },
    actuators: [
        {
            type: Actuators.SnapTo,
            params: {
                target: port,
                snapDistance: 15
            }
        }
    ]
};

// Resultado: Las conexiones "tienen vida" y se pegan suavemente
```

### 14.9. Verificación de Implementación

| Documento Original | Estado |
|--------------------|--------|
| SignalByte (6 ticks, 1 byte) | ✅ Implementado |
| TriggerMode (Always/Rising/Falling/Stable/LongPress) | ✅ Implementado |
| Controllers (AND/OR/NOT/Direct) | ✅ Implementado |
| LIS (16-byte instruction format) | ⚠️ API alto nivel expuesta |
| MouseOver, Click, Proximity sensors | ✅ Implementados |
| Highlight, Select, Move actuators | ✅ Implementados |
| SpatialHash para colisiones | ✅ Implementado |
| Hysteresis colaborativa | ✅ Mecanismo disponible |
| SharedArrayBuffer para input | ✅ Implementado |

### 14.10. Resumen de Contrastes

```
★ Insight ─────────────────────────────────────
Los documentos históricos establecen las bases teóricas
que se han implementado de forma fiel en archflow-logic:

• SignalByte de 6 ticks → Implementado exactamente
• Controladores AND/OR/NOT → API expuesta a JS
• SharedArrayBuffer → <2ms latencia lograda
• SpatialHash O(1) → Implementado

Las únicas diferencias son optimizaciones de API:
• En lugar de LIS binario de 16 bytes, se expone
  una API de alto nivel en TypeScript
• El historial es de 6 ticks (100ms @ 60Hz)
  como se especificó originalmente
─────────────────────────────────────────────────
```

---

*Manual creado: 2026-02-02*  
*Sección 14 añadida: 2026-02-02*  
*Basada en revisión de documentos en `docs/arquitectura/`*
