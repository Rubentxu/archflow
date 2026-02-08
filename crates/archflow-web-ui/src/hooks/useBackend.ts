/**
 * Hook for managing graphics backend selection
 *
 * Handles backend detection, selection, and initialization
 * with WebGL2 as default and WebGPU as optional.
 */

import { useState, useCallback, useEffect, useMemo } from "react";
import type { WasmBridge } from "../wasm/archflow_web.js";
import type { BackendInfo, BackendState, GraphicsBackend } from "../types/wasm";

/**
 * Hook to detect available graphics backends
 */
export function useBackendDetection(bridge: WasmBridge | null): BackendState {
  const [state, setState] = useState<BackendState>({
    availableBackends: null,
    selectedBackend: "webgl2",
    isInitialized: false,
    error: null,
  });

  useEffect(() => {
    console.log("[useBackendDetection] Effect triggered:", {
      hasBridge: !!bridge,
    });

    if (!bridge) {
      console.log("[useBackendDetection] No bridge, clearing state");
      setState((prev) => ({
        ...prev,
        availableBackends: null,
        error: null,
      }));
      return;
    }

    try {
      console.log("[useBackendDetection] Calling detect_available_backends...");
      // Call Rust's detect_available_backends()
      const backends =
        bridge.detect_available_backends() as unknown as BackendInfo;

      console.log("[useBackendDetection] ✓ Detected backends:", backends);
      setState((prev) => ({
        ...prev,
        availableBackends: backends,
        selectedBackend: backends.preferred,
        error: null,
      }));
    } catch (err) {
      console.warn(
        "[useBackendDetection] Detection failed, falling back to WebGL2:",
        err,
      );
      // WebGPU might not be available, fallback to WebGL2 only
      setState((prev) => ({
        ...prev,
        availableBackends: {
          webgl2: true,
          webgpu: false,
          preferred: "webgl2",
          performance: {
            webgl2: "50fps @ 100k entities",
            webgpu: "N/A",
          },
        },
        selectedBackend: "webgl2",
        error: null,
      }));
    }
  }, [bridge]);

  return state;
}

/**
 * Hook to initialize graphics with selected backend
 * NOTE: This hook is NO LONGER USED because it tries to initialize graphics
 * before the engine is ready. Graphics initialization is now handled in Canvas.tsx
 * after bridge.initialize() is called.
 */
export function useBackendInitialization(
  bridge: WasmBridge | null,
  canvas: HTMLCanvasElement | null,
  selectedBackend: GraphicsBackend,
  isEngineInitialized: boolean,
) {
  const [isGraphicsReady, setIsGraphicsReady] = useState(false);
  const [graphicsError, setGraphicsError] = useState<string | null>(null);

  useEffect(() => {
    console.log("[useBackendInitialization] Effect triggered (DEPRECATED):", {
      hasBridge: !!bridge,
      hasCanvas: !!canvas,
      isEngineInitialized,
      selectedBackend,
    });

    // DISABLED: This was causing "Engine not initialized" errors
    // Graphics init is now handled in Canvas.tsx after engine init
    console.log(
      "[useBackendInitialization] Skipping - graphics init moved to Canvas.tsx",
    );
    return;

    // Original problematic code commented out:
    /*
    if (!bridge || !canvas || !isEngineInitialized) {
      console.log("[useBackendInitialization] Skipping - missing dependencies");
      return;
    }

    // Commented out problematic initialization code
    */
  }, [bridge, canvas, selectedBackend, isEngineInitialized]);

  return { isGraphicsReady, graphicsError };
}

/**
 * Hook that combines backend detection (NO initialization - that's in Canvas.tsx)
 */
export function useBackend(
  bridge: WasmBridge | null,
  canvas: HTMLCanvasElement | null,
  isEngineInitialized: boolean,
) {
  const detectionState = useBackendDetection(bridge);
  // REMOVED: useBackendInitialization call - it was causing race conditions
  // Graphics initialization now happens in Canvas.tsx after engine init
  const isGraphicsReady = false; // Not used anymore
  const graphicsError = null; // Not used anymore

  const [preferredBackend, setPreferredBackend] =
    useState<GraphicsBackend>("webgl2");

  // Update preferred when detection changes
  // Use useMemo to prevent infinite loop - only update when preferred actually changes
  const preferredBackendValue = useMemo(
    () => detectionState.availableBackends?.preferred || "webgl2",
    [detectionState.availableBackends?.preferred],
  );

  useEffect(() => {
    console.log("[useBackend] Detection state changed:", {
      availableBackends: detectionState.availableBackends,
    });
    if (detectionState.availableBackends) {
      setPreferredBackend(preferredBackendValue);
    }
  }, [preferredBackendValue]);

  const selectBackend = useCallback((backend: GraphicsBackend) => {
    console.log("[useBackend] Backend selected:", backend);
    setPreferredBackend(backend);
  }, []);

  // Memoize return value to prevent infinite loops in dependent components
  // Only depend on primitive values, not objects
  return useMemo(
    () => ({
      availableBackends: detectionState.availableBackends,
      selectedBackend: preferredBackend,
      isInitialized: detectionState.isInitialized,
      error: detectionState.error,
      selectBackend,
      isGraphicsReady,
      graphicsError,
    }),
    [
      detectionState.availableBackends?.preferred, // Only depend on primitive
      detectionState.isInitialized,
      detectionState.error,
      preferredBackend,
      selectBackend,
    ],
  );
}
