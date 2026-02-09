/**
 * useBehavior Hook - React Hook for declarative Behavior attachment
 *
 * Provides a simple way to attach behaviors to entities in React components.
 * Manages the lifecycle of behaviors automatically.
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * USAGE EXAMPLE
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * import { useBehavior } from './hooks/useBehavior';
 * import { behaviorTemplates } from './sdk/BehaviorTemplates';
 *
 * function DraggableShape({ id }) {
 *   const archflow = useArchFlowWasm();
 *
 *   // Create behavior once
 *   const draggable = useMemo(() =>
 *     behaviorTemplates.draggable(archflow.bridge?.logicSystem)
 *   , [archflow.bridge?.logicSystem]);
 *
 *   // Attach to entity
 *   useBehavior(id, draggable);
 *
 *   return <Shape id={id} />;
 * }
 *
 * // Multiple behaviors
 * function InteractiveShape({ id }) {
 *   const archflow = useArchFlowWasm();
 *
 *   const behaviors = useMemo(() => [
 *     behaviorTemplates.hoverable(archflow.bridge?.logicSystem),
 *     behaviorTemplates.selectable(archflow.bridge?.logicSystem),
 *     behaviorTemplates.draggable(archflow.bridge?.logicSystem),
 *   ], [archflow.bridge?.logicSystem]);
 *
 *   useBehaviors(id, behaviors);
 *
 *   return <Shape id={id} />;
 * }
 *
 * // Custom behavior with events
 * function EventShape({ id }) {
 *   const archflow = useArchFlowWasm();
 *
 *   const behavior = useMemo(() => {
 *     return new BehaviorBuilder(archflow.bridge?.logicSystem)
 *       .onClick()
 *       .emit('customClick')
 *       .build();
 *   }, [archflow.bridge?.logicSystem]);
 *
 *   useBehavior(id, behavior, {
 *     onEvent: (event) => {
 *       console.log('Behavior event:', event);
 *     }
 *   });
 *
 *   return <Shape id={id} />;
 * }
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 */

import { useEffect, useRef, useCallback, useMemo } from 'react';
import type { BehaviorBridge } from '../sdk/BehaviorBuilder';
import type { BehaviorEvent } from '../sdk/BehaviorBuilder';

// ============================================================================
// TYPES
// ============================================================================

/**
 * Options for useBehavior hook
 */
export interface UseBehaviorOptions {
  /** Enable/disable the behavior */
  enabled?: boolean;
  /** Callback for behavior events */
  onEvent?: (event: BehaviorEvent, entityId: string) => void;
  /** Cleanup action on unmount */
  cleanup?: 'detach' | 'destroy' | 'none';
  /** Auto-update behavior each frame */
  autoUpdate?: boolean;
}

/**
 * Hook result
 */
export interface UseBehaviorResult {
  /** Attached entity ID */
  entityId: string | null;
  /** Whether behavior is attached */
  isAttached: boolean;
  /** Force update behavior */
  update: () => void;
  /** Manually detach */
  detach: () => void;
}

// ============================================================================
// HOOKS
// ============================================================================

/**
 * Attach a single behavior to an entity
 *
 * @param entityId - Entity ID to attach behavior to
 * @param behavior - Behavior to attach (null for no behavior)
 * @param options - Hook options
 * @returns Hook result with state
 *
 * @example
 * ```typescript
 * useBehavior(entityId, draggableBehavior, {
 *   onEvent: (event) => console.log(event),
 * });
 * ```
 */
