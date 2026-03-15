use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

fn main() {
    let root_string = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR should always be set by cargo");
    let root = Path::new(&root_string);

    let shader_source_directory = root.join("shaders");
    let shader_output_directory = root.join("target").join("shaders");

    let entries = match fs::read_dir(&shader_source_directory) {
        Ok(entries) => entries,
        Err(error) => {
            println!("cargo::error=Failed to read directory {}: {}", shader_source_directory.display(), error);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("cargo::error=Failed to read directory entry: {}", err);
                continue;
            }
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path.extension().map_or(true, |ext| ext != "slang") {
            continue;
        }

        println!("cargo::rerun-if-changed={}", path.display());

        let shader_file_name = match path.file_name() {
            Some(name) => name,
            None => {
                println!("cargo::warning=Skipping path with no filename: {}", path.display());
                continue;
            }
        };

        let spv_path = shader_output_directory.join(shader_file_name).with_extension("spv");

        let slang_modified_time = match fs::metadata(&path).and_then(|m| m.modified()) {
            Ok(time) => time,
            Err(err) => {
                println!("cargo::error=Failed to get modified time for slang file '{}': {}", path.display(), err);
                continue;
            }
        };

        let spv_modified_time = match fs::metadata(&spv_path).and_then(|m| m.modified()) {
            Ok(time) => time,
            Err(_) => SystemTime::UNIX_EPOCH,
        };

        if spv_modified_time >= slang_modified_time {
            continue
        }

        let status = Command::new("slangc")
            .arg(&path)
            .arg("-target")
            .arg("spirv")
            .arg("-o")
            .arg(&spv_path)
            .status()
            .expect("Failed to run slangc");

        if !status.success() {
            panic!("Failed to compile shader: {}", path.display());
        } 
    }
}
