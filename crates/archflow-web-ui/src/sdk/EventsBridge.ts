/**
 * EventsBridge - Event System for ArchFlow
 *
 * This facade organizes event-related methods from WasmBridge by domain.
 * Provides methods for tick, poll_events, and event handling.
 *
 * @example
 * ```typescript
 * const bridge = new ArchFlowBridge(wasmBridge);
 * bridge.events.tick(timestamp);
 * const eventCount = bridge.events.poll();
 * ```
 */

import type { WasmBridge } from './types';

/**
 * Event types from WASM
 */
export type ArchFlowEventType =
  | 'entity_created'
  | 'entity_deleted'
  | 'entity_moved'
  | 'selection_changed'
  | 'tool_changed'
  | 'zoom_changed'
  | 'other';

/**
 * Event payload
 */
export interface ArchFlowEvent {
  type: ArchFlowEventType;
  timestamp: number;
  data: unknown;
}

/**
 * Tick result information
 */
export interface TickResult {
  eventsProcessed: number;
  tookMs: number;
}

/**
 * Event and tick operations
 */
export class EventsBridge {
  constructor(private bridge: WasmBridge) {}

  // ═══════════════════════════════════════════════════════════════════════════════════════
  // TICK & EVENTS
  // ═══════════════════════════════════════════════════════════════════════════════════════

  /**
   * Main tick function for the engine loop
   * @param timestamp - Current timestamp in milliseconds
   */
  tick(timestamp: number): TickResult {
    const start = performance.now();
    this.bridge.tick(timestamp);
    const eventsProcessed = this.poll();
    const tookMs = performance.now() - start;
    return { eventsProcessed, tookMs };
  }

  /**
   * Poll and process all pending events from WASM
   * @returns Number of events processed
   */
  poll(): number {
    return this.bridge.poll_events();
  }

  /**
   * Get events since last poll
   */
  getEvents(): ArchFlowEvent[] {
    // TODO: Implement event queue in WasmBridge
    console.warn('getEvents() - implementation pending in WasmBridge');
    return [];
  }

  /**
   * Clear all pending events
   */
  clear(): void {
    // TODO: Implement clear_events in WasmBridge
    console.warn('clear() - implementation pending in WasmBridge');
  }

  // ═══════════════════════════════════════════════════════════════════════════════════════
  // EVENT HANDLING
  // ═══════════════════════════════════════════════════════════════════════════════════════

  /**
   * Subscribe to event type
   */
  on<T extends ArchFlowEventType>(
    type: T,
    callback: (event: ArchFlowEvent & { type: T }) => void,
  ): () => void {
    // TODO: Implement event subscription in WasmBridge
    console.warn('on() - implementation pending in WasmBridge');
    return () => {};
  }

  /**
   * Subscribe to event once
   */
  once<T extends ArchFlowEventType>(
    type: T,
    callback: (event: ArchFlowEvent & { type: T }) => void,
  ): void {
    // TODO: Implement once subscription in WasmBridge
    console.warn('once() - implementation pending in WasmBridge');
  }

  /**
   * Unsubscribe from event type
   */
  off<T extends ArchFlowEventType>(type: T): void {
    // TODO: Implement event unsubscription in WasmBridge
    console.warn('off() - implementation pending in WasmBridge');
  }

  /**
   * Emit custom event (for internal use)
   */
  emit(type: ArchFlowEventType, data?: unknown): void {
    // TODO: Implement event emission in WasmBridge
    console.warn('emit() - implementation pending in WasmBridge');
  }
}

/**
 * Create a new EventsBridge instance
 */
export function createEventsBridge(bridge: any): EventsBridge {
  return new EventsBridge(bridge);
}

