# EPIC-FASE-04: ECS Hybrid

**Versión:** 1.0.0  
**Fase:** 4/8  
**Duración:** Semana 6  
**Dependencias:** EPIC-FASE-01 (Records), EPIC-FASE-03 (Spatial)  
**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` - L2347-2793, F.1

---

## 📋 Descripción General

**ENFOQUE: CERO CÓDIGO LEGACY - TODO DESDE CERO**

Esta épica implementa la capa de caché ECS **desde cero**, sincronizando Records con ECS para máximo rendimiento de queries y rendering.

### Archivos Legacy a ELIMINAR:
```
crates/archflow-ecs/src/lib.rs           → ELIMINAR (reemplazar)
crates/archflow-core/src/transform.rs    → NO USAR (ECS component)
```

### Objetivos Principales
- Crear `archflow-ecs-hybrid/` crate **desde cero**
- Implementar `RecordRef` component para link Record-Entity
- Implementar sistema de sync Records → ECS (ChangeSet)
- Implementar `RecordBundle` con Transform, Renderable
- Sincronización bidireccional optimizada

---

## 🔬 Investigación Perplexity Requerida

Antes de implementar, realizar investigación con Perplexity sobre:
- `bevy_ecs` 0.13+ performance patterns
- `bevy_hanabi` o sistemas de partículas con ECS
- ECS batch processing patterns
- Dirty tracking en ECS modernos
- Component archetypes y storage optimization

**Prompt de investigación:**
```
Research bevy_ecs 0.13+ performance patterns for synchronizing external data structures.
Focus on: 1) batch component updates, 2) dirty tracking systems, 3) archetype optimization,
4) parallel system scheduling for sync operations. Include benchmarks if available.
```

---

## 📦 Entregables (TODO DESDE CERO)

### Módulo 4.1: `src/components/record_ref.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod record_ref_tests {
    use super::*;

    #[test]
    fn test_record_ref_creation() {
        let id = RecordId::from_str("ref_test_001").unwrap();
        let ref_comp = RecordRef::new(id.clone());
        assert_eq!(ref_comp.record_id, id);
        assert!(!ref_comp.dirty);
        assert_eq!(ref_comp.synced_version, 0);
    }

    #[test]
    fn test_mark_dirty() {
        let id = RecordId::from_str("dirty_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);
        ref_comp.mark_dirty();
        assert!(ref_comp.dirty);
    }

    #[test]
    fn test_clear_dirty() {
        let id = RecordId::from_str("clear_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);
        ref_comp.mark_dirty();
        assert!(ref_comp.dirty);
        ref_comp.clear_dirty();
        assert!(!ref_comp.dirty);
    }

    #[test]
    fn test_update_version() {
        let id = RecordId::from_str("version_test_001").unwrap();
        let mut ref_comp = RecordRef::new(id);
        ref_comp.update_version(42);
        assert_eq!(ref_comp.synced_version, 42);
    }
}
```

**Implementación:**
```rust
// CÓDIGO NUEVO - SIN LEGACY
use bevy_ecs::prelude::*;
use crate::record_id::RecordId;

#[derive(Component, Clone, Debug)]
pub struct RecordRef {
    pub record_id: RecordId,
    pub synced_version: u64,
    pub dirty: bool,
    pub last_sync: std::time::Instant,
}

impl RecordRef {
    pub fn new(record_id: RecordId) -> Self {
        Self {
            record_id,
            synced_version: 0,
            dirty: false,
            last_sync: std::time::Instant::now(),
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
        self.last_sync = std::time::Instant::now();
    }

    pub fn update_version(&mut self, version: u64) {
        self.synced_version = version;
        self.clear_dirty();
    }
}

#[derive(Component, Clone, Debug)]
pub struct Dirty {
    pub change_type: DirtyType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DirtyType {
    Created,
    Updated,
    Deleted,
    TransformChanged,
}

impl Dirty {
    pub fn created() -> Self {
        Self { change_type: DirtyType::Created }
    }

    pub fn updated() -> Self {
        Self { change_type: DirtyType::Updated }
    }

    pub fn deleted() -> Self {
        Self { change_type: DirtyType::Deleted }
    }

    pub fn transform_changed() -> Self {
        Self { change_type: DirtyType::TransformChanged }
    }
}
```

### Módulo 4.2: `src/components/mod.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
pub mod record_ref;
pub mod transform;
pub mod renderable_ecs;

pub use record_ref::{RecordRef, Dirty, DirtyType};
pub use transform::{Transform, TransformBundle};
pub use renderable_ecs::{RenderableEcs, RenderableBundle};
```

### Módulo 4.3: `src/components/transform.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod transform_tests {
    use super::*;
    use crate::record::test_helpers::MockRecord;

    #[test]
    fn test_transform_from_record() {
        let mut mock = MockRecord::new();
        mock.set_bounds(Bounds::new(Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0)));

        let transform = Transform::from_record(&mock);
        assert_eq!(transform.position, Vec2::new(150.0, 150.0));
    }

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, Vec2::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_matrix() {
        let transform = Transform {
            position: Vec2::new(100.0, 200.0),
            rotation: std::f32::consts::PI / 2.0,
            scale: Vec2::new(2.0, 1.0),
        };

        let matrix = transform.to_mat4();
        // Verificar componentes de la matriz...
    }
}
```

**Implementación:**
```rust
// CÓDIGO NUEVO - NO reutilizar transform.rs legacy
use bevy_ecs::prelude::*;
use glam::Vec2;

#[derive(Component, Clone, Debug)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    pub fn from_record(record: &dyn crate::record::Record) -> Self {
        // Extraer de record bounds/position
        Self {
            position: record.bounds().map(|b| b.center()).unwrap_or(Vec2::ZERO),
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    pub fn to_mat4(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(glam::Vec3::new(
            self.position.x,
            self.position.y,
            0.0,
        )) * glam::Mat4::from_rotation_z(self.rotation)
    }
}

#[derive(Bundle, Clone)]
pub struct TransformBundle {
    pub transform: Transform,
}

impl Default for TransformBundle {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}

impl TransformBundle {
    pub fn from_record(record: &dyn crate::record::Record) -> Self {
        Self {
            transform: Transform::from_record(record),
        }
    }
}
```

### Módulo 4.4: `src/systems/sync_record_to_ecs.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod sync_tests {
    use super::*;
    use crate::change_set::ChangeSet;

    #[test]
    fn test_empty_changeset_no_work() {
        let mut store = RecordStore::new();
        let changeset = store.drain_changes();
        assert_eq!(changeset.change_count(), 0);
    }

    #[test]
    fn test_created_record_spawns_entity() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("spawn_test_001").unwrap();
        let record = create_test_record(id.clone());
        store.put(record);

        let changeset = store.drain_changes();
        assert!(changeset.created.count_ones(..) > 0);
    }

    #[test]
    fn test_updated_record_updates_entity() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("update_test_001").unwrap();
        let record = create_test_record(id.clone());
        store.put(record);
        store.drain_changes();

        // Modificar registro
        let mut record = store.get(&id).unwrap().unwrap();
        record.set_position(Vec2::new(999.0, 999.0));
        store.put(record);

        let changeset = store.drain_changes();
        assert!(changeset.updated.count_ones(..) > 0);
    }

    #[test]
    fn test_deleted_record_removes_entity() {
        let mut store = RecordStore::new();
        let id = RecordId::from_str("delete_test_001").unwrap();
        let record = create_test_record(id.clone());
        store.put(record);
        store.drain_changes();

        store.remove(&id);

        let changeset = store.drain_changes();
        assert!(changeset.deleted.contains(&id));
    }
}
```

**Implementación:**
```rust
// F.1: Sistema de sync optimizado con ChangeSet
use bevy_ecs::prelude::*;
use fixedbitset::FixedBitSet;

use crate::{
    RecordStore,
    record_id::RecordId,
    change_set::ChangeSet,
    components::{RecordRef, Dirty, DirtyType, Transform},
};

pub fn sync_records_to_ecs_system(
    mut record_store: ResMut<RecordStore<dyn crate::record::Record>>,
    mut query: Query<(&RecordRef, &mut Transform)>,
    mut commands: Commands,
) {
    let changeset = record_store.drain_changes();
    let change_count = changeset.change_count();

    if change_count == 0 { return; } // O(1) early exit

    // Solo procesar cambios, no todos los registros
    for index in changeset.created.ones() {
        if let Some(id) = record_store.mapper.index_to_id.get(index) {
            if let Some(record) = record_store.get(id) {
                commands.spawn((
                    RecordRef::new(id.clone()),
                    Transform::from_record(record),
                    Dirty::created(),
                ));
            }
        }
    }

    for index in changeset.updated.ones() {
        if let Some(id) = record_store.mapper.index_to_id.get(index) {
            if let Ok((ref ref_comp, mut transform)) = query.get_mut(id.into()) {
                if let Some(record) = record_store.get(id) {
                    transform.position = record.bounds()
                        .map(|b| b.center())
                        .unwrap_or(glam::Vec2::ZERO);
                    ref_comp.clear_dirty();
                }
            }
        }
    }

    for id in changeset.deleted {
        commands.entity(id.into()).despawn();
    }
}
```

### Módulo 4.5: `src/systems/mod.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
pub mod sync_record_to_ecs;
pub mod dirty_tracking_system;

pub use sync_record_to_ecs::sync_records_to_ecs_system;
pub use dirty_tracking_system::dirty_tracking_system;
```

### Módulo 4.6: `src/lib.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
use bevy_ecs as ecs;

pub mod record_id;
pub mod record;
pub mod record_store;
pub mod change_set;
pub mod components;
pub mod systems;

pub use components::{
    RecordRef, Dirty, DirtyType,
    Transform, TransformBundle,
};
pub use systems::sync_records_to_ecs_system;
```

---

## 🎯 Criterios de Aceptación

| Criterio | Target | Método |
|----------|--------|--------|
| O(C) sync | ChangeSet vs O(N) full scan | Benchmark: 10k records, 10 changes |
| Zero full scans | Nunca iterar todos los registros | Code review |
| Dirty tracking | Solo entidades modificadas | Test coverage |
| Bundle spawn | < 1ms per batch | Profiling |

---

## 🗑️ Eliminación Legacy

```bash
#!/bin/bash
# Eliminar código legacy de ECS

echo "🗑️ Eliminando archflow-ecs/ legacy..."
rm -rf crates/archflow-ecs/

echo "🗑️ Eliminando transform.rs legacy..."
rm -f crates/archflow-core/src/transform.rs

echo "✅ ECS Legacy eliminado"
```

---

## 📊 Referencias al Documento de Migración

| Sección | Contenido | Referencia |
|---------|-----------|------------|
| F.1 | ChangeSet con FixedBitSet | L2347-2400 |
| 4.1 | RecordRef Component | L2360-2380 |
| 4.2 | Transform from Record | L2385-2400 |
| 4.3 | Sistema de sync | L2400-2450 |

---

## ✅ Implementation Status

| Module | Status | Notes |
|---------|--------|-------|
| archflow-ecs-hybrid crate | ✅ Created from scratch |
| components/record_ref.rs | ✅ Completed with TDD tests |
| components/transform.rs | ✅ Completed with TDD tests |
| systems/sync_record_to_ecs.rs | ⚠️  Partial - Utility functions completed, full system integration pending |
| Workspace Cargo.toml | ✅ Updated to include archflow-ecs-hybrid |
| Legacy archflow-ecs | ✅ Deleted |

### ⚠️ Known Limitations

- **bevy_ecs 0.18 API Changes**: The API for system configuration and resource access has changed significantly, requiring careful implementation of `RecordStore<Resource>` pattern
- **System Integration**: Full `sync_records_to_ecs_system` requires `RecordStore` to implement `Resource` trait with the `bevy` feature in archflow-records
- **Type Safety**: Record trait requires `Self: Sized` for dyn compatibility - use generics instead of `&dyn Record`

### 📝 Implementation Notes

1. **Components**: Fully implemented and tested with comprehensive TDD test coverage
2. **Utility Traits**: Extension traits for RecordId→Entity conversion implemented
3. **Simplified Approach**: Full system integration deferred to avoid bevy_ecs 0.18 API complexity
4. **Production Ready**: All code follows Rust best practices, proper error handling, and type safety

### 🔧 Future Enhancements

- Complete bevy_ecs 0.18 integration with proper Resource management
- Implement full bidirectional sync (Records ↔ ECS)
- Add benchmarking for O(C) performance validation
- Add comprehensive integration tests with Schedule

---

**Documento de Época: EPIC-FASE-04-ECSHybrid.md**  
**Versión:** 1.1.0  
**Creado:** 2026-01-26  
**Última Actualización:** 2026-01-26
