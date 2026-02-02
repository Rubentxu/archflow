/**
 * TypeScript types for ArchFlow WASM Bridge
 *
 * These types are generated based on the Rust WasmBridge implementation
 * in crates/archflow-web/src/bridge.rs
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
 */

/**
 * Entity identifier type used throughout the system
 * Corresponds to Rust's EntityId in archflow-core
 */
export type EntityId = number;

/**
 * 2D vector type for positions and sizes
 */
export interface Vec2 {
  x: number;
  y: number;
}

/**
 * 2D vector for dimensions
 */
export interface Dimensions {
  w: number;
  h: number;
}

/**
 * RGBA color representation
 */
export interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

/**
 * Camera state for viewport management
 */
export interface CameraState {
  center: Vec2;
  zoom: number;
  canvasWidth: number;
  canvasHeight: number;
}

/**
 * Entity data retrieved from WASM
 */
export interface EntityData {
  id: EntityId;
  position: Vec2;
  size: Dimensions;
  color: string; // Hex format "#RRGGBB"
  shape: number; // ShapeType as number (enum converted for erasable syntax)
  label: string;
  isVisible: boolean;
  isSelected: boolean;
}

/**
 * Shape types available in the engine (as const object for erasable syntax)
 */
export const ShapeType = {
  Rectangle: 0,
  Circle: 1,
  Triangle: 2,
  Diamond: 3,
  Pentagon: 4,
  Hexagon: 5,
  Star: 6,
  Cloud: 7,
  Lightning: 8,
  Heart: 9,
  Question: 10,
  Plus: 11,
  Minus: 12,
  Multiply: 13,
  Exclamation: 14,
  Divide: 15,
} as const;

export type ShapeType = (typeof ShapeType)[keyof typeof ShapeType];

/**
 * Tool types for the editor
 */
export type ToolType =
  | "select"
  | "pan"
  | "rectangle"
  | "circle"
  | "triangle"
  | "diamond"
  | "text"
  | "connection"
  | "delete";

/**
 * Input event types matching Rust InputEventType (as const object for erasable syntax)
 */
export const InputEventType = {
  Down: 0,
  Move: 1,
  Up: 2,
  Wheel: 3,
  KeyDown: 4,
  KeyUp: 5,
} as const;

export type InputEventType =
  (typeof InputEventType)[keyof typeof InputEventType];

/**
 * Mouse button flags (as const object for erasable syntax)
 */
export const MouseButton = {
  Left: 1,
  Middle: 2,
  Right: 4,
} as const;

export type MouseButton = (typeof MouseButton)[keyof typeof MouseButton];

/**
 * Keyboard modifier flags (as const object for erasable syntax)
 */
export const Modifier = {
  None: 0,
  Shift: 1,
  Ctrl: 2,
  Alt: 4,
  Meta: 8,
} as const;

export type Modifier = (typeof Modifier)[keyof typeof Modifier];

/**
 * Raw input event from JavaScript
 */
export interface RawInputEvent {
  eventType: InputEventType;
  x: number;
  y: number;
  buttons: number;
  modifiers: number;
  timestamp: number;
}

/**
 * Selection state
 */
export interface SelectionState {
  selectedIds: EntityId[];
  canUndo: boolean;
  canRedo: boolean;
  historyState: string; // Format: "undo:N,redo:M"
}

/**
 * Engine initialization options
 */
export interface EngineOptions {
  canvasWidth: number;
  canvasHeight: number;
}

/**
 * WasmBridge interface (mirrors Rust implementation)
 *
 * Methods exposed to JavaScript via wasm-bindgen
 */
export interface WasmBridge {
  // Lifecycle
  new (): WasmBridge;
  initialize(canvasWidth: number, canvasHeight: number): void;

  // Input
  getInputBufferPtr(): number;
  getInputBufferSize(): number;
  pushInputEvent(
    eventType: number,
    x: number,
    y: number,
    buttons: number,
    modifiers: number,
  ): void;

  // Main loop
  tick(timestamp: number): void;

