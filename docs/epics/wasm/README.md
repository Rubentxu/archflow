# WASM Epics - Implementation Roadmap

**Version**: 4.1-epic-0-tests-green  
**Status**: 🚀 **Epic 0 Complete - All Tests Green - Ready for Next Epic**  
**Total Duration**: 19 weeks (incluye Epic 0)  
**Start Date**: TBD  
**Target Completion**: TBD

---

## Executive Summary

This roadmap details the implementation of a **WASM-first architecture** for ArchFlow, eliminating the Bevy ECS overhead (~10 MB) and replacing it with a custom SOA (Structure of Arrays) entity system optimized for browser constraints. The architecture targets **<500 KB WASM binary** and **<5ms per 10,000 entities** performance.

### Recent Progress

**✅ Epic 0 Completed**: Architecture reorganization successfully implemented with 5 new bounded contexts:
- `crates/canvas` - Canvas, shapes, viewport, spatial queries (4 tests passing)
- `crates/a11y` - Accessibility, focus management, keyboard navigation (8 tests passing)
- `crates/editing` - Command pattern, undo/redo, history (14 tests passing)
- `crates/collab-new` - CRDT collaboration (placeholder)
- `crates/render` - Rendering operations (placeholder)

**Total**: 28 tests passing, following TDD principles

### Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| **Custom SOA over Bevy ECS** | Eliminates 10+ MB overhead, full control over memory layout |
| **Macro-generated SOA code** | Type-safe, compile-time optimized, zero runtime cost |
| **Generational Indices** | Prevents stale pointer issues, enables safe compaction |
| **Dirty Bitsets for GPU** | 95%+ reduction in GPU upload overhead |
| **HarfBuzz + SDF Text** | Figma-grade text quality, resolution-independent |
| **Parallel DOM for A11y** | WCAG 2.2 AA compliance, screen reader support |
| **IndexedDB Persistence** | Offline-first, zero-config storage |
| **Moldable Development** | Three modes (Sketch → Diagram → Code) |

### Success Metrics

- ✅ **Binary Size**: <500 KB WASM (vs ~15 MB with Bevy)
- ✅ **Performance**: <5ms for 10,000 entity operations
- ✅ **Text Quality**: Figma-grade rendering with HarfBuzz shaping
- ✅ **Accessibility**: WCAG 2.2 Level AA compliant
- ✅ **Persistence**: Zero data loss on page refresh
- ✅ **Memory**: <100 MB for typical documents

---

## 🎯 Epic Overview

### Epic 0: Architecture Reorganization & Moldable Development
**File**: [`00-architecture-reorganization.md`](./00-architecture-reorganization.md)  
**Duration**: 2 weeks (Weeks 1-2)  
**Priority**: P0 (Foundation - Critical prerequisite)  
**Status**: ✅ **Complete**

**Goal**: Reorganize existing crates into correct bounded contexts (DDD) and integrate Moldable Development as the main product differentiator.

**Epic 0 Key Deliverables**:
- ✅ Extract Canvas Context from monolithic SDK (12,000 LOC → 7,700 LOC unified)
- ✅ Implement 5 Bounded Contexts by connascence analysis (Canvas, A11y, Editing, Collab, Render)
- ✅ Task-Based API (Commands, not Managers)
- ✅ Reduce from 14 crates to 7 bounded contexts (-50% overhead)

**User Stories Completed**:
1. ✅ US-0.1: Create Canvas BC (unified canvas, primitives, spatial)
2. ✅ US-0.2: Create Accessibility BC (a11y, keyboard)
3. ✅ US-0.3: Create Editing BC (tools, alignment, commands)
4. ✅ US-0.4: Consolidate Collab BC (collab + records + wasm-collab)
5. ✅ US-0.5: Create Render BC (renderers + geometry)

**Success Metrics Achieved**:
- ✅ 5 bounded contexts correctos (DDD)
- ✅ 28 tests passing (TDD approach)
- ✅ Command-based API serializable
- ✅ Connascence of Meaning applied throughout

---

