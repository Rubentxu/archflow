/**
 * ZoomControls Component
 *
 * Controls for zooming in/out and fitting content to screen.
 * Positioned in bottom-left corner of the canvas.
 */

import { Minus, Plus, Maximize } from "lucide-react";
import { useCanvasStore } from "../store/useCanvasStore";
import { cn } from "../utils/cn";

interface ZoomControlsProps {
    className?: string;
}

export default function ZoomControls({ className }: ZoomControlsProps) {
    const { zoomIn, zoomOut, resetCamera, camera } = useCanvasStore();

    return (
        <div
            className={cn(
                "flex items-center gap-1 p-1 bg-white dark:bg-surface-dark rounded-lg shadow-lg border border-border-light dark:border-border-dark",
                className,
            )}
        >
            <button
                className="p-1.5 rounded-md text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                onClick={() => zoomOut()}
                title="Zoom Out (-)"
            >
                <Minus className="w-4 h-4" />
            </button>

            <span className="min-w-[3rem] text-center text-xs font-mono font-medium text-slate-600 dark:text-slate-300 select-none">
                {Math.round(camera.zoom * 100)}%
            </span>

            <button
                className="p-1.5 rounded-md text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                onClick={() => zoomIn()}
                title="Zoom In (+)"
            >
                <Plus className="w-4 h-4" />
            </button>

            <div className="w-px h-4 bg-slate-200 dark:bg-slate-700 mx-0.5"></div>

            <button
                className="p-1.5 rounded-md text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                onClick={resetCamera}
                title="Fit to Screen (Shift+1)"
            >
                <Maximize className="w-4 h-4" />
            </button>
        </div>
    );
}
