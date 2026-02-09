/**
 * useBehaviorSystem - Integration Layer for Behavior System
 *
 * Provides a unified interface between:
 * - WASM event system (mouse/keyboard events)
 * - Behavior system (Sensor→Controller→Actuator pattern)
 * - React stores (Zustand for selection, UI state)
 *
 * This hook creates and manages behaviors for all interactive entities
 * while maintaining compatibility with existing stores.
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * USAGE EXAMPLE
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * import { useBehaviorSystem } from './hooks/useBehaviorSystem';
 *
 * function CanvasWithBehaviors() {
 *   const { behaviors, events, hoverState, selectionState } = useBehaviorSystem({
 *     defaultBehaviors: ['hover', 'select', 'drag'],
 *     onSelectionChange: (ids) => console.log('Selected:', ids),
 *     onHoverChange: (id) => console.log('Hovered:', id),
 *   });
 *
 *   return <CanvasComponent behaviors={behaviors} />;
 * }
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 */

import { useEffect, useRef, useCallback, useState, useMemo } from "react";
import { useArchFlowWasm } from "./useArchFlowWasm.tsx";
import { useSelectionStore } from "../store/useSelectionStore";
import { useUIStore } from "../store/useUIStore";
import { useCanvasStore } from "../store/useCanvasStore";
import {
  BehaviorBuilder,
  type BehaviorBridge,
  type BehaviorConfig,
} from "../sdk/BehaviorBuilder";
import { behaviorTemplates } from "../sdk/BehaviorTemplates";
import type { EntityId, Vec2, EntityData } from "../types/wasm";

// ============================================================================
// TYPES
// ============================================================================

/**
 * Configuration for useBehaviorSystem
 */
export interface UseBehaviorSystemOptions {
  /** Default behaviors to enable for all entities */
  defaultBehaviors?: Array<
    "hover" | "select" | "drag" | "resize" | "rotate" | "delete" | "interactive"
  >;

  /** Snap configuration for drag */
  dragSnap?: number;

  /** Callback when selection changes */
  onSelectionChange?: (ids: EntityId[]) => void;

  /** Callback when hover changes */
  onHoverChange?: (entityId: EntityId | null) => void;

  /** Callback when entity is clicked */
  onEntityClick?: (entityId: EntityId) => void;

  /** Callback when entity drag starts */
  onDragStart?: (entityId: EntityId) => void;

  /** Callback when entity is dragged */
  onDrag?: (entityId: EntityId, delta: Vec2) => void;

  /** Callback when entity drag ends */
  onDragEnd?: (entityId: EntityId) => void;

  /** Enable debug logging */
  debug?: boolean;
}

/**
 * State managed by the behavior system
 */
export interface BehaviorSystemState {
  /** Currently hovered entity ID */
  hoveredEntityId: EntityId | null;

  /** Currently selected entity IDs */
  selectedEntityIds: EntityId[];

  /** Currently dragged entity ID */
  draggedEntityId: EntityId | null;

  /** Drag start position in world coordinates */
  dragStartPosition: Vec2 | null;

  /** Current drag position in world coordinates */
  currentDragPosition: Vec2 | null;

  /** Whether marquee selection is active */
  isMarqueeing: boolean;

  /** Marquee selection rectangle */
  marqueeRect: { x: number; y: number; width: number; height: number } | null;
}

/**
 * Return type for useBehaviorSystem
 */
export interface UseBehaviorSystemReturn {
  /** Behavior system state */
  state: BehaviorSystemState;

  /** Pre-configured behavior templates */
  templates: {
    /** Hover behavior */
    hoverable: (entityId: EntityId) => BehaviorBridge;

    /** Selectable behavior */
    selectable: (entityId: EntityId) => BehaviorBridge;

    /** Draggable behavior */
    draggable: (entityId: EntityId) => BehaviorBridge;

    /** Interactive behavior (hover + select + drag) */
    interactive: (entityId: EntityId) => BehaviorBridge;

    /** Deletable behavior */
    deletable: (entityId: EntityId) => BehaviorBridge;

    /** Resizable behavior */
    resizable: (entityId: EntityId) => BehaviorBridge;

    /** Rotatable behavior */
    rotatable: (entityId: EntityId) => BehaviorBridge;
  };

