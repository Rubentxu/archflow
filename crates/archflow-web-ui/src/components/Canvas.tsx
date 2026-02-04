/**
 * Canvas Component - WASM Rendering
 *
 * Main canvas component that delegates ALL rendering to WASM.
 * The canvas element is only used as a display surface; all actual drawing
 * is handled by the Rust/WASM engine through the typed bridge.
 *
 * This component:
 * - Handles pointer events and forwards them to WASM
 * - Manages canvas lifecycle and resize
 * - Delegates rendering entirely to WASM
 * - Provides drag & drop integration
 *
 * Architecture Reference: EPIC-WEB-003, EPIC-WEB-009, EPIC-WEB-010
 * WASM-First: All rendering happens in Rust, NOT in JavaScript
 */

import { useRef, useEffect, useState, useCallback, memo } from "react";
import { useCanvasStore } from "../store/useCanvasStore";
import { useUIStore } from "../store/useUIStore";
import { useDragAndDrop } from "../hooks/useDragAndDrop";
import { useArchFlowWasm } from "../hooks/useArchFlowWasm";
import { useBackend } from "../hooks/useBackend";
import { cn } from "../utils/cn";
import { usePerformanceMonitor } from "../utils/performance";

/**
 * Canvas component props
 */
interface CanvasProps {
  className?: string;
  onPointerDown?: (position: { x: number; y: number }, buttons: number) => void;
  onPointerMove?: (position: { x: number; y: number }, buttons: number) => void;
  onPointerUp?: (position: { x: number; y: number }, buttons: number) => void;
  onWheel?: (position: { x: number; y: number }, delta: number) => void;
}

/**
 * Canvas component with WebGPU rendering and drag & drop support
 */
