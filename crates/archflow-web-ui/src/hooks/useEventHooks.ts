/**
 * Domain Event Hooks for ArchFlow
 *
 * Custom React hooks for subscribing to domain events with automatic cleanup.
 * These hooks provide type-safe, memoized access to event subscriptions.
 *
 * Architecture Reference: EPIC-WEB-013 HU-006
 *
 * @example
 * ```typescript
 * import { useEntityCreated, useDragEvents, useHoverState } from './useEventHooks';
 *
 * function MyComponent() {
 *   // Subscribe to entity creation
 *   useEntityCreated((payload) => {
 *     console.log('Entity created:', payload.id);
 *   });
 *
 *   // Get current selected entity ID
 *   const selectedId = useEntitySelected();
 *
 *   // Track hover state
 *   const isHovered = useHoverState(entityId);
 * }
 * ```
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import type {
  DomainEvents,
  EventEmitter,
  Subscription,
  Vector2,
  MouseButton,
} from '../sdk/EventEmitter';

/**
 * Base hook for subscribing to events with automatic cleanup
 *
 * @param emitter - The EventEmitter instance
 * @param event - The event name to listen to
 * @param callback - Function to call when event is emitted
 * @param options - Event listener options (throttle, debounce)
 *
 * @example
 * ```typescript
 * useEventSubscription(globalEvents, 'entity:created', (payload) => {
 *   console.log('Entity created:', payload);
 * });
 * ```
 */
export function useEventSubscription<K extends keyof DomainEvents>(
  emitter: EventEmitter<DomainEvents>,
  event: K,
  callback: (payload: DomainEvents[K]) => void,
  options?: { throttle?: number; debounce?: number }
): void {
  // Use ref to track if component is mounted
  const isMounted = useRef(true);

  // Wrap callback in ref to avoid stale closures
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    isMounted.current = true;

    // Create wrapped callback that checks mount status
    const wrappedCallback = (payload: DomainEvents[K]) => {
      if (isMounted.current) {
        callbackRef.current(payload);
      }
    };

    const subscription = emitter.on(event, wrappedCallback, options);

    return () => {
      isMounted.current = false;
      subscription.unsubscribe();
    };
  }, [emitter, event, options]);
}

/**
 * Subscribe to entity creation events
 *
 * @param emitter - The EventEmitter instance
 * @param callback - Function called when entity is created
 *
 * @example
 * ```typescript
 * useEntityCreated(globalEvents, ({ id, type }) => {
 *   console.log(`New ${type} entity created: ${id}`);
 * });
 * ```
 */
export function useEntityCreated(
  emitter: EventEmitter<DomainEvents>,
  callback: (payload: { id: string; type: string }) => void
): void {
  useEventSubscription(emitter, 'entity:created', callback);
}

/**
 * Subscribe to entity deletion events
 *
 * @param emitter - The EventEmitter instance
 * @param callback - Function called when entity is deleted
 *
 * @example
 * ```typescript
 * useEntityDeleted(globalEvents, ({ id }) => {
 *   console.log(`Entity deleted: ${id}`);
 * });
 * ```
 */
export function useEntityDeleted(
  emitter: EventEmitter<DomainEvents>,
  callback: (payload: { id: string }) => void
): void {
  useEventSubscription(emitter, 'entity:deleted', callback);
}

/**
 * Get and track the currently selected entity ID
 *
 * Returns the current selected ID and automatically updates when
 * entity:selected events are emitted.
 *
 * @param emitter - The EventEmitter instance
 * @returns Current selected entity ID or null
 *
 * @example
 * ```typescript
 * function EntityInfo() {
 *   const selectedId = useEntitySelected(globalEvents);
 *   return selectedId ? <div>Selected: {selectedId}</div> : null;
 * }
 * ```
 */
export function useEntitySelected(emitter: EventEmitter<DomainEvents>): string | null {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  useEffect(() => {
    // Initial sync could go here if needed
    return;
  }, []);

  useEventSubscription(emitter, 'entity:selected', (payload) => {
    setSelectedId(payload.id);
  });

  useEventSubscription(emitter, 'entity:deselected', (payload) => {
    setSelectedId((current) => (current === payload.id ? null : current));
  });

  return selectedId;
}

/**
 * Subscribe to drag events for an entity
 *
 * @param emitter - The EventEmitter instance
 * @param entityId - The entity ID to track drag events for
 * @param callbacks - Optional drag event handlers
 *
 * @example
 * ```typescript
 * useDragEvents(globalEvents, entityId, {
 *   onStarted: (position) => console.log('Drag started at', position),
 *   onMoved: (delta) => console.log('Drag moved by', delta),
 *   onEnded: (finalPosition) => console.log('Drag ended at', finalPosition),
 * });
 * ```
 */
export function useDragEvents(
  emitter: EventEmitter<DomainEvents>,
  entityId: string,
  callbacks?: {
    onStarted?: (position: Vector2) => void;
    onMoved?: (delta: Vector2) => void;
    onEnded?: (finalPosition: Vector2) => void;
  }
): void {
  useEventSubscription(emitter, 'drag:started', (payload) => {
    if (payload.entityId === entityId && callbacks?.onStarted) {
      callbacks.onStarted(payload.position);
    }
  });

  useEventSubscription(emitter, 'drag:moved', (payload) => {
    if (payload.entityId === entityId && callbacks?.onMoved) {
      callbacks.onMoved(payload.delta);
    }
  });

  useEventSubscription(emitter, 'drag:ended', (payload) => {
    if (payload.entityId === entityId && callbacks?.onEnded) {
      callbacks.onEnded(payload.finalPosition);
    }
  });
}

