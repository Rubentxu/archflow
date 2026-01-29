/**
 * ArchFlow SDK TypeScript Type Definitions
 *
 * This package contains TypeScript type definitions for the ArchFlow SDK.
 * These types are automatically generated from Rust using ts-rs.
 *
 * @packageDocumentation
 */

// Core type wrappers (from archflow_core types)
export { TsVec2 } from './TsVec2';
export { TsColor } from './TsColor';
export { TsEntityId } from './TsEntityId';
export { TsRect } from './TsRect';
export { TsTransform } from './TsTransform';

// WASM interop types (for JavaScript/TypeScript integration)
export { JsVec2 } from './JsVec2';
export { JsColor } from './JsColor';
export { JsRect } from './JsRect';
export { JsSelection } from './JsSelection';
export { JsShape } from './JsShape';

// Re-export with simpler names for convenience
export { TsVec2 as Vec2 } from './TsVec2';
export { TsColor as Color } from './TsColor';
export { TsEntityId as EntityId } from './TsEntityId';
export { TsRect as Rect } from './TsRect';
export { TsTransform as Transform } from './TsTransform';
