import { useUIStore } from "../store/useUIStore";

interface HeaderProps {
  darkMode: boolean;
  onToggleDarkMode: () => void;
}

export default function Header({ darkMode }: HeaderProps) {
  const { activeTool } = useUIStore();

  return (
    <header
      className={`
      h-14 flex items-center justify-between px-4 border-b
      ${darkMode ? "bg-surface-dark border-border-dark" : "bg-white border-gray-200"}
    `}
    >
      <div className="flex items-center gap-2">
        <span className="font-display font-bold text-lg text-primary">
          ArchFlow
        </span>
        <span className="text-gray-500">/</span>
        <span className="text-sm text-gray-400">Whiteboard</span>
      </div>

      <div className="flex items-center gap-2">
        <span className="text-xs text-gray-500">Tool: {activeTool}</span>
      </div>
    </header>
  );
}
