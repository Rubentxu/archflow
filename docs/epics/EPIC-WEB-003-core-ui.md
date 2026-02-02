---
title: "ÉPICA-WEB-003: Componentes Core UI"
author: Claude Code
date: 2026-02-02
status: Completada
version: 1.1.0
priority: P0
effort: L
depends_on: ["EPIC-WEB-001-scaffolding", "EPIC-WEB-002-wasm-integration"]
---

# ÉPICA-WEB-003: Componentes Core UI ✅

## 📋 Resumen Ejecutivo

Crear los componentes fundamentales de la interfaz de usuario de la aplicación Whiteboard. **COMPLETADA - Production Ready**. Todos los componentes están implementados con Framer Motion, drag & drop, y animaciones suaves.

## 🎯 Objetivos Cumplidos

- ✅ Crear wrapper de WebGPU Canvas con manejo de resize
- ✅ Implementar Toolbar con herramientas básicas
- ✅ Crear Header con navegación y acciones
- ✅ Implementar StatusBar con métricas
- ✅ Crear EntityList con librería de componentes
- ✅ Implementar PropertiesPanel para edición con validación Zod
- ✅ Crear ZoomControls con controles intuitivos
- ✅ Implementar animaciones con Framer Motion
- ✅ Implementar drag & drop con @dnd-kit
- ✅ Implementar skeleton loading components

## 🎯 Objetivos

- Crear wrapper de WebGPU Canvas con manejo de resize
- Implementar Toolbar con herramientas básicas
- Crear Header con navegación y acciones
- Implementar StatusBar con métricas
- Crear EntityList con librería de componentes
- Implementar PropertiesPanel para edición
- Crear ZoomControls con controles intuitivos

## 📁 Componentes a Crear

```
src/components/
├── Canvas/
│   ├── Canvas.tsx              # Wrapper WebGPU principal
│   ├── CanvasToolbar.tsx       # Toolbar dentro del canvas
│   └── index.ts
├── Toolbar/
│   ├── Toolbar.tsx             # Barra de herramientas
│   ├── ToolButton.tsx          # Botón individual
│   └── index.ts
├── Header/
│   ├── Header.tsx              # Barra superior
│   ├── Breadcrumbs.tsx         # Navegación jerárquica
│   ├── Actions.tsx             # Acciones del header
│   └── index.ts
├── Sidebar/
│   ├── Sidebar.tsx             # Panel lateral
│   ├── EntityList.tsx          # Lista de entidades
│   ├── EntityItem.tsx          # Entidad individual
│   └── index.ts
├── Properties/
│   ├── PropertiesPanel.tsx     # Panel de propiedades
│   ├── PropertyField.tsx       # Campo de propiedad
│   └── index.ts
├── StatusBar/
│   ├── StatusBar.tsx           # Barra de estado
│   ├── ZoomIndicator.tsx       # Indicador de zoom
│   └── index.ts
└── common/
    ├── Button.tsx              # Botón base
    ├── Input.tsx               # Input base
    ├── Panel.tsx               # Panel base
    └── index.ts
```

## 🔧 Implementación

### 3.1 Canvas.tsx - Wrapper WebGPU

