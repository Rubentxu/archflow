import { useState, useEffect } from "react";

interface SidebarProps {
  onEntitySelect?: (id: number) => void;
  selectedEntity?: number | null;
}

interface LayerEntity {
  id: number;
  label: string;
  shape: number;
  visible: boolean;
  selected: boolean;
}

/**
 * Sidebar - Connected to WASM EntityStore
 *
 * Displays:
 * - Layers: All entities from WASM EntityStore
 * - Tree: Hierarchical view (placeholder for future)
 * - Library: Component library (placeholder for future)
 */
export default function Sidebar({
  onEntitySelect,
  selectedEntity,
}: SidebarProps) {
  const [activeTab, setActiveTab] = useState<"layers" | "tree" | "library">(
    "layers",
  );
  const [entities, setEntities] = useState<LayerEntity[]>([]);
  const [searchQuery, setSearchQuery] = useState("");

  /**
   * Fetch entities from WASM EntityStore
   */
  useEffect(() => {
    if (!window.ArchFlowWasm) return;

    const fetchEntities = () => {
      try {
        const bridge = window.ArchFlowWasm.WasmBridge;
        const aliveEntities = bridge.get_alive_entities();

        const entityList: LayerEntity[] = aliveEntities.map((id) => {
          const label = bridge.get_entity_label(id);
          const shape = bridge.get_entity_shape(id);
          const visible = bridge.is_entity_visible(id);
          const selected = bridge.is_entity_selected(id);

          return { id, label, shape, visible, selected };
        });

        setEntities(entityList);
      } catch (err) {
        console.error("Failed to fetch entities from WASM:", err);
      }
    };

    // Initial fetch
    fetchEntities();

    // Poll for changes (in production, use event-based approach)
    const interval = setInterval(fetchEntities, 200);
    return () => clearInterval(interval);
  }, []);

  /**
   * Get icon for entity based on shape type
   */
  const getShapeIcon = (shape: number): string => {
    switch (shape) {
      case 0:
        return "crop_square"; // Rectangle
      case 1:
        return "circle"; // Circle
      case 2:
        return "ellipse"; // Ellipse
      case 5:
        return "change_history"; // Diamond
      case 6:
        return "storage"; // Cylinder (Database)
      case 7:
        return "person"; // Person
      case 8:
        return "crop_square"; // RoundedRect
      default:
        return "crop_square";
    }
  };

  /**
   * Handle entity selection
   */
  const handleEntityClick = (entityId: number) => {
    if (window.ArchFlowWasm) {
      try {
        window.ArchFlowWasm.WasmBridge.select_entity(entityId);
      } catch (err) {
        console.error("Failed to select entity in WASM:", err);
      }
    }
    onEntitySelect?.(entityId);
  };

  /**
   * Toggle entity visibility
   */
  const toggleVisibility = (entityId: number, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!window.ArchFlowWasm) return;

    try {
      const bridge = window.ArchFlowWasm.WasmBridge;
      const currentVisible = bridge.is_entity_visible(entityId);
      bridge.set_entity_visible(entityId, !currentVisible);
    } catch (err) {
      console.error("Failed to toggle visibility in WASM:", err);
    }
  };

  /**
   * Filter entities by search query
   */
  const filteredEntities = entities.filter(
    (entity) =>
      entity.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      entity.id.toString().includes(searchQuery),
  );

  return (
    <aside className="w-64 border-r border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark overflow-y-auto">
      {/* Search */}
      <div className="p-4">
        <div className="relative">
          <span className="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-sm text-slate-400">
            search
          </span>
          <input
            type="text"
            placeholder="Search entities..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-4 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-border-light dark:border-border-dark">
        <button
          onClick={() => setActiveTab("layers")}
          className={`flex-1 flex items-center justify-center gap-1 py-2 text-sm font-medium transition-colors ${
            activeTab === "layers"
              ? "text-primary border-b-2 border-primary"
              : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
          }`}
        >
          <span className="material-symbols-outlined text-lg">layers</span>
          <span className="hidden sm:inline">Layers</span>
          <span className="ml-1 text-xs bg-slate-200 dark:bg-slate-700 px-1.5 py-0.5 rounded-full">
            {entities.length}
          </span>
        </button>
        <button
          onClick={() => setActiveTab("tree")}
          className={`flex-1 flex items-center justify-center gap-1 py-2 text-sm font-medium transition-colors ${
            activeTab === "tree"
              ? "text-primary border-b-2 border-primary"
              : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
          }`}
        >
          <span className="material-symbols-outlined text-lg">
            account_tree
          </span>
          <span className="hidden sm:inline">Tree</span>
        </button>
        <button
          onClick={() => setActiveTab("library")}
          className={`flex-1 flex items-center justify-center gap-1 py-2 text-sm font-medium transition-colors ${
            activeTab === "library"
              ? "text-primary border-b-2 border-primary"
              : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
          }`}
        >
          <span className="material-symbols-outlined text-lg">extension</span>
          <span className="hidden sm:inline">Library</span>
        </button>
      </div>

      {/* Content */}
      <div className="p-2">
        {activeTab === "layers" && (
          <div className="space-y-1">
            {filteredEntities.length === 0 ? (
              <div className="text-center py-8 text-slate-500 dark:text-slate-400 text-sm">
                {searchQuery ? "No entities found" : "No entities yet"}
              </div>
            ) : (
              filteredEntities.map((entity) => (
                <div
                  key={entity.id}
                  onClick={() => handleEntityClick(entity.id)}
                  className={`flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer transition-colors ${
                    entity.id === selectedEntity
                      ? "bg-primary/10 text-primary border border-primary/30"
                      : "hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-700 dark:text-slate-300"
                  } ${!entity.visible ? "opacity-50" : ""}`}
                >
                  {/* Visibility Toggle */}
                  <button
                    onClick={(e) => toggleVisibility(entity.id, e)}
                    className={`p-1 rounded transition-colors ${
                      entity.visible
                        ? "text-slate-500 hover:text-slate-700 dark:hover:text-slate-200"
                        : "text-slate-300 hover:text-slate-500"
                    }`}
                  >
                    <span className="material-symbols-outlined text-sm">
                      {entity.visible ? "visibility" : "visibility_off"}
                    </span>
                  </button>

                  {/* Shape Icon */}
                  <span className="material-symbols-outlined text-sm text-slate-500 dark:text-slate-400">
                    {getShapeIcon(entity.shape)}
                  </span>

                  {/* Entity Label */}
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium truncate">
                      {entity.label || `Entity ${entity.id}`}
                    </div>
                    <div className="text-xs text-slate-500 dark:text-slate-400">
                      ID: {entity.id}
                    </div>
                  </div>

                  {/* Selection Indicator */}
                  {entity.selected && (
                    <span className="material-symbols-outlined text-sm text-primary">
                      check_circle
                    </span>
                  )}
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === "tree" && (
          <div className="text-center py-8 text-slate-500 dark:text-slate-400 text-sm">
            Tree view coming soon
          </div>
        )}

        {activeTab === "library" && (
          <div className="text-center py-8 text-slate-500 dark:text-slate-400 text-sm">
            Library coming soon
          </div>
        )}
      </div>
    </aside>
  );
}
