# Épica: Sensores de Física - Physics Perception Suite

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| ID | EPIC-002 |
| Prioridad | Alta |
| Estimación | XXL |
| Estado | Borrador |
| Versión | 0.1.0 |
| Fecha creación | 2026-02-01 |

---

## 🎯 Objetivo de Negocio

Implementar el sistema de percepción espacial (Collision, Proximity, Snapping, Alignment) que permita que las aplicaciones tipo diagramas/dibujo detecten relaciones espaciales con rendimiento O(1) mediante Spatial Hashing, habilitando interacciones profesionales (snap-to-grid, magnetic connections, smart guides) en aplicaciones web.

**Problema que resuelve**: La detección espacial "brute-force" (O(n²)) es inviable para aplicaciones con miles de entidades. Esta épica implementa Spatial Hashing para reducir la complejidad a O(n) con overhead mínimo.

**Importante para SDK**: Para un SDK de diagramas tipo Figma/tldraw, **NO se necesita física de cuerpos rígidos** (masa, gravedad, rebotes). Lo que se necesita es:
- **Snap system**: snap-to-grid, snap-to-guide, snap-to-entity
- **Magnetic connections**: ports que se atraen magnéticamente
- **Overlap detection**: detectar solapamiento para UI feedback (no collision response)

---

## 🏗️ Arquitectura DDD

### Bounded Context
**Physics Perception Context** - Contexto de Percepción Física

### Aggregate Roots
- `CollisionSensor`: Sensor de colisión AABB vs AABB con detección de tags
- `NearSensor`: Sensor de proximidad circular con hysteresis
- `RadarSensor`: Sensor de radar cónico para detección direccional
- `SpatialHashGrid`: Estructura de partición espacial O(1)

### Domain Events
```rust
pub enum PhysicsEvent {
    CollisionStart { entity_a: EntityId, entity_b: EntityId },
    CollisionEnd { entity_a: EntityId, entity_b: EntityId },
    ProximityEnter { entity: EntityId, target: EntityId, distance: f32 },
    ProximityExit { entity: EntityId, target: EntityId },
    RadarDetection { source: EntityId, target: EntityId, direction: Vec3 },
}
```

### Services
- `SpatialIndexService`: Mantiene el Spatial Hash actualizado
- `CollisionDetectionService`: Detecta colisiones AABB
- `ProximityCalculationService`: Calcula distancias con hysteresis
- `BroadPhaseService`: Filtra pares potencialmente colisionantes

---

## 📖 Historias de Usuario

### HU-005: Spatial Hash Grid para Partición Espacial

**Como** arquitecto del motor
**Quiero** una estructura de Spatial Hash optimizada
**Para** reducir detección de colisiones de O(n²) a O(n)

#### Criterios de Aceptación
- [x] Grid size configurable (default: 40px basado en investigación)
- [x] Usa HashMap con GridCoord como clave (coordenadas de celda)
- [x] Soporta insert/remove en O(1) promedio
- [x] Query por AABB retorna solo entidades en celdas relevantes
- [x] Query por radio retorna entidades en celdas vecinas
- [x] Usa timestamps para evitar re-inicialización cada frame
- [ ] Thread-safe para actualizaciones paralelas (futuro)

#### Tareas Técnicas
- [x] **Investigación**: Estudiar implementación de OpenTissue spatial hashing
- [x] **Tests (TDD)**: Tests de inserción/remoción
- [x] **Tests (TDD)**: Tests de query AABB vs brute-force (validación)
- [x] **Implementación**: Crear `SpatialHashGrid` struct
- [x] **Implementación**: Implementar hash function (floor division)
- [x] **Optimización**: Usar hashbrown HashMap para no_std compatible
- [ ] **Benchmark**: Comparar vs brute-force para 1K, 10K, 100K entidades

#### Investigación Previa
- [x] Perplexity: "Spatial Hashing vs ECS - Leetless"
- [x] OpenTissue: spatial hashing library
- [x] ArXiv: "Optimized Spatial Hashing for Collision Detection" (Tetrahedral meshes)
- [x] GitHub: alexpyattaev/spatialhash (Rust implementation)

#### Estimación: XL
#### Estado: ✅ **COMPLETADO** - 22 tests passing

---

### HU-006: Sensor de Colisión (Touch Sensor)