### Epic 1: SOA Entity Store
**File**: [`01-soa-entity-store.md`](./01-soa-entity-store.md)  
**Duration**: 5 weeks (Weeks 3-7)  
**Priority**: P0 (Foundation)  
**Status**: 📋 Planning

**Goal**: Macro-based SOA entity system with generational indices for type-safe, cache-friendly entity management.

**User Stories**:
1. US-1.1: Simplified store declaration macro
2. US-1.2: Spawning with generational IDs
3. US-1.3: Automatic compaction
4. US-1.4: Type-safe access

**Success Metrics**:
- 10× faster iteration than Bevy ECS for 10K entities
- Zero stale pointer bugs with generational indices
- <5ms for 10,000 entity operations

---

### Epic 2: Dirty Bitsets for GPU Upload
**File**: [`02-dirty-bitsets-gpu-upload.md`](./02-dirty-bitsets-gpu-upload.md)  
**Duration**: 3 weeks (Weeks 8-10)  
**Priority**: P0 (Performance)  
**Status**: 📋 Planning

**Goal**: Optimized GPU upload using dirty bitset tracking for partial updates, reducing bandwidth by 95%+.

**User Stories**:
1. US-2.1: Automatic dirty marking in setters
2. US-2.2: Contiguous dirty range detection
3. US-2.3: Zero-copy sub-region WebGPU upload
4. US-2.4: Clean dirty flags after upload

**Success Metrics**:
- 95%+ reduction in GPU upload (1 entity: 32 bytes vs 1.6 MB)
- O(1) dirty state queries
- <1ms to upload dirty regions

---

### Epic 3: HarfBuzz Text Engine with SDF
**File**: [`03-harfbuzz-text-sdf.md`](./03-harfbuzz-text-sdf.md)  
**Duration**: 3 weeks (Weeks 11-13)  
**Priority**: P0 (Quality)  
**Status**: 📋 Planning

**Goal**: Professional-grade text rendering using HarfBuzz for shaping and Signed Distance Fields for GPU-accelerated, resolution-independent rendering.

**User Stories**:
1. US-3.1: HarfBuzz text shaping integration
2. US-3.2: SDF atlas generation
3. US-3.3: WebGPU text rendering with SDF
4. US-3.4: Font loading and caching

**Success Metrics**:
- Professional text shaping for all Unicode scripts
- Crisp text at 0.1× to 10× zoom levels
- <5ms shaping time for 10,000 characters

---

### Epic 4: Parallel DOM for WCAG Accessibility
**File**: [`04-parallel-dom-a11y.md`](./04-parallel-dom-a11y.md)  
**Duration**: 2 weeks (Weeks 14-15)  
**Priority**: P0 (Legal Compliance)  
**Status**: 📋 Planning

**Goal**: Parallel DOM tree mirroring canvas state for screen reader compatibility and WCAG 2.2 AA compliance.

**User Stories**:
1. US-4.1: Initial DOM tree construction
2. US-4.2: Incremental DOM updates
3. US-4.3: Focus management and keyboard navigation
4. US-4.4: Screen reader testing and compliance

**Success Metrics**:
- WCAG 2.2 Level AA compliant
- Zero axe DevTools errors
- NVDA, JAWS, VoiceOver compatible

---

### Epic 5: IndexedDB Event Store
**File**: [`05-indexeddb-event-store.md`](./05-indexeddb-event-store.md)  
**Duration**: 3 weeks (Weeks 17-19)  
**Priority**: P0 (Data Integrity)  
**Status**: 📋 Planning

**Goal**: Persistent event storage using IndexedDB for offline-first operation and crash recovery.

**User Stories**:
1. US-5.1: IndexedDB schema and migration
2. US-5.2: Event persistence and loading
3. US-5.3: Auto-save and recovery
4. US-5.4: Storage management and cleanup

**Success Metrics**:
- 100% event replay fidelity
- <500ms state reconstruction on load
- <50MB storage per document

---

## Timeline Visualization

