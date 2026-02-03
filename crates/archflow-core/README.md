# ArchFlow Core

**ArchFlow Core** is the foundational crate providing the shared kernel for the ArchFlow platform. It defines the core domain concepts, value objects, and port interfaces that power the entire system while remaining `no_std` compatible for WASM compilation.

## Overview

ArchFlow Core serves as the **immutable shared kernel** in Hexagonal Architecture, containing:

- **Domain primitives**: Entity identification, 2D geometry, transformations
- **Value objects**: Position, Size, Bounds with validation
- **Vector paths**: Bézier curves and shape primitives
- **Animation system**: Easing functions, timelines, and sequencing
- **Zoom management**: Multi-level viewport with progressive disclosure
- **Port interfaces**: Hexagonal architecture boundaries

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ArchFlow Core                             │
│                      (Shared Kernel - no_std)                    │
├─────────────────────────────────────────────────────────────────────┤
│  Domain Layer          │  Application Layer    │  Infrastructure    │
│                       │  (Port Interfaces)    │  (Adapters)        │
├─────────────────────────────────────────────────────────────────────┤
│  • EntityId           │  • EntityStorePort     │  • Implemented     │
│  • Vec2, Rect         │  • CanvasPort           │    by adapters    │
│  • Transform          │  • EventPublisher       │  • Renderers       │
│  • Paths              │  • CommandExecutor      │  • Event buses     │
│  • Animation          │  • AssetLoader          │                    │
│  • Zoom               │                         │                    │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Domain Concepts

### Entity Identification

The entity system uses **generational indexing** for safe, efficient references:

```rust
pub struct EntityId {
    index: Index,        // Dense array slot (24 bits)
    generation: Generation // Version counter (8 bits)
}
```

**Key properties:**
- **Max entities**: 100,000 (configurable)
- **Memory**: 4 bytes per ID
- **Safety**: Generation tracking prevents use-after-free
- **Performance**: O(1) access with dense arrays

**Example:**
```rust
let id = EntityId::new(42);      // Creates entity at slot 42
let index = id.index();          // Get slot number
let generation = id.generation(); // Get version
let raw = id.as_u32();           // Serialize to u32
```

### 2D Geometry

The geometry module provides `no_std`-compatible 2D math primitives:

#### Vec2
```rust
pub use glam::Vec2; // Re-exported from glam library

let v = Vec2::new(10.0, 20.0);
let magnitude = v.length();
let normalized = v.normalize();
```

#### Rectangle (Rect)
```rust
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    // Spatial queries
    pub fn contains(&self, point: Vec2) -> bool;
    pub fn intersects(&self, other: Rect) -> bool;
    
    // Geometric operations
    pub fn union(&self, other: Rect) -> Rect;
    pub fn closest_point(&self, point: Vec2) -> Vec2;
    
    // Properties
    pub fn center(&self) -> Vec2;
    pub fn size(&self) -> Vec2;
    pub fn bounds(&self) -> Bounds;
}
```

#### Transform
```rust
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,    // Degrees clockwise
    pub scale: f32,       // Uniform scale factor
}
```

### Value Objects

Value objects provide validation and type safety:

#### Position
```rust
pub struct Position(pub Vec2);

impl Position {
    pub fn new(x: f32, y: f32) -> Self;
    pub fn from_vec2(v: Vec2) -> Self;
}
```

#### Size
```rust
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Result<Self, DomainError>;
    
    // Validation: size must be positive
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}
```

#### Bounds
```rust
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub fn from_rect(rect: Rect) -> Self;
    pub fn to_rect(&self) -> Rect;
    pub fn contains(&self, point: Vec2) -> bool;
}
```

### Vector Paths

The paths module provides Bézier curve primitives for 2D shapes:

```rust
pub enum PathCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo { control: Vec2, end: Vec2 },
    CubicTo { control1: Vec2, control2: Vec2, end: Vec2 },
    Close,
}
```

**Circle approximation algorithm:**
```rust
// Approximates a circle using 4 cubic Bézier curves
// Magic constant: k = 4/3 * tan(π/8) ≈ 0.5522848
pub fn ellipse(center: Vec2, radii: Vec2) -> Vec<PathCommand> {
    let k = 0.5522848_f32;
    // Generate 4 cubic curves...
}
```

