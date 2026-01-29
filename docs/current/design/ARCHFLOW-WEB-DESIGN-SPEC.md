# Especificación de Diseño: ArchFlow Web MVP

## 📋 Información General

- **Proyecto**: ArchFlow Web MVP (renombrado desde demo-web)
- **Fecha**: Enero 2025
- **Versión**: 1.0.0
- **Objetivo**: Crear una interfaz profesional tipo Figma/tldraw integrada con el SDK Rust

---

## 🎨 Sistema de Diseño

### 1. Paleta de Colores

#### Colores Principales (Brand)
| Nombre | Hex | Uso |
|--------|-----|-----|
| `--color-primary` | `#0066cc` | Botones activos, selección, acentos |
| `--color-primary-hover` | `#0055aa` | Hover de elementos primarios |
| `--color-primary-light` | `#4d9fff` | Highlights, foco |

#### Colores de Fondo
| Nombre | Hex | Uso |
|--------|-----|-----|
| `--color-bg-canvas` | `#1e1e1e` | Fondo del canvas infinito |
| `--color-bg-sidebar` | `#2c2c2c` | Sidebars izquierdo/derecho |
| `--color-bg-toolbar` | `#252525` | Toolbars superior/inferior |
| `--color-bg-panel` | `#2a2a2a` | Paneles y cards |
| `--color-bg-hover` | `#3a3a3a` | Estado hover de items |
| `--color-bg-active` | `#404040` | Estado activo de items |

#### Colores de Texto
| Nombre | Hex | Uso |
|--------|-----|-----|
| `--color-text-primary` | `#ffffff` | Texto principal |
| `--color-text-secondary` | `#a0a0a0` | Texto secundario, labels |
| `--color-text-muted` | `#666666` | Texto deshabilitado |
| `--color-text-accent` | `#4d9fff` | Links, acentos |

#### Colores de Borde
| Nombre | Hex | Uso |
|--------|-----|-----|
| `--color-border-default` | `#333333` | Bordes por defecto |
| `--color-border-focus` | `#4d9fff` | Bordes en foco |
| `--color-border-divider` | `#2a2a2a` | Separadores |

#### Colores Funcionales
| Nombre | Hex | Uso |
|--------|-----|-----|
| `--color-success` | `#4caf50` | Éxito, confirmaciones |
| `--color-warning` | `#ff9800` | Advertencias |
| `--color-error` | `#f44336` | Errores, eliminar |
| `--color-info` | `#2196f3` | Información |

#### Colores de Formas (Presets)
| Nombre | Hex | Uso |
|--------|-----|-----|
| `--shape-blue` | `#3366cc` | Rectángulos |
| `--shape-green` | `#33aa66` | Elipses |
| `--shape-orange` | `#ff8800` | Líneas |
| `--shape-purple` | `#9933cc` | Grupos |
| `--shape-red` | `#cc3333` | Texto |

---

### 2. Tipografía

#### Familia de Fuentes
```css
--font-family-base: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
--font-family-mono: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
```

#### Escala Tipográfica
| Estilo | Tamaño | Peso | Altura | Uso |
|--------|--------|------|--------|-----|
| **H1** | 24px | 600 | 32px | Título de página |
| **H2** | 18px | 600 | 24px | Títulos de sección |
| **H3** | 14px | 600 | 20px | Títulos de panel |
| **Body** | 13px | 400 | 20px | Texto general |
| **Body Small** | 12px | 400 | 16px | Labels, metadata |
| **Caption** | 11px | 400 | 14px | Status bar, hints |
| **Button** | 13px | 500 | 16px | Botones |
| **Input** | 13px | 400 | 20px | Campos de entrada |
| **Monospace** | 12px | 400 | 16px | Coordenadas, IDs |

---

### 3. Espaciado y Layout

#### Sistema de Espaciado (4px base)
```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 20px;
--space-6: 24px;
--space-8: 32px;
--space-10: 40px;
```

#### Dimensiones de Componentes
| Componente | Altura | Padding | Otros |
|------------|--------|---------|-------|
| **Toolbar** | 48px | 0 16px | - |
| **Status Bar** | 24px | 0 16px | - |
| **Sidebar** | 100% - 72px | 12px | width: 240px |
| **Tool Button** | 32px | 6px 12px | border-radius: 6px |
| **Icon Button** | 32px | 6px | border-radius: 6px |
| **Input Field** | 28px | 4px 8px | border-radius: 4px |
| **Panel Header** | 32px | 0 12px | - |

