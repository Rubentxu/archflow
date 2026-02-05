---
title: "Tests de Integración Headless para WebGPU y WebGL2 en Rust"
author: Claude Code
date: 2025-02-04
status: Completada
context: "Implementación de tests headless para validación de renderizado GPU"
iteration: 1
---

# Investigación: Tests Headless para WebGPU y WebGL2 en Rust

## Resumen Ejecutivo

Los tests de integración headless para WebGPU y WebGL2 representan uno de los mayores desafíos técnicos en el desarrollo de motores de renderizado modernos. Sin embargo, la industria ha desarrollado estrategias probadas que permiten validar el renderizado sin necesidad de ventanas visibles. El proyecto **wgpu** (base de ArchFlow) utiliza un enfoque multi-nivel que combina tests unitarios, tests de integración con `wgpu_test`, y la **WebGPU CTS** (Conformance Test Suite). Para **WebGL2 en WASM**, `wasm-pack test --headless --chrome` permite ejecutar tests en Chrome headless real.

## 1. Contexto y Objetivos

### 1.1 Estado Actual del Proyecto

**ArchFlow** utiliza dos backends de renderizado:
- **WebGL2**: Implementado en `crates/archflow-render/src/webgl2_renderer_real.rs`
- **WebGPU**: Implementado en `crates/archflow-render/src/webgpu_context.rs`

El renderer `GpuRenderer` en `renderer.rs` prepara datos para GPU pero no ejecuta renderizado real. La validación actual se limita a tests unitarios de estructuras de datos.

### 1.2 Módulos Afectados

| Módulo | Propósito | Tests Existentes |
|--------|-----------|------------------|
| `renderer.rs` | Preparación de datos GPU | Unit tests ✓ |
| `webgl2_renderer_real.rs` | WebGL2 real | Ninguno |
| `webgpu_context.rs` | WebGPU real | Ninguno |
| `texture_loader.rs` | Carga de texturas | Ninguno |

### 1.3 Interfaces y Contratos

```rust
// Trait Renderer que todos los backends implementan
pub trait Renderer {
    fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize;
    fn render(&mut self) -> Result<(), RenderError>;
    // ... otros métodos
}
```

## 2. Investigación Externa

### 2.1 WebGPU Headless Testing (wgpu)

