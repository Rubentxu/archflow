import { create } from "zustand";

interface CameraState {
  x: number;
  y: number;
  zoom: number;
}

interface CanvasState {
  camera: CameraState;
  showGrid: boolean;
  gridSize: number;
  snapToGrid: boolean;
  snapToEntities: boolean;

  setCamera: (camera: Partial<CameraState>) => void;
  resetCamera: () => void;
  zoomIn: (factor?: number) => void;
  zoomOut: (factor?: number) => void;
  pan: (deltaX: number, deltaY: number) => void;
  setShowGrid: (show: boolean) => void;
  toggleGrid: () => void;
  toggleSnapToGrid: () => void;
  toggleSnapToEntities: () => void;
}

const defaultCamera: CameraState = { x: 0, y: 0, zoom: 1 };

export const useCanvasStore = create<CanvasState>((set) => ({
  camera: defaultCamera,
  showGrid: true,
  gridSize: 20,
  snapToGrid: true,
  snapToEntities: true,

  setCamera: (updates) =>
    set((state) => ({
      camera: { ...state.camera, ...updates },
    })),

  resetCamera: () => set({ camera: defaultCamera }),

  zoomIn: (factor = 1.2) =>
    set((state) => ({
      camera: {
        ...state.camera,
        zoom: Math.min(state.camera.zoom * factor, 10),
      },
    })),

  zoomOut: (factor = 1.2) =>
    set((state) => ({
      camera: {
        ...state.camera,
        zoom: Math.max(state.camera.zoom / factor, 0.1),
      },
    })),

  pan: (deltaX, deltaY) =>
    set((state) => ({
      camera: {
        ...state.camera,
        x: state.camera.x - deltaX / state.camera.zoom,
        y: state.camera.y - deltaY / state.camera.zoom,
      },
    })),

  setShowGrid: (show: boolean) => set({ showGrid: show }),
  toggleGrid: () => set((state) => ({ showGrid: !state.showGrid })),
  toggleSnapToGrid: () => set((state) => ({ snapToGrid: !state.snapToGrid })),
  toggleSnapToEntities: () =>
    set((state) => ({ snapToEntities: !state.snapToEntities })),
}));
