# Épica: SDK Public API - Developer Experience

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| ID | EPIC-SDK-API |
| Prioridad | CRÍTICA |
| Estimación | L |
| Estado | Borrador |
| Versión | 0.1.0 |
| Fecha creación | 2026-02-01 |

---

## 🎯 Objetivo de Negocio

Definir y documentar la **API pública que verán los desarrolladores** que usan el SDK de ArchFlow. Esta no es una épica de implementación, sino de **diseño de API** y **documentación de extensión**.

**Problema que resuelve**: Las épicas existentes describen implementación interna, pero no definen cómo los desarrolladores externos usarán el SDK. Sin una API pública clara, el SDK será inutilizable.

---

## 🏗️ Principios de Diseño de API

### 1. Ergonomía Rust-first
```rust
// ✅ BIEN: Idiomático Rust
let entity = store.spawn(pos, size);
store.move_by(entity_id, delta);

// ❌ MAL: No idiomático
let entity_id = EntityStore::spawn_entity(&self, pos, size);
EntityStore::move_entity_by_delta(&self, entity_id, delta);
```

### 2. Zero-Cost Abstractions
```rust
// ✅ BIEN: Generics monomorfizan a código nativo
pub trait Actuator {
    fn activate(&mut self, pulse: &Pulse, store: &mut EntityStore);
}

// ❌ MAL: Dynamic dispatch overhead
pub trait Actuator {
    fn activate(&mut self, pulse: &Pulse, store: &mut EntityStore);
}
// Box<dyn Actuator> → vtable lookup en cada llamada
```

### 3. Builder Patterns para Configuración Compleja
```rust
// ✅ BIEN: Builder pattern con defaults
let sensor = MouseSensor::builder()
    .mode(MouseMode::LeftButton)
    .tap(true)
    .invert(false)
    .build();

// ❌ MAL: Constructor con muchos parámetros
let sensor = MouseSensor::new(MouseMode::LeftButton, true, false, false, 0, 0);
```

---

## 📖 Secciones de API Pública

### 1. API de Sensores para Desarrolladores

Los desarrolladores del SDK deben poder crear **sensores custom**:

```rust
/// Trait que deben implementar todos los sensores custom
pub trait Sensor {
    /// Evalúa el sensor y retorna un estado (Positive/Negative/None)
    fn evaluate(&mut self, ctx: &SensorContext) -> SensorState;
    
    /// Retorna la configuración del sensor (para debugging/UI)
    fn config(&self) -> &SensorConfig;
}

/// Contexto proporcionado durante evaluación de sensor
pub struct SensorContext<'a> {
    /// Referencia al EntityStore (read-only)
    pub store: &'a EntityStore,
    
    /// Input snapshot actual (mouse, teclado, etc.)
    pub input: &'a InputSnapshot,
    
    /// Timestamp actual
    pub timestamp: u32,
}

/// Estado de un sensor después de evaluación
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SensorState {
    /// Sensor activado (detectó condición)
    Positive,
    
    /// Sensor desactivado (no detectó condición)
    Negative,
    
    /// Sensor no tiene estado relevante
    None,
}
```

**Ejemplo de uso para desarrollador:**

```rust
// Sensor custom: Detecta cuando el mouse está cerca del centro
struct ProximitySensor {
    entity_id: EntityId,
    threshold: f32,
}

impl Sensor for ProximitySensor {
    fn evaluate(&mut self, ctx: &SensorContext) -> SensorState {
        let entity_pos = ctx.store.position(self.entity_id);
        let mouse_pos = ctx.input.mouse_position;
        
        let distance = (entity_pos - mouse_pos).length();
        
        if distance < self.threshold {
            SensorState::Positive
        } else {
            SensorState::Negative
        }
    }
    
    fn config(&self) -> &SensorConfig {
        &self.config
    }
}
```

---

### 2. API de Actuadores para Desarrolladores

Los desarrolladores deben poder crear **actuadores custom**:

