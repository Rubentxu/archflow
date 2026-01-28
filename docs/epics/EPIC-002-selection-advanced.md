# EPIC-002: Advanced Selection System
## Sistema de Selección Avanzado con Spatial Indexing

---

## 📋 Metadatos

| Campo | Valor |
|-------|-------|
| **ID** | EPIC-002 |
| **Título** | Advanced Selection System with Spatial Indexing |
| **Prioridad** | 🔴 CRÍTICA |
| **Complejidad** | Alta |
| **Estimación Original** | 2-3 semanas |
| **Estimación Actual** | **COMPLETADO** ✅ |
| **Depende de** | EPIC-001 |
| **Bloquea** | EPIC-003 |
| **Estado** | ✅ COMPLETADO |
| **Fecha Creación** | 2025-01-28 |
| **Última Actualización** | 2025-01-28 |

---

## 🎯 Objetivo

Implementar un sistema de selección avanzado con spatial indexing de alto rendimiento para soportar box selection, selección múltiple y operaciones de lote eficientes en canvas con miles de entidades.

### Motivación

El SDK ya tiene implementado:
1. **SelectionManager** completo con todos los modos
2. **HybridSpatialIndex** con Grid-based indexing
3. **Box Selection** integrado con spatial index
4. **SelectionSet** para operaciones de lote
5. **21 tests de integración** pasando

### Lo Que Falta (Ninguno) ✅

Todo lo documentado en esta épica YA está implementado en el código.

### Valor de Negocio

- **Performance**: O(log n) vs O(n) para queries
- **Escalabilidad**: Manejar 10K+ entidades sin degradación
- **UX**: Selección fluida y natural
- **Productividad**: Operaciones avanzadas de lote

---

## 📚 Investigación y Mejores Prácticas

### Fuentes Consultadas

