// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow MTSDF Text Shader
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9, 12
//
// Multi-channel Signed Distance Field (MTSDF) text rendering:
// - Uses pre-generated MTSDF atlas for crisp text at any size
// - Multi-channel SDF for better corner and edge rendering
// - Supports color tinting
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
    @location(1) color: vec4<f32>,
};

struct CameraUniforms {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read> instances: array<InstanceInput>;
@group(0) @binding(2) var text_atlas: texture_2d<f32>;
@group(0) @binding(3) var text_sampler: sampler;

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

    // Calculate UV coordinates from instance uv_rect
    let uv_rect = instance.uv_rect;
    let uv = uv_rect.xy + quad_vert * uv_rect.zw;

    var output: VertexOutput;
    output.clip_pos = camera.view_projection * vec4<f32>(world_pos, 0.0, 1.0);
    output.uv = uv;
    output.color = unpack4x8unorm(instance.color);

    return output;
}

// MTSDF median calculation for improved text quality
fn median(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample MTSDF from texture atlas
    // MTSDF stores signed distance in RGB channels
    let msdf = textureSample(text_atlas, text_sampler, input.uv);

    // Calculate median of RGB channels for the signed distance
    let signed_dist = median(msdf.r, msdf.g, msdf.b) * 2.0 - 1.0;

    // Anti-aliasing using smoothstep with fwidth
    let edge_width = fwidth(signed_dist);
    let alpha = 1.0 - smoothstep(-edge_width, edge_width, signed_dist);

    // Apply color tint with calculated alpha
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
