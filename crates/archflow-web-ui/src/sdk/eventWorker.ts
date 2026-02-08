/**
 * Event Worker System for Heavy Operations
 *
 * This module provides Web Worker support for offloading heavy event
 * processing to background threads, keeping the main UI thread responsive.
 *
 * Architecture Reference: EPIC-WEB-013 HU-005
 *
 * @example
 * ```typescript
 * import { EventWorkerPool, createEventWorker } from './eventWorker';
 *
 * const pool = new EventWorkerPool(4); // 4 workers
 *
 * // Process heavy event in worker
 * const result = await pool.process('heavy-calculation', {
 *   data: new Float32Array(100000),
 *   operation: 'fft'
 * });
 * ```
 */

/**
 * Message types sent between main thread and workers
 */
export interface WorkerMessage<T = unknown> {
  id: number;
  type: string;
  payload: T;
  error?: string;
}

/**
 * Configuration for event worker pool
 */
export interface WorkerPoolConfig {
  /** Number of workers to create (default: navigator.hardwareConcurrency || 4) */
  workerCount?: number;
  /** Timeout for worker operations in ms (default: 5000) */
  timeout?: number;
  /** Enable debug logging */
  debug?: boolean;
}

/**
 * Event worker that processes events in a Web Worker
 *
 * This worker runs in a background thread and can process heavy
 * computations without blocking the UI.
 */
export class EventWorker {
  private worker: Worker | null = null;
  private pending = new Map<number, (result: unknown) => void>();
  private messageId = 0;
  private debug: boolean;

  constructor(debug = false) {
    this.debug = debug;
  }

  /**
   * Initialize the worker (creates worker instance)
   */
  private initializeWorker(): void {
    if (this.worker) return; // Already initialized

    try {
      // Create worker from inline blob to avoid external file dependencies
      const workerCode = `
        self.onmessage = function(e) {
          const { id, type, payload } = e.data;

          try {
            let result;

            switch(type) {
              case 'heavy-calculation':
                result = heavyCalculation(payload);
                break;
              case 'batch-process':
                result = batchProcessing(payload);
                break;
              case 'complex-operation':
                result = complexOperation(payload);
                break;
              default:
                throw new Error(\`Unknown worker operation: \${type}\`);
            }

            self.postMessage({
              id,
              type,
              result,
              error: null
            });

          } catch (error) {
            self.postMessage({
              id,
              type,
              result: null,
              error: error instanceof Error ? error.message : String(error)
            });
          }
        };

        function heavyCalculation(data) {
          if (!data || !data.data) return 0;

          const startTime = performance.now();
          const arr = data.data;
          let processed = 0;

          // Simulate heavy computation
          for (let i = 0; i < arr.length; i++) {
            arr[i] = Math.sqrt(arr[i] * 2) + Math.sin(arr[i]);
            processed++;
          }

          const elapsed = performance.now() - startTime;

          return {
            processed,
            elapsed,
            result: arr
          };
        }

        function batchProcessing(data) {
          if (!data || !Array.isArray(data.items)) return 0;

          return data.items.map((item: number) => {
            return item * 2 + Math.sqrt(item);
          });
        }

        function complexOperation(data) {
          // Simulate complex business logic
          const { input } = data;

          // Multi-stage processing
          let result = input;
          result = result.map((x: number) => x * 2);
          result = result.filter((x: number) => x > 100);
          result = result.sort((a: number, b: number) => a - b);

          return {
            count: result.length,
            sum: result.reduce((a: number, b: number) => a + b, 0),
            average: result.length > 0 ? result.reduce((a: number, b: number) => a + b, 0) / result.length : 0
          };
        }
      `;

      const blob = new Blob([workerCode], { type: 'application/javascript' });
      const workerUrl = URL.createObjectURL(blob);

      this.worker = new Worker(workerUrl);

      // Set up message handler
      this.worker.onmessage = (e: MessageEvent<WorkerMessage>) => {
        this.handleMessage(e.data);
      };

      this.worker.onerror = (error: ErrorEvent) => {
        console.error('[EventWorker] Worker error:', error);
        // Reject all pending promises
        for (const [id, reject] of this.pending) {
          try {
            (reject as (error: Error) => void)(error);
          } catch (e) {
            // Ignore errors in error handlers
          }
        }
        this.pending.clear();
      };

    } catch (error) {
      console.error('[EventWorker] Failed to create worker:', error);
    }
  }

