/* tslint:disable */
/* eslint-disable */

/**
 * Actuator types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { ActuatorType } from '@archflow/sdk';
 *
 * const actuator = ActuatorType.Highlight;
 * ```
 */
export enum ActuatorType {
    /**
     * Highlight actuator - changes entity color
     */
    Highlight = 0,
    /**
     * Select actuator - marks entity as selected
     */
    Select = 1,
    /**
     * Move actuator - moves entity (drag operation)
     */
    Move = 2,
}

/**
 * Constants for actuator types (matching ActuatorType enum)
 */
export class ActuatorTypes {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    static animation(): number;
    static camera(): number;
    static delete(): number;
    static highlight(): number;
    static move_actuator(): number;
    static property(): number;
    static redo(): number;
    static select(): number;
    static undo(): number;
}

/**
 * Single animation sequence (clip)
 *
 * Represents a named animation sequence like "idle", "walk", "run".
 */
export class AnimationClip {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get end frame
     */
    end_frame(): number;
    /**
     * Get FPS
     */
    fps(): number;
    /**
     * Get frame count
     */
    frame_count(): number;
    /**
     * Get frame duration in ms
     */
    frame_duration_ms(): number;
    /**
     * Get if loops
     */
    loop_clip(): boolean;
    /**
     * Get clip name
     */
    name(): string;
    /**
     * Create a new animation clip
     *
     * # Arguments
     * * `name` - Name of the clip (e.g., "idle", "walk")
     * * `start_frame` - Starting frame index
     * * `end_frame` - Ending frame index (inclusive)
     * * `fps` - Frames per second
     * * `loop` - Whether the clip loops
     */
    constructor(name: string, start_frame: number, end_frame: number, fps: number, loop_clip: boolean);
    /**
     * Get start frame
     */
    start_frame(): number;
}

/**
 * Animation Component for sprite animation
 *
 * Use with JsEntityBuilder:
 * ```javascript
 * bridge.world.spawn()
 *     .insert(AnimationComponent.new(8, 100)) // 8 frames, 100ms each
 *     .build();
 * ```
 */
export class AnimationComponent {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get number of clips
     */
    clip_count(): number;
    /**
     * Get current frame index
     */
    current(): number;
    /**
     * Get current clip name
     */
    current_clip_name(): string | undefined;
    /**
     * Get frame count
     */
    frame_count(): number;
    /**
     * Get frame duration in ms
     */
    frame_duration_ms(): number;
    /**
     * Check if playing
     */
    is_playing(): boolean;
    /**
     * Check if loops
     */
    loop_animation(): boolean;
    /**
     * Create a new animation component with single sequence
     *
     * # Arguments
     * * `frame_count` - Total number of frames
     * * `frame_duration_ms` - Duration of each frame in milliseconds
     */
    constructor(frame_count: number, frame_duration_ms: number);
    /**
     * Create with looping disabled (single-shot)
     */
    static new_single_shot(frame_count: number, frame_duration_ms: number): AnimationComponent;
    /**
     * Pause playback
     */
    pause(): void;
    /**
     * Start playing
     */
    play(): void;
    /**
     * Play a specific clip by name
     */
    play_clip(name: string): boolean;
    /**
     * Play a specific clip by index
     */
    play_clip_by_index(index: number): boolean;
    /**
     * Reset to first frame
     */
    reset(): void;
    /**
     * Set a specific frame
     */
    set_frame(frame: number): void;
    /**
     * Update animation (call each frame)
     * Returns Some(new_frame) if frame changed, None otherwise
     */
    tick(delta_ms: number): number | undefined;
    /**
     * Create with multiple animation clips (JSON string format for WASM)
     * Format: [{"name":"idle","start":0,"end":3,"fps":8,"loop":true},...]
     */
    static with_clips_json(clips_json: string): AnimationComponent;
}

/**
 * Blend modes for material rendering
 */
export enum BlendMode {
    /**
     * No blending - opaque
     */
    Opaque = 0,
    /**
     * Alpha blending - standard transparency
     */
    AlphaBlend = 1,
    /**
     * Additive blending - glow effect
     */
    Add = 2,
    /**
     * Multiply blend - darkening
     */
    Multiply = 3,
}

/**
 * Brick Chain Builder - API Fluida implementada en Rust/WASM
 *
 * # JavaScript Example
 * ```javascript
 * const handle = bridge
 *   .sensor(Sensor.Mouse.Click('Left'))
 *   .controller(Controller.And())
 *   .actuator(Actuator.Select.Single())
 *   .connect();
 * ```
 */
export class BrickChainBuilder {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the number of actuators added
     */
    actuator_count(): number;
    /**
     * Add a Highlight actuator
     *
     * # Arguments
     * * `color_argb` - Color in ARGB format
     * * `opacity` - Opacity from 0.0 to 1.0
     */
    actuator_highlight(color_argb: number, opacity: number): BrickChainBuilder;
    /**
     * Add a Move actuator
     *
     * # Arguments
     * * `mode` - Movement mode
     * * `x` - X velocity or position
     * * `y` - Y velocity or position
     */
    actuator_move(mode: number, x: number, y: number): BrickChainBuilder;
    /**
     * Add a Select actuator
     *
     * # Arguments
     * * `mode` - 0=Single, 1=Multi, 2=Replace
     */
    actuator_select(mode: number): BrickChainBuilder;
    /**
     * Add an AND controller (shortcut)
     */
    and(): BrickChainBuilder;
    /**
     * Connect the chain to the entity
     */
    connect(): BrickHandle;
    /**
     * Add a controller to the brick chain
     */
    controller(controller: Controller): BrickChainBuilder;
    /**
     * Get the number of controllers added
     */
    controller_count(): number;
    /**
     * Get the entity ID
     */
    entity_id(): number;
    /**
     * Add a NOT controller (invert signal)
     */
    invert(): BrickChainBuilder;
    /**
     * Set the name for this logic block (for debugging)
     */
    logic_bricks(name: string): BrickChainBuilder;
    /**
     * Get the name of this logic block
     */
    name(): string | undefined;
    /**
     * Creates a new BrickChainBuilder for an entity
     */
    constructor(entity_id: number);
    /**
     * Start a new chain (clears current sensors/controllers, keeps entity_id)
     */
    new_chain(): BrickChainBuilder;
    /**
     * Add an OR controller (shortcut)
     */
    or(): BrickChainBuilder;
    /**
     * Add a sensor to the brick chain
     */
    sensor(sensor: SensorType): BrickChainBuilder;
    /**
     * Get the number of sensors added
     */
    sensor_count(): number;
    /**
     * Add a keyboard key sensor (convenience)
     */
    sensor_key(_key_code: number): BrickChainBuilder;
    /**
     * Creates a new BrickChainBuilder with a mapping table
     */
    static with_mapping_table(entity_id: number, mapping_table: LogicMappingTableWasm): BrickChainBuilder;
}

/**
 * Handle to a registered brick chain
 *
 * Returned by BrickChainBuilder.connect() for runtime control.
 *
 * # JavaScript Example
 * ```javascript
 * const handle = builder.connect();
 * handle.disable();
 * handle.remove();
 * ```
 */
export class BrickHandle {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Disable the brick chain
     */
    disable(): void;
    /**
     * Enable the brick chain
     */
    enable(): void;
    /**
     * Get the brick chain ID
     */
    id(): string;
    /**
     * Check if enabled
     */
    is_enabled(): boolean;
    /**
     * Creates a new BrickHandle with the given ID
     */
    constructor(id: string);
    /**
     * Remove the brick chain
     */
    remove(): void;
    /**
     * Toggle enabled state
     */
    toggle(): boolean;
}

/**
 * Unique callback identifier
 */
export class CallbackId {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly value: number;
}

export class CallbackRegistry {
    free(): void;
    [Symbol.dispose](): void;
    clear(): void;
    event_callback_count(event_type: string): number;
    invoke(event_type: string, data: any): number;
    constructor();
    register(callback: Function, event_type: string, is_oneshot: boolean): CallbackId;
    unregister(id: CallbackId): boolean;
    unregister_all(event_type: string): number;
    readonly total_count: number;
}

/**
 * Configuration for camera actuator
 */
export class CameraConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get duration in milliseconds
     */
    duration_ms(): number;
    /**
     * Creates a new camera configuration
     */
    constructor(target_x: number, target_y: number, zoom: number, duration_ms: number, smooth: number);
    /**
     * Get smoothing factor (0.0 - 1.0)
     */
    smooth(): number;
    /**
     * Get target X position
     */
    target_x(): number;
    /**
     * Get target Y position
     */
    target_y(): number;
    /**
     * Get zoom level
     */
    zoom(): number;
}

/**
 * Controller for boolean logic operations on sensor signals
 *
 * This struct wraps the ControllerType enum with optional parameters:
 * - secondary_sensor: for AND/OR operations
 * - numeric_params: for Blinky, Debounce, Threshold, Pattern controllers
 * - float_params: for Hysteresis controller
 * - custom_data: for Custom controller (name, code)
 */
export class Controller {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates an AND controller with a secondary sensor
     *
     * # Arguments
     * * `sensor` - The secondary sensor type
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.And(SensorType.MouseClick);
     * // Requires both primary sensor AND MouseClick to be active
     * ```
     */
    static and(sensor: SensorType): Controller;
    /**
     * Creates an AND controller (combines all sensors)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.And();
     * // Combines all sensors in the brick chain
     * ```
     */
    static and_any(): Controller;
    /**
     * Creates a Blinky controller that toggles at regular intervals
     *
     * # Arguments
     * * `interval` - Toggle interval in ticks (16.67ms at 60 FPS)
     *
     * # JavaScript Example
     * ```javascript
     * // Blink every 100ms (6 ticks at 60fps)
     * const blinky = Controller.Blinky(6);
     * ```
     */
    static blinky(interval: number): Controller;
    /**
     * Returns the controller type
     */
    controller_type(): ControllerType;
    /**
     * Creates a Custom controller with JavaScript code
     *
     * # Arguments
     * * `name` - Unique identifier for debugging
     * * `code` - JavaScript code to evaluate
     *
     * # JavaScript Example
     * ```javascript
     * const custom = Controller.Custom(
     *   'tooltipOnCtrlHover',
     *   'return signal.isSteady(6) && (context.modifiers & 2) !== 0;'
     * );
     * ```
     */
    static custom(name: string, code: string): Controller;
    /**
     * Returns the custom code (for Custom controllers)
     */
    custom_code(): string | undefined;
    /**
     * Returns the custom name (for Custom controllers)
     */
    custom_name(): string | undefined;
    /**
     * Creates a Debounce controller requiring N stable ticks
     *
     * # Arguments
     * * `ticks` - Number of consecutive ticks signal must be HIGH
     *
     * # JavaScript Example
     * ```javascript
     * // Require 100ms of stable signal (6 ticks)
     * const debounced = Controller.Debounce(6);
     * ```
     */
    static debounce(ticks: number): Controller;
    /**
     * Creates a new Direct controller (pass-through)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Direct();
     * ```
     */
    static direct(): Controller;
    /**
     * Returns the first float parameter (for Hysteresis high, Threshold value)
     */
    float_param1(): number;
    /**
     * Returns the second float parameter (for Hysteresis low)
     */
    float_param2(): number;
    /**
     * Checks if this controller has a secondary sensor
     */
    has_secondary_sensor(): boolean;
    /**
     * Creates a Hysteresis controller with different on/off thresholds
     *
     * # Arguments
     * * `high` - Activation threshold (0.0 to 1.0)
     * * `low` - Deactivation threshold (0.0 to 1.0)
     *
     * # JavaScript Example
     * ```javascript
     * // Activate at 80%, deactivate at 30%
     * const hyst = Controller.Hysteresis(0.8, 0.3);
     * ```
     */
    static hysteresis(high: number, low: number): Controller;
    /**
     * Checks if this controller is a Custom type
     */
    is_custom(): boolean;
    /**
     * Creates a NOT controller (inverts the signal)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Not();
     * // Inverts the primary sensor signal
     * ```
     */
    static not(): Controller;
    /**
     * Returns the numeric parameter (for Blinky, Debounce, Pattern)
     */
    numeric_param(): number;
    /**
     * Creates an OR controller with a secondary sensor
     *
     * # Arguments
     * * `sensor` - The secondary sensor type
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Or(SensorType.MouseClick);
     * // Requires primary sensor OR MouseClick to be active
     * ```
     */
    static or(sensor: SensorType): Controller;
    /**
     * Creates an OR controller (any sensor activates)
     *
     * # JavaScript Example
     * ```javascript
     * const controller = Controller.Or();
     * // Activates if any sensor is active
     * ```
     */
    static or_any(): Controller;
    /**
     * Creates a Pattern controller matching binary pattern
     *
     * # Arguments
     * * `mask` - 6-bit pattern to match
     *
     * # JavaScript Example
     * ```javascript
     * // Match double-click pattern: 100100
     * const pattern = Controller.Pattern(0b00100100);
     * ```
     */
    static pattern(mask: number): Controller;
    /**
     * Returns the secondary sensor (if any)
     *
     * Returns `null` if there is no secondary sensor.
     */
    secondary_sensor(): SensorType | undefined;
    /**
     * Creates a Threshold controller with minimum stability
     *
     * # Arguments
     * * `value` - Minimum stability threshold (0.0 to 1.0)
     *
     * # JavaScript Example
     * ```javascript
     * // Require 50% stability (3 out of 6 ticks)
     * const thresh = Controller.Threshold(0.5);
     * ```
     */
    static threshold(value: number): Controller;
}

/**
 * Controller type enumeration
 *
 * Defines the type of boolean logic to apply when combining sensor signals.
 *
 * # JavaScript Example
 * ```javascript
 * import { Controller, SensorType } from '@archflow/sdk';
 *
 * // Direct: pass through the primary sensor
 * const direct = Controller.Direct();
 *
 * // AND: require both MouseOver AND MouseClick
 * const and = Controller.And(SensorType.MouseClick);
 *
 * // OR: require MouseOver OR MouseClick
 * const or = Controller.Or(SensorType.MouseClick);
 *
 * // NOT: invert the primary sensor
 * const not = Controller.Not();
 *
 * // Blinky: toggle every N ticks
 * const blinky = Controller.Blinky(4);
 *
 * // Debounce: require N stable ticks
 * const debounce = Controller.Debounce(6);
 *
 * // Hysteresis: different on/off thresholds
 * const hyst = Controller.Hysteresis(0.8, 0.3);
 * ```
 */