```rust
/// Trait que deben implementar todos los actuadores custom
pub trait Actuator {
    /// Activa el actuador en respuesta a un pulso
    fn activate(&mut self, pulse: &Pulse, store: &mut EntityStore);
    
    /// Retorna la configuración del actuador
    fn config(&self) -> &ActuatorConfig;
}

/// Pulso emitido por un sensor
#[derive(Clone, Copy, Debug)]
pub struct Pulse {
    /// ID del sensor que generó el pulso
    pub sensor_id: u32,
    
    /// ID de la entidad asociada al sensor
    pub entity_id: EntityId,
    
    /// Estado del sensor (Positive/Negative)
    pub state: SensorState,
    
    /// Timestamp del pulso
    pub timestamp: u32,
}
```

**Ejemplo de uso para desarrollador:**

```rust
// Actuator custom: Resalta la entidad cuando recibe un pulso Positive
struct HighlightActuator {
    highlight_color: u32, // 0xRRGGBBAA
}

impl Actuator for HighlightActuator {
    fn activate(&mut self, pulse: &Pulse, store: &mut EntityStore) {
        if pulse.state == SensorState::Positive {
            store.set_color(pulse.entity_id, self.highlight_color);
        } else {
            store.set_color(pulse.entity_id, 0xFFFFFFFF); // Reset a blanco
        }
    }
    
    fn config(&self) -> &ActuatorConfig {
        &self.config
    }
}
```

---

### 3. API de Wiring/Configuración

Los desarrolladores necesitan conectar sensores con actuadores:

```rust
/// Builder para configurar conexiones entre sensores y actuadores
pub struct WiringBuilder {
    connections: Vec<Connection>,
}

struct Connection {
    sensor_id: u32,
    actuator_id: u32,
    /// Solo activar si el sensor está en estas entidades
    entity_filter: Option<EntityFilter>,
    /// Solo activar si el estado del sensor es...
    state_filter: Option<SensorState>,
}

impl WiringBuilder {
    pub fn new() -> Self {
        Self { connections: Vec::new() }
    }
    
    /// Conecta un sensor a un actuador
    pub fn connect(mut self, sensor_id: u32, actuator_id: u32) -> Self {
        self.connections.push(Connection {
            sensor_id,
            actuator_id,
            entity_filter: None,
            state_filter: None,
        });
        self
    }
    
    /// Filtra por entidades con un tag específico
    pub fn on_entities_with_tag(mut self, tag: &str) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.entity_filter = Some(EntityFilter::Tag(tag.to_string()));
        }
        self
    }
    
    /// Filtra por estado Positive
    pub fn on_positive(mut self) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.state_filter = Some(SensorState::Positive);
        }
        self
    }
    
    /// Construye la tabla de wiring
    pub fn build(self) -> WiringTable {
        WiringTable { connections: self.connections }
    }
}
```

**Ejemplo de uso para desarrollador:**

```rust
// Configurar: Click en botón → Resaltar botón
let wiring = WiringBuilder::new()
    .connect(mouse_click_sensor, highlight_actuator)
    .on_entities_with_tag("button")
    .on_positive()
    .build();
```

---

### 4. API de Extensión

Los desarrolladores necesitan extender el SDK con nuevas funcionalidades:

#### Añadir nuevos tipos de Command

```rust
/// Los Commands son Copy y ≤16 bytes
#[derive(Clone, Copy, Debug)]
pub enum Command {
    // Comandos core
    Spawn { pos: Vec2, size: Vec2, parent: Option<EntityId> } = 0,
    Despawn(EntityId) = 1,
    Move { id: EntityId, delta: Vec2 } = 2,
    
    // Espacio para comandos custom: 128-255
    Custom(u8, [u8; 12]), // tipo + payload
}

// Ejemplo: Añadir comando "Rotate"
impl Command {
    pub fn rotate(id: EntityId, angle_degrees: f32) -> Self {
        let payload = angle_degrees.to_le_bytes();
        Command::Custom(128, [
            payload[0], payload[1], payload[2], payload[3],
            0, 0, 0, 0, 0, 0, 0, 0
        ])
    }
}
```

#### Añadir nuevos ShapeTypes

