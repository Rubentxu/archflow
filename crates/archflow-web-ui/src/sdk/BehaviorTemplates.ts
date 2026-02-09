/**
 * ArchFlow Behavior Templates - Predefined Behavior Patterns
 *
 * Provides ready-to-use behavior templates for common interactive patterns.
 * These templates can be used directly with useBehavior() or attached manually.
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * USAGE EXAMPLE
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * import { behaviorTemplates } from './BehaviorTemplates';
 * import { useBehavior } from '../hooks/useBehavior';
 *
 * function DraggableShape({ id }) {
 *   useBehavior(id, behaviorTemplates.draggable(archflow.logicSystem));
 *   return <Shape id={id} />;
 * }
 *
 * // Or compose multiple behaviors
 * function InteractiveShape({ id }) {
 *   useBehaviors(id, [
 *     behaviorTemplates.hoverable(archflow.logicSystem),
 *     behaviorTemplates.draggable(archflow.logicSystem),
 *     behaviorTemplates.selectable(archflow.logicSystem),
 *   ]);
 *   return <Shape id={id} />;
 * }
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 */

import type { LogicSystemWasm, BehaviorBridge } from './BehaviorBuilder';
import { BehaviorBuilder } from './BehaviorBuilder';

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/**
 * Highlight style configuration
 */
export interface HighlightStyle {
  color?: number;
  opacity?: number;
}

/**
 * Move configuration
 */
export interface MoveConfig {
  axis?: 'both' | 'x' | 'y';
  snap?: number;
}

/**
 * Select configuration
 */
export interface SelectConfig {
  mode?: 'single' | 'multi' | 'toggle';
}

// ============================================================================
// TEMPLATE CREATORS
// ============================================================================

/**
 * Create a draggable behavior
 *
 * Makes an entity draggable with mouse.
 */
export function createDraggable(
  logicSystem: LogicSystemWasm,
  config?: MoveConfig,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onPointerDown()
    .direct()
    .translate()
    .onPointerUp()
    .build()
    .named('draggable')
    .describedAs('Makes entity draggable with mouse');
}

/**
 * Create a selectable behavior
 *
 * Makes an entity clickable for selection.
 */
export function createSelectable(
  logicSystem: LogicSystemWasm,
  config?: SelectConfig,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onClick()
    .direct()
    .select()
    .build()
    .named('selectable')
    .describedAs('Makes entity clickable for selection');
}

/**
 * Create a hoverable behavior
 *
 * Highlights entity on hover.
 */
export function createHoverable(
  logicSystem: LogicSystemWasm,
  highlightStyle?: HighlightStyle,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onHover()
    .direct()
    .highlight(highlightStyle)
    .build()
    .named('hoverable')
    .describedAs('Highlights entity on hover');
}

/**
 * Create an editable behavior
 *
 * Opens editor on double-click.
 */
export function createEditable(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDoubleClick()
    .direct()
    .emit('editStart')
    .build()
    .named('editable')
    .describedAs('Opens editor on double-click');
}

/**
 * Create a deletable behavior
 *
 * Deletes entity on Delete key press.
 */
export function createDeletable(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onKey('Delete')
    .direct()
    .delete()
    .build()
    .named('deletable')
    .describedAs('Deletes entity on Delete key');
}

/**
 * Create a resizable behavior
 *
 * Enables resize via handles.
 */
export function createResizable(
  logicSystem: LogicSystemWasm,
  config?: { snap?: number },
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDrag()
    .direct()
    .resize()
    .build()
    .named('resizable')
    .describedAs('Enables resize via handles');
}

/**
 * Create a rotatable behavior
 *
 * Enables rotation via rotate handle.
 */
export function createRotatable(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDrag()
    .direct()
    .rotate()
    .build()
    .named('rotatable')
    .describedAs('Enables rotation via handle');
}

/**
 * Create an interactive behavior
 *
 * Complete interactive pattern: hover highlight + click select + drag move.
 */
export function createInteractive(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onHover()
    .direct()
    .highlight({ color: 0xff2196f3, opacity: 0.2 })
    .onClick()
    .direct()
    .select()
    .onDrag()
    .direct()
    .translate()
    .build()
    .named('interactive')
    .describedAs('Complete interactive pattern');
}

/**
 * Create a deletable-with-confirm behavior
 *
 * Shows confirmation dialog before delete.
 */
export function createDeletableWithConfirm(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onKey('Delete')
    .direct()
    .emit('deleteRequest')
    .build()
    .named('deletableWithConfirm')
    .describedAs('Shows confirmation before delete');
}

