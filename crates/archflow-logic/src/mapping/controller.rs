// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Controller
//
// This module defines the Controller enum for boolean logic operations
// on sensor signals in LogicMapping connections.
//
// Controllers enable complex logic combinations:
// - Basic: Direct, AND, OR, NOT (BGE-style)
// - Predefined: Blinky, Debounce, Hysteresis, Threshold, Pattern (Rust-optimized)
// - Custom: JavaScript sandbox for maximum flexibility
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::SignalByte;
use crate::mapping::SensorType;
use alloc::string::String;
use alloc::vec::Vec;

/// Boolean logic controllers for combining sensor signals
///
/// # Controller Types
///
/// | Type | Description | Use Case |
/// |------|-------------|----------|
/// | `Direct` | Sensor directly triggers actuator | Simple hover effects |
/// | `AND` | All specified sensors must be active | Hover + Click to select |
/// | `OR` | At least one sensor must be active | Multiple triggers |
/// | `NOT` | Inverts the sensor signal | Disable on hover |
/// | `Blinky` | Toggles active/inactive periodically | Attention patterns |
/// | `Debounce` | Requires stable signal for N ticks | Noise filtering |
/// | `Hysteresis` | Different on/off thresholds | Avoid oscillation |
/// | `Threshold` | Minimum stability percentage | Gradual activation |
/// | `Pattern` | Matches binary pattern | Sequence detection |
/// | `Custom` | JavaScript sandbox evaluation | Complex logic |
///
/// # Examples
///
/// ```
/// use archflow_logic::Controller;
/// use archflow_logic::SensorType;
///
/// // Direct connection: MouseOver directly triggers Highlight
/// let controller = Controller::Direct;
///
/// // AND logic: Both MouseOver AND MouseClick required
/// let controller = Controller::AND(SensorType::MouseClick);
///
/// // Blinky: Toggle every 4 ticks (66ms at 60fps)
/// let controller = Controller::Blinky { interval: 4 };
///
/// // Debounce: Require 6 stable ticks (100ms)
/// let controller = Controller::Debounce { ticks: 6 };
///
/// // Hysteresis: Activate at 80%, deactivate at 30%
/// let controller = Controller::Hysteresis { high: 0.8, low: 0.3 };
/// ```
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Controller {
    // ═══════════════════════════════════════════════════════════════════════════════
    // BASIC CONTROLLERS (BGE-style boolean logic)
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Direct connection: sensor directly triggers actuator
    Direct = 0,

    /// AND logic: all specified sensors must be active
    /// The variant contains the additional sensor to check
    AND(SensorType) = 1,

    /// OR logic: at least one of the specified sensors must be active
    /// The variant contains the alternative sensor to check
    OR(SensorType) = 2,

    /// NOT logic: inverts the sensor signal
    NOT = 3,

    // ═══════════════════════════════════════════════════════════════════════════════
    // PREDEFINED CONTROLLERS (Rust-optimized for performance)
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Blinky: Toggles active/inactive at regular intervals
    ///
    /// # Behavior
    /// - Active for `interval` ticks, inactive for `interval` ticks
    /// - Requires sensor to be currently active
    /// - Useful for: attention patterns, loading indicators, status feedback
    ///
    /// # Timing (at 60 FPS)
    /// | Interval | Active | Inactive | Period |
    /// |----------|--------|----------|--------|
    /// | 2 | 33ms | 33ms | 66ms |
    /// | 4 | 66ms | 66ms | 133ms |
    /// | 6 | 100ms | 100ms | 200ms |
    Blinky {
        /// Toggle interval in ticks (16.67ms at 60 FPS)
        interval: u8,
    } = 4,

    /// Debounce: Requires signal to be stable for N consecutive ticks
    ///
    /// # Behavior
    /// - Only activates when signal is HIGH for `ticks` consecutive ticks
    /// - Deactivates immediately on falling edge
    /// - Useful for: eliminating noise, stable hover detection
    ///
    /// # Timing (at 60 FPS)
    /// | Ticks | Duration | Use Case |
    /// |-------|----------|----------|
    /// | 2 | 33ms | Fast click filter |
    /// | 4 | 66ms | Mouse movement filter |
    /// | 6 | 100ms | Standard debounce |
    /// | 12 | 200ms | Strong debounce |
    Debounce {
        /// Number of consecutive ticks signal must be HIGH
        ticks: u8,
    } = 5,

    /// Hysteresis: Different activation/deactivation thresholds
    ///
    /// # Behavior
    /// - Activates when signal is HIGH for `high`% of history
    /// - Deactivates when signal is HIGH for `low`% of history
    /// - `high` should be >= `low`
    /// - Useful for: avoiding oscillation near threshold, gradual transitions
    ///
    /// # Example
    /// With `high: 0.8, low: 0.3`:
    /// - Signal becomes active when >= 80% stable (5/6 ticks)
    /// - Signal becomes inactive when < 30% stable (< 2/6 ticks)
    /// - No change when between 30% and 80%
    Hysteresis {
        /// Activation threshold (0.0 to 1.0, e.g., 0.8 = 80%)
        high: f32,
        /// Deactivation threshold (0.0 to 1.0, e.g., 0.3 = 30%)
        low: f32,
    } = 6,

    /// Threshold: Requires minimum stability percentage
    ///
    /// # Behavior
    /// - Activates when signal is HIGH for at least `value`% of history
    /// - Deactivates when signal falls below threshold
    /// - Simpler than hysteresis (single threshold)
    /// - Useful for: proximity detection, hover confirmation
    ///
    /// # Example
    /// With `value: 0.5`:
    /// - Active when >= 3 out of 6 ticks are HIGH
    /// - Inactive when < 3 ticks are HIGH
    Threshold {
        /// Minimum stability threshold (0.0 to 1.0)
        value: f32,
    } = 7,

    /// Pattern: Matches specific binary pattern in history
    ///
    /// # Behavior
    /// - Checks if signal history matches the given mask
    /// - Uses bitwise AND: `(signal & mask) == pattern`
    /// - Useful for: sequence detection, custom trigger patterns
    ///
    /// # Example
    /// Pattern `0b00100100` matches: 100100 (click-pause-pause-click-pause-pause)
    Pattern {
        /// 6-bit pattern to match (bits 0-5 = T0-T5)
        mask: u8,
    } = 8,

    // ═══════════════════════════════════════════════════════════════════════════════
    // CUSTOM CONTROLLER (JavaScript sandbox for maximum flexibility)
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Custom: JavaScript sandbox evaluation
    ///
    /// # Behavior
    /// - Executes user-provided JavaScript code
    /// - Code receives signal and context objects
    /// - Returns boolean based on custom logic
    /// - Useful for: complex conditions, domain-specific logic
    ///
    /// # JavaScript API
    ///
    /// ```javascript
    /// // Available on signal object:
    /// signal.getCurrent()      // Current state (boolean)
    /// signal.isRisingEdge()    // 0→1 transition
    /// signal.isFallingEdge()   // 1→0 transition
    /// signal.isSteady(n)       // Stable for n ticks
    /// signal.countOnes()       // Number of HIGH ticks
    /// signal.countZeros()      // Number of LOW ticks
    ///
    /// // Available on context object:
    /// context.timestamp        // Current frame timestamp
    /// context.entityId         // Entity being evaluated
    /// context.getProperty(key) // Get custom property
    /// context.setProperty(k,v) // Set custom property
    /// ```
    Custom {
        /// Unique name for debugging and identification
        name: String,
        /// JavaScript code to execute
        code: String,
    } = 9,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTROLLER CONTEXT (for stateful controllers)
// ═══════════════════════════════════════════════════════════════════════════════

/// Context provided to controllers during evaluation
///
/// Contains additional state that controllers may need:
/// - Timestamp for time-based controllers (Blinky)
/// - Hysteresis state for stateful controllers
/// - Custom properties for sharing state between evaluations
#[derive(Debug)]
pub struct ControllerContext<'a> {
    /// Current frame timestamp (milliseconds since start)
    pub timestamp: u64,
    /// Entity being evaluated
    pub entity_id: u32,
    /// Modifier keys bitmask (1=shift, 2=ctrl, 4=alt, 8=meta)
    pub modifiers: u8,
    /// Reference to hysteresis states (for Hysteresis controller)
    hysteresis_states: &'a mut HysteresisStateMap,
    /// Custom properties for state sharing
    custom_properties: &'a mut CustomPropertyMap,
}

