// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Alignment Actuators
//
// Actuators for aligning and distributing entities: Align Left, Center, Right, Top, Middle, Bottom.
// Implements US-043 from TEMA 10.
//
// Architecture:
// - AlignmentActuator: Commands for aligning entities relative to each other
// - DistributionActuator: Commands for distributing entities evenly
// - Uses EntityStore's position data for O(n) alignment calculations
//
// Performance Characteristics:
// - O(n) for alignment where n = number of entities
// - O(n log n) for distribution (requires sorting)
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, MAX_ENTITIES, Vec2};
use archflow_engine::{Command, EntityStore};

/// Alignment direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    /// Align to leftmost entity
    Left,
    /// Align to center (horizontal)
    CenterHorizontal,
    /// Align to rightmost entity
    Right,
    /// Align to topmost entity
    Top,
    /// Align to middle (vertical)
    Middle,
    /// Align to bottommost entity
    Bottom,
}

/// Distribution axis
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistributionAxis {
    /// Distribute along X axis
    Horizontal,
    /// Distribute along Y axis
    Vertical,
}

/// Alignment operation data for undo/redo
#[derive(Clone, Debug, PartialEq)]
pub struct AlignmentOp {
    /// Entity that was moved
    entity: EntityId,
    /// Previous position
    old_pos: Vec2,
    /// New position
    new_pos: Vec2,
}

/// Actuator for aligning multiple entities.
///
/// Provides 6 alignment modes:
/// - `left()`: Align all entities to the leftmost entity
/// - `center_horizontal()`: Align to horizontal center
/// - `right()`: Align to rightmost entity
/// - `top()`: Align to topmost entity
/// - `middle()`: Align to vertical middle
/// - `bottom()`: Align to bottommost entity
///
/// # Performance
/// - O(n) for finding alignment reference
/// - O(n) for generating move commands
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::alignment::{AlignmentActuator, Alignment};
///
/// let mut actuator = AlignmentActuator::new();
/// let mut store = /* ... */;
/// let entities = vec![entity1, entity2, entity3];
///
/// // Align all to left
/// let cmds = actuator.align_left(&entities, &mut store);
/// ```
pub struct AlignmentActuator {
    /// Snap threshold for alignment (in pixels)
    snap_threshold: f32,
}

