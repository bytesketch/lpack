use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::builder::callback::Callback;
use crate::builder::manifest::Manifest;

pub fn obfuscate(all_bytes: &[u8]) -> Vec<u8> {
    all_bytes
        .iter()
        .map(|b| {
            let x = ((*b ^ 0x5A).wrapping_add(17)) & 0xFF;
            ((x << 3) | (x >> 5)) & 0xFF
        })
        .collect()
}

pub fn compress(
    target_file: &Path,
    dir: &Path,
    call: &mut dyn Callback,
) -> Result<(), Box<dyn std::error::Error>> {
    call.on_some_info(&format!(
        "Compressing '{}' -> '{}'",
        dir.display(),
        target_file.display()
    ));

    let file: File = File::create(target_file)?;
    let mut zip: ZipWriter<File> = ZipWriter::new(file);

    let options: zip::write::FileOptions<'_, ()> = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    for entry in WalkDir::new(dir) {
        let entry: walkdir::DirEntry = entry?;
        let path: &Path = entry.path();

        if path.is_file() {
            let relative: std::borrow::Cow<'_, str> = path.strip_prefix(dir)?.to_string_lossy();
            zip.start_file(relative.as_ref(), options)?;

            let mut f: File = File::open(path)?;
            let mut buffer: Vec<u8> = Vec::new();
            f.read_to_end(&mut buffer)?;

            zip.write_all(&buffer)?;
            call.on_some_success(&format!("Compressed: {}", relative));
        }
    }

    zip.finish()?;

    call.on_some_success(&format!(
        "Archive created successfully: {}",
        target_file.display()
    ));

    Ok(())
}

pub fn parse_manifest(value: Value) -> Result<Manifest, Box<dyn std::error::Error>> {
    let manifest: Manifest = serde_json::from_value(value)?;
    Ok(manifest)
}

pub fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry: fs::DirEntry = entry?;
        let ty: fs::FileType = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}

pub fn build_lpack(root_dir: &Path, call: &mut dyn Callback) {
    if let Err(err) = internal_build_lpack(root_dir, call) {
        call.on_unknown_error(&err.to_string());
    }
}

fn internal_build_lpack(
    root_dir: &Path,
    call: &mut dyn Callback,
) -> Result<(), Box<dyn std::error::Error>> {
    call.on_some_info(&format!("Starting build in '{}'", root_dir.display()));

    if !root_dir.is_dir() {
        return Err(format!("{} is not a directory.", root_dir.display()).into());
    }

    let manifest_file: std::path::PathBuf = root_dir.join("manifest.lpack");
    if !manifest_file.is_file() {
        return Err("manifest.lpack not found.".into());
    }

    call.on_some_success("Manifest file found.");
    let content: String = fs::read_to_string(&manifest_file)?;
    let json_value: Value = serde_json::from_str(&content)?;
    let manifest: Manifest = parse_manifest(json_value)?;
    call.on_some_success("Manifest parsed successfully.");

    let temp: std::path::PathBuf = root_dir.join("lpack").join("temp");
    if temp.exists() {
        if temp.is_dir() {
            call.on_some_warn("Previous temp directory exists. Removing.");
            fs::remove_dir_all(&temp)?;
        } else {
            call.on_some_warn("Temp path exists as file. Removing.");
            fs::remove_file(&temp)?;
        }
    }

    fs::create_dir_all(&temp)?;
    call.on_some_success(&format!("Created temp directory: {}", temp.display()));

    let temp_app: std::path::PathBuf = temp.join("app");
    fs::create_dir_all(&temp_app)?;
    call.on_some_success(&format!("Created app directory: {}", temp_app.display()));

    let build_dir: std::path::PathBuf = root_dir.join(&manifest.build_base_dir);

    if !build_dir.exists() {
        return Err(format!("Build path '{}' does not exist.", build_dir.display()).into());
    }

    call.on_some_info(&format!(
        "Copying build files from '{}'",
        build_dir.display()
    ));

    for entry in fs::read_dir(&build_dir)? {
        let entry: fs::DirEntry = entry?;
        let path: std::path::PathBuf = entry.path();

        let target: std::path::PathBuf = temp_app.join(entry.file_name());
        let result: Result<(), Box<dyn std::error::Error>> = if path.is_file() {
            fs::copy(&path, &target)?;
            Ok(())
        } else {
            copy_dir_all(&path, &target)
        };

        match result {
            Ok(_) => {
                call.on_some_success(&format!(
                    "Copied: {}",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }

            Err(err) => {
                call.on_some_error(&format!("Failed to copy '{}': {}", path.display(), err));
            }
        }
    }

    if let Some(include) = &manifest.include {
        call.on_some_info("Processing include files.");

        for (main_file, target_path) in include {
            let src: std::path::PathBuf = root_dir.join(main_file);
            let dst: std::path::PathBuf = temp_app.join(target_path);

            if !src.exists() {
                call.on_some_error(&format!("Include file not found: {}", src.display()));
                continue;
            }

            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }

            match fs::copy(&src, &dst) {
                Ok(_) => {
                    call.on_some_success(&format!(
                        "Included: {} -> {}",
                        src.display(),
                        dst.display()
                    ));
                }

                Err(err) => {
                    call.on_some_error(&format!("Failed including '{}': {}", main_file, err));
                }
            }
        }
    }

    let mut final_manifest = json!({
        "info": {
            "package": manifest.pack.package_name,
            "name": manifest.pack.name,
            "description": manifest.description,
            "version": manifest.pack.version
        }
    });

    if let Some(app) = &manifest.app {
        final_manifest["app"] = json!({
            "entry": app.entry,
            "executable": app.binary
        });
    }

    if let Some(desk) = &manifest.desk {
        final_manifest["desktop"] = json!({
            "name": desk.name,
            "icon": desk.icon,
            "exec": desk.exec_cmd
        });
    }

    call.on_some_info("Generating obfuscated manifest.");
    let data = obfuscate(serde_json::to_string(&final_manifest)?.as_bytes());

    let mut file: File = File::create(temp.join("manifest"))?;
    file.write_all(&data)?;
    call.on_some_success("Manifest generated successfully.");

    let build_out: std::path::PathBuf = temp.parent().unwrap().join("build");
    fs::create_dir_all(&build_out)?;

    let output_file: std::path::PathBuf = build_out.join(format!(
        "{}-{}.lpk",
        final_manifest["info"]["name"].as_str().unwrap(),
        final_manifest["info"]["version"].as_str().unwrap()
    ));

    compress(&output_file, &temp, call)?;

    Ok(())
}
