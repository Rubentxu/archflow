/**
 * Hook for loading and initializing the ArchFlow WASM bridge
 *
 * This hook manages the lifecycle of the WASM module and provides
 * access to the WasmBridge instance for all other hooks.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type { WasmBridge, UseWasmBridgeReturn } from "../types/wasm";

/**
 * Hook to load and initialize the ArchFlow WASM bridge
 *
 * @returns Object containing bridge instance, loading state, and initialization function
 *
 * @example
 * ```typescript
 * const { bridge, isLoaded, isInitialized, error, initialize } = useArchFlowWasm();
 *
 * useEffect(() => {
 *   if (isLoaded && !isInitialized) {
 *     initialize(800, 600);
 *   }
 * }, [isLoaded, isInitialized]);
 * ```
 */
export function useArchFlowWasm(): UseWasmBridgeReturn {
  const [bridge, setBridge] = useState<WasmBridge | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const initRef = useRef<Promise<void> | null>(null);

  // Load WASM module on mount
  useEffect(() => {
    let mounted = true;

    const loadWasm = async () => {
      try {
        // Dynamically import the WASM module
        // The wasm-bindgen creates a JS module that loads the WASM file
        // @ts-ignore - Dynamic import for WASM module
        const wasmModule = await import("@archflow/web");

        if (!mounted) return;

        // Create bridge instance
        const newBridge = new wasmModule.WasmBridge();
        setBridge(newBridge);
        setIsLoaded(true);
      } catch (err) {
        // WASM module not yet built - this is expected during development
        if (mounted) {
          console.debug("WASM module not loaded yet:", err);
          setError(null); // Don't set error for missing WASM during dev
        }
      }
    };

    loadWasm();

    return () => {
      mounted = false;
    };
  }, []);

  // Initialize the engine
  const initialize = useCallback(
    async (width: number, height: number) => {
      if (!bridge) {
        setError(new Error("WASM bridge not loaded"));
        return;
      }

      // Prevent multiple simultaneous initializations
      if (initRef.current) {
        return initRef.current;
      }

      const initPromise = (async () => {
        try {
          bridge.initialize(width, height);
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
 *
 * This provides direct access to the SharedArrayBuffer for high-performance
 * input handling without marshalling overhead.
 *
 * @returns Object with input buffer pointer and utilities
 *
 * @example
 * ```typescript
 * const { bufferPtr, bufferSize, writeEvent } = useInputBuffer(bridge);
 * ```
 */
export function useInputBuffer(bridge: WasmBridge | null) {
  const [bufferPtr, setBufferPtr] = useState<number>(0);
  const [bufferSize, setBufferSize] = useState<number>(0);

  useEffect(() => {
    if (!bridge) return;

    try {
      const ptr = bridge.getInputBufferPtr();
      const size = bridge.getInputBufferSize();
      setBufferPtr(ptr);
      setBufferSize(size);
    } catch (err) {
      console.error("Failed to get input buffer:", err);
    }
  }, [bridge]);

  // Write event directly to SharedArrayBuffer
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
        bridge.pushInputEvent(eventType, x, y, buttons, modifiers);
        return true;
      } catch (err) {
        console.error("Failed to write input event:", err);
        return false;
      }
    },
    [bridge],
  );

  return {
    bufferPtr,
    bufferSize,
    writeEvent,
  };
}

/**
 * Hook for the main animation loop
 *
 * Manages the requestAnimationFrame loop and ticks the engine.
 *
 * @param bridge - The WASM bridge instance
 * @param enabled - Whether to run the loop
 * @returns Frame timing information
 *
 * @example
 * ```typescript
 * const { fps, frameTime, isRunning, start, stop } = useAnimationLoop(bridge);
 * ```
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
      if (!bridge) return;

      try {
        bridge.tick(timestamp);

        // Calculate FPS every second
        frameCountRef.current++;
        if (timestamp - lastFpsUpdateRef.current >= 1000) {
          setFps(frameCountRef.current);
          frameCountRef.current = 0;
          lastFpsUpdateRef.current = timestamp;
        }

        // Calculate frame time
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

  // Auto-start/stop based on enabled flag
  useEffect(() => {
    if (enabled && bridge && !isRunning) {
      start();
    } else if (!enabled && isRunning) {
      stop();
    }
  }, [enabled, bridge, isRunning, start, stop]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  return {
    isRunning,
    fps,
    frameTime,
    start,
    stop,
    tick,
  };
}