export enum ControllerType {
    /**
     * Pass through the primary sensor signal
     */
    Direct = 0,
    /**
     * AND logic: primary AND other sensor must both be active
     */
    And = 1,
    /**
     * OR logic: primary OR other sensor must be active
     */
    Or = 2,
    /**
     * NOT logic: invert the primary sensor signal
     */
    Not = 3,
    /**
     * Blinky: Toggles active/inactive at regular intervals
     */
    Blinky = 4,
    /**
     * Debounce: Requires signal to be stable for N ticks
     */
    Debounce = 5,
    /**
     * Hysteresis: Different activation/deactivation thresholds
     */
    Hysteresis = 6,
    /**
     * Threshold: Requires minimum stability percentage
     */
    Threshold = 7,
    /**
     * Pattern: Matches specific binary pattern in history
     */
    Pattern = 8,
    /**
     * Custom: JavaScript sandbox evaluation
     */
    Custom = 9,
}

/**
 * Constants for controller types
 */
export class ControllerTypes {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    static and(): number;
    static blinky(): number;
    static debounce(): number;
    static direct(): number;
    static hysteresis(): number;
    static not(): number;
    static or(): number;
    static threshold(): number;
}

/**
 * Effect factories for JavaScript (convenience)
 */
export class Effect {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create a bloom effect
     */
    static bloom(threshold: number, intensity: number, radius: number): PostEffect;
    /**
     * Create a color grading effect
     */
    static color_grading(brightness: number, contrast: number, saturation: number, temperature: number): PostEffect;
    /**
     * Create a grayscale effect
     */
    static grayscale(intensity: number): PostEffect;
}

/**
 * WASM wrapper for accessing the event ring buffer
 *
 * This provides JavaScript access to events generated by the logic system.
 */
export class EventRingBufferWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Drain all pending events from the logic system
     *
     * This method returns all events currently in the ring buffer and
     * clears the buffer for the next frame.
     *
     * # Returns
     *
     * Array of JsLogicEvent objects representing all pending events
     *
     * # JavaScript Example
     *
     * ```javascript
     * const eventBuffer = new EventRingBufferWasm();
     * const events = eventBuffer.drain(logicSystem);
     *
     * for (const event of events) {
     *   console.log(`Event ${event.event_type} from entity ${event.entity_id}`);
     * }
     * ```
     */
    drain(system: LogicSystemWasm): JsLogicEvent[];
    /**
     * Get the number of events currently in the buffer
     *
     * # Returns
     *
     * Number of pending events
     */
    event_count(system: LogicSystemWasm): number;
    /**
     * Check if there are any pending events
     *
     * # Returns
     *
     * true if there are pending events, false otherwise
     */
    has_events(system: LogicSystemWasm): boolean;
    /**
     * Create a new EventRingBuffer accessor
     */
    constructor();
}

/**
 * Event type constants for JavaScript
 *
 * These correspond to the LogicEventType enum in Rust
 */
export class EventType {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Box selection completed
     */
    static readonly box_selection_completed: number;
    /**
     * Drag operation ended
     */
    static readonly drag_ended: number;
    /**
     * Drag operation started
     */
    static readonly drag_started: number;
    /**
     * Entity was selected/deselected
     */
    static readonly entity_selected: number;
    /**
     * Hover state changed
     */
    static readonly hover_changed: number;
    /**
     * Proximity threshold crossed
     */
    static readonly proximity_alert: number;
}

/**
 * Extended actuator types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { ExtendedActuatorType } from '@archflow/sdk';
 *
 * const highlight = ExtendedActuatorType.Highlight;
 * const camera = ExtendedActuatorType.Camera;
 * ```
 */
export enum ExtendedActuatorType {
    /**
     * Highlight actuator - changes entity color
     */
    Highlight = 0,
    /**
     * Select actuator - marks entity as selected
     */
    Select = 1,
    /**
     * Move actuator - moves entity (drag operation)
     */
    Move = 2,
    /**
     * Camera actuator - moves camera
     */
    Camera = 3,
    /**
     * Property actuator - sets entity property
     */
    Property = 4,
    /**
     * State actuator - changes entity state
     */
    State = 5,
}

/**
 * Configuration for highlight actuator
 */
export class HighlightConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the highlight color (ARGB)
     */
    color(): number;
    /**
     * Creates a new highlight configuration
     */
    constructor(color: number, restore_color: number, opacity: number);
    /**
     * Get the opacity (0.0 - 1.0)
     */
    opacity(): number;
    /**
     * Get the restore color (ARGB)
     */
    restore_color(): number;
}

/**
 * Single behavior block in JS API
 */
export class JsBehaviorBlock {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get actuator type
     */
    actuator_type(): number;
    /**
     * Get controller type
     */
    controller_type(): number;
    /**
     * Get key code
     */
    key_code(): number;
    constructor(name: string);
    /**
     * Get sensor type
     */
    sensor_type(): number;
}

/**
 * JS Entity Builder - Fluent API para JavaScript
 *
 * Permite encadenar llamadas en JavaScript:
 * ```javascript
 * const entity = await bridge.world.spawn()
 *     .insert(0, 0, 50, 50)  // x, y, width, height
 *     .behavior('move')
 *         .sensor(1, 25)      // sensor_type, key_code
 *         .actuator(2, 0, 100) // actuator_type, x, y
 *     .build();
 * ```
 */
export class JsEntityBuilder {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add an actuator to current behavior
     * actuator_type: 0=Highlight, 1=Select, 2=Move, etc.
     * x, y: coordinates or values
     */
    actuator(actuator_type: number, x: number, y: number): JsEntityBuilder;
    /**
     * Start a behavior block
     */
    behavior(name: string): JsEntityBuilder;
    /**
     * Get number of behavior blocks
     */
    behavior_count(): number;
    /**
     * Build the entity and return its ID
     */
    build(): number;
    /**
     * Set fill color (r, g, b)
     */
    color(_r: number, _g: number, _b: number): JsEntityBuilder;
    /**
     * Add a controller to current behavior
     * controller_type: 0=Direct, 1=AND, 2=OR, 3=NOT, etc.
     * param: parameter for the controller
     */
    controller(controller_type: number, param: number): JsEntityBuilder;
    /**
     * End current behavior block
     */
    end_behavior(): JsEntityBuilder;
    /**
     * Get the entity ID
     */
    entity_id(): number;
    /**
     * Get behavior at index
     */
    get_behavior(index: number): JsBehaviorBlock | undefined;
    /**
     * Insert a component (x, y, width, height)
     */
    insert(_x: number, _y: number, _width: number, _height: number): JsEntityBuilder;
    /**
     * Set layer for rendering order - uses RenderProperties component
     */
    layer(_layer: number): JsEntityBuilder;
    /**
     * Create a new JSEntityBuilder
     */
    constructor(entity_id: number);
    /**
     * Set position (x, y) - uses Transform component
     */
    position(_x: number, _y: number): JsEntityBuilder;
    /**
     * Add a sensor to current behavior
     * sensor_type: 0=MouseOver, 1=MouseClick, 2=KeyShortcut, etc.
     * key_code: key code for keyboard sensors
     */
    sensor(sensor_type: number, key_code: number): JsEntityBuilder;
    /**
     * Set shape type (0=Rectangle, 1=Circle, 2=Ellipse, etc.)
     * Uses ShapeTypes constants: ShapeTypes.circle(), ShapeTypes.rectangle(), etc.
     */
    shape(_shape_type: number): JsEntityBuilder;
    /**
     * Set size (width, height) - uses RenderProperties component
     */
    size(_width: number, _height: number): JsEntityBuilder;
    /**
     * Set stroke color (r, g, b)
     */
    stroke(_r: number, _g: number, _b: number): JsEntityBuilder;
    /**
     * Set stroke width
     */
    stroke_width(_width: number): JsEntityBuilder;
    /**
     * Set visibility (true=visible, false=hidden) - uses VisibilityComponent
     */
    visible(_is_visible: boolean): JsEntityBuilder;
}

/**
 * EntityCommandBuffer - Deferred command execution for JS-WASM
 *
 * Use this to batch multiple commands and execute them efficiently.
 */
export class JsEntityCommandBuffer {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get capacity
     */
    capacity(): number;
    /**
     * Clear the buffer
     */
    clear(): void;
    /**
     * Get command count
     */
    commands_count(): number;
    /**
     * Get commands pointer
     */
    commands_ptr(): number;
    /**
     * Despawn an entity
     */
    despawn(entity: number): void;
    /**
     * Check if empty
     */
    is_empty(): boolean;
    /**
     * Get command count
     */
    len(): number;
    /**
     * Create a new ECB
     */
    constructor(capacity: number);
    /**
     * Resize entity
     */
    resize(entity: number, width: number, height: number): void;
    /**
     * Set entity color
     */
    set_color(entity: number, color: number): void;
    /**
     * Set entity layer
     */
    set_layer(entity: number, layer: number): void;
    /**
     * Set selection state
     */
    set_selection(entity: number, selected: boolean): void;
    /**
     * Set entity shape (0 = rect, 1 = circle)
     */
    set_shape(entity: number, shape: number): void;
    /**
     * Set entity velocity
     */
    set_velocity(entity: number, vx: number, vy: number): void;
    /**
     * Set entity visibility
     */
    set_visible(entity: number, visible: boolean): void;
    /**
     * Spawn a new entity (returns temp ID for use within ECB)
     */
    spawn(x: number, y: number, width: number, height: number): number;
    /**
     * Get spawned count
     */
    spawned_count(): number;
    /**
     * Get spawned entity IDs
     */
    spawned_ids_ptr(): number;
    /**
     * Teleport entity to position
     */
    teleport(entity: number, x: number, y: number): void;
}

/**
 * Custom error type for JavaScript
 */
export class JsError {
    free(): void;
    [Symbol.dispose](): void;
    message(): string;
    constructor(message: string);
}

/**
 * Structure representing a single event for JavaScript consumption
 *
 * This is a simplified version of LogicEvent that's easy to serialize
 * across the WASM boundary.
 */
export class JsLogicEvent {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get data_1 (context-dependent)
     */
    readonly data_1: number;
    /**
     * Get data_2 (context-dependent)
     */
    readonly data_2: number;
    /**
     * Get data_3 (context-dependent)
     */
    readonly data_3: number;
    /**
     * Get the entity ID
     */
    readonly entity_id: number;
    /**
     * Get the event type
     */
    readonly event_type: number;
    /**
     * Get the timestamp
     */
    readonly timestamp_us: bigint;
}

/**
 * Simplified event data for WASM export
 */
export class JsLogicEventData {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Additional data depending on event type:
     * - ProximityAlert: f32 distance
     * - DragStarted/DragEnded: f32 x, f32 y position
     * - BoxSelectionCompleted: u32 count
     * - HoverChanged: u32 entity_id (or 0 for none)
     */
    data_1: number;
    data_2: number;
    data_3: number;
    /**
     * Entity ID that triggered the event
     */
    entity_id: number;
    /**
     * Event type identifier
     */
    event_type: number;
    /**
     * Timestamp in microseconds
     */
    timestamp_us: bigint;
}

/**
 * Complete Logic Bricks system for the web editor
 *
 * Provides fluent API for declaring sensor-actuator connections and processing input.
 */
export class LogicBricksSystem {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Clear creation state
     */
    clear_creation(): void;
    /**
     * Clear drag state
     */
    clear_drag_state(): void;
    /**
     * Get drag count
     */
    drag_count(): number;
    /**
     * Get event buffer length (WASM compatible - returns cached value)
     */
    event_buffer_len(): number;
    /**
     * Get the active tool
     */
    get_active_tool(): string;
    /**
     * Get creation start position
     */
    get_creation_start_pos(): number;
    /**
     * Get selected entities as array (WASM compatible)
     */
    get_selected_entities(): Array<any>;
    /**
     * Check if there are pending events
     */
    has_events(): boolean;
    /**
     * Check if creating
     */
    is_creating(): boolean;
    /**
     * Check if dragging
     */
    is_dragging(): boolean;
    /**
     * Create a new Logic Bricks system
     */
    constructor();
    /**
     * Get pending command count
     */
    pending_command_count(): number;
    /**
     * Poll all events and return count
     */
    poll_events(): number;
    /**
     * Sample input state from JavaScript
     *
     * Should be called each frame before tick().
     */
    sample_input(screen_x: number, screen_y: number, world_x: number, world_y: number, buttons: number, wheel: number, modifiers: number): void;
    /**
     * Get number of selected entities
     */
    selection_count(): number;
    /**
     * Set the active tool
     */
    set_active_tool(tool: string): void;
    /**
     * Set creation start position
     */
    set_creation_start(x: number, y: number): void;
}

/**
 * Logic Mapping Table for sensor-actuator connections
 *
 * This table manages connections between sensors and actuators for entities,
 * allowing complex behavior definition through the Logic Bricks system.
 *
 * # JavaScript Example
 * ```javascript
 * import { LogicMappingTable, SensorType, Controller, ActuatorType } from '@archflow/sdk';
 *
 * const table = new LogicMappingTable();
 * const entityId = 1;
 *
 * // Connect MouseOver sensor to Highlight actuator
 * table.addHighlight(entityId, SensorType.MouseOver, Controller.Direct());
 *
 * // Check if connection exists
 * console.log(table.hasConnection(entityId, SensorType.MouseOver)); // true
 *
 * // Get connection count
 * console.log(table.connectionCount(entityId)); // 1
 *
 * // Remove connection
 * table.removeConnection(entityId, SensorType.MouseOver);
 * ```
 */
