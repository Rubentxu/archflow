import { useEffect, useRef, useState } from "react";

interface CanvasProps {
  selectedEntity: number | null;
  onSelectEntity: (id: number | null) => void;
}

interface Entity {
  id: number;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  label: string;
}

// Demo entities - will be replaced with WASM integration
const demoEntities: Entity[] = [
  {
    id: 1,
    x: 100,
    y: 100,
    width: 120,
    height: 80,
    color: "#FF6B6B",
    label: "AWS EC2",
  },
  {
    id: 2,
    x: 300,
    y: 150,
    width: 120,
    height: 80,
    color: "#4ECDC4",
    label: "AWS RDS",
  },
  {
    id: 3,
    x: 200,
    y: 300,
    width: 120,
    height: 80,
    color: "#45B7D1",
    label: "AWS S3",
  },
  {
    id: 4,
    x: 450,
    y: 200,
    width: 140,
    height: 80,
    color: "#FFA07A",
    label: "Database",
  },
];

export default function Canvas({
  selectedEntity,
  onSelectEntity,
}: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [entities, setEntities] = useState<Entity[]>(demoEntities);
  const [scale, setScale] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [offset, _setOffset] = useState({ x: 0, y: 0 });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const render = () => {
      const rect = canvas.getBoundingClientRect();
      canvas.width = rect.width;
      canvas.height = rect.height;

      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Apply transforms
      ctx.save();
      ctx.translate(offset.x, offset.y);
      ctx.scale(scale, scale);

      // Render entities
      entities.forEach((entity) => {
        // Shadow
        ctx.shadowColor = "rgba(0, 0, 0, 0.1)";
        ctx.shadowBlur = 10;
        ctx.shadowOffsetY = 4;

        // Background
        ctx.fillStyle = entity.color;
        ctx.beginPath();
        ctx.roundRect(entity.x, entity.y, entity.width, entity.height, 8);
        ctx.fill();

        // Border (highlight if selected)
        if (entity.id === selectedEntity) {
          ctx.strokeStyle = "#13b6ec";
          ctx.lineWidth = 3;
          ctx.stroke();
        }

        // Label
        ctx.shadowColor = "transparent";
        ctx.fillStyle = "#0d181b";
        ctx.font = '500 14px "Noto Sans", sans-serif';
        ctx.textAlign = "center";
        ctx.fillText(
          entity.label,
          entity.x + entity.width / 2,
          entity.y + entity.height / 2,
        );
      });

      // Connections (demo)
      ctx.strokeStyle = "#94a3b8";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(
        entities[0].x + entities[0].width,
        entities[0].y + entities[0].height / 2,
      );
      ctx.lineTo(entities[3].x, entities[3].y + entities[3].height / 2);
      ctx.stroke();

      ctx.restore();
    };

    render();

    const handleResize = () => render();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [entities, offset, scale, selectedEntity]);

  const handlePointerDown = (e: React.PointerEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - offset.x) / scale;
    const y = (e.clientY - rect.top - offset.y) / scale;

    // Check if clicked on entity
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

  const handlePointerMove = (e: React.PointerEvent) => {
    if (!isDragging || selectedEntity === null) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (e.clientX - rect.left - offset.x) / scale;
    const y = (e.clientY - rect.top - offset.y) / scale;

    setEntities((prev) =>
      prev.map((entity: Entity) =>
        entity.id === selectedEntity
          ? { ...entity, x: x - dragStart.x, y: y - dragStart.y }
          : entity,
      ),
    );
  };

  const handlePointerUp = () => {
    setIsDragging(false);
  };

  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setScale((prev) => Math.max(0.25, Math.min(4, prev * delta)));
  };

  return (
    <div className="relative w-full h-full">
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
