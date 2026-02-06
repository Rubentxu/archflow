/**
 * TransformGizmo - Professional 3D-style Transform Controls
 *
 * A professional transform gizmo inspired by Blender/Unity with:
 * - Axis-constrained movement arrows (X: red, Y: green, XY: blue)
 * - Scale handles on corners
 * - Rotation ring with angle display
 * - Screen-space sizing (consistent visual size regardless of zoom)
 *
 * Architecture Reference: docs/epics/EPIC-WHITEBOARD_INTERACTIONS.md - US-035 to US-038
 */

import { useRef, useState, useCallback, memo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { cn } from "../utils/cn";
import { ArrowUp, Move, RotateCw } from "lucide-react";

/**
 * Gizmo mode
 */
export type GizmoMode = "move" | "scale" | "rotate" | "all";

/**
 * Gizmo axis constraints
 */
export type GizmoAxis = "x" | "y" | "xy" | null;

/**
 * TransformGizmo props
 */
interface TransformGizmoProps {
  /** Entity position in world coordinates */
  position: { x: number; y: number };
  /** Entity size (width, height) */
  size: { w: number; h: number };
  /** Entity rotation in degrees */
  rotation: number;
  /** Current gizmo mode */
  mode: GizmoMode;
  /** Gizmo visibility */
  visible?: boolean;
  /** Gizmo enabled state */
  enabled?: boolean;
  /** Screen position (for rendering) */
  screenPosition: { x: number; y: number };
  /** Screen scale (zoom level) */
  screenScale: number;
  /** Callback when drag starts */
  onDragStart?: (axis: GizmoAxis, mode: GizmoMode) => void;
  /** Callback during drag */
  onDrag?: (delta: { x: number; y: number }, axis: GizmoAxis) => void;
  /** Callback when drag ends */
  onDragEnd?: () => void;
  /** Callback when rotation starts */
  onRotateStart?: () => void;
  /** Callback during rotation */
  onRotate?: (angle: number) => void;
  /** Callback when rotation ends */
  onRotateEnd?: () => void;
  /** Callback when scale starts */
  onScaleStart?: () => void;
  /** Callback during scale */
  onScale?: (factor: { x: number; y: number }) => void;
  /** Callback when scale ends */
  onScaleEnd?: () => void;
  /** Additional CSS class */
  className?: string;
}

/**
 * Gizmo arrow configuration
 */
interface GizmoArrow {
  axis: GizmoAxis;
  color: string;
  rotation: number;
  length: number;
}

/**
 * Gizmo handle configuration
 */
interface GizmoHandle {
  type: "scale" | "rotate" | "move";
  position: { x: number; y: number };
  size: number;
  color?: string;
  cursor?: string;
}

/**
 * TransformGizmo Component
 */
export const TransformGizmo = memo(function TransformGizmo({
  // position is available but screenPosition is used for screen-space calculations
  position: _position,
  size,
  rotation,
  mode,
  visible = true,
  enabled = true,
  screenPosition,
  screenScale,
  onDragStart,
  onDrag,
  onDragEnd,
  onRotateStart,
  onRotate,
  onRotateEnd,
  onScaleStart,
  onScale,
  onScaleEnd,
  className,
}: TransformGizmoProps) {
  const [hoveredAxis, setHoveredAxis] = useState<GizmoAxis>(null);
  const [dragState, setDragState] = useState<{
    active: boolean;
    axis: GizmoAxis;
    startPos: { x: number; y: number };
  } | null>(null);
  const [rotateAngle, setRotateAngle] = useState(0);
  const gizmoRef = useRef<HTMLDivElement>(null);

  // Constants for gizmo visual
  const ARROW_LENGTH = 60; // Screen pixels at scale 1.0
  const ARROW_THICKNESS = 8;
  const HANDLE_SIZE = 12;
  const ROTATE_RADIUS = 80;
  const ROTATE_THICKNESS = 4;

  // Calculate visual dimensions based on screen scale
  const visualScale = screenScale;
  const scaledArrowLength = ARROW_LENGTH * visualScale;
  const scaledHandleSize = HANDLE_SIZE * visualScale;
  const scaledRotateRadius = ROTATE_RADIUS * visualScale;
  const scaledRotateThickness = ROTATE_THICKNESS * visualScale;

  // Calculate handle positions (relative to center)
  const centerX = screenPosition.x + (size.w * visualScale) / 2;
  const centerY = screenPosition.y + (size.h * visualScale) / 2;

  // Get bounding box in screen coordinates
  const bbox = {
    left: screenPosition.x,
    right: screenPosition.x + size.w * visualScale,
    top: screenPosition.y,
    bottom: screenPosition.y + size.h * visualScale,
    centerX,
    centerY,
  };

  // Calculate move arrows
  const moveArrows: GizmoArrow[] = [];
  if (mode === "move" || mode === "all") {
    moveArrows.push(
      { axis: "x", color: "#ef4444", rotation: 0, length: scaledArrowLength }, // Red - X axis
      { axis: "y", color: "#22c55e", rotation: -90, length: scaledArrowLength }, // Green - Y axis
    );
    if (mode === "all") {
      moveArrows.push({
        axis: "xy",
        color: "#3b82f6",
        rotation: -45,
        length: scaledArrowLength * 0.7,
      }); // Blue - XY plane
    }
  }

  // Calculate scale handles (corners)
  const scaleHandles: GizmoHandle[] = [];
  if (mode === "scale" || mode === "all") {
    const corners = [
      {
        x: bbox.left - scaledHandleSize / 2,
        y: bbox.top - scaledHandleSize / 2,
      }, // NW
      {
        x: bbox.right - scaledHandleSize / 2,
        y: bbox.top - scaledHandleSize / 2,
      }, // NE
      {
        x: bbox.left - scaledHandleSize / 2,
        y: bbox.bottom - scaledHandleSize / 2,
      }, // SW
      {
        x: bbox.right - scaledHandleSize / 2,
        y: bbox.bottom - scaledHandleSize / 2,
      }, // SE
    ];

    corners.forEach((pos) => {
      scaleHandles.push({
        type: "scale",
        position: pos,
        size: scaledHandleSize,
        color: "#3b82f6",
        cursor: "nwse-resize",
      });
    });
  }

  // Calculate rotation ring
  const rotateHandle: GizmoHandle = {
    type: "rotate",
    position: {
      x: centerX - scaledRotateRadius,
      y: centerY - scaledRotateRadius,
    },
    size: scaledRotateRadius * 2,
    color: "#f59e0b",
    cursor: "grab",
  };

  // Handle mouse events for dragging
  const handleMouseDown = useCallback(
    (axis: GizmoAxis) => {
      if (!enabled || !onDragStart) return;

      setDragState({
        active: true,
        axis,
        startPos: { x: 0, y: 0 },
      });
      onDragStart(axis, mode);

      const handleMouseMove = (e: MouseEvent) => {
        if (!dragState?.active) return;

        // Calculate delta from last position
        const deltaX = e.movementX;
        const deltaY = e.movementY;

        // Apply axis constraints
        let constrainedDelta = { x: deltaX, y: deltaY };
        if (axis === "x") {
          constrainedDelta = { x: deltaX, y: 0 };
        } else if (axis === "y") {
          constrainedDelta = { x: 0, y: deltaY };
        }

        onDrag?.(constrainedDelta, axis);
      };

      const handleMouseUp = () => {
        setDragState(null);
        onDragEnd?.();
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
      };

      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    },
    [enabled, mode, onDragStart, onDrag, onDragEnd, dragState],
  );

  // Handle rotation
  const handleRotateDown = useCallback(() => {
    if (!enabled || !onRotateStart) return;

    const startAngle = rotation;

    const handleRotateMove = (e: MouseEvent) => {
      const dx = e.clientX - centerX;
      const dy = e.clientY - centerY;
      const angle = Math.atan2(dy, dx) * (180 / Math.PI);
      const newAngle = angle + 90; // Adjust for handle position
      const deltaAngle = newAngle - startAngle;

      setRotateAngle(deltaAngle);
      onRotate?.(deltaAngle);
    };

    const handleRotateUp = () => {
      setRotateAngle(0);
      onRotateEnd?.();
      document.removeEventListener("mousemove", handleRotateMove);
      document.removeEventListener("mouseup", handleRotateUp);
    };

    onRotateStart();
    document.addEventListener("mousemove", handleRotateMove);
    document.addEventListener("mouseup", handleRotateUp);
  }, [
    enabled,
    centerX,
    centerY,
    rotation,
    onRotateStart,
    onRotate,
    onRotateEnd,
  ]);

  // Handle scale
  const handleScaleDown = useCallback(() => {
    if (!enabled || !onScaleStart) return;

    const handleScaleMove = (e: MouseEvent) => {
      const scaleFactor = 1 + e.movementX * 0.01;
      onScale?.({ x: scaleFactor, y: scaleFactor });
    };

    const handleScaleUp = () => {
      onScaleEnd?.();
      document.removeEventListener("mousemove", handleScaleMove);
      document.removeEventListener("mouseup", handleScaleUp);
    };

    onScaleStart();
    document.addEventListener("mousemove", handleScaleMove);
    document.addEventListener("mouseup", handleScaleUp);
  }, [enabled, onScaleStart, onScale, onScaleEnd]);

  if (!visible || !enabled) return null;

  return (
    <div
      ref={gizmoRef}
      className={cn("pointer-events-none absolute overflow-visible", className)}
      style={{
        left: 0,
        top: 0,
        width: 0,
        height: 0,
      }}
    >
      {/* Move arrows */}
      <AnimatePresence>
        {(mode === "move" || mode === "all") && (
          <>
            {/* X Axis Arrow */}
            <motion.div
              initial={{ opacity: 0, scale: 0 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0 }}
              className="pointer-events-auto absolute cursor-move"
              style={{
                left: centerX,
                top: centerY,
                width: scaledArrowLength,
                height: ARROW_THICKNESS,
                transformOrigin: "left center",
                transform: "translateY(-50%)",
              }}
              onMouseDown={() => handleMouseDown("x")}
              onMouseEnter={() => setHoveredAxis("x")}
              onMouseLeave={() => setHoveredAxis(null)}
            >
              {/* Arrow shaft */}
              <div
                className={cn(
                  "h-full w-full rounded-l",
                  hoveredAxis === "x"
                    ? "bg-red-500 shadow-lg shadow-red-500/50"
                    : "bg-red-500/80",
                )}
              />
              {/* Arrow head */}
              <div
                className="absolute right-0 top-1/2 -translate-y-1/2 translate-x-1"
                style={{
                  width: 0,
                  height: 0,
                  borderTop: `${ARROW_THICKNESS / 2}px solid transparent`,
                  borderBottom: `${ARROW_THICKNESS / 2}px solid transparent`,
                  borderLeft: `${ARROW_THICKNESS * 1.5}px solid ${
                    hoveredAxis === "x" ? "#ef4444" : "#ef4444"
                  }`,
                }}
              />
            </motion.div>

            {/* Y Axis Arrow */}
            <motion.div
              initial={{ opacity: 0, scale: 0 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0 }}
              className="pointer-events-auto absolute cursor-move"
              style={{
                left: centerX,
                top: centerY,
                width: ARROW_THICKNESS,
                height: scaledArrowLength,
                transformOrigin: "top center",
                transform: "translateX(-50%)",
              }}
              onMouseDown={() => handleMouseDown("y")}
              onMouseEnter={() => setHoveredAxis("y")}
              onMouseLeave={() => setHoveredAxis(null)}
            >
              {/* Arrow shaft */}
              <div
                className={cn(
                  "h-full w-full rounded-t",
                  hoveredAxis === "y"
                    ? "bg-green-500 shadow-lg shadow-green-500/50"
                    : "bg-green-500/80",
                )}
              />
              {/* Arrow head */}
              <div
                className="absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-1"
                style={{
                  width: 0,
                  height: 0,
                  borderLeft: `${ARROW_THICKNESS / 2}px solid transparent`,
                  borderRight: `${ARROW_THICKNESS / 2}px solid transparent`,
                  borderBottom: `${ARROW_THICKNESS * 1.5}px solid ${
                    hoveredAxis === "y" ? "#22c55e" : "#22c55e"
                  }`,
                }}
              />
            </motion.div>

            {/* XY Axis Arrow (center) */}
            {mode === "all" && (
              <motion.div
                initial={{ opacity: 0, scale: 0 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0 }}
                className="pointer-events-auto absolute cursor-move"
                style={{
                  left: centerX,
                  top: centerY,
                  width: scaledArrowLength * 0.7,
                  height: scaledArrowLength * 0.7,
                  transformOrigin: "left top",
                  transform: "translate(-50%, -50%) rotate(-45deg)",
                }}
                onMouseDown={() => handleMouseDown("xy")}
                onMouseEnter={() => setHoveredAxis("xy")}
                onMouseLeave={() => setHoveredAxis(null)}
              >
                <div
                  className={cn(
                    "h-3 w-3 rounded-full",
                    hoveredAxis === "xy"
                      ? "bg-blue-500 shadow-lg shadow-blue-500/50"
                      : "bg-blue-500/80",
                  )}
                />
              </motion.div>
            )}
          </>
        )}
      </AnimatePresence>

      {/* Scale handles */}
      <AnimatePresence>
        {(mode === "scale" || mode === "all") && (
          <>
            {scaleHandles.map((handle, index) => (
              <motion.div
                key={`scale-${index}`}
                initial={{ opacity: 0, scale: 0 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0 }}
                className="pointer-events-auto absolute cursor-nwse-resize rounded border border-white/50 bg-blue-500/80 shadow-lg"
                style={{
                  left: handle.position.x,
                  top: handle.position.y,
                  width: handle.size,
                  height: handle.size,
                }}
                onMouseDown={handleScaleDown}
              />
            ))}
          </>
        )}
      </AnimatePresence>

      {/* Rotation ring */}
      <AnimatePresence>
        {(mode === "rotate" || mode === "all") && (
          <motion.div
            initial={{ opacity: 0, scale: 0 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0 }}
            className="pointer-events-auto absolute rounded-full border-2"
            style={{
              left: rotateHandle.position.x,
              top: rotateHandle.position.y,
              width: rotateHandle.size,
              height: rotateHandle.size,
              borderColor: rotateHandle.color,
              cursor: rotateHandle.cursor,
            }}
            onMouseDown={handleRotateDown}
          >
            {/* Rotation handle indicator */}
            <div
              className="absolute left-1/2 top-0 -translate-x-1/2 -translate-y-1/2"
              style={{
                width: scaledRotateThickness + 4,
                height: scaledRotateThickness + 4,
              }}
            >
              <div
                className="h-full w-full rounded-full bg-amber-500 shadow-lg shadow-amber-500/50"
                style={{
                  width: scaledRotateThickness + 4,
                  height: scaledRotateThickness + 4,
                }}
              />
            </div>

            {/* Angle display during rotation */}
            <AnimatePresence>
              {rotateAngle !== 0 && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                  className="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded bg-gray-900/90 px-2 py-1 text-xs text-white"
                >
                  {rotateAngle.toFixed(1)}°
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Center pivot point */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="pointer-events-none absolute rounded-full border border-white/50 bg-gray-700"
        style={{
          left: centerX - 4,
          top: centerY - 4,
          width: 8,
          height: 8,
        }}
      />
    </div>
  );
});
