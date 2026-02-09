/**
 * ArchFlow BehaviorBuilder - Declarative Fluent API for Behaviors
 *
 * Provides a fluent builder pattern for creating behaviors declaratively.
 * Inspired by tldraw's behavior system and React Hooks API.
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * USAGE EXAMPLE
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * import { BehaviorBuilder } from './BehaviorBuilder';
 * import { behaviorTemplates } from './BehaviorTemplates';
 *
 * // Create a draggable behavior
 * const draggable = new BehaviorBuilder(archflow.logicSystem)
 *   .onPointerDown()
 *   .onDrag()
 *   .translate()
 *   .onPointerUp()
 *   .build();
 *
 * // Attach to entity
 * draggable.attach(entityId);
 *
 * // Use with React
 * function DraggableShape({ id }) {
 *   useBehavior(id, draggable);
 *   return <Shape id={id} />;
 * }
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 */

import type {
  SensorType,
  ControllerType,
  ActuatorType,
  LogicSystemWasm,
  BehaviorBridgeWasm,
} from '../wasm/archflow_web.d';

// ============================================================================
// TYPES
// ============================================================================

/**
 * Configuration for a sensor
 */
export interface SensorConfig {
  type: SensorType | string;
  target?: string;
  options?: Record<string, unknown>;
}

/**
 * Configuration for a controller
 */
export interface ControllerConfig {
  type: ControllerType | string;
  options?: Record<string, unknown>;
}

/**
 * Configuration for an actuator
 */
export interface ActuatorConfig {
  type: ActuatorType | string;
  action: string | ((ctx: ExecutionContext) => void);
  options?: Record<string, unknown>;
}

/**
 * Complete behavior configuration
 */
export interface BehaviorConfig {
  sensors: SensorConfig[];
  controllers: ControllerConfig[];
  actuators: ActuatorConfig[];
  metadata?: BehaviorMetadata;
}

/**
 * Metadata for a behavior
 */
export interface BehaviorMetadata {
  name: string;
  description?: string;
  version?: string;
  tags?: string[];
}

/**
 * Execution context passed to custom actions
 */
export interface ExecutionContext {
  entityId: string;
  state: 'idle' | 'active' | 'complete' | 'error';
  event: PointerEvent | KeyboardEvent | null;
  point: Point | null;
  delta: Point | null;
  select: () => void;
  deselect: () => void;
  toggle: () => void;
  translate: (dx?: number, dy?: number) => void;
  resize: (width: number, height: number) => void;
  rotate: (angle: number) => void;
  delete: () => void;
  emit: (event: string, data?: unknown) => void;
  highlight: (style?: HighlightStyle) => void;
  getState: <T>(key: string) => T | undefined;
  setState: <T>(key: string, value: T) => void;
}

/**
 * Point geometry
 */
export interface Point {
  x: number;
  y: number;
}

/**
 * Size geometry
 */
export interface Size {
  width: number;
  height: number;
}

/**
 * Highlight style configuration
 */
export interface HighlightStyle {
  color?: number;
  opacity?: number;
}

/**
 * Modifier keys state
 */
export interface ModifierKeys {
  shift?: boolean;
  ctrl?: boolean;
  alt?: boolean;
  meta?: boolean;
}

/**
 * Behavior event
 */
export interface BehaviorEvent {
  type: string;
  entityId: string;
  data: unknown;
  timestamp: number;
}

/**
 * Behavior state
 */
export interface BehaviorState {
  isActive: boolean;
  entityCount: number;
  lastUpdate: number;
}

// ============================================================================
// BEHAVIOR BUILDER
// ============================================================================

/**
 * BehaviorBuilder - Fluent API for creating behaviors declaratively
 *
 * This class provides a chainable API for building behaviors without
 * needing to understand the underlying WASM implementation.
 *
 * @example
 * ```typescript
 * const behavior = new BehaviorBuilder(logicSystem)
 *   .onHover()
 *   .highlight({ color: 0xffff00, opacity: 0.2 })
 *   .onClick()
 *   .select()
 *   .build();
 * ```
 */
export class BehaviorBuilder {
  private sensors: SensorConfig[] = [];
  private controllers: ControllerConfig[] = [];
  private actuators: ActuatorConfig[] = [];
  private metadata: BehaviorMetadata = { name: 'anonymous' };
  private conditions: Array<(ctx: ExecutionContext) => boolean> = [];

