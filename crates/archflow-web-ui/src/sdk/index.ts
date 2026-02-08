/**
 * ArchFlow Behaviors SDK - Main Entry Point
 *
 * This module provides the complete ArchFlow SDK for creating interactive
 * canvas shapes with the Logic Bricks system.
 *
 * Architecture Reference: EPIC-WEB-011
 *
 * @example
 * ```typescript
 * import { ArchFlow, createShape, createRectangle, createStickyNote } from '@archflow/sdk';
 *
 * // Using the fluent API
 * const note = ArchFlow.createRectangle()
 *   .position(100, 200)
 *   .size(200, 150)
 *   .fillColor(0xFFFF00)
 *   .cornerRadius(8)
 *   .onHover()
 *   .onClick(() => console.log('Clicked!'))
 *   .draggable()
 *   .build();
 *
 * // Using convenience methods
 * const sticky = createStickyNote()
 *   .position(300, 100)
 *   .textContent('Buy milk')
 *   .build();
 * ```
 */

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

export type {
  // Core types
  Shape,
  ShapeType,
  ShapeId,
  Point2D,
  Size2D,
  Bounds,
  EntityId,

  // Render types
  LineCap,
  LineJoin,
  DashArray,
  ShadowSize,
  ShadowConfig,
  StrokeConfig,

  // Behavior types
  SelectionMode,
  ResizeHandle,
  DragAxis,
  DraggableConfig,
  ResizableConfig,
  SelectableConfig,
  MultiSelectableConfig,
  DroppableConfig,
  TooltipConfig,

  // Canvas types
  CanvasScene,
  RenderConfig,
  ViewportConfig,
  CanvasConfig,
} from "./types";

// ═══════════════════════════════════════════════════════════════════════════════
// WASM BRIDGE FACADES
// ═══════════════════════════════════════════════════════════════════════════════

// Re-export all bridge facades for direct use
export { EntityBridge, createEntityBridge } from "./EntityBridge";
export {
  SelectionBridge,
  createSelectionBridge,
} from "./SelectionBridge";
export { CameraBridge, createCameraBridge } from "./CameraBridge";
export { InputBridge, createInputBridge } from "./InputBridge";
export { HistoryBridge, createHistoryBridge } from "./HistoryBridge";
export { EventsBridge, createEventsBridge } from "./EventsBridge";
export {
  EventEmitter,
  globalEvents,
  type DomainEvents,
  type Callback,
  type EventListenerOptions,
  type Subscription,
  type EventMetricsSnapshot,
  type Vector2,
  type MouseButton,
  type SystemStatus,
} from "./EventEmitter";

// Event System Components
// Event System Components
export {
  EventWorkerPool,
  EventWorker,
  createEventWorker,
  createEventWorkerPool,
  getGlobalWorkerPool,
  terminateGlobalWorkerPool,
} from "./eventWorker";
export type {
  WorkerMessage,
  WorkerPoolConfig as WorkerConfig,
  HeavyCalculationPayload,
  HeavyCalculationResult,
  BatchProcessingPayload,
  ComplexOperationPayload,
} from "./eventWorker";

export {
  EventTracker,
  EventDevTools,
  useEventTracker,
  useEventMetrics,
} from "./eventTracker.tsx";
export type {
  EventLogEntry,
  EventMetricsSnapshot as TrackerMetricsSnapshot,
  EventDirection,
} from "./eventTracker.tsx";

export {
  ToolsBridge,
  createToolsBridge,
  TOOL_REGISTRY,
} from "./ToolsBridge";
export {
  BehaviorBridge,
  createBehaviorBridge,
  BEHAVIOR_TEMPLATES,
} from "./BehaviorBridge";

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN SDK FACADE
// ═══════════════════════════════════════════════════════════════════════════════

export {
  ArchFlowBridge,
  createArchFlowBridge,
} from "./ArchFlowBridge";
export type { ArchFlowBridgeOptions } from "./ArchFlowBridge";

// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR REGISTRY & TEMPLATES
// ═══════════════════════════════════════════════════════════════════════════════

