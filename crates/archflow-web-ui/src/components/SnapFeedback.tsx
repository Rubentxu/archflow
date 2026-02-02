/**
 * Snap Feedback - Visual feedback for snapping during drag operations
 *
 * Provides visual indicators when entities snap to grid,
 * guides, or other entities during transform operations.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { memo } from "react";
import { motion } from "framer-motion";
import { cn } from "../utils/cn";

/**
 * Snap target information
 */
export interface SnapTarget {
  id: string | number;
  type: "grid" | "entity" | "guide" | "center";
  x: number;
  y: number;
  direction: "horizontal" | "vertical" | "both";
}

/**
 * Current snap state
 */
export interface SnapState {
  isActive: boolean;
  targets: SnapTarget[];
  snappedTo?: SnapTarget;
}

/**
 * Props for SnapFeedback component
 */
interface SnapFeedbackProps {
  /** Current snap state */
  snapState: SnapState;
  /** Entity bounds being snapped */
  entityBounds?: { x: number; y: number; w: number; h: number };
  /** Canvas offset for coordinate translation */
  canvasOffset?: { x: number; y: number };
  /** Whether to show guides */
  showGuides?: boolean;
  /** Additional CSS class */
  className?: string;
}

/**
 * Snap line component
 */
const SnapLine = memo(function SnapLine({
  start,
  end,
  isActive,
  color = "#13b6ec",
}: {
  start: { x: number; y: number };
  end: { x: number; y: number };
  isActive: boolean;
  color?: string;
}) {
  return (
    <motion.line
      x1={start.x}
      y1={start.y}
      x2={end.x}
      y2={end.y}
      stroke={color}
      strokeWidth={isActive ? 2 : 1}
      strokeDasharray={isActive ? undefined : "4,4"}
      opacity={isActive ? 1 : 0.5}
      initial={{ opacity: 0 }}
      animate={{ opacity: isActive ? 1 : 0.5 }}
      exit={{ opacity: 0 }}
    />
  );
});

/**
 * Snap point indicator
 */
const SnapIndicator = memo(function SnapIndicator({
  x,
  y,
  isSnapping,
}: {
  x: number;
  y: number;
  isSnapping: boolean;
}) {
  return (
    <motion.g
      initial={{ opacity: 0, scale: 0 }}
      animate={{ opacity: 1, scale: isSnapping ? 1.2 : 1 }}
      exit={{ opacity: 0, scale: 0 }}
    >
      {/* Center crosshair */}
      <circle
        cx={x}
        cy={y}
        r={8}
        fill="none"
        stroke="#13b6ec"
        strokeWidth={2}
      />
      <line
        x1={x - 12}
        y1={y}
        x2={x + 12}
        y2={y}
        stroke="#13b6ec"
        strokeWidth={1}
      />
      <line
        x1={x}
        y1={y - 12}
        x2={x}
        y2={y + 12}
        stroke="#13b6ec"
        strokeWidth={1}
      />
    </motion.g>
  );
});

/**
 * SnapFeedback component
 *
 * Renders visual feedback for snapping during drag operations.
 * Shows snap guides, indicators, and highlight effects.
 */
export const SnapFeedback = memo(function SnapFeedback({
  snapState,
  entityBounds,
  canvasOffset = { x: 0, y: 0 },
  showGuides = true,
  className,
}: SnapFeedbackProps) {
  const { isActive, targets, snappedTo } = snapState;

  if (!isActive) return null;

  // Calculate guide positions based on entity bounds
  const verticalGuides = targets.filter(
    (t) => t.direction === "vertical" || t.direction === "both",
  );
  const horizontalGuides = targets.filter(
    (t) => t.direction === "horizontal" || t.direction === "both",
  );

  // Canvas dimensions (large enough to cover typical viewports)
  const canvasSize = 10000;
  const center = canvasSize / 2;

  return (
    <svg
      className={cn(
        "absolute inset-0 pointer-events-none",
        "w-full h-full",
        className,
      )}
      style={{
        transform: `translate(${-canvasOffset.x}px, ${-canvasOffset.y}px)`,
        overflow: "visible",
      }}
      viewBox={`0 0 ${canvasSize} ${canvasSize}`}
    >
      {/* Render vertical guides */}
      {showGuides &&
        verticalGuides.map((target) => (
          <SnapLine
            key={`v-${target.id}`}
            start={{ x: target.x, y: center - 1000 }}
            end={{ x: target.x, y: center + 1000 }}
            isActive={snappedTo?.id === target.id}
          />
        ))}

      {/* Render horizontal guides */}
      {showGuides &&
        horizontalGuides.map((target) => (
          <SnapLine
            key={`h-${target.id}`}
            start={{ x: center - 1000, y: target.y }}
            end={{ x: center + 1000, y: target.y }}
            isActive={snappedTo?.id === target.id}
          />
        ))}

      {/* Render snap indicator at snapped position */}
      {snappedTo && (
        <SnapIndicator x={snappedTo.x} y={snappedTo.y} isSnapping={true} />
      )}

      {/* Highlight snapped entity if applicable */}
      {snappedTo?.type === "entity" && entityBounds && (
        <rect
          x={entityBounds.x - 2}
          y={entityBounds.y - 2}
          width={entityBounds.w + 4}
          height={entityBounds.h + 4}
          fill="none"
          stroke="#13b6ec"
          strokeWidth={3}
          strokeDasharray="8,4"
          rx={6}
        />
      )}

      {/* Distance labels for snapped alignment */}
      {snappedTo && entityBounds && (
        <g>
          {snappedTo.direction !== "vertical" && (
            <text
              x={(entityBounds.x + entityBounds.w / 2 + snappedTo.x) / 2}
              y={entityBounds.y - 8}
              fill="#13b6ec"
              fontSize={11}
              textAnchor="middle"
              className="pointer-events-none"
            >
              {Math.round(Math.abs(entityBounds.y - snappedTo.y))}px
            </text>
          )}
          {snappedTo.direction !== "horizontal" && (
            <text
              x={entityBounds.x + entityBounds.w + 8}
              y={(entityBounds.y + entityBounds.h / 2 + snappedTo.y) / 2}
              fill="#13b6ec"
              fontSize={11}
              textAnchor="start"
              dominantBaseline="middle"
              className="pointer-events-none"
            >
              {Math.round(Math.abs(entityBounds.x - snappedTo.x))}px
            </text>
          )}
        </g>
      )}
    </svg>
  );
});

