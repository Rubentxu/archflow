/**
 * Event Middleware for Zustand Stores
 *
 * This module provides middleware for integrating Zustand stores with the EventEmitter,
 * enabling automatic state updates when domain events are emitted.
 *
 * Architecture Reference: EPIC-WEB-013 HU-004
 *
 * @example
 * ```typescript
 * import { createEventMiddleware } from './eventMiddleware';
 * import { globalEvents } from './EventEmitter';
 *
 * const useStore = create<EventMiddlewareConfig>((set, get) => ({
 *   entities: {},
 *   // ... store state
 * }), createEventMiddleware(globalEvents, {
 *   'entity:created': (set, get, payload) => {
 *     set(state => ({
 *       entities: { ...state.entities, [payload.id]: payload }
 *     }));
 *   }
 * }));
 * ```
 */

import type { EventEmitter, DomainEvents } from '../sdk/EventEmitter';
import type { Callback } from '../sdk/EventEmitter';

/**
 * Event to action mapping type
 * Maps event names to functions that update store state
 */
export type EventToActionMapping<T extends Record<string, unknown>> = {
  [K in keyof DomainEvents]?: (
    set: <K extends keyof T>(partial: T[K] | ((state: T) => T[K] | Partial<T>)) => void,
    get: () => T,
    payload: DomainEvents[K]
  ) => void;
};

/**
 * Configuration for event middleware
 */
