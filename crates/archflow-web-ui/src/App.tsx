/**
 * Main App Component
 *
 * Main application component with full layout including
 * Header, Sidebar, Canvas, PropertiesPanel, and StatusBar.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import React, { useEffect } from "react";
import Header from "./components/Header";
import Toolbar from "./components/Toolbar";
import Sidebar from "./components/Sidebar";
import Canvas from "./components/Canvas";
import { PropertiesPanel } from "./components/Properties/PropertiesPanel";
import ZoomControls from "./components/ZoomControls";

import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useUIStore } from "./store/useUIStore";
import { useSelectionSync } from "./hooks/useSelectionSync";

/**
 * Main application component
 */
export default function App() {
  const { isSidebarOpen } = useUIStore();

  // Start synchronization between WASM bridge selection and React UI state
  useSelectionSync();

  // Initialize keyboard shortcuts
  const { shortcuts } = useKeyboardShortcuts();

  // Log shortcuts info in development
  useEffect(() => {
    if (import.meta.env.DEV) {
      console.log(
        "Keyboard shortcuts available:",
        shortcuts.map(
          (s) =>
            `${s.ctrl ? "Ctrl+" : ""}${s.shift ? "Shift+" : ""}${s.key}: ${s.description}`,
        ),
      );
    }
  }, [shortcuts]);

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden bg-background-light dark:bg-background-dark font-sans text-slate-900 dark:text-gray-100 transition-colors duration-300">
      <Header />

      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <Sidebar isOpen={isSidebarOpen} />

        {/* Center area */}
        <div className="flex-1 flex flex-col relative bg-white dark:bg-black/20">
          {/* Canvas */}
          <div className="flex-1 relative overflow-hidden">
            {/* Background Grid handled by Canvas component but ensuring container structure */}
            <Canvas className="absolute inset-0" />

            {/* Floating toolbar on canvas - Centered Top */}
            <Toolbar
              position="floating"
              className="absolute top-4 left-1/2 -translate-x-1/2"
            />

            {/* Zoom Controls - Bottom Left */}
            <ZoomControls className="absolute bottom-4 left-4" />
          </div>
        </div>

        {/* Properties Panel */}
        <PropertiesPanel />
      </div>
    </div>
  );
}
