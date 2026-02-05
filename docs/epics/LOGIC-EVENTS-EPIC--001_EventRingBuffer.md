# Épica: EventRingBuffer - Comunicación JS↔Rust Zero-Cost

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-LOGIC-EVENTS-001 |
| Prioridad | Alta |
| Estimación | M |
| Estado | Borrador |
| Versión | 0.1.0 |
| Análisis Previo | SOLID analysis completado en conversación |

## 🎯 Objetivo de Negocio

Implementar la capa de **event output** faltante en la arquitectura actual para permitir comunicación **Rust → JavaScript** sin callbacks, eliminando el overhead de cruce de frontera JS↔WASM durante el game loop.

**Problema actual**: Los eventos de lógica (EntitySelected, ProximityAlert, DragStarted/Ended) se pierden porque no existe un mecanismo de output hacia JS.

**Solución propuesta**: EventRingBuffer con polling en una sola llamada por frame.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-logic` (Event Output)
- **Aggregate Root**: `EventRingBuffer`
- **Domain Events**: `LogicEvent` (EntitySelected, ProximityAlert, DragStarted, DragEnded)
- **Services**: `LogicSystem::drain_events_to_ring_buffer()`

## 📖 Contexto Arquitectural

### Sistema Actual (Lógica de 3 capas)

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA DE SALIDA (JS Events)                  │
│  ┌─────────────────────────────────────────────────────────────┐
│  │  EventRingBuffer (FALTANTE - ESTA ÉPICA)                     │
│  │  - EntitySelected                                            │
│  │  - ProximityAlert                                            │
│  │  - DragStarted/Ended                                         │
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ Polling (poll_events)
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    CAPA DE LÓGICA (Logic System)                 │
│  ┌──────────────────────┐  ┌─────────────────────────────────┐ │
│  │    InputSampler      │  │    PulseBus + LogicMappingTable  │ │
│  │    (YA EXISTE)       │  │    (YA EXISTE)                   │ │
│  │  - Mouse position     │  │  - Sensors → Actuators wiring     │ │
│  │  - Keyboard state     │  │  - Pulses (Positive/Negative)   │ │
│  │  - Button events      │  │  - SignalByte history            │ │
│  └──────────────────────┘  └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Principios SOLID Aplicados

| Principio | Aplicación |
|-----------|------------|
| **SRP** | EventRingBuffer solo gestiona eventos de output |
| **OCP** | Añade funcionalidad sin modificar InputSampler/PulseBus |
| **LSP** | LogicEvent puede extenderse con nuevos tipos |
| **ISP** | Interfaz mínima: push() + drain() |
| **DIP** | Depende de abstracciones, no de concrete types |

## 📖 Historias de Usuario

### HU-LOGIC-EVENTS-001: EventRingBuffer Core

**Como** desarrollador del motor
**Quiero** un EventRingBuffer que almacene eventos de lógica
**Para** que JS pueda pollarlos sin callbacks

#### Criterios de Aceptación
- [ ] `LogicEvent` enum con EntitySelected, ProximityAlert, DragStarted, DragEnded
- [ ] `EventRingBuffer` struct con capacidad configurable
- [ ] `push()` para añadir eventos (O(1))
- [ ] `drain()` para extraer todos los eventos (consume buffer)
- [ ] Tests unitarios con 100% coverage

#### Tareas Técnicas
- [ ] Crear `crates/archflow-logic/src/events.rs`
- [ ] Definir `LogicEvent` enum
- [ ] Implementar `EventRingBuffer` con `Vec<LogicEvent>`
- [ ] Implementar `push()` y `drain()`
- [ ] Escribir tests unitarios
- [ ] Verificar con `cargo test -p archflow-logic`

#### Investigación Previa
- [x] Patrón Ring Buffer identificado
- [x] Especificación en LOGIC_BRICKS_DEVELOPER_GUIDE.md L334-360
- [x] Análisis SOLID completado

#### Estimación: S
#### Estado: Pendiente

---

### HU-LOGIC-EVENTS-002: poll_events() en Engine

**Como** bridge WASM
**Quiero** exponer poll_events() a JavaScript
**Para** que JS pueda obtener todos los eventos del frame en una llamada

#### Criterios de Aceptación
- [ ] `LogicSystem::poll_events()` devuelve `Vec<LogicEvent>`
- [ ] Consumidos eventos son removidos del buffer
- [ ] Expuesto via `bridge.rs` como `poll_events()`
- [ ] TS puede llamar `engine.poll_events()` una vez por frame
- [ ] Tests de integración JS↔Rust

#### Tareas Técnicas
- [ ] Añadir `poll_events()` a `LogicSystem`
- [ ] Integrar con `EventRingBuffer`
- [ ] Exponer en `archflow-web/src/bridge.rs`
- [ ] Generar TypeScript bindings
- [ ] Test de integración end-to-end

#### Estimación: M
#### Estado: Pendiente

---

### HU-LOGIC-EVENTS-003: Integración con Actuators

**Como** sistema de lógica
**Quiero** que los actuators emitan eventos al RingBuffer
**Para** que Selection, Drag y Proximity se comuniquen a JS

#### Criterios de Aceptación
- [ ] SelectActuator emite `EntitySelected` events
- [ ] Drag handlers emiten `DragStarted/Ended` events
- [ ] ProximitySensor emite `ProximityAlert` events
- [ ] Eventos incluyen entity_id y timestamp
- [ ] Eventos son thread-safe

#### Tareas Técnicas
- [ ] Modificar `SelectActuator` para emitir eventos
- [ ] Crear `DragStarted/Ended` handlers
- [ ] Integrar `ProximitySensor` con EventRingBuffer
- [ ] Tests de integración

#### Estimación: M
#### Estado: Pendiente

---

## 🔬 Investigación por Historia

### Patrón Ring Buffer en Rust

```rust
// crates/archflow-logic/src/events.rs

