# archflow-logic

> **Logic Bricks System** - Event-driven reactive architecture inspired by Blender's Game Engine, optimized for Rust and WASM with memory-efficient sensor-actuator patterns.

## Overview

`archflow-logic` implements a **Logic Bricks system**—a reactive, event-driven architecture where sensors detect input events and emit pulses that flow through controllers to actuators. This system provides a declarative way to define interactive behaviors without writing imperative code.

**Key Capabilities:**
- **Sensor-Actuator Pattern** - Declarative behavior composition
- **Event-Driven Architecture** - Pulse-based signal propagation
- **Memory-Efficient** - Bit-packed SignalByte (1 byte per entity per sensor)
- **WASM-Optimized** - Zero-allocation patterns and minimal memory footprint
- **Extensible** - Plugin-style sensors, controllers, and actuators

## Architecture

The Logic Bricks system follows a reactive flow pattern:

```
┌─────────────────────────────────────────────────────────────────┐
│                         SENSORS                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Mouse Sensors│  │Key Sensors   │  │Spatial Sensors│         │
│  │ (Click,Over) │  │(Shortcut)    │  │(Proximity)   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │ Pulse Events
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       CONTROLLERS                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Direct       │  │Boolean Logic │  │Advanced      │          │
│  │ Connection   │  │(AND/OR/NOT)  │  │(Debounce)    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │ Filtered Pulses
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       ACTUATORS                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Select       │  │ Move         │  │ Highlight    │          │
│  │ (Selection)  │  │(Transform)   │  │(Visual)      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │ Commands
                              ▼
                         EntityStore
                      (State Changes)
```

## Core Concepts

### SignalByte - Compact Signal History

The foundation of the Logic Bricks system is the `SignalByte` type, which stores 6 ticks of binary signal history in a single byte:

```rust
pub struct SignalByte(u8);

impl SignalByte {
    // Get current state (most recent bit)
    pub fn is_high(&self) -> bool;
    
    // Edge detection
    pub fn is_rising_edge(&self) -> bool;  // 0→1 transition
    pub fn is_falling_edge(&self) -> bool; // 1→0 transition
    
    // Pattern matching
    pub fn is_steady_high(&self) -> bool;  // 111111
    pub fn is_steady_low(&self) -> bool;   // 000000
    pub fn matches_pattern(&self, pattern: u8) -> bool;
}
```

**Memory Efficiency:**
- **1 byte per entity per sensor**
- 100KB for 100,000 entities (single sensor)
- 400KB for 100,000 entities (4 sensors)

### Sensors - Input Detection

Sensors detect input events and maintain signal history:

```rust
use archflow_logic::sensors::{MouseOverSensor, SensorState};

let mut sensor = MouseOverSensor::new();

// Sample sensor state
let state = sensor.sample(mouse_pos, &entity_store);

match state {
    SensorState::Positive => {
        // Entity is under mouse (rising edge or steady high)
    }
    SensorState::Negative => {
        // Entity not under mouse (falling edge or steady low)
    }
    SensorState::None => {
        // No change from previous state
    }
}
```

**Available Sensors:**

| Category | Sensors | Purpose |
|----------|---------|---------|
| **Mouse** | `MouseOver`, `MouseClick`, `RightClick` | Pointer interaction |
| **Keyboard** | `KeyShortcut`, `LongPress`, `DoubleTap` | Key combinations |
| **Spatial** | `Proximity`, `Collision`, `Radar`, `Near` | Distance-based detection |
| **Touch** | `Touch` | Multi-touch support |

**Sensor Architecture:**
```rust
pub trait Sensor {
    fn sample(&mut self, input: &InputState, store: &EntityStore) -> SensorState;
    fn signals(&self) -> &[SignalByte];
    fn reset(&mut self);
}
```

### Controllers - Logic Flow

Controllers filter and transform sensor pulses:

```rust
use archflow_logic::mapping::controller::{Controller, BlinkyConfig};

// Direct connection (sensor pulse directly triggers actuator)
let controller = Controller::Direct;

// Boolean logic
let controller = Controller::AND(SensorType::MouseOver);
let controller = Controller::OR(SensorType::KeyShortcut);
let controller = Controller::NOT;

// Advanced controllers
let controller = Controller::Blinky { interval: 60 };  // Toggle every 60 ticks
let controller = Controller::Debounce { ticks: 5 };     // Filter noise
let controller = Controller::Hysteresis { high: 0.8, low: 0.2 }; // Prevent oscillation
let controller = Controller::Threshold { percentage: 80 }; // Minimum stability
```