  /** Actions */
  actions: {
    /** Select an entity */
    select: (entityId: EntityId, additive?: boolean) => void;

    /** Deselect an entity */
    deselect: (entityId: EntityId) => void;

    /** Toggle entity selection */
    toggle: (entityId: EntityId) => void;

    /** Clear all selection */
    clearSelection: () => void;

    /** Set hovered entity */
    setHovered: (entityId: EntityId | null) => void;

    /** Start dragging an entity */
    startDrag: (entityId: EntityId, position: Vec2) => void;

    /** Update current drag position */
    updateDrag: (position: Vec2) => void;

    /** End drag operation */
    endDrag: () => void;

    /** Start marquee selection */
    startMarquee: (startPos: Vec2) => void;

    /** Update marquee selection */
    updateMarquee: (currentPos: Vec2) => void;

    /** End marquee selection */
    endMarquee: () => void;

    /** Delete selected entities */
    deleteSelected: () => void;

    /** Duplicate selected entities */
    duplicateSelected: () => EntityId[];

    /** Update behavior system (called each frame) */
    update: (timestamp: number) => void;

    /** Attach behavior to entity */
    attachBehavior: (entityId: EntityId, behavior: BehaviorBridge) => void;

    /** Detach behavior from entity */
    detachBehavior: (entityId: EntityId, behavior: BehaviorBridge) => void;
  };

  /** Event handlers for canvas */
  handlers: {
    /** Pointer down handler */
    onPointerDown: (position: Vec2, button: number, modifiers: number) => void;

    /** Pointer move handler */
    onPointerMove: (position: Vec2, buttons: number, modifiers: number) => void;

    /** Pointer up handler */
    onPointerUp: (position: Vec2, button: number, modifiers: number) => void;

    /** Key down handler */
    onKeyDown: (key: string, modifiers: number) => void;
  };
}

// ============================================================================
// HOOK IMPLEMENTATION
// ============================================================================

/**
 * Hook for integrating behavior system with WASM and stores
 */
