// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - SnapToGridActuator
//
// Actuator for snapping entity positions to a configurable grid.
// Implements US-010 from TEMA 2.
//
// Architecture:
// - SnapToGridActuator: Snaps entity positions to grid lines
// - Uses O(n) iteration for n selected entities
// - Configurable grid size and snap threshold
//
// Performance Characteristics:
// - O(n) for snapping n entities
// - O(1) for single entity snap
// - Threshold-based snapping (configurable proximity)
// ═════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};

/// Configuration for grid snapping behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapConfig {
    /// Grid cell size in world units
    pub grid_size: f32,
    /// Snap threshold in pixels (proximity to snap)
    pub snap_threshold: f32,
    /// Enable/disable horizontal snapping
    pub snap_x: bool,
    /// Enable/disable vertical snapping
    pub snap_y: bool,
    /// Snap to grid origin (0,0) when true
    pub snap_to_origin: bool,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            grid_size: 10.0,
            snap_threshold: 5.0,
            snap_x: true,
            snap_y: true,
            snap_to_origin: true,
        }
    }
}

/// Result of a snap operation containing changed entities.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapResult {
    /// Entities that were snapped
    pub snapped_entities: Vec<EntityId>,
    /// Original positions before snapping
    pub original_positions: Vec<Vec2>,
    /// New positions after snapping
    pub snapped_positions: Vec<Vec2>,
    /// Whether any snapping occurred
    pub did_snap: bool,
}

impl SnapResult {
    /// Creates an empty result (no snapping occurred).
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            snapped_entities: Vec::new(),
            original_positions: Vec::new(),
            snapped_positions: Vec::new(),
            did_snap: false,
        }
    }

    /// Returns true if any entities were snapped.
    #[inline(always)]
    pub fn has_snapped(&self) -> bool {
        self.did_snap
    }
}

/// Actuator for snapping entities to a grid.
///
/// Provides grid-based alignment for precise positioning:
/// - Configurable grid cell size
/// - Adjustable snap threshold
/// - Independent X/Y snapping control
/// - Optional origin snapping
///
/// # Performance
/// - O(n) for n entities
/// - O(1) memory overhead per entity
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::snap::{SnapToGridActuator, SnapConfig};
///
/// let config = SnapConfig {
///     grid_size: 20.0,
///     snap_threshold: 5.0,
///     ..Default::default()
/// };
///
/// let mut actuator = SnapToGridActuator::with_config(config);
/// let mut store = /* ... */;
/// let entities = vec![entity1, entity2];
///
/// // Snap entities to grid
/// let result = actuator.snap_entities(&entities, &mut store);
/// ```
pub struct SnapToGridActuator {
    /// Current snap configuration
    config: SnapConfig,
}

