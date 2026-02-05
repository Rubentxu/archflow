---
title: "Análisis Integrado: Sensores de Mouse Actuales vs. BGE - Estrategia de Unificación"
author: Claude Code
date: 2025-02-01
status: Final
context: Integration of logic-bricks-architecture-investigation.md + BGE-SENSORS-INVESTIGATION.md
---

# Análisis Integrado: Sensores de Mouse - Estado Actual vs. Visión BGE

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| Fecha | 2025-02-01 |
| Estado | Completada |
| Documentos base | logic-bricks-architecture-investigation.md + BGE-SENSORS-INVESTIGATION.md |

---

## 🎯 Objetivo

Analizar la **brecha entre la implementación actual** de sensores de mouse en ArchFlow y la **arquitectura fiel a BGE** descrita en la investigación de sensores, proporcionando una **estrategia clara de migración**.

---

## 1. Estado Actual: Los 6 Sensores de Mouse

### 1.1 Inventario Completo

```
crates/archflow-logic/src/sensors/
├── mouse_over.rs      ~150 LOC, 100 KB memory
├── mouse_click.rs     ~250 LOC, 300 KB memory
├── right_click.rs     ~180 LOC, 100 KB memory
├── double_tap.rs      ~150 LOC, ~1 MB memory
├── long_press.rs      ~140 LOC, ~900 KB memory
└── proximity.rs       ~120 LOC, 100 KB memory
```

**Total**: ~990 LOC, ~2.5 MB para 100k entidades

### 1.2 Tabla Comparativa: Actual vs. BGE

| Aspecto | ArchFlow Actual | BGE Reference | Gap |
|---------|-----------------|---------------|-----|
| **Número de clases** | 6 structs | 1 clase + 1 subclase | 6x |
| **Configuración** | Hardcoded (compilación) | Runtime (`mode` property) | Dinámico vs. Estático |
| **AABB Testing** | 6 implementaciones duplicadas | 1 implementación | Shotgun Surgery |
| **Memory overhead** | 2.5 MB | ~200 KB (estimado) | 12-25x |
| **Extensibilidad** | Requiere nuevo struct | Añadir valor a enum | Alta fricción |

### 1.3 Análisis de Duplicación

**Ejemplo: Código AABB duplicado en 6 archivos**

```rust
// mouse_over.rs - líneas 95-107
for (i, transform) in store.transforms.iter().enumerate() {
    let center_x = transform[0];
    let center_y = transform[1];
    let width = transform[2];
    let height = transform[3];
    let half_w = width * 0.5;
    let half_h = height * 0.5;
    let min_x = center_x - half_w;
    let max_x = center_x + half_w;
    let min_y = center_y - half_h;
    let max_y = center_y + half_h;
    let is_over = mouse_pos.x >= min_x && mouse_pos.x <= max_x
               && mouse_pos.y >= min_y && mouse_pos.y <= max_y;
    // ...
}

// mouse_click.rs - líneas 120-132 (¡IDÉNTICO!)
// right_click.rs - líneas 95-107 (¡IDÉNTICO!)
// double_tap.rs - líneas 74-86 (¡IDÉNTICO!)
// long_press.rs - líneas 67-79 (¡IDÉNTICO!)
```

**Problema**: Si queremos mejorar el algoritmo de colisión (ej. soportar rotación), necesitamos modificar **6 archivos**.

---

## 2. Visión BGE: Arquitectura Unificada

### 2.1 El Patrón BGE

Basado en `BGE-SENSORS-INVESTIGATION.md`:

```python
# BGE Python API
sensor = bge.logic.getCurrentController().sensors["Mouse"]

# Configurar modo en runtime
sensor.mode = KX_MOUSESENSORMODE_LEFTBUTTON  # = 1
# o
sensor.mode = KX_MOUSESENSORMODE_MOVEMENT    # = 10

# Propiedades heredadas de SCA_ISensor
sensor.invert = True   # Invertir salida
sensor.tap = True      # Modo pulso único
sensor.level = 3       # Frecuencia de activación
```

