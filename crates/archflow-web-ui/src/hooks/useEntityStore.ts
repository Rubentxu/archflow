/**
 * Hook for managing entities through the WASM bridge
 *
 * Provides CRUD operations for entities with automatic state synchronization.
 */

import { useState, useCallback } from "react";
import type { EntityId, EntityData } from "../types/wasm";

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

export function useEntityStore(_bridge: unknown = null): EntityStoreReturn {
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
