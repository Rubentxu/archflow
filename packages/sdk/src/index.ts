/**
 * ArchFlow SDK - TypeScript API
 *
 * This file provides the TypeScript API for the ArchFlow editor.
 * The SDK exposes the high-performance Rust engine via a clean JavaScript interface.
 */

// === Types ===

export interface Vec2 {
  x: number;
  y: number;
}

export interface Viewport {
  offset: Vec2;
  zoom: number;
  minZoom: number;
  maxZoom: number;
}

export type C4Level = 'context' | 'container' | 'component' | 'code';

export type GridType = 'dots' | 'lines' | 'isometric';

export interface GridOptions {
  type?: GridType;
  spacing?: number;
  dotRadius?: number;
  dotColor?: string;
  lineColor?: string;
  lineWidth?: number;
  visible?: boolean;
}

export interface LayerConfig {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  opacity: number;
  c4Level: C4Level;
}

export interface ShapeData {
  id: string;
  type: 'rectangle' | 'ellipse' | 'line' | 'path' | 'text' | 'image' | 'group';
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  fillColor: string;
  strokeColor?: string;
  strokeWidth: number;
  opacity: number;
  selected: boolean;
}

export interface Selection {
  shapes: string[];
  bounds: {
    min: Vec2;
    max: Vec2;
  };
  isBox: boolean;
}

export interface EditorOptions {
  canvas: HTMLCanvasElement;
  width?: number;
  height?: number;
  backgroundColor?: string;
  grid?: GridOptions;
  c4Level?: C4Level;
}

// === Event Types ===

export type EditorEventType =
  | 'shapecreate'
  | 'shapeupdate'
  | 'shapedelete'
  | 'selectionchange'
  | 'viewportchange'
  | 'layerchange'
  | 'c4levelchange'
  | 'zoom'
  | 'pan';

export interface EditorEventMap {
  shapecreate: { shape: ShapeData };
  shapeupdate: { id: string; changes: Partial<ShapeData> };
  shapedelete: { id: string };
  selectionchange: Selection;
  viewportchange: Viewport;
  layerchange: { layerId: string; visible?: boolean; locked?: boolean };
  c4levelchange: C4Level;
  zoom: { oldZoom: number; newZoom: number };
  pan: { delta: Vec2 };
}

export type EventCallback<T = unknown> = (event: T) => void;

// === Editor Class ===

/**
 * The main ArchFlow editor class.
 *
 * Example usage:
 * ```typescript
 * const canvas = document.getElementById('canvas') as HTMLCanvasElement;
 * const editor = new ArchFlowEditor({ canvas });
 *
 * // Create a rectangle
 * const rectId = editor.createRectangle(100, 100, 200, 150);
 *
 * // Listen for events
 * editor.on('selectionchange', (selection) => {
 *   console.log('Selected shapes:', selection.shapes);
 * });
 * ```
 */
export class ArchFlowEditor {
  private canvas: HTMLCanvasElement;
  private width: number;
  private height: number;
  private eventListeners: Map<EditorEventType, Set<EventCallback>>;

  constructor(options: EditorOptions) {
    this.canvas = options.canvas;
    this.width = options.width || options.canvas.clientWidth;
    this.height = options.height || options.canvas.clientHeight;
    this.eventListeners = new Map();

    this.setupCanvas();
    this.setupEventListeners();
  }

  private setupCanvas(): void {
    this.canvas.width = this.width;
    this.canvas.height = this.height;
    this.canvas.style.touchAction = 'none';
  }

