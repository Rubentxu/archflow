/**
 * ArchFlow Behaviors SDK - ShapeBuilder API
 *
 * Fluent API for creating interactive canvas shapes.
 * All method chaining ultimately maps to the Logic Bricks system internally.
 *
 * Architecture Reference: EPIC-WEB-011, Section 3.2
 *
 * @example
 * ```typescript
 * import { ArchFlow } from '@archflow/sdk';
 *
 * const note = ArchFlow.createShape('rectangle')
 *   .position(100, 200)
 *   .size(200, 150)
 *   .fillColor(0xFFFF00)
 *   .cornerRadius(8)
 *   .onHover()
 *   .onClick(() => selectNote())
 *   .draggable()
 *   .build();
 * ```
 */

import type {
  Shape,
  ShapeType,
  ShapeId,
  Point2D,
  LineCap,
  LineJoin,
  DashArray,
  ShadowSize,
  SelectionMode,
  ResizeHandle,
  DragAxis,
  BehaviorConfig,
} from "./types";
import { SensorType, Controller, ActuatorType } from "../wasm/archflow_web.d";
import { behaviorRegistry } from "./BehaviorRegistry";
import { customControllerRegistry } from "./logic-sdk";

/**
 * ShapeBuilder - Fluent API for building shapes
 *
 * Provides chainable methods for configuring all aspects of a shape:
 * - Geometry (position, size, rotation, scale)
 * - Render properties (fill, border, shadow, stroke)
 * - Behaviors (hover, click, drag, resize, select)
 * - Metadata (tags, data, externalId)
 *
 * All methods return `this` for fluent chaining.
 */
export class ShapeBuilder {
  private shape: Shape;
  private behaviorConfigs: Array<[string, BehaviorConfig]> = [];
  private eventHandlers: Map<string, Function[]> = new Map();

