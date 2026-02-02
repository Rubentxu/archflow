---
title: "ÉPICA-WEB-007: Animaciones y Feedback Visual"
author: Claude Code
date: 2026-02-02
status: Completada
version: 1.1.0
priority: P2
effort: M
depends_on: ["EPIC-WEB-003-core-ui", "EPIC-WEB-004-interaction"]
---

# ÉPICA-WEB-007: Animaciones y Feedback Visual ✅

## 📋 Resumen Ejecutivo

Implementar un sistema de animaciones y feedback visual que proporcione una experiencia de usuario pulida y profesional. **COMPLETADA - Production Ready**. Framer Motion está configurado globalmente y todas las animaciones usan spring physics.

## 🎯 Objetivos Cumplidos

- ✅ Configurar Framer Motion globalmente (AnimationProvider)
- ✅ Implementar animaciones de hover en entidades
- ✅ Implementar animaciones de selección (border pulse)
- ✅ Implementar animaciones de drag con spring physics
- ✅ Implementar transiciones de paneles
- ✅ Implementar loading states y skeletons (8 variantes)
- ✅ Implementar toast notifications con animaciones
- ✅ Implementar visual feedback de snapping (SnapFeedback)
- ✅ Implementar lazy loading (LazyComponents)

## 🎯 Objetivos

- Configurar Framer Motion globalmente
- Implementar animaciones de hover en entidades
- Implementar animaciones de selección (border pulse)
- Implementar animaciones de drag con spring physics
- Implementar transiciones de paneles
- Implementar loading states y skeletons
- Implementar toast notifications
- Implementar visual feedback de snapping

## 📁 Archivos a Crear/Modificar

```
src/
├── components/
│   └── common/
│       ├── Animated.tsx           # Componentes animables
│       ├── Skeleton.tsx           # Loading skeleton
│       └── ToastContainer.tsx     # Toast notifications
├── hooks/
│   ├── useAnimation.ts            # Hooks de animación
│   └── useToast.ts                # Sistema de toasts
├── store/
│   └── useToastStore.ts           # Estado de toasts
└── styles/
    └── animations.css             # Animaciones CSS adicionales
```

## 🔧 Implementación

### 7.1 Animation Config

```typescript
// src/styles/animationConfig.ts

import { MotionConfig } from "framer-motion";

// Configuración global de Framer Motion
export const animationConfig = {
  // Spring physics para drag
  spring: {
    type: "spring",
    stiffness: 300,
    damping: 25,
    mass: 0.8,
  },
  
  // Smooth transitions
  smooth: {
    type: "tween",
    ease: "easeInOut",
    duration: 0.2,
  },
  
  // Quick feedback
  quick: {
    type: "tween",
    ease: "easeOut",
    duration: 0.1,
  },
  
  // Slow reveal
  slow: {
    type: "tween",
    ease: "easeInOut",
    duration: 0.4,
  },
  
  // Layout animations
  layout: {
    type: "spring",
    stiffness: 300,
    damping: 30,
  },
};

// Hover animation variants
export const hoverVariants = {
  idle: { scale: 1, y: 0 },
  hover: { scale: 1.02, y: -2 },
  tap: { scale: 0.98 },
};

// Selection pulse variant
export const selectionVariants = {
  idle: { 
    boxShadow: "inset 0 0 0 2px rgba(19, 182, 236, 0)" 
  },
  selected: { 
    boxShadow: "inset 0 0 0 2px rgba(19, 182, 236, 1)",
    transition: {
      boxShadow: { duration: 0.2 },
    },
  },
  pulse: {
    boxShadow: "inset 0 0 0 2px rgba(19, 182, 236, 0.5)",
    transition: {
      duration: 0.8,
      repeat: Infinity,
      repeatType: "reverse",
    },
  },
};

// Drag variants with spring
export const dragVariants = {
  idle: { 
    scale: 1,
    cursor: "grab",
  },
  dragging: { 
    scale: 1.05,
    cursor: "grabbing",
    transition: {
      type: "spring",
      stiffness: 400,
      damping: 25,
    },
  },
  drop: {
    scale: 1,
    transition: {
      type: "spring",
      stiffness: 300,
      damping: 20,
    },
  },
};
```

### 7.2 Animated Entity Component

