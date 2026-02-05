# EPIC-WEB-010: ArchFlow SDK con Logic Bricks ✅

## Sistema de Programación Reactiva Sensor→Controller→Actuator con SignalByte de 6 Ticks

---

**Status**: COMPLETADA
**Version**: 1.0.0
**Last Updated**: 2026-02-03

## 1. Resumen Ejecutivo

Esta épica define la arquitectura e implementación del **SDK de ArchFlow basado en Logic Bricks**, un sistema de programación reactiva inspirado en Blender Game Engine (BGE) pero optimizado para aplicaciones de canvas colaborativo con **100,000+ entidades a 60 FPS**.

### 1.1 Diferencias Clave con BGE

| Aspecto | BGE Original | ArchFlow (Esta Épica) |
|---------|--------------|------------------------|
| **Escala** | 100-1,000 objetos | 100,000+ entidades |
| **Patrón** | Sensor→Controller→Actuator (objetos independientes) | Batch processing con SignalBytes compartidos |
| **Historial** | Solo estado actual (boolean) | SignalByte de 6 ticks (100ms) |
| **Controller** | Python scripts | Controllers predefinidos + CustomJS |
| **Memoria** | 100+ bytes por sensor | 1 byte por entidad por sensor |
| **Rendimiento** | O(n²) naive collision | O(n) con SpatialHash |

### 1.2 Arquitectura Core

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ARQUITECTURA LOGIC BRICKS                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                  │
│   │   SENSOR    │────▶│  SIGNAL    │────▶│ CONTROLLER  │                  │
│   │  (Input)    │     │   BYTE     │     │  (Lógica)  │                  │
│   │             │     │ (6 ticks)  │     │             │                  │
│   └─────────────┘     └─────────────┘     └──────┬──────┘                  │
│                                                   │                         │
│                                                   ▼                         │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                  │
│   │   ENTITY    │◀────│  ACTUATOR   │◀────│   PULSE     │                  │
│   │   STORE     │     │  (acción)  │     │  (flancos) │                  │
│   └─────────────┘     └─────────────┘     └─────────────┘                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 SignalByte: El Cerebro del Sistema

```rust
// 1 byte = 8 bits = 6 ticks de historial + 2 flags
// Bit 0: Estado actual (T)
// Bits 1-5: T-1 a T-5 (historial)
// Bits 6-7: Flags internos

struct SignalByte(u8);

impl SignalByte {
    pub fn push(&mut self, active: bool) {
        self.0 = (self.0 << 1) | (active as u8);  // Shift + insert
    }
    
    pub fn is_rising_edge(&self) -> bool {  // 0 → 1
        (self.0 & 0b00000011) == 0b00000001
    }
    
    pub fn is_falling_edge(&self) -> bool { // 1 → 0
        (self.0 & 0b00000011) == 0b00000010
    }
    
    pub fn is_steady(&self, ticks: u8) -> bool { // N ticks estables
        let mask = (1 << ticks) - 1;
        (self.0 & mask) == mask
    }
}
```

### 1.4 Objetivos de la Épica

- ✅ Implementar arquitectura Sensor→Controller→Actuator con SignalByte
- ✅ Proveer controllers predefinidos (AND, OR, NOT, Blinky, Debounce, Hysteresis)
- ✅ Permitir controllers custom mediante JavaScript
- ✅ Integrar con WASM para rendimiento extremo
- ✅ Cubrir 90% de interacciones de canvas colaborativo
- ✅ API developer-friendly con wrappers React

---

## 2. Capa de Sensores (Sensor Layer)

### 2.1 Taxonomía de Sensores

| Sensor | Tipo | Entrada | Salida |
|--------|------|---------|--------|
| `MouseOverSensor` | Posicional | Mouse position | SignalByte(hover) |
| `MouseClickSensor` | Evento | Buttons + Position | SignalByte(click) |
| `TouchSensor` | Colisión | AABB overlap | SignalByte(collision) |
| `ProximitySensor` | Distancia | SpatialHash query | SignalByte(near) |
| `RadarSensor` | Cono | Position + Angle | SignalByte(in_cone) |
| `KeyShortcutSensor` | Teclado | Keys[256] | SignalByte(key_active) |
| `DoubleTapSensor` | Tempo | Click timestamp | SignalByte(double_tap) |
| `LongPressSensor` | Tempo | Hold duration | SignalByte(long_press) |
| `RightClickSensor` | Evento | Button + Position | SignalByte(right_click) |

### 2.2 Interfaz de Sensor

```rust
// crates/archflow-logic/src/sensors/mod.rs

/// Trait base para todos los sensores
pub trait Sensor: Send {
    /// Tipo de sensor (identificador único)
    fn sensor_type(&self) -> SensorType;
    
    /// Evalúa el sensor y actualiza SignalByte
    fn evaluate(&mut self, store: &EntityStore, input: &InputSnapshot);
    
    /// Obtiene el SignalByte para una entidad
    fn signal(&self, entity: EntityId) -> SignalByte;
    
    /// Detecta flanco de subida (entrada)
    fn on_enter(&self, entity: EntityId) -> bool;
    
    /// Detecta flanco de bajada (salida)
    fn on_exit(&self, entity: EntityId) -> bool;
}
```

### 2.3 MouseOverSensor Implementation

```rust
// crates/archflow-logic/src/sensors/mouse_over.rs

pub struct MouseOverSensor {
    signals: Vec<SignalByte>,  // 1 byte por entidad
}

impl MouseOverSensor {
    /// Evalúa todas las entidades en modo batch (cache-friendly)
    pub fn evaluate(&mut self, mouse_pos: Vec2, store: &EntityStore) {
        for (i, transform) in store.transforms.iter().enumerate() {
            let center = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            
            // AABB hit test
            let is_over = 
                mouse_pos.x >= center.x - size.x * 0.5 &&
                mouse_pos.x <= center.x + size.x * 0.5 &&
                mouse_pos.y >= center.y - size.y * 0.5 &&
                mouse_pos.y <= center.y + size.y * 0.5;
            
            // Actualizar historial de 6 ticks
            self.signals[i].push(is_over);
        }
    }
    
    /// Detecta hover enter (rising edge)
    pub fn on_hover_enter(&self, entity: EntityId) -> bool {
        self.signals[entity.index()].is_rising_edge()
    }
    
    /// Detecta hover exit (falling edge)
    pub fn on_hover_exit(&self, entity: EntityId) -> bool {
        self.signals[entity.index()].is_falling_edge()
    }
    
    /// Detecta hover estable (hysteresis)
    pub fn is_stable_over(&self, entity: EntityId, ticks: u8) -> bool {
        self.signals[entity.index()].is_steady(ticks)
    }
}
```

