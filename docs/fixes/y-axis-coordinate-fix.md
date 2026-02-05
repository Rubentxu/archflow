# Corrección del Sistema de Coordenadas del Eje Y

## Problema

El sistema tenía un problema de inversión del eje Y que causaba que las formas se crearan en posiciones incorrectas, especialmente notable al arrastrar círculos hacia abajo (aparecían hacia arriba).

### Síntomas

- Al crear círculos arrastrando hacia abajo, la forma aparecía hacia arriba
- Los rectángulos se posicionaban correctamente después de corregir la doble resta de coordenadas, pero los círculos seguían con comportamiento invertido
- Inconsistencia entre coordenadas de pantalla y coordenadas del mundo

## Causa Raíz

El problema estaba en la función `screen_to_world` en `camera.rs`. El código convertía coordenadas de pantalla a NDC (Normalized Device Coordinates) pero **no invertía el eje Y**.

### Convenciones de Coordenadas

- **Sistema de Pantalla (DOM)**: Y=0 arriba, Y aumenta hacia abajo
- **Sistema del Mundo (matemática estándar)**: Y aumenta hacia arriba

### Código Incorrecto

```rust
let ndc = (screen_pos / screen_size) * 2.0 - Vec2::ONE;
```

Esto producía:
- Screen Y=0 (arriba) → NDC Y=-1 → World Y negativo ❌
- Screen Y=max (abajo) → NDC Y=+1 → World Y positivo ❌

## Solución

Se invirtió el eje Y en `screen_to_world` y `world_to_screen`:

### `screen_to_world` (camera.rs L176-193)

```rust
pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2f64 {
    // Normalize to device normalized coordinates (NDC) [-1, 1]
    let mut ndc = (screen_pos / screen_size) * 2.0 - Vec2::ONE;

    // Invert Y-axis: Screen increases downward, World increases upward
    ndc.y = -ndc.y;

    // Calculate viewport height in world units
    let viewport_height = screen_size.y as f64 / self.zoom as f64;
    let half_height = viewport_height / 2.0;
    let half_width = half_height * self.aspect_ratio as f64;

    Vec2f64::new(
        self.center.x + ndc.x as f64 * half_width,
        self.center.y + ndc.y as f64 * half_height,
    )
}
```

### `world_to_screen` (camera.rs L196-217)

```rust
pub fn world_to_screen(&self, world_pos: Vec2f64, screen_size: Vec2) -> Vec2 {
    let viewport_height = screen_size.y as f64 / self.zoom as f64;
    let half_height = viewport_height / 2.0;
    let half_width = half_height * self.aspect_ratio as f64;

    // World coordinates to NDC [-1, 1]
    let mut ndc = Vec2::new(
        ((world_pos.x - self.center.x) / half_width) as f32,
        ((world_pos.y - self.center.y) / half_height) as f32,
    );

    // Invert Y-axis: World increases upward, Screen increases downward
    ndc.y = -ndc.y;

    // NDC [-1, 1] to Screen [0, width/height]
    (ndc + Vec2::ONE) * 0.5 * screen_size
}
```

## Resultado

Ahora el sistema funciona correctamente:
- Screen Y=0 (arriba) → NDC Y=-1 → **invertido** → NDC Y=+1 → World +Y ✅
- Screen Y=max (abajo) → NDC Y=+1 → **invertido** → NDC Y=-1 → World -Y ✅

### Comportamiento Esperado

- Hacer clic arriba en pantalla crea formas con coordenadas Y positivas en mundo
- Hacer clic abajo en pantalla crea formas con coordenadas Y negativas en mundo
- Arrastrar hacia abajo hace crecer la forma hacia abajo (como se espera visualmente)
- Los círculos y rectángulos se comportan consistentemente

## Tests Actualizados

Se actualizaron todos los tests para reflejar la convención correcta:

1. `test_screen_to_world`: Ahora verifica que screen (100, 0) → world (+, +)
2. `test_e2e_mouse_to_world_hit_test`: Coordenadas ajustadas para la inversión Y
3. `test_pixels_per_unit_consistency`: Top-left ahora mapea a (-400, +300)

Todos los tests pasan exitosamente.

## Archivos Modificados

- `crates/archflow-render/src/camera.rs`: Inversión del eje Y en conversiones
- `crates/archflow-web/src/bridge.rs`: Limpieza de logs de debug

## Verificación

Para verificar que el fix funciona correctamente:

1. Compilar: `just build-wasm`
2. En el navegador, seleccionar herramienta de rectángulo o círculo
3. Hacer clic y arrastrar hacia abajo
4. La forma debe crecer en la dirección del arrastre
5. Las coordenadas del mundo deben ser coherentes con la posición visual

## Notas Técnicas

- La inversión se realiza en la capa de conversión de coordenadas (camera)
- El shader no necesita cambios (trabaja en coordenadas del mundo)
- La lógica de creación de formas (min/max) funciona correctamente con la nueva convención
- Los tests garantizan que la conversión es consistente en ambas direcciones (roundtrip)

---

**Fecha**: 2025-02-05  
**Autor**: Sistema de corrección automática  
**Estado**: ✅ Completado y verificado