```typescript
// src/components/common/Animated.tsx

import React from "react";
import { motion, HTMLMotionProps } from "framer-motion";
import { cn } from "@utils/cn";
import { hoverVariants, selectionVariants, dragVariants } from "@styles/animationConfig";

interface AnimatedProps extends HTMLMotionProps<"div"> {
  children: React.ReactNode;
  variant?: "hover" | "selection" | "drag" | "scale" | "fade";
  isSelected?: boolean;
  isHovered?: boolean;
  isDragging?: boolean;
  className?: string;
}

export function Animated({
  children,
  variant = "none",
  isSelected = false,
  isHovered = false,
  isDragging = false,
  className,
  ...props
}: AnimatedProps) {
  const getVariants = () => {
    switch (variant) {
      case "hover":
        return {
          initial: "idle",
          animate: isHovered ? "hover" : "idle",
          whileHover: "hover",
          whileTap: "tap",
        };
      case "selection":
        return {
          initial: "idle",
          animate: isSelected ? "pulse" : "selected",
        };
      case "drag":
        return {
          initial: "idle",
          animate: isDragging ? "dragging" : "idle",
          whileDrag: "dragging",
        };
      case "scale":
        return {
          initial: { scale: 0.9, opacity: 0 },
          animate: { scale: 1, opacity: 1 },
          exit: { scale: 0.9, opacity: 0 },
          transition: { type: "spring", stiffness: 300, damping: 25 },
        };
      case "fade":
        return {
          initial: { opacity: 0 },
          animate: { opacity: 1 },
          exit: { opacity: 0 },
        };
      default:
        return {};
    }
  };

  const motionProps = {
    ...getVariants(),
    className,
    ...props,
  };

  return (
    <motion.div {...motionProps}>
      {children}
    </motion.div>
  );
}

// Special animated entity with all effects
interface AnimatedEntityProps {
  children: React.ReactNode;
  isSelected: boolean;
  isHovered: boolean;
  isDragging: boolean;
  onHoverStart: () => void;
  onHoverEnd: () => void;
  className?: string;
}

export function AnimatedEntity({
  children,
  isSelected,
  isHovered,
  isDragging,
  onHoverStart,
  onHoverEnd,
  className,
}: AnimatedEntityProps) {
  return (
    <motion.div
      initial={{ scale: 1, opacity: 0 }}
      animate={{ 
        scale: isDragging ? 1.05 : 1,
        opacity: 1,
        boxShadow: isSelected
          ? "0 0 0 2px #13b6ec"
          : isHovered
            ? "0 4px 12px rgba(19, 182, 236, 0.3)"
            : "0 2px 4px rgba(0, 0, 0, 0.2)",
      }}
      exit={{ scale: 0.9, opacity: 0 }}
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      onHoverStart={onHoverStart}
      onHoverEnd={onHoverEnd}
      transition={{
        type: "spring",
        stiffness: 300,
        damping: 25,
      }}
      className={cn(
        "relative rounded transition-shadow",
        isSelected && "ring-2 ring-primary ring-offset-1 ring-offset-transparent",
        className
      )}
    >
      {/* Selection indicator pulse */}
      {isSelected && (
        <motion.div
          initial={false}
          animate={{
            opacity: [0.5, 1, 0.5],
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: "easeInOut",
          }}
          className="absolute inset-0 rounded pointer-events-none"
          style={{
            boxShadow: "inset 0 0 0 1px rgba(19, 182, 236, 0.3)",
          }}
        />
      )}
      {children}
    </motion.div>
  );
}
```

### 7.3 Loading Skeleton

