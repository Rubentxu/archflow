/**
 * EntityBridge - Entity Operations for ArchFlow
 *
 * This facade organizes entity-related methods from WasmBridge by domain.
 * Provides a clean, navigable API for entity lifecycle and properties.
 *
 * @example
 * ```typescript
 * const bridge = new ArchFlowBridge(wasmBridge);
 * const entityId = bridge.entity.spawn(100, 200, 150, 50);
 * bridge.entity.setPosition(entityId, 300, 400);
 * ```
 */

import type { WasmBridge } from "./types";

/**
 * Create a new EntityBridge instance
 *
 * @param bridge - WASM bridge instance
 * @returns EntityBridge instance
 */
export function createEntityBridge(bridge: any): EntityBridge {
  return new EntityBridge(bridge);
}

/**
 * Entity shape types
 */
export type ShapeType =
  | "rectangle"
  | "circle"
  | "ellipse"
  | "path"
  | "text"
  | "image"
  | "group"
  | "connector";

/**
 * Color representation (RGBA)
 */
export interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

/**
 * Entity lifecycle and property operations
 */
export class EntityBridge {
  constructor(private bridge: WasmBridge) {}

  // ═══════════════════════════════════════════════════════════════════════════════
  // LIFECYCLE - Create, Delete, Duplicate
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Spawn a new entity at the specified position and size
   */
  spawn(x: number, y: number, width: number, height: number): number {
    return this.bridge.spawn_entity(x, y, width, height);
  }

  /**
   * Move an entity by delta
   */
  move(entityId: number, dx: number, dy: number): void {
    return this.bridge.move_entity(entityId, dx, dy);
  }

  /**
   * Delete an entity by ID
   */
  delete(entityId: number): void {
    // TODO: Implement delete_entity in WasmBridge if not exists
    console.warn("delete() - implementation pending in WasmBridge");
  }

  /**
   * Delete all selected entities
   */
  deleteSelected(): void {
    this.bridge.delete_selected();
  }

  /**
   * Duplicate an existing entity
   */
  duplicate(entityId: number): number {
    return this.bridge.duplicate_entity(entityId);
  }

  /**
   * Get total count of alive entities
   */
  count(): number {
    return this.bridge.entity_count();
  }

  /**
   * Clear all entities
   */
  clear(): void {
    this.bridge.clear();
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // PROPERTIES - Position, Size, Color, Shape
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set entity position (world coordinates)
   */
  setPosition(entityId: number, x: number, y: number): void {
    this.bridge.set_position(entityId, x, y);
  }

  /**
   * Get entity position (world coordinates)
   */
  getPosition(entityId: number): [number, number] {
    return this.bridge.get_entity_position_screen(entityId);
  }

  /**
   * Set entity size
   */
  setSize(entityId: number, width: number, height: number): void {
    this.bridge.set_size(entityId, width, height);
  }

  /**
   * Get entity size (screen coordinates)
   */
  getSize(entityId: number): [number, number] {
    return this.bridge.get_entity_size_screen(entityId);
  }

  /**
   * Set fill color (RGBA)
   */
  setColor(
    entityId: number,
    r: number,
    g: number,
    b: number,
    a: number = 255,
  ): void {
    this.bridge.set_color(entityId, r, g, b, a);
  }

  /**
   * Get fill color as hex string
   */
  getColorHex(entityId: number): string {
    return this.bridge.get_entity_color_hex(entityId);
  }

  /**
   * Set entity shape type
   */
  setShape(entityId: number, shape: ShapeType): void {
    const shapeMap: Record<ShapeType, number> = {
      rectangle: 0,
      circle: 1,
      ellipse: 2,
      path: 3,
      text: 4,
      image: 5,
      group: 6,
      connector: 7,
    };
    this.bridge.set_shape(entityId, shapeMap[shape]);
  }

  /**
   * Get entity shape type
   */
  getShape(entityId: number): number {
    return this.bridge.get_entity_shape(entityId);
  }

  /**
   * Set entity label
   */
  setLabel(entityId: number, label: string): void {
    this.bridge.set_label(entityId, label);
  }

  /**
   * Get entity label
   */
  getLabel(entityId: number): string {
    return this.bridge.get_label(entityId);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // VISIBILITY
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set entity visibility
   */
  setVisible(entityId: number, visible: boolean): void {
    this.bridge.set_entity_visible(entityId, visible);
  }

  /**
   * Check if entity is visible
   */
  isVisible(entityId: number): boolean {
    return this.bridge.is_entity_visible(entityId);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // QUERIES
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Get all alive entity IDs
   */
  getAll(): number[] {
    return this.bridge.get_alive_entities();
  }

  /**
   * Check if entity is selected
   */
  isSelected(entityId: number): boolean {
    return this.bridge.is_entity_selected(entityId);
  }

  /**
   * Get entity position in screen coordinates
   */
  getPositionScreen(entityId: number): [number, number] {
    return this.bridge.get_entity_position_screen(entityId);
  }

  /**
   * Get entity size in screen coordinates
   */
  getSizeScreen(entityId: number): [number, number] {
    return this.bridge.get_entity_size_screen(entityId);
  }

  /**
   * Get entity position in world coordinates
   */
  getPositionWorld(entityId: number): [number, number] {
    return this.bridge.get_entity_position_world(entityId);
  }

  /**
   * Get entity size in world coordinates
   */
  getSizeWorld(entityId: number): [number, number] {
    return this.bridge.get_entity_size_world(entityId);
  }
}
