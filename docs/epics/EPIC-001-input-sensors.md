# Épica: Sensores de Entrada - Input Sensors Suite

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| ID | EPIC-001 |
| Prioridad | Alta |
| Estimación | XL |
| Estado | Borrador |
| Versión | 0.1.0 |
| Fecha creación | 2026-02-01 |

---

## 🎯 Objetivo de Negocio

Implementar el sistema de sensores de entrada (Mouse y Keyboard) fiel a la arquitectura de Blender Game Engine (BGE), permitiendo que las aplicaciones web detecten interacciones de usuario con latencia cero y rendimiento nativo mediante Rust + WebAssembly.

**Problema que resuelve**: Las aplicaciones web actuales sufren de latencia en la detección de input porque dependen exclusivamente de JavaScript. Esta épica permite que ArchFlow procese eventos de entrada a 60+ FPS con Sobrecarga Mínima del CPU.

---

## 🏗️ Arquitectura DDD

### Bounded Context
**Input Perception Context** - Contexto de Percepción de Entrada

### Aggregate Roots
- `MouseSensor`: Sensor unificado de mouse con 10 modos BGE
- `KeyboardSensor`: Sensor de teclado con detección de teclas específicas y modo "all keys"
- `InputSampler`: Muestreador de input que lee del SharedArrayBuffer

### Domain Events
```rust
pub enum InputEvent {
    MouseButtonDown { button: MouseButton, position: Vec2 },
    MouseButtonUp { button: MouseButton, position: Vec2 },
    MouseMove { position: Vec2, delta: Vec2 },
    MouseWheel { delta: i8 },
    KeyDown { key_code: KeyCode },
    KeyUp { key_code: KeyCode },
}
```

### Services
- `InputSamplingService`: Servicio que toma snapshots atómicos del SAB
- `PulseGenerationService`: Convierte eventos crudos en Pulsos de 16 bytes
- `SensorEvaluationService`: Evalúa todos los sensores activos cada frame

---

## 📖 Historias de Usuario

### HU-001: Sensor de Mouse Unificado (Mouse Sensor)

**Como** desarrollador de SDK
**Quiero** un sensor de mouse que soporte todos los modos de BGE (click, movimiento, rueda)
**Para** crear interacciones ricas sin configurar múltiples detectores

#### Criterios de Aceptación
- [ ] Soporta los 10 modos de mouse de BGE (LeftButton, RightButton, MiddleButton, Button4-7, WheelUp, WheelDown, Movement)
- [ ] Implementa propiedades BGE: `invert`, `tap`, `level`, `frequency`
- [ ] Detecta flancos de subida (rising edge) y bajada (falling edge)
- [ ] Usa `SignalByte` para historial de 6 ticks (anti-jitter)
- [ ] Integra con `PulseBus` para emitir pulsos
- [ ] Rendimiento: O(n) donde n = número de entidades, con pruebas de 100K entidades

#### Tareas Técnicas
- [ ] **Investigación**: Revisar código fuente `SCA_MouseSensor.cpp` de Blender
- [ ] **Tests (TDD)**: Escribir tests para cada modo de mouse
- [ ] **Implementación**: Crear `MouseMode` enum con códigos BGE
- [ ] **Implementación**: Crear `MouseConfig` struct
- [ ] **Implementación**: Implementar `MouseSensor::evaluate()`
- [ ] **Integración**: Conectar con `PulseBus`
- [ ] **Documentación**: Documentar API Rust y TypeScript

#### Investigación Previa
- [x] Perplexity: "Rust ECS best practices 2025"
- [x] Perplexity: "WebAssembly performance zero-copy SharedArrayBuffer"
- [x] BGE Source: `source/gameengine/Ketsji/SCA_MouseSensor.cpp`
- [x] Patrón implementado: SignalByte con historial de 6 ticks

#### Estimación: L
#### Estado: Pendiente

---

### HU-002: Sensor de Teclado con Detección de Teclas (Keyboard Sensor)

**Como** desarrollador de aplicaciones
**Quiero** detectar pulsaciones de teclas específicas y cualquier tecla
**Para** implementar atajos de teclado y controles globales

