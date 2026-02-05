# Épica: CONSOLIDATION-001 - Production Ready

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-CONSOLIDATION-001 |
| Prioridad | Alta |
| Estimación | L |
| Estado | Borrador |
| Versión | 0.1.0 |
| Versión Target | 0.44.0 |

## 🎯 Objetivo de Negocio

Consolidar las implementaciones de DeltaMask, EventRingBuffer y SIMD Batch Operations para alcanzar estado **production-ready**, corrigiendo los issues de integración identificados en el code review y añadiendo benchmarks de rendimiento.

**Problema actual**: El código base tiene deuda técnica de integración:
- `on_entity_destroyed()` existe pero no se llama en `despawn()` → memory leaks
- Actuators no emiten eventos automáticamente → UI no recibe notificaciones
- BoxSelectSensor no existe → selección rectangular no funcional
- Sin benchmarks documentados

**Solución propuesta**: Completar integraciones, implementar missing components, añadir tests de rendimiento.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-engine` + `archflow-logic`
- **Aggregate Root**: `EntityStore` (cleanup coordinator)
- **Domain Events**: `EntityDestroyed` (ya implementado)
- **Services**: `EventRingBuffer`, `BoxSelectSensor`, `EntityCleanupService`

## 📖 Historias de Usuario

### HU-CONSOL-001: Integrar Entity Destroy Hook en Despawn

**Como** sistema de memoria
**Quiero** que `on_entity_destroyed()` se llame automáticamente cuando una entidad es destruida
**Para** evitar memory leaks en sensores, actuators y wiring

#### Criterios de Aceptación
- [ ] `EntityStore::despawn()` llama a `LogicSystem::on_entity_destroyed()`
- [ ] Sensores resetean estado de entidades destruidas
- [ ] Wiring mappings se limpian para entidades destruidas
- [ ] SpatialHash remove se ejecuta desde LogicSystem
- [ ] Test de memory leak: 10k create/destroy cycles sin crecimiento de memoria

#### Tareas Técnicas
- [ ] Modificar `EntityStore::despawn()` para aceptar callback/References
- [ ] Añadir `&mut LogicSystem` como parámetro opcional a `despawn()`
- [ ] Mover cleanup logic de sensors a `on_entity_destroyed()` ya existente
- [ ] Test de integración: verificar que sensores resetean correctamente
- [ ] Test de memory leak con valgrind/tracing

#### Investigación Previa
- [x] `LogicSystem::on_entity_destroyed()` ya implementado en logic_system.rs:159-194
- [x] `EntityDestroyed` event ya existe en events.rs:56
- [x] `SpatialHash.remove()` disponible

#### Estimación: M
#### Estado: Pendiente

---

### HU-CONSOL-002: Integrar Emisión de Eventos en BatchSelectActuator

**Como** sistema de eventos
**Quiero** que `BatchSelectActuator` emita eventos `EntitySelected` automáticamente
**Para** que la UI reciba notificaciones de cambios de selección

#### Criterios de Aceptación
- [ ] `BatchSelectActuator::execute()` acepta callback de eventos
- [ ] Por cada entidad que cambia de estado, se emite `EntitySelected`
- [ ] Para operaciones batch (>10 entidades), se emite `BoxSelectionCompleted`
- [ ] Test: verificar que eventos se push al EventRingBuffer
- [ ] Test: verificar conteo correcto en `BoxSelectionCompleted`

#### Tareas Técnicas
- [ ] Modificar `BatchSelectActuator::execute()` para aceptar `&mut LogicSystem`
- [ ] En modo Multi: emitir `EntitySelected` por cada toggle
- [ ] En modo Single/Replace: emitir `EntitySelected` para cambios
- [ ] Batch >10 entidades: emitir `BoxSelectionCompleted` con count
- [ ] Tests de integración

#### Investigación Previa
- [x] `LogicSystem::emit_entity_selected()` existe en logic_system.rs:113-117
- [x] `LogicEventType::BoxSelectionCompleted` existe en events.rs:51
- [x] `EventRingBuffer.push()` disponible

#### Estimación: S
#### Estado: Pendiente

---

### HU-CONSOL-003: Implementar BoxSelectSensor

**Como** usuario
**Quiero** seleccionar entidades dibujando un rectángulo (marquee)
**Para** seleccionar múltiples objetos simultáneamente en diagrams grandes

#### Criterios de Aceptación
- [ ] `BoxSelectSensor` existe en `crates/archflow-logic/src/sensors/box_select.rs`
- [ ] `BoxSelection` struct con `start: Vec2`, `end: Vec2`
- [ ] `evaluate()` usa `SpatialHash.query_rect()` para O(k) query
- [ ] Test: 100+ entidades, selección rectangular selecciona correctamente
- [ ] Test: entidades fuera del rectángulo no se seleccionan
- [ ] Test: entities parcialmente dentro también se seleccionan (AABB intersection)

#### Tareas Técnicas
- [ ] Crear `crates/archflow-logic/src/sensors/box_select.rs`
- [ ] Definir `BoxSelection` struct con métodos utilitarios
- [ ] Implementar `BoxSelectSensor` con `SpatialHash` opcional
- [ ] `evaluate()` retorna `Vec<EntityId>` de entidades dentro
- [ ] Tests unitarios con entidades conocidas
- [ ] Tests de integración con SpatialHash real

#### Patrón a Implementar
```rust
// SpatialHash O(k) query pattern
let aabb = selection.to_aabb();
let nearby = spatial.query_rect(aabb);  // k = entidades cerca
for entity_id in nearby {
    if aabb.intersects(&entity_aabb) {  // verificación exacta
        selected.push(entity_id);
    }
}
```

#### Estimación: M
#### Estado: Pendiente

---

### HU-CONSOL-004: Añadir Benchmarks de Rendimiento

**Como** desarrollador
**Quiero** benchmarks documentados de las operaciones críticas
**Para** verificar que cumplimos los requisitos de rendimiento (<1ms/100k)

#### Criterios de Aceptación
- [ ] Benchmark: `apply_delta_to_mask()` < 1ms para 100k entidades
- [ ] Benchmark: `update_hierarchy_bfs()` < 2ms para jerarquía profunda
- [ ] Benchmark: `EventRingBuffer::push()` < 0.01ms (10k ops)
- [ ] Benchmark: `DeltaMask::from_indices()` < 0.5ms para 100k indices
- [ ] Benchmarks en `crates/archflow-engine/benches/`
- [ ] Benchmarks en `crates/archflow-logic/benches/`

#### Tareas Técnicas
- [ ] Crear `crates/archflow-engine/benches/simd_batch.rs`
- [ ] Benchmark `apply_delta_to_mask` con 100k entidades
- [ ] Benchmark `update_hierarchy_bfs` con jerarquía 10 niveles × 10k
- [ ] Benchmark `DeltaMask::from_indices` con sparse indices
- [ ] Benchmark `MoveGroup` end-to-end
- [ ] Crear `crates/archflow-logic/benches/events.rs`
- [ ] Benchmark `EventRingBuffer` throughput
- [ ] Documentar resultados en `docs/benchmarks/`

#### Métricas Objetivo
| Operación | Target | Current (esperado) |
|-----------|--------|-------------------|
| apply_delta_to_mask (100k) | <1ms | ~0.8ms |
| update_hierarchy_bfs (10k×10) | <2ms | ~1.5ms |
| DeltaMask::from_indices (100k) | <0.5ms | ~0.4ms |
| EventRingBuffer::push (10k) | <10ms | ~5ms |

#### Estimación: S
#### Estado: Pendiente

---

### HU-CONSOL-005: Tests de Estrés con 100k Entidades

**Como** QA Engineer
**Quiero** tests que verifiquen el comportamiento con 100k entidades
**Para** asegurar que el sistema escala linealmente

#### Criterios de Aceptación
- [ ] Test: crear 100k entidades, verificar alive_count == 100k
- [ ] Test: seleccionar 1000 entidades aleatorias, verificar memoria DeltaMask
- [ ] Test: mover jerarquía con 10k entities, verificar world_transform correcto
- [ ] Test: destroy 10k entidades, verificar que no hay memory leaks
- [ ] Test: eventos con 10k entity_destroyed, verificar EventRingBuffer

#### Tareas Técnicas
- [ ] Añadir `tests/stress_100k_entities.rs` en archflow-engine
- [ ] Test de creación batch con 100k entidades
- [ ] Test de selección batch con 100k
- [ ] Test de jerarquía anidada (10 niveles)
- [ ] Test de memory tracking con allocation counters
- [ ] Tests en archflow-logic para eventos con muchas entidades

#### Investigación Previa
- [x] `MAX_ENTITIES = 100_000` definido en store.rs
- [x] `alive_count()` disponible para verificación
- [x] `dirty_render.ones()` para verificar dirty tracking

#### Estimación: S
#### Estado: Pendiente

---

## 📋 Deuda Técnica Identificada (del Code Review)

| Item | Severity | Descripción | Solución |
|------|----------|-------------|----------|
| DT-001 | **Alta** | `despawn()` no llama `on_entity_destroyed()` | HU-CONSOL-001 |
| DT-002 | **Alta** | Actuators no emiten eventos | HU-CONSOL-002 |
| DT-003 | **Media** | BoxSelectSensor no existe | HU-CONSOL-003 |
| DT-004 | **Baja** | Sin benchmarks documentados | HU-CONSOL-004 |
| DT-005 | **Media** | Tests de estrés faltantes | HU-CONSOL-005 |

---

## 📊 Criterios de Éxito de la Épica

- [ ] 0 memory leaks en tests de destroy (10k ciclos)
- [ ] Eventos se emiten automáticamente desde actuators
- [ ] BoxSelectSensor funcional con SpatialHash
- [ ] Benchmarks documentados y dentro de targets
- [ ] Tests de estrés pasan con 100k entidades
- [ ] Code coverage > 85% en engine y logic
- [ ] v0.44.0 tagged con todos los fixes

---

## 📋 Dependencias

- Depende de: `EPIC-LOGIC-BOXSELECT-002` (DeltaMask existente)
- Depende de: `EPIC-LOGIC-EVENTS-001` (EventRingBuffer existente)
- Depende de: `EPIC-LOGIC-ENGINE-SIMD-004` (SIMD ops existente)

---

## 📋 Timeline

```
Semana 1:
- D1-D2: HU-CONSOL-001 (Entity Destroy Hook integration)
- D3-D4: HU-CONSOL-002 (Actuator event emission)

