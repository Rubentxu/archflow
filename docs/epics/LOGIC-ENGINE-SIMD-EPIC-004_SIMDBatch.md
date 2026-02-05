# Épica: SIMD Batch Operations - Optimización MoveGroup

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-ENGINE-SIMD-004 |
| Prioridad | Media |
| Estimación | M |
| Estado | Borrador |
| Versión | 0.1.0 |
| Análisis Previo | SOLID analysis completado |

## 🎯 Objetivo de Negocio

Implementar operaciones SIMD batch en `EntityStore` para mover jerarquías completas de 100k entidades con rendimiento óptimo, aprovechando la estructura SoA (Structure of Arrays) para vectorización automática.

**Problema actual**: `MoveGroup` itera linealmente, pero con SoA podemos procesar múltiples posiciones en paralelo.

**Solución propuesta**: `apply_delta_to_mask()` con SIMD vectorization para transformar masivas.

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-engine` (Entity Operations)
- **Aggregate Root**: `EntityStore` (batch operations)
- **Domain Events**: `TransformChanged` (lazy propagation)
- **Services**: `apply_delta_to_mask()`, `update_hierarchy_deep()`

## 📖 Contexto Arquitectural

### Sistema Actual vs Propuesto

| Aspecto | Actual | SIMD Batch |
|---------|--------|------------|
| MoveGroup | O(n) iter | O(n/4) con SIMD |
| Jerarquía profunda | Recursivo | Iterativo + vectorizado |
| Dirty tracking | Por-entidad | Por-batch |
| Transform update | Frame time | <1ms/100k |

### Principios SOLID Aplicados

| Principio | Aplicación |
|-----------|------------|
| **SRP** | apply_delta_to_mask solo transforma |
| **OCP** | SIMD可选 (feature flag) |
| **LSP** | Rust auto-vectorization preserva semantics |
| **ISP** | Interfaz: apply_delta_to_mask() |
| **DIP** | Usa EntityStore abstractions |

## 📖 Historias de Usuario

### HU-ENGINE-SIMD-001: apply_delta_to_mask()

**Como** EntityStore
**Quiero** aplicar deltas a múltiples entidades usando vectorización
**Para** mover 100k entidades en <1ms

#### Criterios de Aceptación
- [ ] `apply_delta_to_mask(&mask, delta)` implementado
- [ ] Auto-vectorización por LLVM
- [ ] Usa dirty tracking existente
- [ ] Benchmark: 100k entidades <1ms
- [ ] Tests de correctness

#### Tareas Técnicas
- [ ] Implementar `apply_delta_to_mask()` en `EntityStore`
- [ ] Usar iteradores chunked para SIMD
- [ ] Integrar con dirty_render tracking
- [ ] Benchmark de rendimiento
- [ ] Tests unitarios

#### Investigación Previa
- [x] apply_delta_to_mask especificado en LOGIC_BRICKS_GUIDE.md L440-452
- [x] EntityStore SoA existente en store.rs
- [x] dirty_render tracking existente

#### Estimación: M
#### Estado: Pendiente

---

### HU-ENGINE-SIMD-002: MoveGroup SIMD

**Como** usuario
**Quiero** mover un grupo de 100k entidades
**Para** mantener 60 FPS mientras reorganizo diagrams grandes

#### Criterios de Aceptación
- [ ] `MoveGroup` usa `apply_delta_to_mask()`
- [ ] Move group completo <1ms
- [ ] Jerarquía se propaga correctamente
- [ ] Undo/Redo con delta mask

#### Tareas Técnicas
- [ ] Modificar `Command::MoveGroup` para usar SIMD
- [ ] Propagar transforms a hijos
- [ ] Tests de rendimiento
- [ ] Documentar benchmark

#### Estimación: S
#### Estado: Pendiente

---

### HU-ENGINE-SIMD-003: Hierarchy Propagation Optimized

**Como** sistema de jerarquías
**Quiero** actualizar transforms de jerarquías profundas eficientemente
**Para** evitar lag cuando hay 100k entidades anidadas

#### Criterios de Aceptación
- [ ] `update_hierarchy_deep()` optimizado
- [ ] Topological sort si es necesario
- [ ] Solo entidades dirty se actualizan
- [ ] Benchmark: 10 niveles × 10k entidades <2ms

#### Tareas Técnicas
- [ ] Revisar `update_hierarchy()` actual
- [ ] Optimizar para SoA locality
- [ ] Implementar dirty propagation
- [ ] Tests de jerarquías anidadas

#### Estimación: S
#### Estado: Pendiente

---

## 🔬 Arquitectura Técnica

### apply_delta_to_mask() SIMD

```rust
// crates/archflow-engine/src/store.rs

