# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Completed
- **EPIC_WHITEBOARD_INTERACTIONS: 100% Complete**
  - All 47 actuators implemented and tested
  - All sensors consolidated into unified MouseSensor
  - 519 unit tests passing in archflow-logic
  - 1,176 tests passing workspace-wide

## [0.58.0] - 2026-02-14

### Added
- **feat(wasm-bridge): Fixed Timestep Physics (HU-PERF-001)**
  - Accumulator pattern for stable physics independent of frame rate
  - Configurable timestep (default 60 Hz)
  - Max substeps (default 8) to prevent "spiral of death"
  - New API: `set_fixed_timestep_hz()`, `set_fixed_timestep()`, `get_fixed_timestep()`, `set_max_substeps()`, `get_max_substeps()`, `get_accumulator()`
  - `WorldBounds` struct for boundary management
  - Physics now integrated automatically in `tick()` - no manual `integrate_physics()` call needed

### Changed
- Physics examples now use integrated fixed timestep instead of manual integration
- Performance example 07 updated for new physics API

## [0.45.0] - 2026-02-06

### Added
- **feat(actuators): LineStyleActuator for edge routing styles**
  - Support for Straight, Orthogonal, Bezier, and Elbow connection styles
  - Path calculation for each style with elbow routing actuator
  - Batch style operations for multiple connections
  - 10 new unit tests for comprehensive coverage

### Changed
- **BREAKING CHANGE: Unified MouseSensor API**
  - Consolidated 6 mouse sensors (double_tap, long_press, mouse_click, mouse_over, right_click) into single MouseSensor
  - Removed 1,420 lines of redundant sensor code
  - New MouseSensor handles all mouse interactions with state machine
  - Refactored signals.rs to use unified sensor output

### Refactor
- **ConnectionStyle moved to Shared Kernel (archflow-core)**
  - Avoids circular dependencies between archflow-logic and archflow-engine
  - Now available across all crates without import cycles
- **SetConnectionStyle command added to command.rs**
  - Supports undo/redo for connection style changes

## [0.38.0] - 2026-02-04

### Added
- **HU-RENDER-002**: Complete WebGL2Renderer implementation with instanced drawing
  - GLSL ES 3.0 shaders for SDF-based shape rendering
  - WebGL2Context for glow bindings management
  - Quad vertex buffer and shader storage buffer
  - Renderer trait implementation for backend compatibility
- **HU-RENDER-004**: Shader compilation with Naga
  - WGSL to GLSL compilation in build.rs
  - ShaderConfig struct for shader pipeline management
  - GLSL ES 3.0 output for WebGL2
- **HU-RENDER-005**: Performance benchmarks framework
  - render_bench.rs with criterion benchmarks
  - Benchmarks for 1k, 10k, 100k entity sync
  - Memory allocation and batch distribution benchmarks
- **HU-RENDER-006**: Feature parity tests
  - 14 comprehensive tests for render consistency
  - Tests for batch distribution, viewport culling, shape types
  - Tests for UV rect preservation, color packing, draw order

### Changed
- Updated shader constants to use consistent naming (SHADER_SDF_SHAPES instead of SHADER_SDF_SHAPES_GLSL)

### Fixed
- Fixed test visibility assertions for camera zoom viewport culling

### Expected
- Performance profiling and optimization analysis
- Bundle size reduction to <500KB gzipped target
- Cross-browser testing suite completion

## [0.1.0] - 2026-02-03

### Added
- Initial Web Whiteboard MVP
- **React 19** + TypeScript + Vite 7 setup
- **Tailwind CSS v4** integration with CSS-first configuration
- Zustand for state management
- Framer Motion 12 for animations
- @dnd-kit for drag & drop functionality
- WASM bridge integration with archflow-engine
- Canvas 2D rendering with optimized render loop
- Entity selection and manipulation
- Properties panel with live validation
- C4 Architecture demo component
- Toast notifications system
- Loading skeletons for better UX
- **Performance monitoring utilities** (`src/utils/performance.ts`)
  - FPS tracking and frame time measurement
  - Memory usage monitoring (Chrome/Edge)
  - Render time profiling with slow render detection
  - Interaction latency tracking
  - Development-only performance hooks
