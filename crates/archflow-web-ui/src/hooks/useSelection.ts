/**
 * Hook for managing entity selection
 *
 * Provides selection state management with additive selection,
 * rectangular marquee selection, and keyboard modifier support.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useState, useCallback, useMemo } from "react";
import { useArchFlowWasm } from "./useArchFlowWasm.tsx";
import { useSelectionStore } from "../store/useSelectionStore";
import { getTypedBridge } from "./wasm-bridge";
import type { EntityId, Vec2, EntityData } from "../types/wasm";

interface UseSelectionReturn {
  // State
  selectedIds: EntityId[];
  selectedEntities: EntityData[];
  isSelected: (id: EntityId) => boolean;
  selectionCount: number;
  hasSelection: boolean;
  canSelect: boolean;

  // Single selection
  select: (id: EntityId, additive?: boolean) => void;
  deselect: (id: EntityId) => void;
  toggle: (id: EntityId) => void;

  // Multiple selection
  selectMultiple: (ids: EntityId[], additive?: boolean) => void;
  selectRect: (rect: {
    x: number;
    y: number;
    width: number;
    height: number;
  }) => void;
  selectAll: () => void;

  // Clear
  clearSelection: () => void;

  // Operations on selection
  deleteSelected: () => void;
  duplicateSelected: () => EntityId[];
}

export function useSelection(): UseSelectionReturn {
  const { bridge, isLoaded } = useArchFlowWasm();
  const store = useSelectionStore();

  // Local state for selection
  const [selectionCount, setSelectionCount] = useState(0);

  // Get selected IDs from store
  const selectedIds = store.selectedIds;

  // Check if an entity is selected
  const isSelected = useCallback(
    (id: EntityId) => {
      return selectedIds.includes(id);
    },
    [selectedIds],
  );

  // Check if we have any selection
  const hasSelection = useMemo(() => selectedIds.length > 0, [selectedIds]);

  // Check if selection is allowed
  const canSelect = useMemo(() => isLoaded && !!bridge, [isLoaded, bridge]);

  // Select a single entity
  const select = useCallback(
    (id: EntityId, additive = false) => {
      if (!canSelect) return;

      if (!additive) {
        store.setSelectedIds([id]);
      } else {
        store.addToSelection(id);
      }

      setSelectionCount((prev) => (additive ? prev + 1 : 1));
    },
    [canSelect, store],
  );

  // Deselect an entity
  const deselect = useCallback(
    (id: EntityId) => {
      if (!canSelect) return;

      store.removeFromSelection(id);
      setSelectionCount((prev) => Math.max(0, prev - 1));
    },
    [canSelect, store],
  );

  // Toggle selection
  const toggle = useCallback(
    (id: EntityId) => {
      if (isSelected(id)) {
        deselect(id);
      } else {
        select(id, true);
      }
    },
    [isSelected, deselect, select],
  );

  // Select multiple entities
  const selectMultiple = useCallback(
    (ids: EntityId[], additive = false) => {
      if (!canSelect) return;

      if (!additive) {
        store.setSelectedIds(ids);
      } else {
        ids.forEach((id) => store.addToSelection(id));
      }

      setSelectionCount(ids.length);
    },
    [canSelect, store],
  );

  // Select all entities within a rectangle (marquee selection)
  const selectRect = useCallback(
    (rect: { x: number; y: number; width: number; height: number }) => {
      const typed = getTypedBridge(bridge);
      if (!canSelect || !typed) return;

      try {
        const aliveEntities = typed.getAliveEntities();
        const entitiesInRect: EntityId[] = [];

        for (const id of aliveEntities) {
          try {
            const [ex, ey] = typed.getEntityPositionScreen(id);
            const [ew, eh] = typed.getEntitySizeScreen(id);

            // Check if entity center is within the rectangle
            const centerX = ex + ew / 2;
            const centerY = ey + eh / 2;

            if (
              centerX >= rect.x &&
              centerX <= rect.x + rect.width &&
              centerY >= rect.y &&
              centerY <= rect.y + rect.height
            ) {
              entitiesInRect.push(id);
            }
          } catch {
            // Entity might have been deleted
          }
        }

        store.setSelectedIds(entitiesInRect);
        setSelectionCount(entitiesInRect.length);
      } catch (err) {
        console.error("Marquee selection failed:", err);
      }
    },
    [canSelect, bridge, store],
  );

  // Select all entities
  const selectAll = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!canSelect || !typed) return;

    try {
      const allIds = typed.getAliveEntities();
      store.setSelectedIds(allIds);
      setSelectionCount(allIds.length);
    } catch (err) {
      console.error("Select all failed:", err);
    }
  }, [canSelect, bridge, store]);

  // Clear selection
  const clearSelection = useCallback(() => {
    store.clear();
    setSelectionCount(0);
  }, [store]);

  // Delete selected entities
  const deleteSelected = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!canSelect || !typed || selectedIds.length === 0) return;

    try {
      typed.deleteSelected();
      clearSelection();
    } catch (err) {
      console.error("Delete failed:", err);
    }
  }, [canSelect, bridge, selectedIds, clearSelection]);

  // Duplicate selected entities
  const duplicateSelected = useCallback((): EntityId[] => {
    const typed = getTypedBridge(bridge);
    if (!canSelect || !typed || selectedIds.length === 0) return [];

    const newIds: EntityId[] = [];

    try {
      for (const id of selectedIds) {
        const newId = typed.duplicateEntity(id);
        if (newId && newId >= 0) {
          newIds.push(newId);
        }
      }

      // Select the new entities
      store.setSelectedIds(newIds);
      setSelectionCount(newIds.length);

      return newIds;
    } catch (err) {
      console.error("Duplicate failed:", err);
      return [];
    }
  }, [canSelect, bridge, selectedIds, store]);

  // Selected entities data - fetch from WASM for each selected entity
  const selectedEntities = useMemo((): EntityData[] => {
    if (!bridge || selectedIds.length === 0) return [];

    const typed = getTypedBridge(bridge);
    if (!typed) return [];

    const entities: EntityData[] = [];
    for (const id of selectedIds) {
      try {
        const position = typed.getEntityPositionScreen(id);
        const size = typed.getEntitySizeScreen(id);
        const color = typed.getEntityColorHex(id);
        const shape = typed.getEntityShape(id);
        const label = typed.getEntityLabel(id);
        const isVisible = typed.isEntityVisible(id);
        const isSelected = typed.isEntitySelected(id);

        entities.push({
          id,
          position: { x: position[0], y: position[1] },
          size: { w: size[0], h: size[1] },
          color,
          shape,
          label,
          isVisible,
          isSelected,
        });
      } catch {
        // Entity might have been deleted, skip it
      }
    }
    return entities;
  }, [bridge, selectedIds]);

  return {
    selectedIds,
    selectedEntities,
    isSelected,
    selectionCount,
    hasSelection,
    canSelect,
    select,
    deselect,
    toggle,
    selectMultiple,
    selectRect,
    selectAll,
    clearSelection,
    deleteSelected,
    duplicateSelected,
  };
}

/**
 * Hook for marquee selection interaction
 */