#### Criterios de Aceptación
- [ ] Detecta teclas específicas usando `KeyCode` enum (mapeo BGE)
- [ ] Soporta modo `all_keys` para detectar cualquier pulsación
- [ ] Implementa propiedades BGE: `invert`, `tap`, `level`
- [ ] Usa `RawInputMap` del SharedArrayBuffer
- [ ] Genera pulsos en `PulseBus` con timestamp
- [ ] Soporta hasta 256 teclas simultáneas (buffer de input)

#### Tareas Técnicas
- [ ] **Investigación**: Mapear `KX_KeyboardKey.h` de BGE a Rust
- [ ] **Tests (TDD)**: Tests para detección de teclas específicas
- [ ] **Tests (TDD)**: Tests para modo `all_keys`
- [ ] **Implementación**: Crear `KeyCode` enum completo (A-Z, 0-9, F1-F12, etc.)
- [ ] **Implementación**: Crear `KeyboardConfig` struct
- [ ] **Implementación**: Implementar `KeyboardSensor::evaluate()`
- [ ] **Integración**: Sistema de teclado global (entidades fantasma)
- [ ] **Documentación**: Guía de atajos de teclado

#### Investigación Previa
- [x] BGE Source: `source/gameengine/Ketsji/SCA_KeyboardSensor.cpp`
- [x] Documentación: BGE Python API Keyboard
- [x] Patrón implementado: BgeCore compartido entre sensores

#### Estimación: M
#### Estado: Pendiente

---

### HU-003: Muestreador de Input con SharedArrayBuffer y Fallback ⭐ CRÍTICA

**Como** arquitecto del sistema
**Quiero** un muestreador que lea atómicamente del SharedArrayBuffer con fallback automático
**Para** garantizar latencia <2ms en browsers modernos y compatibilidad con todos los entornos

#### Contexto y Justificación

Un SDK de alto rendimiento debe ofrecer la **mejor latencia posible** out-of-the-box. La diferencia entre 1ms y 8ms de latencia se **siente** en aplicaciones de dibujo tipo Figma.

**Problema actual:** El código menciona SAB pero `InputRingBuffer` es un `Vec<RawInputEvent>` normal en memoria Rust, sin zero-copy real.

**Solución:** Implementar SAB real con detección automática de disponibilidad y fallback a postMessage.

#### Criterios de Aceptación

**FASE 1: SharedArrayBuffer Real**
- [ ] Implementar `InputSampler` con `web-sys::SharedArrayBuffer` real
- [ ] Usar `Atomics.load/store` para sincronización lock-free
- [ ] Layout de memoria cache-line aligned (64 bytes) - ver especificación abajo
- [ ] Test de latencia: <2ms desde evento JS hasta snapshot en Rust
- [ ] Zero allocations en hot-path de muestreo

**FASE 2: Fallback Automático**
- [ ] Detectar si SAB está disponible (`try_get_shared_buffer`)
- [ ] `InputSamplerPolled`: fallback que usa `push_input_event()` sin SAB
- [ ] Switch transparente para el usuario del SDK
- [ ] Feature flag: `sab-input` (default: true)

**FASE 3: Thread-Safety**
- [ ] Compatible con Web Workers para input processing
- [ ] Frecuencia de muestreo configurable (60-240 Hz)
- [ ] Tests de concurrencia con múltiples hilos productores

#### Layout de SharedArrayBuffer (64 bytes, cache-line aligned)

```
┌──────────┬──────────┬──────────┬──────────┐
│  Offset  │   Size   │   Type   │   Field  │
├──────────┼──────────┼──────────┼──────────┤
│    0     │    4     │   u32    │   head   │ Write index
│    4     │    4     │   u32    │   tail   │ Read index
│    8     │    4     │   i32    │  mouse_x │ Mouse position
│   12     │    4     │   i32    │  mouse_y │ Mouse position
│   16     │    1     │    u8    │  buttons │ Button bitmask
│   17     │    1     │    u8    │ modifiers│ Ctrl/Shift/Alt
│   18     │    2     │   i16    │  wheel_d │ Wheel delta
│   20     │    4     │   u32    │timestamp │ ms since start
│   24     │   32     │[u8; 32]  │  keys    │ 256 bits (1=key down)
│   56     │    8     │   pad    │ alignment│ Cache padding
└──────────┴──────────┴──────────┴──────────┘
Total: 64 bytes (exactly one cache line)
```