### 2.2 Mapeo: Sensores Actuales → Modos BGE

| Sensor Actual | Modo BGE Equivalente | Notas |
|--------------|---------------------|-------|
| `MouseOverSensor` | `KX_MOUSESENSORMODE_MOVEMENT` | Direct mapping |
| `MouseClickSensor` (primary) | `KX_MOUSESENSORMODE_LEFTBUTTON` | Uno de los 3 canales |
| `MouseClickSensor` (secondary) | `KX_MOUSESENSORMODE_RIGHTBUTTON` | Canal secundario |
| `MouseClickSensor` (middle) | `KX_MOUSESENSORMODE_MIDDLEBUTTON` | Canal medio |
| `RightClickSensor` | `KX_MOUSESENSORMODE_RIGHTBUTTON` | Duplicado |
| `ProximitySensor` | Custom (no es BGE mouse) | Mantener separado |

### 2.3 ¿Qué NO es BGE-Nativo?

| Sensor Actual | Origen | Veredicto |
|--------------|--------|-----------|
| `DoubleTapSensor` | ✗ No existe en BGE | Mover a **Python Controller** |
| `LongPressSensor` | ✗ No existe en BGE | Mover a **Python Controller** |

**Patrón BGE**: El sensor detecta eventos **primitivos**. Lógica compleja (timing, patrones) va en **Controllers**, no en sensores.

---

## 3. Estrategia de Migración a Unified MouseSensor

### 3.1 Fase 1: Crear MouseSensor BGE-Faithful

**Archivo nuevo**: `sensors/mouse.rs`

```rust
/// Sensor de mouse unificado (BGE-Faithful)
pub struct MouseSensor {
    /// Modo de operación BGE
    mode: MouseMode,
    
    /// Configuración BGE (invert, tap, level)
    config: MouseConfig,
    
    /// Estado compartido (no duplicado por modo)
    signals: Vec<SignalByte>,
    
    /// Para wheel detection
    last_wheel: i8,
}

pub enum MouseMode {
    LeftButton = 1,      // KX_MOUSESENSORMODE_LEFTBUTTON
    MiddleButton = 2,    // KX_MOUSESENSORMODE_MIDDLEBUTTON
    RightButton = 3,     // KX_MOUSESENSORMODE_RIGHTBUTTON
    WheelUp = 8,         // KX_MOUSESENSORMODE_WHEELUP
    WheelDown = 9,       // KX_MOUSESENSORMODE_WHEELDOWN
    Movement = 10,       // KX_MOUSESENSORMODE_MOVEMENT
}
```

### 3.2 Fase 2: Migración de Usuarios

**Antes** (6 structs):

```rust
let mut mouse_over = MouseOverSensor::new();
let mut click_sensor = MouseClickSensor::new();
let mut right_click = RightClickSensor::new();
```

**Después** (1 struct con configuración):

```rust
// Mouse-over (Movement mode)
let mut mouse_over = MouseSensor::new(
    store.capacity(),
    MouseConfig { mode: MouseMode::Movement, ..Default::default() }
);

// Click izquierdo (LeftButton mode)
let mut click_sensor = MouseSensor::new(
    store.capacity(),
    MouseConfig { mode: MouseMode::LeftButton, ..Default::default() }
);

// Click derecho (RightButton mode)
let mut right_click = MouseSensor::new(
    store.capacity(),
    MouseConfig { mode: MouseMode::RightButton, ..Default::default() }
);
```

### 3.3 Fase 3: Double-Click y Long-Press como Controllers

**Patrón BGE**: Lógica de timing en **Controllers**, no en sensores.

