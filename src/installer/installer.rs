use serde_json::{Value, json};
use std::{
    fs::{self, File},
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
};
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::installer::callback::InstallCallback;
use crate::installer::manifest::{InstallManifest, VersionPart};

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub fn deobfuscate(all_bytes: &[u8]) -> Vec<u8> {
    all_bytes
        .iter()
        .map(|b| (((((*b >> 3) | (*b << 5)) & 0xFF).wrapping_sub(17)) & 0xFF) ^ 0x5A)
        .collect()
}

pub fn parse_version(ver: &str) -> Vec<VersionPart> {
    ver.split('.')
        .map(|item| {
            if let Ok(num) = item.parse::<i32>() {
                VersionPart::Number(num)
            } else {
                VersionPart::Text(item.to_string())
            }
        })
        .collect()
}

pub fn load_manifest(data: Value) -> Result<InstallManifest, Box<dyn std::error::Error>> {
    Ok(serde_json::from_value(data)?)
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

pub fn chmod_recursive(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(path) {
        let entry = entry?;
        let metadata = fs::metadata(entry.path())?;
        if metadata.is_file() {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(entry.path(), perms)?;
        }
    }
    Ok(())
}

pub fn extract_zip(zip_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file: File = File::open(zip_path)?;
    let mut archive: ZipArchive<File> = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file: zip::read::ZipFile<'_, File> = archive.by_index(i)?;
        let outpath: PathBuf = out_dir.join(file.name());
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile: File = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            let mut perms = outfile.metadata()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&outpath, perms)?;
        }
    }
    Ok(())
}

pub fn install_lpack(pack: &Path, system_wide: bool, call: &mut dyn InstallCallback) {
    if let Err(err) = internal_install_lpack(pack, system_wide, call) {
        call.on_unknown_error(&err.to_string());
    }
}

