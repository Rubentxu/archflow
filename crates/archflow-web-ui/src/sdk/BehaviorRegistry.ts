/**
 * ArchFlow Behaviors SDK - Behavior Registry
 *
 * Manages behavior templates that can be applied to shapes.
 * Each template defines a set of preconfigured behaviors (hover, click, drag, etc.)
 * that map to the Logic Bricks system.
 *
 * Architecture Reference: EPIC-WEB-011, Section 7
 */

import { SensorType, Controller, ActuatorType } from "../wasm/archflow_web.d";
import type { Shape, EntityId, BehaviorConfig } from "./types";

/**
 * Behavior template definition
 */
export interface BehaviorTemplate {
  /** Template identifier */
  name: string;
  /** Display name */
  displayName: string;
  /** Description */
  description: string;
  /** Default configurations for each behavior type */
  behaviors: Record<string, BehaviorConfig>;
}

/**
 * Registry for behavior templates
 *
 * Manages predefined behavior templates that can be applied to shapes.
 * Templates are applied via method calls like `.interactive()`, `.draggable()`, etc.
 */
export class BehaviorRegistry {
  private templates = new Map<string, BehaviorTemplate>();
  private initialized = false;

  constructor() {
    // Defaults are initialized lazily to ensure WASM is ready
  }

  /**
   * Ensure default templates are registered
   */
  private ensureDefaults(): void {
    if (this.initialized) return;
    this.initialized = true;
    this.initDefaults();
  }

  /**
   * Initialize default behavior templates
   */
  private initDefaults(): void {
    // Interactive template: hover highlight
    this.register({
      name: "interactive",
      displayName: "Interactive",
      description: "Hover highlight with subtle effect",
      behaviors: {
        onHover: {
          sensor: SensorType.MouseOver,
          controller: Controller.direct(),
          actuator: ActuatorType.Highlight,
          config: {
            color: 0xffff00,
            opacity: 0.2,
          },
        },
      },
    });

    // Draggable template
    this.register({
      name: "draggable",
      displayName: "Draggable",
      description: "Can be dragged with mouse",
      behaviors: {
        draggable: {
          config: {
            axis: "both",
            snap: 8,
          },
        },
      },
    });

    // Resizable template
    this.register({
      name: "resizable",
      displayName: "Resizable",
      description: "Can be resized with handles",
      behaviors: {
        resizable: {
          config: {
            handles: ["nw", "ne", "sw", "se"],
            snap: 8,
          },
        },
      },
    });

    // Selectable template (single)
    this.register({
      name: "selectable",
      displayName: "Selectable",
      description: "Can be selected on click",
      behaviors: {
        selectable: {
          sensor: SensorType.MouseClick,
          controller: Controller.direct(),
          actuator: ActuatorType.Select,
          config: {
            mode: "single",
          },
        },
      },
    });

    // Multi-selectable template (with Shift modifier)
    this.register({
      name: "selectable-multi",
      displayName: "Multi-Selectable",
      description: "Multi-select with Shift modifier",
      behaviors: {
        selectable: {
          sensor: SensorType.MouseClick,
          controller: Controller.direct(),
          actuator: ActuatorType.Select,
          config: {
            mode: "multi",
            modifier: "Shift",
          },
        },
      },
    });

    // Sticky note template
    this.register({
      name: "sticky-note",
      displayName: "Sticky Note",
      description: "Yellow sticky note with subtle shadow",
      behaviors: {
        onHover: {
          sensor: SensorType.MouseOver,
          controller: Controller.direct(),
          actuator: ActuatorType.Highlight,
          config: {
            color: 0xffff00,
            opacity: 0.2,
          },
        },
      },
    });

    // Card template
    this.register({
      name: "card",
      displayName: "Card",
      description: "Card with shadow and border",
      behaviors: {
        interactive: {
          sensor: SensorType.MouseOver,
          controller: Controller.direct(),
          actuator: ActuatorType.Highlight,
          config: {
            color: 0x00ffff00,
            opacity: 0.1,
          },
        },
      },
    });

    // Diagram node template
    this.register({
      name: "diagram-node",
      displayName: "Diagram Node",
      description: "Node for diagrams with selection box",
      behaviors: {
        interactive: {
          sensor: SensorType.MouseOver,
          controller: Controller.direct(),
          actuator: ActuatorType.Highlight,
          config: {
            color: 0x00ffffff,
            opacity: 0.15,
          },
        },
      },
    });

    // Diagram edge (connector) template
    this.register({
      name: "diagram-edge",
      displayName: "Diagram Edge",
      description: "Connector line between nodes",
      behaviors: {},
    });

    // Text template
    this.register({
      name: "text",
      displayName: "Text",
      description: "Text with selectable behavior",
      behaviors: {
        selectable: {
          sensor: SensorType.MouseClick,
          controller: Controller.direct(),
          actuator: ActuatorType.Select,
          config: {
            mode: "single",
          },
        },
      },
    });
  }

  /**
   * Register a new behavior template
   *
   * @param template - Template definition
   */
  register(template: BehaviorTemplate): void {
    // Use the raw templates map to simply add it
    this.templates.set(template.name, template);
  }

  /**
   * Get a template by name
   *
   * @param name - Template name
   * @returns Template definition or undefined
   */
  get(name: string): BehaviorTemplate | undefined {
    this.ensureDefaults();
    return this.templates.get(name);
  }

