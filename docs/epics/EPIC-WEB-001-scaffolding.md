---
title: "ÉPICA-WEB-001: Scaffolding y Fundamentos"
author: Claude Code
date: 2026-02-02
status: Completada
version: 1.1.0
priority: P0
effort: L
depends_on: []
---

# ÉPICA-WEB-001: Scaffolding y Fundamentos ✅

## 📋 Resumen Ejecutivo

Configurar el entorno de desarrollo completo para la aplicación Web Whiteboard, incluyendo la actualización de dependencias, estructura de directorios, y herramientas de desarrollo. **COMPLETADA - Production Ready**.

## 🎯 Objetivos Cumplidos

- ✅ Actualizar Tailwind CSS a v4.x con configuración CSS-first
- ✅ Configurar utilidades de desarrollo (clsx, tailwind-merge)
- ✅ Configurar Zustand para gestión de estado global
- ✅ Configurar Framer Motion para animaciones
- ✅ Crear estructura de directorios modular
- ✅ Configurar aliases de imports en TypeScript
- ✅ Configurar Vitest para testing

## 🎯 Objetivos

- Actualizar Tailwind CSS a v4.x con configuración CSS-first
- Configurar utilidades de desarrollo (clsx, tailwind-merge)
- Configurar Zustand para gestión de estado global
- Configurar Framer Motion para animaciones
- Crear estructura de directorios modular
- Configurar aliases de imports en TypeScript
- Configurar Vitest para testing

## 📦 Dependencias npm

```json
{
  "dependencies": {
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "clsx": "^3.0.0",
    "tailwind-merge": "^3.0.0",
    "framer-motion": "^12.0.0",
    "zustand": "^5.0.0",
    "lucide-react": "^7.0.0"
  },
  "devDependencies": {
    "tailwindcss": "^4.0.0",
    "@tailwindcss/vite": "^4.0.0",
    "vite": "^7.2.4",
    "typescript": "~5.9.3",
    "vitest": "^3.0.0",
    "@testing-library/react": "^16.0.0"
  }
}
```

## 📁 Estructura de Directorios

```
crates/archflow-web-ui/src/
├── components/          # Componentes React
│   ├── Canvas/         # Wrapper WebGPU
│   ├── Toolbar/        # Herramientas
│   ├── Header/         # Barra superior
│   ├── Sidebar/        # Panel lateral
│   ├── Properties/     # Editor de propiedades
│   ├── StatusBar/      # Footer
│   └── common/         # Componentes compartidos
├── hooks/              # Hooks personalizados
│   ├── useArchFlowWasm.ts
│   ├── useCamera.ts
│   ├── useSelection.ts
│   └── useKeyboard.ts
├── store/              # Zustand stores
│   ├── useCanvasStore.ts
│   ├── useSelectionStore.ts
│   └── useUIStore.ts
├── utils/              # Utilidades
│   ├── cn.ts
│   ├── geometry.ts
│   └── constants.ts
├── types/              # Tipos TypeScript
│   ├── entity.ts
│   ├── input.ts
│   └── wasm.ts
├── styles/             # Estilos globales
│   └── index.css
├── App.tsx             # Componente principal
└── main.tsx            # Entry point
```

## 🔧 Tareas

### 1.1 Actualizar Tailwind CSS a v4.x

**Objetivo**: Migrar de Tailwind v3 a v4 con configuración CSS-first.

**Pasos**:
1. Instalar `@tailwindcss/vite` y actualizar `tailwindcss` a v4
2. Convertir `tailwind.config.js` a CSS con `@theme`
3. Usar `@import "tailwindcss"` en lugar de directivas `@tailwind`
4. Migrar `@utility` para utilidades personalizadas
5. Eliminar PostCSS y autoprefixer (v4 no los necesita)

**Archivo de referencia**: `src/index.css`

```css
@import "tailwindcss";

@theme {
  --color-primary: #13b6ec;
  --color-background-dark: #101d22;
  --color-surface-dark: #1a2c32;
  /* ... más variables */
}

@utility dot-grid {
  background-image: radial-gradient(#cbd5e1 1px, transparent 1px);
  background-size: 20px 20px;
}
```

### 1.2 Configurar clsx y tailwind-merge

**Objetivo**: Crear utilitario `cn()` para combinar clases condicionalmente.

**Archivo**: `src/utils/cn.ts`

```typescript
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

**Uso**:
```tsx
import { cn } from "@utils/cn";

