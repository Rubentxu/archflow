# Shader Rendering E2E Tests

Este directorio contiene tests end-to-end (E2E) para validar el renderizado de shaders WebGL2 con SDF (Signed Distance Fields).

## 🎯 Objetivo

Validar que los shaders renderizan correctamente círculos, rectángulos y otras formas con:
- **SDF correcto** - Bordes suaves y anti-aliasing
- **Colores correctos** - Validación de formato RGBA/ABGR
- **Geometría precisa** - Verificación de que círculos son circulares, no cuadrados

## 📁 Archivos

- `shader_rendering_e2e.rs` - Tests E2E que renderizan en un canvas WebGL2 real
- `webgl2_snapshot_tests.rs` - Tests de snapshot que validan datos de instancias
- `parity_tests.rs` - Tests de paridad entre diferentes backends

## 🚀 Ejecución Rápida

### Ejecutar todos los tests de shaders E2E

```bash
just test-shader-e2e
```

### Ejecutar con navegador visible (debugging)

```bash
just test-shader-e2e-headed
```

### Ejecutar en Chrome en lugar de Firefox

```bash
just test-shader-e2e-chrome
```

## 🔧 Ejecución Manual

### Prerrequisitos

1. **Instalar wasm-pack**:
   ```bash
   cargo install wasm-pack
   ```

2. **Instalar geckodriver** (Firefox):
   ```bash
   # Ubuntu/Debian
   sudo apt install firefox-geckodriver
   
   # macOS
   brew install geckodriver
   
   # Arch Linux
   sudo pacman -S geckodriver
   ```

3. **Instalar chromedriver** (opcional, para Chrome):
   ```bash
   # Ubuntu/Debian
   sudo apt install chromium-chromedriver
   
   # macOS
   brew install chromedriver
   ```

### Comandos

```bash
# Desde el directorio raíz del proyecto
cd crates/archflow-render

# Ejecutar tests en headless Firefox
wasm-pack test --headless --firefox

# Ejecutar tests en headless Chrome
wasm-pack test --headless --chrome

# Ejecutar tests con navegador visible (útil para debugging)
wasm-pack test --firefox

# Ejecutar un test específico
wasm-pack test --headless --firefox -- test_render_single_blue_circle
```

## 📊 Tests Disponibles

### 1. `test_shader_compilation_succeeds`
Verifica que los shaders de vertex y fragment compilan sin errores.

**Objetivo**: Detectar errores de sintaxis GLSL antes de runtime.

### 2. `test_render_single_red_rectangle`
Renderiza un rectángulo rojo en el centro del canvas.

**Validaciones**:
- ✅ Canal rojo > 200
- ✅ Canales verde y azul < 50
- ✅ Geometría rectangular correcta

### 3. `test_render_single_blue_circle`
Renderiza un círculo azul usando SDF.

**Validaciones**:
- ✅ Canal azul > 200 en el centro
- ✅ Esquinas negras (fuera del círculo por SDF)
- ✅ Círculo circular, no cuadrado

### 4. `test_circle_has_smooth_edges`
Valida que los círculos tienen anti-aliasing correcto.

**Validaciones**:
- ✅ Valores de alpha parciales en los bordes (50-250)
- ✅ Transición suave, no abrupta
- ✅ Bordes suavizados con `smoothstep` y `fwidth`

## 🐛 Debugging

### Ver el canvas renderizado

Para ver qué está renderizando realmente:

```bash
wasm-pack test --firefox  # Sin --headless
```

Esto abrirá Firefox y podrás inspeccionar el canvas con DevTools.

### Capturar screenshots

Modifica el test para guardar la imagen:

```rust
#[wasm_bindgen_test]
fn test_debug_circle() {
    let canvas = create_test_canvas(512, 512);
    let gl = get_webgl2_context(&canvas);
    
    // ... renderizar círculo ...
    
    let pixels = capture_framebuffer(&gl, 512, 512);
    
    // Log a la consola para inspección
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "Center pixel RGB: ({}, {}, {})",
        pixels[center_idx],
        pixels[center_idx + 1],
        pixels[center_idx + 2]
    )));
}
```

### Verificar errores de WebGL

Los tests capturan errores de WebGL automáticamente:

