// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Shader Compilation Build Script
//
// Compiles WGSL shaders to GLSL ES 3.0 for WebGL2 support.
// Uses Naga for shader translation.
// ═══════════════════════════════════════════════════════════════════════════════

fn main() {
    println!("cargo:rerun-if-changed=src/shaders");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    #[cfg(feature = "webgl2")]
    {
        let shaders_dir = std::path::Path::new("src/shaders");
        compile_shaders(shaders_dir, &out_dir);
    }
}

/// Shader compilation configuration
#[derive(Debug, Clone)]
struct ShaderConfig {
    name: &'static str,
    wgsl_filename: &'static str,
    entry_point: &'static str,
}

#[cfg(feature = "webgl2")]
fn compile_shaders(shaders_dir: &std::path::Path, out_dir: &std::path::PathBuf) {
    use naga::ShaderStage;
    use naga::back::glsl;
    use naga::front::wgsl;
    use naga::valid::{Capabilities, ValidationFlags};

    let shader_configs = [
        ShaderConfig {
            name: "sdf_shapes",
            wgsl_filename: "sdf_shapes.wgsl",
            entry_point: "vs_main",
        },
        ShaderConfig {
            name: "icon_texture",
            wgsl_filename: "icon_texture.wgsl",
            entry_point: "vs_main",
        },
        ShaderConfig {
            name: "image_array",
            wgsl_filename: "image_array.wgsl",
            entry_point: "vs_main",
        },
        ShaderConfig {
            name: "mtsdf_text",
            wgsl_filename: "mtsdf_text.wgsl",
            entry_point: "vs_main",
        },
    ];

    for config in &shader_configs {
        let wgsl_path = shaders_dir.join(config.wgsl_filename);

        // Read WGSL source
        let wgsl_content = std::fs::read_to_string(&wgsl_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", config.wgsl_filename, e));

        // Parse WGSL to Naga IR
        let module = wgsl::parse_str(&wgsl_content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", config.wgsl_filename, e));

        // Validate module
        let mut validator =
            naga::valid::Validator::new(ValidationFlags::all(), Capabilities::all());
        let module_info = validator
            .validate(&module)
            .unwrap_or_else(|e| panic!("Failed to validate {}: {:#?}", config.wgsl_filename, e));

        // Determine shader stage from entry point name convention
        let shader_stage = if config.entry_point.starts_with("vs_") {
            ShaderStage::Vertex
        } else if config.entry_point.starts_with("fs_") {
            ShaderStage::Fragment
        } else {
            ShaderStage::Vertex
        };

        // Compile to GLSL
        let mut glsl_output = String::new();
        let mut options = glsl::Options::default();
        options.version = glsl::Version::Embedded {
            version: 310,
            is_webgl: false,
        };

        let pipeline_options = glsl::PipelineOptions {
            entry_point: config.entry_point.to_string(),
            shader_stage,
            multiview: None,
        };

        let mut writer = glsl::Writer::new(
            &mut glsl_output,
            &module,
            &module_info,
            &options,
            &pipeline_options,
            Default::default(),
        )
        .unwrap_or_else(|e| panic!("Failed to create GLSL writer for {}: {}", config.name, e));

        writer
            .write()
            .unwrap_or_else(|e| panic!("Failed to compile {} to GLSL: {}", config.name, e));

        // Write GLSL output
        let glsl_filename = format!("{}.glsl", config.name);
        let glsl_path = out_dir.join(&glsl_filename);

        std::fs::write(&glsl_path, &glsl_output)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", glsl_filename, e));

        println!(
            "cargo:warning=Compiled {} to {} ({} bytes)",
            config.wgsl_filename,
            glsl_filename,
            glsl_output.len()
        );
    }
}