export class LogicMappingTableWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Adds a Highlight actuator connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID (numeric)
     * * `sensor` - The sensor type to connect
     * * `controller` - The controller logic
     *
     * # JavaScript Example
     * ```javascript
     * table.addHighlight(1, SensorType.MouseOver, Controller.Direct());
     * ```
     */
    add_highlight(entity_id: number, sensor: SensorType, controller: Controller): void;
    /**
     * Adds a Move actuator connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID (numeric)
     * * `sensor` - The sensor type to connect
     * * `controller` - The controller logic
     *
     * # JavaScript Example
     * ```javascript
     * table.addMove(1, SensorType.MouseClick, Controller.And(SensorType.MouseOver));
     * ```
     */
    add_move(entity_id: number, sensor: SensorType, controller: Controller): void;
    /**
     * Adds a Select actuator connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID (numeric)
     * * `sensor` - The sensor type to connect
     * * `controller` - The controller logic
     *
     * # JavaScript Example
     * ```javascript
     * table.addSelect(1, SensorType.MouseClick, Controller.Direct());
     * ```
     */
    add_select(entity_id: number, sensor: SensorType, controller: Controller): void;
    /**
     * Clears all connections from the table
     *
     * # JavaScript Example
     * ```javascript
     * table.clear();
     * ```
     */
    clear(): void;
    /**
     * Clears all connections for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID
     *
     * # JavaScript Example
     * ```javascript
     * table.clearEntity(1);
     * ```
     */
    clear_entity(entity_id: number): void;
    /**
     * Gets the number of connections for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID
     *
     * # Returns
     * The number of connections registered for the entity
     *
     * # JavaScript Example
     * ```javascript
     * const count = table.connectionCount(1);
     * console.log(`Entity has ${count} connections`);
     * ```
     */
    connection_count(entity_id: number): number;
    /**
     * Gets all entity IDs that have connections
     *
     * # Returns
     * Array of entity IDs (as u32 values)
     *
     * # JavaScript Example
     * ```javascript
     * const entities = table.getConnectedEntities();
     * console.log(`Connected entities: ${entities}`);
     * ```
     */
    get_connected_entities(): Uint32Array;
    /**
     * Checks if an entity has a connection for a specific sensor
     *
     * # Arguments
     * * `entity_id` - The entity ID
     * * `sensor` - The sensor type to check for
     *
     * # Returns
     * `true` if the entity has a connection for the sensor, `false` otherwise
     *
     * # JavaScript Example
     * ```javascript
     * const hasConnection = table.hasConnection(1, SensorType.MouseOver);
     * ```
     */
    has_connection(entity_id: number, sensor: SensorType): boolean;
    /**
     * Checks if the table is empty
     *
     * # JavaScript Example
     * ```javascript
     * const isEmpty = table.isEmpty();
     * ```
     */
    is_empty(): boolean;
    /**
     * Creates a new LogicMappingTable
     *
     * # JavaScript Example
     * ```javascript
     * const table = new LogicMappingTable();
     * ```
     */
    constructor();
    /**
     * Removes a connection for an entity
     *
     * # Arguments
     * * `entity_id` - The entity ID
     * * `sensor` - The sensor type to disconnect
     *
     * # JavaScript Example
     * ```javascript
     * table.removeConnection(1, SensorType.MouseOver);
     * ```
     */
    remove_connection(entity_id: number, sensor: SensorType): void;
}

/**
 * WASM wrapper for LogicSystem
 *
 * This provides JavaScript access to the main Logic Bricks orchestration system.
 *
 * # JavaScript Example
 * ```javascript
 * import { LogicSystem } from '@archflow/sdk';
 *
 * const system = new LogicSystem();
 * system.update(timestamp);
 * ```
 */
export class LogicSystemWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Attach a behavior to an entity
     */
    attach_behavior(_behavior_id: number, _entity_id: number): void;
    /**
     * Get count of behaviors
     */
    behavior_count(): number;
    /**
     * Check if behavior has events
     */
    behavior_has_events(_behavior_id: number): boolean;
    /**
     * Create a simple behavior
     *
     * # Arguments
     * * `entity_id` - Entity ID for the behavior
     * * `sensor_type` - Sensor type (0=Click, 1=Hover, 2=Drag, 3=Key)
     * * `actuator_type` - Actuator type (0=Highlight, 1=Select, 2=Move, 3=Delete, 4=Emit)
     *
     * # Returns
     * Behavior ID
     */
    create_behavior(_entity_id: number, _sensor_type: number, _actuator_type: number): number;
    /**
     * Detach a behavior
     */
    detach_behavior(_behavior_id: number): void;
    /**
     * Drain all pending events from the event buffer
     *
     * # Returns
     * Array of event data objects (simplified for WASM)
     */
    drain_events(): JsLogicEventData[];
    /**
     * Get the number of pending events
     *
     * # Returns
     * Number of events in the buffer
     */
    event_count(): number;
    /**
     * Get behavior state as JSON
     */
    get_behavior_state(_behavior_id: number): string;
    /**
     * Check if there are pending events in the buffer
     *
     * # Returns
     * true if there are events, false otherwise
     */
    has_events(): boolean;
    /**
     * Creates a new LogicSystem
     *
     * # JavaScript Example
     * ```javascript
     * const system = new LogicSystem();
     * ```
     */
    constructor();
    /**
     * Set behavior enabled/disabled
     */
    set_behavior_enabled(_behavior_id: number, _enabled: boolean): void;
    /**
     * Updates the logic system timestamp
     *
     * This should be called each frame before sensor evaluation.
     *
     * # Arguments
     * * `timestamp_ms` - Current timestamp in milliseconds
     *
     * # JavaScript Example
     * ```javascript
     * system.update(performance.now());
     * ```
     */
    update(timestamp_ms: bigint): void;
}

/**
 * Material Component for rendering properties
 *
 * Use with JsEntityBuilder:
 * ```javascript
 * bridge.world.spawn()
 *     .insert(MaterialComponent.new({
 *         colorMultiply: [1.0, 0.5, 0.5, 1.0],
 *         emission: [0.2, 0.1, 0.0],
 *         blendMode: BlendMode.AlphaBlend,
 *     }))
 *     .build();
 * ```
 */
export class MaterialComponent {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get alpha cutoff
     */
    alpha_cutoff(): number;
    /**
     * Get blend mode
     */
    blend_mode(): BlendMode;
    /**
     * Get color multiply
     */
    color_multiply(): Float32Array;
    /**
     * Create with default values
     */
    static default_material(): MaterialComponent;
    /**
     * Get emission
     */
    emission(): Float32Array;
    /**
     * Create a new material component
     *
     * # Arguments
     * * `config` - MaterialConfig object with color, emission, blend mode
     */
    constructor(config: MaterialConfig);
    /**
     * Set blend mode
     */
    set_blend_mode(mode: BlendMode): void;
    /**
     * Set color multiply
     */
    set_color_multiply(color: Float32Array): void;
    /**
     * Set emission
     */
    set_emission(emission: Float32Array): void;
    /**
     * Set shader ID
     */
    set_shader_id(shader_id: number): void;
    /**
     * Get shader ID
     */
    shader_id(): number;
    /**
     * Create with specific blend mode
     */
    with_blend_mode(mode: BlendMode): MaterialComponent;
    /**
     * Create with color multiply
     */
    with_color_multiply(color: Float32Array): MaterialComponent;
    /**
     * Create with custom shader
     */
    with_shader(shader_id: number): MaterialComponent;
}

/**
 * Material configuration for JavaScript (using getters/setters for WASM)
 */
export class MaterialConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get alpha cutoff
     */
    alpha_cutoff(): number;
    /**
     * Get blend mode
     */
    blend_mode(): BlendMode;
    /**
     * Get color multiply [r, g, b, a]
     */
    color_multiply(): Float32Array;
    /**
     * Get emission color [r, g, b]
     */
    emission(): Float32Array;
    constructor();
    /**
     * Set alpha cutoff
     */
    set_alpha_cutoff(value: number): void;
    /**
     * Set blend mode
     */
    set_blend_mode(mode: BlendMode): void;
    /**
     * Set color multiply [r, g, b, a]
     */
    set_color_multiply(r: number, g: number, b: number, a: number): void;
    /**
     * Set emission color [r, g, b]
     */
    set_emission(r: number, g: number, b: number): void;
    /**
     * Set shader ID
     */
    set_shader_id(id: number): void;
    /**
     * Get shader ID
     */
    shader_id(): number;
}

/**
 * Configuration for move actuator
 */
export class MoveConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Whether X axis is constrained
     */
    constrain_x(): boolean;
    /**
     * Whether Y axis is constrained
     */
    constrain_y(): boolean;
    /**
     * Creates a new move configuration
     */
    constructor(snap: number, constrain_x: boolean, constrain_y: boolean);
    /**
     * Get snap value in pixels
     */
    snap(): number;
}

/**
 * Post-processing effect types (WASM version using structs)
 */
export class PostEffect {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create a color grading effect
     *
     * # Arguments
     * * `brightness` - Brightness adjustment (-1.0 to 1.0)
     * * `contrast` - Contrast adjustment (0.0 to 2.0)
     * * `saturation` - Saturation adjustment (0.0 to 2.0)
     * * `temperature` - Color temperature (-1.0 to 1.0)
     */
    static color_grading(brightness: number, contrast: number, saturation: number, _temperature: number): PostEffect;
    /**
     * Get effect type
     */
    effect_type(): string;
    /**
     * Create a grayscale effect
     *
     * # Arguments
     * * `intensity` - Grayscale intensity (0.0 to 1.0)
     */
    static grayscale(intensity: number): PostEffect;
    /**
     * Create a bloom effect
     *
     * # Arguments
     * * `threshold` - Minimum brightness to trigger bloom (0.0-1.0)
     * * `intensity` - Bloom strength (0.0-2.0)
     * * `radius` - Blur radius (0.0-1.0)
     */
    constructor(threshold: number, intensity: number, radius: number);
    /**
     * Get param1 (threshold/brightness/intensity)
     */
    param1(): number;
    /**
     * Get param2 (intensity/contrast)
     */
    param2(): number;
    /**
     * Get param3 (radius/saturation/temperature)
     */
    param3(): number;
}

/**
 * Post-processing pipeline for screen-wide effects
 *
 * Use:
 * ```javascript
 * const pipeline = bridge.postProcess();
 * pipeline.addEffect(Effect.bloom(0.8, 0.5, 0.5));
 * ```
 */
export class PostProcessPipeline {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add an effect to the pipeline
     */
    add_effect(effect: PostEffect): void;
    /**
     * Clear all effects
     */
    clear(): void;
    /**
     * Check if empty
     */
    is_empty(): boolean;
    /**
     * Check if enabled
     */
    is_enabled(): boolean;
    /**
     * Get number of effects
     */
    len(): number;
    /**
     * Create a new post-process pipeline
     */
    constructor();
    /**
     * Remove an effect by index
     */
    remove_effect(index: number): void;
    /**
     * Enable/disable pipeline
     */
    set_enabled(enabled: boolean): void;
}

/**
 * Configuration for property actuator
 */
export class PropertyConfig {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates a new property configuration
     */
    constructor(property_name: string, value: PropertyValue);
    /**
     * Get property name
     */
    property_name(): string;
    /**
     * Get property value
     */
    value(): PropertyValue;
}

/**
 * Property value wrapper for WASM
 */
export class PropertyValue {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create a boolean property value
     */
    static from_bool(value: boolean): PropertyValue;
    /**
     * Create a number property value
     */
    static from_number(value: number): PropertyValue;
    /**
     * Create a string property value
     */
    static from_string(value: string): PropertyValue;
    /**
     * Get the raw value string
     */
    value(): string;
}

/**
 * WASM wrapper for Pulse events
 *
 * Represents a sensor state change event flowing through the Logic Bricks system.
 *
 * # JavaScript Example
 * ```javascript
 * const pulse = {
 *   entityId: 123,
 *   sensorId: 5,
 *   state: 1, // Positive
 *   timestamp: 1000
 * };
 * ```
 */
export class PulseWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create a new Pulse
     */
    constructor(entity_id: number, sensor_id: number, state: number, timestamp: number);
    /**
     * Get the entity ID
     */
    readonly entity_id: number;
    /**
     * Get the sensor ID
     */
    readonly sensor_id: number;
    /**
     * Get the state (0=None, 1=Positive, 2=Negative)
     */
    readonly state: number;
    /**
     * Get the timestamp
     */
    readonly timestamp: number;
}

/**
 * Select mode for selection actuator (matches core SelectMode)
 */
export enum SelectModeWasm {
    /**
     * Single selection (replaces current selection)
     */
    Single = 0,
    /**
     * Multi selection (adds to current selection)
     */
    Multi = 1,
    /**
     * Replace selection (clears and selects new)
     */
    Replace = 2,
    /**
     * Toggle selection (inverts selection state)
     */
    Toggle = 3,
    /**
     * Add to selection (ensure selected)
     */
    Add = 4,
    /**
     * Subtract from selection (ensure deselected)
     */
    Subtract = 5,
}

/**
 * Sensor types for the Logic Bricks system
 *
 * # JavaScript Example
 * ```javascript
 * import { SensorType } from '@archflow/sdk';
 *
 * const sensor = SensorType.MouseOver;
 * ```
 */
export enum SensorType {
    /**
     * Mouse is hovering over an entity
     */
    MouseOver = 0,
    /**
     * Mouse button was clicked on an entity
     */
    MouseClick = 1,
    /**
     * Another entity is within proximity radius
     */
    Proximity = 2,
    /**
     * Keyboard shortcut was pressed
     */
    KeyShortcut = 3,
    /**
     * AABB collision between entities
     */
    Touch = 4,
    /**
     * Entity in directional cone (radar)
     */
    Radar = 5,
    /**
     * Rapid double-click pattern detected
     */
    DoubleTap = 6,
    /**
     * Mouse button held down (long press)
     */
    LongPress = 7,
    /**
     * Right mouse button click
     */
    RightClick = 8,
    /**
     * Always sensor - constantly active every frame
     */
    Always = 9,
    /**
     * Property sensor - detects changes in entity properties
     */
    Property = 10,
    /**
     * Ray sensor - line of sight detection
     */
    Ray = 11,
    /**
     * Timer sensor - activates after a delay
     */
    Timer = 12,
    /**
     * Channel sensor - listens for messages on a channel
     */
    Channel = 13,
}

/**
 * Constants for sensor types (matching SensorType enum)
 */
export class SensorTypes {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    static key_shortcut(): number;
    static mouse_click(): number;
    static mouse_over(): number;
    static proximity(): number;
    static radar(): number;
    static ray(): number;
    static right_click(): number;
    static touch(): number;
}

export class ShapeTypes {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    static arc(): number;
    static circle(): number;
    static cylinder(): number;
    static diamond(): number;
    static ellipse(): number;
    static line(): number;
    static rectangle(): number;
    static triangle(): number;
}

/**
 * SignalByte WASM wrapper
 *
 * A binary signal with 6-tick history for edge detection and pattern matching.
 * This is the JavaScript-accessible version of the core SignalByte type.
 *
 * # JavaScript Example
 * ```javascript
 * const signal = new SignalByte();
 * signal.push(true);
 * signal.push(true);
 * signal.push(false);
 * console.log(signal.getHistory()); // 6 (0b00000110)
 * console.log(signal.isStableHigh(3)); // false
 * ```
 */
