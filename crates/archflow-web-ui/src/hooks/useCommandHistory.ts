/**
 * Hook for managing command history (undo/redo)
 *
 * Integrates with the WASM CommandHistory system for
 * persistent undo/redo across sessions.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useState, useCallback, useEffect } from "react";
import { useArchFlowWasm } from "./useArchFlowWasm";
import { getTypedBridge } from "./wasm-bridge";

interface UseCommandHistoryReturn {
  canUndo: boolean;
  canRedo: boolean;
  undo: () => void;
  redo: () => void;
  execute: (command: Command) => void;
  getHistoryState: () => { undoCount: number; redoCount: number };
}

interface Command {
  type: string;
  data: Record<string, unknown>;
  execute: () => void;
  undo: () => void;
}

export function useCommandHistory(): UseCommandHistoryReturn {
  const { bridge, isLoaded } = useArchFlowWasm();
  const [canUndoState, setCanUndo] = useState(false);
  const [canRedoState, setCanRedo] = useState(false);

  // Sync state with WASM bridge
  useEffect(() => {
    if (!bridge || !isLoaded) {
      setCanUndo(false);
      setCanRedo(false);
      return;
    }

    try {
      const typed = getTypedBridge(bridge);
      if (!typed) return;
      setCanUndo(typed.canUndo());
      setCanRedo(typed.canRedo());
    } catch (err) {
      console.warn("Failed to sync command history state:", err);
    }
  }, [bridge, isLoaded]);

  const undo = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!typed || !canUndoState) return;

    try {
      typed.undo();
      setCanUndo(typed.canUndo());
      setCanRedo(typed.canRedo());
    } catch (err) {
      console.error("Undo failed:", err);
    }
  }, [bridge, canUndoState]);

  const redo = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!typed || !canRedoState) return;

    try {
      typed.redo();
      setCanUndo(typed.canUndo());
      setCanRedo(typed.canRedo());
    } catch (err) {
      console.error("Redo failed:", err);
    }
  }, [bridge, canRedoState]);

  const execute = useCallback(
    (command: Command) => {
      const typed = getTypedBridge(bridge);
      if (!typed) return;

      try {
        command.execute();
        // Push command to WASM history
        // In a real implementation, we'd serialize the command
        setCanUndo(typed.canUndo());
        setCanRedo(typed.canRedo());
      } catch (err) {
        console.error("Command execution failed:", err);
      }
    },
    [bridge],
  );

  const getHistoryState = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!typed) return { undoCount: 0, redoCount: 0 };

    try {
      const state = typed.getHistoryState();
      // Parse state format: "undo:N,redo:M"
      const parts = state.split(",");
      return {
        undoCount: parseInt(parts[0]?.split(":")[1] || "0"),
        redoCount: parseInt(parts[1]?.split(":")[1] || "0"),
      };
    } catch {
      return { undoCount: 0, redoCount: 0 };
    }
  }, [bridge]);

  return {
    canUndo: canUndoState,
    canRedo: canRedoState,
    undo,
    redo,
    execute,
    getHistoryState,
  };
}

/**
 * Hook for selected entity operations (duplicate, group, etc.)
 */
export function useSelectionCommands() {
  const { bridge, isLoaded } = useArchFlowWasm();
  const [selectedCount, setSelectedCount] = useState(0);

  const duplicate = useCallback(
    (entityId: number) => {
      const typed = getTypedBridge(bridge);
      if (!typed || !isLoaded) return null;

      try {
        const newId = typed.duplicateEntity(entityId);
        return newId >= 0 ? newId : null;
      } catch (err) {
        console.error("Duplicate failed:", err);
        return null;
      }
    },
    [bridge, isLoaded],
  );

  const deleteSelected = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!typed || !isLoaded) return;

    try {
      typed.deleteSelected();
    } catch (err) {
      console.error("Delete failed:", err);
    }
  }, [bridge, isLoaded]);

  const selectAll = useCallback(() => {
    const typed = getTypedBridge(bridge);
    if (!typed || !isLoaded) return;

    try {
      const entities = typed.getAliveEntities();
      entities.forEach((id) => typed.setEntitySelected(id, true));
      setSelectedCount(entities.length);
    } catch (err) {
      console.error("Select all failed:", err);
    }
  }, [bridge, isLoaded]);

  return {
    duplicate,
    deleteSelected,
    selectAll,
    selectedCount,
  };
}
