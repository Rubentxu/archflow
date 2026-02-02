/**
 * Connection Renderer - Renders SVG connections between entities
 *
 * Uses Bezier curves for smooth connection lines with arrow markers.
 * Optimized for performance with React.memo and selective updates.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { memo, useMemo } from "react";
import { motion } from "framer-motion";
import { cn } from "../../utils/cn";
import {
  useConnectionStore,
  type Connection,
  type ConnectionPoint,
} from "../../store/useConnectionStore";
import { connectionRouting } from "../../store/useConnectionStore";

/**
 * Props for individual connection
 */
interface ConnectionPathProps {
  connection: Connection;
  isSelected: boolean;
  isCreating: boolean;
  className?: string;
}

/**
 * Calculate SVG path for connection
 */
function calculatePath(
  source: ConnectionPoint,
  target: ConnectionPoint,
  routingPoints: { x: number; y: number }[] = [],
): string {
  const curve = connectionRouting.calculateCurvedPath(source, target);

  if (routingPoints.length > 0) {
    // Use custom routing points
    let path = `M ${source.x},${source.y}`;
    for (const point of routingPoints) {
      path += ` L ${point.x},${point.y}`;
    }
    path += ` L ${target.x},${target.y}`;
    return path;
  }

  // Use Bezier curve
  return `M ${curve.start.x},${curve.start.y} C ${curve.control1.x},${curve.control1.y} ${curve.control2.x},${curve.control2.y} ${curve.end.x},${curve.end.y}`;
}

/**
 * Single connection path component
 */
const ConnectionPath = memo(function ConnectionPath({
  connection,
  isSelected,
  isCreating,
  className,
}: ConnectionPathProps) {
  const {
    sourcePoint,
    targetPoint,
    style: connectionStyle,
    routingPoints,
  } = connection;

  const path = useMemo(
    () => calculatePath(sourcePoint, targetPoint, routingPoints),
    [sourcePoint, targetPoint, routingPoints],
  );

  const strokeColor = isSelected
    ? "#13b6ec"
    : connectionStyle?.strokeColor || "#64748b";

  const strokeWidth = isSelected
    ? (connectionStyle?.strokeWidth || 2) + 1
    : connectionStyle?.strokeWidth || 2;

  const strokeDasharray =
    connectionStyle?.lineType === "dashed"
      ? "8,4"
      : connectionStyle?.lineType === "dotted"
        ? "4,4"
        : undefined;

  return (
    <g className={className}>
      {/* Shadow/glow effect for selected connections */}
      {isSelected && (
        <path
          d={path}
          stroke={strokeColor}
          strokeWidth={strokeWidth + 4}
          fill="none"
          opacity={0.3}
          style={{ filter: "blur(4px)" }}
        />
      )}

      {/* Main connection line */}
      <motion.path
        d={path}
        stroke={strokeColor}
        strokeWidth={strokeWidth}
        fill="none"
        strokeDasharray={strokeDasharray}
        strokeLinecap="round"
        strokeLinejoin="round"
        initial={isCreating ? { pathLength: 0 } : false}
        animate={isCreating ? { pathLength: 1 } : { pathLength: 1 }}
        transition={
          isCreating ? { duration: 0.2, ease: "easeOut" } : { duration: 0 }
        }
        style={{ pointerEvents: "stroke" }}
        markerEnd={
          connectionStyle?.hasArrow ? `url(#arrow-${connection.id})` : undefined
        }
      />
    </g>
  );
});

/**
 * Props for connection preview during creation
 */
interface ConnectionPreviewProps {
  sourcePoint: ConnectionPoint | null;
  currentPoint: { x: number; y: number } | null;
  color?: string;
}

/**
 * Preview line during connection creation
 */
const ConnectionPreview = memo(function ConnectionPreview({
  sourcePoint,
  currentPoint,
  color = "#13b6ec",
}: ConnectionPreviewProps) {
  if (!sourcePoint || !currentPoint) return null;

  const path = `M ${sourcePoint.x},${sourcePoint.y} L ${currentPoint.x},${currentPoint.y}`;

  return (
    <motion.path
      d={path}
      stroke={color}
      strokeWidth={2}
      strokeDasharray="8,4"
      fill="none"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
    />
  );
});

/**
 * Props for ConnectionRenderer
 */
interface ConnectionRendererProps {
  /** Canvas offset for coordinate translation */
  canvasOffset?: { x: number; y: number };
  /** Current camera zoom level */
  zoom?: number;
  /** Additional CSS class */
  className?: string;
  /** Whether to show connection points */
  showConnectionPoints?: boolean;
}

/**
 * Main connection renderer component
 *
 * Renders all connections as SVG elements overlaid on the canvas.
 * Handles selection, creation preview, and animated transitions.
 */