export class SignalByteWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Returns true if there is any edge (rising or falling)
     */
    any_edge(): boolean;
    /**
     * Returns the raw u8 value (for serialization)
     */
    as_u8(): number;
    /**
     * Counts how many ticks are 1 in the history
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00110111);
     * console.log(signal.countOnes()); // 5
     * ```
     */
    count_ones(): number;
    /**
     * Counts how many ticks are 0 in the history
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00110111);
     * console.log(signal.countZeros()); // 1
     * ```
     */
    count_zeros(): number;
    /**
     * Creates a SignalByte from a u8 value
     */
    static from(value: number): SignalByteWasm;
    /**
     * Returns the current signal state (tick T0, least significant bit)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * console.log(signal.getCurrent()); // true
     * ```
     */
    get_current(): boolean;
    /**
     * Returns the 6-bit history of the signal
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * signal.push(true);
     * signal.push(false);
     * console.log(signal.getHistory()); // 6 (0b00000110)
     * ```
     */
    get_history(): number;
    /**
     * Detects falling edge: 1 in T-1, 0 in T
     *
     * Pattern: [xxxx10]
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);
     * signal.push(false);
     * console.log(signal.isFallingEdge()); // true
     * ```
     */
    is_falling_edge(): boolean;
    /**
     * Detects rising edge: 0 in T-1, 1 in T
     *
     * Pattern: [xxxx01]
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(false);
     * signal.push(true);
     * console.log(signal.isRisingEdge()); // true
     * ```
     */
    is_rising_edge(): boolean;
    /**
     * Alias for isSteadyHigh (for backward compatibility)
     */
    is_steady(ticks: number): boolean;
    /**
     * Checks if the signal has been steady (all 1s) for the last N ticks
     *
     * # Arguments
     * * `ticks` - Number of ticks to check (1-6)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = SignalByte.from(0b00111111);
     * console.log(signal.isSteadyHigh(6)); // true
     * console.log(signal.isSteadyHigh(3)); // true
     * ```
     */
    is_steady_high(ticks: number): boolean;
    /**
     * Checks if the signal has been steady (all 0s) for the last N ticks
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * console.log(signal.isSteadyLow(3)); // true
     * ```
     */
    is_steady_low(ticks: number): boolean;
    /**
     * Creates a new SignalByte with all bits set to 0
     */
    constructor();
    /**
     * Pushes a new signal state, shifting the history left
     *
     * # Arguments
     * * `active` - true if the signal is active, false otherwise
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * signal.push(true);  // 0b00000001
     * signal.push(true);  // 0b00000011
     * signal.push(false); // 0b00000110
     * ```
     */
    push(active: boolean): void;
    /**
     * Returns the size in bytes (always 1)
     *
     * # JavaScript Example
     * ```javascript
     * const signal = new SignalByte();
     * console.log(signal.size()); // 1
     * ```
     */
    size(): number;
}

/**
 * Texture Atlas Component for sprite rendering
 *
 * Use with JsEntityBuilder:
 * ```javascript
 * bridge.world.spawn()
 *     .insert(TextureAtlasComponent.new(0, 32, 32, 4, 4))
 *     .build();
 * ```
 */
export class TextureAtlasComponent {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get columns
     */
    columns(): number;
    /**
     * Get current sprite index
     */
    current_sprite(): number;
    /**
     * Get UV coordinates for current sprite
     */
    current_uv(): Float32Array;
    /**
     * Get/Set flip horizontally
     */
    flip_x(): boolean;
    /**
     * Get/Set flip vertically
     */
    flip_y(): boolean;
    /**
     * Create from atlas ID with sprite index
     */
    static from_atlas(atlas_id: number, sprite_index: number, columns: number, rows: number): TextureAtlasComponent;
    /**
     * Get UV coordinates for a sprite by index
     * Returns array [u0, v0, u1, v1]
     */
    get_uv(index: number): Float32Array;
    /**
     * Create a new texture atlas component
     *
     * # Arguments
     * * `texture_index` - Index into the texture array
     * * `sprite_width` - Width of each sprite in pixels
     * * `sprite_height` - Height of each sprite in pixels
     * * `columns` - Number of columns in the atlas
     * * `rows` - Number of rows in the atlas
     */
    constructor(texture_index: number, sprite_width: number, sprite_height: number, columns: number, rows: number);
    /**
     * Get rows
     */
    rows(): number;
    set_flip_x(flip: boolean): void;
    set_flip_y(flip: boolean): void;
    /**
     * Set sprite index
     */
    set_sprite(index: number): void;
    /**
     * Get total number of sprites in atlas
     */
    sprite_count(): number;
    /**
     * Get sprite height
     */
    sprite_height(): number;
    /**
     * Get sprite width
     */
    sprite_width(): number;
    /**
     * Get texture index
     */
    texture_index(): number;
}

