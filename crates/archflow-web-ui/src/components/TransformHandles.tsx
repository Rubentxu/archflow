/**
 * Transform Handles - Resize and transform controls for entities
 *
 * Provides corner and edge handles for resizing entities with
 * visual feedback and keyboard modifier support.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { memo, useState, useCallback } from "react";
import { motion } from "framer-motion";
import { cn } from "../utils/cn";
import { handleVariants } from "../utils/animations";

/**
 * Handle positions
 */
export type HandlePosition = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";

/**
 * Handle configuration with cursor and position
 */
interface HandleConfig {
  position: HandlePosition;
  cursor: "nwse-resize" | "ns-resize" | "nesw-resize" | "ew-resize";
  x: number;
  y: number;
}

/**
 * Props for TransformHandles component
 */
interface TransformHandlesProps {
  /** Entity position and size */
  position: { x: number; y: number };
  size: { w: number; h: number };
  /** Whether handles should be visible */
  visible?: boolean;
  /** Whether the entity is locked (no transform) */
  locked?: boolean;
  /** Current selected handles (for multi-handle operations) */
  selectedHandles?: HandlePosition[];
  /** Snap state from parent */
  snapState?: {
    isSnapping: boolean;
    snapX?: number;
    snapY?: number;
  };
  /** Callback when resize starts */
  onResizeStart?: (
    handle: HandlePosition,
    startPos: { x: number; y: number },
  ) => void;
  /** Additional CSS class */
  className?: string;
}

/**
 * Get handle configuration for entity bounds
 */
function getHandleConfigs(
  position: { x: number; y: number },
  size: { w: number; h: number },
  handleSize: number = 10,
): HandleConfig[] {
  const halfSize = handleSize / 2;

  return [
    // Corner handles
    {
      position: "nw",
      cursor: "nwse-resize",
      x: position.x - halfSize,
      y: position.y - halfSize,
    },
    {
      position: "ne",
      cursor: "nesw-resize",
      x: position.x + size.w - halfSize,
      y: position.y - halfSize,
    },
    {
      position: "sw",
      cursor: "nesw-resize",
      x: position.x - halfSize,
      y: position.y + size.h - halfSize,
    },
    {
      position: "se",
      cursor: "nwse-resize",
      x: position.x + size.w - halfSize,
      y: position.y + size.h - halfSize,
    },
    // Edge handles
    {
      position: "n",
      cursor: "ns-resize",
      x: position.x + size.w / 2 - halfSize,
      y: position.y - halfSize,
    },
    {
      position: "s",
      cursor: "ns-resize",
      x: position.x + size.w / 2 - halfSize,
      y: position.y + size.h - halfSize,
    },
    {
      position: "w",
      cursor: "ew-resize",
      x: position.x - halfSize,
      y: position.y + size.h / 2 - halfSize,
    },
    {
      position: "e",
      cursor: "ew-resize",
      x: position.x + size.w - halfSize,
      y: position.y + size.h / 2 - halfSize,
    },
  ];
}

/**
 * Single transform handle component
 */
const TransformHandle = memo(function TransformHandle({
  config,
  isSelected,
  isHovered,
  onMouseDown,
}: {
  config: HandleConfig;
  isSelected: boolean;
  isHovered: boolean;
  onMouseDown: (e: React.MouseEvent) => void;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <motion.div
      className={cn(
        "absolute w-3 h-3 bg-primary rounded-sm",
        "border-2 border-white shadow-lg",
        "cursor-pointer select-none",
        isSelected && "bg-primary-light",
      )}
      style={{
        left: config.x,
        top: config.y,
        cursor: config.cursor,
        transform: "translate(-50%, -50%)",
      }}
      variants={handleVariants}
      initial="idle"
      animate={hovered || isHovered ? "hover" : "idle"}
      onMouseDown={onMouseDown}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    />
  );
});

/**
 * TransformHandles component
 *
 * Renders resize handles around an entity's bounds with
 * visual feedback.
 */