export const ConnectionRenderer = memo(function ConnectionRenderer({
  canvasOffset = { x: 0, y: 0 },
  zoom = 1,
  className,
  showConnectionPoints = false,
}: ConnectionRendererProps) {
  const { connections, selectedConnectionIds, creation } = useConnectionStore();

  const {
    sourcePoint: creatingSourcePoint,
    tempEndPoint: creatingTempPoint,
    isCreating,
  } = creation;

  // Generate unique marker IDs for all connections
  const markerIds = useMemo(
    () =>
      connections.map((c) => ({
        id: c.id,
        markerId: `arrow-${c.id}`,
        color: c.style.strokeColor,
        size: c.style.arrowSize,
      })),
    [connections],
  );

  return (
    <svg
      className={cn(
        "absolute inset-0 pointer-events-none",
        "w-full h-full",
        className,
      )}
      style={{
        transform: `translate(${canvasOffset.x}px, ${canvasOffset.y}px) scale(${zoom})`,
        transformOrigin: "top left",
      }}
    >
      {/* Define arrow markers */}
      <defs>
        {markerIds.map(({ markerId, color, size }) => (
          <marker
            key={markerId}
            id={markerId}
            markerWidth={size * 2}
            markerHeight={size * 2}
            refX={size * 1.5}
            refY={size}
            orient="auto"
            markerUnits="strokeWidth"
          >
            <path
              d={`M 0,0 L ${size * 2},${size} L 0,${size * 2} z`}
              fill={color}
            />
          </marker>
        ))}
      </defs>

      {/* Render all connections */}
      <g className="connections-layer">
        {connections.map((connection) => (
          <ConnectionPath
            key={connection.id}
            connection={connection}
            isSelected={selectedConnectionIds.includes(connection.id)}
            isCreating={false}
          />
        ))}
      </g>

      {/* Render connection preview during creation */}
      {isCreating && (
        <g className="connection-preview-layer">
          <ConnectionPreview
            sourcePoint={creatingSourcePoint}
            currentPoint={creatingTempPoint}
          />
        </g>
      )}

      {/* Optional: Render connection points for debugging/visualization */}
      {showConnectionPoints &&
        connections.map((connection) => (
          <g key={`points-${connection.id}`}>
            <circle
              cx={connection.sourcePoint.x}
              cy={connection.sourcePoint.y}
              r={4}
              fill="#13b6ec"
            />
            <circle
              cx={connection.targetPoint.x}
              cy={connection.targetPoint.y}
              r={4}
              fill="#13b6ec"
            />
          </g>
        ))}
    </svg>
  );
});

/**
 * Props for ConnectionPointsOverlay
 */
interface ConnectionPointsOverlayProps {
  position: { x: number; y: number };
  size: { w: number; h: number };
  onPointClick?: (point: ConnectionPoint) => void;
  onPointMouseEnter?: (point: ConnectionPoint) => void;
  onPointMouseLeave?: () => void;
}

/**
 * Overlay showing connection points on an entity
 *
 * Renders small circles on each side of the entity that can be
 * used to start or end connections.
 */
export const ConnectionPointsOverlay = memo(function ConnectionPointsOverlay({
  position,
  size,
  onPointClick,
  onPointMouseEnter,
  onPointMouseLeave,
}: ConnectionPointsOverlayProps) {
  const points: ConnectionPoint[] = [
    { x: position.x + size.w / 2, y: position.y, side: "top" },
    { x: position.x + size.w / 2, y: position.y + size.h, side: "bottom" },
    { x: position.x, y: position.y + size.h / 2, side: "left" },
    { x: position.x + size.w, y: position.y + size.h / 2, side: "right" },
  ];

  return (
    <g className="connection-points-overlay">
      {points.map((point) => (
        <circle
          key={point.side}
          cx={point.x}
          cy={point.y}
          r={6}
          className="cursor-pointer transition-all duration-150"
          fill="rgba(19, 182, 236, 0.3)"
          stroke="#13b6ec"
          strokeWidth={2}
          onClick={() => onPointClick?.(point)}
          onMouseEnter={() => onPointMouseEnter?.(point)}
          onMouseLeave={onPointMouseLeave}
          style={{ pointerEvents: "all" }}
        />
      ))}
    </g>
  );
});

/**
 * Helper function to get connection points for an entity
 */
export function getConnectionPointsForEntity(
  _entityId: number,
  position: { x: number; y: number },
  size: { w: number; h: number },
): ConnectionPoint[] {
  return [
    { x: position.x + size.w / 2, y: position.y, side: "top" },
    { x: position.x + size.w / 2, y: position.y + size.h, side: "bottom" },
    { x: position.x, y: position.y + size.h / 2, side: "left" },
    { x: position.x + size.w, y: position.y + size.h / 2, side: "right" },
  ];
}
