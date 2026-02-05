---
description: Sistema de tracking para estado de tareas, problemas, deuda técnica y notas de review
command_name: /tracking
requires_context: false
context_variables:
  - active_epic         # Épica activa
  - current_task        # Tarea actual
  - issues_found        # Problemas encontrados
  - technical_debt      # Deuda técnica identificada
---

# Sistema de Tracking de Proyecto

> **Rol**: Arquitecto de software Rust, Product Owner, especialista en gestión de deuda técnica y quality assurance.

## 📁 Estructura del Sistema de Tracking

```
docs/tracking/
├── tracking.json          # Estado global (auto-generado)
├── project-status.md      # Estado del proyecto (este archivo)
├── debt-technique.md      # Deuda técnica documentada
├── issues-log.md          # Registro de problemas
└── review-notes.md        # Notas para review futuro
```

---

## 📊 Dashboard Principal

### project-status.md

```markdown
---
project: hodei-archFlow
version: 3.12.0
last_updated: 2025-02-01
tracker_version: 1.0.0
---

# Project Status Dashboard

## 📈 Métricas Generales

| Métrica | Valor | Estado |
|---------|-------|--------|
| Épicas Completadas | 3/5 | 🟡 60% |
| Historias Completadas | 12/18 | 🟡 66% |
| Tests Pasando | 156/162 | 🟢 96% |
| Deuda Técnica Alta | 2 | 🔴 Crítico |
| Deuda Técnica Media | 5 | 🟡 Warning |
| Issues Abiertos | 8 | 🟡 Monitorizar |

## 🎯 Progreso por Épica

| Épica | Progreso | Estado | Notas |
|-------|----------|--------|-------|
| Epic 1: Core Domain | 100% | ✅ Completada | - |
| Epic 2: Web UI | 100% | ✅ Completada | - |
| Epic 3: Sensors System | 75% | 🔄 En Progreso | Bloqueado por HU-003 |
| Epic 4: Event System | 30% | 🔄 En Progreso | - |
| Epic 5: Performance | 0% | ⏳ Pendiente | Depende de Epic 3,4 |

## 📅 Sprint Actual

**Sprint 5** (2025-02-01 al 2025-02-15)

### Objetivos
- [ ] Completar HU-003 (Double Tap Sensor)
- [ ] Resolver deuda técnica alta DEBT-001
- [ ] Investigar integración con Leptos

### Bloqueadores
- 🔴 DEBT-001: Memory leak en event handler
- 🟡 DEBT-003: Acoplamiento fuerte en sensor module

---

## ✅ Estado de Tareas

### Tareas Activas

| ID | Tarea | Épica | Prioridad | Estado | Progress |
|----|-------|-------|-----------|--------|----------|
| T-042 | Implementar Double Tap | Epic 3 | Alta | 🔄 In Progress | 60% |
| T-043 | Fix memory leak DEBT-001 | - | Crítica | 🔄 In Progress | 40% |
| T-044 | Documentar API Sensors | Epic 3 | Media | ⏳ Pending | 0% |
| T-045 | Integrar Leptos signals | Epic 4 | Alta | ⏳ Pending | 0% |

### Tareas Completadas (Últimas)

| ID | Tarea | Completada | Effort |
|----|-------|------------|--------|
| T-041 | Implementar MouseOver | 2025-01-28 | M |
| T-040 | Refactor EventBus | 2025-01-25 | L |
| T-039 | Setup CI/CD | 2025-01-20 | M |

---

## 🔧 Deuda Técnica

### debt-technique.md

```markdown
---
section: Debt Technique
updated: 2025-02-01
---

# Deuda Técnica Documentada

## 🔴 Alta Prioridad (Debe resolverse)

| ID | Severity | Descripción | Módulo | Solución | Estimación |
|----|----------|-------------|--------|----------|------------|
| DEBT-001 | ALTA | Memory leak en EventHandler cuando no hay subscribers | events | Agregar drop check + tests | 2 días |
| DEBT-002 | ALTA | Coupling fuerte entre Sensor y EventBus | sensors | Extraer trait abstraction | 3 días |

## 🟡 Media Prioridad (Resolver en sprint)

| ID | Severity | Descripción | Módulo | Solución | Estimación |
|----|----------|-------------|--------|----------|------------|
| DEBT-003 | MEDIA | Tests de integración lentos (>5min) | testing | Paralelizar con nextest | 1 día |
| DEBT-004 | MEDIA | Cargo.lock desactualizado | root | Regenerar con cargo update | 1 hora |
| DEBT-005 | MEDIA | Documentación API incompleta | docs | Completar KDoc en pub API | 2 días |

## 🟢 Baja Prioridad (Backlog)

| ID | Severity | Descripción | Módulo | Solución | Estimación |
|----|----------|-------------|--------|----------|------------|
| DEBT-006 | BAJA | Naming inconsistente en some functions | utils | Rename batch | 4 horas |
| DEBT-007 | BAJA | Dead code en tests legacy | testing | Eliminar con cargo-udeps | 2 horas |

---

## 📋 Actions Required

