import { useState, useEffect } from "react";

interface ToolbarProps {
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onRun?: () => void;
  activeTool: string;
  onToolChange: (tool: string) => void;
}

/**
 * Toolbar - Connected to WASM ToolManager and History
 *
 * Handles:
 * - Tool selection (synced with WASM tool state)
 * - Zoom controls (synced with WASM camera)
 * - Undo/Redo (synced with WASM history)
 */
export default function Toolbar({
  onZoomIn,
  onZoomOut,
  onRun,
  activeTool,
  onToolChange,
}: ToolbarProps) {
  const [zoom, setZoom] = useState(100);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  const tools = [
    { id: "select", icon: "near_me", label: "Select" },
    { id: "pan", icon: "pan_tool", label: "Pan" },
    { id: "draw", icon: "edit", label: "Draw" },
    { id: "shape", icon: "crop_square", label: "Shape" },
  ];

  /**
   * Sync zoom state with WASM camera
   */
  useEffect(() => {
    if (!window.ArchFlowWasm) return;

    const updateZoom = () => {
      try {
        const wasmZoom = window.ArchFlowWasm.WasmBridge.get_zoom();
        setZoom(Math.round(wasmZoom * 100));
      } catch (err) {
        console.error("Failed to get zoom from WASM:", err);
      }
    };

    updateZoom();
    const interval = setInterval(updateZoom, 100);
    return () => clearInterval(interval);
  }, []);

  /**
   * Sync undo/redo state with WASM history
   */
  useEffect(() => {
    if (!window.ArchFlowWasm) return;

    const updateHistoryState = () => {
      try {
        setCanUndo(window.ArchFlowWasm.WasmBridge.can_undo());
        setCanRedo(window.ArchFlowWasm.WasmBridge.can_redo());
      } catch (err) {
        console.error("Failed to get history state from WASM:", err);
      }
    };

    updateHistoryState();
    const interval = setInterval(updateHistoryState, 200);
    return () => clearInterval(interval);
  }, []);

  /**
   * Handle tool change - sync with WASM
   */
  const handleToolChange = (toolId: string) => {
    if (window.ArchFlowWasm) {
      try {
        window.ArchFlowWasm.WasmBridge.set_tool(toolId);
      } catch (err) {
        console.error("Failed to set tool in WASM:", err);
      }
    }
    onToolChange(toolId);
  };

  const handleZoomIn = () => {
    if (window.ArchFlowWasm) {
      try {
        const currentZoom = window.ArchFlowWasm.WasmBridge.get_zoom();
        const newZoom = Math.min(currentZoom * 1.25, 2.0);
        window.ArchFlowWasm.WasmBridge.set_zoom(newZoom);
        setZoom(Math.round(newZoom * 100));
      } catch (err) {
        console.error("Failed to zoom in WASM:", err);
      }
    }
    onZoomIn?.();
  };

  const handleZoomOut = () => {
    if (window.ArchFlowWasm) {
      try {
        const currentZoom = window.ArchFlowWasm.WasmBridge.get_zoom();
        const newZoom = Math.max(currentZoom * 0.8, 0.25);
        window.ArchFlowWasm.WasmBridge.set_zoom(newZoom);
        setZoom(Math.round(newZoom * 100));
      } catch (err) {
        console.error("Failed to zoom out WASM:", err);
      }
    }
    onZoomOut?.();
  };

  const handleUndo = () => {
    if (window.ArchFlowWasm) {
      try {
        window.ArchFlowWasm.WasmBridge.undo();
      } catch (err) {
        console.error("Failed to undo in WASM:", err);
      }
    }
  };

  const handleRedo = () => {
    if (window.ArchFlowWasm) {
      try {
        window.ArchFlowWasm.WasmBridge.redo();
      } catch (err) {
        console.error("Failed to redo in WASM:", err);
      }
    }
  };

  const handleDelete = () => {
    if (window.ArchFlowWasm) {
      try {
        window.ArchFlowWasm.WasmBridge.delete_selected();
      } catch (err) {
        console.error("Failed to delete selected in WASM:", err);
      }
    }
  };

  return (
    <div className="absolute top-4 left-1/2 -translate-x-1/2 z-10 flex items-center gap-2 bg-surface-light dark:bg-surface-dark rounded-full shadow-lg border border-border-light dark:border-border-dark p-2">
      {/* Tools */}
      <div className="flex items-center gap-1 pr-2 border-r border-border-light dark:border-border-dark">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => handleToolChange(tool.id)}
            className={`p-2 rounded-full transition-colors ${
              activeTool === tool.id
                ? "bg-primary text-white"
                : "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
            }`}
            title={tool.label}
          >
            <span className="material-symbols-outlined text-lg">
              {tool.icon}
            </span>
          </button>
        ))}
      </div>

      {/* Undo/Redo */}
      <div className="flex items-center gap-1 pr-2 border-r border-border-light dark:border-border-dark">
        <button
          onClick={handleUndo}
          disabled={!canUndo}
          className={`p-2 rounded-full transition-colors ${
            canUndo
              ? "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
              : "text-slate-300 dark:text-slate-600 cursor-not-allowed"
          }`}
          title="Undo (Ctrl+Z)"
        >
          <span className="material-symbols-outlined text-lg">undo</span>
        </button>
        <button
          onClick={handleRedo}
          disabled={!canRedo}
          className={`p-2 rounded-full transition-colors ${
            canRedo
              ? "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700"
              : "text-slate-300 dark:text-slate-600 cursor-not-allowed"
          }`}
          title="Redo (Ctrl+Y)"
        >
          <span className="material-symbols-outlined text-lg">redo</span>
        </button>
        <button
          onClick={handleDelete}
          className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-full transition-colors"
          title="Delete Selected (Del)"
        >
          <span className="material-symbols-outlined text-lg">delete</span>
        </button>
      </div>

      {/* Zoom Controls */}
      <div className="flex items-center gap-1">
        <button
          onClick={handleZoomOut}
          className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-full transition-colors"
          title="Zoom Out (Ctrl+-)"
        >
          <span className="material-symbols-outlined text-lg">remove</span>
        </button>
        <span className="text-sm text-slate-600 dark:text-slate-300 font-medium w-12 text-center">
          {zoom}%
        </span>
        <button
          onClick={handleZoomIn}
          className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-full transition-colors"
          title="Zoom In (Ctrl++)"
        >
          <span className="material-symbols-outlined text-lg">add</span>
        </button>
      </div>

      {/* Run Button */}
      <button
        onClick={onRun}
        className="flex items-center gap-1 px-4 py-2 bg-green-500 hover:bg-green-600 text-white rounded-full transition-colors text-sm font-bold shadow-sm"
      >
        <span className="material-symbols-outlined">play_arrow</span>
        <span>SIMULATE</span>
      </button>
    </div>
  );
}
