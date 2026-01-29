//! Layer and C4 model support
//!
//! This module provides the layer system for organizing shapes and
//! supporting the C4 model (Context, Container, Component, Code).

use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// C4 model level for organizing architecture diagrams
///
/// The C4 model defines four levels of abstraction:
/// - Context: Shows the system in context of users and other systems
/// - Container: Shows the high-level technical building blocks
/// - Component: Shows the internal structure of a container
/// - Code: Shows the implementation details (classes, interfaces, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum C4Level {
    /// Level 1: Context - System context diagram
    Context,
    /// Level 2: Container - Container diagram
    Container,
    /// Level 3: Component - Component diagram
    Component,
    /// Level 4: Code - Class/code diagram
    Code,
}

impl Default for C4Level {
    fn default() -> Self {
        Self::Context
    }
}

impl C4Level {
    /// Gets the display name for the level
    #[inline]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Context => "Context",
            Self::Container => "Container",
            Self::Component => "Component",
            Self::Code => "Code",
        }
    }

    /// Gets the description for the level
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Context => "System context showing users and external systems",
            Self::Container => "Container diagram showing applications and databases",
            Self::Component => "Component diagram showing internal structure",
            Self::Code => "Class and code-level details",
        }
    }

    /// Gets the default zoom level for this C4 level
    #[inline]
    pub fn default_zoom(&self) -> f32 {
        match self {
            Self::Context => 0.15,
            Self::Container => 0.5,
            Self::Component => 1.0,
            Self::Code => 2.0,
        }
    }

    /// Converts to a string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Context => "context",
            Self::Container => "container",
            Self::Component => "component",
            Self::Code => "code",
        }
    }

    /// Parses from a string
    #[inline]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "context" => Some(Self::Context),
            "container" => Some(Self::Container),
            "component" => Some(Self::Component),
            "code" => Some(Self::Code),
            _ => None,
        }
    }
}

/// A layer containing shapes
///
/// Layers are used to organize shapes and control their visibility,
/// opacity, and ordering.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Unique layer ID
    pub id: EntityId,
    /// C4 level this layer belongs to
    pub c4_level: C4Level,
    /// Layer name
    pub name: String,
    /// Whether the layer is visible
    pub visible: bool,
    /// Whether the layer is locked (shapes cannot be edited)
    pub locked: bool,
    /// Layer opacity (0.0 to 1.0)
    pub opacity: f32,
    /// Shapes in this layer
    pub shapes: Vec<EntityId>,
    /// Z-order for layer stacking
    pub z_order: i32,
}

impl Layer {
    /// Creates a new layer
    #[inline]
    pub fn new(id: EntityId, c4_level: C4Level, name: String) -> Self {
        Self {
            id,
            c4_level,
            name,
            visible: true,
            locked: false,
            opacity: 1.0,
            shapes: Vec::new(),
            z_order: 0,
        }
    }

    /// Adds a shape to the layer
    #[inline]
    pub fn add_shape(&mut self, shape_id: EntityId) {
        if !self.shapes.contains(&shape_id) {
            self.shapes.push(shape_id);
        }
    }

    /// Removes a shape from the layer
    #[inline]
    pub fn remove_shape(&mut self, shape_id: EntityId) {
        self.shapes.retain(|&id| id != shape_id);
    }

    /// Checks if the layer contains a shape
    #[inline]
    pub fn contains_shape(&self, shape_id: EntityId) -> bool {
        self.shapes.contains(&shape_id)
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new(EntityId::new(), C4Level::Context, "Default".to_string())
    }
}

/// Manages layers for a document
///
/// The layer manager provides operations for creating, deleting,
/// and modifying layers, as well as querying shapes by layer.
#[derive(Debug, Default)]
pub struct LayerManager {
    /// All layers indexed by ID
    layers: HashMap<EntityId, Layer>,
    /// Current C4 level
    current_level: C4Level,
    /// Layer creation order for z-ordering
    layer_order: Vec<EntityId>,
}

