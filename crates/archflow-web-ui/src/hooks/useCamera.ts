/**
 * Hook for managing camera state and viewport operations
 *
 * Provides zoom, pan, and coordinate transformation utilities.
 * Syncs with WASM bridge for persistent camera state.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type {
  WasmBridge,
  CameraState,
  Vec2,
  UseCameraReturn,
} from "../types/wasm";

/**
 * Default camera configuration
 */
const DEFAULT_ZOOM = 1.0;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 10.0;
const ZOOM_FACTOR = 1.2;

/**
 * Hook to manage camera state through the WASM bridge
 *
 * @param bridge - The WASM bridge instance
 * @param initialCenter - Initial camera center position
 * @returns Camera interface with zoom, pan, and coordinate utilities
 *
 * @example
 * ```typescript
 * const { camera, setZoom, zoomIn, zoomOut, pan, worldToScreen, screenToWorld } = useCamera(bridge);
 *
 * // Zoom in
 * zoomIn();
 *
 * // Pan to position
 * setCenter(500, 300);
 *
 * // Convert screen to world coordinates
 * const worldPos = screenToWorld({ x: 100, y: 200 });
 * ```
 */
export function useCamera(
  bridge: WasmBridge | null,
  initialCenter: Vec2 = { x: 0, y: 0 },
): UseCameraReturn {
  const [camera, setCamera] = useState<CameraState>({
    center: initialCenter,
    zoom: DEFAULT_ZOOM,
    canvasWidth: 800,
    canvasHeight: 600,
  });

  const lastCameraRef = useRef<CameraState | null>(null);

  // Sync camera state with WASM bridge
  useEffect(() => {
    if (!bridge) {
      return;
    }

    try {
      const wasmZoom = bridge.getZoom();
      const [wasmCenterX, wasmCenterY] = bridge.getCameraCenter();

      const newCamera: CameraState = {
        center: { x: wasmCenterX, y: wasmCenterY },
        zoom: wasmZoom,
        canvasWidth: camera.canvasWidth,
        canvasHeight: camera.canvasHeight,
      };

      // Only update if something changed
      if (
        !lastCameraRef.current ||
        lastCameraRef.current.center.x !== newCamera.center.x ||
        lastCameraRef.current.center.y !== newCamera.center.y ||
        lastCameraRef.current.zoom !== newCamera.zoom
      ) {
        setCamera(newCamera);
        lastCameraRef.current = newCamera;
      }
    } catch (err) {
      // WASM bridge might not be initialized yet
      console.debug("Camera sync skipped:", err);
    }
  }, [bridge]);

  /**
   * Set zoom level directly
   */
  const setZoom = useCallback(
    (zoom: number): void => {
      const clampedZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom));

      if (bridge) {
        try {
          bridge.setZoom(clampedZoom);
        } catch (err) {
          console.error("Failed to set zoom:", err);
        }
      }

      setCamera((prev) => ({
        ...prev,
        zoom: clampedZoom,
      }));
    },
    [bridge],
  );

  /**
   * Zoom in by the configured factor
   */
  const zoomIn = useCallback(
    (factor: number = ZOOM_FACTOR): void => {
      setZoom(camera.zoom * factor);
    },
    [camera.zoom, setZoom],
  );

  /**
   * Zoom out by the configured factor
   */
  const zoomOut = useCallback(
    (factor: number = ZOOM_FACTOR): void => {
      setZoom(camera.zoom / factor);
    },
    [camera.zoom, setZoom],
  );

  /**
   * Set camera center position
   */
  const setCenter = useCallback(
    (x: number, y: number): void => {
      if (bridge) {
        try {
          bridge.setCameraCenter(x, y);
        } catch (err) {
          console.error("Failed to set camera center:", err);
        }
      }

      setCamera((prev) => ({
        ...prev,
        center: { x, y },
      }));
    },
    [bridge],
  );

  /**
   * Pan the camera by the given delta
   */
  const pan = useCallback(
    (dx: number, dy: number): void => {
      setCamera((prev) => {
        const newCenter = {
          x: prev.center.x + dx / prev.zoom,
          y: prev.center.y + dy / prev.zoom,
        };

        if (bridge) {
          try {
            bridge.setCameraCenter(newCenter.x, newCenter.y);
          } catch (err) {
            console.error("Failed to pan camera:", err);
          }
        }

        return {
          ...prev,
          center: newCenter,
        };
      });
    },
    [bridge],
  );

  /**
   * Convert world coordinates to screen coordinates
   *
   * Formula:
   * screenX = (worldX - centerX) * zoom + canvasWidth / 2
   * screenY = (worldY - centerY) * zoom + canvasHeight / 2
   */
  const worldToScreen = useCallback(
    (worldPos: Vec2): Vec2 => {
      return {
        x:
          (worldPos.x - camera.center.x) * camera.zoom + camera.canvasWidth / 2,
        y:
          (worldPos.y - camera.center.y) * camera.zoom +
          camera.canvasHeight / 2,
      };
    },
    [camera.center, camera.zoom, camera.canvasWidth, camera.canvasHeight],
  );

  /**
   * Convert screen coordinates to world coordinates
   *
   * Formula:
   * worldX = (screenX - canvasWidth / 2) / zoom + centerX
   * worldY = (screenY - canvasHeight / 2) / zoom + centerY
   */
  const screenToWorld = useCallback(
    (screenPos: Vec2): Vec2 => {
      return {
        x:
          (screenPos.x - camera.canvasWidth / 2) / camera.zoom +
          camera.center.x,
        y:
          (screenPos.y - camera.canvasHeight / 2) / camera.zoom +
          camera.center.y,
      };
    },
    [camera.center, camera.zoom, camera.canvasWidth, camera.canvasHeight],
  );

  /**
   * Set canvas dimensions (for responsive resize)
   */
  const setCanvasSize = useCallback((width: number, height: number): void => {
    setCamera((prev) => ({
      ...prev,
      canvasWidth: width,
      canvasHeight: height,
    }));
  }, []);

  /**
   * Fit all content within the viewport
   */
  const fitToContent = useCallback(
    (contentBounds: {
      minX: number;
      minY: number;
      maxX: number;
      maxY: number;
    }): void => {
      const contentWidth = contentBounds.maxX - contentBounds.minX;
      const contentHeight = contentBounds.maxY - contentBounds.minY;

      if (contentWidth === 0 || contentHeight === 0) {
        return;
      }

      // Calculate zoom to fit content with padding
      const padding = 50;
      const zoomX = (camera.canvasWidth - padding * 2) / contentWidth;
      const zoomY = (camera.canvasHeight - padding * 2) / contentHeight;
      const newZoom = Math.min(zoomX, zoomY, MAX_ZOOM);

      // Calculate center
      const centerX = (contentBounds.minX + contentBounds.maxX) / 2;
      const centerY = (contentBounds.minY + contentBounds.maxY) / 2;

      setZoom(newZoom);
      setCenter(centerX, centerY);
    },
    [camera.canvasWidth, camera.canvasHeight, setZoom, setCenter],
  );

  /**
   * Reset camera to default state
   */
  const reset = useCallback((): void => {
    setZoom(DEFAULT_ZOOM);
    setCenter(initialCenter.x, initialCenter.y);
  }, [setZoom, setCenter, initialCenter]);

  return {
    camera,
    setZoom,
    zoomIn,
    zoomOut,
    setCenter,
    pan,
    worldToScreen,
    screenToWorld,
    setCanvasSize,
    fitToContent,
    reset,
  };
}

