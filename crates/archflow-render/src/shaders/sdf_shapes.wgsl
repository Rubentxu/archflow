// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow SDF Shapes Shader
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// SDF-based rendering for 2D shapes:
// - Rectangle with rounded corners
// - Circle/Ellipse
// - Line
// - All shapes use signed distance fields for crisp edges at any zoom
// ═══════════════════════════════════════════════════════════════════════════════

// Vertex shader - common to all phases
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct InstanceInput {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: u32,
    @location(3) shape_type: u32,
    @location(4) stroke_color: u32,
    @location(5) stroke_width_bits: u32,
    @location(6) uv_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) instance_pos: vec2<f32>,
    @location(2) instance_size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) shape_type: u32,
    @location(5) stroke_color: vec4<f32>,
    @location(6) stroke_width: f32,
};

struct CameraUniforms {
    view_projection: mat4x4<f32>,
    camera_pos: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read> instances: array<InstanceInput>;

const QUAD_VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),  // Bottom-left
    vec2<f32>(1.0, 0.0),  // Bottom-right
    vec2<f32>(0.0, 1.0),  // Top-left
    vec2<f32>(0.0, 1.0),  // Top-left
    vec2<f32>(1.0, 0.0),  // Bottom-right
    vec2<f32>(1.0, 1.0),  // Top-right
);

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let instance_idx = input.vertex_index / 6u;
    let vert_idx = input.vertex_index % 6u;

    let instance = instances[instance_idx];
    let quad_vert = QUAD_VERTICES[vert_idx];

    let stroke_width = bitcast<f32>(instance.stroke_width_bits);
    let expansion = stroke_width + 2.0;
    
    // Relative Rendering: Calculate position relative to camera center
    let relative_instance_pos = instance.pos - camera.camera_pos;
    
    // Apply expanded vertex offset
    let expanded_offset = (quad_vert - 0.5) * (instance.size + 2.0 * expansion);
    let final_pos = relative_instance_pos + expanded_offset;
    let world_pos = instance.pos + expanded_offset;

    var output: VertexOutput;
    output.clip_pos = camera.view_projection * vec4<f32>(final_pos, 0.0, 1.0);
    output.world_pos = world_pos;
    output.instance_pos = instance.pos;
    output.instance_size = instance.size;
    output.color = unpack4x8unorm(instance.color);
    output.shape_type = instance.shape_type;
    output.stroke_color = unpack4x8unorm(instance.stroke_color);
    output.stroke_width = stroke_width;

    return output;
}

// Fragment shader - SDF-based shape rendering
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let local_pos = (input.world_pos - input.instance_pos) / input.instance_size;
    let centered = local_pos;  // Already centered from Vertex Shader logic

    var distance: f32 = 0.0;
    let shape_type = input.shape_type & 0xFu;  // Lower 4 bits for shape type

    // Shape type constants
    let SHAPE_RECT: u32 = 0u;
    let SHAPE_CIRCLE: u32 = 1u;
    let SHAPE_ELLIPSE: u32 = 2u;
    let SHAPE_LINE: u32 = 3u;
    let SHAPE_ROUNDED_RECT: u32 = 4u;

    if (shape_type == SHAPE_RECT) {
        // Rectangle SDF
        let half_size = input.instance_size * 0.5;
        let d = abs(centered * input.instance_size) - half_size;
        distance = max(d.x, d.y);
    }
    else if (shape_type == SHAPE_CIRCLE) {
        // Circle SDF
        distance = length(centered * input.instance_size) - (min(input.instance_size.x, input.instance_size.y) * 0.5);
    }
    else if (shape_type == SHAPE_ELLIPSE) {
        // Ellipse SDF (approximation)
        let ab = input.instance_size * 0.5;
        let p = centered * input.instance_size;
        distance = length(p - ab * 0.5) - min(ab.x, ab.y) * 0.5;
    }
    else if (shape_type == SHAPE_ROUNDED_RECT) {
        // Rounded rectangle SDF
        let radius = (input.shape_type >> 8u) & 0xFFu;  // Radius in next byte
        let radius_f = f32(radius) / 255.0 * min(input.instance_size.x, input.instance_size.y) * 0.5;

        let half_size = (input.instance_size * 0.5) - vec2<f32>(radius_f, radius_f);
        let d = abs(centered * input.instance_size) - half_size;
        let outer_dist = min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0, 0.0)));
        distance = outer_dist - radius_f;
    }
    else {
        // Default to rectangle
        let half_size = input.instance_size * 0.5;
        let d = abs(centered * input.instance_size) - half_size;
        distance = max(d.x, d.y);
    }

    // Anti-aliased edge using smoothstep
    let edge_width = fwidth(distance);
    let fill_alpha = 1.0 - smoothstep(-edge_width, edge_width, distance);

    // Stroke calculation
    let stroke_distance = abs(distance) - input.stroke_width;
    let stroke_alpha = 1.0 - smoothstep(-edge_width, edge_width, stroke_distance);

    // Combine fill and stroke
    let fill_color = input.color * fill_alpha;
    let stroke_color_final = input.stroke_color * stroke_alpha;

    // Stroke on top of fill (mix using the inverse of fill alpha to prevent overlap artifacts)
    return mix(fill_color, stroke_color_final, stroke_alpha * (1.0 - fill_alpha));
}