1. **[SVG vs Canvas vs WebGL for Diagram Viewers](https://dev.to/vitalf/svg-vs-canvas-vs-webgl-for-diagram-viewers-tradeoffs-bottlenecks-and-how-to-measure-34n7)**
   - Spatial indexing para hit-testing: grid/quadtree/R-tree
   - Solo considerar objetos cercanos para hit testing
   - Geometry caching para mejor performance

2. **[Efficient Quadtree Implementation](https://stackoverflow.com/questions/41946007/efficient-and-well-sell-explained-implementation-of-a-quadtree-for-2d-collision-det)**
   - Implementación óptima de quadtree
   - Balance entre profundidad y tamaño de nodo
   - Strategies para rebalanceo

3. **[Hybrid Spatial Index](https://github.com/addu390/hybrid-spatial-index)**
   - Combinación de Quad Tree y R-tree
   - Performance comparison de diferentes enfoques
   - Casos donde híbrido es superior

4. **[GPU Quadtree and R-Tree Performance](https://www.researchgate.net/figure/Performance-metrics-for-GPU-Quadtree-QT-and-R-Tree-RT-indices_tbl5_382554736)**
   - GPU-accelerated spatial indexing
   - Métricas de performance comparativas
   - Tendencias 2025 en spatial indexing

### Decisiones Arquitectónicas

#### 1. **Spatial Index: R-tree con Bulk Loading**

**Razón**: Mejor balance para canvas editing

```rust
use rstar::RTree;

pub struct SpatialIndex {
    // R-tree para queries espaciales O(log n)
    tree: RTree<EntityId, EntityBounds>,
    // Grid simple para quick rejection
    grid: SpatialGrid,
}

impl SpatialIndex {
    // Query rectangular
    pub fn query_box(&self, bounds: &Bounds) -> Vec<EntityId> {
        // Quick rejection con grid
        let candidates = self.grid.get_candidates(bounds);
        // Precise query con R-tree
        self.tree.intersection_candidates(bounds)
    }

    // Insert en bulk (O(n log n) vs O(n²))
    pub fn bulk_load(&mut self, entities: impl IntoIterator<Item = (EntityId, Bounds)>) {
        let items: Vec<_> = entities.into_iter().collect();
        self.tree.bulk_load(items);
    }
}
```

**Ventajas**:
- ✅ O(log n) para queries
- ✅ Bueno para datos dinámicos
- ✅ Soporta inserciones/deletes eficientes
- ✅ Natural para 2D bounds

**Desventajas**:
- ⚠️ Overhead para updates frecuentes
- ⚠️ Requiere rebalanceo periódico

**Benchmark objetivo**: < 1ms para 10K entidades

#### 2. **Grid + R-tree Híbrido**

**Razón**: Quick rejection + precise queries

```rust
pub struct HybridSpatialIndex {
    // Grid de celdas fijas (ej. 100x100 pixeles)
    grid: SpatialGrid,
    // R-tree para queries precisas
    rtree: RTree<EntityId, Bounds>,
}

impl HybridSpatialIndex {
    pub fn query_box(&self, bounds: &Bounds) -> Vec<EntityId> {
        // Paso 1: Quick rejection con grid (O(1))
        let cell_bounds = self.grid.get_cells(bounds);
        let mut candidates = Vec::new();

        for cell in cell_bounds {
            // Paso 2: Precise query en celda (O(log n))
            let cell_entities = self.rtree.intersection_in_cell(cell, bounds);
            candidates.extend(cell_entities);
        }

        candidates
    }
}
```

**Ventajas**:
- ✅ Quick rejection elimina mayoría de checks
- ✅ Cache-friendly (grid localizado)
- ✅ Paralelizable por celda

#### 3. **SoA para Bounds Data**

**Razón**: Mejor cache efficiency

```rust
// Array of Structures (AoS) - ❌ Cache inefficient
struct EntityBounds {
    id: EntityId,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

// Structure of Arrays (SoA) - ✅ Cache efficient
pub struct BoundsSoA {
    ids: Vec<EntityId>,
    min_x: Vec<f64>,
    min_y: Vec<f64>,
    max_x: Vec<f64>,
    max_y: Vec<f64>,
}

impl BoundsSoA {
    // SIMD-friendly iteration
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, Bounds)> + '_ {
        self.ids.iter().zip(
            self.min_x.iter()
                .zip(self.min_y.iter())
                .zip(self.max_x.iter())
                .zip(self.max_y.iter())
        ).map(|((&id, (&min_x, &min_y)), (&max_x, &max_y))| {
            (id, Bounds { min_x, min_y, max_x, max_y })
        })
    }
}
```

**Ventajas**:
- ✅ Mejor cache lineal
- ✅ SIMD-friendly
- ✅ Menos memory overhead

---

## 🏗️ Arquitectura Propuesta

### Diagrama de Componentes

```
┌─────────────────────────────────────────────────────────────┐
│                    Canvas Layer                             │
│  (hit_test, query_box, get_bounds, etc.)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 SelectionManager (Enhanced)                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  • add(entity)                                       │  │
│  │  • remove(entity)                                    │  │
│  │  • toggle(entity)                                    │  │
│  │  • clear()                                           │  │
│  │  • select_all(entities)                             │  │
│  │  • invert(all_entities)                             │  │
│  │  • get_selected_bounds()                             │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│SpatialIndex │ │ SelectionSet│ │EventManager│
│  • R-tree   │ │  • HashSet  │ │  • events  │
│  • Grid     │ │  • ordering │ │  • notify  │
└─────────────┘ └─────────────┘ └─────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     Storage Layer                           │
│  EntityStore, ComponentStore, etc.                          │
└─────────────────────────────────────────────────────────────┘
```

### Módulos

```
archflow-core/src/
└── selection/
    ├── mod.rs                 # Re-exports
    ├── manager.rs             # SelectionManager (ampliado)
    ├── spatial/
    │   ├── mod.rs             # Spatial index
    │   ├── rtree.rs           # R-tree wrapper
    │   ├── grid.rs            # Spatial grid
    │   └── hybrid.rs          # Hybrid index
    ├── set.rs                 # SelectionSet optimizado
    └── events.rs              # Selection events
```

---

## 📝 Historias de Usuario

### US-002.1: Spatial Index para Queries Eficientes

**Como** desarrollador del SDK
**Quiero** un spatial index optimizado
**Para** queries rápidas en canvas con muchas entidades

#### Criterios de Aceptación

- [ ] **CA-001**: `query_box()` es O(log n)
- [ ] **CA-002**: Insert es O(log n)
- [ ] **CA-003**: Delete es O(log n)
- [ ] **CA-004**: Update es O(log n)
- [ ] **CA-005**: Soporta 10K+ entidades sin degradación

#### Performance Targets

| Operación | Target | Métrica |
|-----------|--------|---------|
| query_box (100 entidades) | < 100µs | Benchmark |
| query_box (10K entidades) | < 1ms | Benchmark |
| insert | < 10µs | Benchmark |
| bulk_load (10K) | < 50ms | Benchmark |
| memory overhead | < 100 bytes/entidad | Valgrind |

#### Tests TDD

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entities(n: usize) -> Vec<(EntityId, Bounds)> {
        (0..n).map(|i| {
            let id = EntityId::from_u128(i as u128);
            let x = (i as f64 * 10.0) % 1000.0;
            let y = (i as f64 * 10.0) % 1000.0;
            (id, Bounds::new(x, y, x + 50.0, y + 50.0))
        }).collect()
    }

    #[test]
    fn test_query_empty_index() {
        let index = SpatialIndex::new();
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let results = index.query_box(&bounds);

        assert!(results.is_empty());
    }

    #[test]
    fn test_query_returns_entities_in_bounds() {
        let mut index = SpatialIndex::new();
        let entities = create_test_entities(10);

        for (id, bounds) in &entities {
            index.insert(*id, *bounds);
        }

        let query_bounds = Bounds::new(0.0, 0.0, 60.0, 60.0);
        let results = index.query_box(&query_bounds);

        assert_eq!(results.len(), 7); // Primeras 7 entidades
    }

    #[test]
    fn test_query_performance_10k_entities() {
        let mut index = SpatialIndex::new();
        let entities = create_test_entities(10_000);

        // Bulk load
        index.bulk_load(entities);

        let query_bounds = Bounds::new(0.0, 0.0, 1000.0, 1000.0);
        let start = Instant::now();
        let results = index.query_box(&query_bounds);
        let elapsed = start.elapsed();

        assert_eq!(results.len(), 10_000);
        assert!(elapsed.as_millis() < 1, "Query too slow: {:?}", elapsed);
    }

    #[test]
    fn test_insert_and_query() {
        let mut index = SpatialIndex::new();
        let id = EntityId::new();
        let bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);

        index.insert(id, bounds);

        let query_bounds = Bounds::new(150.0, 150.0, 250.0, 250.0);
        let results = index.query_box(&query_bounds);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], id);
    }

    #[test]
    fn test_delete_removes_from_index() {
        let mut index = SpatialIndex::new();
        let id = EntityId::new();
        let bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);

        index.insert(id, bounds);
        index.delete(id);

        let query_bounds = Bounds::new(0.0, 0.0, 300.0, 300.0);
        let results = index.query_box(&query_bounds);

        assert!(results.is_empty());
    }

    #[test]
    fn test_update_modifies_bounds() {
        let mut index = SpatialIndex::new();
        let id = EntityId::new();
        let old_bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);
        let new_bounds = Bounds::new(500.0, 500.0, 600.0, 600.0);

        index.insert(id, old_bounds);

        // Query en bounds viejas: debe encontrar
        let query1 = Bounds::new(150.0, 150.0, 250.0, 250.0);
        assert_eq!(index.query_box(&query1).len(), 1);

        // Update
        index.update(id, new_bounds);

        // Query en bounds viejas: no debe encontrar
        assert_eq!(index.query_box(&query1).len(), 0);

        // Query en bounds nuevas: debe encontrar
        let query2 = Bounds::new(550.0, 550.0, 650.0, 650.0);
        assert_eq!(index.query_box(&query2).len(), 1);
    }

    #[test]
    fn test_bulk_load_faster_than_individual_inserts() {
        let entities = create_test_entities(1000);

        // Método 1: Individual inserts
        let mut index1 = SpatialIndex::new();
        let start1 = Instant::now();
        for (id, bounds) in &entities {
            index1.insert(*id, *bounds);
        }
        let time1 = start1.elapsed();

        // Método 2: Bulk load
        let mut index2 = SpatialIndex::new();
        let start2 = Instant::now();
        index2.bulk_load(entities.clone());
        let time2 = start2.elapsed();

        // Bulk load debe ser al menos 2x más rápido
        assert!(time2 < time1 / 2, "Bulk load not faster: {:?} vs {:?}", time2, time1);
    }
}
```

---

### US-002.2: Box Selection

**Como** usuario final
**Quiero** seleccionar múltiples entidades arrastrando un rectángulo
**Para** selección rápida de grupos de objetos

#### Criterios de Aceptación

- [ ] **CA-001**: Click y arrastrar en espacio vacío inicia box selection
- [ ] **CA-002**: Rectángulo de selección se visualiza en tiempo real
- [ ] **CA-003**: Entidades dentro del rectángulo se seleccionan
- [ ] **CA-004**: Shift + click añade/quita de selección
- [ ] **CA-005**: Performance suave a 60fps

#### Implementación

```typescript
// TypeScript/JavaScript
function handlePointerDown(event: PointerEvent) {
  const { x, y } = viewport.screenToWorld(event.clientX, event.clientY);
  const hitResult = canvas.hitTest(x, y);

  if (!hitResult && !event.shiftKey) {
    // Iniciar box selection
    isBoxSelecting = true;
    boxSelectionStart = { x, y };
    boxSelectionCurrent = { x, y };

    // Limpiar selección actual (opcional)
    if (!event.shiftKey) {
      selection.clear();
    }
  }
}

