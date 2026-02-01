/**
 * ArchFlow - SharedArrayBuffer Input Bridge
 *
 * This module provides the JavaScript side of the zero-copy input pipeline.
 * It writes input events directly to a SharedArrayBuffer that Rust reads atomically.
 *
 * Reference: docs/epics/EPIC-001-input-sensors.md - HU-003
 *
 * Memory Layout (64 bytes total, cache-line aligned):
 * ```
 * Offset | Size | Type    | Field
 * -------|------|---------|----------
 * 0      | 4    | u32     | head (write index)
 * 4      | 4    | u32     | tail (read index)
 * 8      | 4    | i32     | mouse_x
 * 12     | 4    | i32     | mouse_y
 * 16     | 1    | u8      | buttons (bitmask)
 * 17     | 1    | u8      | modifiers (bitmask)
 * 18     | 2    | i16     | wheel_delta
 * 20     | 4    | u32     | timestamp
 * 24     | 32   | [u8;32] | keys (256 bits, 1=key down)
 * 56     | 8    | padding | alignment
 * ```
 */

/** Key code mapping from JavaScript KeyboardEvent to our u8 codes */
export const KEY_CODES: Record<string, number> = {
  // Letters
  'a': 0, 'b': 1, 'c': 2, 'd': 3, 'e': 4, 'f': 5, 'g': 6, 'h': 7, 'i': 8, 'j': 9,
  'k': 10, 'l': 11, 'm': 12, 'n': 13, 'o': 14, 'p': 15, 'q': 16, 'r': 17, 's': 18, 't': 19,
  'u': 20, 'v': 21, 'w': 22, 'x': 23, 'y': 24, 'z': 25,
  // Digits
  '0': 26, '1': 27, '2': 28, '3': 29, '4': 30, '5': 31, '6': 32, '7': 33, '8': 34, '9': 35,
  // Function keys
  'f1': 36, 'f2': 37, 'f3': 38, 'f4': 39, 'f5': 40, 'f6': 41, 'f7': 42, 'f8': 43, 'f9': 44, 'f10': 45,
  'f11': 46, 'f12': 47,
  // Special keys
  'enter': 48, 'escape': 49, 'tab': 50, 'backspace': 51, 'delete': 52, 'insert': 53,
  'home': 54, 'end': 55, 'pageup': 56, 'pagedown': 57,
  'arrowleft': 58, 'arrowup': 59, 'arrowright': 60, 'arrowdown': 61,
  'space': 62,
};

/** Mouse button enum for bitmask */
export enum MouseButton {
  Left = 0b001,
  Right = 0b010,
  Middle = 0b100,
}

/** Modifier key enum for bitmask */
export enum Modifier {
  Shift = 0b001,
  Ctrl = 0b010,
  Alt = 0b100,
}

/**
 * SharedArrayBuffer input writer
 *
 * This class provides methods to write input events to a SharedArrayBuffer
 * that Rust reads atomically using the InputSampler.
 */
export class InputSABWriter {
  private view: DataView;
  private uint8: Uint8Array;

  /**
   * Create a new InputSABWriter
   *
   * @param sab - The SharedArrayBuffer (must be exactly 64 bytes)
   * @throws Error if SAB is not exactly 64 bytes
   */
  constructor(sab: SharedArrayBuffer) {
    if (sab.byteLength !== 64) {
      throw new Error(`SharedArrayBuffer must be exactly 64 bytes, got ${sab.byteLength}`);
    }
    this.view = new DataView(sab);
    this.uint8 = new Uint8Array(sab);
  }

  /**
   * Set mouse position
   *
   * @param x - Mouse X coordinate in pixels
   * @param y - Mouse Y coordinate in pixels
   */
  setMousePosition(x: number, y: number): void {
    this.view.setInt32(8, x, true);  // little-endian
    this.view.setInt32(12, y, true);
    this.updateTimestamp();
  }

