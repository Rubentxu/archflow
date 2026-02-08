/**
 * SelectionBridge - Selection Operations for ArchFlow
 *
 * This facade organizes selection-related methods from WasmBridge by domain.
 * Provides methods for single/multi selection, toggle, and clear.
 *
 * @example
 * ```typescript
 * const bridge = new ArchFlowBridge(wasmBridge);
 * bridge.selection.select(entityId);
 * bridge.selection.add(otherEntityId);
 * const selected = bridge.selection.get();
 * bridge.selection.clear();
 * ```
 */

import type { WasmBridge } from './types';

/**
 * Selection mode
 */
export type SelectMode = 'single' | 'multi' | 'toggle';

/**
 * Selection operations
 */
export class SelectionBridge {
  constructor(private bridge: WasmBridge) {}

  // ═══════════════════════════════════════════════════════════════════════════════
  // SELECTION - Single, Multi, Toggle
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Select a single entity (clears previous selection)
   */
  select(entityId: number): void {
    this.bridge.select_entity(entityId);
  }

  /**
   * Add entity to current selection (multi-select)
   */
  add(entityId: number): void {
    // TODO: Implement add_to_selection in WasmBridge
    console.warn('add() - implementation pending in WasmBridge');
  }

  /**
   * Toggle entity selection state
   */
  toggle(entityId: number): void {
    // TODO: Implement toggle_selection in WasmBridge
    console.warn('toggle() - implementation pending in WasmBridge');
  }

  /**
   * Clear all selections
   */
  clear(): void {
    this.bridge.clear_selection();
  }

  /**
   * Select multiple entities at once
   */
  selectMultiple(entityIds: number[]): void {
    // Clear first, then select each
    this.clear();
    for (const id of entityIds) {
      this.bridge.select_entity(id);
    }
  }

  /**
   * Select all entities
   */
  selectAll(): void {
    // TODO: Implement select_all in WasmBridge
    console.warn('selectAll() - implementation pending in WasmBridge');
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // QUERIES
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Get all selected entity IDs
   */
  get(): number[] {
    return this.bridge.get_selection();
  }

  /**
   * Check if an entity is selected
   */
  isSelected(entityId: number): boolean {
    return this.bridge.is_entity_selected(entityId);
  }

  /**
   * Get count of selected entities
   */
  count(): number {
    return this.bridge.get_selection().length;
  }

  /**
   * Check if any entity is selected
   */
  isEmpty(): boolean {
    return this.count() === 0;
  }

  /**
   * Check if selection has changed (for dirty checking)
   */
  isChanged(lastSelection: number[]): boolean {
    const current = this.get();
    if (current.length !== lastSelection.length) return true;
    return !current.every((id, idx) => id === lastSelection[idx]);
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // MODIFICATION
  // ═══════════════════════════════════════════════════════════════════════════════

  /**
   * Set selection directly (replaces current)
   */
  set(entityIds: number[]): void {
    this.clear();
    for (const id of entityIds) {
      this.bridge.select_entity(id);
    }
  }

  /**
   * Set single entity selected state
   */
  setSelected(entityId: number, selected: boolean): void {
    this.bridge.set_entity_selected(entityId, selected);
  }

  /**
   * Remove entity from selection
   */
  remove(entityId: number): void {
    // TODO: Implement remove_from_selection in WasmBridge
    console.warn('remove() - implementation pending in WasmBridge');
  }

  /**
   * Invert current selection
   */
  invert(): void {
    // TODO: Implement invert_selection in WasmBridge
    console.warn('invert() - implementation pending in WasmBridge');
  }
}

/**
 * Create a new SelectionBridge instance
 */
export function createSelectionBridge(bridge: any): SelectionBridge {
  return new SelectionBridge(bridge);
}

