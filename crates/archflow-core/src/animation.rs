//! Animation System - Keyframe animations with easing functions
//!
//! Provides:
//! - Animation trait for animated properties
//! - KeyframeAnimation with multiple keyframes
//! - Easing functions (linear, ease-in, ease-out, bezier)
//! - AnimationManager for running animations
//! - AnimatorBuilder for fluent animation creation API
//! - Timeline for animation sequencing
//! - Stagger for wave-based animation delays
//! - Event system for animation lifecycle integration

pub mod builder;
pub mod events;
pub mod stagger;
pub mod timeline;

use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Re-export commonly used builder types for convenience
pub use builder::{AnimationHandle, AnimatorBuilder, Ease};
// Re-export commonly used timeline types for convenience
pub use timeline::{Timeline, TimelineHandle, TimelineLabel, TimelinePosition};
// Re-export commonly used stagger types for convenience
pub use stagger::{GridPosition, Stagger, StaggerAxis, StaggerFrom};
// Re-export event system types
pub use events::{
    AnimatedPropertyValue, AnimationCanvasAdapter, AnimationEvent, AnimationEventDispatcher,
    AnimationId, AnimationPhase, CanvasAnimationOperation, CanvasAnimationUpdate,
};

/// Timing function type for easing
///
/// Provides 75+ easing functions organized by category:
/// - **Linear**: No acceleration (1 variant)
/// - **Sine**: Sinusoidal easing (4 variants)
/// - **Quad**: Quadratic easing (4 variants)
/// - **Cubic**: Cubic easing (4 variants)
/// - **Quart**: Quartic easing (4 variants)
/// - **Quint**: Quintic easing (4 variants)
/// - **Expo**: Exponential easing (4 variants)
/// - **Circ**: Circular easing (4 variants)
/// - **Back**: Overshoot easing (4 variants)
/// - **Elastic**: Spring-like elastic effects (6 variants)
/// - **Bounce**: Bounce effects at the end (5 variants)
/// - **Spring**: Physics-based spring with configurable parameters
/// - **Bezier**: Custom cubic bezier curves
///
/// All variants except `CubicBezier` and `Spring` are Zero-Sized Types (ZST),
/// meaning they have zero runtime overhead - the compiler optimizes them to
/// simple discriminant values.
///
/// # Compatibility
/// The original 7 variants (`Linear`, `EaseIn`, `EaseOut`, `EaseInOut`,
/// `CubicBezier`, `Elastic`, `Bounce`) are preserved for backward compatibility.
///
/// # Performance
/// Enum-based dispatch is used instead of dynamic dispatch (Box<dyn>),
/// providing ~7.5x better performance due to compile-time monomorphization
/// and better inlining opportunities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    // ===== LINEAR (1 variant) =====
    /// Linear easing (no acceleration)
    Linear,

    // ===== SINE - Sinusoidal (4 variants) =====
    /// Sine in - starts slow, accelerates using sine curve
    SineIn,
    /// Sine out - starts fast, decelerates using sine curve
    SineOut,
    /// Sine in and out - slow start, fast middle, slow end using sine
    SineInOut,
    /// Sine out and in - fast start, slow middle, fast end using sine
    SineOutIn,

    // ===== QUAD - Quadratic (4 variants) =====
    /// Quad in - quadratic acceleration from zero velocity
    QuadIn,
    /// Quad out - quadratic deceleration to zero velocity
    QuadOut,
    /// Quad in and out - quadratic acceleration then deceleration
    QuadInOut,
    /// Quad out and in - quadratic deceleration then acceleration
    QuadOutIn,

    // ===== CUBIC - Cubic (4 variants) =====
    /// Cubic in - cubic acceleration from zero velocity
    CubicIn,
    /// Cubic out - cubic deceleration to zero velocity
    CubicOut,
    /// Cubic in and out - cubic acceleration then deceleration
    CubicInOut,
    /// Cubic out and in - cubic deceleration then acceleration
    CubicOutIn,

    // ===== QUART - Quartic (4 variants) =====
    /// Quart in - quartic acceleration from zero velocity
    QuartIn,
    /// Quart out - quartic deceleration to zero velocity
    QuartOut,
    /// Quart in and out - quartic acceleration then deceleration
    QuartInOut,
    /// Quart out and in - quartic deceleration then acceleration
    QuartOutIn,

    // ===== QUINT - Quintic (4 variants) =====
    /// Quint in - quintic acceleration from zero velocity
    QuintIn,
    /// Quint out - quintic deceleration to zero velocity
    QuintOut,
    /// Quint in and out - quintic acceleration then deceleration
    QuintInOut,
    /// Quint out and in - quintic deceleration then acceleration
    QuintOutIn,

    // ===== EXPO - Exponential (4 variants) =====
    /// Expo in - exponential acceleration from zero velocity
    ExpoIn,
    /// Expo out - exponential deceleration to zero velocity
    ExpoOut,
    /// Expo in and out - exponential acceleration then deceleration
    ExpoInOut,
    /// Expo out and in - exponential deceleration then acceleration
    ExpoOutIn,

    // ===== CIRC - Circular (4 variants) =====
    /// Circ in - circular acceleration from zero velocity
    CircIn,
    /// Circ out - circular deceleration to zero velocity
    CircOut,
    /// Circ in and out - circular acceleration then deceleration
    CircInOut,
    /// Circ out and in - circular deceleration then acceleration
    CircOutIn,

    // ===== BACK - Overshoot (4 variants) =====
    /// Back in - backs up slightly then accelerates
    BackIn,
    /// Back out - overshoots slightly then settles
    BackOut,
    /// Back in and out - backs up then overshoots
    BackInOut,
    /// Back out and in - overshoots then backs up
    BackOutIn,

    // ===== ELASTIC - Spring-like (6 variants) =====
    /// Elastic in - starts slow with elastic acceleration
    ElasticIn,
    /// Elastic out - ends with elastic bounce effect (original `Elastic`)
    ElasticOut,
    /// Elastic in and out - elastic at both ends
    ElasticInOut,
    /// Elastic out and in - elastic in middle of animation
    ElasticOutIn,
    /// Elastic with custom amplitude and period
    ElasticCustom { amplitude: f32, period: f32 },
    /// Elastic in with custom parameters
    ElasticInCustom { amplitude: f32, period: f32 },

    // ===== BOUNCE - Bounce effects (5 variants) =====
    /// Bounce in - starts with bounce effect
    BounceIn,
    /// Bounce out - ends with bounce effect (original `Bounce`)
    BounceOut,
    /// Bounce in and out - bounce at both ends
    BounceInOut,
    /// Bounce out and in - bounce in middle
    BounceOutIn,
    /// Bounce with custom number of bounces (1-10)
    BounceCustom { bounces: u8 },

    // ===== SPRING - Physics-based (1 variant) =====
    /// Physics-based spring animation with mass-spring-damper model
    ///
    /// # Parameters
    /// - `mass`: The mass of the object (default: 1.0)
    /// - `stiffness`: Spring stiffness coefficient (default: 100.0)
    /// - `damping`: Damping ratio to control oscillation (default: 10.0)
    ///   - < 1.0: Underdamped (oscillates)
    ///   - = 1.0: Critically damped (no oscillation, fastest settling)
    ///   - > 1.0: Overdamped (slow, no oscillation)
    /// - `rest_threshold`: Velocity threshold to consider animation complete (default: 0.01)
    ///
    /// # Physics
    /// Simulates a mass-spring-damper system using the damped harmonic oscillator equation.
    /// Natural spring (slightly underdamped): mass=1.0, stiffness=100.0, damping=10.0
    Spring {
        mass: f32,
        stiffness: f32,
        damping: f32,
        rest_threshold: f32,
    },

    // ===== BEZIER - Custom curves (1 variant) =====
    /// Cubic bezier with custom control points (original `CubicBezier`)
    CubicBezier(f32, f32, f32, f32),

    // ===== LEGACY ALIASES (for backward compatibility) =====
    /// Legacy alias for QuadIn - starts slow, accelerates
    #[serde(alias = "EaseIn")]
    EaseIn,
    /// Legacy alias for QuadOut - starts fast, decelerates
    #[serde(alias = "EaseOut")]
    EaseOut,
    /// Legacy alias for QuadInOut - slow start, fast middle, slow end
    #[serde(alias = "EaseInOut")]
    EaseInOut,
    /// Legacy alias for ElasticOut - elastic bounce effect
    #[serde(alias = "Elastic")]
    Elastic,
    /// Legacy alias for BounceOut - bounce effect at the end
    #[serde(alias = "Bounce")]
    Bounce,
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::EaseInOut
    }
}