  /**
   * Handle message from worker
   */
  private handleMessage(message: WorkerMessage): void {
    const { id, result, error } = message;
    const deferred = this.pending.get(id);

    if (deferred) {
      this.pending.delete(id);

      if (error) {
        try {
          (deferred as (error: Error) => void)(new Error(error));
        } catch (e) {
          // Ignore errors in error handlers
        }
      } else {
        try {
          (deferred as (value: unknown) => void)(result);
        } catch (e) {
          // Ignore errors in success handlers
        }
      }
    }
  }

  /**
   * Process an event in the worker
   *
   * @param type - The type of operation to perform
   * @param payload - Data to process
   * @param timeout - Optional timeout in milliseconds
   * @returns Promise with processing result
   */
  async process<T = unknown, R = unknown>(
    type: string,
    payload: T,
    timeout?: number
  ): Promise<R> {
    // Fallback to main thread if Workers not available
    if (typeof Worker === 'undefined') {
      if (this.debug) {
        console.warn('[EventWorker] Workers not available, using main thread');
      }
      return this.processOnMainThread<T, R>(type, payload);
    }

    this.initializeWorker();

    if (!this.worker) {
      return this.processOnMainThread<T, R>(type, payload);
    }

    const id = ++this.messageId;

    return new Promise((resolve, reject) => {
      // Set timeout
      const timeoutId = timeout
        ? setTimeout(() => {
            this.pending.delete(id);
            reject(new Error(`Worker timeout after ${timeout}ms`));
          }, timeout)
        : undefined;

      // Store resolver for when worker responds
      this.pending.set(id, (result: unknown) => {
        if (timeoutId) clearTimeout(timeoutId);
        (resolve as (value: unknown) => void)(result as R);
      });

      // Send message to worker
      this.worker!.postMessage({ id, type, payload });
    });
  }

  /**
   * Process operation on main thread (fallback)
   */
  private processOnMainThread<T = unknown, R = unknown>(type: string, payload: T): R {
    const startTime = performance.now();

    switch (type) {
      case 'heavy-calculation':
        const data = (payload as { data: Float32Array }).data;
        if (data) {
          for (let i = 0; i < data.length; i++) {
            data[i] = Math.sqrt(data[i] * 2) + Math.sin(data[i]);
          }
        }
        return { processed: data?.length || 0, elapsed: performance.now() - startTime } as R;

      case 'batch-process':
        const items = (payload as { items: number[] }).items || [];
        return items.map((item: number) => item * 2 + Math.sqrt(item)) as R;

      case 'complex-operation':
        const input = (payload as { input: number[] }).input || [];
        const result = input.map((x: number) => x * 2)
          .filter((x: number) => x > 100)
          .sort((a: number, b: number) => a - b);
        return {
          count: result.length,
          sum: result.reduce((a, b) => a + b, 0),
          average: result.length > 0 ? result.reduce((a, b) => a + b, 0) / result.length : 0
        } as R;

      default:
        throw new Error(`Unknown operation type: ${type}`);
    }
  }

  /**
   * Terminate the worker and free resources
   */
  terminate(): void {
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }
    this.pending.clear();
  }
}

/**
 * Pool of event workers for concurrent processing
 *
 * Distributes events across multiple workers to maximize throughput
 * while preventing any single worker from being overwhelmed.
 */
export class EventWorkerPool {
  private workers: EventWorker[] = [];
  private queue: Array<() => void> = [];
  private active = 0;
  private debug: boolean;

