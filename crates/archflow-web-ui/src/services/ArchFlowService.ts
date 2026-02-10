/**
 * ArchFlow Service - Reactive service for ArchFlow WASM bridge
 *
 * This service provides a complete reactive API for managing ArchFlow behaviors
 * and events using RxJS Observables. It acts as an abstraction layer between
 * React components and the WASM bridge.
 *
 * Developer Manual: "Receta 2: React con Servicios Intermedios"
 */

import { BehaviorSubject, Observable, Subject, from, of } from 'rxjs';
import { filter, map, tap, catchError, delay, retryWhen } from 'rxjs/operators';
import { LogicSystemWasm as ArchFlowWASM } from '../sdk';

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/**
 * Behavior definition as specified in JSON
 */
export interface BehaviorDefinition {
  id: string;
  name: string;
  description?: string;
  components: ComponentDefinition[];
}

/**
 * Individual component definition
 */
export interface ComponentDefinition {
  type: string;
  config: Record<string, unknown>;
}

/**
 * ArchFlow event emitted from WASM
 */
export interface ArchFlowEvent {
  type: string;
  entityId: number;
  data: Record<string, unknown>;
  timestamp: number;
}

/**
 * Entity configuration
 */
export interface EntityConfig {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
}

/**
 * ArchFlow configuration
 */
export interface ArchFlowConfig {
  entities: EntityConfig[];
  behaviors: BehaviorDefinition[];
}

// ============================================================================
// ERROR TYPES
// ============================================================================

export class ArchFlowError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: unknown
  ) {
    super(message);
    this.name = 'ArchFlowError';
  }
}

// ============================================================================
// ARCHFLOW SERVICE
// ============================================================================

/**
 * ArchFlowService - Reactive service for ArchFlow WASM bridge
 *
 * Provides:
 * - Observable state (isReady$, events$, errors$)
 * - Batch registration of behaviors
 * - Factories for common behaviors
 * - Cleanup and error handling
 */
export class ArchFlowService {
  private system: ArchFlowWASM | null = null;
  private bridge: any = null;

  // Observables
  private isReadySubject = new BehaviorSubject<boolean>(false);
  private eventsSubject = new Subject<ArchFlowEvent>();
  private errorSubject = new Subject<Error>();

  // Public observables
  readonly isReady$: Observable<boolean> = this.isReadySubject.asObservable();
  readonly events$: Observable<ArchFlowEvent> = this.eventsSubject.asObservable();
  readonly errors$: Observable<Error> = this.errorSubject.asObservable();

  // Batch processing
  private behaviorQueue: BehaviorDefinition[] = [];
  private maxBatchSize = 100;
  private inBatchMode = false;

  /**
   * Initialize the ArchFlow system
   */
  async initialize(): Promise<void> {
    if (this.isReadySubject.value) {
      return;
    }

    try {
      // Load WASM module
      const wasmModule = await import('archflow-logic');
      this.bridge = new wasmModule.WasmBridge();

      // Initialize bridge
      await this.bridge.initialize(800, 600);

      // Initialize graphics (if canvas available)
      const canvas = document.querySelector('canvas');
      if (canvas) {
        await this.bridge.initialize_graphics(canvas);
      }

      // Setup event handling
      this.setupEventHandling();

      this.isReadySubject.next(true);

    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      this.errorSubject.next(err);
      throw new ArchFlowError(
        `Failed to initialize ArchFlow: ${err.message}`,
        'INIT_ERROR',
        err
      );
    }
  }

  /**
   * Setup event handling with callbacks
   */
  private setupEventHandling(): void {
    if (!this.bridge) return;

    // Register event callback
    this.bridge.on_event((eventData: any) => {
      const event: ArchFlowEvent = {
        type: eventData.type || 'Unknown',
        entityId: eventData.entityId || 0,
        data: eventData.data || {},
        timestamp: Date.now()
      };
      this.eventsSubject.next(event);
    });
  }

