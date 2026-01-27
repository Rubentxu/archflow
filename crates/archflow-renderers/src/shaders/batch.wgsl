// Vertex shader for 2D batch rendering with instancing
//
// This shader renders quads with per-instance transformation and color.
// Each instance consists of a model matrix and color passed as vertex attributes.

struct VertexInput {
    @location(0) position: vec2<f32>,  // Quad vertex position (unit quad)
    @location(1) instance_matrix_0: vec4<f32>,  // Column 0 of model matrix
    @location(2) instance_matrix_1: vec4<f32>,  // Column 1 of model matrix
    @location(3) instance_matrix_2: vec4<f32>,  // Column 2 of model matrix
    @location(4) instance_matrix_3: vec4<f32>,  // Column 3 of model matrix
    @location(5) instance_color: vec4<f32>,      // Per-instance color
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Reconstruct the model matrix from instance attributes
    let model_matrix = mat4x4<f32>(
        input.instance_matrix_0,
        input.instance_matrix_1,
        input.instance_matrix_2,
        input.instance_matrix_3
    );

    // Transform position: model matrix * position (z=0, w=1)
    let world_pos = model_matrix * vec4<f32>(input.position, 0.0, 1.0);

    // Output position in clip space (assuming orthographic projection set elsewhere)
    output.position = world_pos;
    output.color = input.instance_color;

    return output;
}

// Fragment shader
@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
