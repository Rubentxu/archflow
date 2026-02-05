# Análisis Comparativo: archflow-render vs Figma/Excalidraw/Tldraw

> Fecha: 2026-02-04 | Versión: v0.41.0

## Resumen Ejecutivo

| Aspecto | archflow-render | Figma | Excalidraw | Tldraw |
|---------|----------------|-------|------------|--------|
| **Formas básicas** | ✅ 4 tipos | ✅ 8+ tipos | ✅ 10+ tipos | ✅ 12+ tipos |
| **Stroke** | Solo color | ✅ Width + Style + Caps | ✅ Doble línea | ✅ Width + Style |
| **Fill** | Solo color sólido | ✅ Gradientes + Patrones | ✅ Punteado | ✅ Color + Opacity |
| **Corner Radius** | ✅ Basic | ✅ Por esquina + Smoothing | ✅ Por esquina | ❌ No nativo |
| **Efectos** | ❌ Shadow + Blur | ✅ Shadow + Layer Blends | ❌ Limitados | ❌ Limitados |
| **SVG/Path** | ❌ No | ✅ Vector networks | ✅ Line → Curve | ✅ Pencil → Shape |

---

## 1. Formas (Shape Types)

### 1.1 Estado Actual (`archflow-render`)

```wgsl
// Shape types soportados actualmente
const SHAPE_RECT: u32 = 0u;        // Rectángulo
const SHAPE_CIRCLE: u32 = 1u;      // Círculo
const SHAPE_ELLIPSE: u32 = 2u;     // Elipse
const SHAPE_LINE: u32 = 3u;        // Línea
const SHAPE_ROUNDED_RECT: u32 = 4u; // Rectángulo redondeado
```

### 1.2 Comparativa Detallada

| Forma | archflow-render | Figma | Excalidraw | Tldraw |
|-------|----------------|-------|------------|--------|
| Rectangle | ✅ | ✅ | ✅ | ✅ |
| Circle | ✅ | ✅ | ✅ | ✅ |
| Ellipse | ✅ | ✅ | ✅ | ✅ |
| Line/Arrow | ✅ | ✅ | ✅ | ✅ |
| Rounded Rect | ✅ (basic) | ✅ (per corner) | ✅ | ✅ |
| Polygon | ❌ | ✅ | ✅ | ✅ |
| Star | ❌ | ✅ | ✅ | ✅ |
| Arc | ❌ | ✅ | ❌ | ✅ |
| Triangle | ❌ | ✅ | ✅ | ✅ |
| Callout | ❌ | ❌ | ✅ | ✅ |
| Laser | ❌ | ❌ | ✅ | ❌ |
| Rectangle 3D | ❌ | ✅ | ❌ | ❌ |

### 1.3 Gap Analysis

```
PRIORIDAD ALTA:
├── Polygon (3+ lados)
├── Star (configurable points)
└── Triangle (equilateral, right, isosceles)

PRIORIDAD MEDIA:
├── Arc tool (pie charts, curved paths)
└── Candle (emoji-style stick figures)
```

---

## 2. Stroke (Trazo)

### 2.1 Estado Actual

```rust
// GpuInstance actual
pub struct GpuInstance {
    color: u32,  // Solo RGBA, sin stroke width/style
    // ...
}
```

**Limitación actual**: Stroke es implícito en el shape type (Line), sin personalización.

### 2.2 Comparativa Detallada

| Propiedad | archflow-render | Figma | Excalidraw | Tldraw |
|-----------|----------------|-------|------------|--------|
| **Color** | ✅ Simple | ✅ Multiple fills | ✅ Color | ✅ Color |
| **Width** | ❌ | ✅ 0.1px - 100px | ✅ 1px - 10px | ✅ 1px - |
| **Cap Style** | ❌ | ✅ Round/Butt/Square | ❌ | ❌ |
| **Join Style** | ❌ | ✅ Round/Miter/Bevel | ❌ | ❌ |
| **Dash Pattern** | ❌ | ✅ Custom dashes | ✅ Predefined | ✅ Dashed |
| **Double Line** | ❌ | ❌ | ✅ Line/Arrow | ❌ |
| **Triple Line** | ❌ | ❌ | ✅ | ❌ |
| **Arrowheads** | ❌ | ✅ Style + Size | ✅ Multiple | ✅ Style |