export function useMarqueeSelection() {
  const [isMarqueeing, setIsMarqueeing] = useState(false);
  const [startPoint, setStartPoint] = useState<Vec2 | null>(null);
  const [currentRect, setCurrentRect] = useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);

  const { selectRect, clearSelection } = useSelection();

  const startMarquee = useCallback(
    (point: Vec2) => {
      setStartPoint(point);
      setIsMarqueeing(true);
      clearSelection();
    },
    [clearSelection],
  );

  const updateMarquee = useCallback(
    (currentPoint: Vec2) => {
      if (!startPoint) return;

      const x = Math.min(startPoint.x, currentPoint.x);
      const y = Math.min(startPoint.y, currentPoint.y);
      const width = Math.abs(currentPoint.x - startPoint.x);
      const height = Math.abs(currentPoint.y - startPoint.y);

      setCurrentRect({ x, y, width, height });
    },
    [startPoint],
  );

  const endMarquee = useCallback(() => {
    if (currentRect && currentRect.width > 5 && currentRect.height > 5) {
      selectRect(currentRect);
    }
    setIsMarqueeing(false);
    setStartPoint(null);
    setCurrentRect(null);
  }, [currentRect, selectRect]);

  const cancelMarquee = useCallback(() => {
    setIsMarqueeing(false);
    setStartPoint(null);
    setCurrentRect(null);
  }, []);

  return {
    isMarqueeing,
    startPoint,
    currentRect,
    startMarquee,
    updateMarquee,
    endMarquee,
    cancelMarquee,
  };
}
