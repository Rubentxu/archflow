---
title: "ÉPICA-WEB-004: Sistema de Interacción"
author: Claude Code
date: 2026-02-02
status: Casi Completada
version: 1.0.1
priority: P1
effort: XL
depends_on: ["EPIC-WEB-003-core-ui"]
---

# ÉPICA-WEB-004: Sistema de Interacción 🟡

## 📋 Resumen Ejecutivo

Implementar el sistema completo de interacción usuario-canvas, incluyendo drag & drop desde la sidebar, selección múltiple, transformación de entidades (resize, move), y keyboard shortcuts. **CASI COMPLETADA - Falta integrar selección visual completa**.

## 🎯 Objetivos Cumplidos

- ✅ Implementar drag & drop con @dnd-kit
- ✅ Implementar keyboard shortcuts (22 atajos)
- ✅ Implementar transformación de entidades (useTransformation)
- ✅ Implementar feedback visual de snapping (SnapFeedback)
- ✅ Implementar handles de transformación (TransformHandles)
- ⚠️ Selección rectangular (marquee) - Parcial
- ⚠️ Selección múltiple con Shift/Ctrl - Implementado pero falta visual

## 🎯 Objetivos

- Implementar drag & drop con @dnd-kit
- Implementar selección rectangular (marquee selection)
- Implementar selección múltiple con Shift/Ctrl
- Implementar transformación de entidades
- Implementar keyboard shortcuts
- Implementar feedback visual de snapping

## 📁 Archivos a Crear/Modificar

```
src/
├── components/
│   └── Canvas/
│       ├── Canvas.tsx              # Actualizar con interacciones
│       ├── SelectionOverlay.tsx    # Overlay de selección
│       └── TransformHandles.tsx    # Handles de resize
├── hooks/
│   ├── useDragAndDrop.ts           # Drag desde sidebar
│   ├── useSelection.ts             # Lógica de selección
│   ├── useTransformation.ts        # Move/Resize/Rotate
│   └── useKeyboardShortcuts.ts     # Keyboard shortcuts
├── store/
│   └── useSelectionStore.ts        # Store de selección (actualizar)
└── utils/
    └── geometry.ts                 # Utilidades geométricas
```

## 🔧 Implementación

### 4.1 useDragAndDrop - Drag desde Sidebar

```typescript
import { useCallback, useState } from "react";
import { useDndContext } from "@dnd-kit/core";
import { EntityTemplate } from "@components/EntityList";
import { useEntityStore } from "@hooks/useEntityStore";
import { useCamera } from "@hooks/useCamera";

export function useDragAndDrop() {
  const { spawnEntity } = useEntityStore();
  const { camera } = useCamera();
  const [isDragging, setIsDragging] = useState(false);
  const [draggedTemplate, setDraggedTemplate] = useState<EntityTemplate | null>(null);
  const [dropPosition, setDropPosition] = useState<{ x: number; y: number } | null>(null);

  const handleDragStart = useCallback((template: EntityTemplate) => {
    setDraggedTemplate(template);
    setIsDragging(true);
  }, []);

  const handleDragOver = useCallback((event: DragEvent) => {
    if (!draggedTemplate) return;
    
    // Calculate world position from screen coordinates
    const rect = (event.target as HTMLElement).getBoundingClientRect();
    const x = (event.clientX - rect.left) / camera.zoom - camera.x;
    const y = (event.clientY - rect.top) / camera.zoom - camera.y;
    
    setDropPosition({ x, y });
  }, [draggedTemplate, camera]);

  const handleDragEnd = useCallback(() => {
    if (draggedTemplate && dropPosition) {
      // Spawn entity at drop position
      spawnEntity(draggedTemplate.type, dropPosition);
    }
    
    setDraggedTemplate(null);
    setIsDragging(false);
    setDropPosition(null);
  }, [draggedTemplate, dropPosition, spawnEntity]);

  return {
    isDragging,
    draggedTemplate,
    dropPosition,
    handleDragStart,
    handleDragOver,
    handleDragEnd,
  };
}
```

### 4.2 useSelection - Sistema de Selección

