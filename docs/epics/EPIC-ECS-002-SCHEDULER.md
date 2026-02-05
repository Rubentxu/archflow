# Épica: ECS Render Scheduler

## 📌 metadata
| Campo | Valor |
|-------|-------|
| ID | EPIC-ECS-002 |
| Prioridad | Media |
| Estimación | XL (3-4 semanas) |
| Estado | Borrador |
| Versión | 0.1.0 |
| Dependencia | EPIC-ECS-001 |

## 🎯 Objetivo de Negocio

Implementar un sistema de scheduling para la pipeline de render que permita ejecutar sistemas en un orden definido con dependencias explícitas. Esto es fundamental para evitar bugs por orden de ejecución incorrecto y habilitar profiling por sistema.

## 🔗 Dependencias

- Depende de: **EPIC-ECS-001** (los sistemas usarán Query abstraction)
- Habilita: **EPIC-ECS-003** (el scheduler habilitará parallel execution)

## 🏗️ Arquitectura DDD

- **Bounded Context**: `archflow-engine` (Render Subsystem)
- **Aggregate Root**: `RenderSchedule`
- **Domain Events**: `SystemStarted`, `SystemCompleted`, `ScheduleExecuted`
- **Value Objects**: `SystemOrder`, `Dependency`, `ScheduleConfig`

## 📖 Historias de Usuario

### HU-SCH-001: RenderSchedule Core

**Como** desarrollador
**Quiero** un `RenderSchedule` que ejecute sistemas en orden
**Para** que la pipeline de render tenga un orden determinista y predecible

#### Criterios de Aceptación
- [ ] Struct `RenderSchedule` con sistemas registrados
- [ ] Método `execute(store: &mut EntityStore, camera: &Camera)`
- [ ] Ejecución secuencial de sistemas en orden de registro
- [ ] Tests verifican orden de ejecución
- [ ] Benchmarks muestran overhead < 5%

#### Tareas Técnicas
- [ ] Investigar Bevy Schedule design
- [ ] Definir trait `RenderSystem`
- [ ] Implementar RenderSchedule básico
- [ ] Escribir tests de ordenamiento
- [ ] Benchmark del scheduler

#### Investigación Previa
- **Bevy Schedule**: Systems tienen scheduling metadata, built-in parallel executor
- **Patrón**: Command pattern + Chain of Responsibility

#### Estimación: L
#### Estado: Pendiente

---

### HU-SCH-002: Sistema de Dependencias

**Como** desarrollador
**Quiero** declarar dependencias entre sistemas (A depende de B)
**Para** que el scheduler respete el orden correcto automáticamente

#### Criterios de Aceptación
- [ ] Método `.before(system)` en RenderSystem
- [ ] Método `.after(system)` en RenderSystem
- [ ] Resolución automática de orden topológico
- [ ] Detección de ciclos (error en tiempo de compilación)
- [ ] Tests verifican resolución de dependencias

#### Tareas Técnicas
- [ ] Diseñar API de dependencias
- [ ] Implementar grafo de dependencias
- [ ] Algoritmo de ordenamiento topológico
- [ ] Detección de ciclos
- [ ] Tests de integración

#### Estimación: L
#### Estado: Pendiente

---

### HU-SCH-003: Pipeline de Render Systems

**Como** sistema de render
**Quiero** tener sistemas especializados para cada etapa
**Para** que cada etapa sea testeable y perfilable independientemente

#### Criterios de Aceptación
- [ ] `TransformSyncSystem` - sincroniza transforms
- [ ] `CullingSystem` - viewport culling
- [ ] `BatchingSystem` - organiza batches
- [ ] `InstanceUploadSystem` - prepara GPU data
- [ ] Tests para cada sistema individual
- [ ] Integración con RenderSchedule

#### Criterios de Aceptación Detallados

**TransformSyncSystem**
- [ ] Lee entidades del store
- [ ] Actualiza world_transform
- [ ] Respet dirty flags
- [ ] O(dirty) complexity

**CullingSystem**
- [ ] Usa camera.viewport_bounds()
- [ ] Marka entidades visibles/no visibles
- [ ] Pre-calcula viewport intersection

**BatchingSystem**
- [ ] Organiza por RenderPhase
- [ ] Crea draw batches
- [ ] Calcula batch indices

**InstanceUploadSystem**
- [ ] Convierte componentes a GpuInstance
- [ ] Prepara camera uniforms
- [ ] Organiza por fase

#### Tareas Técnicas
- [ ] Implementar TransformSyncSystem
- [ ] Implementar CullingSystem
- [ ] Implementar BatchingSystem
- [ ] Implementar InstanceUploadSystem
- [ ] Integrar con RenderSchedule
- [ ] Tests de integración

#### Estimación: XL
#### Estado: Pendiente

---

### HU-SCH-004: Profiling y Métricas

**Como** desarrollador
**Quiero** métricas por sistema (tiempo, iteraciones)
**Para** optimizar cuellos de botella en la pipeline

#### Criterios de Aceptación
- [ ] Métricas por sistema: tiempo ejecución, iteraciones, entidades procesadas
- [ ] API para acceder a métricas post-frame
- [ ] Integración con tracing
- [ ] Visualización de métricas (debug mode)

#### Tareas Técnicas
- [ ] Instrumentar cada sistema con métricas
- [ ] Crear MetricsCollector
- [ ] Integrar con tracing subscriber
- [ ] Tests de métricas

#### Estimación: M
#### Estado: Pendiente

---

## 🔬 Investigación de Referencia

### Bevy Schedule Architecture

```
Schedule
├── Systems (ordenados por depends)
├── Stages ( grupos de sistemas)
└── Parallel Executor
    ├── Dependency graph
    └── Worker threads
```

### Render Pipeline de Referencia

```
Frame Start
    │
    ├── TransformSync ──► (actualiza world_transform)
    │         │
    ├── Culling ──────► (marca visibilidad)
    │         │
    ├── Batching ─────► (organiza por fase)
    │         │
    ├── InstancePrep ──► (convierte a GPU format)
    │         │
    └── Upload ───────► (envía a GPU)
            │
            ▼
        Render
```

---

## 📊 Estado de Tareas

| Historia | Estado | Tests | Deuda |
|----------|--------|-------|-------|
| HU-SCH-001 | ⏳ Pendiente | 0/6 | - |
| HU-SCH-002 | ⏳ Pendiente | 0/8 | - |
| HU-SCH-003 | ⏳ Pendiente | 0/20 | - |
| HU-SCH-004 | ⏳ Pendiente | 0/5 | - |

---

## 📝 Resumen Ejecutivo

Implementar un sistema de scheduling para la pipeline de render que permita ejecutar sistemas especializados (TransformSync, Culling, Batching, InstanceUpload) en un orden definido con dependencias explícitas. Esto es fundamental para evitar bugs de orden de ejecución y habilitar profiling por sistema.

## 🔗 Dependencias

- Depende de: EPIC-ECS-QUERY (los sistemas usarán Query abstraction)
- Habilita: EPIC-ECS-PARALLEL (el scheduler habilitará parallel execution)

## 📁 Archivos de Salida

```
docs/
  epics/
    EPIC-ECS-SCHEDULER.md ← Este archivo
crates/archflow-engine/
  src/
    scheduler/
      mod.rs
      schedule.rs
      system.rs
      metrics.rs
      systems/
        mod.rs
        transform_sync.rs
        culling.rs
        batching.rs
        instance_upload.rs
tests/
  hu_sch_scheduler.rs
```
