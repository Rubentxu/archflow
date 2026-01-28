# ArchFlow Animation API - Developer Experience Design

**Date**: 2025-01-28  
**Purpose**: Design a powerful, developer-friendly API inspired by the best JS/TS libraries  
**Focus**: Rust → WASM/JavaScript integration with optimal DX

## Executive Summary

**★ Insight ─────────────────────────────────────**
**Meta: Crear el API de animación más potente y fácil de usar para Rust/WASM**

Inspirado en lo mejor de:
- **Anime.js**: Timeline, staggering, keyframes
- **Motion/Framer Motion**: Spring physics, gestures
- **GSAP**: Positional parameters, composition
- **p5.js**: Method chaining, intuitive naming

**Principios de diseño**:
1. **Fluidez**: Method chaining como jQuery/p5.js
2. **Expresividad**: Named parameters como React props
3. **Composición**: Timeline nesting como GSAP
4. **Type-safe**: Rust types con inferencia
5. **WASM-friendly**: Serde serialization para JS interop
**─────────────────────────────────────────────────**

---

## Part 1: The Vision - API Comparison

### JavaScript Libraries (The Gold Standard)

#### Anime.js - Clean & Intuitive

```javascript
// Simple and expressive
anime({
  targets: '.box',
  translateX: 250,
  scale: [0.5, 1],           // From-to array syntax
  easing: 'easeOutExpo',
  duration: 750,
  delay: anime.stagger(100)  // Built-in stagger
})

// Timeline composition
const tl = anime.timeline({
  easing: 'easeOutExpo',
  duration: 750
})

tl.add({
  targets: '.logo',
  translateX: 250,
  scale: [0.5, 1]
}).add({
  targets: '.title',
  opacity: [0, 1],
  offset: '-=500'           // Relative timing
})
```

#### Framer Motion - React-First

```javascript
// Component-based API
<motion.div
  animate={{ x: 100 }}
  transition={{
    type: "spring",
    stiffness: 100,
    damping: 10
  }}
/>

// Variants for orchestration
const variants = {
  visible: { opacity: 1, scale: 1 },
  hidden: { opacity: 0, scale: 0 }
}

<motion.div
  initial="hidden"
  animate="visible"
  variants={variants}
/>
```

#### GSAP - Professional Power

```javascript
// Positional parameters (duration, ease, delay)
gsap.to(".box", { x: 100 }, 1, "power2.out", 0.5)

// Timeline nesting
const tl = gsap.timeline()
tl.to(".box", { x: 100 })
  .to(".circle", { y: 50 }, "<")  // Start when previous begins
  .to(".triangle", { opacity: 0 }, "+=0.5")  // 0.5s after previous ends
```

---

## Part 2: ArchFlow Rust API Design

### Core Principles

```rust
// 1. FLUENCY: Method chaining
canvas.animate(shape_id)
    .to_x(100.0)
    .to_y(200.0)
    .scale(1.5)
    .rotate(45.0)
    .duration(Duration::from_millis(500))
    .easing(Ease::OutExpo)
    .start()

// 2. EXPRESSIVENESS: Builder pattern
canvas.animate(shape_id)
    .tween(TweenProperty::Position, (0.0, 0.0), (100.0, 100.0))
    .tween(TweenProperty::Scale, 1.0, 1.5)
    .duration(Duration::from_millis(500))
    .easing(Ease::OutExpo)
    .stagger(Stagger::grid(3, 3).from_center())
    .start()

// 3. COMPOSITION: Timeline builder
let timeline = Timeline::builder()
    .add(Animation::to(shape1).x(100).y(100))
    .add(Animation::to(shape2).opacity(1.0), "-=500ms")
    .add(Animation::to(shape3).scale(1.2), "+=200ms")
    .play()
```

### Complete API Specification