export class WasmBridge {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a sensor connection to an entity
     *
     * Creates a sensor-to-actuator connection using the LogicMappingTable.
     *
     * # Arguments
     *
     * * `entity_id` - The entity to add the sensor to
     * * `sensor_type` - Type of sensor (0=MouseOver, 1=MouseClick, 2=Proximity, 3=KeyShortcut, 4=Touch, 5=Radar, 6=DoubleTap, 7=LongPress, 8=RightClick)
     * * `controller_type` - Type of controller (0=Direct, 1=AND, 2=OR, 3=NOT)
     * * `actuator_type` - Type of actuator (0=Highlight, 1=Select, 2=Move, 3=Sound, 4=Animation, 5=Custom, 6=Property, 7=Visibility)
     *
     * # Returns
     *
     * Ok(true) if connection was added successfully
     */
    add_sensor(entity_id: number, sensor_type: number, controller_type: number, actuator_type: number): boolean;
    /**
     * Batch configure multiple entities with all properties in one call
     *
     * Arrays must all have the same length. Use NaN/0/255 to skip individual fields.
     *
     * # Arguments
     *
     * * `ids` - Entity indices
     * * `xs`, `ys` - Positions (use NaN to skip)
     * * `widths`, `heights` - Sizes (use NaN to skip)
     * * `vxs`, `vys` - Velocities (use NaN to skip)
     * * `axs`, `ays` - Accelerations (use NaN to skip)
     * * `colors` - RGBA colors packed (use 0 to skip)
     * * `shapes` - Shape types (use 255 to skip)
     */
    batch_configure_entities(ids: Uint32Array, xs: Float32Array, ys: Float32Array, widths: Float32Array, heights: Float32Array, vxs: Float32Array, vys: Float32Array, axs: Float32Array, ays: Float32Array, colors: Uint32Array, shapes: Uint8Array): number;
    /**
     * Batch despawn multiple entities
     *
     * ids: array of entity indices to remove
     *
     * DEPRECATED: Use EntityManager or Command pattern instead
     */
    batch_despawn(ids: Uint32Array): number;
    /**
     * Batch set colors for multiple entities
     *
     * ids: array of entity indices
     * colors: array of RGBA colors (u32)
     */
    batch_set_colors(ids: Uint32Array, colors: Uint32Array): number;
    /**
     * Batch set positions for multiple entities
     *
     * ids: array of entity indices
     * xs: array of x positions (same length as ids)
     * ys: array of y positions (same length as ids)
     */
    batch_set_positions(ids: Uint32Array, xs: Float32Array, ys: Float32Array): number;
    /**
     * Batch set shapes for multiple entities (optimized)
     *
     * DEPRECATED: Use PropertyActuator via Logic Bricks instead
     */
    batch_set_shapes(ids: Uint32Array, shapes: Uint8Array): number;
    /**
     * Batch set sizes for multiple entities
     *
     * ids: array of entity indices
     * widths: array of widths
     * heights: array of heights
     */
    batch_set_sizes(ids: Uint32Array, widths: Float32Array, heights: Float32Array): number;
    /**
     * Batch set visibility for multiple entities
     *
     * ids: array of entity indices
     * visible: visibility state to apply to all
     *
     * DEPRECATED: Use VisibilityActuator via Logic Bricks instead
     */
    batch_set_visibility(ids: Uint32Array, visible: boolean): number;
    /**
     * Bulk spawn multiple entities in a single call - ZERO-COPY
     *
     * This is the MOST EFFICIENT way to spawn entities:
     * - positions: flat array of [x0, y0, x1, y1, ...] (2 * count floats)
     * - sizes: flat array of [w0, h0, w1, h1, ...] (2 * count floats)
     * - colors: flat array of [r0, g0, b0, a0, r1, g1, b1, a1, ...] (4 * count u8s)
     *   Pass empty Uint8Array() for random colors
     *
     * Returns: array of spawned entity indices
     *
     * # Example (JavaScript)
     * ```js
     * const positions = new Float32Array([100, 100, 200, 200, 300, 300]);
     * const sizes = new Float32Array([50, 50, 60, 60, 70, 70]);
     * const colors = new Uint8Array([255, 0, 0, 255, 0, 255, 0, 255]); // or empty for random
     * const ids = bridge.bulk_spawn(positions, sizes, colors);
     * ```
     */
    bulk_spawn(positions: Float32Array, sizes: Float32Array, colors: Uint8Array): Uint32Array;
    /**
     * Check if redo is available
     */
    can_redo(): boolean;
    /**
     * Check if undo is available
     */
    can_undo(): boolean;
    /**
     * Clear all entities
     */
    clear(): void;
    /**
     * Clear all logic connections for all entities
     */
    clear_all_logic(): void;
    /**
     * Clear highlight tint (reset to default)
     *
     * DEPRECATED: Use HighlightActuator via Logic Bricks instead
     */
    clear_color_tint(entity_index: number): void;
    /**
     * Clear all logic connections for an entity
     */
    clear_entity_logic(entity_id: number): void;
    /**
     * Clear all selections (deselect all entities)
     *
     * DEPRECATED: Use SelectActuator or query system instead
     */
    clear_selection(): void;
    /**
     * Configure a single entity with all properties in one call
     *
     * This reduces JS-WASM call overhead by setting multiple properties at once.
     *
     * # Arguments
     *
     * * `entity_id` - Entity index to configure
     * * `x`, `y` - Position (pass NaN to skip)
     * * `width`, `height` - Size (pass NaN to skip)
     * * `vx`, `vy` - Velocity (pass NaN to skip)
     * * `ax`, `ay` - Acceleration (pass NaN to skip)
     * * `color` - RGBA color packed (pass 0 to skip)
     * * `stroke_color` - Stroke color packed (pass 0 to skip)
     * * `stroke_width` - Stroke width (pass 0 to skip)
     * * `shape` - Shape type (pass 255 to skip)
     * * `visible` - Visibility (pass 2 to skip, 0=hidden, 1=visible, 2=skip)
     */
    configure_entity(entity_id: number, x: number, y: number, width: number, height: number, vx: number, vy: number, ax: number, ay: number, color: number, stroke_color: number, stroke_width: number, shape: number, visible: number): void;
    /**
     * Configure mouse sensor for an entity
     *
     * # Arguments
     *
     * * `mode` - Mouse mode: 0=movement, 1=left_button, 2=right_button, 3=middle_button, 4=wheel_up
     * * `tap` - Enable tap detection (true) or continuous (false)
     */
    configure_mouse_sensor(mode: number, tap: boolean): void;
    /**
     * Get number of connections for an entity
     */
    connection_count(entity_id: number): number;
    /**
     * Create a new EntityCommandBuffer for batched operations
     *
     * Use this to register multiple commands and execute them in a single
     * batch, minimizing JS↔WASM overhead.
     *
     * # Example
     * ```javascript
     * const ecb = bridge.create_ecb(1024);
     *
     * // Register commands (deferred, not executed yet)
     * ecb.spawn(100, 200, 50, 50);
     * ecb.spawn(150, 250, 50, 50);
     * ecb.set_color(0, 0xFF00FF00);
     * ecb.teleport(1, 300, 400);
     * ecb.despawn(2);
     *
     * // Execute all commands at once
     * const result = ecb.playback();
     * // result.spawned = [0, 1]
     * // result.despawned = [2]
     * ```
     */
    create_ecb(capacity: number): JsEntityCommandBuffer;
    /**
     * Delete all selected entities
     *
     * DEPRECATED: Use SelectActuator + DeleteActuator via Logic Bricks instead
     */
    delete_selected(): void;
    /**
     * Detect available graphics backends
     */
    detect_available_backends(): object;
    /**
     * Duplicate an entity (create a copy at a slight offset)
     *
     * DEPRECATED: Use EntityFactory or Command pattern instead
     */
    duplicate_entity(entity_index: number): number;
    /**
     * Get the number of alive entities
     */
    entity_count(): number;
    /**
     * Execute all pending commands in an EntityCommandBuffer
     *
     * This executes all commands in the ECB buffer on the ECS
     */
    execute_ecb(ecb: JsEntityCommandBuffer): object;
    /**
     * Get the current accumulator value (for debugging)
     */
    get_accumulator(): number;
    /**
     * Get the active fill color (returns RGBA as hex string)
     */
    get_active_color(): string;
    /**
     * Get the active stroke color (returns RGBA as hex string)
     */
    get_active_stroke_color(): string;
    /**
     * Get the active stroke width
     */
    get_active_stroke_width(): number;
    /**
     * Get list of alive entity indices
     */
    get_alive_entities(): Uint32Array;
    /**
     * Get master volume for audio
     */
    get_audio_master_volume(): number;
    /**
     * Get the camera center position
     */
    get_camera_center(): Array<any>;
    /**
     * Get the color of an entity (returns hex string)
     *
     * DEPRECATED: Use query system or EntityStore directly instead
     */
    get_color(entity_index: number): string;
    /**
     * Get pointer to entity colors data
     *
     * Returns a pointer to the colors array (RGBA packed u32) for all entities.
     */
    get_colors_ptr(): number;
    /**
     * Get ECS component count for diagnostics
     */
    get_ecs_component_count(): number;
    /**
     * Get ECS entity count for diagnostics
     */
    get_ecs_entity_count(): number;
    /**
     * Get entity color as hex string
     */
    get_entity_color_hex(entity_index: number): string;
    /**
     * Get the current entity count
     */
    get_entity_count(): number;
    /**
     * Get entity label from string pool
     *
     * DEPRECATED: Use query system or EntityStore instead
     */
    get_entity_label(entity_index: number): string;
    /**
     * Get entity position in screen coordinates
     */
    get_entity_position_screen(entity_index: number): Array<any>;
    /**
     * Get entity position in world coordinates
     */
    get_entity_position_world(entity_index: number): Array<any>;
    /**
     * Get entity shape type
     */
    get_entity_shape(entity_index: number): number;
    /**
     * Get entity size in screen coordinates
     */
    get_entity_size_screen(entity_index: number): Array<any>;
    /**
     * Get entity size in world coordinates
     */
    get_entity_size_world(entity_index: number): Array<any>;
    /**
     * Get current velocity of an entity
     *
     * DEPRECATED: Use query_with_velocity() or EntityStore query instead
     */
    get_entity_velocity(entity_index: number): Float32Array;
    /**
     * Get the current fixed timestep value
     */
    get_fixed_timestep(): number;
    /**
     * Get history state for UI feedback
     */
    get_history_state(): string;
    /**
     * Get a pointer to the SharedArrayBuffer for input events
     *
     * This returns a pointer to the InputRingBuffer that JavaScript can
     * write to directly via SharedArrayBuffer.
     */
    get_input_buffer_ptr(): number;
    /**
     * Get the size of the input buffer in bytes
     */
    static get_input_buffer_size(): number;
    /**
     * Get the current entity capacity (grows dynamically)
     */
    get_max_entities(): number;
    /**
     * Get the current maximum substeps setting
     */
    get_max_substeps(): number;
    /**
     * Get current keyboard modifiers
     *
     * Returns bitmask of pressed modifiers (1=shift, 2=ctrl, 4=alt)
     */
    get_modifiers(): number;
    /**
     * Get current mouse button state
     *
     * Returns bitmask of pressed buttons (1=left, 2=right, 4=middle)
     */
    get_mouse_buttons(): number;
    /**
     * Get current mouse position in screen coordinates
     *
     * Returns tuple of (x, y) or null if engine not initialized.
     */
    get_mouse_position(): string;
    /**
     * Get the list of selected entity IDs
     */
    get_selection(): Array<any>;
    /**
     * Get the stroke color of an entity (returns hex string)
     *
     * DEPRECATED: Use query system or EntityStore directly instead
     */
    get_stroke_color(entity_index: number): string;
    /**
     * Get the stroke width of an entity
     *
     * DEPRECATED: Use query system or EntityStore directly instead
     */
    get_stroke_width(entity_index: number): number;
    /**
     * Get the current tool type
     */
    get_tool(): string;
    /**
     * Get the current tool type as index
     *
     * Returns the tool index (u8) for type-safe handling in JavaScript.
     * See set_tool_by_type for index mapping.
     */
    get_tool_index(): number;
    /**
     * Get count of transforms (entities with valid data)
     */
    get_transforms_count(): number;
    /**
     * Get pointer to entity transforms data
     *
     * Returns a pointer to the transforms array [x, y, width, height] for all entities.
     * Use with get_transforms_count() to know the valid range.
     */
    get_transforms_ptr(): number;
    /**
     * Get pointer to velocities data
     *
     * Returns a pointer to the velocities array [vx, vy] for all entities.
     */
    get_velocities_ptr(): number;
    /**
     * Get the current camera zoom level
     */
    get_zoom(): number;
    /**
     * Initialize audio context (must be called after user interaction)
     */
    init_audio(): boolean;
    /**
     * Initialize the engine
     *
     * This should be called once when the application starts.
     */
    initialize(canvas_width: number, canvas_height: number): void;
    /**
     * Initialize graphics (uses WebGL2/Canvas 2D by default)
     *
     * This should be called after `initialize()` and after the canvas is mounted.
     */
    initialize_graphics(canvas: HTMLCanvasElement): void;
    /**
     * Initialize graphics with a specific backend
     *
     * Supported backends: "webgl2", "webgpu", "canvas2d", "auto"
     */
    initialize_graphics_with_backend(canvas: HTMLCanvasElement, backend: string): void;
    /**
     * Check if entity is selected
     */
    is_entity_selected(entity_index: number): boolean;
    /**
     * Check if entity is visible
     *
     * DEPRECATED: Use query_by_visibility() instead
     */
    is_entity_visible(entity_index: number): boolean;
    /**
     * Check if context recovery is in progress
     */
    is_recovering(): boolean;
    /**
     * Get selection state of an entity
     */
    is_selected(entity_index: number): boolean;
    /**
     * Load a scene from JSON string
     *
     * Expects JSON with format (component-based):
     * ```json
     * {
     *   "entities": [
     *     { "id": "entity1", "components": { "Position": {"x": 100, "y": 200}, "Size": {"width": 50, "height": 50}, "Shape": {"shape": 0}, "Color": {"color": 4294967295} } }
     *   ]
     * }
     * ```
     */
    load_scene(json: string): number;
    /**
     * Load a sound from URL and register it
     *
     * Returns the sound ID that can be used to play this sound.
     * Note: Actual loading happens asynchronously via Web Audio API.
     */
    load_sound(name: string, _url: string): number;
    /**
     * Move an entity by the given delta
     *
     * DEPRECATED: Use MoveActuator via Logic Bricks or configure_entity() instead
     */
    move_entity(entity_index: number, dx: number, dy: number): void;
    /**
     * Move entity by delta (direct position update, not command queue)
     *
     * DEPRECATED: Use MoveActuator via Logic Bricks or configure_entity() instead
     */
    move_entity_by(entity_index: number, dx: number, dy: number): void;
    /**
     * Create a new WASM bridge
     */
    constructor();
    /**
     * Report keyboard event to Logic Bricks sensors
     *
     * This should be called from JavaScript's keydown/keyup event handlers.
     * Triggers keyboard shortcut sensors.
     *
     * # Arguments
     * * `key_code` - DOM keyCode value
     * * `is_down` - true for keydown, false for keyup
     * * `modifiers` - Bitmask of modifiers (1=shift, 2=ctrl, 4=alt)
     */
    on_key(key_code: number, is_down: boolean, modifiers: number): void;
    /**
     * Report mouse down event to Logic Bricks sensors
     *
     * This should be called from JavaScript's mousedown event handler.
     * Triggers mouse click sensors for left/right/middle buttons.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `button` - Mouse button (0=left, 1=right, 2=middle)
     * * `modifiers` - Bitmask of modifiers (1=shift, 2=ctrl, 4=alt)
     */
    on_mouse_down(screen_x: number, screen_y: number, button: number, modifiers: number): void;
    /**
     * Report mouse move event to Logic Bricks sensors
     *
     * This should be called from JavaScript's mousemove event handler.
     * The engine will convert screen coordinates to world coordinates
     * and feed them to the appropriate sensors.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `buttons` - Bitmask of pressed buttons (1=left, 2=right, 4=middle)
     */
    on_mouse_move(screen_x: number, screen_y: number, buttons: number, modifiers: number): void;
    /**
     * Report mouse up event to Logic Bricks sensors
     *
     * This should be called from JavaScript's mouseup event handler.
     * Clears the button state in sensors.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `button` - Mouse button that was released
     */
    on_mouse_up(screen_x: number, screen_y: number, _button: number, modifiers: number): void;
    /**
     * Report mouse wheel event to Logic Bricks sensors
     *
     * This should be called from JavaScript's wheel event handler.
     * Triggers zoom camera actuators when applicable.
     *
     * # Arguments
     * * `screen_x` - Mouse X position in screen pixels
     * * `screen_y` - Mouse Y position in screen pixels
     * * `delta_y` - Scroll delta (positive=up, negative=down)
     * * `modifiers` - Bitmask of modifiers
     */
    on_wheel(screen_x: number, screen_y: number, delta_y: number, modifiers: number): void;
    /**
     * Play a beep sound using Web Audio API oscillator
     *
     * # Arguments
     * * `frequency` - Frequency in Hz (220.0 to 2000.0)
     * * `duration` - Duration in seconds (0.1 to 2.0)
     * * `volume` - Volume/gain from 0.0 to 1.0
     */
    play_beep(frequency: number, _duration: number, volume: number): void;
    /**
     * Play a sound for a specific entity
     *
     * The sound will be played with entity-specific volume settings if AudioComponent exists.
     */
    play_sound(entity_id: number, sound_id: number): void;
    /**
     * Poll all events from the logic system
     *
     * Returns a JavaScript array of events emitted by the logic system
     * during the current frame. Call this once per frame after `tick()`.
     *
     * # Returns
     *
     * Number of events generated during the current frame.
     * Call this once per frame after `tick()`.
     *
     * # Example
     *
     * ```javascript
     * // In your JavaScript/TypeScript code
     * const eventCount = bridge.poll_events();
     * if (eventCount > 0) {
     *     console.log('Events generated:', eventCount);
     * }
     * ```
     */
    poll_events(): number;
    /**
     * Process all pending input events
     *
     * This drains the input ring buffer and feeds events to Logic Bricks sensors.
     * Called automatically by tick(), but can be called manually if needed.
     */
    process_input_events(): void;
    /**
     * Push an input event from JavaScript
     *
     * This is a higher-level alternative to directly writing to SharedArrayBuffer.
     * JavaScript can call this function to push input events.
     */
    push_input_event(event_type: number, x: number, y: number, buttons: number, modifiers: number): void;
    /**
     * Query all alive entities (returns all entity IDs)
     */
    query_all(): Uint32Array;
    /**
     * Query entities by layer
     */
    query_by_layer(layer: number): Uint32Array;
    /**
     * Query entities with minimum size
     */
    query_by_min_size(min_width: number, min_height: number): Uint32Array;
    /**
     * Query entities by selection state
     */
    query_by_selection(selected: boolean): Uint32Array;
    /**
     * Query entities by shape type
     *
     * shape: 0=rectangle, 1=circle, 2=triangle, etc.
     */
    query_by_shape(shape: number): Uint32Array;
    /**
     * Query entities by shape type using ECS
     *
     * shape_type: 0=rectangle, 1=circle, 2=triangle, etc.
     */
    query_by_shape_type(shape_type: number): Uint32Array;
    /**
     * Query entities by visibility
     */
    query_by_visibility(visible: boolean): Uint32Array;
    /**
     * Query all entities with ColorComponent using ECS
     */
    query_color_components(): Uint32Array;
    /**
     * Query entities within bounds (AABB query)
     */
    query_in_bounds(x: number, y: number, width: number, height: number): Uint32Array;
    /**
     * Query all entities with ShapeComponent using ECS
     *
     * Returns entity indices that have a ShapeComponent attached.
     */
    query_shape_components(): Uint32Array;
    /**
     * Query all visible entities using ECS
     */
    query_visible_entities(): Uint32Array;
    /**
     * Query entities that have velocity (moving entities)
     */
    query_with_velocity(): Uint32Array;
    /**
     * Redo the last undone action
     */
    redo(): void;
    /**
     * Remove a sensor connection from an entity
     *
     * # Arguments
     *
     * * `entity_id` - The entity to remove the sensor from
     * * `sensor_type` - Type of sensor to disconnect
     */
    remove_sensor(entity_id: number, sensor_type: number): void;
    /**
     * Resize the engine and renderer
     */
    resize(width: number, height: number): void;
    /**
     * Add an entity to the selection (toggle mode)
     */
    select_entity(entity_index: number): void;
    /**
     * Serialize the current project
     */
    serialize_project(): Uint8Array;
    /**
     * Serialize current scene to JSON string
     */
    serialize_scene(): string;
    /**
     * Set the active fill color for new shapes
     */
    set_active_color(r: number, g: number, b: number, a: number): void;
    /**
     * Set the active stroke color for new shapes
     */
    set_active_stroke_color(r: number, g: number, b: number, a: number): void;
    /**
     * Set the active stroke width for new shapes
     */
    set_active_stroke_width(width: number): void;
    /**
     * Set master volume for all audio
     */
    set_audio_master_volume(volume: number): void;
    /**
     * Mute/unmute all audio
     */
    set_audio_muted(muted: boolean): void;
    /**
     * Set the camera center position
     */
    set_camera_center(x: number, y: number): void;
    /**
     * Set the color of an entity
     *
     * DEPRECATED: Use HighlightActuator via Logic Bricks instead
     */
    set_color(entity_index: number, r: number, g: number, b: number, a: number): void;
    /**
     * Set highlight tint color (for visual feedback on hover/selection)
     *
     * DEPRECATED: Use HighlightActuator via Logic Bricks instead
     */
    set_color_tint(entity_index: number, r: number, g: number, b: number, a: number): void;
    /**
     * Set the selection state of an entity directly
     *
     * Uses DeltaMask for memory-efficient undo/redo via command queue.
     */
    set_entity_selected(entity_index: number, selected: boolean): void;
    /**
     * Set velocity directly (for physics integration)
     *
     * DEPRECATED: Use PhysicsSystem via tick() with fixed timestep instead
     */
    set_entity_velocity(entity_index: number, vx: number, vy: number): void;
    /**
     * Set entity visibility
     *
     * DEPRECATED: Use VisibilityActuator via Logic Bricks instead
     */
    set_entity_visible(entity_index: number, visible: boolean): void;
    /**
     * Set the fixed timestep directly in seconds
     */
    set_fixed_timestep(dt: number): void;
    /**
     * Set the fixed timestep for physics simulation (in Hz)
     */
    set_fixed_timestep_hz(hz: number): void;
    /**
     * Set the label of an entity
     *
     * DEPRECATED: Use MetadataComponent or entity properties instead
     */
    set_label(entity_index: number, label: string): void;
    /**
     * Set master volume
     *
     * volume: 0.0-1.0
     */
    set_master_volume(volume: number): void;
    /**
     * Set the maximum number of substeps per frame
     */
    set_max_substeps(max_steps: number): void;
    /**
     * Set the position of an entity
     *
     * DEPRECATED: Use MoveActuator via Logic Bricks or configure_entity() instead
     */
    set_position(entity_index: number, x: number, y: number): void;
    /**
     * Set selection state of an entity
     */
    set_selected(entity_index: number, selected: boolean): void;
    /**
     * Set the shape type of an entity
     *
     * DEPRECATED: Use JsEntityBuilder.shape() or ShapeComponent ECS instead
     */
    set_shape(entity_index: number, shape: number): void;
    /**
     * Set the size of an entity
     *
     * DEPRECATED: Use GizmoScaleActuator or configure_entity() instead
     */
    set_size(entity_index: number, width: number, height: number): void;
    /**
     * Set the stroke color of an entity
     *
     * DEPRECATED: Use HighlightActuator via Logic Bricks instead
     */
    set_stroke_color(entity_index: number, r: number, g: number, b: number, a: number): void;
    /**
     * Set the stroke width of an entity
     *
     * DEPRECATED: Use HighlightActuator via Logic Bricks instead
     */
    set_stroke_width(entity_index: number, width: number): void;
    /**
     * Set the current tool type
     */
    set_tool(tool: string): void;
    /**
     * Set the current tool type (type-safe version)
     *
     * Takes a tool index (u8) that maps to ToolType enum:
     * - 0: Select
     * - 1: BoxSelect
     * - 2: Pan
     * - 3: Zoom
     * - 4: Rectangle
     * - 5: Circle
     * - 6: Triangle
     * - 7: Diamond
     * - 8: Square
     * - 9: Line
     * - 10: Text
     * - 11: Connection
     * - 12: Delete
     */
    set_tool_by_type(tool_index: number): void;
    /**
     * Set the camera zoom level
     */
    set_zoom(zoom: number): void;
    /**
     * Spawn a new entity at the given position
     */
    spawn_entity(x: number, y: number, width: number, height: number): number;
    /**
     * Spawn a pool of pre-allocated entities for optimal performance
     *
     * Use this to pre-allocate entities at startup, then use set_visible()
     * to show/hide them instead of spawning/despawning.
     *
     * Returns: number of entities spawned
     */
    spawn_pool(count: number): number;
    /**
     * Stop a sound for a specific entity
     */
    stop_sound(entity_id: number): void;
    /**
     * Run one frame of the engine
     *
     * This should be called from requestAnimationFrame.
     * Uses the fluent API: sample_input() → tick() → poll_events()
     *
     * Implements Fixed Timestep (HU-PERF-001) for stable physics:
     * - Uses an accumulator to decouple physics from frame rate
     * - Runs physics in fixed time steps (default: 60 Hz)
     * - Prevents "spiral of death" with max substeps limit
     */
    tick(timestamp: number): void;
    /**
     * Undo the last action
     */
    undo(): void;
}