/**
 * Track hover state for a specific entity
 *
 * Returns whether the entity is currently hovered and automatically
 * updates when hover:changed events are emitted.
 *
 * @param emitter - The EventEmitter instance
 * @param entityId - The entity ID to track hover state for
 * @returns Whether the entity is currently hovered
 *
 * @example
 * ```typescript
 * function Entity({ id }) {
 *   const isHovered = useHoverState(globalEvents, id);
 *   return <div style={{ opacity: isHovered ? 0.8 : 1 }}>Entity</div>;
 * }
 * ```
 */
export function useHoverState(
  emitter: EventEmitter<DomainEvents>,
  entityId: string
): boolean {
  const [isHovered, setIsHovered] = useState(false);

  useEventSubscription(emitter, 'hover:changed', (payload) => {
    if (payload.entityId === entityId) {
      setIsHovered(payload.isHovered);
    }
  });

  return isHovered;
}

/**
 * Subscribe to click events for an entity
 *
 * @param emitter - The EventEmitter instance
 * @param entityId - The entity ID to track clicks for
 * @param callback - Function called when entity is clicked
 *
 * @example
 * ```typescript
 * useClicked(globalEvents, entityId, (button) => {
 *   if (button === 'left') {
 *     console.log('Left clicked!');
 *   }
 * });
 * ```
 */
export function useClicked(
  emitter: EventEmitter<DomainEvents>,
  entityId: string,
  callback: (button: MouseButton) => void
): void {
  useEventSubscription(emitter, 'clicked', (payload) => {
    if (payload.entityId === entityId) {
      callback(payload.button);
    }
  });
}

/**
 * Subscribe to property change events for an entity
 *
 * @param emitter - The EventEmitter instance
 * @param entityId - The entity ID to track property changes for
 * @param property - Optional property name to filter by
 * @param callback - Function called when property changes
 *
 * @example
 * ```typescript
 * usePropertyChanged(globalEvents, entityId, 'color', (value) => {
 *   console.log('Color changed to:', value);
 * });
 * ```
 */
export function usePropertyChanged(
  emitter: EventEmitter<DomainEvents>,
  entityId: string,
  property?: string,
  callback: (value: unknown) => void
): void {
  useEventSubscription(emitter, 'property:changed', (payload) => {
    if (payload.entityId === entityId) {
      if (!property || payload.property === property) {
        callback(payload.value);
      }
    }
  });
}

/**
 * Track system status
 *
 * Returns the current system status and automatically updates when
 * status:changed events are emitted.
 *
 * @param emitter - The EventEmitter instance
 * @returns Current system status
 *
 * @example
 * ```typescript
 * function StatusIndicator() {
 *   const status = useSystemStatus(globalEvents);
 *   return <div>Status: {status}</div>;
 * }
 * ```
 */
export function useSystemStatus(emitter: EventEmitter<DomainEvents>): 'idle' | 'busy' | 'error' | 'ready' {
  const [status, setStatus] = useState<'idle' | 'busy' | 'error' | 'ready'>('idle');

  useEventSubscription(emitter, 'status:changed', (payload) => {
    setStatus(payload.status);
  });

  return status;
}

/**
 * Subscribe to error events
 *
 * @param emitter - The EventEmitter instance
 * @param callback - Function called when error occurs
 *
 * @example
 * ```typescript
 * useErrorOccurred(globalEvents, (error) => {
 *   console.error('Error:', error.message, error.code);
 * });
 * ```
 */
export function useErrorOccurred(
  emitter: EventEmitter<DomainEvents>,
  callback: (error: { message: string; code: number }) => void
): void {
  useEventSubscription(emitter, 'error:occurred', callback);
}

/**
 * Subscribe to wildcard event patterns
 *
 * @param emitter - The EventEmitter instance
 * @param pattern - Wildcard pattern (e.g., 'entity:*', 'drag:*')
 * @param callback - Function called with event name and payload
 * @param options - Event listener options
 *
 * @example
 * ```typescript
 * useWildcardEvent(globalEvents, 'entity:*', (event, payload) => {
 *   console.log(`Entity event: ${event}`, payload);
 * });
 * ```
 */
export function useWildcardEvent(
  emitter: EventEmitter<DomainEvents>,
  pattern: string,
  callback: (event: string, payload: unknown) => void,
  options?: { throttle?: number; debounce?: number }
): void {
  const isMounted = useRef(true);
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    isMounted.current = true;

    const wrappedCallback = (event: string, payload: unknown) => {
      if (isMounted.current) {
        callbackRef.current(event, payload);
      }
    };

    const subscription = emitter.onWildcard(pattern, wrappedCallback, options);

    return () => {
      isMounted.current = false;
      subscription.unsubscribe();
    };
  }, [emitter, pattern, options]);
}

/**
 * Subscribe to events only once
 *
 * @param emitter - The EventEmitter instance
 * @param event - The event name to listen to
 * @param callback - Function to call once when event is emitted
 *
 * @example
 * ```typescript
 * useEventOnce(globalEvents, 'entity:created', (payload) => {
 *   console.log('This will only log once');
 * });
 * ```
 */
export function useEventOnce<K extends keyof DomainEvents>(
  emitter: EventEmitter<DomainEvents>,
  event: K,
  callback: (payload: DomainEvents[K]) => void
): void {
  const isMounted = useRef(true);
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    isMounted.current = true;

    const wrappedCallback = (payload: DomainEvents[K]) => {
      if (isMounted.current) {
        callbackRef.current(payload);
      }
    };

    const subscription = emitter.once(event, wrappedCallback);

    return () => {
      isMounted.current = false;
      subscription.unsubscribe();
    };
  }, [emitter, event]);
}
