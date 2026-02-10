/**
 * BehaviorComposer - Utility for composing complex behaviors
 *
 * This utility provides fluent API for composing behaviors from multiple
 * components (sensors, controllers, actuators), as specified in the Developer Manual.
 *
 * Developer Manual: "Patrón: Behavior Composition"
 */

import { BehaviorDefinition, ComponentDefinition } from '../services/ArchFlowService';

// ============================================================================
// BEHAVIOR COMPOSER
// ============================================================================

/**
 * BehaviorComposer - Fluent API for composing behaviors
 *
 * Provides a chainable API for building complex behaviors from individual
 * components (sensors, controllers, actuators).
 *
 * @example
 * ```typescript
 * const hoverAndClick = new BehaviorComposer('hover-click')
 *   .addSensor('sensor-mouse', { mode: 'hover' })
 *   .addActuator('actuator-highlight', { color: '#ff0' })
 *   .addSensor('sensor-mouse', { mode: 'click' })
 *   .addActuator('actuator-select', { mode: 'single' })
 *   .compose();
 * ```
 */
export class BehaviorComposer {
  private id: string;
  private name: string;
  private description?: string;
  private sensors: ComponentDefinition[] = [];
  private controllers: ComponentDefinition[] = [];
  private actuators: ComponentDefinition[] = [];

  constructor(id: string, name?: string) {
    this.id = id;
    this.name = name || `Behavior-${id}`;
  }

  /**
   * Set behavior description
   */
  describedAs(description: string): this {
    this.description = description;
    return this;
  }

  /**
   * Add sensor component
   */
  addSensor(type: string, config: Record<string, unknown>): this {
    this.sensors.push({ type, config });
    return this;
  }

  /**
   * Add multiple sensors
   */
  addSensors(sensors: ComponentDefinition[]): this {
    this.sensors.push(...sensors);
    return this;
  }

  /**
   * Add controller component
   */
  addController(type: string, config: Record<string, unknown>): this {
    this.controllers.push({ type, config });
    return this;
  }

  /**
   * Add multiple controllers
   */
  addControllers(controllers: ComponentDefinition[]): this {
    this.controllers.push(...controllers);
    return this;
  }

  /**
   * Add actuator component
   */
  addActuator(type: string, config: Record<string, unknown>): this {
    this.actuators.push({ type, config });
    return this;
  }

  /**
   * Add multiple actuators
   */
  addActuators(actuators: ComponentDefinition[]): this {
    this.actuators.push(...actuators);
    return this;
  }

  /**
   * Compose all components into a behavior definition
   */
  compose(): BehaviorDefinition {
    return {
      id: this.id,
      name: this.name,
      description: this.description,
      components: [
        ...this.sensors,
        ...this.controllers,
        ...this.actuators
      ]
    };
  }

  /**
   * Create a simple sensor-controller-actuator chain
   */
  chain(
    sensor: ComponentDefinition,
    controller: ComponentDefinition,
    actuator: ComponentDefinition
  ): BehaviorDefinition {
    return {
      id: this.id,
      name: this.name,
      description: this.description,
      components: [sensor, controller, actuator]
    };
  }
}

// ============================================================================
// PRESET COMPOSERS
// ============================================================================

/**
 * Preset composers for common patterns
 */
export class BehaviorPresets {
  /**
   * Create hover highlight behavior
   */
  static hoverHighlight(
    options: { color?: string; opacity?: number } = {}
  ): BehaviorDefinition {
    return new BehaviorComposer('hover-highlight', 'Hover Highlight')
      .describedAs('Highlights entity on hover')
      .addSensor('sensor-mouse', { mode: 'hover' })
      .addController('controller-direct', {})
      .addActuator('actuator-highlight', {
        color: options.color ?? '#ffff00',
        opacity: options.opacity ?? 0.5
      })
      .compose();
  }

  /**
   * Create click select behavior
   */
  static clickSelect(
    options: { button?: number; mode?: 'single' | 'multi' | 'toggle' } = {}
  ): BehaviorDefinition {
    return new BehaviorComposer('click-select', 'Click Select')
      .describedAs('Selects entity on click')
      .addSensor('sensor-mouse', { mode: 'click', button: options.button ?? 0 })
      .addController('controller-direct', {})
      .addActuator('actuator-select', { mode: options.mode ?? 'single' })
      .compose();
  }

  /**
   * Create draggable behavior
   */
  static draggable(
    options: { debounce?: number; speed?: number } = {}
  ): BehaviorDefinition {
    return new BehaviorComposer('draggable', 'Draggable')
      .describedAs('Makes entity draggable')
      .addSensor('sensor-mouse', { mode: 'drag', button: 0 })
      .addController('controller-debounce', { ticks: options.debounce ?? 6 })
      .addActuator('actuator-move', {
        mode: 'follow-cursor',
        speed: options.speed ?? 5
      })
      .compose();
  }

  /**
   * Create hover and click combined behavior
   */
  static hoverAndClick(
    options: {
      highlightColor?: string;
      selectMode?: 'single' | 'multi' | 'toggle'
    } = {}
  ): BehaviorDefinition {
    const composer = new BehaviorComposer('hover-click', 'Hover and Click')
      .describedAs('Highlights on hover and selects on click');

    // Hover chain
    composer
      .addSensor('sensor-mouse', { mode: 'hover' })
      .addController('controller-direct', {})
      .addActuator('actuator-highlight', {
        color: options.highlightColor ?? '#ffff00',
        opacity: 0.3
      });

    // Click chain
    composer
      .addSensor('sensor-mouse', { mode: 'click', button: 0 })
      .addController('controller-direct', {})
      .addActuator('actuator-select', { mode: options.selectMode ?? 'toggle' });

    return composer.compose();
  }

