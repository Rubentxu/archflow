// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Engine - TypeScript Definitions
//
// Auto-generated TypeScript definitions for the ArchFlow WASM ECS Engine.
// Compatible with EPIC-WASM-100, 101, 102, 103
//
// Usage:
//   import ArchFlow, { Transform, Shape, Color } from './archflow.d.ts';
// ═══════════════════════════════════════════════════════════════════════════════════════

// ============================================================================
// Component Factories
// ============================================================================

/** Transform component - position and size */
export interface Transform {
  at(x: number, y: number): Transform;
  withSize(w: number, h: number): Transform;
  component_type(): string;
}

/** Shape component - visual shape */
export interface Shape {
  circle(): Shape;
  rectangle(): Shape;
  ellipse(): Shape;
  triangle(): Shape;
  diamond(): Shape;
  component_type(): string;
}

/** Color component - fill and stroke */
export interface Color {
  rgb(r: number, g: number, b: number): Color;
  rgba(r: number, g: number, b: number, a: number): Color;
  fromHex(hex: string): Color;
  component_type(): string;
}

/** Visibility component */
export interface Visibility {
  visible(): Visibility;
  hidden(): Visibility;
  component_type(): string;
}

/** Velocity component - movement */
export interface Velocity {
  new(vx: number, vy: number): Velocity;
  zero(): Velocity;
  component_type(): string;
}

/** ZOrder component - render order */
export interface ZOrder {
  front(): ZOrder;
  back(): ZOrder;
  layer(n: number): ZOrder;
  component_type(): string;
}

/** RenderLayer component */
export interface RenderLayer {
  new(n: number): RenderLayer;
  ui(): RenderLayer;
  background(): RenderLayer;
  component_type(): string;
}

/** PixelGridSnap component */
export interface PixelGridSnap {
  enabled(): PixelGridSnap;
  disabled(): PixelGridSnap;
  component_type(): string;
}

// ============================================================================
// Factory Functions
// ============================================================================

export const Transform: {
  at(x: number, y: number): Transform;
  rect(x: number, y: number, w: number, h: number): Transform;
};

export const Shape: {
  circle(): Shape;
  rectangle(): Shape;
  ellipse(): Shape;
  triangle(): Shape;
  diamond(): Shape;
};

export const Color: {
  rgb(r: number, g: number, b: number): Color;
  rgba(r: number, g: number, b: number, a: number): Color;
  fromHex(hex: string): Color;
};

export const Visibility: {
  visible(): Visibility;
  hidden(): Visibility;
};

export const Velocity: {
  new(vx: number, vy: number): Velocity;
  zero(): Velocity;
};

export const ZOrder: {
  front(): ZOrder;
  back(): ZOrder;
  layer(n: number): ZOrder;
};

export const RenderLayer: {
  new(n: number): RenderLayer;
  ui(): RenderLayer;
  background(): RenderLayer;
};

export const PixelGridSnap: {
  enabled(): PixelGridSnap;
  disabled(): PixelGridSnap;
};

// ============================================================================
// Shape Types
// ============================================================================

export const ShapeTypes: {
  rectangle(): number;
  circle(): number;
  ellipse(): number;
  triangle(): number;
  diamond(): number;
  cylinder(): number;
  line(): number;
  arc(): number;
};

// ============================================================================
// Logic Bricks - Sensors
// ============================================================================

export const SensorTypes: {
  mouse_over(): number;
  mouse_click(): number;
  right_click(): number;
  key_shortcut(): number;
  proximity(): number;
  radar(): number;
  touch(): number;
  ray(): number;
};

// ============================================================================
// Logic Bricks - Controllers
// ============================================================================

export const ControllerTypes: {
  direct(): number;
  and(): number;
  or(): number;
  not(): number;
  blinky(): number;
  debounce(): number;
  hysteresis(): number;
  threshold(): number;
};

// ============================================================================
// Logic Bricks - Actuators
// ============================================================================

export const ActuatorTypes: {
  highlight(): number;
  select(): number;
  move_actuator(): number;
  delete(): number;
  undo(): number;
  redo(): number;
  camera(): number;
  property(): number;
  animation(): number;
};

