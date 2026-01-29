//! Keyboard editing module for ArchFlow SDK
//!
//! Provides precision keyboard movement (nudge) functionality with:
//! - Multiple precision levels (Normal, Fast, Precise)
//! - Auto-repeat support
//! - Undo batching for continuous operations
//! - Multi-selection support

use crate::canvas::{Canvas, ShapeChanges};
use crate::commands::{Command, CommandError, CommandResult};
use crate::selection::SelectionDelta;
use archflow_core::{EntityId, Vec2};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Precision level for keyboard nudge operations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrecisionLevel {
    /// Normal precision: 1px per nudge
    Normal,
    /// Fast movement: 10px per nudge (Shift key)
    Fast,
    /// Precise movement: 0.1px per nudge (Alt key)
    Precise,
}

impl PrecisionLevel {
    /// Returns the pixel distance for this precision level
    pub fn distance(self) -> f32 {
        match self {
            PrecisionLevel::Normal => 1.0,
            PrecisionLevel::Fast => 10.0,
            PrecisionLevel::Precise => 0.1,
        }
    }
}

impl Default for PrecisionLevel {
    fn default() -> Self {
        PrecisionLevel::Normal
    }
}

/// Direction for nudge operations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NudgeDirection {
    /// Move up (negative Y)
    Up,
    /// Move down (positive Y)
    Down,
    /// Move left (negative X)
    Left,
    /// Move right (positive X)
    Right,
}

impl NudgeDirection {
    /// Returns the movement vector for this direction and distance
    pub fn to_vector(self, distance: f32) -> Vec2 {
        match self {
            NudgeDirection::Up => Vec2::new(0.0, -distance),
            NudgeDirection::Down => Vec2::new(0.0, distance),
            NudgeDirection::Left => Vec2::new(-distance, 0.0),
            NudgeDirection::Right => Vec2::new(distance, 0.0),
        }
    }
}

/// Represents a nudge operation that can be executed and undone
#[derive(Clone, Debug)]
pub struct NudgeCommand {
    /// Shape IDs to move
    shape_ids: Vec<EntityId>,
    /// Movement delta (x, y)
    delta: Vec2,
    /// Original positions for undo
    original_positions: HashMap<EntityId, Vec2>,
    /// Whether the command has been executed
    executed: bool,
    /// Description of the operation
    description: String,
}

impl NudgeCommand {
    /// Creates a new nudge command
    pub fn new(shape_ids: Vec<EntityId>, delta: Vec2, precision: PrecisionLevel) -> Self {
        let description = format!(
            "Nudge {} shape(s) by ({:.1}, {:.1}) [{:?}]",
            shape_ids.len(),
            delta.x,
            delta.y,
            precision
        );

        Self {
            shape_ids,
            delta,
            original_positions: HashMap::new(),
            executed: false,
            description,
        }
    }

    /// Returns the shape IDs affected by this command
    pub fn shape_ids(&self) -> &[EntityId] {
        &self.shape_ids
    }

    /// Returns the movement delta
    pub fn delta(&self) -> Vec2 {
        self.delta
    }
}

impl Command for NudgeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Capture original positions before moving
        self.original_positions.clear();
        for &id in &self.shape_ids {
            if let Some(shape) = canvas.get_shape(id) {
                self.original_positions
                    .insert(id, Vec2::new(shape.x, shape.y));
            }
        }

        // Apply the nudge to each shape
        for &id in &self.shape_ids {
            if let Some(shape) = canvas.get_shape(id) {
                let new_x = shape.x + self.delta.x;
                let new_y = shape.y + self.delta.y;

                let changes = ShapeChanges {
                    x: Some(new_x),
                    y: Some(new_y),
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                };

                canvas.update_shape(id, changes);
            }
        }

        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Restore original positions
        for (&id, &original_pos) in &self.original_positions {
            let changes = ShapeChanges {
                x: Some(original_pos.x),
                y: Some(original_pos.y),
                width: None,
                height: None,
                rotation: None,
                fill_color: None,
                stroke_color: None,
                stroke_width: None,
                opacity: None,
            };

            canvas.update_shape(id, changes);
        }

        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn merge(&mut self, other: &dyn Command) -> bool {
        // Try to merge with another NudgeCommand
        if let Some(other_nudge) = other.as_any().downcast_ref::<NudgeCommand>() {
            // Only merge if affecting the same shapes
            if self.shape_ids == other_nudge.shape_ids {
                // Combine the deltas
                self.delta = Vec2::new(
                    self.delta.x + other_nudge.delta.x,
                    self.delta.y + other_nudge.delta.y,
                );
                self.description = format!(
                    "Nudge {} shape(s) by ({:.1}, {:.1})",
                    self.shape_ids.len(),
                    self.delta.x,
                    self.delta.y
                );
                return true;
            }
        }
        false
    }
}

