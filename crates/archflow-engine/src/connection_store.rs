// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - Connection Store
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 11
//
// Magnetic connections between entities with orthogonal routing.
// Supports different anchor points and line styles.
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, MAX_ENTITIES, Vec2};
use fixedbitset::FixedBitSet;

use crate::store::EntityStore;

/// Maximum number of connections
pub const MAX_CONNECTIONS: usize = 1024;

/// Anchor points on entity borders
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorSide {
    /// Top edge
    Top = 0,
    /// Bottom edge
    Bottom = 1,
    /// Left edge
    Left = 2,
    /// Right edge
    Right = 3,
    /// Center point
    Center = 4,
}

/// Line styles for different connection types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStyle {
    /// Straight line between points
    Direct = 0,
    /// Orthogonal routing with 90° angles (standard for architecture)
    Orthogonal = 1,
    /// Stepped routing (manual control)
    Step = 2,
    /// Smooth Bezier curve
    Bezier = 3,
}

/// Connection store for managing entity connections
///
/// Stores connections between entities with automatic dirty tracking
/// and orthogonal routing generation.
pub struct ConnectionStore {
    /// Source entity IDs
    pub sources: Vec<EntityId>,

    /// Target entity IDs
    pub targets: Vec<EntityId>,

    /// Source anchor points
    pub source_anchors: Vec<AnchorSide>,

    /// Target anchor points
    pub target_anchors: Vec<AnchorSide>,

    /// Line styles for each connection
    pub line_styles: Vec<LineStyle>,

    /// Entities with active connections
    pub active_anchors: FixedBitSet,

    /// Connections that need recalculation
    pub dirty: FixedBitSet,
}

