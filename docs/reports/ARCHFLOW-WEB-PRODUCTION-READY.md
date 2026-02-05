# ArchFlow Web - Plan de Producción Definitivo

**Fecha:** 2026-02-02
**Estado:** 🚧 EN DESARROLLO - REQUIERE ACCIÓN INMEDIATA
**Versión:** 1.0.0

---

## 📋 Resumen Ejecutivo

Este documento establece el plan definitivo para alcanzar **Production Ready 100%** en el proyecto ArchFlow Web. La estrategia se basa en un principio fundamental:

> **"La fuente de la verdad es el código Rust. El frontend TypeScript debe derivar automáticamente de ella."**

### Problema Actual

El proyecto tiene una arquitectura sólida pero suffer de **desincronización manual** entre el código Rust y el frontend TypeScript:

1. **Build roto** - 11 errores TypeScript impiden compilar
2. **Tipos desincronizados** - 114 líneas de diferencia entre WASM real y frontend
3. **Hardcodeos** - Datos mockeados en lugar de integración real
4. **Workflow manual** - Dependencias entre componentes gestionadas manualmente

### Solución Propuesta

Automatizar completamente el flujo de generación:
```
Rust Source → wasm-pack build → Auto-generate Types → Frontend TypeScript
```

---

## 🔍 Investigación: Herramientas Disponibles (2025)

### 1. Generación Automática de Tipos TypeScript

#### wasm-bindgen (YA EN USO)
- **Estado:** Implementado
- **Genera:** `.d.ts` files automáticamente
- **Limitación:** Tipos básicos, no soporta enums complejos o generics
- **Archivos generados:**
  - `archflow_web.d.ts` (API principal)
  - `archflow_web_bg.d.ts` (Bindings internos)

#### Tsify (RECOMENDADO - MEJORA)
- **Qué es:** Procedural macro que genera tipos TypeScript avanzados
- **Instalación:** `cargo add tsify --features derive`
- **Ventajas:**
  - Soporta enums con valores
  - Genera tipos para structs complejos
  - Compatible con serde
  - Types más precisos que wasm-bindgen nativo
- **Ejemplo de uso:**
```rust
#[derive(Tsify)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct EntityData {
    pub id: u32,
    pub position: (f64, f64),
    pub color: String,
}
```

#### gents (NUEVO - 2025)
- **Qué es:** CLI tool que genera TypeScript bindings desde Rust
- **Instalación:** `cargo install gents`
- **Ventajas:**
  - Genera archivos `.ts` completos, no solo `.d.ts`
  - Soporta generic types
  - Genera funciones helper
- **Útil para:** APIs complejas que necesitan wrapper functions

#### wasm-typescript-definition (LEGACY)
- **Qué es:** Crate para exportar structs/enums a TypeScript
- **Uso:** Con serde para serialización
- **Limitación:** Menos features que Tsify

**RECOMENDACIÓN:** Usar **Tsify** como mejora principal sobre wasm-bindgen nativo.

---

### 2. Automatización de Build (CI/CD)

#### wasm-pack-action (GitHub Actions)
```yaml
- name: Build WASM
  uses: wasm-pack/wasm-pack-action@v1
  with:
    args: build --target web --debug
```

#### justfile (YA IMPLEMENTADO)
El proyecto YA tiene un justfile completo con:
- `just build` - Build todo
- `just verify` - Verificar compilación
- `just types-copy` - Copiar tipos
- `just precommit` - Verificación completa

**PROBLEMA:** Estos scripts no se están ejecutando regularmente.

---

### 3. SharedArrayBuffer para Comunicación Rust↔JS

#### Configuración Requerida
```yaml
# Cargo.toml
[profile.release]
lto = true
opt-level = "z"
codegen-units = 1

# vite.config.ts
server:
  headers:
    Cross-Origin-Opener-Policy: "same-origin"
    Cross-Origin-Embedder-Policy: "require-corp"
```