  constructor(
    private logicSystem: LogicSystemWasm,
  ) {}

  // ==============================
  // SENSORS
  // ==============================

  /**
   * Add hover sensor
   *
   * @param target - Optional target selector
   */
  onHover(target?: string): this {
    this.sensors.push({
      type: SensorType.MouseOver,
      target,
    });
    return this;
  }

  /**
   * Add click sensor
   *
   * @param target - Optional target selector
   */
  onClick(target?: string): this {
    this.sensors.push({
      type: SensorType.MouseClick,
      target,
    });
    return this;
  }

  /**
   * Add double-click sensor
   *
   * @param target - Optional target selector
   */
  onDoubleClick(target?: string): this {
    this.sensors.push({
      type: SensorType.MouseDoubleClick,
      target,
    });
    return this;
  }

  /**
   * Add pointer down sensor
   *
   * @param target - Optional target selector
   */
  onPointerDown(target?: string): this {
    this.sensors.push({
      type: SensorType.MousePress,
      target,
    });
    return this;
  }

  /**
   * Add pointer up sensor
   *
   * @param target - Optional target selector
   */
  onPointerUp(target?: string): this {
    this.sensors.push({
      type: SensorType.MouseRelease,
      target,
    });
    return this;
  }

  /**
   * Add drag sensor
   *
   * @param target - Optional target selector
   */
  onDrag(target?: string): this {
    this.sensors.push({
      type: SensorType.MouseDrag,
      target,
    });
    return this;
  }

  /**
   * Add keyboard sensor
   *
   * @param key - Key code or character
   * @param modifiers - Modifier keys
   */
  onKey(key: string, modifiers?: ModifierKeys): this {
    this.sensors.push({
      type: SensorType.KeyShortcut,
      options: { key, modifiers },
    });
    return this;
  }

  // ==============================
  // CONTROLLERS
  // ==============================

  /**
   * Direct controller - passes through
   */
  direct(): this {
    this.controllers.push({ type: ControllerType.Direct });
    return this;
  }

  /**
   * AND controller - all sensors must be active
   */
  and(): this {
    this.controllers.push({ type: ControllerType.And });
    return this;
  }

  /**
   * OR controller - at least one sensor active
   */
  or(): this {
    this.controllers.push({ type: ControllerType.Or });
    return this;
  }

  /**
   * NOT controller - inverts sensor
   */
  not(): this {
    this.controllers.push({ type: ControllerType.Not });
    return this;
  }

  /**
   * Debounce controller - delays activation
   *
   * @param ms - Delay in milliseconds
   */
  debounce(ms: number): this {
    this.controllers.push({
      type: ControllerType.Debounce,
      options: { delay: ms },
    });
    return this;
  }

  /**
   * Throttle controller - limits frequency
   *
   * @param ms - Minimum interval between activations
   */
  throttle(ms: number): this {
    this.controllers.push({
      type: ControllerType.Debounce,
      options: { delay: ms, mode: 'throttle' },
    });
    return this;
  }

  /**
   * Hysteresis controller - thresholds with enter/exit
   *
   * @param enter - Threshold to activate
   * @param exit - Threshold to deactivate
   */
  hysteresis(enter: number, exit: number): this {
    this.controllers.push({
      type: ControllerType.Hysteresis,
      options: { enter, exit },
    });
    return this;
  }

  // ==============================
  // ACTUATORS
  // ==============================

  /**
   * Select actuator
   */
  select(): this {
    this.actuators.push({
      type: ActuatorType.Select,
      action: 'select',
    });
    return this;
  }

  /**
   * Deselect actuator
   */
  deselect(): this {
    this.actuators.push({
      type: ActuatorType.Select,
      action: 'deselect',
    });
    return this;
  }

  /**
   * Toggle selection actuator
   */
  toggle(): this {
    this.actuators.push({
      type: ActuatorType.Select,
      action: 'toggle',
    });
    return this;
  }

  /**
   * Translate/move actuator
   *
   * @param callback - Optional custom translation function
   */
  translate(callback?: (ctx: ExecutionContext) => Point): this {
    this.actuators.push({
      type: ActuatorType.Move,
      action: callback || 'translate',
    });
    return this;
  }

  /**
   * Resize actuator
   *
   * @param callback - Optional custom resize function
   */
  resize(callback?: (ctx: ExecutionContext) => Size): this {
    this.actuators.push({
      type: ActuatorType.Move,
      action: callback || 'resize',
    });
    return this;
  }

