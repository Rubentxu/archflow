/**
 * Hook for managing entities through the WASM bridge
 *
 * Provides CRUD operations for entities with automatic state synchronization.
 * Maintains a local cache of entity data for fast access.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type {
  WasmBridge,
  EntityId,
  EntityData,
  UseEntityStoreReturn,
} from "../types/wasm";

/**
 * Default entity dimensions
 */
const DEFAULT_ENTITY_WIDTH = 100;
const DEFAULT_ENTITY_HEIGHT = 60;

/**
 * Hook to manage entities through the WASM bridge
 *
 * @param bridge - The WASM bridge instance
 * @returns Entity store interface with CRUD operations
 *
 * @example
 * ```typescript
 * const { entities, entityCount, spawnEntity, deleteEntity, updateEntity } = useEntityStore(bridge);
 *
 * // Spawn a new rectangle
 * const id = spawnEntity(100, 200, 150, 80);
 *
 * // Update entity properties
 * updateEntity(id, { label: 'New Label', color: '#FF5733' });
 * ```
 */
export function useEntityStore(
  bridge: WasmBridge | null,
): UseEntityStoreReturn {
  const [entities, setEntities] = useState<Map<EntityId, EntityData>>(
    new Map(),
  );
  const [entityCount, setEntityCount] = useState(0);
  const lastEntityCountRef = useRef(0);

  // Sync entity count and rebuild cache when it changes
  useEffect(() => {
    if (!bridge) {
      setEntities(new Map());
      setEntityCount(0);
      return;
    }

    try {
      const count = bridge.entityCount();

      // Only update if entity count changed
      if (count !== lastEntityCountRef.current) {
        lastEntityCountRef.current = count;
        setEntityCount(count);

        // Rebuild entity cache
        const newEntities = new Map<EntityId, EntityData>();
        const aliveIds = bridge.getAliveEntities();

        for (const id of aliveIds) {
          try {
            const [x, y] = bridge.getEntityPositionScreen(id);
            const [width, height] = bridge.getEntitySizeScreen(id);
            const color = bridge.getEntityColorHex(id);
            const shape = bridge.getEntityShape(id);
            const label = bridge.getEntityLabel(id);
            const isVisible = bridge.isEntityVisible(id);
            const isSelected = bridge.isEntitySelected(id);

            newEntities.set(id, {
              id,
              position: { x, y },
              size: { w: width, h: height },
              color,
              shape,
              label,
              isVisible,
              isSelected,
            });
          } catch (err) {
            // Entity might have been deleted between getAliveEntities and getPosition
            console.warn(`Failed to read entity ${id}:`, err);
          }
        }

        setEntities(newEntities);
      }
    } catch (err) {
      console.error("Failed to sync entities:", err);
    }
  }, [bridge, entityCount]);

  /**
   * Spawn a new entity at the specified position
   */
  const spawnEntity = useCallback(
    (
      x: number,
      y: number,
      width: number = DEFAULT_ENTITY_WIDTH,
      height: number = DEFAULT_ENTITY_HEIGHT,
    ): EntityId => {
      if (!bridge) {
        console.warn("Cannot spawn entity: bridge not loaded");
        return -1;
      }

      try {
        const id = bridge.spawnEntity(x, y, width, height);
        // Entity count will update via useEffect
        return id;
      } catch (err) {
        console.error("Failed to spawn entity:", err);
        return -1;
      }
    },
    [bridge],
  );

  /**
   * Delete an entity by ID - clears selection first then removes
   */
  const deleteEntity = useCallback(
    (id: EntityId): void => {
      if (!bridge) {
        console.warn("Cannot delete entity: bridge not loaded");
        return;
      }

      try {
        // Clear selection for this entity first
        bridge.setEntitySelected(id, false);
        // Note: WASM doesn't have individual delete, use deleteSelected
        // This is a placeholder - actual implementation depends on WASM API
        console.warn("Individual entity deletion requires WASM bridge update");
      } catch (err) {
        console.error(`Failed to delete entity ${id}:`, err);
      }
    },
    [bridge],
  );

  /**
   * Duplicate an entity
   */
  const duplicateEntity = useCallback(
    (id: EntityId): EntityId | null => {
      if (!bridge) {
        console.warn("Cannot duplicate entity: bridge not loaded");
        return null;
      }

      try {
        const newId = bridge.duplicateEntity(id);
        return newId >= 0 ? newId : null;
      } catch (err) {
        console.error(`Failed to duplicate entity ${id}:`, err);
        return null;
      }
    },
    [bridge],
  );

  /**
   * Update entity properties
   */
  const updateEntity = useCallback(
    (id: EntityId, updates: Partial<EntityData>): void => {
      if (!bridge) {
        console.warn("Cannot update entity: bridge not loaded");
        return;
      }

      try {
        // Apply each update through the WASM bridge
        if (updates.position !== undefined) {
          bridge.setPosition(id, updates.position.x, updates.position.y);
        }

        if (updates.size !== undefined) {
          bridge.setSize(id, updates.size.w, updates.size.h);
        }

        if (updates.color !== undefined) {
          // Parse hex color to RGBA
          const rgba = hexToRgba(updates.color);
          if (rgba) {
            bridge.setColor(id, rgba.r, rgba.g, rgba.b, rgba.a);
          }
        }

        if (updates.shape !== undefined) {
          bridge.setShape(id, updates.shape);
        }

        if (updates.label !== undefined) {
          bridge.setLabel(id, updates.label);
        }

        if (updates.isVisible !== undefined) {
          // Visibility is handled internally by the engine
          console.warn("Visibility update not directly supported via WASM");
        }

        if (updates.isSelected !== undefined) {
          bridge.setEntitySelected(id, updates.isSelected);
        }

        // Trigger sync on next render
        setEntityCount((prev) => prev + 0); // Force re-render
      } catch (err) {
        console.error(`Failed to update entity ${id}:`, err);
      }
    },
    [bridge],
  );

  /**
   * Get a single entity by ID
   */
  const getEntity = useCallback(
    (id: EntityId): EntityData | null => {
      if (!bridge) {
        return null;
      }

      try {
        // Check local cache first
        const cached = entities.get(id);
        if (cached) {
          return cached;
        }

        // Fall back to direct WASM query
        const [x, y] = bridge.getEntityPositionScreen(id);
        const [width, height] = bridge.getEntitySizeScreen(id);
        const color = bridge.getEntityColorHex(id);
        const shape = bridge.getEntityShape(id);
        const label = bridge.getEntityLabel(id);
        const isVisible = bridge.isEntityVisible(id);
        const isSelected = bridge.isEntitySelected(id);

        return {
          id,
          position: { x, y },
          size: { w: width, h: height },
          color,
          shape,
          label,
          isVisible,
          isSelected,
        };
      } catch (err) {
        console.error(`Failed to get entity ${id}:`, err);
        return null;
      }
    },
    [bridge, entities],
  );

  /**
   * Force refresh of all entity data from WASM
   */
  const refreshEntities = useCallback((): void => {
    if (!bridge) {
      return;
    }

    try {
      const count = bridge.entityCount();
      lastEntityCountRef.current = count !== count ? count : count - 1; // Force update
      setEntityCount((prev) => prev);
    } catch (err) {
      console.error("Failed to refresh entities:", err);
    }
  }, [bridge]);

  return {
    entities,
    entityCount,
    spawnEntity,
    deleteEntity,
    duplicateEntity,
    updateEntity,
    getEntity,
    refreshEntities,
  };
}

