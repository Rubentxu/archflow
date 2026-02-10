/**
 * useArchFlow - React Hook for ArchFlow integration
 *
 * This hook provides a complete React integration for the ArchFlow system,
 * following the Developer Manual specification for framework integration.
 *
 * Developer Manual: "Receta 2: React con Servicios Intermedios"
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import { ArchFlowService, BehaviorDefinition, ArchFlowEvent, getArchFlowService, observeEventType } from '../services/ArchFlowService';

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/**
 * Options for useArchFlow hook
 */
export interface UseArchFlowOptions {
  onEntitySelected?: (entityId: number) => void;
  onEntityHovered?: (entityId: number) => void;
  onCustomEvent?: (eventName: string, data: unknown) => void;
  autoInitialize?: boolean;
}

/**
 * Return type for useArchFlow hook
 */
export interface UseArchFlowReturn {
  // State
  isReady: boolean;
  isInitializing: boolean;
  error: Error | null;

  // Methods
  registerBehavior: (behavior: BehaviorDefinition) => void;
  registerBehaviors: (behaviors: BehaviorDefinition[]) => void;
  createBehaviors: () => BehaviorFactory | null;
  sampleInput: (x: number, y: number, buttons: number) => void;
  tick: (dt: number) => void;

  // Batch methods
  beginBatch: () => void;
  addToBatch: (behavior: BehaviorDefinition) => void;
  endBatch: () => void;

  // Template methods
  getBehaviorTemplate: (name: string) => BehaviorDefinition | null;
  listBehaviorTemplates: () => string[];

  // Cleanup
  dispose: () => void;
}

/**
 * Behavior factory return type
 */
export interface BehaviorFactory {
  hoverHighlight: (options?: { color?: string; opacity?: number }) => BehaviorDefinition;
  clickSelect: (options?: { button?: number; mode?: 'single' | 'multi' | 'toggle' }) => BehaviorDefinition;
  wasdMovement: (options?: { keys?: number[]; speed?: number }) => BehaviorDefinition;
  draggable: (options?: { debounce?: number; speed?: number }) => BehaviorDefinition;
  deletable: () => BehaviorDefinition;
}

// ============================================================================
// HOOK IMPLEMENTATION
// ============================================================================

/**
 * useArchFlow - React hook for ArchFlow integration
 *
 * Provides complete ArchFlow functionality with React lifecycle management.
 *
 * @param options - Hook options
 * @returns ArchFlow API
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const {
 *     isReady,
 *     registerBehaviors,
 *     sampleInput,
 *     tick,
 *     createBehaviors
 *   } = useArchFlow({
 *     onEntitySelected: (id) => console.log('Selected:', id),
 *     onEntityHovered: (id) => console.log('Hovered:', id)
 *   });
 *
 *   useEffect(() => {
 *     if (!isReady) return;
 *
 *     const factory = createBehaviors();
 *     if (!factory) return;
 *
 *     registerBehaviors([
 *       factory.hoverHighlight(),
 *       factory.clickSelect()
 *     ]);
 *   }, [isReady]);
 *
 *   // ... rest of component
 * }
 * ```
 */
export function useArchFlow(options: UseArchFlowOptions = {}): UseArchFlowReturn {
  const {
    onEntitySelected,
    onEntityHovered,
    onCustomEvent,
    autoInitialize = true
  } = options;

  const serviceRef = useRef<ArchFlowService | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [isInitializing, setIsInitializing] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Initialize service
  useEffect(() => {
    let mounted = true;

    const init = async () => {
      try {
        const service = getArchFlowService();
        serviceRef.current = service;

        if (autoInitialize && !service.isReady) {
          setIsInitializing(true);

          await service.initialize();

          if (mounted) {
            setIsReady(true);
            setIsInitializing(false);
          }
        } else if (service.isReady) {
          setIsReady(true);
        }

        // Setup event handlers
        if (mounted) {
          const subscription = service.events$.subscribe((event: ArchFlowEvent) => {
            switch (event.type) {
              case 'EntitySelected':
                onEntitySelected?.(event.entityId);
                break;
              case 'EntityHovered':
                onEntityHovered?.(event.entityId);
                break;
              default:
                if (event.type.startsWith('Custom:')) {
                  const eventName = event.type.replace('Custom:', '');
                  onCustomEvent?.(eventName, event.data);
                }
                break;
            }
          });

          return () => {
            subscription.unsubscribe();
          };
        }
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        setIsInitializing(false);
      }
    };

    init();

    return () => {
      mounted = false;
      // Don't dispose service on unmount (it's a singleton)
    };
  }, [autoInitialize, onEntitySelected, onEntityHovered, onCustomEvent]);

  // ══════════════════════════════════════════════════════════════════════════════
  // METHODS
  // ══════════════════════════════════════════════════════════════════════════════

  const registerBehavior = useCallback((behavior: BehaviorDefinition) => {
    serviceRef.current?.registerBehavior(behavior);
  }, []);

  const registerBehaviors = useCallback((behaviors: BehaviorDefinition[]) => {
    serviceRef.current?.registerBehaviors(behaviors);
  }, []);

  const createBehaviors = useCallback((): BehaviorFactory | null => {
    const service = serviceRef.current;
    if (!service || !service.isReady) return null;

    return service.createBehaviorFactory();
  }, [isReady]);

  const sampleInput = useCallback((x: number, y: number, buttons: number) => {
    serviceRef.current?.sampleInput(x, y, buttons);
  }, []);

  const tick = useCallback((dt: number) => {
    serviceRef.current?.tick(dt);
  }, []);

  // Batch methods
  const beginBatch = useCallback(() => {
    serviceRef.current?.beginBatch();
  }, []);

  const addToBatch = useCallback((behavior: BehaviorDefinition) => {
    serviceRef.current?.addToBatch(behavior);
  }, []);

  const endBatch = useCallback(() => {
    serviceRef.current?.endBatch();
  }, []);

  // Template methods
  const getBehaviorTemplate = useCallback((name: string): BehaviorDefinition | null => {
    return serviceRef.current?.getBehaviorTemplate(name) ?? null;
  }, []);

  const listBehaviorTemplates = useCallback((): string[] => {
    return serviceRef.current?.listBehaviorTemplates() ?? [];
  }, []);

  const dispose = useCallback(() => {
    serviceRef.current?.dispose();
    serviceRef.current = null;
    setIsReady(false);
  }, []);

  return {
    // State
    isReady,
    isInitializing,
    error,

    // Methods
    registerBehavior,
    registerBehaviors,
    createBehaviors,
    sampleInput,
    tick,

    // Batch methods
    beginBatch,
    addToBatch,
    endBatch,

    // Template methods
    getBehaviorTemplate,
    listBehaviorTemplates,

    // Cleanup
    dispose,
  };
}