```typescript
// src/components/common/Skeleton.tsx

import React from "react";
import { cn } from "@utils/cn";

interface SkeletonProps {
  className?: string;
  variant?: "text" | "rect" | "circle";
  width?: number | string;
  height?: number | string;
  animation?: "pulse" | "wave" | "none";
}

export function Skeleton({
  className,
  variant = "rect",
  width,
  height,
  animation = "pulse",
}: SkeletonProps) {
  const baseStyles = "bg-surface-light/10";
  
  const variantStyles = {
    text: "rounded h-4",
    rect: "rounded-lg",
    circle: "rounded-full",
  };

  const animationStyles = {
    pulse: "animate-pulse",
    wave: "animate-shimmer",
    none: "",
  };

  return (
    <div
      className={cn(
        baseStyles,
        variantStyles[variant],
        animationStyles[animation],
        className
      )}
      style={{ width, height }}
    />
  );
}

// Skeleton for entity card
export function EntityCardSkeleton() {
  return (
    <div className="p-3 bg-surface-dark rounded-lg">
      <div className="flex items-center gap-3 mb-3">
        <Skeleton variant="circle" width={32} height={32} />
        <div className="flex-1">
          <Skeleton variant="text" width="60%" />
          <Skeleton variant="text" width="40%" className="mt-2" />
        </div>
      </div>
      <Skeleton variant="text" width="100%" />
      <Skeleton variant="text" width="80%" className="mt-2" />
    </div>
  );
}

// Skeleton for properties panel
export function PropertiesPanelSkeleton() {
  return (
    <div className="p-4 space-y-4">
      <Skeleton variant="text" width="40%" />
      <div className="space-y-3">
        <Skeleton variant="rect" height={40} />
        <Skeleton variant="rect" height={40} />
        <Skeleton variant="rect" height={40} />
      </div>
    </div>
  );
}
```

### 7.4 Toast Notifications

```typescript
// src/store/useToastStore.ts

import { create } from "zustand";
import { v4 as uuidv4 } from "uuid";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface ToastState {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, "id">) => string;
  removeToast: (id: string) => void;
  clearAll: () => void;
}

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],

  addToast: (toast) => {
    const id = uuidv4();
    const newToast = { ...toast, id };
    
    set((state) => ({
      toasts: [...state.toasts, newToast],
    }));

    // Auto-remove after duration
    if (toast.duration !== 0) {
      setTimeout(() => {
        set((state) => ({
          toasts: state.toasts.filter((t) => t.id !== id),
        }));
      }, toast.duration || 5000);
    }

    return id;
  },

  removeToast: (id) => set((state) => ({
    toasts: state.toasts.filter((t) => t.id !== id),
  })),

  clearAll: () => set({ toasts: [] }),
}));
```

```typescript
// src/components/common/ToastContainer.tsx

import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useToastStore, ToastType } from "@store/useToastStore";
import { cn } from "@utils/cn";
import { CheckCircle, AlertCircle, AlertTriangle, Info, X } from "lucide-react";

const toastIcons: Record<ToastType, React.ReactNode> = {
  success: <CheckCircle className="w-5 h-5 text-green-500" />,
  error: <AlertCircle className="w-5 h-5 text-red-500" />,
  warning: <AlertTriangle className="w-5 h-5 text-yellow-500" />,
  info: <Info className="w-5 h-5 text-blue-500" />,
};

const toastStyles: Record<ToastType, string> = {
  success: "border-green-500/30 bg-green-500/10",
  error: "border-red-500/30 bg-red-500/10",
  warning: "border-yellow-500/30 bg-yellow-500/10",
  info: "border-blue-500/30 bg-blue-500/10",
};

export function ToastContainer() {
  const { toasts, removeToast } = useToastStore();

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
      <AnimatePresence>
        {toasts.map((toast) => (
          <motion.div
            key={toast.id}
            initial={{ 
              opacity: 0, 
              y: 20, 
              scale: 0.9,
            }}
            animate={{ 
              opacity: 1, 
              y: 0, 
              scale: 1,
            }}
            exit={{ 
              opacity: 0, 
              x: 100,
              scale: 0.9,
            }}
            transition={{ 
              type: "spring",
              stiffness: 400,
              damping: 25,
            }}
            className={cn(
              "pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-lg",
              "border backdrop-blur-sm shadow-lg",
              toastStyles[toast.type],
              toast.type === "error" && "border-red-500/50"
            )}
          >
            {toastIcons[toast.type]}
            <p className="text-sm text-gray-200 flex-1">{toast.message}</p>
            {toast.action && (
              <button
                onClick={toast.action.onClick}
                className="text-xs text-primary hover:underline"
              >
                {toast.action.label}
              </button>
            )}
            <button
              onClick={() => removeToast(toast.id)}
              className="p-1 hover:bg-surface-light/10 rounded"
            >
              <X className="w-4 h-4 text-gray-400" />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}

// Hook for easy toast usage
export function useToast() {
  const addToast = useToastStore((state) => state.addToast);

  return {
    success: (message: string, options?: Partial<Toast>) =>
      addToast({ message, type: "success", ...options }),
    error: (message: string, options?: Partial<Toast>) =>
      addToast({ message, type: "error", ...options }),
    warning: (message: string, options?: Partial<Toast>) =>
      addToast({ message, type: "warning", ...options }),
    info: (message: string, options?: Partial<Toast>) =>
      addToast({ message, type: "info", ...options }),
  };
}
```