### 2.3 Gap Analysis

```
FALTA CRÍTICA:
├── Stroke width por entidad
├── Dash patterns (dashed, dotted)
└── Arrowheads para líneas
```

### 2.4 Implementación Requerida

```rust
// Propuesta: Extender GpuInstance para stroke
pub struct StrokeStyle {
    pub width: f32,           // Ancho del trazo
    pub dash_pattern: [f32; 4], // Patrón de guiones
    pub dash_offset: f32,     // Desplazamiento del patrón
    pub cap: StrokeCap,       // Butt, Round, Square
    pub join: StrokeJoin,     // Miter, Round, Bevel
}

pub enum StrokeCap {
    Butt,   // Fin plano
    Round,  // Extremo redondeado
    Square, // Extensión cuadrada
}

pub enum StrokeJoin {
    Miter,  // Esquina aguda
    Round,  // Esquina redondeada
    Bevel,  // Esquina achaflanada
}
```

---

## 3. Fill (Relleno)

### 3.1 Estado Actual

```rust
// Solo color sólido RGBA
pub color: u32,  // 0xRRGGBBAA
```

### 3.2 Comparativa Detallada

| Propiedad | archflow-render | Figma | Excalidraw | Tldraw |
|-----------|----------------|-------|------------|--------|
| **Solid Color** | ✅ | ✅ | ✅ | ✅ |
| **Linear Gradient** | ❌ | ✅ | ✅ | ✅ |
| **Radial Gradient** | ❌ | ✅ | ✅ | ✅ |
| **Angular Gradient** | ❌ | ✅ | ❌ | ❌ |
| **Diamond Gradient** | ❌ | ✅ | ❌ | ❌ |
| **Pattern Fill** | ❌ | ✅ | ❌ | ❌ |
| **Image Fill** | ❌ | ✅ | ✅ | ✅ |
| **Texture Fill** | ❌ | ✅ | ❌ | ❌ |
| **Opacity/Alpha** | ✅ | ✅ | ✅ | ✅ |
| **Hatch Pattern** | ❌ | Plugin | ✅ | ❌ |

### 3.3 Gap Analysis

```
FALTA IMPORTANTE:
├── Linear gradient (2+ colors, angle)
├── Radial gradient (center, radius)
└── Image/texture fill (url or asset)
```

### 3.4 Implementación Requerida

```rust
pub enum Fill {
    Solid(SolidFill),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

pub struct SolidFill {
    pub color: u32,
    pub opacity: f32,
}

pub struct LinearGradient {
    pub start_color: u32,
    pub end_color: u32,
    pub angle: f32,           // Grados (0-360)
    pub start_offset: f32,    // 0.0 - 1.0
    pub end_offset: f32,      // 0.0 - 1.0
}

pub struct RadialGradient {
    pub inner_color: u32,
    pub outer_color: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
}
```

---

## 4. Corner Radius (Radio de Esquina)

### 4.1 Estado Actual

```wgsl
// En SDF shader - radio embebido en shape_type
let radius = (input.shape_type >> 8u) & 0xFFu;
let radius_f = f32(radius) / 255.0 * min(size.x, size.y) * 0.5;
```

**Limitación**: Radio único para las 4 esquinas (0-255 discretizado).

### 4.2 Comparativa Detallada

| Propiedad | archflow-render | Figma | Excalidraw | Tldraw |
|-----------|----------------|-------|------------|--------|
| **Radio Uniforme** | ✅ | ✅ | ✅ | ✅ |
| **Radio Individual** | ❌ | ✅ (por esquina) | ✅ | ❌ |
| **Smoothing** | ❌ | ✅ (squircles) | ❌ | ❌ |
| **Interactive Handles** | ❌ | ✅ Drag | ✅ | ✅ |

