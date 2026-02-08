/**
 * Event Tracker and Debugging System for ArchFlow
 *
 * This module provides comprehensive event tracking, logging, and debugging
 * capabilities for the ArchFlow event system. It captures all events flowing
 * through the system and provides tools for analysis and visualization.
 *
 * Architecture Reference: EPIC-WEB-013 HU-007
 *
 * @example
 * ```typescript
 * import { EventTracker, TrackedEventEmitter, EventDevTools } from './eventTracker';
 *
 * // Use tracked emitter that automatically logs all events
 * const emitter = new TrackedEventEmitter(globalEvents, 'MainSystem');
 *
 * // Get event logs
 * const logs = emitter.getTracker().getLogs();
 *
 * // Get metrics
 * const metrics = emitter.getTracker().getMetrics();
 * ```
 */

import type { EventEmitter, DomainEvents, Subscription } from "./EventEmitter";

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT LOG ENTRY
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Direction of event flow
 */
export type EventDirection =
  | "wasm-to-js"
  | "js-to-wasm"
  | "js-to-js"
  | "internal";

/**
 * Log entry for a single event
 */
export interface EventLogEntry {
  /** Unique identifier for this log entry */
  id: string;

  /** Timestamp when event was logged (performance.now()) */
  timestamp: number;

  /** Direction of event flow */
  direction: EventDirection;

  /** Event type/name */
  eventType: string;

  /** Event payload (JSON-serializable data) */
  payload: unknown;

  /** Source system/component that emitted the event */
  source: string;

  /** Optional metadata */
  metadata?: {
    /** Processing time in ms */
    processingTime?: number;
    /** Worker thread ID (if processed in worker) */
    workerId?: number;
    /** Batch size (if event was batched) */
    batchSize?: number;
  };
}

/**
 * Event metrics snapshot
 */
export interface EventMetricsSnapshot {
  /** Total events logged */
  totalEvents: number;

  /** Events by type */
  byType: Record<string, number>;

  /** Events by direction */
  byDirection: Partial<Record<EventDirection, number>>;

  /** Event frequency (events per second over last second) */
  frequency: number;

  /** Errors encountered */
  errorCount: number;

  /** Throttled events (dropped due to rate limiting) */
  throttledCount: number;

  /** Timestamp of last event */
  lastEventTime: number;
}

/**
 * Event filter options
 */
export interface EventFilterOptions {
  /** Filter by event type pattern (wildcards supported) */
  type?: string;

  /** Filter by direction */
  direction?: EventDirection;

  /** Filter by source */
  source?: string;

  /** Filter by time range */
  startTime?: number;
  endTime?: number;
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT TRACKER
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Event tracker for logging and monitoring
 *
 * Provides comprehensive event tracking with circular buffer
 * for memory efficiency and real-time metrics.
 */
export class EventTracker {
  private logs: EventLogEntry[] = [];
  private maxLogs: number;
  private listeners: Set<(logs: EventLogEntry[]) => void> = new Set();

  // Metrics tracking
  private totalEvents: number = 0;
  private errorCount: number = 0;
  private throttledCount: number = 0;

  // Frequency calculation
  private frequencyWindow: number[] = [];
  private readonly frequencyWindowMs = 1000; // 1 second window

  constructor(maxLogs = 1000) {
    this.maxLogs = maxLogs;
  }

  /**
   * Log an event
   */
  log(
    direction: EventDirection,
    eventType: string,
    payload: unknown,
    source: string,
    metadata?: EventLogEntry["metadata"],
  ): void {
    const entry: EventLogEntry = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      timestamp: performance.now(),
      direction,
      eventType,
      payload,
      source,
      metadata,
    };

    // Add to circular buffer
    if (this.logs.length >= this.maxLogs) {
      this.logs.shift(); // Remove oldest
    }
    this.logs.push(entry);

    // Update metrics
    this.totalEvents++;
    this.updateFrequency(entry.timestamp);

    // Notify listeners
    this.notifyListeners();
  }

  /**
   * Get all logs with optional filtering
   */
  getLogs(filter?: EventFilterOptions): EventLogEntry[] {
    let logs = [...this.logs];

    if (filter) {
      if (filter.type) {
        const regex = new RegExp(filter.type.replace("*", ".*"));
        logs = logs.filter((log) => regex.test(log.eventType));
      }

      if (filter.direction) {
        logs = logs.filter((log) => log.direction === filter.direction);
      }

      if (filter.source) {
        logs = logs.filter((log) => log.source === filter.source);
      }

      if (filter.startTime !== undefined) {
        logs = logs.filter((log) => log.timestamp >= filter.startTime!);
      }

      if (filter.endTime !== undefined) {
        logs = logs.filter((log) => log.timestamp <= filter.endTime!);
      }
    }

    return logs;
  }

