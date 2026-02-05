# Épica: ECS Parallel Execution

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-ECS-003 |
| Prioridad | Media |
| Estimación | XXL (4-6 semanas) |
| Estado | Borrador |
| Versión | 0.1.0 |
| Dependencia | EPIC-ECS-002 |

## 🎯 Objetivo de Negocio

Implementar ejecución paralela de sistemas ECS utilizando rayon o crossbeam, aprovechando múltiples cores CPU para sistemas independientes.

## 🔗 Dependencias

- Depende de: **EPIC-ECS-002** (usa el scheduler para ordenamiento)
- Habilita: Optimización de rendimiento para escenarios con muchas entidades

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-engine` (Render Subsystem)
- **Aggregate Root**: `ParallelExecutor`
- **Domain Events**: `ParallelExecutionStarted`, `ParallelChunkCompleted`
- **Value Objects**: `WorkChunk`, `ParallelConfig`, `ThreadPool`

## 📖 Historias de Usuario

### HU-PAR-001: ParallelExecutor Core

**Como** scheduler
**Quiero** ejecutar sistemas en paralelo cuando sea posible
**Para** mejorar el throughput de la pipeline de render

#### Criterios de Aceptación
- [ ] Struct `ParallelExecutor` con thread pool
- [ ] Método `execute_systems(systems: &mut [&mut dyn RenderSystem])`
- [ ] Detección automática de sistemas independientes
- [ ] Tests verifican ejecución paralela
- [ ] Benchmarks muestran mejora en multi-core

#### Tareas Técnicas
- [ ] Investigar rayon para Rust
- [ ] Implementar ParallelExecutor con rayon
- [ ] Configurar thread pool size
- [ ] Escribir tests de paralelismo
- [ ] Benchmark vs sequential execution

#### Investigación Previa
- **Bevy**: Built-in parallel executor que considera dependencias
- **Rayon**: Data parallelism para Rust
- **Patrón**: Fork-join parallelism

#### Estimación: XL
#### Estado: Pendiente

---

### HU-PAR-002: Dependency-Aware Scheduling

**Como** ParallelExecutor
**Quiero** conocer las dependencias entre sistemas
**Para** no ejecutar en paralelo sistemas que deben ser secuenciales

#### Criterios de Aceptación
- [ ] Análisis de dependencias entre sistemas
- [ ] Construcción de grafo de dependencias
- [ ] Identificación de sistemas paralelizables
- [ ] Tests verifican correctitud
- [ ] Documentación de restricciones

#### Tareas Técnicas
- [ ] Analizar sistema de dependencias del scheduler
- [ ] Identificar sistemas con/sin write conflicts
- [ ] Implementar dependency analyzer
- [ ] Tests de correctitud

#### Estimación: L
#### Estado: Pendiente

---

### HU-PAR-003: Chunked Parallel Iteration

**Como** sistema de render
**Quiero** iterar sobre entidades en chunks paralelos
**Para** procesar miles de entidades eficientemente

#### Criterios de Aceptación
- [ ] Particionamiento de entidades en chunks
- [ ] Ejecución paralela de chunks
- [ ] Reducción de resultados al final
- [ ] Tests verifican correctitud de resultados
- [ ] Benchmarks muestran speedup lineal

#### Criterios de Aceptación Detallados

**Chunking Strategy**
- [ ] Chunk size configurable (default: 64 entidades)
- [ ] Balance de carga entre threads
- [ ] Overhead de chunking < 1%

**Parallel Iteration**
- [ ]rayon::join para sistemas independientes
- [ ]rayon::par_iter para iteración de entidades
- [ ] Soporte para Write + Read locks

**Result Reduction**
- [ ] Merge de draw batches paralelos
- [ ] Verificación de consistencia
- [ ] Tests de race conditions

#### Tareas Técnicas
- [ ] Implementar chunking strategy
- [ ] Integrar rayon para parallel iter
- [ ] Implementar merge de resultados
- [ ] Tests de race conditions
- [ ] Benchmark de speedup

#### Estimación: XL
#### Estado: Pendiente

---

### HU-PAR-004: Sync Barriers y Frames

**Como** sistema de render
**Quiero** barreras de sincronización entre etapas
**Para** asegurar que todos los threads terminen antes de avanzar

#### Criterios de Aceptación
- [ ] SyncBarrier para fin de etapa
- [ ].wait() blocks hasta completar
- [ ] Tests verifican sincronización
- [ ] Deadlock detection

#### Tareas Técnicas
- [ ] Implementar SyncBarrier
- [ ] Integrar con ParallelExecutor
- [ ] Tests de synchronización
- [ ] Prevention de deadlocks

#### Estimación: M
#### Estado: Pendiente

---

### HU-PAR-005: Thread Safety Verification

**Como** desarrollador
**Quiero** verificación de thread safety en tiempo de compilación
**Para** evitar race conditions en sistemas paralelos

#### Criterios de Aceptación
- [ ] Análisis de Send + Sync traits
- [ ] Tests de thread safety
- [ ] Documentación de requisitos por sistema
- [ ] Herramientas de verificación

#### Tareas Técnicas
- [ ] Revisar Send/Sync para cada componente
- [ ] Tests de thread safety
- [ ] Documentación de constraints
- [ ] CI checks para thread safety

#### Estimación: M
#### Estado: Pendiente

---

## 🔬 Patrones de Referencia

### Bevy Parallel Executor

```
Systems
  ├── Independent (run in parallel)
  │   ├── System A
  │   └── System B
  │
  └── Dependent (run sequentially)
      ├── System C (depends on A, B)
      └── System D (depends on C)