```rust
// archflow-sdk/src/animation/mod.rs

use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Main animation entry point - fluent API
pub struct Animator {
    canvas_id: CanvasId,
    targets: Vec<ShapeId>,
    config: AnimationConfig,
    tweens: Vec<Tween>,
}

impl Animator {
    /// Create a new animator for shapes
    pub fn animate(canvas: &Canvas, targets: impl IntoTargets) -> Self {
        Self {
            canvas_id: canvas.id(),
            targets: targets.into_targets(),
            config: AnimationConfig::default(),
            tweens: Vec::new(),
        }
    }
    
    // === PROPERTY TWEENING ===
    
    /// Tween position (x, y)
    pub fn to(mut self, x: f32, y: f32) -> Self {
        self.tweens.push(Tween::Position { from: None, to: (x, y) });
        self
    }
    
    /// Tween from specific position
    pub fn from(mut self, x: f32, y: f32) -> Self {
        if let Some(Tween::Position { .. }) = self.tweens.last_mut() {
            // Update last tween's from value
        } else {
            self.tweens.push(Tween::Position { from: Some((x, y)), to: (x, y) });
        }
        self
    }
    
    /// Relative position change
    pub fn by(mut self, dx: f32, dy: f32) -> Self {
        self.tweens.push(Tween::PositionBy { dx, dy });
        self
    }
    
    /// Tween scale
    pub fn scale(mut self, scale: f32) -> Self {
        self.tweens.push(Tween::Scale { from: None, to: scale });
        self
    }
    
    /// Tween rotation (degrees)
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.tweens.push(Tween::Rotation { from: None, to: degrees });
        self
    }
    
    /// Tween opacity
    pub fn opacity(mut self, value: f32) -> Self {
        self.tweens.push(Tween::Opacity { from: None, to: value });
        self
    }
    
    /// Tween color
    pub fn color(mut self, color: Color) -> Self {
        self.tweens.push(Tween::Color { from: None, to: color });
        self
    }
    
    // === TIMING ===
    
    /// Set animation duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.duration = duration;
        self
    }
    
    /// Set delay before animation starts
    pub fn delay(mut self, delay: Duration) -> Self {
        self.config.delay = delay;
        self
    }
    
    /// Set easing function
    pub fn easing(mut self, easing: Ease) -> Self {
        self.config.easing = easing;
        self
    }
    
    // === STAGGERING ===
    
    /// Stagger animations across multiple targets
    pub fn stagger(mut self, stagger: Stagger) -> Self {
        self.config.stagger = Some(stagger);
        self
    }
    
    // === LOOPING ===
    
    /// Loop animation infinitely
    pub fn loop_infinitely(mut self) -> Self {
        self.config.loop_type = LoopType::Infinite;
        self
    }
    
    /// Loop animation N times
    pub fn loop_times(mut self, n: u32) -> Self {
        self.config.loop_type = LoopType::Count(n);
        self
    }
    
    /// Ping-pong loop
    pub fn loop_ping_pong(mut self) -> Self {
        self.config.loop_type = LoopType::PingPong;
        self
    }
    
    // === CONTROL ===
    
    /// Start the animation
    pub fn start(self) -> AnimationHandle {
        // ... implementation
    }
    
    /// Pause after starting
    pub fn pause_after(self) -> AnimationHandle {
        // ... implementation
    }
}

/// Convenience methods on Canvas
impl Canvas {
    /// Animate one or more shapes
    pub fn animate(&self, targets: impl IntoTargets) -> Animator {
        Animator::animate(self, targets)
    }
    
    /// Animate a single shape by ID
    pub fn animate_shape(&self, id: ShapeId) -> Animator {
        self.animate(vec![id])
    }
}
```

### Easing Functions - Comprehensive Library

```rust
// archflow-sdk/src/animation/easing.rs

/// All easing functions from Robert Penner + nice_and_easy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Ease {
    // Linear
    Linear,
    
    // Quad (t²)
    InQuad, OutQuad, InOutQuad, OutInQuad,
    
    // Cubic (t³)
    InCubic, OutCubic, InOutCubic, OutInCubic,
    
    // Quart (t⁴)
    InQuart, OutQuart, InOutQuart, OutInQuart,
    
    // Quint (t⁵)
    InQuint, OutQuint, InOutQuint, OutInQuint,
    
    // Sine (sin)
    InSine, OutSine, InOutSine, OutInSine,
    
    // Expo (2^t)
    InExpo, OutExpo, InOutExpo, OutInExpo,
    
    // Circ (√(1-t²))
    InCirc, OutCirc, InOutCirc, OutInCirc,
    
    // Back (overshoot)
    InBack, OutBack, InOutBack, OutInBack,
    
    // Elastic (spring)
    InElastic(Option<f32>, Option<f32>), // amplitude, period
    OutElastic(Option<f32>, Option<f32>),
    InOutElastic(Option<f32>, Option<f32>),
    
    // Bounce (bounce at end)
    InBounce, OutBounce, InOutBounce, OutInBounce,
    
    // Spring physics
    Spring { mass: f32, stiffness: f32, damping: f32 },
    
    // Custom cubic bezier
    CubicBezier(f32, f32, f32, f32),
}

// Parse from string (for JS interop)
impl std::str::FromStr for Ease {
    type Err = ParseEaseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linear" => Ok(Self::Linear),
            "easeInQuad" => Ok(Self::InQuad),
            "easeOutQuad" => Ok(Self::OutQuad),
            "easeInOutQuad" => Ok(Self::InOutQuad),
            "easeOutExpo" => Ok(Self::OutExpo),
            "spring" => Ok(Self::Spring { 
                mass: 1.0, 
                stiffness: 100.0, 
                damping: 10.0 
            }),
            // ... all 75 variants
            _ => Err(ParseEaseError::UnknownEase(s.to_string())),
        }
    }
}

// Display for serialization
impl std::fmt::Display for Ease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => write!(f, "linear"),
            Self::OutExpo => write!(f, "easeOutExpo"),
            Self::Spring { mass, stiffness, damping } => {
                write!(f, "spring({},{},{})", mass, stiffness, damping)
            }
            // ... all variants
        }
    }
}
```

### Staggering - Powerful Wave Effects

