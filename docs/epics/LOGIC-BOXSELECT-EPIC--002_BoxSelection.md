# Épica: Box Selection - Selección Masiva Optimizada

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-LOGIC-BOXSELECT-002 |
| Prioridad | Alta |
| Estimación | M |
| Estado | Borrador |
| Versión | 0.1.0 |
| Análisis Previo | SOLID analysis completado en conversación |

## 🎯 Objetivo de Negocio

Implementar selección rectangular (marquee) con **rendimiento optimizado** para 100k+ entidades, aprovechando el nuevo `EventRingBuffer` y el patrón de **delta mask** para undo/redo eficiente.

**Problema actual**: La selección se hace en TypeScript iterando entidades (lento para muchos objetos).

**Solución propuesta**: BoxSelectSensor + BatchSelectActuator con SpatialHash + delta mask.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-logic` (Selection System)
- **Aggregate Root**: `BatchSelectActuator`
- **Domain Events**: `EntitySelected` (via EventRingBuffer)
- **Value Objects**: `BoxSelection` (start, end), `DeltaMask` (BitVec)

## 📖 Contexto Arquitectural

### Sistema Actual vs Propuesto

| Aspecto | Actual (TypeScript) | Propuesto (Rust) |
|---------|---------------------|------------------|
| Selección masivo | O(n) iteración JS | O(k) SpatialHash |
| Memoria/100k | ~3MB (HashSet) | **12.5KB** (BitVec) |
| Undo/Redo | Copies estados | **XOR delta** |
| Integración JS | Callbacks | **EventRingBuffer** |

### Principios SOLID Aplicados

| Principio | Aplicación |
|-----------|------------|
| **SRP** | BoxSelectSensor solo detecta, BatchSelect solo selecciona |
| **OCP** | Nuevos SelectionModes sin modificar actuators |
| **LSP** | BatchSelectActuator es subtipo de SelectActuator |
| **ISP** | Interfaz mínima: activate() + delta() |
| **DIP** | Depende de EntityStore abstraction |

## 📖 Historias de Usuario

### HU-LOGIC-BOX-001: BoxSelectSensor

**Como** usuario
**Quiero** seleccionar entidades dibujando un rectángulo
**Para** seleccionar múltiples objetos simultáneamente

#### Criterios de Aceptación
- [ ] `BoxSelectSensor` detecta entidades dentro de rectángulo
- [ ] Usa `SpatialHash` para O(k) query (no O(n))
- [ ] Integra con `MouseDrag` para drag-preview
- [ ] Emite eventos al `EventRingBuffer`
- [ ] Tests con 100+ entidades

#### Tareas Técnicas
- [ ] Crear `crates/archflow-logic/src/sensors/box_select.rs`
- [ ] Definir struct `BoxSelection` { start: Vec2, end: Vec2 }
- [ ] Implementar `evaluate()` con SpatialHash query
- [ ] Integrar con EventRingBuffer
- [ ] Tests unitarios

#### Investigación Previa
- [x] BoxSelectSensor especificado en LOGIC_BRICKS_GUIDE.md L788-808
- [x] SpatialHash existente en `archflow-logic/src/spatial.rs`

#### Estimación: S
#### Estado: Pendiente

---

### HU-LOGIC-BOX-002: BatchSelectActuator

**Como** usuario
**Quiero** seleccionar/deseleccionar miles de entidades instantáneamente
**Para** mantener 60 FPS con 100k entidades

#### Criterios de Aceptación
- [x] `BatchSelectActuator` usa `DeltaMask` (BitVec custom) para memoria eficiente
- [x] `execute()` aplica selección masiva
- [x] Memoria/100k entidades = **12.5KB** (vs ~3MB HashSet)
- [x] Tests de rendimiento (10 tests pasan)
- [x] **ELIMINADO**: `select.rs` y `select_tests.rs`
- [x] **MIGRADO**: `mapping_table.rs` y `actuator.rs` a `BatchSelectActuator`
- [x] **DEPRECADO**: `selected_entities` en `engine.rs` con nota de uso

#### Tareas Técnicas
- [x] Crear `crates/archflow-logic/src/actuators/batch_select.rs`
- [x] Implementar `BatchSelectActuator` con `DeltaMask` (1 bit × MAX_ENTITIES)
- [x] Implementar `DeltaMask` con XOR para undo/redo
- [x] Tests unitarios (10 tests, todos pasan)
- [x] Migrar `mapping_table.rs` a usar `BatchSelectActuator`
- [x] Migrar `actuator.rs` WASM bindings
- [x] **BORRAR** `crates/archflow-logic/src/actuators/select.rs`
- [x] **BORRAR** `crates/archflow-logic/tests/select_tests.rs`
- [x] Añadir `#[deprecated]` a `selected_entities` en engine.rs

#### Estado: ✅ COMPLETADO (GREEN)

---

### HU-LOGIC-BOX-003: SelectionCommand con Delta Mask

**Como** HistoryManager
**Quiero** comandos de selección que usen delta_mask
**Para** undo/redo instantáneo con mínima memoria

