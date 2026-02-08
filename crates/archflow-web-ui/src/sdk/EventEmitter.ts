/**
 * EventEmitter - Type-safe Event System for ArchFlow
 *
 * This module provides a centralized, type-safe event system for managing
 * domain events across the ArchFlow application. It supports wildcards,
 * throttling, metrics tracking, and memory-safe cleanup.
 *
 * Architecture Reference: EPIC-WEB-013 HU-002
 *
 * @example
 * ```typescript
 * import { globalEvents, DomainEvents } from './EventEmitter';
 *
 * // Subscribe to entity creation
 * const unsubscribe = globalEvents.on('entity:created', (payload) => {
 *   console.log('Entity created:', payload.id, payload.type);
 * });
 *
 * // Emit an event
 * globalEvents.emit('entity:created', { id: '123', type: 'rectangle' });
 *
 * // Clean up
 * unsubscribe();
 * ```
 */

// ═══════════════════════════════════════════════════════════════════════════════
// DOMAIN EVENTS INTERFACE
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * 2D Vector for position and delta
 */
export interface Vector2 {
  x: number;
  y: number;
}

/**
 * Mouse button types
 */
export type MouseButton = 'left' | 'middle' | 'right';

/**
 * System status types
 */
export type SystemStatus = 'idle' | 'busy' | 'error' | 'ready';

/**
 * Domain events interface with all event types from EPIC-WEB-013
 *
 * Each event type has its corresponding payload type for full type safety.
 */
export interface DomainEvents {
  // Entity lifecycle events
  'entity:created': { id: string; type: string };
  'entity:deleted': { id: string };
  'entity:updated': { id: string; changes: Record<string, unknown> };

  // Selection events
  'entity:selected': { id: string; previous?: string };
  'entity:deselected': { id: string };

  // Drag & Drop events
  'drag:started': { entityId: string; position: Vector2 };
  'drag:moved': { entityId: string; delta: Vector2 };
  'drag:ended': { entityId: string; finalPosition: Vector2 };

  // Interaction events
  'hover:changed': { entityId: string; isHovered: boolean };
  'clicked': { entityId: string; button: MouseButton };

  // Property events
  'property:changed': { entityId: string; property: string; value: unknown };

  // System events
  'status:changed': { status: SystemStatus };
  'error:occurred': { message: string; code: number };
}

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Callback function type for event listeners
 */
export type Callback<T = unknown> = (payload: T) => void;

/**
 * Event listener options
 */
export interface EventListenerOptions {
  /** Throttle time in milliseconds (prevents rapid-fire execution) */
  throttle?: number;
  /** Debounce time in milliseconds (delays execution until pause) */
  debounce?: number;
}

/**
 * Subscription handle for cleanup
 */
export interface Subscription {
  /** Unsubscribe from the event */
  unsubscribe: () => void;
  /** Check if subscription is still active */
  readonly isActive: boolean;
}

/**
 * Internal listener wrapper
 */
interface ListenerWrapper<T> {
  callback: Callback<T>;
  options?: EventListenerOptions;
  lastCalled: number;
  debounceTimer: number | null;
  isOnce: boolean;
}

/**
 * Event metrics for monitoring and debugging
 */
export interface EventMetricsSnapshot {
  /** Total number of events emitted */
  totalEmits: number;
  /** Total number of events processed */
  totalProcessed: number;
  /** Number of events dropped due to throttling */
  throttledCount: number;
  /** Number of errors during event processing */
  errorCount: number;
  /** Listener count per event type */
  listenerCounts: Record<string, number>;
  /** Timestamp of last emit */
  lastEmitTime: number;
}

/**
 * Internal metrics tracker
 */
class EventMetrics {
  private totalEmits: number = 0;
  private totalProcessed: number = 0;
  private throttledCount: number = 0;
  private errorCount: number = 0;
  private listenerCounts: Map<string, number> = new Map();
  private lastEmitTime: number = 0;

  recordEmit(): void {
    this.totalEmits++;
    this.lastEmitTime = performance.now();
  }

  recordProcessed(): void {
    this.totalProcessed++;
  }

  recordThrottled(): void {
    this.throttledCount++;
  }

  recordError(): void {
    this.errorCount++;
  }

  setListenerCount(event: string, count: number): void {
    this.listenerCounts.set(event, count);
  }

  getSnapshot(): EventMetricsSnapshot {
    return {
      totalEmits: this.totalEmits,
      totalProcessed: this.totalProcessed,
      throttledCount: this.throttledCount,
      errorCount: this.errorCount,
      listenerCounts: Object.fromEntries(this.listenerCounts),
      lastEmitTime: this.lastEmitTime,
    };
  }

