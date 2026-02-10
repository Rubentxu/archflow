// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Declarative JSON API for ECS
//
// This module provides a declarative JSON-based API for defining and creating
// ECS components and behaviors. It enables dynamic component creation from
// JSON definitions, supporting data-driven game object creation.
//
// Key Features:
// - ComponentDefinition: JSON-based component definitions
// - BehaviorDefinition: Complete behavior definitions with multiple components
// - ComponentFactory: Creates components from JSON definitions
// - BehaviorRegistry: Stores and retrieves behavior definitions
//
// Architecture:
// - Type-safe component creation via factory pattern
// - JSON Schema validation for component definitions
// - Error handling with detailed error messages
// - Supports dynamic component registration
//
// Note: serde_json requires std, so this module is conditionally compiled
//       and only available when std is available (non-WASM builds).
#![cfg(feature = "std")]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use archflow_core::EntityId;
use serde::{Deserialize, Serialize};

use crate::ecs::{Component, ComponentRegistry};

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentDefinition
// ═══════════════════════════════════════════════════════════════════════════════

/// Defines a component in JSON format
///
/// This structure represents a component that can be created from JSON data.
/// It contains the component type identifier and a generic configuration
/// object using serde_json::Value for flexible schema definition.
///
/// # Examples
///
/// \`\`\`ignore
/// use archflow_logic::api::json::ComponentDefinition;
/// use serde_json::json;
///
/// let def = ComponentDefinition {
///     type: "Position".to_string(),
///     config: json!({ "x": 10.0, "y": 20.0 }),
/// };
/// \`\`\`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComponentDefinition {
    /// The component type identifier (e.g., "Position", "Velocity")
    #[serde(rename = "type")]
    pub component_type: String,

    /// Component configuration data (flexible JSON structure)
    pub config: serde_json::Value,
}

impl ComponentDefinition {
    /// Creates a new ComponentDefinition
    ///
    /// # Arguments
    ///
    /// * \`component_type\` - The type identifier for the component
    /// * \`config\` - The configuration data for the component
    #[inline]
    #[must_use]
    pub const fn new(component_type: String, config: serde_json::Value) -> Self {
        Self {
            component_type,
            config,
        }
    }