fn internal_install_lpack(
    pack: &Path,
    system_wide: bool,
    call: &mut dyn InstallCallback,
) -> Result<(), Box<dyn std::error::Error>> {
    if !pack.is_file() {
        return Err(format!("{} not found.", pack.display()).into());
    }

    #[cfg(target_family = "unix")]
    {
        use nix::unistd::Uid;
        let is_root = Uid::effective().is_root();
        if system_wide && !is_root {
            return Err("Root permissions required.".into());
        }

        if is_root && !system_wide {
            call.on_some_warn(&format!(
                "{}WARNING:{} Running lpack as root without '--system-wide'.",
                BOLD, RESET
            ));

            call.on_some_warn(&format!(
                "This package will be installed into:{} {}{}{}",
                RESET, BOLD, "/root/.lpack", RESET
            ));

            call.on_some_warn("The installation will only be visible to the root user.");

            let confirm = call.prompt_confirm(
                &format!("{}Continue with root-local installation?{}", BOLD, RESET),
                false,
            );

            if !confirm {
                call.on_some_warn("Installation aborted by user.");

                return Ok(());
            }

            call.on_some_warn(&format!(
                "Proceeding with root-local installation.{}",
                RESET
            ));
        }
    }

    call.on_some_info(&format!(
        "Installing '{}'",
        pack.file_name().unwrap().to_string_lossy()
    ));

    let temp: tempfile::TempDir = tempdir()?;
    let extract_dir: PathBuf = temp.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    call.on_some_info(&format!("Extracting into '{}'", extract_dir.display()));
    extract_zip(pack, &extract_dir)?;
    chmod_recursive(&extract_dir)?;
    let manifest_file: PathBuf = extract_dir.join("manifest");

    if !manifest_file.is_file() {
        return Err("Package manifest missing.".into());
    }

    let raw: Vec<u8> = fs::read(&manifest_file)?;
    let manifest: InstallManifest = load_manifest(serde_json::from_slice(&deobfuscate(&raw))?)?;
    call.on_some_success("Manifest loaded.");

    let (base_dir, bin_dir, desk_dir): (PathBuf, PathBuf, PathBuf) = if system_wide {
        (
            PathBuf::from("/var/lib/lpack/installed"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/share/applications"),
        )
    } else {
        let home = std::env::var("HOME")?;
        (
            PathBuf::from(&home).join(".lpack").join("installed"),
            PathBuf::from(&home).join(".local").join("bin"),
            PathBuf::from(&home)
                .join(".local")
                .join("share")
                .join("applications"),
        )
    };

    let install_dir: PathBuf = base_dir.join(&manifest.info.package_name);

    if install_dir.exists() {
        let info_file: PathBuf = install_dir.join(".manifest.json");
        let mut old_ver: Option<String> = None;

        if info_file.is_file() {
            if let Ok(content) = fs::read_to_string(&info_file) {
                if let Ok(v) = serde_json::from_str::<Value>(&content) {
                    if let Some(ver) = v["version"].as_str() {
                        old_ver = Some(ver.to_string());
                    }
                }
            }
        }

        if let Some(old_ver_str) = old_ver {
            let old_v: Vec<VersionPart> = parse_version(&old_ver_str);
            let new_v: Vec<VersionPart> = parse_version(&manifest.info.version);

            if new_v < old_v {
                if !call.prompt_confirm(
                    &format!("Downgrade {} -> {}?", old_ver_str, manifest.info.version),
                    false,
                ) {
                    return Ok(());
                }
            } else if new_v == old_v {
                if !call.prompt_confirm(&format!("Reinstall version {}?", old_ver_str), true) {
                    return Ok(());
                }
            } else {
                if !call.prompt_confirm(
                    &format!("Upgrade {} -> {}?", old_ver_str, manifest.info.version),
                    true,
                ) {
                    return Ok(());
                }
            }
        } else {
            if !call.prompt_confirm("Overwrite existing installation?", false) {
                return Ok(());
            }
        }

        call.on_some_warn("Removing previous installation.");
        fs::remove_dir_all(&install_dir)?;
    }

    fs::create_dir_all(&base_dir)?;

    copy_dir_all(extract_dir.join("app"), &install_dir)?;
    chmod_recursive(&install_dir)?;
    call.on_some_success(&format!("Installed into '{}'", install_dir.display()));

    fs::write(
        install_dir.join(".manifest.json"),
        serde_json::to_string_pretty(&json!({
            "name": manifest.info.name,
            "package": manifest.info.package_name,
            "version": manifest.info.version,
            "description": manifest.info.description
        }))?,
    )?;

    if let Some(app) = &manifest.app {
        fs::create_dir_all(&bin_dir)?;

        let target: PathBuf = install_dir.join(&app.executable);

        if !target.exists() {
            return Err(format!("Executable '{}' missing.", app.executable).into());
        }

        let link = bin_dir.join(&app.entry);

        if link.exists() || link.is_symlink() {
            fs::remove_file(&link)?;
        }

        symlink(target.canonicalize()?, &link)?;

        let mut perms: fs::Permissions = fs::metadata(&target)?.permissions();

        perms.set_mode(0o755);

        fs::set_permissions(&target, perms)?;

        call.on_some_success(&format!("Linked '{}'", link.display()));
    }

    if let Some(desk) = &manifest.desk {
        fs::create_dir_all(&desk_dir)?;

        let desktop_file: PathBuf = desk_dir.join(format!("{}.desktop", manifest.info.package_name));
        let icon_path: PathBuf = install_dir.join(&desk.icon);
        fs::write(
            &desktop_file,
            format!(
                "[Desktop Entry]\nVersion=1.0\nType=Application\nName={}\nExec={}\nIcon={}\nTerminal=false\nCategories=Utility;\n",
                desk.name,
                desk.exec_cmd,
                icon_path.display()
            ),
        )?;

        call.on_some_success("Desktop entry created.");
    }

    call.on_some_success(&format!("Installed '{}' successfully.", manifest.info.name));

    Ok(())
}
