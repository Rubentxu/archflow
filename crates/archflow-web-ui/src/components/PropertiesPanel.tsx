/**
 * PropertiesPanel Component - Entity Property Editor
 */

import { useState, useEffect } from "react";
import { Settings2, Layers } from "lucide-react";
import { cn } from "../utils/cn";
import { useSelectionStore } from "../store/useSelectionStore";
import { useEntityStore } from "../hooks/useEntityStore";

interface PropertiesPanelProps {
  className?: string;
  isOpen?: boolean;
}

export default function PropertiesPanel({
  className,
  isOpen = true,
}: PropertiesPanelProps) {
  const { selectedIds } = useSelectionStore();
  const { getEntity, updateEntity } = useEntityStore();
  const [localValues, setLocalValues] = useState<Record<string, unknown>>({});
  const [hasChanges, setHasChanges] = useState(false);

  const selectedEntity =
    selectedIds.length === 1 ? getEntity(selectedIds[0]) : null;

  useEffect(() => {
    if (selectedEntity) {
      setLocalValues({
        label: selectedEntity.label || "",
        x: selectedEntity.position.x,
        y: selectedEntity.position.y,
        width: selectedEntity.size.w,
        height: selectedEntity.size.h,
      });
      setHasChanges(false);
    } else {
      setLocalValues({});
      setHasChanges(false);
    }
  }, [selectedEntity]);

  const handleValueChange = (key: string, value: unknown) => {
    setLocalValues((prev) => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const handleApply = () => {
    if (!selectedEntity) return;
    updateEntity(selectedEntity.id, { label: localValues.label as string });
    setHasChanges(false);
  };

  if (!isOpen) return null;

  return (
    <aside
      className={cn(
        "w-72 h-full flex flex-col bg-surface-dark/95 border-l border-white/5",
        className,
      )}
    >
      <div className="flex items-center justify-between p-3 border-b border-white/5">
        <div className="flex items-center gap-2">
          <Settings2 className="w-4 h-4 text-primary" />
          <span className="font-medium text-sm text-gray-200">Properties</span>
        </div>
        {selectedIds.length > 0 && (
          <span className="text-xs text-gray-500">
            {selectedIds.length === 1
              ? "1 entity"
              : `${selectedIds.length} entities`}
          </span>
        )}
      </div>

      <div className="flex-1 overflow-y-auto">
        {!selectedEntity ? (
          <div className="flex flex-col items-center justify-center h-full p-4 text-center">
            <Layers className="w-12 h-12 text-gray-600 mb-3" />
            <p className="text-sm text-gray-500">Select an entity to edit</p>
          </div>
        ) : (
          <div className="p-2">
            <div className="mb-4 px-3 py-2 bg-white/5 rounded-lg">
              <span className="text-xs text-gray-500 uppercase tracking-wider">
                Type
              </span>
              <p className="text-sm text-primary font-medium mt-1">Entity</p>
            </div>

            <div className="space-y-3">
              <div>
                <label className="block text-xs text-gray-500 mb-1 ml-1">
                  Label
                </label>
                <input
                  type="text"
                  value={(localValues.label as string) || ""}
                  onChange={(e) => handleValueChange("label", e.target.value)}
                  className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 focus:outline-none focus:border-primary/50"
                />
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-xs text-gray-500 mb-1 ml-1">
                    X
                  </label>
                  <input
                    type="number"
                    value={(localValues.x as number) || 0}
                    onChange={(e) =>
                      handleValueChange("x", parseFloat(e.target.value) || 0)
                    }
                    className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 focus:outline-none focus:border-primary/50"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-500 mb-1 ml-1">
                    Y
                  </label>
                  <input
                    type="number"
                    value={(localValues.y as number) || 0}
                    onChange={(e) =>
                      handleValueChange("y", parseFloat(e.target.value) || 0)
                    }
                    className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 focus:outline-none focus:border-primary/50"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="block text-xs text-gray-500 mb-1 ml-1">
                    Width
                  </label>
                  <input
                    type="number"
                    value={(localValues.width as number) || 100}
                    onChange={(e) =>
                      handleValueChange(
                        "width",
                        parseFloat(e.target.value) || 100,
                      )
                    }
                    className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 focus:outline-none focus:border-primary/50"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-500 mb-1 ml-1">
                    Height
                  </label>
                  <input
                    type="number"
                    value={(localValues.height as number) || 60}
                    onChange={(e) =>
                      handleValueChange(
                        "height",
                        parseFloat(e.target.value) || 60,
                      )
                    }
                    className="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 focus:outline-none focus:border-primary/50"
                  />
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {selectedEntity && (
        <div className="p-3 border-t border-white/5">
          <button
            className={cn(
              "w-full px-3 py-2 rounded-lg text-sm font-medium transition-colors",
              hasChanges
                ? "bg-primary text-white hover:bg-primary/90"
                : "bg-white/5 text-gray-400 cursor-not-allowed",
            )}
            onClick={handleApply}
            disabled={!hasChanges}
          >
            Apply Changes
          </button>
        </div>
      )}
    </aside>
  );
}