### 4.3 Gap Analysis

```
MEJORA RECOMENDADA:
├── Radio por esquina (top-left, top-right, bottom-right, bottom-left)
├── Smoothing (bezier corners)
└── Visual handles para edición interactiva
```

---

## 5. Efectos y Estilos Avanzados

### 5.1 Estado Actual

**No implementado** - Solo rendering SDF básico.

### 5.2 Comparativa Detallada

| Efecto | archflow-render | Figma | Excalidraw | Tldraw |
|--------|----------------|-------|------------|--------|
| **Drop Shadow** | ❌ | ✅ | ❌ | ❌ |
| **Inner Shadow** | ❌ | ✅ | ❌ | ❌ |
| **Layer Blur** | ❌ | ✅ (Background blur) | ❌ | ❌ |
| **Layer Blend** | ❌ | ✅ (Multiply, Screen...) | ❌ | ❌ |
| **Neon Glow** | ❌ | ❌ | ✅ | ❌ |
| **Background Blur** | ❌ | ✅ | ❌ | ❌ |

### 5.3 Gap Analysis

```
BAJA PRIORIDAD (post-MVP):
├── Drop shadow (blur + offset)
├── Inner shadow (inset blur)
└── Layer blend modes (multiply, screen, overlay)
```

---

## 6. Texto y Labels

### 6.1 Estado Actual

```rust
// MTSDF text rendering implementado
// En archivo: mtsdf_text.wgsl
```

### 6.2 Comparativa Detallada

| Propiedad | archflow-render | Figma | Excalidraw | Tldraw |
|-----------|----------------|-------|------------|--------|
| **Font Rendering** | ✅ MTSDF | ✅ | ✅ | ✅ |
| **Size** | ✅ | ✅ | ✅ | ✅ |
| **Color** | ✅ | ✅ | ✅ | ✅ |
| **Bold/Italic** | ❌ | ✅ | ✅ | ✅ |
| **Font Family** | ❌ | ✅ | ✅ | ✅ |
| **Text Align** | ❌ | ✅ | ✅ | ✅ |
| **Text on Path** | ❌ | ✅ | ❌ | ✅ |
| **Auto-resize** | ❌ | ✅ (Auto-layout) | ❌ | ✅ |

---

## 7. Interactivity y UX

### 7.1 Estado Actual

```rust
// Hit testing implementado en camera_controller.rs
let mouse_world = camera.screen_to_world(mouse_screen, screen_size);
// Hit test básico con AABB
```

### 7.2 Comparativa Detallada

| Propiedad | archflow-render | Figma | Excalidraw | Tldraw |
|-----------|----------------|-------|------------|--------|
| **Selection** | ✅ Basic | ✅ | ✅ | ✅ |
| **Multi-select** | ❌ | ✅ | ✅ | ✅ |
| **Transform Handle** | ❌ | ✅ (9-point) | ✅ | ✅ |
| **Resize** | ❌ | ✅ | ✅ | ✅ |
| **Rotate** | ❌ | ✅ | ✅ | ✅ |
| **Corner Edit** | ❌ | ✅ | ✅ | ✅ |
| **Boolean Ops** | ❌ | ✅ | ❌ | ✅ |
| **Alignment Guides** | ❌ | ✅ | ✅ | ✅ |
| **Snap to Grid** | ❌ | ✅ | ✅ | ✅ |
| **Zoom to Fit** | ❌ | ✅ | ✅ | ✅ |

---

## 8. Roadmap de Implementación

### Fase 1: Mejoras Inmediatas (v0.42.0)

```markdown
## Mejoras de Stroke
- [ ] Stroke width en GpuInstance
- [ ] Dash patterns (dashed, dotted)
- [ ] Arrowheads para líneas

## Mejoras de Fill  
- [ ] Linear gradient (2 colors)
- [ ] Radial gradient
```

