/* @ts-self-types="./archflow_web.d.ts" */

export class ArchFlowEditor {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ArchFlowEditorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_archfloweditor_free(ptr, 0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} radius_x
     * @param {number} radius_y
     * @returns {string}
     */
    add_ellipse(x, y, radius_x, radius_y) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.archfloweditor_add_ellipse(this.__wbg_ptr, x, y, radius_x, radius_y);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} x1
     * @param {number} y1
     * @param {number} x2
     * @param {number} y2
     * @returns {string}
     */
    add_line(x1, y1, x2, y2) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.archfloweditor_add_line(this.__wbg_ptr, x1, y1, x2, y2);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     * @returns {string}
     */
    add_rect(x, y, width, height) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.archfloweditor_add_rect(this.__wbg_ptr, x, y, width, height);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    clear_selection() {
        wasm.archfloweditor_clear_selection(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    delete_selected() {
        const ret = wasm.archfloweditor_delete_selected(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {string} id
     * @returns {boolean}
     */
    delete_shape(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.archfloweditor_delete_shape(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * @returns {any}
     */
    get_all_shapes() {
        const ret = wasm.archfloweditor_get_all_shapes(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get_selection() {
        const ret = wasm.archfloweditor_get_selection(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} id
     * @returns {any}
     */
    get_shape(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.archfloweditor_get_shape(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_zoom() {
        const ret = wasm.archfloweditor_get_zoom(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {HTMLCanvasElement} canvas
     */
    constructor(canvas) {
        const ret = wasm.archfloweditor_new(canvas);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        ArchFlowEditorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {string} key
     * @param {boolean} _shift
     * @param {boolean} ctrl
     */
    on_keydown(key, _shift, ctrl) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.archfloweditor_on_keydown(this.__wbg_ptr, ptr0, len0, _shift, ctrl);
    }
    /**
     * @param {string} _key
     */
    on_keyup(_key) {
        const ptr0 = passStringToWasm0(_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.archfloweditor_on_keyup(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} button
     */
    on_mousedown(x, y, button) {
        wasm.archfloweditor_on_mousedown(this.__wbg_ptr, x, y, button);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    on_mousemove(x, y) {
        wasm.archfloweditor_on_mousemove(this.__wbg_ptr, x, y);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    on_mouseup(x, y) {
        wasm.archfloweditor_on_mouseup(this.__wbg_ptr, x, y);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {boolean} zoom_out
     */
    on_wheel(x, y, zoom_out) {
        wasm.archfloweditor_on_wheel(this.__wbg_ptr, x, y, zoom_out);
    }
    /**
     * @param {number} dx
     * @param {number} dy
     */
    pan(dx, dy) {
        wasm.archfloweditor_pan(this.__wbg_ptr, dx, dy);
    }
    render() {
        wasm.archfloweditor_render(this.__wbg_ptr);
    }
    /**
     * @param {number} width
     * @param {number} height
     */
    resize(width, height) {
        wasm.archfloweditor_resize(this.__wbg_ptr, width, height);
    }
    /**
     * @param {string} id
     */
    select(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.archfloweditor_select(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {string[]} ids
     */
    select_multiple(ids) {
        const ptr0 = passArrayJsValueToWasm0(ids, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.archfloweditor_select_multiple(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    selection_count() {
        const ret = wasm.archfloweditor_selection_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} factor
     */
    set_zoom(factor) {
        wasm.archfloweditor_set_zoom(this.__wbg_ptr, factor);
    }
    /**
     * @returns {number}
     */
    shape_count() {
        const ret = wasm.archfloweditor_shape_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} factor
     */
    zoom_at(x, y, factor) {
        wasm.archfloweditor_zoom_at(this.__wbg_ptr, x, y, factor);
    }
}
if (Symbol.dispose) ArchFlowEditor.prototype[Symbol.dispose] = ArchFlowEditor.prototype.free;

export class JsAlignmentManager {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsAlignmentManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsalignmentmanager_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.jsalignmentmanager_new();
        this.__wbg_ptr = ret >>> 0;
        JsAlignmentManagerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) JsAlignmentManager.prototype[Symbol.dispose] = JsAlignmentManager.prototype.free;

export class JsGroupManager {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsGroupManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsgroupmanager_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.jsgroupmanager_new();
        this.__wbg_ptr = ret >>> 0;
        JsGroupManagerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) JsGroupManager.prototype[Symbol.dispose] = JsGroupManager.prototype.free;

export class JsLibraryManager {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsLibraryManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jslibrarymanager_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.jslibrarymanager_new();
        this.__wbg_ptr = ret >>> 0;
        JsLibraryManagerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) JsLibraryManager.prototype[Symbol.dispose] = JsLibraryManager.prototype.free;

export class JsPropertiesManager {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsPropertiesManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jspropertiesmanager_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.jspropertiesmanager_new();
        this.__wbg_ptr = ret >>> 0;
        JsPropertiesManagerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) JsPropertiesManager.prototype[Symbol.dispose] = JsPropertiesManager.prototype.free;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_string_get_72fb696202c56729: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_debug_a4099fa12db6cd61: function(arg0) {
            console.debug(arg0);
        },
        __wbg_ellipse_16e802cdb2b872e2: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.ellipse(arg1, arg2, arg3, arg4, arg5, arg6, arg7);
        }, arguments); },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_9a7fe3f932034cde: function(arg0) {
            console.error(arg0);
        },
        __wbg_fillRect_d44afec47e3a3fab: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fill_6f22ceed79b2a0fa: function(arg0, arg1) {
            arg0.fill(arg1);
        },
        __wbg_getContext_2a5764d48600bc43: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getRandomValues_9b655bdd369112f2: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_height_38750dc6de41ee75: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_info_148d043840582012: function(arg0) {
            console.info(arg0);
        },
        __wbg_instanceof_CanvasRenderingContext2d_4bb052fd1c3d134d: function(arg0) {
            let result;
            try {
                result = arg0 instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_lineTo_66ba9f729def29f6: function(arg0, arg1, arg2) {
            arg0.lineTo(arg1, arg2);
        },
        __wbg_log_6b5ca2e6124b2808: function(arg0) {
            console.log(arg0);
        },
        __wbg_moveTo_9aa9e84bb098adc3: function(arg0, arg1, arg2) {
            arg0.moveTo(arg1, arg2);
        },
        __wbg_new_29687311d45777b4: function() { return handleError(function () {
            const ret = new Path2D();
            return ret;
        }, arguments); },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_rect_9e2babb445c2d08e: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.rect(arg1, arg2, arg3, arg4);
        },
        __wbg_restore_0d233789d098ba64: function(arg0) {
            arg0.restore();
        },
        __wbg_save_e0cc2e58b36d33c9: function(arg0) {
            arg0.save();
        },
        __wbg_scale_543277ecf8cf836b: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.scale(arg1, arg2);
        }, arguments); },
        __wbg_setLineDash_ecf27050368658c9: function() { return handleError(function (arg0, arg1) {
            arg0.setLineDash(arg1);
        }, arguments); },
        __wbg_set_3f1d0b984ed272ed: function(arg0, arg1, arg2) {
            arg0[arg1] = arg2;
        },
        __wbg_set_f43e577aea94465b: function(arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        },
        __wbg_set_fillStyle_4c9682826c3e231a: function(arg0, arg1) {
            arg0.fillStyle = arg1;
        },
        __wbg_set_lineWidth_89fa506592f5b994: function(arg0, arg1) {
            arg0.lineWidth = arg1;
        },
        __wbg_set_strokeStyle_bfb8ff99c1593611: function(arg0, arg1) {
            arg0.strokeStyle = arg1;
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_stroke_f260a9f47e0f2017: function(arg0, arg1) {
            arg0.stroke(arg1);
        },
        __wbg_translate_3aa10730376a8c06: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.translate(arg1, arg2);
        }, arguments); },
        __wbg_warn_f7ae1b2e66ccb930: function(arg0) {
            console.warn(arg0);
        },
        __wbg_width_5f66bde2e810fbde: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
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
        "./archflow_web_bg.js": import0,
    };
}

const ArchFlowEditorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_archfloweditor_free(ptr >>> 0, 1));
const JsAlignmentManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsalignmentmanager_free(ptr >>> 0, 1));
const JsGroupManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsgroupmanager_free(ptr >>> 0, 1));
const JsLibraryManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jslibrarymanager_free(ptr >>> 0, 1));
const JsPropertiesManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jspropertiesmanager_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

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

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
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

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
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

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
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
