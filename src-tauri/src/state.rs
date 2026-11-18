//! Persistent atomic JSON state and per-level manifest storage in `%APPDATA%\PixoDossiers`.

use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;

fn atomic_write_json(target_path: &Path, data: &Value) -> Result<(), String> {
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_name = format!(".tmp_{}_{}.json", process::id(), nanos);
    let tmp_path = target_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(tmp_name);

    let serialized = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    {
        let mut f = fs::File::create(&tmp_path).map_err(|e| e.to_string())?;
        f.write_all(serialized.as_bytes())
            .map_err(|e| e.to_string())?;
        let _ = f.sync_all();
    }
    fs::rename(&tmp_path, target_path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        e.to_string()
    })
}

pub fn get_state_path() -> PathBuf {
    paths::get_appdata_dir().join("state.json")
}

pub fn default_state() -> Value {
    json!({
        "v": 1,
        "groups": {},
        "cur": null,
        "wipe_confirmed": false
    })
}

pub fn load_state() -> Value {
    let st_path = get_state_path();
    if !st_path.is_file() {
        return default_state();
    }
    if let Ok(raw) = fs::read_to_string(&st_path) {
        if let Ok(mut val) = serde_json::from_str::<Value>(&raw) {
            if val.get("v").and_then(|v| v.as_i64()) == Some(1) && val.is_object() {
                let obj = val.as_object_mut().unwrap();
                if !obj.get("groups").map(|g| g.is_object()).unwrap_or(false) {
                    obj.insert("groups".to_string(), json!({}));
                }
                if !obj.contains_key("cur") {
                    obj.insert("cur".to_string(), Value::Null);
                }
                if !obj.contains_key("wipe_confirmed") {
                    obj.insert("wipe_confirmed".to_string(), Value::Bool(false));
                }
                return val;
            }
        }
    }
    default_state()
}

pub fn save_state(mut data: Value) -> Result<(), String> {
    let obj = data
        .as_object_mut()
        .ok_or_else(|| "Format d'état invalide".to_string())?;
    obj.insert("v".to_string(), json!(1));
    if !obj.get("groups").map(|g| g.is_object()).unwrap_or(false) {
        obj.insert("groups".to_string(), json!({}));
    }
    if !obj.contains_key("wipe_confirmed") {
        obj.insert("wipe_confirmed".to_string(), Value::Bool(false));
    }
    atomic_write_json(&get_state_path(), &data)
}

pub fn get_manifest_path(level_id: u32) -> PathBuf {
    paths::get_appdata_dir().join(format!("manifest_level{}.json", level_id))
}

pub fn load_manifest(level_id: u32) -> Value {
    let m_path = get_manifest_path(level_id);
    if let Ok(raw) = fs::read_to_string(&m_path) {
        if let Ok(val) = serde_json::from_str::<Value>(&raw) {
            if val.is_object() {
                return val;
            }
        }
    }
    json!({})
}

pub fn save_manifest(level_id: u32, data: &Value) -> Result<(), String> {
    atomic_write_json(&get_manifest_path(level_id), data)
}
