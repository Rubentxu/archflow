# ArchFlow Engine: Especificación de Primitivas Universales

**Autor:** ArchFlow Team  
**Fecha:** 2026-01-23  
**Objetivo:** Definir un set de primitivas gráficas "Universal" capaz de soportar casos de uso tipo Figma, Draw.io, Excalidraw y CAD ligero.

---

## 1. Filosofía del Engine

El motor de renderizado debe ser **agnóstico al backend**. Las primitivas son descripciones de datos puros que el renderer interpreta.

*   **Data-Driven:** Todo es serializable.
*   **Layered:** Separación estricta entre Geometría (Forma) y Estilizado (Apariencia).
*   **Extensibles:** Soporte para "Compound Shapes" (formas compuestas por otras formas).
*   **Mode-Aware:** Capaz de renderizar en modo "Exact" (Figma/CAD) o "Rough" (Excalidraw/Sketch).

---

## 2. Catálogo de Primitivas (Primitive Catalog)

El corazón del engine es el enum `Primitive`.

### 2.1 Primitivas Básicas (Core)

Estas son las unidades atómicas de renderizado.

| Primitiva | Descripción | Propiedades Clave | Prioridad |
|-----------|-------------|-------------------|-----------|
| **Rect** | Rectángulo | `width`, `height`, `corner_radii` (top-l, top-r, bot-r, bot-l) | Alta |
| **Ellipse** | Círculo/Elipse | `radius_x`, `radius_y`, `start_angle`, `end_angle` | Alta |
| **Line** | Segmento simple | `p1`, `p2` | Alta |
| **Polyline** | Línea quebrada | `points: Vec<Vec2>`, `is_closed` | Alta |
| **Path** | Curvas arbitrarias | `commands: Vec<PathCommand>` (M, L, Q, C, Z) | Critica |
| **Text** | Texto rico | `content`, `font`, `size`, `align`, `bounds`, `wrap` | Critica |
| **Image** | Bitmaps | `source`, `bounds`, `fit_mode` (cover, contain), `filters` | Media |

### 2.2 Primitivas de Diagramación (Diagram Shapes)

Formas semánticas comunes en arquitectura y diagramas de flujo. Se pueden implementar como `Path` pre-calculados, pero tenerlos como tipos nativos facilita la edición (ej: cambiar el "grosor" de una nube).

| Primitiva | Descripción | Propiedades Específicas |
|-----------|-------------|-------------------------|
| **Arrow** | Flecha conectora | `points`, `bend_mode` (elbow, curved), `start_cap`, `end_cap` |
| **Diamond** | Decisión | `bounds` |
| **Cylinder** | Base de datos | `bounds`, `perspective_ratio` |
| **Cloud** | Nube abstracta | `bounds`, `bump_count` |
| **Cube** | Cubo 3D falso | `bounds`, `depth` |
| **Actor** | Stick figure | `bounds` |

---

## 3. Sistema de Estilizado

El estilo se aplica a cualquier primitiva.

```rust
struct Style {
    // Stroke
    stroke_color: Color,
    stroke_width: f32,
    stroke_style: StrokeStyle, // Solid, Dashed, Dotted
    stroke_cap: LineCap,       // Butt, Round, Square
    stroke_join: LineJoin,     // Miter, Round, Bevel
    
    // Fill
    fill_color: Option<Color>,
    fill_rule: FillRule,       // NonZero, EvenOdd
    fill_opacity: f32,
    
    // Effects
    shadow: Option<Shadow>,
    opacity: f32,
    blend_mode: BlendMode,
    
    // "Hand-drawn" capabilities (Excalidraw style)
    roughness: f32,            // 0.0 (Exact) -> 2.0 (Sketchy)
    bowing: f32,               // Curvatura de líneas rectas
    seed: u64,                 // Para determinismo en random noise
}
```

---

## 4. Estructuras de Datos Propuestas (Rust)

### Enums Principales

