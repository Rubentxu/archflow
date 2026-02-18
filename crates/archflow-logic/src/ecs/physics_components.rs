// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Physics Components Module
//
// Provides physics-related components for the ECS system:
// - Position: World position (2D)
// - Velocity: Velocity vector (2D)
// - Acceleration: Acceleration vector (2D)
// - Transform: Combined position, rotation, scale
//
// These components integrate with the BGE logic system for physics-based gameplay.
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════


extern crate alloc;

use alloc::vec::Vec;
use core::f32;

use crate::ecs::component::VecStorage;

/// A 2D velocity component
///
/// Represents the velocity of an entity in world space.
/// Velocity is applied to Position each frame by the physics system.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Velocity {
    /// Velocity in X direction (units per second)
    pub dx: f32,
    /// Velocity in Y direction (units per second)
    pub dy: f32,
}

impl Velocity {
    /// Creates a new velocity with the given components
    #[inline(always)]
    #[must_use]
    pub const fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    /// Creates a zero velocity
    #[inline(always)]
    #[must_use]
    pub const fn zero() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }

    /// Returns the magnitude of the velocity
    #[inline(always)]
    pub fn magnitude(&self) -> f32 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }

    /// Returns the squared magnitude (faster, no sqrt)
    #[inline(always)]
    pub fn magnitude_squared(&self) -> f32 {
        self.dx * self.dx + self.dy * self.dy
    }

    /// Normalizes the velocity to unit length
    #[inline(always)]
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self::new(self.dx / mag, self.dy / mag)
        } else {
            Self::zero()
        }
    }

    /// Scales the velocity by a factor
    #[inline(always)]
    pub fn scale(&self, factor: f32) -> Self {
        Self::new(self.dx * factor, self.dy * factor)
    }

    /// Adds another velocity to this one
    #[inline(always)]
    pub fn add(&self, other: &Velocity) -> Self {
        Self::new(self.dx + other.dx, self.dy + other.dy)
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

/// A 2D acceleration component
///
/// Represents the acceleration applied to an entity's velocity.
/// Acceleration is applied to Velocity each frame by the physics system.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Acceleration {
    /// Acceleration in X direction (units per second squared)
    pub ax: f32,
    /// Acceleration in Y direction (units per second squared)
    pub ay: f32,
}

impl Acceleration {
    /// Creates a new acceleration with the given components
    #[inline(always)]
    #[must_use]
    pub const fn new(ax: f32, ay: f32) -> Self {
        Self { ax, ay }
    }

    /// Creates a zero acceleration
    #[inline(always)]
    #[must_use]
    pub const fn zero() -> Self {
        Self { ax: 0.0, ay: 0.0 }
    }

    /// Creates gravity acceleration (downward)
    #[inline(always)]
    #[must_use]
    pub fn gravity(g: f32) -> Self {
        Self::new(0.0, -g)
    }
}

impl Default for Acceleration {
    fn default() -> Self {
        Self::zero()
    }
}

