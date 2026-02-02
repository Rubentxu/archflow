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
 * Available shortcuts configuration
 */
const shortcuts: KeyboardShortcut[] = [
  // Tool selection
  { key: "v", description: "Select tool", action: () => {} },
  { key: "h", description: "Pan tool", action: () => {} },
  { key: "r", description: "Rectangle tool", action: () => {} },
  { key: "c", description: "Circle tool", action: () => {} },
  { key: "t", description: "Triangle tool", action: () => {} },
  { key: "d", description: "Diamond tool", action: () => {} },
  { key: "x", description: "Text tool", action: () => {} },
  { key: "l", description: "Connection tool", action: () => {} },
  // Edit operations
  { key: "z", ctrl: true, description: "Undo", action: () => {} },
  { key: "y", ctrl: true, description: "Redo", action: () => {} },
  { key: "z", ctrl: true, shift: true, description: "Redo (alternative)", action: () => {} },
  { key: "d", ctrl: true, description: "Duplicate selected", action: () => {} },
  { key: "a", ctrl: true, description: "Select all", action: () => {} },
  { key: "Delete", description: "Delete selected", action: () => {} },
  { key: "Backspace", description: "Delete selected", action: () => {} },
  { key: "Escape", description: "Deselect all", action: () => {} },
  { key: "g", ctrl: true, description: "Group selected", action: () => {} },
  { key: "u", ctrl: true, shift: true, description: "Ungroup", action: () => {} },
  // Zoom
  { key: "+", ctrl: true, description: "Zoom in", action: () => {} },
  { key: "-", ctrl: true, description: "Zoom out", action: () => {} },
  { key: "0", ctrl: true, description: "Reset zoom", action: () => {} },
  { key: "1", ctrl: true, description: "Fit to screen", action: () => {} },
];

/**
 * Tool shortcuts mapping
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
  triggerAction: (key: string, modifiers?: { ctrl?: boolean; shift?: boolean; alt?: boolean }) => void;
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
  }, [setActiveTool, clearSelection, undo, redo, deleteSelected, duplicateSelected, selectAll]);

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

    // Find matching shortcut
    const shortcut = shortcuts.find((s) => {
      const keyMatch = s.key.toLowerCase() === event.key.toLowerCase();
      const ctrlMatch = s.ctrl
        ? event.ctrlKey || event.metaKey
        : !(event.ctrlKey || event.metaKey);
      const shiftMatch = s.shift ? event.shiftKey : !event.shiftKey;
      const altMatch = s.alt ? event.altKey : !event.altKey;
      return keyMatch && ctrlMatch && shiftMatch && altMatch;
    });

    if (!shortcut) return;

    event.preventDefault();

    // Execute action based on shortcut type
    if (toolShortcuts[shortcut.key.toLowerCase()]) {
      // Tool selection
      const tool = toolShortcuts[shortcut.key.toLowerCase()];
      actionsRef.current.setActiveTool(tool);
    } else if (shortcut.key === "z" && shortcut.ctrl && !shortcut.shift) {
      // Undo
      actionsRef.current.undo();
    } else if (
      (shortcut.key === "y" && shortcut.ctrl) ||
      (shortcut.key === "z" && shortcut.ctrl && shortcut.shift)
    ) {
      // Redo
      actionsRef.current.redo();
    } else if (shortcut.key === "d" && shortcut.ctrl) {
      // Duplicate
      actionsRef.current.duplicateSelected();
    } else if (shortcut.key === "a" && shortcut.ctrl) {
      // Select all
      actionsRef.current.selectAll();
    } else if (shortcut.key === "Delete" || shortcut.key === "Backspace") {
      // Delete selected
      actionsRef.current.deleteSelected();
    } else if (shortcut.key === "Escape") {
      // Deselect all
      actionsRef.current.clearSelection();
    } else if (shortcut.key === "g" && shortcut.ctrl) {
      // Group (placeholder)
      console.debug("Group operation not yet implemented");
    } else if (shortcut.key === "u" && shortcut.ctrl && shortcut.shift) {
      // Ungroup (placeholder)
      console.debug("Ungroup operation not yet implemented");
    } else if (shortcut.key === "+" && shortcut.ctrl) {
      // Zoom in (placeholder - would call zoomIn)
      console.debug("Zoom in not yet implemented");
    } else if (shortcut.key === "-" && shortcut.ctrl) {
      // Zoom out (placeholder - would call zoomOut)
      console.debug("Zoom out not yet implemented");
    } else if (shortcut.key === "0" && shortcut.ctrl) {
      // Reset zoom (placeholder)
      console.debug("Reset zoom not yet implemented");
    } else if (shortcut.key === "1" && shortcut.ctrl) {
      // Fit to screen (placeholder)
      console.debug("Fit to screen not yet implemented");
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
    (key: string, modifiers?: { ctrl?: boolean; shift?: boolean; alt?: boolean }) => {
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
    shortcuts,
    triggerAction,
  };
}
