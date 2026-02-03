/**
 * TypeScript types for ArchFlow WASM Bridge
 *
 * These types are generated based on the Rust WasmBridge implementation
 * in crates/archflow-web/src/bridge.rs
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
 */

/**
 * Import WasmBridge type from src/wasm directory
 */
import type { WasmBridge } from "../wasm/archflow_web.js";

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
  /** Entity type for C4 diagram elements (aws-ec2, aws-lambda, etc.) */
  type?: string;
  /** Properties for the entity (used by PropertiesPanel) */
  properties?: Record<string, unknown>;
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
  new(): WasmBridge;
  initialize(canvasWidth: number, canvasHeight: number): void;

  // Input
  get_input_buffer_ptr(): number;
  get_input_buffer_size(): number;
  push_input_event(
    eventType: number,
    x: number,
    y: number,
    buttons: number,
    modifiers: number,
  ): void;

  // Main loop
  tick(timestamp: number): void;

  // Entity operations
  spawn_entity(x: number, y: number, width: number, height: number): EntityId;
  move_entity(entityIndex: EntityId, dx: number, dy: number): void;
  set_color(
    entityIndex: EntityId,
    r: number,
    g: number,
    b: number,
    a: number,
  ): void;
  set_shape(entityIndex: EntityId, shape: number): void;
  set_label(entityIndex: EntityId, label: string): void;
  set_size(entityIndex: EntityId, width: number, height: number): void;
  set_position(entityIndex: EntityId, x: number, y: number): void;
  entity_count(): number;
  clear(): void;
  delete_selected(): void;
  duplicate_entity(entityIndex: EntityId): EntityId;

  // Query operations
  get_alive_entities(): EntityId[];
  get_entity_position_screen(entityIndex: EntityId): [number, number];
  get_entity_size_screen(entityIndex: EntityId): [number, number];
  get_entity_color_hex(entityIndex: EntityId): string;
  get_entity_shape(entityIndex: EntityId): number;
  get_entity_label(entityIndex: EntityId): string;
  is_entity_visible(entityIndex: EntityId): boolean;
  is_entity_selected(entityIndex: EntityId): boolean;

  // Selection
  select_entity(entityIndex: EntityId): void;
  clear_selection(): void;
  get_selection(): EntityId[];
  set_entity_selected(entityIndex: EntityId, selected: boolean): void;

  // Camera
  set_zoom(zoom: number): void;
  get_zoom(): number;
  set_camera_center(x: number, y: number): void;
  get_camera_center(): [number, number];

  // History
  undo(): void;
  redo(): void;
  can_undo(): boolean;
  can_redo(): boolean;
  get_history_state(): string;

  // Serialization
  serialize_project(): Uint8Array;

  // Tools
  set_tool(tool: string): void;
  get_tool(): string;
}

/**
 * Hook return type for WASM bridge
 */
export interface UseWasmBridgeReturn {
  bridge: WasmBridge | null;
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
