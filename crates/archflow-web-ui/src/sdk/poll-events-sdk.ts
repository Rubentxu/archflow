/**
 * ArchFlow Event SDK - Minimal TypeScript SDK for poll_events()
 *
 * Provides typed access to events from the Rust WASM logic system.
 * Each event type includes specific metadata fields.
 *
 * Architecture: EPIC-WEB-011 (JS-tonto hybrid)
 *
 * @example
 * ```typescript
 * import { pollEvents, EventType, type ArchFlowEvent } from '@archflow/sdk';
 *
 * for (const event of pollEvents()) {
 *   if (event.type === EventType.ProximityAlert) {
 *     console.log(`Entity ${event.entityId} is ${event.data1} units away`);
 *   }
 * }
 * ```
 */

import type { WasmBridge } from './types';

// ═══════════════════════════════════════════════════════════════════════════════════
// EVENT TYPE CONSTANTS (matching Rust LogicEventType)
// ═══════════════════════════════════════════════════════════════════════════════════

/** Event type constants from Rust LogicEventType enum */
export const EventType = {
  /** Entity was selected/deselected */
  EntitySelected: 0,
  /** Proximity threshold crossed */
  ProximityAlert: 1,
  /** Drag operation started */
  DragStarted: 2,
  /** Drag operation ended */
  DragEnded: 3,
  /** Entity was destroyed */
  EntityDestroyed: 4,
  /** Selection box completed (for box selection) */
  BoxSelectionCompleted: 5,
  /** Hover state changed */
  HoverChanged: 6,
} as const;

/** Type alias for event type constants */
export type EventType = typeof EventType[keyof typeof EventType];

// ═══════════════════════════════════════════════════════════════════════════════════
// EVENT INTERFACES WITH METADATA
// ═══════════════════════════════════════════════════════════════════════════════════

/** Base event structure from WASM */
interface BaseWasmEvent {
  readonly event_type: EventType;
  readonly entity_id: number;
  readonly timestamp_us: number;
  readonly data_1: number;  // f32 in WASM
  readonly data_2: number;  // f32 in WASM
  readonly data_3: number;  // u32 in WASM
}

/**
 * EntitySelected event - entity selection changed
 * Metadata: none (data_1, data_2, data_3 are all 0)
 */
export interface EntitySelectedEvent {
  readonly type: typeof EventType.EntitySelected;
  readonly entityId: number;
  readonly timestampUs: number;
}

/**
 * ProximityAlert event - entity is within threshold distance
 * data_1: f32 distance value that triggered the alert
 */
export interface ProximityAlertEvent {
  readonly type: typeof EventType.ProximityAlert;
  readonly entityId: number;
  readonly timestampUs: number;
  /** Distance in world units that triggered the alert */
  readonly distance: number;
}

/**
 * DragStarted event - drag operation began
 * data_1: starting X position
 * data_2: starting Y position
 */
export interface DragStartedEvent {
  readonly type: typeof EventType.DragStarted;
  readonly entityId: number;
  readonly timestampUs: number;
  /** Starting X position in world coordinates */
  readonly startX: number;
  /** Starting Y position in world coordinates */
  readonly startY: number;
}

/**
 * DragEnded event - drag operation finished
 * data_1: ending X position
 * data_2: ending Y position
 */
export interface DragEndedEvent {
  readonly type: typeof EventType.DragEnded;
  readonly entityId: number;
  readonly timestampUs: number;
  /** Ending X position in world coordinates */
  readonly endX: number;
  /** Ending Y position in world coordinates */
  readonly endY: number;
}

/**
 * EntityDestroyed event - entity was removed from scene
 * Metadata: none
 */
export interface EntityDestroyedEvent {
  readonly type: typeof EventType.EntityDestroyed;
  readonly entityId: number;
  readonly timestampUs: number;
}

/**
 * BoxSelectionCompleted event - box selection finished
 * data_3: number of entities selected
 */
export interface BoxSelectionCompletedEvent {
  readonly type: typeof EventType.BoxSelectionCompleted;
  readonly entityId: number;  // 0 for box selection
  readonly timestampUs: number;
  /** Number of entities selected by the box */
  readonly selectedCount: number;
}

/**
 * HoverChanged event - hover state changed
 * data_3: entity ID being hovered (0 if none)
 */
export interface HoverChangedEvent {
  readonly type: typeof EventType.HoverChanged;
  readonly entityId: number;
  readonly timestampUs: number;
  /** Entity being hovered, or 0 if no entity */
  readonly hoveredEntityId: number;
}