  /**
   * Register a single behavior from JSON
   */
  registerBehavior(behavior: BehaviorDefinition): void {
    if (!this.bridge) {
      throw new ArchFlowError(
        'ArchFlow not initialized. Call initialize() first.',
        'NOT_INITIALIZED'
      );
    }

    try {
      const json = JSON.stringify(behavior);
      this.bridge.register_behavior(json);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      this.errorSubject.next(err);
      throw new ArchFlowError(
        `Failed to register behavior: ${err.message}`,
        'REGISTRATION_ERROR',
        { behavior, error: err }
      );
    }
  }

  /**
   * Register multiple behaviors in batch (better performance)
   */
  registerBehaviors(behaviors: BehaviorDefinition[]): void {
    if (!this.bridge) {
      throw new ArchFlowError(
        'ArchFlow not initialized. Call initialize() first.',
        'NOT_INITIALIZED'
      );
    }

    try {
      const json = JSON.stringify(behaviors);
      this.bridge.register_behaviors_json(json);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      this.errorSubject.next(err);
      throw new ArchFlowError(
        `Failed to register behaviors: ${err.message}`,
        'BATCH_REGISTRATION_ERROR',
        { behaviors, error: err }
      );
    }
  }

  // ══════════════════════════════════════════════════════════════════════════════
  // BEHAVIOR FACTORY
  // ══════════════════════════════════════════════════════════════════════════════

  /**
   * Create behavior factory with predefined templates
   */
  createBehaviorFactory() {
    return {
      hoverHighlight: (options: { color?: string; opacity?: number } = {}) => {
        return {
          id: `hover-${Date.now()}`,
          name: 'Hover Highlight',
          components: [
            { type: 'sensor-mouse', config: { mode: 'hover' } },
            { type: 'actuator-highlight', config: {
                color: options.color ?? '#ffff00',
                opacity: options.opacity ?? 0.5
              }
            }
          ]
        } as BehaviorDefinition;
      },

      clickSelect: (options: { button?: number; mode?: 'single' | 'multi' | 'toggle' } = {}) => {
        return {
          id: `click-${Date.now()}`,
          name: 'Click Select',
          components: [
            { type: 'sensor-mouse', config: { mode: 'click', button: options.button ?? 0 } },
            { type: 'actuator-select', config: { mode: options.mode ?? 'single' } }
          ]
        } as BehaviorDefinition;
      },

      wasdMovement: (options: { keys?: number[]; speed?: number } = {}) => {
        return {
          id: `wasd-${Date.now()}`,
          name: 'WASD Movement',
          components: [
            { type: 'sensor-keyboard', config: { keys: options.keys ?? [87, 65, 83, 68], modifiers: 0 } },
            { type: 'actuator-move', config: { mode: 'relative', speed: options.speed ?? 5 } }
          ]
        } as BehaviorDefinition;
      },

      draggable: (options: { debounce?: number; speed?: number } = {}) => {
        return {
          id: `draggable-${Date.now()}`,
          name: 'Draggable',
          components: [
            { type: 'sensor-mouse', config: { mode: 'drag', button: 0 } },
            { type: 'controller-debounce', config: { ticks: options.debounce ?? 6 } },
            { type: 'actuator-move', config: { mode: 'follow-cursor', speed: options.speed ?? 5 } }
          ]
        } as BehaviorDefinition;
      },

      deletable: () => {
        return {
          id: `deletable-${Date.now()}`,
          name: 'Deletable',
          components: [
            { type: 'sensor-keyboard', config: { keys: [46], modifiers: 0 } },
            { type: 'actuator-delete', config: {} }
          ]
        } as BehaviorDefinition;
      }
    };
  }

  // ══════════════════════════════════════════════════════════════════════════════
  // TEMPLATE METHODS
  // ══════════════════════════════════════════════════════════════════════════════

  /**
   * Get behavior template as JSON
   */
  getBehaviorTemplate(templateName: string): BehaviorDefinition | null {
    if (!this.bridge) return null;

    try {
      const json = this.bridge.get_behavior_template(templateName);
      return JSON.parse(json);
    } catch {
      return null;
    }
  }

  /**
   * List available behavior templates
   */
  listBehaviorTemplates(): string[] {
    if (!this.bridge) return [];

    const templates = this.bridge.list_behavior_templates();
    return Array.from(templates);
  }

  // ══════════════════════════════════════════════════════════════════════════════
  // BATCH PROCESSING
  // ══════════════════════════════════════════════════════════════════════════════

