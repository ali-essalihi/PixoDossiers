//! Guarded file-system deletion, copy, and write helpers for Pixo et les Dossiers.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config;
use crate::paths;

pub const FRIENDLY_LOCK_ERROR: &str = "Ferme les fenêtres ouvertes puis réessaie";
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;

fn norm_abs(path: &Path) -> String {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap_or_default().join(path)
    };
    // Lexical normalization without following leaf symlinks
    let mut out = PathBuf::new();
    for comp in abs.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

pub fn get_allowed_roots() -> Vec<String> {
    paths::get_all_known_folders()
        .values()
        .map(|p| norm_abs(p))
        .collect()
}

fn get_forbidden_exact_paths() -> Vec<String> {
    let mut forbidden = get_allowed_roots();
    if let Ok(up) = env::var("USERPROFILE") {
        forbidden.push(norm_abs(Path::new(&up)));
    }
    if let Some(sb) = paths::get_sandbox_dir() {
        forbidden.push(norm_abs(&sb));
    }
    forbidden
}

/// Return true if `path` is a symlink or a Windows reparse point (junction).
pub fn is_link_or_junction(path: &Path) -> bool {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::MetadataExt;
            if (meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                return true;
            }
        }
    }
    false
}

/// Verify that `target_path` is strictly inside one of the 6 allowed Known Folders.
pub fn assert_inside_allowed_root(target_path: &Path) -> Result<String, String> {
    let norm_target = norm_abs(target_path);
    if norm_target.is_empty() || norm_target.len() <= 3 {
        return Err(format!(
            "Refus de modifier la racine du disque : {}",
            target_path.display()
        ));
    }

    for f in get_forbidden_exact_paths() {
        if norm_target == f {
            return Err(format!(
                "Refus de supprimer un dossier système protégé : {}",
                target_path.display()
            ));
        }
    }

    let allowed = get_allowed_roots();
    let mut matched_root: Option<String> = None;
    for root in allowed {
        let prefix = format!("{}\\", root);
        if norm_target.starts_with(&prefix) && norm_target != root {
            matched_root = Some(root);
            break;
        }
    }

    let root = matched_root.ok_or_else(|| {
        format!(
            "Opération refusée hors des dossiers autorisés : {}",
            target_path.display()
        )
    })?;

    // Ensure no intermediate parent directory between root and target's parent is a symlink/junction
    let rel_part = &norm_target[root.len()..].trim_start_matches('\\');
    let parts: Vec<&str> = rel_part.split('\\').collect();
    if parts.len() > 1 {
        let mut cur = PathBuf::from(&root);
        for part in &parts[..parts.len() - 1] {
            cur.push(part);
            if cur.exists() && is_link_or_junction(&cur) {
                return Err(format!(
                    "Opération refusée à travers un lien symbolique ou jonction : {}",
                    cur.display()
                ));
            }
        }
    }

    Ok(root)
}

fn clear_readonly(path: &Path) {
    if let Ok(meta) = fs::symlink_metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            perms.set_readonly(false);
            let _ = fs::set_permissions(path, perms);
        }
    }
}

fn retry_io<F, T>(target: Option<&Path>, mut op: F) -> Result<T, String>
where
    F: FnMut() -> std::io::Result<T>,
{
    for attempt in 0..4 {
        match op() {
            Ok(val) => return Ok(val),
            Err(e) => {
                if let Some(p) = target {
                    clear_readonly(p);
                }
                if attempt == 3 {
                    let raw = e.raw_os_error().unwrap_or(0);
                    if e.kind() == std::io::ErrorKind::PermissionDenied || raw == 5 || raw == 32 {
                        return Err(FRIENDLY_LOCK_ERROR.to_string());
                    }
                    return Err(FRIENDLY_LOCK_ERROR.to_string());
                }
                thread::sleep(Duration::from_millis(120 * (attempt + 1)));
            }
        }
    }
    Err(FRIENDLY_LOCK_ERROR.to_string())
}

