/**
 * Hook for handling input events
 * Simplified version without type conflicts.
 */

import { useCallback } from "react";
import { useArchFlowWasm } from "./useArchFlowWasm.tsx";
import { getTypedBridge } from "./wasm-bridge";

// Input event types as numbers (matching Rust)
const INPUT_DOWN = 0;
const INPUT_MOVE = 1;
const INPUT_UP = 2;
const INPUT_WHEEL = 3;

interface UseInputReturn {
  onPointerDown: (position: { x: number; y: number }, buttons: number) => void;
  onPointerMove: (position: { x: number; y: number }, buttons: number) => void;
  onPointerUp: (position: { x: number; y: number }, buttons: number) => void;
  onWheel: (position: { x: number; y: number }, delta: number) => void;
}

export function useInput(): UseInputReturn {
  const { bridge, isLoaded } = useArchFlowWasm();

  const pushEvent = useCallback(
    (
      eventType: number,
      x: number,
      y: number,
      buttons: number,
      modifiers: number,
    ) => {
      if (!isLoaded) return;
      const typedBridge = getTypedBridge(bridge);
      if (!typedBridge) return;
      try {
        typedBridge.pushInputEvent(eventType, x, y, buttons, modifiers);
      } catch {
        // Silent fail for input events
      }
    },
    [bridge, isLoaded],
  );

  const onPointerDown = useCallback(
    (position: { x: number; y: number }, buttons: number) => {
      pushEvent(INPUT_DOWN, position.x, position.y, buttons, 0);
    },
    [pushEvent],
  );

  const onPointerMove = useCallback(
    (position: { x: number; y: number }, buttons: number) => {
      pushEvent(INPUT_MOVE, position.x, position.y, buttons, 0);
    },
    [pushEvent],
  );

  const onPointerUp = useCallback(
    (position: { x: number; y: number }, buttons: number) => {
      pushEvent(INPUT_UP, position.x, position.y, buttons, 0);
    },
    [pushEvent],
  );

  const onWheel = useCallback(
    (_position: { x: number; y: number }, _delta: number) => {
      pushEvent(INPUT_WHEEL, 0, 0, 0, 0);
    },
    [pushEvent],
  );

  return { onPointerDown, onPointerMove, onPointerUp, onWheel };
}

// Keyboard shortcuts hook
export function useKeyboardShortcuts(
  shortcuts: { key: string; ctrl?: boolean; action: () => void }[],
) {
  const handleKeyDown = (event: KeyboardEvent) => {
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    const shortcut = shortcuts.find((s) => {
      const ctrlMatch = s.ctrl
        ? event.ctrlKey || event.metaKey
        : !(event.ctrlKey || event.metaKey);
      return s.key.toLowerCase() === event.key.toLowerCase() && ctrlMatch;
    });

    if (shortcut) {
      event.preventDefault();
      shortcut.action();
    }
  };

  return { handleKeyDown };
}
