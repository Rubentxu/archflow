/**
 * Main App Component
 *
 * Main application component with full layout including
 * Header, Sidebar, Canvas, PropertiesPanel, and StatusBar.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import React, { useEffect, useState, useCallback, useRef } from "react";
import Header from "./components/Header";
import Toolbar from "./components/Toolbar";
import Sidebar from "./components/Sidebar";
import Canvas from "./components/Canvas";
import { PropertiesPanel } from "./components/Properties/PropertiesPanel";
import ZoomControls from "./components/ZoomControls";

import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useUIStore } from "./store/useUIStore";
import { useArchFlowWasm } from "./hooks/useArchFlowWasm";
import { globalEvents } from "./sdk";

/**
 * Main application component
 */
export default function App() {
  const { isSidebarOpen, activeTool } = useUIStore();
  const { bridge, wasmLoaded, isInitialized } = useArchFlowWasm();

  // Shape creation state
  const [isDrawing, setIsDrawing] = useState(false);
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(
    null,
  );

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

  // Handle pointer down - start drawing shape
  const handlePointerDown = useCallback(
    (pos: { x: number; y: number }, buttons: number) => {
      console.log(
        "Pointer down at",
        pos,
        "buttons",
        buttons,
        "activeTool:",
        activeTool,
      );

      if (
        activeTool === "rectangle" ||
        activeTool === "circle" ||
        activeTool === "triangle" ||
        activeTool === "diamond"
      ) {
        setIsDrawing(true);
        setDragStart(pos);
        console.log("Started drawing", activeTool, "at", pos);
      }
    },
    [activeTool],
  );

  // Handle pointer move - update shape preview (could be implemented in future)
  const handlePointerMove = useCallback(
    (pos: { x: number; y: number }, buttons: number) => {
      if (isDrawing && dragStart) {
        console.log("Drawing", activeTool, "from", dragStart, "to", pos);
      }
    },
    [isDrawing, dragStart, activeTool],
  );

  // Handle pointer up - complete shape creation
  const handlePointerUp = useCallback(
    (pos: { x: number; y: number }, buttons: number) => {
      if (isDrawing && dragStart && bridge && wasmLoaded && isInitialized) {
        const width = Math.abs(pos.x - dragStart.x);
        const height = Math.abs(pos.y - dragStart.y);

        // Only create if size is meaningful
        if (width > 5 && height > 5) {
          console.log("Creating shape:", activeTool, {
            x: Math.min(dragStart.x, pos.x),
            y: Math.min(dragStart.y, pos.y),
            width,
            height,
          });

          try {
            // Call WASM to create the shape
            const typedBridge = bridge as any;
            if (typedBridge.create_rectangle) {
              const entityId = typedBridge.create_rectangle(
                Math.min(dragStart.x, pos.x),
                Math.min(dragStart.y, pos.y),
                width,
                height,
              );
              console.log("Shape created with ID:", entityId);

              // Emit event for EPIC-WEB-013 integration
              globalEvents.emit("entity:created", {
                id: entityId,
                type: activeTool,
                position: {
                  x: Math.min(dragStart.x, pos.x),
                  y: Math.min(dragStart.y, pos.y),
                },
                size: { width, height },
              });
            } else {
              console.warn("create_rectangle method not found on bridge");
            }
          } catch (err) {
            console.error("Failed to create shape:", err);
          }
        }
      }

      setIsDrawing(false);
      setDragStart(null);
    },
    [isDrawing, dragStart, bridge, wasmLoaded, isInitialized, activeTool],
  );

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
            <Canvas
              className="absolute inset-0"
              onPointerDown={handlePointerDown}
              onPointerMove={handlePointerMove}
              onPointerUp={handlePointerUp}
            />

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