impl AlignmentActuator {
    /// Creates a new AlignmentActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            snap_threshold: 5.0, // 5 pixel snap threshold
        }
    }

    /// Creates an AlignmentActuator with custom snap threshold
    #[inline(always)]
    #[must_use]
    pub fn with_snap_threshold(threshold: f32) -> Self {
        Self {
            snap_threshold: threshold,
        }
    }

    /// Align entities to the left edge
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to align
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn align_left(&self, entities: &[EntityId], store: &EntityStore) -> Vec<Command> {
        self.align(Alignment::Left, entities, store)
    }

    /// Align entities to the horizontal center
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to align
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn align_center_horizontal(
        &self,
        entities: &[EntityId],
        store: &EntityStore,
    ) -> Vec<Command> {
        self.align(Alignment::CenterHorizontal, entities, store)
    }

    /// Align entities to the right edge
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to align
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn align_right(&self, entities: &[EntityId], store: &EntityStore) -> Vec<Command> {
        self.align(Alignment::Right, entities, store)
    }

    /// Align entities to the top edge
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to align
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn align_top(&self, entities: &[EntityId], store: &EntityStore) -> Vec<Command> {
        self.align(Alignment::Top, entities, store)
    }

    /// Align entities to the vertical middle
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to align
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn align_middle(&self, entities: &[EntityId], store: &EntityStore) -> Vec<Command> {
        self.align(Alignment::Middle, entities, store)
    }

    /// Align entities to the bottom edge
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to align
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn align_bottom(&self, entities: &[EntityId], store: &EntityStore) -> Vec<Command> {
        self.align(Alignment::Bottom, entities, store)
    }

    /// Core alignment implementation
    fn align(
        &self,
        alignment: Alignment,
        entities: &[EntityId],
        store: &EntityStore,
    ) -> Vec<Command> {
        if entities.len() <= 1 {
            return Vec::new();
        }

        // Collect alive entities with their positions
        let alive_entities: Vec<(EntityId, Vec2, Vec2)> = entities
            .iter()
            .filter_map(|&entity| {
                let idx = entity.index().0 as usize;
                if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
                    return None;
                }
                let pos = store.world_pos(idx);
                let size = store.size(idx);
                Some((entity, pos, size))
            })
            .collect();

        if alive_entities.len() <= 1 {
            return Vec::new();
        }

        // Calculate reference position based on alignment
        let reference = match alignment {
            Alignment::Left => {
                // Find minimum left edge
                alive_entities
                    .iter()
                    .map(|(_, pos, size)| pos.x - size.x / 2.0)
                    .fold(f32::MAX, f32::min)
            }
            Alignment::CenterHorizontal => {
                // Average of min and max center
                let min_center = alive_entities
                    .iter()
                    .map(|(_, pos, _)| pos.x)
                    .fold(f32::MAX, f32::min);
                let max_center = alive_entities
                    .iter()
                    .map(|(_, pos, _)| pos.x)
                    .fold(f32::MIN, f32::max);
                (min_center + max_center) / 2.0
            }
            Alignment::Right => {
                // Find maximum right edge
                alive_entities
                    .iter()
                    .map(|(_, pos, size)| pos.x + size.x / 2.0)
                    .fold(f32::MIN, f32::max)
            }
            Alignment::Top => {
                // Find minimum top edge (y decreases going up)
                alive_entities
                    .iter()
                    .map(|(_, pos, size)| pos.y - size.y / 2.0)
                    .fold(f32::MAX, f32::min)
            }
            Alignment::Middle => {
                // Average of min and max center
                let min_center = alive_entities
                    .iter()
                    .map(|(_, pos, _)| pos.y)
                    .fold(f32::MAX, f32::min);
                let max_center = alive_entities
                    .iter()
                    .map(|(_, pos, _)| pos.y)
                    .fold(f32::MIN, f32::max);
                (min_center + max_center) / 2.0
            }
            Alignment::Bottom => {
                // Find maximum bottom edge (y increases going down)
                alive_entities
                    .iter()
                    .map(|(_, pos, size)| pos.y + size.y / 2.0)
                    .fold(f32::MIN, f32::max)
            }
        };

        // Generate move commands
        let mut commands = Vec::with_capacity(alive_entities.len());

        for (entity, pos, size) in &alive_entities {
            let new_pos = match alignment {
                Alignment::Left | Alignment::CenterHorizontal | Alignment::Right => {
                    let target_center = match alignment {
                        Alignment::Left => reference + size.x / 2.0,
                        Alignment::CenterHorizontal => reference,
                        Alignment::Right => reference - size.x / 2.0,
                        _ => unreachable!(),
                    };
                    Vec2::new(target_center, pos.y)
                }
                Alignment::Top | Alignment::Middle | Alignment::Bottom => {
                    let target_center = match alignment {
                        Alignment::Top => reference + size.y / 2.0,
                        Alignment::Middle => reference,
                        Alignment::Bottom => reference - size.y / 2.0,
                        _ => unreachable!(),
                    };
                    Vec2::new(pos.x, target_center)
                }
            };

            // Only generate command if position actually changes
            if (new_pos.x - pos.x).abs() > self.snap_threshold
                || (new_pos.y - pos.y).abs() > self.snap_threshold
            {
                commands.push(Command::Teleport {
                    id: *entity,
                    pos: new_pos,
                });
            }
        }

        commands
    }

    /// Format notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize, alignment: Alignment) -> String {
        let action = match alignment {
            Alignment::Left => "aligned to left",
            Alignment::CenterHorizontal => "aligned to center",
            Alignment::Right => "aligned to right",
            Alignment::Top => "aligned to top",
            Alignment::Middle => "aligned to middle",
            Alignment::Bottom => "aligned to bottom",
        };

        if count == 1 {
            format!("1 entity {}", action)
        } else {
            format!("{} entities {}", count, action)
        }
    }
}