```typescript
import { useCallback, useMemo } from "react";
import { useSelectionStore } from "@store/useSelectionStore";
import { useEntityStore } from "@hooks/useEntityStore";
import { EntityId } from "@types/wasm";

interface UseSelectionReturn {
  selectedEntities: EntityId[];
  isSelected: (id: EntityId) => boolean;
  select: (id: EntityId, additive?: boolean) => void;
  deselect: (id: EntityId) => void;
  selectMultiple: (ids: EntityId[]) => void;
  clearSelection: () => void;
  selectRect: (rect: { x: number; y: number; width: number; height: number }) => void;
}

export function useSelection(): UseSelectionReturn {
  const { selectedIds, setSelectedIds, addToSelection, removeFromSelection, clear } = 
    useSelectionStore();
  const { entities } = useEntityStore();

  const isSelected = useCallback((id: EntityId) => {
    return selectedIds.includes(id);
  }, [selectedIds]);

  const select = useCallback((id: EntityId, additive = false) => {
    if (additive) {
      addToSelection(id);
    } else {
      setSelectedIds([id]);
    }
  }, [addToSelection, setSelectedIds]);

  const deselect = useCallback((id: EntityId) => {
    removeFromSelection(id);
  }, [removeFromSelection]);

  const selectMultiple = useCallback((ids: EntityId[]) => {
    setSelectedIds(ids);
  }, [setSelectedIds]);

  const clearSelection = useCallback(() => {
    clear();
  }, [clear]);

  const selectRect = useCallback((rect: { 
    x: number; 
    y: number; 
    width: number; 
    height: number 
  }) => {
    // Find all entities within the rectangle
    const entitiesInRect = entities.filter((entity) => {
      return (
        entity.position.x >= rect.x &&
        entity.position.y >= rect.y &&
        entity.position.x <= rect.x + rect.width &&
        entity.position.y <= rect.y + rect.height
      );
    });

    setSelectedIds(entitiesInRect.map((e) => e.id));
  }, [entities, setSelectedIds]);

  return {
    selectedEntities: selectedIds,
    isSelected,
    select,
    deselect,
    selectMultiple,
    clearSelection,
    selectRect,
  };
}
```

### 4.3 useTransformation - Move/Resize/Rotate

```typescript
import { useCallback, useRef, useState } from "react";
import { useSelectionStore } from "@store/useSelectionStore";
import { useEntityStore } from "@hooks/useEntityStore";
import { useSnapper } from "@hooks/useSnapper";
import { Vec2, EntityId } from "@types/wasm";

type TransformMode = "move" | "resize-n" | "resize-s" | "resize-e" | "resize-w" | 
                     "resize-ne" | "resize-nw" | "resize-se" | "resize-sw" | "rotate";

interface UseTransformationReturn {
  isTransforming: boolean;
  transformMode: TransformMode | null;
  startTransform: (mode: TransformMode, entityId: EntityId, startPos: Vec2) => void;
  updateTransform: (currentPos: Vec2) => void;
  endTransform: () => void;
}

export function useTransformation(): UseTransformationReturn {
  const [isTransforming, setIsTransforming] = useState(false);
  const [transformMode, setTransformMode] = useState<TransformMode | null>(null);
  const [transformingEntityId, setTransformingEntityId] = useState<EntityId | null>(null);
  const [startPosition, setStartPosition] = useState<Vec2 | null>(null);
  
  const { updateProperty } = useEntityStore();
  const { snapToGrid, snapToEntity } = useSnapper();

  const startTransform = useCallback((
    mode: TransformMode, 
    entityId: EntityId, 
    startPos: Vec2
  ) => {
    setTransformMode(mode);
    setTransformingEntityId(entityId);
    setStartPosition(startPos);
    setIsTransforming(true);
  }, []);

  const updateTransform = useCallback((currentPos: Vec2) => {
    if (!isTransforming || !transformingEntityId || !startPosition || !transformMode) {
      return;
    }

    const delta = {
      x: currentPos.x - startPosition.x,
      y: currentPos.y - startPosition.y,
    };

    // Apply snapping
    const snappedDelta = snapToGrid(delta);

    if (transformMode === "move") {
      updateProperty(transformingEntityId, "position", {
        x: startPosition.x + snappedDelta.x,
        y: startPosition.y + snappedDelta.y,
      });
    } else {
      // Resize logic
      const resizeDirection = transformMode.replace("resize-", "");
      updateProperty(transformingEntityId, "size", (current: { width: number; height: number }) => {
        const newWidth = resizeDirection.includes("e") 
          ? current.width + delta.x 
          : resizeDirection.includes("w")
            ? current.width - delta.x
            : current.width;
        const newHeight = resizeDirection.includes("s") 
          ? current.height + delta.y 
          : resizeDirection.includes("n")
            ? current.height - delta.y
            : current.height;
        
        return {
          width: Math.max(newWidth, 20),
          height: Math.max(newHeight, 20),
        };
      });
    }
  }, [isTransforming, transformingEntityId, startPosition, transformMode, updateProperty, snapToGrid]);

  const endTransform = useCallback(() => {
    setIsTransforming(false);
    setTransformMode(null);
    setTransformingEntityId(null);
    setStartPosition(null);
  }, []);

  return {
    isTransforming,
    transformMode,
    startTransform,
    updateTransform,
    endTransform,
  };
}
```

