/**
 * Hook for managing graphics backend selection
 *
 * Handles backend detection, selection, and initialization
 * with WebGL2 as default and WebGPU as optional.
 */

import { useState, useCallback, useEffect } from "react";
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
    if (!bridge) {
      setState((prev) => ({
        ...prev,
        availableBackends: null,
        error: null,
      }));
      return;
    }

    try {
      // Call Rust's detect_available_backends()
      const backends = bridge.detect_available_backends() as unknown as BackendInfo;

      setState((prev) => ({
        ...prev,
        availableBackends: backends,
        selectedBackend: backends.preferred,
        error: null,
      }));
    } catch (err) {
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
    if (!bridge || !canvas || !isEngineInitialized) {
      return;
    }

    let mounted = true;

    const initGraphics = async () => {
      try {
        setGraphicsError(null);

        // Call Rust's initialize_graphics_with_backend()
        await bridge.initialize_graphics_with_backend(
          canvas,
          selectedBackend,
        );

        if (mounted) {
          setIsGraphicsReady(true);
        }
      } catch (err) {
        if (mounted) {
          const errorMessage = err instanceof Error ? err.message : String(err);
          setGraphicsError(errorMessage);
          setIsGraphicsReady(false);
        }
      }
    };

    initGraphics();

    return () => {
      mounted = false;
    };
  }, [bridge, canvas, selectedBackend, isEngineInitialized]);

  return { isGraphicsReady, graphicsError };
}

/**
 * Hook that combines backend detection and initialization
 */
export function useBackend(
  bridge: WasmBridge | null,
  canvas: HTMLCanvasElement | null,
  isEngineInitialized: boolean,
) {
  const detectionState = useBackendDetection(bridge);
  const { isGraphicsReady, graphicsError } = useBackendInitialization(
    bridge,
    canvas,
    detectionState.selectedBackend,
    isEngineInitialized,
  );

  const [preferredBackend, setPreferredBackend] = useState<
    GraphicsBackend
  >("webgl2");

  // Update preferred when detection changes
  useEffect(() => {
    if (detectionState.availableBackends) {
      setPreferredBackend(detectionState.availableBackends.preferred);
    }
  }, [detectionState.availableBackends]);

  const selectBackend = useCallback((backend: GraphicsBackend) => {
    setPreferredBackend(backend);
  }, []);

  return {
    ...detectionState,
    selectedBackend: preferredBackend,
    selectBackend,
    isGraphicsReady,
    graphicsError,
  };
}