#### Layout Grid
- **Canvas Grid**: 20px (cuadrícula de puntos)
- **UI Grid**: 8px (para componentes)
- **Z-Index Layers**:
  - `z-canvas`: 0 (capa base)
  - `z-grid`: 1 (cuadrícula)
  - `z-shapes`: 10 (formas)
  - `z-selection`: 20 (selección y handles)
  - `z-ui`: 100 (UI overlay)
  - `z-toolbar`: 200 (toolbars)
  - `z-popover`: 300 (menus, tooltips)
  - `z-modal`: 400 (modales)

---

### 4. Componentes UI

#### 4.1 Botones

**Primary Button**
```css
background: var(--color-primary);
color: white;
border: none;
border-radius: 6px;
padding: 6px 12px;
font-size: 13px;
font-weight: 500;
cursor: pointer;
transition: background 0.15s ease;

&:hover { background: var(--color-primary-hover); }
&:active { background: #004488; }
```

**Secondary Button**
```css
background: var(--color-bg-panel);
color: var(--color-text-primary);
border: 1px solid var(--color-border-default);
border-radius: 6px;
padding: 6px 12px;

&:hover { background: var(--color-bg-hover); }
```

**Icon Button**
```css
width: 32px;
height: 32px;
background: transparent;
border: none;
border-radius: 6px;
color: var(--color-text-secondary);

&:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }
&:active { background: var(--color-bg-active); }
&.active { background: var(--color-primary); color: white; }
```

**Tool Button** (para toolbar)
```css
/* Igual que Icon Button pero con label */
padding: 6px 12px;
width: auto;
display: flex;
align-items: center;
gap: 6px;
```

#### 4.2 Inputs

**Text Input**
```css
background: var(--color-bg-panel);
color: var(--color-text-primary);
border: 1px solid var(--color-border-default);
border-radius: 4px;
padding: 4px 8px;
font-size: 13px;
height: 28px;
width: 100%;

&:focus { border-color: var(--color-border-focus); outline: none; }
```

**Number Input** (con flechas)
```css
/* Igual que Text Input */
width: 60px;
text-align: center;
```

**Color Picker**
```css
width: 24px;
height: 24px;
border-radius: 4px;
border: 2px solid var(--color-border-default);
cursor: pointer;

&:hover { border-color: var(--color-primary); }
```

#### 4.3 Paneles

**Panel Container**
```css
background: var(--color-bg-sidebar);
border-right: 1px solid var(--color-border-divider);
display: flex;
flex-direction: column;
```

**Panel Section**
```css
border-bottom: 1px solid var(--color-border-divider);
padding: 12px;
```

**Panel Header**
```css
font-size: 12px;
font-weight: 600;
color: var(--color-text-secondary);
text-transform: uppercase;
letter-spacing: 0.5px;
margin-bottom: 12px;
```

**Collapsible Panel**
```css
/* Panel Header con ícono de toggle */
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
}

.panel-toggle-icon {
  transition: transform 0.2s ease;
}

.panel-toggle-icon.collapsed {
  transform: rotate(-90deg);
}
```

#### 4.4 Lista de Capas (Layers)

**Layer Item**
```css
display: flex;
align-items: center;
gap: 8px;
padding: 6px 8px;
border-radius: 4px;
cursor: pointer;

&:hover { background: var(--color-bg-hover); }
&.selected { background: var(--color-primary); }

.layer-icon { font-size: 14px; }
.layer-name { flex: 1; font-size: 12px; }
.layer-visibility { opacity: 0.6; }
.layer-lock { opacity: 0.6; }
```

#### 4.5 Separadores

**Vertical Divider**
```css
width: 1px;
height: 24px;
background: var(--color-border-divider);
margin: 0 8px;
```

**Horizontal Divider**
```css
width: 100%;
height: 1px;
background: var(--color-border-divider);
margin: 12px 0;
```

#### 4.6 Tooltips

```css
background: var(--color-bg-toolbar);
color: var(--color-text-primary);
padding: 6px 10px;
border-radius: 4px;
font-size: 12px;
box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
position: absolute;
z-index: var(--z-popover);
```

#### 4.7 Context Menu

```css
background: var(--color-bg-toolbar);
border: 1px solid var(--color-border-default);
border-radius: 6px;
box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
padding: 4px;
min-width: 160px;

.menu-item {
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  
  &:hover { background: var(--color-bg-hover); }
}

.menu-separator {
  height: 1px;
  background: var(--color-border-divider);
  margin: 4px 0;
}

.menu-shortcut {
  margin-left: auto;
  color: var(--color-text-muted);
  font-size: 11px;
}
```