#### Patrón de Comunicación (YA IMPLEMENTADO)
```rust
// Rust - InputRingBuffer
pub struct InputRingBuffer {
    ptr: *mut u8,
    capacity: usize,
}

#[wasm_bindgen]
impl WasmBridge {
    pub fn get_input_buffer_ptr(&self) -> *mut u8 { ... }
    pub fn push_input_event(&self, event: u32, x: f64, y: f64, buttons: u32, modifiers: u32) { ... }
}
```

```typescript
// TypeScript - useInput hook
const { bufferPtr } = useInputBuffer(bridge);
// Escribir directamente en SharedArrayBuffer
```

**ESTADO:** ✅ YA IMPLEMENTADO correctamente

---

### 4. Estructura de Crates Actual

```
crates/
├── archflow-web/              # WASM bindings principal
│   ├── src/
│   │   ├── lib.rs            # Exports principales
│   │   ├── bridge.rs         # WasmBridge
│   │   ├── engine.rs         # ArchFlowEngine
│   │   ├── input.rs          # InputProcessor
│   │   └── logic/            # Logic Bricks
│   └── pkg/                  # Output de wasm-pack
│       ├── archflow_web.js
│       ├── archflow_web_bg.wasm
│       └── archflow_web.d.ts
│
├── archflow-web-ui/           # Frontend React
│   ├── src/
│   │   ├── wasm/             # Copia de pkg/ (PROBLEMA)
│   │   ├── hooks/            # useArchFlowWasm, etc.
│   │   ├── components/       # Canvas, Toolbar, etc.
│   │   └── types/            # entity-schemas.ts
│   └── package.json
```

**PROBLEMA IDENTIFICADO:** `src/wasm/` es una copia manual de `crates/archflow-web/pkg/`

---

## 🚨 Análisis de Problemas

### Problema 1: Frontend NO Compila

```bash
$ cd crates/archflow-web-ui && npm run build

src/components/ErrorBoundary.tsx:10 - type-only imports faltantes
src/components/LazyComponents.tsx:19 - Type errors lazy loading
src/components/ToastContainer.tsx:10,97,... - Toast type no encontrado
src/hooks/useTransformation.ts:8,44 - useRef y startAngle sin usar
```

**Causa:** Errores acumulativos de TypeScript que nunca se corrigieron.

---

### Problema 2: Tipos WASM Desincronizados

```bash
# Comparación de líneas
crates/archflow-web/pkg/archflow_web.d.ts:      807 líneas (REAL)
crates/archflow-web-ui/src/wasm/archflow_web.d.ts: 693 líneas (DESACTUALIZADO)

Diferencia: 114 líneas (14%) desincronizadas
```

**Causa:** Copia manual de archivos sin automatización.

**Consecuencia:** El frontend espera métodos que no existen en el WASM real:
- Métodos que el frontend espera pero el WASM no tiene
- Métodos que el WASM tiene pero el frontend no sabe que existen

---

### Problema 3: Hardcodeos Identificados

| Archivo | Hardcodeo | Impacto |
|---------|-----------|---------|
| `useTransformation.ts:32` | `GRID_SIZE = 20` | Grid snapping no configurable desde Rust |
| `useEntityStore.ts:252` | `useEntityStoreMock()` | Fallback mock activo (deprecated pero presente) |
| `DemoArchitecture.tsx:40-180` | 11 entidades hardcodeadas | Demo sin integración real con WASM |

---

### Problema 4: Funcionalidades Incompletas

| Funcionalidad | Estado | Detalle |
|---------------|--------|---------|
| Rotación | ❌ | `startAngle` declarado pero nunca usado |
| Smart Routing | ⚠️ | Paths básicos, sin obstacle avoidance |
| Connection Points | ⚠️ | Store existe, pero no se renderiza |
| Tests integración | ❌ | 0 tests JS↔WASM |

---

## ✅ Lo Que SÍ Funciona

1. **Estructura del proyecto** - Excelente organización hexagonal
2. **WASM compilado** - `archflow_web_bg.wasm` (82KB) correcto
3. **SharedArrayBuffer** - Headers COOP/COEP configurados
4. **Justfile** - Automatización completa disponible
5. **UI Components** - Bonitos y bien estructurados
6. **Animaciones** - Framer Motion bien implementado