/// State for Hysteresis controller per entity
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HysteresisState {
    /// Currently below low threshold
    Low,
    /// Currently above high threshold
    High,
}

/// Map of hysteresis states keyed by entity ID
pub type HysteresisStateMap = Vec<Option<HysteresisState>>;

/// Map of custom properties keyed by (entity_id, property_name)
pub type CustomPropertyMap = Vec<alloc::collections::BTreeMap<String, PropertyValue>>;

/// Value type for custom properties
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Bool(bool),
    Number(f64),
    String(String),
}

impl<'a> ControllerContext<'a> {
    /// Creates a new controller context
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Current frame timestamp
    /// * `entity_id` - Entity being evaluated
    /// * `hysteresis_states` - Mutable reference to hysteresis state storage
    /// * `custom_properties` - Mutable reference to custom property storage
    #[inline(always)]
    #[must_use]
    pub fn new(
        timestamp: u64,
        entity_id: u32,
        modifiers: u8,
        hysteresis_states: &'a mut HysteresisStateMap,
        custom_properties: &'a mut CustomPropertyMap,
    ) -> Self {
        Self {
            timestamp,
            entity_id,
            modifiers,
            hysteresis_states,
            custom_properties,
        }
    }

    /// Gets the current hysteresis state for this entity
    #[inline(always)]
    #[must_use]
    pub fn hysteresis_state(&self, entity_idx: usize) -> HysteresisState {
        self.hysteresis_states
            .get(entity_idx)
            .and_then(|opt| *opt)
            .unwrap_or(HysteresisState::Low)
    }