**Controller Types:**

| Type | Description | Use Case |
|------|-------------|----------|
| `Direct` | Pass-through | Simple interactions |
| `AND` | Logical AND | Require multiple conditions |
| `OR` | Logical OR | Alternative triggers |
| `NOT` | Logical NOT | Invert condition |
| `Blinky` | Periodic toggle | Animation loops |
| `Debounce` | Noise filtering | Prevent rapid triggers |
| `Hysteresis` | Threshold lag | Avoid oscillation |
| `Threshold` | Minimum stability | Require sustained state |
| `Pattern` | Binary pattern match | Complex sequences |
| `Custom` | JavaScript sandbox | User-defined logic |

### Actuators - Command Generation

Actuators respond to filtered pulses by generating commands:

```rust
use archflow_logic::actuators::{SelectActuator, MoveActuator, HighlightActuator};

// Select entity on click
let select = SelectActuator::new(SelectionMode::Single);
let commands = select.update(&pulses, &mut entity_store);

// Move entity
let move_act = MoveActuator::new();
let commands = move_act.update(&pulses, &mut entity_store);

// Highlight entity
let highlight = HighlightActuator::new(0xFFFF00);  // Yellow
let commands = highlight.update(&pulses, &mut entity_store);
```

**Available Actuators:**

| Actuator | Purpose | Commands Generated |
|----------|---------|-------------------|
| `Select` | Entity selection | `SetSelected` |
| `Move` | Position changes | `Move` |
| `Highlight` | Visual feedback | `SetColor`, `SetOpacity` |
| `Camera` | Viewport control | `Pan`, `Zoom` |
| `State` | State machines | `SetState` |
| `Property` | Property modification | Various |
| `Message` | Notifications | `ShowMessage` |

### LogicMappingTable - Behavior Registry

The mapping table connects sensors to actuators via controllers:

```rust
use archflow_logic::mapping::{LogicMappingTable, LogicConnection};

let mut table = LogicMappingTable::default();

// Create connection: MouseOver → Direct → Highlight
table.connect(
    SensorType::MouseOver,
    Controller::Direct,
    ActuatorType::Highlight,
);

// Create connection: KeyShortcut(Space) → AND(MouseOver) → Select
table.connect(
    SensorType::KeyShortcut,
    Controller::AND(SensorType::MouseOver),
    ActuatorType::Select,
);

// Apply entity mask (only affect specific entities)
table.set_entity_mask(connection_id, entity_mask);
```

**Connection Structure:**
```rust
pub struct LogicConnection {
    pub sensor_type: SensorType,
    pub controller: Controller,
    pub actuator_type: ActuatorType,
    pub entity_mask: u32,  // Bitmask for entity filtering
}
```

## Pulse Flow

```
Input Event → Sensor.sample() → SensorState
                                   │
                                   ▼
                            SignalByte update
                                   │
                                   ▼
                            Controller.filter()
                                   │
                                   ▼
                            Actuator.update()
                                   │
                                   ▼
                            Commands
                                   │
                                   ▼
                            EntityStore.apply()
```

## Usage Examples

### Basic Hover Effect

```rust
use archflow_logic::{LogicSystem, LogicMappingTable};
use archflow_logic::mapping::{SensorType, Controller, ActuatorType};

let mut logic = LogicSystem::new();
let mut mapping = LogicMappingTable::default();

// Connect MouseOver sensor to Highlight actuator
mapping.connect(
    SensorType::MouseOver,
    Controller::Direct,
    ActuatorType::Highlight,
);

// Update loop
logic.update(&mut entity_store, &mapping, &input_state);
```

### Click to Select with Modifier

```rust
// Select only when Shift + Click
mapping.connect(
    SensorType::MouseClick,
    Controller::AND(SensorType::KeyModifier),  // Shift key
    ActuatorType::Select,
);
```

### Proximity-Based Highlight

```rust
// Highlight when near another entity
mapping.connect(
    SensorType::Proximity { radius: 50.0 },
    Controller::Direct,
    ActuatorType::Highlight,
);
```

### Debounced Button

