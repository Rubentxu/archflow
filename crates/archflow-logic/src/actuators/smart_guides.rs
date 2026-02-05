// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - SmartGuidesActuator
//
// Actuator for smart alignment guides during entity manipulation.
// Implements US-011 from TEMA 2.
//
// Architecture:
// - SmartGuidesActuator: Detects alignment with nearby entities
// - Provides visual guide positions for alignment feedback
// - Uses spatial queries to find alignment candidates
//
// Performance Characteristics:
// - O(n) for finding alignment candidates
// - O(m log m) for sorting potential alignments
// - Uses spatial hashing for efficient neighbor queries
// ═════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Generation, Index, Vec2};
use archflow_engine::{Command, EntityStore};

/// Configuration for smart guides behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmartGuidesConfig {
    /// Detection distance for alignment (in pixels)
    pub detection_distance: f32,
    /// Maximum number of guides to show
    pub max_guides: usize,
    /// Enable horizontal guides
    pub horizontal_guides: bool,
    /// Enable vertical guides
    pub vertical_guides: bool,
    /// Enable center alignment detection
    pub detect_centers: bool,
    /// Enable edge alignment detection
    pub detect_edges: bool,
}

impl Default for SmartGuidesConfig {
    fn default() -> Self {
        Self {
            detection_distance: 8.0,
            max_guides: 4,
            horizontal_guides: true,
            vertical_guides: true,
            detect_centers: true,
            detect_edges: true,
        }
    }
}

/// Type of alignment detected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignmentType {
    /// Left edge alignment
    LeftEdge,
    /// Right edge alignment
    RightEdge,
    /// Top edge alignment
    TopEdge,
    /// Bottom edge alignment
    BottomEdge,
    /// Center horizontal alignment
    CenterHorizontal,
    /// Center vertical alignment
    CenterVertical,
}

/// A detected alignment guide.
#[derive(Clone, Debug, PartialEq)]
pub struct SmartGuide {
    /// Type of alignment
    pub alignment_type: AlignmentType,
    /// World position of the guide line
    pub position: f32,
    /// The entity this guide is aligned with
    pub target_entity: EntityId,
    /// Distance from the moving entity to this guide
    pub distance: f32,
}

/// Result of smart guide detection.
#[derive(Clone, Debug, PartialEq)]
pub struct SmartGuidesResult {
    /// Detected guides sorted by relevance
    pub guides: Vec<SmartGuide>,
    /// Whether any guides were detected
    pub has_guides: bool,
}

impl SmartGuidesResult {
    /// Creates an empty result.
    #[inline(always)]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            guides: Vec::new(),
            has_guides: false,
        }
    }
}

/// Actuator for smart alignment guides.
///
/// Provides visual feedback during entity manipulation:
/// - Edge alignment detection (left, right, top, bottom)
/// - Center alignment detection
/// - Configurable detection distance
/// - Priority-based guide selection
///
/// # Performance
/// - O(n) for entity scanning
/// - O(m log m) for sorting candidates
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::smart_guides::{SmartGuidesActuator, SmartGuidesConfig};
///
/// let config = SmartGuidesConfig {
///     detection_distance: 10.0,
///     ..Default::default()
/// };
///
/// let mut actuator = SmartGuidesActuator::with_config(config);
/// let store = /* ... */;
/// let moving_entity = /* ... */;
/// let other_entities = vec![e1, e2, e3];
///
/// // Get alignment guides
/// let result = actuator.detect_guides(moving_entity, &other_entities, &store);
/// ```
pub struct SmartGuidesActuator {
    /// Current configuration
    config: SmartGuidesConfig,
}

