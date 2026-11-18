//! Windows Recycle Bin (`$Recycle.Bin\<SID>\$I*`) binary metadata reader (v1 & v2).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

#[derive(Debug, Clone)]
pub struct RecycleEntry {
    pub version: u64,
    pub size: u64,
    pub deleted_time: u64,
    pub original_path: String,
}

#[cfg(target_os = "windows")]
#[link(name = "advapi32")]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut core::ffi::c_void;
    fn OpenProcessToken(
        process_handle: *mut core::ffi::c_void,
        desired_access: u32,
        token_handle: *mut *mut core::ffi::c_void,
    ) -> i32;
    fn GetTokenInformation(
        token_handle: *mut core::ffi::c_void,
        token_information_class: u32,
        token_information: *mut core::ffi::c_void,
        token_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(
        sid: *mut core::ffi::c_void,
        string_sid: *mut *mut u16,
    ) -> i32;
    fn LocalFree(hmem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn CloseHandle(hobject: *mut core::ffi::c_void) -> i32;
}

#[cfg(target_os = "windows")]
fn get_current_user_sid() -> Option<String> {
    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_USER_CLASS: u32 = 1;

    unsafe {
        let mut token: *mut core::ffi::c_void = core::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 || token.is_null() {
            return None;
        }
        let mut needed: u32 = 0;
        GetTokenInformation(
            token,
            TOKEN_USER_CLASS,
            core::ptr::null_mut(),
            0,
            &mut needed,
        );
        if needed == 0 {
            CloseHandle(token);
            return None;
        }
        let mut buf = vec![0u8; needed as usize];
        if GetTokenInformation(
            token,
            TOKEN_USER_CLASS,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            needed,
            &mut needed,
        ) == 0
        {
            CloseHandle(token);
            return None;
        }
        let sid_ptr = *(buf.as_ptr() as *const *mut core::ffi::c_void);
        let mut str_ptr: *mut u16 = core::ptr::null_mut();
        let mut result = None;
        if ConvertSidToStringSidW(sid_ptr, &mut str_ptr) != 0 && !str_ptr.is_null() {
            let mut len = 0usize;
            while *str_ptr.add(len) != 0 {
                len += 1;
            }
            let slice = core::slice::from_raw_parts(str_ptr, len);
            result = Some(String::from_utf16_lossy(slice));
            LocalFree(str_ptr as *mut core::ffi::c_void);
        }
        CloseHandle(token);
        result
    }
}

#[cfg(not(target_os = "windows"))]
fn get_current_user_sid() -> Option<String> {
    None
}

fn decode_utf16le_until_null(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 2 {
        return None;
    }
    let u16_len = bytes.len() / 2;
    let mut units = Vec::with_capacity(u16_len);
    for i in 0..u16_len {
        let val = u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]);
        if val == 0 {
            break;
        }
        units.push(val);
    }
    if units.is_empty() {
        return None;
    }
    Some(String::from_utf16_lossy(&units))
}

/// Parse a Windows `$Recycle.Bin` `$Ixxxxxx` metadata file (format v1 or v2).
pub fn parse_i_file(file_path: &Path) -> Option<RecycleEntry> {
    let raw = fs::read(file_path).ok()?;
    if raw.len() < 28 {
        return None;
    }

    let version = u64::from_le_bytes(raw[0..8].try_into().ok()?);
    let size = u64::from_le_bytes(raw[8..16].try_into().ok()?);
    let deleted_time = u64::from_le_bytes(raw[16..24].try_into().ok()?);

    if version == 1 {
        let end = raw.len().min(544);
        let orig_path = decode_utf16le_until_null(&raw[24..end])?;
        return Some(RecycleEntry {
            version: 1,
            size,
            deleted_time,
            original_path: orig_path,
        });
    }

    if version == 2 {
        let char_count = u32::from_le_bytes(raw[24..28].try_into().ok()?) as usize;
        if char_count == 0 || char_count > 32768 {
            return None;
        }
        let byte_len = char_count * 2;
        let end = raw.len().min(28 + byte_len);
        let orig_path = decode_utf16le_until_null(&raw[28..end])?;
        return Some(RecycleEntry {
            version: 2,
            size,
            deleted_time,
            original_path: orig_path,
        });
    }

    None
}

fn drive_prefix(path: &Path) -> Option<String> {
    let s = path.to_string_lossy();
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        Some(s[0..2].to_uppercase())
    } else {
        None
    }
}

pub fn get_recycle_bin_sid_dirs(reference_paths: &[&Path]) -> Vec<PathBuf> {
    if let Some(sb) = paths::get_sandbox_dir() {
        if env::var("PIXO_SANDBOX_NO_RECYCLEBIN").unwrap_or_default().trim() == "1" {
            return Vec::new();
        }
        let sb_bin = sb.join("$Recycle.Bin").join("S-1-5-21-SANDBOX");
        let _ = fs::create_dir_all(&sb_bin);
        return vec![sb_bin];
    }

    let mut drives = Vec::new();
    if let Ok(up) = env::var("USERPROFILE") {
        if let Some(d) = drive_prefix(Path::new(&up)) {
            drives.push(d);
        }
    }
    for p in reference_paths {
        if let Some(d) = drive_prefix(p) {
            if !drives.contains(&d) {
                drives.push(d);
            }
        }
    }
    if drives.is_empty() {
        drives.push("C:".to_string());
    }

    let sid = get_current_user_sid();
    let mut sid_dirs = Vec::new();

    for d in drives {
        let rb_root = PathBuf::from(format!("{}\\$Recycle.Bin", d));
        if let Some(ref s) = sid {
            let candidate = rb_root.join(s);
            if fs::read_dir(&candidate).is_ok() {
                sid_dirs.push(candidate);
            }
        }
        if let Ok(entries) = fs::read_dir(&rb_root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && !sid_dirs.contains(&p) && fs::read_dir(&p).is_ok() {
                    sid_dirs.push(p);
                }
            }
        }
    }

    sid_dirs
}

/// Read all `$I*` metadata entries from the current user's Recycle Bin.
/// Returns `(accessible, entries)`.
pub fn read_recycle_bin_entries(reference_paths: &[&Path]) -> (bool, Vec<RecycleEntry>) {
    let sid_dirs = get_recycle_bin_sid_dirs(reference_paths);
    if sid_dirs.is_empty() {
        return (false, Vec::new());
    }

    let mut entries = Vec::new();
    let mut any_readable = false;

    for sdir in sid_dirs {
        if let Ok(rd) = fs::read_dir(&sdir) {
            any_readable = true;
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.to_uppercase().starts_with("$I") {
                    let full = entry.path();
                    if full.is_file() {
                        if let Some(parsed) = parse_i_file(&full) {
                            entries.push(parsed);
                        }
                    }
                }
            }
        }
    }

    (any_readable, entries)
}

fn norm_dir_str(p: &Path) -> String {
    p.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// Return true if `filename` originally inside `expected_parent_dir` matches any `$I` entry.
pub fn is_file_in_recycle_bin(
    filename: &str,
    expected_parent_dir: &Path,
    entries: &[RecycleEntry],
) -> bool {
    let target_norm = paths::norm_name(filename);
    let parent_norm = norm_dir_str(expected_parent_dir);

    for item in entries {
        let orig_path = Path::new(&item.original_path);
        let orig_base = orig_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if paths::norm_name(&orig_base) != target_norm {
            continue;
        }
        if let Some(orig_parent) = orig_path.parent() {
            if norm_dir_str(orig_parent) == parent_norm {
                return true;
            }
        }
    }
    false
}