impl Default for AlignmentActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Distribution Actuator
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Actuator for distributing entities evenly.
///
/// Provides 2 distribution modes:
/// - `distribute_horizontally()`: Evenly space along X axis
/// - `distribute_vertically()`: Evenly space along Y axis
///
/// # Performance
/// - O(n log n) for sorting
/// - O(n) for generating move commands
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::alignment::{DistributionActuator, DistributionAxis};
///
/// let mut actuator = DistributionActuator::new();
/// let mut store = /* ... */;
/// let entities = vec![entity1, entity2, entity3];
///
/// // Distribute horizontally
/// let cmds = actuator.distribute_horizontally(&entities, &mut store);
/// ```
pub struct DistributionActuator {
    /// Minimum spacing between entities
    min_spacing: f32,
}

impl DistributionActuator {
    /// Creates a new DistributionActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_spacing: 10.0, // 10 pixel minimum spacing
        }
    }

    /// Creates a DistributionActuator with custom minimum spacing
    #[inline(always)]
    #[must_use]
    pub fn with_min_spacing(spacing: f32) -> Self {
        Self {
            min_spacing: spacing,
        }
    }

    /// Distribute entities evenly along horizontal axis
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to distribute
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn distribute_horizontally(
        &self,
        entities: &[EntityId],
        store: &EntityStore,
    ) -> Vec<Command> {
        self.distribute(DistributionAxis::Horizontal, entities, store)
    }

    /// Distribute entities evenly along vertical axis
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to distribute
    /// * `store` - EntityStore to read positions
    ///
    /// # Returns
    ///
    /// Vector of Move commands for undo/redo
    pub fn distribute_vertically(
        &self,
        entities: &[EntityId],
        store: &EntityStore,
    ) -> Vec<Command> {
        self.distribute(DistributionAxis::Vertical, entities, store)
    }

    /// Core distribution implementation
    fn distribute(
        &self,
        axis: DistributionAxis,
        entities: &[EntityId],
        store: &EntityStore,
    ) -> Vec<Command> {
        if entities.len() <= 2 {
            return Vec::new();
        }

        // Collect alive entities with their positions and sizes
        let mut alive_entities: Vec<(EntityId, Vec2, Vec2)> = entities
            .iter()
            .filter_map(|&entity| {
                let idx = entity.index().0 as usize;
                if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
                    return None;
                }
                let pos = store.world_pos(idx);
                let size = store.size(idx);
                Some((entity, pos, size))
            })
            .collect();

        if alive_entities.len() <= 2 {
            return Vec::new();
        }

        // Sort by position along distribution axis
        match axis {
            DistributionAxis::Horizontal => {
                alive_entities.sort_by(|a, b| {
                    a.1.x
                        .partial_cmp(&b.1.x)
                        .unwrap_or(core::cmp::Ordering::Equal)
                });
            }
            DistributionAxis::Vertical => {
                alive_entities.sort_by(|a, b| {
                    a.1.y
                        .partial_cmp(&b.1.y)
                        .unwrap_or(core::cmp::Ordering::Equal)
                });
            }
        }

        // Calculate bounds (first and last entity keep their positions)
        let first = &alive_entities[0];
        let last = &alive_entities[alive_entities.len() - 1];

        let (start_pos, start_size, end_pos, end_size) = match axis {
            DistributionAxis::Horizontal => (
                first.1.x - first.2.x / 2.0,
                first.2.x,
                last.1.x + last.2.x / 2.0,
                last.2.x,
            ),
            DistributionAxis::Vertical => (
                first.1.y - first.2.y / 2.0,
                first.2.y,
                last.1.y + last.2.y / 2.0,
                last.2.y,
            ),
        };

        // Calculate available space for distribution
        let total_entity_size: f32 = alive_entities
            .iter()
            .map(|(_, _, size)| match axis {
                DistributionAxis::Horizontal => size.x,
                DistributionAxis::Vertical => size.y,
            })
            .sum();

        let total_spacing = (end_pos - start_pos) - total_entity_size;
        let spacing_count = alive_entities.len() as f32 - 1.0;
        let spacing = if spacing_count > 0.0 {
            (total_spacing / spacing_count).max(self.min_spacing)
        } else {
            self.min_spacing
        };

        // Generate move commands
        let mut commands = Vec::with_capacity(alive_entities.len());

        for (i, (entity, pos, size)) in alive_entities.iter().enumerate() {
            let new_pos = match axis {
                DistributionAxis::Horizontal => {
                    // First and last stay, distribute middle ones
                    if i == 0 || i == alive_entities.len() - 1 {
                        Vec2::new(pos.x, pos.y)
                    } else {
                        // Distribute evenly between first and last
                        let t = i as f32 / (alive_entities.len() - 1) as f32;
                        let left_edge = first.1.x - first.2.x / 2.0;
                        let right_edge = last.1.x + last.2.x / 2.0;
                        let total_width = (right_edge - left_edge) - first.2.x - last.2.x;
                        let segment = total_width / (alive_entities.len() - 1) as f32;
                        let new_center =
                            left_edge + first.2.x + (segment * i as f32) + size.x / 2.0;
                        Vec2::new(new_center, pos.y)
                    }
                }
                DistributionAxis::Vertical => {
                    // First and last stay, distribute middle ones
                    if i == 0 || i == alive_entities.len() - 1 {
                        Vec2::new(pos.x, pos.y)
                    } else {
                        let top_edge = first.1.y - first.2.y / 2.0;
                        let bottom_edge = last.1.y + last.2.y / 2.0;
                        let total_height = (bottom_edge - top_edge) - first.2.y - last.2.y;
                        let segment = total_height / (alive_entities.len() - 1) as f32;
                        let new_center = top_edge + first.2.y + (segment * i as f32) + size.y / 2.0;
                        Vec2::new(pos.x, new_center)
                    }
                }
            };

            // Only generate command if position actually changes
            let delta_x = (new_pos.x - pos.x).abs();
            let delta_y = (new_pos.y - pos.y).abs();

            if delta_x > 1.0 || delta_y > 1.0 {
                commands.push(Command::Teleport {
                    id: *entity,
                    pos: new_pos,
                });
            }
        }

        commands
    }

    /// Format notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize, axis: DistributionAxis) -> String {
        let action = match axis {
            DistributionAxis::Horizontal => "distributed horizontally",
            DistributionAxis::Vertical => "distributed vertically",
        };

        if count == 1 {
            format!("1 entity {}", action)
        } else {
            format!("{} entities {}", count, action)
        }
    }
}

