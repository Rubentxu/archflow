---
title: "Épica: Testing E2E Completo para ArchFlow"
author: Claude Code
date: 2025-02-04
status: Completada
context: "Análisis de headless testing para WebGPU/WebGL2 + estructura del proyecto"
iteration: 1
previous_versions: []
---

# Épica: EPIC-TEST-001 - Testing E2E Completo para ArchFlow

## metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-TEST-001 |
| Prioridad | ALTA |
| Estimación | XL (4-6 semanas) |
| Estado | Borrador |
| Versión | 0.1.0 |

## Objetivo de Negocio

Garantizar la calidad y estabilidad de ArchFlow mediante una suite completa de tests end-to-end que cubra:
- **Core**: Entity Store, ECS, spatial hash
- **Renderizado**: WebGL2, WebGPU, texturas
- **Lógica**: Sensors, Actuators, Physics
- **Integración**: WASM bridge, SharedArrayBuffer
- **UI**: Interacción, undo/redo, colaboración

## Arquitectura DDD
- **Bounded Context**: Testing
- **Aggregate Root**: `TestRunner`, `SnapshotManager`
- **Domain Events**: `TestStarted`, `TestCompleted`, `SnapshotCaptured`
- **Services**: `HeadlessRenderer`, `WASMTestExecutor`

---

## 📖 Historias de Usuario

### HU-TEST-001: Tests Unitarios Core Entity Store

**Como** desarrollador
**Quiero** tests unitarios completos del EntityStore
**Para** garantizar la correcta gestión de entidades

#### Criterios de Aceptación
- [ ] Tests para spawn/despawn de entidades
- [ ] Tests para movimiento y transformaciones
- [ ] Tests para visibilidad y culling
- [ ] Tests para dirty tracking
- [ ] Tests para spatial hash queries
- [ ] Coverage > 90%

#### Tareas Técnicas
- [ ] Investigar patrones de testing para ECS en Rust
- [ ] Escribir tests de aceptación
- [ ] Implementar tests de EntityStore
- [ ] Implementar tests de SpatialHash
- [ ] Configurar cargo-nextest
- [ ] Configurar tarpaulin para coverage

#### Investigación Previa
- Perplexity: "Rust ECS testing patterns 2025"
- Context7: "archflow-engine crate testing patterns"

#### Estimación: M
#### Estado: Pendiente

---

### HU-TEST-002: Tests Unitarios Lógica ECS (Sensors/Actuators)

**Como** desarrollador
**Quiero** tests para el sistema de lógica (sensors y actuators)
**Para** garantizar que las interacciones funcionan correctamente

#### Criterios de Aceptación
- [ ] Tests para CollisionSensor
- [ ] Tests para MouseClickSensor
- [ ] Tests para MoveActuator
- [ ] Tests para SelectActuator
- [ ] Tests para HighlightActuator
- [ ] Tests para StateManager
- [ ] Coverage > 85%

#### Tareas Técnicas
- [ ] Investigar patrones de testing para ECS logic
- [ ] Escribir tests de aceptación
- [ ] Implementar tests de sensors
- [ ] Implementar tests de actuators
- [ ] Configurar tests paralelos

#### Investigación Previa
- Perplexity: "Rust ECS systems testing patterns"
- Context7: "archflow-logic crate testing"

#### Estimación: L
#### Estado: Pendiente

---

### HU-TEST-003: Tests Unitarios Renderizado GPU (Headless)

**Como** desarrollador
**Quiero** tests headless para WebGL2 y WebGPU
**Para** validar el renderizado sin necesidad de ventana visible

#### Criterios de Aceptación
- [ ] Tests WebGL2 con wasm-pack --headless --chrome
- [ ] Tests WebGPU con wgpu_test utilities
- [ ] Tests de snapshot rendering (comparación con gold images)
- [ ] Tests de shaders (compilación y ejecución)
- [ ] Tests de texturas
- [ ] Tests de instancing

