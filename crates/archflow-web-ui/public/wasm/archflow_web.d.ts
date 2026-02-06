/* tslint:disable */
/* eslint-disable */

/**
 * Actuator types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { ActuatorType } from '@archflow/sdk';
 *
 * const actuator = ActuatorType.Highlight;
 * ```
 */
export enum ActuatorType {
    /**
     * Highlight actuator - changes entity color
     */
    Highlight = 0,
    /**
     * Select actuator - marks entity as selected
     */
    Select = 1,
    /**
     * Move actuator - moves entity (drag operation)
     */
    Move = 2,
}

/**
 * Configuration for camera actuator
 */
export class CameraConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get duration in milliseconds
     */
    duration_ms(): number;
    /**
     * Creates a new camera configuration
     */
    constructor(target_x: number, target_y: number, zoom: number, duration_ms: number, smooth: number);
    /**
     * Get smoothing factor (0.0 - 1.0)
     */
    smooth(): number;
    /**
     * Get target X position
     */
    target_x(): number;
    /**
     * Get target Y position
     */
    target_y(): number;
    /**
     * Get zoom level
     */
    zoom(): number;
}

/**
 * Controller for boolean logic operations on sensor signals
 *
 * This struct wraps the ControllerType enum with optional parameters:
 * - secondary_sensor: for AND/OR operations
 * - numeric_params: for Blinky, Debounce, Threshold, Pattern controllers
 * - float_params: for Hysteresis controller
 * - custom_data: for Custom controller (name, code)
 */
export class Controller {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates an AND controller with a secondary sensor
     *
     * # Arguments
     * * `sensor` - The secondary sensor type
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.And(SensorType.MouseClick);
     * // Requires both primary sensor AND MouseClick to be active
     * ```
     */
    static and(sensor: SensorType): Controller;
    /**
     * Creates a Blinky controller that toggles at regular intervals
     *
     * # Arguments
     * * `interval` - Toggle interval in ticks (16.67ms at 60 FPS)
     *
     * # JavaScript Example
     * ```javascript
     * // Blink every 100ms (6 ticks at 60fps)
     * const blinky = Controller.Blinky(6);
     * ```
     */
    static blinky(interval: number): Controller;
    /**
     * Returns the controller type
     */
    controller_type(): ControllerType;
    /**
     * Creates a Custom controller with JavaScript code
     *
     * # Arguments
     * * `name` - Unique identifier for debugging
     * * `code` - JavaScript code to evaluate
     *
     * # JavaScript Example
     * ```javascript
     * const custom = Controller.Custom(
     *   'tooltipOnCtrlHover',
     *   'return signal.isSteady(6) && (context.modifiers & 2) !== 0;'
     * );
     * ```
     */
    static custom(name: string, code: string): Controller;
    /**
     * Returns the custom code (for Custom controllers)
     */
    custom_code(): string | undefined;
    /**
     * Returns the custom name (for Custom controllers)
     */
    custom_name(): string | undefined;
    /**
     * Creates a Debounce controller requiring N stable ticks
     *
     * # Arguments
     * * `ticks` - Number of consecutive ticks signal must be HIGH
     *
     * # JavaScript Example
     * ```javascript
     * // Require 100ms of stable signal (6 ticks)
     * const debounced = Controller.Debounce(6);
     * ```
     */
    static debounce(ticks: number): Controller;
    /**
     * Creates a new Direct controller (pass-through)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Direct();
     * ```
     */
    static direct(): Controller;
    /**
     * Returns the first float parameter (for Hysteresis high, Threshold value)
     */
    float_param1(): number;
    /**
     * Returns the second float parameter (for Hysteresis low)
     */
    float_param2(): number;
    /**
     * Checks if this controller has a secondary sensor
     */
    has_secondary_sensor(): boolean;
    /**
     * Creates a Hysteresis controller with different on/off thresholds
     *
     * # Arguments
     * * `high` - Activation threshold (0.0 to 1.0)
     * * `low` - Deactivation threshold (0.0 to 1.0)
     *
     * # JavaScript Example
     * ```javascript
     * // Activate at 80%, deactivate at 30%
     * const hyst = Controller.Hysteresis(0.8, 0.3);
     * ```
     */
    static hysteresis(high: number, low: number): Controller;
    /**
     * Checks if this controller is a Custom type
     */
    is_custom(): boolean;
    /**
     * Creates a NOT controller (inverts the signal)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Not();
     * // Inverts the primary sensor signal
     * ```
     */
    static not(): Controller;
    /**
     * Returns the numeric parameter (for Blinky, Debounce, Pattern)
     */
    numeric_param(): number;
    /**
     * Creates an OR controller with a secondary sensor
     *
     * # Arguments
     * * `sensor` - The secondary sensor type
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Or(SensorType.MouseClick);
     * // Requires primary sensor OR MouseClick to be active
     * ```
     */
    static or(sensor: SensorType): Controller;
    /**
     * Creates a Pattern controller matching binary pattern
     *
     * # Arguments
     * * `mask` - 6-bit pattern to match
     *
     * # JavaScript Example
     * ```javascript
     * // Match double-click pattern: 100100
     * const pattern = Controller.Pattern(0b00100100);
     * ```
     */
    static pattern(mask: number): Controller;
    /**
     * Returns the secondary sensor (if any)
     *
     * Returns `null` if there is no secondary sensor.
     */
    secondary_sensor(): SensorType | undefined;
    /**
     * Creates a Threshold controller with minimum stability
     *
     * # Arguments
     * * `value` - Minimum stability threshold (0.0 to 1.0)
     *
     * # JavaScript Example
     * ```javascript
     * // Require 50% stability (3 out of 6 ticks)
     * const thresh = Controller.Threshold(0.5);
     * ```
     */
    static threshold(value: number): Controller;
}