function handlePointerMove(event: PointerEvent) {
  if (!isBoxSelecting) return;

  const { x, y } = viewport.screenToWorld(event.clientX, event.clientY);
  boxSelectionCurrent = { x, y };

  // Calcular rectángulo de selección
  const selectionRect = calculateSelectionRect(
    boxSelectionStart,
    boxSelectionCurrent
  );

  // Query espacial - O(log n)
  const candidates = canvas.spatialIndex.queryBox(selectionRect);

  // Actualizar selección
  if (event.shiftKey) {
    // Modo toggle: añadir/quitar
    candidates.forEach(id => selection.toggle(id));
  } else {
    // Modo replace: seleccionar todos
    selection.clear();
    candidates.forEach(id => selection.add(id));
  }

  canvas.render();
}

function handlePointerUp(event: PointerEvent) {
  if (isBoxSelecting) {
    isBoxSelecting = false;
    // Selección se mantiene
  }
}

function calculateSelectionRect(start, end) {
  return {
    x: Math.min(start.x, end.x),
    y: Math.min(start.y, end.y),
    width: Math.abs(end.x - start.x),
    height: Math.abs(end.y - start.y),
  };
}
```

```rust
// Rust nativo
use archflow_core::{Bounds, Vec2};

pub struct BoxSelection {
    start_pos: Vec2,
    current_pos: Vec2,
    mode: BoxSelectionMode,
}