/**
 * Create a snap-to-grid behavior
 *
 * Snaps movement to grid during drag.
 */
export function createSnapToGrid(
  logicSystem: LogicSystemWasm,
  gridSize: number = 8,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDrag()
    .direct()
    .translate((ctx) => ({
      x: Math.round((ctx.point?.x || 0) / gridSize) * gridSize,
      y: Math.round((ctx.point?.y || 0) / gridSize) * gridSize,
    }))
    .build()
    .named('snapToGrid')
    .describedAs(`Snaps to ${gridSize}px grid`);
}

/**
 * Create a marquee-selectable behavior
 *
 * Enables marquee selection.
 */
export function createMarqueeSelectable(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDrag()
    .direct()
    .emit('marqueeSelect')
    .build()
    .named('marqueeSelectable')
    .describedAs('Enables marquee selection');
}

/**
 * Create a toggle-select behavior
 *
 * Toggles selection on click (no Shift needed).
 */
export function createToggleSelect(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onClick()
    .direct()
    .toggle()
    .build()
    .named('toggleSelect')
    .describedAs('Toggles selection on click');
}

/**
 * Create a multi-select behavior
 *
 * Multi-select with Shift modifier.
 */
export function createMultiSelect(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onClick()
    .withModifier({ shift: true })
    .direct()
    .toggle()
    .build()
    .named('multiSelect')
    .describedAs('Multi-select with Shift modifier');
}

/**
 * Create a bounded-drag behavior
 *
 * Constrains drag within bounds.
 */
export function createBoundedDrag(
  logicSystem: LogicSystemWasm,
  bounds: { minX?: number; maxX?: number; minY?: number; maxY?: number },
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDrag()
    .direct()
    .translate((ctx) => {
      const point = ctx.point || { x: 0, y: 0 };
      return {
        x: Math.max(bounds.minX || 0, Math.min(bounds.maxX || Infinity, point.x)),
        y: Math.max(bounds.minY || 0, Math.min(bounds.maxY || Infinity, point.y)),
      };
    })
    .build()
    .named('boundedDrag')
    .describedAs('Constrains drag within bounds');
}

/**
 * Create a zoom-to-fit behavior
 *
 * Zooms to fit entity on double-click.
 */
export function createZoomToFit(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDoubleClick()
    .direct()
    .emit('zoomToFit')
    .build()
    .named('zoomToFit')
    .describedAs('Zooms to fit on double-click');
}

/**
 * Create a connection behavior
 *
 * Creates connection on drag from connection point.
 */
export function createConnection(
  logicSystem: LogicSystemWasm,
): BehaviorBridge {
  return new BehaviorBuilder(logicSystem)
    .onDrag()
    .direct()
    .emit('connectionStart')
    .onPointerUp()
    .direct()
    .emit('connectionEnd')
    .build()
    .named('connection')
    .describedAs('Creates connection on drag');
}

// ============================================================================
// TEMPLATE REGISTRY
// ============================================================================

/**
 * Predefined behavior templates
 */
export const behaviorTemplates = {
  draggable: createDraggable,
  selectable: createSelectable,
  hoverable: createHoverable,
  editable: createEditable,
  deletable: createDeletable,
  resizable: createResizable,
  rotatable: createRotatable,
  interactive: createInteractive,
  deletableWithConfirm: createDeletableWithConfirm,
  snapToGrid: createSnapToGrid,
  marqueeSelectable: createMarqueeSelectable,
  toggleSelect: createToggleSelect,
  multiSelect: createMultiSelect,
  boundedDrag: createBoundedDrag,
  zoomToFit: createZoomToFit,
  connection: createConnection,
};

/**
 * Template names for reference
 */
export const TEMPLATE_NAMES = Object.keys(behaviorTemplates) as Array<
  keyof typeof behaviorTemplates
>;

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

/**
 * Get a template by name
 */
export function getBehaviorTemplate(
  name: string,
  logicSystem: LogicSystemWasm,
  ...args: unknown[]
): BehaviorBridge | null {
  const template = behaviorTemplates[name as keyof typeof behaviorTemplates];
  if (!template) {
    console.warn(`Behavior template "${name}" not found`);
    return null;
  }
  return template(logicSystem, ...args);
}

/**
 * Create all default behaviors for an entity
 */
export function createDefaultBehaviors(
  logicSystem: LogicSystemWasm,
): BehaviorBridge[] {
  return [
    createHoverable(logicSystem),
    createSelectable(logicSystem),
    createDraggable(logicSystem),
  ];
}

export default behaviorTemplates;
