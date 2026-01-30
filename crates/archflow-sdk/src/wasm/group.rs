//! WASM bindings for group module
//!
//! Provides WebAssembly bindings for grouping operations

use crate::group::{GroupManager, MAX_GROUP_DEPTH};
use archflow_core::EntityId;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// WASM-exposed group manager
#[wasm_bindgen]
pub struct JsGroupManager {
    inner: GroupManager,
}

#[wasm_bindgen]
impl JsGroupManager {
    /// Creates a new group manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: GroupManager::new(),
        }
    }

    /// Groups multiple shapes together
    #[wasm_bindgen]
    pub fn group(&mut self, _shape_ids: Vec<String>) -> String {
        // TODO: Implement using GroupManager::create_group
        // This requires Canvas reference which is not available in current design
        String::new()
    }

    /// Ungroups a group
    #[wasm_bindgen]
    pub fn ungroup(&mut self, group_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(group_id) {
            return self.inner.ungroup(id).is_ok();
        }
        false
    }

    /// Ungroups a shape from its parent group
    #[wasm_bindgen]
    pub fn ungroup_shape(&mut self, _shape_id: &str) -> bool {
        // TODO: Implement using GroupManager::remove_shape_from_group
        false
    }

    /// Gets the group ID for a shape
    #[wasm_bindgen]
    pub fn get_group_id(&self, shape_id: &str) -> String {
        if let Some(id) = EntityId::from_str(shape_id) {
            if let Some(group_id) = self.inner.get_group_for_shape(id) {
                return group_id.to_string();
            }
        }
        String::new()
    }

    /// Checks if a shape is in a group
    #[wasm_bindgen(js_name = isGrouped)]
    pub fn is_grouped(&self, shape_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(shape_id) {
            self.inner.is_grouped(id)
        } else {
            false
        }
    }

    /// Checks if an ID represents a group
    #[wasm_bindgen(js_name = isGroup)]
    pub fn is_group(&self, id: &str) -> bool {
        EntityId::from_str(id).is_some()
    }

    /// Gets all shapes in a group
    #[wasm_bindgen]
    pub fn get_group_shapes(&self, group_id: &str) -> String {
        if let Some(id) = EntityId::from_str(group_id) {
            if let Some(shapes) = self.inner.get_group_shapes(id) {
                let ids: Vec<String> = shapes.iter().map(|s| s.to_string()).collect();
                return serde_json::to_string(&ids).unwrap_or_default();
            }
        }
        "[]".to_string()
    }

    /// Gets all groups
    #[wasm_bindgen(js_name = getAllGroups)]
    pub fn get_all_groups(&self) -> String {
        let groups: Vec<String> = self
            .inner
            .get_all_groups()
            .iter()
            .map(|g| g.to_string())
            .collect();
        serde_json::to_string(&groups).unwrap_or_default()
    }

    /// Gets the number of groups
    #[wasm_bindgen(getter = groupCount)]
    pub fn group_count(&self) -> usize {
        self.inner.group_count()
    }

    /// Gets the maximum nesting depth
    #[wasm_bindgen(getter = maxGroupDepth)]
    pub fn max_group_depth() -> u32 {
        MAX_GROUP_DEPTH
    }

    /// Locks a group (prevents editing)
    #[wasm_bindgen]
    pub fn lock_group(&mut self, group_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(group_id) {
            return self.inner.lock_group(id).is_ok();
        }
        false
    }

    /// Unlocks a group
    #[wasm_bindgen]
    pub fn unlock_group(&mut self, group_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(group_id) {
            return self.inner.unlock_group(id).is_ok();
        }
        false
    }

    /// Checks if a group is locked
    #[wasm_bindgen(js_name = isGroupLocked)]
    pub fn is_group_locked(&self, group_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(group_id) {
            return self.inner.is_group_locked(id);
        }
        false
    }
}

/// TypeScript definitions for group module
pub const GROUP_TYPES: &str = r#"
/**
 * Group Manager for WASM
 */
export class JsGroupManager {
    constructor();
    group(shapeIds: string[]): string;
    ungroup(groupId: string): boolean;
    ungroupShape(shapeId: string): boolean;
    getGroupId(shapeId: string): string;
    isGrouped(shapeId: string): boolean;
    isGroup(id: string): boolean;
    getGroupShapes(groupId: string): string[];
    getAllGroups(): string[];
    readonly groupCount: number;
    static readonly maxGroupDepth: number;
    lockGroup(groupId: string): boolean;
    unlockGroup(groupId: string): boolean;
    isGroupLocked(groupId: string): boolean;
}
"#;

/// Get TypeScript definitions for group module
#[wasm_bindgen]
pub fn get_group_typescript_definitions() -> String {
    GROUP_TYPES.to_string()
}
