// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Shader E2E Rendering Tests
//
// End-to-end tests for shader rendering validation using headless WebGL2.
// These tests verify that shapes (circles, rectangles) render correctly with SDF.
//
// Testing Strategy:
// - WASM-only tests (require wasm-bindgen-test)
// - Render to canvas using WebGL2
// - Capture framebuffer pixels
// - Compare against golden images with tolerance
// - Detect regressions in shader output
//
// Run with: wasm-pack test --headless --firefox
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════════════
// Test Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a test canvas element with specified dimensions.
fn create_test_canvas(width: u32, height: u32) -> HtmlCanvasElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    canvas.set_width(width);
    canvas.set_height(height);

    canvas
}

/// Gets WebGL2 context from canvas.
fn get_webgl2_context(canvas: &HtmlCanvasElement) -> WebGl2RenderingContext {
    canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap()
}

/// Captures the current framebuffer as RGBA bytes.
fn capture_framebuffer(gl: &WebGl2RenderingContext, width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    gl.read_pixels_with_opt_u8_array(
        0,
        0,
        width as i32,
        height as i32,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(&mut pixels),
    )
    .unwrap();

    pixels
}

/// Compares two RGBA images with tolerance.
/// Returns (passed, matching_pixels, total_pixels, max_diff).
fn compare_images(expected: &[u8], actual: &[u8], tolerance: f64) -> (bool, usize, usize, f64) {
    assert_eq!(expected.len(), actual.len(), "Image sizes must match");

    let pixel_count = expected.len() / 4;
    let mut matching = 0;
    let mut max_diff = 0.0;

    for i in 0..pixel_count {
        let idx = i * 4;

        // Calculate RGB color distance (ignore alpha for now)
        let dr = (expected[idx] as f64 - actual[idx] as f64).abs();
        let dg = (expected[idx + 1] as f64 - actual[idx + 1] as f64).abs();
        let db = (expected[idx + 2] as f64 - actual[idx + 2] as f64).abs();

        let distance = (dr * dr + dg * dg + db * db).sqrt();
        max_diff = max_diff.max(distance);

        if distance <= tolerance {
            matching += 1;
        }
    }

    let passed = matching == pixel_count;
    (passed, matching, pixel_count, max_diff)
}

