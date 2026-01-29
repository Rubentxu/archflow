# Plan de Migración: demo-web → archflow-web

## 📋 Resumen

Migrar el proyecto `demo-web` actual a `archflow-web` con una interfaz profesional tipo Figma/tldraw basada en la especificación de diseño.

---

## 🗂️ Estructura de Carpetas Nueva

```
crates/
├── archflow-sdk/           # SDK existente
├── archflow-core/          # Core existente
└── archflow-web/           # NUEVO: Web app profesional (antes demo-web)
    ├── Cargo.toml
    ├── index.html
    ├── src/
    │   ├── lib.rs
    │   ├── main.rs          # Punto de entrada para wasm-pack
    │   ├── state.rs
    │   ├── shapes.rs
    │   ├── app/
    │   │   ├── mod.rs
    │   │   ├── canvas.rs    # Lógica del canvas
    │   │   └── ui.rs        # Estado de UI
    │   ├── components/
    │   │   ├── mod.rs
    │   │   ├── toolbar.rs   # Toolbar component
    │   │   ├── sidebar.rs   # Sidebars
    │   │   ├── properties.rs # Properties panel
    │   │   └── statusbar.rs  # Status bar
    │   └── renderer/
    │       ├── mod.rs
    │       ├── canvas_renderer.rs
    │       └── ui_renderer.rs
    ├── styles/
    │   ├── main.css
    │   ├── components/
    │   │   ├── toolbar.css
    │   │   ├── sidebar.css
    │   │   ├── panels.css
    │   │   └── canvas.css
    │   └── themes/
    │       ├── dark.css     # Tema actual (default)
    │       └── light.css    # Futuro
    └── assets/
        ├── icons/
        └── fonts/

packages/
├── archflow-sdk-types/     # Tipos TypeScript (existente)
├── sdk/                    # SDK TypeScript (existente)
└── archflow-web/           # NUEVO: Package para distribución web
    ├── package.json
    ├── src/
    │   ├── index.ts
    │   ├── components/
    │   └── hooks/
    └── dist/               # Output de wasm-pack
```

---

## 📝 Paso a Paso

### Paso 1: Renombrar y Restructurar

```bash
# 1. Renombrar directorio
mv crates/demo-web crates/archflow-web

# 2. Actualizar Cargo.toml
cd crates/archflow-web
# Cambiar name = "demo-web" → name = "archflow-web"
# Actualizar description y metadata
```

**Cargo.toml actualizado:**
```toml
[package]
name = "archflow-web"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "ArchFlow Web - Professional diagramming web application"
authors = ["ArchFlow Team"]
license = "MIT"
repository = "https://github.com/archflow/archflow"

[lib]
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "archflow-web"
path = "src/main.rs"

[dependencies]
# ... (mantener dependencias actuales)
archflow-sdk = { path = "../archflow-sdk" }

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Document",
  "Element",
  "HtmlElement",
  "HtmlCanvasElement",
  "CanvasRenderingContext2d",
  "Path2d",
  "Window",
  "MouseEvent",
  "WheelEvent",
  "KeyboardEvent",
  "UiEvent",
  "EventTarget",
  "DomRect",
  "CssStyleDeclaration",
]
```

### Paso 2: Crear Estructura CSS

```bash
mkdir -p crates/archflow-web/styles/components
mkdir -p crates/archflow-web/styles/themes
mkdir -p crates/archflow-web/assets/icons
```