```rust
/// La unidad base de renderizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Primitive {
    // Basic Geometry
    Rect(RectPrimitive),
    Ellipse(EllipsePrimitive),
    Path(PathPrimitive),
    Text(TextPrimitive),
    
    // High-level Diagramming
    Connector(ConnectorPrimitive),
    Compound(CompoundPrimitive),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectPrimitive {
    pub bounds: Rect,
    pub corners: CornerRadii, // [f32; 4]
    pub style: StyleId,
}

/// Comando vectorial tipo SVG encapsulado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo(Vec2, Vec2),      // Control, End
    CubicTo(Vec2, Vec2, Vec2), // Control1, Control2, End
    ArcTo(Vec2, Vec2, f32),    // Point1, Point2, Radius
    Close,
}

/// Primitiva de conexión inteligente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorPrimitive {
    pub start: Vec2,
    pub end: Vec2,
    pub waypoints: Vec<Vec2>,
    pub routing: RoutingType,     // Straight, Orthogonal, Curved
    pub start_head: ArrowHead,    // None, Triangle, Circle, Diamond
    pub end_head: ArrowHead,
    pub style: StyleId,
}

/// Definición de cabeceras de flecha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrowHead {
    None,
    Triangle { width: f32, height: f32, filled: bool },
    Stealth { width: f32, height: f32 }, // Tipo avión
    Diamond { width: f32, filled: bool },
    Circle { radius: f32, filled: bool },
    Bar { width: f32 },
}

/// Para formas complejas (ej: un servidor físico)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundPrimitive {
    pub children: Vec<Primitive>,
    pub transform: Transform,
    pub clip_bounds: Option<Rect>, // Clipping opcional
}
```

---

## 5. Implementación de "Roughness" (Estilo Excalidraw)

Para que el motor sea universal, debe soportar generación procedural de geometría "imprecisa" en tiempo de teselación.

**Estrategia:**
1.  Si `roughness == 0`: Renderizar geometría exacta (GPU/Canvas directo).
2.  Si `roughness > 0`: Pasar la geometría por un procesador que:
    *   Subdivide líneas rectas en segmentos con ruido.
    *   Duplica trazos (hace 2 pasadas con ligero offset para efecto lápiz).
    *   Cambia rellenos sólidos por rellenos "Hachure" (rayado).

Esta lógica debe vivir en un módulo `archflow-renderer::rough`, separado del renderer principal, pero invocado por este antes de dibujar.

---

## 6. Hoja de Ruta de Implementación de Primitivas

### Fase 1: Primitivas Vectoriales Core (Inmediato)
*   [ ] Refactorizar `Renderer` para aceptar `enum Primitive`.
*   [ ] Migrar `draw_rect` / `draw_ellipse` a usar `Primitive::Rect` y `Primitive::Ellipse`.
*   [ ] Implementar `Primitive::Path` (esencial para cualquier forma compleja).
*   [ ] Implementar `PathBuilder` fluido.

### Fase 2: Conectores y Flechas (Siguiente Sprint)
*   [ ] Implementar `ConnectorPrimitive`.
*   [ ] Algoritmo básico de flechas (orientación automática de cabeceras basada en la tangente de la curva).
*   [ ] Renderizado de terminadores (ArrowHead).

### Fase 3: Diagramación Avanzada
*   [ ] Implementar `Cylinder` y `Cloud` usando `PathPrimitive`.
*   [ ] `CompoundPrimitive` para agrupar formas.

### Fase 4: Estilos Avanzados
*   [ ] Soporte para `DashArray`.
*   [ ] Efectos de sombra (Canvas `shadowBlur`).
*   [ ] (Opcional) Rough shader logic.

---

## 7. Comparativa con Referentes

| Feature | ArchFlow Engine (Objetivo) | Tldraw | Draw.io | Excalidraw |
|---------|----------------------------|--------|---------|------------|
| **Core** | Rust/WASM | TS/React | JS/mxGraph | TS/React |
| **Pathing** | GPU-ready Path | SVG | SVG/XML | RoughJS |
| **Connectors**| First-class citizen | Binding arrows | First-class | Binding arrows |
| **Perf** | 10k+ items | Medium | Low | Medium |
| **Mode** | Exact + Rough (Planned) | Exact/Hand | Exact | Rough Only |

Esta especificación permitirá que ArchFlow Engine sea extraído en el futuro como una librería independiente (`archflow-engine`) utilizable para pizarras blancas, herramientas de diagramación o editores visuales.
