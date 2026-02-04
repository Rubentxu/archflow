fn main() {
    println!("cargo:rerun-if-changed=shaders");

    let shaders_dir = std::path::Path::new("shaders");
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    #[cfg(feature = "webgl2")]
    {
        let wgsl_shaders = [
            "sdf_shapes.wgsl",
            "icon_texture.wgsl",
            "image_array.wgsl",
            "mtsdf_text.wgsl",
        ];

        for shader_name in &wgsl_shaders {
            let wgsl_path = shaders_dir.join(shader_name);
            let wgsl_content = std::fs::read_to_string(&wgsl_path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", shader_name, e));

            let glsl_name = shader_name.replace(".wgsl", ".glsl");
            let glsl_path = out_dir.join(&glsl_name);

            let _ = std::fs::write(&glsl_path, &wgsl_content)
                .unwrap_or_else(|e| panic!("Failed to write {}: {}", glsl_name, e));
            println!(
                "cargo:warning=Wrote {} (WGSL source, not compiled to GLSL yet)",
                shader_name
            );
        }
    }
}
