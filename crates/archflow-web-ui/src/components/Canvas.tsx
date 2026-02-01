import { useEffect, useRef, useState, useCallback } from "react";
import { useArchFlowWasm } from "../hooks/useArchFlowWasm";

interface CanvasProps {
  selectedEntity: number | null;
  onSelectEntity: (id: number | null) => void;
}

interface EntityData {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  label: string;
  shape: number;
  visible: boolean;
  selected: boolean;
}

/**
 * Canvas component that renders entities from WASM engine
 *
 * Architecture:
 * - WASM: Manages all entity state, position, color, shape data
 * - JavaScript: Reads data from WASM and renders using Canvas 2D
 * - Future: WebGPU rendering will be done entirely in WASM
 */
export default function Canvas({
  selectedEntity,
  onSelectEntity,
}: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [entities, setEntities] = useState<EntityData[]>([]);
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Load WASM module
  const { isLoading, error, isReady, initializeEngine } = useArchFlowWasm();

  /**
   * Fetch entity data from WASM engine
   * This is where JavaScript reads all data from Rust WASM
   */
  const fetchEntitiesFromWasm = useCallback((): EntityData[] => {
    if (!window.ArchFlowWasm) {
      return [];
    }

    const wasm = window.ArchFlowWasm;
    const bridge = wasm.WasmBridge;

    try {
      // Get all alive entity indices from WASM
      const aliveEntities = bridge.get_alive_entities();
      const entityData: EntityData[] = [];

      for (const entityId of aliveEntities) {
        const position = bridge.get_entity_position_screen(entityId);
        const size = bridge.get_entity_size_screen(entityId);
        const color = bridge.get_entity_color_hex(entityId);
        const label = bridge.get_entity_label(entityId);
        const shape = bridge.get_entity_shape(entityId);
        const visible = bridge.is_entity_visible(entityId);
        const selected = bridge.is_entity_selected(entityId);

        entityData.push({
          id: entityId,
          x: position[0],
          y: position[1],
          width: size[0],
          height: size[1],
          color,
          label,
          shape,
          visible,
          selected,
        });
      }

      return entityData;
    } catch (err) {
      console.error("Failed to fetch entities from WASM:", err);
      return [];
    }
  }, []);

  /**
   * Render entities using Canvas 2D
   * All data comes from WASM, JavaScript only handles rendering
   */
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !isReady) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const render = () => {
      const rect = canvas.getBoundingClientRect();
      canvas.width = rect.width;
      canvas.height = rect.height;

      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Apply transforms (camera)
      ctx.save();
      ctx.translate(offset.x, offset.y);
      ctx.scale(scale, scale);

      // Render entities - all data from WASM!
      entities.forEach((entity) => {
        if (!entity.visible) return;

        // Shadow
        ctx.shadowColor = "rgba(0, 0, 0, 0.1)";
        ctx.shadowBlur = 10;
        ctx.shadowOffsetY = 4;

        // Background with color from WASM
        ctx.fillStyle = entity.color;
        ctx.beginPath();

        // Render based on shape type from WASM
        const cx = entity.x + entity.width / 2;
        const cy = entity.y + entity.height / 2;
        const rx = entity.width / 2;
        const ry = entity.height / 2;

        switch (entity.shape) {
          case 0: // Rectangle
          case 8: // RoundedRect
            ctx.roundRect(entity.x, entity.y, entity.width, entity.height, 8);
            break;
          case 1: // Circle
            ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
            break;
          case 5: // Diamond
            ctx.moveTo(cx, entity.y);
            ctx.lineTo(entity.x + entity.width, cy);
            ctx.lineTo(cx, entity.y + entity.height);
            ctx.lineTo(entity.x, cy);
            ctx.closePath();
            break;
          case 6: // Cylinder (Database)
            ctx.ellipse(cx, entity.y, rx, ry * 0.3, 0, 0, Math.PI * 2);
            ctx.moveTo(entity.x, entity.y + ry * 0.3);
            ctx.lineTo(entity.x, entity.y + entity.height - ry * 0.3);
            ctx.ellipse(
              cx,
              entity.y + entity.height,
              rx,
              ry * 0.3,
              0,
              0,
              Math.PI * 2,
            );
            ctx.moveTo(
              entity.x + entity.width,
              entity.y + entity.height - ry * 0.3,
            );
            ctx.lineTo(entity.x + entity.width, entity.y + ry * 0.3);
            break;
          default:
            ctx.roundRect(entity.x, entity.y, entity.width, entity.height, 8);
            break;
        }

        ctx.fill();

        // Border (highlight if selected)
        if (entity.selected || entity.id === selectedEntity) {
          ctx.strokeStyle = "#13b6ec";
          ctx.lineWidth = 3;
          ctx.stroke();
        }

        // Label from WASM string pool
        ctx.shadowColor = "transparent";
        ctx.fillStyle = "#0d181b";
        ctx.font = '500 14px "Noto Sans", sans-serif';
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(entity.label || "", cx, cy);
      });

      ctx.restore();
    };

    render();

    const handleResize = () => render();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [entities, offset, scale, selectedEntity, isReady]);

  /**
   * Initialize WASM engine when canvas is ready
   */
  useEffect(() => {
    if (isReady && canvasRef.current) {
      const canvas = canvasRef.current;
      const rect = canvas.getBoundingClientRect();

      // Initialize WASM engine with canvas dimensions
      initializeEngine(rect.width, rect.height)
        .then(() => {
          // Spawn demo entities in WASM
          const wasm = window.ArchFlowWasm;
          if (wasm) {
            const bridge = wasm.WasmBridge;

            // Spawn entities in WASM - WASM manages all state!
            const entity1 = bridge.spawn_entity(100, 100, 120, 80);
            bridge.set_color(entity1, 255, 107, 107, 255); // #FF6B6B
            bridge.set_label(entity1, "AWS EC2");

            const entity2 = bridge.spawn_entity(300, 150, 120, 80);
            bridge.set_color(entity2, 78, 205, 196, 255); // #4ECDC4
            bridge.set_label(entity2, "AWS RDS");

            const entity3 = bridge.spawn_entity(200, 300, 120, 80);
            bridge.set_color(entity3, 69, 183, 209, 255); // #45B7D1
            bridge.set_label(entity3, "AWS S3");

            const entity4 = bridge.spawn_entity(450, 200, 140, 80);
            bridge.set_color(entity4, 255, 160, 122, 255); // #FFA07A
            bridge.set_shape(entity4, 6); // Cylinder (Database shape)
            bridge.set_label(entity4, "Database");

            // Fetch entities from WASM and update state
            const fetchedEntities = fetchEntitiesFromWasm();
            setEntities(fetchedEntities);
          }
        })
        .catch((err) => {
          console.error("Failed to initialize WASM engine:", err);
        });
    }
  }, [isReady, initializeEngine, fetchEntitiesFromWasm]);

  /**
   * Handle pointer down - entity selection
   * Send event to WASM via SharedArrayBuffer
   */
  const handlePointerDown = (e: React.PointerEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const rawX = e.clientX - rect.left;
    const rawY = e.clientY - rect.top;

    // Send input event to WASM via push_input_event
    const wasm = window.ArchFlowWasm;
    if (wasm) {
      const buttons = (e.buttons & 1) | ((e.buttons & 2) << 1);
      const modifiers =
        (e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0);

      try {
        wasm.WasmBridge.push_input_event(0, rawX, rawY, buttons, modifiers); // 0 = Down
      } catch (err) {
        console.error("Failed to push input event to WASM:", err);
      }
    }

    const x = (rawX - offset.x) / scale;
    const y = (rawY - offset.y) / scale;

    // Check if clicked on entity (data from WASM)
    for (const entity of entities) {
      if (
        x >= entity.x &&
        x <= entity.x + entity.width &&
        y >= entity.y &&
        y <= entity.y + entity.height
      ) {
        onSelectEntity(entity.id);
        setIsDragging(true);
        setDragStart({ x: x - entity.x, y: y - entity.y });
        return;
      }
    }

    // Clicked on empty space - deselect
    onSelectEntity(null);
  };

  /**
   * Handle pointer move - drag entity
   * Send event to WASM via SharedArrayBuffer
   */
  const handlePointerMove = (e: React.PointerEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const rawX = e.clientX - rect.left;
    const rawY = e.clientY - rect.top;

    // Send input event to WASM via push_input_event
    const wasm = window.ArchFlowWasm;
    if (wasm) {
      const buttons = (e.buttons & 1) | ((e.buttons & 2) << 1);
      const modifiers =
        (e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0);

      try {
        wasm.WasmBridge.push_input_event(1, rawX, rawY, buttons, modifiers); // 1 = Move
      } catch (err) {
        console.error("Failed to push input event to WASM:", err);
      }
    }

    if (!isDragging || selectedEntity === null) return;

    const x = (rawX - offset.x) / scale;
    const y = (rawY - offset.y) / scale;

    // Send move command to WASM
    if (wasm) {
      const newX = x - dragStart.x;
      const newY = y - dragStart.y;

      // Get current position from WASM
      const currentPos =
        wasm.WasmBridge.get_entity_position_screen(selectedEntity);
      const dx = newX - currentPos[0];
      const dy = newY - currentPos[1];

      // Move entity in WASM - WASM handles the state change!
      wasm.WasmBridge.move_entity(selectedEntity, dx, dy);

      // Fetch updated entities from WASM
      const fetchedEntities = fetchEntitiesFromWasm();
      setEntities(fetchedEntities);
    }
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const rawX = e.clientX - rect.left;
    const rawY = e.clientY - rect.top;

    // Send input event to WASM via push_input_event
    const wasm = window.ArchFlowWasm;
    if (wasm) {
      const buttons = 0;
      const modifiers =
        (e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0);

      try {
        wasm.WasmBridge.push_input_event(2, rawX, rawY, buttons, modifiers); // 2 = Up
      } catch (err) {
        console.error("Failed to push input event to WASM:", err);
      }
    }

    setIsDragging(false);
  };

  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.25, Math.min(4, scale * delta));

    // Update zoom in WASM
    const wasm = window.ArchFlowWasm;
    if (wasm) {
      wasm.WasmBridge.set_zoom(newScale);
    }

    setScale(newScale);
  };

  if (isLoading) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-slate-100 dark:bg-slate-800">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary mb-4"></div>
          <p className="text-slate-600 dark:text-slate-400">
            Loading ArchFlow WASM...
          </p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-slate-100 dark:bg-slate-800">
        <div className="text-center text-red-500">
          <p className="text-lg font-semibold mb-2">Failed to load WASM</p>
          <p className="text-sm">{error.message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="relative w-full h-full">
      {isReady && (
        <div className="absolute top-4 left-4 z-10 px-3 py-1 bg-green-500 text-white text-xs font-medium rounded-full">
          WASM Active • {entities.length} Entities
        </div>
      )}
      <canvas
        ref={canvasRef}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onWheel={handleWheel}
        className="w-full h-full cursor-crosshair"
      />
    </div>
  );
}