  /**
   * Set mouse button state
   *
   * @param button - Which button to set
   * @param pressed - Whether the button is pressed
   */
  setMouseButton(button: MouseButton, pressed: true): void;
  setMouseButton(button: MouseButton, pressed: false): void;
  setMouseButton(button: MouseButton, pressed: boolean): void {
    const offset = 16;
    let current = this.view.getUint8(offset);
    if (pressed) {
      current |= button;
    } else {
      current &= ~button;
    }
    this.view.setUint8(offset, current);
    this.updateTimestamp();
  }

  /**
   * Set modifier keys state
   *
   * @param modifiers - Bitmask of active modifiers
   */
  setModifiers(modifiers: number): void {
    this.view.setUint8(17, modifiers);
    this.updateTimestamp();
  }

  /**
   * Set wheel delta
   *
   * @param delta - Wheel delta (positive = up, negative = down)
   */
  setWheelDelta(delta: number): void {
    this.view.setInt16(18, delta, true);
    this.updateTimestamp();
  }

  /**
   * Set key state
   *
   * @param keycode - Key code (0-255)
   * @param pressed - Whether the key is pressed
   */
  setKey(keycode: number, pressed: boolean): void {
    if (keycode < 0 || keycode > 255) {
      throw new Error(`Keycode must be 0-255, got ${keycode}`);
    }
    const byteOffset = 24 + Math.floor(keycode / 8);
    const bitMask = 1 << (keycode % 8);
    const current = this.uint8[byteOffset];
    this.uint8[byteOffset] = pressed ? (current | bitMask) : (current & ~bitMask);
    this.updateTimestamp();
  }

  /**
   * Set key state by key name
   *
   * @param keyName - Name of the key (e.g., 'a', 'Enter', 'F1')
   * @param pressed - Whether the key is pressed
   */
  setKeyByName(keyName: string, pressed: boolean): void {
    const normalized = keyName.toLowerCase();
    const keycode = KEY_CODES[normalized];
    if (keycode === undefined) {
      console.warn(`Unknown key name: ${keyName}`);
      return;
    }
    this.setKey(keycode, pressed);
  }

  /**
   * Update timestamp (called automatically by other methods)
   */
  private updateTimestamp(): void {
    const timestamp = Date.now();
    this.view.setUint32(20, timestamp, true);
  }

  /**
   * Get the current timestamp from the SAB
   */
  getTimestamp(): number {
    return this.view.getUint32(20, true);
  }

  /**
   * Get current mouse position
   */
  getMousePosition(): { x: number; y: number } {
    return {
      x: this.view.getInt32(8, true),
      y: this.view.getInt32(12, true),
    };
  }

  /**
   * Check if a mouse button is pressed
   */
  isMouseButtonPressed(button: MouseButton): boolean {
    return (this.view.getUint8(16) & button) !== 0;
  }

  /**
   * Check if a key is pressed
   */
  isKeyPressed(keycode: number): boolean {
    if (keycode < 0 || keycode > 255) return false;
    const byteOffset = 24 + Math.floor(keycode / 8);
    const bitMask = 1 << (keycode % 8);
    return (this.uint8[byteOffset] & bitMask) !== 0;
  }
}

/**
 * Helper class to bridge DOM events to SharedArrayBuffer
 *
 * This attaches event listeners to a DOM element and writes all input
 * events to the SharedArrayBuffer automatically.
 */
export class DOMInputBridge {
  private writer: InputSABWriter;
  private element: HTMLElement | Window;
  private cleanup: Array<() => void> = [];

  /**
   * Create a new DOM input bridge
   *
   * @param sab - The SharedArrayBuffer to write to
   * @param element - The element to attach listeners to (default: window)
   */
  constructor(sab: SharedArrayBuffer, element: HTMLElement | Window = window) {
    this.writer = new InputSABWriter(sab);
    this.element = element;
  }