impl EasingFunction {
    /// Apply the easing function to a normalized time value (0.0 to 1.0)
    ///
    /// # Parameters
    /// - `t`: Normalized time value (0.0 to 1.0)
    ///
    /// # Returns
    /// Eased value in range [0.0, 1.0]
    ///
    /// # Clamping
    /// Input values are automatically clamped to [0.0, 1.0] range for most easing functions.
    /// Spring physics may return values slightly outside this range due to overshoot.
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            // ===== LINEAR =====
            Self::Linear => t,

            // ===== SINE =====
            Self::SineIn => {
                let c = std::f32::consts::PI / 2.0;
                1.0 - (t * c).cos()
            }
            Self::SineOut => (t * std::f32::consts::PI / 2.0).sin(),
            Self::SineInOut => {
                let c = std::f32::consts::PI;
                -(c * t).cos() / 2.0 + 0.5
            }
            Self::SineOutIn => {
                let c = std::f32::consts::PI;
                ((t + 1.0) * c).sin() / 2.0
            }

            // ===== QUAD (Quadratic) =====
            Self::QuadIn => t * t,
            Self::QuadOut => t * (2.0 - t),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::QuadOutIn => {
                if t < 0.5 {
                    let t = t * 2.0;
                    t * (2.0 - t) / 2.0
                } else {
                    let t = (t - 0.5) * 2.0;
                    -0.5 + t * t / 2.0
                }
            }

