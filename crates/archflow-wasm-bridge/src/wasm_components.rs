// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - ECS Components WASM Bindings
//
// Epic: EPIC-ECS-014, 015, 016, 017 - WASM Integration
// Provides JavaScript bindings for ECS components:
// - TextureAtlasComponent (EPIC-ECS-014)
// - AnimationComponent (EPIC-ECS-015)
// - MaterialComponent (EPIC-ECS-016)
// - PostProcessPipeline (EPIC-ECS-017)
//
// API Pattern: Components use factory constructors that can be inserted via .insert()
//
// Usage in JavaScript:
// ```javascript
// const entity = await bridge.world.spawn()
//     .insert(TextureAtlasComponent.new("sprites.png", 32, 32, 4, 4))
//     .insert(AnimationComponent.new(8, 100))
//     .insert(MaterialComponent.new({ colorMultiply: [1,1,1,1], ... }))
//     .build();
// ```
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

// Re-export ECS components from archflow-logic
use archflow_logic::ecs::components::{
    AnimationClip as LogicAnimationClip, AnimationComponent as LogicAnimationComponent,
    BlendMode as LogicBlendMode, MaterialComponent as LogicMaterialComponent,
    PostEffect as LogicPostEffect, PostProcessPipeline as LogicPostProcessPipeline,
    TextureAtlasComponent as LogicTextureAtlasComponent,
};

// ═══════════════════════════════════════════════════════════════════════════════════════
// TextureAtlasComponent WASM Bindings (EPIC-ECS-014)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Texture Atlas Component for sprite rendering
///
/// Use with JsEntityBuilder:
/// ```javascript
/// bridge.world.spawn()
///     .insert(TextureAtlasComponent.new(0, 32, 32, 4, 4))
///     .build();
/// ```
#[wasm_bindgen]
#[derive(Clone)]
pub struct TextureAtlasComponent {
    inner: LogicTextureAtlasComponent,
}

#[wasm_bindgen]
impl TextureAtlasComponent {
    /// Create a new texture atlas component
    ///
    /// # Arguments
    /// * `texture_index` - Index into the texture array
    /// * `sprite_width` - Width of each sprite in pixels
    /// * `sprite_height` - Height of each sprite in pixels
    /// * `columns` - Number of columns in the atlas
    /// * `rows` - Number of rows in the atlas
    #[wasm_bindgen(constructor)]
    pub fn new(
        texture_index: u16,
        sprite_width: u32,
        sprite_height: u32,
        columns: u32,
        rows: u32,
    ) -> Self {
        Self {
            inner: LogicTextureAtlasComponent::new(
                texture_index,
                sprite_width,
                sprite_height,
                columns,
                rows,
            ),
        }
    }

    /// Create from atlas ID with sprite index
    #[wasm_bindgen]
    pub fn from_atlas(atlas_id: u16, sprite_index: u32, columns: u32, rows: u32) -> Self {
        Self {
            inner: LogicTextureAtlasComponent::from_atlas(atlas_id, sprite_index, columns, rows),
        }
    }

    /// Get texture index
    #[wasm_bindgen]
    pub fn texture_index(&self) -> u16 {
        self.inner.texture_index
    }

    /// Get sprite width
    #[wasm_bindgen]
    pub fn sprite_width(&self) -> u32 {
        self.inner.sprite_width
    }

    /// Get sprite height
    #[wasm_bindgen]
    pub fn sprite_height(&self) -> u32 {
        self.inner.sprite_height
    }

    /// Get columns
    #[wasm_bindgen]
    pub fn columns(&self) -> u32 {
        self.inner.columns
    }

    /// Get rows
    #[wasm_bindgen]
    pub fn rows(&self) -> u32 {
        self.inner.rows
    }

    /// Get current sprite index
    #[wasm_bindgen]
    pub fn current_sprite(&self) -> u32 {
        self.inner.current_sprite
    }

