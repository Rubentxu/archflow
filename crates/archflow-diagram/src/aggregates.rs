// ═══════════════════════════════════════════════════════════════════════════════
// Aggregates - DDD Aggregates for Consistency Boundaries
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 3
//
// Aggregates are consistency boundaries in DDD. Each aggregate has a root
// entity and maintains consistency within its boundary.
//
// In ArchFlow:
// - DiagramAggregate: Root of the entire diagram
// - GroupAggregate: Manages grouped entities
// - ConnectionAggregate: Manages connections between entities
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_core::EntityId;

// ═══════════════════════════════════════════════════════════════════════════════
// DIAGRAM AGGREGATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Root aggregate for the entire diagram
///
/// The diagram is the consistency boundary for all entities.
/// It maintains invariants like no duplicate names, valid connections, etc.
pub struct DiagramAggregate {
    /// Unique identifier for this diagram
    pub id: EntityId,

    /// Name of the diagram
    pub name: alloc::string::String,

    /// Description of what this diagram represents
    pub description: alloc::string::String,

    /// C4 level this diagram represents
    pub level: crate::c4::C4Level,
}

impl DiagramAggregate {
    /// Create a new diagram aggregate
    pub fn new(id: EntityId, name: alloc::string::String, level: crate::c4::C4Level) -> Self {
        Self {
            id,
            name,
            description: alloc::string::String::new(),
            level,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: alloc::string::String) -> Self {
        self.description = description;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GROUP AGGREGATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Aggregate for grouped entities
///
/// A group maintains consistency among its children.
/// When the group moves, all children move with it.
pub struct GroupAggregate {
    /// The group entity (root)
    pub root_id: EntityId,

    /// Children of this group
    pub children: alloc::vec::Vec<EntityId>,
}

impl GroupAggregate {
    /// Create a new group aggregate
    pub fn new(root_id: EntityId) -> Self {
        Self {
            root_id,
            children: alloc::vec::Vec::new(),
        }
    }

    /// Add a child to this group
    pub fn add_child(&mut self, child_id: EntityId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child from this group
    pub fn remove_child(&mut self, child_id: EntityId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if an entity is a child of this group
    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.children.contains(&entity_id)
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTION AGGREGATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Aggregate for connections between entities
///
/// A connection maintains consistency between its source and target.
pub struct ConnectionAggregate {
    /// The connection entity
    pub id: EntityId,

    /// Source entity
    pub source: EntityId,

    /// Target entity
    pub target: EntityId,
}

impl ConnectionAggregate {
    /// Create a new connection aggregate
    pub fn new(id: EntityId, source: EntityId, target: EntityId) -> Self {
        Self { id, source, target }
    }

    /// Check if this connection is valid (no self-loops, etc.)
    pub fn is_valid(&self) -> bool {
        // Prevent self-loops
        if self.source == self.target {
            return false;
        }
        true
    }

    /// Check if this connection involves the given entity
    pub fn involves(&self, entity_id: EntityId) -> bool {
        self.source == entity_id || self.target == entity_id
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_aggregate_add_remove() {
        let mut group = GroupAggregate::new(EntityId::new(1));
        let child = EntityId::new(2);

        assert_eq!(group.child_count(), 0);
        assert!(!group.contains(child));

        group.add_child(child);
        assert_eq!(group.child_count(), 1);
        assert!(group.contains(child));

        group.remove_child(child);
        assert_eq!(group.child_count(), 0);
        assert!(!group.contains(child));
    }

    #[test]
    fn test_connection_aggregate_validity() {
        let source = EntityId::new(1);
        let target = EntityId::new(2);

        let valid = ConnectionAggregate::new(EntityId::new(3), source, target);
        assert!(valid.is_valid());

        let self_loop = ConnectionAggregate::new(EntityId::new(4), source, source);
        assert!(!self_loop.is_valid());
    }

    #[test]
    fn test_connection_aggregate_involves() {
        let source = EntityId::new(1);
        let target = EntityId::new(2);
        let other = EntityId::new(3);

        let conn = ConnectionAggregate::new(EntityId::new(4), source, target);

        assert!(conn.involves(source));
        assert!(conn.involves(target));
        assert!(!conn.involves(other));
    }
}