export default memo(function Canvas({
  className,
  onPointerDown,
  onPointerMove,
  onPointerUp,
  onWheel,
}: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  // DEBUG: Component lifecycle
  useEffect(() => {
    console.log(
      "[Canvas] Component mounted, canvasRef.current:",
      canvasRef.current,
    );
    return () => console.log("[Canvas] Component unmounted");
  }, []);

  // DEBUG: Canvas ref changes
  useEffect(() => {
    console.log("[Canvas] canvasRef.current changed:", canvasRef.current);
  }, [canvasRef.current]);

  // Use specific selectors to prevent re-renders when other store parts change
  const camera = useCanvasStore((state) => state.camera);
  const showGrid = useCanvasStore((state) => state.showGrid);
  const zoomIn = useCanvasStore((state) => state.zoomIn);
  const pan = useCanvasStore((state) => state.pan);
  const activeTool = useUIStore((state) => state.activeTool);

  // WASM bridge access
  const { bridge, isLoaded: wasmLoaded } = useArchFlowWasm();
  const { CanvasDroppable, DragOverlayContent, dragState } = useDragAndDrop();

  // Backend for graphics initialization - useBackend handles detection and initialization
  const backend = useBackend(bridge, canvasRef.current, true);
  const { isGraphicsReady, graphicsError, selectedBackend } = backend;

  // DEBUG: Backend state
  useEffect(() => {
    console.log("[Canvas] Backend state:", {
      isGraphicsReady,
      graphicsError,
      selectedBackend,
      wasmLoaded,
      bridgeExists: !!bridge,
      isInitialized,
    });
  }, [
    isGraphicsReady,
    graphicsError,
    selectedBackend,
    wasmLoaded,
    bridge,
    isInitialized,
  ]);

  // Performance monitoring in development
  usePerformanceMonitor("Canvas");

  /**
   * Get canvas position from pointer or wheel event
   */
  const getCanvasPosition = useCallback(
    (event: React.PointerEvent | React.WheelEvent) => {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (!rect) return { x: 0, y: 0 };

      const dpr = window.devicePixelRatio || 1;
      return {
        x: (event.clientX - rect.left) * dpr,
        y: (event.clientY - rect.top) * dpr,
      };
    },
    [],
  );

  /**
   * Handle pointer down event - forward to WASM
   */
  const handlePointerDown = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      const position = getCanvasPosition(event);

      // Forward to callbacks
      onPointerDown?.(position, event.buttons);

      // Forward to WASM if available
      if (bridge && wasmLoaded) {
        try {
          (
            bridge as {
              push_input_event: (
                e: number,
                x: number,
                y: number,
                b: number,
                m: number,
              ) => void;
            }
          ).push_input_event(
            0, // event_type: 0 = pointer down
            position.x,
            position.y,
            event.buttons,
            0, // modifiers: none
          );
        } catch (err) {
          console.error("Failed to send pointer down to WASM:", err);
        }
      }
    },
    [getCanvasPosition, onPointerDown, bridge, wasmLoaded],
  );

  /**
   * Handle pointer move event - forward to WASM
   */
  const handlePointerMove = useCallback(
    (event: React.PointerEvent) => {
      const position = getCanvasPosition(event);
      onPointerMove?.(position, event.buttons);

      // Forward to WASM if available
      if (bridge && wasmLoaded) {
        try {
          (
            bridge as {
              push_input_event: (
                e: number,
                x: number,
                y: number,
                b: number,
                m: number,
              ) => void;
            }
          ).push_input_event(
            1, // event_type: 1 = pointer move
            position.x,
            position.y,
            event.buttons,
            0, // modifiers: none
          );
        } catch (err) {
          console.error("Failed to send pointer move to WASM:", err);
        }
      }
    },
    [getCanvasPosition, onPointerMove, bridge, wasmLoaded],
  );

  /**
   * Handle pointer up event - forward to WASM
   */
  const handlePointerUp = useCallback(
    (event: React.PointerEvent) => {
      const position = getCanvasPosition(event);
      onPointerUp?.(position, event.buttons);

      // Forward to WASM if available
      if (bridge && wasmLoaded) {
        try {
          (
            bridge as {
              push_input_event: (
                e: number,
                x: number,
                y: number,
                b: number,
                m: number,
              ) => void;
            }
          ).push_input_event(
            2, // event_type: 2 = pointer up
            position.x,
            position.y,
            event.buttons,
            0, // modifiers: none
          );
        } catch (err) {
          console.error("Failed to send pointer up to WASM:", err);
        }
      }
    },
    [getCanvasPosition, onPointerUp, bridge, wasmLoaded],
  );

  /**
   * Handle wheel event - forward to WASM
   */
  const handleWheel = useCallback(
    (event: React.WheelEvent) => {
      event.preventDefault();
      const position = getCanvasPosition(event);

      // Handle zoom/pan through store
      if (event.ctrlKey || event.metaKey) {
        const factor = event.deltaY > 0 ? 0.9 : 1.1;
        zoomIn(factor);
      } else {
        pan(event.deltaX, event.deltaY);
      }

      onWheel?.(position, Math.abs(event.deltaY));

      // Forward to WASM if available
      if (bridge && wasmLoaded) {
        try {
          (
            bridge as {
              push_input_event: (
                e: number,
                x: number,
                y: number,
                b: number,
                m: number,
              ) => void;
            }
          ).push_input_event(
            3, // event_type: 3 = wheel
            position.x,
            position.y,
            event.deltaY,
            event.ctrlKey || event.metaKey ? 1 : 0, // modifiers: ctrl/meta
          );
        } catch (err) {
          console.error("Failed to send wheel to WASM:", err);
        }
      }
    },
    [getCanvasPosition, zoomIn, pan, onWheel, bridge, wasmLoaded],
  );

  /**
   * Get cursor style based on active tool
   */
  const getCursor = useCallback(() => {
    if (camera.zoom !== 1) return "grab";
    switch (activeTool) {
      case "select":
        return "default";
      case "pan":
        return "grab";
      case "rectangle":
      case "circle":
      case "triangle":
      case "diamond":
      case "connection":
        return "crosshair";
      case "text":
        return "text";
      case "delete":
        return "not-allowed";
      default:
        return "default";
    }
  }, [camera.zoom, activeTool]);

  // Resize observer for responsive canvas
  useEffect(() => {
    if (!containerRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        const canvas = canvasRef.current;
        if (canvas) {
          const dpr = window.devicePixelRatio || 1;
          canvas.style.width = `${width}px`;
          canvas.style.height = `${height}px`;
          canvas.width = width * dpr;
          canvas.height = height * dpr;

          // Notify WASM of resize if ready
          if (bridge && wasmLoaded && isInitialized) {
            try {
              (
                bridge as {
                  resize: (w: number, h: number) => void;
                }
              ).resize(canvas.width, canvas.height);
            } catch (err) {
              console.error("Failed to resize WASM engine:", err);
            }
          }
        }
      }
    });

    resizeObserver.observe(containerRef.current);
    return () => resizeObserver.disconnect();
  }, [bridge, wasmLoaded, isInitialized]);

  // Initialize renderer with backend selection
  useEffect(() => {
    console.log("[Canvas] Initialize graphics effect triggered:", {
      hasCanvas: !!canvasRef.current,
      hasBridge: !!bridge,
      wasmLoaded,
      selectedBackend,
    });

    if (!canvasRef.current || !bridge || !wasmLoaded) {
      console.log("[Canvas] Skipping graphics init - missing dependencies");
      return;
    }

    const initializeGraphics = async () => {
      const canvas = canvasRef.current;
      if (!canvas) {
        console.log("[Canvas] Canvas ref lost during init");
        return;
      }

      console.log("[Canvas] Starting graphics initialization...", {
        canvasWidth: canvas.width,
        canvasHeight: canvas.height,
        backend: selectedBackend,
      });

      try {
        // Initialize WASM engine
        // canvas.width/height is already scaled by DPR in the ResizeObserver
        console.log("[Canvas] Calling bridge.initialize...");
        (
          bridge as {
            initialize: (w: number, h: number) => void;
          }
        ).initialize(canvas.width, canvas.height);

        // Initialize graphics with selected backend (WebGL2 by default)
        console.log(
          "[Canvas] Calling bridge.initialize_graphics_with_backend...",
        );
        await (
          bridge as {
            initialize_graphics_with_backend: (
              c: HTMLCanvasElement,
              backend: string,
            ) => Promise<void>;
          }
        ).initialize_graphics_with_backend(canvas, selectedBackend);

        setIsInitialized(true);
        console.log(
          `[Canvas] ✓ Graphics initialized with ${selectedBackend} backend`,
        );
      } catch (err) {
        console.error("[Canvas] ✗ Failed to initialize graphics:", err);
        const errMsg = err instanceof Error ? err.message : String(err);
        console.error("[Canvas] Error details:", errMsg);
      }
    };

    initializeGraphics();
  }, [bridge, wasmLoaded, selectedBackend]);

  // Sync active tool with WASM bridge
  useEffect(() => {
    console.log("[Canvas] Tool sync effect:", {
      hasBridge: !!bridge,
      wasmLoaded,
      isInitialized,
      activeTool,
    });

    if (!bridge || !wasmLoaded || !isInitialized) {
      console.log("[Canvas] Skipping tool sync - not ready");
      return;
    }

    try {
      console.log(`[Canvas] Setting tool to: ${activeTool}`);
      (
        bridge as {
          set_tool: (tool: string) => void;
        }
      ).set_tool(activeTool);
      console.log(`[Canvas] ✓ Tool set to: ${activeTool}`);
    } catch (err) {
      console.error("[Canvas] ✗ Failed to set tool in WASM:", err);
    }
  }, [bridge, wasmLoaded, isInitialized, activeTool]);

  // WASM-driven render loop
  useEffect(() => {
    console.log("[Canvas] Render loop effect:", {
      hasCanvas: !!canvasRef.current,
      hasBridge: !!bridge,
      wasmLoaded,
      isInitialized,
    });

    if (!canvasRef.current || !bridge || !wasmLoaded || !isInitialized) {
      console.log("[Canvas] Skipping render loop - not ready");
      return;
    }

    console.log("[Canvas] Starting render loop...");
    let animationId: number;
    let lastTime = performance.now();
    let frameCount = 0;

    const render = (timestamp: number) => {
      try {
        // Call WASM tick function to process input and render
        (
          bridge as {
            tick: (t: number) => void;
          }
        ).tick(timestamp);

        // Log every 300 frames (roughly 5 seconds at 60fps) to reduce noise
        frameCount++;
        if (frameCount % 300 === 0) {
          console.log(
            `[Canvas] Render loop active, frame ${frameCount}, fps: ${(1000 / (timestamp - lastTime)).toFixed(1)}`,
          );
        }

        lastTime = timestamp;
        animationId = requestAnimationFrame(render);
      } catch (err) {
        console.error("[Canvas] ✗ WASM render tick failed:", err);
        // Continue animation loop even if tick fails
        animationId = requestAnimationFrame(render);
      }
    };

    animationId = requestAnimationFrame(render);
    console.log("[Canvas] ✓ Render loop started");

    return () => {
      if (animationId) {
        cancelAnimationFrame(animationId);
        console.log("[Canvas] Render loop stopped");
      }
    };
  }, [
    bridge,
    wasmLoaded,
    isInitialized,
    camera.x,
    camera.y,
    camera.zoom,
    showGrid,
    dragState,
  ]);

  return (
    <CanvasDroppable>
      {({ isOver, setNodeRef }) => (
        <div
          ref={setNodeRef}
          className={cn(
            "w-full h-full relative",
            isOver && "ring-2 ring-primary/50",
            className,
          )}
        >
          <div ref={containerRef} className="w-full h-full absolute inset-0">
            <canvas
              ref={canvasRef}
              className="touch-none block"
              style={{ cursor: getCursor(), touchAction: "none" }}
              onPointerDown={handlePointerDown}
              onPointerMove={handlePointerMove}
              onPointerUp={handlePointerUp}
              onPointerLeave={handlePointerUp}
              onWheel={handleWheel}
            />
          </div>

          {!isInitialized && (
            <div className="absolute inset-0 flex items-center justify-center bg-background-dark/80">
              <div className="flex flex-col items-center gap-3">
                <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
                <span className="text-sm text-text-secondary">
                  Initializing renderer...
                </span>
                {graphicsError && (
                  <div className="text-xs text-red-400 mt-2 max-w-md text-center">
                    Error: {graphicsError}
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Drag overlay for visual feedback */}
          <DragOverlayContent />
        </div>
      )}
    </CanvasDroppable>
  );
});