/// Trait for commands that can be converted to Any for downcasting
pub trait CommandAny: Command {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl CommandAny for NudgeCommand {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// State tracking for a nudge sequence
#[derive(Clone, Debug)]
struct NudgeSequence {
    /// Last nudge time
    last_nudge_time: Instant,
    /// Accumulated command for batching
    accumulated_command: Option<NudgeCommand>,
    /// Precision level of the sequence
    precision: PrecisionLevel,
}

/// System for handling keyboard nudge operations
///
/// This system manages precision keyboard movement with features like:
/// - Multiple precision levels (Normal, Fast, Precise)
/// - Auto-repeat support with configurable intervals
/// - Undo batching for continuous operations
/// - Multi-selection support
#[derive(Debug)]
pub struct KeyboardNudgeSystem {
    /// Current precision level
    precision_level: PrecisionLevel,
    /// Active nudge sequence for batching
    active_sequence: Option<NudgeSequence>,
    /// Auto-repeat configuration
    auto_repeat_config: AutoRepeatConfig,
    /// Undo batching timeout
    batch_timeout: Duration,
}

/// Configuration for auto-repeat behavior
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoRepeatConfig {
    /// Initial delay before auto-repeat starts
    pub initial_delay: Duration,
    /// Interval between repeated nudges
    pub repeat_interval: Duration,
}

impl Default for AutoRepeatConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            repeat_interval: Duration::from_millis(50),
        }
    }
}

impl KeyboardNudgeSystem {
    /// Creates a new keyboard nudge system with default settings
    pub fn new() -> Self {
        Self {
            precision_level: PrecisionLevel::Normal,
            active_sequence: None,
            auto_repeat_config: AutoRepeatConfig::default(),
            batch_timeout: Duration::from_millis(300),
        }
    }

    /// Creates a new keyboard nudge system with custom configuration
    pub fn with_config(auto_repeat_config: AutoRepeatConfig, batch_timeout: Duration) -> Self {
        Self {
            precision_level: PrecisionLevel::Normal,
            active_sequence: None,
            auto_repeat_config,
            batch_timeout,
        }
    }

    /// Sets the precision level
    pub fn set_precision(&mut self, level: PrecisionLevel) {
        self.precision_level = level;
    }

    /// Gets the current precision level
    pub fn precision_level(&self) -> PrecisionLevel {
        self.precision_level
    }

    /// Sets the auto-repeat configuration
    pub fn set_auto_repeat_config(&mut self, config: AutoRepeatConfig) {
        self.auto_repeat_config = config;
    }

    /// Gets the auto-repeat configuration
    pub fn auto_repeat_config(&self) -> AutoRepeatConfig {
        self.auto_repeat_config
    }

    /// Sets the batch timeout for undo batching
    pub fn set_batch_timeout(&mut self, timeout: Duration) {
        self.batch_timeout = timeout;
    }

    /// Gets the batch timeout
    pub fn batch_timeout(&self) -> Duration {
        self.batch_timeout
    }

