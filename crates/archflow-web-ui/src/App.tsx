/**
 * Main App Component
 *
 * Main application component with full layout including
 * Header, Sidebar, Canvas, PropertiesPanel, and StatusBar.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useState, useEffect } from "react";
import Header from "./components/Header";
import Toolbar from "./components/Toolbar";
import Sidebar from "./components/Sidebar";
import Canvas from "./components/Canvas";
import { PropertiesPanel } from "./components/Properties";
import StatusBar from "./components/StatusBar";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";

/**
 * Main application component
 */
export default function App() {
  const [isSidebarOpen] = useState(true);

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
    <div className="w-screen h-screen flex flex-col bg-background-dark text-white overflow-hidden">
      {/* Header */}
      <Header
        projectName="AWS Architecture Diagram"
        onSave={() => console.log("Save")}
        onExport={() => console.log("Export")}
      />

      {/* Main content area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <Sidebar isOpen={isSidebarOpen} />

        {/* Center area */}
        <div className="flex-1 flex flex-col relative">
          {/* Toolbar */}
          <Toolbar position="top" />

          {/* Canvas */}
          <div className="flex-1 relative">
            <Canvas
              className="absolute inset-0"
              onPointerDown={(pos, buttons) => {
                console.log("Pointer down at", pos, "buttons", buttons);
              }}
            />

            {/* Floating toolbar on canvas */}
            <Toolbar position="floating" />
          </div>

          {/* Status Bar */}
          <StatusBar />
        </div>

        {/* Properties Panel */}
        <PropertiesPanel />
      </div>
    </div>
  );
}