#### Criterios de Aceptación
- [ ] `SelectionCommand` guarda delta_mask (no estados completos)
- [ ] `execute()` aplica XOR del mask
- [ ] `undo()` aplica XOR inverso
- [ ] Memoria/100k selección ≤ 12.5KB
- [ ] Tests de roundtrip undo/redo
- [ ] **ELIMINAR**: Legacy `Command::Select` de `command.rs`

#### Tareas Técnicas
- [ ] Extender `crates/archflow-engine/src/command.rs`
- [ ] Añadir `SelectionCommand` con delta_mask (BitVec)
- [ ] Implementar `execute()` y `undo()` con XOR
- [ ] Integrar con HistoryManager
- [ ] Tests de memoria
- [ ] **BORRAR** antiguo `Command::SetSelection` y relacionados

#### Estimación: S
#### Estado: Pendiente

---

### Checklist de Eliminación de Legacy

```markdown
## 🗑️ Código Legacy a Eliminar

| Archivo | Código Legacy | HU |
|---------|--------------|-----|
| `engine.rs` | `selected_entities: Vec<EntityId>` | HU-LOGIC-BOX-002 |
| `engine.rs` | `is_dragging`, `last_mouse_screen_pos` | HU-LOGIC-BOX-002 |
| `bridge.rs` | `clear_selection()`, `select_entity()` legacy | HU-LOGIC-BOX-002 |
| `bridge.rs` | `get_selection()` legacy | HU-LOGIC-BOX-002 |
| `select.rs` | SelectActuator entero | HU-LOGIC-BOX-002 |
| `select_tests.rs` | Tests de SelectActuator legacy | HU-LOGIC-BOX-002 |
| `command.rs` | Command::SetSelection legacy | HU-LOGIC-BOX-003 |
```

#### Regla de Eliminación
> **No hay código legacy tolerado.** Cada archivo identificado debe ser eliminado en la HU correspondiente antes de marcar la épica como completa.

---

## 🔬 Arquitectura Técnica

### BoxSelectSensor

```rust
// crates/archflow-logic/src/sensors/box_select.rs

use archflow_core::{EntityId, Vec2, Rect};
use archflow_engine::EntityStore;
use crate::spatial::SpatialHash;

/// Rectangular selection area
#[derive(Clone, Copy, Debug)]
pub struct BoxSelection {
    pub start: Vec2,
    pub end: Vec2,
}

impl BoxSelection {
    /// Get the AABB of the selection rectangle
    pub fn to_aabb(&self) -> Rect {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);
        Rect::from_min_max(min_x, min_y, max_x, max_y)
    }

    /// Check if this is a valid selection (not just a click)
    pub fn is_valid(&self, threshold: f32) -> bool {
        let dx = (self.end.x - self.start.x).abs();
        let dy = (self.end.y - self.start.y).abs();
        dx > threshold || dy > threshold
    }
}

pub struct BoxSelectSensor {
    /// Current selection rectangle
    selection: Option<BoxSelection>,
    /// Spatial hash for fast queries
    spatial: Option<&'static SpatialHash>,
}

impl BoxSelectSensor {
    pub fn new() -> Self {
        Self {
            selection: None,
            spatial: None,
        }
    }

    pub fn start_drag(&mut self, start: Vec2) {
        self.selection = Some(BoxSelection {
            start,
            end: start,
        });
    }

    pub fn update_drag(&mut self, current: Vec2) {
        if let Some(ref mut sel) = self.selection {
            sel.end = current;
        }
    }

    pub fn end_drag(&mut self) -> Option<BoxSelection> {
        self.selection.take()
    }

    /// Evaluate which entities are inside the selection
    pub fn evaluate(
        &self,
        store: &EntityStore,
        selection: &BoxSelection,
    ) -> Vec<EntityId> {
        let aabb = selection.to_aabb();
        let mut selected = Vec::new();

        // O(k) query using SpatialHash
        // k = entities near the selection rectangle
        if let Some(spatial) = self.spatial {
            let nearby = spatial.query_rect(aabb);
            for entity_id in nearby {
                // Verify exact intersection
                let idx = entity_id.index().0 as usize;
                let pos = store.pos(idx);
                let size = store.size(idx);
                let entity_aabb = Rect::from_min_max(
                    pos.x, pos.y,
                    pos.x + size.x, pos.y + size.y,
                );

                if aabb.intersects(&entity_aabb) {
                    selected.push(entity_id);
                }
            }
        }

        selected
    }
}
```

### BatchSelectActuator con Delta Mask