/**
 * Controller type enumeration
 *
 * Defines the type of boolean logic to apply when combining sensor signals.
 *
 * # JavaScript Example
 * ```javascript
 * import { Controller, SensorType } from '@archflow/sdk';
 *
 * // Direct: pass through the primary sensor
 * const direct = Controller.Direct();
 *
 * // AND: require both MouseOver AND MouseClick
 * const and = Controller.And(SensorType.MouseClick);
 *
 * // OR: require MouseOver OR MouseClick
 * const or = Controller.Or(SensorType.MouseClick);
 *
 * // NOT: invert the primary sensor
 * const not = Controller.Not();
 *
 * // Blinky: toggle every N ticks
 * const blinky = Controller.Blinky(4);
 *
 * // Debounce: require N stable ticks
 * const debounce = Controller.Debounce(6);
 *
 * // Hysteresis: different on/off thresholds
 * const hyst = Controller.Hysteresis(0.8, 0.3);
 * ```
 */
export enum ControllerType {
    /**
     * Pass through the primary sensor signal
     */
    Direct = 0,
    /**
     * AND logic: primary AND other sensor must both be active
     */
    And = 1,
    /**
     * OR logic: primary OR other sensor must be active
     */
    Or = 2,
    /**
     * NOT logic: invert the primary sensor signal
     */
    Not = 3,
    /**
     * Blinky: Toggles active/inactive at regular intervals
     */
    Blinky = 4,
    /**
     * Debounce: Requires signal to be stable for N ticks
     */
    Debounce = 5,
    /**
     * Hysteresis: Different activation/deactivation thresholds
     */
    Hysteresis = 6,
    /**
     * Threshold: Requires minimum stability percentage
     */
    Threshold = 7,
    /**
     * Pattern: Matches specific binary pattern in history
     */
    Pattern = 8,
    /**
     * Custom: JavaScript sandbox evaluation
     */
    Custom = 9,
}

/**
 * Extended actuator types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { ExtendedActuatorType } from '@archflow/sdk';
 *
 * const highlight = ExtendedActuatorType.Highlight;
 * const camera = ExtendedActuatorType.Camera;
 * ```
 */
export enum ExtendedActuatorType {
    /**
     * Highlight actuator - changes entity color
     */
    Highlight = 0,
    /**
     * Select actuator - marks entity as selected
     */
    Select = 1,
    /**
     * Move actuator - moves entity (drag operation)
     */
    Move = 2,
    /**
     * Camera actuator - moves camera
     */
    Camera = 3,
    /**
     * Property actuator - sets entity property
     */
    Property = 4,
    /**
     * State actuator - changes entity state
     */
    State = 5,
}

/**
 * Configuration for highlight actuator
 */
export class HighlightConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the highlight color (ARGB)
     */
    color(): number;
    /**
     * Creates a new highlight configuration
     */
    constructor(color: number, restore_color: number, opacity: number);
    /**
     * Get the opacity (0.0 - 1.0)
     */
    opacity(): number;
    /**
     * Get the restore color (ARGB)
     */
    restore_color(): number;
}

/**
 * Custom error type for JavaScript
 */
export class JsError {
    free(): void;
    [Symbol.dispose](): void;
    message(): string;
    constructor(message: string);
}