### Inmediato (Esta semana)
- [ ] DEBT-001: Implementar fix memory leak
- [ ] DEBT-002: Diseñar abstracción SensorTrait

### Este Sprint
- [ ] DEBT-003: Configurar nextest
- [ ] DEBT-004: Regenerar Cargo.lock

### Backlog
- [ ] DEBT-006: Batch rename functions
- [ ] DEBT-007: Cleanup dead code
```

---

## 🐛 Registro de Problemas

### issues-log.md

```markdown
---
section: Issues Log
updated: 2025-02-01
---

# Registro de Problemas

## 🔴 Críticos (Bloquean desarrollo)

| ID | Severity | Título | Fecha | Estado | Resolución |
|----|----------|--------|-------|--------|------------|
| ISS-012 | CRÍTICO | Build falla en CI por panic del compilador | 2025-01-30 | 🔄 Resolviendo | Actualizar rust-toolchain |
| ISS-011 | CRÍTICO | Cargo.lock conflicto entre ramas | 2025-01-25 | ✅ Resuelto | Rebase y regenerate |

## 🟡 Medios (Requieren atención)

| ID | Severity | Título | Fecha | Estado | Resolución |
|----|----------|--------|-------|--------|------------|
| ISS-010 | MEDIO | Deprecation warning en tokio::spawn | 2025-01-28 | ⏳ Pendiente | Actualizar en próximo minor |
| ISS-009 | MEDIO | rust-analyzer slow en archivos grandes | 2025-01-22 | ⏳ Pendiente | Configurar incremental |
| ISS-008 | MEDIO | Clippy warnings en workspace | 2025-01-18 | ✅ Resuelto | Aplicar sugerencias |

## 🟢 Resueltos (Historial)

| ID | Severity | Título | Resolución | Fecha |
|----|----------|--------|------------|-------|
| ISS-007 | BAJO | Formato inconsistente | cargo fmt | 2025-01-15 |
| ISS-006 | MEDIO | Panic en test de integración | Mock corrected | 2025-01-12 |
| ISS-005 | CRÍTICO | Race condition en EventBus | Arc<Mutex> + channel | 2025-01-08 |

---

## 📝 Detalle de Problemas Activos

### ISS-012: Build falla en CI por panic del compilador

**Severity:** CRÍTICO  
**Estado:** 🔄 Resolviendo  
**Fecha apertura:** 2025-01-30

**Descripción:**
```
error: internal compiler error: unexpected panic
thread 'rustc' panicked at compiler/rustc_codegen_llvm/src/llvm/ffi.rs:123
```

**Impacto:** CI/CD bloqueado, deploys pausados

**Causa probable:** Incompatibilidad con nightly build

**Solución propuesta:**
```yaml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

**Progreso:** 50%  
**Owner:** @dev-team

---

### ISS-010: Deprecation warning en tokio::spawn

**Severity:** MEDIO  
**Estado:** ⏳ Pendiente  
**Fecha apertura:** 2025-01-28

**Descripción:**
```
warning: use of deprecated function `tokio::spawn`
note: deprecated since tokio 1.40
help: use `tokio::task::spawn` instead
```

**Impacto:** Warnings en compilación, técnico menor

**Solución:** Actualizar llamadas en siguiente release minor

```

---

## 📝 Notas para Review

### review-notes.md

```markdown
---
section: Review Notes
updated: 2025-02-01
---

# Notas para Review Futuro

## 🔍 Items a Revisar en Próximo Review

### Arquitectura
- [ ] Revisar EventBus scalability - ¿soporta 10K eventos/seg?
- [ ] Validar abstracción de sensores - ¿demasiada abstracción?
- [ ] Evaluar uso de Arc<Mutex> vsRwLock

### Código
- [ ] HU-003: ¿Tests cubren edge cases de Double Tap?
- [ ] HU-004: ¿API es idiomática Rust?
- [ ] Documentación: ¿KDoc es completo?

### Proceso
- [ ] Tiempo real vs estimado de tareas
- [ ] Calidad de commits (atomicidad)
- [ ] Cobertura de tests por módulo

---

## 📌 Decisiones Técnicas Documentadas

### DEC-001: Selección de Event Bus

**Fecha:** 2025-01-10  
**Contexto:** Necesidad de comunicación entre sensores y UI

**Decisión:** Usar `tokio::sync::broadcast` + Arc

**Alternativas consideradas:**
- `crossterm` events: Demasiado acoplado a terminal
- `enigo`: Solo mouse, no extensible

**Resultado:** ✅ Funcionando bien, considerara migration a dedicated event bus library si escala

---

### DEC-002: Patrón State para Sensores

**Fecha:** 2025-01-15  
**Contexto:** Múltiples estados en sensor (idle, active, calibrating)

**Decisión:** Type State Pattern con enum state machine

**Resultado:** ✅ Reduce estados inválidos

---

## 💭 Questions to Revisit

| Pregunta | Contexto | Revisitar |
|----------|----------|-----------|
| ¿Leptos es overkill para UI? | Solo 3 views сейчас | Después de Epic 4 |
| ¿Separación en múltiples crates? | Monorepo actual | Después de v1.0 |
| ¿Event sourcing para audit log? | Requiere ahora no | v2.0 |

---

## 📊 Métricas de Review

### Último Review: 2025-01-25

| Métrica | Valor | Objetivo | Delta |
|---------|-------|----------|-------|
| Coverage | 72% | 80% | +5% |
| Cyclomatic complexity avg | 4.2 | <5 | -0.3 |
| Lines of code | 8,450 | - | +420 |
| Technical debt items | 7 | <5 | +2 |
| Aceptance criteria met | 94% | 95% | +2% |

---

## 🎯 Action Items del Último Review

- [ ] Incrementar coverage a 80% (HU-003, HU-004)
- [ ] Reducir deuda técnica de 7 a 5 items
- [ ] Documentar decisiones DEC-001, DEC-002
```