**styles/main.css:**
```css
/* CSS Variables - Design System */
:root {
  /* Colors */
  --color-primary: #0066cc;
  --color-primary-hover: #0055aa;
  --color-primary-light: #4d9fff;
  
  --color-bg-canvas: #1e1e1e;
  --color-bg-sidebar: #2c2c2c;
  --color-bg-toolbar: #252525;
  --color-bg-panel: #2a2a2a;
  --color-bg-hover: #3a3a3a;
  --color-bg-active: #404040;
  
  --color-text-primary: #ffffff;
  --color-text-secondary: #a0a0a0;
  --color-text-muted: #666666;
  
  --color-border-default: #333333;
  --color-border-focus: #4d9fff;
  --color-border-divider: #2a2a2a;
  
  /* Typography */
  --font-family-base: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-family-mono: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
  
  /* Spacing */
  --toolbar-height: 48px;
  --statusbar-height: 24px;
  --sidebar-width: 48px;
  --properties-width: 240px;
  --layers-width: 240px;
  
  /* Z-Index */
  --z-canvas: 0;
  --z-grid: 1;
  --z-shapes: 10;
  --z-selection: 20;
  --z-ui: 100;
  --z-toolbar: 200;
  --z-popover: 300;
  --z-modal: 400;
}

/* Reset & Base */
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: var(--font-family-base);
  font-size: 13px;
  line-height: 1.5;
  color: var(--color-text-primary);
  background: var(--color-bg-canvas);
  overflow: hidden;
  user-select: none;
}

/* Layout Principal */
#app {
  display: grid;
  grid-template-areas:
    "toolbar toolbar toolbar"
    "sidebar canvas properties"
    "statusbar statusbar statusbar";
  grid-template-columns: var(--sidebar-width) 1fr var(--properties-width);
  grid-template-rows: var(--toolbar-height) 1fr var(--statusbar-height);
  height: 100vh;
  width: 100vw;
}

/* Importar componentes */
@import './components/toolbar.css';
@import './components/sidebar.css';
@import './components/panels.css';
@import './components/canvas.css';
```

### Paso 3: Implementar HTML Estructurado