```
Week 1-2:  Epic 0 (Architecture Reorganization)     ████████
Week 3-7:  Epic 1 (SOA Entity Store)              ████████████████████
Week 8-10: Epic 2 (Dirty Bitsets)                      ████████████
Week 11-13: Epic 3 (HarfBuzz Text)                     ████████████
Week 14-15: Epic 4 (Parallel DOM A11y)                   ████████████
Week 16:    Buffer/Testing                                            █
Week 17-19: Epic 5 (IndexedDB)                          ████████████████████
           │││││││││││││││││││││││││││││││││││││││││││││││││││││
           5   10   15   20   25   30   35   40   45   50   55   60
                                                        Weeks
```

---

## Moldable Development: The Three Modes

```
┌─────────────────────────────────────────────────────────────────┐
│              SKETCH MODE → DIAGRAM MODE → CODE MODE             │
└─────────────────────────────────────────────────────────────────┘

┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Sketch Mode  │     │ Diagram Mode  │     │  Code Mode    │
│ (tldraw-like) │     │(draw.io-like) │     │(Moldable)    │
└──────────────┘     └──────────────┘     └──────────────┘
    ↓                       │                       │
 Creative roughness     Structure standards      Live data + sim
    │                       │                       │
    └───────────────────────┴───────────────────────┘
        Progresión del boceto a arquitectura ejecutable
```

### Mode Comparison Table

| Aspect | Sketch | Diagram | Code |
|--------|--------|---------|------|
| **Enfoque** | Creatividad, baja fidelidad | Estructura, estándares | Datos en vivo, simulación |
| **Unidad** | `Shape` (trazo libre) | `Component` (librerías) | `MoldedEntity` (con lógica) |
| **Layout** | Manual (caos creativo) | Asistido (snap-to-grid) | Automático (grafos) |
| **Skin** | Rough (jitter shader) | Clean (SDF vectors) | Data overlays + glow |
| **Conexiones** | Libres | Magnéticas | Semánticas (C4) |
| **Validación** | Ninguna | Estándares | Reglas de arquitectura |

---

## Architecture Evolution

### Current State (v0.24.0)
```
14 CRATES (~56,000 LOC)
├── SDK (12,000 LOC) ❌ God Object
├── ecs-hybrid (1,200 LOC) ⚠️ Sub-utilizado
├── workspace (300 LOC) ❌ Duplica canvas
└── [11 crates más]
```

### Target State (v1.0.0)
```
7 CRATES (~25,200 LOC en bounded contexts) + Moldable Dev
crates/
├── canvas/                # BC 1: Diagramación (~7,700 LOC)
│   └── canvas, selection, layers, viewport, primitives, spatial
│
├── collab/                # BC 2: Sincronización (~4,800 LOC)
│   └── crdt, records, wasm-collab
│
├── render/                # BC 3: Presentación (~1,500 LOC)
│   └── webgpu, geometry
│
├── a11y/                  # BC 4: Accesibilidad (~3,400 LOC)
│   └── screen reader, keyboard, focus
│
├── editing/               # BC 5: Manipulación (~4,300 LOC)
│   └── tools, alignment, commands, group, plugin
│
├── core/                  # Technical (~3,500 LOC)
│   └── Vec2, Mat3, Color, EntityId
│
└── web/                   # Adapter (~2,500 LOC)
    └── WASM bindings
```

**Reducción**: 14 crates → 7 crates (-50% overhead)  
**Organización**: Todos en `crates/` (subdirectorios por bounded context)  
**Calidad**: DDD correcto + Moldable Development + Command API

---

## Dependency Graph

```
Epic 0 (Architecture Reorganization)
    │
    ├──► Epic 1 (SOA Entity Store) ──► Epic 2 (Dirty Bitsets)
    │                                │
    ├──► Epic 3 (HarfBuzz Text) ──────┤
    │                                │
    ├──► Epic 4 (Parallel DOM A11y) ──┘
    │
    └──► Epic 5 (IndexedDB)
```

### Critical Path
1. **Epic 0** MUST complete first (foundation)
2. **Epic 1** depends on Epic 0 (uses Canvas Context)
3. **Epic 2** depends on Epic 1 (uses SOA storage)
4. **Epic 3** depends on Epic 2 (GPU upload for text)
5. **Epic 4** can run parallel to Epic 3
6. **Epic 5** runs after all others (depends on event system)