    /// Creates a ComponentDefinition from a JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string is invalid or cannot be
    /// deserialized into a ComponentDefinition.
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// use archflow_logic::api::json::ComponentDefinition;
    ///
    /// let json = r#"{"type":"Position","config":{"x":10.0,"y":20.0}}"#;
    /// let def = ComponentDefinition::from_json(json).unwrap();
    /// \`\`\`
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Serializes this ComponentDefinition to a JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// use archflow_logic::api::json::ComponentDefinition;
    /// use serde_json::json;
    ///
    /// let def = ComponentDefinition::new("Position".to_string(), json!({ "x": 10.0 }));
    /// let json_str = def.to_json().unwrap();
    /// \`\`\`
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Validates that this component definition has a valid structure
    ///
    /// Checks that:
    /// - The type string is not empty
    /// - The config is an object (not a primitive or array)
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// use archflow_logic::api::json::ComponentDefinition;
    /// use serde_json::json;
    ///
    /// let def = ComponentDefinition::new("Position".to_string(), json!({ "x": 10.0 }));
    /// assert!(def.validate().is_ok());
    /// \`\`\`
    pub fn validate(&self) -> Result<(), ComponentFactoryError> {
        if self.component_type.is_empty() {
            return Err(ComponentFactoryError::InvalidType(
                "Component type cannot be empty".to_string(),
            ));
        }

        if !self.config.is_object() {
            return Err(ComponentFactoryError::InvalidConfig(
                "Component config must be a JSON object".to_string(),
            ));
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BehaviorDefinition
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete definition of a behavior (collection of components)
///
/// A behavior definition represents a complete set of components that
/// define a particular entity behavior. For example, a "Player" behavior
/// might include Position, Velocity, Health, and Input components.
///
/// # Examples
///
/// \`\`\`ignore
/// use archflow_logic::api::json::{BehaviorDefinition, ComponentDefinition};
/// use serde_json::json;
///
/// let behavior = BehaviorDefinition {
///     id: "player_behavior".to_string(),
///     name: "Player".to_string(),
///     description: Some("Controllable player character".to_string()),
///     components: vec![
///         ComponentDefinition::new("Position".to_string(), json!({ "x": 0, "y": 0 })),
///         ComponentDefinition::new("Velocity".to_string(), json!({ "dx": 0, "dy": 0 })),
///     ],
/// };
/// \`\`\`
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BehaviorDefinition {
    /// Unique identifier for this behavior definition
    pub id: String,

    /// Human-readable name for this behavior
    pub name: String,

    /// Optional description of what this behavior does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// List of component definitions that make up this behavior
    pub components: Vec<ComponentDefinition>,
}

impl BehaviorDefinition {
    /// Creates a new BehaviorDefinition
    ///
    /// # Arguments
    ///
    /// * \`id\` - Unique identifier for the behavior
    /// * \`name\` - Human-readable name
    /// * \`description\` - Optional description
    /// * \`components\` - List of component definitions
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

    /// Creates a BehaviorDefinition from a JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string is invalid or cannot be
    /// deserialized into a BehaviorDefinition.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Serializes this BehaviorDefinition to a JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Validates that this behavior definition has a valid structure
    ///
    /// Checks that:
    /// - The id is not empty
    /// - The name is not empty
    /// - All component definitions are valid
    /// - Component types are unique (no duplicates)
    ///
    /// # Errors
    ///
    /// Returns a ComponentFactoryError if validation fails.
    pub fn validate(&self) -> Result<(), ComponentFactoryError> {
        if self.id.is_empty() {
            return Err(ComponentFactoryError::InvalidType(
                "Behavior ID cannot be empty".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(ComponentFactoryError::InvalidType(
                "Behavior name cannot be empty".to_string(),
            ));
        }

        // Validate all components
        for component in &self.components {
            component.validate()?;
        }

        // Check for duplicate component types
        let mut seen_types = BTreeSet::new();
        for component in &self.components {
            if !seen_types.insert(&component.component_type) {
                return Err(ComponentFactoryError::DuplicateComponent(
                    component.component_type.clone(),
                ));
            }
        }

        Ok(())
    }

    /// Returns the number of components in this behavior definition
    #[inline]
    #[must_use]
    pub const fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Checks if this behavior contains a component of the given type
    #[must_use]
    pub fn has_component_type(&self, component_type: &str) -> bool {
        self.components
            .iter()
            .any(|c| c.component_type == component_type)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentFactoryError
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors that can occur during component creation from JSON
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentFactoryError {
    /// The component type is not registered in the factory
    UnregisteredType(String),

    /// The component type string is invalid
    InvalidType(String),

    /// The component configuration is invalid
    InvalidConfig(String),

    /// A duplicate component type was found in a behavior definition
    DuplicateComponent(String),

    /// Failed to deserialize component configuration
    DeserializationError(String),

    /// Component creation failed
    CreationFailed(String),
}

impl alloc::fmt::Display for ComponentFactoryError {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        match self {
            Self::UnregisteredType(type_name) => {
                write!(f, "Component type '{}' is not registered", type_name)
            }
            Self::InvalidType(msg) => write!(f, "Invalid component type: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid component configuration: {}", msg),
            Self::DuplicateComponent(type_name) => {
                write!(f, "Duplicate component type: '{}'", type_name)
            }
            Self::DeserializationError(msg) => {
                write!(f, "Failed to deserialize component: {}", msg)
            }
            Self::CreationFailed(msg) => write!(f, "Component creation failed: {}", msg),
        }
    }
}

// Note: serde_json requires std, so std::error::Error is only available in std environments
// The Display impl above provides error formatting for no_std compatibility

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentCreator Trait
// ═══════════════════════════════════════════════════════════════════════════════

/// Trait for creating components from JSON configuration
///
/// Implement this trait to enable JSON-based creation of custom components.
pub trait ComponentCreator: Send + Sync {
    /// Creates a component from JSON configuration and adds it to the registry
    ///
    /// # Arguments
    ///
    /// * \`entity_id\` - The entity ID to attach the component to
    /// * \`config\` - The JSON configuration for the component
    /// * \`registry\` - The component registry to add the component to
    ///
    /// # Errors
    ///
    /// Returns a ComponentFactoryError if creation fails.
    fn create_component(
        &self,
        entity_id: EntityId,
        config: &serde_json::Value,
        registry: &mut ComponentRegistry,
    ) -> Result<(), ComponentFactoryError>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentFactory
// ═══════════════════════════════════════════════════════════════════════════════

/// Factory for creating components from JSON definitions
///
/// The ComponentFactory manages component type creators and provides
/// a unified interface for creating components from JSON definitions.
///
/// # Examples
///
/// \`\`\`ignore
/// use archflow_logic::api::json::ComponentFactory;
/// use archflow_logic::ecs::ComponentRegistry;
///
/// let mut factory = ComponentFactory::new();
/// let mut registry = ComponentRegistry::new();
///
/// // Register a component creator
/// factory.register_creator("Position", Box::new(PositionCreator));
///
/// // Create a component from JSON
/// let def = ComponentDefinition::new("Position".to_string(), json!({ "x": 10.0 }));
/// factory.create_component_from_json(0, &def, &mut registry).unwrap();
/// \`\`\`
pub struct ComponentFactory {
    /// Map of component type names to their creators
    creators: BTreeMap<String, Box<dyn ComponentCreator>>,
}

impl ComponentFactory {
    /// Creates a new empty ComponentFactory
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            creators: BTreeMap::new(),
        }
    }

    /// Registers a component creator for a given type
    ///
    /// # Arguments
    ///
    /// * \`type_name\` - The type identifier (e.g., "Position")
    /// * \`creator\` - The component creator implementation
    ///
    /// # Panics
    ///
    /// Panics if a creator is already registered for this type.
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// factory.register_creator("Position", Box::new(PositionCreator));
    /// \`\`\`
    pub fn register_creator(&mut self, type_name: &str, creator: Box<dyn ComponentCreator>) {
        if self.creators.contains_key(type_name) {
            panic!(
                "Component creator for type '{}' is already registered",
                type_name
            );
        }
        self.creators.insert(type_name.to_string(), creator);
    }

    /// Creates a component from a ComponentDefinition
    ///
    /// # Arguments
    ///
    /// * \`entity_id\` - The entity ID to attach the component to
    /// * \`def\` - The component definition
    /// * \`registry\` - The component registry to add the component to
    ///
    /// # Errors
    ///
    /// Returns a ComponentFactoryError if:
    /// - The component type is not registered
    /// - The configuration is invalid
    /// - Component creation fails
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// let def = ComponentDefinition::new("Position".to_string(), json!({ "x": 10.0 }));
    /// factory.create_component_from_json(0, &def, &mut registry).unwrap();
    /// \`\`\`
    pub fn create_component_from_json(
        &self,
        entity_id: EntityId,
        def: &ComponentDefinition,
        registry: &mut ComponentRegistry,
    ) -> Result<(), ComponentFactoryError> {
        // Validate the definition
        def.validate()?;

        // Get the creator for this type
        let creator = self
            .creators
            .get(&def.component_type)
            .ok_or_else(|| ComponentFactoryError::UnregisteredType(def.component_type.clone()))?;

        // Create the component
        creator
            .create_component(entity_id, &def.config, registry)
            .map_err(|e| ComponentFactoryError::CreationFailed(format!("{}", e)))
    }

    /// Checks if a component type is registered
    #[inline]
    #[must_use]
    pub fn has_creator(&self, type_name: &str) -> bool {
        self.creators.contains_key(type_name)
    }

    /// Returns the number of registered component creators
    #[inline]
    #[must_use]
    pub fn creator_count(&self) -> usize {
        self.creators.len()
    }
}

impl Default for ComponentFactory {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BehaviorRegistry
// ═══════════════════════════════════════════════════════════════════════════════

/// Registry for behavior definitions
///
/// The BehaviorRegistry stores and manages behavior definitions,
/// allowing retrieval by ID and validation of behaviors.
///
/// # Examples
///
/// \`\`\`ignore
/// use archflow_logic::api::json::{BehaviorRegistry, BehaviorDefinition};
///
/// let mut registry = BehaviorRegistry::new();
/// let behavior = BehaviorDefinition::new(
///     "player".to_string(),
///     "Player".to_string(),
///     Some("Controllable character".to_string()),
///     vec![],
/// );
///
/// registry.add_behavior(behavior);
/// assert!(registry.has_behavior("player"));
/// \`\`\`
pub struct BehaviorRegistry {
    /// Map of behavior IDs to their definitions
    behaviors: BTreeMap<String, BehaviorDefinition>,
}

impl BehaviorRegistry {
    /// Creates a new empty BehaviorRegistry
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            behaviors: BTreeMap::new(),
        }
    }

    /// Adds a behavior definition to the registry
    ///
    /// # Arguments
    ///
    /// * \`behavior\` - The behavior definition to add
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A behavior with the same ID already exists
    /// - The behavior definition is invalid
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// registry.add_behavior(behavior_definition).unwrap();
    /// \`\`\`
    pub fn add_behavior(
        &mut self,
        behavior: BehaviorDefinition,
    ) -> Result<(), ComponentFactoryError> {
        // Validate the behavior
        behavior.validate()?;

        // Check for duplicate ID
        if self.behaviors.contains_key(&behavior.id) {
            return Err(ComponentFactoryError::DuplicateComponent(behavior.id));
        }

        self.behaviors.insert(behavior.id.clone(), behavior);
        Ok(())
    }

    /// Gets a behavior definition by ID
    ///
    /// # Arguments
    ///
    /// * \`id\` - The behavior ID to look up
    ///
    /// # Returns
    ///
    /// Some(&BehaviorDefinition) if found, None otherwise
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// if let Some(behavior) = registry.get_behavior("player") {
    ///     println!("Found behavior: {}", behavior.name);
    /// }
    /// \`\`\`
    #[inline]
    #[must_use]
    pub fn get_behavior(&self, id: &str) -> Option<&BehaviorDefinition> {
        self.behaviors.get(id)
    }

    /// Removes a behavior definition from the registry
    ///
    /// # Arguments
    ///
    /// * \`id\` - The behavior ID to remove
    ///
    /// # Returns
    ///
    /// Some(BehaviorDefinition) if found and removed, None otherwise
    #[inline]
    pub fn remove_behavior(&mut self, id: &str) -> Option<BehaviorDefinition> {
        self.behaviors.remove(id)
    }

    /// Checks if a behavior ID is registered
    #[inline]
    #[must_use]
    pub fn has_behavior(&self, id: &str) -> bool {
        self.behaviors.contains_key(id)
    }

    /// Returns the number of registered behaviors
    #[inline]
    #[must_use]
    pub fn behavior_count(&self) -> usize {
        self.behaviors.len()
    }

    /// Returns an iterator over all behavior definitions
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &BehaviorDefinition> {
        self.behaviors.values()
    }

    /// Returns all behavior IDs
    #[inline]
    pub fn behavior_ids(&self) -> Vec<String> {
        self.behaviors.keys().cloned().collect()
    }
}

impl Default for BehaviorRegistry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// JSON Schema Definitions
// ═══════════════════════════════════════════════════════════════════════════════

/// JSON Schema for ComponentDefinition validation
///
/// This schema defines the expected structure for component definitions:
/// \`\`\`json
/// {
///   "type": "object",
///   "properties": {
///     "type": { "type": "string" },
///     "config": { "type": "object" }
///   },
///   "required": ["type", "config"]
/// }
/// \`\`\`
pub const COMPONENT_DEFINITION_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "ComponentDefinition",
  "description": "Defines a component in JSON format",
  "properties": {
    "type": {
      "type": "string",
      "description": "The component type identifier"
    },
    "config": {
      "type": "object",
      "description": "Component configuration data"
    }
  },
  "required": ["type", "config"],
  "additionalProperties": false
}"#;

/// JSON Schema for BehaviorDefinition validation
///
/// This schema defines the expected structure for behavior definitions:
/// \`\`\`json
/// {
///   "type": "object",
///   "properties": {
///     "id": { "type": "string" },
///     "name": { "type": "string" },
///     "description": { "type": "string" },
///     "components": {
///       "type": "array",
///       "items": { "$ref": "#ComponentDefinition" }
///     }
///   },
///   "required": ["id", "name", "components"]
/// }
/// \`\`\`
pub const BEHAVIOR_DEFINITION_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "BehaviorDefinition",
  "description": "Complete definition of a behavior (collection of components)",
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier for this behavior definition"
    },
    "name": {
      "type": "string",
      "description": "Human-readable name for this behavior"
    },
    "description": {
      "type": "string",
      "description": "Optional description of what this behavior does"
    },
    "components": {
      "type": "array",
      "description": "List of component definitions that make up this behavior",
      "items": {
        "type": "object",
        "properties": {
          "type": { "type": "string" },
          "config": { "type": "object" }
        },
        "required": ["type", "config"]
      }
    }
  },
  "required": ["id", "name", "components"],
  "additionalProperties": false
}"#;

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{Component, VecStorage};

    // Test component for demonstration
    #[derive(Clone, Debug, PartialEq)]
    struct TestPosition {
        x: f32,
        y: f32,
    }

    impl Component for TestPosition {
        type Storage = VecStorage<TestPosition>;
    }

    // Test component creator
    struct TestPositionCreator;

    impl ComponentCreator for TestPositionCreator {
        fn create_component(
            &self,
            entity_id: EntityId,
            config: &serde_json::Value,
            registry: &mut ComponentRegistry,
        ) -> Result<(), ComponentFactoryError> {
            // Ensure the component type is registered
            if !registry.is_registered::<TestPosition>() {
                registry.register::<TestPosition>();
            }

            // Parse the configuration
            let x = config.get("x").and_then(|v| v.as_f64()).ok_or_else(|| {
                ComponentFactoryError::InvalidConfig("Missing or invalid 'x' field".to_string())
            })? as f32;

            let y = config.get("y").and_then(|v| v.as_f64()).ok_or_else(|| {
                ComponentFactoryError::InvalidConfig("Missing or invalid 'y' field".to_string())
            })? as f32;

            // Create and insert the component
            let storage = registry.get_storage_mut::<TestPosition>().ok_or_else(|| {
                ComponentFactoryError::CreationFailed(
                    "Failed to get TestPosition storage".to_string(),
                )
            })?;

            storage.insert(entity_id, TestPosition { x, y });
            Ok(())
        }
    }

    #[test]
    fn test_component_definition_serialization() {
        let def = ComponentDefinition::new(
            "Position".to_string(),
            serde_json::json!({ "x": 10.0, "y": 20.0 }),
        );

        let json_str = def.to_json().unwrap();
        let deserialized = ComponentDefinition::from_json(&json_str).unwrap();

        assert_eq!(def, deserialized);
    }

    #[test]
    fn test_component_definition_from_json() {
        let json_str = r#"{"type":"Position","config":{"x":10.0,"y":20.0}}"#;
        let def = ComponentDefinition::from_json(json_str).unwrap();

        assert_eq!(def.component_type, "Position");
        assert_eq!(def.config["x"], 10.0);
        assert_eq!(def.config["y"], 20.0);
    }

    #[test]
    fn test_component_definition_validate() {
        let valid_def =
            ComponentDefinition::new("Position".to_string(), serde_json::json!({ "x": 10.0 }));
        assert!(valid_def.validate().is_ok());

        let empty_type = ComponentDefinition::new("".to_string(), serde_json::json!({ "x": 10.0 }));
        assert!(empty_type.validate().is_err());

        let invalid_config =
            ComponentDefinition::new("Position".to_string(), serde_json::json!("not an object"));
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_behavior_definition_serialization() {
        let behavior = BehaviorDefinition::new(
            "player_behavior".to_string(),
            "Player".to_string(),
            Some("Controllable player character".to_string()),
            vec![ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 0, "y": 0 }),
            )],
        );

        let json_str = behavior.to_json().unwrap();
        let deserialized = BehaviorDefinition::from_json(&json_str).unwrap();

        assert_eq!(behavior, deserialized);
    }

    #[test]
    fn test_behavior_definition_from_json() {
        let json_str = r#"{
            "id":"player_behavior",
            "name":"Player",
            "description":"Controllable character",
            "components":[
                {"type":"Position","config":{"x":0,"y":0}},
                {"type":"Velocity","config":{"dx":0,"dy":0}}
            ]
        }"#;

        let behavior = BehaviorDefinition::from_json(json_str).unwrap();

        assert_eq!(behavior.id, "player_behavior");
        assert_eq!(behavior.name, "Player");
        assert_eq!(
            behavior.description,
            Some("Controllable character".to_string())
        );
        assert_eq!(behavior.components.len(), 2);
    }

    #[test]
    fn test_behavior_definition_validate() {
        let valid_behavior = BehaviorDefinition::new(
            "player".to_string(),
            "Player".to_string(),
            None,
            vec![ComponentDefinition::new(
                "Position".to_string(),
                serde_json::json!({ "x": 0 }),
            )],
        );
        assert!(valid_behavior.validate().is_ok());

        let empty_id = BehaviorDefinition::new("".to_string(), "Player".to_string(), None, vec![]);
        assert!(empty_id.validate().is_err());

        let duplicate_components = BehaviorDefinition::new(
            "player".to_string(),
            "Player".to_string(),
            None,
            vec![
                ComponentDefinition::new("Position".to_string(), serde_json::json!({})),
                ComponentDefinition::new("Position".to_string(), serde_json::json!({})),
            ],
        );
        assert!(duplicate_components.validate().is_err());
    }

    #[test]
    fn test_behavior_definition_has_component_type() {
        let behavior = BehaviorDefinition::new(
            "player".to_string(),
            "Player".to_string(),
            None,
            vec![
                ComponentDefinition::new("Position".to_string(), serde_json::json!({})),
                ComponentDefinition::new("Velocity".to_string(), serde_json::json!({})),
            ],
        );

        assert!(behavior.has_component_type("Position"));
        assert!(behavior.has_component_type("Velocity"));
        assert!(!behavior.has_component_type("Health"));
    }

    #[test]
    fn test_behavior_definition_component_count() {
        let behavior = BehaviorDefinition::new(
            "player".to_string(),
            "Player".to_string(),
            None,
            vec![
                ComponentDefinition::new("Position".to_string(), serde_json::json!({})),
                ComponentDefinition::new("Velocity".to_string(), serde_json::json!({})),
                ComponentDefinition::new("Health".to_string(), serde_json::json!({})),
            ],
        );

        assert_eq!(behavior.component_count(), 3);
    }

    #[test]
    fn test_component_factory_create() {
        let mut factory = ComponentFactory::new();
        factory.register_creator("Position", Box::new(TestPositionCreator));

        let mut registry = ComponentRegistry::new();
        let def = ComponentDefinition::new(
            "Position".to_string(),
            serde_json::json!({ "x": 10.5, "y": 20.5 }),
        );

        let result = factory.create_component_from_json(0, &def, &mut registry);
        assert!(result.is_ok());

        // Verify the component was created
        let positions = registry.get_storage::<TestPosition>().unwrap();
        let pos = positions.get(0).unwrap();
        assert_eq!(pos.x, 10.5);
        assert_eq!(pos.y, 20.5);
    }

    #[test]
    fn test_component_factory_invalid_type() {
        let factory = ComponentFactory::new();
        let mut registry = ComponentRegistry::new();

        let def =
            ComponentDefinition::new("NonExistent".to_string(), serde_json::json!({ "x": 10.0 }));

        let result = factory.create_component_from_json(0, &def, &mut registry);
        assert!(matches!(
            result,
            Err(ComponentFactoryError::UnregisteredType(_))
        ));
    }

    #[test]
    fn test_component_factory_has_creator() {
        let mut factory = ComponentFactory::new();
        assert!(!factory.has_creator("Position"));

        factory.register_creator("Position", Box::new(TestPositionCreator));
        assert!(factory.has_creator("Position"));
    }

    #[test]
    fn test_component_factory_creator_count() {
        let mut factory = ComponentFactory::new();
        assert_eq!(factory.creator_count(), 0);

        factory.register_creator("Position", Box::new(TestPositionCreator));
        factory.register_creator("Velocity", Box::new(TestPositionCreator));
        assert_eq!(factory.creator_count(), 2);
    }

    #[test]
    fn test_behavior_registry_add_get() {
        let mut registry = BehaviorRegistry::new();
        let behavior =
            BehaviorDefinition::new("player".to_string(), "Player".to_string(), None, vec![]);

        assert!(registry.add_behavior(behavior.clone()).is_ok());
        assert!(registry.has_behavior("player"));

        let retrieved = registry.get_behavior("player");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "player");
    }

    #[test]
    fn test_behavior_registry_duplicate() {
        let mut registry = BehaviorRegistry::new();
        let behavior =
            BehaviorDefinition::new("player".to_string(), "Player".to_string(), None, vec![]);

        assert!(registry.add_behavior(behavior.clone()).is_ok());
        assert!(registry.add_behavior(behavior).is_err());
    }

    #[test]
    fn test_behavior_registry_remove() {
        let mut registry = BehaviorRegistry::new();
        let behavior =
            BehaviorDefinition::new("player".to_string(), "Player".to_string(), None, vec![]);

        registry.add_behavior(behavior).unwrap();
        assert!(registry.has_behavior("player"));

        let removed = registry.remove_behavior("player");
        assert!(removed.is_some());
        assert!(!registry.has_behavior("player"));
    }

    #[test]
    fn test_behavior_registry_behavior_count() {
        let mut registry = BehaviorRegistry::new();
        assert_eq!(registry.behavior_count(), 0);

        registry
            .add_behavior(BehaviorDefinition::new(
                "player".to_string(),
                "Player".to_string(),
                None,
                vec![],
            ))
            .unwrap();
        registry
            .add_behavior(BehaviorDefinition::new(
                "enemy".to_string(),
                "Enemy".to_string(),
                None,
                vec![],
            ))
            .unwrap();

        assert_eq!(registry.behavior_count(), 2);
    }

    #[test]
    fn test_behavior_registry_iter() {
        let mut registry = BehaviorRegistry::new();

        registry
            .add_behavior(BehaviorDefinition::new(
                "player".to_string(),
                "Player".to_string(),
                None,
                vec![],
            ))
            .unwrap();
        registry
            .add_behavior(BehaviorDefinition::new(
                "enemy".to_string(),
                "Enemy".to_string(),
                None,
                vec![],
            ))
            .unwrap();

        let ids: Vec<_> = registry.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"player"));
        assert!(ids.contains(&"enemy"));
    }

    #[test]
    fn test_behavior_registry_behavior_ids() {
        let mut registry = BehaviorRegistry::new();

        registry
            .add_behavior(BehaviorDefinition::new(
                "player".to_string(),
                "Player".to_string(),
                None,
                vec![],
            ))
            .unwrap();
        registry
            .add_behavior(BehaviorDefinition::new(
                "enemy".to_string(),
                "Enemy".to_string(),
                None,
                vec![],
            ))
            .unwrap();

        let ids = registry.behavior_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"player".to_string()));
        assert!(ids.contains(&"enemy".to_string()));
    }

    #[test]
    fn test_component_factory_error_display() {
        let error = ComponentFactoryError::UnregisteredType("TestType".to_string());
        let display_str = alloc::format!("{}", error);
        assert!(display_str.contains("TestType"));
        assert!(display_str.contains("not registered"));
    }

    #[test]
    fn test_json_schemas_defined() {
        // Verify that the schemas are valid JSON
        let _component_schema: serde_json::Value =
            serde_json::from_str(COMPONENT_DEFINITION_SCHEMA).unwrap();
        let _behavior_schema: serde_json::Value =
            serde_json::from_str(BEHAVIOR_DEFINITION_SCHEMA).unwrap();
    }
}