```

### Rayon Parallel Pipeline

```rust
// Parallel iteration con rayon
use rayon::prelude::*;

pub fn process_entities_parallel(entities: &[EntityData]) -> DrawBatch {
    let chunks: Vec<_> = entities.par_chunks(64)
        .map(|chunk| process_chunk(chunk))
        .collect();
    
    merge_batches(chunks)
}
```

### Work Stealing

```
Thread 0 ──┐
Thread 1 ──┼──► Work Stealing Queue ──► Idle threads
Thread 2 ──┘                              │
                                          ▼
                                    Ready work
```

---

## 📊 Métricas de Éxito

| Métrica | Target | Medida |
|---------|--------|--------|
| Speedup (4 cores) | 3.2x | Parallel vs Sequential |
| Overhead de chunking | < 2% | Chunking overhead |
| Frame time (10k entities) | < 16ms | 60 FPS target |
| Thread utilization | > 85% | CPU usage |

---

## 📊 Estado de Tareas

| Historia | Estado | Tests | Deuda |
|----------|--------|-------|-------|
| HU-PAR-001 | ⏳ Pendiente | 0/6 | - |
| HU-PAR-002 | ⏳ Pendiente | 0/8 | - |
| HU-PAR-003 | ⏳ Pendiente | 0/15 | - |
| HU-PAR-004 | ⏳ Pendiente | 0/5 | - |
| HU-PAR-005 | ⏳ Pendiente | 0/4 | - |

---

## 📝 Resumen Ejecutivo

Implementar ejecución paralela de sistemas ECS utilizando rayon, permitiendo que sistemas independientes se ejecuten concurrentemente y que la iteración sobre entidades se realice en chunks paralelos. El objetivo es lograr un speedup de ~3x en escenas con miles de entidades.

## 🔗 Dependencias

- Depende de: EPIC-ECS-SCHEDULER (usa el scheduler para ordenamiento)
- Habilita: Optimización de rendimiento para escenarios con muchas entidades

## 📁 Archivos de Salida

```
docs/
  epics/
    EPIC-ECS-PARALLEL.md ← Este archivo
crates/archflow-engine/
  src/
    parallel/
      mod.rs
      executor.rs
      chunking.rs
      barrier.rs
      thread_pool.rs
tests/
  hu_parallel.rs
```

## ⚠️ Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Race conditions | Alto | Media | Tests exhaustivos |
| Deadlocks | Alto | Baja | Timeout + detection |
| Cache contention | Medio | Media | Chunking strategy |
| Portabilidad | Bajo | Baja | Configurar thread pool |

## 📚 Recursos

- [Rayon documentation](https://docs.rs/rayon/latest/rayon/)
- [Bevy parallel executor](https://docs.rs/bevy_ecs/latest/bevy_ecs/)
- [Work stealing algorithms](https://ieftimov.com/posts/theory-in-practice-understanding-work-stealing/)
