// ═══════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Declarative JSON API for ECS
//
// This module provides pure data structures for declarative entity/component definition
// compatible with A-Frame style JSON scene definitions.
//
// Key Features:
// - ComponentDefinition: Component type + configuration
// - BehaviorDefinition: Collection of components for entity templates
// - EntityDefinition: Complete entity with components, behaviors, children
// - Scene: Complete scene with entities, behaviors, metadata
//
// Usage:
// 1. Parse JSON to Scene/EntityDefinition using serde
// 2. Use application-specific factories to create actual components
// 3. Add components to World using your game's type system
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════
#![cfg(feature = "std")]

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// ComponentDefinition
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Defines a component in JSON format
///
/// A-Frame style: `{"type": "Position", "data": {"x": 10, "y": 20}}`
///
/// This is PURE DATA - no component creation logic.
/// Use with your application's factory to create actual components.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComponentDefinition {
    /// Component type identifier (e.g., "Position", "Velocity", "Health")
    #[serde(rename = "type")]
    pub component_type: String,

    /// Component configuration data (flexible JSON structure)
    pub data: serde_json::Value,
}

impl ComponentDefinition {
    /// Creates a new ComponentDefinition
    #[inline]
    #[must_use]
    pub const fn new(component_type: String, data: serde_json::Value) -> Self {
        Self {
            component_type,
            data,
        }
    }

    /// Returns the component type
    #[inline]
    #[must_use]
    pub fn component_type(&self) -> &str {
        &self.component_type
    }

    /// Returns a reference to the component data
    #[inline]
    #[must_use]
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }

    /// Extracts a field from data as f32
    #[must_use]
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.data
            .get(key)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    /// Extracts a field from data as i32
    #[must_use]
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.data
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
    }

    /// Extracts a field from data as bool
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    /// Extracts a string field from data
    #[must_use]
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// BehaviorDefinition
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Complete definition of a behavior (template for entities)
///
/// A behavior is a reusable collection of components that define
/// a particular entity behavior pattern.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BehaviorDefinition {
    /// Unique identifier for this behavior definition
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Components that make up this behavior
    pub components: Vec<ComponentDefinition>,
}

impl BehaviorDefinition {
    /// Creates a new BehaviorDefinition
    #[inline]
    #[must_use]
    pub const fn new(
        id: String,
        name: String,
        description: Option<String>,
        components: Vec<ComponentDefinition>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            components,
        }
    }

    /// Returns the number of components
    #[inline]
    #[must_use]
    pub const fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Checks if this behavior has a component of the given type
    #[must_use]
    pub fn has_component(&self, component_type: &str) -> bool {
        self.components
            .iter()
            .any(|c| c.component_type == component_type)
    }

    /// Finds a component by type
    #[must_use]
    pub fn get_component(&self, component_type: &str) -> Option<&ComponentDefinition> {
        self.components
            .iter()
            .find(|c| c.component_type == component_type)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// EntityDefinition
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Complete definition of an entity for scene loading
///
/// An entity definition contains all components and references to behaviors
/// needed to recreate an entity. Child entities support hierarchical scenes.
///
/// A-Frame style JSON:
/// ```json
/// {
///   "id": "player",
///   "components": [
///     {"type": "Position", "data": {"x": 0, "y": 0}},
///     {"type": "Velocity", "data": {"dx": 0, "dy": 0}}
///   ],
///   "behaviors": ["player_behavior"],
///   "children": [...]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct EntityDefinition {
    /// Entity identifier
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,

    /// Human-readable name for debugging
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Components directly attached to this entity
    #[serde(default)]
    pub components: Vec<ComponentDefinition>,

    /// Behavior IDs to apply
    #[serde(default)]
    pub behaviors: Vec<String>,

    /// Child entities (hierarchical scenes)
    #[serde(default)]
    pub children: Vec<EntityDefinition>,
}

impl EntityDefinition {
    /// Creates a new EntityDefinition
    #[inline]
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    /// Builder: adds a component
    #[inline]
    pub fn with_component(mut self, component: ComponentDefinition) -> Self {
        self.components.push(component);
        self
    }

    /// Builder: adds a behavior reference
    #[inline]
    pub fn with_behavior(mut self, behavior_id: String) -> Self {
        self.behaviors.push(behavior_id);
        self
    }

    /// Builder: adds a child entity
    #[inline]
    pub fn with_child(mut self, child: EntityDefinition) -> Self {
        self.children.push(child);
        self
    }

    /// Builder: sets the name
    #[inline]
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Total component count including children
    #[must_use]
    pub fn total_component_count(&self) -> usize {
        let mut count = self.components.len();
        for child in &self.children {
            count += child.total_component_count();
        }
        count
    }

