import {
  MousePointer2,
  Hand,
  Square,
  Circle,
  Type,
  Link,
  Undo2,
  Redo2,
} from "lucide-react";

const tools = [
  { id: "select", icon: MousePointer2, label: "Select" },
  { id: "pan", icon: Hand, label: "Pan" },
  { id: "rectangle", icon: Square, label: "Rectangle" },
  { id: "circle", icon: Circle, label: "Circle" },
  { id: "text", icon: Type, label: "Text" },
  { id: "connection", icon: Link, label: "Connection" },
] as const;

interface ToolbarProps {
  activeTool: string;
  onToolChange: (tool: string) => void;
}

export default function Toolbar({ activeTool }: ToolbarProps) {
  return (
    <div className="flex flex-col gap-1 p-2 bg-surface-dark/90 rounded-lg shadow-lg backdrop-blur-sm">
      {tools.map(({ id, icon: Icon, label }) => (
        <button
          key={id}
          className={`
            p-2 rounded transition-all
            hover:bg-white/10
            ${activeTool === id ? "bg-primary text-white" : "text-gray-400"}
          `}
          title={label}
        >
          <Icon className="w-5 h-5" />
        </button>
      ))}

      <div className="border-t border-white/10 my-1" />

      <button
        disabled
        className="
          p-2 rounded transition-all
          hover:bg-white/10
          text-gray-600 cursor-not-allowed
        "
        title="Undo (Ctrl+Z)"
      >
        <Undo2 className="w-5 h-5" />
      </button>

      <button
        disabled
        className="
          p-2 rounded transition-all
          hover:bg-white/10
          text-gray-600 cursor-not-allowed
        "
        title="Redo (Ctrl+Y)"
      >
        <Redo2 className="w-5 h-5" />
      </button>
    </div>
  );
}