**Key operations:**
- **Bounding box**: O(n) where n = path commands
- **Approximation**: Circles use 4 curves, ellipses use 8 curves
- **Validation**: Self-intersection detection available

### Animation System

The animation system provides **75+ easing functions** organized by category:

#### Easing Function Categories

| Category | Functions | Description |
|----------|-----------|-------------|
| **Polynomial** | Linear, Quad, Cubic, Quart, Quint | Power-based interpolation |
| **Trigonometric** | Sine, Cosine, Tangent | Smooth oscillating motion |
| **Exponential** | Expo In/Out | Rapid start/end motion |
| **Circular** | Circ In/Out | Arc-based interpolation |
| **Elastic** | Elastic Out/In | Spring-like overshoot |
| **Bounce** | Bounce In/Out/Out | Bouncing ball effect |
| **Back** | Back In/Out | Overshoot and return |
| **Bezier** | Custom Bezier | User-defined curves |

**Usage:**
```rust
use archflow_core::animation::*;

let progress = 0.5; // 0.0 to 1.0

// Apply easing
let eased = EasingFunction::BounceOut.apply(progress);

// Or use directly
let eased = bounce_out(progress);
```

#### Timeline Management

```rust
pub struct Timeline {
    pub duration: Ticks,
    pub delay: Ticks,
}

impl Timeline {
    pub fn new(duration_ms: u64) -> Self;
    pub fn with_delay(self, delay_ms: u64) -> Self;
    pub fn total_duration(&self) -> Ticks;
}
```

#### Stagger Animation

```rust
pub struct Stagger {
    pub rows: usize,
    pub cols: usize,
    pub delay: Ticks,
}

impl Stagger {
    // Grid-based delay calculation
    pub fn delay_for(&self, index: usize) -> Ticks {
        let row = index / self.cols;
        let col = index % self.cols;
        (row * self.cols + col) as Ticks * self.delay
    }
}
```

### Zoom System

The zoom system enables **progressive disclosure** across different detail levels:

#### Zoom Levels

```rust
pub enum ZoomLevel {
    System,     // 0-100px: High-level architecture
    Container,  // 100-500px: Component containers
    Component,  // 500-1000px: Individual components
    Code,       // 1000px+: Code/detail view
}
```

**Pixel ranges and typical content:**
- **System (0-100px)**: Context boundaries, external systems
- **Container (100-500px)**: Applications, databases, services
- **Component (500-1000px)**: Modules, classes, major functions
- **Code (1000px+)**: Implementation details, code snippets

#### Zoom Configuration

```rust
pub struct ZoomConfig {
    pub level: ZoomLevel,
    pub scale: f32,
    pub min_scale: f32,
    pub max_scale: f32,
}

impl ZoomConfig {
    pub fn detect_level(&self) -> ZoomLevel;
    pub fn can_zoom_in(&self) -> bool;
    pub fn can_zoom_out(&self) -> bool;
}
```

#### Visibility Rules

```rust
pub struct VisibilityRule {
    pub entity_id: EntityId,
    pub min_level: ZoomLevel,
    pub max_level: ZoomLevel,
    pub style_overrides: Vec<StyleOverride>,
}
```

## Port Interfaces

ArchFlow Core defines the **hexagonal architecture** boundaries:

### Primary Ports (Domain → Infrastructure)

```rust
/// Entity lifecycle management
pub trait EntityStorePort {
    fn spawn(&mut self, transform: Transform) -> EntityId;
    fn despawn(&mut self, id: EntityId) -> Result<(), StoreError>;
    fn get(&self, id: EntityId) -> Option<&Entity>;
    fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity>;
}

/// Canvas rendering operations
pub trait CanvasPort {
    fn draw_rect(&mut self, rect: Rect, color: u32);
    fn draw_path(&mut self, commands: &[PathCommand], stroke: &StrokeConfig);
    fn clear(&mut self, color: u32);
}

/// Domain event publishing
pub trait EventPublisher {
    fn publish(&mut self, event: DomainEvent);
}

/// Undoable command execution
pub trait CommandExecutor {
    fn execute(&mut self, command: Command) -> Result<(), CommandError>;
    fn undo(&mut self) -> Result<(), CommandError>;
    fn redo(&mut self) -> Result<(), CommandError>;
}
```