  /**
   * Get current metrics snapshot
   */
  getMetrics(): EventMetricsSnapshot {
    const now = performance.now();
    const recentLogs = this.logs.filter(
      (log) => now - log.timestamp <= this.frequencyWindowMs,
    );

    // Calculate by type
    const byType: Record<string, number> = {};
    for (const log of recentLogs) {
      byType[log.eventType] = (byType[log.eventType] || 0) + 1;
    }

    // Calculate by direction
    const byDirection: Partial<Record<EventDirection, number>> = {};
    for (const log of recentLogs) {
      byDirection[log.direction] = (byDirection[log.direction] || 0) + 1;
    }

    // Calculate frequency
    const frequency = this.frequencyWindow.length;

    return {
      totalEvents: this.totalEvents,
      byType,
      byDirection,
      frequency,
      errorCount: this.errorCount,
      throttledCount: this.throttledCount,
      lastEventTime:
        this.logs.length > 0 ? this.logs[this.logs.length - 1].timestamp : 0,
    };
  }

  /**
   * Export logs to JSON for saving/sharing
   */
  export(): string {
    return JSON.stringify(
      {
        logs: this.logs,
        metrics: this.getMetrics(),
        exportedAt: new Date().toISOString(),
      },
      null,
      2,
    );
  }

  /**
   * Import logs from JSON
   */
  import(jsonData: string): void {
    try {
      const data = JSON.parse(jsonData);
      this.logs = data.logs || [];
      this.totalEvents = data.metrics?.totalEvents || 0;
      this.notifyListeners();
    } catch (error) {
      console.error("[EventTracker] Failed to import logs:", error);
    }
  }

  /**
   * Clear all logs
   */
  clear(): void {
    this.logs = [];
    this.totalEvents = 0;
    this.errorCount = 0;
    this.throttledCount = 0;
    this.frequencyWindow = [];
    this.notifyListeners();
  }