/**
 * Snap preview - Shows where entity will snap before dropping
 */
export const SnapPreview = memo(function SnapPreview({
  position,
  size,
  snappedPosition,
  isSnapping,
}: {
  position: { x: number; y: number };
  size: { w: number; h: number };
  snappedPosition?: { x: number; y: number };
  isSnapping: boolean;
}) {
  if (!isSnapping || !snappedPosition) return null;

  const offset = {
    x: snappedPosition.x - position.x,
    y: snappedPosition.y - position.y,
  };

  return (
    <motion.g
      initial={{ opacity: 0 }}
      animate={{ opacity: 0.5 }}
      exit={{ opacity: 0 }}
    >
      {/* Ghost entity at snapped position */}
      <rect
        x={snappedPosition.x}
        y={snappedPosition.y}
        width={size.w}
        height={size.h}
        fill="rgba(19, 182, 236, 0.1)"
        stroke="#13b6ec"
        strokeWidth={2}
        strokeDasharray="4,4"
        rx={4}
      />

      {/* Offset indicator */}
      {offset.x !== 0 && (
        <line
          x1={position.x + size.w / 2}
          y1={position.y - 10}
          x2={snappedPosition.x + size.w / 2}
          y2={snappedPosition.y - 10}
          stroke="#13b6ec"
          strokeWidth={1}
          markerStart="url(#arrow-start)"
          markerEnd="url(#arrow-end)"
        />
      )}

      {/* Distance label */}
      <rect
        x={position.x + snappedPosition.x + size.w / 2 - 25}
        y={position.y - 28}
        width={50}
        height={16}
        fill="rgba(19, 182, 236, 0.9)"
        rx={4}
      />
      <text
        x={position.x + snappedPosition.x + size.w / 2}
        y={position.y - 16}
        fill="white"
        fontSize={10}
        textAnchor="middle"
        dominantBaseline="middle"
      >
        {Math.round(Math.abs(offset.x))}×{Math.round(Math.abs(offset.y))}
      </text>
    </motion.g>
  );
});

/**
 * Grid snap indicator
 */
export const GridSnapIndicator = memo(function GridSnapIndicator({
  position,
  gridSize,
  isSnapped,
}: {
  position: { x: number; y: number };
  gridSize: number;
  isSnapped: boolean;
}) {
  if (!isSnapped) return null;

  // Calculate nearest grid point
  const snapX = Math.round(position.x / gridSize) * gridSize;
  const snapY = Math.round(position.y / gridSize) * gridSize;

  return (
    <motion.g
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
    >
      {/* Grid point marker */}
      <circle
        cx={snapX}
        cy={snapY}
        r={6}
        fill="#13b6ec"
        fillOpacity={0.3}
        stroke="#13b6ec"
        strokeWidth={2}
      />

      {/* Crosshair lines */}
      <line
        x1={snapX - 10}
        y1={snapY}
        x2={snapX + 10}
        y2={snapY}
        stroke="#13b6ec"
        strokeWidth={1}
      />
      <line
        x1={snapX}
        y1={snapY - 10}
        x2={snapX}
        y2={snapY + 10}
        stroke="#13b6ec"
        strokeWidth={1}
      />
    </motion.g>
  );
});
