/**
 * Toolbar Component - Editor Tools Palette
 *
 * Floating toolbar with tool selection and common actions.
 */

import {
  MousePointer2,
  Hand,
  Square,
  Circle,
  Type,
  Link,
  PlayCircle
} from "lucide-react";
import { useUIStore } from "../store/useUIStore";
import type { ToolType } from "../store/useUIStore";
import { cn } from "../utils/cn";

interface Tool {
  id: ToolType;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  shortcut: string;
}

const tools: Tool[] = [
  { id: "select", icon: MousePointer2, label: "Select", shortcut: "V" },
  { id: "pan", icon: Hand, label: "Pan", shortcut: "H" },
  { id: "rectangle", icon: Square, label: "Rectangle", shortcut: "R" },
  { id: "circle", icon: Circle, label: "Circle", shortcut: "C" },
  { id: "text", icon: Type, label: "Text", shortcut: "X" },
  { id: "connection", icon: Link, label: "Connection", shortcut: "L" },
];

interface ToolbarProps {
  className?: string;
  position?: "left" | "top" | "floating"; // Kept for interface compatibility but unused log removed
}

export default function Toolbar({
  className,
}: ToolbarProps) {
  const { activeTool, setActiveTool } = useUIStore();

  return (
    <div
      className={cn(
        "flex items-center gap-1 p-1 bg-white dark:bg-surface-dark rounded-full shadow-lg border border-border-light dark:border-border-dark",
        "z-40",
        className,
      )}
    >
      {tools.map(({ id, icon: Icon, label, shortcut }) => (
        <button
          key={id}
          className={cn(
            "p-2 rounded-full transition-colors tooltip",
            activeTool === id
              ? "bg-primary/10 text-primary dark:text-primary"
              : "text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800",
          )}
          title={`${label} (${shortcut})`}
          onClick={() => setActiveTool(id)}
        >
          <Icon className="w-5 h-5" />
        </button>
      ))}

      <div className="w-px h-6 bg-slate-200 dark:bg-slate-700 mx-1"></div>

      <button
        className="p-2 pr-3 pl-3 rounded-full bg-slate-50 dark:bg-slate-800 hover:bg-green-50 dark:hover:bg-green-900/30 text-green-600 dark:text-green-400 transition-colors flex items-center gap-1.5"
        title="Simulate"
      >
        <PlayCircle className="w-5 h-5" />
        <span className="text-xs font-bold uppercase tracking-wide">Simulate</span>
      </button>
    </div>
  );
}