export function useBehaviorSystem(
  options: UseBehaviorSystemOptions = {},
): UseBehaviorSystemReturn {
  const {
    defaultBehaviors = ["hover", "select", "drag"],
    dragSnap = 8,
    onSelectionChange,
    onHoverChange,
    onEntityClick,
    onDragStart,
    onDrag,
    onDragEnd,
    debug = false,
  } = options;

  // WASM access
  const { bridge, isLoaded, logicSystem } = useArchFlowWasm();

  // Store access
  const selectionStore = useSelectionStore();
  const uiStore = useUIStore();
  const canvasStore = useCanvasStore();

  // State
  const [state, setState] = useState<BehaviorSystemState>({
    hoveredEntityId: null,
    selectedEntityIds: [],
    draggedEntityId: null,
    dragStartPosition: null,
    currentDragPosition: null,
    isMarqueeing: false,
    marqueeRect: null,
  });

  // Refs for callbacks that need current state
  const stateRef = useRef(state);
  stateRef.current = state;

  // Behavior registry: entityId -> Set<BehaviorBridge>
  const behaviorRegistry = useRef<Map<EntityId, Set<BehaviorBridge>>>(
    new Map(),
  );

  // Cache for pre-built behaviors
  const behaviorCache = useRef<Map<string, BehaviorBridge>>(new Map());

  // Debug logging helper
  const log = useCallback(
    (...args: unknown[]) => {
      if (debug) {
        console.log("[BehaviorSystem]", ...args);
      }
    },
    [debug],
  );

  // ==========================================================================
  // BEHAVIOR TEMPLATE FACTORIES
  // ==========================================================================

  /**
   * Create hover behavior for entity
   */
  const createHoverable = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      const cacheKey = `hover:${entityId}`;
      const cached = behaviorCache.current.get(cacheKey);
      if (cached) return cached;

      const behavior = behaviorTemplates.hoverable(logicSystem, {
        color: 0x2196f3,
        opacity: 0.15,
      });

      // Add hover change handler
      behavior.on("hover:start", () => {
        setState((prev) => {
          if (prev.hoveredEntityId !== entityId) {
            onHoverChange?.(entityId);
            return { ...prev, hoveredEntityId: entityId };
          }
          return prev;
        });
      });

      behavior.on("hover:end", () => {
        setState((prev) => {
          if (prev.hoveredEntityId === entityId) {
            onHoverChange?.(null);
            return { ...prev, hoveredEntityId: null };
          }
          return prev;
        });
      });

      behaviorCache.current.set(cacheKey, behavior);
      return behavior;
    },
    [logicSystem, onHoverChange],
  );

  /**
   * Create selectable behavior for entity
   */
  const createSelectable = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      const cacheKey = `select:${entityId}`;
      const cached = behaviorCache.current.get(cacheKey);
      if (cached) return cached;

      const behavior = behaviorTemplates.selectable(logicSystem, {
        mode: "single",
      });

      behavior.on("select", () => {
        setState((prev) => {
          const newSelection = [...prev.selectedEntityIds, entityId];
          onSelectionChange?.(newSelection);
          selectionStore.setSelectedIds(newSelection);
          return { ...prev, selectedEntityIds: newSelection };
        });
      });

      behavior.on("deselect", () => {
        setState((prev) => {
          const newSelection = prev.selectedEntityIds.filter(
            (id) => id !== entityId,
          );
          onSelectionChange?.(newSelection);
          selectionStore.setSelectedIds(newSelection);
          return { ...prev, selectedEntityIds: newSelection };
        });
      });

      behavior.on("click", () => {
        onEntityClick?.(entityId);
      });

      behaviorCache.current.set(cacheKey, behavior);
      return behavior;
    },
    [logicSystem, onSelectionChange, onEntityClick, selectionStore],
  );

  /**
   * Create draggable behavior for entity
   */
  const createDraggable = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      const cacheKey = `drag:${entityId}`;
      const cached = behaviorCache.current.get(cacheKey);
      if (cached) return cached;

      const behavior = behaviorTemplates.draggable(logicSystem, {
        axis: "both",
        snap: dragSnap,
      });

      behavior.on("drag:start", (event: any) => {
        const position = event.point || event.data?.position;
        setState((prev) => ({
          ...prev,
          draggedEntityId: entityId,
          dragStartPosition: position,
          currentDragPosition: position,
        }));
        onDragStart?.(entityId);
      });

      behavior.on("drag", (event: any) => {
        const position = event.point || event.data?.position;
        const delta = event.delta || event.data?.delta;

        setState((prev) => ({
          ...prev,
          currentDragPosition: position,
        }));

        onDrag?.(entityId, delta);
      });

      behavior.on("drag:end", () => {
        setState((prev) => ({
          ...prev,
          draggedEntityId: null,
          dragStartPosition: null,
          currentDragPosition: null,
        }));
        onDragEnd?.(entityId);
      });

      behaviorCache.current.set(cacheKey, behavior);
      return behavior;
    },
    [logicSystem, onDragStart, onDrag, onDragEnd],
  );

  /**
   * Create interactive behavior (all-in-one)
   */
  const createInteractive = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      const cacheKey = `interactive:${entityId}`;
      const cached = behaviorCache.current.get(cacheKey);
      if (cached) return cached;

      const behavior = behaviorTemplates.interactive(logicSystem);

      // Connect to state changes
      behavior.on("hover:start", () => {
        setState((prev) => {
          if (prev.hoveredEntityId !== entityId) {
            onHoverChange?.(entityId);
            return { ...prev, hoveredEntityId: entityId };
          }
          return prev;
        });
      });

      behavior.on("hover:end", () => {
        setState((prev) => {
          if (prev.hoveredEntityId === entityId) {
            onHoverChange?.(null);
            return { ...prev, hoveredEntityId: null };
          }
          return prev;
        });
      });

      behavior.on("select", () => {
        setState((prev) => {
          const newSelection = [...prev.selectedEntityIds, entityId];
          onSelectionChange?.(newSelection);
          selectionStore.setSelectedIds(newSelection);
          return { ...prev, selectedEntityIds: newSelection };
        });
      });

      behavior.on("drag:start", (event: any) => {
        const position = event.point || event.data?.position;
        setState((prev) => ({
          ...prev,
          draggedEntityId: entityId,
          dragStartPosition: position,
          currentDragPosition: position,
        }));
        onDragStart?.(entityId);
      });

      behavior.on("drag", (event: any) => {
        const delta = event.delta || event.data?.delta;
        onDrag?.(entityId, delta);
      });

      behavior.on("drag:end", () => {
        setState((prev) => ({
          ...prev,
          draggedEntityId: null,
          dragStartPosition: null,
          currentDragPosition: null,
        }));
        onDragEnd?.(entityId);
      });

      behaviorCache.current.set(cacheKey, behavior);
      return behavior;
    },
    [
      logicSystem,
      onHoverChange,
      onSelectionChange,
      onDragStart,
      onDrag,
      onDragEnd,
      selectionStore,
    ],
  );

  /**
   * Create deletable behavior
   */
  const createDeletable = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      const cacheKey = `delete:${entityId}`;
      const cached = behaviorCache.current.get(cacheKey);
      if (cached) return cached;

      const behavior = behaviorTemplates.deletable(logicSystem);

      behavior.on("delete", () => {
        // Find and remove from selection
        setState((prev) => {
          const newSelection = prev.selectedEntityIds.filter(
            (id) => id !== entityId,
          );
          onSelectionChange?.(newSelection);
          selectionStore.setSelectedIds(newSelection);
          return { ...prev, selectedEntityIds: newSelection };
        });
      });

      behaviorCache.current.set(cacheKey, behavior);
      return behavior;
    },
    [logicSystem, onSelectionChange, selectionStore],
  );

  /**
   * Create resizable behavior
   */
  const createResizable = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      return behaviorTemplates.resizable(logicSystem, { snap: dragSnap });
    },
    [logicSystem, dragSnap],
  );

  /**
   * Create rotatable behavior
   */
  const createRotatable = useCallback(
    (entityId: EntityId): BehaviorBridge => {
      return behaviorTemplates.rotatable(logicSystem);
    },
    [logicSystem],
  );

  // ==========================================================================
  // ACTIONS
  // ==========================================================================

  const select = useCallback(
    (entityId: EntityId, additive = false) => {
      log("select", entityId, additive);

      if (!additive) {
        setState((prev) => ({
          ...prev,
          selectedEntityIds: [entityId],
        }));
        selectionStore.setSelectedIds([entityId]);
      } else {
        setState((prev) => {
          if (!prev.selectedEntityIds.includes(entityId)) {
            const newSelection = [...prev.selectedEntityIds, entityId];
            onSelectionChange?.(newSelection);
            selectionStore.setSelectedIds(newSelection);
            return { ...prev, selectedEntityIds: newSelection };
          }
          return prev;
        });
      }

      // Emit select event to behavior
      const behaviors = behaviorRegistry.current.get(entityId);
      behaviors?.forEach((behavior) => {
        behavior.on("select", () => {});
        behavior.on("event", (event: any) => {
          if (event.type === "select") {
            behavior.on("select", () => {});
          }
        });
      });
    },
    [log, onSelectionChange, selectionStore],
  );

  const deselect = useCallback(
    (entityId: EntityId) => {
      log("deselect", entityId);

      setState((prev) => {
        const newSelection = prev.selectedEntityIds.filter(
          (id) => id !== entityId,
        );
        onSelectionChange?.(newSelection);
        selectionStore.setSelectedIds(newSelection);
        return { ...prev, selectedEntityIds: newSelection };
      });
    },
    [log, onSelectionChange, selectionStore],
  );

  const toggle = useCallback(
    (entityId: EntityId) => {
      const isSelected = stateRef.current.selectedEntityIds.includes(entityId);
      if (isSelected) {
        deselect(entityId);
      } else {
        select(entityId, true);
      }
    },
    [select, deselect],
  );

  const clearSelection = useCallback(() => {
    log("clearSelection");
    setState((prev) => ({
      ...prev,
      selectedEntityIds: [],
    }));
    onSelectionChange?.([]);
    selectionStore.clear();
  }, [log, onSelectionChange, selectionStore]);

  const setHovered = useCallback(
    (entityId: EntityId | null) => {
      setState((prev) => {
        if (prev.hoveredEntityId !== entityId) {
          onHoverChange?.(entityId);
          return { ...prev, hoveredEntityId: entityId };
        }
        return prev;
      });
    },
    [onHoverChange],
  );

  const startDrag = useCallback(
    (entityId: EntityId, position: Vec2) => {
      log("startDrag", entityId, position);

      setState((prev) => ({
        ...prev,
        draggedEntityId: entityId,
        dragStartPosition: position,
        currentDragPosition: position,
      }));

      onDragStart?.(entityId);
    },
    [log, onDragStart],
  );

  const updateDrag = useCallback(
    (position: Vec2) => {
      setState((prev) => ({
        ...prev,
        currentDragPosition: position,
      }));

      const draggedId = stateRef.current.draggedEntityId;
      if (draggedId) {
        const delta = stateRef.current.dragStartPosition
          ? {
              x: position.x - stateRef.current.dragStartPosition.x,
              y: position.y - stateRef.current.dragStartPosition.y,
            }
          : { x: 0, y: 0 };

        onDrag?.(draggedId, delta);
      }
    },
    [onDrag],
  );

  const endDrag = useCallback(() => {
    const draggedId = stateRef.current.draggedEntityId;
    log("endDrag", draggedId);

    if (draggedId) {
      onDragEnd?.(draggedId);
    }

    setState((prev) => ({
      ...prev,
      draggedEntityId: null,
      dragStartPosition: null,
      currentDragPosition: null,
    }));
  }, [log, onDragEnd]);

  const startMarquee = useCallback(
    (startPos: Vec2) => {
      log("startMarquee", startPos);
      setState((prev) => ({
        ...prev,
        isMarqueeing: true,
        marqueeRect: { x: startPos.x, y: startPos.y, width: 0, height: 0 },
      }));
    },
    [log],
  );

  const updateMarquee = useCallback((currentPos: Vec2) => {
    const prev = stateRef.current;
    if (!prev.isMarqueeing || !prev.marqueeRect) return;

    const startPos = {
      x: prev.marqueeRect.x,
      y: prev.marqueeRect.y,
    };

    const x = Math.min(startPos.x, currentPos.x);
    const y = Math.min(startPos.y, currentPos.y);
    const width = Math.abs(currentPos.x - startPos.x);
    const height = Math.abs(currentPos.y - startPos.y);

    setState((prevState) => ({
      ...prevState,
      marqueeRect: { x, y, width, height },
    }));
  }, []);

  const endMarquee = useCallback(() => {
    const prev = stateRef.current;
    log("endMarquee", prev.marqueeRect);

    if (
      prev.marqueeRect &&
      prev.marqueeRect.width > 5 &&
      prev.marqueeRect.height > 5
    ) {
      // Find entities within marquee and select them
      // This would need to query the WASM entity store
      // For now, we just clear the marquee state
    }

    setState((prev) => ({
      ...prev,
      isMarqueeing: false,
      marqueeRect: null,
    }));
  }, [log]);

  /**
   * Update behavior system - called each frame
   */
  const update = useCallback((timestamp: number) => {
    // Update all registered behaviors
    behaviorRegistry.current.forEach((behaviors) => {
      behaviors.forEach((behavior) => {
        behavior.update(timestamp);
      });
    });
  }, []);

  const deleteSelected = useCallback(() => {
    const selectedIds = stateRef.current.selectedEntityIds;
    log("deleteSelected", selectedIds);

    selectedIds.forEach((entityId) => {
      const behaviors = behaviorRegistry.current.get(entityId);
      behaviors?.forEach((behavior) => {
        behavior.on("delete", () => {});
        behavior.on("event", (event: any) => {
          if (event.type === "delete") {
            // Actual deletion would happen here
          }
        });
      });
    });

    clearSelection();
  }, [log, clearSelection]);

  const duplicateSelected = useCallback((): EntityId[] => {
    const selectedIds = stateRef.current.selectedEntityIds;
    log("duplicateSelected", selectedIds);
    // Would need WASM bridge for actual duplication
    return [];
  }, [log]);

  const attachBehavior = useCallback(
    (entityId: EntityId, behavior: BehaviorBridge) => {
      log("attachBehavior", entityId);

      if (!behaviorRegistry.current.has(entityId)) {
        behaviorRegistry.current.set(entityId, new Set());
      }
      behaviorRegistry.current.get(entityId)?.add(behavior);

      behavior.attach(String(entityId));
    },
    [log],
  );

  const detachBehavior = useCallback(
    (entityId: EntityId, behavior: BehaviorBridge) => {
      log("detachBehavior", entityId);

      behaviorRegistry.current.get(entityId)?.delete(behavior);
      behavior.detach(String(entityId));
    },
    [log],
  );

  // ==========================================================================
  // EVENT HANDLERS
  // ==========================================================================

  const onPointerDown = useCallback(
    (position: Vec2, button: number, modifiers: number) => {
      log("pointerDown", position, button, modifiers);

      // Check for marquee selection (left button + shift or middle mouse)
      if (button === 0 && modifiers & 0x01) {
        startMarquee(position);
        return;
      }

      // Check if clicking on an entity
      // This would need to query WASM for entity at position
      const entityId = bridge?.getEntityAtScreenPoint?.(position.x, position.y);

      if (entityId !== undefined && entityId >= 0) {
        const additive = modifiers & 0x02; // Ctrl key
        select(entityId, additive);
        startDrag(entityId, position);
      }
    },
    [log, bridge, startMarquee, select, startDrag],
  );

  const onPointerMove = useCallback(
    (position: Vec2, buttons: number, modifiers: number) => {
      // Handle marquee
      if (stateRef.current.isMarqueeing) {
        updateMarquee(position);
        return;
      }

      // Handle drag
      if (stateRef.current.draggedEntityId && buttons === 1) {
        updateDrag(position);
        return;
      }

      // Handle hover
      const entityId = bridge?.getEntityAtScreenPoint?.(position.x, position.y);
      const newHoverId =
        entityId !== undefined && entityId >= 0 ? entityId : null;

      if (stateRef.current.hoveredEntityId !== newHoverId) {
        setHovered(newHoverId);
      }
    },
    [log, bridge, updateMarquee, updateDrag, setHovered],
  );

  const onPointerUp = useCallback(
    (position: Vec2, button: number, modifiers: number) => {
      log("pointerUp", position, button, modifiers);

      if (stateRef.current.isMarqueeing) {
        endMarquee();
        return;
      }

      if (stateRef.current.draggedEntityId) {
        endDrag();
        return;
      }
    },
    [log, endMarquee, endDrag],
  );

  const onKeyDown = useCallback(
    (key: string, modifiers: number) => {
      log("keyDown", key, modifiers);

      // Delete key
      if (key === "Delete" || key === "Backspace") {
        deleteSelected();
      }

      // Escape - clear selection
      if (key === "Escape") {
        clearSelection();
      }

      // Ctrl+A - select all
      if (key === "a" && modifiers & 0x02) {
        // Select all would need WASM query
      }
    },
    [log, deleteSelected, clearSelection],
  );

  // ==========================================================================
  // INITIALIZE DEFAULT BEHAVIORS
  // ==========================================================================

  useEffect(() => {
    if (!isLoaded || !logicSystem) return;

    log("Initializing with default behaviors:", defaultBehaviors);

    // Create behaviors for each alive entity
    // This is a simplified version - full implementation would query WASM
  }, [isLoaded, logicSystem, defaultBehaviors, log]);

  // ==========================================================================
  // RETURN
  // ==========================================================================

  return {
    state,

    templates: {
      hoverable: createHoverable,
      selectable: createSelectable,
      draggable: createDraggable,
      interactive: createInteractive,
      deletable: createDeletable,
      resizable: createResizable,
      rotatable: createRotatable,
    },

    actions: {
      select,
      deselect,
      toggle,
      clearSelection,
      setHovered,
      startDrag,
      updateDrag,
      endDrag,
      startMarquee,
      updateMarquee,
      endMarquee,
      deleteSelected,
      duplicateSelected,
      update,
      attachBehavior,
      detachBehavior,
    },

    handlers: {
      onPointerDown,
      onPointerMove,
      onPointerUp,
      onKeyDown,
    },
  };
}