    /// Set sprite index
    #[wasm_bindgen]
    pub fn set_sprite(&mut self, index: u32) {
        self.inner.set_sprite(index);
    }

    /// Get/Set flip horizontally
    #[wasm_bindgen]
    pub fn flip_x(&self) -> bool {
        self.inner.flip_x
    }

    #[wasm_bindgen]
    pub fn set_flip_x(&mut self, flip: bool) {
        self.inner.set_flip_x(flip);
    }

    /// Get/Set flip vertically
    #[wasm_bindgen]
    pub fn flip_y(&self) -> bool {
        self.inner.flip_y
    }

    #[wasm_bindgen]
    pub fn set_flip_y(&mut self, flip: bool) {
        self.inner.set_flip_y(flip);
    }

    /// Get UV coordinates for a sprite by index
    /// Returns array [u0, v0, u1, v1]
    #[wasm_bindgen]
    pub fn get_uv(&self, index: u32) -> Vec<f32> {
        let uv = self.inner.get_uv(index);
        uv.to_vec()
    }

    /// Get UV coordinates for current sprite
    #[wasm_bindgen]
    pub fn current_uv(&self) -> Vec<f32> {
        let uv = self.inner.current_uv();
        uv.to_vec()
    }

    /// Get total number of sprites in atlas
    #[wasm_bindgen]
    pub fn sprite_count(&self) -> u32 {
        self.inner.columns * self.inner.rows
    }
}

/// Convert to internal ECS component
impl From<TextureAtlasComponent> for LogicTextureAtlasComponent {
    fn from(wasm: TextureAtlasComponent) -> Self {
        wasm.inner
    }
}

impl AsRef<LogicTextureAtlasComponent> for TextureAtlasComponent {
    fn as_ref(&self) -> &LogicTextureAtlasComponent {
        &self.inner
    }
}