impl SmartGuidesActuator {
    /// Creates a new SmartGuidesActuator with default configuration.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SmartGuidesConfig::default(),
        }
    }

    /// Creates a SmartGuidesActuator with custom configuration.
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: SmartGuidesConfig) -> Self {
        Self { config }
    }

    /// Returns the current configuration.
    #[inline(always)]
    #[must_use]
    pub fn config(&self) -> SmartGuidesConfig {
        self.config
    }

    /// Updates the configuration.
    #[inline(always)]
    pub fn set_config(&mut self, config: SmartGuidesConfig) {
        self.config = config;
    }

    /// Sets the detection distance.
    #[inline(always)]
    pub fn set_detection_distance(&mut self, distance: f32) {
        self.config.detection_distance = distance.max(1.0);
    }

    /// Sets the maximum number of guides.
    #[inline(always)]
    pub fn set_max_guides(&mut self, max: usize) {
        self.config.max_guides = max.max(1);
    }

    /// Enables or disables horizontal guides.
    #[inline(always)]
    pub fn set_horizontal_guides(&mut self, enabled: bool) {
        self.config.horizontal_guides = enabled;
    }

    /// Enables or disables vertical guides.
    #[inline(always)]
    pub fn set_vertical_guides(&mut self, enabled: bool) {
        self.config.vertical_guides = enabled;
    }

    /// Gets the boundary positions for an entity.
    #[inline(always)]
    fn get_boundaries(&self, idx: usize, store: &EntityStore) -> (f32, f32, f32, f32) {
        let transform = store.transforms[idx];
        let x = transform[0];
        let y = transform[1];
        let w = transform[2];
        let h = transform[3];

        (x, x + w, y, y + h)
    }

    /// Detects alignment guides between a moving entity and others.
    ///
    /// # Arguments
    ///
    /// * `moving_entity` - The entity being moved
    /// * `other_entities` - Potential alignment candidates
    /// * `store` - The entity store
    ///
    /// # Returns
    ///
    /// Detected alignment guides sorted by distance
    pub fn detect_guides(
        &self,
        moving_entity: EntityId,
        other_entities: &[EntityId],
        store: &EntityStore,
    ) -> SmartGuidesResult {
        let moving_idx = moving_entity.index().0 as usize;
        if moving_idx >= store.transforms.len() {
            return SmartGuidesResult::empty();
        }

        let (m_left, m_right, m_top, m_bottom) = self.get_boundaries(moving_idx, store);
        let m_center_x = (m_left + m_right) / 2.0;
        let m_center_y = (m_top + m_bottom) / 2.0;

        let mut candidates: Vec<SmartGuide> = Vec::new();

        for &entity_id in other_entities {
            if entity_id == moving_entity {
                continue;
            }

            let idx = entity_id.index().0 as usize;
            if idx >= store.transforms.len() {
                continue;
            }

            let (o_left, o_right, o_top, o_bottom) = self.get_boundaries(idx, store);
            let o_center_x = (o_left + o_right) / 2.0;
            let o_center_y = (o_top + o_bottom) / 2.0;

            // Vertical alignments (left, right, center)
            if self.config.vertical_guides {
                // Left edge alignment
                if self.config.detect_edges {
                    let dist = (m_left - o_right).abs();
                    if dist <= self.config.detection_distance {
                        candidates.push(SmartGuide {
                            alignment_type: AlignmentType::LeftEdge,
                            position: o_right,
                            target_entity: entity_id,
                            distance: dist,
                        });
                    }

                    let dist = (m_right - o_left).abs();
                    if dist <= self.config.detection_distance {
                        candidates.push(SmartGuide {
                            alignment_type: AlignmentType::RightEdge,
                            position: o_left,
                            target_entity: entity_id,
                            distance: dist,
                        });
                    }
                }

                // Center vertical alignment
                if self.config.detect_centers {
                    let dist = (m_center_x - o_center_x).abs();
                    if dist <= self.config.detection_distance {
                        candidates.push(SmartGuide {
                            alignment_type: AlignmentType::CenterVertical,
                            position: o_center_x,
                            target_entity: entity_id,
                            distance: dist,
                        });
                    }
                }
            }

            // Horizontal alignments (top, bottom, center)
            if self.config.horizontal_guides {
                // Top edge alignment
                if self.config.detect_edges {
                    let dist = (m_top - o_bottom).abs();
                    if dist <= self.config.detection_distance {
                        candidates.push(SmartGuide {
                            alignment_type: AlignmentType::TopEdge,
                            position: o_bottom,
                            target_entity: entity_id,
                            distance: dist,
                        });
                    }

                    let dist = (m_bottom - o_top).abs();
                    if dist <= self.config.detection_distance {
                        candidates.push(SmartGuide {
                            alignment_type: AlignmentType::BottomEdge,
                            position: o_top,
                            target_entity: entity_id,
                            distance: dist,
                        });
                    }
                }

                // Center horizontal alignment
                if self.config.detect_centers {
                    let dist = (m_center_y - o_center_y).abs();
                    if dist <= self.config.detection_distance {
                        candidates.push(SmartGuide {
                            alignment_type: AlignmentType::CenterHorizontal,
                            position: o_center_y,
                            target_entity: entity_id,
                            distance: dist,
                        });
                    }
                }
            }
        }

        // Sort by distance (closest first)
        candidates.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        // Limit to max guides
        candidates.truncate(self.config.max_guides);

        // Check has_guides before moving candidates
        let has_guides = !candidates.is_empty();

        SmartGuidesResult {
            guides: candidates,
            has_guides,
        }
    }

    /// Gets the best snap position based on detected guides.
    ///
    /// # Arguments
    ///
    /// * `current_pos` - Current position of the entity
    /// * `guides` - Detected guides
    ///
    /// # Returns
    ///
    /// Suggested position snap, or original if no guides
    pub fn get_snap_position(&self, current_pos: Vec2, guides: &[SmartGuide]) -> Vec2 {
        if guides.is_empty() {
            return current_pos;
        }

        let mut snapped = current_pos;

        for guide in guides {
            match guide.alignment_type {
                AlignmentType::LeftEdge
                | AlignmentType::RightEdge
                | AlignmentType::CenterVertical => {
                    snapped.x = guide.position;
                }
                AlignmentType::TopEdge
                | AlignmentType::BottomEdge
                | AlignmentType::CenterHorizontal => {
                    snapped.y = guide.position;
                }
            }
            // Only apply the closest guide's alignment
            break;
        }

        snapped
    }
}

