/**
 * ArchFlow Bridge - Main SDK Facade
 *
 * Unified entry point for all ArchFlow WASM bridge operations.
 * Provides organized access to all domain-specific bridges.
 *
 * Architecture Reference: ARCHITECTURE-CLEAN-BRIDGE.md
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * ORGANIZATION
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * This facade groups related operations into specialized bridges:
 * - EntityBridge: Entity lifecycle and properties
 * - SelectionBridge: Selection operations
 * - CameraBridge: Viewport and camera controls
 * - InputBridge: Mouse, keyboard, and input handling
 * - HistoryBridge: Undo/redo operations
 * - EventsBridge: Event loop and tick management
 * - ToolsBridge: Tool selection and state
 * - BehaviorBridge: Logic Bricks integration
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 */

import type { WasmBridge } from "../wasm/archflow_web.d";

import { EntityBridge, createEntityBridge } from "./EntityBridge";
import { SelectionBridge, createSelectionBridge } from "./SelectionBridge";
import { CameraBridge, createCameraBridge } from "./CameraBridge";
import { InputBridge, createInputBridge } from "./InputBridge";
import { HistoryBridge, createHistoryBridge } from "./HistoryBridge";
import { EventsBridge, createEventsBridge } from "./EventsBridge";
import { ToolsBridge, createToolsBridge } from "./ToolsBridge";
import { BehaviorBridge, createBehaviorBridge } from "./BehaviorBridge";

/**
 * ArchFlow Bridge initialization options
 */
export interface ArchFlowBridgeOptions {
  /** Enable debug logging */
  debug?: boolean;
  /** Initial tool */
  initialTool?: "select" | "pan" | "rectangle" | "circle" | "ellipse";
  /** Canvas dimensions */
  width?: number;
  /** Canvas dimensions */
  height?: number;
}

/**
 * ArchFlowBridge - Main SDK Facade
 *
 * Provides a unified interface to all WASM bridge operations through
 * specialized bridge instances. All operations are available through
 * the main bridge or through individual bridge properties.
 *
 * @example
 * ```typescript
 * import { ArchFlowBridge } from './ArchFlowBridge';
 *
 * // Create bridge from existing WASM bridge
 * const archflow = ArchFlowBridge.fromWasm(bridge);
 *
 * // Access specialized bridges
 * const entities = archflow.entities;
 * const selection = archflow.selection;
 * const camera = archflow.camera;
 *
 * // Create and use entities
 * const id = archflow.entities.spawn(100, 100, 200, 150);
 *
 * // Manipulate selection
 * archflow.selection.select(id);
 *
 * // Control camera
 * archflow.camera.setZoom(1.5);
 * ```
 */
export class ArchFlowBridge {
  /**
   * The underlying WASM bridge
   */
  readonly wasmBridge: WasmBridge;

  /**
   * Entity operations bridge
   */
  readonly entities: EntityBridge;

  /**
   * Selection operations bridge
   */
  readonly selection: SelectionBridge;

  /**
   * Camera and viewport operations bridge
   */
  readonly camera: CameraBridge;

  /**
   * Input handling bridge
   */
  readonly input: InputBridge;

  /**
   * History/undo-redo bridge
   */
  readonly history: HistoryBridge;

  /**
   * Event loop bridge
   */
  readonly events: EventsBridge;

  /**
   * Tool management bridge
   */
  readonly tools: ToolsBridge;

  /**
   * Behavior/Logic Bricks bridge
   */
  readonly behaviors: BehaviorBridge;

  /**
   * Whether debug mode is enabled
   */
  private debug: boolean;

  /**
   * Create ArchFlowBridge from existing WASM bridge
   *
   * @param wasmBridge - The WASM bridge instance
   * @param options - Configuration options
   * @returns Configured ArchFlowBridge instance
   *
   * @example
   * ```typescript
   * import init, { WasmBridge } from '../wasm/archflow_web';
   * import { ArchFlowBridge } from './ArchFlowBridge';
   *
   * // Initialize WASM
   * await init();
   * const wasmBridge = new WasmBridge();
   *
   * // Create facade
   * const archflow = ArchFlowBridge.fromWasm(wasmBridge);
   * ```
   */
  static fromWasm(wasmBridge: WasmBridge, options?: ArchFlowBridgeOptions): ArchFlowBridge {
    return new ArchFlowBridge(wasmBridge, options);
  }