#### Tareas Técnicas
- [ ] Investigar wgpu_test utilities (ya completado en análisis)
- [ ] Configurar wasm-pack para headless testing
- [ ] Crear infraestructura de gold images
- [ ] Implementar test de WebGL2 rectangle
- [ ] Implementar test de WebGL2 instancing
- [ ] Implementar test de WebGPU render-to-texture
- [ ] Configurar GitHub Actions con SwiftShader fallback

#### Investigación Previa
- Perplexity: "wgpu headless testing Rust CI/CD 2025"
- Context7: "wgpu_test utilities documentation"
- Repo参考: gfx-rs/wgpu testing patterns

#### Estimación: XL
#### Estado: Pendiente

---

### HU-TEST-004: Tests Integración WASM Bridge

**Como** desarrollador
**Quiero** tests del bridge WASM entre Rust y JavaScript
**Para** garantizar la correcta comunicación bidireccional

#### Criterios de Aceptación
- [ ] Tests de SharedArrayBuffer communication
- [ ] Tests de BinaryDeltaCodec
- [ ] Tests de canvas initialization
- [ ] Tests de renderer selection (WebGL2 vs WebGPU)
- [ ] Tests de context loss/recovery
- [ ] Tests de input processing

#### Tareas Técnicas
- [ ] Revisar tests existentes en archflow-tests
- [ ] Ampliar tests de SharedBuffer
- [ ] Implementar tests de bridge
- [ ] Implementar tests de context recovery
- [ ] Configurar wasm-pack test

#### Investigación Previa
- Context7: "archflow-tests crate"
- Web: "wasm-bindgen testing patterns"

#### Estimación: L
#### Estado: Pendiente

---

### HU-TEST-005: Tests Integración UI (Playwright)

**Como** QA Engineer
**Quiero** tests E2E de la interfaz de usuario
**Para** garantizar que los usuarios pueden completar flujos completos

#### Criterios de Aceptación
- [ ] Tests de canvas interaction (pan, zoom, select)
- [ ] Tests de toolbar (create shapes, tools)
- [ ] Tests de properties panel
- [ ] Tests de keyboard shortcuts
- [ ] Tests de undo/redo
- [ ] Tests de export functionality

#### Tareas Técnicas
- [ ] Investigar Playwright con Rust/WASM
- [ ] Configurar Playwright tests
- [ ] Implementar tests de canvas interaction
- [ ] Implementar tests de toolbar
- [ ] Implementar tests de export
- [ ] Configurar CI/CD para E2E tests

#### Investigación Previa
- Perplexity: "Playwright testing Rust WASM 2025"
- Web: "Playwright best practices E2E testing"

#### Estimación: XL
#### Estado: Pendiente

---

### HU-TEST-006: Tests de Snapshot Rendering (Gold Images)

**Como** desarrollador
**Quiero** comparar renders con imágenes de referencia
**Para** detectar regresiones visuales automáticamente

#### Criterios de Aceptación
- [ ] Tests de rectangle rendering
- [ ] Tests de circle rendering
- [ ] Tests de icon rendering
- [ ] Tests de text rendering (MTSDF)
- [ ] Tests de layered rendering (z-order)
- [ ] Tests de camera transform (pan/zoom)

#### Tareas Técnicas
- [ ] Crear directorio tests/snapshots/
- [ ] Generar gold images iniciales
- [ ] Implementar comparación de píxeles
- [ ] Implementar threshold de tolerancia
- [ ] Configurar actualización de gold images

#### Estimación: M
#### Estado: Pendiente

---

### HU-TEST-007: Tests de Rendering Performance

**Como** desarrollador
**Quiero** tests de performance del renderizado
**Para** garantizar que se mantiene el target de 60fps

#### Criterios de Aceptación
- [ ] Test de FPS con 100 entidades
- [ ] Test de FPS con 10,000 entidades
- [ ] Test de FPS con 100,000 entidades
- [ ] Test de memory usage
- [ ] Test de GPU memory allocation
- [ ] Baseline metrics documentados

#### Tareas Técnicas
- [ ] Implementar benchmark framework
- [ ] Crear tests de performance
- [ ] Documentar baseline metrics
- [ ] Configurar alertas de regresión
- [ ] Integrar con CI

#### Estimación: M
#### Estado: Pendiente

---

### HU-TEST-008: Tests de Colaboración (CRDT)