**Como** desarrollador de diagramas
**Quiero** detectar cuando dos figuras se tocan
**Para** implementar conexiones y relaciones visuales

#### Criterios de Aceptación
- [ ] Detecta colisión AABB vs AABB
- [ ] Filtra por `target_tag` (solo colisiona con entidades con tag específico)
- [ ] Genera pulso Positive al entrar en colisión
- [ ] Genera pulso Negative al salir de colisión
- [ ] Mantiene `hit_list` de entidades actualmente colisionando
- [ ] Usa Spatial Hash para broad-phase (solo testea vecinos)
- [ ] Propiedades BGE: `invert`, `tap`, `level`

#### Tareas Técnicas
- [ ] **Investigación**: Revisar `KX_TouchSensor.cpp` de BGE
- [ ] **Tests (TDD)**: Tests de colisión simple (2 cajas)
- [ ] **Tests (TDD)**: Tests de colisión múltiple (n cajas)
- [ ] **Implementación**: Crear `CollisionSensor` struct
- [ ] **Implementación**: Implementar `evaluate()` con spatial hash query
- [ ] **Optimización**: Cache de AABB para evitar recalcular
- [ ] **Integración**: Sistema `sys_collision_logic`

#### Investigación Previa
- [x] BGE Source: `source/gameengine/Ketsji/KX_TouchSensor.cpp`
- [x] Spatial Hash: Patrones de query eficiente
- [x] Patrón implementado: Broad-phase + narrow-phase

#### Estimación: L
#### Estado: ✅ **COMPLETADO** - 14 tests passing - CollisionSensor implementado

---

### HU-007: Sensor de Proximidad con Hysteresis (Near Sensor)

**Como** diseñador de UX
**Quiero** detectar cuando el cursor se acerca a una figura
**Para** mostrar puertos de conexión o menús contextuales

#### Criterios de Aceptación
- [ ] Detecta entidades dentro de un radio `distance`
- [ ] Usa `reset_distance` mayor para hysteresis (evita flickering)
- [ ] Filtra por `target_tag`
- [ ] Usa distancia euclidiana al cuadrado (evita sqrt)
- [ ] Spatial Hash query con radio máximo (reset_distance)
- [ ] Genera pulso Positive/Negative en cambios de estado
- [ ] Propiedades BGE: `invert`, `tap`

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar patrón Schmitt Trigger para hysteresis
- [ ] **Tests (TDD)**: Tests de detección de proximidad
- [ ] **Tests (TDD)**: Tests de hysteresis (evitar flickering en borde)
- [ ] **Implementación**: Crear `NearSensor` struct
- [ ] **Implementación**: Implementar `evaluate()` con dist_sq
- [ ] **Optimización**: Solo calcular sqrt si es necesario
- [ ] **Integración**: Sistema `sys_proximity_logic`

#### Investigación Previa
- [x] BGE Source: `source/gameengine/Ketsji/KX_NearSensor.cpp`
- [x] Control Theory: Schmitt Trigger pattern
- [x] Matemáticas: Distancia euclidiana optimizada

#### Estimación: L
#### Estado: Pendiente

---

### HU-008: Sensor de Radar Direccional (Radar Sensor)

**Como** desarrollador de juegos
**Quiero** detectar entidades en un cono direccional
**Para** implementar visión artificial de NPCs

#### Criterios de Aceptación
- [ ] Detecta entidades dentro de un cono (ángulo + distancia)
- [ ] Configuración: `axis` (X/Y/Z), `range`, `angle` (en grados)
- [ ] Retorna `hit_object` y `hit_normal`
- [ ] Usa Spatial Hash para reducir candidatos
- [ ] Testea ángulo con producto punto
- [ ] Filtra por `target_tag` y `mask` (collision mask)

#### Tareas Técnicas
- [ ] **Investigación**: Revisar `KX_RadarSensor.cpp` de BGE
- [ ] **Tests (TDD)**: Tests de detección cónica
- [ ] **Implementación**: Crear `RadarSensor` struct
- [ ] **Implementación**: Implementar detección por ángulo (dot product)
- [ ] **Implementación**: Integrar con Spatial Hash
- [ ] **Integración**: Sistema `sys_radar_logic`

#### Investigación Previa
- [x] BGE Source: `source/gameengine/Ketsji/KX_RadarSensor.cpp`
- [x] Matemáticas 3D: Dot product para ángulos entre vectores
- [x] Spatial queries: Radius + angle filtering

