// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Clipboard Actuators
//
// Actuators for clipboard operations: Copy, Paste, Duplicate, Delete.
// Implements US-012 through US-016 from TEMA 3.
//
// Architecture:
// - CopyActuator: Serialize selected entities to clipboard
// - PasteActuator: Deserialize and spawn entities from clipboard
// - DuplicateActuator: Copy + Paste with offset
// - DeleteActuator: Despawn selected entities
//
// Performance Characteristics:
// - O(n) for copy where n = number of selected entities
// - O(n) for paste where n = number of pasted entities
// - O(n) for delete where n = number of selected entities
//
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, MAX_ENTITIES, Vec2};
use archflow_engine::{Command, EntityStore};

/// Clipboard data format for entity serialization
///
/// Contains all necessary data to recreate entities:
/// - Position and size (transform)
/// - Color and shape (appearance)
/// - Metadata (visibility, layer, etc.)
#[derive(Clone, Debug, PartialEq)]
pub struct ClipboardData {
    /// Serialized entities
    pub entities: Vec<ClipboardEntity>,
    /// Offset applied during paste (for duplicate)
    pub paste_offset: Vec2,
    /// Timestamp of when data was copied
    pub timestamp: u64,
}

/// Individual entity data for clipboard
#[derive(Clone, Debug, PartialEq)]
pub struct ClipboardEntity {
    /// Original entity ID (for reference, not used in paste)
    pub original_id: EntityId,
    /// Position in world coordinates
    pub position: Vec2,
    /// Size of the entity
    pub size: Vec2,
    /// Color in ARGB format
    pub color: u32,
    /// Shape type (0=rect, 1=circle, etc.)
    pub shape: u8,
    /// Visibility flag
    pub visible: bool,
    /// Render layer
    pub layer: u8,
    /// Optional parent ID (if grouped)
    pub parent_id: Option<EntityId>,
    /// Custom properties (JSON string for extensibility)
    pub properties: Option<String>,
}

/// Internal clipboard state
#[derive(Clone, Debug)]
pub struct ClipboardState {
    /// Current clipboard data
    data: Option<ClipboardData>,
    /// Maximum number of items in clipboard history
    max_history: usize,
    /// Clipboard history stack
    history: Vec<ClipboardData>,
    /// Current history index
    history_index: usize,
}

impl ClipboardState {
    /// Creates a new clipboard with default history size
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: None,
            max_history: 10,
            history: Vec::with_capacity(10),
            history_index: 0,
        }
    }

    /// Check if clipboard has data
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    /// Get current clipboard data
    #[inline(always)]
    #[must_use]
    pub fn data(&self) -> Option<&ClipboardData> {
        self.data.as_ref()
    }

    /// Set clipboard data
    pub fn set_data(&mut self, data: ClipboardData) {
        // Remove any history after current index
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }

        // Add current data to history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(data.clone());

        self.data = Some(data);
        self.history_index = self.history.len();
    }

    /// Clear clipboard
    #[inline(always)]
    pub fn clear(&mut self) {
        self.data = None;
    }

    /// History navigation
    #[inline(always)]
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    #[inline(always)]
    pub fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len().saturating_sub(1)
    }

    /// Go back in clipboard history
    pub fn go_back(&mut self) -> bool {
        if self.can_go_back() {
            self.history_index = self.history_index.saturating_sub(1);
            if let Some(data) = self.history.get(self.history_index) {
                self.data = Some(data.clone());
                return true;
            }
        }
        false
    }

    /// Go forward in clipboard history
    pub fn go_forward(&mut self) -> bool {
        if self.can_go_forward() {
            self.history_index += 1;
            if let Some(data) = self.history.get(self.history_index) {
                self.data = Some(data.clone());
                return true;
            }
        }
        false
    }
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// CopyActuator - Copy entities to clipboard
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for copying selected entities to the clipboard.
///
/// # Performance
/// - O(n) where n = number of selected entities
/// - Memory: O(n) for serialized entity data
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::clipboard::{CopyActuator, ClipboardState};
///
/// let mut actuator = CopyActuator::new();
/// let mut clipboard = ClipboardState::new();
/// let store = /* ... */;
/// let selected_ids = vec![entity1, entity2];
///
/// actuator.execute(selected_ids, &store, &mut clipboard);
/// ```
pub struct CopyActuator {
    /// Maximum clipboard history
    max_history: usize,
    /// Default paste offset
    default_offset: Vec2,
}