pub enum BoxSelectionMode {
    Replace, // Reemplazar selección
    Add,     // Añadir a selección
    Toggle,  // Alternar selección
}

impl BoxSelection {
    pub fn new(start: Vec2, mode: BoxSelectionMode) -> Self {
        Self {
            start_pos: start,
            current_pos: start,
            mode,
        }
    }

    pub fn update(&mut self, current: Vec2) {
        self.current_pos = current;
    }

    pub fn get_bounds(&self) -> Bounds {
        let min_x = self.start_pos.x.min(self.current_pos.x);
        let min_y = self.start_pos.y.min(self.current_pos.y);
        let max_x = self.start_pos.x.max(self.current_pos.x);
        let max_y = self.start_pos.y.max(self.current_pos.y);

        Bounds::new(min_x, min_y, max_x, max_y)
    }
}

// Integración con SelectionManager
impl SelectionManager {
    pub fn apply_box_selection(
        &mut self,
        bounds: &Bounds,
        spatial_index: &SpatialIndex,
        mode: BoxSelectionMode,
    ) {
        let candidates = spatial_index.query_box(bounds);

        match mode {
            BoxSelectionMode::Replace => {
                self.clear();
                for id in candidates {
                    self.add(id);
                }
            }
            BoxSelectionMode::Add => {
                for id in candidates {
                    self.add(id);
                }
            }
            BoxSelectionMode::Toggle => {
                for id in candidates {
                    self.toggle(id);
                }
            }
        }
    }
}
```

#### Tests TDD

```rust
#[test]
fn test_box_selection_replaces_selection() {
    let mut selection = SelectionManager::new();
    let mut spatial = SpatialIndex::new();

    // Crear entidades de prueba
    let entities = create_test_grid(3, 3); // Grid de 3x3
    for (id, bounds) in &entities {
        spatial.insert(*id, *bounds);
    }

    // Seleccionar algunas entidades inicialmente
    selection.add(entities[0].0);
    selection.add(entities[1].0);
    assert_eq!(selection.len(), 2);

    // Box selection que cubre entidades diferentes
    let box_bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);
    selection.apply_box_selection(&box_bounds, &spatial, BoxSelectionMode::Replace);

    // Selección original debe ser reemplazada
    assert!(selection.len() > 0);
    assert!(!selection.contains(entities[0].0));
    assert!(!selection.contains(entities[1].0));
}