/** Union type of all possible events */
export type ArchFlowEvent =
  | EntitySelectedEvent
  | ProximityAlertEvent
  | DragStartedEvent
  | DragEndedEvent
  | EntityDestroyedEvent
  | BoxSelectionCompletedEvent
  | HoverChangedEvent;

// ═══════════════════════════════════════════════════════════════════════════════════
// EVENT FACTORY (converts WASM events to typed events)
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * Convert raw WASM event to typed event
 */
function toTypedEvent(raw: BaseWasmEvent): ArchFlowEvent {
  switch (raw.event_type) {
    case EventType.EntitySelected:
      return {
        type: EventType.EntitySelected,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
      };
    case EventType.ProximityAlert:
      return {
        type: EventType.ProximityAlert,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
        distance: raw.data_1,
      };
    case EventType.DragStarted:
      return {
        type: EventType.DragStarted,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
        startX: raw.data_1,
        startY: raw.data_2,
      };
    case EventType.DragEnded:
      return {
        type: EventType.DragEnded,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
        endX: raw.data_1,
        endY: raw.data_2,
      };
    case EventType.EntityDestroyed:
      return {
        type: EventType.EntityDestroyed,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
      };
    case EventType.BoxSelectionCompleted:
      return {
        type: EventType.BoxSelectionCompleted,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
        selectedCount: raw.data_3,
      };
    case EventType.HoverChanged:
      return {
        type: EventType.HoverChanged,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
        hoveredEntityId: raw.data_3,
      };
    default:
      // Unknown event type - return as unknown
      return {
        type: raw.event_type as EventType,
        entityId: raw.entity_id,
        timestampUs: raw.timestamp_us,
      } as ArchFlowEvent;
  }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// POLL EVENTS SDK FUNCTION
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * Poll and process all pending events from WASM
 *
 * This function drains all events from the Rust event ring buffer and returns
 * them as typed JavaScript objects with full metadata.
 *
 * @param bridge - The WASM bridge instance
 * @returns Array of typed events with complete metadata
 *
 * @example
 * ```typescript
 * import { pollEvents, EventType } from '@archflow/sdk';
 *
 * function handleEvents(events: ReturnType<typeof pollEvents>) {
 *   for (const event of events) {
 *     switch (event.type) {
 *       case EventType.ProximityAlert:
 *         console.log(`Alert: ${event.distance.toFixed(2)} units`);
 *         break;
 *       case EventType.DragStarted:
 *         console.log(`Drag started at (${event.startX}, ${event.startY})`);
 *         break;
 *       case EventType.BoxSelectionCompleted:
 *         console.log(`Selected ${event.selectedCount} entities`);
 *         break;
 *     }
 *   }
 * }
 * ```
 */
export function pollEvents(bridge: WasmBridge): ArchFlowEvent[] {
  // Get raw events from WASM
  const rawEvents = bridge.logic.event_buffer.drain();
  // Convert to typed events
  return rawEvents.map(toTypedEvent);
}

/**
 * Get the number of pending events without draining
 */
export function eventCount(bridge: WasmBridge): number {
  return bridge.logic.event_buffer.event_count();
}

/**
 * Check if there are any pending events
 */
export function hasEvents(bridge: WasmBridge): boolean {
  return bridge.logic.event_buffer.has_events();
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CONVENIENCE HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * Filter events by type
 */
export function filterByType<E extends ArchFlowEvent>(
  events: ArchFlowEvent[],
  type: E['type']
): E[] {
  return events.filter((e): e is E => e.type === type) as E[];
}

/**
 * Get the most recent event of a given type
 */
export function findLast<E extends ArchFlowEvent>(
  events: ArchFlowEvent[],
  type: E['type']
): E | undefined {
  for (let i = events.length - 1; i >= 0; i--) {
    if (events[i].type === type) {
      return events[i] as E;
    }
  }
  return undefined;
}

/**
 * Check if any event of the given type exists
 */
export function hasEvent(events: ArchFlowEvent[], type: EventType): boolean {
  return events.some(e => e.type === type);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// DEFAULT EXPORT
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * PollEventsSDK - Namespace for event polling utilities
 */
export const PollEventsSDK = {
  poll: pollEvents,
  count: eventCount,
  hasEvents,
  filterByType,
  findLast,
  hasEvent,
  EventType,
};

export default PollEventsSDK;