    /// Total entity count including children
    #[must_use]
    pub fn total_entity_count(&self) -> usize {
        let mut count = 1;
        for child in &self.children {
            count += child.total_entity_count();
        }
        count
    }

    /// Finds a component by type
    #[must_use]
    pub fn get_component(&self, component_type: &str) -> Option<&ComponentDefinition> {
        self.components
            .iter()
            .find(|c| c.component_type == component_type)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// SceneMetadata
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Scene metadata for configuration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct SceneMetadata {
    /// Scene author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Scene description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default gravity (m/s²), defaults to Earth gravity
    #[serde(default)]
    pub gravity: [f32; 3],

    /// Ambient light color (RGB, 0.0-1.0)
    #[serde(default)]
    pub ambient_light: [f32; 3],

    /// Fog settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog: Option<FogSettings>,
}

impl Default for SceneMetadata {
    fn default() -> Self {
        Self {
            author: None,
            description: None,
            gravity: [0.0, -9.81, 0.0],
            ambient_light: [0.3, 0.3, 0.3],
            fog: None,
        }
    }
}

/// Fog effect settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FogSettings {
    /// Fog type: "linear" or "exponential"
    #[serde(default = "default_fog_type")]
    pub fog_type: String,

    /// Fog color (RGB)
    pub color: [f32; 3],

    /// Linear fog: start distance
    #[serde(default = "default_fog_start")]
    pub start: f32,

    /// Linear fog: end distance
    #[serde(default = "default_fog_end")]
    pub end: f32,

    /// Exponential fog: density
    #[serde(default = "default_fog_density")]
    pub density: f32,
}

fn default_fog_type() -> String {
    "exponential".to_string()
}
fn default_fog_start() -> f32 {
    1.0
}
fn default_fog_end() -> f32 {
    100.0
}
fn default_fog_density() -> f32 {
    0.01
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Scene
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Complete definition of a scene
///
/// The root structure for A-Frame compatible scene definitions.
/// Contains all entities, inline behaviors, and metadata needed to
/// recreate a game scene.
///
/// A-Frame style JSON:
/// ```json
/// {
///   "id": "level_1",
///   "name": "First Level",
///   "metadata": { "gravity": [0, -9.81, 0] },
///   "entities": [...],
///   "behaviors": [...]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Scene {
    /// Unique scene identifier
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,

    /// Human-readable name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Scene version
    #[serde(default)]
    pub version: String,

    /// Scene metadata
    #[serde(default)]
    pub metadata: SceneMetadata,

    /// Top-level entities
    #[serde(default)]
    pub entities: Vec<EntityDefinition>,

    /// Inline behavior definitions
    #[serde(default)]
    pub behaviors: Vec<BehaviorDefinition>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: None,
            version: "1.0.0".to_string(),
            metadata: SceneMetadata::default(),
            entities: Vec::new(),
            behaviors: Vec::new(),
        }
    }
}

impl Scene {
    /// Creates a new Scene
    #[inline]
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    /// Builder: adds an entity
    #[inline]
    pub fn with_entity(mut self, entity: EntityDefinition) -> Self {
        self.entities.push(entity);
        self
    }

    /// Builder: adds a behavior
    #[inline]
    pub fn with_behavior(mut self, behavior: BehaviorDefinition) -> Self {
        self.behaviors.push(behavior);
        self
    }

    /// Builder: sets the name
    #[inline]
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Total entity count including nested children
    #[must_use]
    pub fn total_entity_count(&self) -> usize {
        let mut count = 0;
        for entity in &self.entities {
            count += entity.total_entity_count();
        }
        count
    }

    /// Finds a behavior by ID
    #[must_use]
    pub fn get_behavior(&self, behavior_id: &str) -> Option<&BehaviorDefinition> {
        self.behaviors.iter().find(|b| b.id == behavior_id)
    }

    /// Finds an entity by ID (shallow search)
    #[must_use]
    pub fn get_entity(&self, entity_id: &str) -> Option<&EntityDefinition> {
        self.entities.iter().find(|e| e.id == entity_id)
    }

    /// Finds an entity recursively by ID
    #[must_use]
    pub fn find_entity(&self, entity_id: &str) -> Option<&EntityDefinition> {
        if let Some(e) = self.get_entity(entity_id) {
            return Some(e);
        }
        for entity in &self.entities {
            if let Some(e) = entity.children.iter().find(|e| e.id == entity_id) {
                return Some(e);
            }
            if let Some(e) = self.find_entity_in_children(entity, entity_id) {
                return Some(e);
            }
        }
        None
    }