```rust
// archflow-sdk/src/animation/stagger.rs

/// Stagger configuration - inspired by anime.js + Motion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stagger {
    value: Duration,
    start: Duration,
    from: StaggerFrom,
    grid: Option<(usize, usize)>,
    axis: Option<StaggerAxis>,
    easing: Option<Ease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaggerFrom {
    First,
    Last,
    Center,
    Index(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaggerAxis {
    X,
    Y,
}

impl Stagger {
    /// Simple stagger by duration
    pub fn by(duration: Duration) -> Self {
        Self {
            value: duration,
            start: Duration::ZERO,
            from: StaggerFrom::First,
            grid: None,
            axis: None,
            easing: None,
        }
    }
    
    /// Stagger from center
    pub fn from_center(mut self) -> Self {
        self.from = StaggerFrom::Center;
        self
    }
    
    /// Stagger from last
    pub fn from_last(mut self) -> Self {
        self.from = StaggerFrom::Last;
        self
    }
    
    /// Stagger from specific index
    pub fn from_index(mut self, index: usize) -> Self {
        self.from = StaggerFrom::Index(index);
        self
    }
    
    /// 2D grid staggering
    pub fn grid(mut self, rows: usize, cols: usize) -> Self {
        self.grid = Some((rows, cols));
        self
    }
    
    /// Set axis for grid staggering
    pub fn axis(mut self, axis: StaggerAxis) -> Self {
        self.axis = Some(axis);
        self
    }
    
    /// Apply easing to stagger distribution
    pub fn ease(mut self, easing: Ease) -> Self {
        self.easing = Some(easing);
        self
    }
    
    /// Set initial delay
    pub fn start_delay(mut self, delay: Duration) -> Self {
        self.start = delay;
        self
    }
}

// Convenience constructor
impl From<Duration> for Stagger {
    fn from(duration: Duration) -> Self {
        Self::by(duration)
    }
}
```

### Timeline - Professional Sequencing

```rust
// archflow-sdk/src/animation/timeline.rs

/// Timeline for sequencing animations - GSAP/anime.js inspired
pub struct Timeline {
    canvas_id: CanvasId,
    animations: Vec<TimelineAnimation>,
    position: Duration,
    config: TimelineConfig,
}

struct TimelineAnimation {
    animation: Animation,
    offset: TimeOffset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeOffset {
    Start,
    After(Duration),    // +=duration
    Before(Duration),   // -=duration
    At(Duration),       // Absolute position
    Label(String),      // Reference point
    WithLabel(String),  // Start at same time as label
}

impl Timeline {
    pub fn builder() -> TimelineBuilder {
        TimelineBuilder::new()
    }
    
    /// Add animation to timeline
    pub fn add(&mut self, animation: Animation, offset: TimeOffset) -> &mut Self {
        // ... implementation
        self
    }
    
    /// Create a label for reference
    pub fn label(&mut self, name: impl Into<String>) -> &mut Self {
        // ... implementation
        self
    }
    
    /// Play the timeline
    pub fn play(&mut self) -> TimelineHandle {
        // ... implementation
    }
    
    /// Pause the timeline
    pub fn pause(&mut self) {
        // ... implementation
    }
    
    /// Seek to specific time
    pub fn seek(&mut self, time: Duration) {
        self.position = time;
        // ... update all animations
    }
    
    /// Get timeline duration
    pub fn duration(&self) -> Duration {
        // ... calculate total duration
    }
}

/// Builder for fluent timeline construction
pub struct TimelineBuilder {
    canvas_id: Option<CanvasId>,
    defaults: AnimationConfig,
    animations: Vec<(Animation, TimeOffset)>,
    labels: Vec<(String, Duration)>,
}

impl TimelineBuilder {
    pub fn new() -> Self {
        Self {
            canvas_id: None,
            defaults: AnimationConfig::default(),
            animations: Vec::new(),
            labels: Vec::new(),
        }
    }
    
    pub fn canvas(mut self, id: CanvasId) -> Self {
        self.canvas_id = Some(id);
        self
    }
    
    pub fn default_duration(mut self, duration: Duration) -> Self {
        self.defaults.duration = duration;
        self
    }
    
    pub fn default_easing(mut self, easing: Ease) -> Self {
        self.defaults.easing = easing;
        self
    }
    
    /// Add animation with relative timing
    pub fn add(mut self, animation: Animation) -> Self {
        self.animations.push((animation, TimeOffset::Start));
        self
    }
    
    /// Add animation with offset
    pub fn add_with(mut self, animation: Animation, offset: TimeOffset) -> Self {
        self.animations.push((animation, offset));
        self
    }
    
    /// Add animation after previous (+= syntax)
    pub fn then(mut self, animation: Animation) -> Self {
        let offset = TimeOffset::After(Duration::ZERO);
        self.animations.push((animation, offset));
        self
    }
    
    /// Add animation before previous ends (-= syntax)
    pub fn overlap(mut self, animation: Animation, duration: Duration) -> Self {
        let offset = TimeOffset::Before(duration);
        self.animations.push((animation, offset));
        self
    }
    
    /// Create label at current position
    pub fn label(mut self, name: impl Into<String>) -> Self {
        self.labels.push((name.into(), Duration::ZERO));
        self
    }
    
    pub fn build(self, canvas: &Canvas) -> Timeline {
        Timeline {
            canvas_id: canvas.id(),
            animations: self.animations.into_iter()
                .map(|(anim, offset)| TimelineAnimation { animation: anim, offset })
                .collect(),
            position: Duration::ZERO,
            config: TimelineConfig { defaults: self.defaults },
        }
    }
}

// Convenience: create timeline from Canvas
impl Canvas {
    pub fn timeline(&self) -> TimelineBuilder {
        TimelineBuilder::new().canvas(self.id())
    }
}
```

---

## Part 3: JavaScript/WASM API

### TypeScript Definitions