impl EntityStore {
    /// Apply a delta transformation to all entities in the mask
    ///
    /// This method is optimized for SIMD vectorization by:
    /// 1. Using chunked iterators (4/8 elements per iteration)
    /// 2. Leveraging SoA memory layout (contiguous arrays)
    /// 3. Minimal branching for CPU pipeline efficiency
    ///
    /// # Arguments
    ///
    /// * `mask` - BitVec indicating which entities to transform
    /// * `delta` - Delta vector to apply (x, y)
    ///
    /// # Performance
    ///
    /// - O(n) where n = entities in mask
    /// - Auto-vectorized by LLVM to process 4-8 entities per cycle
    /// - Benchmark: 100k entities < 1ms on modern CPUs
    pub fn apply_delta_to_mask(&mut self, mask: &BitVec, delta: Vec2) {
        // Get mutable references to position arrays
        let positions = &mut self.transforms;

        // Chunked iteration for SIMD efficiency
        // Each chunk processes 4-8 entities depending on CPU
        let mask_bits = mask.as_bitslice();

        // Iterate in chunks for better cache locality
        for (idx, bit) in mask_bits.iter().enumerate() {
            if *bit {
                // Update position (x = transform[0], y = transform[1])
                positions[idx][0] += delta.x;
                positions[idx][1] += delta.y;

                // Mark dirty for GPU update
                self.dirty_render.insert(idx);
                self.dirty_transform.insert(idx);
            }
        }

        // Set z-order dirty flag
        self.dirty_z_order = true;
    }

    /// Apply batch delta with pre-computed chunked mask
    /// More efficient when called repeatedly with same mask
    pub fn apply_batch_delta(
        &mut self,
        indices: &[u32],
        delta: Vec2,
    ) {
        for &idx in indices {
            let idx_usize = idx as usize;
            self.transforms[idx_usize][0] += delta.x;
            self.transforms[idx_usize][1] += delta.y;
            self.dirty_render.insert(idx_usize);
        }
    }
}
```

### MoveGroup Optimizado

```rust
// crates/archflow-engine/src/command.rs

impl Command for MoveGroup {
    fn execute(&self, store: &mut EntityStore) {
        // Get all descendants of the root
        let descendants = store.get_descendants(self.root_id);

        // Build bitmask of affected entities
        let mut mask = BitVec::with_capacity(descendants.len() + 1);
        for idx in descendants {
            mask.push(true);
        }

        // Apply delta using SIMD-optimized method
        store.apply_delta_to_mask(&mask, self.delta);

        // Trigger hierarchy update if parent-child relationships exist
        if self.affects_hierarchy() {
            store.update_hierarchy();
        }
    }

    fn undo(&self, store: &mut EntityStore) {
        let inverse_delta = Vec2::new(-self.delta.x, -self.delta.y);
        let descendants = store.get_descendants(self.root_id);

        let mut mask = BitVec::with_capacity(descendants.len() + 1);
        for idx in descendants {
            mask.push(true);
        }

        store.apply_delta_to_mask(&mask, inverse_delta);

        if self.affects_hierarchy() {
            store.update_hierarchy();
        }
    }
}
```

### Jerarquía Profunda Optimizada

```rust
// crates/archflow-engine/src/store.rs

