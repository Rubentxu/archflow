/**
 * Type helper for WASM bridge with proper typing
 *
 * Provides type-safe access to WASM bridge methods with runtime checks
 */

/** WasmBridge interface for type assertions */
interface WasmBridgeInterface {
  initialize(width: number, height: number): void;
  get_engine(): unknown;
  getInputBufferPtr(): number;
  getInputBufferSize(): number;
  pushInputEvent(eventType: number, x: number, y: number, buttons: number, modifiers: number): void;
  tick(timestamp: number): void;
  spawnEntity(x: number, y: number, width: number, height: number): number;
  moveEntity(entityIndex: number, dx: number, dy: number): void;
  setColor(entityIndex: number, r: number, g: number, b: number, a: number): void;
  setShape(entityIndex: number, shape: number): void;
  setLabel(entityIndex: number, label: string): void;
  setSize(entityIndex: number, width: number, height: number): void;
  setPosition(entityIndex: number, x: number, y: number): void;
  entityCount(): number;
  clear(): void;
  deleteSelected(): void;
  duplicateEntity(entityIndex: number): number;
  getAliveEntities(): number[];
  getEntityPositionScreen(entityIndex: number): [number, number];
  getEntitySizeScreen(entityIndex: number): [number, number];
  getEntityColorHex(entityIndex: number): string;
  getEntityShape(entityIndex: number): number;
  getEntityLabel(entityIndex: number): string;
  isEntityVisible(entityIndex: number): boolean;
  isEntitySelected(entityIndex: number): boolean;
  selectEntity(entityIndex: number): void;
  clearSelection(): void;
  getSelection(): number[];
  setEntitySelected(entityIndex: number, selected: boolean): void;
  setZoom(zoom: number): void;
  getZoom(): number;
  setCameraCenter(x: number, y: number): void;
  getCameraCenter(): [number, number];
  undo(): void;
  redo(): void;
  canUndo(): boolean;
  canRedo(): boolean;
  getHistoryState(): string;
  serializeProject(): Uint8Array;
  setTool(tool: string): void;
  getTool(): string;
}

/**
 * Type guard to check if a value is a WasmBridge
 */
function isWasmBridge(value: unknown): value is WasmBridgeInterface {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as WasmBridgeInterface).initialize === "function" &&
    typeof (value as WasmBridgeInterface).tick === "function"
  );
}

/**
 * Get typed bridge from unknown
 */
export function getTypedBridge(bridge: unknown): WasmBridgeInterface | null {
  if (!bridge || !isWasmBridge(bridge)) {
    return null;
  }
  return bridge;
}

/**
 * Create a type-safe wrapper around bridge methods
 */
export function useWasmBridgeMethods(bridge: unknown) {
  const typedBridge = getTypedBridge(bridge);

  return {
    bridge: typedBridge,
    isLoaded: !!typedBridge,

    // Entity operations
    spawnEntity: typedBridge?.spawnEntity.bind(typedBridge),
    moveEntity: typedBridge?.moveEntity.bind(typedBridge),
    deleteSelected: typedBridge?.deleteSelected.bind(typedBridge),
    duplicateEntity: typedBridge?.duplicateEntity.bind(typedBridge),
    getAliveEntities: typedBridge?.getAliveEntities.bind(typedBridge),
    getEntityPositionScreen: typedBridge?.getEntityPositionScreen.bind(typedBridge),
    getEntitySizeScreen: typedBridge?.getEntitySizeScreen.bind(typedBridge),
    setEntitySelected: typedBridge?.setEntitySelected.bind(typedBridge),

    // Selection
    clearSelection: typedBridge?.clearSelection.bind(typedBridge),
    getSelection: typedBridge?.getSelection.bind(typedBridge),
    selectEntity: typedBridge?.selectEntity.bind(typedBridge),

    // History
    undo: typedBridge?.undo.bind(typedBridge),
    redo: typedBridge?.redo.bind(typedBridge),
    canUndo: typedBridge?.canUndo.bind(typedBridge),
    canRedo: typedBridge?.canRedo.bind(typedBridge),
    getHistoryState: typedBridge?.getHistoryState.bind(typedBridge),

    // Camera
    setZoom: typedBridge?.setZoom.bind(typedBridge),
    getZoom: typedBridge?.getZoom.bind(typedBridge),
    setCameraCenter: typedBridge?.setCameraCenter.bind(typedBridge),
    getCameraCenter: typedBridge?.getCameraCenter.bind(typedBridge),

    // Input
    pushInputEvent: typedBridge?.pushInputEvent.bind(typedBridge),
    getInputBufferPtr: typedBridge?.getInputBufferPtr.bind(typedBridge),

    // Tick
    tick: typedBridge?.tick.bind(typedBridge),
    initialize: typedBridge?.initialize.bind(typedBridge),
  };
}