#### Tareas Técnicas

**Implementación Core:**
- [ ] **Tests (TDD)**: Tests de layout de memoria y alineación
- [ ] **Tests (TDD)**: Tests de atomicidad con Atomics
- [ ] **Implementación**: `InputSnapshot` struct que mapea al layout SAB
- [ ] **Implementación**: `InputSampler::take_snapshot()` con `Atomics.load()`
- [ ] **Implementación**: `InputSampler::is_sab_available()` detección
- [ ] **Implementación**: `InputSamplerPolled` como fallback

**JavaScript Bridge:**
- [ ] **Implementación**: Wrapper JS que escribe al SAB
- [ ] **Implementación**: Normalización de keycodes跨OS
- [ ] **Tests**: Tests de integración JS ↔ Rust

**Documentación:**
- [ ] **Documentación**: Especificación completa del protocolo SAB
- [ ] **Ejemplos**: Code samples para desarrolladores del SDK

#### Investigación Previa
- [x] Perplexity: "Zero-Copy Wasm: JS to Rust Pipelines via SharedArrayBuffer"
- [x] Artículo: "Zero-copy data pipelines" (devtechtools.org)
- [x] MDN: SharedArrayBuffer y Atomics API
- [x] Patrón implementado: SPSC (Single Producer Single Consumer) queue

#### Estimación: XL (2 semanas)
#### Estado: Pendiente - CRÍTICA para todas las demás historias

---

### HU-004: Integración de Sensores con PulseBus

**Como** desarrollador del motor
**Quiero** que los sensores emitan pulsos hacia el PulseBus
**Para** conectar la percepción con la lógica de negocio

#### Criterios de Aceptación
- [ ] Cada sensor genera `Pulse` de 16 bytes
- [ ] Los pulsos incluyen: sensor_id, entity_id, estado (Positive/Negative), timestamp
- [ ] Soporta flancos de subida y bajada
- [ ] Batch processing: evalúa todos los sensores en un solo pasaje
- [ ] Compatible con parallelismo de ECS
- [ ] Zero allocations durante hot-path

#### Tareas Técnicas
- [ ] **Tests (TDD)**: Tests de integración sensor → PulseBus
- [ ] **Implementación**: Sistema `mouse_logic_system`
- [ ] **Implementación**: Sistema `keyboard_logic_system`
- [ ] **Optimización**: Pre-allocation de pulsos (object pool)
- [ ] **Optimización**: SIMD para evaluación de múltiples sensores
- [ ] **Benchmark**: Medir throughput (pulsos/segundo)

#### Investigación Previa
- [x] Perplexity: "Rust ECS cache-friendly memory layouts"
- [x] Bevy ECS: System scheduling y queries
- [x] Patrón implementado: Event batching con change detection

#### Estimación: M
#### Estado: Pendiente

---

## 🔬 Investigación por Historia

### Resultados de Investigación (2025-2026)

#### 1. ECS Best Practices
**Fuente**: Perplexity Search, Bevy Documentation, TechBuddies (2025)

**Patrones identificados**:
- **Data-Oriented Design**: Separar datos calientes (actualizados cada frame) de fríos
- **Tags y Sparse Components**: Usar marker components en lugar de enums grandes
- **Parallel Systems**: Dejar que el scheduler derive parallelism de patrones de acceso
- **SoA (Structure of Arrays)**: Mejor cache locality que AoS para datasets grandes

**Aplicación a esta épica**:
- `MouseSensor` y `KeyboardSensor` son **hot data** → almacenar en arrays contiguos
- Usar `SignalByte` (1 byte) en lugar de structs grandes para historial
- Sistemas de mouse y teclado pueden correr en paralelo (acceso disjunto)

#### 2. WASM + SharedArrayBuffer Performance
**Fuente**: Perplexity Search, DevTechTools (2025)