/**
 * Zero-copy buffer for direct memory access
 */
export class ZeroCopyCommandBuffer {
    free(): void;
    [Symbol.dispose](): void;
    clear(): void;
    count(): number;
    data_ptr(): number;
    constructor(capacity: number);
    set_count(count: number): void;
}

/**
 * Create a delete actuator
 *
 * # Returns
 * ActuatorType.Move (delete operates via move/transform)
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorDelete();
 * ```
 */
export function actuator_delete(): ActuatorType;

/**
 * Create an emit event actuator
 *
 * # Returns
 * ActuatorType.Move (event emission handled via state change)
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorEmitEvent('EntitySelected', '{"id":42}');
 * ```
 */
export function actuator_emit_event(event_name: string, event_data?: string | null): ActuatorType;

/**
 * Create a highlight actuator
 *
 * # Arguments
 * * `color_argb` - Color in ARGB format
 * * `opacity` - Opacity (0.0 to 1.0)
 *
 * # Returns
 * ActuatorType.Highlight
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorHighlight(0xff00ff00, 0.5);
 * ```
 */
export function actuator_highlight(color_argb: number, opacity: number): ActuatorType;

/**
 * Create a move actuator
 *
 * # Arguments
 * * `mode` - 0=To, 1=By, 2=Drag
 * * `x` - X value or offset
 * * `y` - Y value or offset
 *
 * # Returns
 * ActuatorType.Move
 *
 * # JavaScript Example
 * ```javascript
 * const to = actuatorMove(0, 100, 200);
 * ```
 */
export function actuator_move(mode: number, x: number, y: number): ActuatorType;

/**
 * Create a clear select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectClear();
 * ```
 */
export function actuator_select_clear(): ActuatorType;

/**
 * Create a multi-select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectMulti();
 * ```
 */
export function actuator_select_multi(): ActuatorType;

/**
 * Create a single select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectSingle();
 * ```
 */
export function actuator_select_single(): ActuatorType;

/**
 * Create a toggle select actuator
 *
 * # Returns
 * ActuatorType.Select
 *
 * # JavaScript Example
 * ```javascript
 * const actuator = actuatorSelectToggle();
 * ```
 */
export function actuator_select_toggle(): ActuatorType;

/**
 * Create an AND controller
 *
 * # Arguments
 * * `sensor` - Secondary sensor that must also be active
 *
 * # Returns
 * Controller with AND logic (all sensors must be active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerAnd(SensorType.MouseOver);
 * ```
 */
export function factory_and(sensor: SensorType): Controller;

/**
 * Create a Blinky controller
 *
 * # Arguments
 * * `interval` - Toggle interval in ticks
 *
 * # Returns
 * Blinky controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerBlinky(4);
 * ```
 */
export function factory_blinky(interval: number): Controller;

/**
 * Create a Custom controller
 *
 * # Arguments
 * * `name` - Controller name
 * * `code` - Custom evaluation code
 *
 * # Returns
 * Custom controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerCustom('myLogic', 'return signal.isSteady(6);');
 * ```
 */
export function factory_custom(name: string, code: string): Controller;

/**
 * Create a Debounce controller
 *
 * # Arguments
 * * `ticks` - Number of ticks for stability
 *
 * # Returns
 * Debounce controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerDebounce(6);
 * ```
 */
export function factory_debounce(ticks: number): Controller;

/**
 * Create a Direct controller
 *
 * # Returns
 * Direct controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerDirect();
 * ```
 */
export function factory_direct(): Controller;

/**
 * Create a Hysteresis controller
 *
 * # Arguments
 * * `high` - Activation threshold (0.0 to 1.0)
 * * `low` - Deactivation threshold (0.0 to 1.0)
 *
 * # Returns
 * Hysteresis controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerHysteresis(0.8, 0.3);
 * ```
 */
export function factory_hysteresis(high: number, low: number): Controller;

/**
 * Create a NAND controller
 *
 * # Returns
 * Controller with NAND logic (not all sensors active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerNand();
 * ```
 */
export function factory_nand(): Controller;

/**
 * Create a NOR controller
 *
 * # Returns
 * Controller with NOR logic (no sensors active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerNor();
 * ```
 */
export function factory_nor(): Controller;

/**
 * Create a NOT controller
 *
 * # Returns
 * Controller with NOT logic
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerNot();
 * ```
 */
export function factory_not(): Controller;

/**
 * Create an OR controller
 *
 * # Arguments
 * * `sensor` - Alternative sensor that can activate
 *
 * # Returns
 * Controller with OR logic (any sensor activates)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerOr(SensorType.MouseClick);
 * ```
 */
export function factory_or(sensor: SensorType): Controller;

/**
 * Create a Pattern controller
 *
 * # Arguments
 * * `mask` - 6-bit pattern to match
 *
 * # Returns
 * Pattern controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerPattern(0b00100100);
 * ```
 */
export function factory_pattern(mask: number): Controller;

/**
 * Create a Threshold controller
 *
 * # Arguments
 * * `value` - Minimum stability (0.0 to 1.0)
 *
 * # Returns
 * Threshold controller
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerThreshold(0.5);
 * ```
 */
export function factory_threshold(value: number): Controller;

/**
 * Create an XOR controller
 *
 * # Returns
 * Controller with XOR logic (exactly one sensor active)
 *
 * # JavaScript Example
 * ```javascript
 * const ctrl = controllerXor();
 * ```
 */
export function factory_xor(): Controller;

export function get_global_callback_registry(): CallbackRegistry;

/**
 * Create a collision detection sensor
 *
 * # Arguments
 * * `layer_id` - Optional layer ID (0 for default)
 *
 * # Returns
 * SensorType.Touch for collision detection
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorCollisionDetect(0);
 * ```
 */
export function sensor_collision_detect(layer_id: number): SensorType;

/**
 * Create a double-tap sensor
 *
 * # Returns
 * SensorType.DoubleTap
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorDoubleTap();
 * ```
 */
export function sensor_double_tap(): SensorType;

/**
 * Create a keyboard key press sensor
 *
 * # Arguments
 * * `key_code` - Key code number
 * * `modifiers` - Optional bitmask of modifiers (1=Shift, 2=Ctrl, 4=Alt)
 *
 * # Returns
 * SensorType.KeyShortcut
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorKeyboardKey(46, 0); // Delete key
 * ```
 */
export function sensor_keyboard_key(_key_code: number, _modifiers: number): SensorType;

/**
 * Create a long-press sensor
 *
 * # Arguments
 * * `threshold_ms` - Time in ms to consider a "long" press
 *
 * # Returns
 * SensorType.LongPress
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorLongPress(500);
 * ```
 */
export function sensor_long_press(threshold_ms: number): SensorType;

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * Create a mouse click sensor
 *
 * # Arguments
 * * `button` - Button name: 0=Left, 1=Right, 2=Middle
 *
 * # Returns
 * SensorType for use in brick chains
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorMouseClick(0); // Left
 * ```
 */
export function sensor_mouse_click(button: number): SensorType;

/**
 * Create a mouse drag sensor
 *
 * # Arguments
 * * `button` - Button name: 0=Left, 1=Right, 2=Middle
 *
 * # Returns
 * SensorType for drag detection
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorMouseDrag(0);
 * ```
 */
export function sensor_mouse_drag(_button: number): SensorType;

/**
 * Create a mouse hover sensor
 *
 * # Returns
 * SensorType.MouseOver
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorMouseHover();
 * ```
 */
export function sensor_mouse_hover(): SensorType;

/**
 * Create a mouse wheel sensor
 *
 * # Arguments
 * * `direction` - 1=Up, -1=Down
 *
 * # Returns
 * SensorType.Radar (mapped)
 *
 * # JavaScript Example
 * ```javascript
 * const sensorUp = sensorMouseWheel(1);
 * ```
 */
export function sensor_mouse_wheel(_direction: number): SensorType;

/**
 * Create a property change sensor
 *
 * # Arguments
 * * `property_id` - Property ID to monitor
 *
 * # Returns
 * SensorType for property changes
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorPropertyChanged(0);
 * ```
 */
export function sensor_property_changed(property_id: number): SensorType;

/**
 * Create a timer delay sensor
 *
 * # Arguments
 * * `ms` - Delay in milliseconds
 * * `once` - If true, only fires once
 *
 * # Returns
 * SensorType (delay)
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorTimerDelay(500, true);
 * ```
 */
export function sensor_timer_delay(ms: number, once: boolean): SensorType;

/**
 * Create a timer interval sensor
 *
 * # Arguments
 * * `ms` - Interval in milliseconds
 *
 * # Returns
 * SensorType (timer)
 *
 * # JavaScript Example
 * ```javascript
 * const sensor = sensorTimerInterval(1000); // Every second
 * ```
 */