/// A combined transform component for 2D entities
///
/// Contains position, rotation, and scale in a single component
/// for efficient batch processing.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Transform {
    /// Position in world space
    pub position_x: f32,
    pub position_y: f32,
    /// Rotation in radians (counter-clockwise)
    pub rotation: f32,
    /// Scale factors (1.0 = original size)
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Transform {
    /// Creates a new transform at origin with no rotation and unit scale
    #[inline(always)]
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Creates a transform with position only
    #[inline(always)]
    #[must_use]
    pub fn from_position(x: f32, y: f32) -> Self {
        Self {
            position_x: x,
            position_y: y,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Creates a transform with position and scale
    #[inline(always)]
    #[must_use]
    pub fn from_position_scale(x: f32, y: f32, sx: f32, sy: f32) -> Self {
        Self {
            position_x: x,
            position_y: y,
            rotation: 0.0,
            scale_x: sx,
            scale_y: sy,
        }
    }

    /// Updates the position
    #[inline(always)]
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position_x = x;
        self.position_y = y;
    }

    /// Updates the rotation
    #[inline(always)]
    pub fn set_rotation(&mut self, radians: f32) {
        self.rotation = radians;
    }

    /// Updates the scale
    #[inline(always)]
    pub fn set_scale(&mut self, sx: f32, sy: f32) {
        self.scale_x = sx;
        self.scale_y = sy;
    }

    /// Applies a translation
    #[inline(always)]
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position_x += dx;
        self.position_y += dy;
    }

    /// Applies a rotation (additive)
    #[inline(always)]
    pub fn rotate(&mut self, radians: f32) {
        self.rotation += radians;
    }

    /// Applies a scale multiplier
    #[inline(always)]
    pub fn scale_by(&mut self, sx: f32, sy: f32) {
        self.scale_x *= sx;
        self.scale_y *= sy;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Physics material properties
///
/// Defines how an entity responds to physics interactions.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct PhysicsMaterial {
    /// Restitution (bounciness): 0.0 = no bounce, 1.0 = full bounce
    pub restitution: f32,
    /// Friction: 0.0 = no friction, 1.0 = high friction
    pub friction: f32,
    /// Mass in kilograms (0.0 = infinite mass/static)
    pub mass: f32,
    /// Whether the entity is a sensor/trigger (no physical response)
    pub is_sensor: bool,
}

impl PhysicsMaterial {
    /// Creates a default material (solid, some bounce, some friction)
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            restitution: 0.3,
            friction: 0.5,
            mass: 1.0,
            is_sensor: false,
        }
    }

    /// Creates a static (immovable) material
    #[inline(always)]
    #[must_use]
    pub fn static_material() -> Self {
        Self {
            restitution: 0.3,
            friction: 0.5,
            mass: 0.0, // Zero mass = infinite/static
            is_sensor: false,
        }
    }

    /// Creates a sensor/trigger material
    #[inline(always)]
    #[must_use]
    pub fn sensor() -> Self {
        Self {
            restitution: 0.0,
            friction: 0.0,
            mass: 0.0,
            is_sensor: true,
        }
    }

    /// Creates a bouncy material
    #[inline(always)]
    #[must_use]
    pub fn bouncy() -> Self {
        Self {
            restitution: 0.9,
            friction: 0.3,
            mass: 1.0,
            is_sensor: false,
        }
    }
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self::new()
    }
}

/// Highlight state component
///
/// Tracks the visual highlight state of an entity.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct HighlightState {
    /// Current highlight color (RGB)
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    /// Current pulse phase (0.0 to 1.0)
    pub pulse_phase: f32,
    /// Whether the entity is currently highlighted
    pub is_highlighted: bool,
    /// Highlight intensity (0.0 to 1.0)
    pub intensity: f32,
}

impl HighlightState {
    /// Creates a default inactive highlight state
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            color_r: 1.0,
            color_g: 1.0,
            color_b: 0.0,
            pulse_phase: 0.0,
            is_highlighted: false,
            intensity: 0.0,
        }
    }

    /// Creates an active highlight with the given color
    #[inline(always)]
    #[must_use]
    pub fn active(r: f32, g: f32, b: f32) -> Self {
        Self {
            color_r: r,
            color_g: g,
            color_b: b,
            pulse_phase: 0.0,
            is_highlighted: true,
            intensity: 1.0,
        }
    }

    /// Updates the pulse phase
    #[inline(always)]
    pub fn update_pulse(&mut self, delta_time: f32, frequency: f32) {
        self.pulse_phase = (self.pulse_phase + delta_time * frequency) % 1.0;
        self.intensity = 0.5 + 0.5 * (self.pulse_phase * core::f32::consts::TAU).sin();
    }
}

impl Default for HighlightState {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection state component
///
/// Tracks whether an entity is currently selected.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct SelectionState {
    /// Whether the entity is selected
    pub is_selected: bool,
    /// Selection order (for multi-select)
    pub selection_order: u32,
    /// Visual indicator style
    pub indicator_style: u8,
}

impl SelectionState {
    /// Creates a new unselected state
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_selected: false,
            selection_order: 0,
            indicator_style: 0,
        }
    }

    /// Selects the entity
    #[inline(always)]
    pub fn select(&mut self, order: u32) {
        self.is_selected = true;
        self.selection_order = order;
    }

    /// Deselects the entity
    #[inline(always)]
    pub fn deselect(&mut self) {
        self.is_selected = false;
    }
}

/// Animation state component
///
/// Tracks the current animation playback state.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct AnimationState {
    /// Current animation clip ID
    pub clip_id: u32,
    /// Current playback time in seconds
    pub current_time: f32,
    /// Animation duration in seconds
    pub duration: f32,
    /// Playback speed multiplier
    pub speed: f32,
    /// Whether the animation is playing
    pub is_playing: bool,
    /// Whether the animation loops
    pub loop_animation: bool,
    /// Whether the animation completed (one-shot trigger)
    pub completed: bool,
}

