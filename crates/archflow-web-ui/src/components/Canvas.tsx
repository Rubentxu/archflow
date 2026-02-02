import { useRef, useEffect } from "react";
import { useCanvasStore } from "../store/useCanvasStore";

export default function Canvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { camera, showGrid } = useCanvasStore();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Set canvas size
    canvas.width = canvas.clientWidth * window.devicePixelRatio;
    canvas.height = canvas.clientHeight * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);

    // Clear
    ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight);

    // Draw grid
    if (showGrid) {
      ctx.strokeStyle = "#2a3e45";
      ctx.lineWidth = 0.5;
      const gridSize = 20 * camera.zoom;
      const offsetX = (camera.x * camera.zoom) % gridSize;
      const offsetY = (camera.y * camera.zoom) % gridSize;

      for (let x = offsetX; x < canvas.clientWidth; x += gridSize) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, canvas.clientHeight);
        ctx.stroke();
      }
      for (let y = offsetY; y < canvas.clientHeight; y += gridSize) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(canvas.clientWidth, y);
        ctx.stroke();
      }
    }
  }, [camera, showGrid]);

  return (
    <canvas
      ref={canvasRef}
      className="w-full h-full touch-none"
      style={{ cursor: camera.zoom === 1 ? "crosshair" : "grab" }}
    />
  );
}