  reset(): void {
    this.totalEmits = 0;
    this.totalProcessed = 0;
    this.throttledCount = 0;
    this.errorCount = 0;
    this.listenerCounts.clear();
    this.lastEmitTime = 0;
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT EMITTER CLASS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Type-safe EventEmitter class for domain events
 *
 * Features:
 * - Full TypeScript type safety with generics
 * - Wildcard support for event patterns (e.g., 'entity:*')
 * - Throttling and debouncing for high-frequency events
 * - Once subscriptions (auto-unsubscribe after first call)
 * - Metrics tracking for monitoring and debugging
 * - Memory-safe cleanup with unsubscribe handles
 *
 * @template T - Record type mapping event names to their payload types
 */
export class EventEmitter<T extends Record<string, unknown>> {
  private listeners = new Map<keyof T, Set<ListenerWrapper<unknown>>>();
  private wildcardListeners = new Map<string, Set<ListenerWrapper<unknown>>>();
  private metrics = new EventMetrics();

  /**
   * Subscribe to an event with a callback
   *
   * @param event - The event name to listen to
   * @param callback - Function to call when event is emitted
   * @param options - Optional throttle/debounce settings
   * @returns Subscription handle with unsubscribe method
   */
  on<K extends keyof T>(
    event: K,
    callback: Callback<T[K]>,
    options?: EventListenerOptions
  ): Subscription {
    const wrapper: ListenerWrapper<T[K]> = {
      callback: callback as Callback<unknown>,
      options,
      lastCalled: 0,
      debounceTimer: null,
      isOnce: false,
    };

    let eventListeners = this.listeners.get(event);
    if (!eventListeners) {
      eventListeners = new Set();
      this.listeners.set(event, eventListeners);
    }
    eventListeners.add(wrapper as ListenerWrapper<unknown>);

    this.metrics.setListenerCount(event as string, eventListeners.size);

    let isActive = true;
    return {
      unsubscribe: () => {
        if (!isActive) return;
        isActive = false;
        const listeners = this.listeners.get(event);
        if (listeners) {
          listeners.delete(wrapper as ListenerWrapper<unknown>);
          this.metrics.setListenerCount(event as string, listeners.size);
        }
      },
      get isActive() {
        return isActive;
      },
    };
  }

  /**
   * Subscribe to wildcard event patterns (e.g., 'entity:*', 'drag:*')
   */
  onWildcard(
    pattern: string,
    callback: (event: string, payload: unknown) => void,
    options?: EventListenerOptions
  ): Subscription {
    const wrapper: ListenerWrapper<unknown> = {
      callback,
      options,
      lastCalled: 0,
      debounceTimer: null,
      isOnce: false,
    };

    let patternListeners = this.wildcardListeners.get(pattern);
    if (!patternListeners) {
      patternListeners = new Set();
      this.wildcardListeners.set(pattern, patternListeners);
    }
    patternListeners.add(wrapper);

    let isActive = true;
    return {
      unsubscribe: () => {
        if (!isActive) return;
        isActive = false;
        const listeners = this.wildcardListeners.get(pattern);
        if (listeners) {
          listeners.delete(wrapper);
        }
      },
      get isActive() {
        return isActive;
      },
    };
  }

  /**
   * Subscribe to an event that will only fire once
   */
  once<K extends keyof T>(event: K, callback: Callback<T[K]>): Subscription {
    const wrapper: ListenerWrapper<T[K]> = {
      callback: callback as Callback<unknown>,
      options: undefined,
      lastCalled: 0,
      debounceTimer: null,
      isOnce: true,
    };

    let eventListeners = this.listeners.get(event);
    if (!eventListeners) {
      eventListeners = new Set();
      this.listeners.set(event, eventListeners);
    }
    eventListeners.add(wrapper as ListenerWrapper<unknown>);

    this.metrics.setListenerCount(event as string, eventListeners.size);

    let isActive = true;
    return {
      unsubscribe: () => {
        if (!isActive) return;
        isActive = false;
        const listeners = this.listeners.get(event);
        if (listeners) {
          listeners.delete(wrapper as ListenerWrapper<unknown>);
          this.metrics.setListenerCount(event as string, listeners.size);
        }
      },
      get isActive() {
        return isActive;
      },
    };
  }

  /**
   * Emit an event with a payload
   */
  emit<K extends keyof T>(event: K, payload: T[K]): void {
    this.metrics.recordEmit();

    const now = performance.now();
    const eventStr = event as string;

    // Process direct listeners
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      const listenersToRemove: ListenerWrapper<unknown>[] = [];

      for (const wrapper of eventListeners) {
        if (this.shouldExecute(wrapper, now)) {
          try {
            wrapper.callback(payload);
            this.metrics.recordProcessed();

            if (wrapper.isOnce) {
              listenersToRemove.push(wrapper);
            }
            wrapper.lastCalled = now;
          } catch (error) {
            console.error(`Error in event listener for "${eventStr}":`, error);
            this.metrics.recordError();
          }
        }
      }

      // Remove once listeners
      for (const wrapper of listenersToRemove) {
        eventListeners.delete(wrapper);
      }
      this.metrics.setListenerCount(eventStr, eventListeners.size);
    }

    // Process wildcard listeners
    for (const [pattern, patternListeners] of this.wildcardListeners) {
      if (this.matchesWildcard(eventStr, pattern)) {
        const listenersToRemove: ListenerWrapper<unknown>[] = [];

        for (const wrapper of patternListeners) {
          if (this.shouldExecute(wrapper, now)) {
            try {
              (wrapper.callback as (event: string, payload: unknown) => void)(
                eventStr,
                payload
              );
              this.metrics.recordProcessed();

              if (wrapper.isOnce) {
                listenersToRemove.push(wrapper);
              }
              wrapper.lastCalled = now;
            } catch (error) {
              console.error(
                `Error in wildcard listener for pattern "${pattern}":`,
                error
              );
              this.metrics.recordError();
            }
          }
        }

        // Remove once listeners
        for (const wrapper of listenersToRemove) {
          patternListeners.delete(wrapper);
        }
      }
    }
  }

  /**
   * Remove a specific listener for an event
   */
  off<K extends keyof T>(event: K, callback: Callback<T[K]>): void {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners) return;

    for (const wrapper of eventListeners) {
      if (wrapper.callback === callback) {
        eventListeners.delete(wrapper);
        break;
      }
    }

    this.metrics.setListenerCount(event as string, eventListeners.size);
  }