    /// Creates a nudge command for the specified direction and selection
    ///
    /// # Arguments
    ///
    /// * `direction` - Direction to nudge
    /// * `shape_ids` - IDs of shapes to move
    ///
    /// # Returns
    ///
    /// A nudge command ready for execution
    pub fn create_nudge_command(
        &self,
        direction: NudgeDirection,
        shape_ids: Vec<EntityId>,
    ) -> NudgeCommand {
        let distance = self.precision_level.distance();
        let delta = direction.to_vector(distance);
        NudgeCommand::new(shape_ids, delta, self.precision_level)
    }

    /// Processes a nudge operation with batching support
    ///
    /// This method handles undo batching by combining consecutive nudges
    /// within the batch timeout period.
    ///
    /// # Arguments
    ///
    /// * `direction` - Direction to nudge
    /// * `shape_ids` - IDs of shapes to move
    ///
    /// # Returns
    ///
    /// A command that should be executed (either new or merged)
    pub fn process_nudge(
        &mut self,
        direction: NudgeDirection,
        shape_ids: Vec<EntityId>,
    ) -> Option<NudgeCommand> {
        let now = Instant::now();
        let distance = self.precision_level.distance();
        let delta = direction.to_vector(distance);

        if let Some(ref mut sequence) = self.active_sequence {
            // Check if this is part of the same sequence
            let time_since_last = now.duration_since(sequence.last_nudge_time);

            if time_since_last < self.batch_timeout && sequence.precision == self.precision_level {
                // Merge with existing sequence
                if let Some(ref mut cmd) = sequence.accumulated_command {
                    cmd.delta = Vec2::new(cmd.delta.x + delta.x, cmd.delta.y + delta.y);
                    cmd.description = format!(
                        "Nudge {} shape(s) by ({:.1}, {:.1})",
                        cmd.shape_ids.len(),
                        cmd.delta.x,
                        cmd.delta.y
                    );
                }
                sequence.last_nudge_time = now;
                return None; // No new command, merged with existing
            }
        }

        // Start a new sequence
        let new_command = NudgeCommand::new(shape_ids, delta, self.precision_level);
        self.active_sequence = Some(NudgeSequence {
            last_nudge_time: now,
            accumulated_command: Some(new_command.clone()),
            precision: self.precision_level,
        });

        Some(new_command)
    }

    /// Finalizes the current nudge sequence
    ///
    /// Call this when the user releases the nudge key to ensure
    /// the accumulated command is properly committed.
    ///
    /// # Returns
    ///
    /// The accumulated command if there was an active sequence
    pub fn finalize_sequence(&mut self) -> Option<NudgeCommand> {
        self.active_sequence
            .take()
            .and_then(|s| s.accumulated_command)
    }

    /// Checks if there's an active nudge sequence
    pub fn has_active_sequence(&self) -> bool {
        self.active_sequence.is_some()
    }

    /// Clears the active nudge sequence without finalizing
    pub fn clear_sequence(&mut self) {
        self.active_sequence = None;
    }

    /// Determines the precision level based on modifier keys
    ///
    /// # Arguments
    ///
    /// * `shift_pressed` - Whether Shift is pressed (Fast mode)
    /// * `alt_pressed` - Whether Alt is pressed (Precise mode)
    ///
    /// # Returns
    ///
    /// The appropriate precision level (Precise takes priority over Fast)
    pub fn determine_precision(shift_pressed: bool, alt_pressed: bool) -> PrecisionLevel {
        if alt_pressed {
            PrecisionLevel::Precise
        } else if shift_pressed {
            PrecisionLevel::Fast
        } else {
            PrecisionLevel::Normal
        }
    }

    /// Updates the precision level based on modifier keys
    pub fn update_precision(&mut self, shift_pressed: bool, alt_pressed: bool) {
        self.precision_level = Self::determine_precision(shift_pressed, alt_pressed);
    }
}

impl Default for KeyboardNudgeSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait extension for Canvas to support keyboard nudge operations
pub trait CanvasNudgeExt {
    /// Nudges the selected shapes in the specified direction
    fn nudge_selection(&mut self, direction: NudgeDirection, distance: f32);