### Secondary Ports (Infrastructure → Domain)

```rust
/// Event handling
pub trait EventHandler {
    fn on_event(&mut self, event: &DomainEvent);
}
```

## Module Organization

```
archflow-core/
├── lib.rs                    # Public API and re-exports
├── id.rs                     # Entity identification (EntityId)
├── math.rs                   # 2D math primitives
├── vo/                       # Value objects
│   ├── position.rs           # Position value object
│   ├── size.rs               # Size value object
│   └── bounds.rs             # Bounds value object
├── paths.rs                  # Vector paths with Bézier curves
├── animation/                # Animation system
│   ├── easing.rs             # 75+ easing functions
│   ├── timeline.rs           # Timeline management
│   └── stagger.rs            # Grid-based staggering
├── zoom.rs                   # Zoom and viewport management
├── ports.rs                  # Hexagonal architecture ports
├── api.rs                    # High-level API surface
├── transform_enhanced.rs     # Enhanced transform utilities
└── resources.rs              # Resource management (std only)
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Entity creation | O(1) | Direct slot allocation |
| Entity lookup | O(1) | Array indexing |
| Rectangle intersection | O(1) | AABB overlap test |
| Point in rectangle | O(1) | Component-wise compare |
| Path bounding box | O(n) | n = path commands |
| Easing function | O(1) | Mathematical evaluation |
| Stagger delay | O(1) | Grid distance |
| Zoom level detection | O(1) | Scale comparison |

## Design Principles

### No_std Compatibility

All core types are designed for `no_std` environments:

- **Copy types**: All value objects implement `Copy`
- **No allocation**: Core operations avoid heap allocation
- **WASM-ready**: Compiles directly to WebAssembly

### Type Safety

- **Value objects**: Encapsulate validation logic
- **Phantom types**: Prevent invalid states at compile time
- **Newtype patterns**: Prevent type confusion

### Performance

- **Cache-friendly**: Structure of Arrays in data layer
- **Inline-heavy**: Small functions optimized for inlining
- **Zero-cost abstractions**: Ports compile to vtable calls

## Examples

### Creating and Using Entities

```rust
use archflow_core::{EntityId, Position, Size, Transform};

// Create an entity (through port)
let id = store.spawn(Transform {
    position: Position::new(100.0, 200.0).0,
    rotation: 0.0,
    scale: 1.0,
});

// Access entity data
if let Some(entity) = store.get(id) {
    println!("Entity at: {:?}", entity.transform.position);
}
```

### Working with Geometry

```rust
use archflow_core::{Rect, Vec2};

let rect = Rect {
    x: 10.0,
    y: 10.0,
    width: 100.0,
    height: 50.0,
};

let point = Vec2::new(50.0, 30.0);

if rect.contains(point) {
    println!("Point is inside rectangle");
}

let union = rect.union(other_rect);
```

### Animation Easing

```rust
use archflow_core::animation::*;

let duration = Timeline::new(1000); // 1 second
let start = 0.0;
let end = 1.0;

for t in 0..=100 {
    let progress = t as f32 / 100.0;
    let eased = EasingFunction::BounceOut.apply(progress);
    let value = start + (end - start) * eased;
    // Use value...
}
```

### Zoom Management

```rust
use archflow_core::zoom::*;

let config = ZoomConfig {
    level: ZoomLevel::Component,
    scale: 1.0,
    min_scale: 0.1,
    max_scale: 10.0,
};

if config.can_zoom_in() {
    config.scale *= 1.2;
    let new_level = config.detect_level();
}
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | Enable standard library integration | No |
| `serde` | Enable serialization support | No |
| `animation` | Enable animation system | Yes |
| `paths` | Enable vector paths | Yes |

## Dependencies

- **glam**: High-performance math library (required)
- **serde**: Serialization support (optional)
- **thiserror**: Error handling (optional, std feature)

## Integration Points

ArchFlow Core integrates with:

| Crate | Integration Type |
|-------|------------------|
| `archflow-engine` | Implements port interfaces |
| `archflow-logic` | Uses core types in Logic Bricks |
| `archflow-render` | Implements CanvasPort |
| `archflow-diagram` | Uses geometry types |

## License

MIT OR Apache-2.0