### 2.4 ProximitySensor con SpatialHash

```rust
// crates/archflow-logic/src/sensors/proximity.rs

pub struct ProximitySensor {
    signals: Vec<SignalByte>,
    distance: f32,           // Radio de detección
    hysteresis: f32,         // Hysteresis para evitar oscilación
    spatial_hash: SpatialHash,  // Índice espacial O(1)
}

impl ProximitySensor {
    pub fn evaluate(&mut self, store: &EntityStore) {
        // Actualizar SpatialHash con posiciones actuales
        self.spatial_hash.clear();
        for (entity_id, transform) in store.transforms().enumerate() {
            let pos = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            let bounds = Rect::from_origin_size(pos, size);
            self.spatial_hash.insert(EntityId(entity_id), bounds);
        }
        
        // Para cada entidad, encontrar vecinas
        for (i, transform) in store.transforms().enumerate() {
            let pos = Vec2::new(transform[0], transform[1]);
            let nearby = self.spatial_hash.query_radius(pos, self.distance);
            
            // Señal activa si hay хотя бы una entidad cercana
            let is_near = nearby.len() > 1;  // >1 porque incluye sí mismo
            self.signals[i].push(is_near);
        }
    }
}
```

### 2.5 KeyShortcutSensor

```rust
// crates/archflow-logic/src/sensors/key_shortcut.rs

pub struct KeyShortcutSensor {
    signals: Vec<SignalByte>,
    key_states: [u8; 32],  // 256 bits para 256 teclas
    modifiers: u8,         // Shift=1, Ctrl=2, Alt=4, Meta=8
}

pub struct KeyEvent {
    pub keycode: u8,
    pub pressed: bool,
    pub modifiers: u8,
}

impl KeyShortcutSensor {
    /// Procesa evento de teclado
    pub fn process_event(&mut self, event: KeyEvent) {
        let byte_idx = event.keycode / 8;
        let bit_idx = event.keycode % 8;
        
        if event.pressed {
            self.key_states[byte_idx] |= 1 << bit_idx;
        } else {
            self.key_states[byte_idx] &= !(1 << bit_idx);
        }
        
        self.modifiers = event.modifiers;
    }
    
    /// Verifica si una tecla está presionada
    pub fn is_key_pressed(&self, keycode: u8) -> bool {
        let byte_idx = keycode / 8;
        let bit_idx = keycode % 8;
        (self.key_states[byte_idx] >> bit_idx) & 1 == 1
    }
    
    /// Verifica combinación de teclas
    pub fn is_shortcut_active(&self, keycode: u8, required_mods: u8) -> bool {
        self.is_key_pressed(keycode) && 
        (self.modifiers & required_mods) == required_mods
    }
}
```

---

## 3. Capa de Controladores (Controller Layer)

### 3.1 Controller Base y Tipos

```rust
// crates/archflow-logic/src/mapping/controller.rs

pub enum Controller {
    // Controladores lógicos básicos (BGE style)
    Direct,                    // Pasa señal tal cual
    And(SensorType),           // sensor_primary AND sensor_secondary
    Or(SensorType),            // sensor_primary OR sensor_secondary
    Not,                       // Invierte señal
    
    // Controladores predefinidos (Rust, alto rendimiento)
    Blinky {
        interval: u8,          // Parpadea cada N ticks
    },
    Debounce {
        ticks: u8,             // Espera estabilidad N ticks
    },
    Hysteresis {
        high: f32,             // Umbral de activación (0.0-1.0)
        low: f32,              // Umbral de desactivación
    },
    Threshold {
        value: f32,            // Activar si estabilidad >= valor
    },
    Pattern {
        mask: u8,              // Patrón binario a coincidir
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // CUSTOM CONTROLLER (WASM Architecture)
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // Controlador personalizado evaluado en JavaScript (el navegador ya tiene JS).
    //
    // Flujo WASM:
    // 1. TypeScript registra: customControllerRegistry.register(name, code)
    // 2. Rust detecta Controller::Custom y llama a evaluateCustomController()
    // 3. TS construye JSSignalProxy y JSContextProxy
    // 4. TS evalúa: new Function(code)(signal, context) → boolean
    // 5. TS retorna el resultado a Rust
    //
    // Ventajas:
    // - Sin motor JS embebido (el navegador ya tiene JS)
    // - Timeout de 50ms para evitar loops infinitos
    // - Acceso a todas las APIs del navegador
    // - Código familiar para developers web
    //
    Custom {
        name: String,          // Identificador único (para debugging)
        code: String,          // Código JS: (signal, context) => boolean
    },
}
```

### 3.2 Evaluación de Controladores