export function useBehavior(
  entityId: string | null,
  behavior: BehaviorBridge | null,
  options: UseBehaviorOptions = {},
): UseBehaviorResult {
  const {
    enabled = true,
    onEvent,
    cleanup = 'detach',
    autoUpdate = false,
  } = options;

  const behaviorRef = useRef<BehaviorBridge | null>(behavior);
  const entityIdRef = useRef<string | null>(entityId);
  const unsubscribeRef = useRef<(() => void) | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  // Keep refs updated
  useEffect(() => {
    behaviorRef.current = behavior;
    entityIdRef.current = entityId;
  }, [behavior, entityId]);

  // Attach behavior when entityId and behavior are ready
  useEffect(() => {
    if (!enabled || !entityId || !behavior) {
      return;
    }

    const currentBehavior = behaviorRef.current;
    const currentEntityId = entityIdRef.current;

    if (!currentBehavior || !currentEntityId) {
      return;
    }

    // Attach behavior
    currentBehavior.attach(currentEntityId);

    // Subscribe to events if callback provided
    if (onEvent) {
      const unsubscribe = currentBehavior.on('event', (event) => {
        onEvent(event, currentEntityId);
      });
      unsubscribeRef.current = unsubscribe;
    }

    // Animation loop for auto-update
    if (autoUpdate) {
      const animate = (timestamp: number) => {
        if (behaviorRef.current?.isAttached) {
          behaviorRef.current.update(timestamp);
          animationFrameRef.current = requestAnimationFrame(animate);
        }
      };
      animationFrameRef.current = requestAnimationFrame(animate);
    }

    // Cleanup function
    return () => {
      // Unsubscribe from events
      if (unsubscribeRef.current) {
        unsubscribeRef.current();
        unsubscribeRef.current = null;
      }

      // Cancel animation frame
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }

      // Cleanup behavior
      if (currentBehavior && cleanup !== 'none') {
        if (cleanup === 'destroy') {
          currentBehavior.destroy();
        } else {
          currentBehavior.detach(currentEntityId);
        }
      }
    };
  }, [entityId, behavior, enabled, onEvent, cleanup, autoUpdate]);

  // Force update
  const update = useCallback(() => {
    if (behaviorRef.current?.isAttached) {
      behaviorRef.current.update(performance.now());
    }
  }, []);

  // Manual detach
  const detach = useCallback(() => {
    if (behaviorRef.current && entityIdRef.current) {
      behaviorRef.current.detach(entityIdRef.current);
    }
  }, []);

  return {
    entityId,
    isAttached: enabled && !!entityId && !!behavior,
    update,
    detach,
  };
}

/**
 * Attach multiple behaviors to an entity
 *
 * @param entityId - Entity ID to attach behaviors to
 * @param behaviors - Array of behaviors to attach
 * @param options - Hook options
 *
 * @example
 * ```typescript
 * useBehaviors(entityId, [
 *   draggableBehavior,
 *   selectableBehavior,
 *   hoverableBehavior,
 * ]);
 * ```
 */
export function useBehaviors(
  entityId: string | null,
  behaviors: BehaviorBridge[],
  options: UseBehaviorOptions = {},
): void {
  const {
    enabled = true,
    onEvent,
    cleanup = 'detach',
    autoUpdate = false,
  } = options;

  const behaviorsRef = useRef<BehaviorBridge[]>(behaviors);
  const entityIdRef = useRef<string | null>(entityId);
  const unsubscribesRef = useRef<(() => void)[]>([]);

  // Keep refs updated
  useEffect(() => {
    behaviorsRef.current = behaviors;
    entityIdRef.current = entityId;
  }, [behaviors, entityId]);

  // Attach behaviors
  useEffect(() => {
    if (!enabled || !entityId || behaviors.length === 0) {
      return;
    }

    const currentBehaviors = behaviorsRef.current;
    const currentEntityId = entityIdRef.current;
    const unsubscribes: (() => void)[] = [];

    // Attach each behavior
    currentBehaviors.forEach((behavior) => {
      if (!behavior) return;

      behavior.attach(currentEntityId!);

      if (onEvent) {
        const unsubscribe = behavior.on('event', (event) => {
          onEvent(event, currentEntityId!);
        });
        unsubscribes.push(unsubscribe);
      }
    });

    unsubscribesRef.current = unsubscribes;

    // Cleanup
    return () => {
      // Unsubscribe from events
      unsubscribes.forEach((unsubscribe) => unsubscribe());
      unsubscribesRef.current = [];

      // Cleanup behaviors
      if (cleanup !== 'none') {
        currentBehaviors.forEach((behavior) => {
          if (!behavior) return;

          if (cleanup === 'destroy') {
            behavior.destroy();
          } else {
            behavior.detach(currentEntityId!);
          }
        });
      }
    };
  }, [entityId, behaviors, enabled, onEvent, cleanup]);
}