#[derive(Clone, Copy, Debug)]
pub enum LogicEventType {
    EntitySelected,
    ProximityAlert,
    DragStarted,
    DragEnded,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicEvent {
    pub event_type: LogicEventType,
    pub entity_id: u32,
    pub timestamp: u64,
    // Datos específicos del evento
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
        // En producción: buffer circular o warning
    }

    pub fn drain(&mut self) -> Vec<LogicEvent> {
        core::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
```

### Benchmark Expectativas

| Métrica | Sin RingBuffer | Con RingBuffer |
|---------|----------------|----------------|
| JS↔Rust calls/frame | N callbacks | 1 poll |
| Frame time overhead | Variable | <0.1ms |
| Memory/100k events | N/A | capacity × event_size |

## 📊 Estado de Tareas - Documentación Vivo

| Historia | Estado | Tests | Debt Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-LOGIC-EVENTS-001 | ⏳ Pendiente | 0/8 | - | - |
| HU-LOGIC-EVENTS-002 | ⏳ Pendiente | 0/5 | - | - |
| HU-LOGIC-EVENTS-003 | ⏳ Pendiente | 0/6 | - | - |

## 📋 Criterios de Éxito

- [ ] Eliminar callbacks JS→Rust durante game loop
- [ ] Reducir llamadas JS↔Rust a 1 por frame (polling)
- [ ] 100% test coverage en events.rs
- [ ] Documentación API con examples

## 📋 Dependencias

- Ninguna épica previa requerida
- Depende de: `archflow-logic`, `archflow-web`

## 📋 Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Breaking change en bridge | Medio | Baja | Tests de integración |
| Performance no esperado | Alto | Baja | Benchmark antes/después |

## 📋 Timeline

```
Semana 1:
- D1: HU-LOGIC-EVENTS-001 (EventRingBuffer core)
- D3: HU-LOGIC-EVENTS-002 (poll_events integration)
- D5: HU-LOGIC-EVENTS-003 (Actuator integration)
```

## 📚 Documentación Relacionada

- `docs/integration/LOGIC_BRICKS_DEVELOPER_GUIDE.md` L334-396
- `crates/archflow-logic/src/input.rs` (InputSampler existente)
- `crates/archflow-logic/src/pulse.rs` (PulseBus existente)