```rust
impl Controller {
    pub fn evaluate(
        &self,
        sensor_signal: SignalByte,
        context: &ControllerContext,
    ) -> bool {
        match self {
            Controller::Direct => sensor_signal.get_current(),
            
            Controller::And(other) => {
                let other_signal = context.get_sensor_signal(*other);
                sensor_signal.get_current() && other_signal.get_current()
            }
            
            Controller::Or(other) => {
                let other_signal = context.get_sensor_signal(*other);
                sensor_signal.get_current() || other_signal.get_current()
            }
            
            Controller::Not => !sensor_signal.get_current(),
            
            Controller::Blinky { interval } => {
                // Parpadeo: activo en ticks impares
                let tick = context.timestamp() / 16;  // 16ms per tick
                let phase = (tick / *interval as u64) % 2;
                phase == 0 && sensor_signal.get_current()
            }
            
            Controller::Debounce { ticks } => {
                sensor_signal.is_steady(*ticks)
            }
            
            Controller::Hysteresis { high, low } => {
                let state = context.get_hysteresis_state();
                let current = if sensor_signal.get_current() { 1.0 } else { 0.0 };
                match state {
                    HysteresisState::Low => current >= *high,
                    HysteresisState::High => current <= *low,
                }
            }
            
            Controller::Threshold { value } => {
                let stability = signal.count_ones() as f32 / 6.0;
                stability >= *value
            }
            
            Controller::Pattern { mask } => {
                (sensor_signal.as_u8() & mask) == mask
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CUSTOM CONTROLLER (WASM Architecture)
            // ═══════════════════════════════════════════════════════════════════════
            //
            // PRINCIPIO CLAVE: El navegador YA tiene JavaScript. No necesitamos
            // embeber un motor JS en Rust. La evaluación ocurre en JS y Rust
            // recibe el resultado.
            //
            // Flujo:
            // 1. TypeScript registra: customControllerRegistry.register(name, code)
            // 2. Rust detecta Controller::Custom
            // 3. Rust pasa datos a TS via wasm-bindgen: evaluateCustomController(...)
            // 4. TS construye JSSignalProxy y JSContextProxy
            // 5. TS evalúa: new Function(code)(signal, context)
            // 6. TS retorna boolean a Rust
            //
            Controller::Custom { name, .. } => {
                #[cfg(feature = "wasm")]
                {
                    // Delegar evaluación a TypeScript
                    crate::wasm_bridge::evaluate_custom_controller(
                        name,
                        sensor_signal.get_current(),
                        sensor_signal.is_rising_edge(),
                        sensor_signal.is_falling_edge(),
                        sensor_signal.count_ones(),
                        sensor_signal.get_history(),
                        context.timestamp(),
                        context.entity_id(),
                        &context.serialize_properties(),
                    )
                }

                #[cfg(not(feature = "wasm"))]
                {
                    // En modo no-WASM, loggear warning y retornar false
                    log::warn!("CustomController '{}' requiere feature 'wasm'", name);
                    false
                }
            }
        }
    }
}
```

### 3.3 Custom Controller con JavaScript (WASM Architecture)

**Principios Clave:** El navegador YA tiene JavaScript. No necesitamos embeber un motor JS en Rust. La arquitectura es:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    WASM CUSTOM CONTROLLER ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Rust (LogicSystem)         TypeScript (Web UI)         JavaScript Runtime │
│   ────────────────────       ─────────────────────       ─────────────────  │
│                               ┌───────────────────┐                          │
│   register(name, code) ─────►│ CustomController  │                          │
│                              │ Registry          │                          │
│                               └───────────────────┘                          │
│                                      │                                       │
│                                      ▼                                       │
│   evaluate(entity, signal) ──────► eval(code,        │                      │
│                               signalProxy,          │                      │
│                               context)              │                      │
│                                      │              │                      │
│                                      │              ▼                      │
│                                      │      new Function(code)             │
│                                      │      (signal, context) => {...}    │
│                                      │              │                      │
│                                      │              ▼                      │
│                                      └──── return boolean ◄─────┐         │
│                                                                     │         │
│   Rust recibe ◄─────────────────────────────────────────────────────┘         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**TypeScript Implementation:**

```typescript
// crates/archflow-web-ui/src/logic/CustomController.ts

/// Registro global de controllers personalizados
class CustomControllerRegistry {
    private controllers = new Map<string, CustomController>();

    /// Registra un nuevo controller personalizado
    register(name: string, code: string): void {
        this.controllers.set(name, new CustomController(name, code));
    }

    /// Evalúa un controller para una entidad específica
    evaluate(
        name: string,
        signal: JSSignalProxy,
        context: JSContextProxy
    ): boolean {
        const controller = this.controllers.get(name);
        if (!controller) {
            console.warn(`CustomController "${name}" not found`);
            return false;
        }
        return controller.evaluate(signal, context);
    }

    /// Limpia todos los controllers registrados
    clear(): void {
        this.controllers.clear();
    }
}

/// Proxy de SignalByte para JavaScript
interface JSSignalProxy {
    /** Estado actual (boolean) */
    readonly current: boolean;

    /** Flanco de subida: 0 → 1 */
    readonly isRisingEdge: boolean;

    /** Flanco de bajada: 1 → 0 */
    readonly isFallingEdge: boolean;

    /** Estable durante N ticks */
    isSteady(ticks: number): boolean;

    /** Número de ticks en HIGH (0-6) */
    readonly countOnes: number;

    /** Número de ticks en LOW (0-6) */
    readonly countZeros: number;

    /** Historial completo como número (bits 0-5) */
    readonly history: number;
}

/// Contexto para custom controllers
interface JSContextProxy {
    /** Timestamp actual del frame */
    readonly timestamp: number;

    /** ID de la entidad evaluada */
    readonly entityId: number;

    /** Obtiene una propiedad personalizada */
    getProperty(key: string): boolean | number | string | null;

    /** Establece una propiedad personalizada */
    setProperty(key: string, value: boolean | number | string): void;
}

/// Controller personalizado evaluado en JavaScript
export class CustomController {
    readonly name: string;
    private code: string;
    private compiledFn: ((signal: JSSignalProxy, context: JSContextProxy) => boolean) | null = null;

    constructor(name: string, code: string) {
        this.name = name;
        this.code = code;
    }

    /// Compila el código JS una sola vez
    private compile(): void {
        if (this.compiledFn) return;

        // Crear función desde string - ES el patrón estándar para eval() seguro
        // NOTA: new Function() es más seguro que eval() porque no accede al scope local
        const fnBody = `
            "use strict";
            return (function(signal, context) {
                ${this.code}
            });
        `;

        this.compiledFn = new Function(fnBody)();
    }

    /// Evalúa el controller
    evaluate(signal: JSSignalProxy, context: JSContextProxy): boolean {
        try {
            this.compile();

            // Timeout de 50ms para evitar loops infinitos
            const startTime = performance.now();
            const timeoutMs = 50;

            // Wrapper con timeout
            const safeEval = (): boolean => {
                if (performance.now() - startTime > timeoutMs) {
                    console.warn(`CustomController "${this.name}" timeout`);
                    return false;
                }
                return this.compiledFn!(signal, context);
            };

            return safeEval();
        } catch (error) {
            console.warn(`CustomController "${this.name}" error:`, error);
            return false;
        }
    }
}

/// Instancia global del registro
export const customControllerRegistry = new CustomControllerRegistry();
```

**API del Usuario (lo que los developers escriben):**

