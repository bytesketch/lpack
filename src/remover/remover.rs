use serde_json::Value;
use std::{fs, path::PathBuf};

use crate::remover::callback::RemoverCallback;

pub fn remove_lpk(pack_id: &str, system_wide: bool, call: &mut dyn RemoverCallback) {
    if let Err(err) = internal_remove_lpk(pack_id, system_wide, call) {
        call.on_unknown_error(&err.to_string());
    }
}

fn internal_remove_lpk(
    pack_id: &str,
    system_wide: bool,
    call: &mut dyn RemoverCallback,
) -> Result<(), Box<dyn std::error::Error>> {
    if pack_id.trim().is_empty() {
        return Err("Invalid package id.".into());
    }

    #[cfg(target_family = "unix")]
    {
        use nix::unistd::Uid;

        if system_wide && !Uid::effective().is_root() {
            return Err("Root permissions required.".into());
        }
    }

    let (install_base, bin_dir, desk_dir): (PathBuf, PathBuf, PathBuf) = if system_wide {
        (
            PathBuf::from("/var/lib/lpack/installed"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/share/applications"),
        )
    } else {
        let home: String = std::env::var("HOME")?;

        (
            PathBuf::from(&home).join(".lpack").join("installed"),
            PathBuf::from(&home).join(".local").join("bin"),
            PathBuf::from(&home)
                .join(".local")
                .join("share")
                .join("applications"),
        )
    };

    let install_dir: PathBuf = install_base.join(pack_id);

    if !install_dir.exists() {
        return Err(format!("Package '{}' is not installed.", pack_id).into());
    }

    let manifest_file: PathBuf = install_dir.join(".manifest.json");

    let mut manifest: Value = Value::Null;

    if manifest_file.is_file() {
        match fs::read_to_string(&manifest_file) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(v) => {
                    manifest = v;
                }

                Err(err) => {
                    call.on_some_warn(&format!("Failed reading manifest: {}", err));
                }
            },

            Err(err) => {
                call.on_some_warn(&format!("Failed reading manifest: {}", err));
            }
        }
    }

    let pkg_name: &str = manifest["name"].as_str().unwrap_or(pack_id);

    let pkg_ver: &str = manifest["version"].as_str().unwrap_or("unknown");

    if !call.prompt_confirm(&format!("Remove '{}' ({})?", pkg_name, pkg_ver), false) {
        call.on_some_warn("Removal cancelled.");

        return Ok(());
    }

    if bin_dir.exists() {
        for entry in fs::read_dir(&bin_dir)? {
            let entry: fs::DirEntry = entry?;

            let item: PathBuf = entry.path();

            if item.is_symlink() {
                match item.canonicalize() {
                    Ok(target) => {
                        if target.starts_with(install_dir.canonicalize()?) {
                            if call.prompt_confirm(
                                &format!(
                                    "Remove symlink '{}'?",
                                    item.file_name().unwrap().to_string_lossy()
                                ),
                                true,
                            ) {
                                fs::remove_file(&item)?;

                                call.on_some_success(&format!(
                                    "Removed symlink '{}'",
                                    item.file_name().unwrap().to_string_lossy()
                                ));
                            }
                        }
                    }

                    Err(err) => {
                        call.on_some_error(&format!(
                            "Failed removing symlink '{}': {}",
                            item.file_name().unwrap().to_string_lossy(),
                            err
                        ));
                    }
                }
            }
        }
    }

    let desktop_file: PathBuf = desk_dir.join(format!("{}.desktop", pack_id));

    if desktop_file.exists() {
        if call.prompt_confirm(
            &format!(
                "Remove desktop entry '{}'?",
                desktop_file.file_name().unwrap().to_string_lossy()
            ),
            true,
        ) {
            match fs::remove_file(&desktop_file) {
                Ok(_) => {
                    call.on_some_success(&format!(
                        "Removed desktop entry '{}'",
                        desktop_file.file_name().unwrap().to_string_lossy()
                    ));
                }

                Err(err) => {
                    call.on_some_error(&format!("Failed removing desktop entry: {}", err));
                }
            }
        }
    }

    fs::remove_dir_all(&install_dir)?;

    call.on_some_success(&format!("Removed installation '{}'", install_dir.display()));

    if install_base.exists() && install_base.read_dir()?.next().is_none() {
        let _ = fs::remove_dir(&install_base);
    }

    if !system_wide {
        if bin_dir.exists() && bin_dir.read_dir()?.next().is_none() {
            let _ = fs::remove_dir(&bin_dir);
        }

        if desk_dir.exists() && desk_dir.read_dir()?.next().is_none() {
            let _ = fs::remove_dir(&desk_dir);
        }
    }

    call.on_some_success(&format!("Package '{}' removed successfully.", pack_id));

    Ok(())
}