impl SnapToGridActuator {
    /// Creates a new SnapToGridActuator with default configuration.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SnapConfig::default(),
        }
    }

    /// Creates a SnapToGridActuator with custom configuration.
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: SnapConfig) -> Self {
        Self { config }
    }

    /// Returns the current snap configuration.
    #[inline(always)]
    #[must_use]
    pub fn config(&self) -> SnapConfig {
        self.config
    }

    /// Updates the snap configuration.
    #[inline(always)]
    pub fn set_config(&mut self, config: SnapConfig) {
        self.config = config;
    }

    /// Sets the grid cell size.
    #[inline(always)]
    pub fn set_grid_size(&mut self, size: f32) {
        self.config.grid_size = size.max(1.0);
    }

    /// Sets the snap threshold.
    #[inline(always)]
    pub fn set_snap_threshold(&mut self, threshold: f32) {
        self.config.snap_threshold = threshold.max(0.0);
    }

    /// Enables or disables horizontal snapping.
    #[inline(always)]
    pub fn set_snap_x(&mut self, enabled: bool) {
        self.config.snap_x = enabled;
    }

    /// Enables or disables vertical snapping.
    #[inline(always)]
    pub fn set_snap_y(&mut self, enabled: bool) {
        self.config.snap_y = enabled;
    }

    /// Calculates the snapped position for a given world position.
    ///
    /// # Arguments
    ///
    /// * `position` - The original world position
    ///
    /// # Returns
    ///
    /// The snapped position, or the original if snapping didn't occur
    #[inline(always)]
    fn calculate_snapped_position(&self, position: Vec2) -> (Vec2, bool) {
        let mut snapped = position;
        let mut did_snap = false;

        // Snap to grid
        if self.config.snap_x {
            let grid_x = (position.x / self.config.grid_size).round() * self.config.grid_size;
            if (position.x - grid_x).abs() <= self.config.snap_threshold {
                snapped.x = grid_x;
                did_snap = true;
            }
        }

        if self.config.snap_y {
            let grid_y = (position.y / self.config.grid_size).round() * self.config.grid_size;
            if (position.y - grid_y).abs() <= self.config.snap_threshold {
                snapped.y = grid_y;
                did_snap = true;
            }
        }

        // Snap to origin if enabled and within threshold
        if self.config.snap_to_origin {
            if snapped.x.abs() <= self.config.snap_threshold {
                snapped.x = 0.0;
                did_snap = true;
            }
            if snapped.y.abs() <= self.config.snap_threshold {
                snapped.y = 0.0;
                did_snap = true;
            }
        }

        (snapped, did_snap)
    }

    /// Snaps a single entity to the grid.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity to snap
    /// * `store` - The entity store
    ///
    /// # Returns
    ///
    /// The snap result with position changes
    pub fn snap_entity(&mut self, entity_id: EntityId, store: &mut EntityStore) -> SnapResult {
        let idx = entity_id.index().0 as usize;

        if idx >= store.transforms.len() {
            return SnapResult::empty();
        }

        let original_pos = Vec2::new(store.transforms[idx][0], store.transforms[idx][1]);

        let (snapped_pos, did_snap) = self.calculate_snapped_position(original_pos);

        if did_snap {
            // Apply the snap
            store.transforms[idx][0] = snapped_pos.x;
            store.transforms[idx][1] = snapped_pos.y;
            store.dirty_transform.insert(idx);

            SnapResult {
                snapped_entities: vec![entity_id],
                original_positions: vec![original_pos],
                snapped_positions: vec![snapped_pos],
                did_snap: true,
            }
        } else {
            SnapResult::empty()
        }
    }

    /// Snaps multiple entities to the grid.
    ///
    /// # Arguments
    ///
    /// * `entity_ids` - The entities to snap
    /// * `store` - The entity store
    ///
    /// # Returns
    ///
    /// The snap result with all position changes
    pub fn snap_entities(
        &mut self,
        entity_ids: &[EntityId],
        store: &mut EntityStore,
    ) -> SnapResult {
        let mut result = SnapResult::empty();

        for &entity_id in entity_ids {
            let idx = entity_id.index().0 as usize;

            if idx >= store.transforms.len() {
                continue;
            }

            let original_pos = Vec2::new(store.transforms[idx][0], store.transforms[idx][1]);

            let (snapped_pos, did_snap) = self.calculate_snapped_position(original_pos);

            if did_snap {
                store.transforms[idx][0] = snapped_pos.x;
                store.transforms[idx][1] = snapped_pos.y;
                store.dirty_transform.insert(idx);

                result.snapped_entities.push(entity_id);
                result.original_positions.push(original_pos);
                result.snapped_positions.push(snapped_pos);
                result.did_snap = true;
            }
        }

        result
    }

    /// Creates move commands for snapping (for undo/redo).
    ///
    /// # Arguments
    ///
    /// * `entity_ids` - The entities to snap
    /// * `store` - The entity store
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo history
    pub fn create_snap_commands(
        &self,
        entity_ids: &[EntityId],
        store: &EntityStore,
    ) -> Vec<Command> {
        let mut commands = Vec::with_capacity(entity_ids.len());

        for &entity_id in entity_ids {
            let idx = entity_id.index().0 as usize;

            if idx >= store.transforms.len() {
                continue;
            }

            let original_pos = Vec2::new(store.transforms[idx][0], store.transforms[idx][1]);

            let (snapped_pos, _) = self.calculate_snapped_position(original_pos);

            // Calculate delta for Move command
            let delta = Vec2::new(
                snapped_pos.x - original_pos.x,
                snapped_pos.y - original_pos.y,
            );

            // Only create command if position changed
            if delta.x.abs() > 0.001 || delta.y.abs() > 0.001 {
                commands.push(Command::Move {
                    id: entity_id,
                    delta,
                });
            }
        }

        commands
    }

    /// Returns suggested snap positions for preview.
    ///
    /// # Arguments
    ///
    /// * `position` - The current position
    /// * `store` - The entity store (for collision checking)
    ///
    /// # Returns
    ///
    /// Vector of nearby grid snap points
    pub fn get_snap_suggestions(&self, position: Vec2, _store: &EntityStore) -> Vec<Vec2> {
        let mut suggestions = Vec::with_capacity(4);

        let (snapped, did_snap) = self.calculate_snapped_position(position);

        if did_snap {
            suggestions.push(snapped);
        }

        // Add nearby grid lines
        let nearby_x = (position.x / self.config.grid_size).round() * self.config.grid_size;
        let nearby_y = (position.y / self.config.grid_size).round() * self.config.grid_size;

        // Left/right of snap point
        if self.config.snap_x {
            suggestions.push(Vec2::new(nearby_x - self.config.grid_size, position.y));
            suggestions.push(Vec2::new(nearby_x + self.config.grid_size, position.y));
        }

        // Top/bottom of snap point
        if self.config.snap_y {
            suggestions.push(Vec2::new(position.x, nearby_y - self.config.grid_size));
            suggestions.push(Vec2::new(position.x, nearby_y + self.config.grid_size));
        }

        suggestions
    }
}