/**
 * Logic Mapping Table for sensor-actuator connections
 *
 * This table manages connections between sensors and actuators for entities,
 * allowing complex behavior definition through the Logic Bricks system.
 *
 * # JavaScript Example
 * ```javascript
 * import { LogicMappingTable, SensorType, Controller, ActuatorType } from '@archflow/sdk';
 *
 * const table = new LogicMappingTable();
 * const entityId = 1;
 *
 * // Connect MouseOver sensor to Highlight actuator
 * table.addHighlight(entityId, SensorType.MouseOver, Controller.Direct());
 *
 * // Check if connection exists
 * console.log(table.hasConnection(entityId, SensorType.MouseOver)); // true
 *
 * // Get connection count
 * console.log(table.connectionCount(entityId)); // 1
 *
 * // Remove connection
 * table.removeConnection(entityId, SensorType.MouseOver);
 * ```
 */
export class LogicMappingTableWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Adds a Highlight actuator connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID (numeric)
     * * `sensor` - The sensor type to connect
     * * `controller` - The controller logic
     *
     * # JavaScript Example
     * ```javascript
     * table.addHighlight(1, SensorType.MouseOver, Controller.Direct());
     * ```
     */
    add_highlight(entity_id: number, sensor: SensorType, controller: Controller): void;
    /**
     * Adds a Move actuator connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID (numeric)
     * * `sensor` - The sensor type to connect
     * * `controller` - The controller logic
     *
     * # JavaScript Example
     * ```javascript
     * table.addMove(1, SensorType.MouseClick, Controller.And(SensorType.MouseOver));
     * ```
     */
    add_move(entity_id: number, sensor: SensorType, controller: Controller): void;
    /**
     * Adds a Select actuator connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID (numeric)
     * * `sensor` - The sensor type to connect
     * * `controller` - The controller logic
     *
     * # JavaScript Example
     * ```javascript
     * table.addSelect(1, SensorType.MouseClick, Controller.Direct());
     * ```
     */
    add_select(entity_id: number, sensor: SensorType, controller: Controller): void;
    /**
     * Clears all connections from the table
     *
     * # JavaScript Example
     * ```javascript
     * table.clear();
     * ```
     */
    clear(): void;
    /**
     * Clears all connections for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID
     *
     * # JavaScript Example
     * ```javascript
     * table.clearEntity(1);
     * ```
     */
    clear_entity(entity_id: number): void;
    /**
     * Gets the number of connections for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID
     *
     * # Returns
     * The number of connections registered for the entity
     *
     * # JavaScript Example
     * ```javascript
     * const count = table.connectionCount(1);
     * console.log(`Entity has ${count} connections`);
     * ```
     */
    connection_count(entity_id: number): number;
    /**
     * Gets all entity IDs that have connections
     *
     * # Returns
     * Array of entity IDs (as u32 values)
     *
     * # JavaScript Example
     * ```javascript
     * const entities = table.getConnectedEntities();
     * console.log(`Connected entities: ${entities}`);
     * ```
     */
    get_connected_entities(): Uint32Array;
    /**
     * Checks if an entity has a connection for a specific sensor
     *
     * # Arguments
     * * `entity_id` - The entity ID
     * * `sensor` - The sensor type to check for
     *
     * # Returns
     * `true` if the entity has a connection for the sensor, `false` otherwise
     *
     * # JavaScript Example
     * ```javascript
     * const hasConnection = table.hasConnection(1, SensorType.MouseOver);
     * ```
     */
    has_connection(entity_id: number, sensor: SensorType): boolean;
    /**
     * Checks if the table is empty
     *
     * # JavaScript Example
     * ```javascript
     * const isEmpty = table.isEmpty();
     * ```
     */
    is_empty(): boolean;
    /**
     * Creates a new LogicMappingTable
     *
     * # JavaScript Example
     * ```javascript
     * const table = new LogicMappingTable();
     * ```
     */
    constructor();
    /**
     * Removes a connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID
     * * `sensor` - The sensor type to disconnect
     *
     * # JavaScript Example
     * ```javascript
     * table.removeConnection(1, SensorType.MouseOver);
     * ```
     */
    remove_connection(entity_id: number, sensor: SensorType): void;
}

/**
 * WASM wrapper for LogicSystem
 *
 * This provides JavaScript access to the main Logic Bricks orchestration system.
 *
 * # JavaScript Example
 * ```javascript
 * import { LogicSystem } from '@archflow/sdk';
 *
 * const system = new LogicSystem();
 * system.update(timestamp);
 * ```
 */
export class LogicSystemWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates a new LogicSystem
     *
     * # JavaScript Example
     * ```javascript
     * const system = new LogicSystem();
     * ```
     */
    constructor();
    /**
     * Updates the logic system timestamp
     *
     * This should be called each frame before sensor evaluation.
     *
     * # Arguments
     * * `timestamp_ms` - Current timestamp in milliseconds
     *
     * # JavaScript Example
     * ```javascript
     * system.update(performance.now());
     * ```
     */
    update(timestamp_ms: bigint): void;
}

