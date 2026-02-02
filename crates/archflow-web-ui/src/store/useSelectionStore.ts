import { create } from "zustand";

// EntityId as number to match WASM bridge API
export type EntityId = number;

interface SelectionState {
  selectedIds: EntityId[];
  lastSelectedId: EntityId | null;

  // Actions
  setSelectedIds: (ids: EntityId[]) => void;
  addToSelection: (id: EntityId) => void;
  removeFromSelection: (id: EntityId) => void;
  clear: () => void;

  // Queries
  isSelected: (id: EntityId) => boolean;
  getSelectedIds: () => EntityId[];
  getCount: () => number;
}

export const useSelectionStore = create<SelectionState>((set, get) => ({
  selectedIds: [],
  lastSelectedId: null,

  setSelectedIds: (ids: EntityId[]) =>
    set({
      selectedIds: ids,
      lastSelectedId: ids.length > 0 ? ids[ids.length - 1] : null,
    }),

  addToSelection: (id: EntityId) =>
    set((state) => {
      if (state.selectedIds.includes(id)) return state;
      return {
        selectedIds: [...state.selectedIds, id],
        lastSelectedId: id,
      };
    }),

  removeFromSelection: (id: EntityId) =>
    set((state) => ({
      selectedIds: state.selectedIds.filter((i) => i !== id),
      lastSelectedId: state.lastSelectedId === id ? null : state.lastSelectedId,
    })),

  clear: () => set({ selectedIds: [], lastSelectedId: null }),

  isSelected: (id: EntityId) => get().selectedIds.includes(id),

  getSelectedIds: () => get().selectedIds,

  getCount: () => get().selectedIds.length,
}));
