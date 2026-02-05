# Épica: TEST-001 - Testing & Playground de Desarrollo

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-TEST-001 |
| Prioridad | Alta |
| Estimación | XL |
| Estado | Borrador |
| Versión | 0.1.0 |
| Versión Target | 0.45.0 |

## 🎯 Objetivo de Negocio

Crear una **suite de testing comprehensiva** y un **playground visual de desarrollo** que permita:
1. Validar todas las funcionalidades del motor con tests exhaustivos
2. Verificar shaders y render de manera visual e inequívoca
3. Hacer debugging rápido de problemas de render sin overhead
4. Tener confianza al 100% en el código antes de cada release

**Problema actual**: 
- Tests existentes son básicos, no cubren edge cases
- Shaders se "asumen" funcionando pero no hay verificación visual
- Render se testa con asserts de memoria, no con output real
- No hay forma rápida de probar cambios en shaders

**Solución propuesta**: Playground visual + suite de tests completos con casos límite.

---

## 📖 Historias de Usuario

### HU-TEST-001: Framework de Unit Tests Exhaustivos

**Como** desarrollador
**Quiero** tests unitarios que cubran todos los casos límite y edge cases
**Para** tener confianza total en el código

#### Criterios de Aceptación - EntityStore

**Spawn/Despawn:**
- [ ] Test: crear y destruir 1 entidad
- [ ] Test: crear 100k entidades, verificar alive_count
- [ ] Test: destruir entidades en orden aleatorio
- [ ] Test: intentar destruir entidad ya destruida (devuelve false)
- [ ] Test: intentar destruir con EntityId stale (generation mismatch)
- [ ] Test: EntityId válido después de destroy + respawn (generation increment)
- [ ] Test: reuse de índices del free_list
- [ ] Test: free_list comportamiento LIFO

**Transforms:**
- [ ] Test: spawn con posición 0,0
- [ ] Test: spawn con posición negativa
- [ ] Test: spawn con posición float grande (cerca de overflow)
- [ ] Test: move_by con delta 0
- [ ] Test: move_by con delta negativo grande
- [ ] Test: set_pos coincide con move_by
- [ ] Test: resize preserva posición
- [ ] Test: spawn con size 0 (caso edge)
- [ ] Test: spawn con size muy grande

**Metadata:**
- [ ] Test: shape type默认值 (Rectangle)
- [ ] Test: visibility toggle
- [ ] Test: selection toggle
- [ ] Test: layer set/get roundtrip
- [ ] Test: metadata bits no se corrompen entre operaciones
- [ ] Test: metadata aislados entre entidades (no cross-talk)

**Hierarchy:**
- [ ] Test: set_parent y clear_parent roundtrip
- [ ] Test: parent-child relación bidireccional
- [ ] Test: multiple children del mismo parent
- [ ] Test: nested hierarchy (10 niveles)
- [ ] Test: hierarchy cycle prevention (si aplica)
- [ ] Test: update_hierarchy con dirty flag
- [ ] Test: world_transform cálculo correcto con offset
- [ ] Test: detach hierarchy (parent = None) preserva world position

**Dirty Tracking:**
- [ ] Test: spawn marca dirty
- [ ] Test: move_by marca dirty_render
- [ ] Test: set_visible marca dirty_render
- [ ] Test: take_dirty_render_entities consume todos
- [ ] Test: double take no retorna nada
- [ ] Test: dirty_count accuracy
- [ ] Test: dirty flags se limpian correctamente

**Memory:**
- [ ] Test: alive_count准确地 después de mixed operations
- [ ] Test: draw_order contiene solo entidades vivas
- [ ] Test: draw_order orden consistente
- [ ] Test: MAX_ENTITIES enforcement

#### Criterios de Aceptación - DeltaMask

