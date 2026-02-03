/**
 * Hook for keyboard shortcuts management
 *
 * Provides keyboard shortcuts for tool selection, undo/redo,
 * delete, duplicate, and other common actions.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useEffect, useCallback, useRef } from "react";
import { useUIStore } from "../store/useUIStore";
import { useSelectionStore } from "../store/useSelectionStore";
import { useCommandHistory } from "./useCommandHistory";
import { useSelection } from "./useSelection";
import type { ToolType } from "../types/wasm";

/**
 * Keyboard shortcut definition
 */
interface KeyboardShortcut {
  /** Key to press (case insensitive) */
  key: string;
  /** Whether Ctrl or Cmd is required */
  ctrl?: boolean;
  /** Whether Shift is required */
  shift?: boolean;
  /** Alt key requirement */
  alt?: boolean;
  /** Description for tooltip/help */
  description: string;
  /** Action to perform */
  action: () => void;
}

/**
 * Tool shortcuts mapping - connects keys to tool types
 */
const toolShortcuts: Record<string, ToolType> = {
  v: "select",
  h: "pan",
  r: "rectangle",
  c: "circle",
  t: "triangle",
  d: "diamond",
  x: "text",
  l: "connection",
};

/**
 * Hook result for keyboard shortcuts
 */
interface UseKeyboardShortcutsReturn {
  /** Register keyboard shortcuts */
  shortcuts: KeyboardShortcut[];
  /** Trigger an action by key */
  triggerAction: (
    key: string,
    modifiers?: { ctrl?: boolean; shift?: boolean; alt?: boolean },
  ) => void;
}

/**
 * Hook for managing keyboard shortcuts
 *
 * @returns Object with shortcut list and trigger function
 *
 * @example
 * ```typescript
 * const { shortcuts, triggerAction } = useKeyboardShortcuts();
 *
 * // Trigger undo manually
 * triggerAction("z", { ctrl: true });
 * ```
 */
export function useKeyboardShortcuts(): UseKeyboardShortcutsReturn {
  const { setActiveTool } = useUIStore();
  const { clearSelection } = useSelectionStore();
  const { undo, redo } = useCommandHistory();
  const { deleteSelected, duplicateSelected, selectAll } = useSelection();

  const actionsRef = useRef({
    setActiveTool,
    clearSelection,
    undo,
    redo,
    deleteSelected,
    duplicateSelected,
    selectAll,
  });

  // Keep ref updated
  useEffect(() => {
    actionsRef.current = {
      setActiveTool,
      clearSelection,
      undo,
      redo,
      deleteSelected,
      duplicateSelected,
      selectAll,
    };
  }, [
    setActiveTool,
    clearSelection,
    undo,
    redo,
    deleteSelected,
    duplicateSelected,
    selectAll,
  ]);

  /**
   * Handle key down event
   */
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    // Ignore if typing in input field
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    // Tool selection shortcuts
    const tool = toolShortcuts[event.key.toLowerCase()];
    if (tool) {
      event.preventDefault();
      actionsRef.current.setActiveTool(tool);
      return;
    }

    // Edit operations
    if (
      event.key === "z" &&
      (event.ctrlKey || event.metaKey) &&
      !event.shiftKey
    ) {
      event.preventDefault();
      actionsRef.current.undo();
      return;
    }

    if (
      (event.key === "y" && (event.ctrlKey || event.metaKey)) ||
      (event.key === "z" && (event.ctrlKey || event.metaKey) && event.shiftKey)
    ) {
      event.preventDefault();
      actionsRef.current.redo();
      return;
    }

    if (event.key === "d" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      actionsRef.current.duplicateSelected();
      return;
    }

    if (event.key === "a" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      actionsRef.current.selectAll();
      return;
    }

    if (event.key === "Delete" || event.key === "Backspace") {
      event.preventDefault();
      actionsRef.current.deleteSelected();
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      actionsRef.current.clearSelection();
      return;
    }
  }, []);

  // Register event listener
  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  /**
   * Manually trigger an action by key
   */
  const triggerAction = useCallback(
    (
      key: string,
      modifiers?: { ctrl?: boolean; shift?: boolean; alt?: boolean },
    ) => {
      const event = new KeyboardEvent("keydown", {
        key,
        ctrlKey: modifiers?.ctrl,
        shiftKey: modifiers?.shift,
        altKey: modifiers?.alt,
        bubbles: true,
      });
      window.dispatchEvent(event);
    },
    [],
  );

  return {
    shortcuts: [], // Not exposing shortcuts array in this implementation
    triggerAction,
  };
}
