/**
 * Canvas Component - WASM Rendering with Behavior System Integration
 *
 * Main canvas component that delegates ALL rendering to WASM.
 * Uses the Behavior System for entity interactions (hover, select, drag, etc.)
 *
 * This component:
 * - Handles pointer events and forwards them to the Behavior System
 * - Manages canvas lifecycle and resize
 * - Delegates rendering entirely to WASM
 * - Provides drag & drop integration from sidebar
 *
 * Architecture Reference: EPIC-WEB-003, EPIC-WEB-009, EPIC-WEB-010
 * WASM-First: All rendering happens in Rust, NOT in JavaScript
 * Behavior-Driven: Interactions are handled via the Behavior System
 */

import { useRef, useEffect, useState, useCallback, memo } from "react";
import { useCanvasStore } from "../store/useCanvasStore";
import { useUIStore } from "../store/useUIStore";
import { useDragAndDrop } from "../hooks/useDragAndDrop";
import { useArchFlowWasm } from "../hooks/useArchFlowWasm.tsx";
import { useBackend } from "../hooks/useBackend";
import { useContextMenuStore } from "../store/useContextMenuStore";
import { useSelectionStore } from "../store/useSelectionStore";
import { useBehaviorSystem } from "../hooks/useBehaviorSystem";
import { cn } from "../utils/cn";
import { usePerformanceMonitor } from "../utils/performance";
import { ContextMenu } from "./ContextMenu";
import type { Vec2 } from "../types/wasm";

/**
 * Canvas component props
 */
interface CanvasProps {
  className?: string;
}

/**
 * Extract keyboard modifiers as bitmask
 */
const getModifiers = (
  event:
    | PointerEvent
    | React.PointerEvent
    | React.WheelEvent
    | WheelEvent
    | React.MouseEvent
    | MouseEvent,
) => {
  let mods = 0;
  if (event.shiftKey) mods |= 0x01;
  if (event.ctrlKey) mods |= 0x02;
  if (event.altKey) mods |= 0x04;
  if (event.metaKey) mods |= 0x08;
  return mods;
};

/**
 * Canvas component with WebGPU rendering and behavior-driven interactions
 */
