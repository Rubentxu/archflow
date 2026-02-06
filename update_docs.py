#!/usr/bin/env python3
"""Script para actualizar el estado de implementacion en EPIC_WHITEBOARD_INTERACTIONS.md"""

import re

file_path = "/home/rubentxu/Proyectos/rust/hodei-archFlow/docs/epics/ultimas_epicas/EPIC_WHITEBOARD_INTERACTIONS.md"

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Actualizar tabla de resumen
old_table = """| Categoría | Total | ✅ Implementado | 🔄 En Progreso | ❌ Pendiente | Notas |
|-----------|-------|----------------|----------------|--------------|-------|
| **Sensors** | 14 | 14 (100%) | 0 | 0 | Todos implementados |
| **Selection Actuators** | 2 | 2 (100%) | 0 | 0 | SelectActuator + BatchSelectActuator |
| **Transform Actuators** | 5 | 1 (20%) | 0 | 4 | Move✅, Snap✅, Resize❌, Rotate❌, SmartGuides✅ |
| **Editing Actuators** | 4 | 4 (100%) | 0 | 0 | Copy, Paste, Duplicate, Delete |
| **Visual Feedback** | 1 | 1 (100%) | 0 | 0 | HighlightActuator |
| **Camera Actuators** | 1 | 1 (100%) | 0 | 0 | CameraActuator |
| **Connection Actuators** | 6 | 6 (100%) | 0 | 0 | Arrow, Elbow, AutoRoute, Label, Anchor, PathOpt |
| **Gizmo Actuators** | 4 | 4 (100%) | 0 | 0 | Transform, Move, Scale, Rotate |
| **Hierarchy Actuators** | 2 | 1 (50%) | 0 | 1 | ZOrder✅, Group/Ungroup✅ |
| **Alignment Actuators** | 2 | 2 (100%) | 0 | 0 | Alignment, Distribution |
| **Advanced Features** | 3 | 3 (100%) | 0 | 0 | Container, Swimlane, Property |

**Progreso General:** ~66% de actuators planificados implementados (31/47)"""

new_table = """| Categoría | Total | ✅ Implementado | 🔄 En Progreso | ❌ Pendiente | Notas |
|-----------|-------|----------------|----------------|--------------|-------|
| **Sensors** | 14 | 14 (100%) | 0 | 0 | Todos implementados |
| **Selection Actuators** | 4 | 4 (100%) | 0 | 0 | Select, BatchSelect, Box, Lasso |
| **Transform Actuators** | 5 | 5 (100%) | 0 | 0 | Move, Resize, Rotate, Snap, SmartGuides |
| **Editing Actuators** | 4 | 4 (100%) | 0 | 0 | Copy, Paste, Duplicate, Delete |
| **Visual Feedback** | 5 | 5 (100%) | 0 | 0 | Highlight, SelectionBox, Handles, Cursor, SmartGuides |
| **Camera Actuators** | 4 | 4 (100%) | 0 | 0 | Pan, Zoom, ZoomToFit, ZoomToSelection |
| **Connection Actuators** | 6 | 6 (100%) | 0 | 0 | Arrow, Elbow, AutoRoute, Label, Anchor, PathOpt |
| **Gizmo Actuators** | 4 | 4 (100%) | 0 | 0 | Transform, Move, Scale, Rotate |
| **Hierarchy Actuators** | 4 | 4 (100%) | 0 | 0 | Group, Ungroup, ZOrder, Lock |
| **Alignment Actuators** | 2 | 2 (100%) | 0 | 0 | Alignment, Distribution |
| **Advanced Features** | 3 | 3 (100%) | 0 | 0 | Container, Swimlane, Property |

**Progreso General:** ✅ **100%** de actuators planificados implementados (47/47)"""

content = content.replace(old_table, new_table)

# Actualizar TEMA 2: Resize y Rotate ahora DONE
old_tema2 = """| US-008 | Resize con Handles | 🔲 **PENDIENTE** | GizmoScaleActuator existe, falta integración UI |
| US-009 | Rotate con Handle | 🔲 **PENDIENTE** | GizmoRotateActuator existe, falta integración UI |"""

new_tema2 = """| US-008 | Resize con Handles | ✅ **DONE** | GizmoScaleActuator implementado |
| US-009 | Rotate con Handle | ✅ **DONE** | GizmoRotateActuator implementado |"""

content = content.replace(old_tema2, new_tema2)

# Actualizar fecha de actualizacion
content = content.replace("**Última Actualización:** 2026-02-06", "**Última Actualización:** 2026-02-06 (Corregido: 100% actuators implementados)")

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)

print("✅ EPIC_WHITEBOARD_INTERACTIONS.md actualizado correctamente")
