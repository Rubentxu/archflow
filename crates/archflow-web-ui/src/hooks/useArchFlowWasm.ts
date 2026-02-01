/**
 * React hook for loading ArchFlow WASM module
 *
 * This hook handles the asynchronous loading of the WebAssembly module
 * and provides the initialized WASM bridge and logic mapping table.
 */

import { useEffect, useState } from "react";

// Use a global variable approach for WASM loading
declare global {
  interface Window {
    ArchFlowWasm?: {
      WasmBridge: any;
      LogicMappingTableWasm: any;
      SensorType: any;
      ActuatorType: any;
      Controller: any;
      ControllerType: any;
      SignalByteWasm: any;
    };
  }
}

export interface UseArchFlowWasmResult {
  /** Whether WASM is currently loading */
  isLoading: boolean;
  /** Error that occurred during loading, if any */
  error: Error | null;
  /** Whether WASM is loaded and ready */
  isReady: boolean;
  /** Initialize the engine with canvas dimensions */
  initializeEngine: (width: number, height: number) => Promise<void>;
  /** Get WASM module from window */
  getWasm: () => any;
  /** Create a new LogicMappingTable */
  createLogicMappingTable: () => any;
  /** Get the LogicMappingTable constructor */
  getLogicMappingTable: () => any;
  /** Get SensorType enum */
  getSensorType: () => any;
  /** Get ActuatorType enum */
  getActuatorType: () => any;
  /** Get Controller class */
  getController: () => any;
}

/**
 * Hook to load and initialize ArchFlow WASM module
 *
 * @example
 * ```tsx
 * function App() {
 *   const { isLoading, error, isReady, initializeEngine } = useArchFlowWasm()
 *
 *   if (isLoading) return <div>Loading...</div>
 *   if (error) return <div>Error: {error.message}</div>
 *
 *   return (
 *     <Canvas onReady={(canvas) => {
 *       initializeEngine(canvas.width, canvas.height)
 *     }} />
 *   )
 * }
 * ```
 */
export function useArchFlowWasm(): UseArchFlowWasmResult {
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadWasm() {
      try {
        setIsLoading(true);
        setError(null);

        // Load the WASM module script directly
        const script = document.createElement("script");
        script.src = "/crates/archflow-web/pkg/archflow_web.js";
        script.type = "module";

        script.onload = () => {
          if (!cancelled) {
            setIsReady(true);
            setIsLoading(false);
          }
        };

        script.onerror = () => {
          if (!cancelled) {
            setError(new Error("Failed to load WASM script"));
            setIsLoading(false);
          }
        };

        document.head.appendChild(script);
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err : new Error(String(err)));
          setIsLoading(false);
        }
      }
    }

    loadWasm();

    return () => {
      cancelled = true;
    };
  }, []);

  /**
   * Initialize the ArchFlow engine with the given canvas dimensions
   */
  const initializeEngine = async (
    width: number,
    height: number,
  ): Promise<void> => {
    if (!window.ArchFlowWasm) {
      throw new Error("WASM not loaded yet");
    }

    const bridge = new window.ArchFlowWasm.WasmBridge();
    await bridge.initialize(width, height);
  };

  /**
   * Get the WASM module from window
   */
  const getWasm = () => window.ArchFlowWasm;

  /**
   * Create a new LogicMappingTable instance
   */
  const createLogicMappingTable = () => {
    if (!window.ArchFlowWasm) {
      throw new Error("WASM not loaded yet");
    }
    return new window.ArchFlowWasm.LogicMappingTableWasm();
  };

  /**
   * Get the LogicMappingTable constructor
   */
  const getLogicMappingTable = () => {
    if (!window.ArchFlowWasm) {
      return undefined;
    }
    return window.ArchFlowWasm.LogicMappingTableWasm;
  };

  /**
   * Get SensorType enum
   */
  const getSensorType = () => {
    if (!window.ArchFlowWasm) {
      return undefined;
    }
    return window.ArchFlowWasm.SensorType;
  };

  /**
   * Get ActuatorType enum
   */
  const getActuatorType = () => {
    if (!window.ArchFlowWasm) {
      return undefined;
    }
    return window.ArchFlowWasm.ActuatorType;
  };

  /**
   * Get Controller class
   */
  const getController = () => {
    if (!window.ArchFlowWasm) {
      return undefined;
    }
    return window.ArchFlowWasm.Controller;
  };

  return {
    isLoading,
    error,
    isReady,
    initializeEngine,
    getWasm,
    createLogicMappingTable,
    getLogicMappingTable,
    getSensorType,
    getActuatorType,
    getController,
  };
}
