/**
 * Logic Bricks Type Definitions
 *
 * These types mirror the Rust LogicMappingTable structure from archflow-logic crate.
 * They provide type-safe interfaces for the visual Logic Bricks editor.
 */

// ═══════════════════════════════════════════════════════════════════════════════
// Sensor Types
// ═══════════════════════════════════════════════════════════════════════════════

export const SensorType = {
  /** Triggers on mouse hover over entity */
  MouseOver: "MouseOver",

  /** Triggers on mouse click */
  MouseClick: "MouseClick",

  /** Triggers when another entity is within proximity */
  Proximity: "Proximity",

  /** Triggers on keyboard shortcut */
  KeyShortcut: "KeyShortcut",

  /** Always active sensor */
  Always: "Always",
} as const;

export type SensorType = (typeof SensorType)[keyof typeof SensorType];

// ═══════════════════════════════════════════════════════════════════════════════
// Trigger Types (from SignalByte)
// ═══════════════════════════════════════════════════════════════════════════════

export const TriggerType = {
  /** Triggers immediately when sensor is active */
  Direct: "Direct",

  /** Triggers only when signal is stable for N frames */
  Stable: "Stable",

  /** Triggers on rising edge (false → true) */
  RisingEdge: "RisingEdge",

  /** Triggers on falling edge (true → false) */
  FallingEdge: "FallingEdge",
} as const;

export type TriggerType = (typeof TriggerType)[keyof typeof TriggerType];