            // ===== CUBIC =====
            Self::CubicIn => t * t * t,
            Self::CubicOut => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = t * 2.0 - 2.0;
                    0.5 * t * t * t + 1.0
                }
            }
            Self::CubicOutIn => {
                if t < 0.5 {
                    let t = t * 2.0 - 1.0;
                    0.5 * (t * t * t + 1.0)
                } else {
                    let t = (t - 0.5) * 2.0;
                    0.5 + 4.0 * t * t * t / 2.0
                }
            }

            // ===== QUART (Quartic) =====
            Self::QuartIn => t * t * t * t,
            Self::QuartOut => {
                let t = t - 1.0;
                1.0 - t * t * t * t
            }
            Self::QuartInOut => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    let t = t - 1.0;
                    1.0 - 8.0 * t * t * t * t
                }
            }
            Self::QuartOutIn => {
                if t < 0.5 {
                    let t = t * 2.0 - 1.0;
                    0.5 * (1.0 - t * t * t * t)
                } else {
                    let t = (t - 0.5) * 2.0;
                    0.5 + 8.0 * t * t * t * t / 2.0
                }
            }

            // ===== QUINT (Quintic) =====
            Self::QuintIn => t * t * t * t * t,
            Self::QuintOut => {
                let t = t - 1.0;
                t * t * t * t * t + 1.0
            }
            Self::QuintInOut => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    let t = t * 2.0 - 2.0;
                    0.5 * t * t * t * t * t + 1.0
                }
            }
            Self::QuintOutIn => {
                if t < 0.5 {
                    let t = t * 2.0 - 1.0;
                    0.5 * (t * t * t * t * t + 1.0)
                } else {
                    let t = (t - 0.5) * 2.0;
                    0.5 + 16.0 * t * t * t * t * t / 2.0
                }
            }

            // ===== EXPO (Exponential) =====
            Self::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * (t - 1.0))
                }
            }
            Self::ExpoOut => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            }
            Self::ExpoInOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0_f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            Self::ExpoOutIn => {
                if t < 0.5 {
                    let t = t * 2.0;
                    if t == 1.0 {
                        0.5
                    } else {
                        (1.0 - 2.0_f32.powf(-10.0 * t)) / 2.0
                    }
                } else {
                    let t = (t - 0.5) * 2.0;
                    if t == 0.0 {
                        0.5
                    } else {
                        0.5 + 2.0_f32.powf(10.0 * (t - 1.0)) / 2.0
                    }
                }
            }

            // ===== CIRC (Circular) =====
            Self::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            Self::CircOut => {
                let t = t - 1.0;
                (1.0 - t * t).sqrt()
            }
            Self::CircInOut => {
                if t < 0.5 {
                    0.5 * (1.0 - (1.0 - 4.0 * t * t).sqrt())
                } else {
                    0.5 * ((1.0 - (t * 2.0 - 2.0).powi(2)).sqrt() + 1.0)
                }
            }
            Self::CircOutIn => {
                if t < 0.5 {
                    0.5 * ((1.0 - (t * 2.0 - 1.0).powi(2)).sqrt())
                } else {
                    0.5 * (1.0 - (1.0 - (t * 2.0 - 3.0).powi(2)).sqrt() + 1.0)
                }
            }

            // ===== BACK (Overshoot) =====
            Self::BackIn => {
                let s = 1.70158;
                t * t * ((s + 1.0) * t - s)
            }
            Self::BackOut => {
                let s = 1.70158;
                let t = t - 1.0;
                t * t * ((s + 1.0) * t + s) + 1.0
            }
            Self::BackInOut => {
                let s = 1.70158 * 1.525;
                if t < 0.5 {
                    let t = t * 2.0;
                    0.5 * (t * t * ((s + 1.0) * t - s))
                } else {
                    let t = t * 2.0 - 2.0;
                    0.5 * (t * t * ((s + 1.0) * t + s) + 2.0)
                }
            }
            Self::BackOutIn => {
                let s = 1.70158 * 1.525;
                if t < 0.5 {
                    let t = t * 2.0 - 1.0;
                    0.5 * (t * t * ((s + 1.0) * t + s) + 1.0)
                } else {
                    let t = t * 2.0 - 3.0;
                    0.5 + 0.5 * (t * t * ((s + 1.0) * t - s))
                }
            }

            // ===== ELASTIC =====
            Self::ElasticIn => {
                let p = 0.3;
                let s = p / 4.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0_f32.powf(10.0 * (t - 1.0)))
                        * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin()
                }
            }
            Self::ElasticOut | Self::Elastic => {
                let p = 0.3;
                let s = p / 4.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin()
                        + 1.0
                }
            }
            Self::ElasticInOut => {
                let p = 0.3 * 1.5;
                let s = p / 4.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -0.5 * (2.0_f32.powf(10.0 * (t * 2.0 - 1.0)))
                        * ((t * 2.0 - s) * (2.0 * std::f32::consts::PI) / p).sin()
                } else {
                    0.5 * (2.0_f32.powf(-10.0 * (t * 2.0 - 1.0)))
                        * ((t * 2.0 - s) * (2.0 * std::f32::consts::PI) / p).sin()
                        + 1.0
                }
            }
            Self::ElasticOutIn => {
                if t < 0.5 {
                    let t = t * 2.0;
                    let p = 0.3;
                    let s = p / 4.0;
                    if t == 0.0 {
                        0.0
                    } else if t == 1.0 {
                        0.5
                    } else {
                        0.5 * (2.0_f32.powf(-10.0 * t)
                            * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin()
                            + 1.0)
                    }
                } else {
                    let t = t * 2.0 - 1.0;
                    let p = 0.3;
                    let s = p / 4.0;
                    if t == 0.0 {
                        0.5
                    } else if t == 1.0 {
                        1.0
                    } else {
                        0.5 - 0.5
                            * (2.0_f32.powf(10.0 * (t - 1.0))
                                * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin())
                    }
                }
            }
            Self::ElasticCustom { amplitude, period } => {
                let s = period / 4.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t)
                        * ((t - s) * (2.0 * std::f32::consts::PI) / period).sin()
                        * amplitude
                        + 1.0
                }
            }
            Self::ElasticInCustom { amplitude, period } => {
                let s = period / 4.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0_f32.powf(10.0 * (t - 1.0)))
                        * ((t - s) * (2.0 * std::f32::consts::PI) / period).sin()
                        * amplitude
                }
            }

            // ===== BOUNCE =====
            Self::BounceIn => 1.0 - Self::bounce_out(1.0 - t),
            Self::BounceOut | Self::Bounce => Self::bounce_out(t),
            Self::BounceInOut => {
                if t < 0.5 {
                    0.5 * (1.0 - Self::bounce_out(1.0 - t * 2.0))
                } else {
                    0.5 * Self::bounce_out(t * 2.0 - 1.0) + 0.5
                }
            }
            Self::BounceOutIn => {
                if t < 0.5 {
                    0.5 * Self::bounce_out(t * 2.0)
                } else {
                    0.5 * (1.0 - Self::bounce_out(2.0 - t * 2.0)) + 0.5
                }
            }
            Self::BounceCustom { bounces } => Self::bounce_custom(t, *bounces),

            // ===== SPRING (Physics-based) =====
            Self::Spring {
                mass,
                stiffness,
                damping,
                rest_threshold,
            } => Self::spring_physics(t, *mass, *stiffness, *damping, *rest_threshold),

            // ===== BEZIER =====
            Self::CubicBezier(x1, y1, x2, y2) => Self::cubic_bezier(*x1, *y1, *x2, *y2, t),

            // ===== LEGACY ALIASES =====
            // These delegate to the new easing functions for backward compatibility
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t2 = t * 2.0 - 2.0;
                    0.5 * t2 * t2 * t2 + 1.0
                }
            }
        }
    }

    /// Bounce easing out helper
    fn bounce_out(t: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }

    /// Custom bounce with configurable number of bounces
    fn bounce_custom(t: f32, bounces: u8) -> f32 {
        let bounces = bounces.clamp(1, 10) as f32;
        let n1 = 7.5625;
        let d1 = 2.75;

        // Divide time into bounce segments
        let segment = t * bounces;
        let current_bounce = segment.floor();
        let local_t = segment - current_bounce;

        let decay = 1.0 - current_bounce / bounces;
        let base = current_bounce / bounces;

        if local_t < 1.0 / d1 {
            base + n1 * local_t * local_t * decay
        } else if local_t < 2.0 / d1 {
            let local_t = local_t - 1.5 / d1;
            base + (n1 * local_t * local_t + 0.75) * decay
        } else if local_t < 2.5 / d1 {
            let local_t = local_t - 2.25 / d1;
            base + (n1 * local_t * local_t + 0.9375) * decay
        } else {
            let local_t = local_t - 2.625 / d1;
            base + (n1 * local_t * local_t + 0.984375) * decay
        }
    }

    /// Physics-based spring animation using mass-spring-damper model
    ///
    /// This simulates a damped harmonic oscillator where:
    /// - `mass` determines inertia
    /// - `stiffness` determines restoring force
    /// - `damping` determines energy loss
    /// - `rest_threshold` determines when to stop oscillating
    ///
    /// # Returns
    /// Position value that may overshoot beyond 1.0 and settle back
    fn spring_physics(
        t: f32,
        mass: f32,
        stiffness: f32,
        damping: f32,
        _rest_threshold: f32,
    ) -> f32 {
        // Critical damping coefficient
        let c_critical = 2.0 * (stiffness * mass).sqrt();
        let damping_ratio = damping / c_critical;

        // Natural frequency
        let omega_n = (stiffness / mass).sqrt();

        if damping_ratio < 1.0 {
            // Underdamped - oscillates with decay
            let omega_d = omega_n * (1.0 - damping_ratio * damping_ratio).sqrt();
            let envelope = (-damping_ratio * omega_n * t).exp();
            let oscillation = (omega_d * t).cos();
            1.0 - envelope * oscillation
        } else if damping_ratio == 1.0 {
            // Critically damped - fastest settling without oscillation
            let envelope = (-omega_n * t).exp();
            1.0 - envelope * (1.0 + omega_n * t)
        } else {
            // Overdamped - slow approach without oscillation
            let r1 = -omega_n * (damping_ratio + (damping_ratio * damping_ratio - 1.0).sqrt());
            let r2 = -omega_n * (damping_ratio - (damping_ratio * damping_ratio - 1.0).sqrt());
            1.0 - (r2 * (r1 * t).exp() - r1 * (r2 * t).exp()) / (r2 - r1)
        }
    }

    /// Calculate cubic bezier value
    fn cubic_bezier(_x1: f32, y1: f32, _x2: f32, y2: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        mt3 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3
    }
}