impl AnimationState {
    /// Creates a stopped animation state
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            clip_id: 0,
            current_time: 0.0,
            duration: 1.0,
            speed: 1.0,
            is_playing: false,
            loop_animation: false,
            completed: false,
        }
    }

    /// Starts playing an animation
    #[inline(always)]
    pub fn play(&mut self, clip_id: u32, duration: f32, looped: bool) {
        self.clip_id = clip_id;
        self.duration = duration;
        self.current_time = 0.0;
        self.is_playing = true;
        self.loop_animation = looped;
        self.completed = false;
    }

    /// Stops the animation
    #[inline(always)]
    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    /// Updates the animation time
    #[inline(always)]
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.is_playing || self.completed && !self.loop_animation {
            return false;
        }

        self.current_time += delta_time * self.speed;

        if self.current_time >= self.duration {
            if self.loop_animation {
                self.current_time %= self.duration;
            } else {
                self.current_time = self.duration;
                self.is_playing = false;
                self.completed = true;
                return true; // Animation just completed
            }
        }

        false
    }

    /// Gets the normalized progress (0.0 to 1.0)
    #[inline(always)]
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.current_time / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Component Trait Implementations for ECS Integration
// ============================================================================

impl crate::ecs::Component for Velocity {
    type Storage = VecStorage<Velocity>;
}

impl crate::ecs::Component for Acceleration {
    type Storage = VecStorage<Acceleration>;
}

impl crate::ecs::Component for Transform {
    type Storage = VecStorage<Transform>;
}

impl crate::ecs::Component for PhysicsMaterial {
    type Storage = VecStorage<PhysicsMaterial>;
}

impl crate::ecs::Component for HighlightState {
    type Storage = VecStorage<HighlightState>;
}

impl crate::ecs::Component for SelectionState {
    type Storage = VecStorage<SelectionState>;
}

impl crate::ecs::Component for AnimationState {
    type Storage = VecStorage<AnimationState>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_magnitude() {
        let vel = Velocity::new(3.0, 4.0);
        assert!((vel.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_normalized() {
        let vel = Velocity::new(3.0, 4.0);
        let norm = vel.normalized();
        assert!((norm.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_scale() {
        let vel = Velocity::new(1.0, 2.0);
        let scaled = vel.scale(2.0);
        assert_eq!(scaled.dx, 2.0);
        assert_eq!(scaled.dy, 4.0);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::identity();
        assert_eq!(t.position_x, 0.0);
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale_x, 1.0);
    }

    #[test]
    fn test_transform_translate() {
        let mut t = Transform::identity();
        t.translate(10.0, 20.0);
        assert_eq!(t.position_x, 10.0);
        assert_eq!(t.position_y, 20.0);
    }

    #[test]
    fn test_physics_material_default() {
        let mat = PhysicsMaterial::new();
        assert_eq!(mat.restitution, 0.3);
        assert_eq!(mat.mass, 1.0);
        assert!(!mat.is_sensor);
    }

    #[test]
    fn test_physics_material_static() {
        let mat = PhysicsMaterial::static_material();
        assert_eq!(mat.mass, 0.0); // Zero = infinite
    }

    #[test]
    fn test_highlight_state() {
        let mut state = HighlightState::new();
        assert!(!state.is_highlighted);

        state = HighlightState::active(1.0, 0.0, 0.0);
        assert!(state.is_highlighted);
        assert_eq!(state.intensity, 1.0);
    }

    #[test]
    fn test_highlight_state_pulse() {
        let mut state = HighlightState::active(1.0, 0.0, 0.0);
        // Use 0.3s at 2Hz = 0.6 cycles, should not be exactly 0
        state.update_pulse(0.3, 2.0);
        assert!(
            state.pulse_phase > 0.0,
            "pulse_phase should be > 0, got {}",
            state.pulse_phase
        );
    }

    #[test]
    fn test_selection_state() {
        let mut state = SelectionState::new();
        assert!(!state.is_selected);

        state.select(5);
        assert!(state.is_selected);
        assert_eq!(state.selection_order, 5);

        state.deselect();
        assert!(!state.is_selected);
    }

    #[test]
    fn test_animation_state() {
        let mut anim = AnimationState::new();
        assert!(!anim.is_playing);

        anim.play(1, 2.0, false);
        assert!(anim.is_playing);
        assert_eq!(anim.duration, 2.0);

        anim.update(1.0);
        assert_eq!(anim.current_time, 1.0);
        assert!((anim.progress() - 0.5).abs() < 0.001);

        anim.update(1.5); // Should complete
        assert!(anim.completed);
        assert!(!anim.is_playing);
    }

    #[test]
    fn test_animation_looping() {
        let mut anim = AnimationState::new();
        anim.play(1, 1.0, true);

        // Complete one cycle
        anim.update(1.0);
        assert!(anim.is_playing); // Still playing due to loop
        assert!(anim.current_time < 1.0); // Wrapped around
    }
}