impl Default for SmartGuidesActuator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::EntityId;

    fn create_test_store() -> EntityStore {
        EntityStore::new()
    }

    fn entity_id(idx: u32) -> EntityId {
        EntityId::from_parts(Index(idx), Generation(0))
    }

    #[test]
    fn test_detect_left_edge_alignment() {
        let mut store = create_test_store();
        // Entity at x=100, width=50 -> right edge at 150
        let target = store.spawn(Vec2::new(100.0, 0.0), Vec2::new(50.0, 50.0));
        // Moving entity at x=148, width=30 -> left edge at 148
        // Distance to target's right edge = 2
        let moving = store.spawn(Vec2::new(148.0, 10.0), Vec2::new(30.0, 40.0));

        let actuator = SmartGuidesActuator::new();
        let others = vec![target];

        let result = actuator.detect_guides(moving, &others, &store);

        assert!(result.has_guides);
        assert!(
            result
                .guides
                .iter()
                .any(|g| g.alignment_type == AlignmentType::LeftEdge)
        );
    }

    #[test]
    fn test_no_guides_outside_threshold() {
        let mut store = create_test_store();
        // Target at x=100, width=50 -> right edge at 150
        let target = store.spawn(Vec2::new(100.0, 0.0), Vec2::new(50.0, 50.0));
        // Moving at x=200, width=30 -> left edge at 200
        // Distance = 50, exceeds threshold of 8
        let moving = store.spawn(Vec2::new(200.0, 100.0), Vec2::new(30.0, 40.0));

        let actuator = SmartGuidesActuator::new();
        let others = vec![target];

        let result = actuator.detect_guides(moving, &others, &store);

        // Distance is 50 for edges, exceeds threshold of 8
        assert!(!result.has_guides);
    }

    #[test]
    fn test_get_snap_position() {
        let actuator = SmartGuidesActuator::new();
        let current = Vec2::new(100.0, 200.0);

        let guides = vec![SmartGuide {
            alignment_type: AlignmentType::LeftEdge,
            position: 150.0,
            target_entity: entity_id(0),
            distance: 2.0,
        }];

        let snapped = actuator.get_snap_position(current, &guides);

        assert!((snapped.x - 150.0).abs() < 0.001);
        assert!((snapped.y - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_config_options() {
        let mut actuator = SmartGuidesActuator::new();

        actuator.set_detection_distance(15.0);
        assert_eq!(actuator.config().detection_distance, 15.0);

        actuator.set_max_guides(8);
        assert_eq!(actuator.config().max_guides, 8);

        actuator.set_horizontal_guides(false);
        assert!(!actuator.config().horizontal_guides);

        actuator.set_vertical_guides(false);
        assert!(!actuator.config().vertical_guides);
    }

    #[test]
    fn test_empty_result() {
        let actuator = SmartGuidesActuator::new();
        let mut store = create_test_store();
        let moving = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        let result = actuator.detect_guides(moving, &[], &store);

        assert!(!result.has_guides);
        assert!(result.guides.is_empty());
    }

    #[test]
    fn test_empty_result_no_snap() {
        let actuator = SmartGuidesActuator::new();
        let current = Vec2::new(100.0, 200.0);

        let snapped = actuator.get_snap_position(current, &[]);

        assert_eq!(snapped, current);
    }
}