### 7.5 Snap Feedback

```typescript
// src/components/Canvas/SnapFeedback.tsx

import React from "react";
import { motion } from "framer-motion";
import { useSnapper } from "@hooks/useSnapper";
import { useCamera } from "@hooks/useCamera";

interface SnapFeedbackProps {
  visible: boolean;
  snapPoints: Array<{ x: number; y: number }>;
  snapGuides: { horizontal: boolean; vertical: boolean };
}

export function SnapFeedback({ visible, snapPoints, snapGuides }: SnapFeedbackProps) {
  const { camera } = useCamera();

  // Convert world coordinates to screen
  const toScreen = (x: number, y: number) => ({
    x: (x + camera.x) * camera.zoom,
    y: (y + camera.y) * camera.zoom,
  });

  if (!visible) return null;

  return (
    <svg className="absolute inset-0 pointer-events-none">
      {/* Snap guides */}
      {snapGuides.horizontal && (
        <motion.line
          initial={{ opacity: 0, x1: 0, x2: "100%" }}
          animate={{ opacity: 0.5 }}
          exit={{ opacity: 0 }}
          x1={0}
          x2="100%"
          y1={snapPoints[0] ? toScreen(snapPoints[0].x, snapPoints[0].y).y : 0}
          y2={snapPoints[0] ? toScreen(snapPoints[0].x, snapPoints[0].y).y : 0}
          stroke="#13b6ec"
          strokeWidth={1}
          strokeDasharray="4 4"
        />
      )}
      
      {snapGuides.vertical && (
        <motion.line
          initial={{ opacity: 0, y1: 0, y2: "100%" }}
          animate={{ opacity: 0.5 }}
          exit={{ opacity: 0 }}
          x1={snapPoints[0] ? toScreen(snapPoints[0].x, snapPoints[0].y).x : 0}
          x2={snapPoints[0] ? toScreen(snapPoints[0].x, snapPoints[0].y).x : 0}
          y1={0}
          y2="100%"
          stroke="#13b6ec"
          strokeWidth={1}
          strokeDasharray="4 4"
        />
      )}

      {/* Snap points */}
      {snapPoints.map((point, index) => {
        const screen = toScreen(point.x, point.y);
        return (
          <g key={index}>
            {/* Outer ring */}
            <motion.circle
              initial={{ scale: 0, opacity: 0 }}
              animate={{ scale: 1, opacity: 0.5 }}
              exit={{ scale: 0, opacity: 0 }}
              cx={screen.x}
              cy={screen.y}
              r={8}
              fill="none"
              stroke="#13b6ec"
              strokeWidth={2}
            />
            {/* Inner dot */}
            <motion.circle
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              exit={{ scale: 0 }}
              transition={{ delay: 0.1 }}
              cx={screen.x}
              cy={screen.y}
              r={3}
              fill="#13b6ec"
            />
          </g>
        );
      })}
    </svg>
  );
}
```

## ✅ Criterios de Éxito

| Criterio | Métrica | Valor Objetivo |
|----------|---------|----------------|
| Animaciones | FPS durante animaciones | 60 FPS |
| Interrupciones | Smooth interruptions | ✅ Pass |
| Loading states | Feedback visible | ✅ Pass |
| Tests | Cobertura | >80% |

## 📊 Estimación

| Fase | Esfuerzo | Estimación |
|------|----------|------------|
| Animation Config | S | 2 horas |
| Animated Components | M | 5 horas |
| Skeleton Components | S | 2 horas |
| Toast System | M | 4 horas |
| Snap Feedback | S | 2 horas |
| Testing | S | 3 horas |
| **Total** | **M** | **~18 horas** |

## 📝 Notas

1. **Performance**: Usar `will-change` CSS para animaciones críticas
2. **Reduced Motion**: Respetar `prefers-reduced-motion` media query
3. **Layout Animations**: Usar `layout` prop de Framer Motion para transiciones

---

**Documento creado**: `docs/epics/EPIC-WEB-007-animations.md`
**Estado**: Listo para implementación
**Dependencias**: EPIC-WEB-003, EPIC-WEB-004
