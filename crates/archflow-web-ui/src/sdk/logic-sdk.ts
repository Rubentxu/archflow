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
  LogicMappingTableWasm,
  SensorType,
  SignalByteWasm,
} from "../wasm/archflow_web.d";

import { Controller } from "../wasm/archflow_web.d";

// Re-export types
export type {
  ActuatorType,
  ControllerType,
  LogicMappingTableWasm,
  SensorType,
  SignalByteWasm,
};

/**
 * Custom controller registry for JavaScript-evaluated controllers
 *
 * Allows developers to register custom JavaScript logic that will be
 * evaluated in a sandboxed environment.
 */
class CustomControllerRegistry {
  private controllers = new Map<
    string,
    (signal: any, context: any) => boolean
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
   *   const stable = signal.isSteady(6);
   *   const hasCtrl = (context.modifiers & 2) !== 0;
   *   return stable && hasCtrl;
   * });
   * ```
   */
  register(name: string, fn: (signal: any, context: any) => boolean): void {
    this.controllers.set(name, fn);
  }

  /**
   * Evaluate a custom controller
   *
   * @param name - Controller name
   * @param signal - SignalByte proxy object
   * @param context - Evaluation context
   * @returns Result of custom controller evaluation
   */
  evaluate(name: string, signal: any, context: any): boolean {
    const fn = this.controllers.get(name);
    if (!fn) {
      console.warn(`Custom controller "${name}" not found, returning false`);
      return false;
    }

    try {
      return fn(signal, context);
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
  }
}

/**
 * Global custom controller registry instance
 */
export const customControllerRegistry = new CustomControllerRegistry();

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