export { behaviorRegistry } from "./BehaviorRegistry";

export type { BehaviorTemplate } from "./BehaviorRegistry";

export type { BehaviorConfig } from "./types";

// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE BUILDER API
// ═══════════════════════════════════════════════════════════════════════════════

export { ShapeBuilder } from "./ShapeBuilder";

// ═══════════════════════════════════════════════════════════════════════════════
// LOGIC BRICKS SDK
// ═══════════════════════════════════════════════════════════════════════════════

export {
  LogicSDK,
  EntityBuilder,
  createLogicSDK,
  customControllerRegistry,
  Ctrl,
  Timing,
} from "./logic-sdk";

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TYPES
// ═══════════════════════════════════════════════════════════════════════════════

export type {
  ActuatorType,
  ControllerType,
  SensorType,
} from "../wasm/archflow_web.d";

export {
  Controller,
  LogicMappingTableWasm,
  SignalByteWasm,
  WasmBridge,
} from "../wasm/archflow_web.d";

// ═══════════════════════════════════════════════════════════════════════════════
// ARCHFLOW MAIN API
// ═══════════════════════════════════════════════════════════════════════════════

import { ShapeBuilder } from "./ShapeBuilder";
import type { Shape, ShapeType } from "./types";

/**
 * ArchFlow - Main SDK namespace
 *
 * Provides the complete API for creating interactive canvas shapes.
 * All shapes are built using the fluent ShapeBuilder API and automatically
 * map behaviors to the Logic Bricks system.
 *
 * @example
 * ```typescript
 * import { ArchFlow } from '@archflow/sdk';
 *
 * const rect = ArchFlow.createRectangle()
 *   .position(100, 100)
 *   .size(200, 100)
 *   .fillColor(0xFF0000)
 *   .onHover()
 *   .draggable()
 *   .build();
 * ```
 */
export class ArchFlow {
  /**
   * Create a shape using the fluent builder API
   *
   * @param type - The shape type to create
   * @returns ShapeBuilder for fluent configuration
   *
   * @example
   * ```typescript
   * const shape = ArchFlow.createShape('rectangle')
   *   .position(100, 100)
   *   .size(200, 100)
   *   .build();
   * ```
   */
  static createShape(type: ShapeType): ShapeBuilder {
    return new ShapeBuilder(type);
  }

  /**
   * Create a rectangle shape
   *
   * @returns ShapeBuilder configured for rectangle
   *
   * @example
   * ```typescript
   * const rect = ArchFlow.createRectangle()
   *   .position(100, 100)
   *   .size(200, 100)
   *   .fillColor(0xFF0000)
   *   .onHover()
   *   .build();
   * ```
   */
  static createRectangle(): ShapeBuilder {
    return new ShapeBuilder("rectangle");
  }

  /**
   * Create a circle shape
   *
   * @returns ShapeBuilder configured for circle
   *
   * @example
   * ```typescript
   * const circle = ArchFlow.createCircle()
   *   .center(200, 200)
   *   .size(100, 100)
   *   .fillColor(0x00FF00)
   *   .build();
   * ```
   */
  static createCircle(): ShapeBuilder {
    return new ShapeBuilder("circle");
  }

  /**
   * Create an ellipse shape
   *
   * @returns ShapeBuilder configured for ellipse
   *
   * @example
   * ```typescript
   * const ellipse = ArchFlow.createEllipse()
   *   .center(200, 200)
   *   .size(200, 100)
   *   .fillColor(0x0000FF)
   *   .build();
   * ```
   */
  static createEllipse(): ShapeBuilder {
    return new ShapeBuilder("ellipse");
  }

  /**
   * Create a path shape (freehand drawing)
   *
   * @returns ShapeBuilder configured for path
   *
   * @example
   * ```typescript
   * const path = ArchFlow.createPath()
   *   .addPoint(100, 100)
   *   .addPoint(150, 150)
   *   .addPoint(200, 100)
   *   .smooth()
   *   .stroke(2, 0x000000)
   *   .build();
   * ```
   */
  static createPath(): ShapeBuilder {
    return new ShapeBuilder("path");
  }

