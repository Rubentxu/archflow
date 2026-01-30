/**
 * ArchFlow Web SDK Loader
 *
 * This module handles loading the WebAssembly module and provides
 * a unified API for the ArchFlow editor regardless of whether
 * the full SDK or standalone implementation is used.
 */

let wasmInstance = null;
let wasmModule = null;

/**
 * Initialize the WASM module with the canvas element
 * @param {HTMLCanvasElement} canvasElement - The canvas element to bind to
 * @returns {Promise<Object>} The initialized WASM instance
 */
export async function initWasm(canvasElement) {
  if (wasmInstance) {
    console.log("WASM already initialized, reusing instance");
    return wasmInstance;
  }

  try {
    console.log("Loading ArchFlow WASM module...");

    // Dynamic import of the WASM module
    wasmModule = await import("./archflow_web.js");
    console.log("WASM module loaded");

    // Initialize the WASM module
    if (typeof wasmModule.default === "function") {
      await wasmModule.default();
      console.log("WASM default initialization complete");
    }

    // Verify canvas element
    if (!canvasElement) {
      console.error("Canvas element is null or undefined");
      throw new Error("Canvas element not provided");
    }

    console.log("Canvas tag:", canvasElement.tagName);
    console.log("Canvas id:", canvasElement.id);

    // Ensure canvas has proper dimensions
    const container = canvasElement.parentElement;
    if (container) {
      const rect = container.getBoundingClientRect();
      if (canvasElement.width === 0) canvasElement.width = rect.width || 800;
      if (canvasElement.height === 0) canvasElement.height = rect.height || 600;
      console.log(
        "Canvas dimensions set to:",
        canvasElement.width,
        "x",
        canvasElement.height,
      );
    }

    // Create editor instance
    console.log("Creating ArchFlowEditor instance...");
    const editor = new wasmModule.ArchFlowEditor(canvasElement);
    console.log("ArchFlowEditor created successfully");

    wasmInstance = {
      editor,
      module: wasmModule,
      version: getVersion(),
    };

    console.log("ArchFlow WASM initialization complete", {
      version: wasmInstance.version,
      features: getFeatures(),
    });

    return wasmInstance;
  } catch (error) {
    console.error("Failed to load WASM:", error);
    throw error;
  }
}

/**
 * Get the current WASM instance
 * @returns {Object} The current WASM instance
 * @throws {Error} If WASM is not initialized
 */
export function getWasm() {
  if (!wasmInstance) {
    throw new Error(
      "WASM not initialized. Call initWasm(canvasElement) first. " +
        "Make sure the WASM module is built and the canvas element exists.",
    );
  }
  return wasmInstance;
}

/**
 * Get the editor instance
 * @returns {ArchFlowEditor} The editor instance
 */
export function getEditor() {
  return getWasm().editor;
}

/**
 * Get the ArchFlow module
 * @returns {Object} The WASM module
 */
export function getModule() {
  return getWasm().module;
}

/**
 * Get the SDK version
 * @returns {string} The version string
 */
function getVersion() {
  if (wasmModule && wasmModule.VERSION) {
    return wasmModule.VERSION;
  }
  return "0.1.0"; // Default version
}

/**
 * Get available features
 * @returns {Object} Feature flags
 */
function getFeatures() {
  return {
    sdk: true,
    c4Layers: true,
    grid: true,
    selection: true,
    viewport: true,
    coordinateConversion: true,
  };
}

/**
 * Check if WASM is initialized
 * @returns {boolean} True if WASM is initialized
 */
export function isWasmReady() {
  return wasmInstance !== null;
}

/**
 * Destroy the WASM instance (cleanup)
 */
export function destroyWasm() {
  if (wasmInstance) {
    const editor = wasmInstance.editor;

    // Cleanup if the editor has a destroy method
    if (editor && typeof editor.destroy === "function") {
      editor.destroy();
    }

    wasmInstance = null;
    wasmModule = null;
    console.log("WASM instance destroyed");
  }
}

/**
 * Re-initialize WASM (useful for hot reload scenarios)
 * @param {HTMLCanvasElement} canvasElement - The canvas element
 * @returns {Promise<Object>} The new WASM instance
 */
export async function reinitWasm(canvasElement) {
  destroyWasm();
  return initWasm(canvasElement);
}

// ============ Convenience Functions ============

/**
 * Create a rectangle shape
 * @param {number} x - X position
 * @param {number} y - Y position
 * @param {number} width - Width
 * @param {number} height - Height
 * @returns {string} Shape ID
 */