**index.html nuevo:**
```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>ArchFlow - Professional Diagramming</title>
  <link rel="stylesheet" href="./styles/main.css" />
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
</head>
<body>
  <div id="app">
    <!-- Toolbar Superior -->
    <header id="toolbar">
      <div class="toolbar-section logo">
        <span class="logo-icon">🎨</span>
        <span class="logo-text">ArchFlow</span>
      </div>
      
      <div class="toolbar-section tools">
        <button class="tool-btn active" data-tool="select" title="Select (V)">
          <span class="icon">✋</span>
          <span class="label">Select</span>
        </button>
        <button class="tool-btn" data-tool="rect" title="Rectangle (R)">
          <span class="icon">⬜</span>
          <span class="label">Rect</span>
        </button>
        <button class="tool-btn" data-tool="ellipse" title="Ellipse (O)">
          <span class="icon">⭕</span>
          <span class="label">Ellipse</span>
        </button>
        <button class="tool-btn" data-tool="line" title="Line (L)">
          <span class="icon">📏</span>
          <span class="label">Line</span>
        </button>
        <button class="tool-btn" data-tool="text" title="Text (T)">
          <span class="icon">📝</span>
          <span class="label">Text</span>
        </button>
      </div>
      
      <div class="toolbar-section actions">
        <button class="icon-btn" id="btn-undo" title="Undo (Ctrl+Z)">↩️</button>
        <button class="icon-btn" id="btn-redo" title="Redo (Ctrl+Y)">↪️</button>
        <div class="separator"></div>
        <button class="icon-btn" id="btn-clear" title="Clear Canvas">🗑️</button>
      </div>
      
      <div class="toolbar-section zoom">
        <button class="icon-btn" id="zoom-out">−</button>
        <span id="zoom-level">100%</span>
        <button class="icon-btn" id="zoom-in">+</button>
        <button class="tool-btn" id="zoom-fit">Fit</button>
      </div>
    </header>
    
    <!-- Sidebar Izquierda -->
    <aside id="sidebar">
      <div class="tool-palette">
        <button class="tool-icon active" data-tool="select" title="Select (V)">✋</button>
        <button class="tool-icon" data-tool="rect" title="Rectangle (R)">⬜</button>
        <button class="tool-icon" data-tool="ellipse" title="Ellipse (O)">⭕</button>
        <button class="tool-icon" data-tool="line" title="Line (L)">📏</button>
        <button class="tool-icon" data-tool="text" title="Text (T)">📝</button>
        <button class="tool-icon" data-tool="pencil" title="Pencil (P)">✏️</button>
        <div class="separator"></div>
        <button class="tool-icon" data-tool="hand" title="Pan (Space)">🖐️</button>
      </div>
      
      <!-- Panel de Capas (expandible) -->
      <div id="layers-panel" class="panel">
        <div class="panel-header">
          <span>Layers</span>
          <button class="icon-btn" id="btn-add-layer">+</button>
        </div>
        <div class="panel-content" id="layers-list">
          <!-- Capas se generan dinámicamente -->
        </div>
      </div>
    </aside>
    
    <!-- Canvas Area -->
    <main id="canvas-area">
      <canvas id="canvas"></canvas>
      <div id="cursors-overlay"></div>
      <div id="selection-overlay"></div>
    </main>
    
    <!-- Panel de Propiedades Derecho -->
    <aside id="properties">
      <!-- Transform Panel -->
      <div class="panel" id="transform-panel">
        <div class="panel-header">
          <span>Transform</span>
          <button class="panel-toggle">▼</button>
        </div>
        <div class="panel-content">
          <div class="field-row">
            <label>X</label>
            <input type="number" id="prop-x" value="0" step="1">
            <label>Y</label>
            <input type="number" id="prop-y" value="0" step="1">
          </div>
          <div class="field-row">
            <label>W</label>
            <input type="number" id="prop-width" value="100" step="1">
            <label>H</label>
            <input type="number" id="prop-height" value="100" step="1">
          </div>
          <div class="field-row">
            <label>Rotation</label>
            <input type="number" id="prop-rotation" value="0" step="1">
            <span>°</span>
          </div>
          <label class="checkbox">
            <input type="checkbox" id="prop-lock-aspect">
            <span>Lock aspect ratio</span>
          </label>
        </div>
      </div>
      
      <!-- Appearance Panel -->
      <div class="panel" id="appearance-panel">
        <div class="panel-header">
          <span>Appearance</span>
          <button class="panel-toggle">▼</button>
        </div>
        <div class="panel-content">
          <div class="field-row">
            <label>Fill</label>
            <input type="color" id="prop-fill-color" value="#3366cc">
          </div>
          <div class="field-row">
            <label>Stroke</label>
            <input type="color" id="prop-stroke-color" value="#ffffff">
          </div>
          <div class="field-row">
            <label>Stroke Width</label>
            <input type="range" id="prop-stroke-width" min="0" max="10" value="1">
            <span id="stroke-width-value">1px</span>
          </div>
          <div class="field-row">
            <label>Opacity</label>
            <input type="range" id="prop-opacity" min="0" max="100" value="100">
            <span id="opacity-value">100%</span>
          </div>
        </div>
      </div>
      
      <!-- Alignment Panel -->
      <div class="panel" id="alignment-panel">
        <div class="panel-header">
          <span>Align</span>
          <button class="panel-toggle">▼</button>
        </div>
        <div class="panel-content">
          <div class="alignment-grid">
            <button class="align-btn" data-align="left" title="Align Left">⬅️</button>
            <button class="align-btn" data-align="center-h" title="Center Horizontally">↔️</button>
            <button class="align-btn" data-align="right" title="Align Right">➡️</button>
            <button class="align-btn" data-align="top" title="Align Top">⬆️</button>
            <button class="align-btn" data-align="center-v" title="Center Vertically">↕️</button>
            <button class="align-btn" data-align="bottom" title="Align Bottom">⬇️</button>
          </div>
          <div class="field-row">
            <button class="tool-btn" id="btn-distribute-h">⬌ Distribute H</button>
            <button class="tool-btn" id="btn-distribute-v">⬍ Distribute V</button>
          </div>
        </div>
      </div>
    </aside>
    
    <!-- Status Bar -->
    <footer id="statusbar">
      <div class="status-section">
        <span id="status-shapes">Shapes: 0</span>
        <span id="status-selected">Selected: 0</span>
      </div>
      <div class="status-section">
        <span id="status-position">Pos: 0, 0</span>
        <span id="status-zoom">Zoom: 100%</span>
      </div>
      <div class="status-section">
        <button class="status-btn" id="toggle-grid">Grid: ON</button>
        <button class="status-btn" id="toggle-snap">Snap: ON</button>
      </div>
    </footer>
  </div>
  
  <!-- Context Menu (oculto por defecto) -->
  <div id="context-menu" class="context-menu hidden">
    <div class="menu-item" data-action="copy">Copy <span class="shortcut">Ctrl+C</span></div>
    <div class="menu-item" data-action="cut">Cut <span class="shortcut">Ctrl+X</span></div>
    <div class="menu-item" data-action="paste">Paste <span class="shortcut">Ctrl+V</span></div>
    <div class="menu-separator"></div>
    <div class="menu-item" data-action="duplicate">Duplicate <span class="shortcut">Ctrl+D</span></div>
    <div class="menu-separator"></div>
    <div class="menu-item" data-action="delete">Delete <span class="shortcut">Del</span></div>
    <div class="menu-separator"></div>
    <div class="menu-item" data-action="bring-forward">Bring Forward <span class="shortcut">]</span></div>
    <div class="menu-item" data-action="send-backward">Send Backward <span class="shortcut">[</span></div>
  </div>

  <script type="module" src="./app.js"></script>
</body>
</html>
```