/**
 * Hook for wheel-based zoom at mouse position
 *
 * Handles zoom centered on the mouse cursor position.
 *
 * @param bridge - The WASM bridge instance
 * @param camera - Current camera state
 * @param setCamera - State setter for camera
 * @param screenToWorld - Screen to world converter
 * @returns Wheel event handler
 */
export function useWheelZoom(
  bridge: WasmBridge | null,
  camera: CameraState,
  setCamera: React.Dispatch<React.SetStateAction<CameraState>>,
  screenToWorld: (pos: Vec2) => Vec2,
) {
  return useCallback(
    (event: WheelEvent): void => {
      event.preventDefault();

      const zoomFactor = event.deltaY > 0 ? 1 / ZOOM_FACTOR : ZOOM_FACTOR;
      const mousePos: Vec2 = { x: event.clientX, y: event.clientY };

      // Get world position before zoom
      const worldPos = screenToWorld(mousePos);

      // Apply zoom
      const newZoom = Math.max(
        MIN_ZOOM,
        Math.min(MAX_ZOOM, camera.zoom * zoomFactor),
      );

      if (bridge) {
        try {
          bridge.setZoom(newZoom);
        } catch (err) {
          console.error("Failed to set zoom:", err);
        }
      }

      // Calculate new camera center to keep mouse position fixed in world
      const newCenterX =
        worldPos.x - (mousePos.x - camera.canvasWidth / 2) / newZoom;
      const newCenterY =
        worldPos.y - (mousePos.y - camera.canvasHeight / 2) / newZoom;

      if (bridge) {
        try {
          bridge.setCameraCenter(newCenterX, newCenterY);
        } catch (err) {
          console.error("Failed to set camera center:", err);
        }
      }

      setCamera((prev) => ({
        ...prev,
        zoom: newZoom,
        center: { x: newCenterX, y: newCenterY },
      }));
    },
    [bridge, camera, setCamera, screenToWorld],
  );
}

/**
 * Zoom level presets for quick access
 */
export const ZOOM_PRESETS = [
  { label: "25%", value: 0.25 },
  { label: "50%", value: 0.5 },
  { label: "75%", value: 0.75 },
  { label: "100%", value: 1.0 },
  { label: "150%", value: 1.5 },
  { label: "200%", value: 2.0 },
  { label: "400%", value: 4.0 },
];

/**
 * Get zoom percentage label
 */
export function getZoomLabel(zoom: number): string {
  return `${Math.round(zoom * 100)}%`;
}
