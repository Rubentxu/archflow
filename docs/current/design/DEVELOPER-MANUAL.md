# Manual de Usuario Developer - ArchFlow Web MVP

## 📖 Índice

1. [Introducción](#introducción)
2. [Requisitos del Sistema](#requisitos-del-sistema)
3. [Instalación](#instalación)
4. [Estructura del Proyecto](#estructura-del-proyecto)
5. [Desarrollo](#desarrollo)
6. [Testing](#testing)
7. [Build y Deployment](#build-y-deployment)
8. [Integración SDK](#integración-sdk)
9. [Solución de Problemas](#solución-de-problemas)

---

## 1. Introducción

ArchFlow Web MVP es una aplicación de diagramación profesional construida en Rust con WebAssembly. Permite crear diagramas arquitectónicos (C4 Model, UML, Flowchart) con herramientas profesionales de edición.

### Características Principales

- 🎨 **Librerías de Componentes**: General, Flowchart, UML, C4 Model
- 📐 **Herramientas de Alineación**: Alineación y distribución de shapes
- 🔒 **Gestión de Capas**: Reordenar, visibility, lock
- ⌨️ **Atajos de Teclado**: Productividad optimizada
- 📱 **Responsive Design**: Tablet y Mobile
- 🧪 **Testing Completo**: Unitarios y E2E

---

## 2. Requisitos del Sistema

### Requisitos Mínimos

| Componente | Requisito |
|------------|-----------|
| **Rust** | 1.70.0 o superior |
| **Node.js** | 18.0.0 o superior |
| **npm** | 9.0.0 o superior |
| **Sistema** | Linux, macOS, o Windows |

### Verificar Instalación

```bash
# Verificar Rust
rustc --version
# Debe mostrar: rustc 1.70.0+

# Verificar Cargo
cargo --version
# Debe mostrar: cargo 1.70.0+

# Verificar Node.js
node --version
# Debe mostrar: v18.x.x o superior

# Verificar npm
npm --version
# Debe mostrar: 9.x.x o superior
```

---

## 3. Instalación

### Clonar el Repositorio

```bash
# Clonar el repositorio
git clone https://github.com/archflow/archflow.git
cd archflow

# Cambiar a la versión estable
git checkout v0.24.0
```

### Instalar Dependencias Rust

```bash
# Instalar wasm-pack (necesario para compilar WASM)
cargo install wasm-pack

# Verificar instalación
wasm-pack --version
```

### Instalar Dependencias npm

```bash
# Instalar dependencias para archflow-web
cd crates/archflow-web
npm install

# Instalar Playwright para E2E tests
npx playwright install --with-deps chromium
```

### Compilar el Proyecto

```bash
# Compilar todo el workspace
cargo build --workspace

# Compilar solo archflow-sdk (necesario para WASM)
cargo build -p archflow-sdk

# Compilar archflow-web
cargo build -p archflow-web
```

---

## 4. Estructura del Proyecto

```
archflow/
├── Cargo.toml                    # Workspace configuration
├── crates/
│   ├── archflow-core/           # Core domain models
│   ├── archflow-geometry/       # Geometry calculations
│   ├── archflow-spatial/        # Spatial indexing
│   ├── archflow-primitives/     # Reusable primitives
│   ├── archflow-records/        # CRDT records
│   ├── archflow-collab/         # Collaboration
│   ├── archflow-renderers/      # Rendering system
│   ├── archflow-workspace/      # Workspace management
│   ├── archflow-sdk/            # ⭐ SDK principal
│   │   ├── src/
│   │   │   ├── library/         # Component libraries
│   │   │   ├── layers/          # Layer management
│   │   │   ├── group/           # Group operations
│   │   │   ├── properties/      # Properties panel
│   │   │   ├── alignment/       # Alignment tools
│   │   │   ├── keyboard/        # Keyboard handling
│   │   │   ├── selection/       # Selection system
│   │   │   ├── tools/           # Drawing tools
│   │   │   ├── text/            # Text handling
│   │   │   └── wasm/            # WASM bindings
│   │   │       ├── library.rs
│   │   │       ├── layers.rs
│   │   │       ├── group.rs
│   │   │       ├── properties.rs
│   │   │       ├── alignment.rs
│   │   │       └── keyboard.rs
│   │   └── Cargo.toml
│   └── archflow-web/            # ⭐ Aplicación web
│       ├── src/
│       │   └── lib.rs           # WASM entry point
│       ├── styles/
│       │   ├── main.css         # Design tokens
│       │   ├── responsive.css   # Media queries
│       │   └── components/      # UI components
│       ├── tests/
│       │   └── e2e.spec.ts      # E2E tests
│       ├── index.html
│       ├── app.js
│       └── package.json
├── packages/
│   ├── sdk/                     # TypeScript SDK
│   └── archflow-sdk-types/     # Generated types
└── docs/
    └── current/design/          # Documentación
```

---

## 5. Desarrollo

### Iniciar Servidor de Desarrollo

```bash
# Opción 1: Servir con Python (recomendado para desarrollo rápido)
cd crates/archflow-web
python3 -m http.server 8080

# Opción 2: Con Vite (hot reload)
cd crates/archflow-web
npm run dev
```

### Acceder a la Aplicación

```bash
# Abrir en navegador
# ⭐ Asegurarse de que el servidor WASM esté compilado primero
firefox http://localhost:8080
# o
google-chrome http://localhost:8080
```

### Desarrollo con Hot Reload

```bash
# Usar Vite para desarrollo con hot reload
cd crates/archflow-web

# Instalar Vite si no está instalado
npm install

# Iniciar servidor de desarrollo
npm run dev

# El servidor estará en http://localhost:5173
```

### Compilar WASM para Desarrollo

```bash
# Compilar WASM con debug symbols
cd crates/archflow-sdk
wasm-pack build --target web --debug

# Compilar WASM optimizado
cd crates/archflow-sdk
wasm-pack build --target web --release
```

### Verificar Cambios en el SDK

```bash
# Recompilar SDK
cargo build -p archflow-sdk

# Ejecutar tests del SDK
cargo test -p archflow-sdk

# Verificar que no hay warnings de compilación
cargo check -p archflow-sdk
```

---

## 6. Testing

### Tests Unitarios (Rust)

```bash
# Ejecutar todos los tests del workspace
cargo test --workspace

# Tests de un crate específico
cargo test -p archflow-sdk
cargo test -p archflow-core
cargo test -p archflow-geometry

# Tests con output detallado
cargo test -p archflow-sdk -- --nocapture

# Tests ignorados
cargo test --workspace -- --ignored
```

### Tests WASM

```bash
# Compilar y ejecutar tests WASM
cargo test -p archflow-sdk --features wasm

# Tests específicos del módulo layers
cargo test -p archflow-sdk -- layers
```

### Tests E2E con Playwright

```bash
# Instalar Playwright
cd crates/archflow-web
npm install
npx playwright install --with-deps chromium

# Ejecutar tests E2E
npm test

# Tests en modo headed (con navegador visible)
npm run test:headed

# Ver reporte de tests
npm run test:report
```

### Coverage de Tests

```bash
# Instalar cargo-tarpaulin para coverage
cargo install cargo-tarpaulin

# Generar reporte de coverage
cargo tarpaulin --workspace --out Html

# Ver coverage del SDK
cargo tarpaulin -p archflow-sdk --out Html
```

---

## 7. Build y Deployment

### Build de Producción

```bash
# Compilar todo el workspace en release
cargo build --workspace --release

# Compilar archflow-web solo
cargo build -p archflow-web --release

# Generar WASM optimizado
cd crates/archflow-sdk
wasm-pack build --target web --release
```

### Generar TypeScript Types

```bash
# Los tipos se generan automáticamente con ts-rs
cargo build -p archflow-sdk

# Verificar tipos generados
cat packages/sdk/src/generated/index.ts
```

### Build para Diferentes Plataformas

```bash
# Linux
cargo build --release

# macOS (requiere macOS)
cargo build --release --target x86_64-apple-darwin

# Windows (requiere Windows)
cargo build --release --target x86_64-pc-windows-gnu
```

### Deployment en Producción

```bash
# 1. Compilar en release
cargo build --workspace --release

# 2. Generar WASM
cd crates/archflow-sdk
wasm-pack build --target web --release

# 3. Construir frontend (si usa Vite)
cd crates/archflow-web
npm run build

# 4. Los archivos estarán en:
# - pkg/ (WASM)
# - dist/ (Frontendbuildado)
```

---

## 8. Integración SDK

### Uso desde JavaScript

```javascript
// Importar SDK
import init, {
  ArchFlowEditor,
  JsLibraryManager,
  JsLayerManager,
  JsPropertiesManager,
  JsAlignmentManager,
  JsGroupManager
} from './pkg/archflow_web.js';

async function main() {
  // Inicializar WASM
  await init();
  
  // Crear editor
  const editor = new ArchFlowEditor(800, 600);
  
  // Usar Library Manager
  const libraryManager = new JsLibraryManager();
  const libraries = libraryManager.get_libraries();
  
  // Usar Layer Manager
  const layerManager = new JsLayerManager();
  layerManager.create_layer('Context', 'My Layer');
  
  console.log('ArchFlow initialized!');
}
```

### API del Editor

```javascript
// Viewport Operations
editor.pan(dx, dy);           // Desplazar viewport
editor.zoomAt(x, y, factor);  // Zoom en punto
editor.zoom_to_fit();          // Ajustar al contenido

// Shape Operations
editor.create_rectangle(x, y, width, height);  // Crear rectángulo
editor.create_ellipse(x, y, radiusX, radiusY); // Crear elipse
editor.create_line(x1, y1, x2, y2);            // Crear línea
editor.get_shape(id);                           // Obtener shape
editor.update_shape(id, changes);               // Actualizar shape
editor.delete_shape(id);                        // Eliminar shape

// Selection
editor.select(id);              // Seleccionar shape
editor.select_multiple(ids);    // Selección múltiple
editor.clear_selection();       // Limpiar selección
editor.get_selection();         // Obtener selección
```

### Uso de Libraries

```javascript
const libraryManager = new JsLibraryManager();

// Obtener todas las librerías
const libraries = libraryManager.get_libraries();
// Retorna: [{id: "general", name: "General", ...}, ...]

// Buscar items
const results = libraryManager.search_items("rect");
// Retorna: [(libraryId, LibraryItem), ...]

// Obtener item específico
const item = libraryManager.get_item("general", "rect");

// Obtener datos del componente para instanciar
const data = libraryManager.get_component_data("general", "rect");
```

### Uso de Layers

```javascript
const layerManager = new JsLayerManager();

// Crear capa
const layerId = layerManager.create_layer("Context", "Mi Capa");

// Gestionar visibilidad
layerManager.set_layer_visibility(layerId, false); // Ocultar
layerManager.set_layer_visibility(layerId, true);  // Mostrar

// Gestionar lock
layerManager.set_layer_locked(layerId, true);   // Bloquear
layerManager.set_layer_locked(layerId, false);  // Desbloquear

// Reordenar capas
layerManager.move_layer_up(layerId);
layerManager.move_layer_down(layerId);
layerManager.move_layer_to_top(layerId);
layerManager.move_layer_to_bottom(layerId);

// Obtener capas ordenadas
const layers = layerManager.get_layers_in_order();
```

### Uso de Alignment

```javascript
const alignmentManager = new JsAlignmentManager();

// Alinear shapes
alignmentManager.align_left(shapeIds);      // Izquierda
alignmentManager.align_center(shapeIds);    // Centro
alignmentManager.align_right(shapeIds);     // Derecha
alignmentManager.align_top(shapeIds);       // Arriba
alignmentManager.align_middle(shapeIds);    // Medio
alignmentManager.align_bottom(shapeIds);    // Abajo

// Distribuir shapes
alignmentManager.distribute_horizontally(shapeIds);
alignmentManager.distribute_vertically(shapeIds);
```

### Uso de Group

```javascript
const groupManager = new JsGroupManager();

// Agrupar shapes
const groupId = groupManager.group(shapeIds);

// Desagrupar
groupManager.ungroup(groupId);
groupManager.ungroup_shape(shapeId);

// Gestionar locks
groupManager.lock_group(groupId);
groupManager.unlock_group(groupId);
groupManager.is_group_locked(groupId);

// Información de grupos
const shapes = groupManager.get_group_shapes(groupId);
const allGroups = groupManager.get_all_groups();
```

---

## 9. Solución de Problemas

### Problema: WASM no carga

```bash
# Verificar que wasm-pack está instalado
wasm-pack --version

# Recompilar WASM
cd crates/archflow-sdk
wasm-pack build --target web --debug

# Verificar que pkg/ existe
ls -la crates/archflow-sdk/pkg/
```

### Problema: Errores de compilación

```bash
# Limpiar cache de Cargo
cargo clean

# Verificar dependencias
cargo update

# Compilar con output detallado
cargo build -p archflow-sdk -vv
```

### Problema: Tests fallan

```bash
# Verificar que no hay cambios sin commitear
git status

# Actualizar dependencias
cargo update

# Reconstruir
cargo build --workspace

# Ejecutar tests específicos
cargo test -p archflow-sdk -- --nocapture
```

### Problema: Servidor no responde

```bash
# Verificar que el puerto no está ocupado
lsof -i :8080

# Usar otro puerto
python3 -m http.server 8081

# Verificar que el servidor está ejecutándose
curl http://localhost:8080
```

### Problema: Playwright no instala

```bash
# Instalar dependencias del sistema
npx playwright install-deps

# Instalar solo chromium
npx playwright install chromium

# Verificar instalación
npx playwright --version
```

### Problema: Estilos no cargan

```bash
# Verificar que main.css está siendo importado
# En index.html debe haber:
# <link rel="stylesheet" href="styles/main.css">

# Verificar rutas de CSS
ls -la crates/archflow-web/styles/
```

### Problema: Icons no aparecen

```bash
# Verificar que Phosphor Icons está cargado
# En index.html debe haber:
# <script src="https://unpkg.com/@phosphor-icons/web"></script>

# Verificar que los iconos existen
firefox http://localhost:8080
# Abrir Developer Tools > Console > Ver errores
```

### Verificar Salud del Proyecto

```bash
# 1. Verificar compilación
cargo check --workspace

# 2. Ejecutar tests
cargo test --workspace

# 3. Verificar WASM
cd crates/archflow-sdk
wasm-pack build --target web

# 4. Verificar tipos TypeScript
cat packages/sdk/src/generated/index.ts | head -20
```

---

## Atajos de Teclado

| Atajo | Acción |
|-------|--------|
| `V` | Herramienta Select |
| `R` | Herramienta Rectangle |
| `O` | Herramienta Ellipse |
| `L` | Herramienta Line |
| `T` | Herramienta Text |
| `H` | Herramienta Hand |
| `Z` | Herramienta Zoom |
| `Delete` | Eliminar selección |
| `Ctrl+C` | Copiar |
| `Ctrl+V` | Pegar |
| `Ctrl+Z` | Deshacer |
| `Ctrl+Shift+Z` | Rehacer |
| `Arrow Keys` | Nudge selection |
| `Shift+Arrow Keys` | Nudge preciso |
| `Click` | Seleccionar |
| `Shift+Click` | Selección múltiple |
| `Right Click` | Context menu |
| `Drag` | Mover shape |
| `Scroll` | Zoom |

---

## Recursos Adicionales

### Documentación
- [Design Spec](./ARCHFLOW-WEB-DESIGN-SPEC.md)
- [Component Library](./COMPONENT-LIBRARY-SPEC.md)
- [Icon Libraries](./ICON-LIBRARIES-GUIDE.md)

### Comandos Rápidos

```bash
# Desarrollo completo
git pull
cargo build --workspace
wasm-pack build --target web --debug
cd ../archflow-web && python3 -m http.server 8080

# Tests completos
cargo test --workspace
cd crates/archflow-web && npm test

# Build producción
cargo build --workspace --release
wasm-pack build --target web --release
```

---

*Última actualización: Enero 2025*
*ArchFlow Team - v0.24.0*
