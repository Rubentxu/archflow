// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Controller
//
// This module defines the Controller enum for boolean logic operations
// on sensor signals in LogicMapping connections.
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::mapping::SensorType;

/// Boolean logic controllers for combining sensor signals
///
/// Controllers determine how sensor signals are evaluated before
/// triggering an actuator. They enable complex logic like:
/// - Direct: Sensor directly triggers actuator
/// - AND: All specified sensors must be active
/// - OR: At least one sensor must be active
/// - NOT: Inverts the sensor signal
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
/// // OR logic: Either MouseOver OR KeyShortcut triggers
/// let controller = Controller::OR(SensorType::KeyShortcut);
///
/// // NOT logic: MouseOver prevents activation
/// let controller = Controller::NOT;
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    /// Direct connection: sensor directly triggers actuator
    Direct = 0,

    /// AND logic: all specified sensors must be active
    /// The variant contains the additional sensors to check
    AND(SensorType) = 1,

    /// OR logic: at least one of the specified sensors must be active
    /// The variant contains the alternative sensors to check
    OR(SensorType) = 2,

    /// NOT logic: inverts the sensor signal
    NOT = 3,
}

impl Controller {
    /// Evaluates the controller logic given the sensor signals
    ///
    /// # Arguments
    ///
    /// * `primary_sensor` - The primary sensor type for this connection
    /// * `signals` - Slice of (sensor_type, signal_byte) tuples
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
    /// assert!(Controller::Direct.evaluate(SensorType::MouseOver, signals));
    ///
    /// // AND: requires MouseClick to also be active
    /// assert!(!Controller::AND(SensorType::MouseClick).evaluate(SensorType::MouseOver, signals));
    ///
    /// // OR: accepts either MouseOver OR MouseClick
    /// assert!(Controller::OR(SensorType::MouseClick).evaluate(SensorType::MouseOver, signals));
    ///
    /// // NOT: inverts MouseOver (active → inactive)
    /// assert!(!Controller::NOT.evaluate(SensorType::MouseOver, signals));
    /// ```
    #[must_use]
    pub fn evaluate(
        self,
        primary_sensor: SensorType,
        signals: &[(SensorType, crate::SignalByte)],
    ) -> bool {
        // Find primary sensor signal
        let primary_signal = signals
            .iter()
            .find(|(sensor, _)| *sensor == primary_sensor)
            .map(|(_, signal)| signal);

        let primary_active = match primary_signal {
            Some(signal) => signal.is_steady_high(6),
            None => false,
        };

        match self {
            Controller::Direct => primary_active,

            Controller::AND(other_sensor) => {
                // Both primary and other must be active
                let other_active = signals
                    .iter()
                    .find(|(sensor, _)| *sensor == other_sensor)
                    .map(|(_, signal)| signal.is_steady_high(6))
                    .unwrap_or(false);

                primary_active && other_active
            }

            Controller::OR(other_sensor) => {
                // Either primary or other must be active
                let other_active = signals
                    .iter()
                    .find(|(sensor, _)| *sensor == other_sensor)
                    .map(|(_, signal)| signal.is_steady_high(6))
                    .unwrap_or(false);

                primary_active || other_active
            }

            Controller::NOT => !primary_active,
        }
    }
}