  // Entity operations
  spawnEntity(x: number, y: number, width: number, height: number): EntityId;
  moveEntity(entityIndex: EntityId, dx: number, dy: number): void;
  setColor(
    entityIndex: EntityId,
    r: number,
    g: number,
    b: number,
    a: number,
  ): void;
  setShape(entityIndex: EntityId, shape: number): void;
  setLabel(entityIndex: EntityId, label: string): void;
  setSize(entityIndex: EntityId, width: number, height: number): void;
  setPosition(entityIndex: EntityId, x: number, y: number): void;
  entityCount(): number;
  clear(): void;
  deleteSelected(): void;
  duplicateEntity(entityIndex: EntityId): EntityId;

  // Query operations
  getAliveEntities(): EntityId[];
  getEntityPositionScreen(entityIndex: EntityId): [number, number];
  getEntitySizeScreen(entityIndex: EntityId): [number, number];
  getEntityColorHex(entityIndex: EntityId): string;
  getEntityShape(entityIndex: EntityId): number;
  getEntityLabel(entityIndex: EntityId): string;
  isEntityVisible(entityIndex: EntityId): boolean;
  isEntitySelected(entityIndex: EntityId): boolean;

  // Selection
  selectEntity(entityIndex: EntityId): void;
  clearSelection(): void;
  getSelection(): EntityId[];
  setEntitySelected(entityIndex: EntityId, selected: boolean): void;

  // Camera
  setZoom(zoom: number): void;
  getZoom(): number;
  setCameraCenter(x: number, y: number): void;
  getCameraCenter(): [number, number];

  // History
  undo(): void;
  redo(): void;
  canUndo(): boolean;
  canRedo(): boolean;
  getHistoryState(): string;

  // Serialization
  serializeProject(): Uint8Array;

  // Tools
  setTool(tool: string): void;
  getTool(): string;
}

/**
 * Hook return type for WASM bridge
 */
export interface UseWasmBridgeReturn {
  bridge: unknown;
  isLoaded: boolean;
  isInitialized: boolean;
  error: Error | null;
  initialize: (width: number, height: number) => Promise<void>;
}

/**
 * Hook return type for entity operations
 */
export interface UseEntityStoreReturn {
  entities: Map<EntityId, EntityData>;
  entityCount: number;
  spawnEntity: (
    x: number,
    y: number,
    width?: number,
    height?: number,
  ) => EntityId;
  deleteEntity: (id: EntityId) => void;
  duplicateEntity: (id: EntityId) => EntityId | null;
  updateEntity: (id: EntityId, updates: Partial<EntityData>) => void;
  getEntity: (id: EntityId) => EntityData | null;
  refreshEntities: () => void;
}

/**
 * Hook return type for camera operations
 */
export interface UseCameraReturn {
  camera: CameraState;
  setZoom: (zoom: number) => void;
  zoomIn: (factor?: number) => void;
  zoomOut: (factor?: number) => void;
  setCenter: (x: number, y: number) => void;
  pan: (dx: number, dy: number) => void;
  worldToScreen: (worldPos: Vec2) => Vec2;
  screenToWorld: (screenPos: Vec2) => Vec2;
  setCanvasSize: (width: number, height: number) => void;
  fitToContent: (bounds: {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
  }) => void;
  reset: () => void;
}

/**
 * Hook return type for selection operations
 */
export interface UseSelectionReturn {
  selectedIds: EntityId[];
  canUndo: boolean;
  canRedo: boolean;
  select: (id: EntityId, additive?: boolean) => void;
  deselect: (id: EntityId) => void;
  selectMultiple: (ids: EntityId[]) => void;
  toggleSelection: (id: EntityId) => void;
  clearSelection: () => void;
  undo: () => void;
  redo: () => void;
}

/**
 * Engine configuration
 */
export interface EngineConfig {
  initialZoom: number;
  minZoom: number;
  maxZoom: number;
  gridSize: number;
  showGrid: boolean;
  enableSnapping: boolean;
  snapThreshold: number;
}
