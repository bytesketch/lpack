use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

use nix::unistd::Uid;

pub fn get_paths(
    system_wide: bool,
) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    if system_wide {
        if !Uid::effective().is_root() {
            return Err("Root permissions required.".into());
        }

        return Ok((
            PathBuf::from("/var/lib/lpack/installed"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/share/applications"),
        ));
    }

    let home = std::env::var("HOME")?;

    Ok((
        PathBuf::from(&home).join(".lpack").join("installed"),
        PathBuf::from(&home).join(".local").join("bin"),
        PathBuf::from(&home)
            .join(".local")
            .join("share")
            .join("applications"),
    ))
}

pub fn search_all(system_wide: bool) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let (install_dir, _, _) = get_paths(system_wide)?;

    let mut output = Vec::new();

    let mut found = HashSet::new();

    if !install_dir.exists() {
        return Ok(output);
    }

    if !install_dir.is_dir() {
        return Err(format!("{} is not directory.", install_dir.display()).into());
    }

    for entry in fs::read_dir(&install_dir)? {
        let entry = entry?;

        let item = entry.path();

        if !item.is_dir() {
            continue;
        }

        let manifest = item.join(".manifest.json");

        let mut package = item.file_name().unwrap().to_string_lossy().to_string();

        let mut version = "unknown".to_string();

        if manifest.is_file() {
            if let Ok(content) = fs::read_to_string(&manifest) {
                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                    if let Some(obj) = data.as_object() {
                        if let Some(pkg) = obj.get("package") {
                            if let Some(pkg) = pkg.as_str() {
                                package = pkg.to_string();
                            }
                        }

                        if let Some(ver) = obj.get("version") {
                            if let Some(ver) = ver.as_str() {
                                version = ver.to_string();
                            }
                        }
                    }
                }
            }
        }

        let key = (package.clone(), version.clone());

        if found.contains(&key) {
            continue;
        }

        found.insert(key.clone());

        output.push(key);
    }

    Ok(output)
}

pub fn search_one(package: &str, system_wide: bool) -> Result<Value, Box<dyn std::error::Error>> {
    let (install_dir, bin_dir, desk_dir) = get_paths(system_wide)?;

    if package.trim().is_empty() {
        return Err("Invalid package name.".into());
    }

    let target = install_dir.join(package);

    if !target.exists() {
        return Err(format!("Package '{}' is not installed.", package).into());
    }

    let manifest_file = target.join(".manifest.json");

    if !manifest_file.is_file() {
        return Err("Manifest missing.".into());
    }

    let content = fs::read_to_string(&manifest_file)?;

    let data: Value = serde_json::from_str(&content)?;

    let obj = data.as_object().ok_or("Invalid manifest.")?;

    let mut result = serde_json::json!({
        "name": obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(package),

        "version": obj
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown"),

        "description": obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or(""),

        "desktop": Value::Null,
        "symlink": Value::Null
    });

    let desktop_file = desk_dir.join(format!("{}.desktop", package));

    if desktop_file.exists() {
        result["desktop"] = Value::String(desktop_file.canonicalize()?.display().to_string());
    }

    if bin_dir.exists() {
        for entry in fs::read_dir(&bin_dir)? {
            let entry = entry?;

            let item = entry.path();

            if let Ok(meta) = fs::symlink_metadata(&item) {
                if meta.file_type().is_symlink() {
                    if let Ok(resolved) = fs::canonicalize(&item) {
                        if resolved.starts_with(target.canonicalize()?) {
                            result["symlink"] = Value::String(resolved.display().to_string());

                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}
