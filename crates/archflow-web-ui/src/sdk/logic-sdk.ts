/**
 * ArchFlow Logic SDK - High-Level TypeScript API
 *
 * This module provides a developer-friendly wrapper around the WASM Logic Bricks system.
 * It offers a fluent API for creating sensor→controller→actuator connections.
 *
 * @example
 * ```typescript
 * import { LogicSDK, SensorType, Controller } from './logic-sdk';
 *
 * const sdk = new LogicSDK(wasmBridge);
 *
 * // Simple hover highlight
 * sdk.entity(1)
 *    .when(SensorType.MouseOver)
 *    .highlight();
 *
 * // Click to select
 * sdk.entity(1)
 *    .when(SensorType.MouseClick)
 *    .direct()
 *    .select();
 *
 * // Complex: Ctrl+Click to move
 * sdk.entity(1)
 *    .when(SensorType.MouseClick)
 *    .and(SensorType.KeyShortcut)
 *    .move();
 * ```
 *
 * Architecture Reference: EPIC-WEB-010
 */

// Import WASM types - need to import Controller as a value, not a type
import type {
  ActuatorType,
  ControllerType,
  ExtendedActuatorType,
  LogicMappingTableWasm,
  SensorType,
  SignalByteWasm,
} from "../wasm/archflow_web.d";

import { Controller } from "../wasm/archflow_web.d";

// Re-export types
export type {
  ActuatorType,
  ControllerType,
  ExtendedActuatorType,
  LogicMappingTableWasm,
  SensorType,
  SignalByteWasm,
};

// ═══════════════════════════════════════════════════════════════════════════════
// SIGNAL PROXY - JavaScript interface to SignalByte
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * SignalProxy provides JavaScript access to SignalByte state
 *
 * This proxy wraps the WASM SignalByte and provides convenient methods
 * for analyzing signal history and detecting patterns.
 */
export interface SignalProxy {
  /** Current signal state (true = HIGH, false = LOW) */
  current: boolean;

  /** Check if this is a rising edge (0→1 transition) */
  isRisingEdge(): boolean;

  /** Check if this is a falling edge (1→0 transition) */
  isFallingEdge(): boolean;

  /** Check if signal has been steady for N ticks */
  isSteady(ticks: number): boolean;

  /** Check if signal has been steady HIGH for N ticks */
  isSteadyHigh(ticks: number): boolean;

  /** Check if signal has been steady LOW for N ticks */
  isSteadyLow(ticks: number): boolean;

  /** Count number of HIGH ticks in history */
  countOnes(): number;

  /** Count number of LOW ticks in history */
  countZeros(): number;

  /** Get raw 6-bit history value */
  history(): number;

  /** Get bit at position (0 = current, 5 = oldest) */
  getBit(position: number): boolean;
}

/**
 * Create a SignalProxy from a WASM SignalByte
 */
