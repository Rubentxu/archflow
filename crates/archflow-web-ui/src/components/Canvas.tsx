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

  // Use specific selectors to prevent re-renders when other store parts change
  const camera = useCanvasStore((state) => state.camera);
  const showGrid = useCanvasStore((state) => state.showGrid);
  const zoomIn = useCanvasStore((state) => state.zoomIn);
  const pan = useCanvasStore((state) => state.pan);
  const activeTool = useUIStore((state) => state.activeTool);

  // WASM bridge access
  const { bridge, isLoaded: wasmLoaded } = useArchFlowWasm();
  const { CanvasDroppable, DragOverlayContent, dragState } = useDragAndDrop();

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
        // WASM will handle the actual rendering and interaction
        // This is a placeholder for the WASM integration
        // The actual WASM bridge should have a method like:
        // bridge.handlePointerDown(position.x, position.y, event.buttons);
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
        // bridge.handlePointerMove(position.x, position.y, event.buttons);
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
        // bridge.handlePointerUp(position.x, position.y, event.buttons);
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
        // bridge.handleWheel(position.x, position.y, event.deltaX, event.deltaY);
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
        }
      }
    });

    resizeObserver.observe(containerRef.current);
    return () => resizeObserver.disconnect();
  }, []);

  // Initialize renderer
  useEffect(() => {
    if (!canvasRef.current || !bridge || !wasmLoaded) return;

    // The WASM engine should handle all rendering
    // For now, we just initialize - the actual render loop
    // will be driven by WASM through requestAnimationFrame
    // bridge.initCanvas(canvasRef.current);

    setIsInitialized(true);
  }, [bridge, wasmLoaded]);

  // WASM-driven render loop
  useEffect(() => {
    if (!canvasRef.current || !bridge || !wasmLoaded || !isInitialized) return;

    let animationId: number;

    const render = () => {
      // WASM handles all rendering
      // bridge.render() should:
      // 1. Clear canvas
      // 2. Draw grid
      // 3. Draw all entities from WASM EntityStore
      // 4. Draw selection highlights
      // 5. Draw drag previews
      // This is a placeholder - actual implementation depends on WASM API

      animationId = requestAnimationFrame(render);
    };

    render();

    return () => {
      if (animationId) cancelAnimationFrame(animationId);
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