```rust
// Prevent rapid-fire clicks
mapping.connect(
    SensorType::MouseClick,
    Controller::Debounce { ticks: 10 },  // 10 ticks = ~166ms at 60fps
    ActuatorType::Select,
);
```

### Periodic Animation

```rust
// Blink every 60 ticks (1 second at 60fps)
mapping.connect(
    SensorType::Always,
    Controller::Blinky { interval: 60 },
    ActuatorType::Highlight,
);
```

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Memory per sensor | 1 byte/entity | SignalByte |
| Sensor evaluation | <0.1μs | Per entity |
| Pulse propagation | O(n) | n = active connections |
| Update frequency | 60 Hz | Default frame rate |

**Memory Examples:**
- 1,000 entities × 4 sensors = **4 KB**
- 10,000 entities × 4 sensors = **40 KB**
- 100,000 entities × 4 sensors = **400 KB**

## WASM Optimizations

The crate is optimized for WebAssembly compilation:

### Memory Layout
```rust
#[repr(C)]  // C-compatible layout for cross-language boundaries
pub struct LogicConnection {
    sensor_type: SensorType,
    controller: Controller,
    actuator_type: ActuatorType,
    entity_mask: u32,
}
```

### Zero-Allocation Patterns
```rust
// Pre-allocated command buffers
let mut commands = Vec::with_capacity(64);

// Reuse vectors across updates
commands.clear();
// ... populate commands ...
```

### SIMD-Friendly Access
```rust
// Sequential memory access for signal history
for signal in sensor.signals() {
    // Vectorizable operations
}
```

## Extensibility

### Custom Sensor

```rust
use archflow_logic::sensors::{Sensor, SensorState};

pub struct CustomSensor {
    signals: Vec<SignalByte>,
}

impl Sensor for CustomSensor {
    fn sample(&mut self, input: &InputState, store: &EntityStore) -> SensorState {
        // Custom detection logic
        SensorState::Positive
    }
    
    fn signals(&self) -> &[SignalByte] {
        &self.signals
    }
    
    fn reset(&mut self) {
        for signal in &mut self.signals {
            signal.0 = 0;
        }
    }
}
```

### Custom Actuator

```rust
use archflow_logic::actuators::Actuator;

pub struct CustomActuator {
    // State
}

impl Actuator for CustomActuator {
    fn update(&mut self, pulses: &[Pulse], store: &mut EntityStore) -> Vec<Command> {
        let mut commands = Vec::new();
        for pulse in pulses {
            // Generate commands based on pulses
        }
        commands
    }
}
```

## Integration with Other Crates

```toml
[dependencies]
archflow-logic = { version = "0.36", features = ["wasm"] }
archflow-engine = "0.36"
archflow-interaction = "0.36"
```

**Feature Flags:**
- `wasm` - WASM-specific optimizations (default on wasm32)
- `serde` - Serialization support for network play
- `all-sensors` - Include all sensor implementations
- `all-actuators` - Include all actuator implementations

## Design Philosophy

### Why Logic Bricks?

**Traditional Imperative Approach:**
```rust
// Error-prone, hard to maintain
if mouse_over(entity) && mouse_clicked() && shift_pressed() {
    select(entity);
    highlight(entity, YELLOW);
}
```

**Logic Bricks Declarative Approach:**
```rust
// Clear, composable, reusable
mapping.connect(MouseClick, AND(KeyModifier), Select);
mapping.connect(MouseOver, Direct, Highlight);
```

### Benefits

1. **Declarative** - Describe *what* should happen, not *how*
2. **Composable** - Build complex behaviors from simple blocks
3. **Reusable** - Share behavior definitions across entities
4. **Inspectable** - Visual debugging and behavior editing
5. **Network-Friendly** - Behavior definitions are serializable

### Inspiration

The Logic Bricks system is inspired by **Blender's Game Engine (BGE)**, which uses a similar sensor-controller-actuator model for game logic. This crate adapts the concept for:

- **Rust idioms** - Ownership, borrowing, trait-based design
- **WASM compatibility** - No dynamic dispatch, minimal allocations
- **2D interaction** - Optimized for canvas-based tools
- **Collaboration** - CRDT-friendly command generation

## References

- **Blender Game Engine**: Original Logic Bricks concept
- **EPIC-WEB-011**: Behaviors SDK integration
- **archflow-interaction**: Input and event processing
- **archflow-engine**: EntityStore and command execution

## License

MIT License - See LICENSE file for details.