  /**
   * Begin batch operation
   */
  beginBatch(): void {
    if (!this.bridge) {
      throw new ArchFlowError('Not initialized', 'NOT_INITIALIZED');
    }

    this.inBatchMode = true;
    this.bridge.begin_batch();
  }

  /**
   * Add behavior to batch queue
   */
  addToBatch(behavior: BehaviorDefinition): void {
    if (!this.inBatchMode) {
      throw new ArchFlowError(
        'Not in batch mode. Call beginBatch() first.',
        'NOT_IN_BATCH_MODE'
      );
    }

    this.behaviorQueue.push(behavior);

    // Auto-flush if max batch size reached
    if (this.behaviorQueue.length >= this.maxBatchSize) {
      this.flushBatch();
    }
  }

  /**
   * Flush batch queue
   */
  flushBatch(): void {
    if (this.behaviorQueue.length === 0) return;

    try {
      const json = JSON.stringify(this.behaviorQueue);
      this.bridge.register_behaviors_json(json);
      this.behaviorQueue = [];
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      this.errorSubject.next(err);
      throw new ArchFlowError(
        `Batch registration failed: ${err.message}`,
        'BATCH_ERROR',
        err
      );
    }
  }

  /**
   * End batch operation
   */
  endBatch(): void {
    if (!this.bridge) return;

    this.flushBatch();
    this.bridge.end_batch();
    this.inBatchMode = false;
  }

  /**
   * Set max batch size
   */
  setMaxBatchSize(size: number): void {
    this.maxBatchSize = Math.max(1, Math.min(1000, size));
  }

  // ══════════════════════════════════════════════════════════════════════════════
  // INPUT SAMPLING
  // ══════════════════════════════════════════════════════════════════════════════

  /**
   * Sample input from mouse/keyboard
   */
  sampleInput(x: number, y: number, buttons: number): void {
    this.bridge?.sample_input(x, y, buttons);
  }

  /**
   * Advance simulation by dt milliseconds
   */
  tick(dt: number): void {
    this.bridge?.tick(dt);
  }

  // ══════════════════════════════════════════════════════════════════════════════
  // CLEANUP
  // ══════════════════════════════════════════════════════════════════════════════

  /**
   * Dispose service and cleanup resources
   */
  dispose(): void {
    this.isReadySubject.complete();
    this.eventsSubject.complete();
    this.errorSubject.complete();

    if (this.bridge) {
      this.bridge.free();
      this.bridge = null;
    }

    this.system = null;
    this.behaviorQueue = [];
  }

  /**
   * Check if service is ready
   */
  get isReady(): boolean {
    return this.isReadySubject.value;
  }
}

// ============================================================================
// SINGLETON INSTANCE
// ============================================================================

let serviceInstance: ArchFlowService | null = null;

/**
 * Get or create ArchFlowService singleton
 */
export function getArchFlowService(): ArchFlowService {
  if (!serviceInstance) {
    serviceInstance = new ArchFlowService();
  }
  return serviceInstance;
}

/**
 * Reset service instance (mainly for testing)
 */
export function resetArchFlowService(): void {
  if (serviceInstance) {
    serviceInstance.dispose();
    serviceInstance = null;
  }
}

// ============================================================================
// REACTIVE HELPERS
// ============================================================================

/**
 * Create observable for specific event type
 */
export function observeEventType(
  service: ArchFlowService,
  eventType: string
): Observable<ArchFlowEvent> {
  return service.events$.pipe(
    filter(event => event.type === eventType)
  );
}

/**
 * Create observable for entity events
 */
export function observeEntityEvents(
  service: ArchFlowService,
  entityId: number
): Observable<ArchFlowEvent> {
  return service.events$.pipe(
    filter(event => event.entityId === entityId)
  );
}

/**
 * Create observable with retry logic
 */
export function createObservableWithRetry<T>(
  factory: () => Observable<T>,
  maxRetries = 3
): Observable<T> {
  return factory().pipe(
    retryWhen(errors =>
      errors.pipe(
        delay(1000),
        tap((error, index) => {
          if (index >= maxRetries) {
            throw error;
          }
        })
      )
    )
  );
}
