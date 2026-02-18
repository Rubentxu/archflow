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

import init, { WasmBridge } from "../pkg/archflow_wasm_bridge.js";

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
    bridge.push_input_event(
      EventType.DOWN,
      x,
      y,
      getButtons(e),
      getModifiers(e),
    );
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
      callbacks.onPointerMove(
        last.clientX - rect.left,
        last.clientY - rect.top,
      );
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

  canvas.addEventListener(
    "wheel",
    (e) => {
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
    },
    { passive: false },
  );

  canvas.addEventListener("keydown", (e) => {
    bridge.push_input_event(
      EventType.KEY_DOWN,
      e.keyCode || e.which,
      0,
      0,
      getModifiers(e),
    );
    if (callbacks.onKey) callbacks.onKey(e.keyCode || e.which, true);
  });

  canvas.addEventListener("keyup", (e) => {
    bridge.push_input_event(
      EventType.KEY_UP,
      e.keyCode || e.which,
      0,
      0,
      getModifiers(e),
    );
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

    // IMPORTANT: bridge.tick() expects a millisecond timestamp (performance.now()),
    // NOT a delta-time. It uses the timestamp internally to compute fixed-timestep physics.
    bridge.tick(currentTime);

    // FPS calculation
    frameCount++;
    if (currentTime - lastFpsTime >= 1000) {
      fps = Math.round((frameCount * 1000) / (currentTime - lastFpsTime));
      frameCount = 0;
      lastFpsTime = currentTime;
    }

    if (onFrame) onFrame(dt, fps);
    requestAnimationFrame(loop);
  }

  requestAnimationFrame(loop);
  return {
    stop: () => {
      running = false;
    },
    get fps() {
      return fps;
    },
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

// ═══════════════════════════════════════════════════════════════════════════════
// LOGIC BRICKS HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Sensor types (matches WASM SensorType enum)
 */
export const SensorType = {
  MOUSE_OVER: 0,
  MOUSE_CLICK: 1,
  PROXIMITY: 2,
  KEY_SHORTCUT: 3,
  TOUCH: 4,
  RADAR: 5,
  DOUBLE_TAP: 6,
  LONG_PRESS: 7,
  RIGHT_CLICK: 8,
};

/**
 * Controller types (matches WASM Controller enum)
 */
export const ControllerType = {
  DIRECT: 0,
  AND: 1,
  OR: 2,
  NOT: 3,
  BLINKY: 4,
  DEBOUNCE: 5,
};

/**
 * Actuator types (matches WASM ActuatorType enum)
 */
export const ActuatorType = {
  HIGHLIGHT: 0,
  SELECT: 1,
  MOVE: 2,
  DELETE: 3,
  UNDO: 4,
  REDO: 5,
  CAMERA: 6,
};

/**
 * Add a complete Logic Bricks behavior to an entity
 *
 * @param {WasmBridge} bridge - The WASM bridge instance
 * @param {number} entityId - Entity ID
 * @param {Object} config - Behavior configuration
 * @param {number} [config.sensor=SensorType.MOUSE_OVER] - Sensor type
 * @param {number} [config.controller=ControllerType.DIRECT] - Controller type
 * @param {number} [config.actuator=ActuatorType.HIGHLIGHT] - Actuator type
 *
 * @example
 * // Add hover highlight to entity
 * addBehavior(bridge, entityId, {
 *   sensor: SensorType.MOUSE_OVER,
 *   actuator: ActuatorType.HIGHLIGHT
 * });
 *
 * @example
 * // Add click to select
 * addBehavior(bridge, entityId, {
 *   sensor: SensorType.MOUSE_CLICK,
 *   actuator: ActuatorType.SELECT
 * });
 */
export function addBehavior(bridge, entityId, config = {}) {
  const {
    sensor = SensorType.MOUSE_OVER,
    controller = ControllerType.DIRECT,
    actuator = ActuatorType.HIGHLIGHT,
  } = config;

  return bridge.add_sensor(entityId, sensor, controller, actuator);
}

/**
 * Create multiple behaviors for an entity
 *
 * @param {WasmBridge} bridge - The WASM bridge instance
 * @param {number} entityId - Entity ID
 * @param {Array} behaviors - Array of behavior configs
 *
 * @example
 * addBehaviors(bridge, entityId, [
 *   { sensor: SensorType.MOUSE_OVER, actuator: ActuatorType.HIGHLIGHT },
 *   { sensor: SensorType.MOUSE_CLICK, actuator: ActuatorType.SELECT },
 *   { sensor: SensorType.RIGHT_CLICK, actuator: ActuatorType.DELETE },
 * ]);
 */
export function addBehaviors(bridge, entityId, behaviors) {
  const results = [];
  for (const behavior of behaviors) {
    results.push(addBehavior(bridge, entityId, behavior));
  }
  return results;
}

/**
 * Query entities matching criteria
 *
 * @param {WasmBridge} bridge - The WASM bridge instance
 * @param {string} queryType - Query type: 'all', 'visible', 'selected', 'shape', 'bounds'
 * @param {Object} [options] - Query options
 * @param {number} [options.shape] - Shape type (0=rect, 1=circle)
 * @param {number} [options.x] - Bounds x
 * @param {number} [options.y] - Bounds y
 * @param {number} [options.width] - Bounds width
 * @param {number} [options.height] - Bounds height
 *
 * @returns {Uint32Array} Array of entity IDs
 *
 * @example
 * // Get all entities
 * const all = queryEntities(bridge, 'all');
 *
 * @example
 * // Get selected entities
 * const selected = queryEntities(bridge, 'selected');
 *
 * @example
 * // Get rectangles
 * const rects = queryEntities(bridge, 'shape', { shape: 0 });
 */
export function queryEntities(bridge, queryType, options = {}) {
  switch (queryType) {
    case "all":
      return bridge.get_alive_entities();
    case "visible":
      return bridge.query_by_visibility(true);
    case "selected":
      const sel = bridge.get_selection();
      return sel || new Uint32Array(0);
    case "shape":
      return bridge.query_by_shape(options.shape || 0);
    case "bounds":
      return bridge.query_in_bounds(
        options.x || 0,
        options.y || 0,
        options.width || 100,
        options.height || 100,
      );
    default:
      console.warn(`Unknown query type: ${queryType}`);
      return new Uint32Array(0);
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTITY HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Spawn an entity with the given properties
 *
 * @param {WasmBridge} bridge - The WASM bridge instance
 * @param {Object} props - Entity properties
 * @param {number} [props.x=400] - X position
 * @param {number} [props.y=300] - Y position
 * @param {number} [props.width=50] - Width
 * @param {number} [props.height=50] - Height
 * @param {number} [props.color=0x3B82F6FF] - Color (RGBA as hex)
 * @param {number} [props.shape=0] - Shape type (0=rectangle, 1=circle)
 *
 * @returns {number} Entity ID
 *
 * @example
 * const entityId = spawnEntity(bridge, {
 *   x: 400,
 *   y: 300,
 *   width: 100,
 *   height: 60,
 *   color: 0x3B82F6FF,
 *   shape: 0
 * });
 */
export function spawnEntity(bridge, props = {}) {
  const x = props.x || 400;
  const y = props.y || 300;
  const width = props.width || 50;
  const height = props.height || 50;
  const shape = props.shape || 0;
  const color = props.color || 0x3B82F6FF;

  const entityId = bridge.spawn_entity(x, y, width, height);

  if (color !== undefined) {
    const r = (color >> 24) & 0xFF;
    const g = (color >> 16) & 0xFF;
    const b = (color >> 8) & 0xFF;
    const a = color & 0xFF;
    bridge.set_color(entityId, r, g, b, a);
  }

  if (shape !== undefined) {
    bridge.set_shape(entityId, shape);
  }

  return entityId;
}

/**
 * Create a performance/effects HUD overlay
 *
 * @param {HTMLElement} container - Container element to append HUD to
 * @param {Object} options - Configuration
 * @param {boolean} [options.showFps=true] - Show FPS counter
 * @param {boolean} [options.showEntities=true] - Show entity count
 * @param {boolean} [options.showSpawns=false] - Show spawn counter
 * @param {string} [options.position='top-left'] - Position: 'top-left', 'top-right', 'bottom-left', 'bottom-right'
 *
 * @returns {Object} HUD controller with update method
 *
 * @example
 * const hud = createHUD(document.body, { showFps: true, showEntities: true });
 * // Call hud.update(fps, entityCount) each frame
 */
export function createHUD(container, options = {}) {
  const {
    showFps = true,
    showEntities = true,
    showSpawns = false,
    position = 'top-left'
  } = options;

  // Create HUD element
  const hud = document.createElement('div');
  hud.style.cssText = `
    position: absolute;
    ${position.includes('top') ? 'top: 12px' : 'bottom: 12px'};
    ${position.includes('left') ? 'left: 12px' : 'right: 12px'};
    background: rgba(13, 17, 23, 0.85);
    border: 1px solid #30363d;
    border-radius: 6px;
    padding: 10px 14px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 12px;
    color: #c9d1d9;
    z-index: 10000;
    pointer-events: none;
    user-select: none;
    min-width: 140px;
  `;

  // FPS line
  let fpsHtml = '';
  if (showFps) {
    fpsHtml = `<div style="color: #7ee787; margin-bottom: 4px;">FPS: <span id="hud-fps">0</span></div>`;
  }

  // Entities line
  let entitiesHtml = '';
  if (showEntities) {
    entitiesHtml = `<div style="color: #79c0ff;">Entities: <span id="hud-entities">0</span></div>`;
  }

  // Spawns line
  let spawnsHtml = '';
  if (showSpawns) {
    spawnsHtml = `<div style="color: #ffa657;">Spawns: <span id="hud-spawns">0</span></div>`;
  }

  hud.innerHTML = fpsHtml + entitiesHtml + spawnsHtml;

  // Append to container (or body if container not specified)
  (container || document.body).appendChild(hud);

  // Get references to span elements
  const fpsEl = showFps ? hud.querySelector('#hud-fps') : null;
  const entitiesEl = showEntities ? hud.querySelector('#hud-entities') : null;
  const spawnsEl = showSpawns ? hud.querySelector('#hud-spawns') : null;

  // State
  let spawnCount = 0;

  return {
    /**
     * Update HUD display
     * @param {number} fps - Current FPS
     * @param {number} entityCount - Current entity count
     * @param {number} [newSpawns=0] - Number of new spawns to add
     */
    update: function(fps, entityCount, newSpawns = 0) {
      if (fpsEl) fpsEl.textContent = fps;
      if (entitiesEl) entitiesEl.textContent = entityCount;
      if (spawnsEl) {
        spawnCount += newSpawns;
        spawnsEl.textContent = spawnCount;
      }
    },
    /**
     * Increment spawn counter
     * @param {number} count - Number to add
     */
    addSpawns: function(count = 1) {
      spawnCount += count;
      if (spawnsEl) spawnsEl.textContent = spawnCount;
    },
    /**
     * Reset spawn counter
     */
    resetSpawns: function() {
      spawnCount = 0;
      if (spawnsEl) spawnsEl.textContent = 0;
    },
    /**
     * Remove HUD from DOM
     */
    destroy: function() {
      if (hud.parentNode) {
        hud.parentNode.removeChild(hud);
      }
    }
  };
}