impl Default for SnapToGridActuator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_engine::EntityStore;

    fn create_test_store() -> EntityStore {
        EntityStore::new()
    }

    #[test]
    fn test_snap_single_entity() {
        let mut store = create_test_store();
        // 12.3 -> nearest grid 10.0 (diff 2.3, within threshold 5)
        // 24.7 -> nearest grid 20.0 (diff 4.7, within threshold 5)
        let entity_id = store.spawn(Vec2::new(12.3, 24.7), Vec2::new(100.0, 60.0));

        let mut actuator = SnapToGridActuator::new();

        let result = actuator.snap_entity(entity_id, &mut store);

        assert!(result.has_snapped());
        assert_eq!(result.snapped_entities.len(), 1);
        assert_eq!(result.original_positions[0], Vec2::new(12.3, 24.7));
        // 12.3 snaps to 10.0 (nearest grid line)
        assert!((result.snapped_positions[0].x - 10.0).abs() < 0.001);
        // 24.7 snaps to 20.0 (nearest grid line)
        assert!((result.snapped_positions[0].y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_snap_disabled_axes() {
        let mut store = create_test_store();
        // 12.3 -> nearest grid 10.0 (diff 2.3)
        // 24.7 -> nearest grid 20.0 (diff 4.7)
        let entity_id = store.spawn(Vec2::new(12.3, 24.7), Vec2::new(100.0, 60.0));

        let mut actuator = SnapToGridActuator::new();
        actuator.set_snap_x(false); // Disable X snapping
        actuator.set_snap_y(false); // Disable Y snapping

        let result = actuator.snap_entity(entity_id, &mut store);

        assert!(!result.has_snapped());
    }

    #[test]
    fn test_snap_multiple_entities() {
        let mut store = create_test_store();
        let entity1 = store.spawn(Vec2::new(12.3, 24.7), Vec2::new(100.0, 60.0));
        let entity2 = store.spawn(Vec2::new(8.1, 31.9), Vec2::new(80.0, 40.0));
        // entity3 has snapping disabled on axes
        let entity3 = store.spawn(Vec2::new(17.0, 17.0), Vec2::new(50.0, 50.0));

        let mut actuator = SnapToGridActuator::new();
        actuator.set_snap_x(false);
        actuator.set_snap_y(false);
        let entities = vec![entity1, entity2, entity3];

        let result = actuator.snap_entities(&entities, &mut store);

        assert!(!result.has_snapped());
    }

    #[test]
    fn test_config_options() {
        let mut actuator = SnapToGridActuator::new();

        // Disable X snapping
        actuator.set_snap_x(false);
        assert!(!actuator.config().snap_x);

        // Disable Y snapping
        actuator.set_snap_y(false);
        assert!(!actuator.config().snap_y);

        // Change grid size
        actuator.set_grid_size(25.0);
        assert_eq!(actuator.config().grid_size, 25.0);

        // Change threshold
        actuator.set_snap_threshold(10.0);
        assert_eq!(actuator.config().snap_threshold, 10.0);
    }

    #[test]
    fn test_snap_to_origin() {
        let mut store = create_test_store();
        let entity_id = store.spawn(Vec2::new(3.5, 4.2), Vec2::new(100.0, 60.0));

        let mut actuator = SnapToGridActuator::new();

        let result = actuator.snap_entity(entity_id, &mut store);

        assert!(result.has_snapped());
        assert!((result.snapped_positions[0].x - 0.0).abs() < 0.001);
        assert!((result.snapped_positions[0].y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_custom_grid_size() {
        let mut store = create_test_store();
        let entity_id = store.spawn(Vec2::new(27.0, 52.0), Vec2::new(100.0, 60.0));

        let mut actuator = SnapToGridActuator::with_config(SnapConfig {
            grid_size: 25.0,
            snap_threshold: 5.0,
            ..Default::default()
        });

        let result = actuator.snap_entity(entity_id, &mut store);

        assert!(result.has_snapped());
        // 27 should snap to 25, 52 should snap to 50
        assert!((result.snapped_positions[0].x - 25.0).abs() < 0.001);
        assert!((result.snapped_positions[0].y - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_create_snap_commands() {
        let mut store = create_test_store();
        let entity_id = store.spawn(Vec2::new(12.3, 24.7), Vec2::new(100.0, 60.0));

        let actuator = SnapToGridActuator::new();
        let entities = vec![entity_id];

        let commands = actuator.create_snap_commands(&entities, &store);

        assert_eq!(commands.len(), 1);
        if let Command::Move { id: cmd_id, delta } = &commands[0] {
            assert_eq!(*cmd_id, entity_id);
            // 12.3 should snap to 10.0, delta is -2.3
            assert!((delta.x - (-2.3)).abs() < 0.01);
            // 24.7 should snap to 20.0, delta is -4.7
            assert!((delta.y - (-4.7)).abs() < 0.01);
        }
    }

    #[test]
    fn test_empty_result() {
        let result = SnapResult::empty();
        assert!(!result.has_snapped());
        assert!(result.snapped_entities.is_empty());
    }
}