```typescript
// archflow-sdk/src/animation.ts

/**
 * ArchFlow Animation API - TypeScript bindings
 * Generated via wasm-bindgen
 */

export interface ShapeId {
  id: string;
}

export interface CanvasId {
  id: string;
}

/**
 * Easing functions - all 75 variants
 */
export type Ease = 
  | "linear"
  | "easeInQuad" | "easeOutQuad" | "easeInOutQuad" | "easeOutInQuad"
  | "easeInCubic" | "easeOutCubic" | "easeInOutCubic" | "easeOutInCubic"
  | "easeInQuart" | "easeOutQuart" | "easeInOutQuart" | "easeOutInQuart"
  | "easeInQuint" | "easeOutQuint" | "easeInOutQuint" | "easeOutInQuint"
  | "easeInSine" | "easeOutSine" | "easeInOutSine" | "easeOutInSine"
  | "easeInExpo" | "easeOutExpo" | "easeInOutExpo" | "easeOutInExpo"
  | "easeInCirc" | "easeOutCirc" | "easeInOutCirc" | "easeOutInCirc"
  | "easeInBack" | "easeOutBack" | "easeInOutBack" | "easeOutInBack"
  | "easeInElastic" | "easeOutElastic" | "easeInOutElastic"
  | "easeInBounce" | "easeOutBounce" | "easeInOutBounce"
  | { spring: { mass: number, stiffness: number, damping: number } }
  | { cubicBezier: [number, number, number, number] };

/**
 * Stagger configuration
 */
export interface Stagger {
  value: number; // milliseconds
  start?: number; // initial delay
  from?: "first" | "last" | "center" | number;
  grid?: [number, number]; // [rows, cols]
  axis?: "x" | "y";
  ease?: Ease;
}

/**
 * Time offset for timeline
 */
export type TimeOffset = 
  | "start"           // Start at timeline beginning
  | `+=${number}`      // After previous ends (e.g., "+=500")
  | `-=${number}`      // Before previous ends (e.g., "-=500")
  | number;           // Absolute position in milliseconds

/**
 * Animation handle - control running animations
 */
export class AnimationHandle {
  pause(): void;
  play(): void;
  seek(time: number): void;
  reverse(): void;
  cancel(): void;
  
  get progress(): number; // 0.0 to 1.0
  get isPlaying(): boolean;
  get isCompleted(): boolean;
  
  onComplete(callback: () => void): void;
  onUpdate(callback: (progress: number) => void): void;
}

/**
 * Timeline handle - control timeline playback
 */
export class TimelineHandle extends AnimationHandle {
  add(animation: Animation, offset?: TimeOffset): TimelineHandle;
  label(name: string): TimelineHandle;
  
  get duration(): number;
  get position(): number;
}

/**
 * Main animation API - fluent chaining
 */
export class Animator {
  // Position
  to(x: number, y: number): this;
  from(x: number, y: number): this;
  by(dx: number, dy: number): this;
  
  toX(x: number): this;
  toY(y: number): this;
  
  // Transform
  scale(value: number): this;
  rotate(degrees: number): this;
  skew(x: number, y: number): this;
  
  // Style
  opacity(value: number): this;
  color(color: string): this;
  strokeWidth(width: number): this;
  
  // Timing
  duration(milliseconds: number): this;
  delay(milliseconds: number): this;
  easing(easing: Ease): this;
  
  // Staggering
  stagger(stagger: Stagger | number): this;
  
  // Looping
  loop(times?: number): this;
  pingPong(): this;
  
  // Control
  start(): AnimationHandle;
  pauseAfter(): AnimationHandle;
  
  // Composition
  then(animation: Animation): TimelineBuilder;
}

/**
 * Timeline builder
 */
export class TimelineBuilder {
  add(animation: Animation | Animator, offset?: TimeOffset): this;
  label(name: string): this;
  then(animation: Animation | Animator): this;
  overlap(animation: Animation | Animator, milliseconds: number): this;
  
  play(): TimelineHandle;
}

/**
 * Canvas extensions - entry point
 */
export class Canvas {
  // Animate single shape
  animate(shapeId: string): Animator;
  
  // Animate multiple shapes
  animateAll(shapeIds: string[]): Animator;
  
  // Animate with CSS selector-like syntax
  animateSelector(selector: string): Animator;
  
  // Timeline
  timeline(): TimelineBuilder;
  
  // Convenience methods
  fadeIn(shapeId: string, duration?: number): Animator;
  fadeOut(shapeId: string, duration?: number): Animator;
  slideIn(shapeId: string, from: {x: number, y: number}, duration?: number): Animator;
}
```

### JavaScript Usage Examples