  /**
   * Rotate actuator
   *
   * @param callback - Optional custom rotate function
   */
  rotate(callback?: (ctx: ExecutionContext) => number): this {
    this.actuators.push({
      type: ActuatorType.Move,
      action: callback || 'rotate',
    });
    return this;
  }

  /**
   * Delete actuator
   */
  delete(): this {
    this.actuators.push({
      type: ActuatorType.Delete,
      action: 'delete',
    });
    return this;
  }

  /**
   * Highlight actuator
   *
   * @param style - Optional highlight style
   */
  highlight(style?: HighlightStyle): this {
    this.actuators.push({
      type: ActuatorType.Highlight,
      action: 'highlight',
      options: style,
    });
    return this;
  }

  /**
   * Emit custom event actuator
   *
   * @param event - Event name
   * @param dataBuilder - Optional data builder function
   */
  emit(event: string, dataBuilder?: (ctx: ExecutionContext) => unknown): this {
    this.actuators.push({
      type: ActuatorType.EmitEvent,
      action: event,
      options: dataBuilder ? { builder: dataBuilder } : {},
    });
    return this;
  }

  /**
   * Custom action actuator
   *
   * @param action - Action name or function
   */
  do(action: string | ((ctx: ExecutionContext) => void)): this {
    this.actuators.push({
      type: ActuatorType.Custom,
      action,
    });
    return this;
  }

  // ==============================
  // CONDITIONS
  // ==============================

  /**
   * Only when selected
   */
  whenSelected(): this {
    this.conditions.push((ctx) => ctx.state === 'active');
    return this;
  }

  /**
   * Only when not selected
   */
  whenNotSelected(): this {
    this.conditions.push((ctx) => ctx.state !== 'active');
    return this;
  }

  /**
   * With modifier key
   */
  withModifier(modifier: ModifierKeys): this {
    this.conditions.push((ctx) => {
      if (!ctx.event) return false;
      return (
        (modifier.shift && ctx.event.shiftKey) ||
        (modifier.ctrl && (ctx.event.ctrlKey || ctx.event.metaKey)) ||
        (modifier.alt && ctx.event.altKey)
      );
    });
    return this;
  }

  // ==============================
  // METADATA
  // ==============================

  /**
   * Set behavior name
   */
  named(name: string): this {
    this.metadata.name = name;
    return this;
  }

  /**
   * Set behavior description
   */
  describedAs(description: string): this {
    this.metadata.description = description;
    return this;
  }

  /**
   * Set behavior tags
   */
  tagged(...tags: string[]): this {
    this.metadata.tags = tags;
    return this;
  }

  // ==============================
  // BUILD
  // ==============================

  /**
   * Build the behavior and return a BehaviorBridge
   */
  build(): BehaviorBridge {
    const config: BehaviorConfig = {
      sensors: [...this.sensors],
      controllers: [...this.controllers],
      actuators: [...this.actuators],
      metadata: { ...this.metadata },
    };

    return new BehaviorBridge(this.logicSystem, config);
  }
}

// ============================================================================
// BEHAVIOR BRIDGE
// ============================================================================

/**
 * BehaviorBridge - Bridge between JS behavior and WASM
 *
 * This class wraps the WASM behavior system and provides
 * a clean API for attaching behaviors to entities.
 */
export class BehaviorBridge {
  private wasmBehavior: BehaviorBridgeWasm | null = null;
  private isActive = false;
  private attachedEntityId: string | null = null;
  private eventListeners: Map<string, Set<(event: BehaviorEvent) => void>> = new Map();

  constructor(
    private logicSystem: LogicSystemWasm,
    private config: BehaviorConfig,
  ) {
    this.initialize();
  }

  /**
   * Initialize the WASM behavior
   */
  private initialize(): void {
    try {
      // Create behavior from config
      this.wasmBehavior = this.logicSystem.create_behavior_from_config(
        this.configToJsObject(),
      );
    } catch (error) {
      console.error('Failed to initialize behavior:', error);
      // Fallback: create empty behavior
      this.wasmBehavior = new BehaviorBridgeWasm(0);
    }
  }

