// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Image Array Shader
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// Texture2D array rendering for images:
// - Bindless texture array for multiple images
// - Texture index from instance determines array layer
// - Supports color tinting via multiply
// ═══════════════════════════════════════════════════════════════════════════════

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct InstanceInput {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: u32,
    @location(3) texture_index: u32,
    @location(4) uv_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: u32,
    @location(2) color: vec4<f32>,
};

struct CameraUniforms {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read> instances: array<InstanceInput>;
@group(0) @binding(2) var image_array: texture_2d_array<f32>;
@group(0) @binding(3) var image_sampler: sampler;

const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
);

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let instance_idx = input.vertex_index / 6u;
    let vert_idx = input.vertex_index % 6u;

    let instance = instances[instance_idx];
    let quad_vert = QUAD_VERTICES[vert_idx];

    // Calculate world position
    let half_size = instance.size * 0.5;
    let world_pos = instance.pos + (quad_vert - 0.5) * instance.size;

    // Calculate UV coordinates (full texture for images)
    let uv = quad_vert;

    // Texture index determines array layer (offset by 1000 for images)
    let layer = instance.texture_index - 1000u;

    var output: VertexOutput;
    output.clip_pos = camera.view_projection * vec4<f32>(world_pos, 0.0, 1.0);
    output.uv = uv;
    output.layer = layer;
    output.color = unpack4x8unorm(instance.color);

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample from texture array at the specified layer
    let tex_color = textureSample(image_array, image_sampler, input.uv, input.layer);

    // Tint the image with the instance color
    let tinted = tex_color * input.color;

    return tinted;
}
