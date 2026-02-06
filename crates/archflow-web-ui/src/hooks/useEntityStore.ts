/**
 * Hook for managing entities through WASM bridge
 *
 * Provides CRUD operations for entities with automatic state synchronization.
 * Requires WASM bridge to be loaded and initialized.
 *
 * Architecture Reference: EPIC-WEB-002
 */

import { useState, useCallback, useEffect } from "react";
import type { EntityId, EntityData } from "../types/wasm";
import { useArchFlowWasm } from "./useArchFlowWasm.tsx";
import type { WasmBridge } from "../wasm/archflow_web.js";

interface EntityStoreReturn {
  entities: Map<EntityId, EntityData>;
  entityCount: number;
  spawnEntity: (
    x: number,
    y: number,
    width?: number,
    height?: number,
  ) => EntityId;
  deleteEntity: (id: EntityId) => void;
  duplicateEntity: (id: EntityId) => EntityId | null;
  updateEntity: (id: EntityId, updates: Partial<EntityData>) => void;
  updateProperty: <T = unknown>(id: EntityId, key: string, value: T) => void;
  getEntity: (id: EntityId) => EntityData | null;
  refreshEntities: () => void;
}

const DEFAULT_WIDTH = 100;
const DEFAULT_HEIGHT = 60;

function requireWasmBridge(): never {
  throw new Error(
    "WASM bridge is required but not loaded. " +
    "Please build the WASM module first: cd crates/archflow-web && wasm-pack build --target web",
  );
}

export function useEntityStore(): EntityStoreReturn {
  const { bridge, isLoaded, isInitialized } = useArchFlowWasm();
  const [entities, setEntities] = useState<Map<EntityId, EntityData>>(
    new Map(),
  );
  const [entityCount, setEntityCount] = useState(0);

  // Lazy check: only validate bridge when actually needed
  const ensureBridge = useCallback((): WasmBridge => {
    if (!isLoaded || !isInitialized || !bridge) {
      requireWasmBridge();
    }
    return bridge;
  }, [isLoaded, isInitialized, bridge]);

  const syncEntitiesFromWasm = useCallback(() => {
    if (!bridge || !isInitialized) return;

    try {
      const aliveIds = bridge.get_alive_entities();
      const entitiesMap = new Map<EntityId, EntityData>();

      for (const id of aliveIds) {
        const position = bridge.get_entity_position_screen(id);
        const size = bridge.get_entity_size_screen(id);
        const color = bridge.get_entity_color_hex(id);
        const shape = bridge.get_entity_shape(id);
        const label = bridge.get_entity_label(id);
        const isVisible = bridge.is_entity_visible(id);
        const isSelected = bridge.is_entity_selected(id);

        entitiesMap.set(id, {
          id,
          position: { x: position[0], y: position[1] },
          size: { w: size[0], h: size[1] },
          color,
          shape,
          label,
          isVisible,
          isSelected,
        });
      }

      setEntities(entitiesMap);
      setEntityCount(aliveIds.length);
    } catch (err) {
      console.error("Failed to sync entities from WASM:", err);
      throw err;
    }
  }, [bridge, isInitialized]);

  useEffect(() => {
    if (isLoaded && isInitialized && bridge) {
      syncEntitiesFromWasm();
    }
  }, [isLoaded, isInitialized, bridge, syncEntitiesFromWasm]);

  const spawnEntity = useCallback(
    (
      x: number,
      y: number,
      width = DEFAULT_WIDTH,
      height = DEFAULT_HEIGHT,
    ): EntityId => {
      const wasmBridge = ensureBridge();
      const newId = wasmBridge.spawn_entity(x, y, width, height);
      syncEntitiesFromWasm();
      return newId;
    },
    [ensureBridge, syncEntitiesFromWasm],
  );

  const deleteEntity = useCallback(
    (id: EntityId) => {
      const wasmBridge = ensureBridge();
      wasmBridge.select_entity(id);
      wasmBridge.delete_selected();
      syncEntitiesFromWasm();
    },
    [ensureBridge, syncEntitiesFromWasm],
  );

  const duplicateEntity = useCallback(
    (id: EntityId): EntityId | null => {
      const wasmBridge = ensureBridge();

      try {
        const newId = wasmBridge.duplicate_entity(id);
        syncEntitiesFromWasm();
        return newId;
      } catch (err) {
        console.error("Failed to duplicate entity in WASM:", err);
        throw err;
      }
    },
    [ensureBridge, syncEntitiesFromWasm],
  );

  const updateEntity = useCallback(
    (id: EntityId, updates: Partial<EntityData>) => {
      const wasmBridge = ensureBridge();

      if (updates.position) {
        wasmBridge.set_position(id, updates.position.x, updates.position.y);
      }
      if (updates.size) {
        wasmBridge.set_size(id, updates.size.w, updates.size.h);
      }
      if (updates.color) {
        const hex = updates.color.replace("#", "");
        const r = parseInt(hex.substring(0, 2), 16);
        const g = parseInt(hex.substring(2, 4), 16);
        const b = parseInt(hex.substring(4, 6), 16);
        wasmBridge.set_color(id, r, g, b, 255);
      }
      if (updates.shape !== undefined) {
        wasmBridge.set_shape(id, updates.shape);
      }
      if (updates.label !== undefined) {
        wasmBridge.set_label(id, updates.label);
      }
      syncEntitiesFromWasm();
    },
    [ensureBridge, syncEntitiesFromWasm],
  );

  const updateProperty = useCallback(
    <T = unknown>(id: EntityId, key: string, value: T) => {
      const wasmBridge = ensureBridge();

      if (key === "label" && typeof value === "string") {
        wasmBridge.set_label(id, value);
      }

      const entity = entities.get(id);
      if (!entity) return;

      const next = new Map(entities);
      const currentProperties = entity.properties || {};
      next.set(id, {
        ...entity,
        properties: { ...currentProperties, [key]: value },
      });
      setEntities(next);
    },
    [ensureBridge, entities],
  );

  const getEntity = useCallback(
    (id: EntityId): EntityData | null => {
      const wasmBridge = ensureBridge();

      try {
        const position = wasmBridge.get_entity_position_screen(id);
        const size = wasmBridge.get_entity_size_screen(id);
        const color = wasmBridge.get_entity_color_hex(id);
        const shape = wasmBridge.get_entity_shape(id);
        const label = wasmBridge.get_entity_label(id);
        const isVisible = wasmBridge.is_entity_visible(id);
        const isSelected = wasmBridge.is_entity_selected(id);

        return {
          id,
          position: { x: position[0], y: position[1] },
          size: { w: size[0], h: size[1] },
          color,
          shape,
          label,
          isVisible,
          isSelected,
        };
      } catch (err) {
        console.error("Failed to get entity from WASM:", err);
        throw err;
      }
    },
    [ensureBridge],
  );

  const refreshEntities = useCallback(() => {
    syncEntitiesFromWasm();
  }, [syncEntitiesFromWasm]);

  return {
    entities,
    entityCount,
    spawnEntity,
    deleteEntity,
    duplicateEntity,
    updateEntity,
    updateProperty,
    getEntity,
    refreshEntities,
  };
}