```rust
/// Tipos de forma soportados (4 bits en metadata)
#[repr(u8)]
pub enum ShapeType {
    Rectangle = 0,
    Circle = 1,
    Text = 2,
    Line = 3,
    
    // Espacio para shapes custom: 8-15
    CustomShape(u8),
}

// Ejemplo: Añadir shape "Triangle"
impl ShapeType {
    pub const TRIANGLE: u8 = 8;
}
```

#### Integrar Rendering Custom

```rust
/// Callback de renderizado custom
pub type RenderCallback = fn(ctx: &mut RenderContext, entity: EntityId);

/// Registro de renderers custom
pub struct CustomRenderers {
    renderers: HashMap<ShapeType, RenderCallback>,
}

impl CustomRenderers {
    pub fn register(&mut self, shape_type: ShapeType, callback: RenderCallback) {
        self.renderers.insert(shape_type, callback);
    }
}

// Ejemplo: Registrar renderer para triángulos
fn render_triangle(ctx: &mut RenderContext, entity: EntityId) {
    let pos = ctx.store.position(entity);
    let size = ctx.store.size(entity);
    // ... código de renderizado de triángulo
}

let mut custom_renderers = CustomRenderers::new();
custom_renderers.register(ShapeType::TRIANGLE, render_triangle);
```

---

### 5. API de Snap System

Para un SDK tipo Figma/tldraw, el snap system es crítico:

```rust
/// Configuración del sistema de snapping
pub struct SnapConfig {
    /// Tamaño del grid (0 = desactivado)
    pub grid_size: f32,
    
    /// Distancia para activar snap (0 = 50% de grid_size)
    pub threshold: f32,
    
    /// Snap a bordes de entidades
    pub snap_to_edges: bool,
    
    /// Snap a centros de entidades
    pub snap_to_centers: bool,
    
    /// Snap a guías personalizadas
    pub snap_to_guides: bool,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            grid_size: 16.0,
            threshold: 8.0,
            snap_to_edges: true,
            snap_to_centers: true,
            snap_to_guides: true,
        }
    }
}

/// Sistema de snapping
pub struct Snapper {
    config: SnapConfig,
    spatial_hash: SpatialHashGrid,
    guides: Vec<Guide>,
}

impl Snapper {
    pub fn new(config: SnapConfig, spatial_hash: SpatialHashGrid) -> Self {
        Self { config, spatial_hash, guides: Vec::new() }
    }
    
    /// Snap una posición al grid
    pub fn snap_to_grid(&self, pos: Vec2) -> Vec2 {
        if self.config.grid_size == 0.0 {
            return pos;
        }
        
        let snapped_x = (pos.x / self.config.grid_size).round() * self.config.grid_size;
        let snapped_y = (pos.y / self.config.grid_size).round() * self.config.grid_size;
        
        Vec2::new(snapped_x, snapped_y)
    }
    
    /// Snap una posición a entidades cercanas
    pub fn snap_to_entities(&self, pos: Vec2, store: &EntityStore) -> Vec2 {
        if !self.config.snap_to_edges {
            return pos;
        }
        
        // Usar SpatialHash para encontrar entidades cercanas
        let nearby = self.spatial_hash.query_circle(pos, self.config.threshold);
        
        // Encontrar el edge más cercano
        for entity_id in nearby {
            let entity_pos = store.position(entity_id);
            let entity_size = store.size(entity_id);
            
            // Check left edge
            if (pos.x - entity_pos.x).abs() < self.config.threshold {
                return Vec2::new(entity_pos.x, pos.y);
            }
            
            // Check right edge
            if (pos.x - (entity_pos.x + entity_size.x)).abs() < self.config.threshold {
                return Vec2::new(entity_pos.x + entity_size.x, pos.y);
            }
            
            // ... (similar para top/bottom edges)
        }
        
        pos
    }
}
```

**Ejemplo de uso para desarrollador:**

```rust
let snapper = Snapper::new(
    SnapConfig {
        grid_size: 16.0,
        threshold: 8.0,
        ..Default::default()
    },
    spatial_hash,
);

let raw_pos = Vec2::new(123.4, 567.8);
let snapped_pos = snapper.snap_to_grid(raw_pos);
// Result: Vec2(128.0, 576.0) - alineado al grid de 16px
```