```typescript
// Ejemplo 1: Hover estable + modifier
customControllerRegistry.register('tooltipOnCtrlHover', `
    const stable = signal.isSteady(6);  // 100ms estable
    const hasCtrl = (context.getProperty('modifiers') & 2) !== 0;
    return stable && hasCtrl;
`);

// Ejemplo 2: Doble condición con propiedad custom
customControllerRegistry.register('advancedSelect', `
    const isHover = signal.current;
    const wasClicked = context.getProperty('wasClicked');
    const isShift = (context.getProperty('modifiers') & 1) !== 0;

    // Shift+Hover = toggle, solo Hover = select
    if (isShift && wasClicked) {
        context.setProperty('wasClicked', false);
        return false;
    }
    if (isHover && !wasClicked) {
        context.setProperty('wasClicked', true);
        return true;
    }
    return isHover;
`);

// Ejemplo 3: Patrón complejo
customControllerRegistry.register('gestureDetector', `
    const rising = signal.isRisingEdge;
    const stability = signal.countOnes / 6;
    const elapsed = context.timestamp - context.getProperty('lastClickTime');

    if (rising && elapsed > 200 && elapsed < 500) {
        context.setProperty('lastClickTime', context.timestamp);
        return true;
    }
    return false;
`);
```

**TypeScript - Bridge desde WASM:**

```typescript
// crates/archflow-web-ui/src/wasm-bridge.ts

/// Puente hacia WASM para custom controllers
export function registerCustomController(
    name: string,
    code: string
): void {
    customControllerRegistry.register(name, code);
}

/// Evalúa un custom controller (llamado desde Rust via wasm-bindgen)
export function evaluateCustomController(
    name: string,
    signalCurrent: boolean,
    signalRising: boolean,
    signalFalling: boolean,
    signalOnes: number,
    signalHistory: number,
    contextTimestamp: number,
    contextEntityId: number,
    contextPropertiesJson: string
): boolean {
    // Construir proxy de Signal
    const signalProxy: JSSignalProxy = {
        current: signalCurrent,
        isRisingEdge: signalRising,
        isFallingEdge: signalFalling,
        countOnes: signalOnes,
        countZeros: 6 - signalOnes,
        history: signalHistory,
        isSteady(ticks: number): boolean {
            const mask = (1 << ticks) - 1;
            return (signalHistory & mask) === mask;
        }
    };

    // Construir proxy de Context
    const properties = JSON.parse(contextPropertiesJson || '{}');
    const contextProxy: JSContextProxy = {
        timestamp: contextTimestamp,
        entityId: contextEntityId,
        getProperty(key: string) {
            return properties[key] ?? null;
        },
        setProperty(key: string, value: boolean | number | string) {
            properties[key] = value;
        }
    };

    return customControllerRegistry.evaluate(name, signalProxy, contextProxy);
}
```

**Rust - Registro y evaluación:**

```rust
// crates/archflow-logic/src/mapping/controller.rs

#[wasm_bindgen]
pub struct CustomControllerRegistry {
    // Referencia a TS para evaluar
    ts_registry: web_sys::JsValue,
}

#[wasm_bindgen]
impl CustomControllerRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            ts_registry: web_sys::window()
                .unwrap()
                .custom_controller_registry(),
        }
    }

    /// Registra un controller personalizado (llamado desde TS)
    pub fn register(&self, name: &str, code: &str) {
        let registry = js_sys::Reflect::get(
            &self.ts_registry,
            &"register".into()
        ).unwrap();

        let this = self.ts_registry.clone();
        let name = name.into();
        let code = code.into();

        registry.as_function().unwrap().call2(
            &this,
            &name,
            &code
        ).unwrap();
    }

    /// Evalúa un controller personalizado (placeholder)
    /// La evaluación real ocurre en TypeScript
    pub fn evaluate(&self, name: &str) -> bool {
        false // TypeScript evalúa y Rust usa el resultado
    }
}
```

### 3.4 Context para Controllers

```rust
// crates/archflow-logic/src/mapping/controller_context.rs

pub struct ControllerContext {
    pub entity_id: EntityId,
    pub timestamp: u64,
    pub mouse_pos: Vec2,
    pub modifiers: u8,
    hysteresis_state: HashMap<EntityId, HysteresisState>,
    custom_properties: HashMap<String, Value>,
}

impl ControllerContext {
    pub fn get_sensor_signal(&self, sensor: SensorType) -> SignalByte {
        // Obtiene señal de otro sensor para controllers AND/OR
        unimplemented!()
    }
    
    pub fn get_hysteresis_state(&self) -> HysteresisState {
        self.hysteresis_state
            .get(&self.entity_id)
            .copied()
            .unwrap_or(HysteresisState::Low)
    }
    
    pub fn get_property(&self, key: &str) -> Option<Value> {
        self.custom_properties.get(key).cloned()
    }
    
    pub fn set_property(&mut self, key: String, value: Value) {
        self.custom_properties.insert(key, value);
    }
    
    pub fn evaluate_custom(&self, code: &str, signal: SignalByte) -> bool {
        // Para Rust: evaluar código precompilado
        // Para JS: delegar a CustomController
        unimplemented!()
    }
}
```

---

## 4. Capa de Actuadores (Actuator Layer)

### 4.1 Actuator Trait

```rust
// crates/archflow-logic/src/actuators/mod.rs

pub trait Actuator: Send {
    fn actuator_type(&self) -> ActuatorType;
    
    fn execute(&self, pulse: &Pulse, store: &mut EntityStore) -> Command;
    
    fn can_execute(&self, pulse: &Pulse, store: &EntityStore) -> bool;
}
```

### 4.2 HighlightActuator

```rust
// crates/archflow-logic/src/actuators/highlight.rs

pub struct HighlightActuator {
    pub color: u32,      // Color en formato RGBA
    pub intensity: f32,  // 0.0-1.0 para animación
}

impl Actuator for HighlightActuator {
    fn actuator_type(&self) -> ActuatorType {
        ActuatorType::Highlight
    }
    
    fn execute(&self, pulse: &Pulse, store: &mut EntityStore) -> Command {
        let entity_idx = pulse.entity_id as usize;
        
        match pulse.state {
            PulseState::Positive => {
                // Activar highlight
                let new_color = blend_colors(
                    store.colors[entity_idx],
                    self.color,
                    self.intensity
                );
                store.colors[entity_idx] = new_color;
                store.dirty.set(entity_idx, true);
                
                Command::SetColor {
                    entity_idx,
                    color: new_color,
                }
            }
            PulseState::Negative => {
                // Desactivar highlight (revertir)
                let original_color = store.get_original_color(entity_idx);
                store.colors[entity_idx] = original_color;
                store.dirty.set(entity_idx, true);
                
                Command::RestoreColor { entity_idx }
            }
            PulseState::None => Command::None,
        }
    }
}
```