```rust
// Nuevo: sensors/controllers/timing.rs

/// Controller de double-click (lógica, no sensor)
pub struct DoubleClickController {
    threshold_ms: u64,       // 300ms como BGE
    last_click_time: Vec<Option<u64>>,
    click_count: Vec<u8>,
}

impl DoubleClickController {
    pub fn process(
        &mut self,
        entity: EntityId,
        pulse: &Pulse,
        current_time: u64
    ) -> bool {
        if !pulse.is_positive() {
            return false;
        }
        
        // Lógica de timing (extraída de DoubleTapSensor)
        if let Some(last) = self.last_click_time[entity_idx] {
            if current_time - last <= self.threshold_ms {
                self.click_count[entity_idx] += 1;
                return self.click_count[entity_idx] >= 2;
            }
        }
        
        self.last_click_time[entity_idx] = Some(current_time);
        self.click_count[entity_idx] = 1;
        false
    }
}
```

**Ventaja**: El mismo controller funciona con **cualquier sensor** que emita pulsos, no solo mouse.

---

## 4. Plan de Refactorización Paso a Paso

### 4.1 Paso 1: Crear Estructura Base

| Archivo | Acción | LOC estimadas |
|---------|--------|---------------|
| `sensors/mouse.rs` | Crear `MouseSensor`, `MouseMode`, `MouseConfig` | ~300 |
| `sensors/mouse.rs` | Implementar `evaluate()` con switch por modo | ~200 |
| `sensors/mod.rs` | Añadir `pub mod mouse;` | ~1 |
| `tests/mouse_test.rs` | Tests para todos los modos | ~150 |

**Total**: ~650 LOC nuevas

### 4.2 Paso 2: Migrar Usuarios Existentes

| Archivo | Cambio | Razón |
|---------|--------|--------|
| `engine.rs` | Reemplazar `MouseOverSensor` → `MouseSensor::new(Movement)` | Unificar |
| `bridge.rs` | Actualizar exports de WASM | API pública |
| `sensors/mod.rs` | Deprecarar exports viejos | Backward compatibility |

**Backward compatibility**:

```rust
// Deprecación suave
#[deprecated(since = "0.4.0", note = "Use MouseSensor with MouseMode::Movement instead")]
pub type MouseOverSensor = MouseSensor;

impl MouseOverSensor {
    pub fn new(capacity: usize) -> Self {
        Self::with_config(capacity, MouseConfig {
            mode: MouseMode::Movement,
            ..Default::default()
        })
    }
}
```

### 4.3 Paso 3: Mover Lógica de Timing a Controllers

| Archivo | Acción | LOC |
|---------|--------|-----|
| `controllers/timing.rs` | Crear `DoubleClickController`, `LongPressController` | ~200 |
| `mapping/table.rs` | Soportar controllers en conexiones | ~50 |
| `sensors/double_tap.rs` | **ELIMINAR** (mover a controller) | -150 |
| `sensors/long_press.rs` | **ELIMINAR** (mover a controller) | -140 |

**Net change**: -90 LOC (reducción de código)

---

## 5. Análisis de Impacto

### 5.1 Reducción de Código

| Métrica | Antes | Después | Mejora |
|---------|-------|---------|--------|
| **Archivos de mouse** | 6 | 1 | -5 archivos (83%) |
| **Total LOC mouse** | ~990 | ~650 | -340 LOC (34%) |
| **AABB duplicado** | 6 veces | 1 vez | -5 duplicaciones |
| **Memory (100k)** | 2.5 MB | ~200 KB | -2.3 MB (92%) |

### 5.2 Beneficios de la Unificación

1. **Mantenibilidad**: 1 archivo vs. 6 archivos
2. **Extensibilidad**: Añadir modo = añadir valor a enum
3. **Performance**: Menos memory footprint, mejor cache utilization
4. **Fidelidad BGE**: Sigue exactamente el patrón de BGE
5. **Runtime config**: Cambiar modo sin recompilar

### 5.3 Costos de la Migración

| Costo | Estimado | Mitigación |
|-------|----------|------------|
| **Desarrollo** | 3-5 días | Implementación incremental |
| **Testing** | 2-3 días | Tests exhaustivos por modo |
| **Documentación** | 1 día | Ejemplos de migración |
| **Backward compat** | 1 día | Type aliases y deprecations |

**Total**: 7-10 días de trabajo

---

## 6. Comparación Código a Código

