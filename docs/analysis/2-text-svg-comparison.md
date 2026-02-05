# Análisis Comparativo: Texto y SVG
## archflow-render vs Figma/Excalidraw/Tldraw

> Fecha: 2026-02-04 | Versión: v0.41.0

---

## Resumen Ejecutivo

| Aspecto | archflow-render | Figma | Excalidraw | Tldraw |
|---------|----------------|-------|------------|--------|
| **Rendering** | MTSDF ✅ | ✅ SDF | ✅ Canvas | ✅ Canvas |
| **Font Family** | ❌ No | ✅ Sí | ✅ Sí | ✅ Sí |
| **Font Size** | ✅ Básico | ✅ Completo | ✅ Completo | ✅ Completo |
| **Text Styles** | ❌ No | ✅ Text Styles | ❌ Limitado | ✅ Limitado |
| **Text on Path** | ❌ No | ✅ Sí | ❌ No | ✅ Sí |
| **Auto-resize** | ❌ No | ✅ Auto Layout | ❌ No | ✅ Sí |
| **SVG Export** | ❌ No | ✅ Full | ✅ Full | ✅ Full |
| **SVG Import** | ❌ No | ✅ Sí | ⚠️第三方 | ✅ Sí |

---

## 1. TEXTO (Text Rendering)

### 1.1 Estado Actual de archflow-render

```wgsl
// Implementación actual: MTSDF Text Shader
// Archivo: mtsdf_text.wgsl

// Features implementadas:
✅ Rendering MTSDF para texto nítido a cualquier zoom
✅ Color tinting (RGBA)
✅ Rendering vía texture atlas
✅ Anti-aliasing suave con smoothstep

// Features FALTANTES:
❌ Font family selection
❌ Font weight (bold, light, etc.)
❌ Font style (italic)
❌ Text alignment (left, center, right, justify)
❌ Line height / leading
❌ Letter spacing / kerning
❌ Text styles / themes
❌ Text on path
❌ Auto-resize
❌ Rich text (mixed styles en mismo texto)
❌ Bidirectional text (RTL)
❌ Sub/superscript
```

### 1.2 Análisis Detallado de Texto

#### Figma

| Propiedad | Estado | Notas |
|-----------|--------|-------|
| **Font Family** | ✅ | 1000+ Google Fonts + sistemas |
| **Font Size** | ✅ | 1px - 512px |
| **Font Weight** | ✅ | 100-900 + presets |
| **Font Style** | ✅ | Italic toggle |
| **Text Align** | ✅ | Left/Center/Right/Justify |
| **Line Height** | ✅ | Auto, % o px |
| **Letter Spacing** | ✅ | En % |
| **Text Styles** | ✅ | Named reusable styles |
| **Text on Path** | ✅ | Along any vector path |
| **Auto-resize** | ✅ | Hug contents / Fixed size |
| **Rich Text** | ✅ | Inline styles |
| **Vertical Type** | ✅ | Top-down Chinese/Japanese |
| **Truncation** | ✅ | End/middle truncation |

#### Excalidraw

| Propiedad | Estado | Notas |
|-----------|--------|-------|
| **Font Family** | ✅ | Limited set (4-5 fonts) |
| **Font Size** | ✅ | Slider + presets |
| **Font Weight** | ✅ | Bold toggle only |
| **Font Style** | ✅ | Italic toggle |
| **Text Align** | ✅ | Left/Center/Right |
| **Line Height** | ❌ | No soportado |
| **Letter Spacing** | ❌ | No soportado |
| **Text Styles** | ❌ | No soportado |
| **Text on Path** | ❌ | No soportado |
| **Auto-resize** | ❌ | No soportado |
| **Rich Text** | ❌ | Plain text only |

#### Tldraw

| Propiedad | Estado | Notas |
|-----------|--------|-------|
| **Font Family** | ✅ | Multiple fonts |
| **Font Size** | ✅ | Slider + presets |
| **Font Weight** | ✅ | Bold toggle |
| **Font Style** | ✅ | Italic toggle |
| **Text Align** | ✅ | Left/Center/Right |
| **Line Height** | ⚠️ | Partial |
| **Letter Spacing** | ❌ | No soportado |
| **Text Styles** | ⚠️ | Limited |
| **Text on Path** | ✅ | Sí soportado |
| **Auto-resize** | ✅ | Auto width/height |
| **Rich Text** | ⚠️ | Partial |

