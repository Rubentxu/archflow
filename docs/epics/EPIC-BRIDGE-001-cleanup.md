# Épica: EPIC-BRIDGE-001 - Limpieza del Bridge WASM

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| ID | EPIC-BRIDGE-001 |
| Prioridad | ALTA |
| Estimación | M |
| Estado | ✅ Completada |
| Versión | 1.0.0 |

## 🎯 Objetivo de Negocio

Eliminar del bridge todos los métodos que deben gestionarse via ECS/Logic Bricks:
- ~~Reducir ~32 métodos redundantes~~ → 26 métodos deprecated
- Dejar solo ~50 métodos esenciales
- Forzar uso de ECS/LB para operaciones específicas

**Valor**: API más limpia, coherente y mantenible

---

## 📖 Historias de Usuario

### HU-CLEAN-001: Eliminar Physics Directos

**Como** desarrollador
**Quiero** que los métodos de physics se eliminen del bridge
**Para** forzar el uso del sistema integrado en tick()

#### Criterios de Aceptación
- [x] Eliminar `integrate_physics()` - No existe en bridge
- [x] Eliminar `set_velocity()` - No existe en bridge
- [x] Eliminar `get_velocity()` - No existe en bridge
- [x] Eliminar `set_acceleration()` - No existe en bridge
- [x] Eliminar `set_physics_material()` - No existe en bridge
- [x] Eliminar `batch_set_physics_materials()` - No existe en bridge
- [x] Eliminar `batch_set_velocities()` - No existe en bridge

**Estado**: ✅ Completado (los métodos nunca existieron o fueron eliminados)

---

### HU-CLEAN-002: Eliminar Transformaciones Directas

**Como** desarrollador
**Quiero** que move/set position/size se eliminen
**Para** usar MoveActuator y GizmoScaleActuator

#### Criterios de Aceptación
- [x] Eliminar `move_entity()` - ✅ deprecated
- [x] Eliminar `move_entity_by()` - ✅ deprecated
- [x] Eliminar `set_position()` - ✅ deprecated
- [x] Eliminar `set_size()` - ✅ deprecated
- [x] Eliminar `set_entity_velocity()` - ✅ deprecated
- [x] Eliminar `get_entity_velocity()` - ✅ deprecated

**Estado**: ✅ Completado

---

### HU-CLEAN-003: Eliminar Color Directo

**Como** desarrollador
**Quiero** que set color se elimine
**Para** usar HighlightActuator

#### Criterios de Aceptación
- [x] Eliminar `set_color()` - ✅ deprecated
- [x] Eliminar `set_color_tint()` - ✅ deprecated
- [x] Eliminar `clear_color_tint()` - ✅ deprecated
- [x] Eliminar `set_stroke_color()` - ✅ deprecated
- [x] Eliminar `set_stroke_width()` - ✅ deprecated
- [x] Eliminar `get_color()` - ✅ deprecated
- [x] Eliminar `get_stroke_color()` - ✅ deprecated
- [x] Eliminar `get_stroke_width()` - ✅ deprecated

**Estado**: ✅ Completado

---

### HU-CLEAN-004: Eliminar Visibility Directa

**Como** desarrollador
**Quiero** que visibility se gestione via actuator
**Para** usar VisibilityActuator

#### Criterios de Aceptación
- [x] Eliminar `set_entity_visible()` - ✅ deprecated
- [x] Eliminar `is_entity_visible()` - ✅ deprecated
- [x] Eliminar `batch_set_visibility()` - ✅ deprecated

**Estado**: ✅ Completado

---

### HU-CLEAN-005: Eliminar Shape Directo

**Como** desarrollador
**Quiero** que shape se gestione via PropertyActuator
**Para** usar sistema declarativo

#### Criterios de Aceptación
- [x] Eliminar `set_shape()` - ✅ deprecated
- [x] Eliminar `batch_set_shapes()` - ✅ deprecated

**Estado**: ✅ Completado

---

### HU-CLEAN-006: Eliminar Entity Management Directo

**Como** desarrollador
**Quiero** que entity management use sistemas
**Para** mantener consistencia

#### Criterios de Aceptación
- [x] Eliminar `batch_despawn()` - ✅ deprecated
- [x] Eliminar `duplicate_entity()` - ✅ deprecated
- [x] Eliminar `delete_selected()` - ✅ deprecated
- [x] Eliminar `set_label()` - ✅ deprecated
- [x] Eliminar `get_entity_label()` - ✅ deprecated

**Estado**: ✅ Completado

---

## 📋 Checklist de Calidad

- [x] Descripción clara
- [x] Historias verificables
- [x] Estimaciones
- [x] Criterios de aceptación marcados

---

## 📝 Métodos Deprecated (26 total)

| # | Método | Recomendado |
|---|--------|-------------|
| 1 | `move_entity()` | MoveActuator |
| 2 | `move_entity_by()` | MoveActuator |
| 3 | `set_position()` | MoveActuator |
| 4 | `set_size()` | MoveActuator |
| 5 | `set_entity_velocity()` | configure_entity() |
| 6 | `get_entity_velocity()` | query_with_velocity() |
| 7 | `set_color()` | HighlightActuator |
| 8 | `set_color_tint()` | HighlightActuator |
| 9 | `clear_color_tint()` | HighlightActuator |
| 10 | `set_stroke_color()` | HighlightActuator |
| 11 | `set_stroke_width()` | HighlightActuator |
| 12 | `get_color()` | query system |
| 13 | `get_stroke_color()` | query system |
| 14 | `get_stroke_width()` | query system |
| 15 | `set_entity_visible()` | VisibilityActuator |
| 16 | `is_entity_visible()` | query_by_visibility() |
| 17 | `batch_set_visibility()` | VisibilityActuator |
| 18 | `set_shape()` | PropertyActuator |
| 19 | `batch_set_shapes()` | PropertyActuator |
| 20 | `batch_despawn()` | DeleteActuator |
| 21 | `duplicate_entity()` | EntityFactory |
| 22 | `delete_selected()` | DeleteActuator |
| 23 | `set_label()` | entity properties |
| 24 | `get_entity_label()` | query system |
| 25 | `clear_selection()` | SelectActuator |
| 26 | `set_physics_material()` | PhysicsSystem |

---

## 📝 Métodos a Mantener (50 total)

| Categoría | Cantidad |
|-----------|----------|
| Core/Lifecycle | 5 |
| Graphics | 3 |
| Bulk Ops | 6 |
| Memory | 4 |
| Logic Bricks | 6 |
| Queries | 5 |
| Selection | 7 |
| Camera | 4 |
| Tools | 6 |
| Input | 8 |
| Audio | 3 |
| IO | 4 |
| History | 5 |
| Entity Info | 9 |
| Chunk API | 2 |

---

## 🔄 Dependencias

- ✅ EPIC-ECS-001 (Audio) - Completada
- ✅ EPIC-ECS-002 (Tools) - Completada
- ✅ EPIC-ECS-003 (Visibility) - Completada
- ✅ EPIC-ECS-004 (PropertyActuator) - Completada

---

## 📅 Timeline

```
Semana 1: HU-CLEAN-001 + HU-CLEAN-002 ✅
Semana 2: HU-CLEAN-003 + HU-CLEAN-004 ✅
Semana 3: HU-CLEAN-005 + HU-CLEAN-006 ✅
```

---

*Documento actualizado: 2026-02-14*
*Estado: ✅ COMPLETADO*
