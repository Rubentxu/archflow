/**
 * ArchFlow Behaviors SDK - Type Definitions
 *
 * This module defines the TypeScript types for the Behaviors SDK,
 * which provides a fluent API for creating interactive canvas shapes
 * that automatically map to the Logic Bricks system.
 *
 * Architecture Reference: EPIC-WEB-011
 */

/**
 * Shape types available in the canvas
 */
export type ShapeType =
  | "rectangle"
  | "circle"
  | "ellipse"
  | "path"
  | "text"
  | "image"
  | "group"
  | "connector";

/**
 * Shape identification
 */
export type ShapeId = string;

/**
 * 2D Point/Vector
 */
export interface Point2D {
  x: number;
  y: number;
}

/**
 * 2D Size
 */
export interface Size2D {
  width: number;
  height: number;
}

/**
 * Rectangle bounds
 */
export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Line cap style
 */
export type LineCap = "butt" | "round" | "square";

/**
 * Line join style
 */
export type LineJoin = "miter" | "round" | "bevel";

/**
 * Dash array for stroke patterns
 */
export type DashArray = number[];

/**
 * Shadow size presets
 */
export type ShadowSize = "none" | "xs" | "sm" | "md" | "lg" | "xl";

/**
 * Selection mode
 */
export type SelectionMode = "single" | "additive" | "range";

/**
 * Resize handle positions
 */
export type ResizeHandle = "nw" | "n" | "ne" | "w" | "e" | "sw" | "s" | "se";

/**
 * Drag axis constraints
 */
export type DragAxis = "x" | "y" | "both";

// ═══════════════════════════════════════════════════════════════════════════════
// RENDER CONFIGURATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Shadow configuration
 */
export interface ShadowConfig {
  color: number;
  blur: number;
  offsetX: number;
  offsetY: number;
}

/**
 * Hover state configuration
 */
export interface HoverConfig {
  color: number;
  opacity: number;
}

/**
 * Stroke configuration
 */
export interface StrokeConfig {
  width: number;
  color: number;
  lineCap?: LineCap;
  lineJoin?: LineJoin;
  dashArray?: DashArray;
}

// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR CONFIGURATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Behavior configuration for a single interaction
 */
export interface BehaviorConfig {
  /** Sensor type to use */
  sensor?: any;
  /** Controller to apply */
  controller?: any;
  /** Actuator to trigger */
  actuator?: any;
  /** Configuration parameters */
  config?: Record<string, unknown>;
}

/**
 * Draggable configuration
 */
export interface DraggableConfig {
  axis?: DragAxis;
  snap?: number;
}

/**
 * Resizable configuration
 */
export interface ResizableConfig {
  handles?: ResizeHandle[];
  snap?: number;
  keepAspectRatio?: boolean;
}

/**
 * Selectable configuration
 */
export interface SelectableConfig {
  mode?: SelectionMode;
}

/**
 * Multi-selectable configuration
 */
export interface MultiSelectableConfig {
  mode: SelectionMode;
  modifier?: "Shift" | "Ctrl" | "Alt";
}

/**
 * Droppable configuration
 */
export interface DroppableConfig {
  acceptTypes?: string[];
}

/**
 * Tooltip configuration
 */
export interface TooltipConfig {
  content: string;
  delay?: number;
  position?: "top" | "bottom" | "left" | "right";
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE INTERFACE
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Base shape interface
 */
export interface Shape {
  /** Unique identifier */
  id: ShapeId;
  /** Shape type */
  type: ShapeType;
  /** Current position */
  position: Point2D;
  /** Current size */
  size: Size2D;
  /** Rotation angle in degrees */
  rotation: number;
  /** Scale factor */
  scale: number;
  /** Opacity (0-1) */
  opacity: number;
  /** Z-index for rendering order */
  zIndex: number;
  /** Visibility flag */
  visible: boolean;
  /** Locked flag (no interaction) */
  locked: boolean;

  // Render properties
  /** Fill color (ARGB hex) */
  fillColor?: number | null;
  /** Border configuration */
  border?: StrokeConfig | null;
  /** Stroke configuration (for paths and connectors) */
  stroke?: StrokeConfig | null;
  /** Corner radius */
  cornerRadius?: number;
  /** Shadow configuration */
  shadow?: ShadowConfig | null;

  // Text properties (type === 'text')
  /** Text content */
  textContent?: string;
  /** Text color */
  textColor?: number;
  /** Font family */
  fontFamily?: string;
  /** Font size */
  fontSize?: number;
  /** Font weight */
  fontWeight?: string;
  /** Text alignment */
  textAlign?: "left" | "center" | "right";
  /** Vertical alignment */
  verticalAlign?: "top" | "middle" | "bottom";

  // Path properties (type === 'path')
  /** Path points */
  points?: Point2D[];
  /** Smooth the path */
  smooth?: boolean;
  /** Closed path flag */
  closed?: boolean;

  // Connector properties (type === 'connector')
  /** Connected shape ID */
  connectedTo?: ShapeId;
  /** Connector start position */
  startMarker?: MarkerType;
  /** Connector end position */
  endMarker?: MarkerType;

  // Group properties (type === 'group')
  /** Child shapes */
  children?: Shape[];

  // Metadata
  /** Custom data for the shape */
  data?: Record<string, unknown>;
  /** Tag for querying */
  tags?: string[];
  /** External reference ID */
  externalId?: string;

  // Internal state (not exposed via API)
  /** Last hovered state (internal) */
  lastHovered?: boolean;
}

/**
 * Marker type for connectors
 */
export type MarkerType = "none" | "arrow" | "circle" | "diamond" | "square";

/**
 * Canvas scene interface
 */
export interface CanvasScene {
  /** Create a new shape */
  createShape(type: ShapeType): Shape;

  /** Get shape by ID */
  getShape(id: ShapeId): Shape | undefined;

  /** Get all shapes */
  getAllShapes(): Shape[];

  /** Get shapes at a point (hit testing) */
  getShapesAtPoint(point: Point2D): Shape[];

  /** Get shapes by tag */
  getShapesByTag(tag: string): Shape[];

  /** Get shapes with specific behavior */
  getShapesWithBehavior(behavior: string): Shape[];

  /** Delete a shape */
  deleteShape(id: ShapeId): void;

  /** Start the render loop */
  startRenderLoop(): void;

  /** Stop the render loop */
  stopRenderLoop(): void;
}

/**
 * Entity ID (matches WASM EntityId)
 */
export type EntityId = number;

/**
 * Render configuration for canvas
 */
export interface RenderConfig {
  /** Background color */
  backgroundColor?: number;
  /** Grid size */
  gridSize?: number;
  /** Show grid flag */
  showGrid?: boolean;
  /** Grid color */
  gridColor?: number;
  /** Grid opacity */
  gridOpacity?: number;
}

/**
 * Viewport configuration
 */
export interface ViewportConfig {
  x: number;
  y: number;
  zoom: number;
}

/**
 * Canvas configuration
 */
export interface CanvasConfig {
  render?: RenderConfig;
  viewport?: ViewportConfig;
}