    /// Sets the hysteresis state for this entity
    #[inline(always)]
    pub fn set_hysteresis_state(&mut self, entity_idx: usize, state: HysteresisState) {
        if self.hysteresis_states.len() <= entity_idx {
            self.hysteresis_states.resize(entity_idx + 1, None);
        }
        self.hysteresis_states[entity_idx] = Some(state);
    }

    /// Gets a custom property for this entity
    #[inline(always)]
    pub fn get_property(&self, entity_idx: usize, key: &str) -> Option<PropertyValue> {
        self.custom_properties
            .get(entity_idx)
            .and_then(|map| map.get(key).cloned())
    }

    /// Sets a custom property for this entity
    #[inline(always)]
    pub fn set_property(&mut self, entity_idx: usize, key: String, value: PropertyValue) {
        if self.custom_properties.len() <= entity_idx {
            self.custom_properties
                .resize(entity_idx + 1, alloc::collections::BTreeMap::new());
        }
        self.custom_properties[entity_idx].insert(key, value);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTROLLER EVALUATION
// ═══════════════════════════════════════════════════════════════════════════════

impl Controller {
    /// Evaluates the controller logic given the sensor signals
    ///
    /// # Arguments
    ///
    /// * `primary_sensor` - The primary sensor type for this connection
    /// * `signals` - Slice of (sensor_type, signal_byte) tuples
    /// * `context` - Controller context with additional state
    ///
    /// # Returns
    ///
    /// `true` if the controller condition is met, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// let mouse_over = SignalByte::from(0b00111111); // Active
    /// let mouse_click = SignalByte::from(0b00000000); // Inactive
    /// let signals = &[
    ///     (SensorType::MouseOver, mouse_over),
    ///     (SensorType::MouseClick, mouse_click),
    /// ];
    ///
    /// // Direct: only checks primary sensor
    /// assert!(Controller::Direct.evaluate(SensorType::MouseOver, signals, &mut context));
    /// ```
    #[must_use]
    pub fn evaluate(
        &self,
        primary_sensor: SensorType,
        signals: &[(SensorType, SignalByte)],
        context: &mut ControllerContext<'_>,
    ) -> bool {
        // Find primary sensor signal
        let primary_signal = signals
            .iter()
            .find(|(sensor, _)| *sensor == primary_sensor)
            .map(|(_, signal)| signal);

        let primary_active = match primary_signal {
            Some(signal) => signal.get_current(),
            None => false,
        };

        match self {
            // ═══════════════════════════════════════════════════════════════════════
            // BASIC CONTROLLERS
            // ═══════════════════════════════════════════════════════════════════════
            Controller::Direct => primary_active,

            Controller::AND(other_sensor) => {
                // Both primary and other must be active
                let other_active = signals
                    .iter()
                    .find(|(sensor, _)| *sensor == *other_sensor)
                    .map(|(_, signal)| signal.get_current())
                    .unwrap_or(false);

                primary_active && other_active
            }

            Controller::OR(other_sensor) => {
                // Either primary or other must be active
                let other_active = signals
                    .iter()
                    .find(|(sensor, _)| *sensor == *other_sensor)
                    .map(|(_, signal)| signal.get_current())
                    .unwrap_or(false);

                primary_active || other_active
            }

            Controller::NOT => !primary_active,

            // ═══════════════════════════════════════════════════════════════════════
            // PREDEFINED CONTROLLERS
            // ═══════════════════════════════════════════════════════════════════════
            Controller::Blinky { interval } => {
                // Only active if sensor is active AND we're in the "on" phase
                if !primary_active {
                    return false;
                }

                // Calculate phase: (timestamp / 16.67ms) / interval % 2
                // interval=4 means: 4 ticks ON, 4 ticks OFF
                let tick_rate_ms = 16; // Approximate 60 FPS
                let phase_ticks = (context.timestamp / tick_rate_ms as u64) / (*interval as u64);
                let phase = phase_ticks % 2;

                // Phase 0 = ON, Phase 1 = OFF
                phase == 0
            }

            Controller::Debounce { ticks } => {
                // Must be currently active AND stable for N ticks
                let signal = match primary_signal {
                    Some(s) => s,
                    None => return false,
                };

                primary_active && signal.is_steady_high(*ticks)
            }

            Controller::Hysteresis { high, low } => {
                let signal = match primary_signal {
                    Some(s) => s,
                    None => return false,
                };

                let entity_idx = context.entity_id as usize;
                let current_stability = signal.count_ones() as f32 / 6.0;
                let current_state = context.hysteresis_state(entity_idx);

                match current_state {
                    HysteresisState::Low => {
                        // Must reach HIGH threshold to activate
                        if current_stability >= *high {
                            context.set_hysteresis_state(entity_idx, HysteresisState::High);
                            true
                        } else {
                            false
                        }
                    }
                    HysteresisState::High => {
                        // Must drop below LOW threshold to deactivate
                        if current_stability <= *low {
                            context.set_hysteresis_state(entity_idx, HysteresisState::Low);
                            false
                        } else {
                            true
                        }
                    }
                }
            }

            Controller::Threshold { value } => {
                let signal = match primary_signal {
                    Some(s) => s,
                    None => return false,
                };

                let stability = signal.count_ones() as f32 / 6.0;
                stability >= *value
            }

            Controller::Pattern { mask } => {
                let signal = match primary_signal {
                    Some(s) => s,
                    None => return false,
                };

                // Match: (signal & mask) == mask
                // All 1-bits in mask must be 1 in signal
                (signal.get_history() & *mask) == *mask
            }

            // ═══════════════════════════════════════════════════════════════════════
            // CUSTOM CONTROLLER (placeholder - JS evaluation happens in TypeScript)
            // ═══════════════════════════════════════════════════════════════════════
            Controller::Custom { name: _, code: _ } => {
                // Custom controllers are evaluated in JavaScript for sandbox security
                // This is a placeholder that returns false
                // Real evaluation happens via CustomController in TypeScript
                false
            }
        }
    }

    /// Returns the controller type as a u8 for serialization
    #[inline(always)]
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        // Safety: repr(u8) ensures first byte is discriminant
        // This is safe because Controller has #[repr(u8)]
        unsafe { *<*const Self>::from(self).cast::<u8>() }
    }

    /// Returns true if this controller requires context (is stateful)
    #[inline(always)]
    #[must_use]
    pub fn requires_context(&self) -> bool {
        matches!(
            self,
            Controller::Hysteresis { .. } | Controller::Blinky { .. } | Controller::Custom { .. }
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE CONSTRUCTORS
// ═══════════════════════════════════════════════════════════════════════════════

impl Controller {
    /// Creates a Blinky controller with the given interval
    ///
    /// # Arguments
    ///
    /// * `interval` - Toggle interval in ticks (16.67ms at 60 FPS)
    ///
    /// # Examples
    ///
    /// ```
    /// // Blink every 100ms (6 ticks at 60fps)
    /// let blinky = Controller::blinky(6);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn blinky(interval: u8) -> Self {
        Self::Blinky { interval }
    }

    /// Creates a Debounce controller requiring N stable ticks
    ///
    /// # Arguments
    ///
    /// * `ticks` - Number of consecutive ticks signal must be HIGH
    ///
    /// # Examples
    ///
    /// ```
    /// // Require 100ms of stable signal
    /// let debounced = Controller::debounce(6);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn debounce(ticks: u8) -> Self {
        Self::Debounce { ticks }
    }

    /// Creates a Hysteresis controller with activation/deactivation thresholds
    ///
    /// # Arguments
    ///
    /// * `high` - Activation threshold (0.0 to 1.0)
    /// * `low` - Deactivation threshold (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// // Activate at 80%, deactivate at 30%
    /// let hyst = Controller::hysteresis(0.8, 0.3);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn hysteresis(high: f32, low: f32) -> Self {
        debug_assert!(high >= low, "high must be >= low for hysteresis");
        Self::Hysteresis { high, low }
    }

    /// Creates a Threshold controller with minimum stability
    ///
    /// # Arguments
    ///
    /// * `value` - Minimum stability threshold (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// // Require 50% stability
    /// let thresh = Controller::threshold(0.5);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn threshold(value: f32) -> Self {
        debug_assert!((0.0..=1.0).contains(&value), "threshold must be 0.0-1.0");
        Self::Threshold { value }
    }