### 1.3 Matriz de Comparación de Texto

| Feature | archflow | Figma | Excalidraw | Tldraw |
|---------|----------|-------|------------|--------|
| **Rendering Engine** | MTSDF | SDF | Canvas | Canvas |
| Font Family | ❌ | ✅ | ✅ | ✅ |
| Font Size | ✅ | ✅ | ✅ | ✅ |
| Bold/Italic | ❌ | ✅ | ✅ | ✅ |
| Text Alignment | ❌ | ✅ | ✅ | ✅ |
| Line Height | ❌ | ✅ | ❌ | ⚠️ |
| Letter Spacing | ❌ | ✅ | ❌ | ❌ |
| Text Styles | ❌ | ✅ | ❌ | ⚠️ |
| Text on Path | ❌ | ✅ | ❌ | ✅ |
| Auto-resize | ❌ | ✅ | ❌ | ✅ |
| Rich Text | ❌ | ✅ | ❌ | ⚠️ |
| RTL Support | ❌ | ✅ | ❌ | ❌ |
| Export to SVG | ❌ | ✅ | ✅ | ✅ |

### 1.4 Gap Analysis para Texto

```
CRÍTICO (necesario para MVP):
├── Text alignment (left/center/right)
├── Font weight (bold)
└── Font style (italic)

ALTA PRIORIDAD:
├── Line height
├── Rich text (inline styles)
└── Text styles system

MEDIA PRIORIDAD:
├── Text on path
├── Auto-resize
└── RTL support

BAJA PRIORIDAD:
├── Letter spacing
├── Vertical text
└── Subscript/superscript
```

### 1.5 Implementación Requerida para Texto

```rust
// Propuesta: TextStyle struct
pub struct TextStyle {
    pub font_family: FontFamily,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_align: TextAlign,
    pub line_height: f32,
    pub letter_spacing: f32,
}

pub enum FontFamily {
    System(String),  // "Inter", "San Francisco", etc.
    Custom(String),  // Custom font name
}

pub enum FontWeight {
    Thin = 100,
    Light = 300,
    Regular = 400,
    Medium = 500,
    Bold = 700,
    Black = 900,
}

pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

// Extender GpuInstance para texto
pub struct TextInstance {
    pub text: String,           // GPU no puede almacenar strings
    pub style: TextStyle,
    pub max_width: f32,        // Para wrapping
    pub vertical_align: VerticalAlign,  // Top/Middle/Bottom
}
```

---

## 2. SVG (Vector Graphics)

### 2.1 Estado Actual de archflow-render

```rust
// SVG Support: ❌ NO IMPLEMENTADO

// Features relacionadas con SVG:
❌ SVG Import
❌ SVG Export
❌ Vector Path rendering
❌ Pencil/Sketch tool
❌ Pen tool
❌ Path manipulation
❌ Boolean operations on paths
```

### 2.2 Análisis Detallado de SVG

#### Figma

| Propiedad | Estado | Notas |
|-----------|--------|-------|
| **SVG Export** | ✅ | Full fidelity |
| **SVG Import** | ✅ | Parse + editable |
| **Pen Tool** | ✅ | Vector networks |
| **Pencil Tool** | ✅ | Sketch to vector |
| **Path Editing** | ✅ | Full control points |
| **Boolean Ops** | ✅ | Union/Subtract/Intersect |
| **Flatten** | ✅ | To simple paths |
| **Outline Stroke** | ✅ | Convert to shapes |
| **Vector Networks** | ✅ | Custom shapes |

#### Excalidraw

| Propiedad | Estado | Notas |
|-----------|--------|-------|
| **SVG Export** | ✅ | Full (hand-drawn style) |
| **SVG Import** | ⚠️ | Via third-party tools |
| **Pencil Tool** | ✅ | Native sketch |
| **Pen Tool** | ❌ | No vector networks |
| **Arrow Sloppiness** | ✅ | Hand-drawn style |
| **Export Style** | ✅ | Keep as sketch |
| **Stroke Smoothing** | ✅ | Algorithmic smoothing |