export function sensor_timer_interval(ms: number): SensorType;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_jserror_free: (a: number, b: number) => void;
    readonly __wbg_wasmbridge_free: (a: number, b: number) => void;
    readonly jserror_message: (a: number) => [number, number];
    readonly jserror_new: (a: number, b: number) => number;
    readonly wasmbridge_add_sensor: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmbridge_batch_configure_entities: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number, q: number, r: number, s: number, t: number, u: number, v: number, w: number) => [number, number, number];
    readonly wasmbridge_batch_despawn: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmbridge_batch_set_colors: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmbridge_batch_set_positions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly wasmbridge_batch_set_shapes: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmbridge_batch_set_sizes: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly wasmbridge_batch_set_visibility: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmbridge_bulk_spawn: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly wasmbridge_can_redo: (a: number) => [number, number, number];
    readonly wasmbridge_can_undo: (a: number) => [number, number, number];
    readonly wasmbridge_clear: (a: number) => [number, number];
    readonly wasmbridge_clear_all_logic: (a: number) => [number, number];
    readonly wasmbridge_clear_color_tint: (a: number, b: number) => [number, number];
    readonly wasmbridge_clear_entity_logic: (a: number, b: number) => [number, number];
    readonly wasmbridge_clear_selection: (a: number) => [number, number];
    readonly wasmbridge_configure_entity: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => [number, number];
    readonly wasmbridge_configure_mouse_sensor: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_connection_count: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_create_ecb: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_delete_selected: (a: number) => [number, number];
    readonly wasmbridge_detect_available_backends: (a: number) => [number, number, number];
    readonly wasmbridge_duplicate_entity: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_entity_count: (a: number) => [number, number, number];
    readonly wasmbridge_execute_ecb: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_accumulator: (a: number) => number;
    readonly wasmbridge_get_active_color: (a: number) => [number, number, number, number];
    readonly wasmbridge_get_active_stroke_color: (a: number) => [number, number, number, number];
    readonly wasmbridge_get_active_stroke_width: (a: number) => [number, number, number];
    readonly wasmbridge_get_alive_entities: (a: number) => [number, number, number, number];
    readonly wasmbridge_get_audio_master_volume: (a: number) => [number, number, number];
    readonly wasmbridge_get_camera_center: (a: number) => [number, number, number];
    readonly wasmbridge_get_color: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_get_colors_ptr: (a: number) => number;
    readonly wasmbridge_get_ecs_component_count: (a: number) => number;
    readonly wasmbridge_get_ecs_entity_count: (a: number) => number;
    readonly wasmbridge_get_entity_color_hex: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_get_entity_count: (a: number) => [number, number, number];
    readonly wasmbridge_get_entity_label: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_get_entity_position_screen: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_entity_position_world: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_entity_shape: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_entity_size_screen: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_entity_size_world: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_entity_velocity: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_get_fixed_timestep: (a: number) => number;
    readonly wasmbridge_get_history_state: (a: number) => [number, number, number, number];
    readonly wasmbridge_get_input_buffer_ptr: (a: number) => number;
    readonly wasmbridge_get_input_buffer_size: () => number;
    readonly wasmbridge_get_max_entities: (a: number) => number;
    readonly wasmbridge_get_max_substeps: (a: number) => number;
    readonly wasmbridge_get_modifiers: (a: number) => [number, number, number];
    readonly wasmbridge_get_mouse_buttons: (a: number) => [number, number, number];
    readonly wasmbridge_get_mouse_position: (a: number) => [number, number, number, number];
    readonly wasmbridge_get_selection: (a: number) => [number, number, number];
    readonly wasmbridge_get_stroke_color: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_get_stroke_width: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_get_tool: (a: number) => [number, number, number, number];
    readonly wasmbridge_get_tool_index: (a: number) => [number, number, number];
    readonly wasmbridge_get_transforms_count: (a: number) => number;
    readonly wasmbridge_get_transforms_ptr: (a: number) => number;
    readonly wasmbridge_get_velocities_ptr: (a: number) => number;
    readonly wasmbridge_get_zoom: (a: number) => [number, number, number];
    readonly wasmbridge_init_audio: (a: number) => [number, number, number];
    readonly wasmbridge_initialize: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_initialize_graphics: (a: number, b: any) => [number, number];
    readonly wasmbridge_initialize_graphics_with_backend: (a: number, b: any, c: number, d: number) => [number, number];
    readonly wasmbridge_is_entity_selected: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_is_entity_visible: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_is_recovering: (a: number) => number;
    readonly wasmbridge_is_selected: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_load_scene: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmbridge_load_sound: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmbridge_move_entity: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_move_entity_by: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_new: () => number;
    readonly wasmbridge_on_key: (a: number, b: number, c: number, d: number) => void;
    readonly wasmbridge_on_mouse_down: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_on_mouse_move: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_on_mouse_up: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_on_wheel: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmbridge_play_beep: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_play_sound: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_poll_events: (a: number) => number;
    readonly wasmbridge_process_input_events: (a: number) => void;
    readonly wasmbridge_push_input_event: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly wasmbridge_query_all: (a: number) => [number, number, number, number];
    readonly wasmbridge_query_by_layer: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_query_by_min_size: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmbridge_query_by_selection: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_query_by_shape: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_query_by_shape_type: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_query_by_visibility: (a: number, b: number) => [number, number, number, number];
    readonly wasmbridge_query_color_components: (a: number) => [number, number, number, number];
    readonly wasmbridge_query_in_bounds: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly wasmbridge_query_shape_components: (a: number) => [number, number, number, number];
    readonly wasmbridge_query_visible_entities: (a: number) => [number, number, number, number];
    readonly wasmbridge_query_with_velocity: (a: number) => [number, number, number, number];
    readonly wasmbridge_redo: (a: number) => [number, number];
    readonly wasmbridge_remove_sensor: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_resize: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_select_entity: (a: number, b: number) => [number, number];
    readonly wasmbridge_serialize_project: (a: number) => [number, number, number];
    readonly wasmbridge_serialize_scene: (a: number) => [number, number, number, number];
    readonly wasmbridge_set_active_color: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly wasmbridge_set_active_stroke_color: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly wasmbridge_set_active_stroke_width: (a: number, b: number) => [number, number];
    readonly wasmbridge_set_audio_master_volume: (a: number, b: number) => [number, number];
    readonly wasmbridge_set_audio_muted: (a: number, b: number) => [number, number];
    readonly wasmbridge_set_camera_center: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_color: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly wasmbridge_set_color_tint: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly wasmbridge_set_entity_selected: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_entity_velocity: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_set_entity_visible: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_fixed_timestep: (a: number, b: number) => void;
    readonly wasmbridge_set_fixed_timestep_hz: (a: number, b: number) => void;
    readonly wasmbridge_set_label: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_set_master_volume: (a: number, b: number) => [number, number];
    readonly wasmbridge_set_max_substeps: (a: number, b: number) => void;
    readonly wasmbridge_set_position: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_set_selected: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_shape: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_size: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmbridge_set_stroke_color: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly wasmbridge_set_stroke_width: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_tool: (a: number, b: number, c: number) => [number, number];
    readonly wasmbridge_set_tool_by_type: (a: number, b: number) => [number, number];
    readonly wasmbridge_set_zoom: (a: number, b: number) => [number, number];
    readonly wasmbridge_spawn_entity: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmbridge_spawn_pool: (a: number, b: number) => [number, number, number];
    readonly wasmbridge_stop_sound: (a: number, b: number) => [number, number];
    readonly wasmbridge_tick: (a: number, b: number) => [number, number];
    readonly wasmbridge_undo: (a: number) => [number, number];
    readonly __wbg_animationclip_free: (a: number, b: number) => void;
    readonly __wbg_animationcomponent_free: (a: number, b: number) => void;
    readonly __wbg_effect_free: (a: number, b: number) => void;
    readonly __wbg_materialcomponent_free: (a: number, b: number) => void;
    readonly __wbg_materialconfig_free: (a: number, b: number) => void;
    readonly __wbg_posteffect_free: (a: number, b: number) => void;
    readonly __wbg_postprocesspipeline_free: (a: number, b: number) => void;
    readonly __wbg_textureatlascomponent_free: (a: number, b: number) => void;
    readonly animationclip_end_frame: (a: number) => number;
    readonly animationclip_fps: (a: number) => number;
    readonly animationclip_frame_count: (a: number) => number;
    readonly animationclip_frame_duration_ms: (a: number) => number;
    readonly animationclip_loop_clip: (a: number) => number;
    readonly animationclip_name: (a: number) => [number, number];
    readonly animationclip_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly animationclip_start_frame: (a: number) => number;
    readonly animationcomponent_clip_count: (a: number) => number;
    readonly animationcomponent_current: (a: number) => number;
    readonly animationcomponent_current_clip_name: (a: number) => [number, number];
    readonly animationcomponent_frame_count: (a: number) => number;
    readonly animationcomponent_frame_duration_ms: (a: number) => number;
    readonly animationcomponent_is_playing: (a: number) => number;
    readonly animationcomponent_loop_animation: (a: number) => number;
    readonly animationcomponent_new: (a: number, b: number) => number;
    readonly animationcomponent_new_single_shot: (a: number, b: number) => number;
    readonly animationcomponent_pause: (a: number) => void;
    readonly animationcomponent_play: (a: number) => void;
    readonly animationcomponent_play_clip: (a: number, b: number, c: number) => number;
    readonly animationcomponent_play_clip_by_index: (a: number, b: number) => number;
    readonly animationcomponent_reset: (a: number) => void;
    readonly animationcomponent_set_frame: (a: number, b: number) => void;
    readonly animationcomponent_tick: (a: number, b: number) => number;
    readonly animationcomponent_with_clips_json: (a: number, b: number) => number;
    readonly effect_bloom: (a: number, b: number, c: number) => number;
    readonly effect_color_grading: (a: number, b: number, c: number, d: number) => number;
    readonly effect_grayscale: (a: number) => number;
    readonly materialcomponent_alpha_cutoff: (a: number) => number;
    readonly materialcomponent_blend_mode: (a: number) => number;
    readonly materialcomponent_color_multiply: (a: number) => [number, number];
    readonly materialcomponent_default_material: () => number;
    readonly materialcomponent_emission: (a: number) => [number, number];
    readonly materialcomponent_new: (a: number) => number;
    readonly materialcomponent_set_blend_mode: (a: number, b: number) => void;
    readonly materialcomponent_set_color_multiply: (a: number, b: number, c: number) => void;
    readonly materialcomponent_set_emission: (a: number, b: number, c: number) => void;
    readonly materialcomponent_set_shader_id: (a: number, b: number) => void;
    readonly materialcomponent_with_blend_mode: (a: number, b: number) => number;
    readonly materialcomponent_with_color_multiply: (a: number, b: number, c: number) => number;
    readonly materialcomponent_with_shader: (a: number, b: number) => number;
    readonly materialconfig_color_multiply: (a: number) => [number, number];
    readonly materialconfig_emission: (a: number) => [number, number];
    readonly materialconfig_set_alpha_cutoff: (a: number, b: number) => void;
    readonly materialconfig_set_color_multiply: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly materialconfig_set_emission: (a: number, b: number, c: number, d: number) => void;
    readonly posteffect_effect_type: (a: number) => [number, number];
    readonly posteffect_param1: (a: number) => number;
    readonly posteffect_param2: (a: number) => number;
    readonly posteffect_param3: (a: number) => number;
    readonly postprocesspipeline_add_effect: (a: number, b: number) => void;
    readonly postprocesspipeline_clear: (a: number) => void;
    readonly postprocesspipeline_is_empty: (a: number) => number;
    readonly postprocesspipeline_is_enabled: (a: number) => number;
    readonly postprocesspipeline_len: (a: number) => number;
    readonly postprocesspipeline_new: () => number;
    readonly postprocesspipeline_remove_effect: (a: number, b: number) => void;
    readonly postprocesspipeline_set_enabled: (a: number, b: number) => void;
    readonly textureatlascomponent_columns: (a: number) => number;
    readonly textureatlascomponent_current_uv: (a: number) => [number, number];
    readonly textureatlascomponent_flip_x: (a: number) => number;
    readonly textureatlascomponent_flip_y: (a: number) => number;
    readonly textureatlascomponent_from_atlas: (a: number, b: number, c: number, d: number) => number;
    readonly textureatlascomponent_get_uv: (a: number, b: number) => [number, number];
    readonly textureatlascomponent_new: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly textureatlascomponent_set_flip_x: (a: number, b: number) => void;
    readonly textureatlascomponent_set_flip_y: (a: number, b: number) => void;
    readonly textureatlascomponent_set_sprite: (a: number, b: number) => void;
    readonly textureatlascomponent_sprite_count: (a: number) => number;
    readonly textureatlascomponent_sprite_height: (a: number) => number;
    readonly textureatlascomponent_sprite_width: (a: number) => number;
    readonly textureatlascomponent_texture_index: (a: number) => number;
    readonly posteffect_new: (a: number, b: number, c: number) => number;
    readonly materialconfig_new: () => number;
    readonly materialconfig_set_blend_mode: (a: number, b: number) => void;
    readonly materialconfig_set_shader_id: (a: number, b: number) => void;
    readonly materialconfig_blend_mode: (a: number) => number;
    readonly posteffect_color_grading: (a: number, b: number, c: number, d: number) => number;
    readonly materialcomponent_shader_id: (a: number) => number;
    readonly materialconfig_alpha_cutoff: (a: number) => number;
    readonly materialconfig_shader_id: (a: number) => number;
    readonly textureatlascomponent_current_sprite: (a: number) => number;
    readonly textureatlascomponent_rows: (a: number) => number;
    readonly posteffect_grayscale: (a: number) => number;
    readonly __wbg_logicbrickssystem_free: (a: number, b: number) => void;
    readonly logicbrickssystem_clear_creation: (a: number) => void;
    readonly logicbrickssystem_clear_drag_state: (a: number) => void;
    readonly logicbrickssystem_drag_count: (a: number) => number;
    readonly logicbrickssystem_event_buffer_len: (a: number) => number;
    readonly logicbrickssystem_get_active_tool: (a: number) => [number, number];
    readonly logicbrickssystem_get_creation_start_pos: (a: number) => number;
    readonly logicbrickssystem_get_selected_entities: (a: number) => any;
    readonly logicbrickssystem_is_creating: (a: number) => number;
    readonly logicbrickssystem_is_dragging: (a: number) => number;
    readonly logicbrickssystem_new: () => number;
    readonly logicbrickssystem_pending_command_count: (a: number) => number;
    readonly logicbrickssystem_poll_events: (a: number) => number;
    readonly logicbrickssystem_sample_input: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly logicbrickssystem_selection_count: (a: number) => number;
    readonly logicbrickssystem_set_active_tool: (a: number, b: number, c: number) => void;
    readonly logicbrickssystem_set_creation_start: (a: number, b: number, c: number) => void;
    readonly logicbrickssystem_has_events: (a: number) => number;
    readonly __wbg_actuatortypes_free: (a: number, b: number) => void;
    readonly __wbg_controllertypes_free: (a: number, b: number) => void;
    readonly __wbg_jsbehaviorblock_free: (a: number, b: number) => void;
    readonly __wbg_jsentitybuilder_free: (a: number, b: number) => void;
    readonly __wbg_sensortypes_free: (a: number, b: number) => void;
    readonly __wbg_shapetypes_free: (a: number, b: number) => void;
    readonly __wbg_signalbytewasm_free: (a: number, b: number) => void;
    readonly actuatortypes_animation: () => number;
    readonly actuatortypes_camera: () => number;
    readonly actuatortypes_delete: () => number;
    readonly actuatortypes_highlight: () => number;
    readonly actuatortypes_move_actuator: () => number;
    readonly actuatortypes_property: () => number;
    readonly actuatortypes_redo: () => number;
    readonly actuatortypes_select: () => number;
    readonly actuatortypes_undo: () => number;
    readonly jsbehaviorblock_actuator_type: (a: number) => number;
    readonly jsbehaviorblock_controller_type: (a: number) => number;
    readonly jsbehaviorblock_key_code: (a: number) => number;
    readonly jsbehaviorblock_new: (a: number, b: number) => number;
    readonly jsbehaviorblock_sensor_type: (a: number) => number;
    readonly jsentitybuilder_actuator: (a: number, b: number, c: number, d: number) => number;
    readonly jsentitybuilder_behavior: (a: number, b: number, c: number) => number;
    readonly jsentitybuilder_behavior_count: (a: number) => number;
    readonly jsentitybuilder_build: (a: number) => number;
    readonly jsentitybuilder_color: (a: number, b: number, c: number, d: number) => number;
    readonly jsentitybuilder_controller: (a: number, b: number, c: number) => number;
    readonly jsentitybuilder_end_behavior: (a: number) => number;
    readonly jsentitybuilder_entity_id: (a: number) => number;
    readonly jsentitybuilder_get_behavior: (a: number, b: number) => number;
    readonly jsentitybuilder_insert: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly jsentitybuilder_layer: (a: number, b: number) => number;
    readonly jsentitybuilder_new: (a: number) => number;
    readonly jsentitybuilder_position: (a: number, b: number, c: number) => number;
    readonly jsentitybuilder_sensor: (a: number, b: number, c: number) => number;
    readonly jsentitybuilder_stroke_width: (a: number, b: number) => number;
    readonly signalbytewasm_any_edge: (a: number) => number;
    readonly signalbytewasm_as_u8: (a: number) => number;
    readonly signalbytewasm_count_ones: (a: number) => number;
    readonly signalbytewasm_count_zeros: (a: number) => number;
    readonly signalbytewasm_from: (a: number) => number;
    readonly signalbytewasm_get_current: (a: number) => number;
    readonly signalbytewasm_get_history: (a: number) => number;
    readonly signalbytewasm_is_falling_edge: (a: number) => number;
    readonly signalbytewasm_is_rising_edge: (a: number) => number;
    readonly signalbytewasm_is_steady: (a: number, b: number) => number;
    readonly signalbytewasm_is_steady_low: (a: number, b: number) => number;
    readonly signalbytewasm_new: () => number;
    readonly signalbytewasm_push: (a: number, b: number) => void;
    readonly signalbytewasm_size: (a: number) => number;
    readonly jsentitybuilder_size: (a: number, b: number, c: number) => number;
    readonly signalbytewasm_is_steady_high: (a: number, b: number) => number;
    readonly jsentitybuilder_stroke: (a: number, b: number, c: number, d: number) => number;
    readonly controllertypes_and: () => number;
    readonly controllertypes_blinky: () => number;
    readonly controllertypes_debounce: () => number;
    readonly controllertypes_direct: () => number;
    readonly controllertypes_hysteresis: () => number;
    readonly controllertypes_not: () => number;
    readonly controllertypes_or: () => number;
    readonly controllertypes_threshold: () => number;
    readonly sensortypes_key_shortcut: () => number;
    readonly sensortypes_mouse_click: () => number;
    readonly sensortypes_mouse_over: () => number;
    readonly sensortypes_proximity: () => number;
    readonly sensortypes_radar: () => number;
    readonly sensortypes_ray: () => number;
    readonly sensortypes_right_click: () => number;
    readonly sensortypes_touch: () => number;
    readonly shapetypes_arc: () => number;
    readonly shapetypes_circle: () => number;
    readonly shapetypes_cylinder: () => number;
    readonly shapetypes_diamond: () => number;
    readonly shapetypes_ellipse: () => number;
    readonly shapetypes_line: () => number;
    readonly shapetypes_rectangle: () => number;
    readonly shapetypes_triangle: () => number;
    readonly jsentitybuilder_shape: (a: number, b: number) => number;
    readonly jsentitybuilder_visible: (a: number, b: number) => number;
    readonly __wbg_callbackid_free: (a: number, b: number) => void;
    readonly __wbg_callbackregistry_free: (a: number, b: number) => void;
    readonly __wbg_controller_free: (a: number, b: number) => void;
    readonly __wbg_logicmappingtablewasm_free: (a: number, b: number) => void;
    readonly callbackid_value: (a: number) => number;
    readonly callbackregistry_clear: (a: number) => void;
    readonly callbackregistry_event_callback_count: (a: number, b: number, c: number) => number;
    readonly callbackregistry_invoke: (a: number, b: number, c: number, d: any) => number;
    readonly callbackregistry_new: () => number;
    readonly callbackregistry_register: (a: number, b: any, c: number, d: number, e: number) => number;
    readonly callbackregistry_total_count: (a: number) => number;
    readonly callbackregistry_unregister: (a: number, b: number) => number;
    readonly callbackregistry_unregister_all: (a: number, b: number, c: number) => number;
    readonly controller_and: (a: number) => number;
    readonly controller_and_any: () => number;
    readonly controller_blinky: (a: number) => number;
    readonly controller_controller_type: (a: number) => number;
    readonly controller_custom: (a: number, b: number, c: number, d: number) => number;
    readonly controller_custom_code: (a: number) => [number, number];
    readonly controller_custom_name: (a: number) => [number, number];
    readonly controller_debounce: (a: number) => number;
    readonly controller_direct: () => number;
    readonly controller_float_param1: (a: number) => number;
    readonly controller_float_param2: (a: number) => number;
    readonly controller_has_secondary_sensor: (a: number) => number;
    readonly controller_hysteresis: (a: number, b: number) => number;
    readonly controller_is_custom: (a: number) => number;
    readonly controller_not: () => number;
    readonly controller_numeric_param: (a: number) => number;
    readonly controller_or: (a: number) => number;
    readonly controller_or_any: () => number;
    readonly controller_pattern: (a: number) => number;
    readonly controller_secondary_sensor: (a: number) => number;
    readonly controller_threshold: (a: number) => number;
    readonly logicmappingtablewasm_add_highlight: (a: number, b: number, c: number, d: number) => void;
    readonly logicmappingtablewasm_add_move: (a: number, b: number, c: number, d: number) => void;
    readonly logicmappingtablewasm_add_select: (a: number, b: number, c: number, d: number) => void;
    readonly logicmappingtablewasm_clear: (a: number) => void;
    readonly logicmappingtablewasm_clear_entity: (a: number, b: number) => void;
    readonly logicmappingtablewasm_connection_count: (a: number, b: number) => number;
    readonly logicmappingtablewasm_get_connected_entities: (a: number) => [number, number];
    readonly logicmappingtablewasm_has_connection: (a: number, b: number, c: number) => number;
    readonly logicmappingtablewasm_is_empty: (a: number) => number;
    readonly logicmappingtablewasm_new: () => number;
    readonly logicmappingtablewasm_remove_connection: (a: number, b: number, c: number) => void;
    readonly get_global_callback_registry: () => number;
    readonly __wbg_eventringbufferwasm_free: (a: number, b: number) => void;
    readonly __wbg_eventtype_free: (a: number, b: number) => void;
    readonly __wbg_jslogicevent_free: (a: number, b: number) => void;
    readonly eventringbufferwasm_drain: (a: number, b: number) => [number, number];
    readonly eventringbufferwasm_event_count: (a: number, b: number) => number;
    readonly eventringbufferwasm_has_events: (a: number, b: number) => number;
    readonly eventtype_box_selection_completed: () => number;
    readonly eventtype_drag_ended: () => number;
    readonly eventtype_drag_started: () => number;
    readonly eventtype_entity_selected: () => number;
    readonly eventtype_hover_changed: () => number;
    readonly eventtype_proximity_alert: () => number;
    readonly jslogicevent_data_1: (a: number) => number;
    readonly jslogicevent_data_2: (a: number) => number;
    readonly jslogicevent_data_3: (a: number) => number;
    readonly jslogicevent_entity_id: (a: number) => number;
    readonly jslogicevent_event_type: (a: number) => number;
    readonly jslogicevent_timestamp_us: (a: number) => bigint;
    readonly eventringbufferwasm_new: () => number;
    readonly __wbg_jsentitycommandbuffer_free: (a: number, b: number) => void;
    readonly __wbg_zerocopycommandbuffer_free: (a: number, b: number) => void;
    readonly jsentitycommandbuffer_capacity: (a: number) => number;
    readonly jsentitycommandbuffer_clear: (a: number) => void;
    readonly jsentitycommandbuffer_commands_count: (a: number) => number;
    readonly jsentitycommandbuffer_commands_ptr: (a: number) => number;
    readonly jsentitycommandbuffer_despawn: (a: number, b: number) => void;
    readonly jsentitycommandbuffer_is_empty: (a: number) => number;
    readonly jsentitycommandbuffer_new: (a: number) => number;
    readonly jsentitycommandbuffer_resize: (a: number, b: number, c: number, d: number) => void;
    readonly jsentitycommandbuffer_set_color: (a: number, b: number, c: number) => void;
    readonly jsentitycommandbuffer_set_layer: (a: number, b: number, c: number) => void;
    readonly jsentitycommandbuffer_set_selection: (a: number, b: number, c: number) => void;
    readonly jsentitycommandbuffer_set_shape: (a: number, b: number, c: number) => void;
    readonly jsentitycommandbuffer_set_velocity: (a: number, b: number, c: number, d: number) => void;
    readonly jsentitycommandbuffer_set_visible: (a: number, b: number, c: number) => void;
    readonly jsentitycommandbuffer_spawn: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly jsentitycommandbuffer_spawned_count: (a: number) => number;
    readonly jsentitycommandbuffer_spawned_ids_ptr: (a: number) => number;
    readonly jsentitycommandbuffer_teleport: (a: number, b: number, c: number, d: number) => void;
    readonly zerocopycommandbuffer_clear: (a: number) => void;
    readonly zerocopycommandbuffer_count: (a: number) => number;
    readonly zerocopycommandbuffer_data_ptr: (a: number) => number;
    readonly zerocopycommandbuffer_new: (a: number) => number;
    readonly zerocopycommandbuffer_set_count: (a: number, b: number) => void;
    readonly jsentitycommandbuffer_len: (a: number) => number;
    readonly __wbg_brickhandle_free: (a: number, b: number) => void;
    readonly __wbg_get_jslogiceventdata_data_1: (a: number) => number;
    readonly __wbg_get_jslogiceventdata_data_2: (a: number) => number;
    readonly __wbg_get_jslogiceventdata_data_3: (a: number) => number;
    readonly __wbg_get_jslogiceventdata_entity_id: (a: number) => number;
    readonly __wbg_get_jslogiceventdata_event_type: (a: number) => number;
    readonly __wbg_get_jslogiceventdata_timestamp_us: (a: number) => bigint;
    readonly __wbg_jslogiceventdata_free: (a: number, b: number) => void;
    readonly __wbg_logicsystemwasm_free: (a: number, b: number) => void;
    readonly __wbg_pulsewasm_free: (a: number, b: number) => void;
    readonly __wbg_set_jslogiceventdata_data_1: (a: number, b: number) => void;
    readonly __wbg_set_jslogiceventdata_data_2: (a: number, b: number) => void;
    readonly __wbg_set_jslogiceventdata_data_3: (a: number, b: number) => void;
    readonly __wbg_set_jslogiceventdata_entity_id: (a: number, b: number) => void;
    readonly __wbg_set_jslogiceventdata_event_type: (a: number, b: number) => void;
    readonly __wbg_set_jslogiceventdata_timestamp_us: (a: number, b: bigint) => void;
    readonly brickhandle_disable: (a: number) => void;
    readonly brickhandle_id: (a: number) => [number, number];
    readonly brickhandle_is_enabled: (a: number) => number;
    readonly brickhandle_new: (a: number, b: number) => number;
    readonly brickhandle_remove: (a: number) => void;
    readonly brickhandle_toggle: (a: number) => number;
    readonly logicsystemwasm_attach_behavior: (a: number, b: number, c: number) => void;
    readonly logicsystemwasm_behavior_count: (a: number) => number;
    readonly logicsystemwasm_behavior_has_events: (a: number, b: number) => number;
    readonly logicsystemwasm_create_behavior: (a: number, b: number, c: number, d: number) => number;
    readonly logicsystemwasm_detach_behavior: (a: number, b: number) => void;
    readonly logicsystemwasm_drain_events: (a: number) => [number, number];
    readonly logicsystemwasm_event_count: (a: number) => number;
    readonly logicsystemwasm_get_behavior_state: (a: number, b: number) => any;
    readonly logicsystemwasm_has_events: (a: number) => number;
    readonly logicsystemwasm_new: () => number;
    readonly logicsystemwasm_update: (a: number, b: bigint) => void;
    readonly pulsewasm_entity_id: (a: number) => number;
    readonly pulsewasm_new: (a: number, b: number, c: number, d: number) => number;
    readonly pulsewasm_sensor_id: (a: number) => number;
    readonly pulsewasm_state: (a: number) => number;
    readonly pulsewasm_timestamp: (a: number) => number;
    readonly brickhandle_enable: (a: number) => void;
    readonly logicsystemwasm_set_behavior_enabled: (a: number, b: number, c: number) => void;
    readonly __wbg_brickchainbuilder_free: (a: number, b: number) => void;
    readonly __wbg_cameraconfig_free: (a: number, b: number) => void;
    readonly __wbg_highlightconfig_free: (a: number, b: number) => void;
    readonly __wbg_moveconfig_free: (a: number, b: number) => void;
    readonly __wbg_propertyconfig_free: (a: number, b: number) => void;
    readonly __wbg_propertyvalue_free: (a: number, b: number) => void;
    readonly actuator_delete: () => number;
    readonly actuator_emit_event: (a: number, b: number, c: number, d: number) => number;
    readonly actuator_highlight: (a: number, b: number) => number;
    readonly actuator_move: (a: number, b: number, c: number) => number;
    readonly actuator_select_clear: () => number;
    readonly brickchainbuilder_actuator_count: (a: number) => number;
    readonly brickchainbuilder_actuator_highlight: (a: number, b: number, c: number) => number;
    readonly brickchainbuilder_actuator_move: (a: number, b: number, c: number, d: number) => number;
    readonly brickchainbuilder_actuator_select: (a: number, b: number) => number;
    readonly brickchainbuilder_and: (a: number) => number;
    readonly brickchainbuilder_connect: (a: number) => number;
    readonly brickchainbuilder_controller: (a: number, b: number) => number;
    readonly brickchainbuilder_controller_count: (a: number) => number;
    readonly brickchainbuilder_entity_id: (a: number) => number;
    readonly brickchainbuilder_invert: (a: number) => number;
    readonly brickchainbuilder_logic_bricks: (a: number, b: number, c: number) => number;
    readonly brickchainbuilder_name: (a: number) => [number, number];
    readonly brickchainbuilder_new: (a: number) => number;
    readonly brickchainbuilder_new_chain: (a: number) => number;
    readonly brickchainbuilder_or: (a: number) => number;
    readonly brickchainbuilder_sensor: (a: number, b: number) => number;
    readonly brickchainbuilder_sensor_count: (a: number) => number;
    readonly brickchainbuilder_sensor_key: (a: number, b: number) => number;
    readonly brickchainbuilder_with_mapping_table: (a: number, b: number) => number;
    readonly cameraconfig_duration_ms: (a: number) => number;
    readonly cameraconfig_new: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly cameraconfig_smooth: (a: number) => number;
    readonly cameraconfig_target_x: (a: number) => number;
    readonly cameraconfig_target_y: (a: number) => number;
    readonly cameraconfig_zoom: (a: number) => number;
    readonly factory_and: (a: number) => number;
    readonly factory_blinky: (a: number) => number;
    readonly factory_custom: (a: number, b: number, c: number, d: number) => number;
    readonly factory_debounce: (a: number) => number;
    readonly factory_direct: () => number;
    readonly factory_hysteresis: (a: number, b: number) => number;
    readonly factory_nand: () => number;
    readonly factory_nor: () => number;
    readonly factory_not: () => number;
    readonly factory_or: (a: number) => number;
    readonly factory_pattern: (a: number) => number;
    readonly factory_threshold: (a: number) => number;
    readonly factory_xor: () => number;
    readonly highlightconfig_color: (a: number) => number;
    readonly highlightconfig_new: (a: number, b: number, c: number) => number;
    readonly highlightconfig_restore_color: (a: number) => number;
    readonly moveconfig_constrain_x: (a: number) => number;
    readonly moveconfig_constrain_y: (a: number) => number;
    readonly moveconfig_new: (a: number, b: number, c: number) => number;
    readonly propertyconfig_new: (a: number, b: number, c: number) => number;
    readonly propertyconfig_property_name: (a: number) => [number, number];
    readonly propertyconfig_value: (a: number) => number;
    readonly propertyvalue_from_bool: (a: number) => number;
    readonly propertyvalue_from_number: (a: number) => number;
    readonly propertyvalue_from_string: (a: number, b: number) => number;
    readonly propertyvalue_value: (a: number) => [number, number];
    readonly sensor_collision_detect: (a: number) => number;
    readonly sensor_double_tap: () => number;
    readonly sensor_keyboard_key: (a: number, b: number) => number;
    readonly sensor_long_press: (a: number) => number;
    readonly sensor_mouse_click: (a: number) => number;
    readonly sensor_mouse_drag: (a: number) => number;
    readonly sensor_mouse_hover: () => number;
    readonly sensor_mouse_wheel: (a: number) => number;
    readonly sensor_timer_delay: (a: number, b: number) => number;
    readonly sensor_property_changed: (a: number) => number;
    readonly sensor_timer_interval: (a: number) => number;
    readonly actuator_select_multi: () => number;
    readonly actuator_select_single: () => number;
    readonly actuator_select_toggle: () => number;
    readonly highlightconfig_opacity: (a: number) => number;
    readonly moveconfig_snap: (a: number) => number;
    readonly wasm_bindgen__closure__destroy__h3f766b855f724bd7: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h3e58ee20b5cc64ab: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hf1efe96af13873e8: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