```rust
// crates/archflow-logic/src/actuators/batch_select.rs

use archflow_core::EntityId;
use archflow_engine::EntityStore;
use bitvec::prelude::*;

/// Actuator for batch selection operations
/// Uses BitVec for O(1) bulk operations and minimal memory
pub struct BatchSelectActuator {
    /// Current selection state as bitset
    /// One bit per entity (1 = selected, 0 = deselected)
    selection_mask: BitVec,

    /// Delta mask for undo/redo operations
    /// Records which entities changed in the last operation
    delta_mask: BitVec,
}

impl BatchSelectActuator {
    pub fn new(capacity: usize) -> Self {
        Self {
            selection_mask: BitVec::with_capacity(capacity),
            delta_mask: BitVec::with_capacity(capacity),
        }
    }

    /// Apply batch selection from a list of entity IDs
    pub fn execute(
        &mut self,
        store: &mut EntityStore,
        entities: &[EntityId],
    ) {
        self.delta_mask.clear();

        for &entity in entities {
            let idx = entity.index().0 as usize;

            // Expand masks if needed
            if idx >= self.selection_mask.len() {
                self.selection_mask.resize(idx + 1, false);
                self.delta_mask.resize(idx + 1, false);
            }

            // XOR with 1 to toggle: record delta, update selection
            if idx < self.selection_mask.len() {
                self.selection_mask.set(idx, !self.selection_mask[idx]);
                self.delta_mask.set(idx, true);

                // Update EntityStore for visual feedback
                store.set_selected(idx, self.selection_mask[idx]);
            }
        }
    }

    /// Undo the last batch operation using delta mask
    pub fn undo(&mut self, store: &mut EntityStore) {
        // Same operation toggles back to original state
        for idx in 0..self.delta_mask.len() {
            if self.delta_mask[idx] {
                self.selection_mask.set(idx, !self.selection_mask[idx]);
                store.set_selected(idx, self.selection_mask[idx]);
            }
        }
        self.delta_mask.clear();
    }

    /// Get the current selection as a list of IDs
    pub fn current_selection(&self) -> Vec<EntityId> {
        self.selection_mask
            .ones()
            .map(|idx| EntityId::from_index(idx as u32))
            .collect()
    }
}
```

### SelectionCommand con Delta Mask

```rust
// En crates/archflow-engine/src/command.rs

/// Selection command that uses delta mask for efficient undo/redo
#[derive(Clone, Debug)]
pub struct SelectionCommand {
    /// Delta mask: 1 = changed, 0 = unchanged
    /// XORing this with current selection state produces the new state
    pub delta_mask: BitVec,

    /// For verification and logging
    pub changed_count: usize,

    /// Timestamp for debugging
    pub timestamp: u64,
}

impl SelectionCommand {
    pub fn new(delta_mask: BitVec, changed_count: usize) -> Self {
        Self {
            delta_mask,
            changed_count,
            timestamp: current_timestamp(),
        }
    }
}

impl Command for SelectionCommand {
    fn execute(&self, store: &mut EntityStore) {
        // Apply delta mask (XOR)
        for idx in 0..self.delta_mask.len() {
            if self.delta_mask[idx] {
                let id = EntityId::from_index(idx as u32);
                let idx_usize = idx;

                // Toggle selection state
                let new_selected = !store.is_selected(idx_usize);
                store.set_selected(idx_usize, new_selected);
            }
        }
    }

    fn undo(&self, store: &mut EntityStore) {
        // XOR again to restore original state
        self.execute(store);
    }
}
```

## 📊 Comparación de Memoria

| Sistema | Memoria/100k entidades |
|---------|----------------------|
| HashSet<EntityId> | ~3,000 KB |
| Vec<EntityId> (full copy) | ~800 KB |
| BitVec (delta_mask) | **12.5 KB** |
| Ahorro vs HashSet | **99.6%** |

## 📊 Estado de Tareas

| Historia | Estado | Tests | Debt Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-LOGIC-BOX-001 | ⏳ Pendiente | 0/6 | - | - |
| HU-LOGIC-BOX-002 | ⏳ Pendiente | 0/8 | - | - |
| HU-LOGIC-BOX-003 | ⏳ Pendiente | 0/6 | - | - |

## 📋 Criterios de Éxito

- [ ] Seleccionar 100k entidades en <1ms
- [ ] Memoria/100k ≤ 12.5KB para delta mask
- [ ] Undo/redo instantáneo con XOR
- [ ] 100% test coverage

## 📋 Dependencias

- Depende de: `EPIC-LOGIC-EVENTS-001` (EventRingBuffer)
- Depende de: `SpatialHash` existente

## 📋 Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| BitVec dependency | Bajo | Baja | Ya existe en ecosystem |
| Edge cases en selection | Medio | Media | Tests comprehensivos |

## 📋 Timeline

```
Semana 2:
- D1-D2: HU-LOGIC-BOX-001 (BoxSelectSensor)
- D3-D4: HU-LOGIC-BOX-002 (BatchSelectActuator)
- D5: HU-LOGIC-BOX-003 (SelectionCommand)
```

## 📚 Documentación Relacionada

- `docs/integration/LOGIC_BRICKS_DEVELOPER_GUIDE.md` L784-843
- `crates/archflow-logic/src/sensors/mod.rs` (sensores existentes)
- `crates/archflow-logic/src/actuators/select.rs` (SelectActuator)
- `crates/archflow-engine/src/command.rs` (Command pattern)