impl AsMut<LogicTextureAtlasComponent> for TextureAtlasComponent {
    fn as_mut(&mut self) -> &mut LogicTextureAtlasComponent {
        &mut self.inner
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AnimationClip WASM Bindings (EPIC-ECS-015)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Single animation sequence (clip)
///
/// Represents a named animation sequence like "idle", "walk", "run".
#[wasm_bindgen]
#[derive(Clone)]
pub struct AnimationClip {
    inner: LogicAnimationClip,
}

#[wasm_bindgen]
impl AnimationClip {
    /// Create a new animation clip
    ///
    /// # Arguments
    /// * `name` - Name of the clip (e.g., "idle", "walk")
    /// * `start_frame` - Starting frame index
    /// * `end_frame` - Ending frame index (inclusive)
    /// * `fps` - Frames per second
    /// * `loop` - Whether the clip loops
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, start_frame: u32, end_frame: u32, fps: u32, loop_clip: bool) -> Self {
        Self {
            inner: LogicAnimationClip::new(name, start_frame, end_frame, fps, loop_clip),
        }
    }

    /// Get clip name
    #[wasm_bindgen]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get start frame
    #[wasm_bindgen]
    pub fn start_frame(&self) -> u32 {
        self.inner.start_frame()
    }

    /// Get end frame
    #[wasm_bindgen]
    pub fn end_frame(&self) -> u32 {
        self.inner.end_frame()
    }

    /// Get FPS
    #[wasm_bindgen]
    pub fn fps(&self) -> u32 {
        self.inner.fps()
    }

    /// Get if loops
    #[wasm_bindgen]
    pub fn loop_clip(&self) -> bool {
        self.inner.loop_clip()
    }

    /// Get frame count
    #[wasm_bindgen]
    pub fn frame_count(&self) -> u32 {
        self.inner.frame_count()
    }

    /// Get frame duration in ms
    #[wasm_bindgen]
    pub fn frame_duration_ms(&self) -> u32 {
        self.inner.frame_duration_ms()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// AnimationComponent WASM Bindings (EPIC-ECS-015)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Animation Component for sprite animation
///
/// Use with JsEntityBuilder:
/// ```javascript
/// bridge.world.spawn()
///     .insert(AnimationComponent.new(8, 100)) // 8 frames, 100ms each
///     .build();
/// ```
#[wasm_bindgen]
#[derive(Clone)]
pub struct AnimationComponent {
    inner: LogicAnimationComponent,
}

#[wasm_bindgen]
impl AnimationComponent {
    /// Create a new animation component with single sequence
    ///
    /// # Arguments
    /// * `frame_count` - Total number of frames
    /// * `frame_duration_ms` - Duration of each frame in milliseconds
    #[wasm_bindgen(constructor)]
    pub fn new(frame_count: u32, frame_duration_ms: u32) -> Self {
        Self {
            inner: LogicAnimationComponent::new(frame_count, frame_duration_ms),
        }
    }

    /// Create with looping disabled (single-shot)
    #[wasm_bindgen]
    pub fn new_single_shot(frame_count: u32, frame_duration_ms: u32) -> Self {
        Self {
            inner: LogicAnimationComponent::new_single_shot(frame_count, frame_duration_ms),
        }
    }

    /// Create with multiple animation clips (JSON string format for WASM)
    /// Format: [{"name":"idle","start":0,"end":3,"fps":8,"loop":true},...]
    #[wasm_bindgen]
    pub fn with_clips_json(clips_json: &str) -> Self {
        // Parse JSON clips - simplified for WASM
        // For now, return empty clips if parsing fails
        let rust_clips = Self::parse_clips_json(clips_json);
        Self {
            inner: LogicAnimationComponent::with_clips(rust_clips),
        }
    }

    /// Parse clips from JSON string
    fn parse_clips_json(json: &str) -> Vec<LogicAnimationClip> {
        // Simple JSON parsing for clips
        // Expected format: [{"name":"idle","start":0,"end":3,"fps":8,"loop":true}]
        let mut clips = Vec::new();

        // Simple parsing - look for name, start, end, fps, loop
        if json.starts_with('[') {
            // Basic JSON array parsing
            for item in json
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split("},{")
            {
                let item = item.trim_start_matches('{').trim_end_matches('}');

                let mut name = String::new();
                let mut start = 0u32;
                let mut end = 0u32;
                let mut fps = 8u32;
                let mut loop_clip = true;

                for part in item.split(',') {
                    let part = part.trim();
                    if part.starts_with("\"name\"") {
                        if let Some(pos) = part.find(':') {
                            let value = &part[pos + 1..];
                            name = value.trim_matches('"').trim_matches(':').to_string();
                        }
                    } else if part.starts_with("\"start\"") {
                        if let Some(pos) = part.find(':') {
                            if let Ok(n) = part[pos + 1..].trim().parse::<u32>() {
                                start = n;
                            }
                        }
                    } else if part.starts_with("\"end\"") {
                        if let Some(pos) = part.find(':') {
                            if let Ok(n) = part[pos + 1..].trim().parse::<u32>() {
                                end = n;
                            }
                        }
                    } else if part.starts_with("\"fps\"") {
                        if let Some(pos) = part.find(':') {
                            if let Ok(n) = part[pos + 1..].trim().parse::<u32>() {
                                fps = n;
                            }
                        }
                    } else if part.starts_with("\"loop\"") {
                        if let Some(pos) = part.find(':') {
                            let value = part[pos + 1..].trim();
                            loop_clip = value == "true";
                        }
                    }
                }

                if !name.is_empty() {
                    clips.push(LogicAnimationClip::new(name, start, end, fps, loop_clip));
                }
            }
        }

        clips
    }

    /// Start playing
    #[wasm_bindgen]
    pub fn play(&mut self) {
        self.inner.play();
    }

    /// Pause playback
    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.inner.pause();
    }

    /// Reset to first frame
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Set a specific frame
    #[wasm_bindgen]
    pub fn set_frame(&mut self, frame: u32) {
        self.inner.set_frame(frame);
    }

    /// Update animation (call each frame)
    /// Returns Some(new_frame) if frame changed, None otherwise
    #[wasm_bindgen]
    pub fn tick(&mut self, delta_ms: u32) -> Option<u32> {
        self.inner.tick(delta_ms as u64)
    }

    /// Get current frame index
    #[wasm_bindgen]
    pub fn current(&self) -> u32 {
        self.inner.current()
    }

    /// Get frame count
    #[wasm_bindgen]
    pub fn frame_count(&self) -> u32 {
        self.inner.frame_count
    }

    /// Get frame duration in ms
    #[wasm_bindgen]
    pub fn frame_duration_ms(&self) -> u32 {
        self.inner.frame_duration_ms
    }

    /// Check if playing
    #[wasm_bindgen]
    pub fn is_playing(&self) -> bool {
        self.inner.is_playing
    }

    /// Check if loops
    #[wasm_bindgen]
    pub fn loop_animation(&self) -> bool {
        self.inner.loop_animation
    }

    /// Get number of clips
    #[wasm_bindgen]
    pub fn clip_count(&self) -> usize {
        self.inner.clip_count()
    }

    /// Get current clip name
    #[wasm_bindgen]
    pub fn current_clip_name(&self) -> Option<String> {
        self.inner.current_clip_name().map(|s| s.to_string())
    }

    /// Play a specific clip by index
    #[wasm_bindgen]
    pub fn play_clip_by_index(&mut self, index: usize) -> bool {
        self.inner.play_clip_by_index(index)
    }

    /// Play a specific clip by name
    #[wasm_bindgen]
    pub fn play_clip(&mut self, name: &str) -> bool {
        self.inner.play_clip(name)
    }
}

/// Convert to internal ECS component
impl From<AnimationComponent> for LogicAnimationComponent {
    fn from(wasm: AnimationComponent) -> Self {
        wasm.inner
    }
}

impl AsRef<LogicAnimationComponent> for AnimationComponent {
    fn as_ref(&self) -> &LogicAnimationComponent {
        &self.inner
    }
}

impl AsMut<LogicAnimationComponent> for AnimationComponent {
    fn as_mut(&mut self) -> &mut LogicAnimationComponent {
        &mut self.inner
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// BlendMode Enum WASM Bindings (EPIC-ECS-016)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Blend modes for material rendering
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    /// No blending - opaque
    Opaque,
    /// Alpha blending - standard transparency
    AlphaBlend,
    /// Additive blending - glow effect
    Add,
    /// Multiply blend - darkening
    Multiply,
}

impl From<BlendMode> for LogicBlendMode {
    fn from(mode: BlendMode) -> Self {
        match mode {
            BlendMode::Opaque => LogicBlendMode::Opaque,
            BlendMode::AlphaBlend => LogicBlendMode::AlphaBlend,
            BlendMode::Add => LogicBlendMode::Add,
            BlendMode::Multiply => LogicBlendMode::Multiply,
        }
    }
}

impl From<LogicBlendMode> for BlendMode {
    fn from(mode: LogicBlendMode) -> Self {
        match mode {
            LogicBlendMode::Opaque => BlendMode::Opaque,
            LogicBlendMode::AlphaBlend => BlendMode::AlphaBlend,
            LogicBlendMode::Add => BlendMode::Add,
            LogicBlendMode::Multiply => BlendMode::Multiply,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// MaterialComponent WASM Bindings (EPIC-ECS-016)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Material configuration for JavaScript (using getters/setters for WASM)
#[wasm_bindgen]
pub struct MaterialConfig {
    color_multiply: [f32; 4],
    emission: [f32; 3],
    alpha_cutoff: f32,
    blend_mode: BlendMode,
    shader_id: u32,
}

#[wasm_bindgen]
impl MaterialConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            color_multiply: [1.0, 1.0, 1.0, 1.0],
            emission: [0.0, 0.0, 0.0],
            alpha_cutoff: 0.0,
            blend_mode: BlendMode::Opaque,
            shader_id: 0,
        }
    }

    /// Get color multiply [r, g, b, a]
    #[wasm_bindgen]
    pub fn color_multiply(&self) -> Vec<f32> {
        self.color_multiply.to_vec()
    }

    /// Set color multiply [r, g, b, a]
    #[wasm_bindgen]
    pub fn set_color_multiply(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color_multiply = [r, g, b, a];
    }

    /// Get emission color [r, g, b]
    #[wasm_bindgen]
    pub fn emission(&self) -> Vec<f32> {
        self.emission.to_vec()
    }

    /// Set emission color [r, g, b]
    #[wasm_bindgen]
    pub fn set_emission(&mut self, r: f32, g: f32, b: f32) {
        self.emission = [r, g, b];
    }

    /// Get alpha cutoff
    #[wasm_bindgen]
    pub fn alpha_cutoff(&self) -> f32 {
        self.alpha_cutoff
    }

    /// Set alpha cutoff
    #[wasm_bindgen]
    pub fn set_alpha_cutoff(&mut self, value: f32) {
        self.alpha_cutoff = value;
    }

    /// Get blend mode
    #[wasm_bindgen]
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    /// Set blend mode
    #[wasm_bindgen]
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    /// Get shader ID
    #[wasm_bindgen]
    pub fn shader_id(&self) -> u32 {
        self.shader_id
    }

    /// Set shader ID
    #[wasm_bindgen]
    pub fn set_shader_id(&mut self, id: u32) {
        self.shader_id = id;
    }
}

impl Default for MaterialConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Material Component for rendering properties
///
/// Use with JsEntityBuilder:
/// ```javascript
/// bridge.world.spawn()
///     .insert(MaterialComponent.new({
///         colorMultiply: [1.0, 0.5, 0.5, 1.0],
///         emission: [0.2, 0.1, 0.0],
///         blendMode: BlendMode.AlphaBlend,
///     }))
///     .build();
/// ```
#[wasm_bindgen]
#[derive(Clone)]
pub struct MaterialComponent {
    inner: LogicMaterialComponent,
}

#[wasm_bindgen]
impl MaterialComponent {
    /// Create a new material component
    ///
    /// # Arguments
    /// * `config` - MaterialConfig object with color, emission, blend mode
    #[wasm_bindgen(constructor)]
    pub fn new(config: &MaterialConfig) -> Self {
        let color: [f32; 4] = config
            .color_multiply
            .as_slice()
            .try_into()
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let emission: [f32; 3] = config
            .emission
            .as_slice()
            .try_into()
            .unwrap_or([0.0, 0.0, 0.0]);

        Self {
            inner: LogicMaterialComponent::new(color, emission, config.blend_mode.into()),
        }
    }

    /// Create with default values
    #[wasm_bindgen]
    pub fn default_material() -> Self {
        Self {
            inner: LogicMaterialComponent::default_material(),
        }
    }

    /// Create with custom shader
    #[wasm_bindgen]
    pub fn with_shader(mut self, shader_id: u32) -> Self {
        self.inner = self.inner.with_shader(shader_id);
        self
    }

    /// Create with specific blend mode
    #[wasm_bindgen]
    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.inner = self.inner.with_blend_mode(mode.into());
        self
    }

    /// Create with color multiply
    #[wasm_bindgen]
    pub fn with_color_multiply(mut self, color: &[f32]) -> Self {
        if let Ok(color_array) = color.try_into() {
            self.inner = self.inner.with_color_multiply(color_array);
        }
        self
    }

    /// Get color multiply
    #[wasm_bindgen]
    pub fn color_multiply(&self) -> Vec<f32> {
        self.inner.color_multiply.to_vec()
    }

    /// Get emission
    #[wasm_bindgen]
    pub fn emission(&self) -> Vec<f32> {
        self.inner.emission.to_vec()
    }

    /// Get alpha cutoff
    #[wasm_bindgen]
    pub fn alpha_cutoff(&self) -> f32 {
        self.inner.alpha_cutoff
    }

    /// Get blend mode
    #[wasm_bindgen]
    pub fn blend_mode(&self) -> BlendMode {
        self.inner.blend_mode.into()
    }

    /// Get shader ID
    #[wasm_bindgen]
    pub fn shader_id(&self) -> u32 {
        self.inner.shader_id
    }

    /// Set color multiply
    #[wasm_bindgen]
    pub fn set_color_multiply(&mut self, color: &[f32]) {
        if let Ok(color_array) = color.try_into() {
            self.inner.color_multiply = color_array;
        }
    }

    /// Set emission
    #[wasm_bindgen]
    pub fn set_emission(&mut self, emission: &[f32]) {
        if let Ok(emission_array) = emission.try_into() {
            self.inner.emission = emission_array;
        }
    }

    /// Set blend mode
    #[wasm_bindgen]
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.inner.blend_mode = mode.into();
    }

    /// Set shader ID
    #[wasm_bindgen]
    pub fn set_shader_id(&mut self, shader_id: u32) {
        self.inner.shader_id = shader_id;
    }
}

/// Convert to internal ECS component
impl From<MaterialComponent> for LogicMaterialComponent {
    fn from(wasm: MaterialComponent) -> Self {
        wasm.inner
    }
}

impl AsRef<LogicMaterialComponent> for MaterialComponent {
    fn as_ref(&self) -> &LogicMaterialComponent {
        &self.inner
    }
}

impl AsMut<LogicMaterialComponent> for MaterialComponent {
    fn as_mut(&mut self) -> &mut LogicMaterialComponent {
        &mut self.inner
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PostEffect WASM Bindings (EPIC-ECS-017)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Post-processing effect types (WASM version using structs)
#[wasm_bindgen]
#[derive(Clone)]
pub struct PostEffect {
    /// Effect type: "bloom", "color_grading", "grayscale"
    effect_type: String,
    /// Parameter 1 (threshold/brightness/intensity)
    param1: f32,
    /// Parameter 2 (intensity/contrast)
    param2: f32,
    /// Parameter 3 (radius/saturation/temperature)
    param3: f32,
}

#[wasm_bindgen]
impl PostEffect {
    /// Create a bloom effect
    ///
    /// # Arguments
    /// * `threshold` - Minimum brightness to trigger bloom (0.0-1.0)
    /// * `intensity` - Bloom strength (0.0-2.0)
    /// * `radius` - Blur radius (0.0-1.0)
    #[wasm_bindgen(constructor)]
    pub fn new(threshold: f32, intensity: f32, radius: f32) -> Self {
        Self {
            effect_type: "bloom".to_string(),
            param1: threshold.clamp(0.0, 1.0),
            param2: intensity.clamp(0.0, 2.0),
            param3: radius.clamp(0.0, 1.0),
        }
    }

    /// Create a color grading effect
    ///
    /// # Arguments
    /// * `brightness` - Brightness adjustment (-1.0 to 1.0)
    /// * `contrast` - Contrast adjustment (0.0 to 2.0)
    /// * `saturation` - Saturation adjustment (0.0 to 2.0)
    /// * `temperature` - Color temperature (-1.0 to 1.0)
    #[wasm_bindgen]
    pub fn color_grading(
        brightness: f32,
        contrast: f32,
        saturation: f32,
        _temperature: f32,
    ) -> Self {
        Self {
            effect_type: "color_grading".to_string(),
            param1: brightness.clamp(-1.0, 1.0),
            param2: contrast.clamp(0.0, 2.0),
            param3: saturation.clamp(0.0, 2.0),
        }
    }

    /// Create a grayscale effect
    ///
    /// # Arguments
    /// * `intensity` - Grayscale intensity (0.0 to 1.0)
    #[wasm_bindgen]
    pub fn grayscale(intensity: f32) -> Self {
        Self {
            effect_type: "grayscale".to_string(),
            param1: intensity.clamp(0.0, 1.0),
            param2: 0.0,
            param3: 0.0,
        }
    }

    /// Get effect type
    #[wasm_bindgen]
    pub fn effect_type(&self) -> String {
        self.effect_type.clone()
    }

    /// Get param1 (threshold/brightness/intensity)
    #[wasm_bindgen]
    pub fn param1(&self) -> f32 {
        self.param1
    }

    /// Get param2 (intensity/contrast)
    #[wasm_bindgen]
    pub fn param2(&self) -> f32 {
        self.param2
    }

    /// Get param3 (radius/saturation/temperature)
    #[wasm_bindgen]
    pub fn param3(&self) -> f32 {
        self.param3
    }
}

/// Convert WASM PostEffect to internal LogicPostEffect
impl From<PostEffect> for LogicPostEffect {
    fn from(effect: PostEffect) -> Self {
        match effect.effect_type.as_str() {
            "bloom" => LogicPostEffect::bloom(effect.param1, effect.param2, effect.param3),
            "color_grading" => {
                LogicPostEffect::color_grading(effect.param1, effect.param2, effect.param3, 0.0)
            }
            "grayscale" => LogicPostEffect::grayscale(effect.param1),
            _ => LogicPostEffect::grayscale(0.0),
        }
    }
}

/// Effect factories for JavaScript (convenience)
#[wasm_bindgen]
pub struct Effect;

#[wasm_bindgen]
impl Effect {
    /// Create a bloom effect
    #[wasm_bindgen]
    pub fn bloom(threshold: f32, intensity: f32, radius: f32) -> PostEffect {
        PostEffect::new(threshold, intensity, radius)
    }

    /// Create a color grading effect
    #[wasm_bindgen]
    pub fn color_grading(
        brightness: f32,
        contrast: f32,
        saturation: f32,
        temperature: f32,
    ) -> PostEffect {
        PostEffect::color_grading(brightness, contrast, saturation, temperature)
    }

    /// Create a grayscale effect
    #[wasm_bindgen]
    pub fn grayscale(intensity: f32) -> PostEffect {
        PostEffect::grayscale(intensity)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// PostProcessPipeline WASM Bindings (EPIC-ECS-017)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Post-processing pipeline for screen-wide effects
///
/// Use:
/// ```javascript
/// const pipeline = bridge.postProcess();
/// pipeline.addEffect(Effect.bloom(0.8, 0.5, 0.5));
/// ```
#[wasm_bindgen]
#[derive(Clone)]
pub struct PostProcessPipeline {
    inner: LogicPostProcessPipeline,
}

#[wasm_bindgen]
impl PostProcessPipeline {
    /// Create a new post-process pipeline
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LogicPostProcessPipeline::new(),
        }
    }

    /// Add an effect to the pipeline
    #[wasm_bindgen]
    pub fn add_effect(&mut self, effect: PostEffect) {
        self.inner.add_effect(effect.into());
    }

    /// Remove an effect by index
    #[wasm_bindgen]
    pub fn remove_effect(&mut self, index: usize) {
        self.inner.remove_effect(index);
    }

    /// Clear all effects
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get number of effects
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Enable/disable pipeline
    #[wasm_bindgen]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.inner.set_enabled(enabled);
    }

    /// Check if enabled
    #[wasm_bindgen]
    pub fn is_enabled(&self) -> bool {
        self.inner.is_enabled()
    }
}

impl Default for PostProcessPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert to internal ECS component
impl From<PostProcessPipeline> for LogicPostProcessPipeline {
    fn from(wasm: PostProcessPipeline) -> Self {
        wasm.inner
    }
}

impl AsRef<LogicPostProcessPipeline> for PostProcessPipeline {
    fn as_ref(&self) -> &LogicPostProcessPipeline {
        &self.inner
    }
}

impl AsMut<LogicPostProcessPipeline> for PostProcessPipeline {
    fn as_mut(&mut self) -> &mut LogicPostProcessPipeline {
        &mut self.inner
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_atlas_creation() {
        let atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);
        assert_eq!(atlas.texture_index(), 0);
        assert_eq!(atlas.sprite_width(), 32);
        assert_eq!(atlas.columns(), 4);
        assert_eq!(atlas.rows(), 4);
        assert_eq!(atlas.sprite_count(), 16);
    }

