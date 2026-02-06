# Épica: ECS Query Abstraction Layer

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-ECS-001 |
| Prioridad | Alta |
| Estimación | XL (3-4 semanas) |
| Estado | ✅ COMPLETADO |
| Versión | 0.2.0 |

## 🎯 Objetivo de Negocio

Transformar el acceso directo a `EntityStore` en una capa de abstracción Query tipada, siguiendo patrones de ECS modernos (Bevy, Flecs). Esto reducirá bugs por acceso indebido y habilitará futuras optimizaciones como parallel query execution.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-engine` (Render Subsystem)
- **Aggregate Root**: `EntityStore`
- **Domain Events**: `QueryExecuted`, `ComponentAccessed`
- **Value Objects**: `QueryFilter`, `QueryIter`, `QueryItem`

## 📖 Historias de Usuario

### HU-ECS-001: Crear Query Trait Base ✅ COMPLETADO

**Como** desarrollador
**Quiero** funciones de query tipadas
**Para** que el renderer tenga acceso limpio a EntityStore

#### Criterios de Aceptación
- [x] Existe módulo `query.rs` en `archflow-engine`
- [x] Funciones `query_visible()`, `query_dirty_render()`, `query_renderable()`
- [x] Tests verifican que solo se expone lo necesario
- [x] 9 tests pasando

#### Implementación
```rust
// Usage
let store = EntityStore::new();
let results = query_visible(&store);
for result in results {
    // Process result
}
```

#### Estimación: L
#### Estado: ✅ COMPLETADO

---

### HU-ECS-002: Implementar Component Views ✅ COMPLETADO

**Como** sistema de render
**Quiero** views tipados por componente
**Para** acceder solo a los datos que necesito de forma type-safe

#### Criterios de Aceptación
- [x] `TransformView<'a>` para acceso a transformes
- [x] `ColorView<'a>` para acceso a colores
- [x] `MetadataView<'a>` para acceso a metadatos
- [x] Tests verifican lifetime correctness

#### Implementación
```rust
let view = TransformView::new(&store);
let transform = view.transform(index);
```

#### Estimación: L
#### Estado: ✅ COMPLETADO

---

### HU-ECS-003: Query Filters ✅ COMPLETADO

**Como** desarrollador
**Quiero** filtrar entidades por componente
**Para** evitar iterar sobre entidades que no me interesan

#### Criterios de Aceptación
- [x] Filtros via funciones especializadas:
  - `query_visible()` - solo visibles
  - `query_dirty_render()` - solo dirty
  - `query_renderable()` - visibles + dirty + no locked
- [x] Tests verifican filtering correcto

#### Estimación: M
#### Estado: ✅ COMPLETADO

---

### HU-ECS-004: Transformar GpuRenderer para usar Query ✅ COMPLETADO

**Como** sistema de render
**Quiero** usar Query abstraction en lugar de acceso directo
**Para** probar que la abstracción funciona en código real

#### Criterios de Aceptación
- [x] GpuRenderer usa RenderQuery en sync_from_store
- [x] GpuRenderer usa RenderQuery en sync_dirty
- [x] Tests existentes pasan sin modificación
- [x] Benchmark muestra overhead aceptable (< 10%)

#### Tareas Técnicas
- [x] Refactorizar sync_from_store para usar Query
- [x] Refactorizar sync_dirty para usar Query
- [x] Verificar todos los tests pasan
- [x] Benchmark comparando rendimiento

#### Implementación
```rust
// GpuRenderer ahora usa RenderQuery para acceso limpio
let query = RenderQuery::new(store);
let pos = query.pos(index);      // Tipo: Option<(f32, f32)>
let color = query.fill_color(index); // Tipo: Option<u32>
```

#### Estimación: M
#### Estado: ✅ COMPLETADO

---

## 🔬 Investigación de Referencia

### Patrones de Flecs (Data-Oriented)

```rust
// Flecs Rust style - query con filtros
q.each(|(p, v)| { /* ... */ });

// Sistema con scheduling implícito
world
    .system_named::<(&mut Position, &Velocity)>("Move")
    .each(|(p, v)| { /* ... */ });
```

### Patrones de Bevy ECS

```rust
// Query con filtros
fn render_system(
    query: Query<&Transform, With<Visible>>,
    camera: Query<&Camera>,
) {
    for (transform, visible) in query.iter().with(&camera) {
        // Acceso seguro y tipado
    }
}
```

---

## 📊 Estado de Tareas

| Historia | Estado | Tests | Deuda |
|----------|--------|-------|-------|
| HU-ECS-001 | ✅ Completo | 3/3 | - |
| HU-ECS-002 | ✅ Completo | 3/3 | - |
| HU-ECS-003 | ✅ Completo | 3/3 | - |
| HU-ECS-004 | ✅ Completo | 12/12 | - |

---

## 📝 Resumen Ejecutivo

✅ **COMPLETADO** - Capa de abstracción Query implementada y integrada en GpuRenderer:
- Funciones de query tipadas (`query_visible`, `query_dirty_render`, `query_renderable`)
- Component Views (`TransformView`, `ColorView`, `MetadataView`)
- **RenderQuery** para integración con GpuRenderer
- 12 tests pasando
- GpuRenderer refactorizado para usar Query abstraction

## 📁 Archivos de Salida

```
crates/archflow-engine/
  src/
    query.rs ← Nueva abstracción Query (implementada)
    views.rs ← Integrados en query.rs
tests/
  query.rs ← Tests integrados en el módulo
```