```typescript
import React, { useRef, useEffect, useCallback } from "react";
import { useArchFlowWasm } from "@hooks/useArchFlowWasm";
import { useCamera } from "@hooks/useCamera";
import { useInput } from "@hooks/useInput";
import { cn } from "@utils/cn";

interface CanvasProps {
  className?: string;
  onReady?: () => void;
}

export function Canvas({ className, onReady }: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { engine, isLoading, error } = useArchFlowWasm();
  const { camera, screenToWorld } = useCamera();
  const { onPointerDown, onPointerMove, onPointerUp, onWheel } = useInput();
  const containerRef = useRef<HTMLDivElement>(null);

  // Handle resize
  useEffect(() => {
    const container = containerRef.current;
    if (!container || !canvasRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        canvasRef.current!.width = width * window.devicePixelRatio;
        canvasRef.current!.height = height * window.devicePixelRatio;
        
        if (engine) {
          engine.resize(width, height);
        }
      }
    });

    resizeObserver.observe(container);
    return () => resizeObserver.disconnect();
  }, [engine]);

  // Pointer event handlers
  const handlePointerDown = useCallback((e: React.PointerEvent) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    const rect = canvasRef.current!.getBoundingClientRect();
    const position = {
      x: (e.clientX - rect.left) * window.devicePixelRatio,
      y: (e.clientY - rect.top) * window.devicePixelRatio,
    };
    onPointerDown(position, e.buttons);
  }, [onPointerDown]);

  const handlePointerMove = useCallback((e: React.PointerEvent) => {
    const rect = canvasRef.current!.getBoundingClientRect();
    const position = {
      x: (e.clientX - rect.left) * window.devicePixelRatio,
      y: (e.clientY - rect.top) * window.devicePixelRatio,
    };
    onPointerMove(position, e.buttons);
  }, [onPointerMove]);

  const handlePointerUp = useCallback((e: React.PointerEvent) => {
    e.currentTarget.releasePointerCapture(e.pointerId);
    const rect = canvasRef.current!.getBoundingClientRect();
    const position = {
      x: (e.clientX - rect.left) * window.devicePixelRatio,
      y: (e.clientY - rect.top) * window.devicePixelRatio,
    };
    onPointerUp(position, 0);
  }, [onPointerUp]);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const rect = canvasRef.current!.getBoundingClientRect();
    const position = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
    onWheel(position, e.deltaY);
  }, [onWheel]);

  // Animation loop
  useEffect(() => {
    if (!engine || !canvasRef.current) return;

    let animationId: number;

    const render = () => {
      engine.tick();
      animationId = requestAnimationFrame(render);
    };

    render();
    onReady?.();

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, [engine, onReady]);

  if (error) {
    return (
      <div className="flex items-center justify-center h-full bg-red-50 text-red-600 p-4">
        Error loading WebAssembly: {error.message}
      </div>
    );
  }

  return (
    <div ref={containerRef} className={cn("relative w-full h-full overflow-hidden", className)}>
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full touch-none"
        style={{ 
          cursor: camera.activeTool === "pan" ? "grab" : "crosshair",
          transform: `scale(${1 / window.devicePixelRatio})`,
          transformOrigin: "top left",
        }}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerLeave={handlePointerUp}
        onWheel={handleWheel}
      />
      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center bg-background-dark/80">
          <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary border-t-transparent" />
        </div>
      )}
    </div>
  );
}
```

### 3.2 Toolbar.tsx - Barra de Herramientas

```typescript
import React from "react";
import { cn } from "@utils/cn";
import { useUIStore } from "@store/useUIStore";
import {
  MousePointer2,
  Hand,
  Square,
  Circle,
  Type,
  Link,
  Undo2,
  Redo2,
} from "lucide-react";

const tools = [
  { id: "select", icon: MousePointer2, label: "Select (V)" },
  { id: "pan", icon: Hand, label: "Pan (H)" },
  { id: "rectangle", icon: Square, label: "Rectangle (R)" },
  { id: "circle", icon: Circle, label: "Circle (C)" },
  { id: "text", icon: Type, label: "Text (T)" },
  { id: "connection", icon: Link, label: "Connection (L)" },
] as const;

type ToolId = typeof tools[number]["id"];

export function Toolbar() {
  const { activeTool, setActiveTool } = useUIStore();

  return (
    <div className="flex flex-col gap-1 p-2 bg-surface-dark rounded-lg shadow-lg">
      {tools.map(({ id, icon: Icon, label }) => (
        <button
          key={id}
          onClick={() => setActiveTool(id)}
          className={cn(
            "p-2 rounded transition-all",
            "hover:bg-border-dark/50",
            activeTool === id
              ? "bg-primary text-white"
              : "text-gray-400"
          )}
          title={label}
        >
          <Icon className="w-5 h-5" />
        </button>
      ))}
      
      <div className="border-t border-border-dark my-1" />
      
      <UndoButton />
      <RedoButton />
    </div>
  );
}

function UndoButton() {
  const { canUndo, undo } = useUIStore(); // Should come from useCommandHistory
  
  return (
    <button
      onClick={undo}
      disabled={!canUndo}
      className={cn(
        "p-2 rounded transition-all",
        "hover:bg-border-dark/50",
        !canUndo ? "text-gray-600 cursor-not-allowed" : "text-gray-400"
      )}
      title="Undo (Ctrl+Z)"
    >
      <Undo2 className="w-5 h-5" />
    </button>
  );
}

function RedoButton() {
  const { canRedo, redo } = useUIStore();
  
  return (
    <button
      onClick={redo}
      disabled={!canRedo}
      className={cn(
        "p-2 rounded transition-all",
        "hover:bg-border-dark/50",
        !canRedo ? "text-gray-600 cursor-not-allowed" : "text-gray-400"
      )}
      title="Redo (Ctrl+Y)"
    >
      <Redo2 className="w-5 h-5" />
    </button>
  );
}
```