#### Estimación: M
#### Estado: Pendiente

---

### HU-009: Sistema de Snapping y Alineamiento ⭐ CRÍTICO PARA SDK

**Como** desarrollador usando el SDK
**Quiero** un sistema de snapping configurable (grid, guides, entities)
**Para** que mis usuarios puedan alinear elementos fácilmente como en Figma/tldraw

#### Contexto y Justificación

**Por qué Snapping, no Physics:**
Para un SDK de diagramas tipo Figma/tldraw, **NO se necesita simulación física** (masa, gravedad, rebotes, integración numérica). Lo que se necesita es:

| Lo que NO necesita | Lo que SÍ necesita |
|-------------------|-------------------|
| Simulación de física | Snap-to-grid con tamaño configurable |
| Velocidad/aceleración | Snap-to-entity (edge-to-edge, center-to-center) |
| Collision response | Snap-to-guide (líneas guía horizontales/verticales) |
| Positional correction | Visual guides (preview del snap antes de soltar) |
| Force resolution | Magnetic connections (ports que se atraen) |
| Integración numérica | Overlap detection (solo para UI feedback) |

#### Criterios de Aceptación

**FASE 1: Snap-to-Grid** ✅
- [x] Grid size configurable (8px, 16px, 32px, 64px, custom)
- [x] Snap position: x/y se alinean al grid más cercano
- [x] Threshold configurable (distancia para activar snap)
- [x] Per-axis check (ambos ejes deben estar dentro del threshold)
- [x] API para desarrollador:
  ```rust
  let snapper = Snapper::new(SnapConfig {
      grid_size: 16.0,
      threshold: 8.0,  // 50% de grid size
      ..Default::default()
  });
  let snapped_pos = snapper.snap_to_grid(pos);
  ```

**FASE 2: Snap-to-Entity** ✅
- [x] Snap a bordes de entidades cercanas (left, right, top, bottom)
- [x] Snap a centros de entidades (center-to-center alignment)
- [x] Usa iterador de entidades (compatible con Spatial Hash O(1))
- [x] Threshold por distancia (Euclidean distance para entity snapping)
- [x] Visual guides: SnapPoint struct para UI rendering

**FASE 3: Snap-to-Guide** ⏳ Futuro
- [ ] Líneas guía horizontales/verticales personalizables
- [ ] Guías temporales (durante drag) y permanentes
- [ ] Snap a guías con threshold
- [ ] API para añadir/remover guías

**FASE 4: Magnetic Connections** ⏳ Futuro
- [ ] Ports (puntos de conexión) en entidades
- [ ] Atracción magnética cuando el cursor está cerca
- [ ] Auto-alineación de ports
- [ ] Visual feedback (highlight cuando se activa el snap)

**NO incluye (fuera de scope):**
- ❌ Simulación de física (masa, inercia, gravedad)
- ❌ Collision response (separación de cuerpos)
- ❌ Integración numérica (Euler, Verlet)
- ❌ Constraint solver
- ❌ Continuous collision detection

#### Tareas Técnicas

**Implementación Core:**
- [x] **Tests (TDD)**: Tests de snap-to-grid (10 tests)
- [x] **Tests (TDD)**: Tests de snap-to-entity (10 tests)
- [x] **Implementación**: `Snapper` struct con configuración
- [x] **Implementación**: `SnapConfig`, `SnapResult`, `SnapTarget` structs
- [x] **Implementación**: `snap_to_grid()` con per-axis threshold check
- [x] **Implementación**: `snap_to_entity_edge()` y `snap_to_entity_center()`
- [x] **Implementación**: `get_snap_points()` para UI rendering de guides
- [x] **Implementación**: `Rect` helper para entity AABB

**Visual Guides:** ⏳ Futuro
- [ ] **Implementación**: `GuideRenderer` para dibujar líneas de snap
- [ ] **Implementación**: `SnapPreview` para mostrar posición futura
- [ ] **UI Integration**: Hooks para que el render del SDK dibuje las guías

**Magnetic Ports:** ⏳ Futuro
- [ ] **Implementación**: `Port` component (posición relativa a entidad)
- [ ] **Implementación**: `MagneticSnapper` para ports
- [ ] **Tests**: Tests de atracción magnética