export interface TriggerConfig {
  type: TriggerType;
  /** For Stable triggers: number of frames to wait */
  frames?: number;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sensor Configurations
// ═══════════════════════════════════════════════════════════════════════════════

export interface MouseOverConfig {
  trigger: TriggerConfig;
  invert?: boolean;
}

export interface MouseClickConfig {
  trigger: TriggerConfig;
  filters: {
    button: "Primary" | "Secondary" | "Middle";
    modifiers: ("Ctrl" | "Shift" | "Alt" | "Meta")[];
  };
}

export interface ProximityConfig {
  trigger: TriggerConfig;
  distance: number;
  targetEntityIds?: number[];
}

export interface KeyShortcutConfig {
  trigger: TriggerConfig;
  key: string;
  modifiers: ("Ctrl" | "Shift" | "Alt" | "Meta")[];
}

export interface AlwaysConfig {
  trigger: TriggerConfig;
}

export type SensorConfig =
  | { type: typeof SensorType.MouseOver; config: MouseOverConfig }
  | { type: typeof SensorType.MouseClick; config: MouseClickConfig }
  | { type: typeof SensorType.Proximity; config: ProximityConfig }
  | { type: typeof SensorType.KeyShortcut; config: KeyShortcutConfig }
  | { type: typeof SensorType.Always; config: AlwaysConfig };

// ═══════════════════════════════════════════════════════════════════════════════
// Actuator Types
// ═══════════════════════════════════════════════════════════════════════════════

export const ActuatorType = {
  /** Changes visual appearance of entity */
  Highlight: "Highlight",

  /** Adds/removes entity from selection */
  Select: "Select",

  /** Moves entity to new position */
  Move: "Move",

  /** Rotates entity */
  Rotate: "Rotate",

  /** Scales entity */
  Scale: "Scale",

  /** Plays animation */
  Animate: "Animate",
} as const;

export type ActuatorType = (typeof ActuatorType)[keyof typeof ActuatorType];

// ═══════════════════════════════════════════════════════════════════════════════
// Actuator Configurations
// ═══════════════════════════════════════════════════════════════════════════════

export interface HighlightParams {
  color: string;
  borderWidth: number;
  fillColor?: string;
  duration?: number; // ms, undefined = permanent
}

export interface SelectParams {
  addToSelection: boolean;
  clearPrevious?: boolean;
}

export interface MoveParams {
  x: number;
  y: number;
  relative?: boolean;
  duration?: number; // ms, undefined = instant
}

export interface RotateParams {
  angle: number; // degrees
  relative?: boolean;
  duration?: number;
}

export interface ScaleParams {
  scaleX: number;
  scaleY: number;
  relative?: boolean;
  duration?: number;
}

export interface AnimateParams {
  animationType: "pulse" | "bounce" | "shake" | "spin";
  duration: number;
  iterations?: number; // undefined = infinite
}

export type ActuatorParams =
  | { type: typeof ActuatorType.Highlight; params: HighlightParams }
  | { type: typeof ActuatorType.Select; params: SelectParams }
  | { type: typeof ActuatorType.Move; params: MoveParams }
  | { type: typeof ActuatorType.Rotate; params: RotateParams }
  | { type: typeof ActuatorType.Scale; params: ScaleParams }
  | { type: typeof ActuatorType.Animate; params: AnimateParams };

// ═══════════════════════════════════════════════════════════════════════════════
// Controller Types
// ═══════════════════════════════════════════════════════════════════════════════

export const ControllerType = {
  /** All sensors must be active */
  AND: "AND",

  /** At least one sensor must be active */
  OR: "OR",

  /** Inverts sensor output */
  NOT: "NOT",

  /** Sensor must NOT be active */
  NAND: "NAND",

  /** Exactly one sensor must be active */
  XOR: "XOR",
} as const;

export type ControllerType =
  (typeof ControllerType)[keyof typeof ControllerType];

export interface ControllerConfig {
  type: ControllerType;
  /** For binary controllers (AND, OR, XOR), combines multiple sensors */
  inputSensors?: number[]; // sensor indices
}

// ═══════════════════════════════════════════════════════════════════════════════
// Logic Rule (complete sensor-actuator mapping)
// ═══════════════════════════════════════════════════════════════════════════════

export interface LogicRule {
  id: string;
  sensor: SensorConfig;
  controller?: ControllerConfig;
  actuators: ActuatorParams[];
  enabled: boolean;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Entity Logic Configuration
// ═══════════════════════════════════════════════════════════════════════════════

export interface EntityLogic {
  entityId: number;
  rules: LogicRule[];
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Creates a default trigger configuration
 */
export function createDefaultTrigger(): TriggerConfig {
  return { type: TriggerType.Direct };
}

/**
 * Creates a default sensor configuration
 */
export function createDefaultSensor(type: SensorType): SensorConfig {
  const defaultTrigger = createDefaultTrigger();

  switch (type) {
    case SensorType.MouseOver:
      return { type, config: { trigger: defaultTrigger, invert: false } };

    case SensorType.MouseClick:
      return {
        type,
        config: {
          trigger: defaultTrigger,
          filters: { button: "Primary", modifiers: [] },
        },
      };

    case SensorType.Proximity:
      return { type, config: { trigger: defaultTrigger, distance: 100 } };

    case SensorType.KeyShortcut:
      return {
        type,
        config: { trigger: defaultTrigger, key: "", modifiers: [] },
      };

    case SensorType.Always:
      return { type, config: { trigger: defaultTrigger } };
  }
}

/**
 * Creates a default actuator configuration
 */
export function createDefaultActuator(type: ActuatorType): ActuatorParams {
  switch (type) {
    case ActuatorType.Highlight:
      return {
        type,
        params: { color: "#4A90E2", borderWidth: 2 },
      };

    case ActuatorType.Select:
      return {
        type,
        params: { addToSelection: true, clearPrevious: false },
      };

    case ActuatorType.Move:
      return {
        type,
        params: { x: 0, y: 0, relative: true },
      };

    case ActuatorType.Rotate:
      return {
        type,
        params: { angle: 45, relative: true },
      };

    case ActuatorType.Scale:
      return {
        type,
        params: { scaleX: 1.2, scaleY: 1.2, relative: true },
      };

    case ActuatorType.Animate:
      return {
        type,
        params: { animationType: "pulse", duration: 500 },
      };
  }
}

/**
 * Creates a new empty logic rule
 */
export function createLogicRule(): LogicRule {
  return {
    id: `rule-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`,
    sensor: createDefaultSensor(SensorType.MouseOver),
    actuators: [createDefaultActuator(ActuatorType.Highlight)],
    enabled: true,
  };
}

/**
 * Serializes logic rules for WASM transmission
 */
export function serializeLogicRules(logic: EntityLogic): string {
  return JSON.stringify(logic);
}

/**
 * Deserializes logic rules from WASM response
 */
export function deserializeLogicRules(data: string): EntityLogic {
  return JSON.parse(data);
}