  /**
   * Create a text shape
   *
   * @param text - The text content
   * @returns ShapeBuilder configured for text
   *
   * @example
   * ```typescript
   * const text = ArchFlow.createText('Hello, World!')
   *   .position(100, 100)
   *   .fontSize(24)
   *   .textColor(0x000000)
   *   .build();
   * ```
   */
  static createText(text: string): ShapeBuilder {
    return new ShapeBuilder("text").textContent(text);
  }

  /**
   * Create a sticky note shape with default styling
   *
   * @param text - The note content
   * @returns ShapeBuilder configured as sticky note
   *
   * @example
   * ```typescript
   * const note = ArchFlow.createStickyNote('Buy milk')
   *   .position(100, 200)
   *   .onHover()
   *   .draggable()
   *   .build();
   * ```
   */
  static createStickyNote(text?: string): ShapeBuilder {
    const builder = new ShapeBuilder("rectangle")
      .fillColor(0xffff00) // Yellow
      .cornerRadius(8)
      .shadow("md")
      .border(2, 0xcccccc)
      .onHover({ color: 0xffff00, opacity: 0.2 })
      .draggable({ snap: 8 })
      .resizable();

    // If text is provided, create as a text-enabled shape
    if (text !== undefined) {
      builder.textContent(text);
    }

    return builder;
  }

  /**
   * Create a card shape with shadow and border
   *
   * @returns ShapeBuilder configured as card
   *
   * @example
   * ```typescript
   * const card = ArchFlow.createCard()
   *   .position(100, 100)
   *   .size(300, 200)
   *   .interactive()
   *   .build();
   * ```
   */
  static createCard(): ShapeBuilder {
    return new ShapeBuilder("rectangle")
      .fillColor(0xffffff)
      .cornerRadius(12)
      .shadow("lg")
      .border(1, 0xcccccc);
  }

  /**
   * Create a diagram node shape
   *
   * @param _label - The node label (unused in current implementation)
   * @returns ShapeBuilder configured as diagram node
   *
   * @example
   * ```typescript
   * const node = ArchFlow.createNode('Process')
   *   .position(200, 200)
   *   .size(120, 60)
   *   .selectable()
   *   .build();
   * ```
   */
  static createNode(_label?: string): ShapeBuilder {
    return new ShapeBuilder("rectangle")
      .fillColor(0xffffff)
      .border(2, 0x333333)
      .cornerRadius(6)
      .selectable("single");
  }

  /**
   * Create a connector (line) between shapes
   *
   * @param fromId - The starting shape ID (stored in metadata)
   * @param toId - The ending shape ID
   * @returns ShapeBuilder configured as connector
   *
   * @example
   * ```typescript
   * const connector = ArchFlow.createConnector(node1.id, node2.id)
   *   .stroke(2, 0x666666)
   *   .endMarker('arrow')
   *   .build();
   * ```
   */
  static createConnector(fromId: string, toId: string): ShapeBuilder {
    return new ShapeBuilder("connector")
      .connectTo(toId)
      .stroke(2, 0x666666)
      .endMarker("arrow")
      .data({ fromId });
  }

  /**
   * Create an image shape
   *
   * @param src - The image source URL
   * @returns ShapeBuilder configured for image
   *
   * @example
   * ```typescript
   * const img = ArchFlow.createImage('/photo.jpg')
   *   .position(100, 100)
   *   .size(300, 200)
   *   .shadow('md')
   *   .build();
   * ```
   */
  static createImage(src: string): ShapeBuilder {
    return new ShapeBuilder("image").data({ src });
  }

