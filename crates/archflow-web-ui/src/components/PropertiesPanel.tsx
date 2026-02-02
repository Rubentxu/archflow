import { X } from "lucide-react";

interface PropertiesPanelProps {
  selectedEntity: number | null;
  onEntityUpdate: () => void;
}

export default function PropertiesPanel({
  selectedEntity,
}: PropertiesPanelProps) {
  if (!selectedEntity) {
    return (
      <aside className="w-72 bg-surface-dark/90 border-l border-border-dark p-4">
        <p className="text-sm text-gray-500 text-center">
          Select an entity to view properties
        </p>
      </aside>
    );
  }

  return (
    <aside className="w-72 bg-surface-dark/90 border-l border-border-dark flex flex-col">
      <div className="p-3 border-b border-border-dark flex items-center justify-between">
        <h3 className="text-sm font-medium text-gray-300">Properties</h3>
        <button className="p-1 hover:bg-white/10 rounded">
          <X className="w-4 h-4 text-gray-400" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-3">
        <div className="space-y-4">
          <div>
            <label className="block text-xs text-gray-500 mb-1">Type</label>
            <input
              type="text"
              value="Entity"
              disabled
              className="w-full px-2 py-1.5 rounded bg-white/5 border border-white/10 text-sm text-gray-400"
            />
          </div>

          <div>
            <label className="block text-xs text-gray-500 mb-1">X</label>
            <input
              type="number"
              defaultValue={0}
              className="w-full px-2 py-1.5 rounded bg-white/5 border border-white/10 text-sm text-gray-200 focus:border-primary"
            />
          </div>

          <div>
            <label className="block text-xs text-gray-500 mb-1">Y</label>
            <input
              type="number"
              defaultValue={0}
              className="w-full px-2 py-1.5 rounded bg-white/5 border border-white/10 text-sm text-gray-200 focus:border-primary"
            />
          </div>

          <div>
            <label className="block text-xs text-gray-500 mb-1">Width</label>
            <input
              type="number"
              defaultValue={100}
              className="w-full px-2 py-1.5 rounded bg-white/5 border border-white/10 text-sm text-gray-200 focus:border-primary"
            />
          </div>

          <div>
            <label className="block text-xs text-gray-500 mb-1">Height</label>
            <input
              type="number"
              defaultValue={60}
              className="w-full px-2 py-1.5 rounded bg-white/5 border border-white/10 text-sm text-gray-200 focus:border-primary"
            />
          </div>
        </div>
      </div>
    </aside>
  );
}
