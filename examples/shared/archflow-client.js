// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Client — Reusable WASM Engine Bootstrap
// ═══════════════════════════════════════════════════════════════════════════════
//
// Eliminates all boilerplate for ArchFlow WASM projects:
//   - WASM module loading + WasmBridge creation
//   - Canvas setup (fullscreen, prevent defaults)
//   - Engine initialization (initialize + initialize_graphics)
//   - Efficient input (push_input_event + PointerEvents + coalescing)
//   - Resize handling (canvas + bridge.resize)
//   - Render loop (requestAnimationFrame + tick)
//
// Usage:
//   import { createEngine } from "../shared/archflow-client.js";
//   const engine = await createEngine("canvas");
//   // Engine running! Use engine.bridge for API calls
//
// ═══════════════════════════════════════════════════════════════════════════════

import init, { WasmBridge } from "../wasm/archflow_web.js";

// ─── Input Event Types (matches WASM InputEventType enum) ───────────────────

const EventType = {
    DOWN: 0,
    MOVE: 1,
    UP: 2,
    WHEEL: 3,
    KEY_DOWN: 4,
    KEY_UP: 5,
};

// ─── Modifier/Button Helpers ────────────────────────────────────────────────

function getModifiers(e) {
    let bits = 0;
    if (e.shiftKey) bits |= 1;
    if (e.ctrlKey) bits |= 2;
    if (e.altKey) bits |= 4;
    if (e.metaKey) bits |= 8;
    return bits;
}

function getButtons(e) {
    return e.buttons & 0x07;
}

// ─── Input Setup ────────────────────────────────────────────────────────────

function setupInput(canvas, bridge, callbacks) {
    canvas.addEventListener("pointerdown", (e) => {
        e.preventDefault();
        canvas.setPointerCapture(e.pointerId);
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        bridge.push_input_event(EventType.DOWN, x, y, getButtons(e), getModifiers(e));
        if (callbacks.onPointerDown) callbacks.onPointerDown(x, y, e.button);
    });

    canvas.addEventListener("pointermove", (e) => {
        e.preventDefault();
        const rect = canvas.getBoundingClientRect();
        const events = e.getCoalescedEvents ? e.getCoalescedEvents() : [e];
        for (const evt of events) {
            bridge.push_input_event(
                EventType.MOVE,
                evt.clientX - rect.left,
                evt.clientY - rect.top,
                getButtons(evt),
                getModifiers(evt),
            );
        }
        if (callbacks.onPointerMove) {
            const last = events[events.length - 1];
            callbacks.onPointerMove(last.clientX - rect.left, last.clientY - rect.top);
        }
    });

    canvas.addEventListener("pointerup", (e) => {
        e.preventDefault();
        canvas.releasePointerCapture(e.pointerId);
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        bridge.push_input_event(EventType.UP, x, y, 0, getModifiers(e));
        if (callbacks.onPointerUp) callbacks.onPointerUp(x, y);
    });

    canvas.addEventListener("wheel", (e) => {
        e.preventDefault();
        const rect = canvas.getBoundingClientRect();
        bridge.push_input_event(
            EventType.WHEEL,
            e.clientX - rect.left,
            e.clientY - rect.top,
            0,
            getModifiers(e),
        );
        if (callbacks.onWheel) callbacks.onWheel(e.deltaY);
    }, { passive: false });

    canvas.addEventListener("keydown", (e) => {
        bridge.push_input_event(EventType.KEY_DOWN, e.keyCode || e.which, 0, 0, getModifiers(e));
        if (callbacks.onKey) callbacks.onKey(e.keyCode || e.which, true);
    });

    canvas.addEventListener("keyup", (e) => {
        bridge.push_input_event(EventType.KEY_UP, e.keyCode || e.which, 0, 0, getModifiers(e));
        if (callbacks.onKey) callbacks.onKey(e.keyCode || e.which, false);
    });

    canvas.addEventListener("contextmenu", (e) => e.preventDefault());
    canvas.tabIndex = 0;
}

