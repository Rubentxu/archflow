# Épica: ECS Query Abstraction Layer

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-ECS-001 |
| Prioridad | Alta |
| Estimación | XL (3-4 semanas) |
| Estado | Borrador |
| Versión | 0.1.0 |

## 🎯 Objetivo de Negocio

Transformar el acceso directo a `EntityStore` en una capa de abstracción Query tipada, siguiendo patrones de ECS modernos (Bevy, Flecs). Esto reducirá bugs por acceso indebido y habilitará futuras optimizaciones como parallel query execution.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-engine` (Render Subsystem)
- **Aggregate Root**: `EntityStore`
- **Domain Events**: `QueryExecuted`, `ComponentAccessed`
- **Value Objects**: `QueryFilter`, `QueryIter`, `QueryItem`

## 📖 Historias de Usuario

### HU-ECS-001: Crear Query Trait Base

**Como** desarrollador
**Quiero** un trait `RenderQuery` que encapsule el acceso a componentes de render
**Para** que el renderer no tenga acceso directo a todos los campos de EntityStore

#### Criterios de Aceptación
- [ ] Existe trait `RenderQuery<'a>` en `archflow-engine`
- [ ] Método `iter(&'a self) -> QueryIter<'a>`
- [ ] Método `iter_dirty(&'a mut self) -> QueryIter<'a>`
- [ ] Tests verifican que solo se expone lo necesario
- [ ] Tests de integración con GpuRenderer pasan

#### Tareas Técnicas
- [ ] Investigar patrones Query de Bevy ECS
- [ ] Escribir tests de aceptación para Query trait
- [ ] Implementar RenderQuery trait base
- [ ] Implementar QueryIter con lifetime 'a
- [ ] Actualizar GpuRenderer para usar Query
- [ ] Verificar tests pasan

#### Investigación Previa
- **Perplexity**: "Rust ECS Query trait patterns Bevy Flecs"
- **Context7**: `/sandermertens/flecs` - Query iteration patterns
- **Patrón**: Observer pattern + Iterator trait composition

#### Estimación: L
#### Estado: Pendiente

---

### HU-ECS-002: Implementar Component Views

**Como** sistema de render
**Quiero** views tipados por componente (TransformView, ColorView, TextureView)
**Para** acceder solo a los datos que necesito de forma type-safe

#### Criterios de Aceptación
- [ ] Struct `RenderView<'a>` con campos solo-lectura
- [ ] Método `get(idx: usize) -> Option<RenderComponent>`
- [ ] Tests verifican lifetime correctness
- [ ] Benchmarks muestran overhead < 5%

#### Tareas Técnicas
- [ ] Diseñar estructura de Component Views
- [ ] Implementar TransformView, ColorView, TextureView
- [ ] Escribir tests de lifetime safety
- [ ] Benchmark comparando con acceso directo
- [ ] Documentar uso de views

#### Estimación: L
#### Estado: Pendiente

---

### HU-ECS-003: Query Filters (With/Without)

**Como** desarrollador
**Quiero** filtrar entidades por componente (e.g., solo visibles, solo shapes)
**Para** evitar iterar sobre entidades que no me interesan

#### Criterios de Aceptación
- [ ] Filtro `With<T>` para incluir solo entidades con componente T
- [ ] Filtro `Without<T>` para excluir entidades con componente T
- [ ] Tests verifican filtering correcto
- [ ] Documentación con ejemplos

#### Tareas Técnicas
- [ ] Diseñar API de filtros
- [ ] Implementar With/Without filters
- [ ] Tests de integración con RenderView
- [ ] Ejemplos de uso en GpuRenderer

#### Estimación: M
#### Estado: Pendiente

---

### HU-ECS-004: Transformar GpuRenderer para usar Query

**Como** sistema de render
**Quiero** usar Query abstraction en lugar de acceso directo
**Para** probar que la abstracción funciona en código real

#### Criterios de Aceptación
- [ ] GpuRenderer usa RenderQuery en sync_from_store
- [ ] GpuRenderer usa RenderQuery en sync_dirty
- [ ] Tests existentes pasan sin modificación
- [ ] Benchmark muestra overhead aceptable (< 10%)

#### Tareas Técnicas
- [ ] Refactorizar sync_from_store para usar Query
- [ ] Refactorizar sync_dirty para usar Query
- [ ] Verificar todos los tests pasan
- [ ] Benchmark comparando rendimiento

#### Estimación: M
#### Estado: Pendiente

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
| HU-ECS-001 | ⏳ Pendiente | 0/8 | - |
| HU-ECS-002 | ⏳ Pendiente | 0/6 | - |
| HU-ECS-003 | ⏳ Pendiente | 0/5 | - |
| HU-ECS-004 | ⏳ Pendiente | 0/12 | - |

---

## 📝 Resumen Ejecutivo

Crear una capa de abstracción Query que encapsule el acceso a componentes de EntityStore, siguiendo patrones de ECS modernos (Bevy, Flecs). Esto eliminará el acoplamiento directo entre GpuRenderer y EntityStore, mejorando mantenibilidad y habilitando optimizaciones futuras.

## 🔗 Dependencias

- Depende de: Ninguna (épica foundational)
- Habilita: EPIC-ECS-SCHEDULING, EPIC-ECS-PARALLEL

## 📁 Archivos de Salida

```
docs/
  epics/
    EPIC-ECS-QUERY.md ← Este archivo
crates/archflow-engine/
  src/
    query.rs ← Nueva abstracción Query
    views.rs ← Component Views
tests/
  hu_ecs_query.rs ← Tests de aceptación
```