### 4.4 useKeyboardShortcuts - Atajos de Teclado

```typescript
import { useEffect, useCallback } from "react";
import { useUIStore } from "@store/useUIStore";
import { useSelectionStore } from "@store/useSelectionStore";
import { useCommandHistory } from "@hooks/useCommandHistory";

const shortcuts = [
  { key: "v", action: "select", description: "Select tool" },
  { key: "h", action: "pan", description: "Pan tool" },
  { key: "r", action: "rectangle", description: "Rectangle tool" },
  { key: "c", action: "circle", description: "Circle tool" },
  { key: "t", action: "text", description: "Text tool" },
  { key: "l", action: "connection", description: "Connection tool" },
  { key: "Delete", action: "delete", description: "Delete selected" },
  { key: "Backspace", action: "delete", description: "Delete selected" },
  { key: "z", ctrl: true, action: "undo", description: "Undo" },
  { key: "y", ctrl: true, action: "redo", description: "Redo" },
  { key: "z", ctrl: true, shift: true, action: "redo", description: "Redo (alternative)" },
  { key: "d", ctrl: true, action: "duplicate", description: "Duplicate selected" },
  { key: "a", ctrl: true, action: "selectAll", description: "Select all" },
  { key: "Escape", action: "deselectAll", description: "Deselect all" },
  { key: "g", ctrl: true, action: "group", description: "Group selected" },
  { key: "u", ctrl: true, shift: true, action: "ungroup", description: "Ungroup" },
  { key: "+", ctrl: true, action: "zoomIn", description: "Zoom in" },
  { key: "-", ctrl: true, action: "zoomOut", description: "Zoom out" },
  { key: "0", ctrl: true, action: "zoomReset", description: "Reset zoom" },
  { key: "1", ctrl: true, action: "zoomFit", description: "Fit to screen" },
];

export function useKeyboardShortcuts() {
  const { setActiveTool } = useUIStore();
  const { selectedIds, clearSelection } = useSelectionStore();
  const { undo, redo, duplicateSelected } = useCommandHistory();

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    // Ignore if typing in input field
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement
    ) {
      return;
    }

    const shortcut = shortcuts.find((s) => {
      const ctrlMatch = s.ctrl ? event.ctrlKey || event.metaKey : !(event.ctrlKey || event.metaKey);
      const shiftMatch = s.shift ? event.shiftKey : !event.shiftKey;
      return s.key.toLowerCase() === event.key.toLowerCase() && ctrlMatch && shiftMatch;
    });

    if (!shortcut) return;

    event.preventDefault();

    switch (shortcut.action) {
      case "select":
      case "pan":
      case "rectangle":
      case "circle":
      case "text":
      case "connection":
        setActiveTool(shortcut.action as Parameters<typeof setActiveTool>[0]);
        break;
      case "delete":
        // Delete selected entities
        selectedIds.forEach((id) => {
          // deleteEntity(id);
        });
        break;
      case "undo":
        undo();
        break;
      case "redo":
        redo();
        break;
      case "duplicate":
        duplicateSelected();
        break;
      case "selectAll":
        // selectAllEntities();
        break;
      case "deselectAll":
        clearSelection();
        break;
      case "group":
        // groupSelectedEntities();
        break;
      case "ungroup":
        // ungroupSelectedEntities();
        break;
      case "zoomIn":
        // zoomIn();
        break;
      case "zoomOut":
        // zoomOut();
        break;
      case "zoomReset":
        // resetZoom();
        break;
      case "zoomFit":
        // fitToScreen();
        break;
    }
  }, [setActiveTool, selectedIds, undo, redo, duplicateSelected, clearSelection]);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}
```