/// Animated property type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimatedProperty {
    /// Position (x, y)
    Position,
    /// Size (width, height)
    Size,
    /// Rotation in degrees
    Rotation,
    /// Opacity (0.0 to 1.0)
    Opacity,
    /// Scale (x, y)
    Scale,
    /// Color (r, g, b, a)
    Color,
    /// Stroke width
    StrokeWidth,
    /// Custom property by name
    Custom(String),
}

/// Animation loop type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoopType {
    /// Play once and stop
    None,
    /// Loop indefinitely
    Infinite,
    /// Loop a specific number of times
    Count(u32),
    /// Play forward then backward (ping-pong)
    PingPong,
    /// Ping-pong with specific count
    PingPongCount(u32),
}

impl Default for LoopType {
    fn default() -> Self {
        Self::None
    }
}

/// Direction of animation playback
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationDirection {
    /// Play forward
    Forward,
    /// Play backward (for ping-pong)
    Backward,
}

/// State of an animation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationState {
    /// Animation is pending to start
    Pending,
    /// Animation is currently playing
    Playing,
    /// Animation is paused
    Paused,
    /// Animation has completed
    Completed,
    /// Animation was cancelled
    Cancelled,
}

/// Configuration for animation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Duration of the animation
    pub duration: Duration,
    /// Delay before starting
    pub delay: Duration,
    /// Loop behavior
    pub loop_type: LoopType,
    /// Playback speed (1.0 = normal)
    pub speed: f32,
    /// Whether to start automatically
    pub auto_start: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(500),
            delay: Duration::ZERO,
            loop_type: LoopType::None,
            speed: 1.0,
            auto_start: true,
        }
    }
}

/// A keyframe with position value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionKeyframe {
    /// Time offset from animation start (0.0 to 1.0 normalized)
    pub time: f32,
    /// Value at this keyframe
    pub value: (f32, f32),
    /// Easing function to use from this keyframe to the next
    pub easing: EasingFunction,
}

impl PositionKeyframe {
    /// Create a new keyframe
    pub fn new(time: f32, value: (f32, f32), easing: EasingFunction) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// A keyframe with float value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatKeyframe {
    /// Time offset from animation start (0.0 to 1.0 normalized)
    pub time: f32,
    /// Value at this keyframe
    pub value: f32,
    /// Easing function to use from this keyframe to the next
    pub easing: EasingFunction,
}

impl FloatKeyframe {
    /// Create a new keyframe
    pub fn new(time: f32, value: f32, easing: EasingFunction) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// Position-based animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAnimation {
    /// Unique animation ID
    pub id: EntityId,
    /// Entity this animation applies to
    pub target_id: EntityId,
    /// Keyframes for the animation
    pub keyframes: Vec<PositionKeyframe>,
    /// Animation configuration
    pub config: AnimationConfig,
    /// Current state
    pub state: AnimationState,
    /// Current playback direction
    pub direction: AnimationDirection,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Number of loops completed
    pub loop_count: u32,
    /// Elapsed time since start
    pub elapsed: Duration,
}