### 4.3 SelectActuator

```rust
// crates/archflow-logic/src/actuators/select.rs

pub struct SelectActuator {
    pub additive: bool,       // Añadir a selección en lugar de reemplazar
    pub range: bool,          // Selección por rango (primero-último)
}

impl Actuator for SelectActuator {
    fn execute(&self, pulse: &Pulse, store: &mut EntityStore) -> Command {
        let entity_idx = pulse.entity_id as usize;
        
        match pulse.state {
            PulseState::Positive => {
                if self.additive {
                    store.selection.add(entity_idx);
                } else if self.range {
                    // Seleccionar rango desde último seleccionado
                    if let Some(first) = store.selection.last() {
                        store.selection.add_range(first, entity_idx);
                    } else {
                        store.selection.add(entity_idx);
                    }
                } else {
                    // Reemplazar selección
                    store.selection.clear();
                    store.selection.add(entity_idx);
                }
                
                Command::Select {
                    entity_idx,
                    selected: true,
                }
            }
            PulseState::Negative => {
                store.selection.remove(entity_idx);
                Command::Select {
                    entity_idx,
                    selected: false,
                }
            }
            PulseState::None => Command::None,
        }
    }
}
```

### 4.4 MoveActuator

```rust
// crates/archflow-logic/src/actuators/move_.rs

pub struct MoveActuator {
    pub snap: SnapConfig,
    pub constrain: ConstrainAxis,
}

impl Actuator for MoveActuator {
    fn execute(&self, pulse: &Pulse, store: &mut EntityStore) -> Command {
        let entity_idx = pulse.entity_id as usize;
        let delta = pulse.delta;  // Movimiento desde InputSnapshot
        
        // Aplicar constraints
        let mut final_delta = delta;
        if self.constrain == ConstrainAxis::X {
            final_delta.y = 0.0;
        } else if self.constrain == ConstrainAxis::Y {
            final_delta.x = 0.0;
        }
        
        // Aplicar snap
        if let Some(snapped) = self.snap.snap_position(entity_idx, final_delta, store) {
            final_delta = snapped;
        }
        
        // Actualizar posición
        let old_pos = store.get_position(entity_idx);
        let new_pos = old_pos + final_delta;
        store.set_position(entity_idx, new_pos);
        
        // Sincronizar SpatialHash
        store.spatial_hash.update(entity_idx, new_pos);
        store.dirty.set(entity_idx, true);
        
        Command::Move {
            entity_idx,
            from: old_pos,
            to: new_pos,
        }
    }
}
```

---

## 5. Logic System: Orquestación

### 5.1 Arquitectura de Flujo

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        LOGIC SYSTEM LOOP                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 1. INPUT SAMPLER                                                    │ │
│  │    Lee SharedArrayBuffer o buffer de fallback                       │ │
│  │    → InputSnapshot { mouse_pos, buttons, keys, modifiers }          │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 2. EVALUAR SENSORES (Batch)                                         │ │
│  │                                                                     │ │
│  │    MouseOverSensor.evaluate(mouse_pos, store)                      │ │
│  │    TouchSensor.evaluate(store, spatial_hash)                       │ │
│  │    ProximitySensor.evaluate(store, spatial_hash)                   │ │
│  │    KeyShortcutSensor.evaluate(key_states)                          │ │
│  │    DoubleTapSensor.evaluate(click_times)                           │ │
│  │    LongPressSensor.evaluate(hold_times)                            │ │
│  │                                                                     │ │
│  │    Resultado: Array de SignalBytes actualizado                     │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 3. GENERAR PULSOS (Solo flancos)                                    │ │
│  │                                                                     │ │
│  │    Para cada entidad y sensor:                                      │ │
│  │    if signal.is_rising_edge():   push Pulse::positive              │ │
│  │    if signal.is_falling_edge(): push Pulse::negative              │ │
│  │    else:                     no push (optimización)                │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 4. PROCESAR WIRING TABLE                                            │ │
│  │                                                                     │ │
│  │    Para cada pulso:                                                 │ │
│  │    - Buscar conexiones en WiringTable                               │ │
│  │    - Para cada conexión:                                            │ │
│  │      → Evaluar Controller                                          │ │
│  │      → Si controller activa: → Ejecutar Actuator                   │ │
│  │      → Generar Command para undo/redo                              │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ 5. EJECUTAR COMMANDS                                               │ │
│  │                                                                     │ │
│  │    CommandHistory.push(command)  // Para undo/redo                 │ │
│  │    Command.execute(store)         // Aplicar al EntityStore         │ │
│  │    Dirty flags = true             // Marcar para re-render          │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 LogicSystem Implementation