/**
 * Create and attach a behavior inline
 *
 * Convenience hook that creates a behavior from a builder and attaches it.
 *
 * @param entityId - Entity ID to attach behavior to
 * @param builderFactory - Factory function that creates a BehaviorBuilder
 * @param options - Hook options
 *
 * @example
 * ```typescript
 * useBehaviorFromBuilder(entityId, (builder) => {
 *   return builder
 *     .onClick()
 *     .select()
 *     .onHover()
 *     .highlight({ color: 0xffff00 })
 *     .build();
 * });
 * ```
 */
export function useBehaviorFromBuilder(
  entityId: string | null,
  builderFactory: (logicSystem: any) => BehaviorBridge,
  options: UseBehaviorOptions = {},
): UseBehaviorResult {
  const archflow = require('../hooks/useArchFlowWasm').useArchFlowWasm();

  const behavior = useMemo(() => {
    if (!archflow.bridge?.logicSystem) return null;
    return builderFactory(archflow.bridge.logicSystem);
  }, [archflow.bridge?.logicSystem, builderFactory]);

  return useBehavior(entityId, behavior, options);
}

/**
 * Attach a behavior template to an entity
 *
 * Convenience hook for using predefined templates.
 *
 * @param entityId - Entity ID to attach template to
 * @param templateName - Name of the template to use
 * @param templateArgs - Arguments to pass to the template
 * @param options - Hook options
 *
 * @example
 * ```typescript
 * useBehaviorTemplate(entityId, 'draggable', { snap: 8 });
 * useBehaviorTemplate(entityId, 'interactive');
 * ```
 */
export function useBehaviorTemplate(
  entityId: string | null,
  templateName: string,
  templateArgs?: unknown[],
  options: UseBehaviorOptions = {},
): UseBehaviorResult {
  const archflow = require('../hooks/useArchFlowWasm').useArchFlowWasm();

  const behavior = useMemo(() => {
    if (!archflow.bridge?.logicSystem) return null;

    const { getBehaviorTemplate } = require('../sdk/BehaviorTemplates');
    return getBehaviorTemplate(templateName, archflow.bridge.logicSystem, ...(templateArgs || []));
  }, [archflow.bridge?.logicSystem, templateName, templateArgs]);

  return useBehavior(entityId, behavior, options);
}

// ============================================================================
// BEHAVIOR COMPONENT
// ============================================================================

/**
 * Behavior component - declarative JSX component for behaviors
 *
 * Provides a declarative way to attach behaviors to entities.
 *
 * @example
 * ```tsx
 * <Canvas>
 *   <Shape id="rect-1">
 *     <Behavior entityId="rect-1" behavior={draggableBehavior} />
 *   </Shape>
 *   <Shape id="rect-2">
 *     <Behaviors entityId="rect-2">
 *       <Behavior behavior={hoverableBehavior} />
 *       <Behavior behavior={selectableBehavior} />
 *     </Behaviors>
 *   </Shape>
 * </Canvas>
 * ```
 */
import { memo } from 'react';

/**
 * Single behavior component
 */
export const Behavior = memo(function Behavior({
  entityId,
  behavior,
  enabled = true,
  onEvent,
}: {
  entityId: string;
  behavior: BehaviorBridge | null;
  enabled?: boolean;
  onEvent?: (event: BehaviorEvent) => void;
}) {
  useBehavior(entityId, behavior, { enabled, onEvent });
  return null;
});

/**
 * Multiple behaviors component
 */
export const Behaviors = memo(function Behaviors({
  entityId,
  behaviors,
  enabled = true,
  onEvent,
}: {
  entityId: string;
  behaviors: BehaviorBridge[];
  enabled?: boolean;
  onEvent?: (event: BehaviorEvent) => void;
}) {
  useBehaviors(entityId, behaviors, { enabled, onEvent });
  return null;
});

/**
 * Template behavior component
 */
export const BehaviorTemplate = memo(function BehaviorTemplate({
  entityId,
  template,
  args,
  enabled = true,
  onEvent,
}: {
  entityId: string;
  template: string;
  args?: unknown[];
  enabled?: boolean;
  onEvent?: (event: BehaviorEvent) => void;
}) {
  useBehaviorTemplate(entityId, template, args, { enabled, onEvent });
  return null;
});

export default useBehavior;