```javascript
// === BASIC ANIMATIONS ===

// Simple fade in
canvas.fadeIn("shape-123", 500).start();

// Move and scale
canvas.animate("shape-123")
  .to(100, 200)
  .scale(1.5)
  .rotate(45)
  .duration(750)
  .easing("easeOutExpo")
  .start();

// From-to animation
canvas.animate("shape-123")
  .from(0, 0)
  .to(100, 100)
  .duration(500)
  .start();

// Relative movement
canvas.animate("shape-123")
  .by(50, 50)  // Move 50px right and down
  .duration(300)
  .start();

// === STAGGERING ===

// Stagger multiple shapes
canvas.animateAll(["shape-1", "shape-2", "shape-3"])
  .to(100, 100)
  .stagger(100)  // 100ms delay between each
  .start();

// Stagger from center with grid
canvas.animateSelector(".grid-item")
  .scale(1.2)
  .stagger({
    value: 100,
    from: "center",
    grid: [4, 4],
    axis: "x",
    ease: "easeOut"
  })
  .start();

// === TIMELINES ===

// Simple sequence
const tl = canvas.timeline();
tl.add(canvas.animate("box-1").to(100, 100).duration(500))
  .then(canvas.animate("box-2").to(200, 200).duration(500))
  .then(canvas.animate("box-3").opacity(1).duration(300))
  .play();

// Complex timeline with overlaps
const tl = canvas.timeline();
tl.add(canvas.animate("logo").scale(1).duration(500))
  .add(canvas.animate("title").opacity(1), "-=300")  // Start 300ms before logo ends
  .add(canvas.animate("subtitle").opacity(1), "+=200")  // Start 200ms after title
  .label("intro-complete")
  .add(canvas.animate("content").to(0, 100), "intro-complete")  // Start at label
  .play();

// Nested timelines
const mainTl = canvas.timeline();
const subTl = canvas.timeline();

subTl.add(canvas.animate("a").to(10, 10))
      .add(canvas.animate("b").to(20, 20))
      .play();

mainTl.add(canvas.animate("main").scale(1))
      .add(subTl)
      .add(canvas.animate("end").opacity(1))
      .play();

// === SPRING PHYSICS ===

// Spring animation
canvas.animate("box")
  .to(100, 100)
  .easing({ spring: { mass: 1, stiffness: 100, damping: 10 } })
  .start();

// === CONTROLS ===

const anim = canvas.animate("box")
  .to(100, 100)
  .duration(1000)
  .start();

// Pause/Play
anim.pause();
anim.play();

// Seek
anim.seek(500);  // Jump to 500ms

// Reverse
anim.reverse();

// Events
anim.onComplete(() => console.log("Done!"));
anim.onUpdate((progress) => console.log(`Progress: ${progress * 100}%`));

// Check state
if (anim.isPlaying) {
  console.log("Animation is playing");
}
if (anim.isCompleted) {
  console.log("Animation completed");
}

// === LOOPING ===

// Infinite loop
canvas.animate("spinner")
  .rotate(360)
  .duration(1000)
  .loop()
  .start();

// Loop 3 times
canvas.animate("pulse")
  .scale(1.2)
  .duration(300)
  .loop(3)
  .start();

// Ping-pong
canvas.animate("bounce")
  .to(0, 100)
  .duration(500)
  .pingPong()
  .start();

// === EASING ===

// All easing functions available
canvas.animate("box")
  .to(100, 100)
  .easing("easeOutBounce")
  .start();

// Cubic bezier
canvas.animate("box")
  .to(100, 100)
  .easing({ cubicBezier: [0.32, 0.23, 0.4, 0.9] })
  .start();

// Spring with custom parameters
canvas.animate("box")
  .to(100, 100)
  .easing({ spring: { mass: 2, stiffness: 150, damping: 15 } })
  .start();

// === PRESET ANIMATIONS ===

// Fade effects
canvas.fadeIn("shape", 500).start();
canvas.fadeOut("shape", 500).start();

// Slide effects
canvas.slideIn("shape", { x: -100, y: 0 }, 500).start();

// Presets built-in
canvas.animate("shape")
  .preset("fadeInUp")
  .start();

// === SELECTORS ===

// Animate by class (CSS-like)
canvas.animateSelector(".highlighted")
  .opacity(1)
  .duration(300)
  .stagger(50)
  .start();

// Animate by data attribute
canvas.animateSelector("[data-active='true']")
  .scale(1.1)
  .duration(200)
  .start();

// === CHAINED ANIMATIONS ===

// Method chaining
canvas.animate("box")
  .from(0, 0)
  .to(100, 100)
  .scale(1.5)
  .rotate(45)
  .opacity(0.8)
  .duration(750)
  .delay(200)
  .easing("easeOutExpo")
  .stagger({ value: 100, from: "center" })
  .loop(3)
  .pingPong()
  .start();

// === REAL-WORLD EXAMPLES ===

// 1. Loading spinner
const spinner = canvas.animate("spinner")
  .rotate(360)
  .duration(1000)
  .loop()
  .easing("linear")
  .start();

// 2. Card flip animation
canvas.animate("card-front")
  .scaleX(0)
  .duration(300)
  .easing("easeInCubic")
  .start();

canvas.animate("card-back")
  .scaleX(1)
  .duration(300)
  .delay(150)
  .easing("easeOutCubic")
  .start();

// 3. Staggered list animation
const items = document.querySelectorAll(".list-item");
canvas.animateAll([...items].map(el => el.getAttribute("data-shape-id")))
  .from(100, 0)
  .to(0, 0)
  .opacity(1)
  .stagger({ value: 75, from: "first", ease: "easeOut" })
  .duration(400)
  .start();

// 4. Hero animation sequence
const heroTl = canvas.timeline();

heroTl
  .add(canvas.animate("logo").scale(1).opacity(1).duration(800).easing("easeOutExpo"))
  .add(canvas.animate("tagline").opacity(1).from(0, 20).to(0, 0), "-=600")
  .add(canvas.animate("cta-button").scale(1).from(0.8).duration(500).easing("easeOutBack"), "-=400")
  .label("intro-done")
  .add(canvas.animate("features").opacity(1).from(0, 50).to(0, 0), "intro-done")
  .add(canvas.animateSelector(".feature").scale(1).stagger(100), "+=200")
  .play();

// 5. Interactive hover (with spring)
document.querySelector(".card").addEventListener("mouseenter", () => {
  canvas.animate("card-highlight")
    .scale(1.05)
    .duration(300)
    .easing({ spring: { stiffness: 300, damping: 20 } })
    .start();
});
```

---

## Part 4: Rust Implementation Details

### WASM Bindings