/// Compiles a shader program from vertex and fragment shader sources.
fn compile_shader_program(
    gl: &WebGl2RenderingContext,
    vs_source: &str,
    fs_source: &str,
) -> web_sys::WebGlProgram {
    // Compile vertex shader
    let vs = gl
        .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
        .unwrap();
    gl.shader_source(&vs, vs_source);
    gl.compile_shader(&vs);

    if !gl
        .get_shader_parameter(&vs, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap()
    {
        let log = gl.get_shader_info_log(&vs).unwrap_or_default();
        panic!("Vertex shader compilation failed: {}", log);
    }

    // Compile fragment shader
    let fs = gl
        .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
        .unwrap();
    gl.shader_source(&fs, fs_source);
    gl.compile_shader(&fs);

    if !gl
        .get_shader_parameter(&fs, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap()
    {
        let log = gl.get_shader_info_log(&fs).unwrap_or_default();
        panic!("Fragment shader compilation failed: {}", log);
    }

    // Link program
    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vs);
    gl.attach_shader(&program, &fs);
    gl.link_program(&program);

    if !gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap()
    {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        panic!("Shader program linking failed: {}", log);
    }

    program
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shader Sources (copied from webgl2_renderer_real.rs)
// ═══════════════════════════════════════════════════════════════════════════════

const VERTEX_SHADER_SOURCE: &str = "#version 300 es
    layout(location = 0) in vec2 a_position;
    layout(location = 1) in vec2 a_instance_pos;
    layout(location = 2) in vec2 a_instance_size;
    layout(location = 3) in vec4 a_instance_color;
    layout(location = 4) in float a_shape_type;
    uniform mat4 u_view_projection;
    uniform vec2 u_camera_pos;
    out vec4 v_color;
    out vec2 v_world_pos;
    out vec2 v_instance_pos;
    out vec2 v_instance_size;
    flat out float v_shape_type;
    void main() {
        vec2 centered_vert = a_position - 0.5;
        vec2 world_pos = a_instance_pos + centered_vert * a_instance_size;
        vec2 camera_relative_pos = world_pos - u_camera_pos;
        gl_Position = u_view_projection * vec4(camera_relative_pos, 0.0, 1.0);
        v_color = a_instance_color;
        v_world_pos = world_pos;
        v_instance_pos = a_instance_pos;
        v_instance_size = a_instance_size;
        v_shape_type = a_shape_type;
    }";

const FRAGMENT_SHADER_SOURCE: &str = "#version 300 es
    precision highp float;
    in vec4 v_color;
    in vec2 v_world_pos;
    in vec2 v_instance_pos;
    in vec2 v_instance_size;
    flat in float v_shape_type;
    out vec4 frag_color;

    void main() {
        vec2 local_pos = (v_world_pos - v_instance_pos) / v_instance_size;
        vec2 centered = local_pos;

        float distance = 0.0;
        int shape_type = int(v_shape_type) & 15;

        const int SHAPE_RECT = 0;
        const int SHAPE_CIRCLE = 1;
        const int SHAPE_ELLIPSE = 2;

        if (shape_type == SHAPE_RECT) {
            vec2 half_size = v_instance_size * 0.5;
            vec2 d = abs(centered * v_instance_size) - half_size;
            distance = max(d.x, d.y);
        }
        else if (shape_type == SHAPE_CIRCLE) {
            distance = length(centered * v_instance_size) - (min(v_instance_size.x, v_instance_size.y) * 0.5);
        }
        else {
            vec2 half_size = v_instance_size * 0.5;
            vec2 d = abs(centered * v_instance_size) - half_size;
            distance = max(d.x, d.y);
        }

        float edge_width = fwidth(distance);
        float alpha = 1.0 - smoothstep(-edge_width, edge_width, distance);

        frag_color = vec4(v_color.rgb, v_color.a * alpha);
    }";

// ═══════════════════════════════════════════════════════════════════════════════
// E2E Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
fn test_shader_compilation_succeeds() {
    let canvas = create_test_canvas(256, 256);
    let gl = get_webgl2_context(&canvas);

    // Should compile without errors
    let _program = compile_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

    // If we get here, compilation succeeded
    assert!(true, "Shader compilation successful");
}

#[wasm_bindgen_test]
fn test_render_single_red_rectangle() {
    let canvas = create_test_canvas(256, 256);
    let gl = get_webgl2_context(&canvas);
    let program = compile_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

    // Clear to black
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    gl.use_program(Some(&program));

    // Create a simple identity projection matrix (orthographic -1 to 1)
    let identity_matrix: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    let u_view_proj_loc = gl.get_uniform_location(&program, "u_view_projection");
    gl.uniform_matrix4fv_with_f32_array(u_view_proj_loc.as_ref(), false, &identity_matrix);

    let u_camera_pos_loc = gl.get_uniform_location(&program, "u_camera_pos");
    gl.uniform2f(u_camera_pos_loc.as_ref(), 0.0, 0.0);

    // Create quad vertices (unit square)
    let vertices: [f32; 8] = [
        0.0, 0.0, // bottom-left
        1.0, 0.0, // bottom-right
        0.0, 1.0, // top-left
        1.0, 1.0, // top-right
    ];

    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);

    // Create instance data for a red rectangle at center
    // GpuInstance: pos(8) + size(8) + color(4) + shape_type(4) + padding(8) + uv_rect(16) = 48 bytes
    let instance_data: Vec<u8> = {
        let mut data = Vec::new();

        // pos: vec2<f32> = [0.0, 0.0] (center)
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // size: vec2<f32> = [0.5, 0.5] (half screen)
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        // color: u32 = 0xFF0000FF (red, full alpha in ABGR format)
        data.extend_from_slice(&0xFF0000FFu32.to_le_bytes());

        // shape_type: u32 = 0 (rectangle)
        data.extend_from_slice(&0u32.to_le_bytes());

        // padding: [u32; 2]
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // uv_rect: [f32; 4]
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        data
    };

    let instance_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&instance_buffer));
    unsafe {
        let array = js_sys::Uint8Array::view(&instance_data);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    // Setup instance attributes
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_with_i32(1, 2, WebGl2RenderingContext::FLOAT, false, 48, 0);
    gl.vertex_attrib_divisor(1, 1);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_with_i32(2, 2, WebGl2RenderingContext::FLOAT, false, 48, 8);
    gl.vertex_attrib_divisor(2, 1);

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_with_i32(3, 4, WebGl2RenderingContext::UNSIGNED_BYTE, true, 48, 16);
    gl.vertex_attrib_divisor(3, 1);

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::UNSIGNED_INT, false, 48, 20);
    gl.vertex_attrib_divisor(4, 1);

    // Enable blending
    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    // Draw
    gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, 1);

    // Capture pixels
    let pixels = capture_framebuffer(&gl, 256, 256);

    // Verify center pixel is red-ish (allowing for anti-aliasing)
    let center_idx = (128 * 256 + 128) * 4;
    let r = pixels[center_idx];
    let g = pixels[center_idx + 1];
    let b = pixels[center_idx + 2];

    // Red channel should be high, green and blue low
    assert!(r > 200, "Red channel should be high, got {}", r);
    assert!(g < 50, "Green channel should be low, got {}", g);
    assert!(b < 50, "Blue channel should be low, got {}", b);
}

