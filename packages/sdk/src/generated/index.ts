/**
 * Type definitions for ArchFlow SDK WASM bindings
 * Auto-generated from Rust source code
 *
 * This file is automatically updated during the Rust build process.
 * Do not edit manually - changes will be overwritten.
 */

// Core type wrappers (from archflow_core types)
export type { TsVec2 } from './TsVec2';
export type { TsColor } from './TsColor';
export type { TsEntityId } from './TsEntityId';
export type { TsRect } from './TsRect';
export type { TsTransform } from './TsTransform';

// WASM interop types (for JavaScript/TypeScript integration)
export type { JsVec2 } from './JsVec2';
export type { JsColor } from './JsColor';
export type { JsRect } from './JsRect';
export type { JsSelection } from './JsSelection';
export type { JsShape } from './JsShape';

// Re-export with simpler names for convenience
export type { TsVec2 as Vec2 } from './TsVec2';
export type { TsColor as Color } from './TsColor';
export type { TsEntityId as EntityId } from './TsEntityId';
export type { TsRect as Rect } from './TsRect';
export type { TsTransform as Transform } from './TsTransform';

// Re-export WASM types with Js prefix for clarity
export type { JsVec2 as WasmVec2 } from './JsVec2';
export type { JsColor as WasmColor } from './JsColor';
export type { JsRect as WasmRect } from './JsRect';
export type { JsSelection as WasmSelection } from './JsSelection';
export type { JsShape as WasmShape } from './JsShape';