  constructor(type: ShapeType, existingShape?: Shape) {
    // Create or use existing shape
    this.shape =
      existingShape ||
      ({
        id: generateShapeId(),
        type,
        position: { x: 0, y: 0 },
        size: { width: 100, height: 100 },
        rotation: 0,
        scale: 1,
        opacity: 1,
        zIndex: 0,
        visible: true,
        locked: false,
      } as Shape);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // GEOMETRY - Position and size
  // ═════════════════════════════════════════════════════════════════════════════

  /**
   * Set position (x, y)
   * @returns this for chaining
   */
  position(x: number, y: number): this {
    this.shape.position = { x, y };
    return this;
  }

  /**
   * Set X position
   * @returns this for chaining
   */
  x(value: number): this {
    this.shape.position.x = value;
    return this;
  }

  /**
   * Set Y position
   * @returns this for chaining
   */
  y(value: number): this {
    this.shape.position.y = value;
    return this;
  }

  /**
   * Set size (width, height)
   * @returns this for chaining
   */
  size(width: number, height: number): this {
    this.shape.size = { width, height };
    return this;
  }

  /**
   * Set width
   * @returns this for chaining
   */
  width(value: number): this {
    this.shape.size.width = value;
    return this;
  }

  /**
   * Set height
   * @returns this for chaining
   */
  height(value: number): this {
    this.shape.size.height = value;
    return this;
  }

  /**
   * Set bounds (x, y, width, height) - combines position and size
   * @returns this for chaining
   */
  bounds(x: number, y: number, width: number, height: number): this {
    this.shape.position = { x, y };
    this.shape.size = { width, height };
    return this;
  }

  /**
   * Center the shape at the given point (adjusts position based on size)
   * @returns this for chaining
   */
  center(x: number, y: number): this {
    const w = this.shape.size.width;
    const h = this.shape.size.height;
    this.shape.position = { x: x - w / 2, y: y - h / 2 };
    return this;
  }

  /**
   * Set rotation angle in degrees
   * @returns this for chaining
   */
  rotation(degrees: number): this {
    this.shape.rotation = degrees;
    return this;
  }

  /**
   * Set scale factor
   * @returns this for chaining
   */
  scale(factor: number): this {
    this.shape.scale = factor;
    return this;
  }

  /**
   * Set opacity (0-1)
   * @returns this for chaining
   */
  opacity(value: number): this {
    this.shape.opacity = Math.max(0, Math.min(1, value));
    return this;
  }

  /**
   * Set z-index (rendering order)
   * @returns this for chaining
   */
  zIndex(value: number): this {
    this.shape.zIndex = value;
    return this;
  }

  /**
   * Set visibility
   * @returns this for chaining
   */
  visible(value: boolean): this {
    this.shape.visible = value;
    return this;
  }

  /**
   * Set locked state (no interaction)
   * @returns this for chaining
   */
  locked(value = true): this {
    this.shape.locked = value;
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // RENDER PROPERTIES - Visual appearance
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set fill color (ARGB hex format: 0xAABBGGRR)
   * @returns this for chaining
   */
  fillColor(hex: number): this {
    this.shape.fillColor = hex;
    return this;
  }

  /**
   * Remove fill (make transparent)
   * @returns this for chaining
   */
  noFill(): this {
    this.shape.fillColor = null;
    return this;
  }

  /**
   * Set border (width, color, optional style)
   * @returns this for chaining
   */
  border(width: number, color: number, _style?: string): this {
    this.shape.border = { width, color };
    return this;
  }

  /**
   * Remove border
   * @returns this for chaining
   */
  noBorder(): this {
    this.shape.border = null;
    return this;
  }

  /**
   * Set corner radius (for rectangles)
   * @returns this for chaining
   */
  cornerRadius(radius: number): this {
    this.shape.cornerRadius = radius;
    return this;
  }

  /**
   * Set shadow using preset size
   * @returns this for chaining
   */
  shadow(size: ShadowSize): this {
    const shadowConfigs = {
      none: { color: 0, blur: 0, offsetX: 0, offsetY: 0 },
      xs: { color: 0x000000, blur: 2, offsetX: 0, offsetY: 1 },
      sm: { color: 0x000000, blur: 4, offsetX: 0, offsetY: 2 },
      md: { color: 0x000000, blur: 8, offsetX: 0, offsetY: 4 },
      lg: { color: 0x000000, blur: 16, offsetX: 0, offsetY: 8 },
      xl: { color: 0x000000, blur: 32, offsetX: 0, offsetY: 16 },
    };
    this.shape.shadow = shadowConfigs[size];
    return this;
  }

  /**
   * Set custom shadow configuration
   * @returns this for chaining
   */
  customShadow(config: {
    color: number;
    blur: number;
    offsetX: number;
    offsetY: number;
  }): this {
    this.shape.shadow = config;
    return this;
  }

  /**
   * Set stroke (for lines and connectors)
   * @returns this for chaining
   */
  stroke(width: number, color: number): this {
    this.shape.stroke = { width, color };
    return this;
  }

  /**
   * Set line cap style (for paths)
   * @returns this for chaining
   */
  lineCap(style: LineCap): this {
    if (this.shape.stroke) {
      this.shape.stroke.lineCap = style;
    }
    return this;
  }

  /**
   * Set line join style (for paths)
   * @returns this for chaining
   */
  lineJoin(style: LineJoin): this {
    if (this.shape.stroke) {
      this.shape.stroke.lineJoin = style;
    }
    return this;
  }

  /**
   * Set dash array for dashed lines
   * @returns this for chaining
   */
  dashArray(dash: DashArray): this {
    if (this.shape.stroke) {
      this.shape.stroke.dashArray = dash;
    }
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // TEXT PROPERTIES - For text shapes
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set text content
   * @returns this for chaining
   */
  textContent(text: string): this {
    if (this.shape.type === "text") {
      this.shape.textContent = text;
    }
    return this;
  }

  /**
   * Set text color (ARGB hex)
   * @returns this for chaining
   */
  textColor(hex: number): this {
    if (this.shape.type === "text") {
      this.shape.textColor = hex;
    }
    return this;
  }

  /**
   * Set font family
   * @returns this for chaining
   */
  fontFamily(fontFamily: string): this {
    if (this.shape.type === "text") {
      this.shape.fontFamily = fontFamily;
    }
    return this;
  }

  /**
   * Set font size
   * @returns this for chaining
   */
  fontSize(size: number): this {
    if (this.shape.type === "text") {
      this.shape.fontSize = size;
    }
    return this;
  }

  /**
   * Set font weight
   * @returns this for chaining
   */
  fontWeight(weight: string): this {
    if (this.shape.type === "text") {
      this.shape.fontWeight = weight;
    }
    return this;
  }

  /**
   * Set text alignment
   * @returns this for chaining
   */
  textAlign(align: "left" | "center" | "right"): this {
    if (this.shape.type === "text") {
      this.shape.textAlign = align;
    }
    return this;
  }

  /**
   * Set vertical alignment
   * @returns this for chaining
   */
  verticalAlign(align: "top" | "middle" | "bottom"): this {
    if (this.shape.type === "text") {
      this.shape.verticalAlign = align;
    }
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // PATH PROPERTIES - For path shapes
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set path points
   * @returns this for chaining
   */
  points(points: Point2D[]): this {
    if (this.shape.type === "path") {
      this.shape.points = points;
    }
    return this;
  }

  /**
   * Add a point to the path
   * @returns this for chaining
   */
  addPoint(x: number, y: number): this {
    if (this.shape.type === "path") {
      this.shape.points = this.shape.points || [];
      this.shape.points.push({ x, y });
    }
    return this;
  }

  /**
   * Smooth the path using curve fitting
   * @returns this for chaining
   */
  smooth(): this {
    if (this.shape.type === "path") {
      this.shape.smooth = true;
    }
    return this;
  }

  /**
   * Close the path (connect last point to first)
   * @returns this for chaining
   */
  closed(): this {
    if (this.shape.type === "path") {
      this.shape.closed = true;
    }
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // CONNECTOR PROPERTIES - For connector shapes
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Connect to another shape
   * @returns this for chaining
   */
  connectTo(shapeId: ShapeId): this {
    if (this.shape.type === "connector") {
      this.shape.connectedTo = shapeId;
    }
    return this;
  }

  /**
   * Set connector style
   * @returns this for chaining
   */
  connectorStyle(width: number, color: number): this {
    if (this.shape.type === "connector") {
      this.shape.stroke = { width, color };
    }
    return this;
  }

  /**
   * Set end marker
   * @returns this for chaining
   */
  endMarker(marker: "none" | "arrow" | "circle" | "diamond" | "square"): this {
    if (this.shape.type === "connector") {
      this.shape.endMarker = marker;
    }
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // BEHAVIORS - User interactions
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Add hover behavior
   * @param config - Optional hover configuration
   * @returns this for chaining
   */
  onHover(config?: { color?: number; opacity?: number }): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.MouseOver,
      controller: Controller.direct(),
      actuator: ActuatorType.Highlight,
      config: {
        color: config?.color ?? 0xffff00,
        opacity: config?.opacity ?? 0.3,
      },
    };
    this.behaviorConfigs.push(["onHover", behaviorConfig]);
    return this;
  }

  /**
   * Add click behavior
   * @param callback - Function to call on click
   * @returns this for chaining
   */
  onClick(callback: () => void): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.MouseClick,
      controller: Controller.direct(),
      actuator: ActuatorType.Select,
      config: {},
    };
    this.behaviorConfigs.push(["onClick", behaviorConfig]);
    this.eventHandlers.set("onClick", [callback]);
    return this;
  }

  /**
   * Add double-click behavior
   * @param callback - Function to call on double-click
   * @returns this for chaining
   */
  onDoubleClick(callback: () => void): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.MouseClick,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { handler: "doubleClick" },
    };
    this.behaviorConfigs.push(["onDoubleClick", behaviorConfig]);
    this.eventHandlers.set("onDoubleClick", [callback]);
    return this;
  }

