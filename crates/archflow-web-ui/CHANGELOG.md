# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **P0-1**: WASM Bridge integration with `useEntityStore`
  - Real-time entity synchronization from WASM engine
  - Automatic fallback to mock data when WASM is not compiled
  - Methods: spawnEntity, deleteEntity, duplicateEntity, updateEntity, updateProperty, getEntity

- **P1-1**: `useTransformation` hook for entity transformations
  - Move, resize (8 directions), and rotate support
  - Grid snapping with 20px grid size
  - Centralized transformation logic

- **P1-2**: Toast notification system
  - `useToastStore` Zustand store for managing toasts
  - `ToastContainer` component with Framer Motion animations
  - `useToast` hook for easy toast usage (success, error, warning, info)
  - Auto-dismissal after 5 seconds (configurable)

- **P1-3**: Skeleton loading components
  - Base `Skeleton` component with variants (text, rect, circle)
  - Specialized skeletons: EntityCardSkeleton, PropertiesPanelSkeleton, ToolbarSkeleton, SidebarSkeleton
  - Support for pulse and shimmer animations

- **P1-4**: Entity schemas completion
  - AWS VPC schema with CIDR validation
  - AWS DynamoDB schema with partition/sort keys, billing mode, TTL, SSE
  - Added to `entitySchemas` registry

- **P2-2**: Error Boundary component
  - Class component with error catching and reporting
  - Fallback UI with error details display
  - Reset/try again functionality

- **P2-3**: Lazy loading support
  - `LazyComponents.tsx` with lazy-loaded DemoArchitecture
  - Suspense fallback with skeleton

### Changed
- **P0-2**: Updated dependencies
  - React: 18.3.1 → 19.0.0
  - React DOM: 18.3.1 → 19.0.0
  - Zustand: 4.5.0 → 5.0.0
  - Framer Motion: 11.0.0 (no change, keeping stable version)
  - Vite: 5.2.0 → 5.4.0
  - TypeScript: 5.5.0 → 5.6.0
  - Vitest: 1.4.0 → 2.0.0
  - @types/react: 18.3.0 → 19.0.0
  - @types/react-dom: 18.3.0 → 19.0.0
  - Lucide React: 0.344.0 → 0.400.0
  - tailwind-merge: 2.2.0 → 2.5.0

- **P0-3**: Removed duplicate PropertiesPanel
  - Deleted `src/components/PropertiesPanel.tsx` (simple version)
  - Updated `App.tsx` to use advanced version from `src/components/Properties/PropertiesPanel.tsx`
  - Removed `isPropertiesPanelOpen` state from App

- **P2-1**: Fixed path aliases
  - Added path aliases to `tsconfig.app.json` to match `vite.config.ts`
  - Aliases: `@components/*`, `@hooks/*`, `@utils/*`, `@types/*`, `@store/*`, `@archflow/web`

### Fixed
- **P1-5**: Property validators
  - Confirmed all validators are present (required, min, max, minLength, maxLength, pattern, email, url, custom)
  - All entity schemas have appropriate validation rules

## [0.0.0] - 2025-XX-XX

### Initial Release
- Basic scaffolding with Tailwind CSS v4
- Zustand stores (UI, Canvas, Selection)
- Framer Motion animations setup
- Core UI components (Canvas, Toolbar, Header, Sidebar, StatusBar)
- Drag & drop with @dnd-kit
- Keyboard shortcuts system
- Properties panel with form validation
- Connection store and renderer
- Demo C4 architecture
- Canvas 2D rendering (fallback from WebGPU)

---

**Note**: Version numbers follow [Semantic Versioning](https://semver.org/).
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes
