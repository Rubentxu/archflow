import { useState, useEffect } from "react";
import LogicBricksEditor from "./LogicBricksEditor";
import { type EntityLogic } from "../types/logic";

interface PropertiesPanelProps {
  selectedEntity: number | null;
  onEntityUpdate?: () => void;
}

interface EntityProps {
  type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  fillColor: string;
  strokeColor: string;
  borderWidth: number;
}

const entityTypes = [
  { value: "aws-ec2", label: "AWS EC2" },
  { value: "aws-rds", label: "AWS RDS" },
  { value: "aws-s3", label: "AWS S3" },
  { value: "aws-lambda", label: "AWS Lambda" },
  { value: "database", label: "Database" },
  { value: "queue", label: "Queue" },
];

/**
 * Properties Panel - Connected to WASM EntityStore
 *
 * All entity data is read from and written to WASM engine.
 * This component is a thin wrapper that syncs React state with WASM state.
 */
export default function PropertiesPanel({
  selectedEntity,
  onEntityUpdate,
}: PropertiesPanelProps) {
  const [props, setProps] = useState<EntityProps>({
    type: "aws-ec2",
    x: 100,
    y: 100,
    width: 120,
    height: 80,
    fillColor: "#FF6B6B",
    strokeColor: "#13b6ec",
    borderWidth: 2,
  });

  const [entityLogic, setEntityLogic] = useState<EntityLogic>({
    entityId: selectedEntity || 0,
    rules: [],
  });

  /**
   * Fetch entity data from WASM when selection changes
   */
  useEffect(() => {
    if (selectedEntity === null || !window.ArchFlowWasm) {
      return;
    }

    try {
      const wasm = window.ArchFlowWasm;
      const bridge = wasm.WasmBridge;

      // Get position from WASM
      const position = bridge.get_entity_position_screen(selectedEntity);
      const size = bridge.get_entity_size_screen(selectedEntity);
      const color = bridge.get_entity_color_hex(selectedEntity);
      const shape = bridge.get_entity_shape(selectedEntity);

      setProps({
        type: getEntityTypeName(shape),
        x: position[0],
        y: position[1],
        width: size[0],
        height: size[1],
        fillColor: color,
        strokeColor: "#13b6ec",
        borderWidth: 2,
      });
    } catch (err) {
      console.error("Failed to fetch entity data from WASM:", err);
    }
  }, [selectedEntity]);

  const updateProp = <K extends keyof EntityProps>(
    key: K,
    value: EntityProps[K],
  ) => {
    setProps((prev) => ({ ...prev, [key]: value }));

    // Sync with WASM
    if (selectedEntity !== null && window.ArchFlowWasm) {
      const bridge = window.ArchFlowWasm.WasmBridge;

      try {
        switch (key) {
          case "x":
          case "y":
            const currentPos =
              bridge.get_entity_position_screen(selectedEntity);
            const newX = key === "x" ? (value as number) : currentPos[0];
            const newY = key === "y" ? (value as number) : currentPos[1];
            bridge.set_position(selectedEntity, newX, newY);
            break;
          case "width":
          case "height":
            const currentSize = bridge.get_entity_size_screen(selectedEntity);
            const newWidth =
              key === "width" ? (value as number) : currentSize[0];
            const newHeight =
              key === "height" ? (value as number) : currentSize[1];
            bridge.set_size(selectedEntity, newWidth, newHeight);
            break;
          case "fillColor":
            const hex = value as string;
            const r = parseInt(hex.slice(1, 3), 16);
            const g = parseInt(hex.slice(3, 5), 16);
            const b = parseInt(hex.slice(5, 7), 16);
            bridge.set_color(selectedEntity, r, g, b, 255);
            break;
        }
        onEntityUpdate?.();
      } catch (err) {
        console.error("Failed to update entity in WASM:", err);
      }
    }
  };

  if (selectedEntity === null) {
    return (
      <aside className="w-72 border-l border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark overflow-y-auto">
        <div className="p-6 text-center text-slate-500 dark:text-slate-400">
          <span className="material-symbols-outlined text-4xl mb-2">
            select
          </span>
          <p className="text-sm">Select an entity to view its properties</p>
        </div>
      </aside>
    );
  }

  return (
    <aside className="w-72 border-l border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark overflow-y-auto">
      <div className="p-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-slate-900 dark:text-white">
            Properties
          </h3>
          <div className="text-xs text-slate-500 dark:text-slate-400">
            ID: {selectedEntity}
          </div>
        </div>

        {/* Entity Type */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
            Type
          </label>
          <select
            value={props.type}
            onChange={(e) => updateProp("type", e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          >
            {entityTypes.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
        </div>

        {/* Position */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
            Position
          </label>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">
                X
              </label>
              <input
                type="number"
                value={Math.round(props.x)}
                onChange={(e) => updateProp("x", Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">
                Y
              </label>
              <input
                type="number"
                value={Math.round(props.y)}
                onChange={(e) => updateProp("y", Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>
        </div>

        {/* Size */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
            Size
          </label>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">
                Width
              </label>
              <input
                type="number"
                value={Math.round(props.width)}
                onChange={(e) => updateProp("width", Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">
                Height
              </label>
              <input
                type="number"
                value={Math.round(props.height)}
                onChange={(e) => updateProp("height", Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>
        </div>

        {/* Style */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
            Style
          </label>
          <div className="space-y-2">
            <div>
              <label className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
                <input
                  type="color"
                  value={props.fillColor}
                  onChange={(e) => updateProp("fillColor", e.target.value)}
                  className="w-8 h-8 rounded cursor-pointer"
                />
                Fill
              </label>
            </div>
            <div>
              <label className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
                <input
                  type="color"
                  value={props.strokeColor}
                  onChange={(e) => updateProp("strokeColor", e.target.value)}
                  className="w-8 h-8 rounded cursor-pointer"
                />
                Stroke
              </label>
            </div>
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">
                Border Width
              </label>
              <input
                type="number"
                min="0"
                max="10"
                value={props.borderWidth}
                onChange={(e) =>
                  updateProp("borderWidth", Number(e.target.value))
                }
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>
        </div>

        {/* Logic Bricks Section */}
        <div className="border-t border-border-light dark:border-border-dark pt-4">
          <LogicBricksEditor
            entityId={selectedEntity}
            logic={entityLogic}
            onLogicChange={setEntityLogic}
          />
        </div>
      </div>
    </aside>
  );
}

/**
 * Map shape type to entity type name
 */
function getEntityTypeName(shape: number): string {
  switch (shape) {
    case 0:
      return "aws-ec2"; // Rectangle
    case 6:
      return "database"; // Cylinder
    case 1:
      return "aws-s3"; // Circle
    case 5:
      return "queue"; // Diamond
    default:
      return "aws-ec2";
  }
}