    /// Nudges specific shapes in the specified direction
    fn nudge_shapes(&mut self, shape_ids: &[EntityId], direction: NudgeDirection, distance: f32);
}

impl CanvasNudgeExt for Canvas {
    fn nudge_selection(&mut self, direction: NudgeDirection, distance: f32) {
        let selection = self.selection();
        let shape_ids: Vec<EntityId> = selection.shapes.clone();
        self.nudge_shapes(&shape_ids, direction, distance);
    }

    fn nudge_shapes(&mut self, shape_ids: &[EntityId], direction: NudgeDirection, distance: f32) {
        let delta = direction.to_vector(distance);

        for &id in shape_ids {
            if let Some(shape) = self.get_shape(id) {
                let new_x = shape.x + delta.x;
                let new_y = shape.y + delta.y;

                let changes = ShapeChanges {
                    x: Some(new_x),
                    y: Some(new_y),
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                };

                self.update_shape(id, changes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_level_distances() {
        assert_eq!(PrecisionLevel::Normal.distance(), 1.0);
        assert_eq!(PrecisionLevel::Fast.distance(), 10.0);
        assert_eq!(PrecisionLevel::Precise.distance(), 0.1);
    }

    #[test]
    fn test_nudge_direction_vectors() {
        assert_eq!(NudgeDirection::Up.to_vector(1.0), Vec2::new(0.0, -1.0));
        assert_eq!(NudgeDirection::Down.to_vector(1.0), Vec2::new(0.0, 1.0));
        assert_eq!(NudgeDirection::Left.to_vector(1.0), Vec2::new(-1.0, 0.0));
        assert_eq!(NudgeDirection::Right.to_vector(1.0), Vec2::new(1.0, 0.0));
    }

    #[test]
    fn test_nudge_direction_with_different_distances() {
        assert_eq!(NudgeDirection::Up.to_vector(10.0), Vec2::new(0.0, -10.0));
        assert_eq!(NudgeDirection::Right.to_vector(0.1), Vec2::new(0.1, 0.0));
    }

    #[test]
    fn test_determine_precision() {
        assert_eq!(
            KeyboardNudgeSystem::determine_precision(false, false),
            PrecisionLevel::Normal
        );
        assert_eq!(
            KeyboardNudgeSystem::determine_precision(true, false),
            PrecisionLevel::Fast
        );
        assert_eq!(
            KeyboardNudgeSystem::determine_precision(false, true),
            PrecisionLevel::Precise
        );
        assert_eq!(
            KeyboardNudgeSystem::determine_precision(true, true),
            PrecisionLevel::Precise
        );
    }

    #[test]
    fn test_nudge_system_default() {
        let system = KeyboardNudgeSystem::new();
        assert_eq!(system.precision_level(), PrecisionLevel::Normal);
        assert!(!system.has_active_sequence());
    }

    #[test]
    fn test_nudge_system_with_config() {
        let config = AutoRepeatConfig {
            initial_delay: Duration::from_millis(300),
            repeat_interval: Duration::from_millis(30),
        };
        let system = KeyboardNudgeSystem::with_config(config, Duration::from_millis(200));
        assert_eq!(
            system.auto_repeat_config().initial_delay,
            Duration::from_millis(300)
        );
        assert_eq!(system.batch_timeout(), Duration::from_millis(200));
    }

    #[test]
    fn test_create_nudge_command() {
        let mut system = KeyboardNudgeSystem::new();
        let shape_ids = vec![EntityId::new(), EntityId::new()];

        let cmd = system.create_nudge_command(NudgeDirection::Right, shape_ids.clone());
        assert_eq!(cmd.shape_ids(), &shape_ids);
        assert_eq!(cmd.delta(), Vec2::new(1.0, 0.0));

        system.set_precision(PrecisionLevel::Fast);
        let cmd = system.create_nudge_command(NudgeDirection::Down, shape_ids.clone());
        assert_eq!(cmd.delta(), Vec2::new(0.0, 10.0));
    }

    #[test]
    fn test_nudge_command_execute_undo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut cmd = NudgeCommand::new(vec![id], Vec2::new(10.0, 5.0), PrecisionLevel::Normal);

        // Execute
        cmd.execute(&mut canvas).unwrap();
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, 110.0);
        assert_eq!(shape.y, 105.0);

        // Undo
        cmd.undo(&mut canvas).unwrap();
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, 100.0);
        assert_eq!(shape.y, 100.0);
    }

    #[test]
    fn test_nudge_command_multi_selection() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        let mut cmd = NudgeCommand::new(
            vec![id1, id2],
            Vec2::new(10.0, 10.0),
            PrecisionLevel::Normal,
        );

        // Execute
        cmd.execute(&mut canvas).unwrap();
        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();
        assert_eq!(shape1.x, 110.0);
        assert_eq!(shape1.y, 110.0);
        assert_eq!(shape2.x, 210.0);
        assert_eq!(shape2.y, 210.0);

        // Undo
        cmd.undo(&mut canvas).unwrap();
        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();
        assert_eq!(shape1.x, 100.0);
        assert_eq!(shape1.y, 100.0);
        assert_eq!(shape2.x, 200.0);
        assert_eq!(shape2.y, 200.0);
    }