```rust
// archflow-sdk/src/animation/wasm.rs

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

/// Animation API for JavaScript/WASM
#[wasm_bindgen]
pub struct Animator {
    inner: crate::animation::Animator,
}

#[wasm_bindgen]
impl Animator {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, targets: JsValue) -> Result<Animator, JsValue> {
        // Parse targets from JS
        let target_ids: Vec<String> = targets.into_serde()?;
        // Create animator
        Ok(Animator { 
            inner: crate::animation::Animator::new(/* ... */) 
        })
    }
    
    // Position methods
    #[wasm_bindgen(method)]
    pub fn to(mut self, x: f32, y: f32) -> Self {
        self.inner = self.inner.to(x, y);
        self
    }
    
    #[wasm_bindgen(method)]
    pub fn scale(mut self, value: f32) -> Self {
        self.inner = self.inner.scale(value);
        self
    }
    
    #[wasm_bindgen(method)]
    pub fn opacity(mut self, value: f32) -> Self {
        self.inner = self.inner.opacity(value);
        self
    }
    
    // Timing
    #[wasm_bindgen(method)]
    pub fn duration(mut self, ms: f64) -> Self {
        self.inner = self.inner.duration(Duration::from_millis_f64(ms));
        self
    }
    
    #[wasm_bindgen(method)]
    pub fn easing(mut self, easing: &str) -> Result<Self, JsValue> {
        let ease = easing.parse::<Ease>()
            .map_err(|e| JsValue::from_str(&format!("Invalid easing: {}", e)))?;
        self.inner = self.inner.easing(ease);
        Ok(self)
    }
    
    // Stagger
    #[wasm_bindgen(method)]
    pub fn stagger(mut self, value: JsValue) -> Result<Self, JsValue> {
        let stagger: StaggerConfig = value.into_serde()?;
        self.inner = self.inner.stagger(stagger);
        Ok(self)
    }
    
    // Start
    #[wasm_bindgen(method)]
    pub fn start(self) -> AnimationHandle {
        AnimationHandle::new(self.inner.start())
    }
}

/// Animation control handle
#[wasm_bindgen]
pub struct AnimationHandle {
    inner: crate::animation::AnimationHandle,
}

#[wasm_bindgen]
impl AnimationHandle {
    #[wasm_bindgen(method)]
    pub fn pause(&mut self) {
        self.inner.pause();
    }
    
    #[wasm_bindgen(method)]
    pub fn play(&mut self) {
        self.inner.play();
    }
    
    #[wasm_bindgen(method)]
    pub fn seek(&mut self, time_ms: f64) {
        self.inner.seek(Duration::from_millis_f64(time_ms));
    }
    
    #[wasm_bindgen(method, getter)]
    pub fn progress(&self) -> f64 {
        self.inner.progress() as f64
    }
    
    #[wasm_bindgen(method, getter)]
    pub fn isPlaying(&self) -> bool {
        self.inner.is_playing()
    }
    
    #[wasm_bindgen(method, getter)]
    pub fn isCompleted(&self) -> bool {
        self.inner.is_completed()
    }
    
    #[wasm_bindgen(method)]
    pub fn onComplete(&self, callback: &js_sys::Function) {
        self.inner.on_complete(move || {
            callback.call0(&JsValue::NULL).ok();
        });
    }
    
    #[wasm_bindgen(method)]
    pub fn onUpdate(&self, callback: &js_sys::Function) {
        self.inner.on_update(move |progress| {
            callback.call1(&JsValue::NULL, &JsValue::from_f64(progress as f64)).ok();
        });
    }
}

/// Timeline for WASM
#[wasm_bindgen]
pub struct Timeline {
    inner: crate::animation::Timeline,
}

#[wasm_bindgen]
impl Timeline {
    #[wasm_bindgen(method)]
    pub fn add(&mut self, animation: Animator, offset: JsValue) -> Result<Timeline, JsValue> {
        let offset = parse_time_offset(offset)?;
        self.inner.add(animation.inner, offset);
        Ok(Timeline { inner: self.inner.clone() })
    }
    
    #[wasm_bindgen(method)]
    pub fn play(&mut self) -> TimelineHandle {
        TimelineHandle::new(self.inner.play())
    }
}

#[wasm_bindgen]
pub struct TimelineHandle {
    inner: crate::animation::TimelineHandle,
}

#[wasm_bindgen]
impl TimelineHandle {
    // Inherit all AnimationHandle methods
    // Add timeline-specific methods
}
```

### Performance Optimizations

```rust
// archflow-sdk/src/animation/performance.rs

/// Object pooling for zero-allocation animation updates
pub struct AnimationPool<T> {
    free: Vec<Box<T>>,
    capacity: usize,
}

impl<T> AnimationPool<T> {
    pub fn acquire(&mut self) -> Box<T> {
        self.free.pop().unwrap_or_else(|| Box::new(T::default()))
    }
    
    pub fn release(&mut self, item: Box<T>) {
        if self.free.len() < self.capacity {
            self.free.push(item);
        }
    }
}

/// Batch update for multiple animations
pub fn update_animations_batch(animators: &mut [Animator], delta: Duration) {
    // Parallel iteration using rayon if available
    #[cfg(feature = "parallel")]
    {
        animators.par_iter_mut().for_each(|anim| {
            anim.update(delta);
        });
    }
    
    #[cfg(not(feature = "parallel"))]
    {
        for anim in animators.iter_mut() {
            anim.update(delta);
        }
    }
}

/// SIMD-optimized tweening for large batches
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline(always)]
pub fn lerp_vec4_simd(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let a_vec = _mm_loadu_ps(a.as_ptr());
        let b_vec = _mm_loadu_ps(b.as_ptr());
        let t_vec = _mm_set1_ps(t);
        
        let result = _mm_add_ps(
            a_vec,
            _mm_mul_ps(
                _mm_sub_ps(b_vec, a_vec),
                t_vec
            )
        );
        
        let mut out = [0.0; 4];
        _mm_storeu_ps(out.as_mut_ptr(), result);
        out
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ]
    }
}
```