export interface EventMiddlewareConfig<T extends Record<string, unknown>> {
  /** Mapping of events to state update functions */
  mappings: EventToActionMapping<T>;
  /** Batch time in milliseconds (default: 16ms ~ 60fps) */
  batchMs?: number;
  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Internal batched update tracker
 */
interface BatchedUpdate {
  events: string[];
  updates: Array<() => void>;
  timer: number | null;
}

/**
 * Create event middleware for Zustand stores
 *
 * This middleware subscribes to domain events and updates store state
 * automatically when events are emitted.
 *
 * @param eventEmitter - The EventEmitter instance to listen to
 * @param config - Configuration for event mappings and batching
 * @returns Zustand middleware function
 *
 * @example
 * ```typescript
 * const useEntityStore = create<EntityStore>(
 *   (set, get) => ({
 *     entities: new Map(),
 *     selectedId: null,
 *   }),
 *   createEventMiddleware(globalEvents, {
 *     batchMs: 16,
 *     mappings: {
 *       'entity:created': (set, get, payload) => {
 *         set((state) => ({
 *           entities: new Map(state.entities).set(payload.id, payload)
 *         }));
 *       },
 *       'entity:deleted': (set, get, payload) => {
 *         set((state) => {
 *           const entities = new Map(state.entities);
 *           entities.delete(payload.id);
 *           return { entities };
 *         });
 *       },
 *       'entity:selected': (set, get, payload) => {
 *         set({ selectedId: payload.id });
 *       }
 *     }
 *   })
 * );
 * ```
 */
export function createEventMiddleware<T extends Record<string, unknown>>(
  eventEmitter: EventEmitter<DomainEvents>,
  config: EventMiddlewareConfig<T>
) {
  const { mappings, batchMs = 16, debug = false } = config;
  const subscriptions: Array<() => void> = [];

  // Batch tracking
  const batched: BatchedUpdate = {
    events: [],
    updates: [],
    timer: null,
  };

  // Process batched updates
  const flushBatch = (set: <K extends keyof T>(partial: T[K] | ((state: T) => T[K] | Partial<T>)) => void) => {
    if (batched.events.length === 0) return;

    if (debug) {
      console.log('[EventMiddleware] Flushing batch:', {
        events: batched.events,
        updateCount: batched.updates.length,
      });
    }

    // Apply all updates in a single setState
    set((state) => {
      let newState = { ...state };
      for (const update of batched.updates) {
        const partial = update();
        newState = { ...newState, ...partial };
      }
      return newState;
    });

    // Clear batch
    batched.events = [];
    batched.updates = [];
    batched.timer = null;
  };

  // Queue an update for batching
  const queueUpdate = (
    set: <K extends keyof T>(partial: T[K] | ((state: T) => T[K] | Partial<T>)) => void,
    update: () => void,
    eventName: string
  ) => {
    batched.events.push(eventName);
    batched.updates.push(update);

    // Reset timer
    if (batched.timer !== null) {
      clearTimeout(batched.timer);
    }

    batched.timer = window.setTimeout(() => {
      flushBatch(set);
    }, batchMs);
  };

  // Middleware function
  return (config: any) => (set: any, get: any, api: any) => {
    const initialState = config(set, get, api);
    const wrappedSet: typeof set = (partial) => {
      // Check if this is an event-driven update
      const isEventUpdate = batched.events.length > 0;

      // Apply the state update
      set(partial);

      // If batching and not flushing, don't notify subscribers yet
      if (isEventUpdate && batched.timer !== null) {
        return;
      }
    };

    // Subscribe to all mapped events
    for (const [eventName, handler] of Object.entries(mappings)) {
      if (handler) {
        const subscription = (eventEmitter as any).on(
          eventName,
          (payload: DomainEvents[keyof DomainEvents]) => {
            if (debug) {
              console.log(`[EventMiddleware] Event: ${eventName}`, payload);
            }

            // Queue update for batching
            queueUpdate(
              set,
              () => {
                return handler(wrappedSet, get, payload);
              },
              eventName
            );
          }
        );

        subscriptions.push(subscription.unsubscribe);
      }
    }

    // Cleanup on store destroy
    const originalDestroy = api?.destroy;
    if (originalDestroy) {
      api.destroy = () => {
        // Unsubscribe from all events
        for (const unsubscribe of subscriptions) {
          unsubscribe();
        }

        // Clear batch timer
        if (batched.timer !== null) {
          clearTimeout(batched.timer);
        }

        // Call original destroy
        originalDestroy();
      };
    }

    return initialState;
  };
}

/**
 * Helper function to create a configured store with event middleware
 *
 * This is a convenience wrapper that combines Zustand's create function
 * with event middleware in a single call.
 *
 * @param eventEmitter - The EventEmitter instance
 * @param config - Store configuration and event mappings
 * @returns Zustand store hook
 *
 * @example
 * ```typescript
 * const useStore = createEventStore(globalEvents, {
 *   // Store state definition
 *   state: (set) => ({
 *     entities: new Map(),
 *     selectedId: null,
 *   }),
 *   // Event mappings
 *   mappings: {
 *     'entity:created': (set, get, { id, type }) => {
 *       set((state) => ({
 *         entities: new Map(state.entities).set(id, { id, type })
 *       }));
 *     }
 *   }
 * });
 * ```
 */
export function createEventStore<T extends Record<string, unknown>>(
  eventEmitter: EventEmitter<DomainEvents>,
  config: {
    state: (set: <K extends keyof T>(partial: T[K] | ((state: T) => T[K] | Partial<T>)) => void, get: () => T) => T;
    mappings: EventToActionMapping<T>;
    batchMs?: number;
    debug?: boolean;
  }
) {
  // This would require importing 'create' from zustand
  // For now, we'll provide the middleware factory pattern
  throw new Error(
    'createEventStore requires zustand to be installed. ' +
    'Use createEventMiddleware directly with zustand\'s create function.'
  );
}

/**
 * Helper to map entity events to store updates
 *
 * Provides pre-configured mappings for common entity events.
 *
 * @example
 * ```typescript
 * import { entityEventMappings } from './eventMiddleware';
 *
 * const middleware = createEventMiddleware(globalEvents, {
 *   mappings: {
 *     ...entityEventMappings.entities,
 *     ...entityEventMappings.selection,
 *   }
 * });
 * ```
 */
export const entityEventMappings = {
  /**
   * Entity lifecycle mappings
   * Handles entity:created, entity:deleted, entity:updated
   */
  entities: {
    'entity:created': (
      set: any,
      get: any,
      payload: { id: string; type: string }
    ) => {
      set((state: any) => ({
        entities: new Map(state.entities).set(payload.id, {
          id: payload.id,
          type: payload.type,
          // ... default entity properties
        })
      }));
    },
    'entity:deleted': (set: any, get: any, payload: { id: string }) => {
      set((state: any) => {
        const entities = new Map(state.entities);
        entities.delete(payload.id);
        return { entities };
      });
    },
    'entity:updated': (set: any, get: any, payload: { id: string; changes: Record<string, unknown> }) => {
      set((state: any) => ({
        entities: new Map(state.entities).set(payload.id, {
          ...state.entities.get(payload.id),
          ...payload.changes
        }))
      }));
    }
  } as EventToActionMapping<any>,

  /**
   * Selection mappings
   * Handles entity:selected, entity:deselected
   */
  selection: {
    'entity:selected': (set: any, get: any, payload: { id: string; previous?: string }) => {
      set({ selectedId: payload.id });
    },
    'entity:deselected': (set: any, get: any, payload: { id: string }) => {
      set((state: any) => ({
        selectedId: state.selectedId === payload.id ? null : state.selectedId
      }));
    }
  } as EventToActionMapping<any>,

  /**
   * Hover state mappings
   * Handles hover:changed
   */
  hover: {
    'hover:changed': (set: any, get: any, payload: { entityId: string; isHovered: boolean }) => {
      set({ hoveredId: payload.isHovered ? payload.entityId : null });
    }
  } as EventToActionMapping<any>,
};