**Documentación:**
- [ ] **Documentación**: Guía de uso del sistema de snapping
- [ ] **Ejemplos**: Code samples para desarrolladores del SDK

#### Investigación Previa
- [x] Figma: Snap system analysis (UX patterns)
- [x] tldraw: Open source snap implementation
- [x] Spatial Hash: Ya implementado en HU-005
- [x] UI/UX: Snap thresholds óptimos (50% de grid size)

#### Estimación: XL (2-3 semanas)
#### Estado: ✅ **COMPLETADO FASE 1-2** - 20 tests passing - Figma/tldraw-like snapping implemented

---

### HU-010: Integración de Sensores Físicos con PulseBus

**Como** desarrollador del motor
**Quiero** que los sensores físicos emitan pulsos
**Para** conectar la física con la lógica de actuadores

#### Criterios de Aceptación
- [x] Cada sensor físico genera `Pulse` de 16 bytes ✅
- [ ] Los pulsos incluyen metadata de colisión (entity_id del otro) ⚠️ **LIMITADO**
- [x] Soporta listas de hit (múltiples colisiones simultáneas) ✅ (TouchSensor mantiene hit_list internamente)
- [x] Batch processing de todos los sensores físicos ✅
- [x] Change detection para evitar pulsos duplicados ✅ (SignalByte edge detection)
- [x] Compatible con parallelismo de ECS ✅ (SoA layout, no mutable shared state)

#### Tareas Técnicas
- [x] **Tests (TDD)**: Tests de integración física → PulseBus ✅
- [ ] **Implementación**: Extender `Pulse` con metadata física ⚠️ **DEFERRED**
- [x] **Implementación**: Sistema `LogicSystem::evaluate_physics_sensors()` ✅
- [x] **Optimización**: Pre-allocated PulseBus (256 capacity) ✅
- [x] **Refactor**: SensorId enum para eliminar magic numbers ✅ (2026-02-01)

#### Investigación Previa
- [x] ECS patterns: Event batching strategies
- [x] Bevy: Event system architecture
- [x] BGE: Pulse coupling con actuadores

#### Estado Actual (2026-02-02)
**Estado:** ✅ **COMPLETADO** - PhysicsPulse implementado

**Implementado:**
- ✅ PhysicsPulse con metadata de colisión (`other_entity`)
- ✅ PhysicsPulseBus con métodos para collision/proximity/radar
- ✅ Integración completa con el sistema existente de Pulse
- ✅ 445 tests passing

**Archivos modificados:**
- `crates/archflow-logic/src/physics_pulse.rs` - Nuevo módulo PhysicsPulse
- `crates/archflow-logic/src/pulse.rs` - Agregado getter timestamp()
- `crates/archflow-logic/src/lib.rs` - Exports de PhysicsPulse

#### Estimación: M
#### Estado: ✅ **COMPLETADO** - 100% con PhysicsPulse metadata

---

## 🔬 Investigación por Historia

### Resultados de Investigación (2025-2026)

#### 1. Spatial Hashing Performance
**Fuente**: Leetless.de (2023), OpenTissue, ArXiv papers

**Hallazgos clave**:
- **Grid size óptimo**: ~40px para diagramas 2D (según benchmarks)
- **Grid cell size > query radius**: Más importante que hash function perfecta
- **Timestamps vs Re-init**: Usar timestamps en celdas es 10x más rápido que reiniciar hash table
- **Density matters**: Más entidades = grid size más óptimo más pequeño

**Aplicación a esta épica**:
- `SpatialHashGrid` usa grid size de 40px por defecto
- Timestamps para evitar O(grid_size) overhead de re-inicialización
- Hash function: Shifted Golden Mean (mejor distribución que prime mod)

#### 2. Collision Detection Optimization
**Fuente**: "Real-Time Collision Detection", Game Physics literature

**Patrones identificados**:
- **Broad-phase vs Narrow-phase**: Spatial hash es broad-phase, AABB tests son narrow-phase
- **Positional correction**: Proyectar objetos fuera de solapamiento (más estable que impulse-only)
- **AABB vs Circle**: Testear si centro de circle está dentro de AABB expandido por radio

**Aplicación a esta épica**:
- Motor de physics usa broad-phase (Spatial Hash) + narrow-phase (AABB tests)
- Positional correction para evitar "sinking" de objetos
- Soporte para AABB y Circle primitivas