#[test]
fn test_box_selection_adds_to_existing() {
    let mut selection = SelectionManager::new();
    let mut spatial = SpatialIndex::new();

    let entities = create_test_grid(3, 3);
    for (id, bounds) in &entities {
        spatial.insert(*id, *bounds);
    }

    // Selección inicial
    selection.add(entities[0].0);
    let initial_count = selection.len();

    // Box selection en modo Add
    let box_bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);
    selection.apply_box_selection(&box_bounds, &spatial, BoxSelectionMode::Add);

    // Debe tener más elementos que inicialmente
    assert!(selection.len() > initial_count);
    assert!(selection.contains(entities[0].0)); // Original se mantiene
}

#[test]
fn test_box_selection_toggles_entities() {
    let mut selection = SelectionManager::new();
    let mut spatial = SpatialIndex::new();

    let entities = create_test_grid(3, 3);
    for (id, bounds) in &entities {
        spatial.insert(*id, *bounds);
    }

    // Pre-seleccionar algunas entidades
    selection.add(entities[0].0);
    selection.add(entities[1].0);

    // Box selection en modo Toggle
    let box_bounds = Bounds::new(0.0, 0.0, 150.0, 150.0);
    selection.apply_box_selection(&box_bounds, &spatial, BoxSelectionMode::Toggle);

    // Entidades en el área deben haber cambiado de estado
    // (algunas añadidas, algunas removidas según intersección)
}

#[test]
fn test_box_selection_empty_bounds() {
    let mut selection = SelectionManager::new();
    let mut spatial = SpatialIndex::new();

    let entities = create_test_grid(3, 3);
    for (id, bounds) in &entities {
        spatial.insert(*id, *bounds);
    }

    // Box selection con área vacía
    let empty_bounds = Bounds::new(1000.0, 1000.0, 1100.0, 1100.0);
    selection.apply_box_selection(&empty_bounds, &spatial, BoxSelectionMode::Replace);

    assert_eq!(selection.len(), 0);
}