/**
 * Configuration for move actuator
 */
export class MoveConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Whether X axis is constrained
     */
    constrain_x(): boolean;
    /**
     * Whether Y axis is constrained
     */
    constrain_y(): boolean;
    /**
     * Creates a new move configuration
     */
    constructor(snap: number, constrain_x: boolean, constrain_y: boolean);
    /**
     * Get snap value in pixels
     */
    snap(): number;
}

/**
 * Configuration for property actuator
 */
export class PropertyConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates a new property configuration
     */
    constructor(property_name: string, value: PropertyValue);
    /**
     * Get property name
     */
    property_name(): string;
    /**
     * Get property value
     */
    value(): PropertyValue;
}

/**
 * Property value wrapper for WASM
 */
export class PropertyValue {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create a boolean property value
     */
    static from_bool(value: boolean): PropertyValue;
    /**
     * Create a number property value
     */
    static from_number(value: number): PropertyValue;
    /**
     * Create a string property value
     */
    static from_string(value: string): PropertyValue;
    /**
     * Get the raw value string
     */
    value(): string;
}

/**
 * WASM wrapper for Pulse events
 *
 * Represents a sensor state change event flowing through the Logic Bricks system.
 *
 * # JavaScript Example
 * ```javascript
 * const pulse = {
 *   entityId: 123,
 *   sensorId: 5,
 *   isActive: true,
 *   timestamp: 1000
 * };
 * ```
 */
export class PulseWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the entity ID
     */
    entity_id(): number;
    /**
     * Check if the pulse is active (positive edge)
     *
     * Returns true for positive pulses (sensor became TRUE)
     * Returns false for negative pulses (sensor became FALSE)
     */
    is_active(): boolean;
    /**
     * Creates a new PulseWasm
     *
     * # JavaScript Example
     * ```javascript
     * const pulse = new PulseWasm(123, 5, true, 1000);
     * ```
     */
    constructor(entity_id: number, sensor_id: number, state: boolean, timestamp: number);
    /**
     * Get the sensor ID
     */
    sensor_id(): number;
    /**
     * Get the timestamp
     */
    timestamp(): number;
}

/**
 * Select mode for selection actuator (matches core SelectMode)
 */
export enum SelectModeWasm {
    /**
     * Single selection (replaces current selection)
     */
    Single = 0,
    /**
     * Multi selection (adds to current selection)
     */
    Multi = 1,
    /**
     * Replace selection (clears and selects new)
     */
    Replace = 2,
}

/**
 * Sensor types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { SensorType } from '@archflow/sdk';
 *
 * const sensor = SensorType.MouseOver;
 * ```
 */
export enum SensorType {
    /**
     * Mouse is hovering over an entity
     */
    MouseOver = 0,
    /**
     * Mouse button was clicked on an entity
     */
    MouseClick = 1,
    /**
     * Another entity is within proximity radius
     */
    Proximity = 2,
    /**
     * Keyboard shortcut was pressed
     */
    KeyShortcut = 3,
    /**
     * AABB collision between entities
     */
    Touch = 4,
    /**
     * Entity in directional cone (radar)
     */
    Radar = 5,
    /**
     * Rapid double-click pattern detected
     */
    DoubleTap = 6,
    /**
     * Mouse button held down (long press)
     */
    LongPress = 7,
    /**
     * Right mouse button click
     */
    RightClick = 8,
}

/**
 * SignalByte WASM wrapper
 *
 * A binary signal with 6-tick history for edge detection and pattern matching.
 * This is the JavaScript-accessible version of the core SignalByte type.
 *
 * # JavaScript Example
 * ```javascript
 * const signal = new SignalByte();
 * signal.push(true);
 * signal.push(true);
 * signal.push(false);
 * console.log(signal.getHistory()); // 6 (0b00000110)
 * console.log(signal.isStableHigh(3)); // false
 * ```
 */