export function createRectangle(x, y, width, height) {
  return getEditor().add_rect(x, y, width, height);
}

/**
 * Create an ellipse shape
 * @param {number} x - X position (center)
 * @param {number} y - Y position (center)
 * @param {number} radiusX - X radius
 * @param {number} radiusY - Y radius
 * @returns {string} Shape ID
 */
export function createEllipse(x, y, radiusX, radiusY) {
  return getEditor().add_ellipse(x, y, radiusX, radiusY);
}

/**
 * Create a line shape
 * @param {number} x1 - Start X
 * @param {number} y1 - Start Y
 * @param {number} x2 - End X
 * @param {number} y2 - End Y
 * @returns {string} Shape ID
 */
export function createLine(x1, y1, x2, y2) {
  return getEditor().add_line(x1, y1, x2, y2);
}

/**
 * Create a text shape
 * @param {number} x - X position
 * @param {number} y - Y position
 * @param {string} text - Text content
 * @returns {string} Shape ID
 */
export function createText(x, y, text) {
  return getEditor().add_text(x, y, text);
}

/**
 * Delete a shape by ID
 * @param {string} id - Shape ID
 * @returns {boolean} True if deleted
 */
export function deleteShape(id) {
  return getEditor().delete_shape(id);
}

/**
 * Get a shape by ID
 * @param {string} id - Shape ID
 * @returns {Object|null} Shape data or null
 */
export function getShape(id) {
  return getEditor().get_shape(id);
}

/**
 * Get all shapes
 * @returns {Array} Array of shape objects
 */
export function getAllShapes() {
  return getEditor().get_all_shapes();
}

/**
 * Select a shape
 * @param {string} id - Shape ID
 */
export function selectShape(id) {
  getEditor().select(id);
}

/**
 * Clear selection
 */
export function clearSelection() {
  getEditor().clear_selection();
}

/**
 * Get current selection
 * @returns {Object} Selection data
 */
export function getSelection() {
  return getEditor().get_selection();
}

/**
 * Pan the viewport
 * @param {number} dx - Delta X
 * @param {number} dy - Delta Y
 */
export function pan(dx, dy) {
  getEditor().pan(dx, dy);
}

/**
 * Zoom at a point
 * @param {number} x - Screen X
 * @param {number} y - Screen Y
 * @param {number} factor - Zoom factor
 */
export function zoomAt(x, y, factor) {
  getEditor().zoom_at(x, y, factor);
}

/**
 * Zoom in
 */
export function zoomIn() {
  getEditor().zoom_in();
}

/**
 * Zoom out
 */
export function zoomOut() {
  getEditor().zoom_out();
}

/**
 * Zoom to fit all content
 */
export function zoomToFit() {
  getEditor().zoom_to_fit();
}

/**
 * Get current zoom level
 * @returns {number} Zoom factor
 */
export function getZoom() {
  return getEditor().get_zoom();
}

/**
 * Set zoom level
 * @param {number} factor - Zoom factor
 */
export function setZoom(factor) {
  getEditor().set_zoom(factor);
}

/**
 * Convert screen to canvas coordinates
 * @param {number} x - Screen X
 * @param {number} y - Screen Y
 * @returns {Object} Canvas coordinates {x, y}
 */
export function screenToCanvas(x, y) {
  return getEditor().screen_to_canvas(x, y);
}

/**
 * Convert canvas to screen coordinates
 * @param {number} x - Canvas X
 * @param {number} y - Canvas Y
 * @returns {Object} Screen coordinates {x, y}
 */
export function canvasToScreen(x, y) {
  return getEditor().canvas_to_screen(x, y);
}

/**
 * Get current C4 level
 * @returns {string} C4 level (Context, Container, Component, Code)
 */
export function getC4Level() {
  return getEditor().get_c4_level();
}

/**
 * Set C4 level
 * @param {string} level - C4 level
 */
export function setC4Level(level) {
  getEditor().set_c4_level(level);
}

/**
 * Render the canvas
 */
export function render() {
  getEditor().render();
}

/**
 * Start simulation mode
 */
export function startSimulation() {
  getEditor().start_simulation();
}

/**
 * Stop simulation mode
 */
export function stopSimulation() {
  getEditor().stop_simulation();
}

/**
 * Deploy architecture
 */
export function deployArchitecture() {
  getEditor().deploy_architecture();
}