  /**
   * Attach all event listeners
   *
   * Call this after creating the bridge to start capturing input.
   */
  attach(): void {
    const target = this.element;

    // Mouse events
    this.addEventListener(target, 'mousemove', (e: MouseEvent) => this.onMouseMove(e));
    this.addEventListener(target, 'mousedown', (e: MouseEvent) => this.onMouseDown(e));
    this.addEventListener(target, 'mouseup', (e: MouseEvent) => this.onMouseUp(e));
    this.addEventListener(target, 'wheel', (e: WheelEvent) => this.onWheel(e));

    // Keyboard events (attach to window for global capture)
    this.addEventListener(window, 'keydown', (e: KeyboardEvent) => this.onKeyDown(e));
    this.addEventListener(window, 'keyup', (e: KeyboardEvent) => this.onKeyUp(e));
  }

  /**
   * Remove all event listeners
   *
   * Call this when cleaning up to prevent memory leaks.
   */
  detach(): void {
    this.cleanup.forEach(fn => fn());
    this.cleanup = [];
  }

  private addEventListener<T extends EventTarget>(
    target: T,
    type: string,
    listener: (e: Event) => void
  ): void {
    target.addEventListener(type, listener);
    this.cleanup.push(() => target.removeEventListener(type, listener));
  }

  private onMouseMove(e: MouseEvent): void {
    this.writer.setMousePosition(e.clientX, e.clientY);
    this.updateModifiers(e);
  }

  private onMouseDown(e: MouseEvent): void {
    const button = this.mouseButtonToMask(e.button);
    this.writer.setMouseButton(button, true);
    this.updateModifiers(e);
  }

  private onMouseUp(e: MouseEvent): void {
    const button = this.mouseButtonToMask(e.button);
    this.writer.setMouseButton(button, false);
    this.updateModifiers(e);
  }

  private onWheel(e: WheelEvent): void {
    // Normalize wheel delta (-1 to 1 range)
    const delta = Math.sign(e.deltaY) * -1; // Invert so up is positive
    this.writer.setWheelDelta(delta);
  }

  private onKeyDown(e: KeyboardEvent): void {
    this.writer.setKeyByName(e.key, true);
    this.updateModifiers(e);
  }

  private onKeyUp(e: KeyboardEvent): void {
    this.writer.setKeyByName(e.key, false);
    this.updateModifiers(e);
  }

  private updateModifiers(e: MouseEvent | KeyboardEvent): void {
    let modifiers = 0;
    if (e.shiftKey) modifiers |= Modifier.Shift;
    if (e.ctrlKey) modifiers |= Modifier.Ctrl;
    if (e.altKey) modifiers |= Modifier.Alt;
    this.writer.setModifiers(modifiers);
  }

  private mouseButtonToMask(button: number): MouseButton {
    switch (button) {
      case 0: return MouseButton.Left;
      case 1: return MouseButton.Middle;
      case 2: return MouseButton.Right;
      default: return MouseButton.Left;
    }
  }
}

/**
 * Check if SharedArrayBuffer is available in the current environment
 *
 * SAB requires specific HTTP headers (COOP/COEP) and may not be available
 * in all browsers or contexts.
 */
export function isSharedArrayBufferAvailable(): boolean {
  try {
    // Check if SharedArrayBuffer constructor exists
    if (typeof SharedArrayBuffer === 'undefined') {
      return false;
    }

    // Try to create a small SAB to verify it actually works
    const sab = new SharedArrayBuffer(4);
    const view = new Int32Array(sab);
    view[0] = 42;

    // In some environments, SAB exists but throws when accessed
    return view[0] === 42;
  } catch {
    return false;
  }
}

/**
 * Create a SharedArrayBuffer for input (if available)
 *
 * Returns null if SAB is not available, allowing for fallback behavior.
 */
export function createInputSAB(): SharedArrayBuffer | null {
  if (!isSharedArrayBufferAvailable()) {
    console.warn('SharedArrayBuffer not available, falling back to postMessage input');
    return null;
  }
  return new SharedArrayBuffer(64);
}