**Como** desarrollador
**Quiero** tests del sistema de colaboración
**Para** garantizar la sincronización correcta entre usuarios

#### Criterios de Aceptación
- [ ] Tests de merge de operaciones
- [ ] Tests de conflict resolution
- [ ] Tests de state synchronization
- [ ] Tests de cursor sharing
- [ ] Tests de undo/redo colaborativo

#### Tareas Técnicas
- [ ] Investigar CRDT testing patterns
- [ ] Implementar tests de merge
- [ ] Implementar tests de sync
- [ ] Configurar tests multi-cliente

#### Estimación: L
#### Estado: Pendiente

---

## 🔬 Investigación por Historia

### HU-TEST-001: Entity Store Testing

```bash
# Patrones de testing para ECS en Rust
perplexity_search("Rust ECS testing patterns bytemuck 2025")

# Mejores prácticas
context7_query("archflow-engine", "EntityStore testing patterns")

# Ejemplos de código
web_search("GitHub Rust ECS testing example")
```

### HU-TEST-003: GPU Headless Testing

**Ya investigado** en `docs/analysis/headless-testing-webgpu-webgl2.md`

**Hallazgos clave:**
- `wgpu_test` utilities para WebGPU
- `wasm-pack test --headless --chrome` para WebGL2
- `image` crate para snapshot comparison
- GitHub Actions con SwiftShader fallback

### HU-TEST-005: Playwright E2E

```bash
# Playwright con Rust
perplexity_search("Playwright Rust WASM E2E testing 2025")

# Mejores prácticas
web_search("Playwright best practices CI/CD")
```

---

## 🧪 Enfoque TDD por Historia

### Fase 1: Rojo (Test Fallando)

```rust
// tests/hu_test_003_webgl2_snapshot.rs
#[wasm_bindgen_test]
async fn test_webgl2_rectangle_rendering() {
    let canvas = create_test_canvas();
    let renderer = WebGL2Renderer::new(canvas.clone()).unwrap();
    
    let store = create_test_store_with_rect();
    let camera = create_test_camera();
    renderer.sync_from_store(&store, &camera);
    renderer.render().unwrap();
    
    // Fallará porque no hay gold image aún
    assert_snapshot_matches!("rect.png", &canvas.pixels());
}
```

### Fase 2: Verde (Implementación Mínima)

```rust
// Generar gold image inicial
fn generate_gold_images() {
    // Genera imágenes de referencia
    // Se guarda en tests/snapshots/
}
```

### Fase 3: Refactor

```rust
// Mejorar test con:
// - Reutilización de fixtures
// - Parametrización
// - Clear assertions
#[test_case(rect_test_case(); "rectangle")]
#[test_case(circle_test_case(); "circle")]
#[test_case(text_test_case(); "text")]
async fn test_shape_rendering(shape: ShapeTestCase) {
    let renderer = create_renderer();
    renderer.render_shape(&shape).unwrap();
    assert_snapshot_matches!(&shape.name, &renderer.pixels());
}
```

---

## 📊 Estado de Tareas

| Historia | Estado | Tests | Debt Técnica |
|----------|--------|-------|--------------|
| HU-TEST-001 | ⏳ Pendiente | 0/15 | Ninguna |
| HU-TEST-002 | ⏳ Pendiente | 0/20 | Ninguna |
| HU-TEST-003 | ⏳ Pendiente | 0/25 | Media |
| HU-TEST-004 | ⏳ Pendiente | 0/15 | Baja |
| HU-TEST-005 | ⏳ Pendiente | 0/30 | Alta |
| HU-TEST-006 | ⏳ Pendiente | 0/20 | Baja |
| HU-TEST-007 | ⏳ Pendiente | 0/10 | Ninguna |
| HU-TEST-008 | ⏳ Pendiente | 0/15 | Alta |

---

## 📝 Alcance

**Incluye:**
- Tests unitarios de core (EntityStore, ECS, SpatialHash)
- Tests unitarios de lógica (Sensors, Actuators)
- Tests de integración GPU (WebGL2, WebGPU)
- Tests de integración WASM (Bridge, SharedArrayBuffer)
- Tests E2E UI (Playwright)
- Tests de snapshot rendering
- Tests de performance
- Tests de colaboración (CRDT)
- Configuración CI/CD completa

