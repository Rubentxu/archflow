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
import { useArchFlowWasm } from "./useArchFlowWasm";
import { getTypedBridge } from "./wasm-bridge";

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
    "Please build the WASM module first: cargo build -p archflow-web && wasm-pack build --target web"
  );
}

export function useEntityStore(): EntityStoreReturn {
  const { bridge, isLoaded, isInitialized } = useArchFlowWasm();
  const [entities, setEntities] = useState<Map<EntityId, EntityData>>(
    new Map(),
  );
  const [entityCount, setEntityCount] = useState(0);

  const typedBridge = getTypedBridge(bridge);

  if (!isLoaded || !isInitialized || !typedBridge) {
    requireWasmBridge();
  }

  const syncEntitiesFromWasm = useCallback(() => {
    if (!typedBridge || !isInitialized) return;

    try {
      const aliveIds = typedBridge.getAliveEntities();
      const entitiesMap = new Map<EntityId, EntityData>();

      for (const id of aliveIds) {
        const position = typedBridge.getEntityPositionScreen(id);
        const size = typedBridge.getEntitySizeScreen(id);
        const color = typedBridge.getEntityColorHex(id);
        const shape = typedBridge.getEntityShape(id);
        const label = typedBridge.getEntityLabel(id);
        const isVisible = typedBridge.isEntityVisible(id);
        const isSelected = typedBridge.isEntitySelected(id);

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
  }, [typedBridge, isInitialized]);

  useEffect(() => {
    if (isLoaded && isInitialized) {
      syncEntitiesFromWasm();
    }
  }, [isLoaded, isInitialized, syncEntitiesFromWasm]);

  const spawnEntity = useCallback(
    (
      x: number,
      y: number,
      width = DEFAULT_WIDTH,
      height = DEFAULT_HEIGHT,
    ): EntityId => {
      if (!typedBridge) {
        requireWasmBridge();
      }

      const newId = typedBridge.spawnEntity(x, y, width, height);
      syncEntitiesFromWasm();
      return newId;
    },
    [typedBridge, syncEntitiesFromWasm],
  );

  const deleteEntity = useCallback(
    (id: EntityId) => {
      if (!typedBridge) {
        requireWasmBridge();
      }

      typedBridge.selectEntity(id);
      typedBridge.deleteSelected();
      syncEntitiesFromWasm();
    },
    [typedBridge, syncEntitiesFromWasm],
  );

  const duplicateEntity = useCallback(
    (id: EntityId): EntityId | null => {
      if (!typedBridge) {
        requireWasmBridge();
      }

      try {
        const newId = typedBridge.duplicateEntity(id);
        syncEntitiesFromWasm();
        return newId;
      } catch (err) {
        console.error("Failed to duplicate entity in WASM:", err);
        throw err;
      }
    },
    [typedBridge, syncEntitiesFromWasm],
  );

  const updateEntity = useCallback(
    (id: EntityId, updates: Partial<EntityData>) => {
      if (!typedBridge) {
        requireWasmBridge();
      }

      if (updates.position) {
        typedBridge.setPosition(id, updates.position.x, updates.position.y);
      }
      if (updates.size) {
        typedBridge.setSize(id, updates.size.w, updates.size.h);
      }
      if (updates.color) {
        const hex = updates.color.replace("#", "");
        const r = parseInt(hex.substring(0, 2), 16);
        const g = parseInt(hex.substring(2, 4), 16);
        const b = parseInt(hex.substring(4, 6), 16);
        typedBridge.setColor(id, r, g, b, 255);
      }
      if (updates.shape !== undefined) {
        typedBridge.setShape(id, updates.shape);
      }
      if (updates.label !== undefined) {
        typedBridge.setLabel(id, updates.label);
      }
      syncEntitiesFromWasm();
    },
    [typedBridge, syncEntitiesFromWasm],
  );

  const updateProperty = useCallback(
    <T = unknown>(id: EntityId, key: string, value: T) => {
      if (!typedBridge) {
        requireWasmBridge();
      }

      if (key === "label" && typeof value === "string") {
        typedBridge.setLabel(id, value);
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
    [typedBridge, entities],
  );

  const getEntity = useCallback(
    (id: EntityId): EntityData | null => {
      if (!typedBridge) {
        requireWasmBridge();
      }

      try {
        const position = typedBridge.getEntityPositionScreen(id);
        const size = typedBridge.getEntitySizeScreen(id);
        const color = typedBridge.getEntityColorHex(id);
        const shape = typedBridge.getEntityShape(id);
        const label = typedBridge.getEntityLabel(id);
        const isVisible = typedBridge.isEntityVisible(id);
        const isSelected = typedBridge.isEntitySelected(id);

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
    [typedBridge],
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

// Deprecated mock version - kept for backward compatibility
// Use useEntityStore() above instead
export function useEntityStoreMock(_bridge: unknown = null): EntityStoreReturn {
  const [entities, setEntities] = useState<Map<EntityId, EntityData>>(
    new Map(),
  );
  const [entityCount, setEntityCount] = useState(0);

  const spawnEntity = useCallback(
    (
      x: number,
      y: number,
      width = DEFAULT_WIDTH,
      height = DEFAULT_HEIGHT,
    ): EntityId => {
      // Generate a new entity ID
      const newId = Date.now() + Math.floor(Math.random() * 1000);

      const newEntity: EntityData = {
        id: newId,
        position: { x, y },
        size: { w: width, h: height },
        color: "#1a2c32",
        shape: 0,
        label: `Entity ${newId}`,
        isVisible: true,
        isSelected: false,
      };

      setEntities((prev) => new Map(prev).set(newId, newEntity));
      setEntityCount((prev) => prev + 1);

      return newId;
    },
    [],
  );

  const deleteEntity = useCallback((id: EntityId) => {
    setEntities((prev) => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
    setEntityCount((prev) => Math.max(0, prev - 1));
  }, []);

  const duplicateEntity = useCallback(
    (id: EntityId): EntityId | null => {
      const entity = entities.get(id);
      if (!entity) return null;

      const newId = Date.now() + Math.floor(Math.random() * 1000);
      const newEntity: EntityData = {
        ...entity,
        id: newId,
        position: { x: entity.position.x + 20, y: entity.position.y + 20 },
        label: `${entity.label} (copy)`,
      };

      setEntities((prev) => new Map(prev).set(newId, newEntity));
      setEntityCount((prev) => prev + 1);

      return newId;
    },
    [entities],
  );

  const updateEntity = useCallback(
    (id: EntityId, updates: Partial<EntityData>) => {
      setEntities((prev) => {
        const entity = prev.get(id);
        if (!entity) return prev;

        const next = new Map(prev);
        next.set(id, { ...entity, ...updates });
        return next;
      });
    },
    [],
  );

  const updateProperty = useCallback(
    <T = unknown>(id: EntityId, key: string, value: T) => {
      setEntities((prev) => {
        const entity = prev.get(id);
        if (!entity) return prev;

        const next = new Map(prev);
        const currentProperties = entity.properties || {};
        next.set(id, {
          ...entity,
          properties: { ...currentProperties, [key]: value },
        });
        return next;
      });
    },
    [],
  );

  const getEntity = useCallback(
    (id: EntityId): EntityData | null => {
      return entities.get(id) || null;
    },
    [entities],
  );

  const refreshEntities = useCallback(() => {
    // Force re-render by updating count
    setEntityCount((prev) => prev);
  }, []);

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