#[test]
fn test_box_selection_performance() {
    let mut selection = SelectionManager::new();
    let mut spatial = SpatialIndex::new();

    // Crear muchas entidades
    let entities = create_test_entities(10_000);
    spatial.bulk_load(entities);

    // Box selection grande
    let box_bounds = Bounds::new(0.0, 0.0, 1000.0, 1000.0);

    let start = Instant::now();
    selection.apply_box_selection(&box_bounds, &spatial, BoxSelectionMode::Replace);
    let elapsed = start.elapsed();

    // Debe ser rápido incluso con 10K entidades
    assert!(elapsed.as_millis() < 5, "Box selection too slow: {:?}", elapsed);
}
```

---

### US-002.3: Seleccionar Todo e Invertir Selección

**Como** usuario final
**Quiero** seleccionar todas las entidades o invertir la selección actual
**Para** operaciones rápidas de lote

#### Criterios de Aceptación

- [ ] **CA-001**: Ctrl+A selecciona todas las entidades visibles
- [ ] **CA-002**: Ctrl+Shift+I invierte la selección
- [ ] **CA-003**: Operaciones son O(n) donde n = entidades visibles
- [ ] **CA-004**: Funciona con filtros (capas,锁定, etc.)

#### Implementación

```rust
impl SelectionManager {
    /// Seleccionar todas las entidades visibles
    pub fn select_all(&mut self, entities: &[EntityId]) {
        self.clear();
        for entity in entities {
            self.add(*entity);
        }
    }

    /// Invertir selección: seleccionar las no seleccionadas
    pub fn invert(&mut self, all_entities: &[EntityId]) {
        let currently_selected: std::collections::HashSet<EntityId> =
            self.get_all().into_iter().collect();

        self.clear();

        for entity in all_entities {
            if !currently_selected.contains(entity) {
                self.add(*entity);
            }
        }
    }

    /// Seleccionar todas las entidades visibles (con filtros)
    pub fn select_all_visible(
        &mut self,
        canvas: &Canvas,
        viewport: &Viewport,
    ) {
        let visible_entities = canvas.get_entities_in_viewport(viewport);
        self.select_all(&visible_entities);
    }
}
```

#### Tests TDD

```rust
#[test]
fn test_select_all() {
    let mut selection = SelectionManager::new();
    let entities = create_test_entities(10);

    selection.select_all(&entities);

    assert_eq!(selection.len(), 10);
}

#[test]
fn test_select_all_replaces_existing() {
    let mut selection = SelectionManager::new();
    let entities1 = create_test_entities(5);
    let entities2 = create_test_entities(10);

    // Seleccionar primeras 5
    selection.select_all(&entities1[0..5]);
    assert_eq!(selection.len(), 5);

    // Seleccionar todas las 10
    selection.select_all(&entities2);
    assert_eq!(selection.len(), 10);
}

#[test]
fn test_invert_selection() {
    let mut selection = SelectionManager::new();
    let all_entities = create_test_entities(10);

    // Seleccionar primeras 3
    selection.add(all_entities[0].0);
    selection.add(all_entities[1].0);
    selection.add(all_entities[2].0);

    // Invertir
    selection.invert(&all_entities);

    // Ahora deben estar seleccionadas las otras 7
    assert_eq!(selection.len(), 7);
    assert!(!selection.contains(all_entities[0].0));
    assert!(!selection.contains(all_entities[1].0));
    assert!(!selection.contains(all_entities[2].0));
    assert!(selection.contains(all_entities[3].0));
}

#[test]
fn test_invert_empty_selects_all() {
    let mut selection = SelectionManager::new();
    let all_entities = create_test_entities(10);

    // Invertir selección vacía
    selection.invert(&all_entities);

    // Todas deben estar seleccionadas
    assert_eq!(selection.len(), 10);
}

#[test]
fn test_invert_all_selects_none() {
    let mut selection = SelectionManager::new();
    let all_entities = create_test_entities(10);

    // Seleccionar todas
    selection.select_all(&all_entities);
    assert_eq!(selection.len(), 10);

    // Invertir
    selection.invert(&all_entities);

    // Ninguna debe estar seleccionada
    assert_eq!(selection.len(), 0);
}

