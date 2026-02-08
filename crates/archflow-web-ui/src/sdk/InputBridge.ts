/**
 * InputBridge - Input Event Operations for ArchFlow
 *
 * This facade organizes input-related methods from WasmBridge by domain.
 * Provides methods for mouse, keyboard, and input state queries.
 *
 * @example
 * ```typescript
 * const bridge = new ArchFlowBridge(wasmBridge);
 * canvas.onmousemove = (e) => bridge.input.onMouseMove(e.clientX, e.clientY, e.buttons);
 * const pos = bridge.input.getMousePosition();
 * ```
 */

import type { WasmBridge } from './types';

/**
 * Mouse button constants
 */
export const MouseButton = {
  LEFT: 0,
  RIGHT: 1,
  MIDDLE: 2,
} as const;

/**
 * Modifier key constants
 */
export const ModifierKey = {
  SHIFT: 1,
  CTRL: 2,
  ALT: 4,
  META: 8,
} as const;

/**
 * Mouse button state
 */
export interface MouseState {
  x: number;
  y: number;
  buttons: number;
}

/**
 * Input event types
 */
export const InputEventType = {
  DOWN: 0,
  MOVE: 1,
  UP: 2,
  WHEEL: 3,
  KEY_DOWN: 4,
  KEY_UP: 5,
} as const;

/**
 * Input operations
 */
export class InputBridge {
  constructor(private bridge: WasmBridge) {}

  // ═══════════════════════════════════════════════════════════════════════════════
  // MOUSE EVENTS
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Report mouse movement
   */
  onMouseMove(screenX: number, screenY: number, buttons: number): void {
    this.bridge.on_mouse_move(screenX, screenY, buttons);
  }

  /**
   * Report mouse button press
   */
  onMouseDown(screenX: number, screenY: number, button: number, modifiers: number = 0): void {
    this.bridge.on_mouse_down(screenX, screenY, button, modifiers);
  }

  /**
   * Report mouse button release
   */
  onMouseUp(screenX: number, screenY: number, button: number): void {
    this.bridge.on_mouse_up(screenX, screenY, button);
  }

  /**
   * Report mouse wheel scroll
   */
  onWheel(screenX: number, screenY: number, deltaY: number, modifiers: number = 0): void {
    this.bridge.on_wheel(screenX, screenY, deltaY, modifiers);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // KEYBOARD EVENTS
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Report key press
   */
  onKeyDown(keyCode: number, modifiers: number = 0): void {
    this.bridge.on_key(keyCode, true, modifiers);
  }

  /**
   * Report key release
   */
  onKeyUp(keyCode: number, modifiers: number = 0): void {
    this.bridge.on_key(keyCode, false, modifiers);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // INPUT STATE QUERIES
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Get current mouse position as "x,y" string
   */
  getMousePosition(): string {
    return this.bridge.get_mouse_position();
  }

  /**
   * Get current mouse position as coordinates
   */
  getMousePositionCoords(): [number, number] {
    const pos = this.bridge.get_mouse_position();
    const [x, y] = pos.split(',').map(Number);
    return [x, y];
  }

  /**
   * Get current mouse button state as bitmask
   */
  getMouseButtons(): number {
    return this.bridge.get_mouse_buttons();
  }

  /**
   * Check if specific mouse button is pressed
   */
  isMouseButtonPressed(button: number): boolean {
    return (this.getMouseButtons() & (1 << button)) !== 0;
  }

  /**
   * Get current modifier keys as bitmask
   */
  getModifiers(): number {
    return this.bridge.get_modifiers();
  }

  /**
   * Check if specific modifier is active
   */
  isModifierActive(modifier: number): boolean {
    return (this.getModifiers() & modifier) !== 0;
  }

  /**
   * Get complete mouse state
   */
  getMouseState(): MouseState {
    const [x, y] = this.getMousePositionCoords();
    return {
      x,
      y,
      buttons: this.getMouseButtons(),
    };
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // INPUT PROCESSING
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Process all pending input events
   */
  processEvents(): void {
    this.bridge.process_input_events();
  }

  /**
   * Push input event to WASM processor
   */
  pushEvent(
    eventType: number,
    x: number,
    y: number,
    buttons: number,
    modifiers: number = 0,
  ): void {
    // TODO: Implement push_input_event in WasmBridge if needed
    console.warn('pushEvent() - use direct event methods instead');
  }
}

/**
 * Create a new InputBridge instance
 */
export function createInputBridge(bridge: any): InputBridge {
  return new InputBridge(bridge);
}