```rust
// crates/archflow-logic/src/logic_system.rs

pub struct LogicSystem {
    pub input_sampler: InputSampler,
    pub pulse_bus: PulseBus,
    pub wiring: LogicMappingTable,
    
    // Sensores
    pub mouse_over: MouseOverSensor,
    pub mouse_click: MouseClickSensor,
    pub touch: TouchSensor,
    pub proximity: ProximitySensor,
    pub radar: RadarSensor,
    pub key_shortcut: KeyShortcutSensor,
    pub double_tap: DoubleTapSensor,
    pub long_press: LongPressSensor,
    pub right_click: RightClickSensor,
    
    // Command History para undo/redo
    pub command_history: CommandHistory,
}

impl LogicSystem {
    pub fn update(&mut self, store: &mut EntityStore) {
        // 1. Sample input
        let snapshot = self.input_sampler.take_snapshot();
        
        // 2. Evaluar sensores
        self.evaluate_sensors(store, &snapshot);
        
        // 3. Generar pulsos desde flancos
        let pulses = self.generate_pulses(store);
        
        // 4. Procesar a través de wiring table
        let commands = self.process_pulses(pulses, store);
        
        // 5. Ejecutar comandos
        for cmd in commands {
            cmd.execute(store);
            self.command_history.push(cmd);
        }
    }
    
    fn evaluate_sensors(&mut self, store: &EntityStore, snapshot: &InputSnapshot) {
        let mouse_pos = Vec2::new(snapshot.mouse_x as f32, snapshot.mouse_y as f32);
        
        // Batch evaluation de todos los sensores
        self.mouse_over.evaluate(mouse_pos, store);
        self.mouse_click.evaluate(mouse_pos, snapshot.buttons, store);
        self.touch.evaluate(store, &self.spatial_hash);
        self.proximity.evaluate(store, &self.spatial_hash);
        self.key_shortcut.evaluate(snapshot.key_states);
        self.double_tap.evaluate(snapshot.timestamp, snapshot.buttons, store);
        self.long_press.evaluate(snapshot.timestamp, snapshot.buttons, store);
        self.right_click.evaluate(mouse_pos, snapshot.buttons, store);
    }
    
    fn generate_pulses(&self, store: &EntityStore) -> Vec<Pulse> {
        let mut pulses = Vec::new();
        let timestamp = self.input_sampler.timestamp();
        
        // Para cada entidad, generar pulsos desde flancos
        for entity_idx in 0..store.len() {
            let entity_id = entity_idx as u32;
            
            // MouseOver → Rising/Falling
            let mo_signal = self.mouse_over.signal(entity_id);
            if mo_signal.is_rising_edge() {
                pulses.push(Pulse::positive(SensorType::MouseOver as u32, entity_id, timestamp));
            } else if mo_signal.is_falling_edge() {
                pulses.push(Pulse::negative(SensorType::MouseOver as u32, entity_id, timestamp));
            }
            
            // Touch → Rising/Falling
            let touch_signal = self.touch.signal(entity_id);
            if touch_signal.is_rising_edge() {
                pulses.push(Pulse::positive(SensorType::Touch as u32, entity_id, timestamp));
            } else if touch_signal.is_falling_edge() {
                pulses.push(Pulse::negative(SensorType::Touch as u32, entity_id, timestamp));
            }
            
            // Proximity → Solo positive (hysteresis ya manejada)
            let prox_signal = self.proximity.signal(entity_id);
            if prox_signal.get_current() {
                pulses.push(Pulse::positive(SensorType::Proximity as u32, entity_id, timestamp));
            }
            
            // DoubleTap → Rising edge solo
            let dt_signal = self.double_tap.signal(entity_id);
            if dt_signal.is_rising_edge() {
                pulses.push(Pulse::positive(SensorType::DoubleTap as u32, entity_id, timestamp));
            }
            
            // LongPress → Rising edge solo
            let lp_signal = self.long_press.signal(entity_id);
            if lp_signal.is_rising_edge() {
                pulses.push(Pulse::positive(SensorType::LongPress as u32, entity_id, timestamp));
            }
        }
        
        pulses
    }
    
    fn process_pulses(&self, pulses: Vec<Pulse>, store: &EntityStore) -> Vec<Command> {
        let mut commands = Vec::new();
        
        for pulse in pulses {
            // Buscar conexiones en wiring table
            let connections = self.wiring.get_connections(pulse.entity_id, pulse.sensor_id);
            
            for connection in connections {
                let context = ControllerContext::new(pulse.entity_id, pulse.timestamp);
                
                // Evaluar controller
                let sensor_signal = self.get_sensor_signal(connection.sensor_type, pulse.entity_id);
                if connection.controller.evaluate(sensor_signal, &context) {
                    // Controller activo → ejecutar actuator
                    let cmd = connection.actuator.execute(&pulse, store);
                    commands.push(cmd);
                }
            }
        }
        
        commands
    }
}
```

---

## 6. Wiring Table: Conexiones Sensor→Controller→Actuator

### 6.1 Estructura

```rust
// crates/archflow-logic/src/mapping/wiring_table.rs

pub struct Connection {
    pub entity_id: EntityId,
    pub sensor_type: SensorType,
    pub controller: Controller,
    pub actuator_type: ActuatorType,
    pub enabled: bool,
}

pub struct WiringTable {
    // Por entidad y sensor, lista de conexiones
    connections: HashMap<EntityId, HashMap<SensorType, Vec<Connection>>},
    // Por actuator, lista de entidades conectadas
    by_actuator: HashMap<ActuatorType, Vec<EntityId>>,
}

impl WiringTable {
    pub fn add_connection(
        &mut self,
        entity_id: EntityId,
        sensor_type: SensorType,
        controller: Controller,
        actuator_type: ActuatorType,
    ) {
        let connection = Connection {
            entity_id,
            sensor_type,
            controller,
            actuator_type,
            enabled: true,
        };
        
        self.connections
            .entry(entity_id)
            .or_default()
            .entry(sensor_type)
            .or_default()
            .push(connection);
        
        self.by_actuator
            .entry(actuator_type)
            .or_default()
            .push(entity_id);
    }
    
    pub fn get_connections(&self, entity_id: EntityId, sensor_type: SensorType) -> &[Connection] {
        self.connections
            .get(&entity_id)
            .and_then(|m| m.get(&sensor_type))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
    
    pub fn remove_connection(&mut self, entity_id: EntityId, sensor_type: SensorType) {
        if let Some(sensor_map) = self.connections.get_mut(&entity_id) {
            sensor_map.remove(&sensor_type);
        }
    }
    
    pub fn clear_entity(&mut self, entity_id: EntityId) {
        self.connections.remove(&entity_id);
    }
}
```

---

## 7. Integración con JavaScript/TypeScript

### 7.1 API de Alto Nivel

