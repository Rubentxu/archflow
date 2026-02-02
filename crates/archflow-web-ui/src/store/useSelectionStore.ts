import { create } from "zustand";

// EntityId será definido en types/wasm.ts
type EntityId = string;

interface SelectionState {
  selectedIds: EntityId[];
  lastSelectedId: EntityId | null;

  select: (id: EntityId, additive?: boolean) => void;
  deselect: (id: EntityId) => void;
  selectMultiple: (ids: EntityId[]) => void;
  clearSelection: () => void;
  isSelected: (id: EntityId) => boolean;
}

export const useSelectionStore = create<SelectionState>((set, get) => ({
  selectedIds: [],
  lastSelectedId: null,

  select: (id, additive = false) => set((state) => ({
    selectedIds: additive ? [...state.selectedIds, id] : [id],
    lastSelectedId: id,
  })),

  deselect: (id) => set((state) => ({
    selectedIds: state.selectedIds.filter((i) => i !== id),
    lastSelectedId: state.lastSelectedId === id ? null : state.lastSelectedId,
  })),

  selectMultiple: (ids) => set({
    selectedIds: ids,
    lastSelectedId: ids.length > 0 ? ids[ids.length - 1] : null,
  }),

  clearSelection: () => set({ selectedIds: [], lastSelectedId: null }),

  isSelected: (id) => get().selectedIds.includes(id),
}));