---

## 🔧 Feature Flags para SDK Modular

El SDK debe ser modular. No todos los desarrolladores necesitan todas las features:

```toml
[features]
default = ["sensors-basic", "actuators-basic"]

# Input
sab-input = ["web-sys/SharedArrayBuffer"]  # SAB o fallback
simd-input = []  # SIMD processing de input

# Sensores
sensors-basic = []     # Mouse, Keyboard
sensors-physics = []   # Proximity, Collision

# Actuadores  
actuators-basic = []   # Move, Highlight
actuators-tween = []   # Animaciones interpoladas

# Networking
networking = ["tokio", "tungstenite"]
networking-crdt = ["networking"]

# Esto permite:
# - SDK mínimo para apps simples
# - SDK completo para Figma-like
```

**Ejemplos de uso:**

```toml
# App simple: solo input básico
archflow = { version = "0.1", features = ["sensors-basic"] }

# App completa: todas las features
archflow = { version = "0.1", features = ["default"] }

# App con colaboración en red
archflow = { version = "0.1", features = ["default", "networking"] }

# App sin SIMD (compatibilidad máxima)
archflow = { version = "0.1", default-features = false, features = ["sensors-basic"] }
```

---

## 📊 Criterios de Aceptación

### Documentación
- [ ] Guía "Getting Started" para desarrolladores del SDK
- [ ] Referencia completa de API pública con rustdoc
- [ ] Ejemplos de código para cada trait público
- [ ] Guía de extensión (añadir sensores/actuadores custom)
- [ ] Guía de feature flags (qué incluir para cada caso de uso)

### API Design
- [ ] Traits `Sensor` y `Actuator` con ejemplos de implementación
- [ ] `WiringBuilder` para configuración declarativa
- [ ] `Snapper` API simple e idiomática
- [ ] Sistema de extensiones (Commands, Shapes, Renderers)

### Testing
- [ ] Tests de ejemplo que demuestran uso de API pública
- [ ] Ejemplos compilables que pueden servir como templates
- [ ] Tests de integración para extensiones custom

---

## 📝 Entregables

1. **Documentación en `docs/sdk/`**
   - `getting-started.md`
   - `api-reference.md`
   - `extension-guide.md`
   - `feature-flags.md`

2. **Ejemplos en `examples/`**
   - `custom_sensor.rs` - Ejemplo de sensor custom
   - `custom_actuator.rs` - Ejemplo de actuador custom
   - `wiring.rs` - Ejemplo de configuración
   - `snap_system.rs` - Ejemplo de snapping

3. **Código de infraestructura**
   - Traits públicos en `src/sdk/`
   - Re-exports en `lib.rs` para API pública
   - Feature flags en `Cargo.toml`

---

## 🎯 Estimación y Timeline

| Fase | Duración | Entregables |
|------|----------|-------------|
| Diseño de API | 3 días | Traits `Sensor`, `Actuator`, `WiringBuilder` |
| Implementación Snapper | 2 días | `Snapper` con grid/entity snapping |
| Documentación | 3 días | Guías completas con ejemplos |
| Examples | 2 días | 4+ ejemplos compilables |

**Total: 2 semanas (10 días laborables)**

---

`★ Insight ─────────────────────────────────────`
**API Pública es el Producto**

Para un SDK, **la API pública es el producto**. Los desarrolladores no interactúan con la implementación interna, solo con la API pública.

1. **Ergonomía > Optimización**: Es mejor 1ms más lento pero fácil de usar, que ultra-optimizado pero difícil.
2. **Traits para extensión**: Permitir que los desarrolladores extiendan el SDK sin modificarlo.
3. **Feature flags**: Un desarrollador de apps simples no debería pagar el costo de features que no usa.
4. **Ejemplos > Documentación**: Un ejemplo funcional vale más que 100 páginas de docs.
`─────────────────────────────────────────────────`

---

**Fin de Épica EPIC-SDK-API: SDK Public API**