```typescript
// @archflow/sdk/src/logic/wiring.ts

import { WasmBridge } from '../wasm-bridge';

export class LogicMappingTable {
    private wasm: WasmBridge;
    private nativeTable: any;  // Referencia a LogicMappingTableWasm
    
    constructor(wasm: WasmBridge) {
        this.wasm = wasm;
        this.nativeTable = new wasm.LogicMappingTableWasm();
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Controladores básicos (BGE style)
    // ═══════════════════════════════════════════════════════════════════════
    
    static Direct(): Controller {
        return new Controller('direct');
    }
    
    static And(sensor: SensorType): Controller {
        return new Controller('and', sensor);
    }
    
    static Or(sensor: SensorType): Controller {
        return new Controller('or', sensor);
    }
    
    static Not(): Controller {
        return new Controller('not');
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Controladores predefinidos (Rust)
    // ═══════════════════════════════════════════════════════════════════════
    
    static Blinky(interval: number): Controller {
        return new Controller('blinky', { interval });
    }
    
    static Debounce(ticks: number): Controller {
        return new Controller('debounce', { ticks });
    }
    
    static Hysteresis(high: number, low: number): Controller {
        return new Controller('hysteresis', { high, low });
    }
    
    static Threshold(value: number): Controller {
        return new Controller('threshold', { value });
    }
    
    static Pattern(mask: number): Controller {
        return new Controller('pattern', { mask });
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Controlador personalizado (JavaScript)
    // ═══════════════════════════════════════════════════════════════════════
    
    static Custom(name: string, code: string): Controller {
        return new Controller('custom', { name, code });
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Conexiones sensor → actuator
    // ═══════════════════════════════════════════════════════════════════════
    
    addHighlight(
        entityId: number,
        sensor: SensorType,
        controller: Controller
    ): void {
        this.nativeTable.addHighlight(entityId, sensor, controller.toWasm());
    }
    
    addSelect(
        entityId: number,
        sensor: SensorType,
        controller: Controller
    ): void {
        this.nativeTable.addSelect(entityId, sensor, controller.toWasm());
    }
    
    addMove(
        entityId: number,
        sensor: SensorType,
        controller: Controller
    ): void {
        this.nativeTable.addMove(entityId, sensor, controller.toWasm());
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Consultas
    // ═══════════════════════════════════════════════════════════════════════
    
    hasConnection(entityId: number, sensor: SensorType): boolean {
        return this.nativeTable.hasConnection(entityId, sensor);
    }
    
    getConnectedEntities(): Uint32Array {
        return this.nativeTable.getConnectedEntities();
    }
    
    removeConnection(entityId: number, sensor: SensorType): void {
        this.nativeTable.removeConnection(entityId, sensor);
    }
    
    clear(): void {
        this.nativeTable.clear();
    }
}
```

### 7.2 Ejemplo de Uso

```typescript
// Ejemplo completo: Comportamiento de hover con highlight

const wiring = new LogicMappingTable(wasmBridge);
const entityId = 42;

// Highlight cuando mouse entra (rising edge)
wiring.addHighlight(
    entityId,
    SensorType.MouseOver,
    Controller.Direct()  // Signal pasa directo
);

// Highlight doble: hover + mantener 100ms (stable)
wiring.addHighlight(
    entityId,
    SensorType.MouseOver,
    Controller.Debounce({ ticks: 6 })  // 6 ticks = 100ms @ 60fps
);

// Seleccionar solo si hover + click
wiring.addSelect(
    entityId,
    SensorType.MouseClick,
    Controller.And(SensorType.MouseOver)  // Click AND Hover
);

// Mostrar tooltip después de hover estable + modifiers
wiring.addProperty(
    entityId,
    SensorType.MouseOver,
    Controller.Custom({
        name: 'tooltipWithModifier',
        code: `(signal, context) => {
            const stable = signal.is_steady(6);  // 100ms estable
            const modifiers = context.modifiers;
            const hasCtrl = (modifiers & 2) !== 0;
            return stable && hasCtrl;  // Ctrl + Hover estable
        }`
    })
);

// Parpadeo mientras drag
wiring.addHighlight(
    entityId,
    SensorType.Touch,
    Controller.Blinky({ interval: 3 })  // Parpadeo cada 3 ticks
);
```

---

## 8. Flow de Datos: JS → WASM → JS

### 8.1 SharedArrayBuffer Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SHARED ARRAY BUFFER (64 bytes)                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Offset  Size  Field                  Description                        │
│   ──────  ────  ──────              ────────────                        │
│     0     4    head                  Write index (atomic)                │
│     4     4    tail                  Read index (atomic)                 │
│     8     4    mouse_x               Mouse X position                   │
│    12     4    mouse_y               Mouse Y position                   │
│    16     1    buttons               Bitmask: 1=left, 2=right, 4=mid    │
│    17     1    modifiers             Bitmask: 1=shift, 2=ctrl, 4=alt    │
│    18     2    wheel_delta           Scroll wheel delta                 │
│    20     4    timestamp             Frame timestamp                    │
│    24    32    keys[32]              256 bits for key states           │
│    56     8    padding               Cache-line alignment               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Input Event Types

```typescript
enum InputEventType {
    MOUSE_MOVE = 1,
    MOUSE_DOWN = 0,
    MOUSE_UP = 2,
    MOUSE_WHEEL = 3,
    KEY_DOWN = 4,
    KEY_UP = 5,
}

interface InputEvent {
    type: InputEventType;
    x: number;       // Mouse X or keycode
    y: number;       // Mouse Y
    buttons: number; // Button bitmask
    modifiers: number; // Modifier bitmask
}
```

### 8.3 Push Input Event API

```rust
// crates/archflow-web/src/bridge.rs

#[wasm_bindgen]
pub fn push_input_event(
    event_type: u8,
    x: f32,
    y: f32,
    buttons: u8,
    modifiers: u8,
) -> Result<(), JsValue> {
    let event = match event_type {
        0 => InputEvent::MouseDown { button: buttons },
        1 => InputEvent::MouseMove { x, y },
        2 => InputEvent::MouseUp { button: buttons },
        3 => InputEvent::MouseWheel { delta: modifiers as i16 },  // modifiers carry delta
        4 => InputEvent::KeyDown { keycode: x as u8 },
        5 => InputEvent::KeyUp { keycode: x as u8 },
        _ => return Err(JsError::new("Invalid event type").into()),
    };
    
    INPUT_PROCESSOR.with(|processor| {
        processor.borrow_mut().buffer.push_event(event);
    });
    
    Ok(())
}
```

---

## 9. Command Pattern para Undo/Redo

### 9.1 Command Trait

```rust
// crates/archflow-engine/src/command.rs

pub trait Command: Send {
    fn execute(&self, store: &mut EntityStore);
    fn undo(&self, store: &mut EntityStore);
    fn is_undoable(&self) -> bool;
}
```