---

### 5. Iconografía

#### Iconos a Implementar (SVG)

**Toolbar Icons** (24x24px)
| Icono | Nombre | Unicode/Descripción |
|-------|--------|---------------------|
| ✋ | select | Cursor/manita |
| ⬜ | rectangle | Rectángulo |
| ⭕ | ellipse | Círculo/elipse |
| 📏 | line | Línea |
| 📝 | text | Texto |
| ✏️ | pencil | Lápiz/dibujo |
| 🖱️ | hand | Pan/mano |
| 🔍+ | zoom-in | Zoom in |
| 🔍- | zoom-out | Zoom out |
| 🗑️ | delete | Eliminar |
| 📋 | copy | Copiar |
| ✂️ | cut | Cortar |
| 📌 | paste | Pegar |
| ↩️ | undo | Undo |
| ↪️ | redo | Redo |
| 👁️ | visible | Visible |
| 👁️‍🗨️ | hidden | Oculto |
| 🔒 | locked | Bloqueado |
| 🔓 | unlocked | Desbloqueado |
| 📂 | group | Grupo |
| 📁 | ungroup | Desagrupar |
| ⚙️ | settings | Configuración |

**Layer Icons** (16x16px)
| Icono | Tipo |
|-------|------|
| ⬜ | Rectángulo |
| ⭕ | Elipse |
| 📏 | Línea |
| 📝 | Texto |
| 📂 | Grupo |

**Alignment Icons** (16x16px)
| Icono | Alineación |
|-------|-----------|
| ⬅️ | Left |
| ↔️ | Center H |
| ➡️ | Right |
| ⬆️ | Top |
| ↕️ | Center V |
| ⬇️ | Bottom |
| ⬌ | Distribute H |
| ⬍ | Distribute V |

---

## 🖥️ Estructura de la Interfaz

### Layout Principal

```
┌─────────────────────────────────────────────────────────────────────────┐
│ [Toolbar Superior]                                                      │
├──────────┬──────────┬───────────────────────────────────┬───────────────┤
│          │          │                                   │               │
│ Library  │ Sidebar  │                                   │  Properties   │
│ Panel    │ Tools    │         CANVAS AREA               │   Panel       │
│ (280px)  │ (48px)   │                                   │  (240px)      │
│          │          │                                   │               │
│          │          │                                   │               │
├──────────┴──────────┴───────────────────────────────────┴───────────────┤
│ [Status Bar]                                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

**Layout Alternativo** (Library como panel colapsable):
```
┌─────────────────────────────────────────────────────────────────┐
│ [Toolbar Superior]                                              │
├──────┬──────────┬───────────────────────────────────┬───────────┤
│      │          │                                   │           │
│ Lib  │ Sidebar  │                                   │ Properties│
│(48px)│ Tools    │           CANVAS AREA             │  Panel    │
│      │ (48px)   │                                   │  (240px)  │
├──────┴──────────┴───────────────────────────────────┴───────────┤
│ [Status Bar]                                                    │
└─────────────────────────────────────────────────────────────────┘