---

## Part 5: Usage Examples - Rust Side

### Basic Usage

```rust
use archflow_sdk::animation::*;
use archflow_sdk::canvas::Canvas;
use std::time::Duration;

// Simple animation
let canvas = Canvas::new("my-canvas");
let shape = canvas.create_rectangle(0.0, 0.0, 50.0, 50.0);

canvas.animate(shape.id())
    .to(100.0, 100.0)
    .scale(1.5)
    .rotate(45.0)
    .duration(Duration::from_millis(500))
    .easing(Ease::OutExpo)
    .start();

// From-to animation
canvas.animate(shape.id())
    .from(0.0, 0.0)
    .to(100.0, 100.0)
    .duration(Duration::from_millis(750))
    .start();

// Relative movement
canvas.animate(shape.id())
    .by(50.0, 50.0)
    .duration(Duration::from_millis(300))
    .start();
```

### Staggering

```rust
// Simple stagger
let shapes: Vec<ShapeId> = /* ... */;

canvas.animate_all(&shapes)
    .to(100.0, 100.0)
    .stagger(Stagger::by(Duration::from_millis(100)))
    .start();

// Grid stagger from center
canvas.animate_all(&shapes)
    .scale(1.2)
    .stagger(
        Stagger::by(Duration::from_millis(100))
            .from_center()
            .grid(3, 3)
            .axis(StaggerAxis::X)
            .ease(Ease::Out)
    )
    .start();
```

### Timeline

```rust
// Build timeline
let timeline = Timeline::builder()
    .canvas(canvas.id())
    .default_duration(Duration::from_millis(500))
    .default_easing(Ease::OutExpo)
    .then(Animation::to(shape1).x(100).y(100))
    .then(Animation::to(shape2).opacity(1.0))
    .overlap(Animation::to(shape3).scale(1.2), Duration::from_millis(200))
    .label("intro-complete")
    .add(Animation::to(shape4).to(0, 100), TimeOffset::Label("intro-complete".into()))
    .build(&canvas);

timeline.play();

// Or using canvas shorthand
let tl = canvas.timeline()
    .add(canvas.animate(shape1).to(100, 100).duration(500))
    .add(canvas.animate(shape2).opacity(1), "-=300ms")
    .add(canvas.animate(shape3).scale(1.2), "+=200ms")
    .play();
```

### Controls

```rust
let handle = canvas.animate(shape)
    .to(100, 100)
    .duration(Duration::from_secs(1))
    .start();

// Pause/Play
handle.pause();
handle.play();

// Seek
handle.seek(Duration::from_millis(500));

// Events
handle.on_complete(|| {
    println!("Animation completed!");
});

handle.on_update(|progress| {
    println!("Progress: {}%", progress * 100.0);
});
```

### Spring Physics

```rust
// Spring animation
canvas.animate(shape)
    .to(100, 100)
    .easing(Ease::Spring { 
        mass: 1.0, 
        stiffness: 100.0, 
        damping: 10.0 
    })
    .start();
```

---

## Part 6: Testing Strategy

```rust
// archflow-sdk/src/animation/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fluent_api() {
        let canvas = Canvas::new("test");
        let shape = canvas.create_rectangle(0.0, 0.0, 10.0, 10.0);
        
        let handle = canvas.animate(shape.id())
            .to(100.0, 100.0)
            .scale(1.5)
            .rotate(45.0)
            .duration(Duration::from_millis(500))
            .easing(Ease::OutExpo)
            .start();
        
        assert!(!handle.is_completed());
    }
    
    #[test]
    fn test_stagger_grid_center() {
        let shapes: Vec<_> = (0..9).map(|_| {
            canvas.create_rectangle(0.0, 0.0, 10.0, 10.0)
        }).collect();
        
        canvas.animate_all(&shapes)
            .scale(1.2)
            .stagger(Stagger::by(Duration::from_millis(100)).from_center().grid(3, 3))
            .start();
        
        // Verify center shape animates first
        let delays = calculate_stagger_delays(&shapes);
        assert_eq!(delays[4], Duration::ZERO); // Center of 3x3 grid
    }
    
    #[test]
    fn test_timeline_overlaps() {
        let tl = canvas.timeline()
            .add(canvas.animate(s1).to(100, 100).duration(500))
            .add(canvas.animate(s2).opacity(1), "-=300")
            .play();
        
        // Verify second animation starts before first ends
        assert!(tl.get_animation_start_time(1) < tl.get_animation_start_time(0) + Duration::from_millis(500));
    }
    
    #[test]
    fn test_easing_comprehensive() {
        // Test all 75 easing functions
        let easings = vec![
            Ease::Linear,
            Ease::InQuad, Ease::OutQuad, Ease::InOutQuad,
            // ... all variants
        ];
        
        for easing in easings {
            let value = easing.apply(0.5);
            assert!(value >= 0.0 && value <= 1.0, "Invalid easing value for {:?}", easing);
        }
    }
}
```