### 9.2 Command History

```rust
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_depth: usize,
}

impl CommandHistory {
    pub fn execute(&mut self, cmd: Box<dyn Command>, store: &mut EntityStore) {
        cmd.execute(store);
        
        if cmd.is_undoable() {
            self.undo_stack.push(cmd);
            
            // Limpiar redo stack si hacemos nueva acción
            self.redo_stack.clear();
            
            // Limitar profundidad
            while self.undo_stack.len() > self.max_depth {
                self.undo_stack.remove(0);
            }
        }
    }
    
    pub fn undo(&mut self, store: &mut EntityStore) -> Result<(), CommandError> {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.undo(store);
            self.redo_stack.push(cmd);
            Ok(())
        } else {
            Err(CommandError::NothingToUndo)
        }
    }
    
    pub fn redo(&mut self, store: &mut EntityStore) -> Result<(), CommandError> {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.execute(store);  // Redo es re-ejecutar
            self.undo_stack.push(cmd);
            Ok(())
        } else {
            Err(CommandError::NothingToRedo)
        }
    }
}
```

---

## 10. Estado de Implementación

### 10.1 Sensores

| Sensor | Archivo | Estado |
|--------|---------|--------|
| `MouseOverSensor` | `sensors/mouse_over.rs` | ✅ Implementado |
| `MouseClickSensor` | `sensors/mouse_click.rs` | ✅ Implementado |
| `TouchSensor` | `sensors/touch.rs` | ✅ Implementado |
| `ProximitySensor` | `sensors/proximity.rs` | ✅ Implementado |
| `RadarSensor` | `sensors/radar.rs` | ✅ Implementado |
| `KeyShortcutSensor` | `sensors/key_shortcut.rs` | ✅ Implementado |
| `DoubleTapSensor` | `sensors/double_tap.rs` | ✅ Implementado |
| `LongPressSensor` | `sensors/long_press.rs` | ✅ Implementado |
| `RightClickSensor` | `sensors/right_click.rs` | ✅ Implementado |

### 10.2 Controllers

| Controller | Estado |
|------------|--------|
| `Direct` | ✅ Implementado (Rust + WASM) |
| `And` | ✅ Implementado (Rust + WASM) |
| `Or` | ✅ Implementado (Rust + WASM) |
| `Not` | ✅ Implementado (Rust + WASM) |
| `Blinky` | ✅ Implementado (Rust + WASM) |
| `Debounce` | ✅ Implementado (Rust + WASM) |
| `Hysteresis` | ✅ Implementado (Rust + WASM) |
| `Threshold` | ✅ Implementado (Rust + WASM) |
| `Pattern` | ✅ Implementado (Rust + WASM) |
| `Custom` | ✅ Implementado (Rust + WASM + JS Registry) |

### 10.3 Actuators

| Actuator | Estado |
|----------|--------|
| `HighlightActuator` | ✅ Implementado |
| `SelectActuator` | ✅ Implementado |
| `MoveActuator` | ✅ Implementado |
| `PropertyActuator` | ✅ Implementado |
| `CameraActuator` | ✅ Implementado |
| `StateActuator` | ✅ Implementado |
| `MessageActuator` | ✅ Implementado |

### 10.4 WASM Bindings

| Componente | Archivo | Estado |
|------------|---------|--------|
| `SignalByteWasm` | `archflow-web/src/logic/signal_byte.rs` | ✅ Implementado |
| `SensorType` | `archflow-web/src/logic/sensor_type.rs` | ✅ Implementado (9 sensores) |
| `Controller` | `archflow-web/src/logic/controller.rs` | ✅ Implementado (10 tipos) |
| `LogicMappingTableWasm` | `archflow-web/src/logic/mapping_table.rs` | ✅ Implementado |

### 10.5 TypeScript SDK

| Componente | Archivo | Estado |
|------------|---------|--------|
| `LogicSDK` | `archflow-web-ui/src/sdk/logic-sdk.ts` | ✅ Implementado |
| `EntityBuilder` | `archflow-web-ui/src/sdk/logic-sdk.ts` | ✅ Implementado |
| `CustomControllerRegistry` | `archflow-web-ui/src/sdk/logic-sdk.ts` | ✅ Implementado |
| `Examples` | `archflow-web-ui/docs/LOGIC-SDK-EXAMPLES.md` | ✅ Documentado |

---

## 11. Plan de Implementación

### Fase 1: Controllers Predefinidos (Semana 1)
- [ ] Implementar `Controller.Blinky`
- [ ] Implementar `Controller.Debounce`
- [ ] Implementar `Controller.Hysteresis`
- [ ] Implementar `Controller.Threshold`
- [ ] Implementar `Controller.Pattern`
- [ ] Tests unitarios

### Fase 2: Custom Controllers (Semana 2)

**Cambio de arquitectura:** El navegador YA tiene JavaScript. No necesitamos embeber rquickjs.

- [ ] Diseñar API de `JSSignalProxy` y `JSContextProxy`
- [ ] Implementar `CustomControllerRegistry` en TypeScript
- [ ] Implementar bridge `evaluateCustomController()` via wasm-bindgen
- [ ] Timeout de 50ms en JS para evitar loops infinitos
- [ ] API TypeScript wrapper con tipos seguros
- [ ] Tests de seguridad (timeout, código malicioso)
- [ ] Documentación de ejemplos de código JS

### Fase 3: Actuators Faltantes (Semana 3)
- [ ] `PropertyActuator`
- [ ] `CameraActuator`
- [ ] `StateActuator`
- [ ] `MessageActuator`
- [ ] Integración completa

### Fase 4: API Developer (Semana 4)
- [ ] Wrapper React `useLogicBricks()`
- [ ] Componente visual para crear conexiones
- [ ] Templates predefinidos
- [ ] Documentación y ejemplos

---

## 12. Referencias

- Blender Game Engine Logic Bricks: https://docs.blender.org/manual/en/game_engine/logic/
- Signal Processing with Bitwise Operations: https://en.wikipedia.org/wiki/Bitwise_operation
- Hysteresis in User Interfaces: https://www.nngroup.com/articles/hysteresis/

---

*Documento creado: 2026-02-02*  
*Basado en análisis de BGE, arquitectura SoA, y requirements de EPIC-WEB-010*
