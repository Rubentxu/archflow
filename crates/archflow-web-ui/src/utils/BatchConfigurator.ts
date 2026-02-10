/**
 * BatchConfigurator - Utility for batch behavior registration
 *
 * This utility provides efficient batch registration of behaviors to minimize
 * WASM bridge overhead, as specified in the Developer Manual.
 *
 * Developer Manual: "Batch Processing y Performance"
 */

import { ArchFlowService, BehaviorDefinition } from '../services/ArchFlowService';

// ============================================================================
// BATCH CONFIGURATOR
// ============================================================================

/**
 * BatchConfigurator - Configures behaviors in batches for performance
 *
 * Behaviors are queued and registered in a single WASM bridge call to
 * minimize overhead. Auto-flushes when max batch size is reached.
 *
 * @example
 * ```typescript
 * const configurator = new BatchConfigurator(service);
 * configurator.setMaxBatchSize(50);
 *
 * entities.forEach(entity => {
 *   configurator.enqueue({
 *     id: `behavior-${entity.id}`,
 *     components: [...]
 *   });
 * });
 *
 * configurator.flush();
 * ```
 */
export class BatchConfigurator {
  private behaviorQueue: BehaviorDefinition[] = [];
  private maxBatchSize = 100;
  private service: ArchFlowService;

  constructor(service: ArchFlowService) {
    this.service = service;
  }

  /**
   * Add behavior to batch queue
   *
   * Automatically flushes when max batch size is reached.
   *
   * @param behavior - Behavior definition to queue
   */
  enqueue(behavior: BehaviorDefinition): void {
    this.behaviorQueue.push(behavior);

    // Auto-flush when max batch size reached
    if (this.behaviorQueue.length >= this.maxBatchSize) {
      this.flush();
    }
  }

  /**
   * Flush all queued behaviors to WASM
   *
   * Registers all queued behaviors in a single WASM bridge call.
   */
  flush(): void {
    if (this.behaviorQueue.length === 0) return;

    try {
      this.service.registerBehaviors(this.behaviorQueue);
      this.behaviorQueue = [];
    } catch (error) {
      console.error('Batch registration failed:', error);
      throw error;
    }
  }

  /**
   * Set maximum batch size
   *
   * @param size - Maximum batch size (1-1000)
   */
  setMaxBatchSize(size: number): void {
    this.maxBatchSize = Math.max(1, Math.min(1000, size));
  }

  /**
   * Get current queue size
   */
  getQueueSize(): number {
    return this.behaviorQueue.length;
  }

  /**
   * Check if queue is empty
   */
  isEmpty(): boolean {
    return this.behaviorQueue.length === 0;
  }

  /**
   * Clear queue without registering
   */
  clear(): void {
    this.behaviorQueue = [];
  }

  /**
   * Get queued behaviors
   */
  getQueuedBehaviors(): BehaviorDefinition[] {
    return [...this.behaviorQueue];
  }
}

// ============================================================================
// REACTIVE BATCH CONFIGURATOR
// ============================================================================

/**
 * ReactiveBatchConfigurator - RxJS-enabled batch configurator
 *
 * Provides reactive streams for batch processing status.
 */
export class ReactiveBatchConfigurator extends BatchConfigurator {
  private progressSubject = new BehaviorSubject<number>(0);
  private completeSubject = new Subject<void>();
  private errorSubject = new Subject<Error>();

  readonly progress$ = this.progressSubject.asObservable();
  readonly complete$ = this.completeSubject.asObservable();
  readonly error$ = this.errorSubject.asObservable();

  /**
   * Flush with progress updates
   */
  async flushAsync(): Promise<void> {
    const total = this.getQueueSize();

    try {
      for (let i = 0; i < total; i++) {
        // Register in smaller chunks for progress updates
        const chunk = this.behaviorQueue.splice(0, 10);
        await this.service.registerBehaviors(chunk);

        this.progressSubject.next((total - this.behaviorQueue.length) / total);
      }

      this.completeSubject.next();
      this.progressSubject.next(1);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      this.errorSubject.next(err);
      throw err;
    }
  }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

/**
 * Create batch configurator for service
 */
export function createBatchConfigurator(
  service: ArchFlowService,
  maxSize = 100
): BatchConfigurator {
  const configurator = new BatchConfigurator(service);
  configurator.setMaxBatchSize(maxSize);
  return configurator;
}

// ============================================================================
// BATCH PROCESSING HELPERS
// ============================================================================

/**
 * Process behaviors in automatic batches
 *
 * @param service - ArchFlow service
 * @param behaviors - Behaviors to process
 * @param batchSize - Batch size
 */
export async function processBehaviorsInBatches(
  service: ArchFlowService,
  behaviors: BehaviorDefinition[],
  batchSize = 100
): Promise<void> {
  const configurator = new BatchConfigurator(service);
  configurator.setMaxBatchSize(batchSize);

  for (const behavior of behaviors) {
    configurator.enqueue(behavior);
  }

  configurator.flush();
}

/**
 * Create batch processing pipeline
 */
export function createBatchPipeline(
  service: ArchFlowService,
  options: {
    batchSize?: number;
    progressCallback?: (progress: number) => void;
    errorCallback?: (error: Error) => void;
  } = {}
) {
  const {
    batchSize = 100,
    progressCallback,
    errorCallback
  } = options;

  return {
    async process(behaviors: BehaviorDefinition[]): Promise<void> {
      try {
        const configurator = new BatchConfigurator(service);
        configurator.setMaxBatchSize(batchSize);

        const total = behaviors.length;
        let processed = 0;

        for (const behavior of behaviors) {
          configurator.enqueue(behavior);
          processed++;

          if (progressCallback && configurator.isEmpty()) {
            progressCallback(processed / total);
          }
        }

        configurator.flush();

        if (progressCallback) {
          progressCallback(1);
        }
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        if (errorCallback) {
          errorCallback(err);
        } else {
          throw err;
        }
      }
    }
  };
}

// Re-add imports for ReactiveBatchConfigurator
import { BehaviorSubject, Subject } from 'rxjs';
