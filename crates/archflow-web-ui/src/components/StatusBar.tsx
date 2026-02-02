/**
 * StatusBar Component - Application Footer
 */

import { useState, useEffect } from "react";
import { Layers, CircleDot, Zap } from "lucide-react";
import { cn } from "../utils/cn";

interface StatusBarProps {
  className?: string;
  fps?: number;
  entityCount?: number;
  selectedCount?: number;
  zoom?: number;
  showGrid?: boolean;
  gridSize?: number;
}

export default function StatusBar({
  className,
  fps: initialFps = 60,
  entityCount: initialEntityCount = 0,
  selectedCount = 0,
  zoom = 1,
  showGrid = true,
  gridSize = 20,
}: StatusBarProps) {
  const [fps, setFps] = useState(initialFps);
  const [lastFrameTime, setLastFrameTime] = useState(performance.now());
  const [frameCount, setFrameCount] = useState(0);

  useEffect(() => {
    let animationId: number;
    const updateFps = () => {
      const now = performance.now();
      const delta = now - lastFrameTime;
      if (delta >= 1000) {
        setFps(Math.round((frameCount * 1000) / delta));
        setFrameCount(0);
        setLastFrameTime(now);
      } else {
        setFrameCount((prev) => prev + 1);
      }
      animationId = requestAnimationFrame(updateFps);
    };
    animationId = requestAnimationFrame(updateFps);
    return () => {
      if (animationId) cancelAnimationFrame(animationId);
    };
  }, [lastFrameTime, frameCount]);

  return (
    <footer
      className={cn(
        "h-8 flex items-center justify-between px-4 border-t border-white/5 bg-surface-dark/95 text-xs text-gray-400",
        className,
      )}
    >
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1.5">
          <Layers className="w-3.5 h-3.5" />
          <span>
            <strong className="text-gray-300">{initialEntityCount}</strong>{" "}
            entities
          </span>
        </div>
        {selectedCount > 0 && (
          <span className="text-primary">
            <strong>{selectedCount}</strong> selected
          </span>
        )}
      </div>
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-1.5">
          <CircleDot
            className={cn(
              "w-3.5 h-3.5",
              fps >= 55
                ? "text-green-400"
                : fps >= 30
                  ? "text-yellow-400"
                  : "text-red-400",
            )}
          />
          <span>
            <strong
              className={
                fps >= 55
                  ? "text-green-400"
                  : fps >= 30
                    ? "text-yellow-400"
                    : "text-red-400"
              }
            >
              {fps}
            </strong>{" "}
            FPS
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <Zap className="w-3.5 h-3.5" />
          <span>WebGPU</span>
        </div>
      </div>
      <div className="flex items-center gap-3">
        <span>{Math.round(zoom * 100)}%</span>
        <span className={cn(showGrid ? "text-primary" : "text-gray-500")}>
          {gridSize}px
        </span>
      </div>
    </footer>
  );
}