- [ ] Test: new() crea mask con todos bits en 0
- [ ] Test: toggle() cambia bit de 0→1→0
- [ ] Test: is_set()准确性
- [ ] Test: from_indices() set bits correctos
- [ ] Test: indices fuera de bounds se ignoran
- [ ] Test: count_ones()准确性
- [ ] Test: apply_to() XOR correcto
- [ ] Test: diff() calcula diferencia correcta
- [ ] Test: is_compatible() con capacities match/mismatch
- [ ] Test: clear() resetea a 0
- [ ] Test: capacity()准确性
- [ ] Test: len_bytes()准确性
- [ ] Test: memory usage = (capacity + 7) / 8 bytes
- [ ] Test: clone() preserva estado
- [ ] Test: serialize/deserialize roundtrip (si aplica)

#### Criterios de Aceptación - EventRingBuffer

- [ ] Test: new() crea buffer vacío
- [ ] Test: push() incrementa len
- [ ] Test: push() cuando full retorna false
- [ ] Test: overflow behavior (oldest overwritten)
- [ ] Test: drain() retorna todos y limpia
- [ ] Test: drain() en empty retorna []
- [ ] Test: peek() no modifica buffer
- [ ] Test: len() y is_empty() accuracy
- [ ] Test: capacity()准确性
- [ ] Test: is_full() accuracy
- [ ] Test: clear() empty el buffer
- [ ] Test: lost_count() accuracy
- [ ] Test: 10k pushes sin leak
- [ ] Test: event order preservado (FIFO)

#### Criterios de Aceptación - Command System

- [ ] Test: Command inverse roundtrip
- [ ] Test: Command::Select con delta vacío
- [ ] Test: Command::Select con delta full
- [ ] Test: Command::Move inverse es -delta
- [ ] Test: Command::Teleport inverse usa posición actual
- [ ] Test: Command::Select inverse es mismo mask (XOR)
- [ ] Test: CommandQueue push/pop
- [ ] Test: CommandQueue full behavior
- [ ] Test: CommandHistory undo/redo stack
- [ ] Test: CommandHistory empty undo

#### Estimación: L
#### Estado: Pendiente

---

### HU-TEST-002: Integration Tests End-to-End

**Como** sistema de integración
**Quiero** tests que ejerciten flujos completos
**Para** verificar que los componentes funcionan juntos

#### Criterios de Aceptación

**Selection Flow:**
- [ ] Test: crear entidades → seleccionar una → verificar selección
- [ ] Test: crear entidades → seleccionar múltiples → verificar count
- [ ] Test: seleccionar → deseleccionar → verificar clear
- [ ] Test: seleccionar en modo Multi → toggle behavior
- [ ] Test: seleccionar en modo Single → clear previous
- [ ] Test: undo después de selection
- [ ] Test: redo después de undo selection

**MoveGroup Flow:**
- [ ] Test: crear jerarquía → move root → verificar children moved
- [ ] Test: move hierarchy → undo → verificar restore
- [ ] Test: move hierarchy con delta 0 → no-op
- [ ] Test: nested hierarchy (5 niveles) → move deepest → verificar propagates up

**Spawn/Despawn Flow:**
- [ ] Test: spawn → verify alive → despawn → verify not alive
- [ ] Test: spawn hierarchy → despawn parent → children still alive
- [ ] Test: spawn hierarchy → despawn all → verify clean state
- [ ] Test: despawn while selected → selection updates

**Event Flow:**
- [ ] Test: selection → poll_events() → verify EntitySelected event
- [ ] Test: despawn → poll_events() → verify EntityDestroyed event
- [ ] Test: multiple events → drain → verify order

**Dirty Propagation Flow:**
- [ ] Test: modify entity → take_dirty_render_entities → count correct
- [ ] Test: modify parent → hierarchy dirty propagates
- [ ] Test: modify many entities → dirty_count accuracy

#### Estimación: M
#### Estado: Pendiente

---

### HU-TEST-003: Shader Playground Visual

**Como** desarrollador de shaders
**Quiero** un playground visual donde pueda ver el resultado de mis shaders en tiempo real
**Para** verificar que los shaders renderizan correctamente sin guesses

#### Criterios de Aceptación - Playground Core