---

## 📋 Comandos de Uso del Sistema

### Ver estado general
```bash
# Ver dashboard
cat docs/tracking/project-status.md

# Ver deuda técnica
cat docs/tracking/debt-technique.md

# Ver problemas activos
cat docs/tracking/issues-log.md
```

### Actualizar estado de tarea
```bash
# Editar directamente en project-status.md
# Buscar la tarea y cambiar estado:
# ⏳ Pending → 🔄 In Progress → ✅ Completed
```

### Añadir nuevo problema
```bash
# Editar issues-log.md
# Añadir entrada en tabla correspondiente (Crítico/Medio/Bajo)
```

### Documentar deuda técnica
```bash
# Editar debt-technique.md
# Añadir entrada con formato:
# | ID | Severity | Descripción | Módulo | Solución | Estimación |
```

### Añadir nota para review
```bash
# Editar review-notes.md
# Añadir en sección correspondiente
```

---

## 🔄 Flujo de Trabajo con Tracking

### Inicio de Día
```bash
# 1. Ver estado actual
cat docs/tracking/project-status.md

# 2. Revisar tareas activas
grep -A5 "Tareas Activas" docs/tracking/project-status.md

# 3. Revisar bloqueadores
grep -A10 "Bloqueadores" docs/tracking/project-status.md
```

### Fin de Día
```bash
# 1. Actualizar progreso de tareas
# Editar project-status.md

# 2. Documentar problemas encontrados
# Editar issues-log.md

# 3. Documentar deuda técnica nueva
# Editar debt-technique.md

# 4. Añadir notas para review si applicable
# Editar review-notes.md
```

### En Review (Sprint/Mes)
```bash
# 1. Revisar métricas
grep -A20 "Métricas de Review" docs/tracking/review-notes.md

# 2. Evaluar deuda técnica
cat docs/tracking/debt-technique.md

# 3. Revisar problemas resueltos
grep -A5 "Resueltos" docs/tracking/issues-log.md

# 4. Actualizar notas de siguiente review
```

---

## 📊 Plantillas Quick-Add

### Nueva Tarea
```markdown
| T-XXX | [Título] | [Épica] | Alta/Media/Baja | ⏳ Pending | 0% |
```

### Nuevo Issue
```markdown
| ISS-XXX | CRÍTICO/MEDIO/BAJO | [Título] | YYYY-MM-DD | 🔄 Resolviendo | [Resolución] |
```

### Nueva Deuda Técnica
```markdown
| DEBT-XXX | ALTA/MEDIA/BAJA | [Descripción] | [Módulo] | [Solución] | [Estimación] |
```

### Nueva Nota para Review
```markdown
- [ ] [Item a revisar]
```

---

## 🛠️ Scripts de Utilidad

### Generar resumen rápido
```bash
#!/bin/bash
echo "=== Project Status ==="
echo "Épicas: $(grep -c '✅ Completada' docs/tracking/project-status.md)/$(grep -c 'Epic' docs/tracking/project-status.md)"
echo "Deuda Alta: $(grep -c 'ALTA' docs/tracking/debt-technique.md)"
echo "Issues Abiertos: $(grep -c '🔄 Resolviendo' docs/tracking/issues-log.md)"
```

### Verificar completitud
```bash
#!/bin/bash
echo "=== Tracking Health Check ==="
ls -la docs/tracking/
echo ""
echo "Tareas activas: $(grep -c '🔄 In Progress' docs/tracking/project-status.md)"
echo "Issues críticos: $(grep -c 'CRÍTICO' docs/tracking/issues-log.md)"
echo "Deuda alta: $(grep -c 'ALTA' docs/tracking/debt-technique.md)"
```

---

## ✅ Checklist de Mantenimiento

```markdown
- [ ] Daily: Actualizar estado de tareas
- [ ] Daily: Documentar problemas nuevos
- [ ] Weekly: Revisar deuda técnica
- [ ] Weekly: Actualizar métricas
- [ ] Sprint: Review completo del tracking
- [ ] Sprint: Archivar issues resueltos (>2 sprints)
- [ ] Release: Actualizar versión en metadata
```

---

## 📚 Recursos

- [Tracking files](docs/tracking/)
- [Éntregable templates](#plantillas-quick-add)
- [Scripts de utilidad](#scripts-de-utilidad)
