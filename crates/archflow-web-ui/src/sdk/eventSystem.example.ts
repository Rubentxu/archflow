/**
 * ArchFlow Event System - Integration Examples
 *
 * This file demonstrates the complete integration of all event system components:
 * - EventEmitter for type-safe events
 * - Zustand middleware for state management
 * - React hooks for component integration
 * - Web Workers for heavy processing
 * - EventTracker for debugging
 *
 * Architecture Reference: EPIC-WEB-013
 */

import {
  EventEmitter,
  globalEvents,
  DomainEvents,
  createEventMiddleware,
  useEntityCreated,
  useEntitySelected,
  useDragEvents,
  useHoverState,
  EventWorkerPool,
  EventTracker,
  TrackedEventEmitter,
  EventDirection,
} from './index';

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 1: Basic Event Emission and Subscription
// ═══════════════════════════════════════════════════════════════════════════════

export function basicEventExample() {
  console.log('=== Basic Event Example ===');

  // Subscribe to entity creation events
  const unsubscribe = globalEvents.on('entity:created', (payload) => {
    console.log('Entity created:', payload.id, payload.type);
  });

  // Emit an event
  globalEvents.emit('entity:created', {
    id: 'entity-123',
    type: 'rectangle',
    position: { x: 100, y: 100 },
  });

  // Clean up
  unsubscribe();

  // Output:
  // Entity created: entity-123 rectangle
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 2: Event Middleware with Zustand
// ═══════════════════════════════════════════════════════════════════════════════

import { create } from 'zustand';

interface EntityState {
  entities: Map<string, any>;
  selectedId: string | null;
  addEntity: (entity: any) => void;
  selectEntity: (id: string) => void;
}

export function createEventIntegratedStore() {
  // Create the event middleware
  const eventMiddleware = createEventMiddleware(globalEvents, {
    mappings: {
      'entity:created': (state, payload) => ({
        entities: new Map(state.entities).set(payload.id, payload),
      }),
      'entity:selected': (state, payload) => ({
        selectedId: payload.id,
      }),
      'entity:updated': (state, payload) => {
        const entities = new Map(state.entities);
        const existing = entities.get(payload.id);
        if (existing) {
          entities.set(payload.id, { ...existing, ...payload.changes });
        }
        return { entities };
      },
    },
    batchWindow: 16, // 60fps batching
  });

  // Create store with middleware
  return create<EntityState>()(
    eventMiddleware((set, get) => ({
      entities: new Map(),
      selectedId: null,

      addEntity: (entity) => {
        const entities = new Map(get().entities);
        entities.set(entity.id, entity);
        set({ entities });

        // Emit event for other components
        globalEvents.emit('entity:created', entity);
      },

      selectEntity: (id) => {
        set({ selectedId: id });
        globalEvents.emit('entity:selected', { id });
      },
    }))
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 3: React Components with Event Hooks
// ═══════════════════════════════════════════════════════════════════════════════

import React, { useEffect } from 'react';

export function EntityList() {
  // Track created entities
  const createdEntities = useEntityCreated(globalEvents);

  // Track selected entity
  const selectedId = useEntitySelected(globalEvents);

  return (
    <div>
      <h3>Entities ({createdEntities.length})</h3>
      <ul>
        {createdEntities.map((entity) => (
          <li
            key={entity.id}
            style={{
              fontWeight: entity.id === selectedId ? 'bold' : 'normal',
            }}
          >
            {entity.type}: {entity.id}
          </li>
        ))}
      </ul>
    </div>
  );
}

export function DraggableShape({ id }: { id: string }) {
  // Track drag state
  const { isDragging, dragStart, currentPosition } = useDragEvents(globalEvents, id);

  // Track hover state
  const isHovered = useHoverState(globalEvents, id);

  return (
    <div
      style={{
        position: 'absolute',
        left: currentPosition.x,
        top: currentPosition.y,
        cursor: isDragging ? 'grabbing' : 'grab',
        backgroundColor: isHovered ? 'lightblue' : 'white',
        border: '1px solid black',
        padding: '10px',
      }}
    >
      Shape {id}
      {isDragging && <span> (dragging...)</span>}
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 4: Web Workers for Heavy Processing
// ═══════════════════════════════════════════════════════════════════════════════

export function workerProcessingExample() {
  console.log('=== Web Worker Example ===');

  // Create worker pool
  const pool = new EventWorkerPool({
    maxWorkers: 4,
    workerTimeout: 5000,
  });

  // Register a collision detection processor
  pool.registerProcessor<{ entities: any[] }, { collisions: string[] }>(
    'collision-detection',
    async (task) => {
      // Simulate heavy computation
      const collisions: string[] = [];

      for (let i = 0; i < task.data.entities.length; i++) {
        for (let j = i + 1; j < task.data.entities.length; j++) {
          const e1 = task.data.entities[i];
          const e2 = task.data.entities[j];

          // Simple bounding box collision
          if (
            e1.x < e2.x + e2.width &&
            e1.x + e1.width > e2.x &&
            e1.y < e2.y + e2.height &&
            e1.y + e1.height > e2.y
          ) {
            collisions.push(`${e1.id} <-> ${e2.id}`);
          }
        }
      }

      return { collisions };
    }
  );

  // Process entities in worker
  async function checkCollisions(entities: any[]) {
    try {
      const result = await pool.process('collision-detection', { entities });
      console.log('Collisions found:', result.collisions);

      // Emit event with results
      globalEvents.emit('collisions:detected', {
        collisions: result.collisions
      });
    } catch (error) {
      console.error('Collision detection failed:', error);
    }
  }

  // Example usage
  checkCollisions([
    { id: 'rect1', x: 0, y: 0, width: 100, height: 100 },
    { id: 'rect2', x: 50, y: 50, width: 100, height: 100 },
  ]);

  // Cleanup when done
  return () => pool.terminate();
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 5: Event Tracking and Debugging
// ═══════════════════════════════════════════════════════════════════════════════

export function debugExample() {
  console.log('=== Event Tracking Example ===');

  // Create event tracker
  const tracker = new EventTracker({
    maxEvents: 1000,
    enableMetrics: true,
    enableStackTrace: false,
  });

  // Wrap emitter for automatic tracking
  const trackedEmitter = new TrackedEventEmitter(globalEvents, tracker, 'App');

  // Subscribe with tracking
  const sub = trackedEmitter.on('entity:created', (payload) => {
    console.log('Created:', payload.id);
  });

  // Emit events (automatically tracked)
  trackedEmitter.emit('entity:created', { id: '1', type: 'rect' });
  trackedEmitter.emit('entity:selected', { id: '1' });
  trackedEmitter.emit('entity:moved', { id: '1', position: { x: 100, y: 100 } });

  // Get metrics
  const metrics = tracker.getMetrics();
  console.log('Event metrics:', metrics);

  // Export logs
  const logs = tracker.export();
  console.log('Event logs:', logs);

  // Cleanup
  sub.unsubscribe();
  tracker.clear();
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 6: Complete Integration with WASM Bridge
// ═══════════════════════════════════════════════════════════════════════════════

export function completeWasmIntegrationExample() {
  console.log('=== Complete WASM Integration ===');

  // This shows how to integrate the event system with the WASM bridge

  // 1. Create event tracker
  const tracker = new EventTracker();

  // 2. Wrap global events for tracking
  const trackedEvents = new TrackedEventEmitter(globalEvents, tracker, 'WASM-Bridge');

  // 3. Listen for WASM events and forward to event system
  trackedEvents.on('wasm:logic-event', (payload) => {
    console.log('WASM Logic Event:', payload);

    // Forward to appropriate domain event
    switch (payload.event_type) {
      case 'EntityCreated':
        trackedEvents.emit('entity:created', {
          id: payload.entity_id,
          type: payload.entity_type,
        });
        break;
      case 'SensorTriggered':
        trackedEvents.emit('sensor:triggered', {
          entityId: payload.entity_id,
          sensorType: payload.sensor_type,
        });
        break;
      case 'ActuatorFired':
        trackedEvents.emit('actuator:fired', {
          entityId: payload.entity_id,
          actuatorType: payload.actuator_type,
        });
        break;
    }
  });

  // 4. Set up poll-based event draining for high-frequency events
  let drainInterval: number | null = null;

  function startEventDraining(wasmBridge: any) {
    drainInterval = window.setInterval(() => {
      try {
        const events = wasmBridge.drainEvents();

        for (const event of events) {
          trackedEvents.emit('wasm:logic-event', event);
        }
      } catch (error) {
        console.error('Failed to drain WASM events:', error);
      }
    }, 16); // ~60fps
  }

  function stopEventDraining() {
    if (drainInterval !== null) {
      clearInterval(drainInterval);
      drainInterval = null;
    }
  }

  // 5. Set up push-based callbacks for critical events
  function setupWasmCallbacks(wasmBridge: any) {
    // Register callback for entity selection
    wasmBridge.registerCallback('entity-selected', (entityId: string) => {
      trackedEvents.emit('entity:selected', { id: entityId });
    });

    // Register callback for errors
    wasmBridge.registerCallback('error', (error: Error) => {
      trackedEvents.emit('system:error', {
        source: 'wasm',
        message: error.message,
      });
    });
  }

  return {
    tracker,
    trackedEvents,
    startEventDraining,
    stopEventDraining,
    setupWasmCallbacks,
  };
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 7: Performance Optimization with Batching
// ═══════════════════════════════════════════════════════════════════════════════

export function batchedProcessingExample() {
  console.log('=== Batched Processing Example ===');

  // Create a batch processor for high-frequency events
  class BatchProcessor {
    private batch: any[] = [];
    private timer: number | null = null;
    private batchSize = 100;
    private batchTimeout = 50; // ms

    constructor(private emitter: EventEmitter<DomainEvents>) {
      // Subscribe to high-frequency event
      this.emitter.on('entity:moved', this.addToBatch.bind(this));
    }

    private addToBatch(payload: any) {
      this.batch.push(payload);

      if (this.batch.length >= this.batchSize) {
        this.flush();
      } else if (this.timer === null) {
        this.timer = window.setTimeout(() => this.flush(), this.batchTimeout);
      }
    }

    private flush() {
      if (this.timer !== null) {
        clearTimeout(this.timer);
        this.timer = null;
      }

      if (this.batch.length === 0) return;

      // Process batch
      console.log(`Processing batch of ${this.batch.length} moves`);

      // Emit batched event
      this.emitter.emit('entity:moved:batch', {
        moves: this.batch,
        count: this.batch.length,
      });

      this.batch = [];
    }

    dispose() {
      if (this.timer !== null) {
        clearTimeout(this.timer);
      }
      this.flush();
    }
  }

  // Create and use batch processor
  const processor = new BatchProcessor(globalEvents);

  // Simulate many move events
  for (let i = 0; i < 250; i++) {
    globalEvents.emit('entity:moved', {
      id: `entity-${i % 10}`,
      position: { x: i * 10, y: i * 10 },
    });
  }

  // Cleanup
  return () => processor.dispose();
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXAMPLE 8: Event DevTools Integration
// ═══════════════════════════════════════════════════════════════════════════════

import React from 'react';
import { createRoot } from 'react-dom/client';

export function setupDevTools() {
  // Create tracker
  const tracker = new EventTracker();

  // Wrap global events
  const trackedEvents = new TrackedEventEmitter(globalEvents, tracker, 'App');

  // Create DevTools component
  const DevTools = () => {
    const metrics = tracker.getMetrics();

    return (
      <div style={{
        position: 'fixed',
        bottom: 0,
        right: 0,
        backgroundColor: '#1e1e1e',
        color: '#fff',
        padding: '10px',
        fontSize: '12px',
        fontFamily: 'monospace',
        maxHeight: '300px',
        overflow: 'auto',
      }}>
        <h4>Event DevTools</h4>
        <div>Total Events: {metrics.totalEvents}</div>
        <div>By Direction:</div>
        <ul>
          <li>Emitted: {metrics.eventsByDirection.emitted}</li>
          <li>Received: {metrics.eventsByDirection.received}</li>
        </ul>
        <div>By Type:</div>
        <ul>
          {Object.entries(metrics.eventsByType).map(([type, count]) => (
            <li key={type}>{type}: {count}</li>
          ))}
        </ul>
        <button onClick={() => tracker.clear()}>Clear</button>
        <button onClick={() => {
          const logs = tracker.export();
          console.log('Exported logs:', logs);
        }}>
          Export to Console
        </button>
      </div>
    );
  };

  // Mount DevTools
  const container = document.getElementById('event-devtools');
  if (container) {
    const root = createRoot(container);
    root.render(<DevTools />);
  }

  return { tracker, trackedEvents };
}

// ═══════════════════════════════════════════════════════════════════════════════
// USAGE SUMMARY
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Quick Integration Guide:
 *
 * 1. Basic Events:
 *    import { globalEvents } from '@archflow/sdk';
 *    globalEvents.on('entity:created', (payload) => console.log(payload));
 *    globalEvents.emit('entity:created', { id: '1', type: 'rect' });
 *
 * 2. Zustand Integration:
 *    import { createEventMiddleware } from '@archflow/sdk';
 *    const middleware = createEventMiddleware(globalEvents, { mappings: {...} });
 *
 * 3. React Hooks:
 *    import { useEntityCreated, useDragEvents } from '@archflow/sdk';
 *    const entities = useEntityCreated(globalEvents);
 *    const { isDragging } = useDragEvents(globalEvents, entityId);
 *
 * 4. Web Workers:
 *    import { EventWorkerPool } from '@archflow/sdk';
 *    const pool = new EventWorkerPool();
 *    const result = await pool.process('task-type', data);
 *
 * 5. Debugging:
 *    import { EventTracker, TrackedEventEmitter } from '@archflow/sdk';
 *    const tracker = new EventTracker();
 *    const tracked = new TrackedEventEmitter(globalEvents, tracker);
 *    console.log(tracker.getMetrics());
 */