// ============================================================================
// Entity Builder
// ============================================================================

/** Fluent entity builder */
export interface JsEntityBuilder {
  insert(component: Transform | Shape | Color | Visibility | Velocity | ZOrder | RenderLayer | PixelGridSnap): JsEntityBuilder;
  position(x: number, y: number): JsEntityBuilder;
  size(width: number, height: number): JsEntityBuilder;
  color_rgb(r: number, g: number, b: number): JsEntityBuilder;
  layer(layer: number): JsEntityBuilder;
  visible(isVisible: boolean): JsEntityBuilder;
  stroke(r: number, g: number, b: number): JsBehaviorBuilder;
  stroke_width(width: number): JsEntityBuilder;
  
  // Behavior blocks
  behavior(name: string): JsEntityBuilder;
  sensor(sensorType: number, keyCode?: number): JsEntityBuilder;
  controller(controllerType: number, param?: number): JsEntityBuilder;
  actuator(actuatorType: number, x?: number, y?: number): JsEntityBuilder;
  end_behavior(): JsEntityBuilder;
  
  // Build
  build(): Promise<number>;
}

/** Behavior block builder */
export interface JsBehaviorBuilder {
  sensor(sensorType: number, keyCode?: number): JsBehaviorBuilder;
  controller(controllerType: number, param?: number): JsBehaviorBuilder;
  actuator(actuatorType: number, x?: number, y?: number): JsBehaviorBuilder;
}

// ============================================================================
// World
// ============================================================================

/** World interface */
export interface JsWorld {
  spawn(): JsEntityBuilder;
}

// ============================================================================
// Engine
// ============================================================================

/** Main engine interface */
export interface ArchFlowEngine {
  world: JsWorld;
  
  // Initialization
  initialize(canvasWidth: number, canvasHeight: number): Promise<void>;
  
  // Main loop
  tick(timestamp: number): void;
  
  // Entity operations
  spawn_entity(x: number, y: number, width: number, height: number): number;
  despawn_entity(entityId: number): boolean;
  
  // Selection
  get_selected_entities(): number[];
  clear_selection(): void;
  
  // Camera
  set_camera_position(x: number, y: number): void;
  set_zoom(zoom: number): void;
  get_zoom(): number;
  
  // Colors
  set_active_color(r: number, g: number, b: number, a: number): void;
  set_active_stroke_color(r: number, g: number, b: number, a: number): void;
  set_active_stroke_width(width: number): void;
  
  // History
  undo(): boolean;
  redo(): boolean;
  can_undo(): boolean;
  can_redo(): boolean;
}

// ============================================================================
// Facades (EPIC-WASM-103)
// ============================================================================

export interface WasmEntityFacade {
  entity_count(): number;
  is_entity_alive(entityId: number): boolean;
}

export interface WasmComponentFacade {
  get_position(entityId: number): [number, number];
  set_position(entityId: number, x: number, y: number): boolean;
  get_size(entityId: number): [number, number];
  set_size(entityId: number, width: number, height: number): boolean;
  get_color(entityId: number): number;
  set_color(entityId: number, color: number): boolean;
  get_visibility(entityId: number): boolean;
  set_visibility(entityId: number, visible: boolean): boolean;
}

export interface WasmBehaviorFacade {
  add_sensor(entityId: number, sensorType: number, controllerType: number, actuatorType: number): boolean;
  behavior_count(): number;
}

export interface WasmInputFacade {
  on_mouse_click(x: number, y: number, button: number): boolean;
  on_mouse_hover(x: number, y: number): boolean;
  on_key_down(keyCode: number): boolean;
  get_pointer_position(): [number, number];
}

export interface WasmRenderFacade {
  set_camera_position(x: number, y: number): boolean;
  set_zoom(zoom: number): boolean;
  get_zoom(): number;
  apply_effect(effectName: string): boolean;
}

// ============================================================================
// Factory
// ============================================================================

/** Create a new ArchFlow engine instance */
export function createEngine(): ArchFlowEngine;