---

## Part 7: Migration Guide

### From Old API to New API

```rust
// OLD API (archflow-core 0.15.0)
let mut animation = PositionAnimation::new(
    shape_id,
    vec![
        PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::EaseInOut),
        PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::EaseInOut),
    ]
);
animation.config = AnimationConfig {
    duration: Duration::from_millis(500),
    ..Default::default()
};
animation.start();

// NEW API (archflow-sdk 0.16.0)
canvas.animate(shape_id)
    .to(100.0, 100.0)
    .duration(Duration::from_millis(500))
    .easing(Ease::InOut)
    .start();
```

```javascript
// OLD API (WASM 0.15.0)
const anim = new PositionAnimation(shapeId, keyframes, config);
anim.start();

// NEW API (WASM 0.16.0)
canvas.animate(shapeId)
  .to(100, 100)
  .duration(500)
  .start();
```

---

## Part 8: Performance Benchmarks

### Target Performance

```rust
// archflow-sdk/src/animation/bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_simple_animation(c: &mut Criterion) {
    c.bench_function("simple_animation", |b| {
        b.iter(|| {
            canvas.animate(shape)
                .to(100.0, 100.0)
                .duration(Duration::from_millis(500))
                .start();
        });
    });
}

fn bench_1000_animations(c: &mut Criterion) {
    c.bench_function("1000_animations", |b| {
        b.iter(|| {
            for i in 0..1000 {
                canvas.animate(shapes[i])
                    .to(100.0, 100.0)
                    .stagger(Stagger::by(Duration::from_millis(1)))
                    .start();
            }
        });
    });
}

fn bench_timeline_complex(c: &mut Criterion) {
    c.bench_function("complex_timeline", |b| {
        b.iter(|| {
            canvas.timeline()
                .add(canvas.animate(s1).to(100, 100).duration(500))
                .add(canvas.animate(s2).opacity(1), "-=300")
                .add(canvas.animate(s3).scale(1.2), "+=200")
                .play();
        });
    });
}

criterion_group!(benches, bench_simple_animation, bench_1000_animations, bench_timeline_complex);
criterion_main!(benches);

// TARGETS:
// - Simple animation creation: <100ns
// - 1000 concurrent animations: <16ms/frame (60 FPS)
// - Timeline construction: <10μs
```

---

## Part 9: API Design Checklist

### Developer Experience Features

- [x] **Method chaining** - Fluent API
- [x] **Named parameters** - Clear intent
- [x] **Type safety** - Rust's type system
- [x] **Zero-cost abstractions** - ZST tweens
- [x] **Comprehensive easing** - 75 functions
- [x] **Timeline composition** - Nested timelines
- [x] **Staggering** - Grid/axis/from support
- [x] **Spring physics** - Natural animations
- [x] **Event callbacks** - onComplete, onUpdate
- [x] **Pause/Play/Seek** - Full control
- [x] **Looping** - Infinite, count, ping-pong
- [x] **WASM-friendly** - Serde serialization
- [x] **TypeScript definitions** - Full IntelliSense
- [x] **Selector syntax** - CSS-like targeting
- [x] **Preset animations** - fadeIn, slideIn, etc.

### JavaScript Parity

| Feature | Anime.js | Framer Motion | GSAP | ArchFlow |
|---------|----------|---------------|------|----------|
| Timeline | ✅ | ✅ | ✅ | ✅ |
| Stagger | ✅ Grid | ✅ Eased | ✅ | ✅ Both |
| Spring | ⚠️ Elastic | ✅ Physics | ❌ | ✅ Physics |
| Keyframes | ✅ Per-property | ✅ Variants | ✅ | ✅ |
| Events | ✅ | ✅ | ✅ | ✅ |
| Time labels | ✅ | ❌ | ✅ | ✅ |
| Nested timelines | ❌ | ❌ | ✅ | ✅ |
| Selectors | ✅ CSS | React | ❌ | ✅ CSS-like |

---

## Part 10: Next Steps

### Implementation Priority

1. **Week 1-2**: Core Animator API + tween crate integration
   - Method chaining
   - All property tweens
   - Timing controls

2. **Week 2**: Easing functions + nice_and_easy
   - 75 easing functions
   - String parsing for WASM
   - TypeScript definitions

3. **Week 3**: Staggering system
   - 1D staggering
   - Grid staggering
   - Axis support

4. **Week 4**: Timeline system
   - Basic sequencing
   - Relative offsets
   - Labels

5. **Week 5**: Spring physics
   - Spring solver
   - Parameters (mass, stiffness, damping)

6. **Week 6**: WASM bindings + TypeScript
   - wasm-bindgen integration
   - TypeScript definitions
   - Examples and documentation

---

## Conclusion

Este diseño de API logra el equilibrio perfecto entre:

1. **Potencia de Rust**: Zero-cost abstractions, type safety, performance
2. **Expresividad de JavaScript**: Method chaining, named parameters, composition
3. **Mejor de ambos mundos**: Timeline composition como GSAP, staggering como Anime.js, spring physics como Motion

El resultado es un API que se siente natural tanto en Rust como en JavaScript, con una curva de aprendizaje mínima gracias a los patrones ya familiares de las librerías más populares del ecosistema JavaScript/TypeScript.

**Document Version**: 1.0  
**Last Updated**: 2025-01-28  
**Next Review**: After Week 2 implementation