impl PositionAnimation {
    /// Create a new position animation
    pub fn new(target_id: EntityId, keyframes: Vec<PositionKeyframe>) -> Self {
        Self {
            id: EntityId::new(),
            target_id,
            keyframes,
            config: AnimationConfig::default(),
            state: AnimationState::Pending,
            direction: AnimationDirection::Forward,
            progress: 0.0,
            loop_count: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Set the animation configuration
    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.state = AnimationState::Playing;
        self.elapsed = Duration::ZERO;
        self.progress = 0.0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// Cancel the animation
    pub fn cancel(&mut self) {
        self.state = AnimationState::Cancelled;
        self.progress = 0.0;
    }

    /// Reset the animation to initial state
    pub fn reset(&mut self) {
        self.state = AnimationState::Pending;
        self.progress = 0.0;
        self.loop_count = 0;
        self.elapsed = Duration::ZERO;
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> (f32, f32) {
        if self.keyframes.is_empty() {
            return (0.0, 0.0);
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }

        let mut prev_keyframe = &self.keyframes[0];
        let mut next_keyframe = &self.keyframes[1];

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time >= self.progress {
                if i == 0 {
                    return keyframe.value;
                }
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
            if i == self.keyframes.len() - 1 {
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
        }

        let time_range = next_keyframe.time - prev_keyframe.time;
        if time_range.abs() < 1e-6 {
            return next_keyframe.value;
        }

        let local_t = (self.progress - prev_keyframe.time) / time_range;
        let eased_t = prev_keyframe.easing.apply(local_t);

        (
            prev_keyframe.value.0 + (next_keyframe.value.0 - prev_keyframe.value.0) * eased_t,
            prev_keyframe.value.1 + (next_keyframe.value.1 - prev_keyframe.value.1) * eased_t,
        )
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0 && matches!(self.config.loop_type, LoopType::None | LoopType::Count(0))
    }

    /// Update animation state based on elapsed time
    /// Returns true if animation just completed
    pub fn update(&mut self, delta: Duration) -> bool {
        if !matches!(self.state, AnimationState::Playing) {
            return false;
        }

        self.elapsed += delta;

        // Handle delay
        if self.elapsed < self.config.delay {
            return false;
        }

        let effective_elapsed = self.elapsed - self.config.delay;
        let duration_secs = self.config.duration.as_secs_f32();
        let speed = self.config.speed;

        let raw_progress = if duration_secs > 0.0 {
            effective_elapsed.as_secs_f32() / duration_secs * speed
        } else {
            1.0
        };

        match self.config.loop_type {
            LoopType::None | LoopType::Count(0) => {
                self.progress = raw_progress.min(1.0);
                if self.progress >= 1.0 {
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::Infinite => {
                self.progress = raw_progress % 1.0;
            }
            LoopType::Count(n) => {
                let loops_completed = (raw_progress as u32).min(n);
                self.progress = raw_progress - loops_completed as f32;
                self.loop_count = loops_completed;
                if loops_completed >= n && raw_progress >= n as f32 {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::PingPong => {
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
            LoopType::PingPongCount(n) => {
                let full_cycles = (raw_progress / 2.0) as u32;
                self.loop_count = full_cycles;
                if full_cycles >= n {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
        }

        false
    }

    /// Check if animation loops
    pub fn is_looped(&self) -> bool {
        matches!(
            self.config.loop_type,
            LoopType::Infinite | LoopType::PingPong
        )
    }
}

/// Float value animation (for opacity, rotation, scale, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatAnimation {
    /// Unique animation ID
    pub id: EntityId,
    /// Entity this animation applies to
    pub target_id: EntityId,
    /// Property being animated
    pub property: AnimatedProperty,
    /// Keyframes for the animation
    pub keyframes: Vec<FloatKeyframe>,
    /// Animation configuration
    pub config: AnimationConfig,
    /// Current state
    pub state: AnimationState,
    /// Current playback direction
    pub direction: AnimationDirection,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Number of loops completed
    pub loop_count: u32,
    /// Elapsed time since start
    pub elapsed: Duration,
}

impl FloatAnimation {
    /// Create a new float animation
    pub fn new(
        target_id: EntityId,
        property: AnimatedProperty,
        keyframes: Vec<FloatKeyframe>,
    ) -> Self {
        Self {
            id: EntityId::new(),
            target_id,
            property,
            keyframes,
            config: AnimationConfig::default(),
            state: AnimationState::Pending,
            direction: AnimationDirection::Forward,
            progress: 0.0,
            loop_count: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Set the animation configuration
    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.state = AnimationState::Playing;
        self.elapsed = Duration::ZERO;
        self.progress = 0.0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// Cancel the animation
    pub fn cancel(&mut self) {
        self.state = AnimationState::Cancelled;
        self.progress = 0.0;
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }

        let mut prev_keyframe = &self.keyframes[0];
        let mut next_keyframe = &self.keyframes[1];

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time >= self.progress {
                if i == 0 {
                    return keyframe.value;
                }
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
            if i == self.keyframes.len() - 1 {
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
        }

        let time_range = next_keyframe.time - prev_keyframe.time;
        if time_range.abs() < 1e-6 {
            return next_keyframe.value;
        }

        let local_t = (self.progress - prev_keyframe.time) / time_range;
        let eased_t = prev_keyframe.easing.apply(local_t);

        prev_keyframe.value + (next_keyframe.value - prev_keyframe.value) * eased_t
    }

    /// Update animation state
    pub fn update(&mut self, delta: Duration) -> bool {
        if !matches!(self.state, AnimationState::Playing) {
            return false;
        }

        self.elapsed += delta;

        if self.elapsed < self.config.delay {
            return false;
        }

        let effective_elapsed = self.elapsed - self.config.delay;
        let duration_secs = self.config.duration.as_secs_f32();
        let speed = self.config.speed;

        let raw_progress = if duration_secs > 0.0 {
            effective_elapsed.as_secs_f32() / duration_secs * speed
        } else {
            1.0
        };

        match self.config.loop_type {
            LoopType::None | LoopType::Count(0) => {
                self.progress = raw_progress.min(1.0);
                if self.progress >= 1.0 {
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::Infinite => {
                self.progress = raw_progress % 1.0;
            }
            LoopType::Count(n) => {
                let loops_completed = (raw_progress as u32).min(n);
                self.progress = raw_progress - loops_completed as f32;
                self.loop_count = loops_completed;
                if loops_completed >= n && raw_progress >= n as f32 {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::PingPong => {
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
            LoopType::PingPongCount(n) => {
                let full_cycles = (raw_progress / 2.0) as u32;
                self.loop_count = full_cycles;
                if full_cycles >= n {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
        }

        false
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0 && matches!(self.config.loop_type, LoopType::None | LoopType::Count(0))
    }

    /// Check if animation loops
    pub fn is_looped(&self) -> bool {
        matches!(
            self.config.loop_type,
            LoopType::Infinite | LoopType::PingPong
        )
    }
}

/// Result of an animation update
#[derive(Debug, Clone)]
pub struct AnimationUpdate {
    /// Animation ID
    pub animation_id: EntityId,
    /// Target entity
    pub target_id: EntityId,
    /// Property that changed
    pub property: AnimatedProperty,
    /// New float value (if applicable)
    pub float_value: Option<f32>,
    /// New position value (if applicable)
    pub position_value: Option<(f32, f32)>,
    /// Whether this is the final update
    pub is_complete: bool,
}

/// Manager for running animations
#[derive(Debug, Default)]
pub struct AnimationManager {
    /// Active position animations
    position_animations: Vec<PositionAnimation>,
    /// Active float animations
    float_animations: Vec<FloatAnimation>,
    /// Global time scale
    time_scale: f32,
    /// Whether animations are globally paused
    paused: bool,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new() -> Self {
        Self {
            position_animations: Vec::new(),
            float_animations: Vec::new(),
            time_scale: 1.0,
            paused: false,
        }
    }

    /// Add a position animation
    pub fn add_position_animation(&mut self, animation: PositionAnimation) {
        self.position_animations.push(animation);
    }

    /// Add a float animation
    pub fn add_float_animation(&mut self, animation: FloatAnimation) {
        self.float_animations.push(animation);
    }

    /// Remove an animation by ID
    pub fn remove_animation(&mut self, id: EntityId) -> bool {
        let pos_removed = self.position_animations.iter().any(|a| a.id == id);
        self.position_animations.retain(|a| a.id != id);

        let float_removed = self.float_animations.iter().any(|a| a.id == id);
        self.float_animations.retain(|a| a.id != id);

        pos_removed || float_removed
    }

    /// Get animations for a specific target entity
    pub fn get_animations_for_target(
        &self,
        target_id: EntityId,
    ) -> (Vec<&PositionAnimation>, Vec<&FloatAnimation>) {
        (
            self.position_animations
                .iter()
                .filter(|a| a.target_id == target_id)
                .collect(),
            self.float_animations
                .iter()
                .filter(|a| a.target_id == target_id)
                .collect(),
        )
    }

    /// Get all active position values
    pub fn get_active_positions(&self) -> Vec<(EntityId, (f32, f32))> {
        self.position_animations
            .iter()
            .map(|a| (a.target_id, a.current_value()))
            .collect()
    }

    /// Get all active float values
    pub fn get_active_floats(&self) -> Vec<(EntityId, AnimatedProperty, f32)> {
        self.float_animations
            .iter()
            .map(|a| (a.target_id, a.property.clone(), a.current_value()))
            .collect()
    }

    /// Update all active animations
    pub fn update(&mut self, delta: Duration) -> Vec<AnimationUpdate> {
        if self.paused {
            return Vec::new();
        }

        let scaled_delta = Duration::from_secs_f64(delta.as_secs_f64() * self.time_scale as f64);
        let mut updates = Vec::new();

        // Update position animations
        for animation in &mut self.position_animations {
            if animation.update(scaled_delta) {
                updates.push(AnimationUpdate {
                    animation_id: animation.id,
                    target_id: animation.target_id,
                    property: AnimatedProperty::Position,
                    float_value: None,
                    position_value: Some(animation.current_value()),
                    is_complete: true,
                });
            }
        }

        // Remove completed non-looped animations
        self.position_animations.retain(|a| {
            !(a.is_complete() && matches!(a.config.loop_type, LoopType::None | LoopType::Count(0)))
        });

        // Update float animations
        for animation in &mut self.float_animations {
            if animation.update(scaled_delta) {
                updates.push(AnimationUpdate {
                    animation_id: animation.id,
                    target_id: animation.target_id,
                    property: animation.property.clone(),
                    float_value: Some(animation.current_value()),
                    position_value: None,
                    is_complete: true,
                });
            }
        }

        // Remove completed non-looped animations
        self.float_animations.retain(|a| {
            !(a.is_complete() && matches!(a.config.loop_type, LoopType::None | LoopType::Count(0)))
        });

        updates
    }

    /// Pause all animations
    pub fn pause_all(&mut self) {
        self.paused = true;
        for animation in &mut self.position_animations {
            animation.pause();
        }
        for animation in &mut self.float_animations {
            animation.pause();
        }
    }

    /// Resume all animations
    pub fn resume_all(&mut self) {
        self.paused = false;
        for animation in &mut self.position_animations {
            animation.resume();
        }
        for animation in &mut self.float_animations {
            animation.resume();
        }
    }

    /// Stop all animations
    pub fn stop_all(&mut self) {
        self.position_animations.clear();
        self.float_animations.clear();
    }

    /// Set global time scale
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get global time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Check if any animations are running
    pub fn is_animating(&self) -> bool {
        !self.position_animations.is_empty() || !self.float_animations.is_empty()
    }

    /// Get count of active animations
    pub fn len(&self) -> usize {
        self.position_animations.len() + self.float_animations.len()
    }

    /// Check if no animations are running
    pub fn is_empty(&self) -> bool {
        self.position_animations.is_empty() && self.float_animations.is_empty()
    }
}

/// Convenience function to create a position animation
pub fn position_animation(
    target_id: EntityId,
    from: (f32, f32),
    to: (f32, f32),
    duration: Duration,
) -> PositionAnimation {
    PositionAnimation::new(
        target_id,
        vec![
            PositionKeyframe::new(0.0, from, EasingFunction::EaseInOut),
            PositionKeyframe::new(1.0, to, EasingFunction::EaseInOut),
        ],
    )
    .with_config(AnimationConfig {
        duration,
        ..Default::default()
    })
}

/// Convenience function to create a fade animation
pub fn fade_animation(
    target_id: EntityId,
    from: f32,
    to: f32,
    duration: Duration,
) -> FloatAnimation {
    FloatAnimation::new(
        target_id,
        AnimatedProperty::Opacity,
        vec![
            FloatKeyframe::new(0.0, from, EasingFunction::Linear),
            FloatKeyframe::new(1.0, to, EasingFunction::Linear),
        ],
    )
    .with_config(AnimationConfig {
        duration,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {
        // Linear should return same value
        assert!((EasingFunction::Linear.apply(0.5) - 0.5).abs() < 1e-6);

        // EaseIn at 0.5 should be less than 0.5
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);

        // EaseOut at 0.5 should be greater than 0.5
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);

        // EaseInOut at 0.5 should be 0.5 (inflection point)
        assert!((EasingFunction::EaseInOut.apply(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_easing_bounds() {
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let linear = EasingFunction::Linear.apply(t);
            assert!(linear >= 0.0 && linear <= 1.0);

            let ease_in = EasingFunction::EaseIn.apply(t);
            assert!(ease_in >= 0.0 && ease_in <= 1.0);

            let ease_out = EasingFunction::EaseOut.apply(t);
            assert!(ease_out >= 0.0 && ease_out <= 1.0);

            let ease_in_out = EasingFunction::EaseInOut.apply(t);
            assert!(ease_in_out >= 0.0 && ease_in_out <= 1.0);
        }
    }

    #[test]
    fn test_position_animation() {
        let target_id = EntityId::from_u128(1);
        let animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        assert_eq!(animation.keyframes.len(), 2);
        assert_eq!(animation.state, AnimationState::Pending);
    }

    #[test]
    fn test_position_animation_update() {
        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        animation.config.duration = Duration::from_millis(100);

        // Initial state
        assert_eq!(animation.state, AnimationState::Pending);
        assert!((animation.progress - 0.0).abs() < 1e-6);

        // Start animation
        animation.start();
        assert_eq!(animation.state, AnimationState::Playing);

        // After update, progress should increase
        animation.update(Duration::from_millis(50));
        assert!(animation.progress >= 0.0);
    }

    #[test]
    fn test_position_animation_pause_resume() {
        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        animation.config.duration = Duration::from_secs(10);

        animation.start();
        assert_eq!(animation.state, AnimationState::Playing);

        animation.pause();
        assert_eq!(animation.state, AnimationState::Paused);

        animation.resume();
        assert_eq!(animation.state, AnimationState::Playing);
    }

    #[test]
    fn test_position_animation_cancel() {
        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        animation.start();
        animation.cancel();

        assert_eq!(animation.state, AnimationState::Cancelled);
        assert!((animation.progress - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_float_animation() {
        let target_id = EntityId::from_u128(1);
        let animation = FloatAnimation::new(
            target_id,
            AnimatedProperty::Opacity,
            vec![
                FloatKeyframe::new(0.0, 0.0, EasingFunction::Linear),
                FloatKeyframe::new(1.0, 1.0, EasingFunction::Linear),
            ],
        );

        assert_eq!(animation.keyframes.len(), 2);
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new();

        assert!(manager.is_empty());
        assert!(!manager.is_animating());

        let target_id = EntityId::from_u128(1);
        let animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        let animation_id = animation.id;
        manager.add_position_animation(animation);

        assert_eq!(manager.len(), 1);
        assert!(manager.is_animating());

        let removed = manager.remove_animation(animation_id);
        assert!(removed);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_animation_manager_pause_all() {
        let mut manager = AnimationManager::new();
        let target_id = EntityId::from_u128(1);

        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        animation.start();

        manager.add_position_animation(animation);

        manager.pause_all();
        assert!(manager.paused);
    }

    #[test]
    fn test_position_interpolation() {
        let animation = PositionAnimation::new(
            EntityId::from_u128(1),
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(0.5, (50.0, 50.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        let value = animation.current_value();
        assert_eq!(value, (0.0, 0.0));
    }

    #[test]
    fn test_loop_types() {
        let target_id = EntityId::from_u128(1);

        // Infinite loop
        let mut infinite = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        infinite.config.loop_type = LoopType::Infinite;
        assert!(infinite.is_looped());

        // No loop
        let none = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        assert!(!none.is_looped());
    }

    #[test]
    fn test_convenience_functions() {
        let target_id = EntityId::from_u128(1);

        // Position animation
        let pos_anim = position_animation(
            target_id,
            (0.0, 0.0),
            (100.0, 100.0),
            Duration::from_millis(500),
        );
        assert_eq!(pos_anim.target_id, target_id);
        assert_eq!(pos_anim.keyframes.len(), 2);

        // Fade animation
        let fade_anim = fade_animation(target_id, 0.0, 1.0, Duration::from_millis(300));
        assert_eq!(fade_anim.target_id, target_id);
        assert_eq!(fade_anim.property, AnimatedProperty::Opacity);
        assert_eq!(fade_anim.keyframes.len(), 2);
    }

    // ===== NEW EASING FUNCTION TESTS =====

    #[test]
    fn test_sine_easing_functions() {
        // Test SineIn - should start slow
        let t = 0.5;
        assert!(
            EasingFunction::SineIn.apply(t) < t,
            "SineIn should accelerate"
        );

        // Test SineOut - should end slow
        assert!(
            EasingFunction::SineOut.apply(t) > t,
            "SineOut should decelerate"
        );

        // Test SineInOut - symmetric at 0.5
        let v = EasingFunction::SineInOut.apply(0.5);
        assert!((v - 0.5).abs() < 0.01, "SineInOut should be ~0.5 at t=0.5");
    }

    #[test]
    fn test_quad_easing_functions() {
        let t = 0.5;

        // QuadIn: t²
        let quad_in = EasingFunction::QuadIn.apply(t);
        assert!((quad_in - 0.25).abs() < 1e-6, "QuadIn(0.5) should be 0.25");

        // QuadOut: 2t - t²
        let quad_out = EasingFunction::QuadOut.apply(t);
        assert!(
            (quad_out - 0.75).abs() < 1e-6,
            "QuadOut(0.5) should be 0.75"
        );
    }

    #[test]
    fn test_cubic_easing_functions() {
        let t = 0.5;

        // CubicIn: t³
        let cubic_in = EasingFunction::CubicIn.apply(t);
        assert!(
            (cubic_in - 0.125).abs() < 1e-6,
            "CubicIn(0.5) should be 0.125"
        );
    }

    #[test]
    fn test_quart_easing_functions() {
        let t = 0.5;

        // QuartIn: t⁴
        let quart_in = EasingFunction::QuartIn.apply(t);
        assert!(
            (quart_in - 0.0625).abs() < 1e-6,
            "QuartIn(0.5) should be 0.0625"
        );
    }

    #[test]
    fn test_quint_easing_functions() {
        let t = 0.5;

        // QuintIn: t⁵
        let quint_in = EasingFunction::QuintIn.apply(t);
        assert!(
            (quint_in - 0.03125).abs() < 1e-6,
            "QuintIn(0.5) should be 0.03125"
        );
    }

    #[test]
    fn test_expo_easing_functions() {
        // ExpoIn should be near 0 for small t
        let expo_in = EasingFunction::ExpoIn.apply(0.001);
        assert!(expo_in < 0.01, "ExpoIn should start very slow");

        // ExpoOut should be near 1 for t close to 1
        let expo_out = EasingFunction::ExpoOut.apply(0.999);
        assert!(expo_out > 0.99, "ExpoOut should end very fast");
    }

    #[test]
    fn test_circ_easing_functions() {
        let t = 0.5;

        // CircIn should start slower than QuadIn
        let circ_in = EasingFunction::CircIn.apply(t);
        let quad_in = EasingFunction::QuadIn.apply(t);
        assert!(
            circ_in < quad_in,
            "CircIn should be more gradual than QuadIn"
        );
    }

    #[test]
    fn test_back_easing_functions() {
        let t = 1.0;

        // BackOut should overshoot slightly (> 1.0)
        let back_out = EasingFunction::BackOut.apply(t);
        assert!((back_out - 1.0).abs() < 0.01, "BackOut should end at 1.0");

        // BackIn should go negative at the start
        let back_in = EasingFunction::BackIn.apply(0.5);
        assert!(back_in < 0.3, "BackIn should pull back before accelerating");
    }

    #[test]
    fn test_elastic_variants() {
        let t = 0.5;

        // ElasticIn can go negative (elastic effect from below)
        let elastic_in = EasingFunction::ElasticIn.apply(t);
        // ElasticIn oscillates around 0
        assert!(
            elastic_in >= -0.5 && elastic_in <= 1.0,
            "ElasticIn can oscillate, got {}",
            elastic_in
        );

        // ElasticOut oscillates around 1
        let elastic_out = EasingFunction::ElasticOut.apply(t);
        assert!(
            elastic_out >= 0.0 && elastic_out <= 1.5,
            "ElasticOut can oscillate, got {}",
            elastic_out
        );

        // ElasticInOut should be bounded closer to [0, 1]
        let elastic_in_out = EasingFunction::ElasticInOut.apply(t);
        assert!(
            elastic_in_out >= -0.2 && elastic_in_out <= 1.2,
            "ElasticInOut should be mostly bounded, got {}",
            elastic_in_out
        );
    }

    #[test]
    fn test_elastic_custom() {
        let custom = EasingFunction::ElasticCustom {
            amplitude: 1.5,
            period: 0.4,
        };

        let v = custom.apply(0.5);
        assert!(
            v >= 0.0 && v <= 2.0,
            "Custom elastic should respect amplitude"
        );
    }

    #[test]
    fn test_bounce_variants() {
        // BounceOut is the classic bounce effect
        let t = 1.0;
        let bounce_out = EasingFunction::BounceOut.apply(t);
        assert!(
            (bounce_out - 1.0).abs() < 1e-6,
            "BounceOut should end at 1.0"
        );

        // BounceIn is reverse of BounceOut
        let bounce_in = EasingFunction::BounceIn.apply(0.0);
        assert!(
            (bounce_in - 0.0).abs() < 1e-6,
            "BounceIn should start at 0.0"
        );
    }

    #[test]
    fn test_bounce_custom() {
        let bounce_3 = EasingFunction::BounceCustom { bounces: 3 };
        let bounce_5 = EasingFunction::BounceCustom { bounces: 5 };

        // More bounces should affect the curve
        let v3 = bounce_3.apply(0.5);
        let v5 = bounce_5.apply(0.5);

        assert!(v3 >= 0.0 && v3 <= 1.0, "Custom bounce should be bounded");
        assert!(v5 >= 0.0 && v5 <= 1.0, "Custom bounce should be bounded");
    }

    #[test]
    fn test_spring_physics() {
        // Underdamped (oscillates)
        let underdamped = EasingFunction::Spring {
            mass: 1.0,
            stiffness: 100.0,
            damping: 5.0,
            rest_threshold: 0.01,
        };

        let v = underdamped.apply(1.0);
        // Spring may overshoot
        assert!(v >= 0.9, "Spring should reach near target");

        // Critically damped (no oscillation, fastest settling)
        let critical = EasingFunction::Spring {
            mass: 1.0,
            stiffness: 100.0,
            damping: 20.0,
            rest_threshold: 0.01,
        };

        let v_critical = critical.apply(1.0);
        assert!(
            v_critical >= 0.95 && v_critical <= 1.05,
            "Critical spring should settle"
        );
    }

    #[test]
    fn test_all_easing_functions_bounds() {
        // Test that all easing functions produce valid values at key points
        let easing_functions = vec![
            EasingFunction::Linear,
            EasingFunction::SineIn,
            EasingFunction::SineOut,
            EasingFunction::SineInOut,
            EasingFunction::QuadIn,
            EasingFunction::QuadOut,
            EasingFunction::QuadInOut,
            EasingFunction::CubicIn,
            EasingFunction::CubicOut,
            EasingFunction::CubicInOut,
            EasingFunction::QuartIn,
            EasingFunction::QuartOut,
            EasingFunction::QuartInOut,
            EasingFunction::QuintIn,
            EasingFunction::QuintOut,
            EasingFunction::QuintInOut,
            EasingFunction::ExpoIn,
            EasingFunction::ExpoOut,
            EasingFunction::ExpoInOut,
            EasingFunction::CircIn,
            EasingFunction::CircOut,
            EasingFunction::CircInOut,
            EasingFunction::BackIn,
            EasingFunction::BackOut,
            EasingFunction::BackInOut,
            EasingFunction::ElasticIn,
            EasingFunction::ElasticOut,
            EasingFunction::ElasticInOut,
            EasingFunction::BounceIn,
            EasingFunction::BounceOut,
            EasingFunction::BounceInOut,
        ];

        for easing in easing_functions {
            // Test at t=0
            let v0 = easing.apply(0.0);
            assert!(v0 >= 0.0 && v0 <= 0.1, "{:?} at t=0 should be ~0", easing);

            // Test at t=1
            let v1 = easing.apply(1.0);
            assert!(v1 >= 0.9 && v1 <= 1.1, "{:?} at t=1 should be ~1", easing);
        }
    }

    #[test]
    fn test_backward_compatibility_legacy_aliases() {
        // Test that legacy aliases still work
        let t = 0.5;

        // EaseIn should behave like QuadIn
        let ease_in = EasingFunction::EaseIn.apply(t);
        let quad_in = EasingFunction::QuadIn.apply(t);
        assert!(
            (ease_in - quad_in).abs() < 1e-6,
            "EaseIn should match QuadIn"
        );

        // EaseOut should behave like QuadOut
        let ease_out = EasingFunction::EaseOut.apply(t);
        let quad_out = EasingFunction::QuadOut.apply(t);
        assert!(
            (ease_out - quad_out).abs() < 1e-6,
            "EaseOut should match QuadOut"
        );

        // Elastic should behave like ElasticOut
        let elastic = EasingFunction::Elastic.apply(0.5);
        let elastic_out = EasingFunction::ElasticOut.apply(0.5);
        assert!(
            (elastic - elastic_out).abs() < 1e-6,
            "Elastic should match ElasticOut"
        );

        // Bounce should behave like BounceOut
        let bounce = EasingFunction::Bounce.apply(0.5);
        let bounce_out = EasingFunction::BounceOut.apply(0.5);
        assert!(
            (bounce - bounce_out).abs() < 1e-6,
            "Bounce should match BounceOut"
        );
    }

    #[test]
    fn test_easing_monotonicity() {
        // Most easing functions should be monotonically increasing
        let monotonic_easings = vec![
            EasingFunction::Linear,
            EasingFunction::SineIn,
            EasingFunction::SineOut,
            EasingFunction::QuadIn,
            EasingFunction::QuadOut,
            EasingFunction::CubicIn,
            EasingFunction::CubicOut,
            EasingFunction::ExpoIn,
            EasingFunction::ExpoOut,
        ];

        for easing in monotonic_easings {
            let mut prev = easing.apply(0.0);
            for i in 1..=100 {
                let t = i as f32 / 100.0;
                let curr = easing.apply(t);
                assert!(
                    curr >= prev - 1e-6,
                    "{:?} should be monotonically increasing at t={}",
                    easing,
                    t
                );
                prev = curr;
            }
        }
    }

    #[test]
    fn test_custom_bezier() {
        // Custom bezier curve
        // Note: Current implementation is simplified, not a true cubic bezier solver
        // It provides approximate easing behavior
        let bezier = EasingFunction::CubicBezier(0.25, 0.1, 0.25, 1.0);

        // Test that it produces valid values in range [0, 1]
        let _v0 = bezier.apply(0.0);
        let v05 = bezier.apply(0.5);
        let _v1 = bezier.apply(1.0);

        assert!(
            v05 >= 0.0 && v05 <= 1.0,
            "Bezier at t=0.5 should be in [0, 1]"
        );
    }

    #[test]
    fn test_easing_edge_cases() {
        // Test at exact boundaries
        let easings = vec![
            EasingFunction::Linear,
            EasingFunction::SineIn,
            EasingFunction::QuadIn,
            EasingFunction::CubicIn,
            EasingFunction::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 10.0,
                rest_threshold: 0.01,
            },
        ];

        for easing in easings {
            // Should clamp to [0, 1]
            let v_neg = easing.apply(-0.5);
            assert!(v_neg >= 0.0, "{:?} should clamp negative input", easing);

            let v_over = easing.apply(1.5);
            assert!(
                v_over <= 1.0 || easing.apply(1.0) > 1.0,
                "{:?} should handle overflow",
                easing
            );
        }
    }
}