  /**
   * Create a group from multiple shapes
   *
   * @param shapes - The shapes to group
   * @returns ShapeBuilder configured as group
   *
   * @example
   * ```typescript
   * const group = ArchFlow.createGroup(shape1, shape2, shape3)
   *   .draggable()
   *   .build();
   * ```
   */
  static createGroup(...shapes: Shape[]): ShapeBuilder {
    return ShapeBuilder.group(...shapes);
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Create a shape using the fluent builder API
 *
 * This is a convenience function equivalent to `ArchFlow.createShape()`.
 *
 * @param type - The shape type to create
 * @returns ShapeBuilder for fluent configuration
 *
 * @example
 * ```typescript
 * import { createShape } from '@archflow/sdk';
 *
 * const shape = createShape('rectangle')
 *   .position(100, 100)
 *   .build();
 * ```
 */
export function createShape(type: ShapeType): ShapeBuilder {
  return ArchFlow.createShape(type);
}

/**
 * Create a rectangle shape
 *
 * @example
 * ```typescript
 * import { createRectangle } from '@archflow/sdk';
 *
 * const rect = createRectangle()
 *   .position(100, 100)
 *   .size(200, 100)
 *   .fillColor(0xFF0000)
 *   .build();
 * ```
 */
export function createRectangle(): ShapeBuilder {
  return ArchFlow.createRectangle();
}

/**
 * Create a circle shape
 *
 * @example
 * ```typescript
 * import { createCircle } from '@archflow/sdk';
 *
 * const circle = createCircle()
 *   .center(200, 200)
 *   .size(100, 100)
 *   .fillColor(0x00FF00)
 *   .build();
 * ```
 */
export function createCircle(): ShapeBuilder {
  return ArchFlow.createCircle();
}

/**
 * Create an ellipse shape
 */
export function createEllipse(): ShapeBuilder {
  return ArchFlow.createEllipse();
}

/**
 * Create a path shape
 */
export function createPath(): ShapeBuilder {
  return ArchFlow.createPath();
}

/**
 * Create a text shape
 *
 * @param text - The text content
 */
export function createText(text: string): ShapeBuilder {
  return ArchFlow.createText(text);
}

/**
 * Create a sticky note with default styling
 *
 * @param text - The note content
 */
export function createStickyNote(text?: string): ShapeBuilder {
  return ArchFlow.createStickyNote(text);
}

/**
 * Create a card with shadow and border
 */
export function createCard(): ShapeBuilder {
  return ArchFlow.createCard();
}

/**
 * Create a diagram node
 *
 * @param label - The node label
 */
export function createNode(label?: string): ShapeBuilder {
  return ArchFlow.createNode(label);
}

/**
 * Create a connector between shapes
 *
 * @param fromId - Starting shape ID
 * @param toId - Ending shape ID
 */
export function createConnector(fromId: string, toId: string): ShapeBuilder {
  return ArchFlow.createConnector(fromId, toId);
}

/**
 * Create an image shape
 *
 * @param src - Image source URL
 */
export function createImage(src: string): ShapeBuilder {
  return ArchFlow.createImage(src);
}

/**
 * Create a group from shapes
 *
 * @param shapes - Shapes to group
 */
export function createGroup(...shapes: Shape[]): ShapeBuilder {
  return ArchFlow.createGroup(...shapes);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * ArchFlow SDK version
 */
export const SDK_VERSION = "0.1.0";

/**
 * Default canvas grid size
 */
export const DEFAULT_GRID_SIZE = 20;

/**
 * Default snap size for dragging
 */
export const DEFAULT_SNAP_SIZE = 8;

/**
 * Default colors
 */
export const Colors = {
  /** Yellow for sticky notes */
  StickyNote: 0xffff00,
  /** White for cards */
  Card: 0xffffff,
  /** Gray for borders */
  Border: 0xcccccc,
  /** Dark gray for connectors */
  Connector: 0x666666,
  /** Black for text */
  Text: 0x000000,
  /** Red highlight */
  HighlightRed: 0xff0000,
  /** Green highlight */
  HighlightGreen: 0x00ff00,
  /** Blue highlight */
  HighlightBlue: 0x0000ff,
  /** Yellow highlight */
  HighlightYellow: 0xffff00,
} as const;

// ═══════════════════════════════════════════════════════════════════════════════
// RE-EXPORT DEFAULT
// ═══════════════════════════════════════════════════════════════════════════════

export default ArchFlow;
