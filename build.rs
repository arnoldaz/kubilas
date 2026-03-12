use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let shader_source_directory = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("shaders");
    let shader_output_directory = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("target").join("shaders");

    println!("cargo::warning=Shader source: {}", shader_source_directory.display());
    println!("cargo::warning=Shader output: {}", shader_output_directory.display());

    for entry in fs::read_dir(shader_source_directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("slang") {
            println!("cargo::rerun-if-changed={}", path.display());

            println!("cargo::warning=shader: {}", path.display());
            let filename = path.file_stem().unwrap().to_str().unwrap();
            let out = shader_output_directory.join(format!("{}.spv", filename));

            println!("cargo::warning=Recompiling shader: {}", out.display());

            // Compile vertex shader
            let status = Command::new("slangc")
                .arg(&path)
                .arg("-target")
                .arg("spirv")
                .arg("-o")
                .arg(&out)
                .status()
                .expect("failed to run slangc for vertex shader");
            assert!(status.success());
        }
    }
}