- **Cross-browser compatibility test suite** (`src/test/crossBrowser.test.ts`)
  - Modern JavaScript feature detection
  - Web API compatibility checks
  - Canvas 2D and Pointer Events validation
  - Performance benchmarks
- **Error boundary** with recovery UI
- **Lazy loading** for heavy demo components
- Keyboard shortcuts system
- Command history (undo/redo)
- Transformation system with handles
- Connection rendering between entities

### Changed
- Updated from Tailwind CSS v3 to v4 (CSS-first config)
- Refactored component architecture for better modularity
- **Optimized Canvas component** with `React.memo` to prevent unnecessary re-renders
- **Optimized Zustand store selectors** to only subscribe to needed state
- **Improved render loop** with specific dependency tracking instead of full camera object

### Performance
- **Implemented aggressive bundle optimization**:
  - Terser minification with console removal in production
  - Advanced code splitting with manual chunks for vendors
  - Separate chunks for React, animations, DnD, icons, forms, and utilities
  - Better caching strategy with granular vendor chunks
- **Reduced re-renders** in Canvas component through:
  - Memoized callbacks with `useCallback`
  - Specific Zustand selectors instead of full store subscription
  - React.memo wrapper for component
- **Added performance monitoring** for development:
  - FPS and frame time tracking
  - Slow render warnings (>16ms threshold)
  - Memory usage tracking
  - Interaction latency monitoring
- **Optimized Vite build configuration**:
  - Chunk size warning limit reduced to 200KB
  - Report compressed bundle size
  - Manual chunk splitting for better tree-shaking

### Security
- Added Content Security Policy headers (COOP/COEP) for SharedArrayBuffer support
- Cross-origin isolation enabled for WASM threading support

### Testing
- Added comprehensive cross-browser compatibility tests
- Performance benchmark tests for rendering speed
- WASM compatibility validation
- Touch and pointer event support verification

### Developer Experience
- Performance monitoring utilities for profiling
- Error boundaries with detailed error reporting
- Loading skeletons for better perceived performance
- Lazy loading for code splitting

### Documentation
- Updated CHANGELOG.md with comprehensive release notes
- Inline documentation for performance optimizations
- Architecture references in component headers

## Dependencies

### Added
- terser: ^5.36.0 (production minification)

### Current Core Dependencies
- react: ^19.0.0
- react-dom: ^19.0.0
- vite: ^5.4.0
- typescript: ^5.6.0
- tailwindcss: ^4.0.0
- framer-motion: ^11.0.0
- zustand: ^5.0.0
- @dnd-kit/core: ^6.1.0
- lucide-react: ^0.400.0
- react-hook-form: ^7.51.0
- zod: ^3.23.0

### Current Dev Dependencies
- @vitejs/plugin-react: ^4.3.0
- @tailwindcss/vite: ^4.0.0
- vitest: ^2.0.0
- @testing-library/react: ^16.0.0
- eslint: ^9.9.0
- terser: ^5.36.0

---

## Version Strategy

This project follows Semantic Versioning:
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

## Release Notes Format

Each release includes:
- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Performance**: Performance improvements
- **Security**: Security improvements
- **Testing**: Test additions or changes

## Upcoming Releases

### 0.2.0 (Planned)
- WebGPU rendering implementation
- Advanced connection routing (orthogonal, curved)
- Entity grouping and nesting
- Export to PNG/SVG/PDF
- Collaborative editing foundation

### 0.3.0 (Planned)
- Real-time collaboration
- Version history and branching
- Advanced shape library
- Plugin system foundation
- Cloud storage integration

---

**Last Updated**: 2026-02-03
**Maintained By**: Hodei ArchFlow Team
**License**: [Specify License Here]