### Fase 2: Formas Extendidas (v0.43.0)

```markdown
## Nuevas Formas
- [ ] Polygon (3-8 lados)
- [ ] Star (configurable points)
- [ ] Triangle (equilateral, right)

## Corner Radius
- [ ] Radio por esquina individual
```

### Fase 3: Efectos y Texturas (v0.44.0)

```markdown
## Texturas
- [ ] Image fill
- [ ] Texture atlas expandido

## Efectos
- [ ] Drop shadow
- [ ] Layer blur
```

### Fase 4: Interactividad (v0.45.0)

```markdown
## UX
- [ ] Transform handles (9-point)
- [ ] Multi-select
- [ ] Boolean operations
- [ ] Alignment guides
```

---

## 9. Cambios Requeridos en GpuInstance

### 9.1 Layout Actual (48 bytes)

```rust
#[repr(C, align(16))]
pub struct GpuInstance {
    pos: [f32; 2],              // 8 bytes (offset 0)
    size: [f32; 2],             // 8 bytes (offset 8)
    color: u32,                  // 4 bytes (offset 16)
    shape_type: u32,             // 4 bytes (offset 20)
    _padding: [u32; 2],          // 8 bytes (offset 24)
    uv_rect: [f32; 4],           // 16 bytes (offset 32)
                                   // Total: 48 bytes
}
```

### 9.2 Propuesta Extendida (64 bytes)

```rust
#[repr(C, align(16))]
pub struct GpuInstance {
    // Posición y tamaño (16 bytes)
    pos: [f32; 2],
    size: [f32; 2],
    
    // Color (8 bytes)
    fill_color: u32,             // 0xRRGGBBAA
    stroke_color: u32,           // 0xRRGGBBAA (0 = no stroke)
    
    // Stroke properties (8 bytes)
    stroke_width: f32,           // Ancho del trazo
    dash_pattern: u32,           // 4x4 bits: dash, gap, dash, gap
    
    // Shape encoding (4 bytes)
    shape_type: u8,              // 0-15: tipo de forma
    corner_radius: u8,           // Radio (0-255)
    padding: [u8; 2],           // Reserved
    
    // Gradient/Texture (24 bytes)
    gradient_type: u8,           // 0=none, 1=linear, 2=radial
    gradient_stops: u8,          // Number of stops (2-4)
    gradient_start: [f32; 2],    // Start point (normalized)
    gradient_end: [f32; 2],      // End point (normalized)
    gradient_colors: [u32; 4],   // Up to 4 gradient colors
    
    // UV (8 bytes)
    uv_rect: [f32; 4],
    
                                   // Total: 64 bytes
}
```

---

## 10. Matriz de Priorización

| Feature | Impacto | Esfuerzo | Prioridad |
|---------|---------|----------|-----------|
| Stroke width | Alto | Bajo | P1 |
| Dash patterns | Medio | Medio | P2 |
| Arrowheads | Alto | Medio | P1 |
| Linear gradient | Alto | Medio | P1 |
| Polygon/Star | Medio | Medio | P2 |
| Corner per edge | Medio | Alto | P3 |
| Drop shadow | Bajo | Alto | P4 |
| Multi-select | Alto | Alto | P2 |

---

## 11. Conclusiones

### Fortalezas de archflow-render
1. ✅ Rendering SDF eficiente y escalable
2. ✅ Multi-phase rendering optimizado
3. ✅ Infinite canvas con viewport culling
4. ✅ Sistema de instancias GPU-friendly

### Áreas de Mejora
1. ⚠️ Stroke limitado a color simple
2. ⚠️ Sin gradientes ni patrones
3. ⚠️ Solo 4 tipos de formas básicas
4. ⚠️ Sin interactividad de transformación

### Recomendación Principal
**Implementar stroke + gradientes** antes de nuevas formas, ya que son más usados por usuarios de diagramas de arquitectura.