fn remove_single_link_or_junction(path: &Path) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.is_dir() && fs::remove_dir(path).is_ok() {
            return Ok(());
        }
    }
    if fs::remove_file(path).is_ok() {
        return Ok(());
    }
    retry_io(Some(path), || fs::remove_dir(path))
}

fn delete_tree_no_follow(target_path: &Path) -> Result<(), String> {
    assert_inside_allowed_root(target_path)?;
    if fs::symlink_metadata(target_path).is_err() {
        return Ok(());
    }

    if is_link_or_junction(target_path) {
        return remove_single_link_or_junction(target_path);
    }

    if target_path.is_dir() {
        let rd = fs::read_dir(target_path).map_err(|_| FRIENDLY_LOCK_ERROR.to_string())?;
        for entry in rd.flatten() {
            delete_tree_no_follow(&entry.path())?;
        }
        retry_io(Some(target_path), || fs::remove_dir(target_path))?;
    } else {
        retry_io(Some(target_path), || fs::remove_file(target_path))?;
    }
    Ok(())
}

fn backup_item(target_path: &Path, stamp: &str) -> Result<(), String> {
    let matched_root = assert_inside_allowed_root(target_path)?;
    if fs::symlink_metadata(target_path).is_err() {
        return Ok(());
    }
    let root_name = Path::new(&matched_root)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".to_string());
    let norm_t = norm_abs(target_path);
    let rel = norm_t[matched_root.len()..].trim_start_matches('\\');
    let dest = paths::get_appdata_dir()
        .join("backup")
        .join(stamp)
        .join(root_name)
        .join(rel);
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    retry_io(Some(target_path), || fs::rename(target_path, &dest))
}

/// Safely remove (or backup per `config::WIPE_MODE`) a file or directory inside an allowed root.
pub fn safe_remove_path(target_path: &Path) -> Result<(), String> {
    assert_inside_allowed_root(target_path)?;
    if fs::symlink_metadata(target_path).is_err() {
        return Ok(());
    }
    if config::WIPE_MODE == "backup" && !is_link_or_junction(target_path) {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        backup_item(target_path, &format!("bak_{}", secs))
    } else {
        delete_tree_no_follow(target_path)
    }
}

/// Safely remove all contents inside `folder_path` without deleting `folder_path` itself.
pub fn safe_clear_folder_contents(folder_path: &Path) -> Result<(), String> {
    let norm_folder = norm_abs(folder_path);
    let allowed = get_allowed_roots();
    if !allowed.contains(&norm_folder) {
        assert_inside_allowed_root(folder_path)?;
    }
    if !folder_path.is_dir() {
        let _ = fs::create_dir_all(folder_path);
        return Ok(());
    }
    let rd = fs::read_dir(folder_path).map_err(|_| FRIENDLY_LOCK_ERROR.to_string())?;
    for entry in rd.flatten() {
        safe_remove_path(&entry.path())?;
    }
    Ok(())
}

/// Safely copy `src_path` to `dst_path` inside an allowed root.
pub fn safe_copy_file(src_path: &Path, dst_path: &Path) -> Result<(), String> {
    assert_inside_allowed_root(dst_path)?;
    if !src_path.is_file() {
        let name = src_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        return Err(format!("Fichier source introuvable : {}", name));
    }
    if let Some(parent) = dst_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::symlink_metadata(dst_path).is_ok() {
        safe_remove_path(dst_path)?;
    }
    retry_io(Some(dst_path), || fs::copy(src_path, dst_path).map(|_| ()))
}

/// Safely write a small UTF-8 text file inside an allowed root.
pub fn safe_write_text(dst_path: &Path, content: &str) -> Result<(), String> {
    assert_inside_allowed_root(dst_path)?;
    if let Some(parent) = dst_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::symlink_metadata(dst_path).is_ok() {
        safe_remove_path(dst_path)?;
    }
    retry_io(Some(dst_path), || {
        let mut f = fs::File::create(dst_path)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()
    })
}