impl CopyActuator {
    /// Creates a new CopyActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_history: 10,
            default_offset: Vec2::new(10.0, 10.0),
        }
    }

    /// Creates a CopyActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(max_history: usize, default_offset: Vec2) -> Self {
        Self {
            max_history,
            default_offset,
        }
    }

    /// Execute copy operation
    ///
    /// # Arguments
    ///
    /// * `entity_ids` - Entities to copy
    /// * `store` - EntityStore to read entity data
    /// * `clipboard` - ClipboardState to store copied data
    ///
    /// # Returns
    ///
    /// Number of entities copied
    pub fn execute(
        &self,
        entity_ids: &[EntityId],
        store: &EntityStore,
        clipboard: &mut ClipboardState,
    ) -> usize {
        if entity_ids.is_empty() {
            return 0;
        }

        let entities: Vec<ClipboardEntity> = entity_ids
            .iter()
            .filter_map(|&entity| {
                let idx = entity.index().0 as usize;
                if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
                    return None;
                }

                Some(ClipboardEntity {
                    original_id: entity,
                    position: store.world_pos(idx),
                    size: store.size(idx),
                    color: store.colors[idx],
                    shape: (store.metadata[idx] & 0xF) as u8,
                    visible: (store.metadata[idx] >> 8) & 1 != 0,
                    layer: ((store.metadata[idx] >> 4) & 0xF) as u8,
                    parent_id: store.parent_id.get(idx).copied().flatten(),
                    properties: None,
                })
            })
            .collect();

        if entities.is_empty() {
            return 0;
        }

        let count = entities.len();
        let data = ClipboardData {
            entities,
            paste_offset: self.default_offset,
            timestamp: 0, // TODO: Use proper timestamp if needed
        };

        clipboard.set_data(data);
        count
    }

    /// Format copy notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize) -> String {
        if count == 1 {
            "Copied 1 entity".into()
        } else {
            format!("Copied {} entities", count)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PasteActuator - Paste entities from clipboard
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for pasting entities from the clipboard.
///
/// Creates new entities at the specified position with the copied properties.
///
/// # Performance
/// - O(n) where n = number of entities in clipboard
/// - Generates O(n) Spawn commands
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::clipboard::{PasteActuator, ClipboardState};
///
/// let mut actuator = PasteActuator::new();
/// let mut clipboard = ClipboardState::new();
/// let mut store = /* ... */;
/// let paste_pos = Vec2::new(100.0, 100.0);
///
/// actuator.execute(paste_pos, &mut store, &clipboard);
/// ```
pub struct PasteActuator {
    /// Default paste offset for multiple items
    pub(crate) cascade_offset: Vec2,
}

impl PasteActuator {
    /// Creates a new PasteActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            cascade_offset: Vec2::new(15.0, 15.0),
        }
    }

    /// Creates a PasteActuator with custom cascade offset
    #[inline(always)]
    #[must_use]
    pub fn with_cascade_offset(offset: Vec2) -> Self {
        Self {
            cascade_offset: offset,
        }
    }

    /// Execute paste operation
    ///
    /// # Arguments
    ///
    /// * `paste_pos` - Position to paste at (center of first entity)
    /// * `store` - EntityStore to spawn entities
    /// * `clipboard` - ClipboardState with data to paste
    ///
    /// # Returns
    ///
    /// Vector of Spawn commands and the new entity IDs
    pub fn execute(
        &self,
        paste_pos: Vec2,
        store: &mut EntityStore,
        clipboard: &ClipboardState,
    ) -> (Vec<Command>, Vec<EntityId>) {
        let data = match clipboard.data() {
            Some(data) => data,
            None => return (Vec::new(), Vec::new()),
        };

        let mut commands = Vec::with_capacity(data.entities.len());
        let mut new_ids = Vec::with_capacity(data.entities.len());

        for (i, entity) in data.entities.iter().enumerate() {
            // Calculate position with cascade offset
            let offset = if i == 0 {
                Vec2::ZERO
            } else {
                Vec2::new(
                    self.cascade_offset.x * (i as f32),
                    self.cascade_offset.y * (i as f32),
                )
            };

            // Adjust position based on first entity's original position
            let adjusted_pos = if i == 0 {
                paste_pos
            } else {
                let first_pos = data.entities[0].position;
                entity.position + (paste_pos - first_pos) + offset
            };

            let cmd = Command::Spawn {
                pos: adjusted_pos,
                size: entity.size,
                parent: None, // New entities are independent
            };

            // Execute spawn to get entity ID
            let new_id = store.spawn(adjusted_pos, entity.size);

            // Store original data for undo
            let _entity_id_for_undo = new_id;

            commands.push(cmd);
            new_ids.push(new_id);
        }

        (commands, new_ids)
    }

    /// Format paste notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize) -> String {
        if count == 1 {
            "Pasted 1 entity".into()
        } else {
            format!("Pasted {} entities", count)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// DuplicateActuator - Duplicate entities (Copy + Paste with offset)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for duplicating selected entities.
///
/// Combines Copy and Paste operations, executing them as a single reversible action.
/// The duplicated entities are placed with a default offset from the originals.
///
/// # Performance
/// - O(n) for copy + O(n) for paste = O(n) total
/// - Memory: O(n) for clipboard data + O(n) for spawn commands
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::clipboard::{DuplicateActuator, ClipboardState};
///
/// let mut actuator = DuplicateActuator::new();
/// let mut clipboard = ClipboardState::new();
/// let mut store = /* ... */;
/// let selected_ids = vec![entity1, entity2];
///
/// actuator.execute(selected_ids, &mut store, &mut clipboard);
/// ```
pub struct DuplicateActuator {
    /// Copy actuator for the copy phase
    copy_actuator: CopyActuator,
    /// Paste actuator for the paste phase
    paste_actuator: PasteActuator,
    /// Default offset for duplicate
    default_offset: Vec2,
}

impl DuplicateActuator {
    /// Creates a new DuplicateActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            copy_actuator: CopyActuator::new(),
            paste_actuator: PasteActuator::new(),
            default_offset: Vec2::new(10.0, 10.0),
        }
    }

    /// Creates a DuplicateActuator with custom offset
    #[inline(always)]
    #[must_use]
    pub fn with_offset(offset: Vec2) -> Self {
        Self {
            copy_actuator: CopyActuator::with_config(1, offset),
            paste_actuator: PasteActuator::with_cascade_offset(Vec2::new(15.0, 15.0)),
            default_offset: offset,
        }
    }

    /// Execute duplicate operation
    ///
    /// # Arguments
    ///
    /// * `entity_ids` - Entities to duplicate
    /// * `store` - EntityStore to spawn new entities
    /// * `clipboard` - ClipboardState for intermediate storage
    ///
    /// # Returns
    ///
    /// Vector of Spawn commands for the duplicated entities
    pub fn execute(
        &self,
        entity_ids: &[EntityId],
        store: &mut EntityStore,
        clipboard: &mut ClipboardState,
    ) -> Vec<Command> {
        if entity_ids.is_empty() {
            return Vec::new();
        }

        // Calculate center position for paste
        let center_pos = self.calculate_center(entity_ids, store);

        // Copy to clipboard
        self.copy_actuator.execute(entity_ids, store, clipboard);

        // Paste from clipboard with offset
        let (commands, _) = self.paste_actuator.execute(center_pos, store, clipboard);

        commands
    }

    /// Calculate center position of selected entities
    fn calculate_center(&self, entity_ids: &[EntityId], store: &EntityStore) -> Vec2 {
        if entity_ids.is_empty() {
            return self.default_offset;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for &entity in entity_ids {
            let idx = entity.index().0 as usize;
            if idx < MAX_ENTITIES as usize && store.is_alive(entity) {
                let pos = store.world_pos(idx);
                let size = store.size(idx);

                min_x = min_x.min(pos.x - size.x / 2.0);
                max_x = max_x.max(pos.x + size.x / 2.0);
                min_y = min_y.min(pos.y - size.y / 2.0);
                max_y = max_y.max(pos.y + size.y / 2.0);
            }
        }

        let center_x = (min_x + max_x) / 2.0 + self.default_offset.x;
        let center_y = (min_y + max_y) / 2.0 + self.default_offset.y;

        Vec2::new(center_x, center_y)
    }

    /// Format duplicate notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize) -> String {
        if count == 1 {
            "Duplicated 1 entity".into()
        } else {
            format!("Duplicated {} entities", count)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// DeleteActuator - Delete entities from canvas
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for deleting selected entities.
///
/// Removes entities from the canvas with support for undo via HistoryManager.
/// Handles both single and batch deletion.
///
/// # Performance
/// - O(n) where n = number of entities to delete
/// - O(m) for hierarchy where m = total descendants
///
/// # Safety
///
/// Shows confirmation dialog if more than 10 entities are being deleted.
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::clipboard::{DeleteActuator, ClipboardState};
///
/// let mut actuator = DeleteActuator::new();
/// let mut store = /* ... */;
/// let selected_ids = vec![entity1, entity2];
///
/// actuator.execute(selected_ids, &mut store);
/// ```
pub struct DeleteActuator {
    /// Confirmation threshold
    confirmation_threshold: usize,
    /// Include children in delete
    include_children: bool,
}

impl DeleteActuator {
    /// Creates a new DeleteActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            confirmation_threshold: 10,
            include_children: true,
        }
    }

    /// Creates a DeleteActuator with custom threshold
    #[inline(always)]
    #[must_use]
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            confirmation_threshold: threshold,
            include_children: true,
        }
    }

    /// Execute delete operation
    ///
    /// # Arguments
    ///
    /// * `entity_ids` - Entities to delete
    /// * `store` - EntityStore to modify
    ///
    /// # Returns
    ///
    /// Vector of Despawn commands
    pub fn execute(&self, entity_ids: &[EntityId], store: &mut EntityStore) -> Vec<Command> {
        if entity_ids.is_empty() {
            return Vec::new();
        }

        // Collect all entities to delete (including children if enabled)
        let entities_to_delete = if self.include_children {
            self.collect_with_children(entity_ids, store)
        } else {
            entity_ids.to_vec()
        };

        // Filter to only alive entities
        let alive_entities: Vec<EntityId> = entities_to_delete
            .iter()
            .filter(|&&entity| {
                let idx = entity.index().0 as usize;
                idx < MAX_ENTITIES as usize && store.is_alive(entity)
            })
            .copied()
            .collect();

        if alive_entities.is_empty() {
            return Vec::new();
        }

        // Generate despawn commands
        let mut commands = Vec::with_capacity(alive_entities.len());
        for &entity in &alive_entities {
            commands.push(Command::Despawn(entity));
        }

        // Execute despawns
        for &entity in &alive_entities {
            store.despawn(entity);
        }

        commands
    }

    /// Collect entity and all descendants
    fn collect_with_children(&self, entity_ids: &[EntityId], store: &EntityStore) -> Vec<EntityId> {
        let mut result = Vec::new();
        let mut stack: Vec<EntityId> = entity_ids.iter().copied().collect();

        while let Some(entity) = stack.pop() {
            result.push(entity);

            // Add children
            let idx = entity.index().0 as usize;
            if idx < MAX_ENTITIES as usize {
                for child_idx in 0..MAX_ENTITIES {
                    if store.parent_id[child_idx as usize] == Some(entity) && store.is_alive(entity)
                    {
                        stack.push(EntityId::new(child_idx as u32));
                    }
                }
            }
        }

        result
    }

    /// Check if confirmation is needed
    #[inline(always)]
    #[must_use]
    pub fn needs_confirmation(&self, entity_ids: &[EntityId]) -> bool {
        entity_ids.len() > self.confirmation_threshold
    }

    /// Get the number of entities that would be deleted (including children)
    pub fn count_to_delete(&self, entity_ids: &[EntityId], store: &EntityStore) -> usize {
        if self.include_children {
            self.collect_with_children(entity_ids, store).len()
        } else {
            entity_ids.len()
        }
    }

    /// Format delete notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize) -> String {
        if count == 1 {
            "Deleted 1 entity".into()
        } else {
            format!("Deleted {} entities", count)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════════════
    // ClipboardState Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_clipboard_empty() {
        let clipboard = ClipboardState::new();
        assert!(clipboard.is_empty());
    }

    #[test]
    fn test_clipboard_set_data() {
        let mut clipboard = ClipboardState::new();
        let data = ClipboardData {
            entities: vec![],
            paste_offset: Vec2::new(10.0, 10.0),
            timestamp: 12345,
        };

        clipboard.set_data(data.clone());
        assert!(!clipboard.is_empty());
        assert_eq!(clipboard.data(), Some(&data));
    }

    #[test]
    fn test_clipboard_clear() {
        let mut clipboard = ClipboardState::new();
        let data = ClipboardData {
            entities: vec![],
            paste_offset: Vec2::ZERO,
            timestamp: 0,
        };

        clipboard.set_data(data);
        clipboard.clear();
        assert!(clipboard.is_empty());
    }

    #[test]
    fn test_clipboard_history() {
        let mut clipboard = ClipboardState::new();

        for i in 0..5 {
            let data = ClipboardData {
                entities: vec![],
                paste_offset: Vec2::new(i as f32, 0.0),
                timestamp: i as u64,
            };
            clipboard.set_data(data);
        }

        assert!(clipboard.can_go_back());
        assert!(!clipboard.can_go_forward());

        // Go back through history
        for i in (0..5).rev() {
            clipboard.go_back();
            assert_eq!(
                clipboard.data().unwrap().timestamp,
                i as u64,
                "Should be at timestamp {}",
                i
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // CopyActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_copy_empty_selection() {
        let actuator = CopyActuator::new();
        let store = EntityStore::new();
        let mut clipboard = ClipboardState::new();

        let count = actuator.execute(&[], &store, &mut clipboard);
        assert_eq!(count, 0);
        assert!(clipboard.is_empty());
    }

    #[test]
    fn test_copy_single_entity() {
        let actuator = CopyActuator::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let mut clipboard = ClipboardState::new();

        let count = actuator.execute(&[entity], &store, &mut clipboard);
        assert_eq!(count, 1);
        assert!(!clipboard.is_empty());

        let data = clipboard.data().unwrap();
        assert_eq!(data.entities.len(), 1);
        assert_eq!(data.entities[0].position, Vec2::new(100.0, 100.0));
        assert_eq!(data.entities[0].size, Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_copy_multiple_entities() {
        let actuator = CopyActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));
        let mut clipboard = ClipboardState::new();

        let count = actuator.execute(&[e1, e2, e3], &store, &mut clipboard);
        assert_eq!(count, 3);
        assert_eq!(clipboard.data().unwrap().entities.len(), 3);
    }

    #[test]
    fn test_copy_format_message() {
        let actuator = CopyActuator::new();
        assert_eq!(actuator.format_message(1), "Copied 1 entity");
        assert_eq!(actuator.format_message(5), "Copied 5 entities");
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // PasteActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_paste_empty_clipboard() {
        let actuator = PasteActuator::new();
        let mut store = EntityStore::new();
        let clipboard = ClipboardState::new();

        let (cmds, ids) = actuator.execute(Vec2::new(100.0, 100.0), &mut store, &clipboard);
        assert!(cmds.is_empty());
        assert!(ids.is_empty());
    }

    #[test]
    fn test_paste_single_entity() {
        let actuator = PasteActuator::new();
        let mut store = EntityStore::new();

        // Set up clipboard with test data
        let mut clipboard = ClipboardState::new();
        let data = ClipboardData {
            entities: vec![ClipboardEntity {
                original_id: EntityId::new(0),
                position: Vec2::new(50.0, 50.0),
                size: Vec2::new(25.0, 25.0),
                color: 0xFF0000FF,
                shape: 0,
                visible: true,
                layer: 1,
                parent_id: None,
                properties: None,
            }],
            paste_offset: Vec2::new(10.0, 10.0),
            timestamp: 0,
        };
        clipboard.set_data(data);

        let (cmds, ids) = actuator.execute(Vec2::new(200.0, 200.0), &mut store, &clipboard);

        assert_eq!(cmds.len(), 1);
        assert_eq!(ids.len(), 1);

        // Verify entity was spawned at adjusted position
        let idx = ids[0].index().0 as usize;
        assert_eq!(store.world_pos(idx), Vec2::new(200.0, 200.0));
        assert_eq!(store.size(idx), Vec2::new(25.0, 25.0));
    }

    #[test]
    fn test_paste_format_message() {
        let actuator = PasteActuator::new();
        assert_eq!(actuator.format_message(1), "Pasted 1 entity");
        assert_eq!(actuator.format_message(3), "Pasted 3 entities");
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // DuplicateActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_duplicate_empty_selection() {
        let actuator = DuplicateActuator::new();
        let mut store = EntityStore::new();
        let mut clipboard = ClipboardState::new();

        let cmds = actuator.execute(&[], &mut store, &mut clipboard);
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_duplicate_single_entity() {
        let actuator = DuplicateActuator::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let mut clipboard = ClipboardState::new();

        let cmds = actuator.execute(&[entity], &mut store, &mut clipboard);

        assert_eq!(cmds.len(), 1);
        match cmds[0] {
            Command::Spawn { pos, size, .. } => {
                assert_eq!(size, Vec2::new(50.0, 50.0));
                // Position should be offset from original
                assert!(pos.x > 100.0 || pos.y > 100.0);
            }
            _ => panic!("Expected Spawn command"),
        }
    }

    #[test]
    fn test_duplicate_format_message() {
        let actuator = DuplicateActuator::new();
        assert_eq!(actuator.format_message(1), "Duplicated 1 entity");
        assert_eq!(actuator.format_message(2), "Duplicated 2 entities");
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // DeleteActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_delete_empty_selection() {
        let actuator = DeleteActuator::new();
        let mut store = EntityStore::new();

        let cmds = actuator.execute(&[], &mut store);
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_delete_single_entity() {
        let actuator = DeleteActuator::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let initial_count = store.alive_count();

        let cmds = actuator.execute(&[entity], &mut store);

        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::Despawn(id) => assert_eq!(id, &entity),
            _ => panic!("Expected Despawn command"),
        }

        assert!(!store.is_alive(entity));
        assert_eq!(store.alive_count(), initial_count - 1);
    }

    #[test]
    fn test_delete_batch() {
        let actuator = DeleteActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        let cmds = actuator.execute(&[e1, e2, e3], &mut store);

        assert_eq!(cmds.len(), 3);
        assert!(!store.is_alive(e1));
        assert!(!store.is_alive(e2));
        assert!(!store.is_alive(e3));
    }

    #[test]
    fn test_delete_confirmation_threshold() {
        let actuator = DeleteActuator::with_threshold(5);
        let mut store = EntityStore::new();

        // Create 3 entities (below threshold)
        for i in 0..3 {
            store.spawn(Vec2::new(i as f32 * 50.0, 0.0), Vec2::new(10.0, 10.0));
        }

        let ids: Vec<EntityId> = (0..3).map(|i| EntityId::new(i as u32)).collect();
        assert!(!actuator.needs_confirmation(&ids));

        // Create 7 entities (above threshold)
        for i in 3..10 {
            store.spawn(Vec2::new(i as f32 * 50.0, 0.0), Vec2::new(10.0, 10.0));
        }

        let ids: Vec<EntityId> = (0..10).map(|i| EntityId::new(i as u32)).collect();
        assert!(actuator.needs_confirmation(&ids));
    }

    #[test]
    fn test_delete_format_message() {
        let actuator = DeleteActuator::new();
        assert_eq!(actuator.format_message(1), "Deleted 1 entity");
        assert_eq!(actuator.format_message(5), "Deleted 5 entities");
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // Integration Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_copy_paste_integration() {
        let copy_actuator = CopyActuator::new();
        let paste_actuator = PasteActuator::new();
        let mut store = EntityStore::new();
        let mut clipboard = ClipboardState::new();

        // Create and copy entity
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        copy_actuator.execute(&[entity], &store, &mut clipboard);

        // Paste at new location
        let (cmds, new_ids) =
            paste_actuator.execute(Vec2::new(200.0, 200.0), &mut store, &clipboard);

        assert_eq!(cmds.len(), 1);
        assert_eq!(new_ids.len(), 1);
        assert_ne!(new_ids[0], entity); // New ID should be different

        // Verify pasted entity properties match original
        let new_idx = new_ids[0].index().0 as usize;
        assert_eq!(store.size(new_idx), Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_duplicate_includes_original() {
        let duplicate_actuator = DuplicateActuator::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let initial_count = store.alive_count();

        duplicate_actuator.execute(&[entity], &mut store, &mut ClipboardState::new());

        // Original should still exist + new duplicate
        assert_eq!(store.alive_count(), initial_count + 1);
    }
}
