use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Only run this in development mode, not during publishing
    if env::var("CARGO_PUBLISH").is_ok() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_bindings_dir = PathBuf::from(&manifest_dir).join("bindings");

    // Target directory in packages/
    let packages_dir = PathBuf::from(&manifest_dir)
        .parent() // crates/
        .unwrap()
        .parent() // root
        .unwrap()
        .join("packages")
        .join("archflow-sdk-types")
        .join("src")
        .join("generated");

    // Check if bindings directory exists
    if !crate_bindings_dir.exists() {
        println!(
            "cargo:warning=Bindings directory not found: {:?}",
            crate_bindings_dir
        );
        return;
    }

    // Ensure target directory exists
    if let Err(e) = fs::create_dir_all(&packages_dir) {
        println!("cargo:warning=Failed to create packages directory: {}", e);
        return;
    }

    // Copy all .ts files
    let mut copied_count = 0;
    if let Ok(entries) = fs::read_dir(&crate_bindings_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                let file_name = path.file_name().unwrap();
                let target_path = packages_dir.join(file_name);

                match fs::copy(&path, &target_path) {
                    Ok(_) => {
                        copied_count += 1;
                        println!("cargo:rerun-if-changed={}", path.display());
                    }
                    Err(e) => {
                        println!("cargo:warning=Failed to copy {:?}: {}", file_name, e);
                    }
                }
            }
        }
    }

    if copied_count > 0 {
        println!(
            "cargo:warning=Copied {} TypeScript binding files to {:?}",
            copied_count, packages_dir
        );
    }

    // Also copy to WASM package if it exists
    let wasm_bindings_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packages")
        .join("sdk")
        .join("src")
        .join("generated");

    if wasm_bindings_dir.exists() || fs::create_dir_all(&wasm_bindings_dir).is_ok() {
        if let Ok(entries) = fs::read_dir(&crate_bindings_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                    let file_name = path.file_name().unwrap();
                    let target_path = wasm_bindings_dir.join(file_name);
                    let _ = fs::copy(&path, &target_path);
                }
            }
        }
    }

    // Rerun if bindings directory changes
    println!("cargo:rerun-if-changed={}", crate_bindings_dir.display());
}