**UI Requirements:**
- [ ] Panel de preview del canvas de render
- [ ] Controles para crear entidades de test (rectángulos, círculos, etc.)
- [ ] Controles para modificar parámetros de entidades (color, shape, etc.)
- [ ] Panel de controls de cámara (pan, zoom)
- [ ] Panel de controls de shaders (sliders, toggles)
- [ ] Botón de "Capture Screenshot" para comparison
- [ ] Botón de "Run Shader Test" que compara con golden image

**Golden Image System:**
- [ ] Guardar screenshot como "golden" reference
- [ ] Comparar screenshot actual con golden
- [ ] Reporte de diferencias (pixels diferentes)
- [ ] Threshold configurable para diferencias aceptables
- [ ] Acept/Reject workflow para golden images

**Test Modes:**
- [ ] Mode: Single entity rendering
- [ ] Mode: Multiple entities (10, 100, 1000)
- [ ] Mode: Hierarchy rendering
- [ ] Mode: Selection highlight overlay
- [ ] Mode: Drag selection preview
- [ ] Mode: Z-order rendering (capas)

#### Playground Architecture

```typescript
// Estructura del playground
interface ShaderPlayground {
  // Canvas de preview
  canvas: HTMLCanvasElement;
  
  // Controles de entidades
  entityControls: {
    addRectangle: () => void;
    addCircle: () => void;
    addText: () => void;
    clearAll: () => void;
  };
  
  // Controles de shaders
  shaderControls: {
    useCustomShader: boolean;
    fragmentShader: string;
    vertexShader: string;
    uniforms: Record<string, number>;
  };
  
  // Sistema de golden images
  goldenImages: {
    capture: () => Promise<void>;
    compare: () => Promise<TestResult>;
    approve: () => Promise<void>;
    reset: () => void;
  };
  
  // Test runner
  testRunner: {
    runAll: () => Promise<TestSuiteResult>;
    runSingle: (testId: string) => Promise<TestResult>;
    exportResults: () => string;
  };
}
```

#### Estimación: XL
#### Estado: Pendiente

---

### HU-TEST-004: Shader Unit Tests (Math & Logic)

**Como** desarrollador
**Quiero** tests unitarios que verifiquen la lógica de los shaders matemáticamente
**Para** detectar bugs en shaders antes de renderizar

#### Criterios de Aceptación - Math Tests

**Shape Rendering:**
- [ ] Test: Rectangle shader output boundary pixels
- [ ] Test: Circle shader distance field (Signed Distance Function)
- [ ] Test: Ellipse aspect ratio correcto
- [ ] Test: Line thickness consistente
- [ ] Test: Anti-aliasing coverage correcto

**Color Operations:**
- [ ] Test: RGBA to ABGR conversion correctness
- [ ] Test: Color tint multiplication
- [ ] Test: Alpha blending correctness
- [ ] Test: Stroke color separate del fill
- [ ] Test: Selection highlight color

**Transforms:**
- [ ] Test: Local to World transform matrix
- [ ] Test: Rotation matrix correctness
- [ ] Test: Scale transform accuracy
- [ ] Test: Translation accuracy
- [ ] Test: Combined transform order (T * R * S)

**Text Rendering:**
- [ ] Test: UV coordinate calculation
- [ ] Test: Font scale application
- [ ] Test: Glyph bounds calculation

#### Math Test Pattern

```rust
// tests/shader_math.rs

#[test]
fn test_circle_sdf_edge() {
    // Given: círculo unitario en centro
    let center = Vec2::new(50.0, 50.0);
    let radius = 25.0;
    
    // When: calculamos SDF para puntos en el borde
    let on_edge = circle_sdf(center + Vec2::new(radius, 0.0), center, radius);
    let inside = circle_sdf(center + Vec2::new(10.0, 0.0), center, radius);
    let outside = circle_sdf(center + Vec2::new(40.0, 0.0), center, radius);
    
    // Then: valores del SDF correctos
    assert!((on_edge - 0.0).abs() < 0.001);  // En el borde = 0
    assert!(inside < 0.0);  // Dentro = negativo
    assert!(outside > 0.0);  // Fuera = positivo
}

#[test]
fn test_color_conversion_rgba_to_abgr() {
    let rgba = 0x11223344;  // R=0x11, G=0x22, B=0x33, A=0x44
    let expected_abgr = 0x44332211;
    
    let result = rgba_to_abgr(rgba);
    assert_eq!(result, expected_abgr);
}
```