  /**
   * Private constructor - use static factory method
   */
  private constructor(wasmBridge: WasmBridge, options?: ArchFlowBridgeOptions) {
    this.wasmBridge = wasmBridge;
    this.debug = options?.debug ?? false;

    // Initialize specialized bridges
    this.entities = createEntityBridge(wasmBridge);
    this.selection = createSelectionBridge(wasmBridge);
    this.camera = createCameraBridge(wasmBridge);
    this.input = createInputBridge(wasmBridge);
    this.history = createHistoryBridge(wasmBridge);
    this.events = createEventsBridge(wasmBridge);
    this.tools = createToolsBridge(wasmBridge);
    this.behaviors = createBehaviorBridge(wasmBridge);

    // Apply initial configuration
    if (options?.initialTool) {
      this.tools.setTool(options.initialTool);
    }

    if (this.debug) {
      console.debug("[ArchFlowBridge] Initialized with debug enabled");
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // INITIALIZATION
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Initialize graphics with canvas element
   *
   * @param canvas - HTML canvas element
   * @returns Promise that resolves when initialized
   *
   * @example
   * ```typescript
   * const canvas = document.getElementById('canvas') as HTMLCanvasElement;
   * await archflow.initializeGraphics(canvas);
   * ```
   */
  async initializeGraphics(canvas: HTMLCanvasElement): Promise<void> {
    await this.wasmBridge.initialize_graphics(canvas);
    this.log("[ArchFlowBridge] Graphics initialized");
  }

  /**
   * Resize the canvas
   *
   * @param width - New width
   * @param height - New height
   *
   * @example
   * ```typescript
   * archflow.resize(800, 600);
   * ```
   */
  resize(width: number, height: number): void {
    this.wasmBridge.resize(width, height);
    this.log(`[ArchFlowBridge] Resized to ${width}x${height}`);
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // PROJECT OPERATIONS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Serialize project to bytes
   *
   * @returns Project data as Uint8Array
   *
   * @example
   * ```typescript
   * const data = archflow.serializeProject();
   * localStorage.setItem('project', JSON.stringify(data));
   * ```
   */
  serializeProject(): Uint8Array {
    return this.wasmBridge.serialize_project();
  }

  /**
   * Clear all entities
   *
   * @example
   * ```typescript
   * archflow.clear();
   * ```
   */
  clear(): void {
    this.wasmBridge.clear();
    this.log("[ArchFlowBridge] Canvas cleared");
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TICK & EVENTS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Process a tick/update
   *
   * @param timestamp - Current timestamp in milliseconds
   *
   * @example
   * ```typescript
   * function animate(timestamp: number) {
   *   archflow.tick(timestamp);
   *   requestAnimationFrame(animate);
   * }
   * ```
   */
  tick(timestamp: number): void {
    this.wasmBridge.tick(timestamp);
  }

  /**
   * Poll for events
   *
   * @returns Number of events processed
   *
   * @example
   * ```typescript
   * const count = archflow.pollEvents();
   * if (count > 0) {
   *   // Handle new events
   * }
   * ```
   */
  pollEvents(): number {
    return this.wasmBridge.poll_events();
  }

  /**
   * Process pending input events
   *
   * @example
   * ```typescript
   * archflow.processInputEvents();
   * ```
   */
  processInputEvents(): void {
    this.wasmBridge.process_input_events();
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // CONVENIENCE: COMPOSED OPERATIONS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Create a shape with selection enabled
   *
   * @param x - X position
   * @param y - Y position
   * @param width - Width
   * @param height - Height
   * @returns Entity ID
   *
   * @example
   * ```typescript
   * const id = archflow.createSelectableShape(100, 100, 200, 150);
   * ```
   */
  createSelectableShape(x: number, y: number, width: number, height: number): number {
    const entityId = this.entities.spawn(x, y, width, height);
    this.selection.select(entityId);
    this.behaviors.attachSelectable(entityId, "single");
    return entityId;
  }

  /**
   * Create a draggable shape
   *
   * @param x - X position
   * @param y - Y position
   * @param width - Width
   * @param height - Height
   * @returns Entity ID
   *
   * @example
   * ```typescript
   * const id = archflow.createDraggableShape(100, 100, 200, 150);
   * ```
   */
  createDraggableShape(x: number, y: number, width: number, height: number): number {
    const entityId = this.entities.spawn(x, y, width, height);
    this.behaviors.attachDraggable(entityId);
    this.behaviors.attachSelectable(entityId, "single");
    return entityId;
  }

  /**
   * Create a fully interactive shape
   *
   * Combines selectable, draggable, and hover highlight.
   *
   * @param x - X position
   * @param y - Y position
   * @param width - Width
   * @param height - Height
   * @param highlightColor - Hover highlight color
   * @returns Entity ID
   *
   * @example
   * ```typescript
   * const id = archflow.createInteractiveShape(100, 100, 200, 150, 0xffff00);
   * ```
   */
  createInteractiveShape(
    x: number,
    y: number,
    width: number,
    height: number,
    highlightColor = 0xffff00,
  ): number {
    const entityId = this.entities.spawn(x, y, width, height);
    this.behaviors.attachHoverHighlight(entityId, highlightColor, 0.2);
    this.behaviors.attachDraggable(entityId);
    this.behaviors.attachSelectable(entityId, "single");
    return entityId;
  }

  /**
   * Focus on entity
   *
   * Centers camera on entity and selects it.
   *
   * @param entityId - Entity to focus on
   * @param zoom - Optional zoom level
   *
   * @example
   * ```typescript
   * archflow.focusOn(5, 2.0);
   * ```
   */
  focusOn(entityId: number, zoom?: number): void {
    const pos = this.entities.getPositionWorld(entityId);
    if (pos) {
      this.camera.setCenter(pos.x, pos.y);
      if (zoom !== undefined) {
        this.camera.setZoom(zoom);
      }
    }
    this.selection.select(entityId);
  }

  /**
   * Zoom to fit all entities
   *
   * @param padding - Padding around entities
   *
   * @example
   * ```typescript
   * archflow.zoomToFit(50);
   * ```
   */
  zoomToFit(padding = 50): void {
    const bounds = this.entities.getBounds();
    if (!bounds) return;

    const width = bounds.maxX - bounds.minX;
    const height = bounds.maxY - bounds.minY;

    if (width === 0 || height === 0) return;

    // Calculate zoom to fit
    const viewWidth = this.camera.getViewportWidth();
    const viewHeight = this.camera.getViewportHeight();
    const zoomX = (viewWidth - padding * 2) / width;
    const zoomY = (viewHeight - padding * 2) / height;
    const zoom = Math.min(zoomX, zoomY, 3); // Max zoom 3x

    // Center camera on entities
    const centerX = (bounds.minX + bounds.maxX) / 2;
    const centerY = (bounds.minY + bounds.maxY) / 2;

    this.camera.setZoom(zoom);
    this.camera.setCenter(centerX, centerY);
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // DEBUG UTILITIES
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Enable or disable debug mode
   *
   * @param enabled - Whether debug is enabled
   */
  setDebug(enabled: boolean): void {
    this.debug = enabled;
  }

  /**
   * Get bridge status for debugging
   *
   * @returns Debug information
   *
   * @example
   * ```typescript
   * console.table(archflow.getDebugStatus());
   * ```
   */
  getDebugStatus(): Record<string, unknown> {
    return {
      entityCount: this.entities.getCount(),
      selection: this.selection.get(),
      zoom: this.camera.getZoom(),
      tool: this.tools.getTool(),
      canUndo: this.history.canUndo(),
      canRedo: this.history.canRedo(),
    };
  }

  /**
   * Log debug message if debug is enabled
   */
  private log(message: string): void {
    if (this.debug) {
      console.debug(message);
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE RE-EXPORTS
// ═══════════════════════════════════════════════════════════════════════════════

// Entity types
export type { EntityId, Position, Size, Bounds } from "./EntityBridge";

// Selection types
export type { SelectionMode } from "./SelectionBridge";

// Camera types
export type { ViewportInfo } from "./CameraBridge";

// Input types
export type { InputState } from "./InputBridge";

// History types
export type { HistoryState } from "./HistoryBridge";

// Tools types
export type { ToolType, ToolInfo } from "./ToolsBridge";

// Behavior types
export type { BehaviorConfig, BehaviorResult } from "./BehaviorBridge";

// ═══════════════════════════════════════════════════════════════════════════════
// DEFAULT EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Create ArchFlowBridge from WASM bridge
 *
 * @param wasmBridge - The WASM bridge instance
 * @param options - Configuration options
 * @returns ArchFlowBridge instance
 *
 * @example
 * ```typescript
 * import { createArchFlowBridge } from './ArchFlowBridge';
 *
 * const archflow = createArchFlowBridge(bridge);
 * ```
 */
export function createArchFlowBridge(
  wasmBridge: WasmBridge,
  options?: ArchFlowBridgeOptions,
): ArchFlowBridge {
  return ArchFlowBridge.fromWasm(wasmBridge, options);
}

export default ArchFlowBridge;