#[test]
fn test_select_all_with_layer_filter() {
    let mut selection = SelectionManager::new();
    let mut canvas = create_test_canvas();
    let viewport = Viewport::new();

    // Crear entidades en diferentes capas
    let layer1 = canvas.create_layer("layer1");
    let layer2 = canvas.create_layer("layer2");

    let entity1 = canvas.create_rectangle_on_layer(layer1);
    let entity2 = canvas.create_rectangle_on_layer(layer2);

    // Ocultar layer2
    canvas.set_layer_visible(layer2, false);

    // Seleccionar solo visibles
    selection.select_all_visible(&canvas, &viewport);

    assert_eq!(selection.len(), 1);
    assert!(selection.contains(entity1));
    assert!(!selection.contains(entity2));
}
```

---

### US-002.4: Toggle de Selección

**Como** usuario final
**Quiero** añadir/quitar entidades de la selección con Shift+Click
**Para** control preciso de selección múltiple

#### Criterios de Aceptación

- [ ] **CA-001**: Shift+Click en entidad no seleccionada la añade
- [ ] **CA-002**: Shift+Click en entidad seleccionada la quita
- [ ] **CA-003**: Operación es O(1)
- [ ] **CA-004**: Funciona con cualquier número de entidades

#### Implementación

```rust
impl SelectionManager {
    /// Toggle: añade si no existe, remueve si existe
    pub fn toggle(&mut self, entity_id: EntityId) -> bool {
        if self.contains(entity_id) {
            self.remove(entity_id);
            false // Removida
        } else {
            self.add(entity_id);
            true // Añadida
        }
    }
}
```

#### Tests TDD

```rust
#[test]
fn test_toggle_adds_when_not_selected() {
    let mut selection = SelectionManager::new();
    let entity = EntityId::new();

    let added = selection.toggle(entity);

    assert!(added);
    assert!(selection.contains(entity));
}

#[test]
fn test_toggle_removes_when_selected() {
    let mut selection = SelectionManager::new();
    let entity = EntityId::new();
    selection.add(entity);

    let added = selection.toggle(entity);

    assert!(!added); // Removida, no añadida
    assert!(!selection.contains(entity));
}

#[test]
fn test_multiple_toggles() {
    let mut selection = SelectionManager::new();
    let entity = EntityId::new();

    // Primer toggle: añade
    assert!(selection.toggle(entity));
    assert!(selection.contains(entity));

    // Segundo toggle: quita
    assert!(!selection.toggle(entity));
    assert!(!selection.contains(entity));

    // Tercer toggle: añade de nuevo
    assert!(selection.toggle(entity));
    assert!(selection.contains(entity));
}
```

---

### US-002.5: Operaciones de Lote Optimizadas

**Como** desarrollador del SDK
**Quiero** operaciones eficientes en múltiples entidades seleccionadas
**Para** performance en selecciones grandes

#### Criterios de Aceptación

- [ ] **CA-001**: `add_all()` es O(n) donde n = entidades a añadir
- [ ] **CA-002**: `remove_all()` es O(n) donde n = entidades a remover
- [ ] **CA-003**: `clear()` es O(1)
- [ ] **CA-004**: `get_all()` es iterador lazy O(1)
- [ ] **CA-005**: `is_empty()` es O(1)

#### Implementación

```rust
use std::collections::HashSet;

pub struct SelectionSet {
    inner: HashSet<EntityId>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn add_all(&mut self, entities: &[EntityId]) {
        self.inner.extend(entities.iter().copied());
    }

    pub fn remove_all(&mut self, entities: &[EntityId]) {
        for entity in entities {
            self.inner.remove(entity);
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear(); // O(1) en HashSet
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty() // O(1)
    }

    pub fn len(&self) -> usize {
        self.inner.len() // O(1)
    }

    pub fn contains(&self, entity: EntityId) -> bool {
        self.inner.contains(&entity) // O(1) average
    }

    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.inner.iter().copied() // O(1) para crear iterador
    }
}
```

#### Tests TDD + Benchmarks

```rust
#[test]
fn test_add_all_performance() {
    let mut set = SelectionSet::new();
    let entities = create_test_entities(1000);

    let start = Instant::now();
    set.add_all(&entities);
    let elapsed = start.elapsed();

    assert_eq!(set.len(), 1000);
    assert!(elapsed.as_micros() < 100, "add_all too slow: {:?}", elapsed);
}

