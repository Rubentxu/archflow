/**
 * ArchFlow Behavior Bridge - Logic Bricks Integration
 *
 * Provides organized access to Logic Bricks behavior system operations.
 * Wraps the WASM bridge methods for managing sensor→controller→actuator patterns.
 *
 * Architecture Reference: ARCHITECTURE-CLEAN-BRIDGE.md
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * CONCEPTS
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * The Logic Bricks system follows the Entity-Component pattern:
 * - Sensors: Detect events (mouse click, hover, keyboard)
 * - Controllers: Process signals and determine actions
 * - Actuators: Perform actions (highlight, select, move, delete)
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 */

import type { WasmBridge } from "../wasm/archflow_web.d";
import {
  SensorType,
  ControllerType,
  ActuatorType,
  Controller,
  LogicMappingTableWasm,
} from "../wasm/archflow_web.d";

/**
 * Behavior system configuration
 */
export interface BehaviorConfig {
  /** Entity ID to attach behavior to */
  entityId: number;
  /** Sensor type for detecting events */
  sensor: SensorType;
  /** Controller for signal processing */
  controller: Controller;
  /** Actuator for performing actions */
  actuator: ActuatorType;
  /** Optional configuration parameters */
  config?: Record<string, unknown>;
}

/**
 * Behavior attachment result
 */
export interface BehaviorResult {
  /** Whether attachment was successful */
  success: boolean;
  /** Attached entity ID */
  entityId: number;
  /** Behavior type attached */
  behavior: string;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SENSOR DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Available sensor types for behavior detection
 */
export const SENSOR_DEFINITIONS: Record<
  SensorType,
  { name: string; description: string }
> = {
  [SensorType.MouseOver]: {
    name: "Mouse Over",
    description: "Triggered when mouse enters entity area",
  },
  [SensorType.MouseClick]: {
    name: "Mouse Click",
    description: "Triggered on mouse button press",
  },
  [SensorType.MouseDoubleClick]: {
    name: "Double Click",
    description: "Triggered on rapid double click",
  },
  [SensorType.MousePress]: {
    name: "Mouse Press",
    description: "Triggered while mouse button is held",
  },
  [SensorType.MouseRelease]: {
    name: "Mouse Release",
    description: "Triggered when mouse button is released",
  },
  [SensorType.MouseDrag]: {
    name: "Mouse Drag",
    description: "Triggered during drag operations",
  },
  [SensorType.KeyShortcut]: {
    name: "Keyboard Shortcut",
    description: "Triggered on key combination",
  },
  [SensorType.KeyPress]: {
    name: "Key Press",
    description: "Triggered on key down",
  },
  [SensorType.KeyRelease]: {
    name: "Key Release",
    description: "Triggered on key up",
  },
  [SensorType.Tick]: {
    name: "Tick",
    description: "Triggered on each frame update",
  },
  [SensorType.Custom]: {
    name: "Custom",
    description: "Custom event trigger",
  },
};

// ═══════════════════════════════════════════════════════════════════════════════
// ACTUATOR DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Available actuator types for behavior actions
 */
export const ACTUATOR_DEFINITIONS: Record<
  ActuatorType,
  { name: string; description: string }
> = {
  [ActuatorType.Highlight]: {
    name: "Highlight",
    description: "Visual highlight effect on entity",
  },
  [ActuatorType.Select]: {
    name: "Select",
    description: "Select or deselect entity",
  },
  [ActuatorType.Move]: {
    name: "Move",
    description: "Move or transform entity",
  },
  [ActuatorType.Delete]: {
    name: "Delete",
    description: "Remove entity from canvas",
  },
  [ActuatorType.EmitEvent]: {
    name: "Emit Event",
    description: "Trigger custom event",
  },
  [ActuatorType.Custom]: {
    name: "Custom",
    description: "Custom action",
  },
};

// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR BRIDGE
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * BehaviorBridge - Facade for Logic Bricks behavior operations
 *
 * Provides a clean API for:
 * - Attaching behaviors to entities
 * - Configuring sensor→controller→actuator patterns
 * - Managing behavior lifecycle
 *
 * @example
 * ```typescript
 * const behaviors = new BehaviorBridge(bridge);
 *
 * // Make entity interactive on hover
 * behaviors.attach({
 *   entityId: 1,
 *   sensor: SensorType.MouseOver,
 *   controller: Controller.direct(),
 *   actuator: ActuatorType.Highlight,
 *   config: { color: 0xffff00, opacity: 0.3 }
 * });
 *
 * // Make entity selectable on click
 * behaviors.attach({
 *   entityId: 1,
 *   sensor: SensorType.MouseClick,
 *   controller: Controller.direct(),
 *   actuator: ActuatorType.Select,
 *   config: { mode: "single" }
 * });
 * ```
 */
export class BehaviorBridge {
  /**
   * Reference to the underlying WASM bridge
   */
  private bridge: WasmBridge;

  /**
   * Mapping table for Logic Bricks configuration
   */
  private mappingTable: LogicMappingTableWasm;