  /**
   * Remove all listeners for a specific event, or all events if none specified
   */
  removeAllListeners<K extends keyof T>(event?: K): void {
    if (event !== undefined) {
      this.listeners.delete(event);
      this.metrics.setListenerCount(event as string, 0);
    } else {
      this.listeners.clear();
      this.wildcardListeners.clear();
      this.metrics.listenerCounts.clear();
    }
  }

  /**
   * Clear all listeners and reset metrics
   */
  clear(): void {
    this.listeners.clear();
    this.wildcardListeners.clear();
    this.metrics.reset();
  }

  /**
   * Get current metrics snapshot
   */
  getMetrics(): EventMetricsSnapshot {
    // Update listener counts before returning
    for (const [event, listeners] of this.listeners) {
      this.metrics.setListenerCount(event as string, listeners.size);
    }
    return this.metrics.getSnapshot();
  }

  /**
   * Get the number of listeners for a specific event
   */
  getListenerCount<K extends keyof T>(event: K): number {
    const listeners = this.listeners.get(event);
    return listeners ? listeners.size : 0;
  }

  /**
   * Check if there are any listeners for an event
   */
  hasListeners<K extends keyof T>(event: K): boolean {
    const listeners = this.listeners.get(event);
    return listeners ? listeners.size > 0 : false;
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // PRIVATE METHODS
  // ═══════════════════════════════════════════════════════════════════════════════

  private shouldExecute(wrapper: ListenerWrapper<unknown>, now: number): boolean {
    const { throttle, debounce } = wrapper.options || {};

    // Handle debounce
    if (debounce !== undefined) {
      if (wrapper.debounceTimer !== null) {
        clearTimeout(wrapper.debounceTimer);
      }

      wrapper.debounceTimer = window.setTimeout(() => {
        wrapper.callback = () => {};
        wrapper.debounceTimer = null;
      }, debounce);

      return false;
    }

    // Handle throttle
    if (throttle !== undefined) {
      const timeSinceLastCall = now - wrapper.lastCalled;
      if (timeSinceLastCall < throttle) {
        this.metrics.recordThrottled();
        return false;
      }
    }

    return true;
  }

  private matchesWildcard(event: string, pattern: string): boolean {
    const regex = new RegExp(
      '^' + pattern.replace(/\*/g, '.*').replace(/\?/g, '.') + '$'
    );
    return regex.test(event);
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL EVENT EMITTER INSTANCE
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Global event emitter instance for domain events
 */
export const globalEvents = new EventEmitter<DomainEvents>();

/**
 * Expose global events to DevTools in development mode
 */
if (typeof window !== 'undefined' && process.env.NODE_ENV === 'development') {
  (window as any).__ARCHFLOW_EVENTS__ = globalEvents;
}

export default EventEmitter;