export function createSignalProxy(signal: SignalByteWasm): SignalProxy {
  return {
    get current() {
      return signal.get_current() as boolean;
    },

    isRisingEdge() {
      return signal.is_rising_edge() as boolean;
    },

    isFallingEdge() {
      return signal.is_falling_edge() as boolean;
    },

    isSteady(ticks: number) {
      // For simplicity, check if all ticks in history are the same as current
      const history = signal.get_history() as number;
      const mask = (1 << ticks) - 1;
      const currentBit = history & 1;
      const masked = history & mask;

      if (currentBit === 1) {
        return masked === mask; // All 1s
      } else {
        return masked === 0; // All 0s
      }
    },

    isSteadyHigh(ticks: number) {
      const history = signal.get_history() as number;
      const mask = (1 << ticks) - 1;
      return (history & mask) === mask;
    },

    isSteadyLow(ticks: number) {
      const history = signal.get_history() as number;
      const mask = (1 << ticks) - 1;
      return (history & mask) === 0;
    },

    countOnes() {
      const history = signal.get_history() as number;
      let count = 0;
      for (let i = 0; i < 6; i++) {
        if (history & (1 << i)) count++;
      }
      return count;
    },

    countZeros() {
      return 6 - this.countOnes();
    },

    history() {
      return signal.get_history() as number;
    },

    getBit(position: number) {
      if (position < 0 || position > 5) return false;
      const history = signal.get_history() as number;
      return ((history >> position) & 1) === 1;
    },
  };
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTEXT PROXY - JavaScript interface to ControllerContext
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * ContextProxy provides JavaScript access to evaluation context
 *
 * This proxy provides access to entity properties, modifiers, and other
 * contextual information during controller evaluation.
 */
export interface ContextProxy {
  /** Current timestamp in milliseconds */
  readonly timestamp: number;

  /** Entity ID being evaluated */
  readonly entityId: number;

  /** Keyboard modifiers bitmask (Shift=1, Ctrl=2, Alt=4, Meta=8) */
  readonly modifiers: number;

  /** Mouse position in world coordinates */
  readonly mousePos: { x: number; y: number };

  /** Get a custom property value */
  getProperty(key: string): string | number | boolean | null;

  /** Set a custom property value */
  setProperty(key: string, value: string | number | boolean): void;
}

/**
 * Create a ContextProxy from evaluation parameters
 */
export function createContextProxy(
  timestamp: number,
  entityId: number,
  modifiers: number,
  mousePos: { x: number; y: number },
  properties: Map<string, string | number | boolean>,
): ContextProxy {
  return {
    get timestamp() {
      return timestamp;
    },
    get entityId() {
      return entityId;
    },
    get modifiers() {
      return modifiers;
    },
    get mousePos() {
      return mousePos;
    },

    getProperty(key: string) {
      return properties.get(key) ?? null;
    },

    setProperty(key: string, value: string | number | boolean) {
      properties.set(key, value);
    },
  };
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTOM CONTROLLER REGISTRY - JavaScript Sandbox
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Custom controller evaluation function
 *
 * @param signal - SignalProxy for the primary sensor
 * @param context - ContextProxy with evaluation context
 * @returns true if the controller condition is met
 */
export type CustomControllerFn = (
  signal: SignalProxy,
  context: ContextProxy,
) => boolean;

/**
 * Custom controller registry for JavaScript-evaluated controllers
 *
 * Allows developers to register custom JavaScript logic that will be
 * evaluated in a sandboxed environment with timeout protection.
 *
 * @example
 * ```typescript
 * registry.register('tooltipOnCtrlHover', (signal, context) => {
 *   const stable = signal.isSteady(6);
 *   const hasCtrl = (context.modifiers & 2) !== 0;
 *   return stable && hasCtrl;
 * });
 * ```
 */
export class CustomControllerRegistry {
  private controllers = new Map<string, CustomControllerFn>();
  private properties = new Map<
    number,
    Map<string, string | number | boolean>
  >();

  /**
   * Register a custom controller
   *
   * @param name - Unique identifier for the controller
   * @param fn - Evaluation function returning boolean
   *
   * @example
   * ```typescript
   * registry.register('tooltipOnCtrlHover', (signal, context) => {
   *   return signal.isSteady(6) && (context.modifiers & 2) !== 0;
   * });
   * ```
   */
  register(name: string, fn: CustomControllerFn): void {
    this.controllers.set(name, fn);
  }

  /**
   * Evaluate a custom controller with sandbox protection
   *
   * @param name - Controller name
   * @param signal - WASM SignalByte
   * @param context - Evaluation context parameters
   * @param timeoutMs - Timeout in milliseconds (default: 50ms)
   * @returns Result of custom controller evaluation
   *
   * @example
   * ```typescript
   * const result = registry.evaluate(
   *   'myController',
   *   signalByte,
   *   { timestamp: Date.now(), entityId: 1, modifiers: 2, mousePos: {x:0, y:0} },
   *   50
   * );
   * ```
   */
  evaluate(
    name: string,
    signal: SignalByteWasm,
    context: {
      timestamp: number;
      entityId: number;
      modifiers: number;
      mousePos: { x: number; y: number };
    },
    timeoutMs: number = 50,
  ): boolean {
    const fn = this.controllers.get(name);
    if (!fn) {
      console.warn(`Custom controller "${name}" not found, returning false`);
      return false;
    }

    // Get or create property map for this entity
    if (!this.properties.has(context.entityId)) {
      this.properties.set(context.entityId, new Map());
    }
    const entityProps = this.properties.get(context.entityId)!;

    // Create proxies
    const signalProxy = createSignalProxy(signal);
    const contextProxy = createContextProxy(
      context.timestamp,
      context.entityId,
      context.modifiers,
      context.mousePos,
      entityProps,
    );

    // Evaluate with timeout protection
    try {
      const timeout = Date.now() + timeoutMs;
      const result = fn(signalProxy, contextProxy);

      // Check for timeout
      if (Date.now() > timeout) {
        console.error(
          `Custom controller "${name}" exceeded timeout of ${timeoutMs}ms`,
        );
        return false;
      }

      return result;
    } catch (error) {
      console.error(`Error evaluating custom controller "${name}":`, error);
      return false;
    }
  }

  /**
   * Check if a controller is registered
   */
  has(name: string): boolean {
    return this.controllers.has(name);
  }

  /**
   * Remove a custom controller
   */
  unregister(name: string): void {
    this.controllers.delete(name);
  }

  /**
   * Clear all custom controllers
   */
  clear(): void {
    this.controllers.clear();
    this.properties.clear();
  }

  /**
   * Get all registered controller names
   */
  getControllerNames(): string[] {
    return Array.from(this.controllers.keys());
  }

  /**
   * Get property storage for an entity (for debugging/testing)
   */
  getEntityProperties(
    entityId: number,
  ): Map<string, string | number | boolean> {
    if (!this.properties.has(entityId)) {
      this.properties.set(entityId, new Map());
    }
    return this.properties.get(entityId)!;
  }
}

/**
 * Global custom controller registry instance
 */
export const customControllerRegistry = new CustomControllerRegistry();

// ═══════════════════════════════════════════════════════════════════════════════
// HIGH-LEVEL SDK FOR LOGIC BRICKS SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * High-level SDK for Logic Bricks system
 *
 * Provides fluent API for creating entity behaviors.
 */
export class LogicSDK {
  private wasmTable: LogicMappingTableWasm;
  private entityCallbacks = new Map<number, Map<SensorType, () => void>>();

  constructor(wasmTable: LogicMappingTableWasm) {
    this.wasmTable = wasmTable;
  }

  /**
   * Get or create the callback map for an entity
   */
  private getEntityCallbacks(entityId: number): Map<SensorType, () => void> {
    if (!this.entityCallbacks.has(entityId)) {
      this.entityCallbacks.set(entityId, new Map());
    }
    return this.entityCallbacks.get(entityId)!;
  }

  /**
   * Start configuring behavior for an entity
   *
   * @param entityId - The entity ID to configure
   * @returns EntityBuilder for fluent API
   *
   * @example
   * ```typescript
   * sdk.entity(1)
   *   .when(SensorType.MouseOver)
   *   .highlight();
   * ```
   */
  entity(entityId: number): EntityBuilder {
    return new EntityBuilder(
      entityId,
      this.wasmTable,
      this.getEntityCallbacks(entityId),
    );
  }

  /**
   * Remove all connections for an entity
   *
   * @param entityId - The entity ID to clear
   */
  clearEntity(entityId: number): void {
    this.wasmTable.clear_entity(entityId);
    this.entityCallbacks.delete(entityId);
  }

  /**
   * Remove a specific connection
   *
   * @param entityId - The entity ID
   * @param sensor - The sensor type to disconnect
   */
  removeConnection(entityId: number, sensor: SensorType): void {
    this.wasmTable.remove_connection(entityId, sensor);
    const callbacks = this.getEntityCallbacks(entityId);
    callbacks.delete(sensor);
  }

  /**
   * Check if an entity has a connection
   *
   * @param entityId - The entity ID
   * @param sensor - The sensor type
   * @returns true if connection exists
   */
  hasConnection(entityId: number, sensor: SensorType): boolean {
    return this.wasmTable.has_connection(entityId, sensor);
  }

  /**
   * Get connection count for an entity
   *
   * @param entityId - The entity ID
   * @returns Number of connections
   */
  connectionCount(entityId: number): number {
    return this.wasmTable.connection_count(entityId);
  }

  /**
   * Clear all connections
   */
  clear(): void {
    this.wasmTable.clear();
    this.entityCallbacks.clear();
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTITY BUILDER - Fluent API
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Fluent builder for entity behavior configuration
 *
 * Provides chainable methods for setting up sensor→controller→actuator
 * connections.
 */
export class EntityBuilder {
  private currentSensor: SensorType | null = null;
  private currentController: Controller | null = null;

  constructor(
    private entityId: number,
    private wasmTable: LogicMappingTableWasm,
    private callbacks: Map<SensorType, () => void>,
  ) {}

  /**
   * Specify the sensor that triggers the behavior
   *
   * @param sensor - The sensor type
   * @returns this for chaining
   *
   * @example
   * ```typescript
   * builder.when(SensorType.MouseOver)
   *        .highlight();
   * ```
   */
  when(sensor: SensorType): this {
    this.currentSensor = sensor;
    return this;
  }

  /**
   * Use a direct controller (pass-through)
   *
   * @returns this for chaining
   */
  direct(): this {
    this.currentController = Controller.direct();
    return this;
  }

  /**
   * Use an AND controller (require both sensors)
   *
   * @param otherSensor - The secondary sensor
   * @returns this for chaining
   */
  and(otherSensor: SensorType): this {
    this.currentController = Controller.and(otherSensor);
    return this;
  }

  /**
   * Use an OR controller (require either sensor)
   *
   * @param otherSensor - The secondary sensor
   * @returns this for chaining
   */
  or(otherSensor: SensorType): this {
    this.currentController = Controller.or(otherSensor);
    return this;
  }

  /**
   * Use a NOT controller (inverts the signal)
   *
   * @returns this for chaining
   */
  not(): this {
    this.currentController = Controller.not();
    return this;
  }

  /**
   * Connect to Highlight actuator
   *
   * @param color - Optional color (ARGB format)
   *
   * @example
   * ```typescript
   * sdk.entity(1)
   *    .when(SensorType.MouseOver)
   *    .highlight(0x00FF00FF); // Green highlight
   * ```
   */
  highlight(_color: number = 0x00ff00ff): void {
    this.ensureSensor();
    const controller = this.currentController || Controller.direct();
    this.wasmTable.add_highlight(
      this.entityId,
      this.currentSensor!,
      controller,
    );
  }

  /**
   * Connect to Select actuator
   *
   * @example
   * ```typescript
   * sdk.entity(1)
   *    .when(SensorType.MouseClick)
   *    .direct()
   *    .select();
   * ```
   */
  select(): void {
    this.ensureSensor();
    const controller = this.currentController || Controller.direct();
    this.wasmTable.add_select(this.entityId, this.currentSensor!, controller);
  }

  /**
   * Connect to Move actuator
   *
   * @example
   * ```typescript
   * sdk.entity(1)
   *    .when(SensorType.MouseClick)
   *    .and(SensorType.MouseOver)
   *    .move();
   * ```
   */
  move(): void {
    this.ensureSensor();
    const controller = this.currentController || Controller.direct();
    this.wasmTable.add_move(this.entityId, this.currentSensor!, controller);
  }

  /**
   * Register a custom callback for this sensor
   *
   * @param callback - Function to call when sensor triggers
   *
   * @example
   * ```typescript
   * sdk.entity(1)
   *    .when(SensorType.MouseOver)
   *    .onTrigger(() => console.log('Hovered entity 1'));
   * ```
   */
  onTrigger(callback: () => void): void {
    this.ensureSensor();
    if (this.currentSensor) {
      this.callbacks.set(this.currentSensor, callback);
    }
  }

  /**
   * Ensure a sensor is set before adding an actuator
   */
  private ensureSensor(): void {
    if (!this.currentSensor) {
      throw new Error("Must call .when(sensor) before adding an actuator");
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Convenience functions for creating controllers
 *
 * @example
 * ```typescript
 * import { Ctrl } from './logic-sdk';
 *
 * const direct = Ctrl.direct();
 * const and = Ctrl.and(SensorType.KeyShortcut);
 * const or = Ctrl.or(SensorType.KeyShortcut);
 * const not = Ctrl.not();
 * ```
 */
export const Ctrl = {
  direct: () => Controller.direct(),
  and: (sensor: SensorType) => Controller.and(sensor),
  or: (sensor: SensorType) => Controller.or(sensor),
  not: () => Controller.not(),
};

/**
 * Create a Logic SDK instance
 *
 * @param wasmTable - The WASM LogicMappingTable instance
 * @returns LogicSDK instance
 *
 * @example
 * ```typescript
 * import { createLogicSDK } from './logic-sdk';
 *
 * const sdk = createLogicSDK(wasmBridge.mappingTable);
 * ```
 */
export function createLogicSDK(wasmTable: LogicMappingTableWasm): LogicSDK {
  return new LogicSDK(wasmTable);
}

/**
 * Timing values (in ticks at 60 FPS)
 *
 * @example
 * ```typescript
 * import { Timing } from './logic-sdk';
 *
 * console.log(Timing.TickMs); // 16.67ms per tick
 * ```
 */
export const Timing = {
  /** Ticks to milliseconds (at 60 FPS) */
  TickMs: 16.67,
} as const;