  /**
   * Create a new BehaviorBridge
   *
   * @param bridge - The WASM bridge instance
   */
  constructor(bridge: WasmBridge) {
    this.bridge = bridge;
    this.mappingTable = new LogicMappingTableWasm();
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // BEHAVIOR ATTACHMENT
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Attach a behavior to an entity
   *
   * Creates a sensor→controller→actuator connection for the entity.
   *
   * @param config - Behavior configuration
   * @returns Attachment result
   *
   * @example
   * ```typescript
   * behaviors.attach({
   *   entityId: 5,
   *   sensor: SensorType.MouseOver,
   *   controller: Controller.direct(),
   *   actuator: ActuatorType.Highlight,
   *   config: { color: 0xff0000, opacity: 0.5 }
   * });
   * ```
   */
  attach(config: BehaviorConfig): BehaviorResult {
    const {
      entityId,
      sensor,
      controller,
      actuator,
      config: behaviorConfig,
    } = config;

    try {
      // Register the mapping in the Logic Bricks system
      // This creates the actual sensor→controller→actuator chain
      this.mappingTable.add_mapping(entityId, sensor, controller, actuator);

      // Apply configuration if provided
      if (behaviorConfig) {
        this.applyConfig(entityId, actuator, behaviorConfig);
      }

      return {
        success: true,
        entityId,
        behavior: `${SensorType[sensor]} → ${ActuatorType[actuator]}`,
      };
    } catch (error) {
      console.error(`Failed to attach behavior to entity ${entityId}:`, error);
      return {
        success: false,
        entityId,
        behavior: `${SensorType[sensor]} → ${ActuatorType[actuator]}`,
      };
    }
  }

  /**
   * Attach hover highlight behavior
   *
   * Convenience method for common hover effect.
   *
   * @param entityId - Entity to make interactive
   * @param color - Highlight color (hex)
   * @param opacity - Highlight opacity (0-1)
   *
   * @example
   * ```typescript
   * // Yellow highlight on hover
   * behaviors.attachHoverHighlight(1, 0xffff00, 0.3);
   * ```
   */
  attachHoverHighlight(
    entityId: number,
    color: number,
    opacity: number,
  ): BehaviorResult {
    return this.attach({
      entityId,
      sensor: SensorType.MouseOver,
      controller: Controller.direct(),
      actuator: ActuatorType.Highlight,
      config: { color, opacity },
    });
  }

  /**
   * Attach selectable behavior
   *
   * Convenience method for click-to-select.
   *
   * @param entityId - Entity to make selectable
   * @param mode - Selection mode ("single" | "multi")
   *
   * @example
   * ```typescript
   * // Single selection
   * behaviors.attachSelectable(1, "single");
   *
   * // Multi-selection with Shift
   * behaviors.attachSelectable(2, "multi");
   * ```
   */
  attachSelectable(
    entityId: number,
    mode: "single" | "multi" = "single",
  ): BehaviorResult {
    return this.attach({
      entityId,
      sensor: SensorType.MouseClick,
      controller: Controller.direct(),
      actuator: ActuatorType.Select,
      config: { mode },
    });
  }

  /**
   * Attach draggable behavior
   *
   * Convenience method for drag operations.
   *
   * @param entityId - Entity to make draggable
   * @param axis - Movement axis ("both" | "x" | "y")
   * @param snap - Snap to grid size
   *
   * @example
   * ```typescript
   * behaviors.attachDraggable(1, "both", 8);
   * ```
   */
  attachDraggable(
    entityId: number,
    axis: "both" | "x" | "y" = "both",
    snap?: number,
  ): BehaviorResult {
    return this.attach({
      entityId,
      sensor: SensorType.MouseDrag,
      controller: Controller.direct(),
      actuator: ActuatorType.Move,
      config: { axis, snap },
    });
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // BEHAVIOR REMOVAL
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Remove all behaviors from an entity
   *
   * @param entityId - Entity to clear behaviors from
   *
   * @example
   * ```typescript
   * behaviors.clearEntityBehaviors(1);
   * ```
   */
  clearEntityBehaviors(entityId: number): void {
    this.mappingTable.clear_entity(entityId);
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // QUERY OPERATIONS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Get available sensor types
   *
   * @returns Array of sensor type definitions
   *
   * @example
   * ```typescript
   * const sensors = behaviors.getAvailableSensors();
   * sensors.forEach(s => console.log(s.name));
   * ```
   */
  getAvailableSensors(): Array<{
    type: SensorType;
    name: string;
    description: string;
  }> {
    return Object.entries(SENSOR_DEFINITIONS).map(([key, value]) => ({
      type: parseInt(key) as SensorType,
      name: value.name,
      description: value.description,
    }));
  }

  /**
   * Get available actuator types
   *
   * @returns Array of actuator type definitions
   *
   * @example
   * ```typescript
   * const actuators = behaviors.getAvailableActuators();
   * actuators.forEach(a => console.log(a.name));
   * ```
   */
  getAvailableActuators(): Array<{
    type: ActuatorType;
    name: string;
    description: string;
  }> {
    return Object.entries(ACTUATOR_DEFINITIONS).map(([key, value]) => ({
      type: parseInt(key) as ActuatorType,
      name: value.name,
      description: value.description,
    }));
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // PRIVATE HELPERS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Apply configuration to an actuator
   *
   * @param entityId - Entity ID
   * @param actuator - Actuator type
   * @param config - Configuration parameters
   */
  private applyConfig(
    entityId: number,
    actuator: ActuatorType,
    config: Record<string, unknown>,
  ): void {
    // Configuration is applied through the mapping table
    // Each actuator type has its own config structure
    switch (actuator) {
      case ActuatorType.Highlight:
        // HighlightConfig: { color: number, opacity: number }
        if (
          typeof config.color === "number" &&
          typeof config.opacity === "number"
        ) {
          // Apply highlight color and opacity
          this.applyHighlightConfig(entityId, config.color, config.opacity);
        }
        break;

      case ActuatorType.Select:
        // SelectConfig: { mode: "single" | "multi" }
        if (config.mode === "multi") {
          // Enable multi-select mode
        }
        break;

      case ActuatorType.Move:
        // MoveConfig: { axis: string, snap: number }
        // Configuration handled by controller
        break;

      case ActuatorType.Delete:
        // No additional config
        break;

      case ActuatorType.EmitEvent:
        // EventConfig: { eventName: string, data: unknown }
        break;

      default:
        // Custom actuators
        break;
    }
  }

  /**
   * Apply highlight configuration
   *
   * @param entityId - Entity ID
   * @param color - Highlight color (hex)
   * @param opacity - Opacity (0-1)
   */
  private applyHighlightConfig(
    entityId: number,
    color: number,
    opacity: number,
  ): void {
    // Convert opacity from 0-1 to 0-255 for WASM
    const alpha = Math.round(opacity * 255);
    const r = (color >> 16) & 0xff;
    const g = (color >> 8) & 0xff;
    const b = color & 0xff;

    // Set the active color for highlighting
    this.bridge.set_active_color(r, g, b, alpha);
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE: COMMON BEHAVIOR TEMPLATES
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Predefined behavior templates for common use cases
 */
export const BEHAVIOR_TEMPLATES = {
  /**
   * Interactive: Hover highlight effect
   */
  interactive: (
    entityId: number,
    color = 0xffff00,
    opacity = 0.2,
  ): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseOver,
    controller: Controller.direct(),
    actuator: ActuatorType.Highlight,
    config: { color, opacity },
  }),

  /**
   * Selectable: Single click to select
   */
  selectable: (entityId: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseClick,
    controller: Controller.direct(),
    actuator: ActuatorType.Select,
    config: { mode: "single" },
  }),

  /**
   * MultiSelectable: Click with Shift for multi-select
   */
  multiSelectable: (entityId: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseClick,
    controller: Controller.direct(),
    actuator: ActuatorType.Select,
    config: { mode: "multi" },
  }),

  /**
   * Draggable: Drag to move
   */
  draggable: (entityId: number, snap?: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseDrag,
    controller: Controller.direct(),
    actuator: ActuatorType.Move,
    config: { axis: "both", snap },
  }),

  /**
   * Resizable: Drag handles to resize
   */
  resizable: (entityId: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseDrag,
    controller: Controller.direct(),
    actuator: ActuatorType.Move,
    config: { axis: "both", handles: ["nw", "ne", "sw", "se"] },
  }),