  /**
   * Subscribe to log updates
   */
  subscribe(callback: (logs: EventLogEntry[]) => void): () => void {
    this.listeners.add(callback);

    // Return unsubscribe function
    return () => {
      this.listeners.delete(callback);
    };
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // PRIVATE METHODS
  // ═══════════════════════════════════════════════════════════════════════════════

  private updateFrequency(timestamp: number): void {
    this.frequencyWindow.push(timestamp);

    // Keep only events within the time window
    const cutoff = timestamp - this.frequencyWindowMs;
    this.frequencyWindow = this.frequencyWindow.filter((t) => t > cutoff);
  }

  private notifyListeners(): void {
    for (const listener of this.listeners) {
      try {
        listener(this.logs);
      } catch (error) {
        console.error("[EventTracker] Error in log listener:", error);
      }
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRACKED EVENT EMITTER
// ═════════════════════════════════════════════════════════════════════════════

/**
 * Event emitter wrapper that automatically tracks all events
 *
 * This wraps an existing EventEmitter and logs every emit operation.
 */
export class TrackedEventEmitter implements EventEmitter<DomainEvents> {
  private inner: EventEmitter<DomainEvents>;
  private tracker: EventTracker;
  private source: string;

  constructor(
    inner: EventEmitter<DomainEvents>,
    source: string,
    tracker?: EventTracker,
  ) {
    this.inner = inner;
    this.source = source;
    this.tracker = tracker || new EventTracker();
  }

  on<K extends keyof DomainEvents>(
    event: K,
    callback: (payload: DomainEvents[K]) => void,
    options?: { throttle?: number; debounce?: number },
  ): Subscription {
    return this.inner.on(event, callback, options);
  }

  onWildcard(
    pattern: string,
    callback: (event: string, payload: unknown) => void,
    options?: { throttle?: number; debounce?: number },
  ): Subscription {
    return this.inner.onWildcard(pattern, callback, options);
  }

  once<K extends keyof DomainEvents>(
    event: K,
    callback: (payload: DomainEvents[K]) => void,
  ): Subscription {
    return this.inner.once(event, callback);
  }

  emit<K extends keyof DomainEvents>(event: K, payload: DomainEvents[K]): void {
    const startTime = performance.now();

    // Log before emitting
    this.tracker.log("js-to-js", event as string, payload, this.source);

    // Emit the event
    this.inner.emit(event, payload);

    // Log processing time
    const elapsed = performance.now() - startTime;
    if (elapsed > 10) {
      // Only log if significant
      this.tracker.log(
        "internal",
        "performance-check",
        {
          event: event as string,
          elapsed,
          threshold: 10,
        },
        this.source,
        { processingTime: elapsed },
      );
    }
  }

  off<K extends keyof DomainEvents>(
    event: K,
    callback: (payload: DomainEvents[K]) => void,
  ): void {
    this.inner.off(event, callback);
  }

  removeAllListeners<K extends keyof DomainEvents>(event?: K): void {
    this.inner.removeAllListeners(event);
  }

  clear(): void {
    this.inner.clear();
    this.tracker.clear();
  }

  getMetrics(): EventMetricsSnapshot {
    return this.tracker.getMetrics();
  }

  getTracker(): EventTracker {
    return this.tracker;
  }

  getListenerCount<K extends keyof DomainEvents>(event: K): number {
    return this.inner.getListenerCount(event);
  }

  hasListeners<K extends keyof DomainEvents>(event: K): boolean {
    return this.inner.hasListeners(event);
  }
}

// ═════════════════════════════════════════════════════════════════════════════
// REACT DEVTOOLS COMPONENT
// ═════════════════════════════════════════════════════════════════════════════

/**
 * EventDevTools - React component for event debugging UI
 *
 * This component provides a real-time visualization of events flowing
 * through the system with filtering and metrics.
 */
export function EventDevTools() {
  const [logs, setLogs] = React.useState<EventLogEntry[]>([]);
  const [filter, setFilter] = React.useState<EventFilterOptions>({});
  const [metrics, setMetrics] = React.useState<EventMetricsSnapshot | null>(
    null,
  );
  const [tracker, setTracker] = React.useState<EventTracker | null>(null);
  const [isExpanded, setIsExpanded] = React.useState(true);

  // Subscribe to a global tracker
  React.useEffect(() => {
    // This would connect to the global tracked emitter
    // For now, we'll assume it's passed through context or props
    return () => {};
  }, []);

  // Auto-refresh metrics
  React.useEffect(() => {
    if (!tracker) return;

    const interval = setInterval(() => {
      setMetrics(tracker.getMetrics());
    }, 1000);

    return () => clearInterval(interval);
  }, [tracker]);

  const filteredLogs = React.useMemo(() => {
    if (!tracker) return [];
    return tracker.getLogs(filter);
  }, [tracker, filter]);

  const clearLogs = () => {
    tracker?.clear();
  };

  const exportLogs = () => {
    if (tracker) {
      const json = tracker.export();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `archflow-events-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  if (!isExpanded) {
    return (
      <button
        onClick={() => setIsExpanded(true)}
        className="fixed bottom-4 right-4 bg-blue-600 text-white px-4 py-2 rounded-lg shadow-lg"
      >
        Events ({filteredLogs.length})
      </button>
    );
  }

  return (
    <div className="fixed top-4 right-4 w-96 bg-white rounded-lg shadow-xl border border-gray-200 max-h-[600px] flex flex-col">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-200 flex justify-between items-center">
        <h3 className="font-semibold text-gray-800">Event DevTools</h3>
        <button
          onClick={() => setIsExpanded(false)}
          className="text-gray-400 hover:text-gray-600"
        >
          ✕
        </button>
      </div>

      {/* Metrics */}
      {metrics && (
        <div className="px-4 py-2 border-b border-gray-100 bg-gray-50 text-xs">
          <div className="grid grid-cols-3 gap-2">
            <div>
              <span className="text-gray-500">Total: </span>
              <span className="font-mono font-semibold">
                {metrics.totalEvents}
              </span>
            </div>
            <div>
              <span className="text-gray-500">Freq: </span>
              <span className="font-mono font-semibold">
                {metrics.frequency.toFixed(1)}/s
              </span>
            </div>
            <div>
              <span className="text-gray-500">Errors: </span>
              <span className="font-mono font-semibold text-red-600">
                {metrics.errorCount}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="px-4 py-2 border-b border-gray-100">
        <div className="flex gap-2 mb-2">
          <select
            className="flex-1 text-xs border rounded px-2 py-1"
            value={filter.direction}
            onChange={(e) =>
              setFilter({
                ...filter,
                direction: e.target.value as EventDirection,
              })
            }
          >
            <option value="">All Directions</option>
            <option value="wasm-to-js">WASM → JS</option>
            <option value="js-to-wasm">JS → WASM</option>
            <option value="js-to-js">JS → JS</option>
          </select>
          <input
            type="text"
            className="flex-1 text-xs border rounded px-2 py-1"
            placeholder="Filter by type..."
            onChange={(e) => setFilter({ ...filter, type: e.target.value })}
          />
        </div>
      </div>

      {/* Event List */}
      <div className="flex-1 overflow-y-auto px-2">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-white">
            <tr className="text-left text-gray-500">
              <th className="px-2 py-1">Time</th>
              <th className="px-2 py-1">Direction</th>
              <th className="px-2 py-1">Event</th>
              <th className="px-2 py-1">Source</th>
              <th className="px-2 py-1">Payload</th>
            </tr>
          </thead>
          <tbody>
            {filteredLogs.slice(-100).map((log, i) => (
              <tr
                key={log.id}
                className="border-t border-gray-50 hover:bg-gray-50"
              >
                <td className="px-2 py-1 font-mono">
                  {(log.timestamp % 1000).toFixed(0)}
                </td>
                <td className="px-2 py-1">
                  <span
                    className={`px-1.5 py-0.5 rounded text-xs ${
                      log.direction === "wasm-to-js"
                        ? "bg-blue-100 text-blue-700"
                        : log.direction === "js-to-wasm"
                          ? "bg-green-100 text-green-700"
                          : "bg-gray-100 text-gray-700"
                    }`}
                  >
                    {log.direction}
                  </span>
                </td>
                <td className="px-2 py-1 font-mono">{log.eventType}</td>
                <td
                  className="px-2 py-1 truncate max-w-[100px]"
                  title={log.source}
                >
                  {log.source}
                </td>
                <td
                  className="px-2 py-1 truncate max-w-[150px]"
                  title={JSON.stringify(log.payload)}
                >
                  {JSON.stringify(log.payload).slice(0, 50)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Footer */}
      <div className="px-4 py-2 border-t border-gray-100 flex justify-between items-center">
        <span className="text-xs text-gray-500">
          {filteredLogs.length} events shown
        </span>
        <div className="flex gap-2">
          <button
            onClick={clearLogs}
            className="text-xs bg-red-50 text-red-600 px-3 py-1 rounded hover:bg-red-100"
          >
            Clear
          </button>
          <button
            onClick={exportLogs}
            className="text-xs bg-blue-50 text-blue-600 px-3 py-1 rounded hover:bg-blue-100"
          >
            Export
          </button>
        </div>
      </div>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL EVENT TRACKER
// ═══════════════════════════════════════════════════════════════════════════════

let globalEventTracker: EventTracker | null = null;

/**
 * Get the global event tracker instance
 *
 * Creates the tracker on first call and reuses it for subsequent calls.
 */
export function getGlobalEventTracker(): EventTracker {
  if (!globalEventTracker) {
    globalEventTracker = new EventTracker(1000);
  }
  return globalEventTracker;
}

/**
 * Reset the global event tracker
 *
 * Should be called when shutting down the application.
 */
export function resetGlobalEventTracker(): void {
  if (globalEventTracker) {
    globalEventTracker.clear();
    globalEventTracker = null;
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REACT HOOK FOR DEVTOOLS
// ═══════════════════════════════════════════════════════════════════════════════

import React from "react";

/**
 * React hook for accessing event tracker in components
 *
 * @returns Event tracker instance or null if not initialized
 */
export function useEventTracker(): EventTracker | null {
  const [tracker, setTracker] = React.useState<EventTracker | null>(null);

  React.useEffect(() => {
    // Initialize with global tracker
    setTracker(getGlobalEventTracker());

    return () => {
      // Cleanup on unmount
      // Don't reset the global tracker, just stop accessing it
    };
  }, []);

  return tracker;
}

/**
 * React hook for event metrics with auto-refresh
 *
 * @returns Current event metrics or null
 */
export function useEventMetrics(): EventMetricsSnapshot | null {
  const tracker = useEventTracker();
  const [metrics, setMetrics] = React.useState<EventMetricsSnapshot | null>(
    null,
  );

  React.useEffect(() => {
    if (!tracker) return;

    // Update metrics every second
    const interval = setInterval(() => {
      setMetrics(tracker.getMetrics());
    }, 1000);

    return () => clearInterval(interval);
  }, [tracker]);

  return metrics;
}