    /// Creates a Pattern controller matching binary pattern
    ///
    /// # Arguments
    ///
    /// * `mask` - 6-bit pattern to match
    ///
    /// # Examples
    ///
    /// ```
    /// // Match pattern 100100 (double-click)
    /// let pattern = Controller::pattern(0b00100100);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn pattern(mask: u8) -> Self {
        debug_assert!(mask <= 0b111111, "pattern must be 6-bit");
        Self::Pattern { mask }
    }

    /// Creates a Custom controller with JavaScript code
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for debugging
    /// * `code` - JavaScript code to evaluate
    ///
    /// # Examples
    ///
    /// ```
    /// let custom = Controller::custom("tooltipOnCtrlHover", r#"
    ///     (signal, context) => {
    ///         const stable = signal.isSteady(6);
    ///         const hasCtrl = (context.modifiers & 2) !== 0;
    ///         return stable && hasCtrl;
    ///     }
    /// "#);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn custom(name: &str, code: &str) -> Self {
        Self::Custom {
            name: String::from(name),
            code: String::from(code),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::SensorType;

    // Helper macro to create context variables with proper lifetime
    macro_rules! ctx_vars {
        () => {
            let mut hyst_vec: HysteresisStateMap = Vec::new();
            let mut prop_vec: CustomPropertyMap = Vec::new();
            let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);
        };
        ($timestamp:expr, $entity_id:expr) => {
            let mut hyst_vec: HysteresisStateMap = Vec::new();
            let mut prop_vec: CustomPropertyMap = Vec::new();
            let mut context =
                ControllerContext::new($timestamp, $entity_id, 0, &mut hyst_vec, &mut prop_vec);
        };
    }

    #[test]
    fn test_direct_controller() {
        let signal = SignalByte::from(0b00111111); // Active
        let signals = &[(SensorType::MouseOver, signal)];
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        assert!(Controller::Direct.evaluate(SensorType::MouseOver, signals, &mut context));
    }

    #[test]
    fn test_and_controller() {
        let active = SignalByte::from(0b00111111);
        let inactive = SignalByte::from(0b00000000);
        let signals = &[
            (SensorType::MouseOver, active),
            (SensorType::MouseClick, inactive),
        ];
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        let and_ctrl = Controller::AND(SensorType::MouseClick);
        assert!(!and_ctrl.evaluate(SensorType::MouseOver, signals, &mut context));

        let active_click = SignalByte::from(0b00111111);
        let signals2 = &[
            (SensorType::MouseOver, active),
            (SensorType::MouseClick, active_click),
        ];
        assert!(and_ctrl.evaluate(SensorType::MouseOver, signals2, &mut context));
    }

    #[test]
    fn test_or_controller() {
        let active = SignalByte::from(0b00111111);
        let inactive = SignalByte::from(0b00000000);
        let signals = &[
            (SensorType::MouseOver, active),
            (SensorType::MouseClick, inactive),
        ];
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        let or_ctrl = Controller::OR(SensorType::MouseClick);
        assert!(or_ctrl.evaluate(SensorType::MouseOver, signals, &mut context));

        let inactive_over = SignalByte::from(0b00000000);
        let signals2 = &[
            (SensorType::MouseOver, inactive_over),
            (SensorType::MouseClick, inactive),
        ];
        assert!(!or_ctrl.evaluate(SensorType::MouseOver, signals2, &mut context));
    }

    #[test]
    fn test_not_controller() {
        let active = SignalByte::from(0b00111111);
        let signals = &[(SensorType::MouseOver, active)];
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        assert!(!Controller::NOT.evaluate(SensorType::MouseOver, signals, &mut context));

        let inactive = SignalByte::from(0b00000000);
        let signals2 = &[(SensorType::MouseOver, inactive)];
        assert!(Controller::NOT.evaluate(SensorType::MouseOver, signals2, &mut context));
    }

    #[test]
    fn test_blinky_controller() {
        let active = SignalByte::from(0b00111111);
        let signals = &[(SensorType::MouseOver, active)];
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        // Phase 0 should be ON
        context.timestamp = 0;
        assert!(Controller::blinky(4).evaluate(SensorType::MouseOver, signals, &mut context));

        // Phase 1 should be OFF
        context.timestamp = 64; // 4 ticks * 16ms
        assert!(!Controller::blinky(4).evaluate(SensorType::MouseOver, signals, &mut context));

        // Inactive signal should always return false
        let inactive = SignalByte::from(0b00000000);
        let signals_inactive = &[(SensorType::MouseOver, inactive)];
        context.timestamp = 0;
        assert!(!Controller::blinky(4).evaluate(
            SensorType::MouseOver,
            signals_inactive,
            &mut context
        ));
    }

    #[test]
    fn test_debounce_controller() {
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        // Not stable - only 2 ticks
        let partially_stable = SignalByte::from(0b00000011);
        let signals = &[(SensorType::MouseOver, partially_stable)];
        assert!(!Controller::debounce(4).evaluate(SensorType::MouseOver, signals, &mut context));

        // Stable for 6 ticks
        let stable = SignalByte::from(0b00111111);
        let signals2 = &[(SensorType::MouseOver, stable)];
        assert!(Controller::debounce(4).evaluate(SensorType::MouseOver, signals2, &mut context));
    }

    #[test]
    fn test_hysteresis_controller() {
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 1, 0, &mut hyst_vec, &mut prop_vec); // entity_id = 1

        // Start at low threshold (30%)
        let low_signal = SignalByte::from(0b00001111); // 4/6 = 67% > 30%, should transition to High
        let signals_low = &[(SensorType::Proximity, low_signal)];

        // First evaluation should activate (67% >= 30% is true, but we start at Low)
        // Actually, with our logic: Low -> High when >= 0.8
        // So 67% should NOT activate
        assert!(!Controller::hysteresis(0.8, 0.3).evaluate(
            SensorType::Proximity,
            signals_low,
            &mut context
        ));

        // Now reach high threshold (80% = 5/6 ticks)
        let high_signal = SignalByte::from(0b00111111); // 6/6 = 100%
        let signals_high = &[(SensorType::Proximity, high_signal)];

        assert!(Controller::hysteresis(0.8, 0.3).evaluate(
            SensorType::Proximity,
            signals_high,
            &mut context
        ));

        // Drop below low threshold
        let drop_signal = SignalByte::from(0b00000001); // 1/6 = 17%
        let signals_drop = &[(SensorType::Proximity, drop_signal)];

        assert!(!Controller::hysteresis(0.8, 0.3).evaluate(
            SensorType::Proximity,
            signals_drop,
            &mut context
        ));
    }

    #[test]
    fn test_threshold_controller() {
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        // 50% stability (3/6 ticks)
        let threshold_signal = SignalByte::from(0b00010111); // 4 ones = 67%
        let signals = &[(SensorType::MouseOver, threshold_signal)];

        assert!(Controller::threshold(0.5).evaluate(SensorType::MouseOver, signals, &mut context));
        assert!(!Controller::threshold(0.8).evaluate(SensorType::MouseOver, signals, &mut context));
    }

    #[test]
    fn test_pattern_controller() {
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        // Double-click pattern: click-pause-pause-click-pause-pause
        // History: 100100 = 0b00100100
        let double_click = SignalByte::from(0b00100100);
        let signals = &[(SensorType::MouseClick, double_click)];

        assert!(Controller::pattern(0b00100100).evaluate(
            SensorType::MouseClick,
            signals,
            &mut context
        ));

        // Wrong pattern
        let single_click = SignalByte::from(0b00000001);
        let signals2 = &[(SensorType::MouseClick, single_click)];
        assert!(!Controller::pattern(0b00100100).evaluate(
            SensorType::MouseClick,
            signals2,
            &mut context
        ));
    }

    #[test]
    fn test_custom_controller_returns_false() {
        // Custom controllers are evaluated in JS, Rust returns false as placeholder
        let signal = SignalByte::from(0b00111111);
        let signals = &[(SensorType::MouseOver, signal)];
        let mut hyst_vec: HysteresisStateMap = Vec::new();
        let mut prop_vec: CustomPropertyMap = Vec::new();
        let mut context = ControllerContext::new(0, 0, 0, &mut hyst_vec, &mut prop_vec);

        let custom = Controller::custom("test", "return true;");
        assert!(!custom.evaluate(SensorType::MouseOver, signals, &mut context));
    }

    #[test]
    fn test_convenience_constructors() {
        assert!(matches!(Controller::Direct, Controller::Direct));
        assert!(matches!(
            Controller::blinky(4),
            Controller::Blinky { interval: 4 }
        ));
        assert!(matches!(
            Controller::debounce(6),
            Controller::Debounce { ticks: 6 }
        ));
        assert!(matches!(
            Controller::hysteresis(0.8, 0.3),
            Controller::Hysteresis {
                high: 0.8,
                low: 0.3
            }
        ));
        assert!(matches!(
            Controller::threshold(0.5),
            Controller::Threshold { value: 0.5 }
        ));
        assert!(matches!(
            Controller::pattern(0b00100100),
            Controller::Pattern { mask: 0b00100100 }
        ));
        assert!(matches!(
            Controller::custom("name", "code"),
            Controller::Custom { name: _, code: _ }
        ));
    }

    #[test]
    fn test_controller_as_u8() {
        assert_eq!(Controller::Direct.as_u8(), 0);
        assert_eq!(Controller::AND(SensorType::MouseOver).as_u8(), 1);
        assert_eq!(Controller::OR(SensorType::MouseOver).as_u8(), 2);
        assert_eq!(Controller::NOT.as_u8(), 3);
        assert_eq!(Controller::blinky(4).as_u8(), 4);
        assert_eq!(Controller::debounce(6).as_u8(), 5);
        assert_eq!(Controller::hysteresis(0.8, 0.3).as_u8(), 6);
        assert_eq!(Controller::threshold(0.5).as_u8(), 7);
        assert_eq!(Controller::pattern(0b00100100).as_u8(), 8);
    }

    #[test]
    fn test_requires_context() {
        assert!(!Controller::Direct.requires_context());
        assert!(!Controller::AND(SensorType::MouseOver).requires_context());
        assert!(!Controller::NOT.requires_context());
        assert!(Controller::blinky(4).requires_context());
        assert!(Controller::hysteresis(0.8, 0.3).requires_context());
        assert!(Controller::custom("test", "code").requires_context());
        assert!(!Controller::debounce(6).requires_context());
        assert!(!Controller::threshold(0.5).requires_context());
        assert!(!Controller::pattern(0b00100100).requires_context());
    }
}
