/**
 * Canvas Component - WebGPU Wrapper with Interaction Handling
 */

import { useRef, useEffect, useState, useCallback } from "react";
import { useCanvasStore } from "../store/useCanvasStore";
import { useUIStore } from "../store/useUIStore";
import { cn } from "../utils/cn";

interface CanvasProps {
  className?: string;
  onPointerDown?: (position: { x: number; y: number }, buttons: number) => void;
  onPointerMove?: (position: { x: number; y: number }, buttons: number) => void;
  onPointerUp?: (position: { x: number; y: number }, buttons: number) => void;
  onWheel?: (position: { x: number; y: number }, delta: number) => void;
}

export default function Canvas({
  className,
  onPointerDown,
  onPointerMove,
  onPointerUp,
  onWheel,
}: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isInitialized, setIsInitialized] = useState(false);

  const { camera, showGrid, zoomIn, pan } = useCanvasStore();
  const { activeTool } = useUIStore();

  // Resize observer
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

  // Initialize
  useEffect(() => {
    setIsInitialized(true);
  }, []);

  // Render loop
  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const width = canvas.width / dpr;
    const height = canvas.height / dpr;

    let animationId: number;

    const render = () => {
      ctx.clearRect(0, 0, width, height);
      ctx.fillStyle = "#101d22";
      ctx.fillRect(0, 0, width, height);

      // Draw grid
      if (showGrid) {
        ctx.strokeStyle = "#1a2c32";
        ctx.lineWidth = 0.5;
        const gridSize = 20 * camera.zoom;
        const offsetX = (camera.x * camera.zoom) % gridSize;
        const offsetY = (camera.y * camera.zoom) % gridSize;

        for (let x = offsetX; x < width; x += gridSize) {
          ctx.beginPath();
          ctx.moveTo(x, 0);
          ctx.lineTo(x, height);
          ctx.stroke();
        }
        for (let y = offsetY; y < height; y += gridSize) {
          ctx.beginPath();
          ctx.moveTo(0, y);
          ctx.lineTo(width, y);
          ctx.stroke();
        }
      }

      // Draw sample entities
      ctx.fillStyle = "#1a2c32";
      ctx.strokeStyle = "#13b6ec";
      ctx.lineWidth = 2;

      const sampleEntities = [
        { x: 100, y: 100, w: 120, h: 80 },
        { x: 250, y: 150, w: 100, h: 100 },
        { x: 400, y: 120, w: 150, h: 90 },
      ];

      sampleEntities.forEach((entity) => {
        ctx.fillRect(entity.x, entity.y, entity.w, entity.h);
        ctx.strokeRect(entity.x, entity.y, entity.w, entity.h);
        ctx.fillStyle = "#cbd5e1";
        ctx.font = "12px system-ui";
        ctx.fillText("Entity", entity.x + 8, entity.y + 20);
        ctx.fillStyle = "#1a2c32";
      });

      animationId = requestAnimationFrame(render);
    };

    render();

    return () => {
      if (animationId) cancelAnimationFrame(animationId);
    };
  }, [camera, showGrid]);

  // Get canvas position from event
  const getCanvasPosition = (event: React.PointerEvent | React.WheelEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;

    return {
      x: (event.clientX - rect.left) * dpr,
      y: (event.clientY - rect.top) * dpr,
    };
  };

  const handlePointerDown = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      const position = getCanvasPosition(event);
      onPointerDown?.(position, event.buttons);
    },
    [onPointerDown],
  );

  const handlePointerMove = useCallback(
    (event: React.PointerEvent) => {
      const position = getCanvasPosition(event);
      onPointerMove?.(position, event.buttons);
    },
    [onPointerMove],
  );

  const handlePointerUp = useCallback(
    (event: React.PointerEvent) => {
      const position = getCanvasPosition(event);
      onPointerUp?.(position, event.buttons);
    },
    [onPointerUp],
  );

  const handleWheel = useCallback(
    (event: React.WheelEvent) => {
      event.preventDefault();
      const position = getCanvasPosition(event);

      if (event.ctrlKey || event.metaKey) {
        const factor = event.deltaY > 0 ? 0.9 : 1.1;
        zoomIn(factor);
      } else {
        pan(event.deltaX, event.deltaY);
      }

      onWheel?.(position, Math.abs(event.deltaY));
    },
    [zoomIn, pan, onWheel],
  );

  const getCursor = () => {
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
  };

  return (
    <div ref={containerRef} className={cn("w-full h-full relative", className)}>
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
    </div>
  );
}