#### Tldraw

| Propiedad | Estado | Notas |
|-----------|--------|-------|
| **SVG Export** | ✅ | Full |
| **SVG Import** | ✅ | Native support |
| **Pen Tool** | ✅ | Vector paths |
| **Pencil Tool** | ✅ | Quick sketch |
| **Shape Shapes** | ✅ | Pre-made shapes |
| **Custom Shapes** | ✅ | via components |
| **TLML** | ✅ | Custom markup |

### 2.3 Matriz de Comparación SVG

| Feature | archflow | Figma | Excalidraw | Tldraw |
|---------|----------|-------|------------|--------|
| **SVG Export** | ❌ | ✅ Full | ✅ Full | ✅ Full |
| **SVG Import** | ❌ | ✅ Full | ⚠️ Limited | ✅ Full |
| **Pen Tool** | ❌ | ✅ Networks | ❌ | ✅ |
| **Pencil Tool** | ❌ | ✅ | ✅ Native | ✅ |
| **Path Editing** | ❌ | ✅ Full | ⚠️ Basic | ✅ |
| **Boolean Ops** | ❌ | ✅ | ❌ | ❌ |
| **Stroke → Fill** | ❌ | ✅ | ❌ | ❌ |
| **Smoothing** | ❌ | ✅ | ✅ | ✅ |
| **Arrowheads** | ❌ | ✅ | ✅ | ✅ |

### 2.4 Gap Analysis para SVG

```
CRÍTICO (diagrams need SVG export):
├── SVG export (para diagrams exportables)
├── SVG import (para assets externos)
└── Pencil tool (para anotaciones rápidas)

ALTA PRIORIDAD:
├── Pen tool básico (líneas rectas → curvas)
├── Arrowheads para líneas
└── Stroke smoothing (opcional)

MEDIA PRIORIDAD:
├── Full path editing
├── Boolean operations
└── Vector networks (Figma-style)
```

### 2.5 Opciones de Implementación SVG

#### Opción A: SVG Externo Library

```rust
// Usar usvg o svg-hush para parsing
use usvg::{Options, Tree};

pub fn import_svg(svg_data: &[u8]) -> Result<EntityStore, Error> {
    let opt = Options::default();
    let tree = Tree::from_data(svg_data, &opt)?;
    
    // Convertir elementos SVG a entidades
    for node in tree.root().descendants() {
        match *node {
            Node::Path(path) => {
                // Convertir path a shapes
                let shape = convert_path_to_shape(&path)?;
                store.spawn(shape);
            }
            Node::Text(text) => {
                // Convertir texto
                let text_entity = convert_text_node(&text)?;
                store.spawn(text_entity);
            }
            _ => {} // Ignorar otros elementos
        }
    }
    Ok(store)
}
```

#### Opción B: Pencil Tool Propio

```rust
// Implementación de Pencil/Sketch tool
pub struct PencilTool {
    smoothing_factor: f32,
    min_distance: f32,
}

impl PencilTool {
    pub fn process_points(&self, raw_points: &[Vec2]) -> Vec<Vec2> {
        // Algoritmo de smoothing (Chaikin's curves o similar)
        let mut smoothed = raw_points.to_vec();
        
        // Aplicar smoothing iterativo
        for _ in 0..3 {
            smoothed = self.chaikin_smooth(&smoothed);
        }
        
        // Simplificar puntos redundantes
        self.ramer_douglas_peucker(&smoothed, 2.0)
    }
    
    fn chaikin_smooth(&self, points: &[Vec2]) -> Vec<Vec2> {
        // Corner cutting algorithm
        points.windows(2).map(|w| {
            let q = w[0] * 0.75 + w[1] * 0.25;
            let r = w[0] * 0.25 + w[1] * 0.75;
            vec![q, r]
        }).flatten().collect()
    }
}
```

### 2.6 SVG Export Template