Cuando Library está expandido:
├─────────┬──────────┬─────────────────────────────┬───────────────┤
│ Library  │ Sidebar  │                             │               │
│ Panel    │ Tools    │       CANVAS AREA           │  Properties   │
│ (280px)  │ (48px)   │                             │   Panel       │
└─────────┴──────────┴─────────────────────────────┴───────────────┘
```

### 1. Toolbar Superior (48px)

**Secciones** (izquierda a derecha):

1. **Logo/App** (80px)
   - Icono 🎨 + "ArchFlow"

2. **Tools** (con separadores)
   - Select (V)
   - Rectangle (R)
   - Ellipse (O)
   - Line (L)
   - Text (T)
   - Pencil (P)

3. **Separator** |

4. **Actions**
   - Undo ↩️
   - Redo ↪️

5. **Separator** |

6. **Zoom Controls** (derecha)
   - Zoom Out -
   - [100%] (dropdown)
   - Zoom In +
   - Fit to Screen

---

### 2. Sidebar Izquierdo (48px)

**Vertical Icon Bar**:
- Move (V) - activo por defecto
- Rectangle (R)
- Ellipse (O)
- Line (L)
- Text (T)
- Pencil (P)
- Separator
- Hand (pan)
- Zoom

**Panel de Capas** (expandible, 240px ancho):
- Header: "Layers" + [+]
- Lista de capas (drag & drop)
- Controles de visibilidad/bloqueo

---

### 3. Component Library Panel (280px) - draw.io/excalidraw style

Panel de librerías de componentes para arrastrar y soltar elementos predefinidos.

**Estructura:**
```
┌──────────────────────────────────────┐
│ 🔍 Search components...        [⚙️] │
├──────────────────────────────────────┤
│ 📚 General                    [v]   │
│   ⬜ Rectangle        ⬭ Rounded     │
│   ● Circle           ⬭ Ellipse      │
│   ◆ Diamond          ▲ Triangle     │
├──────────────────────────────────────┤
│ 📊 Flowchart                  [v]   │
│   ⬭ Start/End        ⬜ Process     │
│   ◆ Decision         🛢️ Database    │
├──────────────────────────────────────┤
│ 🏗️ UML                        [v]   │
│   ⬜ Class           👤 Actor        │
│   ⬭ Use Case        📁 Package     │
├──────────────────────────────────────┤
│ ☁️ AWS                         [v]   │
│   🖥️ EC2             💾 S3          │
├──────────────────────────────────────┤
│ 🏛️ C4 Model                    [v]   │
│   👤 Person          🏢 System       │
│   📱 Container       ⚙️ Component    │
├──────────────────────────────────────┤
│ ⭐ My Library                  [v]   │
│   [Custom shapes...]                 │
├──────────────────────────────────────┤
│ [+] Import Library                   │
└──────────────────────────────────────┘
```

**Características:**
- **Search**: Búsqueda en tiempo real por nombre, descripción o tags
- **Categories**: Colapsables, organizadas por tipo (General, Flowchart, UML, AWS, C4)
- **Grid Layout**: Componentes en grid de 3 columnas
- **Drag & Drop**: Arrastrar componentes al canvas
- **Import/Export**: Soporte para librerías personalizadas

**Librerías Incluidas:**
1. **General**: Rectángulos, círculos, diamantes, triángulos, hexágonos
2. **Flowchart**: Símbolos de diagramas de flujo (start/end, process, decision, DB)
3. **UML**: Diagramas UML (clases, actores, casos de uso, paquetes)
4. **AWS**: Iconos de arquitectura AWS (EC2, S3, RDS, VPC)
5. **C4 Model**: Diagramas C4 (Person, System, Container, Component)
6. **My Library**: Componentes personalizados del usuario

**Ver especificación completa**: [COMPONENT-LIBRARY-SPEC.md](./COMPONENT-LIBRARY-SPEC.md)

---

### 4. Panel de Propiedades Derecho (240px)

#### Sección: Transform
```
┌─────────────────────────────┐
│ TRANSFORM            [v]    │
├─────────────────────────────┤
│ X: [____]  Y: [____]       │
│ W: [____]  H: [____]       │
│ Rotation: [____]°          │
│                             │
│ [🔗] Lock aspect ratio      │
└─────────────────────────────┘
```

#### Sección: Appearance
```
┌─────────────────────────────┐
│ APPEARANCE           [v]    │
├─────────────────────────────┤
│ Fill:     [🔲] [#3366cc]   │
│ Stroke:   [🔲] [#ffffff]   │
│ Width:    [ 2] px          │
│ Opacity:  [100%]           │
│                             │
│ [ ] Shadow                 │
│ [ ] Rounded corners        │
└─────────────────────────────┘
```

#### Sección: Alignment
```
┌─────────────────────────────┐
│ ALIGN                [v]    │
├─────────────────────────────┤
│ [⬅️][↔️][➡️]              │
│ [⬆️][↕️][⬇️]              │
│                             │
│ [⬌] Distribute H          │
│ [⬍] Distribute V          │
└─────────────────────────────┘
```

#### Sección: Layers
```
┌─────────────────────────────┐
│ LAYERS               [v]    │
├─────────────────────────────┤
│ [⬆️] Bring Forward         │
│ [⬇️] Send Backward         │
│                             │
│ [Group] [Ungroup]          │
└─────────────────────────────┘
```

---

### 4. Canvas Area

**Características**:
- Fondo: `#1e1e1e`
- Grid: Puntos cada 20px, color `#2a2a2a`
- Infinite scroll/pan

**Elementos Overlay**:
- Box selection (dashed blue)
- Selection bounds (blue dashed + handles)
- Remote cursors (con nombre)

**Handles de Selección** (8 puntos):
```
    ●────●────●
    │         │
    ●    ×    ●  (× = center, ● = handles)
    │         │
    ●────●────●
```

**Handle Visual**:
- Tamaño: 8x8px
- Color: `#0066cc`
- Border: 1px white
- Cursor según posición (nwse, nesw, ew, ns)

---

### 5. Status Bar (24px)

**Izquierda**:
- Shapes: [count]
- Selected: [count]

**Centro**:
- Pos: [X], [Y]
- Zoom: [100%]

**Derecha**:
- Grid: [ON/OFF]
- Snap: [ON/OFF]

---

## 🎭 Estados y Animaciones

### Estados de Componentes

**Button States**:
```
Normal -> Hover -> Active -> Disabled
(300ms transitions)
```

**Selection States**:
```
None -> Hover -> Selected -> Editing
```

### Animaciones

| Animación | Duración | Easing | Uso |
|-----------|----------|--------|-----|
| **fade-in** | 150ms | ease-out | Tooltips, menus |
| **slide-in** | 200ms | ease-out | Sidebar panels |
| **scale** | 100ms | ease-in-out | Button clicks |
| **color** | 150ms | ease | Hover states |
| **selection** | 200ms | ease-out | Selection bounds |

---

## 🎯 Integración con SDK

### Funcionalidades a Implementar

#### Fase 1: Core UI (Week 1-2) ✅ COMPLETADO
- [x] Layout base con sidebars
- [x] Toolbar con herramientas
- [x] Canvas con grid
- [x] Status bar

#### Fase 2: Tools Integration (Week 2-3) ✅ COMPLETADO
- [x] Tool state machine (V, R, O, L, T, P)
- [x] Canvas click handlers
- [x] Shape creation
- [x] Selection display

#### Fase 3: Properties Panel (Week 3-4) ⚠️ PARCIAL
- [x] Transform panel (X, Y, W, H, Rotation) - Estructura UI
- [x] Appearance panel (Fill, Stroke, Opacity) - Estructura UI
- [ ] Real-time updates - Conectar con selección
- [ ] Multi-selection support

#### Fase 4: Component Library (Week 4-5) ⚠️ PARCIAL
- [x] Library sidebar UI
- [x] Drag & drop functionality
- [x] Search and filter
- [ ] Built-in libraries (General, Flowchart, UML, C4) - En desarrollo
- [ ] Import/export custom libraries

#### Fase 5: Advanced Features (Week 5-6) ❌ PENDIENTE
- [ ] Layers panel (reorder, visibility)
- [ ] Alignment tools
- [ ] Group/ungroup
- [ ] Context menus

#### Fase 6: Polish (Week 6-7) ⚠️ PARCIAL
- [x] Keyboard shortcuts - Implementados en lib.rs
- [ ] Tooltips
- [ ] Animations
- [ ] Dark/light theme

---

## 📱 Responsive Considerations

### Estado: ❌ NO IMPLEMENTADO
El diseño responsive no está implementado aún.

| Breakpoint | Ancho | Cambios |
|------------|-------|---------|
| Desktop XL | >1440px | Full layout |
| Desktop | 1024-1440px | Standard |
| Tablet | 768-1024px | Collapse right panel |
| Mobile | <768px | Single column, floating panels |

### Mobile Adaptations (Pendiente)
- Toolbar inferior (floating)
- Panels como modales
- Gestos táctiles prioritarios
- Zoom por pinza

---

## 🔧 Assets Necesarios

### Iconos SVG
✅ **Phosphor Icons integrado** via CDN `@phosphor-icons/core@2.0.2`
No necesita assets SVG adicionales.

### Fuentes
✅ **Inter** cargada desde Google Fonts
```
assets/fonts/
├── Inter/
│   ├── Inter-Regular.woff2
│   ├── Inter-Medium.woff2
│   └── Inter-SemiBold.woff2
└── JetBrainsMono/
    └── JetBrainsMono-Regular.woff2
```

---

## 📚 Referencias

- [Figma Design](https://figma.com) - Referencia de diseño
- [tldraw](https://tldraw.dev) - Referencia de interacción
- [USER-INTERACTION-STUDY.md](./USER-INTERACTION-STUDY.md) - Estudio de interacción
- [SDK Documentation](../../crates/archflow-sdk/README.md) - Documentación del SDK

---

## 🔄 Changelog

| Fecha | Versión | Cambios |
|-------|---------|---------|
| 2025-01-29 | 1.0.0 | Especificación inicial basada en imagen y SDK |

---

*Documento generado para ArchFlow MVP - Web Interface Design Specification*
