# Análisis API Fluida Rust - LOGIC_BRICKS_DEVELOPER_GUIDE.md

## 📌 Estado Actual vs Guide

### Lo que YA existe ✅

| API | En Código | Archivo |
|-----|-----------|---------|
| `WiringBuilder` | ✅ Completo | `crates/archflow-sdk/src/wiring.rs` |
| `Sensor` trait | ✅ Completo | `crates/archflow-sdk/src/sensors.rs` |
| `Actuator` trait | ✅ Completo | `crates/archflow-sdk/src/actuators.rs` |
| `SnapConfig`/`Snapper` | ✅ Completo | `crates/archflow-sdk/src/snap.rs` |
| Bridge básico | ✅ Funcional | `crates/archflow-web/src/bridge.rs` |

### Lo que FALTA ❌

| API | En Guide | Estado | Ubicación |
|-----|----------|--------|-----------|
| `poll_events()` | ✅ L365-367 | ❌ No existe | bridge.rs |
| `EventRingBuffer` | ✅ L346-360 | ❌ No existe | archflow-logic |
| Behavior builders | ✅ L610-617 | ❌ No existe | archflow-sdk |
| Shape builders TS | ✅ L736-741 | ✅ Parcial | archflow-web-ui |

---

## 🔍 Análisis Comparativo Detallado

### 1. WiringBuilder (YA EXISTE - API FLUIDA)

```rust
// CÓDIGO ACTUAL - crates/archflow-sdk/src/wiring.rs

let wiring = WiringBuilder::new()
    .connect(0, 10)                    // Sensor 0 → Actuator 10
    .on_entities_with_tag("button")   // Filter: tag
    .on_positive()                    // Filter: state
    .connect(1, 11)                   // Nueva conexión
    .on_entities_in_layer(5)           // Filter: layer
    .build();
```

**Veredicto**: ✅ API FLUIDA COMPLETA - NO IMPLEMENTAR

---

### 2. Sensor/Actuator Traits (YA EXISTEN)

```rust
// CÓDIGO ACTUAL - crates/archflow-sdk/src/sensors.rs

pub trait Sensor {
    fn evaluate(&mut self, ctx: &SensorContext) -> SensorState;
    fn config(&self) -> &SensorConfig;
    fn reset(&mut self);
}
```

**Veredicto**: ✅ INTERFACES COMPLETAS - NO IMPLEMENTAR

---

### 3. Lo que SÍ FALTA: poll_events()

```rust
// GUIDE - L365-367
impl Engine {
    pub fn poll_events(&mut self) -> Vec<LogicEvent> {
        // Una sola llamada para todos los eventos del frame
    }
}
```

**Dónde va**: `crates/archflow-web/src/bridge.rs` + `archflow-logic/src/events.rs`

```rust
// PROPUESTA: bridge.rs

#[wasm_bindgen]
impl WasmBridge {
    /// Poll all events from the EventRingBuffer
    ///
    /// # Returns
    /// Array of LogicEvent structs:
    /// - { type: "EntitySelected", entity_id: u32, timestamp: u64 }
    /// - { type: "ProximityAlert", entity_id: u32, distance: f32, timestamp: u64 }
    /// - { type: "DragStarted", entity_id: u32, timestamp: u64 }
    /// - { type: "DragEnded", entity_id: u32, timestamp: u64 }
    #[wasm_bindgen]
    pub fn poll_events(&self) -> js_sys::Array {
        let events = self.engine.borrow().poll_events();
        
        let array = js_sys::Array::new();
        for event in events {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"type".into(), &event.event_type_string());
            js_sys::Reflect::set(&obj, &"entity_id".into(), &(event.entity_id as f64));
            js_sys::Reflect::set(&obj, &"timestamp".into(), &(event.timestamp as f64));
            array.push(&obj);
        }
        array
    }
}
```

---

### 4. Lo que SÍ FALTA: EventRingBuffer

```rust
// PROPUESTA: crates/archflow-logic/src/events.rs

#[derive(Clone, Copy, Debug)]
pub enum LogicEventType {
    EntitySelected,
    ProximityAlert,
    DragStarted,
    DragEnded,
    EntityDestroyed,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicEvent {
    pub event_type: LogicEventType,
    pub entity_id: u32,
    pub timestamp: u64,
    pub data: EventData,
}

#[derive(Clone, Copy, Debug)]
pub enum EventData {
    None,
    Proximity { distance: f32 },
    Drag { start_pos: (f32, f32), current_pos: (f32, f32) },
}

pub struct EventRingBuffer {
    events: Vec<LogicEvent>,
    capacity: usize,
}

impl EventRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, event: LogicEvent) {
        if self.events.len() < self.capacity {
            self.events.push(event);
        }
    }

    pub fn drain(&mut self) -> Vec<LogicEvent> {
        core::mem::take(&mut self.events)
    }
}
```

---

### 5. Shape Builder (PARCIAL - en TypeScript)

```typescript
// CÓDICO ACTUAL - archflow-web-ui/src/sdk/ShapeBuilder.ts

export class ShapeBuilder {
    private entity: number;
    
    static rectangle(x: number, y: number, w: number, h: number) {
        return new ShapeBuilder(engine.spawn_rectangle(x, y, w, h));
    }
    
    color(hex: string): ShapeBuilder {
        this.engine.set_entity_color(this.entity, hex);
        return this;
    }
    
    attach(behavior: Behavior): ShapeBuilder {
        this.engine.attach_behavior(this.entity, behavior);
        return this;
    }
    
    build(): EntityId {
        return this.entity;
    }
}

// USO
const rect = ShapeBuilder
    .rectangle(100, 100, 50, 50)
    .color('#3b82f6')
    .attach(Behaviors.DragDrop.default())
    .attach(Behaviors.Selection.singleClick())
    .build();
```