#### Estimación: M
#### Estado: Pendiente

---

### HU-TEST-005: Visual Regression Tests (Golden Images)

**Como** QA
**Quiero** tests que comparen screenshots con imágenes de referencia (golden)
**Para** detectar regresiones visuales automáticamente

#### Criterios de Aceptación

**Golden Image Pipeline:**
- [ ] Capturar golden image de estado conocido
- [ ] Guardar golden image en repo (formato PNG)
- [ ] En cada test: renderizar → capturar → comparar
- [ ] Generar diff image si falla
- [ ] Reporte HTML con resultados

**Test Scenarios:**
- [ ] Test: Rectángulo default render
- [ ] Test: Círculo default render
- [ ] Test: Shape con color personalizado
- [ ] Test: Shape con stroke
- [ ] Test: Shape con opacity < 1.0
- [ ] Test: Selección highlight
- [ ] Test: Múltiples shapes con z-order
- [ ] Test: Hierarchy render con offset
- [ ] Test: Text rendering
- [ ] Test: Canvas vacío (background color)

**Comparison Engine:**
- [ ] Pixel-perfect comparison (exact match)
- [ ] Fuzzy comparison con threshold configurable
- [ ] Ignore regions (para anti-aliasing borders)
- [ ] Perceptual diff (PHash) como fallback
- [ ] Metrics: SSIM, PSNR, MSE

#### Herramientas a Evaluar

| Tool | Pros | Contras |
|------|------|---------|
| `pixelmatch` | JS, ampliamente usado | Solo JS |
| `looks-same` | Node.js, threshold | Menos features |
| `resemblejs` | HTML output | Lento |
| `cargo-golden` (custom) | Rust, integrados | Needs build |

**Decisión**: Implementar custom Rust comparator con WASM bindings para maximum control.

#### Estimación: L
#### Estado: Pendiente

---

### HU-TEST-006: Performance Benchmark Suite

**Como** desarrollador
**Quiero** benchmarks que midan rendimiento en diferentes condiciones
**Para** detectar regresiones de performance

#### Criterios de Aceptación

**Core Benchmarks:**
- [ ] Benchmark: spawn 10k entities time
- [ ] Benchmark: despawn 10k entities time
- [ ] Benchmark: select 1k entities
- [ ] Benchmark: move 1k entities (MoveGroup)
- [ ] Benchmark: update_hierarchy (jerarquía 10 niveles × 1k)
- [ ] Benchmark: poll_events (10k events)
- [ ] Benchmark: render frame (1k entities)

**Memory Benchmarks:**
- [ ] Benchmark: memory usage con 100k entities alive
- [ ] Benchmark: memory growth después de 10k spawn/despawn cycles
- [ ] Benchmark: DeltaMask memory vs Vec<EntityId>
- [ ] Benchmark: EventRingBuffer memory efficiency

**Threshold Tests:**
- [ ] Test: 100k spawn < 500ms
- [ ] Test: move 100k entities < 1ms
- [ ] Test: update_hierarchy 10k × 10 niveles < 2ms
- [ ] Test: poll_events 10k < 1ms
- [ ] Test: render 100k entities < 16ms (60 FPS)

#### Estimación: M
#### Estado: Pendiente

---

### HU-TEST-007: Fuzz Testing para Edge Cases

**Como** QA
**Quiero** fuzz testing para descubrir edge cases inesperados
**Para** encontrar bugs que los tests normales no encuentran

#### Criterios de Aceptación