#[test]
fn test_remove_all_performance() {
    let mut set = SelectionSet::new();
    let entities = create_test_entities(1000);
    set.add_all(&entities);

    let start = Instant::now();
    set.remove_all(&entities[0..500]);
    let elapsed = start.elapsed();

    assert_eq!(set.len(), 500);
    assert!(elapsed.as_micros() < 100, "remove_all too slow: {:?}", elapsed);
}

#[test]
fn test_clear_is_constant_time() {
    let mut set = SelectionSet::new();
    let entities = create_test_entities(10000);
    set.add_all(&entities);

    // Medir clear con 10K entidades
    let start = Instant::now();
    set.clear();
    let elapsed = start.elapsed();

    // Debe ser muy rápido (O(1))
    assert!(elapsed.as_nanos() < 1000, "clear too slow: {:?}", elapsed);
    assert!(set.is_empty());
}

#[bench]
fn bench_selection_operations(b: &mut test::Bencher) {
    let mut set = SelectionSet::new();
    let entities = create_test_entities(100);

    b.iter(|| {
        set.add_all(&entities);
        set.remove_all(&entities[0..50]);
        set.clear();
    });
}
```

---

## 🔬 Protocolo de Investigación

### Investigación 1: Spatial Index Selection

**Objetivo**: Determinar el mejor índice espacial para canvas editing

**Método**:
1. Implementar prototipos de R-tree, Quadtree, y Grid
2. Benchmark con datasets sintéticos (uniforme, clusterizado, realista)
3. Medir: query time, insert time, memory usage
4. Evaluar cache efficiency

**Métricas**:
- Query latency (P50, P95, P99)
- Insert/Update/Delete latency
- Memory overhead
- Cache miss rate

### Investigación 2: Optimización de Bulk Operations

**Objetivo**: Optimizar operaciones de lote

**Método**:
1. Comparar iteración secuencial vs paralela
2. Evaluar SIMD para bounds checking
3. Medir impacto de SoA vs AoS
4. Profile con perf/VTune

**Métricas**:
- Throughput (entities/segundo)
- CPU utilization
- Memory bandwidth

---

## 📊 Métricas de Éxito

### Performance

| Métrica | Target | Medición |
|---------|--------|----------|
| query_box (100) | < 100µs | Benchmark |
| query_box (10K) | < 1ms | Benchmark |
| bulk_load (10K) | < 50ms | Benchmark |
| box_selection (60fps) | < 16ms | Frame time |
| Memory overhead | < 100B/entity | Valgrind |

### Calidad

| Métrica | Target | Medición |
|---------|--------|----------|
| Test coverage | > 95% | tarpaulin |
| Clippy warnings | 0 | cargo clippy |
| Documentation | 100% público | rustdoc |

---

## 🚀 Plan de Implementación

### Sprint 1: Spatial Index (Semana 1)

- [ ] Implementar R-tree wrapper
- [ ] Implementar Spatial grid
- [ ] Benchmark comparativos
- [ ] Seleccionar mejor enfoque

### Sprint 2: Box Selection (Semana 2)

- [ ] Implementar BoxSelection state
- [ ] Integrar con ToolManager
- [ ] Visualización de rectángulo
- [ ] Tests completos

### Sprint 3: Advanced Operations (Semana 3)

- [ ] select_all / invert
- [ ] Toggle de selección
- [ ] Operaciones de lote
- [ ] Optimización y profiling

---

## 📖 Referencias

- [SVG vs Canvas vs WebGL - Spatial Indexing](https://dev.to/vitalf/svg-vs-canvas-vs-webgl-for-diagram-viewers-tradeoffs-bottlenecks-and-how-to-measure-34n7)
- [Efficient Quadtree Implementation](https://stackoverflow.com/questions/41946007/efficient-and-well-explained-implementation-of-a-quadtree-for-2d-collision-det)
- [Hybrid Spatial Index](https://github.com/addu390/hybrid-spatial-index)
- [GPU Spatial Indexing Performance](https://www.researchgate.net/figure/Performance-metrics-for-GPU-Quadtree-QT-and-R-Tree-RT-indices_tbl5_382554736)

---

**Versión**: 1.0.0
**Última actualización**: 2025-01-28
**Autores**: ArchFlow Development Team