**Veridicto**: ✅ YA EXISTE EN TYPESCRIPT - NO IMPLEMENTAR EN RUST

---

## 📋 API FLUIDA RUST: LO QUE FALTA

### Resumen de Implementación Requerida

| Item | Dónde | Esfuerzo | Prioridad |
|------|--------|----------|-----------|
| `EventRingBuffer` | `archflow-logic/src/events.rs` | S | Alta |
| `LogicEvent` types | `archflow-logic/src/events.rs` | XS | Alta |
| `poll_events()` bridge | `archflow-web/src/bridge.rs` | S | Alta |
| `LogicSystem::emit_event()` | `archflow-logic/src/logic_system.rs` | S | Media |

---

## 🔧 Plan de Implementación API

### Fase 1: Event Types (XS)

```rust
// crates/archflow-logic/src/events.rs

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicEventType {
    EntitySelected = 0,
    ProximityAlert = 1,
    DragStarted = 2,
    DragEnded = 3,
    EntityDestroyed = 4,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicEvent {
    pub event_type: LogicEventType,
    pub entity_id: u32,
    pub timestamp: u64,
    pub payload: EventPayload,
}

#[derive(Clone, Copy, Debug)]
pub enum EventPayload {
    None,
    Proximity { distance: f32 },
    Drag { start_x: f32, start_y: f32, current_x: f32, current_y: f32 },
}
```

### Fase 2: EventRingBuffer (S)

```rust
// crates/archflow-logic/src/events.rs

pub struct EventRingBuffer {
    events: Vec<LogicEvent>,
    capacity: usize,
    write_idx: usize,
}

impl EventRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity,
            write_idx: 0,
        }
    }

    pub fn push(&mut self, event: LogicEvent) {
        if self.events.len() < self.capacity {
            self.events.push(event);
        } else {
            // Ring buffer overwrite (circular)
            self.events[self.write_idx] = event;
            self.write_idx = (self.write_idx + 1) % self.capacity;
        }
    }

    pub fn drain(&mut self) -> Vec<LogicEvent> {
        self.events.drain(..).collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.write_idx = 0;
    }
}
```

### Fase 3: Integración en LogicSystem (S)

```rust
// crates/archflow-logic/src/logic_system.rs

impl LogicSystem {
    // Añadir campo
    event_buffer: EventRingBuffer,

    pub fn new() -> Self {
        Self {
            // ... existentes
            event_buffer: EventRingBuffer::new(1024),
        }
    }

    pub fn emit_event(&mut self, event: LogicEvent) {
        self.event_buffer.push(event);
    }

    pub fn poll_events(&mut self) -> Vec<LogicEvent> {
        self.event_buffer.drain()
    }
}
```

### Fase 4: Exposición WASM (S)

```rust
// crates/archflow-web/src/bridge.rs

#[wasm_bindgen]
impl WasmBridge {
    /// Poll all logic events from the buffer
    ///
    /// Returns a JavaScript array of event objects.
    /// Each event has:
    /// - type: string ("EntitySelected", "ProximityAlert", etc.)
    /// - entity_id: number
    /// - timestamp: number
    /// - data: object (event-specific data)
    #[wasm_bindgen]
    pub fn poll_events(&self) -> Result<js_sys::Array, JsValue> {
        let engine = self.engine.borrow();
        let Some(ref engine) = *engine else {
            return Err(JsValue::from_str("Engine not initialized"));
        };

        let events = engine.logic_system.poll_events();
        let array = js_sys::Array::new();

        for event in events {
            let obj = js_sys::Object::new();
            
            let type_str = match event.event_type {
                LogicEventType::EntitySelected => "EntitySelected",
                LogicEventType::ProximityAlert => "ProximityAlert",
                LogicEventType::DragStarted => "DragStarted",
                LogicEventType::DragEnded => "DragEnded",
                LogicEventType::EntityDestroyed => "EntityDestroyed",
            };
            
            js_sys::Reflect::set(&obj, &"type".into(), &type_str.into())?;
            js_sys::Reflect::set(&obj, &"entity_id".into(), &(event.entity_id as f64))?;
            js_sys::Reflect::set(&obj, &"timestamp".into(), &(event.timestamp as f64))?;
            
            array.push(&obj);
        }

        Ok(array)
    }
}
```

---

## 📊 Comparación: Guide vs Implementación

| API del Guide | Implementado | Dónde |
|--------------|--------------|-------|
| `EventRingBuffer` | ❌ Falta | events.rs |
| `LogicEvent` enum | ❌ Falta | events.rs |
| `poll_events()` | ❌ Falta | bridge.rs |
| `LogicSystem::emit()` | ❌ Falta | logic_system.rs |
| `WiringBuilder` | ✅ Listo | wiring.rs |
| `Sensor` trait | ✅ Listo | sensors.rs |
| `Actuator` trait | ✅ Listo | actuators.rs |
| `SnapConfig` | ✅ Listo | snap.rs |

---

## 🎯 Conclusión

**Lo que SÍ hay que implementar en Rust:**

1. **`EventRingBuffer`** + **`LogicEvent`** types → `crates/archflow-logic/src/events.rs`
2. **Integración en `LogicSystem`** → `crates/archflow-logic/src/logic_system.rs`
3. **Exposición WASM `poll_events()`** → `crates/archflow-web/src/bridge.rs`

**Lo que NO hay que implementar (YA EXISTE):**
- `WiringBuilder` (API fluida completa)
- `Sensor`/`Actuator` traits
- `SnapConfig`/`Snapper`
- `ShapeBuilder` (existe en TypeScript)