### 6.1 Antes: 6 Sensores Separados

```rust
// USO ACTUAL (6 structs diferentes)
let mut mouse_over = MouseOverSensor::new();
let mut click = MouseClickSensor::new();
let mut right = RightClickSensor::new();
let mut double = DoubleTapSensor::new();
let mut long = LongPressSensor::new();

// Cada uno con su propia evaluación
mouse_over.sample(mouse_pos, store);
click.sample(mouse_pos, buttons, store);
right.sample(mouse_pos, buttons, store);
double.sample(mouse_pos, true, time, buttons, store);
long.sample(mouse_pos, true, time, store);
```

### 6.2 Después: MouseSensor Unificado

```rust
// USO PROPUESTO (1 struct, múltiples configuraciones)
let mut mouse_over = MouseSensor::new(
    store.capacity(), 
    MouseConfig { mode: MouseMode::Movement, ..Default::default() }
);

let mut click = MouseSensor::new(
    store.capacity(),
    MouseConfig { mode: MouseMode::LeftButton, ..Default::default() }
);

let mut right = MouseSensor::new(
    store.capacity(),
    MouseConfig { mode: MouseMode::RightButton, ..Default::default() }
);

// Evaluación unificada
mouse_over.evaluate(mouse_pos, buttons, wheel, store);
click.evaluate(mouse_pos, buttons, wheel, store);
right.evaluate(mouse_pos, buttons, wheel, store);

// Double-click y Long-press ahora son CONTROLLERS
let mut double_ctrl = DoubleClickController::new();
let mut long_ctrl = LongPressController::new();

// Procesar pulsos con lógica de timing
if click.positive(entity) {
    if double_ctrl.process(entity, pulse, time) {
        // Es double-click
    }
}
```

---

## 7. Roadmap de Implementación

### Semana 1: Fundamentos

- [ ] Día 1-2: Crear `MouseSensor`, `MouseMode`, `MouseConfig`
- [ ] Día 3-4: Implementar `evaluate()` con todos los modos
- [ ] Día 5: Tests unitarios exhaustivos

### Semana 2: Migración

- [ ] Día 1: Migrar `MouseOverSensor` → `MouseMode::Movement`
- [ ] Día 2: Migrar `MouseClickSensor` → `MouseMode::LeftButton`
- [ ] Día 3: Migrar `RightClickSensor` → `MouseMode::RightButton`
- [ ] Día 4-5: Actualizar `engine.rs` y `bridge.rs`

### Semana 3: Controllers y Limpieza

- [ ] Día 1-2: Crear `DoubleClickController` y `LongPressController`
- [ ] Día 3: Actualizar `LogicMappingTable` para soportar controllers
- [ ] Día 4: Deprecarar/eliminar sensores viejos
- [ ] Día 5: Documentación y ejemplos

---

## 8. Recomendación Final

✅ **APROBAR REFRACTORIZACIÓN**

La migración a `MouseSensor` unificado es **recomendada** porque:

1. **Alineación con BGE**: Sigue fielmente el patrón de BGE
2. **Reducción de deuda**: Elimina 5 archivos y 340 LOC de duplicación
3. **Mantenibilidad**: 1 archivo vs. 6 para mantener
4. **Extensibilidad**: Añadir modos es trivial
5. **Performance**: 92% menos memory usage

**Prioridad**: Alta, pero **NO bloqueante**. Puede hacerse en paralelo con BgeCore.

---

## 9. Referencias Cruzadas

| Documento | Sección Clave |
|-----------|--------------|
| `logic-bricks-architecture-investigation.md` | Sección 2.1: Estado Actual |
| `BGE-SENSORS-INVESTIGATION.md` | Sección 3: Ejemplo Rust MouseSensor |
| `refinamiento-logic-brics.md` | Sección 2.2: Pipeline de Ejecución |

---

**Fin del Análisis Integrado**

---

*Análisis realizado por Claude Code*
*Fecha: 2025-02-01*
*Proyecto: ArchFlow - Mouse Sensors Unification Strategy*