impl LayerManager {
    /// Creates a new layer manager
    #[inline]
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            current_level: C4Level::default(),
            layer_order: Vec::new(),
        }
    }

    /// Creates a new layer
    ///
    /// # Returns
    ///
    /// The ID of the created layer
    #[inline]
    pub fn create_layer(&mut self, c4_level: C4Level, name: String) -> EntityId {
        let id = EntityId::new();
        let z_order = self.layer_order.len() as i32;
        let mut layer = Layer::new(id, c4_level, name);
        layer.z_order = z_order;
        self.layers.insert(id, layer);
        self.layer_order.push(id);
        id
    }

    /// Gets a layer by ID
    #[inline]
    pub fn get_layer(&self, id: EntityId) -> Option<&Layer> {
        self.layers.get(&id)
    }

    /// Gets a mutable layer by ID
    #[inline]
    pub fn get_layer_mut(&mut self, id: EntityId) -> Option<&mut Layer> {
        self.layers.get_mut(&id)
    }

    /// Sets layer visibility
    #[inline]
    pub fn set_layer_visibility(&mut self, id: EntityId, visible: bool) -> bool {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.visible = visible;
            true
        } else {
            false
        }
    }

    /// Sets layer locked state
    #[inline]
    pub fn set_layer_locked(&mut self, id: EntityId, locked: bool) -> bool {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.locked = locked;
            true
        } else {
            false
        }
    }

    /// Sets layer opacity
    #[inline]
    pub fn set_layer_opacity(&mut self, id: EntityId, opacity: f32) -> bool {
        let opacity = opacity.clamp(0.0, 1.0);
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.opacity = opacity;
            true
        } else {
            false
        }
    }

    /// Deletes a layer
    ///
    /// Shapes in the layer are not deleted, just removed from the layer.
    #[inline]
    pub fn delete_layer(&mut self, id: EntityId) -> bool {
        if let Some(layer) = self.layers.remove(&id) {
            self.layer_order.retain(|&layer_id| layer_id != id);
            true
        } else {
            false
        }
    }

    /// Gets all layers for a specific C4 level
    #[inline]
    pub fn get_layers_for_level(&self, level: C4Level) -> Vec<&Layer> {
        self.layers
            .values()
            .filter(|layer| layer.c4_level == level)
            .collect()
    }

    /// Gets all visible layers
    #[inline]
    pub fn visible_layers(&self) -> Vec<&Layer> {
        self.layers.values().filter(|layer| layer.visible).collect()
    }

    /// Gets all layers
    #[inline]
    pub fn all_layers(&self) -> Vec<&Layer> {
        self.layers.values().collect()
    }

    /// Gets the current C4 level
    #[inline]
    pub fn current_level(&self) -> C4Level {
        self.current_level
    }

    /// Sets the current C4 level
    #[inline]
    pub fn set_current_level(&mut self, level: C4Level) {
        self.current_level = level;
    }

    /// Gets shapes for the current C4 level
    #[inline]
    pub fn get_shapes_for_current_level(&self) -> Vec<&EntityId> {
        self.get_layers_for_level(self.current_level)
            .iter()
            .flat_map(|layer| layer.shapes.iter())
            .collect()
    }

    /// Gets the zoom level for the current C4 level
    #[inline]
    pub fn get_zoom_for_current_level(&self) -> f32 {
        self.current_level.default_zoom()
    }

    /// Adds a shape to a layer
    #[inline]
    pub fn add_shape_to_layer(&mut self, layer_id: EntityId, shape_id: EntityId) -> bool {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.add_shape(shape_id);
            true
        } else {
            false
        }
    }

    /// Removes a shape from a layer
    #[inline]
    pub fn remove_shape_from_layer(&mut self, layer_id: EntityId, shape_id: EntityId) -> bool {
        if let Some(layer) = self.layers.get_mut(&layer_id) {
            layer.remove_shape(shape_id);
            true
        } else {
            false
        }
    }

    /// Gets the number of layers
    #[inline]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Moves a layer up in z-order
    #[inline]
    pub fn move_layer_up(&mut self, id: EntityId) -> bool {
        if let Some(pos) = self.layer_order.iter().position(|&layer_id| layer_id == id) {
            if pos < self.layer_order.len() - 1 {
                self.layer_order.swap(pos, pos + 1);
                // Update z-orders
                for (idx, &layer_id) in self.layer_order.iter().enumerate() {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.z_order = idx as i32;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Moves a layer down in z-order
    #[inline]
    pub fn move_layer_down(&mut self, id: EntityId) -> bool {
        if let Some(pos) = self.layer_order.iter().position(|&layer_id| layer_id == id) {
            if pos > 0 {
                self.layer_order.swap(pos, pos - 1);
                // Update z-orders
                for (idx, &layer_id) in self.layer_order.iter().enumerate() {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.z_order = idx as i32;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Moves a layer to the top (front)
    #[inline]
    pub fn move_layer_to_top(&mut self, id: EntityId) -> bool {
        if let Some(pos) = self.layer_order.iter().position(|&layer_id| layer_id == id) {
            if pos < self.layer_order.len() - 1 {
                let layer_id = self.layer_order.remove(pos);
                self.layer_order.push(layer_id);
                // Update z-orders
                for (idx, &layer_id) in self.layer_order.iter().enumerate() {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.z_order = idx as i32;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Moves a layer to the bottom (back)
    #[inline]
    pub fn move_layer_to_bottom(&mut self, id: EntityId) -> bool {
        if let Some(pos) = self.layer_order.iter().position(|&layer_id| layer_id == id) {
            if pos > 0 {
                let layer_id = self.layer_order.remove(pos);
                self.layer_order.insert(0, layer_id);
                // Update z-orders
                for (idx, &layer_id) in self.layer_order.iter().enumerate() {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.z_order = idx as i32;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Gets layers in z-order
    #[inline]
    pub fn get_layers_in_order(&self) -> Vec<&Layer> {
        let mut layers: Vec<&Layer> = self.layers.values().collect();
        layers.sort_by_key(|layer| layer.z_order);
        layers
    }

    /// Renames a layer
    #[inline]
    pub fn rename_layer(&mut self, id: EntityId, name: String) -> bool {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.name = name;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c4_level_defaults() {
        let level = C4Level::default();
        assert_eq!(level, C4Level::Context);
        assert_eq!(level.display_name(), "Context");
    }

    #[test]
    fn test_c4_level_zoom() {
        assert!((C4Level::Context.default_zoom() - 0.15).abs() < 0.01);
        assert!((C4Level::Container.default_zoom() - 0.5).abs() < 0.01);
        assert!((C4Level::Component.default_zoom() - 1.0).abs() < 0.01);
        assert!((C4Level::Code.default_zoom() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_c4_level_string_conversion() {
        assert_eq!(C4Level::Context.as_str(), "context");
        assert_eq!(C4Level::Container.as_str(), "container");
        assert_eq!(C4Level::Component.as_str(), "component");
        assert_eq!(C4Level::Code.as_str(), "code");

        assert_eq!(C4Level::from_str("context"), Some(C4Level::Context));
        assert_eq!(C4Level::from_str("invalid"), None);
    }

    #[test]
    fn test_layer_creation() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer".to_string());

        let layer = manager.get_layer(id);
        assert!(layer.is_some());
        assert_eq!(layer.unwrap().name, "Test Layer");
    }

    #[test]
    fn test_layer_visibility() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer".to_string());

        assert!(manager.set_layer_visibility(id, false));
        assert!(!manager.get_layer(id).unwrap().visible);

        assert!(manager.set_layer_visibility(id, true));
        assert!(manager.get_layer(id).unwrap().visible);
    }

    #[test]
    fn test_layer_opacity() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer".to_string());

        assert!(manager.set_layer_opacity(id, 0.5));
        assert!((manager.get_layer(id).unwrap().opacity - 0.5).abs() < 0.01);

        // Opacity should be clamped
        assert!(manager.set_layer_opacity(id, 1.5));
        assert!((manager.get_layer(id).unwrap().opacity - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_layers_for_level() {
        let mut manager = LayerManager::new();
        manager.create_layer(C4Level::Context, "Context Layer 1".to_string());
        manager.create_layer(C4Level::Context, "Context Layer 2".to_string());
        manager.create_layer(C4Level::Container, "Container Layer".to_string());

        let context_layers = manager.get_layers_for_level(C4Level::Context);
        assert_eq!(context_layers.len(), 2);

        let container_layers = manager.get_layers_for_level(C4Level::Container);
        assert_eq!(container_layers.len(), 1);
    }

    #[test]
    fn test_delete_layer() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer".to_string());
        assert_eq!(manager.layer_count(), 1);

        assert!(manager.delete_layer(id));
        assert_eq!(manager.layer_count(), 0);

        assert!(!manager.delete_layer(id)); // Delete again should fail
    }

    #[test]
    fn test_move_layer_up() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1".to_string());
        let id2 = manager.create_layer(C4Level::Context, "Layer 2".to_string());

        assert!(manager.move_layer_up(id1));
        let layers = manager.get_layers_in_order();
        assert_eq!(layers[0].id, id2);
        assert_eq!(layers[1].id, id1);
    }

    #[test]
    fn test_move_layer_down() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1".to_string());
        let id2 = manager.create_layer(C4Level::Context, "Layer 2".to_string());

        assert!(manager.move_layer_down(id2));
        let layers = manager.get_layers_in_order();
        assert_eq!(layers[0].id, id2);
        assert_eq!(layers[1].id, id1);
    }

    #[test]
    fn test_move_layer_to_top() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1".to_string());
        let id2 = manager.create_layer(C4Level::Context, "Layer 2".to_string());
        let id3 = manager.create_layer(C4Level::Context, "Layer 3".to_string());

        assert!(manager.move_layer_to_top(id1));
        let layers = manager.get_layers_in_order();
        assert_eq!(layers[0].id, id2);
        assert_eq!(layers[1].id, id3);
        assert_eq!(layers[2].id, id1);
    }

    #[test]
    fn test_move_layer_to_bottom() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1".to_string());
        let id2 = manager.create_layer(C4Level::Context, "Layer 2".to_string());
        let id3 = manager.create_layer(C4Level::Context, "Layer 3".to_string());

        assert!(manager.move_layer_to_bottom(id3));
        let layers = manager.get_layers_in_order();
        assert_eq!(layers[0].id, id3);
        assert_eq!(layers[1].id, id1);
        assert_eq!(layers[2].id, id2);
    }

    #[test]
    fn test_get_layers_in_order() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1".to_string());
        let id2 = manager.create_layer(C4Level::Context, "Layer 2".to_string());
        let id3 = manager.create_layer(C4Level::Context, "Layer 3".to_string());

        let layers = manager.get_layers_in_order();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0].id, id1);
        assert_eq!(layers[1].id, id2);
        assert_eq!(layers[2].id, id3);
    }

    #[test]
    fn test_rename_layer() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Original".to_string());

        assert!(manager.rename_layer(id, "Renamed".to_string()));
        assert_eq!(manager.get_layer(id).unwrap().name, "Renamed");

        assert!(!manager.rename_layer(EntityId::new(), "Should Fail".to_string()));
    }
}
