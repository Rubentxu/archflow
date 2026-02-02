import { create } from "zustand";

// Tool types matching Rust ActuatorType and frontend tools
export type ToolType =
  | "select"
  | "pan"
  | "rectangle"
  | "circle"
  | "triangle"
  | "diamond"
  | "text"
  | "connection"
  | "delete";

interface UIState {
  theme: "light" | "dark";
  isSidebarOpen: boolean;
  isPropertiesPanelOpen: boolean;
  activeTool: ToolType;

  setTheme: (theme: "light" | "dark") => void;
  toggleTheme: () => void;
  toggleSidebar: () => void;
  togglePropertiesPanel: () => void;
  setActiveTool: (tool: ToolType) => void;
}

export const useUIStore = create<UIState>((set) => ({
  theme: "dark",
  isSidebarOpen: true,
  isPropertiesPanelOpen: true,
  activeTool: "select",

  setTheme: (theme) => {
    set({ theme });
    document.documentElement.classList.toggle("dark", theme === "dark");
  },

  toggleTheme: () =>
    set((state) => {
      const newTheme = state.theme === "light" ? "dark" : "light";
      document.documentElement.classList.toggle("dark", newTheme === "dark");
      return { theme: newTheme };
    }),

  toggleSidebar: () =>
    set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),
  togglePropertiesPanel: () =>
    set((state) => ({ isPropertiesPanelOpen: !state.isPropertiesPanelOpen })),
  setActiveTool: (tool) => set({ activeTool: tool }),
}));