export class SignalByteWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns true if there is any edge (rising or falling)
     */
    any_edge(): boolean;
    /**
     * Returns the raw u8 value (for serialization)
     */
    as_u8(): number;
    /**
     * Counts how many ticks are 1 in the history
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00110111);
     * console.log(signal.countOnes()); // 5
     * ```
     */
    count_ones(): number;
    /**
     * Counts how many ticks are 0 in the history
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00110111);
     * console.log(signal.countZeros()); // 1
     * ```
     */
    count_zeros(): number;
    /**
     * Creates a SignalByte from a u8 value
     */
    static from(value: number): SignalByteWasm;
    /**
     * Returns the current signal state (tick T0, least significant bit)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * console.log(signal.getCurrent()); // true
     * ```
     */
    get_current(): boolean;
    /**
     * Returns the 6-bit history of the signal
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * signal.push(true);
     * signal.push(false);
     * console.log(signal.getHistory()); // 6 (0b00000110)
     * ```
     */
    get_history(): number;
    /**
     * Detects falling edge: 1 in T-1, 0 in T
     *
     * Pattern: [xxxx10]
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * signal.push(false);
     * console.log(signal.isFallingEdge()); // true
     * ```
     */
    is_falling_edge(): boolean;
    /**
     * Detects rising edge: 0 in T-1, 1 in T
     *
     * Pattern: [xxxx01]
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(false);
     * signal.push(true);
     * console.log(signal.isRisingEdge()); // true
     * ```
     */
    is_rising_edge(): boolean;
    /**
     * Alias for isSteadyHigh (for backward compatibility)
     */
    is_steady(ticks: number): boolean;
    /**
     * Checks if the signal has been steady (all 1s) for the last N ticks
     *
     * # Arguments
     * * `ticks` - Number of ticks to check (1-6)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00111111);
     * console.log(signal.isSteadyHigh(6)); // true
     * console.log(signal.isSteadyHigh(3)); // true
     * ```
     */
    is_steady_high(ticks: number): boolean;
    /**
     * Checks if the signal has been steady (all 0s) for the last N ticks
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * console.log(signal.isSteadyLow(3)); // true
     * ```
     */
    is_steady_low(ticks: number): boolean;
    /**
     * Creates a new SignalByte with all bits set to 0
     */
    constructor();
    /**
     * Pushes a new signal state, shifting the history left
     *
     * # Arguments
     * * `active` - true if the signal is active, false otherwise
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);  // 0b00000001
     * signal.push(true);  // 0b00000011
     * signal.push(false); // 0b00000110
     * ```
     */
    push(active: boolean): void;
    /**
     * Returns the size in bytes (always 1)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * console.log(signal.size()); // 1
     * ```
     */
    size(): number;
}

