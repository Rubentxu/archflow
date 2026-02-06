// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Logic Bricks Integration
//
// This module integrates the Logic Bricks system into the ArchFlow engine,
// following the patterns from LOGIC_BRICKS_DEVELOPER_GUIDE.md
//
// Architecture:
// 1. SAMPLE PHASE: Sensors read EntityStore (immutable)
// 2. LOGIC PHASE: Controllers filter Pulses
// 3. ACTUATE PHASE: Actuators write Commands → CommandQueue
// 4. COMMIT PHASE: Batch-apply all Commands
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_core::Vec2;
use archflow_engine::EntityStore;
use archflow_logic::{BatchSelectActuator, InputSampler, LogicEvent, LogicSystem, SelectMode};

/// Input state sampled from JavaScript
///
/// This struct stores the current input state (mouse, keyboard) that is
/// sampled once per frame from JavaScript via `sample_input()`.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Mouse position in world coordinates
    pub mouse_pos: Vec2,

    /// Mouse button state (bit 0=left, 1=right, 2=middle)
    pub buttons: u8,

    /// Mouse wheel delta
    pub wheel: i8,

    /// Pressed keys (for keyboard sensors)
    pub keys: alloc::vec::Vec<alloc::string::String>,
}

/// Integrated Logic Bricks system for the web engine
///
/// This struct manages all sensors, actuators, and their wiring.
/// It follows the 4-phase execution model from the developer guide.
pub struct LogicBricksSystem {
    /// Core logic system (manages sensors, actuators, pulse bus)
    pub logic_system: LogicSystem,

    /// Batch selection actuator (replaces engine.selected_entities)
    pub batch_select: BatchSelectActuator,

    /// Current input state (sampled from JavaScript)
    pub input_state: InputState,
}

impl LogicBricksSystem {
    /// Create a new Logic Bricks system
    ///
    /// This initializes all built-in sensors and actuators needed for
    /// the web editor functionality.
    pub fn new() -> Self {
        let logic_system = LogicSystem::new();
        let batch_select = BatchSelectActuator::new();

        Self {
            logic_system,
            batch_select,
            input_state: InputState::default(),
        }
    }

    /// Execute the full Logic Bricks pipeline for one frame
    ///
    /// This implements the 4-phase execution model:
    /// 1. SAMPLE: Sensors read EntityStore
    /// 2. LOGIC: Controllers filter pulses
    /// 3. ACTUATE: Actuators write commands
    /// 4. COMMIT: Batch-apply commands (handled by caller)
    ///
    /// # Performance
    /// Target: <500µs for 1000 entities (3% of 16ms frame budget)
    pub fn tick(&mut self, store: &mut EntityStore, timestamp: u32) {
        // Update timestamp before evaluation
        self.logic_system.set_timestamp(timestamp);

        // Phase 1-3: Logic system handles sensor sampling, logic, and actuation
        // The LogicSystem internally manages the pulse bus and command queue
        self.logic_system.update(store);

        // Note: Phase 4 (COMMIT) is handled by the engine's command queue
        // after this function returns
    }

    /// Get mutable reference to the input sampler
    ///
    /// This is used by the bridge to configure SharedArrayBuffer pointer
    pub fn input_sampler(&mut self) -> &mut InputSampler {
        self.logic_system.input_sampler()
    }

    /// Poll all events from the logic system
    ///
    /// This should be called once per frame to drain all accumulated events.
    /// Uses the Event Ring-Buffer pattern for zero-copy JS interop.
    pub fn poll_events(&mut self) -> alloc::vec::Vec<LogicEvent> {
        self.logic_system.poll_events()
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        self.logic_system.has_events()
    }

    /// Get the number of pending events
    pub fn pending_event_count(&self) -> usize {
        self.logic_system.pending_event_count()
    }
}

impl Default for LogicBricksSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Setup default logic bricks for the web editor
///
/// This configures the standard sensor→actuator mappings for:
/// - Entity selection (click)
/// - Hover effects
/// - Drag and drop
/// - Keyboard shortcuts
///
/// # Example
/// ```rust
/// let mut logic = LogicBricksSystem::new();
/// setup_default_logic_bricks(&mut logic);
/// ```
pub fn setup_default_logic_bricks(_system: &mut LogicBricksSystem) {
    // TODO: Wire up sensors to actuators using LogicMappingTable
    // This will be implemented in the next phase following the
    // WiringBuilder pattern from the developer guide

    // Example (to be implemented):
    // let mouse_click = MouseSensor::with_config(MAX_ENTITIES, MouseConfig::left_button());
    // let select_actuator = &mut system.batch_select;
    //
    // WiringBuilder::new()
    //     .connect(mouse_click.id(), select_actuator.id())
    //     .on_positive()
    //     .build()
}

/// Selection mode for batch select operations
///
/// This is exposed to JavaScript for UI controls.
/// Maps to archflow_logic::SelectMode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Clear previous selection and select new entities
    Single,
    /// Toggle entities (add/remove from selection)
    Multi,
    /// Same as Single
    Replace,
}

impl From<SelectionMode> for SelectMode {
    fn from(mode: SelectionMode) -> Self {
        match mode {
            SelectionMode::Single => SelectMode::Single,
            SelectionMode::Multi => SelectMode::Multi,
            SelectionMode::Replace => SelectMode::Replace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_system_creation() {
        let system = LogicBricksSystem::new();
        assert_eq!(system.batch_select.selection_count(), 0);
    }

    #[test]
    fn test_selection_mode_conversion() {
        assert_eq!(SelectMode::from(SelectionMode::Single), SelectMode::Single);
        assert_eq!(SelectMode::from(SelectionMode::Multi), SelectMode::Multi);
        assert_eq!(
            SelectMode::from(SelectionMode::Replace),
            SelectMode::Replace
        );
    }
}