export const TransformHandles = memo(function TransformHandles({
  position,
  size,
  visible = true,
  locked = false,
  selectedHandles = [],
  snapState,
  onResizeStart,
  className,
}: TransformHandlesProps) {
  const handleConfigs = getHandleConfigs(position, size);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent, handle: HandlePosition) => {
      e.preventDefault();
      e.stopPropagation();
      onResizeStart?.(handle, { x: e.clientX, y: e.clientY });
    },
    [onResizeStart],
  );

  if (!visible || locked) return null;

  return (
    <g className={cn("transform-handles", className)}>
      {/* Selection border */}
      <motion.rect
        x={position.x - 1}
        y={position.y - 1}
        width={size.w + 2}
        height={size.h + 2}
        fill="none"
        stroke="#13b6ec"
        strokeWidth={2}
        strokeDasharray="4,4"
        rx={4}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
      />

      {/* Snap indicator overlay */}
      {snapState?.isSnapping && (
        <motion.g
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.8 }}
        >
          {snapState.snapX !== undefined && (
            <line
              x1={snapState.snapX}
              y1={position.y - 10}
              x2={snapState.snapX}
              y2={position.y + size.h + 10}
              stroke="#13b6ec"
              strokeWidth={2}
              strokeDasharray="4,4"
            />
          )}
          {snapState.snapY !== undefined && (
            <line
              x1={position.x - 10}
              y1={snapState.snapY}
              x2={position.x + size.w + 10}
              y2={snapState.snapY}
              stroke="#13b6ec"
              strokeWidth={2}
              strokeDasharray="4,4"
            />
          )}
        </motion.g>
      )}

      {/* Render all handles */}
      {handleConfigs.map((config) => (
        <foreignObject
          key={config.position}
          x={config.x - 6}
          y={config.y - 6}
          width={12}
          height={12}
          style={{ overflow: "visible" }}
        >
          <TransformHandle
            config={config}
            isSelected={selectedHandles.includes(config.position)}
            isHovered={false}
            onMouseDown={(e) => handleMouseDown(e, config.position)}
          />
        </foreignObject>
      ))}
    </g>
  );
});

/**
 * Multi-select handles for marquee selection
 */
export const MultiSelectHandles = memo(function MultiSelectHandles({
  bounds,
  visible = true,
  className,
}: {
  bounds: { x: number; y: number; w: number; h: number };
  visible?: boolean;
  className?: string;
}) {
  if (!visible) return null;

  return (
    <g className={cn("multi-select-handles", className)}>
      {/* Dashed selection box */}
      <rect
        x={bounds.x}
        y={bounds.y}
        width={bounds.w}
        height={bounds.h}
        fill="rgba(19, 182, 236, 0.1)"
        stroke="#13b6ec"
        strokeWidth={1}
        strokeDasharray="4,4"
        rx={4}
      />

      {/* Corner handles only for multi-select */}
      {[
        { x: bounds.x, y: bounds.y },
        { x: bounds.x + bounds.w, y: bounds.y },
        { x: bounds.x, y: bounds.y + bounds.h },
        { x: bounds.x + bounds.w, y: bounds.y + bounds.h },
      ].map((pos, i) => (
        <circle
          key={i}
          cx={pos.x}
          cy={pos.y}
          r={6}
          className="cursor-nwse-resize"
          fill="white"
          stroke="#13b6ec"
          strokeWidth={2}
        />
      ))}
    </g>
  );
});

/**
 * Resize direction helpers
 */
export const resizeDirections = {
  nw: { x: -1, y: -1 },
  n: { x: 0, y: -1 },
  ne: { x: 1, y: -1 },
  e: { x: 1, y: 0 },
  se: { x: 1, y: 1 },
  s: { x: 0, y: 1 },
  sw: { x: -1, y: 1 },
  w: { x: -1, y: 0 },
} satisfies Record<HandlePosition, { x: -1 | 0 | 1; y: -1 | 0 | 1 }>;