  private setupEventListeners(): void {
    // Mouse events
    this.canvas.addEventListener('mousedown', this.handleMouseDown.bind(this));
    this.canvas.addEventListener('mousemove', this.handleMouseMove.bind(this));
    this.canvas.addEventListener('mouseup', this.handleMouseUp.bind(this));
    this.canvas.addEventListener('wheel', this.handleWheel.bind(this));

    // Touch events
    this.canvas.addEventListener('touchstart', this.handleTouchStart.bind(this));
    this.canvas.addEventListener('touchmove', this.handleTouchMove.bind(this));
    this.canvas.addEventListener('touchend', this.handleTouchEnd.bind(this));

    // Resize observer
    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        this.resize(entry.contentRect.width, entry.contentRect.height);
      }
    });
    resizeObserver.observe(this.canvas);
  }

  private handleMouseDown(event: MouseEvent): void {
    // TODO: Implement selection and dragging
  }

  private handleMouseMove(event: MouseEvent): void {
    // TODO: Implement dragging and hover effects
  }

  private handleMouseUp(event: MouseEvent): void {
    // TODO: End dragging
  }

  private handleWheel(event: WheelEvent): void {
    event.preventDefault();
    const rect = this.canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    if (event.ctrlKey || event.metaKey) {
      // Zoom
      const factor = event.deltaY < 0 ? 1.1 : 0.9;
      this.zoomAt({ x, y }, factor);
    } else {
      // Pan
      const delta = { x: event.deltaX, y: event.deltaY };
      this.pan(delta);
    }
  }

  private handleTouchStart(event: TouchEvent): void {
    event.preventDefault();
    // TODO: Implement touch selection
  }

  private handleTouchMove(event: TouchEvent): void {
    event.preventDefault();
    // TODO: Implement touch dragging
  }

  private handleTouchEnd(event: TouchEvent): void {
    event.preventDefault();
    // TODO: End touch dragging
  }

  // === Shape Operations ===

  /**
   * Creates a new rectangle shape.
   *
   * @param x - X coordinate of the top-left corner
   * @param y - Y coordinate of the top-left corner
   * @param width - Width of the rectangle
   * @param height - Height of the rectangle
   * @returns The ID of the created shape
   */
  createRectangle(x: number, y: number, width: number, height: number): string {
    // TODO: Call WASM to create rectangle
    const id = crypto.randomUUID();
    this.emit('shapecreate', { id, type: 'rectangle', x, y, width, height, rotation: 0, fillColor: '#3366cc', strokeWidth: 0, opacity: 1, selected: false } as ShapeData);
    return id;
  }

  /**
   * Creates a new ellipse shape.
   *
   * @param x - X coordinate of the center
   * @param y - Y coordinate of the center
   * @param radiusX - Horizontal radius
   * @param radiusY - Vertical radius
   * @returns The ID of the created shape
   */
  createEllipse(x: number, y: number, radiusX: number, radiusY: number): string {
    const id = crypto.randomUUID();
    this.emit('shapecreate', {
      id,
      type: 'ellipse',
      x: x - radiusX,
      y: y - radiusY,
      width: radiusX * 2,
      height: radiusY * 2,
      rotation: 0,
      fillColor: '#339966',
      strokeWidth: 0,
      opacity: 1,
      selected: false,
    } as ShapeData);
    return id;
  }

  /**
   * Creates a new line shape.
   *
   * @param x1 - X coordinate of the start point
   * @param y1 - Y coordinate of the start point
   * @param x2 - X coordinate of the end point
   * @param y2 - Y coordinate of the end point
   * @returns The ID of the created shape
   */
  createLine(x1: number, y1: number, x2: number, y2: number): string {
    const id = crypto.randomUUID();
    const minX = Math.min(x1, x2);
    const minY = Math.min(y1, y2);
    this.emit('shapecreate', {
      id,
      type: 'line',
      x: minX,
      y: minY,
      width: Math.abs(x2 - x1),
      height: Math.abs(y2 - y1),
      rotation: 0,
      fillColor: 'transparent',
      strokeColor: '#4d4d4d',
      strokeWidth: 2,
      opacity: 1,
      selected: false,
    } as ShapeData);
    return id;
  }

  /**
   * Gets a shape by ID.
   *
   * @param id - The shape ID
   * @returns The shape data or null if not found
   */
  getShape(id: string): ShapeData | null {
    // TODO: Get shape from WASM
    return null;
  }

  /**
   * Updates a shape.
   *
   * @param id - The shape ID
   * @param changes - The properties to update
   * @returns True if the shape was found and updated
   */
  updateShape(id: string, changes: Partial<ShapeData>): boolean {
    // TODO: Call WASM to update shape
    this.emit('shapeupdate', { id, changes });
    return true;
  }

  /**
   * Deletes a shape.
   *
   * @param id - The shape ID
   * @returns True if the shape was found and deleted
   */
  deleteShape(id: string): boolean {
    // TODO: Call WASM to delete shape
    this.emit('shapedelete', { id });
    return true;
  }

  // === Selection Operations ===

  /**
   * Gets the current selection.
   */
  getSelection(): Selection {
    // TODO: Get selection from WASM
    return { shapes: [], bounds: { min: { x: 0, y: 0 }, max: { x: 0, y: 0 } }, isBox: false };
  }

  /**
   * Selects a single shape.
   *
   * @param id - The shape ID to select
   */
  select(id: string): void {
    // TODO: Call WASM to select
    this.emit('selectionchange', { shapes: [id], bounds: { min: { x: 0, y: 0 }, max: { x: 0, y: 0 } }, isBox: false });
  }

  /**
   * Selects multiple shapes.
   *
   * @param ids - The shape IDs to select
   */
  selectMultiple(ids: string[]): void {
    // TODO: Call WASM to select multiple
    this.emit('selectionchange', { shapes: ids, bounds: { min: { x: 0, y: 0 }, max: { x: 0, y: 0 } }, isBox: false });
  }

  /**
   * Selects all shapes.
   */
  selectAll(): void {
    // TODO: Call WASM to select all
  }

  /**
   * Clears the selection.
   */
  clearSelection(): void {
    // TODO: Call WASM to clear selection
    this.emit('selectionchange', { shapes: [], bounds: { min: { x: 0, y: 0 }, max: { x: 0, y: 0 } }, isBox: false });
  }

  // === Viewport Operations ===

  /**
   * Gets the current viewport.
   */
  getViewport(): Viewport {
    return { offset: { x: 0, y: 0 }, zoom: 1.0, minZoom: 0.1, maxZoom: 10.0 };
  }

  /**
   * Sets the viewport.
   *
   * @param viewport - The new viewport configuration
   */
  setViewport(viewport: Partial<Viewport>): void {
    // TODO: Call WASM to set viewport
    this.emit('viewportchange', this.getViewport());
  }

  /**
   * Pans the viewport.
   *
   * @param delta - The delta in screen coordinates
   */
  pan(delta: Vec2): void {
    const current = this.getViewport();
    this.setViewport({
      offset: {
        x: current.offset.x - delta.x / current.zoom,
        y: current.offset.y - delta.y / current.zoom,
      },
    });
    this.emit('pan', { delta });
  }

  /**
   * Zooms at a screen point.
   *
   * @param screenPoint - The point to zoom around in screen coordinates
   * @param factor - The zoom factor (e.g., 1.1 for 10% in, 0.9 for 10% out)
   */
  zoomAt(screenPoint: Vec2, factor: number): void {
    const current = this.getViewport();
    const newZoom = Math.max(0.1, Math.min(10.0, current.zoom * factor));
    const zoomRatio = newZoom / current.zoom;
    const newOffset = {
      x: current.offset.x - (screenPoint.x / current.zoom) * (1 - zoomRatio) / zoomRatio,
      y: current.offset.y - (screenPoint.y / current.zoom) * (1 - zoomRatio) / zoomRatio,
    };
    const oldZoom = current.zoom;
    this.setViewport({ zoom: newZoom, offset: newOffset });
    this.emit('zoom', { oldZoom, newZoom });
  }

  /**
   * Zooms in by a factor.
   *
   * @param factor - The zoom factor (default 1.2)
   * @param center - Optional center point (defaults to canvas center)
   */
  zoomIn(factor: number = 1.2, center?: Vec2): void {
    const centerPoint = center || { x: this.width / 2, y: this.height / 2 };
    this.zoomAt(centerPoint, factor);
  }

  /**
   * Zooms out by a factor.
   *
   * @param factor - The zoom factor (default 1.2)
   * @param center - Optional center point (defaults to canvas center)
   */
  zoomOut(factor: number = 1.2, center?: Vec2): void {
    this.zoomAt(center || { x: this.width / 2, y: this.height / 2 }, 1 / factor);
  }

  /**
   * Zooms to fit all content.
   */
  zoomToFit(): void {
    // TODO: Call WASM to zoom to fit
  }

  /**
   * Zooms to fit the selection.
   */
  zoomToSelection(): void {
    // TODO: Call WASM to zoom to selection
  }

  // === Layer Operations ===

  /**
   * Gets all layers.
   */
  getLayers(): LayerConfig[] {
    // TODO: Get layers from WASM
    return [];
  }

  /**
   * Sets layer visibility.
   *
   * @param layerId - The layer ID
   * @param visible - The visibility state
   */
  setLayerVisibility(layerId: string, visible: boolean): void {
    // TODO: Call WASM to set layer visibility
    this.emit('layerchange', { layerId, visible });
  }

  /**
   * Sets the current C4 level.
   *
   * @param level - The C4 level
   * @param animate - Whether to animate the transition
   */
  setC4Level(level: C4Level, animate: boolean = true): void {
    // TODO: Call WASM to set C4 level
    this.emit('c4levelchange', level);
  }

  // === Grid Operations ===

  /**
   * Sets the grid configuration.
   *
   * @param options - The grid options
   */
  setGridConfig(options: GridOptions): void {
    // TODO: Call WASM to set grid config
  }

  // === Event System ===

  /**
   * Subscribes to an event.
   *
   * @param event - The event type
   * @param callback - The callback function
   * @returns A function to unsubscribe
   */
  on<K extends EditorEventType>(event: K, callback: EventCallback<EditorEventMap[K]>): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event)!.add(callback as EventCallback);

    return () => {
      this.eventListeners.get(event)?.delete(callback as EventCallback);
    };
  }

  /**
   * Unsubscribes from an event.
   *
   * @param event - The event type
   * @param callback - Optional callback to remove
   */
  off<K extends EditorEventType>(event: K, callback?: EventCallback<EditorEventMap[K]>): void {
    if (callback) {
      this.eventListeners.get(event)?.delete(callback as EventCallback);
    } else {
      this.eventListeners.delete(event);
    }
  }

  private emit<K extends EditorEventType>(event: K, data: EditorEventMap[K]): void {
    this.eventListeners.get(event)?.forEach((callback) => {
      callback(data);
    });
  }

  // === Render Control ===

  /**
   * Triggers a render.
   */
  render(): void {
    // TODO: Call WASM to render
  }

  /**
   * Resizes the canvas.
   *
   * @param width - The new width
   * @param height - The new height
   */
  resize(width: number, height: number): void {
    this.width = width;
    this.height = height;
    this.canvas.width = width;
    this.canvas.height = height;
    this.render();
  }

  /**
   * Destroys the editor and cleans up resources.
   */
  destroy(): void {
    // Remove event listeners
    this.canvas.removeEventListener('mousedown', this.handleMouseDown.bind(this));
    this.canvas.removeEventListener('mousemove', this.handleMouseMove.bind(this));
    this.canvas.removeEventListener('mouseup', this.handleMouseUp.bind(this));
    this.canvas.removeEventListener('wheel', this.handleWheel.bind(this));
    this.canvas.removeEventListener('touchstart', this.handleTouchStart.bind(this));
    this.canvas.removeEventListener('touchmove', this.handleTouchMove.bind(this));
    this.canvas.removeEventListener('touchend', this.handleTouchEnd.bind(this));

    this.eventListeners.clear();
  }
}

// === Helper Functions ===

/**
 * Creates a new ArchFlow editor instance.
 *
 * @param options - The editor options
 * @returns A new ArchFlowEditor instance
 */
export function createEditor(options: EditorOptions): ArchFlowEditor {
  return new ArchFlowEditor(options);
}

// === Export utilities ===

export { ArchFlowEditor as default };