  /**
   * Add right-click behavior
   * @param callback - Function to call on right-click
   * @returns this for chaining
   */
  onRightClick(callback: () => void): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.KeyShortcut,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { handler: "rightClick" },
    };
    this.behaviorConfigs.push(["onRightClick", behaviorConfig]);
    this.eventHandlers.set("onRightClick", [callback]);
    return this;
  }

  /**
   * Make the shape draggable
   * @param config - Draggable configuration
   * @returns this for chaining
   */
  draggable(config?: { axis?: DragAxis; snap?: number }): this {
    const behaviorConfig: BehaviorConfig = {
      config: {
        axis: config?.axis ?? "both",
        snap: config?.snap ?? 8,
      },
    };
    this.behaviorConfigs.push(["draggable", behaviorConfig]);
    return this;
  }

  /**
   * Make the shape resizable
   * @param config - Resizable configuration
   * @returns this for chaining
   */
  resizable(config?: { handles?: ResizeHandle[]; snap?: number }): this {
    const behaviorConfig: BehaviorConfig = {
      config: {
        handles: config?.handles ?? ["nw", "ne", "sw", "se"],
        snap: config?.snap ?? 8,
      },
    };
    this.behaviorConfigs.push(["resizable", behaviorConfig]);
    return this;
  }

  /**
   * Make the shape selectable
   * @param mode - Selection mode
   * @returns this for chaining
   */
  selectable(mode: SelectionMode = "single"): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.MouseClick,
      controller: Controller.direct(),
      actuator: ActuatorType.Select,
      config: { mode },
    };
    this.behaviorConfigs.push(["selectable", behaviorConfig]);
    return this;
  }

  /**
   * Make the shape multi-selectable (with Shift modifier)
   * @returns this for chaining
   */
  multiSelectable(modifier: "Shift" | "Ctrl" | "Alt" = "Shift"): this {
    const behaviorConfig: BehaviorConfig = {
      config: {
        mode: "multi",
        modifier,
      },
    };
    this.behaviorConfigs.push(["selectable-multi", behaviorConfig]);
    return this;
  }

  /**
   * Add a tooltip on hover
   * @param content - Tooltip text
   * @param delay - Delay in ticks before showing tooltip
   * @returns this for chaining
   */
  tooltip(content: string, delay = 6): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.MouseOver,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { content, delay },
    };
    this.behaviorConfigs.push(["tooltip", behaviorConfig]);
    return this;
  }

  /**
   * Add key down behavior
   * @param _key - Key code or character (not used in current implementation)
   * @param callback - Function to call when key is pressed
   * @returns this for chaining
   */
  onKeyDown(_key: string, callback: () => void): this {
    this.eventHandlers.set("onKeyDown", [callback]);
    return this;
  }

  /**
   * Add delete behavior (Delete key)
   * @returns this for chaining
   */
  onDelete(): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.KeyShortcut,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { handler: "delete" },
    };
    this.behaviorConfigs.push(["onDelete", behaviorConfig]);
    return this;
  }

  /**
   * Add escape key behavior
   * @returns this for chaining
   */
  onEscape(): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.KeyShortcut,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { handler: "escape" },
    };
    this.behaviorConfigs.push(["onEscape", behaviorConfig]);
    return this;
  }

  /**
   * Add copy behavior (Ctrl+C)
   * @returns this for chaining
   */
  onCopy(): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.KeyShortcut,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { handler: "copy" },
    };
    this.behaviorConfigs.push(["onCopy", behaviorConfig]);
    return this;
  }

  /**
   * Add paste behavior (Ctrl+V)
   * @returns this for chaining
   */
  onPaste(): this {
    const behaviorConfig: BehaviorConfig = {
      sensor: SensorType.KeyShortcut,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { handler: "paste" },
    };
    this.behaviorConfigs.push(["onPaste", behaviorConfig]);
    return this;
  }

  /**
   * Add custom shortcut behavior
   * @param _keys - Key combination (e.g., 'Ctrl+Shift+S')
   * @param callback - Function to call
   * @returns this for chaining
   */
  onShortcut(_keys: string, callback: () => void): this {
    this.eventHandlers.set("onShortcut", [callback]);
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // LIFECYCLE HOOKS
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Add onCreate callback
   * @param callback - Function to call when shape is created
   * @returns this for chaining
   */
  onCreate(callback: () => void): this {
    this.eventHandlers.set("onCreate", [callback]);
    return this;
  }

  /**
   * Add onDestroy callback
   * @param callback - Function to call when shape is destroyed
   * @returns this for chaining
   */
  onDestroy(callback: () => void): this {
    this.eventHandlers.set("onDestroy", [callback]);
    return this;
  }

  /**
   * Add onUpdate callback
   * @param callback - Function to call when shape is modified
   * @returns this for chaining
   */
  onUpdate(callback: () => void): this {
    this.eventHandlers.set("onUpdate", [callback]);
    return this;
  }

  /**
   * Add onSelect callback
   * @param callback - Function to call when shape is selected
   * @returns this for chaining
   */
  onSelect(callback: () => void): this {
    this.eventHandlers.set("onSelect", [callback]);
    return this;
  }

  /**
   * Add onDeselect callback
   * @param callback - Function to call when shape is deselected
   * @returns this for chaining
   */
  onDeselect(callback: () => void): this {
    this.eventHandlers.set("onDeselect", [callback]);
    return this;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // METADATA
  // ═════════════════════════════════════════════════════════════════════════════

  /**
   * Set custom data on the shape
   * @param data - Data object
   * @returns this for chaining
   */
  data(data: Record<string, unknown>): this {
    this.shape.data = { ...this.shape.data, ...data };
    return this;
  }

  /**
   * Add a tag for querying
   * @param tag - Tag to add
   * @returns this for chaining
   */
  tag(tag: string): this {
    if (!this.shape.tags) {
      this.shape.tags = [];
    }
    this.shape.tags.push(tag);
    return this;
  }

  /**
   * Set external reference ID
   * @param id - External ID
   * @returns this for chaining
   */
  externalId(id: string): this {
    this.shape.externalId = id;
    return this;
  }

  // ═════════════════════════════════════════════════════════════════════════════
  // GROUP METHODS
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Add a child shape (for group shapes)
   * @returns this for chaining
   */
  addChild(child: Shape): this {
    if (this.shape.type === "group") {
      this.shape.children = this.shape.children || [];
      this.shape.children.push(child);
    }
    return this;
  }

  /**
   * Create a group from multiple shapes
   * @returns New ShapeBuilder for the group
   */
  static group(...shapes: Shape[]): ShapeBuilder {
    const groupShape: Shape = {
      id: generateShapeId(),
      type: "group",
      position: { x: 0, y: 0 },
      size: { width: 0, height: 0 },
      rotation: 0,
      scale: 1,
      opacity: 1,
      zIndex: 0,
      visible: true,
      locked: false,
      children: [...shapes],
    };

    // Calculate bounding box
    shapes.forEach((s) => {
      groupShape.size.width = Math.max(
        groupShape.size.width,
        s.position.x + s.size.width,
      );
      groupShape.size.height = Math.max(
        groupShape.size.height,
        s.position.y + s.size.height,
      );
    });

    return new ShapeBuilder("group", groupShape);
  }

  // ═════════════════════════════════════════════════════════════════════════════
  // BUILD - Create the shape and apply all behaviors
  // ═════════════════════════════════════════════════════════════════════════════════

  /**
   * Build the shape and return it
   * @returns The created shape
   */
  build(): Shape {
    // Apply all behavior configurations through Logic Bricks
    this.behaviorConfigs.forEach(([behaviorName, _config]) => {
      behaviorRegistry.apply(this.shape, behaviorName);
    });

    // Register event handlers
    this.eventHandlers.forEach((handlers, handlerName) => {
      if (handlers.length > 0 && typeof handlers[0] === "function") {
        const callback = handlers[0] as Function;
        const uniqueHandlerName = `${this.shape.id}_${handlerName}`;
        customControllerRegistry.register(
          uniqueHandlerName,
          (_signal: any, _context: any) => {
            // Invoke the callback
            return callback(this.shape);
          },
        );
      }
    });

    return this.shape;
  }

  /**
   * Get the underlying shape (without building)
   * @returns The shape being built
   */
  getShape(): Shape {
    return this.shape;
  }
}

/**
 * Generate unique shape IDs
 */
function generateShapeId(): ShapeId {
  return `shape_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}