---

## 📋 Plan de Acción: Production Ready 100%

### FASE 1: Corregir Build (1-2 horas)

**Objetivo:** Que `npm run build` pase sin errores.

#### 1.1 Corregir type-only imports
```typescript
// ANTES (Error)
import { React, ErrorInfo, ReactNode } from "react";

// DESPUÉS (Correcto)
import type { ErrorInfo, ReactNode } from "react";
import { Component } from "react";
```

#### 1.2 Corregir LazyComponents
```typescript
// ANTES (Error)
const C4ArchitectureDemo = lazy(() => 
  import("@demos/C4ArchitectureDemo")
);

// DESPUÉS (Correcto)
const C4ArchitectureDemo = lazy(() => 
  import("@demos/C4ArchitectureDemo").then(module => ({ 
    default: module.C4ArchitectureDemo 
  }))
);
```

#### 1.3 Eliminar código muerto
```typescript
// useTransformation.ts - Eliminar startAngle
// Quejarse si no se usa, no declarar variables sin usar
```

#### 1.4 Importar Toast type
```typescript
// ToastContainer.tsx - Importar el tipo Toast
import type { Toast } from "@store/useToastStore";
```

**Verificación:**
```bash
cd crates/archflow-web-ui
npm run build  # Debe pasar sin errores
```

---

### FASE 2: Sincronización Automática de Tipos (30 min)

**Objetivo:** Eliminar la copia manual de tipos.

#### 2.1 Opción A: Script de sincronización (RECOMENDADA)

Crear script `scripts/sync-wasm-types.sh`:
```bash
#!/bin/bash
# scripts/sync-wasm-types.sh

set -e

SRC="crates/archflow-web/pkg"
DEST="crates/archflow-web-ui/src/wasm"

echo "🔄 Sincronizando tipos WASM..."

# Copiar archivos de tipos
cp "$SRC/archflow_web.d.ts" "$DEST/"
cp "$SRC/archflow_web_bg.d.ts" "$DEST/"
cp "$SRC/archflow_web_bg.wasm.d.ts" "$DEST/"

# Copiar JS bindings
cp "$SRC/archflow_web.js" "$DEST/"
cp "$SRC/archflow_web_bg.js" "$DEST/"

# Copiar WASM (solo si cambió)
cp "$SRC/archflow_web_bg.wasm" "$DEST/"

echo "✅ Tipos sincronizados"
```

Hacer ejecutable y añadir al justfile:
```makefile
# justfile
sync-wasm-types:
    @./scripts/sync-wasm-types.sh
```

#### 2.2 Opción B: Integrar en wasm-pack build

Modificar justfile:
```makefile
build-wasm:
    @echo "Building WASM..."
    @cd crates/archflow-web && wasm-pack build --target web --debug
    @echo "Sincronizando tipos..."
    @./scripts/sync-wasm-types.sh
    @echo "WASM built!"
```

#### 2.3 Verificación
```bash
# Comparar que los archivos son idénticos
diff crates/archflow-web/pkg/archflow_web.d.ts \
     crates/archflow-web-ui/src/wasm/archflow_web.d.ts
# Debe dar 0 diferencias
```

---

### FASE 3: Integración Real con WASM (4-6 horas)

**Objetivo:** Eliminar todos los hardcodeos y mocks.

#### 3.1 Eliminar Mock de EntityStore

**PASO 1:** Verificar que `useEntityStore()` funciona correctamente
```typescript
// Verificar en useEntityStore.ts que no se usa useEntityStoreMock
// Si se usa, el WASM no está cargado correctamente
```

**PASO 2:** Manejar error gracefully
```typescript
// useEntityStore.ts
export function useEntityStore(): EntityStoreReturn {
  const { bridge, isLoaded, isInitialized } = useArchFlowWasm();
  
  if (!isLoaded || !isInitialized || !bridge) {
    throw new Error(
      "WASM bridge no cargado. Ejecuta 'just build-wasm' primero."
    );
  }
  
  // ... resto del código real
}
```

#### 3.2 Hacer Grid Size Configurable

