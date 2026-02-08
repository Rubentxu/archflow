/**
 * HistoryBridge - Undo/Redo Operations for ArchFlow
 *
 * This facade organizes history-related methods from WasmBridge by domain.
 * Provides methods for undo/redo operations and history state queries.
 *
 * @example
 * ```typescript
 * const bridge = new ArchFlowBridge(wasmBridge);
 * bridge.history.undo();
 * bridge.history.redo();
 * const canUndo = bridge.history.canUndo();
 * ```
 */

import type { WasmBridge } from './types';

/**
 * History action types
 */
export type ActionType =
  | 'move'
  | 'create'
  | 'delete'
  | 'resize'
  | 'color_change'
  | 'select'
  | 'other';

/**
 * History entry information
 */
export interface HistoryEntry {
  id: number;
  type: ActionType;
  timestamp: number;
  description: string;
}

/**
 * History state snapshot
 */
export interface HistoryState {
  canUndo: boolean;
  canRedo: boolean;
  undoCount: number;
  redoCount: number;
  currentIndex: number;
}

/**
 * Undo/Redo operations
 */
export class HistoryBridge {
  constructor(private bridge: WasmBridge) {}

  // ═══════════════════════════════════════════════════════════════════════════════════════
  // UNDO/REDO
  // ═══════════════════════════════════════════════════════════════════════════════════════

  /**
   * Undo the last action
   */
  undo(): void {
    this.bridge.undo();
  }

  /**
   * Redo the last undone action
   */
  redo(): void {
    this.bridge.redo();
  }

  /**
   * Undo multiple actions at once
   */
  undo(count: number): void {
    for (let i = 0; i < count; i++) {
      if (!this.canUndo()) break;
      this.undo();
    }
  }

  /**
   * Redo multiple actions at once
   */
  redo(count: number): void {
    for (let i = 0; i < count; i++) {
      if (!this.canRedo()) break;
      this.redo();
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════════════════
  // STATE QUERIES
  // ═══════════════════════════════════════════════════════════════════════════════════════

  /**
   * Check if undo is available
   */
  canUndo(): boolean {
    return this.bridge.can_undo();
  }

  /**
   * Check if redo is available
   */
  canRedo(): boolean {
    return this.bridge.can_redo();
  }

  /**
   * Get history state as JSON string
   */
  getState(): string {
    return this.bridge.get_history_state();
  }

  /**
   * Get complete history state
   */
  getFullState(): HistoryState {
    return {
      canUndo: this.canUndo(),
      canRedo: this.canRedo(),
      undoCount: this.getUndoCount(),
      redoCount: this.getRedoCount(),
      currentIndex: this.getCurrentIndex(),
    };
  }

  /**
   * Get count of available undo actions
   */
  getUndoCount(): number {
    // TODO: Implement in WasmBridge if not exists
    console.warn('getUndoCount() - implementation pending in WasmBridge');
    return 0;
  }

  /**
   * Get count of available redo actions
   */
  getRedoCount(): number {
    // TODO: Implement in WasmBridge if not exists
    console.warn('getRedoCount() - implementation pending in WasmBridge');
    return 0;
  }

  /**
   * Get current position in history stack
   */
  getCurrentIndex(): number {
    // TODO: Implement in WasmBridge if not exists
    console.warn('getCurrentIndex() - implementation pending in WasmBridge');
    return 0;
  }

  // ═══════════════════════════════════════════════════════════════════════════════════════
  // HISTORY MANAGEMENT
  // ═══════════════════════════════════════════════════════════════════════════════════════

  /**
   * Clear entire history
   */
  clear(): void {
    // TODO: Implement clear_history in WasmBridge
    console.warn('clear() - implementation pending in WasmBridge');
  }

  /**
   * Get undo stack as array
   */
  getUndoStack(): HistoryEntry[] {
    // TODO: Implement in WasmBridge if not exists
    console.warn('getUndoStack() - implementation pending in WasmBridge');
    return [];
  }

  /**
   * Get redo stack as array
   */
  getRedoStack(): HistoryEntry[] {
    // TODO: Implement in WasmBridge if not exists
    console.warn('getRedoStack() - implementation pending in WasmBridge');
    return [];
  }

  /**
   * Jump to specific history position
   */
  goTo(index: number): void {
    // TODO: Implement go_to in WasmBridge
    console.warn('goTo() - implementation pending in WasmBridge');
  }

  /**
   * Mark current state as saved (for dirty checking)
   */
  markSaved(): void {
    // TODO: Implement mark_saved in WasmBridge
    console.warn('markSaved() - implementation pending in WasmBridge');
  }

  /**
   * Check if document has unsaved changes
   */
  hasUnsavedChanges(): boolean {
    // TODO: Implement has_unsaved_changes in WasmBridge
    console.warn('hasUnsavedChanges() - implementation pending in WasmBridge');
    return false;
  }
}

/**
 * Create a new HistoryBridge instance
 */
export function createHistoryBridge(bridge: any): HistoryBridge {
  return new HistoryBridge(bridge);
}

