/**
 * Hook for loading and initializing the ArchFlow WASM bridge
 *
 * This hook manages the lifecycle of the WASM module and provides
 * access to the WasmBridge instance for all other hooks.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 *
 * Updated to use vite-plugin-wasm for proper WASM integration with Vite.
 * The WASM module is imported statically from the pkg directory generated
 * by wasm-pack with --target web.
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type { UseWasmBridgeReturn } from "../types/wasm";

// Static import from src/wasm directory (Vite processes files in src as modules)
import init, { WasmBridge } from "../wasm/archflow_web.js";

// Track if WASM has been initialized
let isWasmInitialized = false;
let initPromise: Promise<void> | null = null;

/**
 * Initialize WASM module (singleton pattern)
 */
async function initializeWasm(): Promise<void> {
  if (isWasmInitialized) {
    return;
  }

  if (initPromise) {
    return initPromise;
  }

  initPromise = (async () => {
    try {
      await init();
      isWasmInitialized = true;
    } catch (err) {
      initPromise = null;
      throw err;
    }
  })();

  return initPromise;
}

/**
 * Hook to load and initialize the ArchFlow WASM bridge
 */
// Context for sharing WASM bridge instance
import { createContext, useContext, ReactNode } from "react";

const WasmContext = createContext<UseWasmBridgeReturn | null>(null);

/**
 * Provider component that manages the WASM bridge lifecycle
 */
export function ArchFlowWasmProvider({ children }: { children: ReactNode }) {
  const [bridge, setBridge] = useState<WasmBridge | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const initRef = useRef<Promise<void> | null>(null);

  // DEBUG: State changes
  useEffect(() => {
    console.log("[ArchFlowWasmProvider] State:", {
      hasBridge: !!bridge,
      isLoaded,
      isInitialized,
      hasError: !!error,
    });
  }, [bridge, isLoaded, isInitialized, error]);

  // Load WASM module on mount
  useEffect(() => {
    console.log("[ArchFlowWasmProvider] Mounting, starting WASM load...");
    let mounted = true;

    const loadWasm = async () => {
      try {
        console.log("[ArchFlowWasmProvider] Calling initializeWasm...");
        await initializeWasm();

        if (!mounted) {
          console.log("[ArchFlowWasmProvider] Aborted (unmounted)");
          return;
        }

        console.log("[ArchFlowWasmProvider] Creating WasmBridge instance...");
        const newBridge = new WasmBridge();
        setBridge(newBridge);
        setIsLoaded(true);
        setError(null);
        console.log("[ArchFlowWasmProvider] ✓ WASM loaded and bridge created");
      } catch (err) {
        if (mounted) {
          const errorMessage = err instanceof Error ? err.message : String(err);
          console.error("[ArchFlowWasmProvider] ✗ WASM load failed:", errorMessage);
          setError(
            new Error(
              `Failed to load WASM module: ${errorMessage}\n\n` +
              `Please build WASM first:\n` +
              `  cd crates/archflow-web && wasm-pack build --target web\n\n` +
              `Ensure COOP/COEP headers are configured.`
            )
          );
        }
      }
    };

    loadWasm();

    return () => {
      console.log("[ArchFlowWasmProvider] Unmounting");
      mounted = false;
    };
  }, []);

  const initialize = useCallback(
    async (width: number, height: number) => {
      console.log("[ArchFlowWasmProvider] initialize called:", {
        width,
        height,
        hasBridge: !!bridge,
      });

      if (!bridge) {
        const error = new Error("WASM bridge not loaded. Cannot initialize.");
        console.error("[ArchFlowWasmProvider] ✗", error.message);
        throw error;
      }

      if (initRef.current) {
        return initRef.current;
      }

      const initPromise = (async () => {
        try {
          console.log("[ArchFlowWasmProvider] Calling bridge.initialize...");
          bridge.initialize(width, height);
          setIsInitialized(true);
          setError(null);
          console.log("[ArchFlowWasmProvider] ✓ Bridge initialized");
        } catch (err) {
          console.error("[ArchFlowWasmProvider] ✗ Initialize failed:", err);
          setError(err instanceof Error ? err : new Error(String(err)));
          throw err;
        }
      })();

      initRef.current = initPromise;

      try {
        await initPromise;
      } finally {
        initRef.current = null;
      }
    },
    [bridge]
  );

  const value = {
    bridge,
    isLoaded,
    isInitialized,
    error,
    initialize,
  };

  return <WasmContext.Provider value={ value }> { children } </WasmContext.Provider>;
}

