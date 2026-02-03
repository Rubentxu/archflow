/**
 * Type helper for WASM bridge with proper typing
 *
 * Provides type-safe access to WASM bridge methods with runtime checks
 */

/** WasmBridge interface for type assertions */
interface WasmBridgeInterface {
  initialize(width: number, height: number): void;
  get_engine(): unknown;
  get_input_buffer_ptr(): number;
  get_input_buffer_size(): number;
  push_input_event(eventType: number, x: number, y: number, buttons: number, modifiers: number): void;
  tick(timestamp: number): void;
  spawn_entity(x: number, y: number, width: number, height: number): number;
  move_entity(entityIndex: number, dx: number, dy: number): void;
  set_color(entityIndex: number, r: number, g: number, b: number, a: number): void;
  set_shape(entityIndex: number, shape: number): void;
  set_label(entityIndex: number, label: string): void;
  set_size(entityIndex: number, width: number, height: number): void;
  set_position(entityIndex: number, x: number, y: number): void;
  entity_count(): number;
  clear(): void;
  delete_selected(): void;
  duplicate_entity(entityIndex: number): number;
  get_alive_entities(): number[];
  get_entity_position_screen(entityIndex: number): [number, number];
  get_entity_size_screen(entityIndex: number): [number, number];
  get_entity_color_hex(entityIndex: number): string;
  get_entity_shape(entityIndex: number): number;
  get_entity_label(entityIndex: number): string;
  is_entity_visible(entityIndex: number): boolean;
  is_entity_selected(entityIndex: number): boolean;
  select_entity(entityIndex: number): void;
  clear_selection(): void;
  get_selection(): number[];
  set_entity_selected(entityIndex: number, selected: boolean): void;
  set_zoom(zoom: number): void;
  get_zoom(): number;
  set_camera_center(x: number, y: number): void;
  get_camera_center(): [number, number];
  undo(): void;
  redo(): void;
  can_undo(): boolean;
  can_redo(): boolean;
  get_history_state(): string;
  serialize_project(): Uint8Array;
  set_tool(tool: string): void;
  get_tool(): string;
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
    spawn_entity: typedBridge?.spawn_entity.bind(typedBridge),
    move_entity: typedBridge?.move_entity.bind(typedBridge),
    delete_selected: typedBridge?.delete_selected.bind(typedBridge),
    duplicate_entity: typedBridge?.duplicate_entity.bind(typedBridge),
    get_alive_entities: typedBridge?.get_alive_entities.bind(typedBridge),
    get_entity_position_screen: typedBridge?.get_entity_position_screen.bind(typedBridge),
    get_entity_size_screen: typedBridge?.get_entity_size_screen.bind(typedBridge),
    set_entity_selected: typedBridge?.set_entity_selected.bind(typedBridge),

    // Selection
    clear_selection: typedBridge?.clear_selection.bind(typedBridge),
    get_selection: typedBridge?.get_selection.bind(typedBridge),
    select_entity: typedBridge?.select_entity.bind(typedBridge),

    // History
    undo: typedBridge?.undo.bind(typedBridge),
    redo: typedBridge?.redo.bind(typedBridge),
    can_undo: typedBridge?.can_undo.bind(typedBridge),
    can_redo: typedBridge?.can_redo.bind(typedBridge),
    get_history_state: typedBridge?.get_history_state.bind(typedBridge),

    // Camera
    set_zoom: typedBridge?.set_zoom.bind(typedBridge),
    get_zoom: typedBridge?.get_zoom.bind(typedBridge),
    set_camera_center: typedBridge?.set_camera_center.bind(typedBridge),
    get_camera_center: typedBridge?.get_camera_center.bind(typedBridge),

    // Input
    push_input_event: typedBridge?.push_input_event.bind(typedBridge),
    get_input_buffer_ptr: typedBridge?.get_input_buffer_ptr.bind(typedBridge),

    // Tick
    tick: typedBridge?.tick.bind(typedBridge),
    initialize: typedBridge?.initialize.bind(typedBridge),
  };
}