// ============================================================================
// SPECIALIZED HOOKS
// ============================================================================

/**
 * useArchFlowEvents - Hook for subscribing to specific event types
 *
 * @param eventType - Event type to observe
 * @returns Observable of events
 *
 * @example
 * ```tsx
 * function EntitySelector() {
 *   const events = useArchFlowEvents('EntitySelected');
 *
 *   useEffect(() => {
 *     const subscription = events.subscribe(({ entityId }) => {
 *       console.log('Entity selected:', entityId);
 *     });
 *
 *     return () => subscription.unsubscribe();
 *   }, [events]);
 *
 *   return <div>Entity Selector</div>;
 * }
 * ```
 */
export function useArchFlowEvents(eventType: string) {
  const service = getArchFlowService();
  return observeEventType(service, eventType);
}

/**
 * useArchFlowReady - Hook that fires callback when ArchFlow is ready
 *
 * @param callback - Callback to fire when ready
 * @param deps - Dependencies
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   useArchFlowReady(() => {
 *     console.log('ArchFlow is ready!');
 *   }, []);
 *
 *   return <div>My Component</div>;
 * }
 * ```
 */
export function useArchFlowReady(callback: () => void, deps: any[] = []) {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const service = getArchFlowService();

    const subscription = service.isReady$.subscribe(isReady => {
      if (isReady && !ready) {
        setReady(true);
        callback();
      }
    });

    return () => {
      subscription.unsubscribe();
    };
  }, deps);

  return ready;
}

// ============================================================================
// CONVENIENCE COMPONENTS
// ============================================================================

/**
 * ArchFlowProvider - Context provider for ArchFlow service
 *
 * @example
 * ```tsx
 * function App() {
 *   return (
 *     <ArchFlowProvider>
 *       <YourComponents />
 *     </ArchFlowProvider>
 *   );
 * }
 * ```
 */
export function ArchFlowProvider({ children }: { children: React.ReactNode }) {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const service = getArchFlowService();

    const subscription = service.isReady$.subscribe(isReady => {
      setReady(isReady);
    });

    return () => {
      subscription.unsubscribe();
    };
  }, []);

  return (
    <>
      {ready ? children : (
        <div style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100vh',
          fontSize: '1.2rem',
          color: '#666'
        }}>
          🔄 Initializing ArchFlow...
        </div>
      )}
    </>
  );
}

/**
 * ArchFlowInitializer - Component that initializes ArchFlow
 *
 * @example
 * ```tsx
 * function App() {
 *   return (
 *     <>
 *       <ArchFlowInitializer />
 *       <YourComponents />
 *     </>
 *   );
 * }
 * ```
 */
export function ArchFlowInitializer() {
  const { isReady, isInitializing, error } = useArchFlow({
    autoInitialize: true
  });

  if (error) {
    return (
      <div style={{
        padding: '20px',
        backgroundColor: '#fee',
        color: '#c00',
        borderRadius: '4px'
      }}>
        <strong>ArchFlow Error:</strong> {error.message}
      </div>
    );
  }

  if (isInitializing || !isReady) {
    return (
      <div style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
        fontSize: '1.2rem',
        color: '#666'
      }}>
        🔄 Initializing ArchFlow...
      </div>
    );
  }

  return null;
}

export default useArchFlow;