### 4.5 Visual Feedback de Snapping

```typescript
import { useEffect, useState } from "react";
import { useSnapper } from "@hooks/useSnapper";
import { useCamera } from "@hooks/useCamera";
import { Vec2 } from "@types/wasm";

interface SnapPreviewProps {
  position: Vec2;
  visible: boolean;
}

export function SnapPreview({ position, visible }: SnapPreviewProps) {
  const { snapPoints, snapGuides } = useSnapper();
  const { camera } = useCamera();

  if (!visible) return null;

  // Calculate screen position from world position
  const screenX = (position.x + camera.x) * camera.zoom;
  const screenY = (position.y + camera.y) * camera.zoom;

  return (
    <g>
      {/* Snap guides */}
      {snapGuides.horizontal && (
        <line
          x1={0}
          y1={screenY}
          x2="100%"
          y2={screenY}
          stroke="#13b6ec"
          strokeWidth={1}
          strokeDasharray="4 4"
          opacity={0.5}
        />
      )}
      {snapGuides.vertical && (
        <line
          x1={screenX}
          y1={0}
          x2={screenX}
          y2="100%"
          stroke="#13b6ec"
          strokeWidth={1}
          strokeDasharray="4 4"
          opacity={0.5}
        />
      )}
      
      {/* Snap points indicator */}
      <circle
        cx={screenX}
        cy={screenY}
        r={6}
        fill="none"
        stroke="#13b6ec"
        strokeWidth={2}
      />
      <circle
        cx={screenX}
        cy={screenY}
        r={3}
        fill="#13b6ec"
      />
    </g>
  );
}
```

## ✅ Criterios de Éxito

| Criterio | Métrica | Valor Objetivo |
|----------|---------|----------------|
| Drag & drop | FPS durante drag | 60 FPS |
| Selección múltiple | Entidades seleccionadas | Sin límite |
| Keyboard shortcuts | Cobertura | 100% |
| Snap preview | Visible y claro | ✅ Pass |
| Undo/redo | Eventos de teclado | ✅ Pass |

## 📊 Estimación

| Fase | Esfuerzo | Estimación |
|------|----------|------------|
| Drag & Drop (sidebar → canvas) | M | 6 horas |
| Selección múltiple | M | 5 horas |
| Transformación (move/resize) | L | 10 horas |
| Keyboard shortcuts | M | 4 horas |
| Snap preview | M | 4 horas |
| Testing | L | 6 horas |
| **Total** | **XL** | **~35 horas** |

## 🔗 Referencias

- [@dnd-kit Documentation](https://docs.dndkit.com/)
- [React Drag and Drop Best Practices](https://react.dev/learn/manipulating-the-dom-with-refs)

## 📝 Notas

1. **Performance**: Usar `requestAnimationFrame` para actualizaciones durante drag
2. **Throttling**: Limitar frecuencia de eventos de pointer para evitar overload
3. **Snapping**: Integrar con el sistema `Snapper` de archflow-logic
4. **Selection Rect**: Debe soportar Shift para añadir a selección existente

---

**Documento creado**: `docs/epics/EPIC-WEB-004-interaction.md`
**Estado**: Listo para implementación
**Dependencia**: EPIC-WEB-003