<div className={cn(
  "base-class",
  isActive && "active-class",
  variant === "primary" && "primary-variant"
)} />
```

### 1.3 Configurar Zustand Store Base

**Objetivo**: Crear stores modulares para estado global.

**Archivos**:
- `src/store/useUIStore.ts` - Estado de UI (theme, panels, modals)
- `src/store/useCanvasStore.ts` - Estado del canvas (zoom, pan, grid)
- `src/store/useSelectionStore.ts` - Estado de selección

**Ejemplo - useUIStore.ts**:
```typescript
import { create } from "zustand";

interface UIState {
  theme: "light" | "dark";
  isSidebarOpen: boolean;
  isPropertiesPanelOpen: boolean;
  activeTool: "select" | "pan" | "rectangle" | "connection";
  setTheme: (theme: "light" | "dark") => void;
  toggleSidebar: () => void;
  setActiveTool: (tool: UIState["activeTool"]) => void;
}

export const useUIStore = create<UIState>((set) => ({
  theme: "dark",
  isSidebarOpen: true,
  isPropertiesPanelOpen: true,
  activeTool: "select",
  setTheme: (theme) => set({ theme }),
  toggleSidebar: () => set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),
  setActiveTool: (tool) => set({ activeTool: tool }),
}));
```

### 1.4 Configurar Framer Motion

**Objetivo**: Configurar Framer Motion globalmente para animaciones fluidas.

**Archivo**: `src/hooks/useAnimationConfig.ts`

```typescript
import { MotionConfig } from "framer-motion";

// Configuración global de animaciones
export const animationConfig = {
  transition: {
    type: "spring",
    stiffness: 300,
    damping: 30,
  },
  layout: {
    transition: { duration: 0.2 },
  },
};
```

### 1.5 Configurar Path Aliases

**Objetivo**: Simplificar imports con aliases.

**Archivo**: `vite.config.ts`

```typescript
resolve: {
  alias: {
    "@components": "./src/components",
    "@hooks": "./src/hooks",
    "@utils": "./src/utils",
    "@types": "./src/types",
    "@store": "./src/store",
    "@archflow/web": "../../archflow-web/pkg",
  },
}
```

**Archivo**: `tsconfig.json` (extendido)

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@components/*": ["src/components/*"],
      "@hooks/*": ["src/hooks/*"],
      "@utils/*": ["src/utils/*"],
      "@types/*": ["src/types/*"],
      "@store/*": ["src/store/*"]
    }
  }
}
```

### 1.6 Configurar Vitest

**Objetivo**: Configurar testing unitario con Vitest.

**Archivo**: `vite.config.ts` (actualizado)

```typescript
/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
```

**Archivo**: `src/test/setup.ts`

```typescript
import "@testing-library/jest-dom";
import { beforeEach } from "vitest";

beforeEach(() => {
  // Reset Zustand stores between tests
  // Clean up any timers or subscriptions
});
```

## ✅ Criterios de Éxito

| Criterio | Métrica | Valor Objetivo |
|----------|---------|----------------|
| npm install | Sin warnings de dependencias | ✅ Pass |
| TypeScript | Errores de compilación | 0 errores |
| Vite dev server | Hot Module Reload | <100ms |
| Tests | Tests unitarios pasando | 100% pass |
| Bundle size | gzipped | <200KB |
| Tailwind v4 | CSS-first config funcionando | ✅ Pass |
| Path aliases | Imports funcionando | ✅ Pass |

## 📊 Estimación

| Fase | Esfuerzo | Estimación |
|------|----------|------------|
| Tailwind v4 | M | 2 horas |
| clsx/tailwind-merge | S | 1 hora |
| Zustand stores | M | 3 horas |
| Framer Motion | S | 1 hora |
| Path aliases | S | 30 min |
| Vitest setup | M | 2 horas |
| **Total** | **L** | **~10 horas** |

## 🔗 Referencias

- [Tailwind CSS v4 Docs](https://tailwindcss.com/docs/upgrade-guide)
- [Zustand Docs](https://zustand.docs.pmnd.rs/)
- [Framer Motion Docs](https://www.framer.com/motion/)
- [Vitest Docs](https://vitest.dev/)

## 📝 Notas

1. **Importante**: Tailwind v4 usa `@import "tailwindcss"` en lugar de `@tailwind base/components/utilities`
2. **Zustand**: Usar `create<T>()` con inferencia de tipos automática
3. **Framer Motion**: Configurar `MotionConfig` en el root para consistencia
4. **Vitest**: Necesario `@testing-library/jest-dom` para assertions de DOM

---

**Documento creado**: `docs/epics/EPIC-WEB-001-scaffolding.md`
**Estado**: Listo para implementación