  /**
   * Clickable: Click to trigger action
   */
  clickable: (entityId: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseClick,
    controller: Controller.direct(),
    actuator: ActuatorType.EmitEvent,
    config: { eventName: "click" },
  }),

  /**
   * DoubleClickable: Double click to trigger action
   */
  doubleClickable: (entityId: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.MouseDoubleClick,
    controller: Controller.direct(),
    actuator: ActuatorType.EmitEvent,
    config: { eventName: "doubleClick" },
  }),

  /**
   * Deleteable: Press Delete key to remove
   */
  deleteable: (entityId: number): BehaviorConfig => ({
    entityId,
    sensor: SensorType.KeyShortcut,
    controller: Controller.direct(),
    actuator: ActuatorType.Delete,
    config: { key: "Delete" },
  }),
};

// ═══════════════════════════════════════════════════════════════════════════════
// DEFAULT EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Create a new BehaviorBridge
 *
 * @param bridge - The WASM bridge instance
 * @returns Configured BehaviorBridge instance
 *
 * @example
 * ```typescript
 * import { createBehaviorBridge } from './BehaviorBridge';
 *
 * const behaviors = createBehaviorBridge(bridge);
 * behaviors.attachHoverHighlight(1, 0xffff00, 0.3);
 * ```
 */
export function createBehaviorBridge(bridge: any): BehaviorBridge {
  return new BehaviorBridge(bridge);
}

export default BehaviorBridge;