export class WasmBridge {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Check if redo is available
     */
    can_redo(): boolean;
    /**
     * Check if undo is available
     */
    can_undo(): boolean;
    /**
     * Clear all entities
     */
    clear(): void;
    /**
     * Clear all selections (deselect all entities)
     */
    clear_selection(): void;
    /**
     * Delete all selected entities
     */
    delete_selected(): void;
    /**
     * Detect available graphics backends
     */
    detect_available_backends(): object;
    /**
     * Duplicate an entity (create a copy at a slight offset)
     */
    duplicate_entity(entity_index: number): number;
    /**
     * Get the number of alive entities
     */
    entity_count(): number;
    /**
     * Get the active fill color (returns RGBA as hex string)
     */
    get_active_color(): string;
    /**
     * Get the active stroke color (returns RGBA as hex string)
     */
    get_active_stroke_color(): string;
    /**
     * Get the active stroke width
     */
    get_active_stroke_width(): number;
    /**
     * Get list of alive entity indices
     */
    get_alive_entities(): Uint32Array;
    /**
     * Get the camera center position
     */
    get_camera_center(): Array<any>;
    /**
     * Get the color of an entity (returns hex string)
     */
    get_color(entity_index: number): string;
    /**
     * Get entity color as hex string
     */
    get_entity_color_hex(entity_index: number): string;
    /**
     * Get entity label from string pool
     */
    get_entity_label(entity_index: number): string;
    /**
     * Get entity position in screen coordinates
     */
    get_entity_position_screen(entity_index: number): Array<any>;
    /**
     * Get entity position in world coordinates
     */
    get_entity_position_world(entity_index: number): Array<any>;
    /**
     * Get entity shape type
     */
    get_entity_shape(entity_index: number): number;
    /**
     * Get entity size in screen coordinates
     */
    get_entity_size_screen(entity_index: number): Array<any>;
    /**
     * Get entity size in world coordinates
     */
    get_entity_size_world(entity_index: number): Array<any>;
    /**
     * Get history state for UI feedback
     */
    get_history_state(): string;
    /**
     * Get a pointer to the SharedArrayBuffer for input events
     *
     * This returns a pointer to the InputRingBuffer that JavaScript can
     * write to directly via SharedArrayBuffer.
     */
    get_input_buffer_ptr(): number;
    /**
     * Get the size of the input buffer in bytes
     */
    static get_input_buffer_size(): number;
    /**
     * Get the list of selected entity IDs
     */
    get_selection(): Array<any>;
    /**
     * Get the stroke color of an entity (returns hex string)
     */
    get_stroke_color(entity_index: number): string;
    /**
     * Get the stroke width of an entity
     */
    get_stroke_width(entity_index: number): number;
    /**
     * Get the current tool type
     */
    get_tool(): string;
    /**
     * Get the current camera zoom level
     */
    get_zoom(): number;
    /**
     * Initialize the engine
     *
     * This should be called once when the application starts.
     */
    initialize(canvas_width: number, canvas_height: number): void;
    /**
     * Initialize graphics (uses WebGL2/Canvas 2D by default)
     *
     * This should be called after `initialize()` and after the canvas is mounted.
     */
    initialize_graphics(canvas: HTMLCanvasElement): void;
    /**
     * Initialize graphics with a specific backend
     *
     * Supported backends: "webgl2", "webgpu", "canvas2d", "auto"
     */
    initialize_graphics_with_backend(canvas: HTMLCanvasElement, backend: string): void;
    /**
     * Check if entity is selected
     */
    is_entity_selected(entity_index: number): boolean;
    /**
     * Check if entity is visible
     */
    is_entity_visible(entity_index: number): boolean;
    /**
     * Check if context recovery is in progress
     */
    is_recovering(): boolean;
    /**
     * Move an entity by the given delta
     */
    move_entity(entity_index: number, dx: number, dy: number): void;
    /**
     * Create a new WASM bridge
     */
    constructor();
    /**
     * Poll all events from the logic system
     *
     * Returns a JavaScript array of events emitted by the logic system
     * during the current frame. Call this once per frame after `tick()`.
     *
     * # Returns
     *
     * A JS array where each event is an object with:
     * - `type`: Event type (0=EntitySelected, 1=ProximityAlert, 2=DragStarted, 3=DragEnded, 4=EntityDestroyed, 5=BoxSelectionCompleted, 6=HoverChanged)
     * - `entityId`: Entity ID (or 0 for global events)
     * - `timestamp`: Timestamp in microseconds
     * - `data`: Event-specific data (varies by type)
     *
     * # Example
     *
     * ```javascript
     * // In your JavaScript/TypeScript code
     * const events = bridge.poll_events();
     * for (const event of events) {
     *     console.log('Event:', event.type, event.entityId);
     * }
     * ```
     */
    poll_events(): any;
    /**
     * Push an input event from JavaScript
     *
     * This is a higher-level alternative to directly writing to SharedArrayBuffer.
     * JavaScript can call this function to push input events.
     */
    push_input_event(event_type: number, x: number, y: number, buttons: number, modifiers: number): void;
    /**
     * Redo the last undone action
     */
    redo(): void;
    /**
     * Resize the engine and renderer
     */
    resize(width: number, height: number): void;
    /**
     * Add an entity to the selection (toggle mode)
     */
    select_entity(entity_index: number): void;
    /**
     * Serialize the current project
     */
    serialize_project(): Uint8Array;
    /**
     * Set the active fill color for new shapes
     */
    set_active_color(r: number, g: number, b: number, a: number): void;
    /**
     * Set the active stroke color for new shapes
     */
    set_active_stroke_color(r: number, g: number, b: number, a: number): void;
    /**
     * Set the active stroke width for new shapes
     */
    set_active_stroke_width(width: number): void;
    /**
     * Set the camera center position
     */
    set_camera_center(x: number, y: number): void;
    /**
     * Set the color of an entity
     */
    set_color(entity_index: number, r: number, g: number, b: number, a: number): void;
    /**
     * Set the selection state of an entity directly
     *
     * Uses DeltaMask for memory-efficient undo/redo via command queue.
     */
    set_entity_selected(entity_index: number, selected: boolean): void;
    /**
     * Set entity visibility
     */
    set_entity_visible(entity_index: number, visible: boolean): void;
    /**
     * Set the label of an entity
     */
    set_label(entity_index: number, label: string): void;
    /**
     * Set the position of an entity
     */
    set_position(entity_index: number, x: number, y: number): void;
    /**
     * Set the shape type of an entity
     */
    set_shape(entity_index: number, shape: number): void;
    /**
     * Set the size of an entity
     */
    set_size(entity_index: number, width: number, height: number): void;
    /**
     * Set the stroke color of an entity
     */
    set_stroke_color(entity_index: number, r: number, g: number, b: number, a: number): void;
    /**
     * Set the stroke width of an entity
     */
    set_stroke_width(entity_index: number, width: number): void;
    /**
     * Set the current tool type
     */
    set_tool(tool: string): void;
    /**
     * Set the camera zoom level
     */
    set_zoom(zoom: number): void;
    /**
     * Spawn a new entity at the given position
     */
    spawn_entity(x: number, y: number, width: number, height: number): number;
    /**
     * Run one frame of the engine
     *
     * This should be called from requestAnimationFrame.
     */
    tick(timestamp: number): void;
    /**
     * Undo the last action
     */
    undo(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_cameraconfig_free: (a: number, b: number) => void;
    readonly __wbg_controller_free: (a: number, b: number) => void;
    readonly __wbg_highlightconfig_free: (a: number, b: number) => void;
    readonly __wbg_jserror_free: (a: number, b: number) => void;
    readonly __wbg_logicmappingtablewasm_free: (a: number, b: number) => void;
    readonly __wbg_logicsystemwasm_free: (a: number, b: number) => void;
    readonly __wbg_moveconfig_free: (a: number, b: number) => void;
    readonly __wbg_propertyconfig_free: (a: number, b: number) => void;
    readonly __wbg_pulsewasm_free: (a: number, b: number) => void;
    readonly __wbg_signalbytewasm_free: (a: number, b: number) => void;
    readonly __wbg_wasmbridge_free: (a: number, b: number) => void;
    readonly cameraconfig_duration_ms: (a: number) => number;
    readonly cameraconfig_new: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly cameraconfig_smooth: (a: number) => number;
    readonly cameraconfig_target_x: (a: number) => number;
    readonly cameraconfig_target_y: (a: number) => number;
    readonly cameraconfig_zoom: (a: number) => number;
    readonly controller_and: (a: number) => number;
    readonly controller_blinky: (a: number) => number;
    readonly controller_controller_type: (a: number) => number;
    readonly controller_custom: (a: number, b: number, c: number, d: number) => number;
    readonly controller_custom_code: (a: number, b: number) => void;
    readonly controller_custom_name: (a: number, b: number) => void;
    readonly controller_debounce: (a: number) => number;
    readonly controller_direct: () => number;
    readonly controller_float_param1: (a: number) => number;
    readonly controller_float_param2: (a: number) => number;
    readonly controller_has_secondary_sensor: (a: number) => number;
    readonly controller_hysteresis: (a: number, b: number) => number;
    readonly controller_is_custom: (a: number) => number;
    readonly controller_not: () => number;
    readonly controller_numeric_param: (a: number) => number;
    readonly controller_or: (a: number) => number;
    readonly controller_pattern: (a: number) => number;
    readonly controller_secondary_sensor: (a: number) => number;
    readonly controller_threshold: (a: number) => number;
    readonly highlightconfig_color: (a: number) => number;
    readonly highlightconfig_new: (a: number, b: number, c: number) => number;
    readonly highlightconfig_opacity: (a: number) => number;
    readonly highlightconfig_restore_color: (a: number) => number;
    readonly jserror_message: (a: number, b: number) => void;
    readonly jserror_new: (a: number, b: number) => number;
    readonly logicmappingtablewasm_add_highlight: (a: number, b: number, c: number, d: number) => void;
    readonly logicmappingtablewasm_add_move: (a: number, b: number, c: number, d: number) => void;
    readonly logicmappingtablewasm_add_select: (a: number, b: number, c: number, d: number) => void;
    readonly logicmappingtablewasm_clear: (a: number) => void;
    readonly logicmappingtablewasm_clear_entity: (a: number, b: number) => void;
    readonly logicmappingtablewasm_connection_count: (a: number, b: number) => number;
    readonly logicmappingtablewasm_get_connected_entities: (a: number, b: number) => void;
    readonly logicmappingtablewasm_has_connection: (a: number, b: number, c: number) => number;
    readonly logicmappingtablewasm_is_empty: (a: number) => number;
    readonly logicmappingtablewasm_new: () => number;
    readonly logicmappingtablewasm_remove_connection: (a: number, b: number, c: number) => void;
    readonly logicsystemwasm_new: () => number;
    readonly logicsystemwasm_update: (a: number, b: bigint) => void;
    readonly moveconfig_constrain_x: (a: number) => number;
    readonly moveconfig_constrain_y: (a: number) => number;
    readonly moveconfig_new: (a: number, b: number, c: number) => number;
    readonly moveconfig_snap: (a: number) => number;
    readonly propertyconfig_new: (a: number, b: number, c: number) => number;
    readonly propertyconfig_property_name: (a: number, b: number) => void;
    readonly propertyconfig_value: (a: number) => number;
    readonly propertyvalue_from_bool: (a: number) => number;
    readonly propertyvalue_from_number: (a: number) => number;
    readonly propertyvalue_from_string: (a: number, b: number) => number;
    readonly pulsewasm_entity_id: (a: number) => number;
    readonly pulsewasm_is_active: (a: number) => number;
    readonly pulsewasm_new: (a: number, b: number, c: number, d: number) => number;
    readonly pulsewasm_sensor_id: (a: number) => number;
    readonly pulsewasm_timestamp: (a: number) => number;
    readonly signalbytewasm_any_edge: (a: number) => number;
    readonly signalbytewasm_as_u8: (a: number) => number;
    readonly signalbytewasm_count_ones: (a: number) => number;
    readonly signalbytewasm_count_zeros: (a: number) => number;
    readonly signalbytewasm_from: (a: number) => number;
    readonly signalbytewasm_get_current: (a: number) => number;
    readonly signalbytewasm_get_history: (a: number) => number;
    readonly signalbytewasm_is_falling_edge: (a: number) => number;
    readonly signalbytewasm_is_rising_edge: (a: number) => number;
    readonly signalbytewasm_is_steady: (a: number, b: number) => number;
    readonly signalbytewasm_is_steady_high: (a: number, b: number) => number;
    readonly signalbytewasm_is_steady_low: (a: number, b: number) => number;
    readonly signalbytewasm_new: () => number;
    readonly signalbytewasm_push: (a: number, b: number) => void;
    readonly signalbytewasm_size: (a: number) => number;
    readonly wasmbridge_can_redo: (a: number, b: number) => void;
    readonly wasmbridge_can_undo: (a: number, b: number) => void;
    readonly wasmbridge_clear: (a: number, b: number) => void;
    readonly wasmbridge_clear_selection: (a: number, b: number) => void;
    readonly wasmbridge_delete_selected: (a: number, b: number) => void;
    readonly wasmbridge_detect_available_backends: (a: number, b: number) => void;
    readonly wasmbridge_duplicate_entity: (a: number, b: number, c: number) => void;
    readonly wasmbridge_entity_count: (a: number, b: number) => void;
    readonly wasmbridge_get_active_color: (a: number, b: number) => void;
    readonly wasmbridge_get_active_stroke_color: (a: number, b: number) => void;
    readonly wasmbridge_get_active_stroke_width: (a: number, b: number) => void;
    readonly wasmbridge_get_alive_entities: (a: number, b: number) => void;
    readonly wasmbridge_get_camera_center: (a: number, b: number) => void;
    readonly wasmbridge_get_color: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_color_hex: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_label: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_position_screen: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_position_world: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_shape: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_size_screen: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_entity_size_world: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_history_state: (a: number, b: number) => void;
    readonly wasmbridge_get_input_buffer_ptr: (a: number) => number;
    readonly wasmbridge_get_selection: (a: number, b: number) => void;
    readonly wasmbridge_get_stroke_color: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_stroke_width: (a: number, b: number, c: number) => void;
    readonly wasmbridge_get_tool: (a: number, b: number) => void;
    readonly wasmbridge_get_zoom: (a: number, b: number) => void;
    readonly wasmbridge_initialize: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_initialize_graphics: (a: number, b: number, c: number) => void;
    readonly wasmbridge_initialize_graphics_with_backend: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_is_entity_selected: (a: number, b: number, c: number) => void;
    readonly wasmbridge_is_entity_visible: (a: number, b: number, c: number) => void;
    readonly wasmbridge_is_recovering: (a: number) => number;
    readonly wasmbridge_move_entity: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_new: () => number;
    readonly wasmbridge_poll_events: (a: number) => number;
    readonly wasmbridge_push_input_event: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmbridge_redo: (a: number, b: number) => void;
    readonly wasmbridge_resize: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_select_entity: (a: number, b: number, c: number) => void;
    readonly wasmbridge_serialize_project: (a: number, b: number) => void;
    readonly wasmbridge_set_active_color: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasmbridge_set_active_stroke_color: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasmbridge_set_active_stroke_width: (a: number, b: number, c: number) => void;
    readonly wasmbridge_set_camera_center: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_set_color: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmbridge_set_entity_selected: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_set_entity_visible: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_set_label: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_set_position: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_set_shape: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_set_size: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_set_stroke_color: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly wasmbridge_set_stroke_width: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_set_tool: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_set_zoom: (a: number, b: number, c: number) => void;
    readonly wasmbridge_spawn_entity: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasmbridge_tick: (a: number, b: number, c: number) => void;
    readonly wasmbridge_undo: (a: number, b: number) => void;
    readonly wasmbridge_get_input_buffer_size: () => number;
    readonly __wbg_propertyvalue_free: (a: number, b: number) => void;
    readonly propertyvalue_value: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_83: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_425: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_424: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