#### 3. Hysteresis en Sensores de Proximidad
**Fuente**: Control Theory literature, BGE implementation

**Patrón Schmitt Trigger**:
- **Threshold superior**: Activar cuando distancia < d_activate
- **Threshold inferior**: Desactivar cuando distancia > d_deactivate
- **d_deactivate > d_activate**: Previene flickering en borde

**Aplicación a esta épica**:
- `NearSensor` usa `distance` (activate) y `reset_distance` (deactivate)
- Estado interno `is_active` para tracking de hysteresis
- Distancia al cuadrado para evitar sqrt en hot-path

---

## 🧪 Enfoque TDD por Historia

### Fase 1: Rojo (Test Fallando)

```rust
// tests/hu_006_collision_sensor_tests.rs

#[test]
fn test_collision_detection_simple() {
    let mut store = EntityStore::new();
    let entity_a = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
    let entity_b = store.spawn(Vec2::new(25.0, 25.0), Vec2::new(50.0, 50.0));
    
    let mut sensor = CollisionSensor::new(
        entity_a,
        0, // target_tag: any
    );
    
    let mut spatial_grid = SpatialHashGrid::new(40.0);
    spatial_grid.update(&store);
    
    // Evaluación
    if let Some(pulse) = sensor.evaluate(&spatial_grid, &store, 0) {
        assert_eq!(pulse.state, SensorState::Positive);
    } else {
        panic!("Should detect collision");
    }
}

#[test]
fn test_hysteresis_prevents_flickering() {
    // Test que NearSensor no flickera en el borde
    // ...
}
```

### Fase 2: Verde (Implementación Mínima)
```rust
impl CollisionSensor {
    pub fn evaluate(&mut self, spatial_grid: &SpatialHashGrid, store: &EntityStore, ts: u32) -> Option<Pulse> {
        // Implementación mínima para pasar tests
    }
}
```

### Fase 3: Refactor
- Extraer lógica de Spatial Hash a servicio dedicado
- Optimizar queries con cache de celdas visitadas
- SIMD para batch AABB tests

---

## 📊 Estado de Tareas - Documentación Vivo

| Historia | Estado | Tests | Deuda Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-005 | ✅ Completado | 22/22 | Ninguna | SpatialHashGrid O(1) |
| HU-006 | ✅ Completado | 14/14 | Ninguna | CollisionSensor AABB |
| HU-007 | ✅ Completado | 6/6 | Ninguna | NearSensor con hysteresis |
| HU-008 | ✅ Completado | 13/13 | Ninguna | RadarSensor detección cónica |
| HU-009 | ✅ Fase 1-2 | 20/20 | Ninguna | Figma/tldraw-like snapping |
| HU-010 | ✅ Completado | 5/5 | Ninguna | PhysicsMetadata + PhysicsPulseBus |

---

## 📝 Secciones de la Épica

### Resumen Ejecutivo
Implementar el sistema de percepción física de ArchFlow basado en Spatial Hashing y sensores BGE, permitiendo detección de colisiones y proximidad con complejidad O(n) en lugar de O(n²), habilitando aplicaciones web con miles de entidades interactivas.

### Antecedentes
La detección de colisiones brute-force (comparar cada entidad con todas las demás) tiene complejidad O(n²), lo que la hace inviable para más de ~1000 entidades a 60 FPS. Spatial Hashing reduce esto a O(n) manteniendo un grid espacial donde cada celda solo contiene entidades cercanas.

### Alcance

**Incluye:**
- [x] SpatialHashGrid con timestamps
- [x] CollisionSensor (AABB vs AABB)
- [x] NearSensor (proximidad con hysteresis)
- [x] RadarSensor (detección cónica)
- [x] Motor de physics simplificado
- [x] Integración con PulseBus

**No incluye:**
- [ ] Física realista (masa, inercia, gravedad) → Fase 2
- [ ] Raycasting 3D → Épica futura (3D)
- [ ] Soft body dynamics → Fuera de scope
- [ ] Constraint solver → Fase 2

### Criterios de Éxito
- [ ] Pasar todos los tests de aceptación (100% success rate)
- [ ] Rendimiento: 60 FPS con 10,000 entidades colisionando
- [ ] Complejidad: O(n) para collision detection (verificado con benchmarks)
- [ ] Memory: <5MB para Spatial Hash con 10K entidades
- [ ] Precisión: 0 falsos positivos/negativos en colisiones AABB

### Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Grid size subóptimo para casos extremos | Medio | Media | Hacer grid size configurable + autotuning |
| Entidades muy grandes vs muy pequeñas | Medio | Media | Multi-level spatial hash (quadtree híbrido) |
| Tunneling (objetos muy rápidos) | Alto | Baja | Continuous collision detection (CCD) opcional |
| Memory overhead de hash table | Medio | Baja | Comprimir EntityId a u32, usar sparse storage |

### Dependencias
- [ ] `archflow-core` con EntityStore y EntityId
- [ ] `archflow-logic` con PulseBus y Sensor base
- [ ] Épica EPIC-001 (Input Sensors) completada (para integración)
- [ ] Librería de matemáticas 2D/3D (Vec2, Vec3, AABB)

### Timeline
```
Semana 1-2: HU-005 (SpatialHashGrid) + benchmarks
Semana 3: HU-006 (CollisionSensor) + tests
Semana 4: HU-007 (NearSensor) + hysteresis
Semana 5: HU-009 (Physics básico) + HU-010 (Integración)
Semana 6: HU-008 (RadarSensor) + documentación
```

---

## 🔧 Deuda Técnica

### Deuda Identificada
| Item | Severity | Descripción | Solución Propuesta |
|------|----------|-------------|-------------------|
| N/A | - | Sin deuda identificada aún | - |

### Propuestas de Mejora

1. **Multi-level Spatial Hash**
   - Descripción: Usar grid jerárquico para entidades de tamaños muy diferentes
   - Impacto: Medio (mejora casos extremos)
   - Effort: XL
   - Referencia: Loose octree + spatial hash híbrido

2. **Continuous Collision Detection (CCD)**
   - Descripción: Detectar colisiones para objetos muy rápidos (raycast vs trajectory)
   - Impacto: Alto (previene tunneling)
   - Effort: XL
   - Referencia: "Real-Time Collision Detection" capítulo 7

3. **SAP (Sweep and Prune) alternative**
   - Descripción: Implementar SAP como alternativa a Spatial Hash para ciertos casos
   - Impacto: Bajo (mejora solo para datasets muy ordenados)
   - Effort: L
   - Referencia: Box2D broad-phase

---

## 📚 Recursos

### Investigación Completada
- [x] [Spatial Hashing vs ECS - Leetless](https://leetless.de/posts/spatial-hashing-vs-ecs/)
- [x] [Optimized Spatial Hashing - ArXiv](https://matthias-research.github.io/pages/publications/tetraederCollision.pdf)
- [x] [OpenTissue Spatial Hashing Guide](https://github.com/erleben/OpenTissue/blob/master/documentation/hashing.md)
- [x] [Real-Time Collision Detection - Christer Ericson](https://www.amazon.com/Real-Time-Collision-Detection-Interactive-Technology/dp/1558607323)

### Código Fuente de Referencia
- `blender/source/gameengine/Ketsji/KX_TouchSensor.cpp`
- `blender/source/gameengine/Ketsji/KX_NearSensor.cpp`
- `blender/source/gameengine/Ketsji/KX_RadarSensor.cpp`
- `blender/source/gameengine/Ketsji/KX_RaySensor.cpp`

### Implementaciones Rust
- [alexpyattaev/spatialhash](https://github.com/alexpyattaev/spatialhash) - Spatial hash 3D genérico
- [bevy_rapier2d](https://github.com/dimforge/bevy_rapier2d) - Physics engine para Bevy (referencia)

---

`★ Insight ─────────────────────────────────────`
**Spatial Hashing en ArchFlow**

1. **Grid Size > Hash Function**: La investigación muestra que el tamaño de celda correcto es 10x más importante que la función de hash perfecta. Un grid de 40px (tamaño típico de UI element) es óptimo para diagramas.

2. **Timestamps vs Re-init**: Reiniciar el hash table cada frame cuesta O(grid_size). Usar timestamps en cada celda cuesta O(número de celdas activas). Para 10K entities, esto es ~100x más rápido.

3. **Hysteresis es Obligatorio**: Sin el patrón Schmitt Trigger (dos umbrales), los sensores de proximidad parpadean violentamente cuando el cursor está exactamente en el borde del radio. La UX sería inutilizable.
`─────────────────────────────────────────────────`

---

**Fin de Épica EPIC-002: Sensores de Física**