    fn find_entity_in_children<'a>(
        &'a self,
        entity: &'a EntityDefinition,
        entity_id: &str,
    ) -> Option<&'a EntityDefinition> {
        if let Some(e) = entity.children.iter().find(|e| e.id == entity_id) {
            return Some(e);
        }
        for child in &entity.children {
            if let Some(e) = self.find_entity_in_children(child, entity_id) {
                return Some(e);
            }
        }
        None
    }

    /// Parses a Scene from JSON string
    ///
    /// # Errors
    /// Returns `serde_json::Error` if the JSON is invalid
    #[inline]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serializes this Scene to a JSON string
    ///
    /// # Errors
    /// Returns `serde_json::Error` if serialization fails
    #[inline]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Builder: sets the metadata
    #[inline]
    pub fn with_metadata(mut self, metadata: SceneMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Error Types
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Errors during scene loading
///
/// These are PURE DATA errors - no I/O or system integration.
/// Use in your application's error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneLoaderError {
    /// Invalid JSON format
    InvalidJson(String),

    /// Scene validation failed
    ValidationError(String),

    /// Referenced entity not found
    EntityNotFound(String),

    /// Referenced behavior not found
    BehaviorNotFound(String),

    /// Component type not registered
    ComponentNotRegistered(String),

    /// Failed to create component from data
    ComponentCreationFailed(String),

    /// Invalid metadata
    InvalidMetadata(String),
}

impl core::fmt::Display for SceneLoaderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::EntityNotFound(id) => write!(f, "Entity '{}' not found", id),
            Self::BehaviorNotFound(id) => write!(f, "Behavior '{}' not found", id),
            Self::ComponentNotRegistered(type_name) => {
                write!(f, "Component type '{}' not registered", type_name)
            }
            Self::ComponentCreationFailed(msg) => write!(f, "Component creation failed: {}", msg),
            Self::InvalidMetadata(msg) => write!(f, "Invalid metadata: {}", msg),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// JSON Schemas (for validation)
// ═══════════════════════════════════════════════════════════════════════════════════════════════

pub const COMPONENT_DEFINITION_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "ComponentDefinition",
  "properties": {
    "type": { "type": "string" },
    "data": { "type": "object" }
  },
  "required": ["type", "data"],
  "additionalProperties": false
}"#;

pub const BEHAVIOR_DEFINITION_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "BehaviorDefinition",
  "properties": {
    "id": { "type": "string" },
    "name": { "type": "string" },
    "description": { "type": "string" },
    "components": { "type": "array" }
  },
  "required": ["id", "name", "components"],
  "additionalProperties": false
}"#;

pub const ENTITY_DEFINITION_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "EntityDefinition",
  "properties": {
    "id": { "type": "string" },
    "name": { "type": "string" },
    "components": { "type": "array" },
    "behaviors": { "type": "array" },
    "children": { "type": "array" }
  },
  "additionalProperties": false
}"#;

