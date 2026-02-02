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
  Triangle,
  Diamond,
  Type,
  Link,
  Trash2,
  Undo2,
  Redo2,
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
  { id: "triangle", icon: Triangle, label: "Triangle", shortcut: "T" },
  { id: "diamond", icon: Diamond, label: "Diamond", shortcut: "D" },
  { id: "text", icon: Type, label: "Text", shortcut: "X" },
  { id: "connection", icon: Link, label: "Connection", shortcut: "L" },
];

interface ToolbarProps {
  className?: string;
  position?: "left" | "top" | "floating";
}

export default function Toolbar({
  className,
  position = "floating",
}: ToolbarProps) {
  const { activeTool, setActiveTool } = useUIStore();

  const positionClasses = {
    left: "left-4 top-1/2 -translate-y-1/2 flex-col",
    top: "top-4 left-1/2 -translate-x-1/2 flex-row",
    floating: "left-4 top-4 flex-col",
  };

  const handleDelete = () => {
    // Delete logic handled elsewhere
  };

  return (
    <div
      className={cn(
        "flex gap-1 p-2 bg-surface-dark/95 rounded-lg shadow-xl backdrop-blur-sm border border-white/5",
        positionClasses[position],
        className,
      )}
    >
      {tools.map(({ id, icon: Icon, label, shortcut }) => (
        <button
          key={id}
          className={cn(
            "group relative p-2.5 rounded-lg transition-all hover:bg-white/10 active:scale-95",
            activeTool === id
              ? "bg-primary text-white shadow-lg shadow-primary/20"
              : "text-gray-400 hover:text-white",
          )}
          title={`${label} (${shortcut})`}
          onClick={() => setActiveTool(id)}
        >
          <Icon className="w-5 h-5" />
        </button>
      ))}

      <div
        className={cn(
          "border-white/10",
          position === "left" || position === "floating"
            ? "border-t my-1"
            : "border-r mx-1",
        )}
      />

      <button
        className={cn(
          "p-2.5 rounded-lg transition-all hover:bg-red-500/20 hover:text-red-400 active:scale-95",
          activeTool === "delete" && "bg-red-500 text-white",
        )}
        title="Delete (Del)"
        onClick={() => {
          setActiveTool("delete");
          handleDelete();
        }}
      >
        <Trash2 className="w-5 h-5" />
      </button>

      <div
        className={cn(
          "border-white/10",
          position === "left" || position === "floating"
            ? "border-t my-1"
            : "border-r mx-1",
        )}
      />

      <button
        className="p-2.5 rounded-lg text-gray-600 cursor-not-allowed"
        title="Undo (Ctrl+Z)"
      >
        <Undo2 className="w-5 h-5" />
      </button>
      <button
        className="p-2.5 rounded-lg text-gray-600 cursor-not-allowed"
        title="Redo (Ctrl+Y)"
      >
        <Redo2 className="w-5 h-5" />
      </button>
    </div>
  );
}
