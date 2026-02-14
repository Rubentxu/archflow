/* @ts-self-types="./archflow_wasm_bridge.d.ts" */

//#region exports

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
 * Brick Chain Builder - API Fluida implementada en Rust/WASM
 *
 * # JavaScript Example
 * ```javascript
 * const handle = bridge
 *   .sensor(Sensor.Mouse.Click('Left'))
 *   .controller(Controller.And())
 *   .actuator(Actuator.Select.Single())
 *   .connect();
 * ```
 */
export class BrickChainBuilder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BrickChainBuilder.prototype);
        obj.__wbg_ptr = ptr;
        BrickChainBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BrickChainBuilderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_brickchainbuilder_free(ptr, 0);
    }
    /**
     * Get the number of actuators
     * @returns {number}
     */
    actuator_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.brickchainbuilder_actuator_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Add a Highlight actuator
     *
     * # Arguments
     * * `color_argb` - Color in ARGB format
     * * `opacity` - Opacity value
     * @param {number} color_argb
     * @param {number} opacity
     * @returns {BrickChainBuilder}
     */
    actuator_highlight(color_argb, opacity) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        _assertNum(color_argb);
        const ret = wasm.brickchainbuilder_actuator_highlight(ptr, color_argb, opacity);
        return BrickChainBuilder.__wrap(ret);
    }
    /**
     * Add a Move actuator
     *
     * # Arguments
     * * `mode` - 0=To, 1=By, 2=Drag
     * * `x` - X value or offset
     * * `y` - Y value or offset
     * @param {number} mode
     * @param {number} x
     * @param {number} y
     * @returns {BrickChainBuilder}
     */
    actuator_move(mode, x, y) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        _assertNum(mode);
        const ret = wasm.brickchainbuilder_actuator_move(ptr, mode, x, y);
        return BrickChainBuilder.__wrap(ret);
    }
    /**
     * Add a Select actuator
     *
     * # Arguments
     * * `mode` - 0=Single, 1=Multi, 2=Replace
     * @param {number} mode
     * @returns {BrickChainBuilder}
     */
    actuator_select(mode) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        _assertNum(mode);
        const ret = wasm.brickchainbuilder_actuator_select(ptr, mode);
        return BrickChainBuilder.__wrap(ret);
    }
    /**
     * Connect and register the brick chain
     * @returns {BrickHandle}
     */
    connect() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        const ret = wasm.brickchainbuilder_connect(ptr);
        return BrickHandle.__wrap(ret);
    }
    /**
     * Add a controller to the brick chain
     * @param {Controller} controller
     * @returns {BrickChainBuilder}
     */
    controller(controller) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        _assertClass(controller, Controller);
        if (controller.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        var ptr0 = controller.__destroy_into_raw();
        const ret = wasm.brickchainbuilder_controller(ptr, ptr0);
        return BrickChainBuilder.__wrap(ret);
    }
    /**
     * Get the number of controllers
     * @returns {number}
     */
    controller_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.brickchainbuilder_controller_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the entity ID
     * @returns {number}
     */
    entity_id() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.brickchainbuilder_entity_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Creates a new BrickChainBuilder for an entity
     * @param {number} entity_id
     */
    constructor(entity_id) {
        _assertNum(entity_id);
        const ret = wasm.brickchainbuilder_new(entity_id);
        this.__wbg_ptr = ret >>> 0;
        BrickChainBuilderFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Add a sensor to the brick chain
     * @param {SensorType} sensor
     * @returns {BrickChainBuilder}
     */
    sensor(sensor) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        _assertNum(sensor);
        const ret = wasm.brickchainbuilder_sensor(ptr, sensor);
        return BrickChainBuilder.__wrap(ret);
    }
    /**
     * Get the number of sensors
     * @returns {number}
     */
    sensor_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.brickchainbuilder_sensor_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Add a keyboard key sensor (convenience)
     * @param {number} key_code
     * @returns {BrickChainBuilder}
     */
    sensor_key(key_code) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        _assertNum(key_code);
        const ret = wasm.brickchainbuilder_sensor_key(ptr, key_code);
        return BrickChainBuilder.__wrap(ret);
    }
    /**
     * Creates a new BrickChainBuilder with a mapping table
     * @param {number} entity_id
     * @param {LogicMappingTableWasm} mapping_table
     * @returns {BrickChainBuilder}
     */
    static with_mapping_table(entity_id, mapping_table) {
        _assertNum(entity_id);
        _assertClass(mapping_table, LogicMappingTableWasm);
        if (mapping_table.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        var ptr0 = mapping_table.__destroy_into_raw();
        const ret = wasm.brickchainbuilder_with_mapping_table(entity_id, ptr0);
        return BrickChainBuilder.__wrap(ret);
    }
}
if (Symbol.dispose) BrickChainBuilder.prototype[Symbol.dispose] = BrickChainBuilder.prototype.free;

/**
 * Handle to a registered brick chain
 *
 * Returned by BrickChainBuilder.connect() for runtime control.
 *
 * # JavaScript Example
 * ```javascript
 * const handle = builder.connect();
 * handle.disable();
 * handle.remove();
 * ```
 */
export class BrickHandle {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BrickHandle.prototype);
        obj.__wbg_ptr = ptr;
        BrickHandleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BrickHandleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_brickhandle_free(ptr, 0);
    }
    /**
     * Disable the brick chain
     */
    disable() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.brickhandle_disable(this.__wbg_ptr);
    }
    /**
     * Enable the brick chain
     */
    enable() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.brickhandle_enable(this.__wbg_ptr);
    }
    /**
     * Get the brick chain ID
     * @returns {string}
     */
    id() {
        let deferred1_0;
        let deferred1_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.brickhandle_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Check if enabled
     * @returns {boolean}
     */
    is_enabled() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.brickhandle_is_enabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates a new BrickHandle with the given ID
     * @param {string} id
     */
    constructor(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.brickhandle_new(ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        BrickHandleFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Remove the brick chain
     */
    remove() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        const ptr = this.__destroy_into_raw();
        _assertNum(ptr);
        wasm.brickhandle_remove(ptr);
    }
    /**
     * Toggle enabled state
     * @returns {boolean}
     */
    toggle() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.brickhandle_toggle(this.__wbg_ptr);
        return ret !== 0;
    }
}
if (Symbol.dispose) BrickHandle.prototype[Symbol.dispose] = BrickHandle.prototype.free;

/**
 * Unique callback identifier
 */
export class CallbackId {
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CallbackId.prototype);
        obj.__wbg_ptr = ptr;
        CallbackIdFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CallbackIdFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_callbackid_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get value() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.callbackid_value(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) CallbackId.prototype[Symbol.dispose] = CallbackId.prototype.free;

