# ArchFlow Engine: Revisión Final y Refinamiento (v1.5)

**Fecha:** 2026-01-23
**Estado:** Análisis Pre-Implementación
**Referencia:** `ENGINE-2D-ANALYSIS.md` (v1.5)

Este documento realiza una revisión de "Sanidad Técnica" sobre la arquitectura propuesta para asegurar que soporta los casos de uso avanzados de "Figma for Architects" antes de empezar a escribir código masivo.

---

## 1. Crítica Constructiva

### 1.1 El Problema de la Jerarquía (Groups & Containers)
El documento menciona `Parent` y `Children` en los componentes, pero **no define cómo se propagan las transformaciones**.
*   **Riesgo:** En un modelo C4, si mueves un "Container" (padre), todos los "Components" (hijos) deben moverse visualmente, pero sus coordenadas locales no deberían cambiar.
*   **Faltante:** Falta el concepto de **`GlobalTransform`** (coordenadas de mundo calculadas) vs **`Transform`** (coordenadas locales). Sin esto, el rendering anidado será imposible o muy lento de calcular en cada frame.

### 1.2 Flujo de Datos: Store vs ECS
El diagrama muestra `Store (Delta)` separado de `ECS Core`.
*   **Conflicto:** Si el ECS tiene el estado vivo (posición, selección), ¿cuándo se actualiza el Store para Undo/Redo?
*   **Riesgo:** Desincronización. Si arrastro un nodo, ¿se actualiza el Store en cada frame (lento) o al soltar (mouse up)?
*   **Propuesta:** Definir claramente el **Ciclo de Acción**:
    1.  *Input*: Dragging muda el ECS `Transform` (feedback visual instantáneo).
    2.  *Commit*: Al soltar (`PointerUp`), se emite un `Command::MoveEntity`.
    3.  *Store*: El Store registra la transacción y actualiza su estado "persistente".
    4.  *Sync*: (Opcional) El ECS se reconcilia con el Store si hubo cambios externos (colaboración).

### 1.3 Sistema de Eventos de Puntero
El documento menciona "Events" genéricamente. Para una herramienta de diagrama profesional, necesitamos un sistema de **Picking** robusto.
*   **Necesidad:** No basta con saber "qué entidad está bajo el mouse". Necesitamos saber si el click fue "consumido" por un hijo antes de llegar al padre (Event Bubbling/Capturing) o si bloquea la selección de otros objetos.

### 1.4 Routing Ortogonal (Manhattan)
Proponer A* (A-Star) sobre una grilla virtual para conexiones es computacionalmente costoso para un MVP en tiempo real (60fps) si hay muchos obstáculos.
*   **Mejora:** Sugerir una heurística más simple para el MVP: **"Doorway" Routing** o ruteo basado en bounding boxes expandidos, en lugar de pathfinding completo de grilla.

---

## 2. Propuestas Técnicas de Mejora

### 2.1 Refinamiento de Transformaciones (ECS)

Añadir un sistema explícito de propagación de transformadas.

```rust
// archflow-ecs/src/components.rs

/// Transformada Local (relativa al padre)
#[derive(Component)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

/// Transformada Global (calculada frame a frame, usada por el Renderer)
#[derive(Component)]
pub struct GlobalTransform {
    pub matrix: Mat3, // Matriz 3x3 de transformación afín 2D
    pub z_index: i32, // Profundidad calculada acumulada
}
```

**Sistema de Propagación:**
Un sistema que recorre la jerarquía (usando `Parent`/`Children`) y multiplica las matrices: `GlobalTransform(Child) = GlobalTransform(Parent) * Transform(Child)`.

### 2.2 Estrategia de Rendering para Canvas 2D
Para evitar saturar el Canvas API con 5,000 llamadas `ctx.stroke()` por frame:

1.  **Culling Agresivo:** Usar el R-Tree para dibujar *solo* lo que intercepta con el Viewport.
2.  **Path Caching:** Para formas complejas estáticas (como íconos de AWS SVG), cachear el `Path2D` en un componente `RenderCache` para no reconstruirlo en cada frame.
3.  **Dirty Rectangles:** (Opcional para MVP, pero bueno tenerlo en mente) Solo limpiar y redibujar la región de la pantalla que cambió.

### 2.3 Definición de "Port" (Puerto de Conexión)
Definir cómo se calculan los puntos de anclaje dinámicos.

```rust
pub enum PortBinding {
    /// Punto fijo relativo al centro (ej: (0.5, 0.0) es derecha centro)
    Fixed(Vec2),
    /// Punto dinámico que se mueve al borde más cercano a la otra entidad
    AutoClosest,
    /// Punto dinámico alineado a la brújula (North, South, East, West)
    AutoCompass,
}
```

---

## 3. Conclusión de la Revisión

El análisis `ENGINE-2D-ANALYSIS.md` es **aprobado para implementación** con la condición de integrar el sistema de **GlobalTransform** (Jerarquía) desde el día 1. Sin jerarquía robusta, el modelo C4 (Zoom semántico) no funcionará.

**Siguiente paso recomendado:** Implementar el esqueleto del ECS con `World`, `Transform` (Local/Global) y el `Renderer` básico conectado a Canvas.
