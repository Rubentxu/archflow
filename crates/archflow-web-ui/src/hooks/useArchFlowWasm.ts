/**
 * Hook for loading and initializing the ArchFlow WASM bridge
 *
 * This hook manages the lifecycle of the WASM module and provides
 * access to the WasmBridge instance for all other hooks.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type { UseWasmBridgeReturn } from "../types/wasm";

// Lazy load WASM to avoid build issues
let wasmBridgeClass: unknown = null;

/**
 * Hook to load and initialize the ArchFlow WASM bridge
 */
export function useArchFlowWasm(): UseWasmBridgeReturn {
  const [bridge, setBridge] = useState<unknown>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const initRef = useRef<Promise<void> | null>(null);

  // Load WASM module on mount
  useEffect(() => {
    let mounted = true;

    const loadWasm = async () => {
      try {
        // Dynamic import - Vite will handle this at runtime
        const wasmModule = await import("@archflow/web");
        wasmBridgeClass = wasmModule.WasmBridge;

        if (!mounted) return;

        const newBridge = new (wasmBridgeClass as new () => unknown)();
        setBridge(newBridge);
        setIsLoaded(true);
      } catch (err) {
        // WASM not built yet - this is expected during initial setup
        if (mounted) {
          console.debug(
            "WASM module not loaded. Build WASM first with: cargo build -p archflow-web && wasm-pack build --target web",
          );
          setError(null);
        }
      }
    };

    loadWasm();

    return () => {
      mounted = false;
    };
  }, []);

  const initialize = useCallback(
    async (width: number, height: number) => {
      if (!bridge || !wasmBridgeClass) {
        setError(new Error("WASM bridge not loaded"));
        return;
      }

      if (initRef.current) {
        return initRef.current;
      }

      const initPromise = (async () => {
        try {
          (bridge as { initialize: (w: number, h: number) => void }).initialize(
            width,
            height,
          );
          setIsInitialized(true);
          setError(null);
        } catch (err) {
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
    [bridge],
  );

  return {
    bridge,
    isLoaded,
    isInitialized,
    error,
    initialize,
  };
}

/**
 * Hook to get the input buffer pointer for SharedArrayBuffer communication
 */
export function useInputBuffer(bridge: unknown) {
  const [bufferPtr, setBufferPtr] = useState<number>(0);
  const [bufferSize, setBufferSize] = useState<number>(0);

  useEffect(() => {
    if (!bridge) return;

    try {
      const ptr = (
        bridge as { getInputBufferPtr: () => number }
      ).getInputBufferPtr();
      const size = (
        bridge as { getInputBufferSize: () => number }
      ).getInputBufferSize();
      setBufferPtr(ptr);
      setBufferSize(size);
    } catch (err) {
      console.error("Failed to get input buffer:", err);
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
      if (!bridge) return false;

      try {
        (
          bridge as {
            pushInputEvent: (
              e: number,
              x: number,
              y: number,
              b: number,
              m: number,
            ) => void;
          }
        ).pushInputEvent(eventType, x, y, buttons, modifiers);
        return true;
      } catch (err) {
        console.error("Failed to write input event:", err);
        return false;
      }
    },
    [bridge],
  );

  return { bufferPtr, bufferSize, writeEvent };
}

/**
 * Hook for the main animation loop
 */
export function useAnimationLoop(bridge: unknown, enabled: boolean = true) {
  const [isRunning, setIsRunning] = useState(false);
  const [fps, setFps] = useState(0);
  const [frameTime, setFrameTime] = useState(0);
  const animationRef = useRef<number>(0);
  const lastTimeRef = useRef<number>(0);
  const frameCountRef = useRef(0);
  const lastFpsUpdateRef = useRef(0);

  const tick = useCallback(
    (timestamp: number) => {
      if (!bridge) return;

      try {
        (bridge as { tick: (t: number) => void }).tick(timestamp);

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
        console.error("Tick failed:", err);
      }
    },
    [bridge],
  );

  const start = useCallback(() => {
    if (isRunning || !bridge) return;

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