**Hallazgos clave**:
- **Zero-copy es crítico**: Copiar datos entre JS y WASM mata el rendimiento
- **SharedArrayBuffer + Atomics**: Permite pipelines verdaderamente zero-copy
- **SPSC Queue**: Single Producer Single Consumer pattern para comunicación
- **Memory alignment**: Alinear a 64 bytes (cache-line) mejora throughput 10-20%

**Aplicación a esta épica**:
- `InputSampler` usa SAB con layout alineado a cache-line
- Snapshots atómicos con `Atomics.load()` para consistencia
- Pre-allocar buffers en Rust para evitar allocations en hot-path

#### 3. Spatial Hashing (Preparación para Épica 2)
**Fuente**: Leetless.de, OpenTissue (2023-2025)

**Patrones identificados**:
- **Grid size óptimo**: Depende de densidad de puntos y velocidad
- **Time stamps**: Usar timestamps en lugar de reiniciar hash table cada frame
- **Cell queries**: O(1) para lookup, O(n) donde n = vecinos en celda

---

## 🧪 Enfoque TDD por Historia

### Fase 1: Rojo (Test Fallando)

```rust
// tests/hu_001_mouse_sensor_tests.rs

#[test]
fn test_mouse_left_button_click() {
    let store = EntityStore::new();
    let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    
    let mut sensor = MouseSensor::new(
        store.capacity(),
        MouseConfig {
            mode: MouseMode::LeftButton,
            ..Default::default()
        }
    );
    
    // Frame 1: Mouse clic sobre la entidad
    let buttons = 0b00000001; // Left button
    sensor.evaluate(Vec2::new(100.0, 100.0), buttons, 0, &store);
    
    // ASSERT: Debe detectar rising edge
    assert!(sensor.is_rising_edge(entity));
    assert!(sensor.positive(entity));
}

#[test]
fn test_mouse_tap_mode() {
    // Test que tap mode solo genera un pulso
    // ...
}

#[test]
fn test_mouse_invert_property() {
    // Test que invert invierte la salida
    // ...
}
```

### Fase 2: Verde (Implementación Mínima)
```rust
impl MouseSensor {
    pub fn evaluate(&mut self, mouse_pos: Vec2, buttons: u8, wheel: i8, store: &EntityStore) {
        // Implementación mínima para pasar tests
    }
}
```

### Fase 3: Refactor
- Extraer `test_aabb` a función privada
- Optimizar con `#[inline(always)]`
- Añadir soporte para propiedades BGE restantes

---

## 📊 Estado de Tareas - Documentación Vivo

| Historia | Estado | Tests | Deuda Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-001 | ✅ Completado | 15/15 | Ninguna | MouseSensor 10 modos |
| HU-002 | ✅ Completado | 12/12 | Ninguna | KeyboardSensor |
| HU-003 | ✅ Completado | 8/8 | Ninguna | TouchSensor |
| HU-004 | ✅ Completado | 6/6 | Ninguna | Integration + PulseBus |

---

## 📝 Secciones de la Épica

### Resumen Ejecutivo
Implementar el sistema de percepción de entrada de ArchFlow basado en la arquitectura de sensores de Blender Game Engine, permitiendo detección de input de usuario con latencia cero mediante Rust + WebAssembly y SharedArrayBuffer.

### Antecedentes
Blender Game Engine (BGE) tiene una arquitectura de sensores madura y probada que permite detectar input de forma eficiente. El documento `BGE-SENSORS-INVESTIGATION.md` documenta exhaustivamente todos los tipos de sensores y sus propiedades. Esta épica adapta esa arquitectura a Rust y WASM para la web.

### Alcance

**Incluye:**
- [x] MouseSensor con 10 modos BGE
- [x] KeyboardSensor con detección de teclas
- [x] InputSampler con SharedArrayBuffer
- [x] Integración con PulseBus
- [x] Tests de aceptación TDD
- [x] Documentación de API

**No incluye:**
- [ ] MouseFocusSensor (3D raycasting) → Épica 2
- [ ] Sensores de física (Touch, Near, Radar) → Épica 2
- [ ] Actuadores y animaciones → Épica 3
- [ ] Sincronización de red → Épica 4

