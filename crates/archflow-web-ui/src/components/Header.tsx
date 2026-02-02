/**
 * Header Component - Application Top Bar
 */

import {
  ChevronRight,
  Save,
  Download,
  Upload,
  Settings,
  HelpCircle,
  Menu,
  ZoomIn,
  ZoomOut,
  Maximize,
} from "lucide-react";
import { useUIStore } from "../store/useUIStore";
import { useCanvasStore } from "../store/useCanvasStore";
import { cn } from "../utils/cn";

interface HeaderProps {
  className?: string;
  projectName?: string;
  onSave?: () => void;
  onLoad?: () => void;
  onExport?: () => void;
  onSettings?: () => void;
}

export default function Header({
  className,
  projectName = "Untitled Project",
  onSave,
  onLoad,
  onExport,
  onSettings,
}: HeaderProps) {
  const { toggleSidebar } = useUIStore();
  const { camera, zoomIn, zoomOut } = useCanvasStore();
  const zoom = camera.zoom;

  const formatZoom = (z: number) => `${Math.round(z * 100)}%`;

  return (
    <header
      className={cn(
        "h-12 flex items-center justify-between px-4 border-b border-white/10",
        "bg-surface-dark/95 backdrop-blur-sm",
        className,
      )}
    >
      <div className="flex items-center gap-3">
        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors lg:hidden"
          onClick={toggleSidebar}
        >
          <Menu className="w-5 h-5 text-gray-400" />
        </button>

        <div className="flex items-center gap-2">
          <span className="font-display font-bold text-lg text-primary">
            ArchFlow
          </span>
          <ChevronRight className="w-4 h-4 text-gray-600" />
          <span className="text-sm text-gray-300">{projectName}</span>
        </div>
      </div>

      <div className="hidden lg:flex items-center gap-1">
        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          onClick={() => zoomOut(1.2)}
          title="Zoom out"
        >
          <ZoomOut className="w-4 h-4" />
        </button>

        <button
          className="px-3 py-1 rounded-lg text-sm text-gray-300 hover:bg-white/10 transition-colors min-w-[60px]"
          onClick={() => useCanvasStore.getState().setCamera({ zoom: 1 })}
          title="Reset zoom"
        >
          {formatZoom(zoom)}
        </button>

        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          onClick={() => zoomIn(1.2)}
          title="Zoom in"
        >
          <ZoomIn className="w-4 h-4" />
        </button>

        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white ml-1"
          onClick={() => useCanvasStore.getState().setCamera({ zoom: 1 })}
          title="Fit to screen"
        >
          <Maximize className="w-4 h-4" />
        </button>
      </div>

      <div className="flex items-center gap-1">
        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          onClick={onSave}
          title="Save project"
        >
          <Save className="w-4 h-4" />
        </button>

        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          onClick={onLoad}
          title="Load project"
        >
          <Upload className="w-4 h-4" />
        </button>

        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          onClick={onExport}
          title="Export as image"
        >
          <Download className="w-4 h-4" />
        </button>

        <div className="w-px h-6 bg-white/10 mx-2" />

        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          onClick={onSettings}
          title="Settings"
        >
          <Settings className="w-4 h-4" />
        </button>

        <button
          className="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          title="Help"
        >
          <HelpCircle className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
}