  /**
   * Create WASD movement behavior
   */
  static wasdMovement(
    options: { keys?: number[]; speed?: number } = {}
  ): BehaviorDefinition {
    return new BehaviorComposer('wasd-movement', 'WASD Movement')
      .describedAs('Move entity with WASD keys')
      .addSensor('sensor-keyboard', {
        keys: options.keys ?? [87, 65, 83, 68],
        modifiers: 0
      })
      .addController('controller-direct', {})
      .addActuator('actuator-move', {
        mode: 'relative',
        speed: options.speed ?? 5
      })
      .compose();
  }

  /**
   * Create deletable behavior
   */
  static deletable(): BehaviorDefinition {
    return new BehaviorComposer('deletable', 'Deletable')
      .describedAs('Delete entity with Delete key')
      .addSensor('sensor-keyboard', { keys: [46], modifiers: 0 })
      .addController('controller-direct', {})
      .addActuator('actuator-delete', {})
      .compose();
  }

  /**
   * Create editable behavior
   */
  static editable(): BehaviorDefinition {
    return new BehaviorComposer('editable', 'Editable')
      .describedAs('Double-click to edit')
      .addSensor('sensor-mouse', { mode: 'dblclick' })
      .addController('controller-direct', {})
      .addActuator('actuator-event', {
        name: 'editStart',
        data: {}
      })
      .compose();
  }

  /**
   * Create snap-to-grid behavior
   */
  static snapToGrid(gridSize: number = 8): BehaviorDefinition {
    return new BehaviorComposer(`snap-grid-${gridSize}`, 'Snap to Grid')
      .describedAs(`Snap movement to ${gridSize}px grid`)
      .addSensor('sensor-mouse', { mode: 'drag', button: 0 })
      .addController('controller-debounce', { ticks: 3 })
      .addActuator('actuator-move', {
        mode: 'follow-cursor',
        speed: 5,
        snap: gridSize
      })
      .compose();
  }

  /**
   * Create multi-select behavior
   */
  static multiSelect(): BehaviorDefinition {
    return new BehaviorComposer('multi-select', 'Multi-Select')
      .describedAs('Multi-select with Shift modifier')
      .addSensor('sensor-mouse', { mode: 'click', button: 0 })
      .addSensor('sensor-keyboard', { keys: [16], modifiers: 1 }) // Shift
      .addController('controller-and', {})
      .addActuator('actuator-select', { mode: 'toggle' })
      .compose();
  }

  /**
   * Create bounded drag behavior
   */
  static boundedDrag(
    bounds: { minX?: number; maxX?: number; minY?: number; maxY?: number }
  ): BehaviorDefinition {
    return new BehaviorComposer('bounded-drag', 'Bounded Drag')
      .describedAs('Constrains drag within bounds')
      .addSensor('sensor-mouse', { mode: 'drag', button: 0 })
      .addController('controller-debounce', { ticks: 3 })
      .addActuator('actuator-move', {
        mode: 'follow-cursor',
        speed: 5,
        bounds
      })
      .compose();
  }
}

// ============================================================================
// ADVANCED COMPOSITION
// ============================================================================

/**
 * BehaviorComposerBuilder - Advanced composition with conditions
 */
export class BehaviorComposerBuilder extends BehaviorComposer {
  private conditions: Array<(behavior: BehaviorDefinition) => boolean> = [];

  /**
   * Add condition for behavior composition
   */
  when(condition: (behavior: BehaviorDefinition) => boolean): this {
    this.conditions.push(condition);
    return this;
  }

  /**
   * Compose if all conditions are met
   */
  composeIf(): BehaviorDefinition | null {
    const behavior = this.compose();
    const passes = this.conditions.every(cond => cond(behavior));

    return passes ? behavior : null;
  }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/**
 * Create behavior composer with fluent API
 */
export function composeBehavior(id: string, name?: string): BehaviorComposer {
  return new BehaviorComposer(id, name);
}

/**
 * Create behavior from preset
 */
export function createPreset(
  preset: keyof typeof BehaviorPresets,
  options?: any
): BehaviorDefinition {
  return BehaviorPresets[preset](options);
}

/**
 * Combine multiple behaviors into one
 */
export function combineBehaviors(
  id: string,
  name: string,
  ...behaviors: BehaviorDefinition[]
): BehaviorDefinition {
  const composer = new BehaviorComposer(id, name);

  for (const behavior of behaviors) {
    for (const component of behavior.components) {
      const type = component.type;

      if (type.startsWith('sensor-')) {
        composer.addSensor(type, component.config);
      } else if (type.startsWith('controller-')) {
        composer.addController(type, component.config);
      } else if (type.startsWith('actuator-')) {
        composer.addActuator(type, component.config);
      }
    }
  }

  return composer.compose();
}

// ============================================================================
// VALIDATION
// ============================================================================

/**
 * Validate behavior definition
 */
export function validateBehavior(behavior: BehaviorDefinition): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!behavior.id) {
    errors.push('Behavior ID is required');
  }

  if (!behavior.name) {
    errors.push('Behavior name is required');
  }

  if (!behavior.components || behavior.components.length === 0) {
    errors.push('Behavior must have at least one component');
  }

  let hasSensor = false;
  let hasActuator = false;

  for (const component of behavior.components) {
    if (!component.type) {
      errors.push('Component type is required');
      continue;
    }

    if (component.type.startsWith('sensor-')) {
      hasSensor = true;
    } else if (component.type.startsWith('actuator-')) {
      hasActuator = true;
    }
  }

  if (!hasSensor) {
    errors.push('Behavior must have at least one sensor');
  }

  if (!hasActuator) {
    errors.push('Behavior must have at least one actuator');
  }

  return {
    valid: errors.length === 0,
    errors
  };
}

export default BehaviorComposer;
