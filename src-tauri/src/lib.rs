//! Main Tauri application library and exposed commands for Pixo et les Dossiers.

pub mod config;
pub mod levels;
pub mod paths;
pub mod recyclebin;
pub mod safety;
pub mod scoring;
pub mod state;

use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tauri::Manager;

use crate::scoring::VerifyResponse;

#[tauri::command]
pub fn load_state() -> Value {
    state::load_state()
}

#[tauri::command]
pub fn save_state(data: Value) -> Value {
    match state::save_state(data) {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
pub fn check_environment() -> Value {
    let missing = levels::check_missing_payload(None);
    let mut by_level = HashMap::new();
    for lv in 1..=5 {
        by_level.insert(lv.to_string(), levels::check_missing_payload(Some(lv)));
    }

    let banner = if !missing.is_empty() {
        let preview: Vec<String> = missing.iter().take(5).cloned().collect();
        let ellipsis = if missing.len() > 5 { "…" } else { "" };
        format!(
            "Il manque {} fichier(s) dans le dossier payload/ ({}{}).",
            missing.len(),
            preview.join(", "),
            ellipsis
        )
    } else {
        String::new()
    };

    json!({
        "ok": missing.is_empty(),
        "missing_payload": missing,
        "missing_by_level": by_level,
        "sandbox": paths::is_sandbox(),
        "banner": banner
    })
}

#[tauri::command]
pub fn start_task(
    level_id: u32,
    member_name: Option<String>,
    group_name: Option<String>,
) -> Value {
    let missing = levels::check_missing_payload(Some(level_id));
    if !missing.is_empty() {
        let preview: Vec<String> = missing.into_iter().take(5).collect();
        return json!({
            "ok": false,
            "error": format!("Fichiers manquants dans payload/ : {}", preview.join(", "))
        });
    }

    match levels::prepare_level(level_id, member_name.as_deref(), group_name.as_deref()) {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
pub fn verify_task(
    level_id: u32,
    member_name: Option<String>,
    group_name: Option<String>,
) -> VerifyResponse {
    match levels::verify_level(level_id, member_name.as_deref(), group_name.as_deref()) {
        Ok(res) => res,
        Err(e) => VerifyResponse {
            checks: Vec::new(),
            score: 0,
            stars: 0,
            note: None,
            error: Some(e),
        },
    }
}

#[tauri::command]
pub fn open_folder(key: String) -> Value {
    if key == "RecycleBin" {
        let res = if paths::is_sandbox() {
            let sb_bin = paths::get_sandbox_dir().unwrap().join("$Recycle.Bin");
            let _ = fs::create_dir_all(&sb_bin);
            Command::new("explorer.exe").arg(sb_bin).spawn()
        } else {
            Command::new("explorer.exe")
                .arg("shell:RecycleBinFolder")
                .spawn()
        };
        return match res {
            Ok(_) => json!({ "ok": true }),
            Err(e) => json!({ "ok": false, "error": e.to_string() }),
        };
    }

    match paths::get_known_folder(&key) {
        Ok(folder_path) => {
            let _ = fs::create_dir_all(&folder_path);
            match Command::new("explorer.exe").arg(folder_path).spawn() {
                Ok(_) => json!({ "ok": true }),
                Err(e) => json!({ "ok": false, "error": e.to_string() }),
            }
        }
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Ok(res_dir) = app.path().resource_dir() {
                paths::set_resource_dir(res_dir);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_state,
            save_state,
            check_environment,
            start_task,
            verify_task,
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_all_levels_in_sandbox() {
        let sb = env::temp_dir().join(format!("pixo_rust_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&sb);
        fs::create_dir_all(&sb).unwrap();
        env::set_var("PIXO_SANDBOX", &sb);

        // 1. State round-trip
        let st = load_state();
        assert_eq!(st.get("v").and_then(|v| v.as_i64()), Some(1));
        let new_st = serde_json::json!({
            "v": 1,
            "cur": "Les Explorateurs",
            "groups": {
                "Les Explorateurs": {
                    "members": [{ "n": "Lina", "av": "🦊" }],
                    "prog": {},
                    "start": 0
                }
            },
            "wipe_confirmed": true
        });
        save_state(new_st);
        let loaded = load_state();
        assert_eq!(
            loaded.get("cur").and_then(|v| v.as_str()),
            Some("Les Explorateurs")
        );

        // 2. Safety layer refuses outside allowed roots and Known Folder roots
        let outside = sb.join("secret.txt");
        fs::write(&outside, b"keep").unwrap();
        assert!(safety::safe_remove_path(&outside).is_err());
        let desktop = paths::get_known_folder("Desktop").unwrap();
        assert!(safety::safe_remove_path(&desktop).is_err());

        // 3. Level 4 (Présentation)
        let r4 = start_task(
            4,
            Some("Lina".to_string()),
            Some("Les Explorateurs".to_string()),
        );
        assert_eq!(r4.get("ok").and_then(|v| v.as_bool()), Some(true));
        let docs = paths::get_known_folder("Documents").unwrap();
        let lina_dir = docs.join("Les Explorateurs").join("Lina");
        fs::create_dir_all(&lina_dir).unwrap();
        fs::write(lina_dir.join("bonjour.txt"), "Bonjour, je suis Lina\n").unwrap();
        let v4 = verify_task(
            4,
            Some("Lina".to_string()),
            Some("Les Explorateurs".to_string()),
        );
        assert_eq!(v4.stars, 3);
        assert_eq!(v4.score, 100);

        // 4. Level 5 (Dessin Paint)
        let r5 = start_task(
            5,
            Some("Lina".to_string()),
            Some("Les Explorateurs".to_string()),
        );
        assert_eq!(r5.get("ok").and_then(|v| v.as_bool()), Some(true));
        let png_bytes: &[u8] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        fs::write(lina_dir.join("mon-dessin.png"), png_bytes).unwrap();
        let v5 = verify_task(
            5,
            Some("Lina".to_string()),
            Some("Les Explorateurs".to_string()),
        );
        assert_eq!(v5.stars, 3);
        assert_eq!(v5.score, 100);

        let _ = fs::remove_dir_all(&sb);
        let _ = PathBuf::from(&sb);
    }
}