// ============================================================================
// CONVENIENCE HOOKS
// ============================================================================

/**
 * Hook for entity-specific behaviors
 *
 * @param entityId - Entity to manage behaviors for
 * @param behaviorTypes - Types of behaviors to enable
 * @returns Behavior control methods
 */
export function useEntityBehaviors(
  entityId: EntityId,
  behaviorTypes: Array<
    "hover" | "select" | "drag" | "delete" | "resize" | "rotate"
  > = ["hover", "select", "drag"],
) {
  const behaviorSystem = useBehaviorSystem({ debug: false });

  const { templates, actions, attachBehavior, detachBehavior } = behaviorSystem;

  // Memoize entity-specific behaviors
  const behaviors = useMemo(() => {
    const result: BehaviorBridge[] = [];

    if (behaviorTypes.includes("hover")) {
      result.push(templates.hoverable(entityId));
    }
    if (behaviorTypes.includes("select")) {
      result.push(templates.selectable(entityId));
    }
    if (behaviorTypes.includes("drag")) {
      result.push(templates.draggable(entityId));
    }
    if (behaviorTypes.includes("delete")) {
      result.push(templates.deletable(entityId));
    }
    if (behaviorTypes.includes("resize")) {
      result.push(templates.resizable(entityId));
    }
    if (behaviorTypes.includes("rotate")) {
      result.push(templates.rotatable(entityId));
    }

    return result;
  }, [entityId, behaviorTypes, templates]);

  // Attach behaviors on mount
  useEffect(() => {
    behaviors.forEach((behavior) => {
      attachBehavior(entityId, behavior);
    });

    return () => {
      behaviors.forEach((behavior) => {
        detachBehavior(entityId, behavior);
      });
    };
  }, [entityId, behaviors, attachBehavior, detachBehavior]);

  return {
    behaviors,
    ...actions,
  };
}

/**
 * Hook for canvas-level interactions (marquee, pan, zoom)
 */
export function useCanvasInteractions(
  options: {
    onMarqueeSelect?: (entityIds: EntityId[]) => void;
    onPan?: (delta: Vec2) => void;
    onZoom?: (factor: number, center: Vec2) => void;
  } = {},
) {
  const behaviorSystem = useBehaviorSystem({ debug: false });
  const canvasStore = useCanvasStore();

  const { state, handlers } = behaviorSystem;

  // Marquee selection callback
  useEffect(() => {
    if (
      state.marqueeRect &&
      state.marqueeRect.width > 5 &&
      state.marqueeRect.height > 5
    ) {
      options.onMarqueeSelect?.([]);
    }
  }, [state.marqueeRect, options]);

  return {
    isMarqueeing: state.isMarqueeing,
    marqueeRect: state.marqueeRect,
    handlers,
  };
}

export default useBehaviorSystem;
