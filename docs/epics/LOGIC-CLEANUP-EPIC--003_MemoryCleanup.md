# Épica: Memory Cleanup - Auto-Limpieza de Recursos

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-LOGIC-CLEANUP-003 |
| Prioridad | Media-Alta |
| Estimación | S |
| Estado | Borrador |
| Versión | 0.1.0 |
| Análisis Previo | SOLID analysis completado |

## 🎯 Objetivo de Negocio

Implementar **limpieza automática de recursos** cuando entidades son destruidas, evitando memory leaks en el heap de WASM. Actualmente `despawn()` solo limpia el SpatialHash, dejando mappings huérfanos en LogicSystem.

**Problema actual**: Memory leaks potenciales cuando sensores/actuators no se limpian al destruir entidades.

**Solución propuesta**: Hook de cleanup que desconecta todos los Logic Bricks asociados a una entidad.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-logic` (Resource Management)
- **Aggregate Root**: `LogicSystem` (cleanup coordinator)
- **Domain Events**: `EntityDestroyed`
- **Services**: `cleanup_entity()`, `detach_all_behaviors()`

## 📖 Contexto Arquitectural

### Problema Actual

```
Entity.destroy() ──────┬──► store.despawn() ✅
                       │
                       ├──► spatial_hash.remove() ✅
                       │
                       └──► logic_system.cleanup() ❌ FALTA
                           ├──► sensors[entity].remove() ?
                           ├──► actuators[entity].remove() ?
                           └──► wiring[entity].clear() ?
```

### Sistema Propuesto

```
Entity.destroy() ──────┬──► store.despawn() ✅
                       ├──► spatial_hash.remove() ✅
                       └──► logic_system.on_entity_destroyed(id)
                           ├──► disconnect_all_sensors(id)
                           ├──► disconnect_all_actuators(id)
                           ├──► clear_wiring(id)
                           └──► emit EntityDestroyed event
```

## 📖 Historias de Usuario

### HU-LOGIC-CLEANUP-001: Entity Destroy Hook

**Como** sistema de lógica
**Quiero** un hook que se ejecute cuando una entidad es destruida
**Para** limpiar todos los recursos asociados

#### Criterios de Aceptación
- [ ] `LogicSystem::on_entity_destroyed(entity_id)` existe
- [ ] Limpia sensores de la entidad
- [ ] Limpia actuadores de la entidad
- [ ] Limpia wiring mappings de la entidad
- [ ] Tests de memory leak

#### Tareas Técnicas
- [ ] Añadir `on_entity_destroyed()` a `LogicSystem`
- [ ] Implementar cleanup de sensors/actuators
- [ ] Integrar con `despawn()` en EntityStore
- [ ] Tests de cleanup
- [ ] Verificar sin memory leaks con `valgrind` equivalente

#### Investigación Previa
- [x] Despawn existe en store.rs L407
- [x] SpatialHash.remove() se llama
- [x] Crítica: Memory leaks en actuators (LOGIC_BRICKS_GUIDE.md L1055-1078)

#### Estimación: S
#### Estado: Pendiente

---

### HU-LOGIC-CLEANUP-002: EntityDestroyed Event

**Como** sistema de eventos
**Quiero** emitir eventos cuando entidades son destruidas
**Para** que otros sistemas puedan reaccionar al cleanup

#### Criterios de Aceptación
- [ ] `EntityDestroyed` event en EventRingBuffer
- [ ] Incluye entity_id y timestamp
- [ ] Otros sistemas pueden suscribirse
- [ ] Tests de eventos

#### Tareas Técnicas
- [ ] Añadir `EntityDestroyed` a `LogicEvent` enum
- [ ] Emitir en `on_entity_destroyed()`
- [ ] Integrar con EventRingBuffer
- [ ] Tests de integración

#### Estimación: XS
#### Estado: Pendiente

---

## 🔬 Arquitectura Técnica

### Hook de Cleanup

```rust
// crates/archflow-logic/src/logic_system.rs

impl LogicSystem {
    /// Cleanup all resources associated with an entity
    /// Called when entity is destroyed (before or after store.despawn)
    pub fn on_entity_destroyed(&mut self, entity_id: EntityId) {
        // 1. Emit event for UI/other systems
        self.event_ring_buffer.push(LogicEvent {
            event_type: LogicEventType::EntityDestroyed,
            entity_id: entity_id.index().0,
            timestamp: self.timestamp,
            data: EventData::None,
        });

        // 2. Clear all sensor state for this entity
        self.mouse_over.reset_entity(entity_id);
        self.mouse_click.reset_entity(entity_id);
        self.double_tap.reset_entity(entity_id);
        self.long_press.reset_entity(entity_id);
        self.right_click.reset_entity(entity_id);

        // 3. Clear actuator state for this entity
        // actuators don't typically track per-entity state
        // but we could add if needed

        // 4. Clear wiring/connections for this entity
        self.wiring.clear_entity(entity_id);

        // 5. Remove from spatial hash (already done in store)
        self.spatial_hash.remove(entity_id);

        // 6. Clear any entity-specific signals in SignalByte
        self.signal_byte.clear_entity(entity_id);
    }
}
```

### Reset por Sensor

```rust
// En cada sensor, añadir:

impl MouseOverSensor {
    pub fn reset_entity(&mut self, entity_id: EntityId) {
        let idx = entity_id.index().0 as usize;
        if idx < self.signals.len() {
            // Reset signal to None
            self.signals[idx] = SignalByte::from(0);
        }
    }
}
```

### Integración con EntityStore

```rust
// crates/archflow-engine/src/store.rs

impl EntityStore {
    pub fn despawn(&mut self, id: EntityId) -> bool {
        let index = id.index().0 as usize;

        // ... existing validation code ...

        // NEW: Notify LogicSystem before cleanup
        if let Some(ref mut logic) = self.logic_system.take() {
            logic.on_entity_destroyed(id);
        }

        // ... existing cleanup code ...
    }
}
```

## 📊 Estado de Tareas

| Historia | Estado | Tests | Debt Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-LOGIC-CLEANUP-001 | ⏳ Pendiente | 0/8 | - | - |
| HU-LOGIC-CLEANUP-002 | ⏳ Pendiente | 0/4 | - | - |

## 📋 Criterios de Éxito

- [ ] No memory leaks con 10k create/destroy cycles
- [ ] cleanup en <1ms para 100k entidades
- [ ] 100% test coverage

## 📋 Dependencias

- Depende de: `EPIC-LOGIC-EVENTS-001` (EventRingBuffer)

## 📋 Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Circular references | Alto | Baja | ownership analysis |
| Drop order issues | Medio | Baja | tests |

## 📋 Timeline

```
Semana 3:
- D1-D2: HU-LOGIC-CLEANUP-001 (Entity Destroy Hook)
- D3: HU-LOGIC-CLEANUP-002 (EntityDestroyed Event)
```

## 📚 Documentación Relacionada

- `docs/integration/LOGIC_BRICKS_DEVELOPER_GUIDE.md` L1055-1078 (Memory Leaks)
- `crates/archflow-engine/src/store.rs` (despawn)
