/**
 * Hook for handling entity transformations (move, resize, rotate)
 *
 * Provides centralized transformation logic with snapping support.
 * Architecture Reference: EPIC-WEB-004
 */

import { useState, useCallback, useRef } from "react";
import type { EntityId, Vec2 } from "../types/wasm";
import { useEntityStore } from "./useEntityStore";

type TransformMode =
  | "move"
  | "resize-n"
  | "resize-s"
  | "resize-e"
  | "resize-w"
  | "resize-ne"
  | "resize-nw"
  | "resize-se"
  | "resize-sw"
  | "rotate";

interface UseTransformationReturn {
  isTransforming: boolean;
  transformMode: TransformMode | null;
  startTransform: (mode: TransformMode, entityId: EntityId, startPos: Vec2) => void;
  updateTransform: (currentPos: Vec2) => void;
  endTransform: () => void;
}

const GRID_SIZE = 20;

function snapToGrid(value: number): number {
  return Math.round(value / GRID_SIZE) * GRID_SIZE;
}

export function useTransformation(): UseTransformationReturn {
  const [isTransforming, setIsTransforming] = useState(false);
  const [transformMode, setTransformMode] = useState<TransformMode | null>(null);
  const [transformingEntityId, setTransformingEntityId] = useState<EntityId | null>(null);
  const [startPosition, setStartPosition] = useState<Vec2 | null>(null);
  const [startSize, setStartSize] = useState<{ w: number; h: number } | null>(null);
  const [startAngle, setStartAngle] = useState<number>(0);

  const { updateEntity, getEntity } = useEntityStore();

  const startTransform = useCallback(
    (mode: TransformMode, entityId: EntityId, startPos: Vec2) => {
      const entity = getEntity(entityId);
      if (!entity) return;

      setTransformMode(mode);
      setTransformingEntityId(entityId);
      setStartPosition(startPos);
      setStartSize({ w: entity.size.w, h: entity.size.h });
      setStartAngle(0);
      setIsTransforming(true);
    },
    [getEntity],
  );

  const updateTransform = useCallback(
    (currentPos: Vec2) => {
      if (!isTransforming || !transformingEntityId || !startPosition || !transformMode) {
        return;
      }

      const entity = getEntity(transformingEntityId);
      if (!entity) return;

      const delta = {
        x: currentPos.x - startPosition.x,
        y: currentPos.y - startPosition.y,
      };

      if (transformMode === "move") {
        const snappedDelta = {
          x: snapToGrid(delta.x),
          y: snapToGrid(delta.y),
        };
        updateEntity(transformingEntityId, {
          position: {
            x: startPosition.x + snappedDelta.x,
            y: startPosition.y + snappedDelta.y,
          },
        });
      } else if (startSize) {
        const resizeDirection = transformMode.replace("resize-", "");
        let newWidth = startSize.w;
        let newHeight = startSize.h;
        let newX = entity.position.x;
        let newY = entity.position.y;

        const snappedDelta = {
          x: snapToGrid(delta.x),
          y: snapToGrid(delta.y),
        };

        if (resizeDirection.includes("e")) {
          newWidth = Math.max(20, startSize.w + snappedDelta.x);
        }
        if (resizeDirection.includes("w")) {
          newWidth = Math.max(20, startSize.w - snappedDelta.x);
          newX = startPosition.x + snappedDelta.x;
        }
        if (resizeDirection.includes("s")) {
          newHeight = Math.max(20, startSize.h + snappedDelta.y);
        }
        if (resizeDirection.includes("n")) {
          newHeight = Math.max(20, startSize.h - snappedDelta.y);
          newY = startPosition.y + snappedDelta.y;
        }

        updateEntity(transformingEntityId, {
          position: { x: newX, y: newY },
          size: { w: newWidth, h: newHeight },
        });
      }
    },
    [
      isTransforming,
      transformingEntityId,
      startPosition,
      startSize,
      transformMode,
      getEntity,
      updateEntity,
    ],
  );

  const endTransform = useCallback(() => {
    setIsTransforming(false);
    setTransformMode(null);
    setTransformingEntityId(null);
    setStartPosition(null);
    setStartSize(null);
    setStartAngle(0);
  }, []);

  return {
    isTransforming,
    transformMode,
    startTransform,
    updateTransform,
    endTransform,
  };
}