export default memo(function Canvas({ className }: CanvasProps) {
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
  const zoomIn = useCanvasStore((state) => state.zoomIn);
  const pan = useCanvasStore((state) => state.pan);
  const activeTool = useUIStore((state) => state.activeTool);

  // WASM bridge access
  const { bridge, isLoaded: wasmLoaded, initialize } = useArchFlowWasm();
  const { CanvasDroppable, DragOverlayContent, dragState } = useDragAndDrop();

  // Behavior system integration
  const behaviorSystem = useBehaviorSystem({
    defaultBehaviors: ["hover", "select", "drag"],
    dragSnap: 8,
    onSelectionChange: (ids) => {
      console.log("[Canvas] Selection changed:", ids);
    },
    onHoverChange: (entityId) => {
      console.log("[Canvas] Hover changed:", entityId);
    },
    debug: false,
  });

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
   * Supports both React synthetic events and native DOM events
   */
  const getCanvasPosition = useCallback(
    (
      event: React.PointerEvent | React.WheelEvent | PointerEvent | WheelEvent,
    ): Vec2 => {
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
   * Handle pointer down event - forward to Behavior System
   */
  const handlePointerDown = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      const position = getCanvasPosition(event);

      // Convert pointer button to mouse button (0=left, 1=right, 2=middle)
      const button = event.button;
      const modifiers = getModifiers(event);

      // Forward to Behavior System handlers
      behaviorSystem.handlers.onPointerDown(position, button, modifiers);

      // Also forward to WASM for legacy support and rendering
      if (bridge && wasmLoaded) {
        try {
          bridge.on_mouse_down(position.x, position.y, button, modifiers);
        } catch (err) {
          console.error("Failed to send pointer down to WASM:", err);
        }
      }
    },
    [getCanvasPosition, behaviorSystem, bridge, wasmLoaded],
  );

  /**
   * Handle pointer move event - forward to Behavior System
   */
  const handlePointerMove = useCallback(
    (event: React.PointerEvent) => {
      const position = getCanvasPosition(event);
      const buttons = event.buttons;
      const modifiers = getModifiers(event);

      // Forward to Behavior System handlers
      behaviorSystem.handlers.onPointerMove(position, buttons, modifiers);

      // Also forward to WASM for legacy support
      if (bridge && wasmLoaded) {
        try {
          bridge.on_mouse_move(
            position.x,
            position.y,
            event.buttons,
            getModifiers(event),
          );
        } catch (err) {
          console.error("Failed to send pointer move to WASM:", err);
        }
      }
    },
    [getCanvasPosition, behaviorSystem, bridge, wasmLoaded],
  );

  /**
   * Handle pointer up event - forward to Behavior System
   */
  const handlePointerUp = useCallback(
    (event: React.PointerEvent) => {
      const position = getCanvasPosition(event);
      const button = event.button;
      const modifiers = getModifiers(event);

      // Forward to Behavior System handlers
      behaviorSystem.handlers.onPointerUp(position, button, modifiers);

      // Also forward to WASM for legacy support
      if (bridge && wasmLoaded) {
        try {
          bridge.on_mouse_up(position.x, position.y, button, modifiers);
        } catch (err) {
          console.error("Failed to send pointer up to WASM:", err);
        }
      }
    },
    [getCanvasPosition, behaviorSystem, bridge, wasmLoaded],
  );

  /**
   * Handle context menu (right-click) - open context menu
   */
  const handleContextMenu = useCallback((event: React.MouseEvent) => {
    event.preventDefault();

    const position = { x: event.clientX, y: event.clientY };
    const { selectedIds } = useSelectionStore.getState();

    // Determine menu type based on selection
    let menuType: "canvas" | "entity" | "selection" = "canvas";
    if (selectedIds.length > 0) {
      menuType = selectedIds.length === 1 ? "entity" : "selection";
    }

    // Open context menu
    useContextMenuStore.getState().open(menuType, position, {
      entityId: selectedIds.length === 1 ? selectedIds[0] : undefined,
      entityIds: selectedIds,
    });
  }, []);

  /**
   * Handle wheel event - forward to WASM (behavior system doesn't handle wheel)
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
            getModifiers(event),
          );
        } catch (err) {
          console.error("Failed to send wheel to WASM:", err);
        }
      }
    },
    [getCanvasPosition, zoomIn, pan, bridge, wasmLoaded],
  );

  /**
   * Get cursor style based on active tool and behavior state
   */
  const getCursor = useCallback(() => {
    if (camera.zoom !== 1) return "grab";

    // Check behavior system state
    if (behaviorSystem.state.isMarqueeing) return "crosshair";
    if (behaviorSystem.state.draggedEntityId) return "grabbing";

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
  }, [camera.zoom, activeTool, behaviorSystem.state]);

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

  // Native DOM event listeners for events that might not go through React
  // This ensures programmatic events and direct DOM manipulation also work
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !bridge || !wasmLoaded) return;

    // Native pointer down handler
    const nativePointerDown = (event: PointerEvent) => {
      console.log("🖱️ Native pointerdown handler executed", {
        clientX: event.clientX,
        clientY: event.clientY,
        button: event.button,
        isTrusted: event.isTrusted,
      });

      const position = getCanvasPosition(event);
      const button = event.button;
      const modifiers = getModifiers(event);

      console.log("  → Canvas position:", position, "button:", button);

      // Forward to behavior system
      behaviorSystem.handlers.onPointerDown(position, button, modifiers);

      try {
        bridge.on_mouse_down(position.x, position.y, button, modifiers);
        console.log("  ✓ bridge.on_mouse_down called successfully");
      } catch (err) {
        console.error("Native pointer down failed:", err);
      }
    };

    // Native pointer move handler
    const nativePointerMove = (event: PointerEvent) => {
      const position = getCanvasPosition(event);
      const modifiers = getModifiers(event);

      // Forward to behavior system
      behaviorSystem.handlers.onPointerMove(position, event.buttons, modifiers);

      try {
        bridge.on_mouse_move(position.x, position.y, event.buttons, modifiers);
      } catch (err) {
        console.error("Native pointer move failed:", err);
      }
    };

    // Native pointer up handler
    const nativePointerUp = (event: PointerEvent) => {
      console.log("🖱️ Native pointerup handler executed", {
        clientX: event.clientX,
        clientY: event.clientY,
        button: event.button,
      });

      const position = getCanvasPosition(event);
      const button = event.button;
      const modifiers = getModifiers(event);

      // Forward to behavior system
      behaviorSystem.handlers.onPointerUp(position, button, modifiers);

      try {
        bridge.on_mouse_up(position.x, position.y, button, modifiers);
        console.log("  ✓ bridge.on_mouse_up called successfully");
      } catch (err) {
        console.error("Native pointer up failed:", err);
      }
    };

    // Add native event listeners
    console.log("📌 Registering native event listeners on canvas", {
      hasCanvas: !!canvas,
      hasBridge: !!bridge,
      wasmLoaded,
    });

    canvas.addEventListener("pointerdown", nativePointerDown);
    canvas.addEventListener("pointermove", nativePointerMove);
    canvas.addEventListener("pointerup", nativePointerUp);

    console.log("✓ Native event listeners registered");

    // Cleanup
    return () => {
      console.log("🧹 Removing native event listeners");
      canvas.removeEventListener("pointerdown", nativePointerDown);
      canvas.removeEventListener("pointermove", nativePointerMove);
      canvas.removeEventListener("pointerup", nativePointerUp);
    };
  }, [bridge, wasmLoaded, getCanvasPosition, behaviorSystem]);

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
        console.log("[Canvas] Calling initialize...");
        await initialize(canvas.width, canvas.height);

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
  }, [bridge, wasmLoaded, selectedBackend, initialize]);

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
    if (!canvasRef.current || !bridge || !wasmLoaded || !isInitialized) {
      return;
    }

    let animationId: number;

    const render = (timestamp: number) => {
      try {
        // Process behavior system updates
        behaviorSystem.actions.update?.(timestamp);

        // Call WASM tick function to process input and render
        (
          bridge as {
            tick: (t: number) => void;
          }
        ).tick(timestamp);

        animationId = requestAnimationFrame(render);
      } catch (err) {
        console.error("[Canvas] ✗ WASM render tick failed:", err);
        animationId = requestAnimationFrame(render);
      }
    };

    animationId = requestAnimationFrame(render);

    return () => {
      if (animationId) {
        cancelAnimationFrame(animationId);
      }
    };
  }, [bridge, wasmLoaded, isInitialized, behaviorSystem]);

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
              onContextMenu={handleContextMenu}
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

          {/* Marquee selection overlay */}
          {behaviorSystem.state.isMarqueeing &&
            behaviorSystem.state.marqueeRect && (
              <div
                className="absolute border-2 border-primary bg-primary/10 pointer-events-none"
                style={{
                  left: behaviorSystem.state.marqueeRect.x,
                  top: behaviorSystem.state.marqueeRect.y,
                  width: behaviorSystem.state.marqueeRect.width,
                  height: behaviorSystem.state.marqueeRect.height,
                }}
              />
            )}

          {/* Context Menu */}
          <ContextMenu />
        </div>
      )}
    </CanvasDroppable>
  );
});