pub const SCENE_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "Scene",
  "properties": {
    "id": { "type": "string" },
    "name": { "type": "string" },
    "version": { "type": "string" },
    "metadata": { "type": "object" },
    "entities": { "type": "array" },
    "behaviors": { "type": "array" }
  },
  "required": ["id", "entities"],
  "additionalProperties": false
}"#;

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// SceneLoader - Integración JSON → ECS World
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Factory trait for creating components from JSON definitions.
///
/// This trait enables flexible component creation strategies:
/// - Default factory for standard component types
/// - Custom factories for application-specific components
/// - Test factories with mock components
pub trait ComponentFactory {
    /// Creates a component from a definition, or returns an error.
    ///
    /// # Errors
    /// Returns `SceneLoaderError::ComponentCreationFailed` if creation fails.
    fn create_component(
        &self,
        component_type: &str,
        data: &serde_json::Value,
    ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send + Sync>, SceneLoaderError>;

    /// Returns true if this factory can create the given component type.
    fn can_create(&self, component_type: &str) -> bool;
}

/// Default component factory with built-in physics component support.
///
/// Supports standard ECS components like Velocity, Acceleration, Transform, etc.
#[derive(Debug)]
pub struct DefaultComponentFactory;

impl DefaultComponentFactory {
    /// Creates a new DefaultComponentFactory
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ComponentFactory for DefaultComponentFactory {
    fn can_create(&self, component_type: &str) -> bool {
        matches!(
            component_type,
            "Velocity" | "Acceleration" | "Transform" | "PhysicsMaterial"
        )
    }

    fn create_component(
        &self,
        component_type: &str,
        data: &serde_json::Value,
    ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send + Sync>, SceneLoaderError> {
        match component_type {
            "Velocity" => self.create_velocity(data),
            "Acceleration" => self.create_acceleration(data),
            "Transform" => self.create_transform(data),
            "PhysicsMaterial" => self.create_physics_material(data),
            _ => Err(SceneLoaderError::ComponentCreationFailed(format!(
                "Unknown component type: '{}'",
                component_type
            ))),
        }
    }
}

impl DefaultComponentFactory {
    #[inline]
    fn extract_f32(&self, data: &serde_json::Value, key: &str) -> Result<f32, SceneLoaderError> {
        data.get(key)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .ok_or_else(|| {
                SceneLoaderError::ComponentCreationFailed(format!(
                    "Missing or invalid '{}' field",
                    key
                ))
            })
    }

    #[inline]
    fn extract_opt_f32(
        &self,
        data: &serde_json::Value,
        key: &str,
    ) -> Result<Option<f32>, SceneLoaderError> {
        Ok(data.get(key).and_then(|v| v.as_f64().map(|v| v as f32)))
    }

    #[inline]
    fn extract_bool(&self, data: &serde_json::Value, key: &str) -> Result<bool, SceneLoaderError> {
        data.get(key).and_then(|v| v.as_bool()).ok_or_else(|| {
            SceneLoaderError::ComponentCreationFailed(format!("Missing or invalid '{}' field", key))
        })
    }

    fn create_velocity(
        &self,
        data: &serde_json::Value,
    ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send + Sync>, SceneLoaderError> {
        use crate::ecs::physics_components::Velocity;
        Ok(alloc::boxed::Box::new(Velocity {
            dx: self.extract_f32(data, "dx")?,
            dy: self.extract_f32(data, "dy")?,
        }))
    }

    fn create_acceleration(
        &self,
        data: &serde_json::Value,
    ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send + Sync>, SceneLoaderError> {
        use crate::ecs::physics_components::Acceleration;
        Ok(alloc::boxed::Box::new(Acceleration {
            ax: self.extract_f32(data, "ax")?,
            ay: self.extract_f32(data, "ay")?,
        }))
    }

    fn create_transform(
        &self,
        data: &serde_json::Value,
    ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send + Sync>, SceneLoaderError> {
        use crate::ecs::physics_components::Transform;
        Ok(alloc::boxed::Box::new(Transform {
            position_x: self.extract_f32(data, "x")?,
            position_y: self.extract_f32(data, "y")?,
            rotation: self.extract_opt_f32(data, "rotation")?.unwrap_or(0.0),
            scale_x: self.extract_opt_f32(data, "scale_x")?.unwrap_or(1.0),
            scale_y: self.extract_opt_f32(data, "scale_y")?.unwrap_or(1.0),
        }))
    }

    fn create_physics_material(
        &self,
        data: &serde_json::Value,
    ) -> Result<alloc::boxed::Box<dyn core::any::Any + Send + Sync>, SceneLoaderError> {
        use crate::ecs::physics_components::PhysicsMaterial;
        Ok(alloc::boxed::Box::new(PhysicsMaterial {
            restitution: self.extract_opt_f32(data, "restitution")?.unwrap_or(0.3),
            friction: self.extract_opt_f32(data, "friction")?.unwrap_or(0.5),
            mass: self.extract_opt_f32(data, "mass")?.unwrap_or(1.0),
            is_sensor: self
                .extract_opt_f32(data, "is_sensor")?
                .map(|v| v > 0.5)
                .unwrap_or(false),
        }))
    }
}

/// Result of loading a scene.
#[derive(Debug)]
pub struct SceneLoadResult {
    /// Number of entities created
    pub entity_count: usize,
    /// Mapping from JSON entity ID to ECS EntityId
    pub entity_map: alloc::collections::BTreeMap<alloc::string::String, crate::ecs::EntityId>,
}

/// Loads a scene from JSON data into an ECS World.
///
/// This is the main entry point for scene loading. It parses the JSON,
/// validates the scene structure, and creates entities in the World.
///
/// # Examples
///
/// ```ignore
/// use archflow_logic::ecs::{World, Component};
/// use archflow_logic::api::json::{SceneLoader, DefaultComponentFactory};
///
/// // Define your component types
/// #[derive(Clone, Debug)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position { type Storage = VecStorage<Position>; }
///
/// // Load scene
/// let json = load_scene_from_file("level.json");
/// let mut world = World::new();
/// let factory = DefaultComponentFactory::new();
///
/// let loader = SceneLoader::new(factory);
/// let result = loader.load_scene(&mut world, &json).unwrap();
///
/// println!("Created {} entities", result.entity_count);
/// ```
#[derive(Debug)]
pub struct SceneLoader<F = DefaultComponentFactory>
where
    F: ComponentFactory,
{
    factory: F,
}

impl<F> SceneLoader<F>
where
    F: ComponentFactory,
{
    /// Creates a new SceneLoader with the given component factory.
    #[inline]
    #[must_use]
    pub fn new(factory: F) -> Self {
        Self { factory }
    }

    /// Loads a scene from a JSON string into the World.
    ///
    /// # Errors
    /// Returns errors for invalid JSON, missing references, or component creation failures.
    #[inline]
    pub fn load_scene(
        &self,
        world: &mut crate::ecs::World,
        json: &str,
    ) -> Result<SceneLoadResult, SceneLoaderError> {
        let scene: Scene =
            serde_json::from_str(json).map_err(|e| SceneLoaderError::InvalidJson(e.to_string()))?;

        self.load_parsed_scene(world, &scene)
    }

    /// Loads a pre-parsed Scene into the World.
    ///
    /// This is useful when you already have a Scene struct (e.g., from caching).
    #[inline]
    pub fn load_parsed_scene(
        &self,
        world: &mut crate::ecs::World,
        scene: &Scene,
    ) -> Result<SceneLoadResult, SceneLoaderError> {
        let mut entity_map = alloc::collections::BTreeMap::new();
        let mut entity_count = 0;

        // Register behaviors for quick lookup
        let behavior_map: alloc::collections::BTreeMap<&str, &BehaviorDefinition> =
            scene.behaviors.iter().map(|b| (b.id.as_str(), b)).collect();

        // Process top-level entities
        for entity_def in &scene.entities {
            let count = self.load_entity_recursive(
                world,
                entity_def,
                &behavior_map,
                &self.factory,
                &mut entity_map,
                None,
            )?;
            entity_count += count;
        }

        Ok(SceneLoadResult {
            entity_count,
            entity_map,
        })
    }

    #[inline]
    fn load_entity_recursive(
        &self,
        world: &mut crate::ecs::World,
        entity_def: &EntityDefinition,
        behavior_map: &alloc::collections::BTreeMap<&str, &BehaviorDefinition>,
        factory: &F,
        entity_map: &mut alloc::collections::BTreeMap<alloc::string::String, crate::ecs::EntityId>,
        _parent_entity: Option<crate::ecs::EntityId>,
    ) -> Result<usize, SceneLoaderError> {
        // Create entity
        let entity = world.create_entity();
        let mut count = 1;

        // Map entity ID if present
        if !entity_def.id.is_empty() {
            entity_map.insert(entity_def.id.clone(), entity);
        }

        // Add components from the entity definition
        for comp_def in &entity_def.components {
            self.add_component_to_entity(world, entity, comp_def, factory)?;
        }

        // Apply behaviors
        for behavior_id in &entity_def.behaviors {
            let behavior = behavior_map
                .get(behavior_id.as_str())
                .ok_or_else(|| SceneLoaderError::BehaviorNotFound(behavior_id.clone()))?;

            for comp_def in &behavior.components {
                self.add_component_to_entity(world, entity, comp_def, factory)?;
            }
        }

        // Process child entities
        for child_def in &entity_def.children {
            let child_count = self.load_entity_recursive(
                world,
                child_def,
                behavior_map,
                factory,
                entity_map,
                Some(entity),
            )?;
            count += child_count;
        }

        Ok(count)
    }

    #[inline]
    fn add_component_to_entity(
        &self,
        world: &mut crate::ecs::World,
        entity: crate::ecs::EntityId,
        component_def: &ComponentDefinition,
        factory: &F,
    ) -> Result<(), SceneLoaderError> {
        let component_type = component_def.component_type();

        // Use factory to create component
        let component = factory
            .create_component(component_type, component_def.data())
            .map_err(|e| {
                SceneLoaderError::ComponentCreationFailed(format!(
                    "Failed to create component '{}': {}",
                    component_type, e
                ))
            })?;

        // Add to world using type-based dispatch
        // This is a simplified version - in production you'd use a registry
        self.add_component_by_type(world, entity, component_type, component)?;

        Ok(())
    }

    #[inline]
    fn add_component_by_type(
        &self,
        world: &mut crate::ecs::World,
        entity: crate::ecs::EntityId,
        component_type: &str,
        component: alloc::boxed::Box<dyn core::any::Any + Send + Sync>,
    ) -> Result<(), SceneLoaderError> {
        // Type-based component addition
        // In a full implementation, this would use the ComponentRegistry
        match component_type {
            "Velocity" => {
                if let Ok(vel) = component.downcast::<crate::ecs::physics_components::Velocity>() {
                    world.add_component(entity, *vel);
                    Ok(())
                } else {
                    Err(SceneLoaderError::ComponentCreationFailed(
                        "Type mismatch for Velocity".to_string(),
                    ))
                }
            }
            "Acceleration" => {
                if let Ok(acc) =
                    component.downcast::<crate::ecs::physics_components::Acceleration>()
                {
                    world.add_component(entity, *acc);
                    Ok(())
                } else {
                    Err(SceneLoaderError::ComponentCreationFailed(
                        "Type mismatch for Acceleration".to_string(),
                    ))
                }
            }
            "Transform" => {
                if let Ok(transform) =
                    component.downcast::<crate::ecs::physics_components::Transform>()
                {
                    world.add_component(entity, *transform);
                    Ok(())
                } else {
                    Err(SceneLoaderError::ComponentCreationFailed(
                        "Type mismatch for Transform".to_string(),
                    ))
                }
            }
            "PhysicsMaterial" => {
                if let Ok(mat) =
                    component.downcast::<crate::ecs::physics_components::PhysicsMaterial>()
                {
                    world.add_component(entity, *mat);
                    Ok(())
                } else {
                    Err(SceneLoaderError::ComponentCreationFailed(
                        "Type mismatch for PhysicsMaterial".to_string(),
                    ))
                }
            }
            _ => Err(SceneLoaderError::ComponentNotRegistered(
                component_type.to_string(),
            )),
        }
    }
}

impl Default for DefaultComponentFactory {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    // =========================================================================
    // ComponentDefinition Tests
    // =========================================================================

    #[test]
    fn test_component_definition_new() {
        let comp = ComponentDefinition::new(
            "Position".to_string(),
            serde_json::json!({ "x": 10, "y": 20 }),
        );

        assert_eq!(comp.component_type, "Position");
        assert_eq!(comp.data["x"], 10);
        assert_eq!(comp.data["y"], 20);
    }

    #[test]
    fn test_component_definition_serde_roundtrip() {
        let original = ComponentDefinition::new(
            "Position".to_string(),
            serde_json::json!({ "x": 1.5, "y": 2.5 }),
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ComponentDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_component_definition_get_f32() {
        let comp = ComponentDefinition::new(
            "Position".to_string(),
            serde_json::json!({ "x": 10.5, "y": 20.0 }),
        );

        assert_eq!(comp.get_f32("x"), Some(10.5));
        assert_eq!(comp.get_f32("y"), Some(20.0));
        assert_eq!(comp.get_f32("z"), None);
    }

    #[test]
    fn test_component_definition_get_i32() {
        let comp = ComponentDefinition::new(
            "Health".to_string(),
            serde_json::json!({ "current": 100, "max": 100 }),
        );

        assert_eq!(comp.get_i32("current"), Some(100));
        assert_eq!(comp.get_i32("missing"), None);
    }

    // =========================================================================
    // BehaviorDefinition Tests
    // =========================================================================

    #[test]
    fn test_behavior_definition_new() {
        let behavior = BehaviorDefinition::new(
            "player_behavior".to_string(),
            "Player".to_string(),
            Some("Controllable character".to_string()),
            vec![ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 0 }),
            )],
        );

        assert_eq!(behavior.id, "player_behavior");
        assert_eq!(behavior.name, "Player");
        assert_eq!(
            behavior.description,
            Some("Controllable character".to_string())
        );
        assert_eq!(behavior.component_count(), 1);
    }

    #[test]
    fn test_behavior_definition_has_component() {
        let behavior = BehaviorDefinition::new(
            "player".to_string(),
            "Player".to_string(),
            None,
            vec![
                ComponentDefinition::new("Position".to_string(), serde_json::json!({})),
                ComponentDefinition::new("Velocity".to_string(), serde_json::json!({})),
            ],
        );

        assert!(behavior.has_component("Position"));
        assert!(behavior.has_component("Velocity"));
        assert!(!behavior.has_component("Health"));
    }

    #[test]
    fn test_behavior_definition_get_component() {
        let pos = ComponentDefinition::new("Position".to_string(), serde_json::json!({"x": 0}));
        let vel = ComponentDefinition::new("Velocity".to_string(), serde_json::json!({"dx": 0}));

        let behavior = BehaviorDefinition::new(
            "player".to_string(),
            "Player".to_string(),
            None,
            vec![pos.clone(), vel],
        );

        let found = behavior.get_component("Position");
        assert!(found.is_some());
        assert_eq!(found.unwrap().component_type, "Position");
    }

    // =========================================================================
    // EntityDefinition Tests
    // =========================================================================

    #[test]
    fn test_entity_definition_new() {
        let entity = EntityDefinition::new("player".to_string());

        assert_eq!(entity.id, "player");
        assert!(entity.components.is_empty());
        assert!(entity.behaviors.is_empty());
        assert!(entity.children.is_empty());
    }

    #[test]
    fn test_entity_definition_builder() {
        let entity = EntityDefinition::new("player".to_string())
            .with_name("Player Entity".to_string())
            .with_component(ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 10 }),
            ))
            .with_behavior("player_behavior".to_string());

        assert_eq!(entity.name, Some("Player Entity".to_string()));
        assert_eq!(entity.components.len(), 1);
        assert_eq!(entity.behaviors.len(), 1);
    }

    #[test]
    fn test_entity_definition_nested_children() {
        let child = EntityDefinition::new("child".to_string()).with_component(
            ComponentDefinition::new("Scale".to_string(), serde_json::json!({ "x": 1 })),
        );

        let parent = EntityDefinition::new("parent".to_string())
            .with_component(ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 0 }),
            ))
            .with_child(child);