impl EntityStore {
    /// Update world transforms for entire hierarchy
    /// Optimized to only process dirty entities
    ///
    /// Algorithm:
    /// 1. Find all dirty roots (entities with dirty_hierarchy)
    /// 2. Traverse in breadth-first order (parents before children)
    /// 3. Propagate transforms to children
    /// 4. Mark children as dirty if parent moved
    pub fn update_hierarchy(&mut self) {
        // Find roots with dirty hierarchy
        let dirty_roots: Vec<usize> = self
            .dirty_hierarchy
            .ones()
            .filter(|&idx| {
                // Root has no parent OR parent is not dirty
                match self.parent_id[idx] {
                    None => true,
                    Some(parent) => {
                        let parent_idx = parent.index().0 as usize;
                        !self.dirty_hierarchy.contains(parent_idx)
                    }
                }
            })
            .collect();

        // BFS traversal for correct parent→child order
        let mut queue: Vec<usize> = dirty_roots;
        let mut processed = 0;

        while processed < queue.len() {
            let current_idx = queue[processed];
            processed += 1;

            // Find all children of current entity
            let children: Vec<usize> = self
                .parent_id
                .iter()
                .enumerate()
                .filter(|(_, &parent)| {
                    parent.map(|p| p.index().0 as usize) == Some(current_idx)
                })
                .map(|(idx, _)| idx)
                .collect();

            // Update world transform for each child
            for &child_idx in &children {
                if let Some(parent) = self.parent_id[child_idx] {
                    let parent_idx = parent.index().0 as usize;

                    // Child world = Parent world + Child local
                    self.world_transform[child_idx][0] =
                        self.world_transform[parent_idx][0] + self.local_transform[child_idx][0];
                    self.world_transform[child_idx][1] =
                        self.world_transform[parent_idx][1] + self.local_transform[child_idx][1];

                    // Mark child as dirty for rendering
                    self.dirty_render.insert(child_idx);
                    self.dirty_hierarchy.insert(child_idx);

                    // Add to queue for further propagation
                    queue.push(child_idx);
                }
            }
        }

        // Clear dirty flags
        self.dirty_hierarchy.clear();
    }
}
```

## 📊 Benchmarks Esperados

| Operación | Sin SIMD | Con SIMD | Speedup |
|-----------|----------|----------|---------|
| Move 100k entities | 4.2ms | **0.8ms** | 5.25x |
| Update hierarchy (10 niveles) | 8.5ms | **2.1ms** | 4.0x |
| apply_delta_to_mask | 3.1ms | **0.6ms** | 5.2x |

## 📊 Estado de Tareas

| Historia | Estado | Tests | Debt Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-ENGINE-SIMD-001 | ⏳ Pendiente | 0/8 | - | - |
| HU-ENGINE-SIMD-002 | ⏳ Pendiente | 0/6 | - | - |
| HU-ENGINE-SIMD-003 | ⏳ Pendiente | 0/8 | - | - |

## 📋 Criterios de Éxito

- [ ] Move 100k entidades <1ms
- [ ] apply_delta_to_mask 5x más rápido
- [ ] Update hierarchy jerarquías profundas <2ms
- [ ] Tests de correctness

## 📋 Dependencias

- Ninguna épica previa requerida
- Depende de: EntityStore existente, Command pattern

## 📋 Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| SIMD no auto-vectoriza | Medio | Baja | Profiling, manual vectorization |
| Branch prediction misses | Medio | Media | BitVec optimizations |

## 📋 Timeline

```
Semana 4:
- D1-D2: HU-ENGINE-SIMD-001 (apply_delta_to_mask)
- D3: HU-ENGINE-SIMD-002 (MoveGroup SIMD)
- D4-D5: HU-ENGINE-SIMD-003 (Hierarchy propagation)
```

## 📚 Documentación Relacionada

- `docs/integration/LOGIC_BRICKS_DEVELOPER_GUIDE.md` L434-508
- `crates/archflow-engine/src/store.rs` (EntityStore)
- `crates/archflow-engine/src/command.rs` (MoveGroup)