**Fuentes consultadas:**
- [wgpu Testing Documentation](https://wgpu.rs/doc/wgpu_test/index.html)
- [wgpu GitHub Discussions #1611](https://github.com/gfx-rs/wgpu/discussions/1611)
- [wgpu CTS Testing](https://github.com/gfx-rs/wgpu/blob/trunk/docs/testing.md)

**Estrategias principales:**

1. **wgpu_test Utilities**: El repositorio de wgpu proporciona `wgpu_test` con helpers para:
   - Inicialización de dispositivos GPU en tests
   - Render-to-texture para validación de píxeles
   - Comparación de snapshots

2. **WebGPU CTS**: Suite de pruebas de conformidad oficial:
   ```bash
   cargo xtask cts                    # Tests por defecto
   cargo xtask cts 'webgpu:api,operation,*'  # Tests específicos
   ```

3. **Render-to-Texture Pattern**:
   ```rust
   // Pseudocódigo del patrón
   let texture = device.create_texture(&wgpu::TextureDescriptor {
       size: (256, 256),
       format: wgpu::TextureFormat::Rgba8Unorm,
       usage: wgpu::TextureUsages::RENDER_ATTACHMENT | 
              wgpu::TextureUsages::COPY_SRC,
   });
   
   // Renderizar a textura
   encoder.copy_texture_to_buffer(&texture, &buffer);
   
   // Leer buffer desde CPU
   let data = buffer_slice.map_async().await;
   ```

### 2.2 WebGL2 Headless Testing (WASM)

**Fuentes consultadas:**
- [wasm-bindgen Testing Guide](https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/browsers.html)
- [wasm-pack test --headless](https://rustwasm.github.io/docs/wasm-pack/commands/test.html)

**Estrategias:**

1. **wasm-pack test con Chrome Headless**:
   ```bash
   wasm-pack test --headless --chrome
   ```

2. **OffscreenCanvas API** (requiere Chrome 69+):
   ```rust
   #[wasm_bindgen_test]
   fn test_webgl2_rendering() {
       let canvas = offscreen_canvas().unwrap();
       let gl = canvas.get_context("webgl2").unwrap();
       // ...
   }
   ```

3. **Comparación con Gold Images**:
   - Guardar imagen de referencia en `tests/snapshots/`
   - Comparar bytes del canvas con imagen de referencia
   - Usar crate `image` para comparación

### 2.3 Snapshot Testing para Renderizado

| Herramienta | Uso | Ventajas |
|-------------|-----|----------|
| `image` crate | Cargar/guardar PNG | Soporte amplio |
| `pixelmatch` (JS) | Comparación visual | Detección de subpíxeles |
| `resemble.js` | Diff visual | Heatmaps de diferencias |

## 3. Análisis de Encaje

### 3.1 Compatibilidad con Arquitectura Actual

| Aspecto | Propuesta | Código Actual | Gap | Solución |
|---------|-----------|---------------|-----|----------|
| Backend | GpuRenderer + GPU backends | GpuRenderer existente | Bajo | Implementar backends GPU |
| Trait Renderer | Mismo trait | Ya existe | Ninguno | Usar trait existente |
| Tests unitarios | GpuRenderer tests | Existen | Ninguno | Extender |
| Tests integración | Nuevos | No existen | Alto | Crear infraestructura |

### 3.2 Impacto de Cambios

**Breaking Changes Identificados:**
- Ninguno. Los tests son adiciones ortogonales.

**Cambios necesarios:**
- Nuevo módulo `tests/integration.rs` en `archflow-render`
- Dependencia `image` crate para snapshots
- Configuración de CI para tests GPU

### 3.3 Matriz de Compatibilidad

| Criterio | Peso | Puntuación | Justificación |
|----------|------|-------------|---------------|
| Fit arquitectónico | 25% | 5 | Trait Renderer ya diseñado |
| Complejidad de cambio | 20% | 3 | requiere nueva infraestructura |
| Mantenibilidad | 20% | 4 | Tests automáticos previenen regresiones |
| Performance | 15% | 5 | Tests headless no afectan runtime |
| Seguridad | 10% | 5 | Tests de solo-lectura |
| Team expertise | 10% | 4 | Curva de aprendizaje moderada |

## 4. Infraestructura de CI Recomendada

### 4.1 GitHub Actions para WebGPU

```yaml
# .github/workflows/gpu-tests.yml
name: GPU Tests

on: [push, pull_request]

jobs:
  wgpu-tests:
    runs-on: ubuntu-latest
    container: ghcr.io/gfx-rs/ci:latest
    steps:
      - uses: actions/checkout@v4
      - name: Run wgpu CTS
        run: cargo xtask cts
      - name: Run unit tests
        run: cargo test --workspace

  # SwiftShader para máquinas sin GPU
  swiftshader-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install SwiftShader
        run: apt-get install -y vulkan-tools
      - name: Run tests with software rendering
        env: WGPU_BACKEND: vulkan
        run: cargo test
```

### 4.2 GitHub Actions para WebGL2 WASM

```yaml
# .github/workflows/wasm-tests.yml
name: WASM Tests

on: [push, pull_request]

jobs:
  webgl2-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Chrome/Chromedriver
        uses: browser-actions/setup-chrome@latest
      - name: Run wasm-pack tests
        run: |
          rustup target add wasm32-unknown-unknown
          cargo install wasm-pack
          wasm-pack test --headless --chrome
```

## 5. Evaluación de Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Tests flake en CI | Media | Medio | Reintentos, gold images estables |
| Compatibilidad browsers | Baja | Alto | Tests multi-browser (Chrome/Firefox) |
| GPU hardware diversity | Media | Medio | SwiftShader como fallback |
| Performance CI | Baja | Bajo | Tests paralelos, cacheo |

## 6. Recomendación

### 6.1 Decisión

✅ **APROBAR** - Implementar infraestructura de tests headless

### 6.2 Justificación

1. **Trait Renderer ya existe**: El diseño actual permite agregar backends sin modificar código existente
2. **wgpu proporciona herramientas**: `wgpu_test` y CTS son infraestructura probada
3. **wasm-pack soporta headless**: Tests WebGL2 pueden ejecutarse en CI
4. **ROI positivo**: Tests automáticos previenen regresiones costosas

### 6.3 Condiciones

1. **Fase 1**: Tests unitarios extendidos (existente)
2. **Fase 2**: Tests de integración WebGL2 con wasm-pack
3. **Fase 3**: Tests WebGPU con wgpu_test
4. **Fase 4**: Configuración CI/CD completa

### 6.4 Próximos Pasos

1. Crear `crates/archflow-render/tests/integration.rs`
2. Añadir dependencia `image` crate
3. Implementar test de snapshot básico
4. Configurar GitHub Actions
5. Documentar procedimiento de gold images

### 6.5 Riesgos Residuales

- **CI sin GPU**: Usar SwiftShader como fallback
- **Gold images desactualizados**: Proceso de revisión en PR

## 7. Resumen de Cambios Necesarios

### Archivos a Crear

| Archivo | Propósito |
|---------|-----------|
| `crates/archflow-render/tests/integration.rs` | Tests de integración GPU |
| `crates/archflow-render/tests/snapshots/` | Imágenes de referencia |

### Dependencias a Añadir

| Dependencia | Uso | Impacto |
|-------------|-----|---------|
| `image = "0.25"` | Comparación de snapshots | Bajo |
| `wgpu-test` (custom) | Utilidades GPU | Bajo |

### Crates Modificados

| Crate | Cambio | Impacto |
|-------|--------|---------|
| `archflow-render` | Añadir tests | Bajo |
| `archflow-web` | Configurar wasm-pack test | Bajo |

## 8. Propuestas de Mejora

### 8.1 Quick Wins

| Propuesta | Esfuerzo | Impacto |
|-----------|----------|---------|
| Añadir test de snapshot básico | 1 día | Alto |
| Documentar gold images | 0.5 días | Medio |

### 8.2 Medium Term

| Propuesta | Esfuerzo | Impacto |
|-----------|----------|---------|
| Integración wgpu_test | 1 semana | Alto |
| CI con GPU | 2 semanas | Alto |

### 8.3 Long Term

| Propuesta | Esfuerzo | Impacto |
|-----------|----------|---------|
| WebGPU CTS integration | 1 mes | Muy Alto |

## 9. Pensamiento Lateral

### 9.1 ¿Y si...

**¿Y si validáramos por datos en lugar de píxeles?**

En lugar de comparar imágenes, validaríamos los datos enviados a la GPU:
- `GpuInstance` bytes antes de upload
- Shaders compilados y sus hashes
- Call counts de draw calls

**Ventaja**: Tests más rápidos, sin gold images
**Desventaja**: No valida rendering real

### 9.2 Patrón de "GPU Mock"

```rust
// NullRenderer para tests de lógica
struct MockRenderer {
    captured_instances: Vec<GpuInstance>,
    draw_call_count: u32,
}

impl Renderer for MockRenderer {
    fn render(&mut self) -> Result<(), RenderError> {
        // Solo registra, no renderiza
        self.draw_call_count += 1;
        Ok(())
    }
}

// Test de integración lógica
#[test]
fn test_batch_organization() {
    let mut renderer = MockRenderer::new();
    let store = create_test_store();
    let camera = Camera::new(800, 600);
    
    renderer.sync_from_store(&store, &camera);
    
    assert_eq!(renderer.draw_call_count, 4); // Shapes, Icons, Images, Text
    assert!(!renderer.captured_instances.is_empty());
}
```

## 10. Deuda Técnica Identificada

| Item | Severity | Descripción |
|------|----------|-------------|
| Tests GPU missing | ALTA | Sin validación de renderizado real |
| Gold images missing | MEDIA | Sin baseline para comparación |
| CI GPU no configurado | MEDIA | Sin tests automatizados |

## Apéndice A: Ejemplo de Test de Snapshot

```rust
// crates/archflow-render/tests/webgl2_snapshot.rs
use wasm_bindgen_test::*;
use archflow_render::WebGL2Renderer;

#[wasm_bindgen_test]
async fn test_webgl2_rectangle_rendering() {
    // Crear canvas offscreen
    let canvas = offscreen_canvas().unwrap();
    canvas.set_size(256, 256);
    
    // Crear renderer
    let renderer = WebGL2Renderer::new(canvas.clone()).unwrap();
    
    // Renderizar rectángulo de prueba
    let store = create_test_store_with_rect();
    let camera = create_test_camera();
    renderer.sync_from_store(&store, &camera);
    renderer.render().unwrap();
    
    // Extraer píxeles
    let pixels = canvas.read_pixels();
    
    // Comparar con gold image
    let gold = image::open("tests/snapshots/rect.png")
        .unwrap()
        .to_rgba8();
    
    assert_eq!(pixels, gold);
}
```

## Apéndice B: Referencias

- [wgpu Testing](https://wgpu.rs/doc/wgpu_test/index.html)
- [WebGPU CTS](https://github.com/gpuweb/cts)
- [wasm-pack test](https://rustwasm.github.io/docs/wasm-pack/commands/test.html)
- [Rust Wasm Testing Guide](https://rustwasm.github.io/docs/wasm-bindgen/wasm-bindgen-test/browsers.html)