/**
 * Hook to consume the ArchFlow WASM bridge context
 */
export function useArchFlowWasm(): UseWasmBridgeReturn {
  const context = useContext(WasmContext);
  if (!context) {
    throw new Error("useArchFlowWasm must be used within an ArchFlowWasmProvider");
  }
  return context;
}

/**
 * Hook to get the input buffer pointer for SharedArrayBuffer communication
 */
export function useInputBuffer(bridge: WasmBridge | null) {
  const [bufferPtr, setBufferPtr] = useState<number>(0);
  const [bufferSize, setBufferSize] = useState<number>(0);

  useEffect(() => {
    if (!bridge) {
      throw new Error(
        "WASM bridge is required but not loaded for input buffer access",
      );
    }

    try {
      const ptr = bridge.get_input_buffer_ptr();
      const size = WasmBridge.get_input_buffer_size();
      setBufferPtr(ptr);
      setBufferSize(size);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      throw new Error(`Failed to get input buffer from WASM: ${errorMsg}`);
    }
  }, [bridge]);

  const writeEvent = useCallback(
    (
      eventType: number,
      x: number,
      y: number,
      buttons: number,
      modifiers: number,
    ) => {
      if (!bridge) {
        throw new Error(
          "WASM bridge is required but not loaded for input event writing",
        );
      }

      try {
        bridge.push_input_event(eventType, x, y, buttons, modifiers);
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to write input event to WASM: ${errorMsg}`);
      }
    },
    [bridge],
  );

  return { bufferPtr, bufferSize, writeEvent };
}

/**
 * Hook for the main animation loop
 */
export function useAnimationLoop(
  bridge: WasmBridge | null,
  enabled: boolean = true,
) {
  const [isRunning, setIsRunning] = useState(false);
  const [fps, setFps] = useState(0);
  const [frameTime, setFrameTime] = useState(0);
  const animationRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(0);
  const frameCountRef = useRef(0);
  const lastFpsUpdateRef = useRef(0);

  const tick = useCallback(
    (timestamp: number) => {
      if (!bridge) {
        throw new Error(
          "WASM bridge is required for animation loop but not loaded",
        );
      }

      try {
        bridge.tick(timestamp);

        frameCountRef.current++;
        if (timestamp - lastFpsUpdateRef.current >= 1000) {
          setFps(frameCountRef.current);
          frameCountRef.current = 0;
          lastFpsUpdateRef.current = timestamp;
        }

        const delta = timestamp - lastTimeRef.current;
        setFrameTime(delta);
        lastTimeRef.current = timestamp;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`Animation loop tick failed: ${errorMsg}`);
      }
    },
    [bridge],
  );

  const start = useCallback(() => {
    if (isRunning || !bridge) {
      if (!bridge) {
        throw new Error("Cannot start animation loop: WASM bridge not loaded");
      }
      return;
    }

    setIsRunning(true);
    lastTimeRef.current = performance.now();
    lastFpsUpdateRef.current = performance.now();
    frameCountRef.current = 0;

    const loop = (timestamp: number) => {
      if (!isRunning) return;
      tick(timestamp);
      animationRef.current = requestAnimationFrame(loop);
    };

    animationRef.current = requestAnimationFrame(loop);
  }, [bridge, isRunning, tick]);

  const stop = useCallback(() => {
    if (!isRunning) return;

    setIsRunning(false);
    if (animationRef.current) {
      cancelAnimationFrame(animationRef.current);
      animationRef.current = 0;
    }
  }, [isRunning]);

  useEffect(() => {
    if (enabled && bridge && !isRunning) {
      start();
    } else if (!enabled && isRunning) {
      stop();
    }
  }, [enabled, bridge, isRunning, start, stop]);

  useEffect(() => {
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  return { isRunning, fps, frameTime, start, stop, tick };
}