  constructor(config: WorkerPoolConfig = {}) {
    this.debug = config.debug || false;

    const workerCount = config.workerCount || (typeof navigator !== 'undefined'
      ? (navigator.hardwareConcurrency || 4)
      : 4
    );

    // Create workers
    for (let i = 0; i < workerCount; i++) {
      this.workers.push(new EventWorker(this.debug));
    }

    if (this.debug) {
      console.log(`[EventWorkerPool] Initialized with ${workerCount} workers`);
    }
  }

  /**
   * Process an event using an available worker
   *
   * Automatically selects the worker with the least load.
   *
   * @param type - Operation type
   * @param payload - Data to process
   * @param timeout - Optional timeout
   * @returns Promise with processing result
   */
  async process<T = unknown, R = unknown>(
    type: string,
    payload: T,
    timeout?: number
  ): Promise<R> {
    // Find available worker (least busy)
    let workerIndex = 0;
    let minLoad = Infinity;

    for (let i = 0; i < this.workers.length; i++) {
      const load = this.workers[i].pending.size;
      if (load < minLoad) {
        minLoad = load;
        workerIndex = i;
      }
    }

    const worker = this.workers[workerIndex];

    if (this.debug) {
      console.log(`[EventWorkerPool] Using worker ${workerIndex} (load: ${minLoad})`);
    }

    return worker.process<T, R>(type, payload, timeout);
  }

  /**
   * Get statistics about the pool
   */
  getStats() {
    return {
      totalWorkers: this.workers.length,
      activeJobs: this.active,
      queueLength: this.queue.length,
      workerLoads: this.workers.map(w => w.pending.size),
    };
  }

  /**
   * Terminate all workers and free resources
   */
  terminate(): void {
    for (const worker of this.workers) {
      worker.terminate();
    }
    this.workers = [];
    this.queue = [];
    this.active = 0;
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WORKER OPERATION DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Heavy calculation operation
 *
 * Processes large arrays of numerical data with square root and sine operations.
 *
 * @param data - Float32Array of numerical data
 * @returns Processing result with count and elapsed time
 */
export interface HeavyCalculationPayload {
  data: Float32Array;
  operation?: 'fft' | 'filter' | 'transform';
}

export interface HeavyCalculationResult {
  processed: number;
  elapsed: number;
  result: Float32Array;
}

/**
 * Batch processing operation
 *
 * Processes multiple items in parallel.
 */
export interface BatchProcessingPayload {
  items: Array<number | string>;
  operation: 'double' | 'square' | 'transform';
}

/**
 * Complex business logic operation
 *
 * Multi-stage processing with filtering and sorting.
 */
export interface ComplexOperationPayload {
  input: Array<number>;
  threshold?: number;
  sort?: 'asc' | 'desc';
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Create a single event worker with debug logging
 */
export function createEventWorker(debug = false): EventWorker {
  return new EventWorker(debug);
}

/**
 * Create an event worker pool with configuration
 */
export function createEventWorkerPool(config?: WorkerPoolConfig): EventWorkerPool {
  return new EventWorkerPool(config);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL INSTANCE (lazy initialized)
// ═══════════════════════════════════════════════════════════════════════════════

let globalPool: EventWorkerPool | null = null;

/**
 * Get the global event worker pool
 *
 * Creates the pool on first call and reuses it for subsequent calls.
 *
 * @param config - Optional pool configuration
 * @returns The global worker pool instance
 */
export function getGlobalWorkerPool(config?: WorkerPoolConfig): EventWorkerPool {
  if (!globalPool) {
    globalPool = new EventWorkerPool(config);
  }
  return globalPool;
}

/**
 * Terminate the global worker pool
 *
 * Should be called when shutting down the application to free resources.
 */
export function terminateGlobalWorkerPool(): void {
  if (globalPool) {
    globalPool.terminate();
    globalPool = null;
  }
}