```rust
let err = gl.get_error();
if err != WebGl2RenderingContext::NO_ERROR {
    panic!("WebGL Error: {}", err);
}
```

## 📝 Agregar Nuevos Tests

### Template básico

```rust
#[wasm_bindgen_test]
fn test_my_new_shape() {
    let canvas = create_test_canvas(256, 256);
    let gl = get_webgl2_context(&canvas);
    let program = compile_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    
    // Setup
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.use_program(Some(&program));
    
    // Setup uniforms, buffers, instance data...
    
    // Render
    gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, 1);
    
    // Capture and assert
    let pixels = capture_framebuffer(&gl, 256, 256);
    assert!(/* your validation */);
}
```

### Formato de color (importante!)

Los colores en el shader usan **ABGR little-endian** debido a `unpack4x8unorm`:

```rust
// ❌ INCORRECTO (RGBA)
let red_color = 0xFF0000FF;

// ✅ CORRECTO (ABGR)
let red_color = 0xFF0000FF;   // Alpha=FF, Blue=00, Green=00, Red=FF
let blue_color = 0xFFFF0000;  // Alpha=FF, Blue=FF, Green=00, Red=00
let green_color = 0xFF00FF00; // Alpha=FF, Blue=00, Green=FF, Red=00
```

### Shape types

```rust
const SHAPE_RECT: u32 = 0;
const SHAPE_CIRCLE: u32 = 1;
const SHAPE_ELLIPSE: u32 = 2;
const SHAPE_LINE: u32 = 3;
const SHAPE_ROUNDED_RECT: u32 = 4;
```

## 🎓 Buenas Prácticas

### ✅ DO

- **Test formas individuales** - Un test, una forma
- **Validar anti-aliasing** - No solo colores planos
- **Testear casos extremos** - Tamaños muy pequeños/grandes
- **Usar tolerancias** - Permitir diferencias de ±5 en RGB
- **Capturar framebuffer** - No asumir, verificar

### ❌ DON'T

- **No hardcodear valores exactos** - Anti-aliasing causa variaciones
- **No ignorar esquinas** - Son clave para validar SDF de círculos
- **No asumir precisión perfecta** - GPU tiene errores de float32
- **No testear todo en un solo test** - Separa responsabilidades

## 📚 Referencias

- [wasm-bindgen-test docs](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html)
- [WebGL2 Reference](https://www.khronos.org/registry/webgl/specs/latest/2.0/)
- [SDF Rendering Guide](https://iquilezles.org/articles/distfunctions2d/)
- [GLSL Smoothstep](https://thebookofshaders.com/glossary/?search=smoothstep)

## 🚨 Troubleshooting

### Error: "geckodriver not found"

```bash
sudo apt install firefox-geckodriver
# o
brew install geckodriver
```

### Error: "WebGL context creation failed"

Verifica que tienes aceleración de hardware:

```bash
# Linux: verificar que mesa está instalado
glxinfo | grep "OpenGL"
```

### Tests pasan localmente pero fallan en CI

Usa headless mode y configura CI con Firefox:

```yaml
# .github/workflows/test.yml
- name: Install geckodriver
  run: |
    sudo apt-get update
    sudo apt-get install -y firefox-geckodriver
    
- name: Run shader tests
  run: just test-shader-e2e
```

### Performance lenta

Los tests WASM son más lentos que tests nativos. Para acelerar:

```bash
# Solo ejecutar tests específicos
wasm-pack test --headless --firefox -- test_shader_compilation

# Usar release mode (más rápido pero sin símbolos de debug)
wasm-pack test --headless --firefox --release
```

## 💡 Tips

1. **Desarrollo iterativo**: Usa `--firefox` (sin headless) para ver qué estás renderizando
2. **Log a consola**: `web_sys::console::log_1()` funciona en tests
3. **Snapshot golden images**: Guarda capturas buenas y compara con `compare_images()`
4. **CI/CD**: Ejecuta estos tests en cada PR para detectar regresiones visuales

## 🎯 Roadmap

- [ ] Golden image comparison con imágenes de referencia
- [ ] Tests de performance/benchmark de renderizado
- [ ] Tests de estrés con 1000+ formas
- [ ] Validación de z-ordering
- [ ] Tests de alpha blending
- [ ] Snapshot tests con diferentes GPUs