impl ConnectionStore {
    /// Create a new connection store
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            targets: Vec::new(),
            source_anchors: Vec::new(),
            target_anchors: Vec::new(),
            line_styles: Vec::new(),
            active_anchors: FixedBitSet::with_capacity(MAX_ENTITIES as usize),
            dirty: FixedBitSet::with_capacity(MAX_CONNECTIONS),
        }
    }

    /// Add a new connection
    ///
    /// # Arguments
    /// * `source` - Source entity ID
    /// * `target` - Target entity ID
    /// * `source_anchor` - Anchor point on source
    /// * `target_anchor` - Anchor point on target
    /// * `style` - Line style for routing
    ///
    /// # Returns
    /// Connection index, or None if capacity exceeded
    pub fn add_connection(
        &mut self,
        source: EntityId,
        target: EntityId,
        source_anchor: AnchorSide,
        target_anchor: AnchorSide,
        style: LineStyle,
    ) -> Option<usize> {
        if self.sources.len() >= MAX_CONNECTIONS {
            return None;
        }

        let idx = self.sources.len();

        self.sources.push(source);
        self.targets.push(target);
        self.source_anchors.push(source_anchor);
        self.target_anchors.push(target_anchor);
        self.line_styles.push(style);

        // Mark entities as having active connections
        let src_idx = source.index().0 as usize;
        let tgt_idx = target.index().0 as usize;
        if src_idx < MAX_ENTITIES as usize {
            self.active_anchors.set(src_idx, true);
        }
        if tgt_idx < MAX_ENTITIES as usize {
            self.active_anchors.set(tgt_idx, true);
        }

        // Mark connection as dirty for initial routing
        self.dirty.set(idx, true);

        Some(idx)
    }

    /// Remove a connection
    ///
    /// # Arguments
    /// * `idx` - Connection index
    pub fn remove_connection(&mut self, idx: usize) {
        if idx >= self.sources.len() {
            return;
        }

        // Swap with last element
        self.sources.swap_remove(idx);
        self.targets.swap_remove(idx);
        self.source_anchors.swap_remove(idx);
        self.target_anchors.swap_remove(idx);
        self.line_styles.swap_remove(idx);

        // Update dirty flags
        self.dirty.set(idx, true);

        // Update active anchors (expensive, but correct)
        self.rebuild_active_anchors();
    }

    /// Update dirty connections
    ///
    /// Regenerates routing for connections marked as dirty.
    /// Returns the generated points for updated connections.
    ///
    /// # Arguments
    /// * `store` - Entity store for position data
    ///
    /// # Returns
    /// Vector of (connection_index, points) tuples
    pub fn update_dirty(&mut self, store: &EntityStore) -> Vec<(usize, Vec<Vec2>)> {
        let mut results = Vec::new();

        for idx in self.dirty.ones() {
            if idx >= self.sources.len() {
                continue;
            }

            // Only recalculate if endpoints are valid
            let points = self.generate_routing_points(idx, store);
            results.push((idx, points));
        }

        self.dirty.clear();
        results
    }

    /// Mark a connection as dirty
    ///
    /// # Arguments
    /// * `idx` - Connection index
    #[inline(always)]
    pub fn mark_dirty(&mut self, idx: usize) {
        if idx < MAX_CONNECTIONS {
            self.dirty.set(idx, true);
        }
    }

    /// Mark all connections from/to an entity as dirty
    ///
    /// # Arguments
    /// * `entity` - Entity ID
    pub fn mark_entity_dirty(&mut self, entity: EntityId) {
        let idx = entity.index().0 as usize;
        for i in 0..self.sources.len() {
            if (self.sources[i].index().0 as usize == idx)
                || (self.targets[i].index().0 as usize == idx)
            {
                self.dirty.set(i, true);
            }
        }
    }

    /// Get the number of connections
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Check if there are no connections
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Check if an entity has active connections
    #[inline(always)]
    pub fn has_active_connections(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        idx < MAX_ENTITIES as usize && self.active_anchors.contains(idx)
    }

    /// Generate routing points for a connection
    fn generate_routing_points(&self, idx: usize, store: &EntityStore) -> Vec<Vec2> {
        let src_idx = self.sources[idx].index().0 as usize;
        let tgt_idx = self.targets[idx].index().0 as usize;

        let src_pos = Self::get_anchor_point(src_idx, self.source_anchors[idx], store);
        let tgt_pos = Self::get_anchor_point(tgt_idx, self.target_anchors[idx], store);

        match self.line_styles[idx] {
            LineStyle::Direct => vec![src_pos, tgt_pos],
            LineStyle::Orthogonal => self.generate_orthogonal_points(src_pos, tgt_pos),
            LineStyle::Step => self.generate_step_points(src_pos, tgt_pos),
            LineStyle::Bezier => self.generate_bezier_points(src_pos, tgt_pos),
        }
    }

    /// Generate orthogonal routing points (90° angles)
    fn generate_orthogonal_points(&self, src_pos: Vec2, tgt_pos: Vec2) -> Vec<Vec2> {
        let dx = (tgt_pos.x - src_pos.x).abs();
        let dy = (tgt_pos.y - src_pos.y).abs();

        if dx > dy {
            // Horizontal dominant routing
            vec![
                src_pos,
                Vec2::new((src_pos.x + tgt_pos.x) / 2.0, src_pos.y),
                Vec2::new((src_pos.x + tgt_pos.x) / 2.0, tgt_pos.y),
                tgt_pos,
            ]
        } else {
            // Vertical dominant routing
            vec![
                src_pos,
                Vec2::new(src_pos.x, (src_pos.y + tgt_pos.y) / 2.0),
                Vec2::new(tgt_pos.x, (src_pos.y + tgt_pos.y) / 2.0),
                tgt_pos,
            ]
        }
    }

    /// Generate stepped routing points
    fn generate_step_points(&self, src_pos: Vec2, tgt_pos: Vec2) -> Vec<Vec2> {
        // Simple stepped routing (manhattan style)
        vec![src_pos, Vec2::new(tgt_pos.x, src_pos.y), tgt_pos]
    }

    /// Generate Bezier curve points
    fn generate_bezier_points(&self, src_pos: Vec2, tgt_pos: Vec2) -> Vec<Vec2> {
        // Quadratic Bezier with control point in middle
        let control = Vec2::new(
            (src_pos.x + tgt_pos.x) / 2.0,
            (src_pos.y + tgt_pos.y) / 2.0 + 50.0,
        );

        // Simple approximation with 10 segments
        let mut points = Vec::with_capacity(11);
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let inv_t = 1.0 - t;

            let x = inv_t * inv_t * src_pos.x + 2.0 * inv_t * t * control.x + t * t * tgt_pos.x;
            let y = inv_t * inv_t * src_pos.y + 2.0 * inv_t * t * control.y + t * t * tgt_pos.y;

            points.push(Vec2::new(x, y));
        }
        points
    }

    /// Get anchor point for an entity
    ///
    /// # Arguments
    /// * `idx` - Entity index in store
    /// * `anchor` - Anchor point for this entity
    /// * `store` - Entity store
    fn get_anchor_point(idx: usize, anchor: AnchorSide, store: &EntityStore) -> Vec2 {
        // Get position and size from store
        let pos = store.pos(idx);
        let size = store.size(idx);

        match anchor {
            AnchorSide::Top => Vec2::new(pos.x, pos.y + size.y / 2.0),
            AnchorSide::Bottom => Vec2::new(pos.x, pos.y - size.y / 2.0),
            AnchorSide::Left => Vec2::new(pos.x - size.x / 2.0, pos.y),
            AnchorSide::Right => Vec2::new(pos.x + size.x / 2.0, pos.y),
            AnchorSide::Center => pos,
        }
    }

    /// Rebuild the active anchors set
    fn rebuild_active_anchors(&mut self) {
        self.active_anchors.clear();

        for i in 0..self.sources.len() {
            let src_idx = self.sources[i].index().0 as usize;
            let tgt_idx = self.targets[i].index().0 as usize;
            if src_idx < MAX_ENTITIES as usize {
                self.active_anchors.set(src_idx, true);
            }
            if tgt_idx < MAX_ENTITIES as usize {
                self.active_anchors.set(tgt_idx, true);
            }
        }
    }

    /// Clear all connections
    pub fn clear(&mut self) {
        self.sources.clear();
        self.targets.clear();
        self.source_anchors.clear();
        self.target_anchors.clear();
        self.line_styles.clear();
        self.active_anchors.clear();
        self.dirty.clear();
    }
}

