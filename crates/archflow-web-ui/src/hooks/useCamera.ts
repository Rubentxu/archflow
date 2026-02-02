/**
 * Hook for managing camera state
 *
 * Provides zoom, pan, and coordinate transformations.
 * Can work with or without WASM bridge.
 */

import { useState, useCallback } from "react";
import type { CameraState as CameraStateType, Vec2 } from "../types/wasm";

interface CameraReturn {
  camera: CameraStateType;
  setZoom: (zoom: number) => void;
  zoomIn: (factor?: number) => void;
  zoomOut: (factor?: number) => void;
  setCenter: (x: number, y: number) => void;
  pan: (dx: number, dy: number) => void;
  worldToScreen: (worldPos: Vec2) => Vec2;
  screenToWorld: (screenPos: Vec2) => Vec2;
  setCanvasSize: (width: number, height: number) => void;
}

const DEFAULT_ZOOM = 1.0;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 10.0;
const ZOOM_FACTOR = 1.2;

export function useCamera(_bridge: unknown = null): CameraReturn {
  const [camera, setCamera] = useState<CameraStateType>({
    center: { x: 0, y: 0 },
    zoom: DEFAULT_ZOOM,
    canvasWidth: 800,
    canvasHeight: 600,
  });

  const setZoom = useCallback((zoom: number) => {
    setCamera((prev) => ({
      ...prev,
      zoom: Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom)),
    }));
  }, []);

  const zoomIn = useCallback((factor = ZOOM_FACTOR) => {
    setCamera((prev) => ({
      ...prev,
      zoom: Math.min(prev.zoom * factor, MAX_ZOOM),
    }));
  }, []);

  const zoomOut = useCallback((factor = ZOOM_FACTOR) => {
    setCamera((prev) => ({
      ...prev,
      zoom: Math.max(prev.zoom / factor, MIN_ZOOM),
    }));
  }, []);

  const setCenter = useCallback((x: number, y: number) => {
    setCamera((prev) => ({
      ...prev,
      center: { x, y },
    }));
  }, []);

  const pan = useCallback((dx: number, dy: number) => {
    setCamera((prev) => ({
      ...prev,
      center: {
        x: prev.center.x - dx / prev.zoom,
        y: prev.center.y - dy / prev.zoom,
      },
    }));
  }, []);

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
    [camera],
  );

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
    [camera],
  );

  const setCanvasSize = useCallback((width: number, height: number) => {
    setCamera((prev) => ({
      ...prev,
      canvasWidth: width,
      canvasHeight: height,
    }));
  }, []);

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
  };
}
