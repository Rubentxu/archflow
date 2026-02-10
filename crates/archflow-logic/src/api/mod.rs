// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - API Module
//
// This module provides various APIs for interacting with the ArchFlow Logic system,
// including declarative JSON-based entity and component definition.
//
// Note: The JSON API requires serde_json which depends on std, so this module
//       is primarily intended for non-WASM builds or testing environments.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(clippy::std_instead_of_alloc)]

pub mod json;

// Re-export commonly used types
pub use json::{
    BehaviorDefinition, BehaviorRegistry, ComponentCreator, ComponentDefinition,
    ComponentFactory, ComponentFactoryError, BEHAVIOR_DEFINITION_SCHEMA,
    COMPONENT_DEFINITION_SCHEMA,
};