    #[test]
    fn test_texture_atlas_uv() {
        let atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);
        let uv = atlas.get_uv(0);
        assert_eq!(uv.len(), 4);
        // First sprite should be at u=0, v=0
        assert_eq!(uv[0], 0.0);
        assert_eq!(uv[1], 0.0);
    }

    #[test]
    fn test_animation_creation() {
        let anim = AnimationComponent::new(8, 100);
        assert_eq!(anim.frame_count(), 8);
        assert_eq!(anim.frame_duration_ms(), 100);
        assert!(!anim.is_playing());
    }

    #[test]
    fn test_animation_play() {
        let mut anim = AnimationComponent::new(8, 100);
        anim.play();
        assert!(anim.is_playing());
        anim.pause();
        assert!(!anim.is_playing());
    }

    #[test]
    fn test_animation_tick() {
        let mut anim = AnimationComponent::new(4, 50); // 4 frames, 50ms each
        anim.play();
        let frame1 = anim.tick(50);
        assert!(frame1.is_some());
        assert_eq!(anim.current(), 1);
    }

    #[test]
    fn test_animation_clip() {
        let clip = AnimationClip::new("idle", 0, 3, 8, true);
        assert_eq!(clip.name(), "idle");
        assert_eq!(clip.start_frame(), 0);
        assert_eq!(clip.end_frame(), 3);
        assert_eq!(clip.frame_count(), 4);
    }

