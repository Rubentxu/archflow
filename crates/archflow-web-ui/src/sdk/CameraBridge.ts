/**
 * CameraBridge - Camera/Viewport Operations for ArchFlow
 *
 * This facade organizes camera-related methods from WasmBridge by domain.
 * Provides methods for zoom, pan, and viewport control.
 *
 * @example
 * ```typescript
 * const bridge = new ArchFlowBridge(wasmBridge);
 * bridge.camera.setZoom(2.0);
 * bridge.camera.setCenter(100, 200);
 * const zoom = bridge.camera.getZoom();
 * ```
 */

import type { WasmBridge } from "./types";

/**
 * Create a new CameraBridge instance
 *
 * @param bridge - WASM bridge instance
 * @returns CameraBridge instance
 */
export function createCameraBridge(bridge: any): CameraBridge {
  return new CameraBridge(bridge);
}

/**
 * Camera state snapshot
 */
export interface CameraState {
  zoom: number;
  center: [number, number];
  viewportWidth: number;
  viewportHeight: number;
}

/**
 * Camera/Viewport operations
 */
export class CameraBridge {
  constructor(private bridge: WasmBridge) {}

  // ═══════════════════════════════════════════════════════════════════════════════
  // ZOOM
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set zoom level
   * @param zoom - Zoom factor (1.0 = 100%, 2.0 = 200%)
   */
  setZoom(zoom: number): void {
    this.bridge.set_zoom(zoom);
  }

  /**
   * Get current zoom level
   */
  getZoom(): number {
    return this.bridge.get_zoom();
  }

  /**
   * Zoom in by factor
   */
  zoomIn(factor: number = 1.5): void {
    const current = this.getZoom();
    this.setZoom(current * factor);
  }

  /**
   * Zoom out by factor
   */
  zoomOut(factor: number = 1.5): void {
    const current = this.getZoom();
    this.setZoom(current / factor);
  }

  /**
   * Reset zoom to default (1.0)
   */
  resetZoom(): void {
    this.setZoom(1.0);
  }

  /**
   * Clamp zoom to valid range
   */
  clampZoom(min: number = 0.1, max: number = 10): void {
    const current = this.getZoom();
    this.setZoom(Math.max(min, Math.min(max, current)));
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // PAN
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set camera center position
   */
  setCenter(x: number, y: number): void {
    this.bridge.set_camera_center(x, y);
  }

  /**
   * Get camera center position
   */
  getCenter(): [number, number] {
    return this.bridge.get_camera_center();
  }

  /**
   * Pan camera by delta
   */
  pan(dx: number, dy: number): void {
    const [cx, cy] = this.getCenter();
    this.setCenter(cx + dx, cy + dy);
  }

  /**
   * Pan to center on entity
   */
  focusOn(
    entityId: number,
    entityBridge: import("./EntityBridge").EntityBridge,
  ): void {
    const pos = entityBridge.getPosition(entityId);
    this.setCenter(pos[0], pos[1]);
  }

  /**
   * Center on point
   */
  centerOn(x: number, y: number): void {
    this.setCenter(x, y);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // STATE
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Get complete camera state
   */
  getState(): CameraState {
    return {
      zoom: this.getZoom(),
      center: this.getCenter(),
      viewportWidth: 0, // TODO: Get from canvas
      viewportHeight: 0,
    };
  }

  /**
   * Restore camera state
   */
  setState(state: Partial<CameraState>): void {
    if (state.zoom !== undefined) {
      this.setZoom(state.zoom);
    }
    if (state.center !== undefined) {
      this.setCenter(state.center[0], state.center[1]);
    }
  }

  /**
   * Reset camera to default state
   */
  reset(): void {
    this.resetZoom();
    this.setCenter(0, 0);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // CONVERSION
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Convert screen coordinates to world coordinates
   */
  screenToWorld(screenX: number, screenY: number): [number, number] {
    // TODO: Implement in WasmBridge
    console.warn("screenToWorld() - implementation pending in WasmBridge");
    return [screenX, screenY];
  }

  /**
   * Convert world coordinates to screen coordinates
   */
  worldToScreen(worldX: number, worldY: number): [number, number] {
    // TODO: Implement in WasmBridge
    console.warn("worldToScreen() - implementation pending in WasmBridge");
    return [worldX, worldY];
  }
}