#[wasm_bindgen_test]
fn test_render_single_blue_circle() {
    let canvas = create_test_canvas(256, 256);
    let gl = get_webgl2_context(&canvas);
    let program = compile_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.use_program(Some(&program));

    // Setup uniforms
    let identity_matrix: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    let u_view_proj_loc = gl.get_uniform_location(&program, "u_view_projection");
    gl.uniform_matrix4fv_with_f32_array(u_view_proj_loc.as_ref(), false, &identity_matrix);

    let u_camera_pos_loc = gl.get_uniform_location(&program, "u_camera_pos");
    gl.uniform2f(u_camera_pos_loc.as_ref(), 0.0, 0.0);

    // Setup vertex buffer
    let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);

    // Create instance data for a BLUE CIRCLE at center
    let instance_data: Vec<u8> = {
        let mut data = Vec::new();

        // pos: [0.0, 0.0]
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());

        // size: [0.5, 0.5]
        data.extend_from_slice(&0.5f32.to_le_bytes());
        data.extend_from_slice(&0.5f32.to_le_bytes());

        // color: u32 = 0xFFFF0000 (blue in ABGR format)
        data.extend_from_slice(&0xFFFF0000u32.to_le_bytes());

        // shape_type: u32 = 1 (CIRCLE)
        data.extend_from_slice(&1u32.to_le_bytes());

        // padding
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // uv_rect
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        data
    };

    let instance_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&instance_buffer));
    unsafe {
        let array = js_sys::Uint8Array::view(&instance_data);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_with_i32(1, 2, WebGl2RenderingContext::FLOAT, false, 48, 0);
    gl.vertex_attrib_divisor(1, 1);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_with_i32(2, 2, WebGl2RenderingContext::FLOAT, false, 48, 8);
    gl.vertex_attrib_divisor(2, 1);

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_with_i32(3, 4, WebGl2RenderingContext::UNSIGNED_BYTE, true, 48, 16);
    gl.vertex_attrib_divisor(3, 1);

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::UNSIGNED_INT, false, 48, 20);
    gl.vertex_attrib_divisor(4, 1);

    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    // Draw the circle
    gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, 1);

    // Capture and verify
    let pixels = capture_framebuffer(&gl, 256, 256);

    // Center pixel should be blue
    let center_idx = (128 * 256 + 128) * 4;
    let r = pixels[center_idx];
    let g = pixels[center_idx + 1];
    let b = pixels[center_idx + 2];

    assert!(b > 200, "Blue channel should be high, got {}", b);
    assert!(r < 50, "Red channel should be low, got {}", r);
    assert!(g < 50, "Green channel should be low, got {}", g);

    // Corner pixel should be black (outside circle due to SDF)
    let corner_idx = (10 * 256 + 10) * 4;
    let corner_r = pixels[corner_idx];
    let corner_g = pixels[corner_idx + 1];
    let corner_b = pixels[corner_idx + 2];

    // Corners should be mostly black (background)
    assert!(
        corner_r < 50 && corner_g < 50 && corner_b < 50,
        "Corner should be black (circle SDF cutoff), got RGB({}, {}, {})",
        corner_r,
        corner_g,
        corner_b
    );
}

#[wasm_bindgen_test]
fn test_circle_has_smooth_edges() {
    let canvas = create_test_canvas(512, 512);
    let gl = get_webgl2_context(&canvas);
    let program = compile_shader_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.use_program(Some(&program));

    // Setup uniforms and buffers (same as previous test)
    let identity_matrix: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    let u_view_proj_loc = gl.get_uniform_location(&program, "u_view_projection");
    gl.uniform_matrix4fv_with_f32_array(u_view_proj_loc.as_ref(), false, &identity_matrix);

    let u_camera_pos_loc = gl.get_uniform_location(&program, "u_camera_pos");
    gl.uniform2f(u_camera_pos_loc.as_ref(), 0.0, 0.0);

    let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);

    // White circle
    let instance_data: Vec<u8> = {
        let mut data = Vec::new();
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.6f32.to_le_bytes());
        data.extend_from_slice(&0.6f32.to_le_bytes());
        data.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // White in ABGR
        data.extend_from_slice(&1u32.to_le_bytes()); // Circle
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data
    };

    let instance_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&instance_buffer));
    unsafe {
        let array = js_sys::Uint8Array::view(&instance_data);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_with_i32(1, 2, WebGl2RenderingContext::FLOAT, false, 48, 0);
    gl.vertex_attrib_divisor(1, 1);

    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_with_i32(2, 2, WebGl2RenderingContext::FLOAT, false, 48, 8);
    gl.vertex_attrib_divisor(2, 1);

    gl.enable_vertex_attrib_array(3);
    gl.vertex_attrib_pointer_with_i32(3, 4, WebGl2RenderingContext::UNSIGNED_BYTE, true, 48, 16);
    gl.vertex_attrib_divisor(3, 1);

    gl.enable_vertex_attrib_array(4);
    gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::UNSIGNED_INT, false, 48, 20);
    gl.vertex_attrib_divisor(4, 1);

    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    gl.draw_arrays_instanced(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, 1);

    let pixels = capture_framebuffer(&gl, 512, 512);

    // Check for anti-aliasing at the edge (should have gradual falloff)
    // Sample along horizontal line through center
    let y = 256;
    let mut edge_found = false;
    let mut has_partial_alpha = false;

    for x in 200..300 {
        let idx = (y * 512 + x) * 4;
        let r = pixels[idx];

        // If we find values between 50-250, we have anti-aliasing
        if r > 50 && r < 250 {
            has_partial_alpha = true;
        }

        if r > 128 && !edge_found {
            edge_found = true;
        }
    }

    assert!(
        has_partial_alpha,
        "Circle should have smooth anti-aliased edges (found no partial alpha values)"
    );
}