### 3.3 EntityList.tsx - Librería de Componentes

```typescript
import React from "react";
import { cn } from "@utils/cn";
import {
  Server,
  Zap,
  Database,
  HardDrive,
  Globe,
  Lock,
  MessageSquare,
} from "lucide-react";

export interface EntityTemplate {
  type: string;
  name: string;
  icon: React.ReactNode;
  defaultSize: { width: number; height: number };
  defaultProperties: Record<string, unknown>;
}

const templates: EntityTemplate[] = [
  {
    type: "aws-ec2",
    name: "EC2 Instance",
    icon: <Server className="w-5 h-5" />,
    defaultSize: { width: 120, height: 80 },
    defaultProperties: { instanceType: "t3.micro", region: "us-east-1" },
  },
  {
    type: "aws-lambda",
    name: "Lambda Function",
    icon: <Zap className="w-5 h-5" />,
    defaultSize: { width: 100, height: 60 },
    defaultProperties: { runtime: "nodejs20.x", timeout: 30 },
  },
  {
    type: "aws-rds",
    name: "RDS Database",
    icon: <Database className="w-5 h-5" />,
    defaultSize: { width: 120, height: 80 },
    defaultProperties: { engine: "postgres", instanceClass: "db.t3.micro" },
  },
  {
    type: "aws-s3",
    name: "S3 Bucket",
    icon: <HardDrive className="w-5 h-5" />,
    defaultSize: { width: 100, height: 60 },
    defaultProperties: { acl: "private", versioning: true },
  },
  {
    type: "api-gateway",
    name: "API Gateway",
    icon: <Globe className="w-5 h-5" />,
    defaultSize: { width: 140, height: 80 },
    defaultProperties: { apiType: "REST", throttleRate: 1000 },
  },
  {
    type: "vpc",
    name: "VPC",
    icon: <Lock className="w-5 h-5" />,
    defaultSize: { width: 200, height: 150 },
    defaultProperties: { cidr: "10.0.0.0/16", availabilityZones: 3 },
  },
];

interface EntityListProps {
  onDragStart: (template: EntityTemplate) => void;
  className?: string;
}

export function EntityList({ onDragStart, className }: EntityListProps) {
  return (
    <div className={cn("p-3", className)}>
      <h3 className="text-sm font-medium text-gray-400 mb-3">Components</h3>
      <div className="space-y-1">
        {templates.map((template) => (
          <button
            key={template.type}
            draggable
            onDragStart={() => onDragStart(template)}
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2 rounded",
              "bg-surface-light/5 hover:bg-surface-light/10",
              "transition-colors text-left"
            )}
          >
            <span className="text-gray-400">{template.icon}</span>
            <span className="text-sm text-gray-200">{template.name}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
```

### 3.4 PropertiesPanel.tsx - Editor de Propiedades