```rust
// Generador de SVG básico
pub struct SvgExporter {
    canvas_width: f32,
    canvas_height: f32,
}

impl SvgExporter {
    pub fn export(&self, store: &EntityStore) -> String {
        let mut svg = String::new();
        
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            self.canvas_width, self.canvas_height
        ));
        
        for entity in store.entities() {
            match entity.shape_type {
                ShapeType::Rect => {
                    svg.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="#{:08x}" />"#,
                        entity.pos.x, entity.pos.y,
                        entity.size.x, entity.size.y,
                        entity.color
                    ));
                }
                ShapeType::Circle => {
                    svg.push_str(&format!(
                        r#"<circle cx="{}" cy="{}" r="{}" fill="#{:08x}" />"#,
                        entity.pos.x + entity.size.x / 2.0,
                        entity.pos.y + entity.size.y / 2.0,
                        entity.size.x / 2.0,
                        entity.color
                    ));
                }
                _ => {} // Otras formas
            }
        }
        
        svg.push_str("</svg>");
        svg
    }
}
```

---

## 3. Roadmap de Implementación

### Fase 1: Texto Esencial (v0.42.0)

```markdown
## Text Core
- [ ] Add font family selection
- [ ] Add font weight (bold)
- [ ] Add font style (italic)
- [ ] Add text alignment (left/center/right)
- [ ] Add line height
- [ ] Rich text support (mixed styles)
```

### Fase 2: SVG Export/Import (v0.43.0)

```markdown
## SVG Support
- [ ] Basic SVG export
- [ ] Basic SVG import (paths → shapes)
- [ ] Arrowheads for lines
```

### Fase 3: Pencil Tool (v0.44.0)

```markdown
## Sketch Support
- [ ] Pencil tool with smoothing
- [ ] Point simplification
- [ ] Sketch rendering via SDF
```

### Fase 4: Texto Avanzado (v0.45.0)

```markdown
## Advanced Text
- [ ] Text on path
- [ ] Auto-resize
- [ ] RTL support
- [ ] Text styles system
```

---

## 4. Impacto en GpuInstance

### 4.1 Texto Adicional

```rust
// GpuInstance existente + campos para texto
pub struct GpuInstance {
    // ... campos existentes ...
    
    // Para texto
    pub font_id: u16,           // Index into font atlas
    pub font_size: u16,         // Font size in pixels
    pub font_style: u8,         // Bold=1, Italic=2
    pub text_align: u8,         // Left=0, Center=1, Right=2
}
```

### 4.2 SVG Path Data

```rust
// Para paths SVG importados
pub struct PathData {
    pub points: Vec<Vec2>,      // Control points
    pub is_closed: bool,        // Closed path
    pub stroke_width: f32,      // If stroked
}

// En EntityStore
pub struct EntityStore {
    // ... existentes ...
    pub path_data: HashMap<EntityId, PathData>,
}
```

---

## 5. Dependencias Recomendadas

### Para Texto

| Library | Purpose | License |
|---------|---------|---------|
| `rustybuzz` | HarfBuzz port, text shaping | MPL-2.0 |
| `font-kit` | Font discovery/loading | MIT/Apache |
| `ttf-parser` | Parse font files | MIT |

### Para SVG

| Library | Purpose | License |
|---------|---------|---------|
| `usvg` | SVG parsing/rendering | MPL-2.0 |
| `svg-hush` | SVG sanitization | MIT |
| `kurbo` | 2D geometry (Beziers) | MIT/Apache |

---

## 6. Conclusiones

### Fortalezas Actuales de archflow-render
1. ✅ MTSDF implementado (excelente base)
2. ✅ Anti-aliasing suave
3. ✅ Texture atlas para fonts

### Debilidades Críticas
1. ❌ Sin control de fuente (family, weight, style)
2. ❌ Sin alineación de texto
3. ❌ Sin soporte SVG
4. ❌ Sin pencil/sketch tool

### Recomendación Principal

**Priorizar texto esencial (alineación + bold/italic)** antes de SVG, ya que texto es más usado en diagrams de arquitectura. SVG puede implementarse gradualmente.

| Priority | Action | Effort | Impact |
|----------|--------|--------|--------|
| P1 | Text alignment | Low | High |
| P1 | Bold/Italic | Low | High |
| P2 | Font family | Medium | Medium |
| P2 | SVG Export | Medium | High |
| P3 | SVG Import | High | Medium |
| P3 | Pencil tool | High | Low |
