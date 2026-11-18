//! Windows Known Folders resolution via SHGetKnownFolderPath, sandbox mapping, and Unicode normalization.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

static RESOURCE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Store the Tauri resource directory at startup so level modules can locate bundled payload files.
pub fn set_resource_dir(dir: PathBuf) {
    if let Ok(mut guard) = RESOURCE_DIR.write() {
        *guard = Some(dir);
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
#[link(name = "ole32")]
extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const Guid,
        dwflags: u32,
        htoken: *mut core::ffi::c_void,
        ppszpath: *mut *mut u16,
    ) -> i32;
    fn CoTaskMemFree(pv: *const core::ffi::c_void);
}

#[cfg(target_os = "windows")]
fn known_folder_guid(key: &str) -> Option<Guid> {
    match key {
        // {B4BFCC3A-DB2C-424C-B029-7FE99A87C641}
        "Desktop" => Some(Guid {
            data1: 0xB4BFCC3A,
            data2: 0xDB2C,
            data3: 0x424C,
            data4: [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41],
        }),
        // {374DE290-123F-4565-9164-39C4925E467B}
        "Downloads" => Some(Guid {
            data1: 0x374DE290,
            data2: 0x123F,
            data3: 0x4565,
            data4: [0x91, 0x64, 0x39, 0xC4, 0x92, 0x5E, 0x46, 0x7B],
        }),
        // {FDD39AD0-238F-46AF-ADB4-6C85480369C7}
        "Documents" => Some(Guid {
            data1: 0xFDD39AD0,
            data2: 0x238F,
            data3: 0x46AF,
            data4: [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7],
        }),
        // {33E28130-4E1E-4676-835A-98395C3BC3BB}
        "Pictures" => Some(Guid {
            data1: 0x33E28130,
            data2: 0x4E1E,
            data3: 0x4676,
            data4: [0x83, 0x5A, 0x98, 0x39, 0x5C, 0x3B, 0xC3, 0xBB],
        }),
        // {4BD8D571-6D19-48D3-BE97-422220080E43}
        "Music" => Some(Guid {
            data1: 0x4BD8D571,
            data2: 0x6D19,
            data3: 0x48D3,
            data4: [0xBE, 0x97, 0x42, 0x22, 0x20, 0x08, 0x0E, 0x43],
        }),
        // {18989B1D-99B5-455B-841C-AB7C74E4DDFC}
        "Videos" => Some(Guid {
            data1: 0x18989B1D,
            data2: 0x99B5,
            data3: 0x455B,
            data4: [0x84, 0x1C, 0xAB, 0x7C, 0x74, 0xE4, 0xDD, 0xFC],
        }),
        // {3EB685DB-65F9-4CF6-A03A-E3EF65729F3D}
        "RoamingAppData" => Some(Guid {
            data1: 0x3EB685DB,
            data2: 0x65F9,
            data3: 0x4CF6,
            data4: [0xA0, 0x3A, 0xE3, 0xEF, 0x65, 0x72, 0x9F, 0x3D],
        }),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn sh_get_known_folder_path(key: &str) -> Option<PathBuf> {
    let guid = known_folder_guid(key)?;
    unsafe {
        let mut path_ptr: *mut u16 = core::ptr::null_mut();
        let hr = SHGetKnownFolderPath(&guid, 0, core::ptr::null_mut(), &mut path_ptr);
        if hr == 0 && !path_ptr.is_null() {
            let mut len = 0usize;
            while *path_ptr.add(len) != 0 {
                len += 1;
            }
            let slice = core::slice::from_raw_parts(path_ptr, len);
            let s = String::from_utf16_lossy(slice);
            CoTaskMemFree(path_ptr as *const core::ffi::c_void);
            if !s.is_empty() {
                return Some(PathBuf::from(s));
            }
        } else if !path_ptr.is_null() {
            CoTaskMemFree(path_ptr as *const core::ffi::c_void);
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn sh_get_known_folder_path(_key: &str) -> Option<PathBuf> {
    None
}

/// Return the active `PIXO_SANDBOX` directory if set and non-empty.
pub fn get_sandbox_dir() -> Option<PathBuf> {
    if let Ok(val) = env::var("PIXO_SANDBOX") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    None
}

/// Return true if running inside `PIXO_SANDBOX` mode.
pub fn is_sandbox() -> bool {
    get_sandbox_dir().is_some()
}

/// Resolve a Known Folder (`Desktop`, `Downloads`, `Documents`, `Pictures`, `Music`, `Videos`).
pub fn get_known_folder(key: &str) -> Result<PathBuf, String> {
    const ALLOWED: &[&str] = &[
        "Desktop",
        "Downloads",
        "Documents",
        "Pictures",
        "Music",
        "Videos",
    ];
    if !ALLOWED.contains(&key) {
        return Err(format!("Dossier inconnu : {}", key));
    }

    if let Some(sb) = get_sandbox_dir() {
        let folder = sb.join(key);
        let _ = fs::create_dir_all(&folder);
        return Ok(folder);
    }

    if let Some(resolved) = sh_get_known_folder_path(key) {
        return Ok(resolved);
    }

    let user_profile = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    Ok(PathBuf::from(user_profile).join(key))
}

/// Return a map of all 6 Known Folders.
pub fn get_all_known_folders() -> HashMap<String, PathBuf> {
    let mut map = HashMap::new();
    for key in &[
        "Desktop",
        "Downloads",
        "Documents",
        "Pictures",
        "Music",
        "Videos",
    ] {
        if let Ok(p) = get_known_folder(key) {
            map.insert((*key).to_string(), p);
        }
    }
    map
}

/// Return `%APPDATA%\PixoDossiers` (or `<PIXO_SANDBOX>\AppData\PixoDossiers` in sandbox mode).
pub fn get_appdata_dir() -> PathBuf {
    if let Some(sb) = get_sandbox_dir() {
        let dir = sb.join("AppData").join("PixoDossiers");
        let _ = fs::create_dir_all(&dir);
        return dir;
    }

    let base = sh_get_known_folder_path("RoamingAppData")
        .or_else(|| env::var("APPDATA").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("PixoDossiers");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Locate the flat `payload/` directory across portable exe, Tauri resource_dir, and dev workspace.
pub fn get_payload_dir() -> PathBuf {
    // 1. Check Tauri resource_dir if registered
    if let Ok(guard) = RESOURCE_DIR.read() {
        if let Some(res_dir) = guard.as_ref() {
            let candidate = res_dir.join("payload");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }

    // 2. Check next to the executable (portable mode)
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join("payload");
            if candidate.is_dir() {
                return candidate;
            }
            // Also check parent directories during `cargo test` / `cargo tauri dev`
            let mut cur = Some(exe_dir);
            while let Some(d) = cur {
                let c = d.join("payload");
                if c.is_dir() {
                    return c;
                }
                cur = d.parent();
            }
        }
    }

    // 3. Check current working directory and parent
    if let Ok(cwd) = env::current_dir() {
        let c1 = cwd.join("payload");
        if c1.is_dir() {
            return c1;
        }
        if let Some(p) = cwd.parent() {
            let c2 = p.join("payload");
            if c2.is_dir() {
                return c2;
            }
        }
    }

    PathBuf::from("payload")
}

/// Normalize a file or folder name: NFD decomposition, strip combining accents, lowercase, trim.
pub fn norm_name(name: &str) -> String {
    name.nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
        .trim()
        .to_string()
}

/// Find a direct child in `parent_dir` matching `target_name` (case/accent-insensitive).
/// - `want_dir = Some(true)`: only match directories
/// - `want_dir = Some(false)`: only match files
/// - `want_dir = None`: match either
pub fn find_child_matching(
    parent_dir: &Path,
    target_name: &str,
    want_dir: Option<bool>,
) -> Option<PathBuf> {
    if !parent_dir.is_dir() {
        return None;
    }
    let target_norm = norm_name(target_name);
    let entries = fs::read_dir(parent_dir).ok()?;
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if norm_name(&file_name) == target_norm {
            let full = entry.path();
            if let Some(need_dir) = want_dir {
                if need_dir && !full.is_dir() {
                    continue;
                }
                if !need_dir && !full.is_file() {
                    continue;
                }
            }
            return Some(full);
        }
    }
    None
}
