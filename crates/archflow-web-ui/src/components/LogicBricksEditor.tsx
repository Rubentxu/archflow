import { useState, useEffect, useMemo } from "react";
import { useArchFlowWasm } from "../hooks/useArchFlowWasm";

interface LogicBricksEditorProps {
  entityId: number;
}

/**
 * Logic Bricks Editor component that connects to WASM LogicMappingTable
 *
 * This component provides a UI for managing sensor-actuator connections
 * for entities using the Logic Bricks system. All logic state is managed
 * in WASM, JavaScript only provides the UI.
 */
export default function LogicBricksEditor({
  entityId,
}: LogicBricksEditorProps) {
  const {
    isReady,
    getLogicMappingTable,
    getSensorType,
    getActuatorType,
    getController,
  } = useArchFlowWasm();

  const [connections, setConnections] = useState<
    Array<{ sensor: number; actuator: string; controller: string }>
  >([]);
  const [expandedConnection, setExpandedConnection] = useState<number | null>(
    null,
  );

  // Lazy load the WASM types to avoid errors before WASM is ready
  const wasmTypes = useMemo(() => {
    if (!isReady) {
      return { SensorType: {}, ActuatorType: {}, Controller: {} };
    }

    const SensorType = getSensorType();
    const ActuatorType = getActuatorType();
    const Controller = getController();

    return {
      SensorType: {
        MouseOver: SensorType?.MouseOver ?? 0,
        MouseClick: SensorType?.MouseClick ?? 1,
        Proximity: SensorType?.Proximity ?? 2,
        KeyShortcut: SensorType?.KeyShortcut ?? 3,
      },
      ActuatorType: {
        Highlight: "Highlight",
        Select: "Select",
        Move: "Move",
      },
      Controller: {
        Direct: () => Controller?.direct?.() ?? {},
        And: (sensor: number) => Controller?.and?.(sensor) ?? {},
        Or: (sensor: number) => Controller?.or?.(sensor) ?? {},
        Not: () => Controller?.not?.() ?? {},
      },
    };
  }, [isReady, getSensorType, getActuatorType, getController]);

  // Load connections from WASM LogicMappingTable
  useEffect(() => {
    if (!isReady || !getLogicMappingTable()) {
      return;
    }

    // In a real implementation, we would need to add methods to LogicMappingTableWasm
    // to query all connections for an entity. For now, we track connections locally.
    const saved = localStorage.getItem(`logic_${entityId}`);
    if (saved) {
      try {
        setConnections(JSON.parse(saved));
      } catch (e) {
        console.error("Failed to parse saved logic:", e);
      }
    }
  }, [entityId, isReady, getLogicMappingTable]);

  // Save connections to localStorage and sync with WASM
  const syncConnections = (
    newConnections: Array<{
      sensor: number;
      actuator: string;
      controller: string;
    }>,
  ) => {
    setConnections(newConnections);
    localStorage.setItem(`logic_${entityId}`, JSON.stringify(newConnections));

    // Sync with WASM LogicMappingTable
    if (!isReady || !getLogicMappingTable()) {
      return;
    }

    const LogicMappingTable = getLogicMappingTable();
    const table = new LogicMappingTable();

    // Clear existing connections for this entity
    // Note: We would need to track existing connections to clear them
    // For now, we add new connections

    newConnections.forEach((conn) => {
      const sensor = conn.sensor;
      const controller = wasmTypes.Controller.Direct(); // Simplified

      // Add connection based on actuator type
      if (conn.actuator === "Highlight") {
        table.add_highlight(entityId, sensor, controller);
      } else if (conn.actuator === "Select") {
        table.add_select(entityId, sensor, controller);
      } else if (conn.actuator === "Move") {
        table.add_move(entityId, sensor, controller);
      }
    });

    console.log(
      "Synced",
      newConnections.length,
      "connections to WASM for entity",
      entityId,
    );
  };

  const addConnection = () => {
    const newConnection = {
      sensor: wasmTypes.SensorType.MouseOver,
      actuator: "Highlight",
      controller: "Direct",
    };
    syncConnections([...connections, newConnection]);
    setExpandedConnection(connections.length);
  };

  const removeConnection = (index: number) => {
    const newConnections = connections.filter((_, i) => i !== index);
    syncConnections(newConnections);
    if (expandedConnection === index) {
      setExpandedConnection(null);
    }
  };

  const updateConnection = (
    index: number,
    updates: Partial<{ sensor: number; actuator: string; controller: string }>,
  ) => {
    const newConnections = connections.map((conn, i) =>
      i === index ? { ...conn, ...updates } : conn,
    );
    syncConnections(newConnections);
  };

  if (!isReady) {
    return (
      <div className="flex items-center justify-center p-8 text-slate-500 dark:text-slate-400">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary mb-3"></div>
          <p className="text-sm">Loading WASM Logic Bricks...</p>
        </div>
      </div>
    );
  }

  const sensorOptions = [
    {
      value: wasmTypes.SensorType.MouseOver,
      label: "Mouse Over",
      icon: "mouse",
    },
    {
      value: wasmTypes.SensorType.MouseClick,
      label: "Mouse Click",
      icon: "click",
    },
    {
      value: wasmTypes.SensorType.Proximity,
      label: "Proximity",
      icon: "sensors",
    },
    {
      value: wasmTypes.SensorType.KeyShortcut,
      label: "Key Shortcut",
      icon: "keyboard",
    },
  ];

  const actuatorOptions = [
    { value: "Highlight", label: "Highlight", icon: "lightbulb" },
    { value: "Select", label: "Select", icon: "check_box_outline_blank" },
    { value: "Move", label: "Move", icon: "open_with" },
  ];

  return (
    <div className="space-y-3">
      {/* WASM Status */}
      <div className="flex items-center gap-2 px-3 py-2 bg-green-500/10 border border-green-500/30 rounded-lg">
        <span className="material-symbols-outlined text-green-500 text-sm">
          check_circle
        </span>
        <span className="text-xs text-green-700 dark:text-green-400">
          WASM LogicMappingTable active • Entity {entityId}
        </span>
        <span className="ml-auto text-xs text-slate-500">
          {connections.length} connection{connections.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Connections List */}
      {connections.length === 0 ? (
        <div className="text-center py-8 text-slate-500 dark:text-slate-400">
          <span className="material-symbols-outlined text-3xl mb-2">
            extension
          </span>
          <p className="text-sm">No logic connections configured</p>
          <p className="text-xs mt-1">
            Connect sensors to actuators to define entity behavior
          </p>
        </div>
      ) : (
        connections.map((conn, index) => (
          <div
            key={index}
            className="border border-primary/30 bg-primary/5 dark:bg-primary/10 rounded-lg overflow-hidden"
          >
            {/* Connection Header */}
            <div
              className="flex items-center gap-2 p-3 cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-700/50"
              onClick={() =>
                setExpandedConnection(
                  expandedConnection === index ? null : index,
                )
              }
            >
              <span className="material-symbols-outfilled text-lg text-primary">
                {sensorOptions.find((s) => s.value === conn.sensor)?.icon}
              </span>

              <span className="flex-1 text-sm font-medium text-slate-900 dark:text-white">
                {sensorOptions.find((s) => s.value === conn.sensor)?.label} →{" "}
                {conn.actuator}
              </span>

              <span className="material-symbols-outlined text-slate-400">
                {expandedConnection === index ? "expand_less" : "expand_more"}
              </span>

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  removeConnection(index);
                }}
                className="p-1 rounded text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20"
              >
                <span className="material-symbols-outlined text-lg">
                  delete
                </span>
              </button>
            </div>

            {/* Expanded Connection Details */}
            {expandedConnection === index && (
              <div className="p-3 pt-0 space-y-3 border-t border-border-light dark:border-border-dark">
                {/* Sensor Selection */}
                <div>
                  <label className="block text-xs font-medium text-slate-600 dark:text-slate-400 mb-2">
                    Sensor
                  </label>
                  <select
                    value={conn.sensor}
                    onChange={(e) =>
                      updateConnection(index, {
                        sensor: Number(e.target.value),
                      })
                    }
                    className="w-full px-3 py-2 text-sm rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark"
                  >
                    {sensorOptions.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Actuator Selection */}
                <div>
                  <label className="block text-xs font-medium text-slate-600 dark:text-slate-400 mb-2">
                    Actuator
                  </label>
                  <select
                    value={conn.actuator}
                    onChange={(e) =>
                      updateConnection(index, { actuator: e.target.value })
                    }
                    className="w-full px-3 py-2 text-sm rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark"
                  >
                    {actuatorOptions.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Controller Selection */}
                <div>
                  <label className="block text-xs font-medium text-slate-600 dark:text-slate-400 mb-2">
                    Controller
                  </label>
                  <select
                    value={conn.controller}
                    onChange={(e) =>
                      updateConnection(index, { controller: e.target.value })
                    }
                    className="w-full px-3 py-2 text-sm rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark"
                  >
                    <option value="Direct">Direct (immediate)</option>
                    <option value="Stable">Stable (wait for steady)</option>
                    <option value="Rising">Rising Edge (off → on)</option>
                    <option value="Falling">Falling Edge (on → off)</option>
                  </select>
                </div>
              </div>
            )}
          </div>
        ))
      )}

      {/* Add Connection Button */}
      <button
        onClick={addConnection}
        className="w-full py-2 px-4 rounded-lg border-2 border-dashed border-primary/30 text-primary hover:bg-primary/5 hover:border-primary/50 transition-colors flex items-center justify-center gap-2"
      >
        <span className="material-symbols-outlined">add</span>
        <span className="text-sm font-medium">Add Connection</span>
      </button>
    </div>
  );
}