export class CallbackRegistry {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CallbackRegistry.prototype);
        obj.__wbg_ptr = ptr;
        CallbackRegistryFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CallbackRegistryFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_callbackregistry_free(ptr, 0);
    }
    clear() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.callbackregistry_clear(this.__wbg_ptr);
    }
    /**
     * @param {string} event_type
     * @returns {number}
     */
    event_callback_count(event_type) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(event_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.callbackregistry_event_callback_count(this.__wbg_ptr, ptr0, len0);
        return ret >>> 0;
    }
    /**
     * @param {string} event_type
     * @param {any} data
     * @returns {number}
     */
    invoke(event_type, data) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(event_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.callbackregistry_invoke(this.__wbg_ptr, ptr0, len0, data);
        return ret >>> 0;
    }
    constructor() {
        const ret = wasm.callbackregistry_new();
        this.__wbg_ptr = ret >>> 0;
        CallbackRegistryFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Function} callback
     * @param {string} event_type
     * @param {boolean} is_oneshot
     * @returns {CallbackId}
     */
    register(callback, event_type, is_oneshot) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(event_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertBoolean(is_oneshot);
        const ret = wasm.callbackregistry_register(this.__wbg_ptr, callback, ptr0, len0, is_oneshot);
        return CallbackId.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    get total_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.callbackregistry_total_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {CallbackId} id
     * @returns {boolean}
     */
    unregister(id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertClass(id, CallbackId);
        if (id.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        var ptr0 = id.__destroy_into_raw();
        const ret = wasm.callbackregistry_unregister(this.__wbg_ptr, ptr0);
        return ret !== 0;
    }
    /**
     * @param {string} event_type
     * @returns {number}
     */
    unregister_all(event_type) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(event_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.callbackregistry_unregister_all(this.__wbg_ptr, ptr0, len0);
        return ret >>> 0;
    }
}
if (Symbol.dispose) CallbackRegistry.prototype[Symbol.dispose] = CallbackRegistry.prototype.free;

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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        _assertNum(duration_ms);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.cameraconfig_smooth(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get target X position
     * @returns {number}
     */
    target_x() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.cameraconfig_target_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get target Y position
     * @returns {number}
     */
    target_y() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.cameraconfig_target_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get zoom level
     * @returns {number}
     */
    zoom() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
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
        _assertNum(sensor);
        const ret = wasm.controller_and(sensor);
        return Controller.__wrap(ret);
    }
    /**
     * Creates an AND controller (combines all sensors)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.And();
     * // Combines all sensors in the brick chain
     * ```
     * @returns {Controller}
     */
    static and_any() {
        const ret = wasm.controller_and_any();
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
        _assertNum(interval);
        const ret = wasm.controller_blinky(interval);
        return Controller.__wrap(ret);
    }
    /**
     * Returns the controller type
     * @returns {ControllerType}
     */
    controller_type() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(code, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.controller_custom(ptr0, len0, ptr1, len1);
        return Controller.__wrap(ret);
    }
    /**
     * Returns the custom code (for Custom controllers)
     * @returns {string | undefined}
     */
    custom_code() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.controller_custom_code(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Returns the custom name (for Custom controllers)
     * @returns {string | undefined}
     */
    custom_name() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.controller_custom_name(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
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
        _assertNum(ticks);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.controller_float_param1(this.__wbg_ptr);
        return ret;
    }
    /**
     * Returns the second float parameter (for Hysteresis low)
     * @returns {number}
     */
    float_param2() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.controller_float_param2(this.__wbg_ptr);
        return ret;
    }
    /**
     * Checks if this controller has a secondary sensor
     * @returns {boolean}
     */
    has_secondary_sensor() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        _assertNum(sensor);
        const ret = wasm.controller_or(sensor);
        return Controller.__wrap(ret);
    }
    /**
     * Creates an OR controller (any sensor activates)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Or();
     * // Activates if any sensor is active
     * ```
     * @returns {Controller}
     */
    static or_any() {
        const ret = wasm.controller_or_any();
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
        _assertNum(mask);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
 * WASM wrapper for accessing the event ring buffer
 *
 * This provides JavaScript access to events generated by the logic system.
 */
export class EventRingBufferWasm {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EventRingBufferWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_eventringbufferwasm_free(ptr, 0);
    }
    /**
     * Drain all pending events from the logic system
     *
     * This method returns all events currently in the ring buffer and
     * clears the buffer for the next frame.
     *
     * # Returns
     *
     * Array of JsLogicEvent objects representing all pending events
     *
     * # JavaScript Example
     *
     * ```javascript
     * const eventBuffer = new EventRingBufferWasm();
     * const events = eventBuffer.drain(logicSystem);
     *
     * for (const event of events) {
     *   console.log(`Event ${event.event_type} from entity ${event.entity_id}`);
     * }
     * ```
     * @param {LogicSystemWasm} system
     * @returns {JsLogicEvent[]}
     */
    drain(system) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertClass(system, LogicSystemWasm);
        if (system.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        const ret = wasm.eventringbufferwasm_drain(this.__wbg_ptr, system.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get the number of events currently in the buffer
     *
     * # Returns
     *
     * Number of pending events
     * @param {LogicSystemWasm} system
     * @returns {number}
     */
    event_count(system) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertClass(system, LogicSystemWasm);
        if (system.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        const ret = wasm.eventringbufferwasm_event_count(this.__wbg_ptr, system.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Check if there are any pending events
     *
     * # Returns
     *
     * true if there are pending events, false otherwise
     * @param {LogicSystemWasm} system
     * @returns {boolean}
     */
    has_events(system) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertClass(system, LogicSystemWasm);
        if (system.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
        const ret = wasm.eventringbufferwasm_has_events(this.__wbg_ptr, system.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Create a new EventRingBuffer accessor
     */
    constructor() {
        const ret = wasm.eventringbufferwasm_new();
        this.__wbg_ptr = ret >>> 0;
        EventRingBufferWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) EventRingBufferWasm.prototype[Symbol.dispose] = EventRingBufferWasm.prototype.free;

/**
 * Event type constants for JavaScript
 *
 * These correspond to the LogicEventType enum in Rust
 */
export class EventType {
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        EventTypeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_eventtype_free(ptr, 0);
    }
    /**
     * Box selection completed
     * @returns {number}
     */
    static get box_selection_completed() {
        const ret = wasm.eventtype_box_selection_completed();
        return ret;
    }
    /**
     * Drag operation ended
     * @returns {number}
     */
    static get drag_ended() {
        const ret = wasm.eventtype_drag_ended();
        return ret;
    }
    /**
     * Drag operation started
     * @returns {number}
     */
    static get drag_started() {
        const ret = wasm.eventtype_drag_started();
        return ret;
    }
    /**
     * Entity was selected/deselected
     * @returns {number}
     */
    static get entity_selected() {
        const ret = wasm.eventtype_entity_selected();
        return ret;
    }
    /**
     * Hover state changed
     * @returns {number}
     */
    static get hover_changed() {
        const ret = wasm.eventtype_hover_changed();
        return ret;
    }
    /**
     * Proximity threshold crossed
     * @returns {number}
     */
    static get proximity_alert() {
        const ret = wasm.eventtype_proximity_alert();
        return ret;
    }
}
if (Symbol.dispose) EventType.prototype[Symbol.dispose] = EventType.prototype.free;

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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        _assertNum(color);
        _assertNum(restore_color);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.highlightconfig_opacity(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get the restore color (ARGB)
     * @returns {number}
     */
    restore_color() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.jserror_message(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} message
     */
    constructor(message) {
        const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.jserror_new(ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        JsErrorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) JsError.prototype[Symbol.dispose] = JsError.prototype.free;

/**
 * Structure representing a single event for JavaScript consumption
 *
 * This is a simplified version of LogicEvent that's easy to serialize
 * across the WASM boundary.
 */
export class JsLogicEvent {
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JsLogicEvent.prototype);
        obj.__wbg_ptr = ptr;
        JsLogicEventFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsLogicEventFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jslogicevent_free(ptr, 0);
    }
    /**
     * Get data_1 (context-dependent)
     * @returns {number}
     */
    get data_1() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.jslogicevent_data_1(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get data_2 (context-dependent)
     * @returns {number}
     */
    get data_2() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.jslogicevent_data_2(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get data_3 (context-dependent)
     * @returns {number}
     */
    get data_3() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.jslogicevent_data_3(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the entity ID
     * @returns {number}
     */
    get entity_id() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.jslogicevent_entity_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the event type
     * @returns {number}
     */
    get event_type() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.jslogicevent_event_type(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get the timestamp
     * @returns {bigint}
     */
    get timestamp_us() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.jslogicevent_timestamp_us(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
}
if (Symbol.dispose) JsLogicEvent.prototype[Symbol.dispose] = JsLogicEvent.prototype.free;

/**
 * Simplified event data for WASM export
 */
export class JsLogicEventData {
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JsLogicEventData.prototype);
        obj.__wbg_ptr = ptr;
        JsLogicEventDataFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsLogicEventDataFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jslogiceventdata_free(ptr, 0);
    }
    /**
     * Additional data depending on event type:
     * - ProximityAlert: f32 distance
     * - DragStarted/DragEnded: f32 x, f32 y position
     * - BoxSelectionCompleted: u32 count
     * - HoverChanged: u32 entity_id (or 0 for none)
     * @returns {number}
     */
    get data_1() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.__wbg_get_jslogiceventdata_data_1(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get data_2() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.__wbg_get_jslogiceventdata_data_2(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get data_3() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.__wbg_get_jslogiceventdata_data_3(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Entity ID that triggered the event
     * @returns {number}
     */
    get entity_id() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.__wbg_get_jslogiceventdata_entity_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Event type identifier
     * @returns {number}
     */
    get event_type() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.__wbg_get_jslogiceventdata_event_type(this.__wbg_ptr);
        return ret;
    }
    /**
     * Timestamp in microseconds
     * @returns {bigint}
     */
    get timestamp_us() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.__wbg_get_jslogiceventdata_timestamp_us(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Additional data depending on event type:
     * - ProximityAlert: f32 distance
     * - DragStarted/DragEnded: f32 x, f32 y position
     * - BoxSelectionCompleted: u32 count
     * - HoverChanged: u32 entity_id (or 0 for none)
     * @param {number} arg0
     */
    set data_1(arg0) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.__wbg_set_jslogiceventdata_data_1(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set data_2(arg0) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.__wbg_set_jslogiceventdata_data_2(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set data_3(arg0) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(arg0);
        wasm.__wbg_set_jslogiceventdata_data_3(this.__wbg_ptr, arg0);
    }
    /**
     * Entity ID that triggered the event
     * @param {number} arg0
     */
    set entity_id(arg0) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(arg0);
        wasm.__wbg_set_jslogiceventdata_entity_id(this.__wbg_ptr, arg0);
    }
    /**
     * Event type identifier
     * @param {number} arg0
     */
    set event_type(arg0) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(arg0);
        wasm.__wbg_set_jslogiceventdata_event_type(this.__wbg_ptr, arg0);
    }
    /**
     * Timestamp in microseconds
     * @param {bigint} arg0
     */
    set timestamp_us(arg0) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBigInt(arg0);
        wasm.__wbg_set_jslogiceventdata_timestamp_us(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) JsLogicEventData.prototype[Symbol.dispose] = JsLogicEventData.prototype.free;

/**
 * Complete Logic Bricks system for the web editor
 *
 * Provides fluent API for declaring sensor-actuator connections and processing input.
 */
export class LogicBricksSystem {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LogicBricksSystemFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_logicbrickssystem_free(ptr, 0);
    }
    /**
     * Clear creation state
     */
    clear_creation() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.logicbrickssystem_clear_creation(this.__wbg_ptr);
    }
    /**
     * Clear drag state
     */
    clear_drag_state() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.logicbrickssystem_clear_drag_state(this.__wbg_ptr);
    }
    /**
     * Get drag count
     * @returns {number}
     */
    drag_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_drag_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get event buffer length (WASM compatible - returns cached value)
     * @returns {number}
     */
    event_buffer_len() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_event_buffer_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the active tool
     * @returns {string}
     */
    get_active_tool() {
        let deferred1_0;
        let deferred1_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.logicbrickssystem_get_active_tool(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get creation start position
     * @returns {number}
     */
    get_creation_start_pos() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_get_creation_start_pos(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get selected entities as array (WASM compatible)
     * @returns {Array<any>}
     */
    get_selected_entities() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_get_selected_entities(this.__wbg_ptr);
        return ret;
    }
    /**
     * Check if there are pending events
     * @returns {boolean}
     */
    has_events() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_has_events(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Check if creating
     * @returns {boolean}
     */
    is_creating() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_is_creating(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Check if dragging
     * @returns {boolean}
     */
    is_dragging() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_is_dragging(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Create a new Logic Bricks system
     */
    constructor() {
        const ret = wasm.logicbrickssystem_new();
        this.__wbg_ptr = ret >>> 0;
        LogicBricksSystemFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get pending command count
     * @returns {number}
     */
    pending_command_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_pending_command_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Poll all events and return count
     * @returns {number}
     */
    poll_events() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_poll_events(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Sample input state from JavaScript
     *
     * Should be called each frame before tick().
     * @param {number} screen_x
     * @param {number} screen_y
     * @param {number} world_x
     * @param {number} world_y
     * @param {number} buttons
     * @param {number} wheel
     * @param {number} modifiers
     */
    sample_input(screen_x, screen_y, world_x, world_y, buttons, wheel, modifiers) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(buttons);
        _assertNum(wheel);
        _assertNum(modifiers);
        wasm.logicbrickssystem_sample_input(this.__wbg_ptr, screen_x, screen_y, world_x, world_y, buttons, wheel, modifiers);
    }
    /**
     * Get number of selected entities
     * @returns {number}
     */
    selection_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicbrickssystem_selection_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Set the active tool
     * @param {string} tool
     */
    set_active_tool(tool) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(tool, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.logicbrickssystem_set_active_tool(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Set creation start position
     * @param {number} x
     * @param {number} y
     */
    set_creation_start(x, y) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.logicbrickssystem_set_creation_start(this.__wbg_ptr, x, y);
    }
}
if (Symbol.dispose) LogicBricksSystem.prototype[Symbol.dispose] = LogicBricksSystem.prototype.free;

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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor);
        _assertClass(controller, Controller);
        if (controller.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor);
        _assertClass(controller, Controller);
        if (controller.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor);
        _assertClass(controller, Controller);
        if (controller.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicmappingtablewasm_get_connected_entities(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor);
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
     * Attach a behavior to an entity
     * @param {number} _behavior_id
     * @param {number} _entity_id
     */
    attach_behavior(_behavior_id, _entity_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(_behavior_id);
        _assertNum(_entity_id);
        wasm.logicsystemwasm_attach_behavior(this.__wbg_ptr, _behavior_id, _entity_id);
    }
    /**
     * Get count of behaviors
     * @returns {number}
     */
    behavior_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicsystemwasm_behavior_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Check if behavior has events
     * @param {number} _behavior_id
     * @returns {boolean}
     */
    behavior_has_events(_behavior_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(_behavior_id);
        const ret = wasm.logicsystemwasm_behavior_has_events(this.__wbg_ptr, _behavior_id);
        return ret !== 0;
    }
    /**
     * Create a simple behavior
     *
     * # Arguments
     * * `entity_id` - Entity ID for the behavior
     * * `sensor_type` - Sensor type (0=Click, 1=Hover, 2=Drag, 3=Key)
     * * `actuator_type` - Actuator type (0=Highlight, 1=Select, 2=Move, 3=Delete, 4=Emit)
     *
     * # Returns
     * Behavior ID
     * @param {number} _entity_id
     * @param {number} _sensor_type
     * @param {number} _actuator_type
     * @returns {number}
     */
    create_behavior(_entity_id, _sensor_type, _actuator_type) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(_entity_id);
        _assertNum(_sensor_type);
        _assertNum(_actuator_type);
        const ret = wasm.logicsystemwasm_create_behavior(this.__wbg_ptr, _entity_id, _sensor_type, _actuator_type);
        return ret >>> 0;
    }
    /**
     * Detach a behavior
     * @param {number} _behavior_id
     */
    detach_behavior(_behavior_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(_behavior_id);
        wasm.logicsystemwasm_detach_behavior(this.__wbg_ptr, _behavior_id);
    }
    /**
     * Drain all pending events from the event buffer
     *
     * # Returns
     * Array of event data objects (simplified for WASM)
     * @returns {JsLogicEventData[]}
     */
    drain_events() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicsystemwasm_drain_events(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get the number of pending events
     *
     * # Returns
     * Number of events in the buffer
     * @returns {number}
     */
    event_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicsystemwasm_event_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get behavior state as JSON
     * @param {number} _behavior_id
     * @returns {string}
     */
    get_behavior_state(_behavior_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(_behavior_id);
        const ret = wasm.logicsystemwasm_get_behavior_state(this.__wbg_ptr, _behavior_id);
        return ret;
    }
    /**
     * Check if there are pending events in the buffer
     *
     * # Returns
     * true if there are events, false otherwise
     * @returns {boolean}
     */
    has_events() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.logicsystemwasm_has_events(this.__wbg_ptr);
        return ret !== 0;
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
     * Set behavior enabled/disabled
     * @param {number} _behavior_id
     * @param {boolean} _enabled
     */
    set_behavior_enabled(_behavior_id, _enabled) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(_behavior_id);
        _assertBoolean(_enabled);
        wasm.logicsystemwasm_set_behavior_enabled(this.__wbg_ptr, _behavior_id, _enabled);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBigInt(timestamp_ms);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.moveconfig_constrain_x(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether Y axis is constrained
     * @returns {boolean}
     */
    constrain_y() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        _assertBoolean(constrain_x);
        _assertBoolean(constrain_y);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        const ptr0 = passStringToWasm0(property_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(value, PropertyValue);
        if (value.__wbg_ptr === 0) {
            throw new Error('Attempt to use a moved value');
        }
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.propertyconfig_property_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get property value
     * @returns {PropertyValue}
     */
    value() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.propertyconfig_value(this.__wbg_ptr);
        return PropertyValue.__wrap(ret);
    }
}
if (Symbol.dispose) PropertyConfig.prototype[Symbol.dispose] = PropertyConfig.prototype.free;

/**
 * Property value wrapper for WASM
 */
export class PropertyValue {
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
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
        _assertBoolean(value);
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
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.propertyvalue_value(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
 *   state: 1, // Positive
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
    get entity_id() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.pulsewasm_entity_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new Pulse
     * @param {number} entity_id
     * @param {number} sensor_id
     * @param {number} state
     * @param {number} timestamp
     */
    constructor(entity_id, sensor_id, state, timestamp) {
        _assertNum(entity_id);
        _assertNum(sensor_id);
        _assertNum(state);
        _assertNum(timestamp);
        const ret = wasm.pulsewasm_new(entity_id, sensor_id, state, timestamp);
        this.__wbg_ptr = ret >>> 0;
        PulseWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get the sensor ID
     * @returns {number}
     */
    get sensor_id() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.pulsewasm_sensor_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the state (0=None, 1=Positive, 2=Negative)
     * @returns {number}
     */
    get state() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.pulsewasm_state(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get the timestamp
     * @returns {number}
     */
    get timestamp() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.pulsewasm_timestamp(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) PulseWasm.prototype[Symbol.dispose] = PulseWasm.prototype.free;

/**
 * Select mode for selection actuator (matches core SelectMode)
 * @enum {0 | 1 | 2 | 3 | 4 | 5}
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
    /**
     * Toggle selection (inverts selection state)
     */
    Toggle: 3, "3": "Toggle",
    /**
     * Add to selection (ensure selected)
     */
    Add: 4, "4": "Add",
    /**
     * Subtract from selection (ensure deselected)
     */
    Subtract: 5, "5": "Subtract",
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.signalbytewasm_any_edge(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns the raw u8 value (for serialization)
     * @returns {number}
     */
    as_u8() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.signalbytewasm_count_zeros(this.__wbg_ptr);
        return ret;
    }
    /**
     * Creates a SignalByte from a u8 value
     * @param {number} value
     * @returns {SignalByteWasm}
     */
    static from(value) {
        _assertNum(value);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.signalbytewasm_is_rising_edge(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Alias for isSteadyHigh (for backward compatibility)
     * @param {number} ticks
     * @returns {boolean}
     */
    is_steady(ticks) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(ticks);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(ticks);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(ticks);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBoolean(active);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
     * Add a sensor connection to an entity
     *
     * Creates a sensor-to-actuator connection using the LogicMappingTable.
     *
     * # Arguments
     *
     * * `entity_id` - The entity to add the sensor to
     * * `sensor_type` - Type of sensor (0=MouseOver, 1=MouseClick, 2=Proximity, 3=KeyShortcut, 4=Touch, 5=Radar, 6=DoubleTap, 7=LongPress, 8=RightClick)
     * * `controller_type` - Type of controller (0=Direct, 1=AND, 2=OR, 3=NOT)
     * * `actuator_type` - Type of actuator (0=Highlight, 1=Select, 2=Move, 3=Sound, 4=Animation, 5=Custom, 6=Property, 7=Visibility)
     *
     * # Returns
     *
     * Ok(true) if connection was added successfully
     * @param {number} entity_id
     * @param {number} sensor_type
     * @param {number} controller_type
     * @param {number} actuator_type
     * @returns {boolean}
     */
    add_sensor(entity_id, sensor_type, controller_type, actuator_type) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor_type);
        _assertNum(controller_type);
        _assertNum(actuator_type);
        const ret = wasm.wasmbridge_add_sensor(this.__wbg_ptr, entity_id, sensor_type, controller_type, actuator_type);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Batch despawn multiple entities
     *
     * ids: array of entity indices to remove
     * @param {Uint32Array} ids
     * @returns {number}
     */
    batch_despawn(ids) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_despawn(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Batch set colors for multiple entities
     *
     * ids: array of entity indices
     * colors: array of RGBA colors (u32)
     * @param {Uint32Array} ids
     * @param {Uint32Array} colors
     * @returns {number}
     */
    batch_set_colors(ids, colors) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray32ToWasm0(colors, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_set_colors(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Batch set physics material for multiple entities
     * This is more efficient than calling set_physics_material for each entity
     * @param {Uint32Array} ids
     * @param {number} restitution
     * @param {number} friction
     * @param {number} mass
     */
    batch_set_physics_materials(ids, restitution, friction, mass) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_set_physics_materials(this.__wbg_ptr, ptr0, len0, restitution, friction, mass);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Batch set positions for multiple entities
     *
     * ids: array of entity indices
     * xs: array of x positions (same length as ids)
     * ys: array of y positions (same length as ids)
     * @param {Uint32Array} ids
     * @param {Float32Array} xs
     * @param {Float32Array} ys
     * @returns {number}
     */
    batch_set_positions(ids, xs, ys) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF32ToWasm0(xs, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayF32ToWasm0(ys, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_set_positions(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Batch set shapes for multiple entities (optimized)
     * @param {Uint32Array} ids
     * @param {Uint8Array} shapes
     * @returns {number}
     */
    batch_set_shapes(ids, shapes) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(shapes, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_set_shapes(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Batch set sizes for multiple entities
     *
     * ids: array of entity indices
     * widths: array of widths
     * heights: array of heights
     * @param {Uint32Array} ids
     * @param {Float32Array} widths
     * @param {Float32Array} heights
     * @returns {number}
     */
    batch_set_sizes(ids, widths, heights) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF32ToWasm0(widths, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayF32ToWasm0(heights, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_set_sizes(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Batch set velocities for multiple entities
     * ids: array of entity IDs
     * vx, vy: flat arrays of velocities
     * @param {Uint32Array} ids
     * @param {Float32Array} vx
     * @param {Float32Array} vy
     */
    batch_set_velocities(ids, vx, vy) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF32ToWasm0(vx, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayF32ToWasm0(vy, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_batch_set_velocities(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Batch set visibility for multiple entities
     *
     * ids: array of entity indices
     * visible: visibility state to apply to all
     * @param {Uint32Array} ids
     * @param {boolean} visible
     * @returns {number}
     */
    batch_set_visibility(ids, visible) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArray32ToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertBoolean(visible);
        const ret = wasm.wasmbridge_batch_set_visibility(this.__wbg_ptr, ptr0, len0, visible);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Bulk spawn multiple entities in a single call - ZERO-COPY
     *
     * This is the MOST EFFICIENT way to spawn entities:
     * - positions: flat array of [x0, y0, x1, y1, ...] (2 * count floats)
     * - sizes: flat array of [w0, h0, w1, h1, ...] (2 * count floats)
     * - colors: flat array of [r0, g0, b0, a0, r1, g1, b1, a1, ...] (4 * count u8s)
     *   Pass empty Uint8Array() for random colors
     *
     * Returns: array of spawned entity indices
     *
     * # Example (JavaScript)
     * ```js
     * const positions = new Float32Array([100, 100, 200, 200, 300, 300]);
     * const sizes = new Float32Array([50, 50, 60, 60, 70, 70]);
     * const colors = new Uint8Array([255, 0, 0, 255, 0, 255, 0, 255]); // or empty for random
     * const ids = bridge.bulk_spawn(positions, sizes, colors);
     * ```
     * @param {Float32Array} positions
     * @param {Float32Array} sizes
     * @param {Uint8Array} colors
     * @returns {Uint32Array}
     */
    bulk_spawn(positions, sizes, colors) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passArrayF32ToWasm0(positions, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF32ToWasm0(sizes, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArray8ToWasm0(colors, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_bulk_spawn(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v4 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v4;
    }
    /**
     * Check if redo is available
     * @returns {boolean}
     */
    can_redo() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_can_redo(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Check if undo is available
     * @returns {boolean}
     */
    can_undo() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_can_undo(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Clear all entities
     */
    clear() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_clear(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear all logic connections for all entities
     */
    clear_all_logic() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_clear_all_logic(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear highlight tint (reset to default)
     * @param {number} entity_index
     */
    clear_color_tint(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_clear_color_tint(this.__wbg_ptr, entity_index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear all logic connections for an entity
     * @param {number} entity_id
     */
    clear_entity_logic(entity_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        const ret = wasm.wasmbridge_clear_entity_logic(this.__wbg_ptr, entity_id);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Clear all selections (deselect all entities)
     */
    clear_selection() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_clear_selection(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Configure mouse sensor for an entity
     *
     * # Arguments
     *
     * * `mode` - Mouse mode: 0=movement, 1=left_button, 2=right_button, 3=middle_button, 4=wheel_up
     * * `tap` - Enable tap detection (true) or continuous (false)
     * @param {number} mode
     * @param {boolean} tap
     */
    configure_mouse_sensor(mode, tap) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(mode);
        _assertBoolean(tap);
        const ret = wasm.wasmbridge_configure_mouse_sensor(this.__wbg_ptr, mode, tap);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get number of connections for an entity
     * @param {number} entity_id
     * @returns {number}
     */
    connection_count(entity_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        const ret = wasm.wasmbridge_connection_count(this.__wbg_ptr, entity_id);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Delete all selected entities
     */
    delete_selected() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_delete_selected(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Detect available graphics backends
     * @returns {object}
     */
    detect_available_backends() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_detect_available_backends(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Duplicate an entity (create a copy at a slight offset)
     * @param {number} entity_index
     * @returns {number}
     */
    duplicate_entity(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_duplicate_entity(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Get the number of alive entities
     * @returns {number}
     */
    entity_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_entity_count(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Get the active fill color (returns RGBA as hex string)
     * @returns {string}
     */
    get_active_color() {
        let deferred2_0;
        let deferred2_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.wasmbridge_get_active_color(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.wasmbridge_get_active_stroke_color(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the active stroke width
     * @returns {number}
     */
    get_active_stroke_width() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_active_stroke_width(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Get list of alive entity indices
     * @returns {Uint32Array}
     */
    get_alive_entities() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_alive_entities(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get the camera center position
     * @returns {Array<any>}
     */
    get_camera_center() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_camera_center(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            _assertNum(entity_index);
            const ret = wasm.wasmbridge_get_color(this.__wbg_ptr, entity_index);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            _assertNum(entity_index);
            const ret = wasm.wasmbridge_get_entity_color_hex(this.__wbg_ptr, entity_index);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the current entity count
     * @returns {number}
     */
    get_entity_count() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_entity_count(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            _assertNum(entity_index);
            const ret = wasm.wasmbridge_get_entity_label(this.__wbg_ptr, entity_index);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get entity position in screen coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_position_screen(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_entity_position_screen(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get entity position in world coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_position_world(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_entity_position_world(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get entity shape type
     * @param {number} entity_index
     * @returns {number}
     */
    get_entity_shape(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_entity_shape(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Get entity size in screen coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_size_screen(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_entity_size_screen(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get entity size in world coordinates
     * @param {number} entity_index
     * @returns {Array<any>}
     */
    get_entity_size_world(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_entity_size_world(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get current velocity of an entity
     * @param {number} entity_index
     * @returns {Float32Array}
     */
    get_entity_velocity(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_entity_velocity(this.__wbg_ptr, entity_index);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get history state for UI feedback
     * @returns {string}
     */
    get_history_state() {
        let deferred2_0;
        let deferred2_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.wasmbridge_get_history_state(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
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
     * Get the maximum entity capacity
     * @returns {number}
     */
    get_max_entities() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_max_entities(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get current keyboard modifiers
     *
     * Returns bitmask of pressed modifiers (1=shift, 2=ctrl, 4=alt)
     * @returns {number}
     */
    get_modifiers() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_modifiers(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Get current mouse button state
     *
     * Returns bitmask of pressed buttons (1=left, 2=right, 4=middle)
     * @returns {number}
     */
    get_mouse_buttons() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_mouse_buttons(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Get current mouse position in screen coordinates
     *
     * Returns tuple of (x, y) or null if engine not initialized.
     * @returns {string}
     */
    get_mouse_position() {
        let deferred2_0;
        let deferred2_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.wasmbridge_get_mouse_position(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the list of selected entity IDs
     * @returns {Array<any>}
     */
    get_selection() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_selection(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
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
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            _assertNum(entity_index);
            const ret = wasm.wasmbridge_get_stroke_color(this.__wbg_ptr, entity_index);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get the stroke width of an entity
     * @param {number} entity_index
     * @returns {number}
     */
    get_stroke_width(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_get_stroke_width(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Get the current tool type
     * @returns {string}
     */
    get_tool() {
        let deferred2_0;
        let deferred2_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.wasmbridge_get_tool(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get velocity of an entity
     * Returns [vx, vy]
     * @param {number} entity_id
     * @returns {Float32Array}
     */
    get_velocity(entity_id) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        const ret = wasm.wasmbridge_get_velocity(this.__wbg_ptr, entity_id);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get the current camera zoom level
     * @returns {number}
     */
    get_zoom() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_get_zoom(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Initialize audio context (must be called after user interaction)
     * @returns {boolean}
     */
    init_audio() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_init_audio(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Initialize the engine
     *
     * This should be called once when the application starts.
     * @param {number} canvas_width
     * @param {number} canvas_height
     */
    initialize(canvas_width, canvas_height) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_initialize(this.__wbg_ptr, canvas_width, canvas_height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Initialize graphics (uses WebGL2/Canvas 2D by default)
     *
     * This should be called after `initialize()` and after the canvas is mounted.
     * @param {HTMLCanvasElement} canvas
     */
    initialize_graphics(canvas) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_initialize_graphics(this.__wbg_ptr, canvas);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(backend, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_initialize_graphics_with_backend(this.__wbg_ptr, canvas, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Integrate physics for all entities
     * This should be called every frame for physics to work
     * Returns number of entities processed
     * @param {number} dt
     * @param {number} min_x
     * @param {number} min_y
     * @param {number} max_x
     * @param {number} max_y
     * @returns {number}
     */
    integrate_physics(dt, min_x, min_y, max_x, max_y) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_integrate_physics(this.__wbg_ptr, dt, min_x, min_y, max_x, max_y);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Check if entity is selected
     * @param {number} entity_index
     * @returns {boolean}
     */
    is_entity_selected(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_is_entity_selected(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Check if entity is visible
     * @param {number} entity_index
     * @returns {boolean}
     */
    is_entity_visible(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_is_entity_visible(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Check if context recovery is in progress
     * @returns {boolean}
     */
    is_recovering() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_is_recovering(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Get selection state of an entity
     * @param {number} entity_index
     * @returns {boolean}
     */
    is_selected(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_is_selected(this.__wbg_ptr, entity_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Load a scene from JSON string
     *
     * Expects JSON with format (component-based):
     * ```json
     * {
     *   "entities": [
     *     { "id": "entity1", "components": { "Position": {"x": 100, "y": 200}, "Size": {"width": 50, "height": 50}, "Shape": {"shape": 0}, "Color": {"color": 4294967295} } }
     *   ]
     * }
     * ```
     * @param {string} json
     * @returns {number}
     */
    load_scene(json) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_load_scene(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Move an entity by the given delta
     * @param {number} entity_index
     * @param {number} dx
     * @param {number} dy
     */
    move_entity(entity_index, dx, dy) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_move_entity(this.__wbg_ptr, entity_index, dx, dy);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Move entity by delta (direct position update, not command queue)
     * @param {number} entity_index
     * @param {number} dx
     * @param {number} dy
     */
    move_entity_by(entity_index, dx, dy) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_move_entity_by(this.__wbg_ptr, entity_index, dx, dy);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
     * Report keyboard event to Logic Bricks sensors
     *
     * This should be called from JavaScript's keydown/keyup event handlers.
     * Triggers keyboard shortcut sensors.
     *
     * # Arguments
     * * `key_code` - DOM keyCode value
     * * `is_down` - true for keydown, false for keyup
     * * `modifiers` - Bitmask of modifiers (1=shift, 2=ctrl, 4=alt)
     * @param {number} key_code
     * @param {boolean} is_down
     * @param {number} modifiers
     */
    on_key(key_code, is_down, modifiers) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(key_code);
        _assertBoolean(is_down);
        _assertNum(modifiers);
        wasm.wasmbridge_on_key(this.__wbg_ptr, key_code, is_down, modifiers);
    }
    /**
     * Report mouse down event to Logic Bricks sensors
     *
     * This should be called from JavaScript's mousedown event handler.
     * Triggers mouse click sensors for left/right/middle buttons.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `button` - Mouse button (0=left, 1=right, 2=middle)
     * * `modifiers` - Bitmask of modifiers (1=shift, 2=ctrl, 4=alt)
     * @param {number} screen_x
     * @param {number} screen_y
     * @param {number} button
     * @param {number} modifiers
     */
    on_mouse_down(screen_x, screen_y, button, modifiers) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(button);
        _assertNum(modifiers);
        wasm.wasmbridge_on_mouse_down(this.__wbg_ptr, screen_x, screen_y, button, modifiers);
    }
    /**
     * Report mouse move event to Logic Bricks sensors
     *
     * This should be called from JavaScript's mousemove event handler.
     * The engine will convert screen coordinates to world coordinates
     * and feed them to the appropriate sensors.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `buttons` - Bitmask of pressed buttons (1=left, 2=right, 4=middle)
     * @param {number} screen_x
     * @param {number} screen_y
     * @param {number} buttons
     * @param {number} modifiers
     */
    on_mouse_move(screen_x, screen_y, buttons, modifiers) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(buttons);
        _assertNum(modifiers);
        wasm.wasmbridge_on_mouse_move(this.__wbg_ptr, screen_x, screen_y, buttons, modifiers);
    }
    /**
     * Report mouse up event to Logic Bricks sensors
     *
     * This should be called from JavaScript's mouseup event handler.
     * Clears the button state in sensors.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `button` - Mouse button that was released
     * @param {number} screen_x
     * @param {number} screen_y
     * @param {number} button
     * @param {number} modifiers
     */
    on_mouse_up(screen_x, screen_y, button, modifiers) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(button);
        _assertNum(modifiers);
        wasm.wasmbridge_on_mouse_up(this.__wbg_ptr, screen_x, screen_y, button, modifiers);
    }
    /**
     * Report mouse wheel event to Logic Bricks sensors
     *
     * This should be called from JavaScript's wheel event handler.
     * Triggers zoom camera actuators when applicable.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `delta_y` - Scroll delta (positive=up, negative=down)
     * * `modifiers` - Bitmask of modifiers
     * @param {number} screen_x
     * @param {number} screen_y
     * @param {number} delta_y
     * @param {number} modifiers
     */
    on_wheel(screen_x, screen_y, delta_y, modifiers) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(modifiers);
        wasm.wasmbridge_on_wheel(this.__wbg_ptr, screen_x, screen_y, delta_y, modifiers);
    }
    /**
     * Play a beep sound using Web Audio API oscillator
     *
     * # Arguments
     * * `frequency` - Frequency in Hz (220.0 to 2000.0)
     * * `duration` - Duration in seconds (0.1 to 2.0)
     * * `volume` - Volume/gain from 0.0 to 1.0
     * @param {number} frequency
     * @param {number} _duration
     * @param {number} volume
     */
    play_beep(frequency, _duration, volume) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_play_beep(this.__wbg_ptr, frequency, _duration, volume);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Poll all events from the logic system
     *
     * Returns a JavaScript array of events emitted by the logic system
     * during the current frame. Call this once per frame after `tick()`.
     *
     * # Returns
     *
     * Number of events generated during the current frame.
     * Call this once per frame after `tick()`.
     *
     * # Example
     *
     * ```javascript
     * // In your JavaScript/TypeScript code
     * const eventCount = bridge.poll_events();
     * if (eventCount > 0) {
     *     console.log('Events generated:', eventCount);
     * }
     * ```
     * @returns {number}
     */
    poll_events() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_poll_events(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Process all pending input events
     *
     * This drains the input ring buffer and feeds events to Logic Bricks sensors.
     * Called automatically by tick(), but can be called manually if needed.
     */
    process_input_events() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.wasmbridge_process_input_events(this.__wbg_ptr);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(event_type);
        _assertNum(buttons);
        _assertNum(modifiers);
        const ret = wasm.wasmbridge_push_input_event(this.__wbg_ptr, event_type, x, y, buttons, modifiers);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Query all alive entities (returns all entity IDs)
     * @returns {Uint32Array}
     */
    query_all() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_query_all(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities by layer
     * @param {number} layer
     * @returns {Uint32Array}
     */
    query_by_layer(layer) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(layer);
        const ret = wasm.wasmbridge_query_by_layer(this.__wbg_ptr, layer);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities with minimum size
     * @param {number} min_width
     * @param {number} min_height
     * @returns {Uint32Array}
     */
    query_by_min_size(min_width, min_height) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_query_by_min_size(this.__wbg_ptr, min_width, min_height);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities by selection state
     * @param {boolean} selected
     * @returns {Uint32Array}
     */
    query_by_selection(selected) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBoolean(selected);
        const ret = wasm.wasmbridge_query_by_selection(this.__wbg_ptr, selected);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities by shape type
     *
     * shape: 0=rectangle, 1=circle, 2=triangle, etc.
     * @param {number} shape
     * @returns {Uint32Array}
     */
    query_by_shape(shape) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(shape);
        const ret = wasm.wasmbridge_query_by_shape(this.__wbg_ptr, shape);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities by visibility
     * @param {boolean} visible
     * @returns {Uint32Array}
     */
    query_by_visibility(visible) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertBoolean(visible);
        const ret = wasm.wasmbridge_query_by_visibility(this.__wbg_ptr, visible);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities within bounds (AABB query)
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     * @returns {Uint32Array}
     */
    query_in_bounds(x, y, width, height) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_query_in_bounds(this.__wbg_ptr, x, y, width, height);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Query entities that have velocity (moving entities)
     * @returns {Uint32Array}
     */
    query_with_velocity() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_query_with_velocity(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Redo the last undone action
     */
    redo() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_redo(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Remove a sensor connection from an entity
     *
     * # Arguments
     *
     * * `entity_id` - The entity to remove the sensor from
     * * `sensor_type` - Type of sensor to disconnect
     * @param {number} entity_id
     * @param {number} sensor_type
     */
    remove_sensor(entity_id, sensor_type) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        _assertNum(sensor_type);
        const ret = wasm.wasmbridge_remove_sensor(this.__wbg_ptr, entity_id, sensor_type);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Resize the engine and renderer
     * @param {number} width
     * @param {number} height
     */
    resize(width, height) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_resize(this.__wbg_ptr, width, height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Add an entity to the selection (toggle mode)
     * @param {number} entity_index
     */
    select_entity(entity_index) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_select_entity(this.__wbg_ptr, entity_index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Serialize the current project
     * @returns {Uint8Array}
     */
    serialize_project() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_serialize_project(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Serialize current scene to JSON string
     * @returns {string}
     */
    serialize_scene() {
        let deferred2_0;
        let deferred2_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ret = wasm.wasmbridge_serialize_scene(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Set acceleration for physics simulation
     * ax, ay = acceleration in units/second^2
     * @param {number} entity_id
     * @param {number} ax
     * @param {number} ay
     */
    set_acceleration(entity_id, ax, ay) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        const ret = wasm.wasmbridge_set_acceleration(this.__wbg_ptr, entity_id, ax, ay);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(r);
        _assertNum(g);
        _assertNum(b);
        _assertNum(a);
        const ret = wasm.wasmbridge_set_active_color(this.__wbg_ptr, r, g, b, a);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(r);
        _assertNum(g);
        _assertNum(b);
        _assertNum(a);
        const ret = wasm.wasmbridge_set_active_stroke_color(this.__wbg_ptr, r, g, b, a);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the active stroke width for new shapes
     * @param {number} width
     */
    set_active_stroke_width(width) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_set_active_stroke_width(this.__wbg_ptr, width);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the camera center position
     * @param {number} x
     * @param {number} y
     */
    set_camera_center(x, y) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_set_camera_center(this.__wbg_ptr, x, y);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        _assertNum(r);
        _assertNum(g);
        _assertNum(b);
        _assertNum(a);
        const ret = wasm.wasmbridge_set_color(this.__wbg_ptr, entity_index, r, g, b, a);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set highlight tint color (for visual feedback on hover/selection)
     * @param {number} entity_index
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} a
     */
    set_color_tint(entity_index, r, g, b, a) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_set_color_tint(this.__wbg_ptr, entity_index, r, g, b, a);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        _assertBoolean(selected);
        const ret = wasm.wasmbridge_set_entity_selected(this.__wbg_ptr, entity_index, selected);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set velocity directly (for physics integration)
     * @param {number} entity_index
     * @param {number} vx
     * @param {number} vy
     */
    set_entity_velocity(entity_index, vx, vy) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_set_entity_velocity(this.__wbg_ptr, entity_index, vx, vy);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set entity visibility
     * @param {number} entity_index
     * @param {boolean} visible
     */
    set_entity_visible(entity_index, visible) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        _assertBoolean(visible);
        const ret = wasm.wasmbridge_set_entity_visible(this.__wbg_ptr, entity_index, visible);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the label of an entity
     * @param {number} entity_index
     * @param {string} label
     */
    set_label(entity_index, label) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_set_label(this.__wbg_ptr, entity_index, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set master volume
     *
     * volume: 0.0-1.0
     * @param {number} volume
     */
    set_master_volume(volume) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_set_master_volume(this.__wbg_ptr, volume);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set physics material properties
     * restitution: 0.0 = no bounce, 1.0 = full bounce
     * friction: 0.0 = no friction, 1.0 = high friction
     * mass: 0.0 = infinite/static, >0 = dynamic
     * @param {number} entity_id
     * @param {number} restitution
     * @param {number} friction
     * @param {number} mass
     */
    set_physics_material(entity_id, restitution, friction, mass) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        const ret = wasm.wasmbridge_set_physics_material(this.__wbg_ptr, entity_id, restitution, friction, mass);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the position of an entity
     * @param {number} entity_index
     * @param {number} x
     * @param {number} y
     */
    set_position(entity_index, x, y) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_set_position(this.__wbg_ptr, entity_index, x, y);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set selection state of an entity
     * @param {number} entity_index
     * @param {boolean} selected
     */
    set_selected(entity_index, selected) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        _assertBoolean(selected);
        const ret = wasm.wasmbridge_set_selected(this.__wbg_ptr, entity_index, selected);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the shape type of an entity
     * @param {number} entity_index
     * @param {number} shape
     */
    set_shape(entity_index, shape) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        _assertNum(shape);
        const ret = wasm.wasmbridge_set_shape(this.__wbg_ptr, entity_index, shape);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the size of an entity
     * @param {number} entity_index
     * @param {number} width
     * @param {number} height
     */
    set_size(entity_index, width, height) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_set_size(this.__wbg_ptr, entity_index, width, height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        _assertNum(r);
        _assertNum(g);
        _assertNum(b);
        _assertNum(a);
        const ret = wasm.wasmbridge_set_stroke_color(this.__wbg_ptr, entity_index, r, g, b, a);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the stroke width of an entity
     * @param {number} entity_index
     * @param {number} width
     */
    set_stroke_width(entity_index, width) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_index);
        const ret = wasm.wasmbridge_set_stroke_width(this.__wbg_ptr, entity_index, width);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the current tool type
     * @param {string} tool
     */
    set_tool(tool) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(tool, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmbridge_set_tool(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set velocity for physics simulation
     * vx, vy = velocity in units/second
     * @param {number} entity_id
     * @param {number} vx
     * @param {number} vy
     */
    set_velocity(entity_id, vx, vy) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(entity_id);
        const ret = wasm.wasmbridge_set_velocity(this.__wbg_ptr, entity_id, vx, vy);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set the camera zoom level
     * @param {number} zoom
     */
    set_zoom(zoom) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_set_zoom(this.__wbg_ptr, zoom);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
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
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_spawn_entity(this.__wbg_ptr, x, y, width, height);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Spawn a pool of pre-allocated entities for optimal performance
     *
     * Use this to pre-allocate entities at startup, then use set_visible()
     * to show/hide them instead of spawning/despawning.
     *
     * Returns: number of entities spawned
     * @param {number} count
     * @returns {number}
     */
    spawn_pool(count) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        _assertNum(count);
        const ret = wasm.wasmbridge_spawn_pool(this.__wbg_ptr, count);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Run one frame of the engine
     *
     * This should be called from requestAnimationFrame.
     * Uses the fluent API: sample_input() → tick() → poll_events()
     * @param {number} timestamp
     */
    tick(timestamp) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_tick(this.__wbg_ptr, timestamp);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Undo the last action
     */
    undo() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ret = wasm.wasmbridge_undo(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) WasmBridge.prototype[Symbol.dispose] = WasmBridge.prototype.free;

/**
 * Create a delete actuator
 *
 * # Returns
 * ActuatorType.Move (delete operates via move/transform)
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorDelete();
 * ```
 * @returns {ActuatorType}
 */
export function actuator_delete() {
    const ret = wasm.actuator_delete();
    return ret;
}

/**
 * Create an emit event actuator
 *
 * # Returns
 * ActuatorType.Move (event emission handled via state change)
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorEmitEvent('EntitySelected', '{"id":42}');
 * ```
 * @param {string} event_name
 * @param {string | null} [event_data]
 * @returns {ActuatorType}
 */
export function actuator_emit_event(event_name, event_data) {
    const ptr0 = passStringToWasm0(event_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(event_data) ? 0 : passStringToWasm0(event_data, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    const ret = wasm.actuator_emit_event(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * Create a highlight actuator
 *
 * # Arguments
 * * `color_argb` - Color in ARGB format
 * * `opacity` - Opacity (0.0 to 1.0)
 *
 * # Returns
 * ActuatorType.Highlight
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorHighlight(0xff00ff00, 0.5);
 * ```
 * @param {number} color_argb
 * @param {number} opacity
 * @returns {ActuatorType}
 */
export function actuator_highlight(color_argb, opacity) {
    _assertNum(color_argb);
    const ret = wasm.actuator_highlight(color_argb, opacity);
    return ret;
}

/**
 * Create a move actuator
 *
 * # Arguments
 * * `mode` - 0=To, 1=By, 2=Drag
 * * `x` - X value or offset
 * * `y` - Y value or offset
 *
 * # Returns
 * ActuatorType.Move
 *
 * # JavaScript Example
 * ```javascript
 * const to = actuatorMove(0, 100, 200);
 * ```
 * @param {number} mode
 * @param {number} x
 * @param {number} y
 * @returns {ActuatorType}
 */
export function actuator_move(mode, x, y) {
    _assertNum(mode);
    const ret = wasm.actuator_move(mode, x, y);
    return ret;
}

/**
 * Create a clear select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectClear();
 * ```
 * @returns {ActuatorType}
 */
export function actuator_select_clear() {
    const ret = wasm.actuator_select_clear();
    return ret;
}

/**
 * Create a multi-select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectMulti();
 * ```
 * @returns {ActuatorType}
 */
export function actuator_select_multi() {
    const ret = wasm.actuator_select_multi();
    return ret;
}

/**
 * Create a single select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectSingle();
 * ```
 * @returns {ActuatorType}
 */
export function actuator_select_single() {
    const ret = wasm.actuator_select_single();
    return ret;
}

/**
 * Create a toggle select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectToggle();
 * ```
 * @returns {ActuatorType}
 */
export function actuator_select_toggle() {
    const ret = wasm.actuator_select_toggle();
    return ret;
}

/**
 * Create an AND controller
 *
 * # Arguments
 * * `sensor` - Secondary sensor that must also be active
 *
 * # Returns
 * Controller with AND logic (all sensors must be active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerAnd(SensorType.MouseOver);
 * ```
 * @param {SensorType} sensor
 * @returns {Controller}
 */
export function factory_and(sensor) {
    _assertNum(sensor);
    const ret = wasm.factory_and(sensor);
    return Controller.__wrap(ret);
}

/**
 * Create a Blinky controller
 *
 * # Arguments
 * * `interval` - Toggle interval in ticks
 *
 * # Returns
 * Blinky controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerBlinky(4);
 * ```
 * @param {number} interval
 * @returns {Controller}
 */
export function factory_blinky(interval) {
    _assertNum(interval);
    const ret = wasm.factory_blinky(interval);
    return Controller.__wrap(ret);
}

/**
 * Create a Custom controller
 *
 * # Arguments
 * * `name` - Controller name
 * * `code` - Custom evaluation code
 *
 * # Returns
 * Custom controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerCustom('myLogic', 'return signal.isSteady(6);');
 * ```
 * @param {string} name
 * @param {string} code
 * @returns {Controller}
 */
export function factory_custom(name, code) {
    const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(code, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.factory_custom(ptr0, len0, ptr1, len1);
    return Controller.__wrap(ret);
}

/**
 * Create a Debounce controller
 *
 * # Arguments
 * * `ticks` - Number of ticks for stability
 *
 * # Returns
 * Debounce controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerDebounce(6);
 * ```
 * @param {number} ticks
 * @returns {Controller}
 */
export function factory_debounce(ticks) {
    _assertNum(ticks);
    const ret = wasm.factory_debounce(ticks);
    return Controller.__wrap(ret);
}

/**
 * Create a Direct controller
 *
 * # Returns
 * Direct controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerDirect();
 * ```
 * @returns {Controller}
 */
export function factory_direct() {
    const ret = wasm.factory_direct();
    return Controller.__wrap(ret);
}

/**
 * Create a Hysteresis controller
 *
 * # Arguments
 * * `high` - Activation threshold (0.0 to 1.0)
 * * `low` - Deactivation threshold (0.0 to 1.0)
 *
 * # Returns
 * Hysteresis controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerHysteresis(0.8, 0.3);
 * ```
 * @param {number} high
 * @param {number} low
 * @returns {Controller}
 */
export function factory_hysteresis(high, low) {
    const ret = wasm.factory_hysteresis(high, low);
    return Controller.__wrap(ret);
}

/**
 * Create a NAND controller
 *
 * # Returns
 * Controller with NAND logic (not all sensors active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerNand();
 * ```
 * @returns {Controller}
 */
export function factory_nand() {
    const ret = wasm.factory_nand();
    return Controller.__wrap(ret);
}

/**
 * Create a NOR controller
 *
 * # Returns
 * Controller with NOR logic (no sensors active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerNor();
 * ```
 * @returns {Controller}
 */
export function factory_nor() {
    const ret = wasm.factory_nor();
    return Controller.__wrap(ret);
}

/**
 * Create a NOT controller
 *
 * # Returns
 * Controller with NOT logic
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerNot();
 * ```
 * @returns {Controller}
 */
export function factory_not() {
    const ret = wasm.factory_not();
    return Controller.__wrap(ret);
}

/**
 * Create an OR controller
 *
 * # Arguments
 * * `sensor` - Alternative sensor that can activate
 *
 * # Returns
 * Controller with OR logic (any sensor activates)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerOr(SensorType.MouseClick);
 * ```
 * @param {SensorType} sensor
 * @returns {Controller}
 */
export function factory_or(sensor) {
    _assertNum(sensor);
    const ret = wasm.factory_or(sensor);
    return Controller.__wrap(ret);
}

/**
 * Create a Pattern controller
 *
 * # Arguments
 * * `mask` - 6-bit pattern to match
 *
 * # Returns
 * Pattern controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerPattern(0b00100100);
 * ```
 * @param {number} mask
 * @returns {Controller}
 */
export function factory_pattern(mask) {
    _assertNum(mask);
    const ret = wasm.factory_pattern(mask);
    return Controller.__wrap(ret);
}

/**
 * Create a Threshold controller
 *
 * # Arguments
 * * `value` - Minimum stability (0.0 to 1.0)
 *
 * # Returns
 * Threshold controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerThreshold(0.5);
 * ```
 * @param {number} value
 * @returns {Controller}
 */
export function factory_threshold(value) {
    const ret = wasm.factory_threshold(value);
    return Controller.__wrap(ret);
}

/**
 * Create an XOR controller
 *
 * # Returns
 * Controller with XOR logic (exactly one sensor active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerXor();
 * ```
 * @returns {Controller}
 */
export function factory_xor() {
    const ret = wasm.factory_xor();
    return Controller.__wrap(ret);
}

/**
 * @returns {CallbackRegistry}
 */
export function get_global_callback_registry() {
    const ret = wasm.get_global_callback_registry();
    return CallbackRegistry.__wrap(ret);
}

/**
 * Create a collision detection sensor
 *
 * # Arguments
 * * `layer_id` - Optional layer ID (0 for default)
 *
 * # Returns
 * SensorType.Touch for collision detection
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorCollisionDetect(0);
 * ```
 * @param {number} layer_id
 * @returns {SensorType}
 */
export function sensor_collision_detect(layer_id) {
    _assertNum(layer_id);
    const ret = wasm.sensor_collision_detect(layer_id);
    return ret;
}

/**
 * Create a double-tap sensor
 *
 * # Returns
 * SensorType.DoubleTap
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorDoubleTap();
 * ```
 * @returns {SensorType}
 */
export function sensor_double_tap() {
    const ret = wasm.sensor_double_tap();
    return ret;
}

/**
 * Create a keyboard key press sensor
 *
 * # Arguments
 * * `key_code` - Key code number
 * * `modifiers` - Optional bitmask of modifiers (1=Shift, 2=Ctrl, 4=Alt)
 *
 * # Returns
 * SensorType.KeyShortcut
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorKeyboardKey(46, 0); // Delete key
 * ```
 * @param {number} _key_code
 * @param {number} _modifiers
 * @returns {SensorType}
 */
export function sensor_keyboard_key(_key_code, _modifiers) {
    _assertNum(_key_code);
    _assertNum(_modifiers);
    const ret = wasm.sensor_keyboard_key(_key_code, _modifiers);
    return ret;
}

/**
 * Create a long-press sensor
 *
 * # Arguments
 * * `threshold_ms` - Time in ms to consider a "long" press
 *
 * # Returns
 * SensorType.LongPress
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorLongPress(500);
 * ```
 * @param {number} threshold_ms
 * @returns {SensorType}
 */
export function sensor_long_press(threshold_ms) {
    _assertNum(threshold_ms);
    const ret = wasm.sensor_long_press(threshold_ms);
    return ret;
}

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * Create a mouse click sensor
 *
 * # Arguments
 * * `button` - Button name: 0=Left, 1=Right, 2=Middle
 *
 * # Returns
 * SensorType for use in brick chains
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorMouseClick(0); // Left
 * ```
 * @param {number} button
 * @returns {SensorType}
 */
export function sensor_mouse_click(button) {
    _assertNum(button);
    const ret = wasm.sensor_mouse_click(button);
    return ret;
}

/**
 * Create a mouse drag sensor
 *
 * # Arguments
 * * `button` - Button name: 0=Left, 1=Right, 2=Middle
 *
 * # Returns
 * SensorType for drag detection
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorMouseDrag(0);
 * ```
 * @param {number} _button
 * @returns {SensorType}
 */
export function sensor_mouse_drag(_button) {
    _assertNum(_button);
    const ret = wasm.sensor_mouse_drag(_button);
    return ret;
}

/**
 * Create a mouse hover sensor
 *
 * # Returns
 * SensorType.MouseOver
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorMouseHover();
 * ```
 * @returns {SensorType}
 */
export function sensor_mouse_hover() {
    const ret = wasm.sensor_mouse_hover();
    return ret;
}

/**
 * Create a mouse wheel sensor
 *
 * # Arguments
 * * `direction` - 1=Up, -1=Down
 *
 * # Returns
 * SensorType.Radar (mapped)
 *
 * # JavaScript Example
 * ```javascript
 * const sensorUp = sensorMouseWheel(1);
 * ```
 * @param {number} _direction
 * @returns {SensorType}
 */
export function sensor_mouse_wheel(_direction) {
    _assertNum(_direction);
    const ret = wasm.sensor_mouse_wheel(_direction);
    return ret;
}

/**
 * Create a property change sensor
 *
 * # Arguments
 * * `property_id` - Property ID to monitor
 *
 * # Returns
 * SensorType for property changes
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorPropertyChanged(0);
 * ```
 * @param {number} property_id
 * @returns {SensorType}
 */
export function sensor_property_changed(property_id) {
    _assertNum(property_id);
    const ret = wasm.sensor_property_changed(property_id);
    return ret;
}

/**
 * Create a timer delay sensor
 *
 * # Arguments
 * * `ms` - Delay in milliseconds
 * * `once` - If true, only fires once
 *
 * # Returns
 * SensorType (delay)
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorTimerDelay(500, true);
 * ```
 * @param {number} ms
 * @param {boolean} once
 * @returns {SensorType}
 */
export function sensor_timer_delay(ms, once) {
    _assertNum(ms);
    _assertBoolean(once);
    const ret = wasm.sensor_timer_delay(ms, once);
    return ret;
}

/**
 * Create a timer interval sensor
 *
 * # Arguments
 * * `ms` - Interval in milliseconds
 *
 * # Returns
 * SensorType (timer)
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorTimerInterval(1000); // Every second
 * ```
 * @param {number} ms
 * @returns {SensorType}
 */
export function sensor_timer_interval(ms) {
    _assertNum(ms);
    const ret = wasm.sensor_timer_interval(ms);
    return ret;
}

//#endregion

//#region wasm imports

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            if (!isLikeNone(ret)) {
                _assertBoolean(ret);
            }
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = arg0 === undefined;
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            if (!isLikeNone(ret)) {
                _assertNum(ret);
            }
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_d9b87ff7982e3b21: function() { return logError(function (arg0) {
            arg0._wbg_cb_unref();
        }, arguments); },
        __wbg_addEventListener_3acb0aad4483804c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_attachShader_b36058e5c9eeaf54: function() { return logError(function (arg0, arg1, arg2) {
            arg0.attachShader(arg1, arg2);
        }, arguments); },
        __wbg_bindBuffer_c9068e8712a034f5: function() { return logError(function (arg0, arg1, arg2) {
            arg0.bindBuffer(arg1 >>> 0, arg2);
        }, arguments); },
        __wbg_bindVertexArray_78220d1edb1d2382: function() { return logError(function (arg0, arg1) {
            arg0.bindVertexArray(arg1);
        }, arguments); },
        __wbg_blendFunc_2ef59299d10c662d: function() { return logError(function (arg0, arg1, arg2) {
            arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
        }, arguments); },
        __wbg_bufferData_98f6c413a8f0f139: function() { return logError(function (arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        }, arguments); },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_call_4708e0c13bdc8e95: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_clearColor_404a3b16d43db93b: function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.clearColor(arg1, arg2, arg3, arg4);
        }, arguments); },
        __wbg_clear_7187030f892c5ca0: function() { return logError(function (arg0, arg1) {
            arg0.clear(arg1 >>> 0);
        }, arguments); },
        __wbg_compileShader_94718a93495d565d: function() { return logError(function (arg0, arg1) {
            arg0.compileShader(arg1);
        }, arguments); },
        __wbg_createBuffer_26534c05e01b8559: function() { return logError(function (arg0) {
            const ret = arg0.createBuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_createElement_49f60fdcaae809c8: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createProgram_9b7710a1f2701c2c: function() { return logError(function (arg0) {
            const ret = arg0.createProgram();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_createShader_e3ac08ed8c5b14b2: function() { return logError(function (arg0, arg1) {
            const ret = arg0.createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_createVertexArray_ad5294951ae57497: function() { return logError(function (arg0) {
            const ret = arg0.createVertexArray();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_document_ee35a3d3ae34ef6c: function() { return logError(function (arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_drawArraysInstanced_ec30adc616ec58d5: function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
        }, arguments); },
        __wbg_enableVertexAttribArray_475e06c31777296d: function() { return logError(function (arg0, arg1) {
            arg0.enableVertexAttribArray(arg1 >>> 0);
        }, arguments); },
        __wbg_enable_d1ac04dfdd2fb3ae: function() { return logError(function (arg0, arg1) {
            arg0.enable(arg1 >>> 0);
        }, arguments); },
        __wbg_error_7534b8e9a36f1ab4: function() { return logError(function (arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_error_9a7fe3f932034cde: function() { return logError(function (arg0) {
            console.error(arg0);
        }, arguments); },
        __wbg_eval_3f0b9f0cbaf45a34: function() { return handleError(function (arg0, arg1) {
            const ret = eval(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_getContext_2a5764d48600bc43: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getError_bba8594facbfd5e1: function() { return logError(function (arg0) {
            const ret = arg0.getError();
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_getProgramInfoLog_2ffa30e3abb8b5c2: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg1.getProgramInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getProgramParameter_92e4540ca9da06b2: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getShaderInfoLog_9e0b96da4b13ae49: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg1.getShaderInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getShaderParameter_afa4a3dd9dd397c1: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getUniformLocation_d06b3a5b3c60e95c: function() { return logError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_get_b3ed3ad4be2bc8ac: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_height_38750dc6de41ee75: function() { return logError(function (arg0) {
            const ret = arg0.height;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_HtmlCanvasElement_3f2f6e1edb1c9792: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_WebGl2RenderingContext_4a08a94517ed5240: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof WebGL2RenderingContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Window_ed49b2db8df90359: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_jserror_new: function() { return logError(function (arg0) {
            const ret = JsError.__wrap(arg0);
            return ret;
        }, arguments); },
        __wbg_jslogicevent_new: function() { return logError(function (arg0) {
            const ret = JsLogicEvent.__wrap(arg0);
            return ret;
        }, arguments); },
        __wbg_jslogiceventdata_new: function() { return logError(function (arg0) {
            const ret = JsLogicEventData.__wrap(arg0);
            return ret;
        }, arguments); },
        __wbg_length_35a7bace40f36eac: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_linkProgram_6600dd2c0863bbfd: function() { return logError(function (arg0, arg1) {
            arg0.linkProgram(arg1);
        }, arguments); },
        __wbg_log_6b5ca2e6124b2808: function() { return logError(function (arg0) {
            console.log(arg0);
        }, arguments); },
        __wbg_log_98ea330cbdc64a56: function() { return logError(function (arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_log_f996de40931ab7d1: function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_mark_49688daf5a319979: function() { return logError(function (arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        }, arguments); },
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
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
            }
        }, arguments); },
        __wbg_new_361308b2356cecd0: function() { return logError(function () {
            const ret = new Object();
            return ret;
        }, arguments); },
        __wbg_new_3eb36ae241fe6f44: function() { return logError(function () {
            const ret = new Array();
            return ret;
        }, arguments); },
        __wbg_new_8a6f238a6ece86ea: function() { return logError(function () {
            const ret = new Error();
            return ret;
        }, arguments); },
        __wbg_new_no_args_1c7c842f08d00ebb: function() { return logError(function (arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_preventDefault_cdcfcd7e301b9702: function() { return logError(function (arg0) {
            arg0.preventDefault();
        }, arguments); },
        __wbg_push_8ffdcb2063340ba5: function() { return logError(function (arg0, arg1) {
            const ret = arg0.push(arg1);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_random_912284dbf636f269: function() { return logError(function () {
            const ret = Math.random();
            return ret;
        }, arguments); },
        __wbg_setTimeout_eff32631ea138533: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_set_6cb8631f80447a67: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_shaderSource_32425cfe6e5a1e52: function() { return logError(function (arg0, arg1, arg2, arg3) {
            arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_stack_0ed75d68575b0f3c: function() { return logError(function (arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_static_accessor_GLOBAL_12837167ad935116: function() { return logError(function () {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() { return logError(function () {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() { return logError(function () {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() { return logError(function () {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_uniform2f_1887b1268f65bfee: function() { return logError(function (arg0, arg1, arg2, arg3) {
            arg0.uniform2f(arg1, arg2, arg3);
        }, arguments); },
        __wbg_uniformMatrix4fv_0e724dbebd372526: function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_useProgram_fe720ade4d3b6edb: function() { return logError(function (arg0, arg1) {
            arg0.useProgram(arg1);
        }, arguments); },
        __wbg_vertexAttribDivisor_744c0ca468594894: function() { return logError(function (arg0, arg1, arg2) {
            arg0.vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
        }, arguments); },
        __wbg_vertexAttribPointer_75f6ff47f6c9f8cb: function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        }, arguments); },
        __wbg_viewport_df236eac68bc7467: function() { return logError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.viewport(arg1, arg2, arg3, arg4);
        }, arguments); },
        __wbg_warn_f7ae1b2e66ccb930: function() { return logError(function (arg0) {
            console.warn(arg0);
        }, arguments); },
        __wbg_width_5f66bde2e810fbde: function() { return logError(function (arg0) {
            const ret = arg0.width;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbindgen_cast_0000000000000001: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 83, function: Function { arguments: [NamedExternref("Event")], shim_idx: 52, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__he0ad02f7d6eb91b4, wasm_bindgen__convert__closures_____invoke__he38ed39e9585f820);
            return ret;
        }, arguments); },
        __wbindgen_cast_0000000000000002: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 84, function: Function { arguments: [], shim_idx: 51, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hf1ebcb809a7ca87b, wasm_bindgen__convert__closures_____invoke__h08c40c31efbb9c66);
            return ret;
        }, arguments); },
        __wbindgen_cast_0000000000000003: function() { return logError(function (arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        }, arguments); },
        __wbindgen_cast_0000000000000004: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
            const ret = getArrayF32FromWasm0(arg0, arg1);
            return ret;
        }, arguments); },
        __wbindgen_cast_0000000000000005: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        }, arguments); },
        __wbindgen_cast_0000000000000006: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        }, arguments); },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./archflow_wasm_bridge_bg.js": import0,
    };
}


//#endregion
function wasm_bindgen__convert__closures_____invoke__h08c40c31efbb9c66(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures_____invoke__h08c40c31efbb9c66(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__he38ed39e9585f820(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm.wasm_bindgen__convert__closures_____invoke__he38ed39e9585f820(arg0, arg1, arg2);
}

const BrickChainBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_brickchainbuilder_free(ptr >>> 0, 1));
const BrickHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_brickhandle_free(ptr >>> 0, 1));
const CallbackIdFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_callbackid_free(ptr >>> 0, 1));
const CallbackRegistryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_callbackregistry_free(ptr >>> 0, 1));
const CameraConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_cameraconfig_free(ptr >>> 0, 1));
const ControllerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_controller_free(ptr >>> 0, 1));
const EventRingBufferWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_eventringbufferwasm_free(ptr >>> 0, 1));
const EventTypeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_eventtype_free(ptr >>> 0, 1));
const HighlightConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_highlightconfig_free(ptr >>> 0, 1));
const JsErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jserror_free(ptr >>> 0, 1));
const JsLogicEventFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jslogicevent_free(ptr >>> 0, 1));
const JsLogicEventDataFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jslogiceventdata_free(ptr >>> 0, 1));
const LogicBricksSystemFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_logicbrickssystem_free(ptr >>> 0, 1));
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


//#region intrinsics
function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertBigInt(n) {
    if (typeof(n) !== 'bigint') throw new Error(`expected a bigint argument, found ${typeof(n)}`);
}

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error(`expected a boolean argument, found ${typeof(n)}`);
    }
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error(`expected a number argument, found ${typeof(n)}`);
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

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
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

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
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

function passArray32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getUint32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getFloat32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (typeof(arg) !== 'string') throw new Error(`expected a string argument, found ${typeof(arg)}`);
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
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
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


//#endregion

//#region wasm loading
let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
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
//#endregion
export { wasm as __wasm }