**Fuzz Targets:**
- [ ] Target: EntityStore operations con random indices
- [ ] Target: DeltaMask con random operations
- [ ] Target: EventRingBuffer con random push/drain
- [ ] Target: Command serialization con random commands
- [ ] Target: SpatialHash con random positions

**Fuzz Infrastructure:**
- [ ] Integración con libFuzzer
- [ ] Corpus de seeds guardado
- [ ] Minimización automática de crashes
- [ ] Reproducción de bugs desde crash log

#### Estimación: S
#### Estado: Pendiente

---

## 📊 Estado de Tareas

| Historia | Estado | Tests | Dependencias |
|----------|--------|-------|--------------|
| HU-TEST-001 | ⏳ Pendiente | 100+ | Ninguna |
| HU-TEST-002 | ⏳ Pendiente | 20+ | HU-TEST-001 |
| HU-TEST-003 | ⏳ Pendiente | 1 (playground) | Ninguna |
| HU-TEST-004 | ⏳ Pendiente | 30+ | Ninguna |
| HU-TEST-005 | ⏳ Pendiente | 15+ | HU-TEST-003 |
| HU-TEST-006 | ⏳ Pendiente | 20+ | Ninguna |
| HU-TEST-007 | ⏳ Pendiente | Fuzzer | Ninguna |

---

## 📋 Deuda Técnica de Testing

| Item | Severity | Descripción | Solución |
|------|----------|-------------|----------|
| DT-TEST-001 | **Alta** | Tests existentes son básicos | HU-TEST-001 |
| DT-TEST-002 | **Alta** | Sin verificación visual de shaders | HU-TEST-003 |
| DT-TEST-003 | Media | Sin golden images | HU-TEST-005 |
| DT-TEST-004 | Media | Sin performance baselines | HU-TEST-006 |

---

## 📋 Criterios de Éxito de la Épica

- [ ] 100+ unit tests con 100% coverage en core modules
- [ ] 20+ integration tests para flujos completos
- [ ] Playground funcional para debugging visual
- [ ] 15+ golden image tests para regresiones visuales
- [ ] 20+ benchmarks con thresholds documentados
- [ ] Fuzzer configurado y corriendo nightly
- [ ] Code coverage > 90% en archflow-engine
- [ ] Code coverage > 85% en archflow-logic
- [ ] Zero crashes en fuzz testing (24h)

---

## 📋 Dependencias

- Depende de: Ninguna (work standalone)
- Depende de: CONSOLIDATION-EPIC-001 para features stability

---

## 📋 Timeline

```
Semana 1-2:
- D1-D5: HU-TEST-001 (Unit Tests Exhaustivos)

Semana 3:
- D1-D3: HU-TEST-002 (Integration Tests)
- D4-D5: HU-TEST-004 (Shader Math Tests)

Semana 4:
- D1-D5: HU-TEST-003 (Shader Playground - UI heavy)

Semana 5:
- D1-D3: HU-TEST-005 (Golden Images)
- D4-D5: HU-TEST-006 (Benchmarks)

Semana 6:
- D1-D3: HU-TEST-007 (Fuzz Testing)
- D4-D5: Integration y Release v0.45.0
```

---

## 📚 Playground UI Design

### Layout Principal