    #[test]
    fn test_nudge_command_merge() {
        let mut cmd1 = NudgeCommand::new(
            vec![EntityId::new()],
            Vec2::new(10.0, 0.0),
            PrecisionLevel::Normal,
        );
        let cmd2 = NudgeCommand::new(
            vec![EntityId::new()],
            Vec2::new(5.0, 5.0),
            PrecisionLevel::Normal,
        );

        // Different shape IDs should not merge
        assert!(!cmd1.merge(&cmd2));
        assert_eq!(cmd1.delta(), Vec2::new(10.0, 0.0));

        // Same shape IDs should merge
        let shape_id = EntityId::new();
        let mut cmd3 =
            NudgeCommand::new(vec![shape_id], Vec2::new(10.0, 0.0), PrecisionLevel::Normal);
        let cmd4 = NudgeCommand::new(vec![shape_id], Vec2::new(5.0, 5.0), PrecisionLevel::Normal);

        assert!(cmd3.merge(&cmd4));
        assert_eq!(cmd3.delta(), Vec2::new(15.0, 5.0));
    }

    #[test]
    fn test_process_nudge_batching() {
        let mut system = KeyboardNudgeSystem::with_config(
            AutoRepeatConfig::default(),
            Duration::from_millis(100),
        );

        let shape_id = EntityId::new();

        // First nudge should return a command
        let cmd1 = system.process_nudge(NudgeDirection::Right, vec![shape_id]);
        assert!(cmd1.is_some());
        assert!(system.has_active_sequence());

        // Second nudge within timeout should merge (return None)
        let cmd2 = system.process_nudge(NudgeDirection::Right, vec![shape_id]);
        assert!(cmd2.is_none());

        // Finalize should return the accumulated command
        let final_cmd = system.finalize_sequence();
        assert!(final_cmd.is_some());
        assert!(!system.has_active_sequence());

        let final_cmd = final_cmd.unwrap();
        assert_eq!(final_cmd.delta(), Vec2::new(2.0, 0.0)); // Two nudges merged
    }

    #[test]
    fn test_process_nudge_new_sequence_after_timeout() {
        let mut system = KeyboardNudgeSystem::with_config(
            AutoRepeatConfig::default(),
            Duration::from_millis(1), // Very short timeout
        );

        let shape_id = EntityId::new();

        // First nudge
        let cmd1 = system.process_nudge(NudgeDirection::Right, vec![shape_id]);
        assert!(cmd1.is_some());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(10));