### Paso 4: Actualizar Código Rust

**src/main.rs** (nuevo archivo para binario wasm-pack):
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logging");
    
    log::info!("ArchFlow Web initialized");
}
```

**Refactor src/lib.rs**:
- Separar lógica en módulos
- Mantener API pública para WASM
- Mejorar estructura

### Paso 5: Integración con SDK

Actualizar el `index.html` para usar el SDK real:

```javascript
import init, { 
  ArchFlowEditor,  // Desde WASM bindings
  SelectionManager,
  ToolManager 
} from "./pkg/archflow_web.js";
```

---

## 🎯 Checklist de Implementación

### Semana 1: Estructura Base
- [ ] Renombrar demo-web → archflow-web
- [ ] Actualizar Cargo.toml
- [ ] Crear estructura de carpetas CSS
- [ ] Implementar HTML base con layout
- [ ] Migrar lógica Rust existente

### Semana 2: Componentes UI
- [ ] Toolbar superior funcional
- [ ] Sidebar izquierda con tools
- [ ] Panel de propiedades (estructura)
- [ ] Status bar
- [ ] Canvas area con grid

### Semana 3: Integración SDK
- [ ] Conectar ToolManager con UI
- [ ] Integrar SelectionManager
- [ ] Implementar creación de formas
- [ ] Mostrar selección en UI
- [ ] Properties panel funcional

### Semana 4: Features Avanzadas
- [ ] Layers panel
- [ ] Alignment tools
- [ ] Context menus
- [ ] Keyboard shortcuts completos
- [ ] Animaciones y transiciones

### Semana 5: Polish
- [ ] Testing en diferentes navegadores
- [ ] Optimización de performance
- [ ] Documentación
- [ ] Deploy script

---

## 🚀 Comandos Útiles

```bash
# Construir crate WASM
cd crates/archflow-web
wasm-pack build --target web --out-dir ../../packages/archflow-web/dist

# Servir localmente
cd packages/archflow-web
python3 -m http.server 8080
# o
npx serve .

# Limpiar y reconstruir
cargo clean
wasm-pack build --target web
```

---

## 📝 Notas Importantes

1. **Mantener compatibilidad**: El SDK debe funcionar tanto para demo-web como para archflow-web durante la transición

2. **Feature flags**: Considerar usar feature flags en el crate para diferentes modos

3. **Assets**: Los iconos SVG se deben optimizar antes de incluirlos

4. **Testing**: Mantener tests existentes y agregar tests de integración UI

5. **Performance**: Usar `requestAnimationFrame` para todas las animaciones

---

## 🔗 Referencias

- [Design Spec](./ARCHFLOW-WEB-DESIGN-SPEC.md)
- [User Interaction Study](../analysis/USER-INTERACTION-STUDY.md)
- [SDK Documentation](../../crates/archflow-sdk/README.md)

---

*Plan creado: Enero 2025*
*Versión: 1.0*