        assert_eq!(parent.components.len(), 1);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.total_component_count(), 2);
        assert_eq!(parent.total_entity_count(), 2);
    }

    #[test]
    fn test_entity_definition_get_component() {
        let entity = EntityDefinition::new("player".to_string()).with_component(
            ComponentDefinition::new("Position".to_string(), serde_json::json!({ "x": 10 })),
        );

        let pos = entity.get_component("Position");
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().component_type, "Position");
    }

    #[test]
    fn test_entity_definition_serde_roundtrip() {
        let original = EntityDefinition::new("player".to_string())
            .with_name("Test Player".to_string())
            .with_component(ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 5 }),
            ));

        let json = serde_json::to_string_pretty(&original).unwrap();
        let deserialized: EntityDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.components.len(), deserialized.components.len());
    }

    // =========================================================================
    // Scene Tests
    // =========================================================================

    #[test]
    fn test_scene_new() {
        let scene = Scene::new("level_1".to_string());

        assert_eq!(scene.id, "level_1");
        assert_eq!(scene.version, "1.0.0");
        assert!(scene.entities.is_empty());
        assert!(scene.behaviors.is_empty());
    }

    #[test]
    fn test_scene_builder() {
        let scene = Scene::new("level_1".to_string())
            .with_name("First Level".to_string())
            .with_entity(EntityDefinition::new("player".to_string()))
            .with_entity(EntityDefinition::new("enemy".to_string()));

        assert_eq!(scene.name, Some("First Level".to_string()));
        assert_eq!(scene.entities.len(), 2);
    }

    #[test]
    fn test_scene_total_entity_count() {
        let child = EntityDefinition::new("child".to_string());
        let entity = EntityDefinition::new("parent".to_string()).with_child(child);

        let scene = Scene::new("test".to_string())
            .with_entity(EntityDefinition::new("e1".to_string()))
            .with_entity(entity);

        // e1 + parent + child = 3
        assert_eq!(scene.total_entity_count(), 3);
    }

    #[test]
    fn test_scene_get_behavior() {
        let behavior = BehaviorDefinition::new("npc".to_string(), "NPC".to_string(), None, vec![]);

        let scene = Scene::new("test".to_string()).with_behavior(behavior.clone());

        let found = scene.get_behavior("npc");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "npc");
    }

    #[test]
    fn test_scene_find_entity_shallow() {
        let scene = Scene::new("test".to_string())
            .with_entity(EntityDefinition::new("player".to_string()))
            .with_entity(EntityDefinition::new("enemy".to_string()));

        let found = scene.find_entity("player");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "player");
    }

    #[test]
    fn test_scene_find_entity_nested() {
        let child = EntityDefinition::new("child_entity".to_string());
        let parent = EntityDefinition::new("parent_entity".to_string()).with_child(child);

        let scene = Scene::new("test".to_string()).with_entity(parent);

        let found = scene.find_entity("child_entity");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "child_entity");
    }

    #[test]
    fn test_scene_find_entity_not_found() {
        let scene =
            Scene::new("test".to_string()).with_entity(EntityDefinition::new("player".to_string()));

        let found = scene.find_entity("nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_scene_from_json() {
        let json = r#"{
            "id": "test_scene",
            "name": "Test Scene",
            "version": "1.0.0",
            "metadata": {
                "author": "Test Author"
            },
            "entities": [
                {
                    "id": "player",
                    "components": [
                        {"type": "Position", "data": {"x": 0, "y": 0}}
                    ]
                }
            ],
            "behaviors": []
        }"#;

        let scene: Scene = serde_json::from_str(json).unwrap();

        assert_eq!(scene.id, "test_scene");
        assert_eq!(scene.name, Some("Test Scene".to_string()));
        assert_eq!(scene.metadata.author, Some("Test Author".to_string()));
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].id, "player");
    }

    #[test]
    fn test_scene_to_json() {
        let scene = Scene::new("test".to_string())
            .with_name("Test".to_string())
            .with_entity(EntityDefinition::new("entity_1".to_string()));

        let json = scene.to_json().unwrap();
        let deserialized: Scene = Scene::from_json(&json).unwrap();

        assert_eq!(scene.id, deserialized.id);
        assert_eq!(scene.name, deserialized.name);
        assert_eq!(scene.entities.len(), deserialized.entities.len());
    }

    // =========================================================================
    // SceneMetadata Tests
    // =========================================================================

    #[test]
    fn test_scene_metadata_defaults() {
        let metadata = SceneMetadata::default();

        assert_eq!(metadata.gravity, [0.0, -9.81, 0.0]);
        assert_eq!(metadata.ambient_light, [0.3, 0.3, 0.3]);
        assert!(metadata.fog.is_none());
        assert!(metadata.author.is_none());
    }

    #[test]
    fn test_scene_metadata_with_fog() {
        let fog = FogSettings {
            fog_type: "exponential".to_string(),
            color: [1.0, 1.0, 1.0],
            start: 10.0,
            end: 100.0,
            density: 0.02,
        };

        let metadata = SceneMetadata {
            fog: Some(fog),
            author: Some("Test Author".to_string()),
            description: Some("Test description".to_string()),
            ..SceneMetadata::default()
        };

        assert!(metadata.fog.is_some());
        assert_eq!(metadata.author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_fog_settings_defaults() {
        let fog = FogSettings {
            fog_type: "linear".to_string(),
            color: [0.5, 0.5, 0.5],
            start: 5.0,
            end: 50.0,
            density: 0.01,
        };

        assert_eq!(fog.fog_type, "linear");
        assert_eq!(fog.start, 5.0);
        assert_eq!(fog.end, 50.0);
    }

    // =========================================================================
    // Error Display Tests
    // =========================================================================

    #[test]
    fn test_error_display() {
        let error = SceneLoaderError::InvalidJson("test".to_string());
        assert!(alloc::format!("{}", error).contains("Invalid JSON"));

        let error = SceneLoaderError::EntityNotFound("player".to_string());
        assert!(alloc::format!("{}", error).contains("player"));
    }

    // =========================================================================
    // Schema Validation Tests
    // =========================================================================

    #[test]
    fn test_schemas_are_valid_json() {
        let _: serde_json::Value = serde_json::from_str(COMPONENT_DEFINITION_SCHEMA).unwrap();
        let _: serde_json::Value = serde_json::from_str(BEHAVIOR_DEFINITION_SCHEMA).unwrap();
        let _: serde_json::Value = serde_json::from_str(ENTITY_DEFINITION_SCHEMA).unwrap();
        let _: serde_json::Value = serde_json::from_str(SCENE_SCHEMA).unwrap();
    }

    // =========================================================================
    // Full Scene Integration Test
    // =========================================================================

    #[test]
    fn test_full_scene_serialization() {
        // Create a complete scene with entities and behaviors
        let behavior = BehaviorDefinition::new(
            "npc_behavior".to_string(),
            "NPC".to_string(),
            Some("Non-player character".to_string()),
            vec![
                ComponentDefinition::new(
                    "Health".to_string(),
                    serde_json::json!({ "current": 100, "max": 100 }),
                ),
                ComponentDefinition::new("AI".to_string(), serde_json::json!({ "state": "idle" })),
            ],
        );

        let child_entity = EntityDefinition::new("weapon_slot".to_string()).with_component(
            ComponentDefinition::new("Weapon".to_string(), serde_json::json!({ "damage": 10 })),
        );

        let player_entity = EntityDefinition::new("player".to_string())
            .with_name("Main Player".to_string())
            .with_component(ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 100, "y": 50 }),
            ))
            .with_component(ComponentDefinition::new(
                "Velocity".to_string(),
                serde_json::json!({ "dx": 0, "dy": 0 }),
            ))
            .with_behavior("player_behavior".to_string())
            .with_child(child_entity);

        let scene = Scene::new("level_1".to_string())
            .with_name("First Level".to_string())
            .with_metadata(SceneMetadata {
                author: Some("Game Designer".to_string()),
                description: Some("Tutorial level".to_string()),
                ..SceneMetadata::default()
            })
            .with_entity(player_entity)
            .with_entity(EntityDefinition::new("ground".to_string()))
            .with_behavior(behavior);

        // Serialize to JSON
        let json = scene.to_json().unwrap();

        // Deserialize back
        let deserialized: Scene = Scene::from_json(&json).unwrap();

        // Verify
        assert_eq!(deserialized.id, "level_1");
        assert_eq!(deserialized.name, Some("First Level".to_string()));
        assert_eq!(
            deserialized.metadata.author,
            Some("Game Designer".to_string())
        );
        assert_eq!(deserialized.entities.len(), 2);
        assert_eq!(deserialized.behaviors.len(), 1);

        // Check nested entity
        let player = deserialized.find_entity("player").unwrap();
        assert_eq!(player.name, Some("Main Player".to_string()));
        assert_eq!(player.components.len(), 2);
        assert_eq!(player.children.len(), 1);

        // Check behavior
        let npc_behavior = deserialized.get_behavior("npc_behavior").unwrap();
        assert_eq!(npc_behavior.component_count(), 2);
    }
}