**PASO 1:** Exportar desde Rust
```rust
// archflow-web/src/config.rs
#[wasm_bindgen]
pub struct Config {
    grid_size: u32,
}

#[wasm_bindgen]
impl WasmBridge {
    pub fn get_grid_size(&self) -> u32 {
        CONFIG.grid_size
    }
    
    pub fn set_grid_size(&mut self, size: u32) {
        CONFIG.grid_size = size;
    }
}
```

**PASO 2:** Usar en TypeScript
```typescript
// useTransformation.ts
const { bridge } = useArchFlowWasm();
const GRID_SIZE = bridge?.get_grid_size() ?? 20;
```

#### 3.3 Integrar Demo con WASM Real

**PASO 1:** Modificar DemoArchitecture para usar datos reales
```typescript
// DemoArchitecture.tsx
export function DemoArchitecture({ ... }) {
  const { entities } = useEntityStore();
  const demoEntities = useMemo(() => {
    return Array.from(entities.values()).slice(0, 11);
  }, [entities]);
  
  // Si no hay entidades, mostrar datos de ejemplo
  const displayEntities = demoEntities.length > 0 
    ? demoEntities 
    : DEFAULT_DEMO_ENTITIES;
}
```

#### 3.4 Implementar Rotación Completa

```typescript
// useTransformation.ts
const startTransform = useCallback((
  mode: TransformMode,
  entityId: EntityId,
  startPos: Vec2,
  startAngle?: number  // AGREGAR parámetro
) => {
  setStartAngle(startAngle ?? 0);
  // ... resto
});
```

---

### FASE 4: Mejora de Tipos con Tsify (2-3 horas)

**Objetivo:** Tipos TypeScript más precisos derivados de Rust.

#### 4.1 Instalar Tsify

```bash
cd crates/archflow-web
cargo add tsify --features derive,serde-serialize
```

#### 4.2 Aplicar a structs principales

```rust
// archflow-web/src/entity.rs
use tsify::Tsify;
use serde::{Serialize, Deserialize};

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct EntityData {
    pub id: u32,
    pub position: Position,
    pub size: Size,
    pub color: String,
    pub label: String,
    pub is_visible: bool,
    pub is_selected: bool,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(from_wasm_abi, into_wasm_abi)]
pub struct Size {
    pub w: f64,
    pub h: f64,
}
```

#### 4.3 Regenerar y sincronizar
```bash
just build-wasm
just sync-wasm-types
```

---

### FASE 5: Tests de Integración (2-3 horas)

**Objetivo:** Verificar que JS↔WASM funciona correctamente.

#### 5.1 Crear test de integración

```typescript
// src/test/wasm-integration.test.ts
import { describe, it, expect, beforeAll } from "vitest";
import { render, screen } from "@testing-library/react";
import { renderHook, act } from "@testing-library/react";
import { useArchFlowWasm } from "@hooks/useArchFlowWasm";

describe("WASM Integration", () => {
  it("should load WASM module", async () => {
    const { result } = renderHook(() => useArchFlowWasm());
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 1000));
    });
    
    expect(result.current.isLoaded).toBe(true);
    expect(result.current.error).toBeNull();
  });
  
  it("should spawn entity via WASM", async () => {
    const { result } = renderHook(() => useArchFlowWasm());
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 1000));
    });
    
    if (result.current.isLoaded && result.current.bridge) {
      const entityId = result.current.bridge.spawn_entity(100, 100, 120, 80);
      expect(entityId).toBeGreaterThan(0);
    }
  });
});
```

#### 5.2 Añadir a package.json

```json
{
  "scripts": {
    "test": "vitest run",
    "test:integration": "vitest run --config vitest.integration.config.ts"
  }
}
```

---

### FASE 6: CI/CD Automatizado (1-2 horas)

**Objetivo:** Automatizar todo el flujo en GitHub Actions.

#### 6.1 Crear workflow `.github/workflows/ci.yml`

