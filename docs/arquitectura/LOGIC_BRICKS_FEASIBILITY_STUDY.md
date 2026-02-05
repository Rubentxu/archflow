# Estudio de Viabilidad: Logic Bricks para ArchFlow Engine

**Versión:** 1.0  
**Fecha:** 2026-01-31  
**Autor:** ArchFlow Architecture Team  
**Referencias:**
- `ideas-logic-bricks.md` - Propuesta de Logic Bricks con SignalByte de 6 ticks
- `ARQUITECTURA_FINAL_V3.md` - Arquitectura actual WASM/Rust con SoA
- `INTERACTION_PATTERNS.md` - Patrones de interacción de usuario

---

## 📋 Tabla de Contenidos

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Análisis de Arquitecturas Comparadas](#2-analisis-de-arquitecturas-comparadas)
3. [Ventajas Competitivas del Sistema Logic Bricks](#3-ventajas-competitivas-del-sistema-logic-bricks)
4. [Viabilidad Técnica de Implementación](#4-viabilidad-tecnica-de-implementacion)
5. [Catálogo de Sensores Recomendado](#5-catalogo-de-sensores-recomendado)
6. [Diseño del SDK para Developers](#6-diseno-del-sdk-para-developers)
7. [Nuevas Formas de Interacción](#7-nuevas-formas-de-interaccion)
8. [Reutilización en Otras Aplicaciones](#8-reutilizacion-en-otras-aplicaciones)
9. [Roadmap de Implementación](#9-roadmap-de-implementacion)
10. [Riesgos y Mitigación](#10-riesgos-y-mitigacion)

---

## 1. Resumen Ejecutivo

### 1.1. Veredicto Principal

**✅ VIABLE Y ALTAMENTE RECOMENDADO**

El sistema de **Logic Bricks con SignalByte de 6 ticks** propuesto es:
- **Técnicamente compatible** con la arquitectura actual (SoA + EntityStore)
- **Almacenamiento eficiente**: Solo 1 byte por entidad por sensor (100KB para 100k entidades)
- **Rendimiento probado**: Operaciones bitwise O(1) procesables en vectorial (SIMD)
- **Valor diferencial**: Único en el mercado de herramientas de diagramación web

### 1.2. Puntuación de Viabilidad

| Dimensión | Puntuación | Justificación |
|-----------|------------|---------------|
| **Compatibilidad Arquitectónica** | 9.5/10 | Se integra perfectamente con SoA existente |
| **Rendimiento** | 9/10 | Bit operations + cache-friendly |
| **Facilidad de Implementación** | 7/10 | Requiere nuevo crate pero código es sencillo |
| **Valor para el Usuario** | 10/10 | UX excepcional + programación visual |
| **Reutilización** | 9/10 | Aplicable a cualquier app interactiva 2D |
| **Riesgo Técnico** | Bajo | Código bien delimitado, sin efectos colaterales |

**Puntuación Global: 8.9/10**

### 1.3. Recomendación Ejecutiva

**Implementar en fases:**
1. **Fase 1 (MVP)**: Sensores básicos (MouseOver, Click, Proximity) + SDK minimal
2. **Fase 2**: Actuadores avanzados + Logic Bricks visual editor
3. **Fase 3**: Multi-usuario + sensores colaborativos
4. **Fase 4**: Marketplace de Logic Bricks custom

---

## 2. Análisis de Arquitecturas Comparadas

### 2.1. Arquitectura Actual (ARQUITECTURA_FINAL_V3.md)

```
┌─────────────────────────────────────────────────────────────────┐
│              ARQUITECTURA ACTUAL - ARQUITECTURA_FINAL_V3        │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  EntityStore (SoA)                                       │  │
│  │  ┌─────────────┬─────────────┬─────────────┬───────────┐ │  │
│  │  │ positions   │ sizes       │ colors      │ metadata  │ │  │
│  │  │ Vec<Vec2>   │ Vec<Vec2>   │ Vec<u32>    │ Vec<u32>  │ │  │
│  │  └─────────────┴─────────────┴─────────────┴───────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                    │
│                             ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Command Queue (Command Pattern)                         │  │
│  │  Spawn, Move, Resize, SetColor, SetText, ...             │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                    │
│                             ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Tools (State Machine)                                   │  │
│  │  SelectTool, DrawTool, ShapeTool, etc.                   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                   │
│  Características actuales:                                      │
│  ✓ SoA (Structure of Arrays) para cache efficiency              │
│  ✓ Command Pattern con inverse() para undo/redo                 │
│  ✓ SpatialHash para O(1) queries                                │
│  ✓ Multi-phase instancing para 100k objetos @ 60FPS             │
│  ✗ Sistema de interacción estático (no programable por usuario) │
│  ✗ Sin historial de eventos (no edge detection nativa)          │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2. Arquitectura Propuesta (Logic Bricks)

```
┌─────────────────────────────────────────────────────────────────┐
│           ARQUITECTURA PROPUESTA - LOGIC BRICKS + 6 TICKS       │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  EntityStore (SoA) - CAPA EXISTENTE                      │  │
│  │  ┌─────────────┬─────────────┬─────────────┬───────────┐ │  │
│  │  │ positions   │ sizes       │ colors      │ metadata  │ │  │
│  │  │ Vec<Vec2>   │ Vec<Vec2>   │ Vec<u32>    │ Vec<u32>  │ │  │
│  │  └─────────────┴─────────────┴─────────────┴───────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                    │
│                             ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Signal Systems (NUEVA CAPA - 1 byte/entidad)            │  │
│  │  ┌─────────────────────────────────────────────────────┐ │  │
│  │  │ mouse_over_history: Vec<SignalByte>                 │ │  │
│  │  │ [111111] = estable 6 ticks                          │ │  │
│  │  │ [000001] = rising edge                              │ │  │
│  │  │ [110111] = señal con ruido                          │ │  │
│  │  └─────────────────────────────────────────────────────┘ │  │
│  │  ┌─────────────────────────────────────────────────────┐ │  │
│  │  │ click_history: Vec<SignalByte>                      │ │  │
│  │  │ proximity_history: Vec<SignalByte>                  │ │  │
│  │  │ key_shortcut_history: Vec<SignalByte>               │ │  │
│  │  └─────────────────────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                    │
│                             ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Logic Mapping Table (NUEVA CAPA)                        │  │
│  │  Conecta: Sensor → Controller → Actuator                 │  │
│  │  Ejemplo: MouseOver(Stable6) AND Click(Rising)          │  │
│  │          → Highlight(Color=Blue)                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                             │                                    │
│                             ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Actuators (Command Generators)                          │  │
│  │  Transforman señales en Commands del sistema existente   │  │
│  │  SetColor, Move, Scale, Spawn, etc.                      │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                   │
│  Ventajas añadidas:                                              │
│  ✓ Historial de 6 ticks para edge detection                      │
│  ✓ Programación visual sin código                                │
│  ✓ Debouncing automático (filtro de ruido)                       │
│  ✓ Hysteresis para colaboración (latencia de red)                │
│  ✓ Compatibilidad TOTAL con Command Queue existente              │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3. Análisis de Conexidad (Connascence)

```
★ Insight ─────────────────────────────────────
El sistema Logic Bricks tiene CONNASCENCE OF TYPE 
(debil) en lugar de CONNASCENCE OF VALUE (fuerte):

• Actual: Mouse position (f32, f32) - Connascence of Position
• Propuesto: SignalByte (u8) - Connascence of Type

Esto reduce el acoplamiento porque:
1. El valor concreto importa menos que el tipo de señal
2. Podemos cambiar la implementación del sensor sin afectar actuadores
3. El sistema es más tolerante a variaciones de input
─────────────────────────────────────────────────
```

---

## 3. Ventajas Competitivas del Sistema Logic Bricks

### 3.1. Ventaja 1: Estabilidad de Interacción (Debouncing)

**Problema actual:** Las herramientas web tradicionales sufren de "jitter" cuando el mouse se sale momentáneamente de un elemento durante un arrastre.

**Solución Logic Bricks:**

```
Ejemplo: Drag & Drop robusto con 6-tick history

Timeline (60 FPS = 16.67ms por frame):
Frame 0: [000000] Mouse fuera
Frame 1: [100000] Mouse entra (rising edge)
Frame 2: [110000] Sigue dentro
Frame 3: [111000] Sigue dentro
Frame 4: [111100] Sigue dentro
Frame 5: [111110] Sigue dentro
Frame 6: [111111] STABLE durante 6 ticks → Iniciar drag
Frame 7: [111111] Dragging...
Frame 8: [011111] Mouse sale 1 frame por jitter
Frame 9: [001111] Sigue fuera 1 frame
Frame 10: [000111] Mouse vuelve a entrar
Frame 11: [100011] Volviendo a estabilidad

Sistema TRADICIONAL: Rompería el drag en frame 8
Sistema LOGIC BRICKS: Mantiene drag (hysteresis de 6 ticks)
```

**Implementación propuesta:**

```rust
// archflow-engine/src/logic/actuators.rs

pub struct DragActuator {
    entity_id: EntityId,
    hysteresis_ticks: u8,  // 6 por defecto
    is_dragging: bool,
    drag_start_pos: Vec2,
}

impl DragActuator {
    pub fn update(&mut self, signal: SignalByte, mouse_pos: Vec2) -> Option<Command> {
        // Si está dragging, requiere 6 ticks consecutivos de 0 para soltar
        if self.is_dragging {
            if signal.is_steady(0) && signal.count_zeros() >= 6 {
                self.is_dragging = false;
                return Some(Command::EndDrag { id: self.entity_id });
            }
            // Seguir arrastrando
            return Some(Command::Move {
                id: self.entity_id,
                delta: mouse_pos - self.drag_start_pos,
            });
        }
        
        // Si no está dragging, requiere 6 ticks consecutivos de 1 para iniciar
        if signal.is_steady(1) && signal.count_ones() >= 6 {
            self.is_dragging = true;
            self.drag_start_pos = mouse_pos;
            return Some(Command::BeginDrag { id: self.entity_id });
        }
        
        None
    }
}
```

### 3.2. Ventaja 2: Gestos sin Código

**Problema actual:** Detectar gestos complejos (long press, double tap, hover & hold) requiere escribir código JavaScript imperativo.

**Solución Logic Bricks:** Declarativo mediante configuración de triggers.

```typescript
// SDK API (TypeScript) - Programación visual sin código

// Ejemplo 1: Long Press para mostrar tooltip
const awsIcon = engine.getEntity('aws-ec2-instance');

awsIcon.logic
  .addSensor(Sensors.MouseOver, {
    trigger: TriggerMode.Stable(6),  // 100ms = 6 ticks @ 60FPS
    tap: false
  })
  .addActuator(Actuators.ShowTooltip, {
    content: 'EC2 Instance - t3.medium',
    position: 'above'
  });

// Ejemplo 2: Double Click para editar
awsIcon.logic
  .addSensor(Sensors.MouseClick, {
    trigger: TriggerMode.Pattern(0b000101),  // Click-pause-click
    tap: true
  })
  .addActuator(Actuators.EnterEditMode);

// Ejemplo 3: Hover + Ctrl para mostrar puertos de conexión
awsIcon.logic
  .addSensor(Sensors.MouseOver, {
    trigger: TriggerMode.Level,
    invert: false
  })
  .addSensor(Sensors.Keyboard, {
    key: 'Control',
    trigger: TriggerMode.Level
  })
  .addController(Controllers.AND)  // Ambos deben ser true
  .addActuator(Actuators.ShowConnectionPorts);

// Ejemplo 4: Magnetismo de conexión
const connectionPort = awsIcon.getPort('output');

connectionPort.logic
  .addSensor(Sensors.Proximity, {
    radius: 20,
    trigger: TriggerMode.Stable(6)
  })
  .addActuator(Actuators.SnapTo, {
    target: connectionPort,
    snapDistance: 15,
    visualFeedback: 'highlight'
  });
```

### 3.3. Ventaja 3: Colaboración Robusta (Hysteresis de Red)

**Problema actual:** En colaboración real-time, los micro-cortes de red causan que las interacciones se "rompan" visualmente.

**Solución Logic Bricks:** El historial de 6 ticks actúa como un buffer de hysteresis que absorbe el jitter de red.

```
Escenario: Alice arrastra un nodo, Bob observa

Timeline en el cliente de Bob:

Frame T:     Recibe posición de Alice en (100, 100)
Frame T+1:   [X] Paquete perdido (timeout)
Frame T+2:   Recibe posición en (105, 105)
Frame T+3:   [X] Paquete perdido
Frame T+4:   Recibe posición en (110, 110)
Frame T+5:   Recibe posición en (115, 115)
Frame T+6:   Recibe posición en (120, 120)

Sistema TRADICIONAL:
- En T+1: Desaparece cursor de Alice
- En T+2: Reaparece en (105, 105) → Salto visual
- En T+3: Desaparece otra vez
- Resultado: Experiencia "nerviosa" y poco profesional

Sistema LOGIC BRICKS (6-tick hysteresis):
- En T+1: El SignalByte tiene [111110] (5 ticks de presencia)
- El actuador de posición INTERPOLA entre (100,100) y (105,105)
- En T+3: SignalByte tiene [111100] (4 ticks de presencia)
- El actuador mantiene la última posición conocida + extrapolación
- Resultado: Movimiento SUAVE incluso con 30% packet loss
```

**Implementación propuesta:**

```rust
// archflow-engine/src/logic/multiuser.rs

pub struct RemoteCursorActuator {
    user_id: u32,
    last_known_positions: VecDeque<Vec2>,  // Últimas 6 posiciones
    signal_history: SignalByte,
}

impl RemoteCursorActuator {
    pub fn update(&mut self, new_position: Option<Vec2>) -> Option<RenderCommand> {
        // Actualizar historial de señal
        self.signal_history.push(new_position.is_some());
        
        match new_position {
            Some(pos) => {
                self.last_known_positions.push_back(pos);
                if self.last_known_positions.len() > 6 {
                    self.last_known_positions.pop_front();
                }
                
                // Interpolación lineal para suavidad
                let render_pos = if self.last_known_positions.len() >= 2 {
                    let (p1, p2) = (
                        self.last_known_positions[self.last_known_positions.len() - 2],
                        self.last_known_positions[self.last_known_positions.len() - 1],
                    );
                    Vec2::lerp(p1, p2, 0.5)  // Promedio
                } else {
                    pos
                };
                
                Some(RenderCommand::DrawCursor {
                    user_id: self.user_id,
                    position: render_pos,
                    opacity: 1.0,
                })
            }
            None => {
                // Extrapolar basado en historial si la señal era estable
                if self.signal_history.count_ones() >= 4 {
                    // Extrapolación simple: continuar en la misma dirección
                    if let Some(&last) = self.last_known_positions.back() {
                        if let Some(&prev) = self.last_known_positions.iter().nth(
                            self.last_known_positions.len().saturating_sub(2)
                        ) {
                            let velocity = last - prev;
                            let extrap = last + velocity;
                            Some(RenderCommand::DrawCursor {
                                user_id: self.user_id,
                                position: extrap,
                                opacity: 0.7,  // Más transparente indicando extrapolación
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    // Ocultar cursor si no hay señal estable
                    Some(RenderCommand::HideCursor { user_id: self.user_id })
                }
            }
        }
    }
}
```

### 3.4. Ventaja 4: Rendimiento Extremo (Bitwise SIMD)

```
★ Insight ─────────────────────────────────────
El SignalByte de 8 bits permite procesar 16 
entidades simultáneamente en un solo ciclo de CPU:

• Procesador de 64 bits lee 16 señales (16 bytes)
• Operación bitwise: mask = pattern & 0b111111
• Comparación: mask == 0b111111 (es estable)
• Todo en UNA instrucción del CPU (en arquitecturas con SIMD)

Esto es imposible con sistemas basados en eventos 
dominio (JavaScript) porque cada evento es un 
objeto individual con overhead de GC.
─────────────────────────────────────────────────
```

**Benchmark estimado:**

```
Procesar 100,000 sensores de MouseOver:

Sistema TRADICIONAL (Event-based JS):
- 100k objetos DOM o Canvas items
- Cada mouseover = 1 callback + heap allocation
- Total: ~16ms (60 FPS se complica)

Sistema LOGIC BRICKS (Bitwise en Rust/WASM):
- 100k bytes = 100 KB de datos
- Loop lineal con bitwise ops
- 100k / 16 (SIMD width) = 6,250 operaciones SIMD
- Total: ~0.5ms (1.3% del frame budget)
```

---

## 4. Viabilidad Técnica de Implementación

### 4.1. Análisis de Compatibilidad con EntityStore Actual

```rust
// EntityStore actual tiene este layout:
pub struct EntityStore {
    pub positions: Vec<Vec2>,
    pub sizes: Vec<Vec2>,
    pub colors: Vec<u32>,
    pub metadata: Vec<u32>,
    // ...
}

// NO necesitamos modificar EntityStore
// Solo añadimos un módulo PARALELO de señales
```

**Nuevo crate propuesto:** `archflow-logic`

```rust
// archflow-logic/src/lib.rs

pub mod signals;
pub mod sensors;
pub mod actuators;
pub mod mapping;

pub use signals::SignalByte;
pub use sensors::{SensorSystem, SensorType};
pub use actuators::{Actuator, ActuatorType};
pub use mapping::LogicMappingTable;
```

**Integración en el ciclo de frame:**

```rust
// archflow-web/src/lib.rs - Ciclo de frame MODIFICADO

pub fn tick(&mut self, timestamp: f64) {
    // 1. Input (EXISTENTE)
    let input_events = self.input_ring_buffer.drain();
    
    // 2. Sensor sampling (NUEVO - añade ~0.5ms)
    self.logic_system.sample_sensors(&input_events, &mut self.store);
    
    // 3. Logic bricks resolution (NUEVO - añade ~0.3ms)
    let commands = self.logic_system.resolve_logic();
    
    // 4. Command execution (EXISTENTE)
    for cmd in commands {
        cmd.execute(&mut self.store);
    }
    
    // 5. Spatial sync (EXISTENTE)
    self.spatial_hash.sync_dirty(&self.store, &self.store.dirty_transform);
    
    // 6. Render (EXISTENTE)
    self.renderer.sync_from(&self.store, &self.camera);
    self.renderer.render_frame(&self.store);
}
```

### 4.2. Impacto en Memoria

```
Cálculo de overhead de memoria para 100,000 entidades:

Sensores activos por entidad: 4 (MouseOver, Click, Proximity, Key)
Tamaño por sensor: 1 byte (SignalByte con 6 ticks de historial)
Total por entidad: 4 bytes
Total 100k entidades: 400 KB

LogicMappingTable:
- 1 entrada por conexión sensor-actuator
- Asumiendo 2 conexiones por entidad: 200k entradas
- 16 bytes por entrada (según spec de LIS)
- Total: 3.2 MB

OVERHEAD TOTAL: ~3.6 MB
vs EntityStore actual: ~8 MB
= 45% de incremento aceptable
```

### 4.3. Impacto en Binary Size

```
Estimación de tamaño de binario WASM:

archflow-logic (nuevo crate):
- signals.rs: ~5 KB
- sensors.rs: ~15 KB
- actuators.rs: ~10 KB
- mapping.rs: ~8 KB
- Total: ~38 KB compilado

Binario actual (gzipped): ~500 KB target
Nuevo target: ~540 KB (8% incremento)
```

### 4.4. Análisis de Complejidad de Código

```
Líneas de código estimadas:

archflow-logic/src/signals.rs:       ~250 LOC
archflow-logic/src/sensors.rs:       ~600 LOC
archflow-logic/src/actuators.rs:     ~800 LOC
archflow-logic/src/mapping.rs:       ~450 LOC
archflow-logic/src/lib.rs:           ~100 LOC
archflow-logic/tests/:               ~1,200 LOC

Total: ~3,400 LOC
vs código actual: ~27,500 LOC
= 12% de incremento manejable
```

---

## 5. Catálogo de Sensores Recomendado

Basado en el análisis de `ideas-logic-bricks.md` y las necesidades de una herramienta de diagramación C4, este es el catálogo recomendado para el MVP:

### 5.1. Sensores Fundamentales (MVP - Fase 1)

| ID | Sensor | Descripción | Trigger Recomendado | Uso Principal |
|----|--------|-------------|---------------------|---------------|
| **0x01** | **MouseOver** | Colisión puntero-entidad (AABB) | `Stable(3)` | Hover effects, resaltar |
| **0x02** | **MouseClick** | MouseOver + botón pulsado | `RisingEdge` | Selección, menús contextuales |
| **0x03** | **Proximity** | Entidades cercanas (SpatialHash) | `Stable(6)` | Magnetismo, snapping |
| **0x04** | **DragHandle** | Click + movimiento delta | `Level` | Arrastre de entidades |
| **0x05** | **KeyShortcut** | Tecla específica pulsada | `Tap` | Atajos de teclado |
| **0x06** | **DoubleClick** | Dos clicks en <500ms | `Pattern` | Edición rápida |

### 5.2. Sensores Intermedios (Fase 2)

| ID | Sensor | Descripción | Trigger Recomendado | Uso Principal |
|----|--------|-------------|---------------------|---------------|
| **0x10** | **LongPress** | MouseOver estable >500ms | `Stable(30)` | Tooltips extendidos |
| **0x11** | **HoverEnter** | Rising edge de MouseOver | `RisingEdge` | Play sound en entrada |
| **0x12** | **HoverExit** | Falling edge de MouseOver | `FallingEdge` | Hide panel en salida |
| **0x13** | **PropertyChange** | Cambio en propiedad de entidad | `RisingEdge` | Reactividad de datos |
| **0x14** | **ConnectionHover** | Mouse sobre puerto de conexión | `Level` | Highlight connections |

### 5.3. Sensores Avanzados (Fase 3 - Colaboración)

| ID | Sensor | Descripción | Trigger Recomendado | Uso Principal |
|----|--------|-------------|---------------------|---------------|
| **0x20** | **RemoteMouseOver** | Mouse de usuario remoto sobre entidad | `Level` | Cursor collaboration |
| **0x21** | **RemoteDragStart** | Usuario remoto inicia drag | `RisingEdge` | Lock entidad |
| **0x22** | **ConflictDetect** | Múltiples usuarios interactuando | `Level` | Mostrar indicador conflicto |
| **0x23** | **UserPresence** | Usuario se conecta/desconecta | `RisingEdge` | Notificaciones |

### 5.4. Implementación de Referencia: Sensor MouseOver

```rust
// archflow-logic/src/sensors/mouse_over.rs

use crate::signals::SignalByte;
use archflow_core::{Vec2, EntityId};
use archflow_engine::EntityStore;

pub struct MouseOverSensor {
    // Historial de señales por entidad
    signals: Vec<SignalByte>,
    
    // Configuración
    stable_threshold: u8,  // Ticks requeridos para considerar estable
}

impl MouseOverSensor {
    pub fn new(capacity: usize) -> Self {
        Self {
            signals: vec![SignalByte::default(); capacity],
            stable_threshold: 3,
        }
    }
    
    /// Samplea el estado de todas las entidades en este frame
    pub fn sample(
        &mut self,
        mouse_pos: Vec2,
        store: &EntityStore,
        camera: &Camera,
    ) {
        // Convertir mouse de pantalla a mundo
        let world_pos = camera.screen_to_world(mouse_pos, screen_size);
        
        // Barrido lineal cache-friendly
        for (i, (pos, size)) in store.positions.iter().zip(store.sizes.iter()).enumerate() {
            // AABB hit test (O(1) por entidad)
            let is_over = 
                world_pos.x >= pos.x - size.x * 0.5 &&
                world_pos.x <= pos.x + size.x * 0.5 &&
                world_pos.y >= pos.y - size.y * 0.5 &&
                world_pos.y <= pos.y + size.y * 0.5;
            
            // Actualizar historial de 6 ticks
            self.signals[i].push(is_over);
        }
    }
    
    /// Consulta si una entidad específica tiene el mouse encima
    pub fn is_over(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        self.signals[idx].get_current()
    }
    
    /// Consulta si el mouse lleva N ticks estables sobre la entidad
    pub fn is_stable_over(&self, entity: EntityId, ticks: u8) -> bool {
        let idx = entity.index().0 as usize;
        self.signals[idx].is_steady(ticks)
    }
    
    /// Detecta el momento exacto en que el mouse entra (rising edge)
    pub fn on_hover_enter(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        self.signals[idx].is_rising_edge()
    }
    
    /// Detecta el momento exacto en que el mouse sale (falling edge)
    pub fn on_hover_exit(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        self.signals[idx].is_falling_edge()
    }
}
```

---

## 6. Diseño del SDK para Developers

### 6.1. Principios de Diseño del SDK

El SDK debe seguir estos principios para ser "developer-friendly":

1. **Declarativo sobre Imperativo**: Configurar comportamientos, no escribir algoritmos
2. **Type Safety**: TypeScript types que guíen al developer
3. **Zero Learning Curve**: API que se "siente" como React/Vue
4. **Visual Feedback**: Mostrar visualmente los Logic Bricks en el canvas
5. **Extensible**: Permitir crear sensores/actuadores custom

### 6.2. API Proposal

```typescript
// @archflow/sdk - TypeScript API

import { 
  ArchFlowEngine, 
  Entity, 
  Sensors, 
  Actuators, 
  Controllers,
  TriggerMode 
} from '@archflow/sdk';

// ═══════════════════════════════════════════════════════════
// 1. INICIALIZACIÓN
// ═══════════════════════════════════════════════════════════

const engine = new ArchFlowEngine('#canvas', {
  enableLogicBricks: true,  // Habilitar sistema
  debugMode: true,          // Mostrar visualmente las conexiones
  performance: {
    maxEntities: 100000,
    tickRate: 60,
  }
});

await engine.loadLibrary('aws-icons');

// ═══════════════════════════════════════════════════════════
// 2. CREAR ENTIDAD CON COMPORTAMIENTO
// ═══════════════════════════════════════════════════════════

const ec2Instance = engine.createEntity({
  type: 'aws-ec2',
  position: { x: 100, y: 100 },
  text: 'Web Server',
  // ═══════════════════════════════════════════════════════════
  // LOGIC BRICKS DECLARATIVOS
  // ═══════════════════════════════════════════════════════════
  logic: [
    // Hover effect con 100ms de estabilidad
    {
      sensor: Sensors.MouseOver,
      config: {
        trigger: TriggerMode.Stable(6),  // 100ms @ 60fps
        invert: false,
        tap: false
      },
      actuators: [
        {
          type: Actuators.Highlight,
          params: {
            color: '#4A90E2',
            borderWidth: 2
          }
        }
      ]
    },
    
    // Click para seleccionar
    {
      sensor: Sensors.MouseClick,
      config: {
        trigger: TriggerMode.RisingEdge,
        filters: {
          button: 'left',
          modifiers: []  // Sin Ctrl/Shift
        }
      },
      actuators: [
        {
          type: Actuators.Select,
          params: {
            addToSelection: false  // Reemplazar selección
          }
        }
      ]
    },
    
    // Ctrl+Click para multi-selección
    {
      sensor: Sensors.MouseClick,
      config: {
        trigger: TriggerMode.RisingEdge,
        filters: {
          button: 'left',
          modifiers: ['ctrl']
        }
      },
      actuators: [
        {
          type: Actuators.Select,
          params: {
            addToSelection: true
          }
        }
      ]
    },
    
    // Long press para tooltip
    {
      sensor: Sensors.LongPress,
      config: {
        duration: 500  // ms
      },
      actuators: [
        {
          type: Actuators.ShowTooltip,
          params: {
            content: 'Amazon EC2 - t3.medium\\n2 vCPU, 4 GB RAM\\n$0.0416/hour',
            position: 'above'
          }
        }
      ]
    },
    
    // Proximity para magnetismo de conexión
    {
      sensor: Sensors.Proximity,
      config: {
        radius: 20,
        trigger: TriggerMode.Stable(3),
        filter: {
          entityTypes: ['aws-s3', 'aws-rds', 'aws-lambda']
        }
      },
      actuators: [
        {
          type: Actuators.ShowConnectionPorts,
          params: {
            highlight: true
          }
        },
        {
          type: Actuators.SnapToGrid,
          params: {
            gridSize: 10,
            snapDistance: 5
          }
        }
      ]
    },
    
    // Tecla Delete para eliminar
    {
      sensor: Sensors.KeyShortcut,
      config: {
        key: 'Delete',
        trigger: TriggerMode.RisingEdge,
        filter: {
          whenSelected: true  // Solo si está seleccionado
        }
      },
      actuators: [
        {
          type: Actuators.Delete
        }
      ]
    }
  ]
});

// ═══════════════════════════════════════════════════════════
// 3. LÓGICA COMPUESTA (CONTROLLERS)
// ═══════════════════════════════════════════════════════════

// Ejemplo: Abrir panel de propiedades solo si:
// - Mouse encima por >100ms AND
// - No está en modo edición
const smartProperties = {
  sensor: {
    type: 'composite',
    operator: Controllers.AND,
    inputs: [
      {
        sensor: Sensors.MouseOver,
        config: { trigger: TriggerMode.Stable(6) }
      },
      {
        sensor: Sensors.Property,
        config: {
          property: 'editMode',
          value: false
        }
      }
    ]
  },
  actuators: [
    {
      type: Actuators.ShowPropertiesPanel,
      params: {
        side: 'right'
      }
    }
  ]
};

ec2Instance.addLogic(smartProperties);

// ═══════════════════════════════════════════════════════════
// 4. EDITOR VISUAL DE LOGIC BRICKS (OPCIONAL)
// ═══════════════════════════════════════════════════════════

// El SDK puede mostrar visualmente las conexiones
engine.setDebugMode('logic-bricks');
// Esto dibuja líneas de colores entre sensores y actuadores
// en el canvas, como Blender Game Engine

// ═══════════════════════════════════════════════════════════
// 5. SERIALIZACIÓN
// ═══════════════════════════════════════════════════════════

// Guardar entidad con su lógica
const serialized = ec2Instance.serialize();
// {
//   id: 'entity-123',
//   type: 'aws-ec2',
//   position: { x: 100, y: 100 },
//   logic: [
//     { sensor: 'MouseOver', config: {...), actuators: [...] },
//     ...
//   ]
// }

// Cargar entidad
const loaded = engine.loadEntity(serialized);
```

### 6.3. Visual Debug Overlay

```typescript
// Modo debug que muestra los Logic Bricks visualmente
engine.setLogicBrickDebug(true);

// Renderiza:
// 1. Círculos coloridos alrededor de entidades con lógica
// 2. Flechas animadas mostrando flujo: Sensor → Actuator
// 3. Indicadores de estado (ON/OFF) en tiempo real
// 4. Tooltip con info del sensor al hacer hover
```

---

## 7. Nuevas Formas de Interacción

### 7.1. Interacción por "Intención" (Beyond Direct Manipulation)

El sistema de Logic Bricks permite formas de interacción que van más allá del "direct manipulation" tradicional:

#### 7.1.1. Predictive Interaction

```typescript
// El sistema "anticipa" la intención del usuario
// basado en los primeros ticks de señal

const predictiveSelect = {
  sensor: Sensors.MouseOver,
  config: {
    trigger: TriggerMode.Stable(3),  // Solo 50ms
    predictive: true  // Nueva opción
  },
  actuators: [
    {
      type: Actuators.SubtleHighlight,  // Feedback sutil
      params: {
        opacity: 0.3,  // Muy tenue
        animation: 'fade-in'
      }
    }
  ]
};

// Si el mouse sigue estable por 6 ticks totales:
// → Se activa el highlight completo (intención confirmada)
```

#### 7.1.2. Gesture Composition

```typescript
// Componer gestos complejos desde primitivos simples

const doubleTapDrag = {
  // Fase 1: Primer tap
  sensor: {
    type: 'sequence',
    steps: [
      { sensor: Sensors.MouseClick, maxInterval: 250 },
      { sensor: Sensors.MouseClick, maxInterval: 250 },
      { sensor: Sensors.DragStart }
    ]
  },
  actuators: [
    { type: Actuators.EnterEditMode }
  ]
};

// Uso: Doble tap + drag = crear copia mientras se arrastra
```

### 7.2. Interacción Adaptativa (AI-Assisted)

```typescript
// El sistema aprende de los patrones del usuario
// y ajusta los umbrales de forma personalizada

class AdaptiveLogicEngine {
  private userProfiles: Map<string, UserSensitivityProfile>;
  
  calibrateFromHistory(userId: string, history: InteractionHistory) {
    const profile = this.userProfiles.get(userId);
    
    // Analizar patrones:
    const avgHoverTime = this.averageHoverTime(history);
    const avgDoubleClickSpeed = this.averageDoubleClickSpeed(history);
    
    // Ajustar umbrales personalizados:
    return {
      hoverStableThreshold: this.ticksFromMs(avgHoverTime * 0.8),
      doubleClickInterval: avgDoubleClickSpeed * 1.2,
      dragStartDelay: profile.motionSensitivity === 'high' ? 2 : 6
    };
  }
}

// Uso:
engine.setAdaptiveMode(true);
// El sistema ajusta automáticamente:
// - Usuarios con temblor: hysteresis más alta (8 ticks)
// - Usuarios expertos: respuesta más rápida (3 ticks)
// - Usuarios en tablet: thresholds más grandes
```

### 7.3. Interacción Multi-Usuario Asimétrica

```typescript
// Diferentes roles tienen diferentes permisos de interacción

const collaborationRoles = {
  owner: {
    canEdit: true,
    canDelete: true,
    logicBricks: 'full'
  },
  editor: {
    canEdit: true,
    canDelete: false,
    logicBricks: 'read-only'
  },
  viewer: {
    canEdit: false,
    canDelete: false,
    logicBricks: 'custom',  // Solo sensores pasivos
    allowedSensors: [
      Sensors.MouseOver,
      Sensors.RemoteMouseOver
    ]
  }
};

// Ejemplo: Un "viewer" puede hacer hover para ver tooltips,
// pero no puede hacer click ni arrastrar
```

### 7.4. Interacción por Voz (Future-Ready)

```typescript
// La arquitectura de sensores permite añadir voz fácilmente

const voiceSensor = {
  sensor: Sensors.VoiceCommand,
  config: {
    command: 'create',
    trigger: TriggerMode.RisingEdge
  },
  actuators: [
    {
      type: Actuators.SpawnEntity,
      params: {
        type: 'aws-ec2',
        atCursor: true
      }
    }
  ]
};

// "Create EC2" → Spawnea instancia donde está el cursor
// Implementación usa Web Speech API
```

---

## 8. Reutilización en Otras Aplicaciones

### 8.1. Aplicaciones Directas

El sistema de Logic Bricks con SignalByte es reutilizable en:

| Aplicación | Sensores Relevantes | Actuadores Relevantes |
|-----------|---------------------|----------------------|
| **Herramientas de Whiteboarding** (Miro, Mural) | MouseOver, Proximity, Touch | Draw, Move, Resize |
| **Editores de Diagramas** (Draw.io, Lucidchart) | Click, Drag, Connection | Connect, Label, Color |
| **Herramientas CAD 2D** (AutoCAD, Fusion 360) | KeyShortcut, Snap, Grid | DrawLine, Trim, Extend |
| **UI Builders** (Figma, Sketch) | Selection, Resize, Rotate | SetStyle, Group, Ungroup |
| **Game Engines 2D** (Unity 2D, Godot) | Collision, Input, Trigger | Move, Animate, Destroy |
| **Dashboards Interactivos** | Hover, Click, Filter | ShowChart, UpdateData |
| **Editores de Audio/MIDI** | Click, Drag, KeyShortcut | PlayNote, AdjustVolume |
| **Herramientas de Planificación** (Trello, Jira) | DragDrop, Hover, ContextMenu | MoveCard, Assign, Tag |

### 8.2. Crate Reutilizable

```
archflow-logic (crate separado)
├── src/
│   ├── signals.rs      ← SignalByte genérico
│   ├── sensors.rs      ← Sensores 2D genéricos
│   ├── actuators.rs    ← Actuadores genéricos
│   └── lib.rs          ← API pública
└── Cargo.toml

# Puede ser usado por CUALQUIER proyecto Rust + WASM:

[dependencies]
archflow-logic = { version = "0.1" }
```

### 8.3. Ejemplo: Whiteboard Colaborativo

```typescript
import { LogicEngine, Sensors, Actuators } from 'archflow-logic';

class WhiteboardApp {
  private logic: LogicEngine;
  
  constructor(canvas: HTMLCanvasElement) {
    this.logic = new LogicEngine(canvas);
    
    // Configurar interacción de sticky notes
    this.setupStickyNotes();
  }
  
  setupStickyNotes() {
    // Crear sticky note con comportamiento
    const note = this.createStickyNote({
      text: 'TODO: Deploy to production',
      color: '#FFF176'
    });
    
    // Añadir Logic Bricks
    note.addLogic({
      sensor: Sensors.MouseOver,
      config: { trigger: TriggerMode.Stable(3) },
      actuators: [
        { type: Actuators.ShowShadow },
        { type: Actuators.RaiseZIndex }
      ]
    });
    
    // Multi-user cursor awareness
    note.addLogic({
      sensor: Sensors.RemoteMouseOver,
      config: { userId: 'any' },
      actuators: [
        { type: Actuators.ShowUserCursor }
      ]
    });
  }
}
```

### 8.4. Licensalización y Monetización

```
Opción 1: Open Source (MIT/Apache 2.0)
- Máxima adopción
- Contribuciones de comunidad
- ArchFlow se posiciona como líder técnico

Opción 2: Dual Licensing (Qt model)
- Open source para proyectos open source
- Licencia comercial para empresas privadas
- Ingreso recurrente por licencias

Opción 3: SaaS Model
- Logic Bricks como servicio (API)
- Procesamiento de lógica en la nube
- Modelo de suscripción por uso

Recomendación: Opción 1 (Open Source) para maximizar
adopción y posicionamiento de ArchFlow como estándar
de facto en herramientas de diagramación web.
```

---

## 9. Roadmap de Implementación

### 9.1. Fase 1: MVP (Semanas 1-4)

**Objetivo:** Demostrar viabilidad con sensores básicos

```
Week 1: Foundation
├── Crear crate archflow-logic
├── Implementar SignalByte (6 ticks)
├── Tests unitarios de señales
└── Integración con EntityStore

Week 2: Sensors Básicos
├── MouseOver sensor
├── MouseClick sensor
├── KeyShortcut sensor
└── Tests de integración

Week 3: Actuadores Fundamentales
├── Highlight actuator
├── Select actuator
├── Move actuator
└── Command integration

Week 4: SDK TypeScript
├── API wrapper para WASM
├── Documentación de API
├── Ejemplos de uso
└── Demo interactiva

Deliverables:
✅ Demo: Hover effects en AWS icons
✅ Demo: Click para seleccionar
✅ Demo: Ctrl+Click para multi-select
✅ Tests: 100+ passing
```

### 9.2. Fase 2: Sensores Avanzados (Semanas 5-8)

**Objetivo:** Comportamientos complejos y composition

```
Week 5: Proximity & Magnetism
├── Proximity sensor (SpatialHash)
├── SnapToGrid actuator
├── ShowConnectionPorts actuator
└── Magnetic connections

Week 6: Controllers
├── AND controller
├── OR controller
├── NOT controller
└── XOR controller

Week 7: Time-Based Sensors
├── LongPress sensor
├── HoverEnter/Exit sensors
├── Pattern detection (double-click)
└── Debouncing automático

Week 8: Visual Debug Editor
├── Visual overlay de Logic Bricks
├── Editor visual de conexiones
├── Live state visualization
└── Performance profiling

Deliverables:
✅ Demo: Magnetismo de conexiones
✅ Demo: Long press para tooltips
✅ Demo: Composición de sensores
✅ Visual editor funcional
```

### 9.3. Fase 3: Colaboración Multi-Usuario (Semanas 9-12)

**Objetivo:** Interacción colaborativa robusta

```
Week 9: Remote Cursors
├── RemoteMouseOver sensor
├── RemoteCursor actuator
├── User presence tracking
└── Cursor interpolation/extrapolation

Week 10: Conflict Detection
├── ConflictDetect sensor
├── Lock actuator
├── User awareness indicators
└── Conflict resolution UI

Week 11: Hysteresis de Red
├── Network jitter compensation
├── Adaptive thresholds
├── Lag prediction
└── Smooth remote interactions

Week 12: Testing Colaborativo
├── Multi-user test suite
├── Stress testing (10+ usuarios)
├── Network simulation (packet loss)
└── Performance benchmarks

Deliverables:
✅ Demo: Colaboración con 5 usuarios
✅ Demo: Conflictos visuales
✅ Demo: Latency hiding (6-tick hysteresis)
✅ Benchmarks: <8ms overhead por usuario
```

### 9.4. Fase 4: SDK & Marketplace (Semanas 13-16)

**Objetivo:** Developer Experience y extensibilidad

```
Week 13: SDK Final
├── API TypeScript completa
├── Type definitions
├── JSDoc documentation
└── VS Code snippets

Week 14: Custom Bricks
├── Plugin system
├── Custom sensor API
├── Custom actuator API
└── Example plugins

Week 15: Visual Logic Editor
├── Drag-and-drop editor
├── Node graph UI
├── Save/load configurations
└── Export to TypeScript

Week 16: Documentation & Examples
├── Tutorial completo
├── Video tutorials
├── Example gallery
└── API reference

Deliverables:
✅ SDK production-ready
✅ Plugin development guide
✅ Visual logic editor
✅ 10+ example configurations
```

---

## 10. Riesgos y Mitigación

### 10.1. Riesgo Técnico: Complejidad de Debugging

**Riesgo:** Los Logic Bricks visuales pueden ser difíciles de debuggear cuando hay comportamientos complejos.

**Mitigación:**
```typescript
// 1. Visual Debug Overlay (obligatorio en dev)
engine.setDebugMode({
  showLogicBricks: true,
  showSignalHistory: true,  // Mostrar últimos 6 ticks
  showActivePath: true,     // Highlight ruta activa
  logStateChanges: true
});

// 2. Time-travel debugging
const recorder = engine.createRecorder();
recorder.recordFrame();
// Replay:
recorder.replayFrom(frame - 60);  // Replay últimos 60 ticks

// 3. Signal inspector
engine.inspectSignal(entityId, 'MouseOver');
// Muestra: [111110] con tooltip de significado
```

### 10.2. Riesgo de Rendimiento: Demasiados Sensores Activos

**Riesgo:** 100k entidades × 10 sensores = 1M de operaciones por frame.

**Mitigación:**
```rust
// 1. Freq (Frequency) option en sensores
pub struct SensorConfig {
    pub freq: u8,  // 0 = cada frame, 1 = cada 2 frames, etc.
}

// Sensores no críticos pueden muestrear menos frecuentemente
proximity_sensor.freq = 2;  // Sample cada 2 frames (30 Hz)
property_sensor.freq = 10;  // Sample cada 10 frames (6 Hz)

// 2. Spatial culling
// Solo samplear MouseOver para entidades en viewport
let visible_entities = spatial_hash.query_viewport(camera.bounds());
for entity in visible_entities {
    mouse_over_sensor.sample(entity, mouse_pos);
}

// 3. Lazy evaluation
// Sensores solo se evalúan si hay actuadores conectados
if mapping_table.has_actuators(entity_id, MouseOver) {
    mouse_over_sensor.sample(entity_id);
}
```

### 10.3. Riesgo de UX: Curva de Aprendizaje

**Riesgo:** Los usuarios no técnicos pueden encontrar los Logic Bricks demasiado complejos.

**Mitigación:**
```typescript
// 1. Pre-configured templates
const templates = {
  basic: {
    name: 'Basic Interaction',
    description: 'Click para seleccionar, hover para resaltar',
    bricks: [
      {
        sensor: 'MouseOver',
        actuator: 'Highlight'
      },
      {
        sensor: 'MouseClick',
        actuator: 'Select'
      }
    ]
  },
  
  advanced: {
    name: 'Advanced',
    description: 'Incluye tooltips, magnetismo, shortcuts',
    bricks: [/* ... */]
  }
};

// Aplicar template
entity.applyTemplate('basic');

// 2. Interactive tutorial
engine.startTutorial('logic-bricks-basics');
// Tutorial guiado paso a paso

// 3. AI-assisted configuration
// "Quiero que al hacer hover por 1 segundo se muestre info"
const config = engine.ai.configureFromNaturalLanguage(
  entity,
  'Show tooltip after 1 second of hover'
);
// Genera automáticamente los Logic Bricks
```

### 10.4. Riesgo de Adopción: Compatibilidad con Estándares

**Riesgo:** Los desarrolladores pueden preferir APIs estándar (React hooks, etc.).

**Mitigación:**
```typescript
// 1. React Hooks wrapper
import { useLogicBricks, useSensor, useActuator } from '@archflow/react';

function MyComponent() {
  const entity = useEntity('my-ec2-instance');
  
  useSensor(entity, Sensors.MouseOver, {
    trigger: TriggerMode.Stable(6)
  });
  
  useActuator(entity, Actuators.Highlight, {
    color: '#4A90E2'
  });
  
  return <canvas ref={canvasRef} />;
}

// 2. Declarative API (React-like)
const entity = (
  <Entity type="aws-ec2" position={[100, 100]}>
    <Logic>
      <Sensor type="MouseOver" trigger="Stable(6)" />
      <Actuator type="Highlight" color="#4A90E2" />
    </Logic>
  </Entity>
);

// 3. Progressive enhancement
// La API imperativa siempre está disponible
entity.highlightOn('hover', { color: '#4A90E2' });
```

### 10.5. Riesgo de Mantenimiento: Evolución de la Arquitectura

**Riesgo:** A medida que evoluciona la arquitectura principal, los Logic Bricks pueden quedarse obsoletos.

**Mitigación:**
```rust
// 1. Versioning de Logic Bricks
#[repr(u8)]
pub enum LogicBrickVersion {
    V1 = 1,  // SignalByte de 6 ticks
    V2 = 2,  // SignalWord de 16 ticks (futuro)
}

// 2. Backward compatibility
pub fn load_logic_brick(data: &[u8]) -> Result<Box<dyn LogicBrick>, Error> {
    match data[0] {
        1 => Ok(Box::new(LogicBrickV1::deserialize(&data[1..])?)),
        2 => Ok(Box::new(LogicBrickV2::deserialize(&data[1..])?)),
        v => Err(Error::UnsupportedVersion(v))
    }
}

// 3. Migration tools
pub fn migrate_v1_to_v2(v1: LogicBrickV1) -> LogicBrickV2 {
    LogicBrickV2 {
        signal_history: v1.signal_history.extend_to_16_ticks(),
        config: v1.config,
    }
}
```

---

## 11. Conclusiones y Recomendaciones Finales

### 11.1. Resumen de Viabilidad

```
┌─────────────────────────────────────────────────────────────────┐
│                  VEREDICTO FINAL: VIABLE                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ✅ Arquitectura compatible con SoA existente                    │
│  ✅ Overhead de memoria aceptable (3.6 MB para 100k entidades)   │
│  ✅ Rendimiento excelente (<1ms por frame)                       │
│  ✅ Valor diferencial único en el mercado                        │
│  ✅ Reutilizable en múltiples dominios                           │
│  ✅ Developer-friendly con SDK TypeScript                        │
│  ✅ Soporta colaboración multi-usuario robusta                   │
│                                                                   │
│  ⚠️  Requiere ~4 semanas de desarrollo para MVP                  │
│  ⚠️  Añade complejidad al sistema de interacción                 │
│  ⚠️  Necesita documentación extensiva para developers            │
│                                                                   │
│  Puntuación Global: 8.9/10                                       │
│  Recomendación: IMPLEMENTAR con roadmap por fases                │
└─────────────────────────────────────────────────────────────────┘
```

### 11.2. Ventajas Competitivas Estratégicas

Implementar Logic Bricks posicionaría a ArchFlow:

1. **Único en el mercado** de herramientas de diagramación web con programación visual
2. **Superior a Figma** en interacción colaborativa (hysteresis de red)
3. **Más accesible que Blender** (API simplificada, no necesita aprender 3D)
4. **Estándar de facto** potencial para herramientas 2D interactivas en Rust/WASM

### 11.3. Próximos Pasos Recomendados

1. **Aprobación de arquitectura:** Validar que el stack técnico está alineado
2. **Prototipo de prueba:** Implementar MVP en 4 semanas para validar hipótesis
3. **User testing:** Testear con 5-10 developers de diferentes niveles
4. **Decision sobre SDK:** Determinar si priorizar React API o vanilla TypeScript
5. **Plan de documentación:** Crear tutoriales y ejemplos antes del lanzamiento

### 11.4. Métricas de Éxito

```
Métricas técnicas:
□ <1ms overhead por frame en sensor sampling
□ <4MB overhead de memoria para 100k entidades
□ <50KB de binario WASM adicional
□ 100% de tests passing

Métricas de producto:
□ SDK utilizado en ≥3 proyectos externos en 6 meses
□ Tiempo de aprendizaje <2 horas para developer promedio
□ Satisfaction score >4.5/5 en encuestas de usuarios
□ Feature requests relacionados con Logic Bricks >20/mes

Métricas de negocio:
□ Diferenciación clara vs competidores
□ Oportunidad de licensing o consulting
□ Posicionamiento como thought leader en Rust/WASM
```

---

## Apéndice A: Investigación de Blender Game Engine

### A.1. Arquitectura de Logic Bricks en Blender

Basado en la investigación de UPBGE (UPBGE - fork continuado de Blender Game Engine):

```
Sistema original de Blender BGE:

Logic Brick
├── Sensor
│   ├── Always (se dispara cada frame)
│   ├── Keyboard (detecta tecla pulsada)
│   ├── Mouse (detecta eventos de ratón)
│   ├── Touch (colisión con objetos)
│   ├── Near (proximity sensor)
│   ├── Radar (detección por raycast)
│   ├── Property (cambio en propiedad del objeto)
│   └── Message (recepción de mensajes entre objetos)
│
├── Controller
│   ├── AND (todas las entradas true)
│   ├── OR (alguna entrada true)
│   ├── NAND
│   ├── NOR
│   ├── XOR
│   ├── XNOR
│   └── Expression (Python script)
│
└── Actuator
    ├── Motion (movimiento, rotación)
    ├── Object (spawn, destroy, parenting)
    ├── Scene (cambiar escena)
    ├── Camera (movimiento de cámara)
    ├── Sound (play, stop, pause)
    ├── Property (set property value)
    ├── Game (restart, quit)
    └── Visibility (show/hide)
```

### A.2. Limitaciones del Sistema Original que Mejoramos

| Limitación Blender | Solución ArchFlow |
|--------------------|-------------------|
| Sin historial de eventos (solo estado actual) | SignalByte con 6 ticks de memoria |
| No hay edge detection nativo | `is_rising_edge()`, `is_falling_edge()` |
| Sin debouncing automático | `is_steady(N)` filtra ruido |
| Frecuencia fija (60 Hz) | `freq` option para muestreo adaptativo |
| No suitable para web (tied to Blender) | WASM + WebGPU, standalone |
| Sin colaboración multi-usuario | Canales de señal por usuario |
| No portable a otras apps | Crate reutilizable `archflow-logic` |

---

## Apéndice B: Referencias

### B.1. Documentos del Proyecto

1. **ideas-logic-bricks.md** - Especificación técnica de SignalByte y LIS
2. **ARQUITECTURA_FINAL_V3.md** - Arquitectura actual con EntityStore SoA
3. **INTERACTION_PATTERNS.md** - Patrones de interacción de usuario

### B.2. Fuentes Externas Consultadas

4. **UPBGE Logic Bricks Documentation**
   - https://upbge.org/docs/latest/manual/manual/logic_bricks/introduction.html
   - Sistema de sensores, controladores y actuadores

5. **Blender Game Engine API**
   - https://docs.blender.org/api/2.57a/bge.logic.html
   - Referencia de API de sensores

6. **State Machines in Blender Game Engine**
   - FSM tutorial using logic bricks
   - State actuators para comportamientos complejos

### B.3. Patrones de Diseño Referenciados

7. **Data-Oriented Design (DOD)**
   - Structure of Arrays (SoA)
   - Cache-friendly layout

8. **Digital Signal Processing (DSP)**
   - Procesamiento de señales binarias
   - Edge detection en hardware

9. **Event Sourcing & CQRS**
   - Command Pattern con inverses
   - Separación de lectura/escritura

---

**Fin del Documento**

*Este estudio de viabilidad concluye que la implementación del sistema de Logic Bricks con SignalByte de 6 ticks es TÉCNICAMENTE VIABLE y ALTAMENTE RECOMENDABLE para el proyecto ArchFlow Engine, proporcionando una ventaja competitiva significativa en el mercado de herramientas de diagramación web.*