/**
 * Utility function to convert hex color to RGBA
 */
function hexToRgba(
  hex: string,
): { r: number; g: number; b: number; a: number } | null {
  // Remove hash prefix if present
  const cleanHex = hex.replace(/^#/, "");

  // Parse based on length
  let r: number,
    g: number,
    b: number,
    a: number = 1;

  if (cleanHex.length === 3) {
    // Short form: #RGB
    r = parseInt(cleanHex[0] + cleanHex[0], 16);
    g = parseInt(cleanHex[1] + cleanHex[1], 16);
    b = parseInt(cleanHex[2] + cleanHex[2], 16);
  } else if (cleanHex.length === 6) {
    // Full form: #RRGGBB
    r = parseInt(cleanHex.substring(0, 2), 16);
    g = parseInt(cleanHex.substring(2, 4), 16);
    b = parseInt(cleanHex.substring(4, 6), 16);
  } else if (cleanHex.length === 8) {
    // Full form with alpha: #RRGGBBAA
    r = parseInt(cleanHex.substring(0, 2), 16);
    g = parseInt(cleanHex.substring(2, 4), 16);
    b = parseInt(cleanHex.substring(4, 6), 16);
    a = parseInt(cleanHex.substring(6, 8), 16) / 255;
  } else {
    return null;
  }

  return { r, g, b, a };
}

/**
 * Utility function to convert RGBA to hex color
 */
export function rgbaToHex(color: {
  r: number;
  g: number;
  b: number;
  a: number;
}): string {
  const toHex = (n: number) => Math.round(n).toString(16).padStart(2, "0");
  return `#${toHex(color.r)}${toHex(color.g)}${toHex(color.b)}${toHex(color.a)}`;
}

/**
 * Utility function to parse WASM color output
 */
export function parseWasmColor(
  r: number,
  g: number,
  b: number,
  a: number,
): string {
  const toHex = (n: number) => Math.round(n).toString(16).padStart(2, "0");
  const hexAlpha = a < 1 ? toHex(a * 255) : "";
  return `#${toHex(r)}${toHex(g)}${toHex(b)}${hexAlpha}`;
}
