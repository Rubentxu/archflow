# ArchFlow SDK TypeScript Types

This package contains TypeScript type definitions automatically generated from the Rust ArchFlow SDK using [ts-rs](https://github.com/Aleph-Alpha/ts-rs).

## Overview

The types in this package provide TypeScript definitions for the core data structures used in the ArchFlow SDK:

### Core Type Wrappers (Ts*)
- **TsVec2** - 2D vector with x and y coordinates
- **TsColor** - RGBA color representation
- **TsEntityId** - Entity identifier wrapper
- **TsRect** - Rectangle with min/max corners
- **TsTransform** - 2D transformation (translation, rotation, scale)

### WASM Interop Types (Js*)
- **JsVec2** - JavaScript interop version of Vec2
- **JsColor** - JavaScript interop version of Color
- **JsRect** - JavaScript interop rectangle
- **JsSelection** - Selection state for JavaScript
- **JsShape** - Shape data for JavaScript

## Usage

```typescript
import { TsVec2, TsColor, TsEntityId, JsShape } from '@archflow/sdk-types';

// Core type wrappers
const position: TsVec2 = { x: 100, y: 200 };
const fillColor: TsColor = { r: 0.5, g: 0.6, b: 0.7, a: 1.0 };
const shapeId: TsEntityId = { id: "550e8400-e29b-41d4-a716-446655440000" };

// WASM interop types
const shape: JsShape = {
  id: "shape-1",
  shapeType: "rectangle",
  x: 100,
  y: 200,
  width: 300,
  height: 200,
  rotation: 0,
  fillColor: { r: 0.2, g: 0.4, b: 0.8, a: 1.0 },
  strokeColor: null,
  strokeWidth: 0,
  opacity: 1.0,
  layerId: "layer-1",
  selected: false
};
```

## Regenerating Types

### Automatic (Recommended)

The types are automatically regenerated when you build the Rust crate:

```bash
cargo build --package archflow-sdk --features wasm
```

The build script (`crates/archflow-sdk/build.rs`) will automatically:
1. Detect the generated TypeScript files in `crates/archflow-sdk/bindings/`
2. Copy them to `packages/archflow-sdk-types/src/generated/`
3. Copy them to `packages/sdk/src/generated/`

### Manual

If you need to regenerate manually:

```bash
# Run tests to generate bindings
cargo test --package archflow-sdk --features wasm

# Or use the helper script
./scripts/generate-types.sh
```

## Architecture

### Automatic Build Integration

The build process is fully automated via `crates/archflow-sdk/build.rs`:

```
Rust Build
    ↓
Generate bindings/ (via ts-rs during tests)
    ↓
Build script copies files to:
    ├── packages/archflow-sdk-types/src/generated/
    └── packages/sdk/src/generated/
```

### Type Categories

1. **Core Wrappers (Ts*)**: Bridge types for `archflow_core` types
2. **WASM Interop (Js*)**: Types used in WASM FFI layer

### Why Wrapper Types?

Core Rust types (`Vec2`, `Color`, `EntityId` from `archflow_core`) don't implement the `TS` trait from ts-rs. We created wrapper types that:

1. Implement `TS` for automatic TypeScript generation
2. Provide `From`/`Into` conversions to/from the core types
3. Maintain compatibility with the WASM bindings

### Configuration

The ts-rs configuration is in `crates/archflow-sdk/Cargo.toml`:

```toml
[package.metadata.ts-rs]
compatibility = "es2020"
export = true
output_dir = "bindings"
export_to = "bindings"
```

The build script handles copying to the packages directory.

## Development

### Adding New Types

To add new TypeScript-exported types:

1. Create a wrapper type in `crates/archflow-sdk/src/ts_export.rs` (for core types) or
   `crates/archflow-sdk/src/wasm/*.rs` (for WASM interop types)
2. Derive `TS` and add `#[ts(export)]` attribute
3. Implement `From`/`Into` conversions for the core type
4. Run `cargo build --package archflow-sdk --features wasm`
5. The build script will automatically copy the new types

### File Structure

```
crates/archflow-sdk/
├── build.rs                    # Build script that copies files
├── Cargo.toml                  # ts-rs configuration
├── src/
│   ├── ts_export.rs           # Core wrapper types (Ts*)
│   └── wasm/
│       ├── mod.rs             # WASM interop types (Js*)
│       ├── keyboard.rs
│       ├── group.rs
│       └── ...
└── bindings/                   # Generated files (gitignored)
    ├── TsVec2.ts
    ├── TsColor.ts
    ├── JsShape.ts
    └── ...

packages/
├── archflow-sdk-types/
│   └── src/generated/         # Copied from bindings/
└── sdk/
    └── src/generated/         # Copied from bindings/
```

## License

This package is part of the ArchFlow project and follows the same license terms.
