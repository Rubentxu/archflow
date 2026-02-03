/**
 * Hook for drag and drop functionality using @dnd-kit
 *
 * Manages drag operations from sidebar to canvas with coordinate
 * transformation and visual feedback.
 */

import { useCallback, useState } from "react";
import {
  DndContext,
  useSensor,
  useSensors,
  PointerSensor,
  useDraggable,
  useDroppable,
  type DraggableAttributes,
} from "@dnd-kit/core";
import type {
  DragStartEvent,
  DragEndEvent,
  DragOverEvent,
} from "@dnd-kit/core";
import { useArchFlowWasm } from "./useArchFlowWasm";
import { useCamera } from "./useCamera";
import { useUIStore } from "../store/useUIStore";
import { getTypedBridge } from "./wasm-bridge";
import type { EntityId, Vec2, CameraState } from "../types/wasm";

/** Entity template for drag operations */
export interface EntityTemplate {
  type: string;
  name: string;
  icon: React.ComponentType<{ className?: string }>;
  category: string;
  defaultSize: { width: number; height: number };
  description: string;
}

/** Drag state */
interface DragState {
  isDragging: boolean;
  activeTemplate: EntityTemplate | null;
  dropPosition: Vec2 | null;
}

/** Callback props for render prop components */
interface DraggableCallbacks {
  isDragging: boolean;
  attributes: DraggableAttributes;
  listeners: ReturnType<typeof useDraggable>["listeners"];
  setNodeRef: (node: HTMLElement | null) => void;
  transform: { x: number; y: number } | null;
}

/** Droppable callbacks */
interface DroppableCallbacks {
  isOver: boolean;
  setNodeRef: (node: HTMLElement | null) => void;
}

/** Hook result for drag and drop operations */
export interface UseDragAndDropReturn {
  DndProvider: React.ComponentType<{ children: React.ReactNode }>;
  DraggableItem: React.ComponentType<{
    template: EntityTemplate;
    children: (props: DraggableCallbacks) => React.ReactNode;
  }>;
  CanvasDroppable: React.ComponentType<{
    children: (props: DroppableCallbacks) => React.ReactNode;
  }>;
  DragOverlayContent: React.ComponentType;
  dragState: DragState;
  spawnAtPosition: (template: EntityTemplate, position: Vec2) => EntityId;
}

/** Custom hook for drag and drop implementation */
export function useDragAndDrop(): UseDragAndDropReturn {
  const { bridge, isLoaded } = useArchFlowWasm();
  const { camera } = useCamera();
  const { setActiveTool } = useUIStore();

  const [dragState, setDragState] = useState<DragState>({
    isDragging: false,
    activeTemplate: null,
    dropPosition: null,
  });

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    }),
  );

  const screenToWorld = useCallback(
    (screenX: number, screenY: number, elementRect?: DOMRect): Vec2 => {
      const rect = elementRect;
      const camState = camera as CameraState;
      const canvasX = rect ? screenX - rect.left : screenX;
      const canvasY = rect ? screenY - rect.top : screenY;
      const center = camState.center || { x: 0, y: 0 };
      return {
        x: canvasX / camState.zoom - center.x,
        y: canvasY / camState.zoom - center.y,
      };
    },
    [camera],
  );

  const spawnAtPosition = useCallback(
    (template: EntityTemplate, position: Vec2): EntityId => {
      const typed = getTypedBridge(bridge);
      if (!isLoaded || !typed) {
        return Date.now() + Math.floor(Math.random() * 1000);
      }
      try {
        const entityId = typed.spawn_entity(
          position.x,
          position.y,
          template.defaultSize.width,
          template.defaultSize.height,
        );
        return entityId;
      } catch (error) {
        console.error("Failed to spawn entity:", error);
        return -1;
      }
    },
    [isLoaded, bridge],
  );

  const handleDragStart = useCallback((event: DragStartEvent) => {
    const template = event.active.data.current as EntityTemplate | undefined;
    if (template) {
      setDragState((prev) => ({
        ...prev,
        isDragging: true,
        activeTemplate: template,
        dropPosition: null,
      }));
    }
  }, []);

  const handleDragOver = useCallback(
    (event: DragOverEvent) => {
      const { activatorEvent } = event;
      if (dragState.activeTemplate && activatorEvent) {
        const ptrEvent = activatorEvent as PointerEvent;
        const rect =
          ptrEvent.target instanceof HTMLElement
            ? ptrEvent.target.getBoundingClientRect()
            : undefined;
        const worldPos = screenToWorld(
          ptrEvent.clientX,
          ptrEvent.clientY,
          rect,
        );
        setDragState((prev) => ({ ...prev, dropPosition: worldPos }));
      }
    },
    [dragState.activeTemplate, screenToWorld],
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { activatorEvent } = event;
      if (dragState.activeTemplate && activatorEvent) {
        const ptrEvent = activatorEvent as PointerEvent;
        const rect =
          ptrEvent.target instanceof HTMLElement
            ? ptrEvent.target.getBoundingClientRect()
            : undefined;
        const worldPos = screenToWorld(
          ptrEvent.clientX,
          ptrEvent.clientY,
          rect,
        );
        spawnAtPosition(dragState.activeTemplate, worldPos);
        setActiveTool("select");
      }
      setDragState({
        isDragging: false,
        activeTemplate: null,
        dropPosition: null,
      });
    },
    [dragState.activeTemplate, screenToWorld, spawnAtPosition, setActiveTool],
  );

  const DraggableItem = useCallback(
    ({
      template,
      children,
    }: {
      template: EntityTemplate;
      children: (props: DraggableCallbacks) => React.ReactNode;
    }) => {
      const { attributes, listeners, setNodeRef, transform, isDragging } =
        useDraggable({
          id: `template-${template.type}`,
          data: template,
        });
      return children({
        isDragging,
        attributes,
        listeners,
        setNodeRef,
        transform,
      });
    },
    [],
  );

  const CanvasDroppable = useCallback(
    ({
      children,
    }: {
      children: (props: DroppableCallbacks) => React.ReactNode;
    }) => {
      const { setNodeRef, isOver } = useDroppable({ id: "canvas-droppable" });
      return children({ isOver, setNodeRef });
    },
    [],
  );

  const DndProvider = useCallback(
    ({ children }: { children: React.ReactNode }) => (
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragOver={handleDragOver}
        onDragEnd={handleDragEnd}
      >
        {children}
      </DndContext>
    ),
    [sensors, handleDragStart, handleDragOver, handleDragEnd],
  );

  const DragOverlayContent = useCallback(() => {
    if (!dragState.activeTemplate) return null;
    const Icon = dragState.activeTemplate.icon;
    return (
      <div className="flex items-center gap-2 px-3 py-2 bg-surface-dark/95 border border-primary/50 rounded-lg shadow-xl">
        <Icon className="w-4 h-4 text-primary" />
        <span className="text-sm text-gray-200">
          {dragState.activeTemplate.name}
        </span>
      </div>
    );
  }, [dragState.activeTemplate]);

  return {
    DndProvider,
    DraggableItem,
    CanvasDroppable,
    DragOverlayContent,
    dragState,
    spawnAtPosition,
  };
}