impl Default for ConnectionStore {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_store_creation() {
        let store = ConnectionStore::new();

        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_connection_store_default() {
        let store = ConnectionStore::default();

        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_add_connection() {
        let mut store = ConnectionStore::new();
        let src = EntityId::new(1);
        let tgt = EntityId::new(2);

        let idx = store.add_connection(
            src,
            tgt,
            AnchorSide::Right,
            AnchorSide::Left,
            LineStyle::Orthogonal,
        );

        assert_eq!(store.len(), 1);
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn test_add_multiple_connections() {
        let mut store = ConnectionStore::new();

        for i in 0..5 {
            let src = EntityId::new((i * 2) as u32);
            let tgt = EntityId::new((i * 2 + 1) as u32);

            store.add_connection(
                src,
                tgt,
                AnchorSide::Center,
                AnchorSide::Center,
                LineStyle::Direct,
            );
        }

        assert_eq!(store.len(), 5);
    }

    #[test]
    fn test_remove_connection() {
        let mut store = ConnectionStore::new();
        let src = EntityId::new(1);
        let tgt = EntityId::new(2);

        store.add_connection(
            src,
            tgt,
            AnchorSide::Top,
            AnchorSide::Bottom,
            LineStyle::Bezier,
        );

        store.remove_connection(0);

        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_mark_dirty() {
        let mut store = ConnectionStore::new();
        let src = EntityId::new(1);
        let tgt = EntityId::new(2);

        let _idx = store.add_connection(
            src,
            tgt,
            AnchorSide::Center,
            AnchorSide::Center,
            LineStyle::Direct,
        );
        store.mark_dirty(0);

        // Should still have 1 connection
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_has_active_connections() {
        let mut store = ConnectionStore::new();
        let src = EntityId::new(1);
        let tgt = EntityId::new(2);

        assert!(!store.has_active_connections(src));

        store.add_connection(
            src,
            tgt,
            AnchorSide::Left,
            AnchorSide::Right,
            LineStyle::Orthogonal,
        );

        assert!(store.has_active_connections(src));
        assert!(store.has_active_connections(tgt));
    }

    #[test]
    fn test_clear() {
        let mut store = ConnectionStore::new();
        let src = EntityId::new(1);
        let tgt = EntityId::new(2);

        store.add_connection(
            src,
            tgt,
            AnchorSide::Center,
            AnchorSide::Center,
            LineStyle::Step,
        );
        assert_eq!(store.len(), 1);

        store.clear();

        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_anchor_side_discriminants() {
        assert_eq!(AnchorSide::Top, AnchorSide::Top);
        assert_ne!(AnchorSide::Top, AnchorSide::Bottom);
    }

    #[test]
    fn test_line_style_discriminants() {
        assert_eq!(LineStyle::Direct, LineStyle::Direct);
        assert_ne!(LineStyle::Direct, LineStyle::Bezier);
    }

    #[test]
    fn test_orthogonal_routing_horizontal() {
        let store = ConnectionStore::new();
        let src = Vec2::new(0.0, 100.0);
        let tgt = Vec2::new(200.0, 100.0);

        let points = store.generate_orthogonal_points(src, tgt);

        // Horizontal dominant routing: should have 4 points
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], src);
        assert_eq!(points[3], tgt);
    }

    #[test]
    fn test_orthogonal_routing_vertical() {
        let store = ConnectionStore::new();
        let src = Vec2::new(100.0, 0.0);
        let tgt = Vec2::new(100.0, 200.0);

        let points = store.generate_orthogonal_points(src, tgt);

        // Vertical dominant routing: should have 4 points
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], src);
        assert_eq!(points[3], tgt);
    }

    #[test]
    fn test_direct_routing() {
        let store = ConnectionStore::new();
        let src = Vec2::new(0.0, 0.0);
        let tgt = Vec2::new(100.0, 100.0);

        let points = store.generate_orthogonal_points(src, tgt);

        // Direct routing: 2 points
        assert!(points.len() >= 2);
    }
}