  /**
   * Convert config to JS object for WASM
   */
  private configToJsObject(): Record<string, unknown> {
    return {
      sensors: this.config.sensors.map((s) => ({
        type: typeof s.type === 'string' ? s.type : SensorType[s.type],
        target: s.target,
        options: s.options,
      })),
      controllers: this.config.controllers.map((c) => ({
        type: typeof c.type === 'string' ? c.type : ControllerType[c.type],
        options: c.options,
      })),
      actuators: this.config.actuators.map((a) => ({
        type: typeof a.type === 'string' ? a.type : ActuatorType[a.type],
        action: typeof a.action === 'string' ? a.action : 'custom',
        options: a.options,
      })),
      metadata: this.config.metadata,
    };
  }

  /**
   * Attach behavior to an entity
   *
   * @param entityId - Entity ID to attach to
   */
  attach(entityId: string): void {
    if (!this.wasmBehavior) {
      throw new Error('Behavior not initialized');
    }

    const entityIdNum = parseInt(entityId, 10);
    this.logicSystem.attach_behavior(entityIdNum, this.wasmBehavior);
    this.attachedEntityId = entityId;
    this.isActive = true;
  }

  /**
   * Detach behavior from an entity
   *
   * @param entityId - Entity ID to detach from
   */
  detach(entityId?: string): void {
    if (!this.wasmBehavior) return;

    const entityIdNum = entityId ? parseInt(entityId, 10) :
      (this.attachedEntityId ? parseInt(this.attachedEntityId, 10) : 0);

    this.logicSystem.detach_behavior(entityIdNum, this.wasmBehavior);
    this.attachedEntityId = null;
    this.isActive = false;
  }

  /**
   * Update behavior (called each frame)
   *
   * @param timestamp - Current timestamp in milliseconds
   */
  update(timestamp: number): void {
    if (!this.wasmBehavior || !this.isActive) return;
    this.wasmBehavior.update(timestamp);
  }

  /**
   * Check if behavior has pending events
   */
  hasEvents(): boolean {
    return this.wasmBehavior?.has_events() || false;
  }

  /**
   * Get and drain pending events
   */
  drainEvents(): BehaviorEvent[] {
    if (!this.wasmBehavior) return [];

    const events = this.wasmBehavior.drain_events();
    return events.map((e) => ({
      type: `event_${e.event_type}`,
      entityId: String(e.entity_id),
      data: e.data ? JSON.parse(e.data) : null,
      timestamp: Number(e.timestamp),
    }));
  }

  /**
   * Get behavior state
   */
  getState(): BehaviorState {
    return {
      isActive: this.isActive,
      entityCount: this.attachedEntityId ? 1 : 0,
      lastUpdate: Date.now(),
    };
  }

  /**
   * Add event listener
   */
  on(event: string, callback: (event: BehaviorEvent) => void): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event)!.add(callback);

    // Return unsubscribe function
    return () => {
      this.eventListeners.get(event)?.delete(callback);
    };
  }

  /**
   * Add once event listener
   */
  once(event: string, callback: (event: BehaviorEvent) => void): void {
    const unsubscribe = this.on(event, (e) => {
      callback(e);
      unsubscribe();
    });
  }

  /**
   * Emit event to listeners
   */
  private emit(event: string, data: BehaviorEvent): void {
    this.eventListeners.get(event)?.forEach((callback) => {
      try {
        callback(data);
      } catch (error) {
        console.error('Error in behavior event listener:', error);
      }
    });
  }

  /**
   * Destroy the behavior
   */
  destroy(): void {
    if (this.wasmBehavior) {
      this.wasmBehavior.destroy();
      this.wasmBehavior = null;
    }
    this.eventListeners.clear();
    this.isActive = false;
    this.attachedEntityId = null;
  }

  /**
   * Get the WASM bridge (for advanced use)
   */
  getWasmBridge(): BehaviorBridgeWasm | null {
    return this.wasmBehavior;
  }

  /**
   * Check if behavior is active
   */
  get isAttached(): boolean {
    return this.isActive;
  }

  /**
   * Get attached entity ID
   */
  get attachedEntity(): string | null {
    return this.attachedEntityId;
  }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

/**
 * Create a new BehaviorBuilder
 *
 * @param logicSystem - The WASM logic system
 * @returns BehaviorBuilder instance
 */
export function createBehaviorBuilder(logicSystem: LogicSystemWasm): BehaviorBuilder {
  return new BehaviorBuilder(logicSystem);
}

export default BehaviorBuilder;