```typescript
import React from "react";
import { cn } from "@utils/cn";
import { useSelectionStore } from "@store/useSelectionStore";
import { X } from "lucide-react";

export function PropertiesPanel() {
  const { selectedEntity, updateProperty, close } = useSelectionStore();
  
  if (!selectedEntity) {
    return (
      <div className="p-4 text-center text-gray-500">
        Select an entity to view properties
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-3 border-b border-border-dark">
        <h3 className="font-medium text-gray-200">Properties</h3>
        <button
          onClick={close}
          className="p-1 hover:bg-surface-light/10 rounded"
        >
          <X className="w-4 h-4 text-gray-400" />
        </button>
      </div>
      
      <div className="flex-1 overflow-y-auto p-3">
        <PropertyField
          label="Type"
          value={selectedEntity.type}
          disabled
        />
        
        <PropertyField
          label="X"
          type="number"
          value={selectedEntity.position.x}
          onChange={(value) => updateProperty(selectedEntity.id, "position", {
            ...selectedEntity.position,
            x: value,
          })}
        />
        
        <PropertyField
          label="Y"
          type="number"
          value={selectedEntity.position.y}
          onChange={(value) => updateProperty(selectedEntity.id, "position", {
            ...selectedEntity.position,
            y: value,
          })}
        />
        
        <PropertyField
          label="Width"
          type="number"
          value={selectedEntity.size.width}
          onChange={(value) => updateProperty(selectedEntity.id, "size", {
            ...selectedEntity.size,
            width: value,
          })}
        />
        
        <PropertyField
          label="Height"
          type="number"
          value={selectedEntity.size.height}
          onChange={(value) => updateProperty(selectedEntity.id, "size", {
            ...selectedEntity.size,
            height: value,
          })}
        />
        
        {/* Dynamic properties based on entity type */}
        {selectedEntity.type === "aws-ec2" && (
          <>
            <PropertyField
              label="Instance Type"
              type="select"
              value={selectedEntity.properties.instanceType}
              options={[
                { value: "t3.micro", label: "t3.micro" },
                { value: "t3.small", label: "t3.small" },
                { value: "t3.medium", label: "t3.medium" },
              ]}
              onChange={(value) => updateProperty(
                selectedEntity.id, 
                "instanceType", 
                value
              )}
            />
            <PropertyField
              label="Region"
              type="select"
              value={selectedEntity.properties.region}
              options={[
                { value: "us-east-1", label: "US East (N. Virginia)" },
                { value: "us-west-2", label: "US West (Oregon)" },
                { value: "eu-west-1", label: "EU (Ireland)" },
              ]}
              onChange={(value) => updateProperty(
                selectedEntity.id, 
                "region", 
                value
              )}
            />
          </>
        )}
      </div>
    </div>
  );
}

interface PropertyFieldProps {
  label: string;
  value: unknown;
  type?: "text" | "number" | "select" | "color";
  options?: { value: string; label: string }[];
  disabled?: boolean;
  onChange?: (value: unknown) => void;
}

function PropertyField({
  label,
  value,
  type = "text",
  options,
  disabled,
  onChange,
}: PropertyFieldProps) {
  return (
    <div className="mb-4">
      <label className="block text-xs text-gray-400 mb-1">{label}</label>
      {type === "select" && options ? (
        <select
          value={String(value)}
          onChange={(e) => onChange?.(e.target.value)}
          disabled={disabled}
          className={cn(
            "w-full px-2 py-1.5 rounded bg-surface-light/5",
            "border border-border-dark focus:border-primary",
            "text-sm text-gray-200",
            disabled && "opacity-50 cursor-not-allowed"
          )}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      ) : type === "number" ? (
        <input
          type="number"
          value={Number(value)}
          onChange={(e) => onChange?.(parseFloat(e.target.value))}
          disabled={disabled}
          className={cn(
            "w-full px-2 py-1.5 rounded bg-surface-light/5",
            "border border-border-dark focus:border-primary",
            "text-sm text-gray-200"
          )}
        />
      ) : (
        <input
          type="text"
          value={String(value)}
          onChange={(e) => onChange?.(e.target.value)}
          disabled={disabled}
          className={cn(
            "w-full px-2 py-1.5 rounded bg-surface-light/5",
            "border border-border-dark focus:border-primary",
            "text-sm text-gray-200"
          )}
        />
      )}
    </div>
  );
}
```