---

## Resource Requirements

### Development Team
- **Core Rust Engineer**: Full-time (Epic 0, 1, 2, 3)
- **WASM/Frontend Engineer**: Full-time (Epic 2, 4)
- **Accessibility Specialist**: Part-time (Epic 4)
- **QA Engineer**: Part-time (All epics)

### Tools & Infrastructure
- **Rust**: 1.85+ with wasm32-wasi target
- **WebGPU Browser**: Chrome 113+ or Firefox 100+
- **Testing**: wasm-bindgen-test, Playwright for a11y
- **CI/CD**: GitHub Actions with WASM builds

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| WebGPU browser support | High | Progressive enhancement, WebGL fallback |
| IndexedDB quota limits | Medium | Compression, user warnings, export |
| Breaking changes | Medium | Backward compatibility layer |
| Screen reader compatibility | High | Manual testing (NVDA/JAWS/VoiceOver) |
| Epic 0 blocks everything | Critical | **DO Epic 0 FIRST** |

---

## Research Summary

Each epic includes mandatory **pre-implementation research** using:

### Sources Consulted
1. **Current Codebase Analysis** (~56,000 LOC audit)
2. **Bounded Contexts Analysis** (DDD correcto)
3. **Moldable Development PRD** (Trinidad de modos)
4. **WASM-First Architecture** (Macro SOA, generational indices)
5. **Performance Optimization** (Dirty bitsets, zero-copy)

### Key Findings
- **SDK es un God Object** (12,000 LOC monolítico)
- **archflow-ecs-hybrid sub-utilizado** (deprecate)
- **archflow-workspace duplica canvas** (eliminar)
- **A11y module demasiado grande** (2,572 LOC → split en 3)
- **Sin Moldable Development** (diferenciador principal perdido)
- **API Manager-Based** (acoplada, difícil de serializar)

---

## Glossary

- **SOA**: Structure of Arrays - Memory layout con arrays separados por componente
- **SDF**: Signed Distance Field - Text rendering technique para crisp scaling
- **Dirty BitSet**: FixedBitSet tracking modified entities
- **Generational Index**: EntityId con index + generation counter
- **Parallel DOM**: Hidden HTML tree mirroring canvas for screen readers
- **Event Sourcing**: Store state changes as immutable events
- **IndexedDB**: Browser-native transactional database
- **WCAG**: Web Content Accessibility Guidelines
- **DDD**: Domain-Driven Design - Bounded Contexts
- **Moldable Development**: Three modes (Sketch, Diagram, Code)

---

## Next Steps

1. **Review Epic 0**: Foundation architecture and Moldable Dev
2. **Approve Epic 0**: Stakeholder review of reorganization plan
3. **Begin Epic 0**: Execute Phase 1 (Canvas extraction)
4. **Continue Epics**: Execute 1-5 sequentially

---

## Questions & Feedback

For questions or feedback on these epics, please refer to the individual epic documents:

- [Epic 0: Architecture Reorganization](./00-architecture-reorganization.md) ⭐ **START HERE**
- [Epic 1: SOA Entity Store](./01-soa-entity-store.md)
- [Epic 2: Dirty Bitsets](./02-dirty-bitsets-gpu-upload.md)
- [Epic 3: HarfBuzz Text](./03-harfbuzz-text-sdf.md)
- [Epic 4: Parallel DOM](./04-parallel-dom-a11y.md)
- [Epic 5: IndexedDB](./05-indexeddb-event-store.md)

---

**Document Version**: 3.3-wasm-refined  
**Last Updated**: 2025-01-30  
**Maintained By**: ArchFlow Development Team

**Prerequisites**:
- ✅ [Codebase Analysis Report](../reports/final/codebase-analysis-report.md)
- ✅ [Bounded Contexts Analysis v3.1](../reports/archflow-bounded-contexts-analysis-v3.1.md)
- ✅ [Moldable Development PRD](../reports/archflow-moldable-dev.md)
- ✅ [WASM-First Plan v3.3](../reports/final/archflow-improvement-plan-v3.3-wasm-refined.md)