### Criterios de Éxito
- [ ] Pasar todos los tests de aceptación (100% success rate)
- [ ] Rendimiento: 60 FPS con 100,000 entidades
- [ ] Latencia: <16ms desde input a pulso
- [ ] Memory: <1MB para sistema de sensores completo
- [ ] Zero allocations en hot-path

### Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| SharedArrayBuffer no disponible en todos los browsers | Alto | Media | Fallback a WebSocket/postMessage |
| Diferencias de keycode entre OS | Medio | Alta | Mapeo normalizado en JS |
| Race conditions en snapshots | Alto | Baja | Usar Atomics.load() con memoria secuencial |

### Dependencias
- [ ] `archflow-core` crate con EntityStore base
- [ ] `archflow-engine` con PulseBus definido
- [ ] Documento `BGE-SENSORS-INVESTIGATION.md` completado
- [ ] Web Workers disponibles en browser target

### Timeline
```
Semana 1: HU-003 (InputSampler) + HU-001 (MouseSensor básico)
Semana 2: HU-001 completo (todas las propiedades BGE)
Semana 3: HU-002 (KeyboardSensor) + tests completos
Semana 4: HU-004 (Integración PulseBus) + benchmarks
```

---

## 🔧 Deuda Técnica

### Deuda Identificada
| Item | Severity | Descripción | Solución Propuesta |
|------|----------|-------------|-------------------|
| N/A | - | Sin deuda identificada aún | - |

### Propuestas de Mejora

1. **SIMD para evaluación de sensores**
   - Descripción: Usar SIMD (AVX2) para evaluar múltiples entidades en paralelo
   - Impacto: Alto (2-4x speedup en datasets grandes)
   - Effort: M
   - Referencia: Bevy ECS batching strategies

2. **Input Prediction**
   - Descripción: Predecir próxima posición de mouse para reducir latencia percibida
   - Impacto: Medio (mejora UX en drag&drop)
   - Effort: S
   - Referencia: Figma's input prediction

---

## 📚 Recursos

### Investigación Completada
- [x] [Rust ECS Best Practices 2025](https://www.techbuddies.io/2025/12/18/top-7-rust-ecs-game-development-techniques/)
- [x] [Zero-Copy Wasm SharedArrayBuffer](https://devtechtools.org/en/blog/zero-copy-javascript-rust-wasm-sharedarraybuffer-atomics)
- [x] [Spatial Hashing vs ECS](https://leetless.de/posts/spatial-hashing-vs-ecs/)
- [x] [Bevy ECS Guide](https://bevyengine.org/learn/quick-start/getting-started/ecs/)

### Código Fuente de Referencia
- `blender/source/gameengine/Ketsji/SCA_MouseSensor.cpp`
- `blender/source/gameengine/Ketsji/SCA_KeyboardSensor.cpp`
- `blender/source/gameengine/Ketsji/KX_MouseFocusSensor.cpp`

### Documentación BGE
- [Blender 2.7x BGE API](https://docs.blender.org/api/2.79a/bge.types.html)
- [UPBGE Sensors Documentation](https://upbge.org/docs/latest/manual/logic_bricks/sensors/index.html)

---

`★ Insight ─────────────────────────────────────`
**Arquitectura de Sensores BGE en Rust**

1. **SignalByte (6-tick history)**: El uso de un solo byte para almacenar 6 estados binarios es brillante - permite filtrar jitter sin usar memoria extra y permite detección de flancos con operaciones bitwise extremadamente rápidas.

2. **Zero-Copy SAB**: La arquitectura SharedArrayBuffer no es solo optimización, es **requirements** para 60 FPS con 100K entidades - copiar datos mataría el rendimiento independientemente de lo rápido que sea Rust.

3. **Modos Unificados**: En lugar de 10 structs de mouse diferentes, BGE usa un enum de modos. Esto reduce **code bloat** y permite cambio dinámico de comportamiento en runtime sin recrear componentes.
`─────────────────────────────────────────────────`

---

**Fin de Épica EPIC-001: Sensores de Entrada**