// ─── Render Loop ────────────────────────────────────────────────────────────

function createLoop(bridge, onFrame) {
    let lastTime = performance.now();
    let running = true;
    let frameCount = 0;
    let fps = 0;
    let lastFpsTime = performance.now();

    function loop(currentTime) {
        if (!running) return;
        const dt = (currentTime - lastTime) / 1000;
        lastTime = currentTime;

        bridge.tick(dt);

        // FPS calculation
        frameCount++;
        if (currentTime - lastFpsTime >= 1000) {
            fps = Math.round(frameCount * 1000 / (currentTime - lastFpsTime));
            frameCount = 0;
            lastFpsTime = currentTime;
        }

        if (onFrame) onFrame(dt, fps);
        requestAnimationFrame(loop);
    }

    requestAnimationFrame(loop);
    return {
        stop: () => { running = false; },
        get fps() { return fps; },
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Create and initialize an ArchFlow WASM engine.
 *
 * Handles ALL boilerplate:
 *   - WASM module loading
 *   - WasmBridge creation + initialization
 *   - Canvas fullscreen setup
 *   - Efficient input (push_input_event + PointerEvents + coalescing)
 *   - Resize handling
 *   - Render loop
 *
 * @param {string|HTMLCanvasElement} canvasOrId - Canvas element or its ID
 * @param {Object} [options] - Configuration options
 * @param {boolean} [options.fullscreen=true] - Make canvas fullscreen
 * @param {function} [options.onFrame] - Called each frame with (dt, fps)
 * @param {function} [options.onPointerDown] - Called on pointer down with (x, y, button)
 * @param {function} [options.onPointerMove] - Called on pointer move with (x, y)
 * @param {function} [options.onPointerUp] - Called on pointer up with (x, y)
 * @param {function} [options.onWheel] - Called on wheel with (deltaY)
 * @param {function} [options.onKey] - Called on key with (keyCode, isDown)
 * @param {function} [options.onReady] - Called when engine is ready with (bridge)
 *
 * @returns {Promise<{bridge: WasmBridge, loop: {stop, fps}, canvas: HTMLCanvasElement}>}
 *
 * @example
 *   // Minimal — zero boilerplate
 *   const { bridge } = await createEngine("canvas");
 *
 * @example
 *   // With options
 *   const { bridge, canvas } = await createEngine("canvas", {
 *       onFrame: (dt, fps) => updateHUD(fps),
 *       onPointerDown: (x, y) => showCoords(x, y),
 *   });
 *   bridge.set_active_color(255, 0, 0, 255);
 */
export async function createEngine(canvasOrId, options = {}) {
    const {
        fullscreen = true,
        onFrame = null,
        onPointerDown = null,
        onPointerMove = null,
        onPointerUp = null,
        onWheel = null,
        onKey = null,
        onReady = null,
    } = options;

    // Resolve canvas element
    const canvas =
        typeof canvasOrId === "string"
            ? document.getElementById(canvasOrId)
            : canvasOrId;

    if (!canvas) throw new Error(`Canvas not found: ${canvasOrId}`);

    // Fullscreen setup
    if (fullscreen) {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
    }

    // Initialize WASM
    await init();

    // Create and initialize bridge
    const bridge = new WasmBridge();
    bridge.initialize(canvas.width, canvas.height);
    bridge.initialize_graphics(canvas);

    // Efficient input via ring buffer
    setupInput(canvas, bridge, {
        onPointerDown,
        onPointerMove,
        onPointerUp,
        onWheel,
        onKey,
    });

    // Resize handling
    if (fullscreen) {
        window.addEventListener("resize", () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
            bridge.resize(canvas.width, canvas.height);
        });
    }

    // Start render loop
    const loop = createLoop(bridge, onFrame);

    if (onReady) onReady(bridge);

    return { bridge, loop, canvas };
}

/**
 * Re-export EventType constants for advanced use cases
 */
export { EventType };