  /**
   * List all registered templates
   *
   * @returns Array of template names
   */
  list(): string[] {
    this.ensureDefaults();
    return Array.from(this.templates.keys());
  }

  /**
   * Apply a template to a shape
   *
   * @param shape - The shape to apply the template to
   * @param templateName - Name of the template to apply
   */
  apply(shape: Shape, templateName: string): void {
    this.ensureDefaults();
    const template = this.templates.get(templateName);
    if (!template) {
      console.warn(`Behavior template "${templateName}" not found`);
      return;
    }

    // Apply all behavior configurations from the template
    Object.entries(template.behaviors).forEach(([behaviorName, config]) => {
      this.applyBehavior(shape, behaviorName, config);
    });
  }

  /**
   * Apply a single behavior to a shape
   *
   * @param shape - The shape to apply behavior to
   * @param behaviorName - Name of the behavior (e.g., 'onHover', 'draggable')
   * @param config - Behavior configuration
   */
  private applyBehavior(
    shape: Shape,
    behaviorName: string,
    config: BehaviorConfig,
  ): void {
    // Apply the behavior to the shape through Logic Bricks translation
    this.translateToLogicBricks(
      parseInt(shape.id.replace(/\D/g, ""), 10) || 0,
      behaviorName,
      config,
    );
  }

  /**
   * Translate behavior to Logic Bricks system
   *
   * @param entityId - Entity ID
   * @param behaviorName - Behavior name (e.g., 'onHover', 'draggable')
   * @param config - Configuration parameters
   */
  private translateToLogicBricks(
    entityId: EntityId,
    behaviorName: string,
    config: BehaviorConfig,
  ): void {
    // Import the Logic SDK to apply behaviors
    // This creates the actual Sensor→Controller→Actuator connections
    const translationMap = CANVAS_TRANSLATION_MAP[behaviorName];
    if (!translationMap) {
      console.warn(`No translation map found for behavior: ${behaviorName}`);
      return;
    }

    const { sensor, actuator: actuatorType } = translationMap;

    // Create the appropriate controller based on config
    // For now, we only support Direct controller
    let finalController = Controller.direct();

    // Apply to WASM through the Logic SDK
    // This will be connected when the main SDK is available
    // For now, we store it for later application
    const pendingConnection = {
      entityId,
      sensor,
      controller: finalController,
      actuator: actuatorType,
      config,
    };

    // Store for later application
    if (!(window as any).__archFlowPendingBehaviors) {
      (window as any).__archFlowPendingBehaviors = new Map();
    }
    (window as any).__archFlowPendingBehaviors.set(
      `${entityId}:${behaviorName}`,
      pendingConnection,
    );
  }
}

/**
 * Global behavior registry instance
 */
export const behaviorRegistry = new BehaviorRegistry();

// ═══════════════════════════════════════════════════════════════════════════════
// CANVAS TRANSLATION MAP
// Maps behavior names to Logic Bricks components
// ═════════════════════════════════════════════════════════════════════════════

/**
 * Translation map from behavior names to Logic Bricks components
 *
 * This defines how each high-level behavior maps to the underlying
 * Sensor→Controller→Actuator pattern.
 */
export const CANVAS_TRANSLATION_MAP: Record<
  string,
  {
    sensor: SensorType;
    controller: "Direct" | "And" | "Or";
    actuator: ActuatorType;
    config: Record<string, unknown>;
  }
> = {
  onHover: {
    sensor: SensorType.MouseOver,
    controller: "Direct",
    actuator: ActuatorType.Highlight,
    config: {
      color: 0xffff00,
      opacity: 0.2,
    },
  },

  onClick: {
    sensor: SensorType.MouseClick,
    controller: "Direct",
    actuator: ActuatorType.Select,
    config: {},
  },

  onDoubleClick: {
    sensor: SensorType.MouseClick,
    controller: "Direct",
    actuator: ActuatorType.Move,
    config: {
      handler: "doubleClick",
    },
  },

  onRightClick: {
    sensor: SensorType.KeyShortcut,
    controller: "Direct",
    actuator: ActuatorType.Move,
    config: {},
  },

  draggable: {
    sensor: SensorType.MouseClick,
    controller: "Direct",
    actuator: ActuatorType.Move,
    config: {
      axis: "both",
      snap: 8,
    },
  },

  resizable: {
    sensor: SensorType.MouseClick,
    controller: "Direct",
    actuator: ActuatorType.Move,
    config: {
      handles: ["nw", "ne", "sw", "se"],
    },
  },

  selectable: {
    sensor: SensorType.MouseClick,
    controller: "Direct",
    actuator: ActuatorType.Select,
    config: {
      mode: "single",
    },
  },

  "selectable-multi": {
    sensor: SensorType.MouseClick,
    controller: "Direct",
    actuator: ActuatorType.Select,
    config: {
      mode: "multi",
      modifier: "Shift",
    },
  },

  interactive: {
    sensor: SensorType.MouseOver,
    controller: "Direct",
    actuator: ActuatorType.Highlight,
    config: {
      color: 0xffff00,
      opacity: 0.15,
    },
  },

  tooltip: {
    sensor: SensorType.MouseOver,
    controller: "Direct",
    actuator: ActuatorType.Move,
    config: {
      ticks: 6,
      content: "",
    },
  },

  contextMenu: {
    sensor: SensorType.KeyShortcut,
    controller: "Direct",
    actuator: ActuatorType.Move,
    config: {},
  },
};