```yaml
name: CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: crates/archflow-web-ui/package-lock.json
      
      - name: Install wasm-pack
        uses: wasm-pack/wasm-pack-action@v1
        with:
          version: latest
      
      - name: Install just (command runner)
        run: cargo install just
      
      - name: Build WASM
        run: just build-wasm
      
      - name: Sync WASM types
        run: just sync-wasm-types
      
      - name: Install frontend deps
        run: cd crates/archflow-web-ui && npm ci
      
      - name: Build frontend
        run: cd crates/archflow-web-ui && npm run build
      
      - name: Run tests
        run: cd crates/archflow-web-ui && npm test
      
      - name: Run linter
        run: cd crates/archflow-web-ui && npm run lint
```

#### 6.2 Añadir verificación de tipos al PR

```yaml
# Añadir al workflow anterior, después de "Build frontend"
- name: Verify WASM types sync
  run: |
    echo "Verificando sincronización de tipos..."
    diff crates/archflow-web/pkg/archflow_web.d.ts \
         crates/archflow-web-ui/src/wasm/archflow_web.d.ts
    echo "✅ Tipos sincronizados"
```

---

## 📊 Estimación de Tiempo

| Fase | Tiempo | Dependencias |
|------|--------|--------------|
| FASE 1: Corregir build | 1-2h | Ninguna |
| FASE 2: Sincronización | 30min | FASE 1 |
| FASE 3: Integración WASM | 4-6h | FASE 2 |
| FASE 4: Tsify | 2-3h | FASE 3 |
| FASE 5: Tests | 2-3h | FASE 4 |
| FASE 6: CI/CD | 1-2h | FASE 5 |

**Total estimado: 11-16 horas de trabajo enfocado**

---

## 🎯 Checklist de Producción

### Pre-Launch Checklist

- [ ] `npm run build` pasa sin errores
- [ ] `just build` compila todo correctamente
- [ ] Tipos WASM sincronizados (0 diferencias)
- [ ] `useEntityStoreMock()` eliminado
- [ ] Grid size configurable desde Rust
- [ ] Demo usa datos reales de WASM
- [ ] Rotación implementada completamente
- [ ] Tests de integración pasan
- [ ] GitHub Actions CI/CD verde
- [ ] Lighthouse performance > 90
- [ ] Bundle size < 500KB gzipped

### Commands de Verificación

```bash
# Verificación rápida (pre-commit)
just precommit

# Verificación completa
just verify
npm run build
npm test

# Verificar tipos
diff crates/archflow-web/pkg/archflow_web.d.ts \
     crates/archflow-web-ui/src/wasm/archflow_web.d.ts
# Debe dar: "Files are identical" o no dar output
```

---

## 🔄 Flujo de Desarrollo Recomendado

### Daily Development

```bash
# 1. Antes de empezar: verificar estado
just status

# 2. Hacer cambios en Rust
# ... edits ...

# 3. Build y sync automático
just build-wasm    # Build WASM + sincroniza tipos
cd ../archflow-web-ui && npm run build  # Verificar frontend
```

### Pre-Commit

```bash
just precommit  # fmt + verify + test-sdk
```

### Release

```bash
just build-release  # Build todo en release
# Verificar que todo funciona
# Tag y push
```

---

## 📚 Referencias

### Documentación Oficial

- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [Tsify Crate](https://docs.rs/crate/tsify/latest)
- [SharedArrayBuffer MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)

### Herramientas Mentionadas

- **wasm-pack-action**: GitHub Actions para wasm-pack
- **Tsify**: Generación de tipos TypeScript desde Rust
- **gents**: CLI para generar TypeScript bindings
- **just**: Command runner (ya instalado en el proyecto)

---

## 📝 Notas

1. **Orden de implementación:** Las fases deben seguirse en orden. Cada fase depende de la anterior.

2. **Tip:** Después de cada fase, ejecutar `just verify` para confirmar que no se rompió nada.

3. **Tip:** Crear script de rollback por si algo sale mal:
```bash
#!/bin/bash
# scripts/rollback.sh
git checkout HEAD -- crates/archflow-web-ui/src/wasm/
```

---

**Documento creado:** `docs/reports/ARCHFLOW-WEB-PRODUCTION-READY.md`
**Próximo paso:** Ejecutar FASE 1 (Corregir build)
