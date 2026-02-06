/* @ts-self-types="./archflow_web.d.ts" */

/**
 * Actuator types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { ActuatorType } from '@archflow/sdk';
 *
 * const actuator = ActuatorType.Highlight;
 * ```
 * @enum {0 | 1 | 2}
 */
export const ActuatorType = Object.freeze({
    /**
     * Highlight actuator - changes entity color
     */
    Highlight: 0, "0": "Highlight",
    /**
     * Select actuator - marks entity as selected
     */
    Select: 1, "1": "Select",
    /**
     * Move actuator - moves entity (drag operation)
     */
    Move: 2, "2": "Move",
});

/**
 * Configuration for camera actuator
 */
export class CameraConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CameraConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_cameraconfig_free(ptr, 0);
    }
    /**
     * Get duration in milliseconds
     * @returns {number}
     */
    duration_ms() {
        const ret = wasm.cameraconfig_duration_ms(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Creates a new camera configuration
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} zoom
     * @param {number} duration_ms
     * @param {number} smooth
     */
    constructor(target_x, target_y, zoom, duration_ms, smooth) {
        const ret = wasm.cameraconfig_new(target_x, target_y, zoom, duration_ms, smooth);
        this.__wbg_ptr = ret >>> 0;
        CameraConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get smoothing factor (0.0 - 1.0)
     * @returns {number}
     */
    smooth() {
        const ret = wasm.cameraconfig_smooth(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get target X position
     * @returns {number}
     */
    target_x() {
        const ret = wasm.cameraconfig_target_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get target Y position
     * @returns {number}
     */
    target_y() {
        const ret = wasm.cameraconfig_target_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get zoom level
     * @returns {number}
     */
    zoom() {
        const ret = wasm.cameraconfig_zoom(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) CameraConfig.prototype[Symbol.dispose] = CameraConfig.prototype.free;

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
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Controller.prototype);
        obj.__wbg_ptr = ptr;
        ControllerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ControllerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_controller_free(ptr, 0);
    }
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
     * @param {SensorType} sensor
     * @returns {Controller}
     */
    static and(sensor) {
        const ret = wasm.controller_and(sensor);
        return Controller.__wrap(ret);
    }
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
     * @param {number} interval
     * @returns {Controller}
     */
    static blinky(interval) {
        const ret = wasm.controller_blinky(interval);
        return Controller.__wrap(ret);
    }
    /**
     * Returns the controller type
     * @returns {ControllerType}
     */
    controller_type() {
        const ret = wasm.controller_controller_type(this.__wbg_ptr);
        return ret;
    }
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
     * @param {string} name
     * @param {string} code
     * @returns {Controller}
     */
    static custom(name, code) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(code, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.controller_custom(ptr0, len0, ptr1, len1);
        return Controller.__wrap(ret);
    }
    /**
     * Returns the custom code (for Custom controllers)
     * @returns {string | undefined}
     */
    custom_code() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.controller_custom_code(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Returns the custom name (for Custom controllers)
     * @returns {string | undefined}
     */
    custom_name() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.controller_custom_name(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
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
     * @param {number} ticks
     * @returns {Controller}
     */
    static debounce(ticks) {
        const ret = wasm.controller_debounce(ticks);
        return Controller.__wrap(ret);
    }
    /**
     * Creates a new Direct controller (pass-through)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Direct();
     * ```
     * @returns {Controller}
     */
    static direct() {
        const ret = wasm.controller_direct();
        return Controller.__wrap(ret);
    }
    /**
     * Returns the first float parameter (for Hysteresis high, Threshold value)
     * @returns {number}
     */
    float_param1() {
        const ret = wasm.controller_float_param1(this.__wbg_ptr);
        return ret;
    }
    /**
     * Returns the second float parameter (for Hysteresis low)
     * @returns {number}
     */
    float_param2() {
        const ret = wasm.controller_float_param2(this.__wbg_ptr);
        return ret;
    }
    /**
     * Checks if this controller has a secondary sensor
     * @returns {boolean}
     */
    has_secondary_sensor() {
        const ret = wasm.controller_has_secondary_sensor(this.__wbg_ptr);
        return ret !== 0;
    }
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
     * @param {number} high
     * @param {number} low
     * @returns {Controller}
     */
    static hysteresis(high, low) {
        const ret = wasm.controller_hysteresis(high, low);
        return Controller.__wrap(ret);
    }
    /**
     * Checks if this controller is a Custom type
     * @returns {boolean}
     */
    is_custom() {
        const ret = wasm.controller_is_custom(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates a NOT controller (inverts the signal)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Not();
     * // Inverts the primary sensor signal
     * ```
     * @returns {Controller}
     */
    static not() {
        const ret = wasm.controller_not();
        return Controller.__wrap(ret);
    }
    /**
     * Returns the numeric parameter (for Blinky, Debounce, Pattern)
     * @returns {number}
     */
    numeric_param() {
        const ret = wasm.controller_numeric_param(this.__wbg_ptr);
        return ret;
    }
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
     * @param {SensorType} sensor
     * @returns {Controller}
     */
    static or(sensor) {
        const ret = wasm.controller_or(sensor);
        return Controller.__wrap(ret);
    }
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
     * @param {number} mask
     * @returns {Controller}
     */
    static pattern(mask) {
        const ret = wasm.controller_pattern(mask);
        return Controller.__wrap(ret);
    }
    /**
     * Returns the secondary sensor (if any)
     *
     * Returns `null` if there is no secondary sensor.
     * @returns {SensorType | undefined}
     */
    secondary_sensor() {
        const ret = wasm.controller_secondary_sensor(this.__wbg_ptr);
        return ret === 9 ? undefined : ret;
    }
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
     * @param {number} value
     * @returns {Controller}
     */
    static threshold(value) {
        const ret = wasm.controller_threshold(value);
        return Controller.__wrap(ret);
    }
}
if (Symbol.dispose) Controller.prototype[Symbol.dispose] = Controller.prototype.free;

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
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9}
 */
export const ControllerType = Object.freeze({
    /**
     * Pass through the primary sensor signal
     */
    Direct: 0, "0": "Direct",
    /**
     * AND logic: primary AND other sensor must both be active
     */
    And: 1, "1": "And",
    /**
     * OR logic: primary OR other sensor must be active
     */
    Or: 2, "2": "Or",
    /**
     * NOT logic: invert the primary sensor signal
     */
    Not: 3, "3": "Not",
    /**
     * Blinky: Toggles active/inactive at regular intervals
     */
    Blinky: 4, "4": "Blinky",
    /**
     * Debounce: Requires signal to be stable for N ticks
     */
    Debounce: 5, "5": "Debounce",
    /**
     * Hysteresis: Different activation/deactivation thresholds
     */
    Hysteresis: 6, "6": "Hysteresis",
    /**
     * Threshold: Requires minimum stability percentage
     */
    Threshold: 7, "7": "Threshold",
    /**
     * Pattern: Matches specific binary pattern in history
     */
    Pattern: 8, "8": "Pattern",
    /**
     * Custom: JavaScript sandbox evaluation
     */
    Custom: 9, "9": "Custom",
});

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
 * @enum {0 | 1 | 2 | 3 | 4 | 5}
 */
export const ExtendedActuatorType = Object.freeze({
    /**
     * Highlight actuator - changes entity color
     */
    Highlight: 0, "0": "Highlight",
    /**
     * Select actuator - marks entity as selected
     */
    Select: 1, "1": "Select",
    /**
     * Move actuator - moves entity (drag operation)
     */
    Move: 2, "2": "Move",
    /**
     * Camera actuator - moves camera
     */
    Camera: 3, "3": "Camera",
    /**
     * Property actuator - sets entity property
     */
    Property: 4, "4": "Property",
    /**
     * State actuator - changes entity state
     */
    State: 5, "5": "State",
});

/**
 * Configuration for highlight actuator
 */
export class HighlightConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        HighlightConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_highlightconfig_free(ptr, 0);
    }
    /**
     * Get the highlight color (ARGB)
     * @returns {number}
     */
    color() {
        const ret = wasm.highlightconfig_color(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Creates a new highlight configuration
     * @param {number} color
     * @param {number} restore_color
     * @param {number} opacity
     */
    constructor(color, restore_color, opacity) {
        const ret = wasm.highlightconfig_new(color, restore_color, opacity);
        this.__wbg_ptr = ret >>> 0;
        HighlightConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get the opacity (0.0 - 1.0)
     * @returns {number}
     */
    opacity() {
        const ret = wasm.highlightconfig_opacity(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get the restore color (ARGB)
     * @returns {number}
     */
    restore_color() {
        const ret = wasm.highlightconfig_restore_color(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) HighlightConfig.prototype[Symbol.dispose] = HighlightConfig.prototype.free;

/**
 * Custom error type for JavaScript
 */
export class JsError {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JsError.prototype);
        obj.__wbg_ptr = ptr;
        JsErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsErrorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jserror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    message() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jserror_message(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} message
     */
    constructor(message) {
        const ptr0 = passStringToWasm0(message, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.jserror_new(ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        JsErrorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) JsError.prototype[Symbol.dispose] = JsError.prototype.free;

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
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LogicMappingTableWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_logicmappingtablewasm_free(ptr, 0);
    }
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
     * @param {number} entity_id
     * @param {SensorType} sensor
     * @param {Controller} controller
     */
    add_highlight(entity_id, sensor, controller) {
        _assertClass(controller, Controller);
        var ptr0 = controller.__destroy_into_raw();
        wasm.logicmappingtablewasm_add_highlight(this.__wbg_ptr, entity_id, sensor, ptr0);
    }
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
     * @param {number} entity_id
     * @param {SensorType} sensor
     * @param {Controller} controller
     */
    add_move(entity_id, sensor, controller) {
        _assertClass(controller, Controller);
        var ptr0 = controller.__destroy_into_raw();
        wasm.logicmappingtablewasm_add_move(this.__wbg_ptr, entity_id, sensor, ptr0);
    }
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
     * @param {number} entity_id
     * @param {SensorType} sensor
     * @param {Controller} controller
     */
    add_select(entity_id, sensor, controller) {
        _assertClass(controller, Controller);
        var ptr0 = controller.__destroy_into_raw();
        wasm.logicmappingtablewasm_add_select(this.__wbg_ptr, entity_id, sensor, ptr0);
    }
    /**
     * Clears all connections from the table
     *
     * # JavaScript Example
     * ```javascript
     * table.clear();
     * ```
     */
    clear() {
        wasm.logicmappingtablewasm_clear(this.__wbg_ptr);
    }
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
     * @param {number} entity_id
     */
    clear_entity(entity_id) {
        wasm.logicmappingtablewasm_clear_entity(this.__wbg_ptr, entity_id);
    }
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
     * @param {number} entity_id
     * @returns {number}
     */
    connection_count(entity_id) {
        const ret = wasm.logicmappingtablewasm_connection_count(this.__wbg_ptr, entity_id);
        return ret >>> 0;
    }
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
     * @returns {Uint32Array}
     */
    get_connected_entities() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.logicmappingtablewasm_get_connected_entities(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
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
     * @param {number} entity_id
     * @param {SensorType} sensor
     * @returns {boolean}
     */
    has_connection(entity_id, sensor) {
        const ret = wasm.logicmappingtablewasm_has_connection(this.__wbg_ptr, entity_id, sensor);
        return ret !== 0;
    }
    /**
     * Checks if the table is empty
     *
     * # JavaScript Example
     * ```javascript
     * const isEmpty = table.isEmpty();
     * ```
     * @returns {boolean}
     */
    is_empty() {
        const ret = wasm.logicmappingtablewasm_is_empty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates a new LogicMappingTable
     *
     * # JavaScript Example
     * ```javascript
     * const table = new LogicMappingTable();
     * ```
     */
    constructor() {
        const ret = wasm.logicmappingtablewasm_new();
        this.__wbg_ptr = ret >>> 0;
        LogicMappingTableWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
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
     * @param {number} entity_id
     * @param {SensorType} sensor
     */
    remove_connection(entity_id, sensor) {
        wasm.logicmappingtablewasm_remove_connection(this.__wbg_ptr, entity_id, sensor);
    }
}
if (Symbol.dispose) LogicMappingTableWasm.prototype[Symbol.dispose] = LogicMappingTableWasm.prototype.free;

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
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LogicSystemWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_logicsystemwasm_free(ptr, 0);
    }
    /**
     * Creates a new LogicSystem
     *
     * # JavaScript Example
     * ```javascript
     * const system = new LogicSystem();
     * ```
     */
    constructor() {
        const ret = wasm.logicsystemwasm_new();
        this.__wbg_ptr = ret >>> 0;
        LogicSystemWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
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
     * @param {bigint} timestamp_ms
     */
    update(timestamp_ms) {
        wasm.logicsystemwasm_update(this.__wbg_ptr, timestamp_ms);
    }
}
if (Symbol.dispose) LogicSystemWasm.prototype[Symbol.dispose] = LogicSystemWasm.prototype.free;

/**
 * Configuration for move actuator
 */
export class MoveConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MoveConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_moveconfig_free(ptr, 0);
    }
    /**
     * Whether X axis is constrained
     * @returns {boolean}
     */
    constrain_x() {
        const ret = wasm.moveconfig_constrain_x(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether Y axis is constrained
     * @returns {boolean}
     */
    constrain_y() {
        const ret = wasm.moveconfig_constrain_y(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates a new move configuration
     * @param {number} snap
     * @param {boolean} constrain_x
     * @param {boolean} constrain_y
     */
    constructor(snap, constrain_x, constrain_y) {
        const ret = wasm.moveconfig_new(snap, constrain_x, constrain_y);
        this.__wbg_ptr = ret >>> 0;
        MoveConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get snap value in pixels
     * @returns {number}
     */
    snap() {
        const ret = wasm.moveconfig_snap(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) MoveConfig.prototype[Symbol.dispose] = MoveConfig.prototype.free;

/**
 * Configuration for property actuator
 */
export class PropertyConfig {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PropertyConfigFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_propertyconfig_free(ptr, 0);
    }
    /**
     * Creates a new property configuration
     * @param {string} property_name
     * @param {PropertyValue} value
     */
    constructor(property_name, value) {
        const ptr0 = passStringToWasm0(property_name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(value, PropertyValue);
        var ptr1 = value.__destroy_into_raw();
        const ret = wasm.propertyconfig_new(ptr0, len0, ptr1);
        this.__wbg_ptr = ret >>> 0;
        PropertyConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get property name
     * @returns {string}
     */
    property_name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.propertyconfig_property_name(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get property value
     * @returns {PropertyValue}
     */
    value() {
        const ret = wasm.propertyconfig_value(this.__wbg_ptr);
        return PropertyValue.__wrap(ret);
    }
}
if (Symbol.dispose) PropertyConfig.prototype[Symbol.dispose] = PropertyConfig.prototype.free;

/**
 * Property value wrapper for WASM
 */
export class PropertyValue {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PropertyValue.prototype);
        obj.__wbg_ptr = ptr;
        PropertyValueFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PropertyValueFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_propertyvalue_free(ptr, 0);
    }
    /**
     * Create a boolean property value
     * @param {boolean} value
     * @returns {PropertyValue}
     */
    static from_bool(value) {
        const ret = wasm.propertyvalue_from_bool(value);
        return PropertyValue.__wrap(ret);
    }
    /**
     * Create a number property value
     * @param {number} value
     * @returns {PropertyValue}
     */
    static from_number(value) {
        const ret = wasm.propertyvalue_from_number(value);
        return PropertyValue.__wrap(ret);
    }
    /**
     * Create a string property value
     * @param {string} value
     * @returns {PropertyValue}
     */
    static from_string(value) {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.propertyvalue_from_string(ptr0, len0);
        return PropertyValue.__wrap(ret);
    }
    /**
     * Get the raw value string
     * @returns {string}
     */
    value() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.jserror_message(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) PropertyValue.prototype[Symbol.dispose] = PropertyValue.prototype.free;

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
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PulseWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pulsewasm_free(ptr, 0);
    }
    /**
     * Get the entity ID
     * @returns {number}
     */
    entity_id() {
        const ret = wasm.pulsewasm_entity_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Check if the pulse is active (positive edge)
     *
     * Returns true for positive pulses (sensor became TRUE)
     * Returns false for negative pulses (sensor became FALSE)
     * @returns {boolean}
     */
    is_active() {
        const ret = wasm.pulsewasm_is_active(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates a new PulseWasm
     *
     * # JavaScript Example
     * ```javascript
     * const pulse = new PulseWasm(123, 5, true, 1000);
     * ```
     * @param {number} entity_id
     * @param {number} sensor_id
     * @param {boolean} state
     * @param {number} timestamp
     */
    constructor(entity_id, sensor_id, state, timestamp) {
        const ret = wasm.pulsewasm_new(entity_id, sensor_id, state, timestamp);
        this.__wbg_ptr = ret >>> 0;
        PulseWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get the sensor ID
     * @returns {number}
     */
    sensor_id() {
        const ret = wasm.pulsewasm_sensor_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the timestamp
     * @returns {number}
     */
    timestamp() {
        const ret = wasm.pulsewasm_timestamp(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) PulseWasm.prototype[Symbol.dispose] = PulseWasm.prototype.free;

/**
 * Select mode for selection actuator (matches core SelectMode)
 * @enum {0 | 1 | 2}
 */
export const SelectModeWasm = Object.freeze({
    /**
     * Single selection (replaces current selection)
     */
    Single: 0, "0": "Single",
    /**
     * Multi selection (adds to current selection)
     */
    Multi: 1, "1": "Multi",
    /**
     * Replace selection (clears and selects new)
     */
    Replace: 2, "2": "Replace",
});

/**
 * Sensor types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { SensorType } from '@archflow/sdk';
 *
 * const sensor = SensorType.MouseOver;
 * ```
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8}
 */
export const SensorType = Object.freeze({
    /**
     * Mouse is hovering over an entity
     */
    MouseOver: 0, "0": "MouseOver",
    /**
     * Mouse button was clicked on an entity
     */
    MouseClick: 1, "1": "MouseClick",
    /**
     * Another entity is within proximity radius
     */
    Proximity: 2, "2": "Proximity",
    /**
     * Keyboard shortcut was pressed
     */
    KeyShortcut: 3, "3": "KeyShortcut",
    /**
     * AABB collision between entities
     */
    Touch: 4, "4": "Touch",
    /**
     * Entity in directional cone (radar)
     */
    Radar: 5, "5": "Radar",
    /**
     * Rapid double-click pattern detected
     */
    DoubleTap: 6, "6": "DoubleTap",
    /**
     * Mouse button held down (long press)
     */
    LongPress: 7, "7": "LongPress",
    /**
     * Right mouse button click
     */
    RightClick: 8, "8": "RightClick",
});

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
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SignalByteWasm.prototype);
        obj.__wbg_ptr = ptr;
        SignalByteWasmFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SignalByteWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_signalbytewasm_free(ptr, 0);
    }
    /**
     * Returns true if there is any edge (rising or falling)
     * @returns {boolean}
     */
    any_edge() {
        const ret = wasm.signalbytewasm_any_edge(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns the raw u8 value (for serialization)
     * @returns {number}
     */
    as_u8() {
        const ret = wasm.signalbytewasm_as_u8(this.__wbg_ptr);
        return ret;
    }
    /**
     * Counts how many ticks are 1 in the history
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00110111);
     * console.log(signal.countOnes()); // 5
     * ```
     * @returns {number}
     */
    count_ones() {
        const ret = wasm.signalbytewasm_count_ones(this.__wbg_ptr);
        return ret;
    }
    /**
     * Counts how many ticks are 0 in the history
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00110111);
     * console.log(signal.countZeros()); // 1
     * ```
     * @returns {number}
     */
    count_zeros() {
        const ret = wasm.signalbytewasm_count_zeros(this.__wbg_ptr);
        return ret;
    }
    /**
     * Creates a SignalByte from a u8 value
     * @param {number} value
     * @returns {SignalByteWasm}
     */
    static from(value) {
        const ret = wasm.signalbytewasm_from(value);
        return SignalByteWasm.__wrap(ret);
    }
    /**
     * Returns the current signal state (tick T0, least significant bit)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * console.log(signal.getCurrent()); // true
     * ```
     * @returns {boolean}
     */
    get_current() {
        const ret = wasm.signalbytewasm_get_current(this.__wbg_ptr);
        return ret !== 0;
    }
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
     * @returns {number}
     */
    get_history() {
        const ret = wasm.signalbytewasm_get_history(this.__wbg_ptr);
        return ret;
    }
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
     * @returns {boolean}
     */
    is_falling_edge() {
        const ret = wasm.signalbytewasm_is_falling_edge(this.__wbg_ptr);
        return ret !== 0;
    }
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
     * @returns {boolean}
     */
    is_rising_edge() {
        const ret = wasm.signalbytewasm_is_rising_edge(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Alias for isSteadyHigh (for backward compatibility)
     * @param {number} ticks
     * @returns {boolean}
     */
    is_steady(ticks) {
        const ret = wasm.signalbytewasm_is_steady(this.__wbg_ptr, ticks);
        return ret !== 0;
    }
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
     * @param {number} ticks
     * @returns {boolean}
     */
    is_steady_high(ticks) {
        const ret = wasm.signalbytewasm_is_steady_high(this.__wbg_ptr, ticks);
        return ret !== 0;
    }
    /**
     * Checks if the signal has been steady (all 0s) for the last N ticks
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * console.log(signal.isSteadyLow(3)); // true
     * ```
     * @param {number} ticks
     * @returns {boolean}
     */
    is_steady_low(ticks) {
        const ret = wasm.signalbytewasm_is_steady_low(this.__wbg_ptr, ticks);
        return ret !== 0;
    }
    /**
     * Creates a new SignalByte with all bits set to 0
     */
    constructor() {
        const ret = wasm.signalbytewasm_new();
        this.__wbg_ptr = ret >>> 0;
        SignalByteWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
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
     * @param {boolean} active
     */
    push(active) {
        wasm.signalbytewasm_push(this.__wbg_ptr, active);
    }
    /**
     * Returns the size in bytes (always 1)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * console.log(signal.size()); // 1
     * ```
     * @returns {number}
     */
    size() {
        const ret = wasm.signalbytewasm_size(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) SignalByteWasm.prototype[Symbol.dispose] = SignalByteWasm.prototype.free;

export class WasmBridge {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmBridgeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmbridge_free(ptr, 0);
    }
    /**
     * Check if redo is available
     * @returns {boolean}
     */
    can_redo() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_can_redo(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Check if undo is available
     * @returns {boolean}
     */
    can_undo() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_can_undo(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Clear all entities
     */
    clear() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_clear(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Clear all selections (deselect all entities)
     */
    clear_selection() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_clear_selection(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Delete all selected entities
     */
    delete_selected() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_delete_selected(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Detect available graphics backends
     * @returns {object}
     */
    detect_available_backends() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_detect_available_backends(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Duplicate an entity (create a copy at a slight offset)
     * @param {number} entity_index
     * @returns {number}
     */
    duplicate_entity(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_duplicate_entity(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the number of alive entities
     * @returns {number}
     */
    entity_count() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_entity_count(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the active fill color (returns RGBA as hex string)
     * @returns {string}
     */
    get_active_color() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_active_color(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the active stroke color (returns RGBA as hex string)
     * @returns {string}
     */
    get_active_stroke_color() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_active_stroke_color(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the active stroke width
     * @returns {number}
     */
    get_active_stroke_width() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_active_stroke_width(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getFloat32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get list of alive entity indices
     * @returns {Uint32Array}
     */
    get_alive_entities() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_alive_entities(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v1 = getArrayU32FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the camera center position
     * @returns {Array<any>}
     */
    get_camera_center() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_camera_center(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the color of an entity (returns hex string)
     * @param {number} entity_index
     * @returns {string}
     */
    get_color(entity_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_color(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get entity color as hex string
     * @param {number} entity_index
     * @returns {string}
     */
    get_entity_color_hex(entity_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_color_hex(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get entity label from string pool
     * @param {number} entity_index
     * @returns {string}
     */
    get_entity_label(entity_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_label(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get entity position in screen coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_position_screen(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_position_screen(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get entity position in world coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_position_world(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_position_world(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get entity shape type
     * @param {number} entity_index
     * @returns {number}
     */
    get_entity_shape(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_shape(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get entity size in screen coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_size_screen(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_size_screen(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get entity size in world coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_size_world(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_entity_size_world(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get history state for UI feedback
     * @returns {string}
     */
    get_history_state() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_history_state(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get a pointer to the SharedArrayBuffer for input events
     *
     * This returns a pointer to the InputRingBuffer that JavaScript can
     * write to directly via SharedArrayBuffer.
     * @returns {number}
     */
    get_input_buffer_ptr() {
        const ret = wasm.wasmbridge_get_input_buffer_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the size of the input buffer in bytes
     * @returns {number}
     */
    static get_input_buffer_size() {
        const ret = wasm.wasmbridge_get_input_buffer_size();
        return ret >>> 0;
    }
    /**
     * Get the list of selected entity IDs
     * @returns {Array<any>}
     */
    get_selection() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_selection(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the stroke color of an entity (returns hex string)
     * @param {number} entity_index
     * @returns {string}
     */
    get_stroke_color(entity_index) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_stroke_color(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the stroke width of an entity
     * @param {number} entity_index
     * @returns {number}
     */
    get_stroke_width(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_stroke_width(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getFloat32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the current tool type
     * @returns {string}
     */
    get_tool() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_tool(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the current camera zoom level
     * @returns {number}
     */
    get_zoom() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_get_zoom(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getFloat32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Initialize the engine
     *
     * This should be called once when the application starts.
     * @param {number} canvas_width
     * @param {number} canvas_height
     */
    initialize(canvas_width, canvas_height) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_initialize(retptr, this.__wbg_ptr, canvas_width, canvas_height);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Initialize graphics (uses WebGL2/Canvas 2D by default)
     *
     * This should be called after `initialize()` and after the canvas is mounted.
     * @param {HTMLCanvasElement} canvas
     */
    initialize_graphics(canvas) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_initialize_graphics(retptr, this.__wbg_ptr, addHeapObject(canvas));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Initialize graphics with a specific backend
     *
     * Supported backends: "webgl2", "webgpu", "canvas2d", "auto"
     * @param {HTMLCanvasElement} canvas
     * @param {string} backend
     */
    initialize_graphics_with_backend(canvas, backend) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(backend, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmbridge_initialize_graphics_with_backend(retptr, this.__wbg_ptr, addHeapObject(canvas), ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Check if entity is selected
     * @param {number} entity_index
     * @returns {boolean}
     */
    is_entity_selected(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_is_entity_selected(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Check if entity is visible
     * @param {number} entity_index
     * @returns {boolean}
     */
    is_entity_visible(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_is_entity_visible(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Check if context recovery is in progress
     * @returns {boolean}
     */
    is_recovering() {
        const ret = wasm.wasmbridge_is_recovering(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Move an entity by the given delta
     * @param {number} entity_index
     * @param {number} dx
     * @param {number} dy
     */
    move_entity(entity_index, dx, dy) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_move_entity(retptr, this.__wbg_ptr, entity_index, dx, dy);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Create a new WASM bridge
     */
    constructor() {
        const ret = wasm.wasmbridge_new();
        this.__wbg_ptr = ret >>> 0;
        WasmBridgeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
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
     * @returns {any}
     */
    poll_events() {
        const ret = wasm.wasmbridge_poll_events(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Push an input event from JavaScript
     *
     * This is a higher-level alternative to directly writing to SharedArrayBuffer.
     * JavaScript can call this function to push input events.
     * @param {number} event_type
     * @param {number} x
     * @param {number} y
     * @param {number} buttons
     * @param {number} modifiers
     */
    push_input_event(event_type, x, y, buttons, modifiers) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_push_input_event(retptr, this.__wbg_ptr, event_type, x, y, buttons, modifiers);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Redo the last undone action
     */
    redo() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_redo(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Resize the engine and renderer
     * @param {number} width
     * @param {number} height
     */
    resize(width, height) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_resize(retptr, this.__wbg_ptr, width, height);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Add an entity to the selection (toggle mode)
     * @param {number} entity_index
     */
    select_entity(entity_index) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_select_entity(retptr, this.__wbg_ptr, entity_index);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Serialize the current project
     * @returns {Uint8Array}
     */
    serialize_project() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_serialize_project(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the active fill color for new shapes
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} a
     */
    set_active_color(r, g, b, a) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_active_color(retptr, this.__wbg_ptr, r, g, b, a);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the active stroke color for new shapes
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} a
     */
    set_active_stroke_color(r, g, b, a) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_active_stroke_color(retptr, this.__wbg_ptr, r, g, b, a);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the active stroke width for new shapes
     * @param {number} width
     */
    set_active_stroke_width(width) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_active_stroke_width(retptr, this.__wbg_ptr, width);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the camera center position
     * @param {number} x
     * @param {number} y
     */
    set_camera_center(x, y) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_camera_center(retptr, this.__wbg_ptr, x, y);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the color of an entity
     * @param {number} entity_index
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} a
     */
    set_color(entity_index, r, g, b, a) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_color(retptr, this.__wbg_ptr, entity_index, r, g, b, a);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the selection state of an entity directly
     *
     * Uses DeltaMask for memory-efficient undo/redo via command queue.
     * @param {number} entity_index
     * @param {boolean} selected
     */
    set_entity_selected(entity_index, selected) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_entity_selected(retptr, this.__wbg_ptr, entity_index, selected);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set entity visibility
     * @param {number} entity_index
     * @param {boolean} visible
     */
    set_entity_visible(entity_index, visible) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_entity_visible(retptr, this.__wbg_ptr, entity_index, visible);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the label of an entity
     * @param {number} entity_index
     * @param {string} label
     */
    set_label(entity_index, label) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(label, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmbridge_set_label(retptr, this.__wbg_ptr, entity_index, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the position of an entity
     * @param {number} entity_index
     * @param {number} x
     * @param {number} y
     */
    set_position(entity_index, x, y) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_position(retptr, this.__wbg_ptr, entity_index, x, y);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the shape type of an entity
     * @param {number} entity_index
     * @param {number} shape
     */
    set_shape(entity_index, shape) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_shape(retptr, this.__wbg_ptr, entity_index, shape);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the size of an entity
     * @param {number} entity_index
     * @param {number} width
     * @param {number} height
     */
    set_size(entity_index, width, height) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_size(retptr, this.__wbg_ptr, entity_index, width, height);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the stroke color of an entity
     * @param {number} entity_index
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} a
     */
    set_stroke_color(entity_index, r, g, b, a) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_stroke_color(retptr, this.__wbg_ptr, entity_index, r, g, b, a);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the stroke width of an entity
     * @param {number} entity_index
     * @param {number} width
     */
    set_stroke_width(entity_index, width) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_stroke_width(retptr, this.__wbg_ptr, entity_index, width);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the current tool type
     * @param {string} tool
     */
    set_tool(tool) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(tool, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmbridge_set_tool(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the camera zoom level
     * @param {number} zoom
     */
    set_zoom(zoom) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_set_zoom(retptr, this.__wbg_ptr, zoom);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Spawn a new entity at the given position
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     * @returns {number}
     */
    spawn_entity(x, y, width, height) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_spawn_entity(retptr, this.__wbg_ptr, x, y, width, height);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Run one frame of the engine
     *
     * This should be called from requestAnimationFrame.
     * @param {number} timestamp
     */
    tick(timestamp) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_tick(retptr, this.__wbg_ptr, timestamp);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Undo the last action
     */
    undo() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmbridge_undo(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) WasmBridge.prototype[Symbol.dispose] = WasmBridge.prototype.free;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_d9b87ff7982e3b21: function(arg0) {
            getObject(arg0)._wbg_cb_unref();
        },
        __wbg_addEventListener_3acb0aad4483804c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
        }, arguments); },
        __wbg_attachShader_b36058e5c9eeaf54: function(arg0, arg1, arg2) {
            getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
        },
        __wbg_bindBuffer_c9068e8712a034f5: function(arg0, arg1, arg2) {
            getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
        },
        __wbg_bindVertexArray_78220d1edb1d2382: function(arg0, arg1) {
            getObject(arg0).bindVertexArray(getObject(arg1));
        },
        __wbg_blendFunc_2ef59299d10c662d: function(arg0, arg1, arg2) {
            getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_bufferData_98f6c413a8f0f139: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
        },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_clearColor_404a3b16d43db93b: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).clearColor(arg1, arg2, arg3, arg4);
        },
        __wbg_clear_7187030f892c5ca0: function(arg0, arg1) {
            getObject(arg0).clear(arg1 >>> 0);
        },
        __wbg_compileShader_94718a93495d565d: function(arg0, arg1) {
            getObject(arg0).compileShader(getObject(arg1));
        },
        __wbg_createBuffer_26534c05e01b8559: function(arg0) {
            const ret = getObject(arg0).createBuffer();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_createElement_49f60fdcaae809c8: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_createProgram_9b7710a1f2701c2c: function(arg0) {
            const ret = getObject(arg0).createProgram();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_createShader_e3ac08ed8c5b14b2: function(arg0, arg1) {
            const ret = getObject(arg0).createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_createVertexArray_ad5294951ae57497: function(arg0) {
            const ret = getObject(arg0).createVertexArray();
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_document_ee35a3d3ae34ef6c: function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_drawArraysInstanced_ec30adc616ec58d5: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_enableVertexAttribArray_475e06c31777296d: function(arg0, arg1) {
            getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_enable_d1ac04dfdd2fb3ae: function(arg0, arg1) {
            getObject(arg0).enable(arg1 >>> 0);
        },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_9a7fe3f932034cde: function(arg0) {
            console.error(getObject(arg0));
        },
        __wbg_getContext_2a5764d48600bc43: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_getError_bba8594facbfd5e1: function(arg0) {
            const ret = getObject(arg0).getError();
            return ret;
        },
        __wbg_getProgramInfoLog_2ffa30e3abb8b5c2: function(arg0, arg1, arg2) {
            const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getProgramParameter_92e4540ca9da06b2: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_getShaderInfoLog_9e0b96da4b13ae49: function(arg0, arg1, arg2) {
            const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getShaderParameter_afa4a3dd9dd397c1: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_getUniformLocation_d06b3a5b3c60e95c: function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_height_38750dc6de41ee75: function(arg0) {
            const ret = getObject(arg0).height;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_3f2f6e1edb1c9792: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_WebGl2RenderingContext_4a08a94517ed5240: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof WebGL2RenderingContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_ed49b2db8df90359: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_jserror_new: function(arg0) {
            const ret = JsError.__wrap(arg0);
            return addHeapObject(ret);
        },
        __wbg_linkProgram_6600dd2c0863bbfd: function(arg0, arg1) {
            getObject(arg0).linkProgram(getObject(arg1));
        },
        __wbg_log_6b5ca2e6124b2808: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_log_98ea330cbdc64a56: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_log_f996de40931ab7d1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_mark_49688daf5a319979: function(arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        },
        __wbg_measure_52555d98d3c0f41a: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
            }
        }, arguments); },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_no_args_1c7c842f08d00ebb: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_preventDefault_cdcfcd7e301b9702: function(arg0) {
            getObject(arg0).preventDefault();
        },
        __wbg_push_8ffdcb2063340ba5: function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        },
        __wbg_random_912284dbf636f269: function() {
            const ret = Math.random();
            return ret;
        },
        __wbg_setTimeout_eff32631ea138533: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
            return ret;
        }, arguments); },
        __wbg_set_6cb8631f80447a67: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments); },
        __wbg_shaderSource_32425cfe6e5a1e52: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_12837167ad935116: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_uniform2f_1887b1268f65bfee: function(arg0, arg1, arg2, arg3) {
            getObject(arg0).uniform2f(getObject(arg1), arg2, arg3);
        },
        __wbg_uniformMatrix4fv_0e724dbebd372526: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_useProgram_fe720ade4d3b6edb: function(arg0, arg1) {
            getObject(arg0).useProgram(getObject(arg1));
        },
        __wbg_vertexAttribDivisor_744c0ca468594894: function(arg0, arg1, arg2) {
            getObject(arg0).vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_vertexAttribPointer_75f6ff47f6c9f8cb: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        },
        __wbg_viewport_df236eac68bc7467: function(arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).viewport(arg1, arg2, arg3, arg4);
        },
        __wbg_warn_f7ae1b2e66ccb930: function(arg0) {
            console.warn(getObject(arg0));
        },
        __wbg_width_5f66bde2e810fbde: function(arg0) {
            const ret = getObject(arg0).width;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1, function: Function { arguments: [NamedExternref("Event")], shim_idx: 3, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_83, __wasm_bindgen_func_elem_425);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1, function: Function { arguments: [], shim_idx: 2, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.__wasm_bindgen_func_elem_83, __wasm_bindgen_func_elem_424);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000003: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
            const ret = getArrayF32FromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000007: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./archflow_web_bg.js": import0,
    };
}

function __wasm_bindgen_func_elem_424(arg0, arg1) {
    wasm.__wasm_bindgen_func_elem_424(arg0, arg1);
}

function __wasm_bindgen_func_elem_425(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_425(arg0, arg1, addHeapObject(arg2));
}

const CameraConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_cameraconfig_free(ptr >>> 0, 1));
const ControllerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_controller_free(ptr >>> 0, 1));
const HighlightConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_highlightconfig_free(ptr >>> 0, 1));
const JsErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jserror_free(ptr >>> 0, 1));
const LogicMappingTableWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_logicmappingtablewasm_free(ptr >>> 0, 1));
const LogicSystemWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_logicsystemwasm_free(ptr >>> 0, 1));
const MoveConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_moveconfig_free(ptr >>> 0, 1));
const PropertyConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_propertyconfig_free(ptr >>> 0, 1));
const PropertyValueFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_propertyvalue_free(ptr >>> 0, 1));
const PulseWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pulsewasm_free(ptr >>> 0, 1));
const SignalByteWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_signalbytewasm_free(ptr >>> 0, 1));
const WasmBridgeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmbridge_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(128).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('archflow_web_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