    #[test]
    fn test_material_creation() {
        let config = MaterialConfig::new();
        let material = MaterialComponent::new(&config);
        assert_eq!(material.blend_mode(), BlendMode::Opaque);
    }

    #[test]
    fn test_material_with_blend() {
        let mut material = MaterialComponent::default_material();
        material.set_blend_mode(BlendMode::Add);
        assert_eq!(material.blend_mode(), BlendMode::Add);
    }

    #[test]
    fn test_blend_mode_conversion() {
        let mode = BlendMode::AlphaBlend;
        let rust_mode: LogicBlendMode = mode.into();
        let back: BlendMode = rust_mode.into();
        assert_eq!(back, BlendMode::AlphaBlend);
    }

    #[test]
    fn test_post_process_pipeline() {
        let mut pipeline = PostProcessPipeline::new();
        assert!(pipeline.is_empty());

        pipeline.add_effect(Effect::bloom(0.8, 0.5, 0.5));
        assert_eq!(pipeline.len(), 1);

        pipeline.remove_effect(0);
        assert!(pipeline.is_empty());
    }

    #[test]
    fn test_post_process_enabled() {
        let mut pipeline = PostProcessPipeline::new();
        assert!(pipeline.is_enabled());

        pipeline.set_enabled(false);
        assert!(!pipeline.is_enabled());
    }

    #[test]
    fn test_effect_factories() {
        let bloom = Effect::bloom(0.8, 0.5, 0.5);
        let _color = Effect::color_grading(0.0, 1.0, 1.0, 0.0);
        let _gray = Effect::grayscale(0.5);

        // Verify bloom effect
        assert_eq!(bloom.effect_type(), "bloom");
        assert_eq!(bloom.param1(), 0.8);
        assert_eq!(bloom.param2(), 0.5);
        assert_eq!(bloom.param3(), 0.5);
    }

    #[test]
    fn test_post_effect_types() {
        let bloom = PostEffect::new(0.8, 0.5, 0.5);
        assert_eq!(bloom.effect_type(), "bloom");

        let color = PostEffect::color_grading(0.1, 1.2, 0.9, 0.0);
        assert_eq!(color.effect_type(), "color_grading");

        let gray = PostEffect::grayscale(0.5);
        assert_eq!(gray.effect_type(), "grayscale");
    }
}