**No incluye:**
- Tests de seguridad (separada en otra épica)
- Tests de load testing (separada en otra épica)
- Tests de accesibilidad (separada en otra épica)

---

## 📈 Timeline

```
Semana 1: HU-TEST-001, HU-TEST-002 (Core + Logic)
Semana 2: HU-TEST-003 (GPU Headless - infraestructura)
Semana 3: HU-TEST-004 (WASM Bridge)
Semana 4: HU-TEST-005 (Playwright E2E)
Semana 5: HU-TEST-006 (Snapshots), HU-TEST-007 (Performance)
Semana 6: HU-TEST-008 (CRDT), CI/CD final
```

---

## ⚙️ Infraestructura de Tests

### Estructura de Directorios

```
crates/
├── archflow-tests/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── wasm_integration_tests.rs    # WASM tests
│   │   ├── e2e_tests.rs                 # Playwright wrapper
│   │   └── performance_benchmarks.rs    # Performance tests
│   ├── tests/
│   │   ├── unit/                        # Tests unitarios
│   │   │   ├── core/
│   │   │   ├── logic/
│   │   │   └── render/
│   │   ├── integration/                 # Tests integración
│   │   │   ├── gpu/
│   │   │   └── wasm/
│   │   ├── snapshots/                   # Gold images
│   │   │   ├── rect.png
│   │   │   ├── circle.png
│   │   │   └── ...
│   │   └── e2e/                         # Tests E2E
│   │       ├── canvas.spec.ts
│   │       ├── toolbar.spec.ts
│   │       └── ...
│   └── Cargo.toml

.github/
├── workflows/
│   ├── tests-unit.yml
│   ├── tests-gpu.yml
│   └── tests-e2e.yml
```

### Dependencias Necesarias

```toml
# Cargo.toml de archflow-tests

[dev-dependencies]
# Testing frameworks
test-case = "3"
trybuild = "1"

# GPU Testing
image = "0.25"
wgpu-test = { path = "../wgpu-test" }  # Custom wrapper

# WASM Testing
wasm-bindgen-test = "0.3"
wasm-pack = "0.12"

# Performance
criterion = "0.5"
divan = "0.1"

# E2E
playwright = "1.40"
```

---

## 📋 Checklist de Calidad

### Tests Unitarios
- [ ] Naming convention: `test_[module]_[function]`
- [ ] Given-When-Then structure
- [ ] Tests idempotentes
- [ ] Tests independientes
- [ ] Coverage > 80% por módulo

### Tests Integración
- [ ] Setup/Teardown claros
- [ ] Mocking de dependencias
- [ ] Assertions semánticos
- [ ] Logging de fallos

### Tests E2E
- [ ] Selectores estables
- [ ] Waits apropiados
- [ ] Screenshots en fallo
- [ ] Videos en fallo (CI)

---

## Deuda Técnica Identificada

| Item | Severity | Descripción | Solución Propuesta |
|------|----------|-------------|-------------------|
| Gold images missing | ALTA | Sin baseline visual | Generar en Fase 1 |
| GPU CI no configurado | ALTA | Sin tests automatizados | GitHub Actions + SwiftShader |
| Playwright sin integrar | MEDIA | Tests UI manuales | Integrar en Fase 4 |
| Coverage parcial | MEDIA | < 80% en algunos módulos | Mejorar en cada historia |

---

## Mejoras Futuras

1. **Visual Regression Testing**: Integración con Percy o Chromatic
2. **Fuzz Testing**: Para validaciones de entrada
3. **Mutation Testing**: Para evaluar calidad de tests
4. **Property-Based Testing**: Con `quickcheck` o `proptest`

---

## Recursos

- [Análisis de Headless Testing](docs/analysis/headless-testing-webgpu-webgl2.md)
- [Tests Existentes](crates/archflow-tests/src/)
- [wgpu Testing Docs](https://wgpu.rs/doc/wgpu_test/)
- [Playwright Testing](https://playwright.dev/docs/test-intro)
