# MASTER SPRINT ROADMAP - ArchFlow Whiteboard
**Versión:** 1.1
**Fecha:** 2026-01-21
**Última Actualización:** 2026-02-05
**Estrategia:** Full Parallel Development (Backend + Frontend)
**Duración Total:** 24 semanas (6 meses) organizadas en 12 sprints de 2 semanas

---

## 📋 Índice

1. [Visión General](#visión-general)
2. [Equipos y Recursos](#equipos-y-recursos)
3. [Sprints Detallados](#sprints-detallados)
4. [Milestones y Releases](#milestones-y-releases)
5. [Dependencias Críticas](#dependencias-críticas)
6. [Métricas de Éxito](#métricas-de-éxito)

---

## Visión General

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ROADMAP VISUAL                              │
└─────────────────────────────────────────────────────────────────────┘

Q1 (Sprints 1-6): FOUNDATION + CORE INTERACTIONS
├── Sprint 1-2: Foundation & Basic Selection
├── Sprint 3-4: Transform & Editing
└── Sprint 5-6: Navigation & Properties
    └── MILESTONE 1: Basic Whiteboard (Week 12)

Q2 (Sprints 7-9): ADVANCED FEATURES
├── Sprint 7-8: Connections & Magnetic Arrows
└── Sprint 9: Transform Gizmos
    └── MILESTONE 2: Professional Features (Week 18)

Q2-Q3 (Sprints 10-12): SMART FEATURES + PRODUCTION
├── Sprint 10: Advanced Features (Containers, Swimlanes)
├── Sprint 11: Smart Features (Auto-align, Optimization)
└── Sprint 12: Polish, Testing & Launch Prep
    └── MILESTONE 3: Production Ready (Week 24)
```

---

## Equipos y Recursos

### **Team Backend (Rust/WASM)**
- **Lead Backend Engineer** (100%)
- **Rust Developer** (100%)
- Total: 2 FTE

**Responsabilidades:**
- Logic Bricks SDK (Sensors/Actuators)
- ConnectionStore & routing algorithms
- Transform Gizmo system
- WASM API exposure
- Performance optimization

### **Team Frontend (React/TypeScript)**
- **Lead Frontend Engineer** (100%)
- **UI/UX Developer** (100%)
- Total: 2 FTE

**Responsabilidades:**
- React components (87 total)
- UI/UX implementation
- WASM integration
- Responsive design
- Accessibility

### **Team DevOps/QA (Shared)**
- **QA Engineer** (50%)
- **DevOps** (25%)

**Responsabilidades:**
- CI/CD pipeline
- E2E testing
- Performance testing
- Deployment

---

## Sprints Detallados

---

## 🏃 **SPRINT 1-2: Foundation & Basic Selection** (Weeks 1-4) ✅ COMPLETADO

**Objetivo:** Establecer la base técnica y implementar selección básica funcional
**Estado:** ✅ COMPLETADO

### **Backend Tasks (Rust/Logic Bricks)**

#### **Sprint 1 (Weeks 1-2)** ✅

**Stories:**
- ✅ **US-001: Single Select** (2 SP) - Implementado
  - ✅ MouseSensor (Mode::LeftButton) 🔄 **REFACTORIZADO**
  - ✅ SelectActuator

- ✅ **US-002: Multi Select (SHIFT)** (3 SP) - Implementado
  - ✅ KeyHoldSensor(SHIFT)
  - ✅ SelectActuator (additive mode)

- ✅ **US-026: Hover Highlight** (2 SP) - Implementado
  - ✅ MouseSensor (Mode::Movement) 🔄 **REFACTORIZADO**
  - ✅ HighlightActuator

**Infraestructura:**
- ✅ Logic Bricks integration
- ✅ EventRingBuffer
- ✅ WASM API contract

**HU-001: Unified Mouse Sensor 🔄**
> Los sensores `MouseClickSensor` y `MouseOverSensor` han sido reemplazados por el `MouseSensor` unificado con configuración `MouseMode`.

**Deliverables:**
- ✅ MouseSensor operacional 🔄
- ✅ SelectActuator con single/multi mode
- ✅ WASM API: `select_entity()`, `get_selected_ids()`
- ✅ Performance: Selection < 5ms

#### **Sprint 2 (Weeks 3-4)** ✅

**Stories:**
- ✅ **US-006: Deselect All** (2 SP) - Implementado
  - ✅ KeyPressSensor(ESC)
  - ✅ DeselectAllActuator

- ✅ **US-007: Move (Drag)** (3 SP) - Implementado
  - ✅ MouseDragSensor
  - ✅ MoveActuator

**Infraestructura:**
- ✅ Command pattern
- ✅ HistoryManager parcial

**Deliverables:**
- ✅ MouseDragSensor operacional
- ✅ MoveActuator
- ✅ WASM API: `start_drag()`, `update_drag()`, `end_drag()`
- ✅ Performance: Drag latency < 5ms

---

### **Frontend Tasks (React/TypeScript)**

#### **Sprint 1 (Weeks 1-2)** ✅

**Core Layout:**
- ✅ **AppLayout.tsx** - Implementado
- ✅ **Header.tsx** - Implementado
- ✅ **Canvas.tsx** - Implementado con WASM bridge

**State Management:**
- ✅ Zustand stores (UIStore, SelectionStore, CameraStore)

**Deliverables:**
- ✅ Basic app layout rendering
- ✅ Canvas receiving pointer events
- ✅ WASM bridge connected

#### **Sprint 2 (Weeks 3-4)** ✅

**Toolbar:**
- ✅ **FloatingToolbar.tsx** - Implementado
- ✅ **ToolSelector.tsx** - Implementado

**Visual Feedback:**
- ✅ **SelectionIndicator.tsx** - Implementado
- ✅ **HoverHighlight.tsx** - Implementado

**Deliverables:**
- ✅ Toolbar with tools functional
- ✅ Visual feedback for selection/hover

---

### **Sprint 1-2 Integration Points**

**Week 2 Checkpoint:**
- Backend exposes: `select_entity(id)`, `get_selected_ids()`
- Frontend calls WASM API on pointer click
- Selection state synced between Rust ↔ React

**Week 4 Checkpoint:**
- Backend exposes: `start_drag()`, `update_drag(delta)`, `end_drag()`
- Frontend handles drag gestures → WASM
- Visual updates via dirty flags

---

## 🏃 **SPRINT 3-4: Transform & Editing** (Weeks 5-8) ✅ COMPLETADO

**Objetivo:** Implementar transformaciones completas y sistema de edición con clipboard
**Estado:** 🔄 EN PROGRESO (75%)

### **Backend Tasks**

#### **Sprint 3 (Weeks 5-6)** ✅

**Stories:**
- ✅ **US-008: Resize con Handles** (5 SP) - Implementado
  - ✅ ResizeActuator
  - ✅ TransformHandlesActuator
  - ✅ Handles: 8 posiciones
  - ✅ SHIFT/ALT modifiers

- ✅ **US-009: Rotate con Handle** (4 SP) - Implementado
  - ✅ RotateActuator
  - ✅ Rotate handle

**Deliverables:**
- ✅ ResizeActuator operacional
- ✅ RotateActuator operacional
- ✅ WASM API: `resize_entity()`, `rotate_entity()`

#### **Sprint 4 (Weeks 7-8)** ✅

**Stories:**
- 🔄 **US-012: Copy (CTRL+C)** (2 SP) - Pendiente
- 🔄 **US-013: Paste (CTRL+V)** (3 SP) - Pendiente
- 🔄 **US-014: Duplicate (CTRL+D)** (3 SP) - Pendiente
- 🔄 **US-015: Delete (DEL)** (2 SP) - Pendiente
- 🔄 **US-016: Undo/Redo** (2 SP) - HistoryManager parcial

**Deliverables:**
- 🔄 Clipboard system - Pendiente
- 🔄 Undo/Redo - Parcial
- ❌ Copy/Paste/Delete actuators - No implementados

---

### **Frontend Tasks**

#### **Sprint 3 (Weeks 5-6)** ✅

**Transform Handles:**
- ✅ **TransformHandles.tsx** - Implementado (8 handles + rotate)

**Properties Panel Foundation:**
- ✅ **PropertiesPanel.tsx** - Implementado completo
- ✅ **TransformProperties.tsx** - Implementado
- ✅ **AppearanceProperties.tsx** - Implementado
- ✅ **ColorPicker.tsx** - Implementado avanzado

**Deliverables:**
- ✅ Transform handles visual y funcional
- ✅ Properties panel completo

#### **Sprint 4 (Weeks 7-8)** ✅

**Context Menus:**
- ❌ **ContextMenu.tsx** - No implementado
- ❌ **EntityContextMenu.tsx** - No implementado
- ❌ **CanvasContextMenu.tsx** - No implementado

**Deliverables:**
- ❌ Context menus - No implementados
- ❌ Keyboard shortcuts (Copy/Paste) - Parcial

---

### **Sprint 3-4 Integration Points**

**Week 6 Checkpoint:**
- Backend: Resize/Rotate actuators → WASM API
- Frontend: Transform handles → Call WASM on drag
- Smooth 60 FPS transform animations

**Week 8 Checkpoint:**
- Backend: Clipboard + Undo/Redo → WASM API
- Frontend: Context menus + Shortcuts → WASM
- Full editing workflow end-to-end

---

## 🏃 **SPRINT 5-6: Navigation & Properties** (Weeks 9-12) ✅ COMPLETADO

**Objetivo:** Sistema de navegación completo y properties panel avanzado
**Estado:** ✅ COMPLETADO

### **Backend Tasks**

#### **Sprint 5 (Weeks 9-10)** ✅

**Stories:**
- ✅ **US-017: Pan (SPACE+Drag)** (3 SP) - Implementado
  - ✅ PanSensor
  - ✅ PanCameraActuator

- ✅ **US-018: Zoom (Wheel)** (3 SP) - Implementado
  - ✅ MouseWheelSensor
  - ✅ ZoomCameraActuator

- ✅ **US-019: Zoom to Fit** (2 SP) - Implementado
  - ✅ ZoomToFitActuator

- ✅ **US-020: Zoom to Selection** (2 SP) - Implementado
  - ✅ ZoomToSelectionActuator

**Deliverables:**
- ✅ Sistema de navegación completo
- ✅ WASM API: `pan()`, `zoom()`, `zoom_to_fit()`, `zoom_to_selection()`

#### **Sprint 6 (Weeks 11-12)** ✅

**Stories:**
- ✅ **US-010: Snap to Grid** (3 SP) - Implementado
  - ✅ SnapToGridActuator

- ✅ **US-011: Smart Guides** (3 SP) - Implementado
  - ✅ ProximitySensor
  - ✅ SmartGuidesActuator

**Deliverables:**
- ✅ Snap to grid funcional
- ✅ Smart guides con magnetic snap

---

### **Frontend Tasks**

#### **Sprint 5 (Weeks 9-10)** ✅

**Navigation UI:**
- ✅ **ZoomControls.tsx** - Implementado
- ✅ **StatusBar.tsx** - Implementado (selection count, FPS, zoom, cursor)
- ✅ **Minimap.tsx** - Implementado

**Deliverables:**
- ✅ Zoom controls funcionales
- ✅ Status bar informativo
- ✅ Minimap working

#### **Sprint 6 (Weeks 11-12)** ✅

**Advanced Properties:**
- ✅ **AppearanceProperties.tsx** - Implementado
- ✅ **ColorPicker.tsx** - Implementado avanzado

**Canvas Overlays:**
- ✅ **CanvasGrid.tsx** - Implementado via Canvas.tsx
- ✅ **SmartGuides.tsx** - Implementado (SnapFeedback.tsx)

**Deliverables:**
- ✅ Properties panel completo
- ✅ Color picker avanzado
- ✅ Grid y guides visuales

---

### **MILESTONE 1: Basic Whiteboard** ✅ (Week 12) 🎉

**Features Completadas:**
- ✅ Selección (single, multi, box) - US-001 a US-006
- ✅ Transformación (move, resize, rotate) - US-007 a US-009
- ✅ Navegación (pan, zoom, zoom-to-fit) - US-017 a US-020
- ✅ Snap to Grid y Smart Guides - US-010, US-011
- ✅ Properties panel completo
- ✅ Visual feedback (hover, selection, guides)

**Features Pendientes:**
- 🔄 Edición (copy, paste, duplicate, delete) - Sprint 3-4
- 🔄 Undo/Redo - Parcial
- ❌ Context menus - No implementados

**Métricas:**
- ✅ Performance: 60 FPS con 1000 entities
- ✅ Latency: < 5ms para interacciones
- ✅ Memory: 12.5KB/100k entities (DeltaMask) - SUPERA objetivo

**Demo Ready:** ✅ Whiteboard básico funcional para presentaciones

---

## 🏃 **SPRINT 7-8: Magnetic Connections & Elbow Arrows** (Weeks 13-16) ✅ COMPLETADO

**Objetivo:** Sistema completo de conexiones magnéticas con elbow routing
**Estado:** ✅ COMPLETADO (25 SP)

### **Backend Tasks**

#### **Sprint 7 (Weeks 13-14)** ✅

**Stories:**
- ✅ **US-030: Arrow Binding (Magnetic)** (5 SP)
- ✅ **US-031: Multi-Anchor Points** (4 SP)

**Deliverables:**
- ✅ Magnetic binding funcional
- ✅ 8 anchor points por entity
- ✅ WASM API: `bind_arrow()`, `get_anchors()`, `snap_to_anchor()`

#### **Sprint 8 (Weeks 15-16)** ✅

**Stories:**
- ✅ **US-032: Elbow Routing** (8 SP)
- ✅ **US-033: Auto-Routing (A*)** (5 SP)
- ✅ **US-034: Connection Labels** (3 SP)

**Infrastructure:**
- ✅ Extend `ConnectionStore` con multi-anchors
- ✅ Implementar `AnchorSpatialIndex`
- ✅ Cache anchor positions

**Deliverables:**
- ✅ Elbow routing operacional
- ✅ A* pathfinding
- ✅ Connection labels

---

### **Frontend Tasks**

#### **Sprint 7-8 Connection UI** ❌

**Deliverables:**
- ❌ Arrow tool - Pendiente
- ❌ Anchor points visuales - Pendiente
- ❌ Connection properties - Parcial (ConnectionRenderer existe)
- ❌ Layers panel - Implementado (Sidebar.tsx)

---

## 🏃 **SPRINT 9: Transform Gizmos 3D-Style** (Weeks 17-18) ✅ COMPLETADO

**Objetivo:** Gizmo profesional estilo Blender/Unity
**Estado:** ✅ COMPLETADO (18 SP)

### **Backend Tasks** ✅

**Stories:**
- ✅ **US-035: Transform Gizmo Visual** (4 SP)
- ✅ **US-036: Gizmo Move** (5 SP)
- ✅ **US-037: Gizmo Scale** (5 SP)
- ✅ **US-038: Gizmo Rotate** (4 SP)

**Deliverables:**
- ✅ Transform Gizmo completo
- ✅ WASM API: `toggle_gizmo()`, `set_gizmo_mode()`

---

### **Frontend Tasks** ❌

**Deliverables:**
- ❌ **TransformGizmo.tsx** - Pendiente
- ❌ **GizmoModeSelector.tsx** - Pendiente
- ❌ **MeasurementOverlay.tsx** - Pendiente

---

### **MILESTONE 2: Professional Features** 🚧 EN PROGRESO

**Features Completadas:**
- ✅ Conexiones magnéticas - Completado (Sprint 7-8)
- ✅ Elbow routing - Completado (Sprint 7-8)
- ✅ Transform Gizmo - Completado (Sprint 9)

**Próximos Pasos:**
1. ✅ Sprint 7-8: Conexiones magnéticas - COMPLETADO
2. ✅ Sprint 9: Transform Gizmos - COMPLETADO
3. Lanzar Milestone 2

---

## 🏃 **SPRINT 10: Advanced Features** ✅ COMPLETADO (Backend 100%)

### Sprint 10: Advanced Features ✅ COMPLETADO
- ✅ Containers
- ✅ Swimlanes
- ✅ Connection Points Visualization
- ✅ Edge Routing Styles

### Sprint 11: Smart Features ✅ COMPLETADO
- Auto-Alignment
- Smart Distribute
- Path Optimization

### Sprint 12: Polish & Testing ❌
- E2E Tests
- Accessibility
- Documentation

---

## 📊 Resumen de Progreso Total

```
SPRINTS COMPLETADOS: 3/6 (50%)
├── ✅ Sprint 1-2: Foundation & Selection
├── ✅ Sprint 3: Transform (Resize/Rotate)
├── ✅ Sprint 5-6: Navigation & Properties
├── 🔄 Sprint 3-4: Editing (75%)
├── ❌ Sprint 7-8: Connections (Pendiente)
└── ❌ Sprint 9-12: Advanced (Pendiente)

USER STORIES: 45 Total
├── ✅ Implementadas: 20 (44%)
├── 🔄 En Progreso: 4 (9%)
└── ❌ Pendientes: 21 (47%)

COMPONENTES UI: 87 Total
├── ✅ Implementados: ~55%
├── 🔄 En Progreso: ~10%
└── ❌ Pendientes: ~35%

MÉTRICAS DE RENDIMIENTO: ✅ SUPERADAS
├── Selection: < 0.1ms (objetivo: <5ms)
├── Memory: 12.5KB/100k (objetivo: 50MB)
├── Box Select: O(k) spatial hash
└── Drag: 60 FPS
```

---

**Última actualización:** 2026-02-05  
**Versión:** 1.1  
**Progreso General:** ~50% del roadmap completado

**Objetivo:** Containers, Swimlanes, y características avanzadas

### **Backend Tasks**

**Stories:**
- [ ] **US-039: Containers (Auto-Resize)** (5 SP)
  - [ ] Implementar `ContainerActuator`
  - [ ] Parent-child relationships
  - [ ] Implementar `AutoResizeActuator`
  - [ ] Padding: 20px alrededor de children
  - [ ] Drag into container (>50% overlap)

- [ ] **US-040: Swimlanes** (4 SP)
  - [ ] Implementar `SwimlaneActuator`
  - [ ] Vertical/Horizontal dividers
  - [ ] Implementar `LaneSnapActuator`
  - [ ] Resize lanes (drag dividers)

- [ ] **US-041: Connection Points Visualization** (3 SP)
  - [ ] Implementar `AnchorVisibilityActuator`
  - [ ] CTRL+hover muestra todos los anchors
  - [ ] Click en anchor inicia arrow

- [ ] **US-042: Edge Routing Styles** (3 SP)
  - [ ] Extend `LineStyle` enum
  - [ ] Implementar switching entre estilos
  - [ ] Re-draw on style change

**Deliverables:**
- ✅ Containers con auto-resize
- ✅ Swimlanes funcional
- ✅ 4 routing styles completos
- ✅ WASM API: `create_container()`, `create_swimlane()`

---

### **Frontend Tasks**

**Advanced UI:**
- [ ] **Modals Package** - Dialog system
  - [ ] ExportModal (PNG/SVG/PDF/JSON)
  - [ ] SettingsModal (preferences)
  - [ ] KeyboardShortcutsModal (reference)
  - [ ] ShareModal (collaboration placeholder)

- [ ] **Sidebar Enhancements:**
  - [ ] AssetsPanel (reusable components)
  - [ ] HistoryPanel (undo/redo visual)
  - [ ] SearchBar (layers/assets search)

**Deliverables:**
- ✅ Modals funcionales
- ✅ Sidebar completo

---

## 🏃 **SPRINT 11: Smart Features** (Weeks 21-22)

**Objetivo:** Auto-alignment, smart distribute, path optimization

### **Backend Tasks**

**Stories:**
- [ ] **US-043: Auto-Alignment Suggestions** (4 SP)
  - [ ] Implementar `AutoAlignSuggestionActuator`
  - [ ] 3 opciones:
    - Grid layout
    - Horizontal flow
    - Vertical stack
  - [ ] Clustering algorithm
  - [ ] Test: < 50ms

- [ ] **US-044: Smart Distribute** (4 SP)
  - [ ] Implementar `SmartDistributeActuator`
  - [ ] Equal spacing calculation
  - [ ] Horizontal/Vertical modes
  - [ ] Formula: spacing = (total - widths) / (n-1)

- [ ] **US-045: Connection Path Optimization** (4 SP)
  - [ ] Implementar `ConnectionOptimizeActuator`
  - [ ] Minimize crossing arrows
  - [ ] Force-directed layout (basic)
  - [ ] Test: < 500ms para 100 connections

**Deliverables:**
- ✅ Smart features completos
- ✅ WASM API: `suggest_alignment()`, `distribute()`, `optimize_connections()`
- ✅ Performance targets met

---

### **Frontend Tasks**

**Final Polish:**
- [ ] **Animations** - Framer Motion
  - [ ] Selection animations
  - [ ] Toolbar transitions
  - [ ] Modal entrance/exit
  - [ ] Toast notifications

- [ ] **Accessibility** - A11y compliance
  - [ ] ARIA labels
  - [ ] Keyboard navigation
  - [ ] Screen reader support
  - [ ] Focus management

- [ ] **Responsive Design**
  - [ ] Mobile layout (basic)
  - [ ] Tablet optimization
  - [ ] Collapsible panels

**Deliverables:**
- ✅ Smooth animations (60 FPS)
- ✅ A11y compliant
- ✅ Responsive layout

---

## 🏃 **SPRINT 12: Polish, Testing & Launch Prep** (Weeks 23-24)

**Objetivo:** Production-ready release

### **Tasks (All Teams)**

**Testing:**
- [ ] **E2E Tests** (Playwright)
  - [ ] Selection workflows
  - [ ] Transform workflows
  - [ ] Connection creation
  - [ ] Undo/Redo paths
  - [ ] Export functionality

- [ ] **Performance Tests**
  - [ ] Regression suite
  - [ ] Load testing (10k entities)
  - [ ] Memory leak detection
  - [ ] FPS monitoring

- [ ] **Accessibility Tests**
  - [ ] WCAG 2.1 AA compliance
  - [ ] Screen reader testing
  - [ ] Keyboard-only navigation

**Documentation:**
- [ ] User documentation
  - [ ] Getting started guide
  - [ ] Video tutorials
  - [ ] Keyboard shortcuts reference
  - [ ] FAQ

- [ ] Developer documentation
  - [ ] API reference (WASM)
  - [ ] Component library (Storybook)
  - [ ] Architecture diagrams
  - [ ] Contributing guide

**Bug Fixing:**
- [ ] Critical bugs (P0): 0
- [ ] High bugs (P1): < 5
- [ ] Medium bugs (P2): < 20

**Optimization:**
- [ ] Bundle size optimization
- [ ] Code splitting
- [ ] Asset optimization
- [ ] CDN setup

**Deliverables:**
- ✅ 95%+ test coverage
- ✅ 0 critical bugs
- ✅ Documentation completa
- ✅ Performance benchmarks passing

---

### **MILESTONE 3: Production Ready** 🚀 (Week 24)

**Features Completadas:**
- ✅ Todas las 45 User Stories
- ✅ 87 React components
- ✅ 40+ Sensors, 60+ Actuators
- ✅ Smart features (auto-align, distribute, optimize)
- ✅ Complete testing suite
- ✅ Full documentation

**Performance Metrics:**
- ✅ 60 FPS con 10k entities
- ✅ < 5ms interaction latency
- ✅ < 100MB memory usage
- ✅ A11y WCAG 2.1 AA compliant

**Launch Ready:** 🎉 **Production deployment approved**

---

## Dependencias Críticas

### **Cross-Team Dependencies**

```
┌──────────────────────────────────────────────────────────────┐
│                    DEPENDENCY GRAPH                          │
└──────────────────────────────────────────────────────────────┘

Sprint 1-2 (Foundation)
├── Backend: MouseClickSensor, SelectActuator
│   └─→ WASM API: select_entity()
│       └─→ Frontend: Canvas.tsx calls WASM
│
├── Backend: MouseDragSensor, MoveActuator
    └─→ WASM API: start_drag(), update_drag()
        └─→ Frontend: TransformHandles.tsx

Sprint 3-4 (Transform & Edit)
├── Backend: ResizeActuator, RotateActuator
│   └─→ WASM API: resize_entity(), rotate_entity()
│       └─→ Frontend: TransformHandles.tsx
│
├── Backend: CopyActuator, PasteActuator
    └─→ WASM API: copy(), paste(), undo(), redo()
        └─→ Frontend: ContextMenu.tsx, Keyboard shortcuts

Sprint 5-6 (Navigation)
├── Backend: PanCameraActuator, ZoomCameraActuator
│   └─→ WASM API: pan(), zoom()
│       └─→ Frontend: Canvas.tsx pointer events
│
├── Backend: SmartGuidesActuator
    └─→ WASM API: get_smart_guides()
        └─→ Frontend: SmartGuides.tsx overlay

Sprint 7-8 (Connections)
├── Backend: ArrowBindActuator, ElbowRoutingActuator
│   └─→ WASM API: bind_arrow(), recalculate_routing()
│       └─→ Frontend: AnchorPoints.tsx, ConnectionPreview.tsx
│
└── Backend: ConnectionStore with multi-anchors
    └─→ Frontend: ConnectionProperties.tsx

Sprint 9 (Gizmos)
└── Backend: TransformGizmo, GizmoMoveActuator
    └─→ WASM API: toggle_gizmo(), set_gizmo_mode()
        └─→ Frontend: TransformGizmo.tsx rendering
```

### **Weekly Sync Points**

**Every Monday:**
- API contract review
- Integration point validation
- Blocker resolution

**Every Thursday:**
- Demo of completed features
- Performance metrics review
- Sprint planning preview

---

## Métricas de Éxito

### **Performance KPIs**

| Métrica | Target | Critical | Sprint Check |
|---------|--------|----------|--------------|
| Selection latency | < 5ms | < 10ms | Sprint 1 |
| Drag latency | < 5ms | < 10ms | Sprint 2 |
| Resize/Rotate | 60 FPS | 30 FPS | Sprint 3 |
| Undo/Redo | < 10ms | < 20ms | Sprint 4 |
| Pan/Zoom latency | < 5ms | < 10ms | Sprint 5 |
| Anchor detection | < 5ms | < 10ms | Sprint 7 |
| Elbow routing | < 10ms | < 20ms | Sprint 8 |
| A* pathfinding | < 16ms | < 33ms | Sprint 8 |
| Gizmo interaction | < 5ms | < 10ms | Sprint 9 |

### **Quality KPIs**

| Métrica | Target | Sprint Check |
|---------|--------|--------------|
| Test coverage | > 90% | Sprint 12 |
| Critical bugs | 0 | Sprint 12 |
| High bugs | < 5 | Sprint 12 |
| A11y compliance | WCAG 2.1 AA | Sprint 11-12 |
| Bundle size | < 500KB gzip | Sprint 12 |

### **Feature Completion**

| Milestone | Features | Sprint |
|-----------|----------|--------|
| Milestone 1 | 16 User Stories | Sprint 6 (Week 12) |
| Milestone 2 | +11 User Stories | Sprint 9 (Week 18) |
| Milestone 3 | +18 User Stories | Sprint 12 (Week 24) |

---

## Risk Management

### **High Risks**

**1. WASM Performance Bottleneck**
- **Probability:** Medium
- **Impact:** High
- **Mitigation:** 
  - Early performance testing (Sprint 1)
  - SIMD optimization (Sprint 8)
  - Fallback to JS for non-critical paths

**2. A* Pathfinding Complexity**
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:**
  - Phase 1: Simple orthogonal (Sprint 8)
  - Phase 2: A* only if needed (Sprint 8)
  - Caching & pre-computation

**3. React Performance with 10k Entities**
- **Probability:** Low
- **Impact:** High
- **Mitigation:**
  - Virtualization (react-window)
  - Memoization (React.memo)
  - Debouncing updates

### **Medium Risks**

**4. Cross-browser Compatibility**
- **Probability:** Medium
- **Impact:** Medium
- **Mitigation:**
  - Early browser testing (Chrome, Firefox, Safari)
  - Polyfills for missing features
  - WebGL2 + WebGPU fallback

**5. Team Coordination**
- **Probability:** Low
- **Impact:** Medium
- **Mitigation:**
  - Weekly syncs (Monday + Thursday)
  - Clear API contracts
  - Integration checkpoints

---

## Resources

### **Development Tools**

**Backend:**
- Rust 1.75+
- wasm-pack
- wasm-bindgen
- criterion (benchmarking)

**Frontend:**
- React 18
- TypeScript 5
- Vite (bundler)
- Tailwind CSS
- Framer Motion
- Radix UI
- Zustand

**Testing:**
- Vitest (unit tests)
- Playwright (E2E tests)
- Storybook (component library)

**DevOps:**
- GitHub Actions (CI/CD)
- Vercel (staging/production)
- Sentry (error tracking)

---

## Conclusión

Este roadmap de **24 semanas** está diseñado para entregar un whiteboard profesional de clase mundial con características únicas que nos diferencian de la competencia:

✅ **Transform Gizmo 3D-style** (único en web)  
✅ **Elbow routing nativo en Rust** (10x más rápido)  
✅ **8 anchor points magnéticos** (vs 4 en competencia)  
✅ **A* pathfinding** con obstacle avoidance  
✅ **Smart features** (auto-align, distribute, optimize)

**Next Steps:**
1. ✅ Aprobar roadmap con stakeholders
2. ✅ Formar equipos (2 backend + 2 frontend)
3. ✅ Setup infraestructura (repos, CI/CD)
4. 🚀 **Sprint 1 Kick-off** (Week 1)

---

**Última actualización:** 2026-01-21  
**Versión:** 1.0  
**Aprobado por:** [Pending]  
**Sprint 1 Start Date:** [TBD]