Semana 2:
- D1-D3: HU-CONSOL-003 (BoxSelectSensor)
- D4-D5: HU-CONSOL-004 (Benchmarks)

Semana 3:
- D1-D3: HU-CONSOL-005 (Stress tests)
- D4: Integration testing
- D5: v0.44.0 release
```

---

## 📚 Documentación Relacionada

- Code Review Report (conversación previa)
- `docs/epics/LOGIC-BOXSELECT-EPIC--002_BoxSelection.md`
- `docs/epics/LOGIC-EVENTS-EPIC--001_EventRingBuffer.md`
- `docs/epics/LOGIC-ENGINE-SIMD-EPIC-004_SIMDBatch.md`
- `docs/epics/LOGIC-CLEANUP-EPIC--003_MemoryCleanup.md`

---

## 🔧 Checklist de Producción

```markdown
## Pre-Release Checklist

- [ ] cargo clippy --workspace -- -D warnings (sin warnings)
- [ ] cargo fmt --all (código formateado)
- [ ] cargo test --workspace (todos los tests pasan)
- [ ] cargo bench (benchmarks dentro de targets)
- [ ] cargo doc --no-deps --all (documentación genera)
- [ ] Actualizar CHANGELOG.md
- [ ] Verificar que Cargo.toml version == v0.44.0
- [ ] git tag v0.44.0
```

---

## 💡 Notas de Arquitectura

### Integración Pattern: EntityStore + LogicSystem

```
EntityStore::despawn(id, Some(&mut logic_system))
    ↓
    logic_system.on_entity_destroyed(id)
        ↓
        ├─► event_buffer.push(EntityDestroyed)
        ├─► sensors[entity].reset()
        ├─► wiring.clear_entity(entity)
        └─► spatial_hash.remove(entity)
```

### BoxSelectSensor O(k) Query

```
BoxSelection { start, end }
    ↓
aabb = to_aabb()
    ↓
nearby = spatial_hash.query_rect(aabb)  // O(k)
    ↓
selected = nearby.filter(entity → entity_aabb.intersects(aabb))
```

---

## 📝 Historias Ordenadas por Priority

| Priority | Historia | Estimación | Dependencias |
|----------|----------|-------------|--------------|
| P0 | HU-CONSOL-001 | M | Ninguna |
| P0 | HU-CONSOL-002 | S | HU-CONSOL-001 |
| P1 | HU-CONSOL-003 | M | EventRingBuffer |
| P1 | HU-CONSOL-004 | S | Ninguna |
| P2 | HU-CONSOL-005 | S | HU-CONSOL-001 |