```
┌─────────────────────────────────────────────────────────────────┐
│  🧪 Shader Playground v1.0                              ─ □ X │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────────────────────────┐  │
│  │ Entity Controls │  │                                     │  │
│  │                 │  │                                     │  │
│  │ [+] Rectangle  │  │         CANVAS PREVIEW             │  │
│  │ [+] Circle     │  │                                     │  │
│  │ [+] Ellipse    │  │                                     │  │
│  │ [+] Text       │  │                                     │  │
│  │ [+] From SVG   │  │                                     │  │
│  │ ─────────────  │  │                                     │  │
│  │ [Clear All]    │  │                                     │  │
│  │ [Reset Camera] │  │                                     │  │
│  │                 │  │                                     │  │
│  ├─────────────────┤  │                                     │  │
│  │ Selected:      │  │                                     │  │
│  │ ID: 42         │  │                                     │  │
│  │ Pos: 100, 200  │  │                                     │  │
│  │ Size: 50x50    │  │                                     │  │
│  │ Color: #FF0000 │  │                                     │  │
│  │ Shape: Rect    │  │                                     │  │
│  │ [Delete]       │  │                                     │  │
│  └─────────────────┘  └─────────────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────────────────────────┐  │
│  │ Camera          │  │ Shader Controls                     │  │
│  │ Zoom: [ 1.0x ] │  │                                     │  │
│  │ Pan X: [ 0   ] │  │ [✓] Use Custom Shader              │  │
│  │ Pan Y: [ 0   ] │  │ Vertex Shader: [ ▼ select ]         │  │
│  │ [Reset]        │  │ Fragment Shader: [ ▼ select ]      │  │
│  ├─────────────────┤  │ Uniforms:                          │  │
│  │ Selection Mode  │  │ [Time     ] [0.0      ]────────   │  │
│  │ (o) Single     │  │ [Glow     ] [1.0      ]────────   │  │
│  │ ( ) Multi      │  │ [Pulse    ] [0.5      ]────────   │  │
│  │ ( ) Box        │  │                                     │  │
│  └─────────────────┘  └─────────────────────────────────────┘  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ 🧪 Tests & Golden Images                                 │  │
│  │ [📷 Capture Golden] [▶ Run Tests] [✓ Accept All]       │  │
│  │ Status: ✅ 15/15 passed | ⏱️ 123ms | 📊 0% diff       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Features del Playground

**1. Entity Quick Add:**
- Keyboard shortcuts: `R`=Rect, `C`=Circle, `E`=Ellipse, `T`=Text
- Click-drag para crear entidades con tamaño visual
- Paste SVG/PNG directamente

**2. Shader Live Editing:**
- Editor de texto con syntax highlighting para GLSL
- Live preview mientras escribes
- Error overlay para errores de compilación
- Preset shaders: glow, pulse, gradient, outline

**3. Golden Image Workflow:**
1. Configurar escenario de test
2. Click "Capture Golden"
3. Modificar código/shader
4. Click "Run Tests"
5. Ver diff - si es esperado: "Approve"
6. Si no es esperado: bug encontrado!

**4. Performance Mode:**
- Toggle: Show FPS
- Toggle: Show draw calls
- Toggle: Show memory usage
- Profile: Time each operation

---

## 📋 Herramientas Recomendadas

### Para Visual Regression
- **Resemble.js** - Comparison HTML visual
- **Pixelmatch** - Alta precisión, rápido
- **Odiff** - WASM-based, buen performance

### Para Fuzzing
- **cargo-fuzz** - Integración libFuzzer
- **AFL++** - Alternative fuzzer
- **QuickCheck** - Property-based testing

### Para Benchmarks
- **criterion** - Statistics-based, trend detection
- **bencher** - CLI benchmarking
- **iai** - Callgrind-based para Rust

---

## 🎯 Métricas de Calidad de Testing

| Métrica | Target | Current |
|---------|--------|---------|
| Unit Test Coverage | >90% | ~60% |
| Integration Test Coverage | >80% | ~30% |
| Shader Math Tests | 30+ | 0 |
| Golden Images | 15+ | 0 |
| Performance Baselines | 20+ | 5 |
| Fuzz Crashes (24h) | 0 | N/A |
| Test Execution Time | <30s | ~10s |

---

## 📚 Documentación Relacionada

- `docs/development/TESTING_GUIDE.md` (a crear)
- `docs/development/PLAYGROUND_GUIDE.md` (a crear)
- `docs/development/GOLDEN_IMAGES.md` (a crear)

---

## 🔧 Comandos de Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with coverage
cargo tarpaulin --workspace

# Run benchmarks
cargo bench

# Run fuzzer (requires cargo-fuzz)
cargo fuzz run entity_store

# Run visual playground
npm run dev -- --playground

# Update golden images
cargo test -- --update-golden
```