impl Default for DistributionActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════════════════
    // AlignmentActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_align_left() {
        let actuator = AlignmentActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 50.0), Vec2::new(20.0, 20.0)); // Center at 100, left at 90
        let e2 = store.spawn(Vec2::new(200.0, 50.0), Vec2::new(30.0, 30.0)); // Center at 200, left at 185
        let e3 = store.spawn(Vec2::new(300.0, 50.0), Vec2::new(40.0, 40.0)); // Center at 300, left at 280

        let cmds = actuator.align_left(&[e1, e2, e3], &store);

        // e1 should not move (already leftmost)
        // e2 should move from 200 to 90 + 15 = 105 (center = left + width/2)
        // e3 should move from 300 to 90 + 20 = 110 (center = left + width/2)
        assert_eq!(cmds.len(), 2);

        // Verify e2 new position
        if let Command::Teleport { id, pos } = &cmds[0] {
            if *id == e2 {
                assert!((pos.x - 105.0).abs() < 1.0);
            }
        }
    }

    #[test]
    fn test_align_right() {
        let actuator = AlignmentActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(200.0, 50.0), Vec2::new(30.0, 30.0));
        let e3 = store.spawn(Vec2::new(300.0, 50.0), Vec2::new(40.0, 40.0));

        let cmds = actuator.align_right(&[e1, e2, e3], &store);

        // e3 should not move (already rightmost at 320 right edge)
        // e1 should move to 320 - 10 = 310 (center = right - width/2)
        // e2 should move to 320 - 15 = 305 (center = right - width/2)
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_align_center_horizontal() {
        let actuator = AlignmentActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(200.0, 50.0), Vec2::new(30.0, 30.0));
        let e3 = store.spawn(Vec2::new(300.0, 50.0), Vec2::new(40.0, 40.0));

        let cmds = actuator.align_center_horizontal(&[e1, e2, e3], &store);

        // Centers: 100, 200, 300 → min=100, max=300, center=200
        // All should align to x=200
        assert_eq!(cmds.len(), 2); // e1 and e3 move
    }

    #[test]
    fn test_align_top() {
        let actuator = AlignmentActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 100.0), Vec2::new(20.0, 20.0)); // Center y=100, top=90
        let e2 = store.spawn(Vec2::new(50.0, 200.0), Vec2::new(30.0, 30.0)); // Center y=200, top=185
        let e3 = store.spawn(Vec2::new(50.0, 300.0), Vec2::new(40.0, 40.0)); // Center y=300, top=280

        let cmds = actuator.align_top(&[e1, e2, e3], &store);

        // e1 should not move (already topmost at y=90)
        // e2 should move to 90 + 15 = 105
        // e3 should move to 90 + 20 = 110
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_align_single_entity() {
        let actuator = AlignmentActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 50.0), Vec2::new(20.0, 20.0));

        let cmds = actuator.align_left(&[e1], &store);

        assert!(cmds.is_empty());
    }

    #[test]
    fn test_align_empty() {
        let actuator = AlignmentActuator::new();
        let store = EntityStore::new();

        let cmds = actuator.align_left(&[], &store);

        assert!(cmds.is_empty());
    }

    #[test]
    fn test_format_message() {
        let actuator = AlignmentActuator::new();

        assert_eq!(
            actuator.format_message(1, Alignment::Left),
            "1 entity aligned to left"
        );
        assert_eq!(
            actuator.format_message(5, Alignment::CenterHorizontal),
            "5 entities aligned to center"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════════════════
    // DistributionActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_distribute_horizontally() {
        let actuator = DistributionActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(250.0, 50.0), Vec2::new(20.0, 20.0));
        let e4 = store.spawn(Vec2::new(350.0, 50.0), Vec2::new(20.0, 20.0));

        // 4 entities at 50, 150, 250, 350 → spread to cover 100-300 range
        let cmds = actuator.distribute_horizontally(&[e1, e2, e3, e4], &store);

        // e1 and e4 should stay, e2 and e3 should move
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_distribute_vertically() {
        let actuator = DistributionActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(50.0, 150.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(50.0, 250.0), Vec2::new(20.0, 20.0));

        let cmds = actuator.distribute_vertically(&[e1, e2, e3], &store);

        // e1 and e3 should stay, e2 should move
        assert_eq!(cmds.len(), 1);
    }

    #[test]
    fn test_distribute_single_entity() {
        let actuator = DistributionActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 50.0), Vec2::new(20.0, 20.0));

        let cmds = actuator.distribute_horizontally(&[e1], &store);

        assert!(cmds.is_empty());
    }

    #[test]
    fn test_distribution_format_message() {
        let actuator = DistributionActuator::new();

        assert_eq!(
            actuator.format_message(3, DistributionAxis::Horizontal),
            "3 entities distributed horizontally"
        );
    }
}