        // Second nudge after timeout should start new sequence
        let cmd2 = system.process_nudge(NudgeDirection::Right, vec![shape_id]);
        assert!(cmd2.is_some());
    }

    #[test]
    fn test_canvas_nudge_extension() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        canvas.select(id);

        // Test nudge_selection
        canvas.nudge_selection(NudgeDirection::Right, 5.0);
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, 105.0);

        // Test nudge_shapes
        canvas.nudge_shapes(&[id], NudgeDirection::Down, 10.0);
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.y, 110.0);
    }

    #[test]
    fn test_precision_level_default() {
        let level: PrecisionLevel = Default::default();
        assert_eq!(level, PrecisionLevel::Normal);
    }

    #[test]
    fn test_auto_repeat_config_default() {
        let config: AutoRepeatConfig = Default::default();
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.repeat_interval, Duration::from_millis(50));
    }

    #[test]
    fn test_keyboard_nudge_system_default_trait() {
        let system: KeyboardNudgeSystem = Default::default();
        assert_eq!(system.precision_level(), PrecisionLevel::Normal);
    }

    #[test]
    fn test_clear_sequence() {
        let mut system = KeyboardNudgeSystem::new();
        let shape_id = EntityId::new();

        system.process_nudge(NudgeDirection::Right, vec![shape_id]);
        assert!(system.has_active_sequence());

        system.clear_sequence();
        assert!(!system.has_active_sequence());
    }

    #[test]
    fn test_nudge_precision_levels_in_command() {
        let cmd_normal = NudgeCommand::new(
            vec![EntityId::new()],
            Vec2::new(1.0, 0.0),
            PrecisionLevel::Normal,
        );
        let cmd_fast = NudgeCommand::new(
            vec![EntityId::new()],
            Vec2::new(10.0, 0.0),
            PrecisionLevel::Fast,
        );
        let cmd_precise = NudgeCommand::new(
            vec![EntityId::new()],
            Vec2::new(0.1, 0.0),
            PrecisionLevel::Precise,
        );

        assert!(cmd_normal.description().contains("Normal"));
        assert!(cmd_fast.description().contains("Fast"));
        assert!(cmd_precise.description().contains("Precise"));
    }

    #[test]
    fn test_nudge_left_direction() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut cmd = NudgeCommand::new(
            vec![id],
            NudgeDirection::Left.to_vector(5.0),
            PrecisionLevel::Normal,
        );

        cmd.execute(&mut canvas).unwrap();
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, 95.0);
    }

    #[test]
    fn test_nudge_up_direction() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut cmd = NudgeCommand::new(
            vec![id],
            NudgeDirection::Up.to_vector(5.0),
            PrecisionLevel::Normal,
        );

        cmd.execute(&mut canvas).unwrap();
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.y, 95.0);
    }

    #[test]
    fn test_update_precision() {
        let mut system = KeyboardNudgeSystem::new();
        assert_eq!(system.precision_level(), PrecisionLevel::Normal);

        system.update_precision(true, false);
        assert_eq!(system.precision_level(), PrecisionLevel::Fast);

        system.update_precision(false, true);
        assert_eq!(system.precision_level(), PrecisionLevel::Precise);

        system.update_precision(true, true);
        assert_eq!(system.precision_level(), PrecisionLevel::Precise);

        system.update_precision(false, false);
        assert_eq!(system.precision_level(), PrecisionLevel::Normal);
    }

    #[test]
    fn test_precise_nudge_0_1px() {
        let mut system = KeyboardNudgeSystem::new();
        system.set_precision(PrecisionLevel::Precise);

        let id = EntityId::new();
        let cmd = system.create_nudge_command(NudgeDirection::Right, vec![id]);

        assert_eq!(cmd.delta().x, 0.1);
        assert_eq!(cmd.delta().y, 0.0);
    }

    #[test]
    fn test_fast_nudge_10px() {
        let mut system = KeyboardNudgeSystem::new();
        system.set_precision(PrecisionLevel::Fast);

        let id = EntityId::new();
        let cmd = system.create_nudge_command(NudgeDirection::Down, vec![id]);

        assert_eq!(cmd.delta().x, 0.0);
        assert_eq!(cmd.delta().y, 10.0);
    }
}