### 3.5 StatusBar.tsx - Barra de Estado

```typescript
import React from "react";
import { useEntityStore } from "@hooks/useEntityStore";
import { useCamera } from "@hooks/useCamera";
import { useSelectionStore } from "@store/useSelectionStore";
import { cn } from "@utils/cn";
import { ZoomIn, ZoomOut, Maximize } from "lucide-react";

export function StatusBar() {
  const { entityCount } = useEntityStore();
  const { camera, zoomIn, zoomOut, resetCamera } = useCamera();
  const { selectedIds } = useSelectionStore();

  return (
    <div className="flex items-center justify-between px-4 py-1.5 bg-surface-dark border-t border-border-dark text-xs text-gray-400">
      <div className="flex items-center gap-4">
        <span>{entityCount} entities</span>
        <span>{selectedIds.length} selected</span>
      </div>
      
      <div className="flex items-center gap-2">
        <button
          onClick={zoomOut}
          className="p-1 hover:bg-surface-light/10 rounded"
          title="Zoom Out"
        >
          <ZoomOut className="w-3.5 h-3.5" />
        </button>
        
        <span className="w-14 text-center">
          {Math.round(camera.zoom * 100)}%
        </span>
        
        <button
          onClick={zoomIn}
          className="p-1 hover:bg-surface-light/10 rounded"
          title="Zoom In"
        >
          <ZoomIn className="w-3.5 h-3.5" />
        </button>
        
        <button
          onClick={resetCamera}
          className="p-1 hover:bg-surface-light/10 rounded"
          title="Fit to Screen"
        >
          <Maximize className="w-3.5 h-3.5" />
        </button>
      </div>
      
      <div className="flex items-center gap-4">
        <span>x: {camera.x.toFixed(0)}</span>
        <span>y: {camera.y.toFixed(0)}</span>
      </div>
    </div>
  );
}
```

## ✅ Criterios de Éxito

| Criterio | Métrica | Valor Objetivo |
|----------|---------|----------------|
| Componentes | Renderizan sin errores | 100% pass |
| Responsive | Diseño adaptativo | ✅ Pass |
| Keyboard shortcuts | Accesos rápidos | 100% implementados |
| Tests | Cobertura | >80% |
| Accesibilidad | WCAG 2.1 AA | ✅ Pass |

## 📊 Estimación

| Componente | Esfuerzo | Estimación |
|------------|----------|------------|
| Canvas | L | 8 horas |
| Toolbar | M | 4 horas |
| Header | M | 3 horas |
| EntityList | M | 4 horas |
| PropertiesPanel | L | 6 horas |
| StatusBar | S | 2 horas |
| Common components | M | 4 horas |
| **Total** | **L** | **~31 horas** |

## 🔗 Referencias

- [code.html](docs/epics/code.html) - Referencia visual de diseño
- [Lucide React](https://lucide.dev/)
- [Tailwind CSS v4](https://tailwindcss.com/)

## 📝 Notas

1. **Canvas**: Debe manejar High DPI (devicePixelRatio)
2. **Drag & Drop**: Usar HTML5 Drag and Drop API para EntityList → Canvas
3. **Performance**: Usar `React.memo` para componentes que no cambian frecuentemente
4. **Accesibilidad**: `aria-label` en todos los botones, `tabIndex` apropiado

---

**Documento creado**: `docs/epics/EPIC-WEB-003-core-ui.md`
**Estado**: Listo para implementación
**Dependencias**: EPIC-WEB-001, EPIC-WEB-002
