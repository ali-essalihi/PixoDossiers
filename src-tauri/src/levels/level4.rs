//! Level 4: Présentation (Documents\<group_name>\<member_name>\*.txt with "Bonjour, je suis <member_name>").

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;
use crate::safety;
use crate::scoring::{compute_verify_response, CheckItem, CheckResult, VerifyResponse};
use crate::state;

pub fn resolve_group_and_student(
    member_name: Option<&str>,
    group_name: Option<&str>,
    level_id: u32,
) -> (String, String) {
    let m = state::load_manifest(level_id);
    let st = state::load_state();

    let resolved_group = group_name
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            m.get("group_name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            st.get("cur")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "Groupe".to_string());

    let resolved_student = member_name
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            m.get("student_name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            st.get("groups")
                .and_then(|g| g.get(&resolved_group))
                .and_then(|grp| grp.get("members"))
                .and_then(|arr| arr.as_array())
                .and_then(|arr| arr.first())
                .and_then(|first| first.get("n"))
                .and_then(|n| n.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "Pixo".to_string());

    (resolved_group, resolved_student)
}

fn read_text_any_encoding(file_path: &Path) -> String {
    let raw = match fs::read(file_path) {
        Ok(b) if !b.is_empty() => b,
        _ => return String::new(),
    };

    // Check UTF-16LE BOM
    if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xFE {
        let u16_slice: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&u16_slice);
    }

    // Check UTF-8 (with optional UTF-8 BOM)
    let slice = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        &raw[..]
    };
    if let Ok(s) = std::str::from_utf8(slice) {
        return s.to_string();
    }

    // Fallback to Windows-1252 / Latin-1 byte-to-char mapping
    slice.iter().map(|&b| b as char).collect()
}

fn norm_phrase(text: &str) -> String {
    let base = paths::norm_name(text);
    // Normalize spaces around commas and collapse whitespace
    let cleaned = base.replace(',', " , ");
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    let mut joined = tokens.join(" ").replace(" ,", ",");
    while joined.ends_with('.') || joined.ends_with('!') || joined.ends_with(' ') {
        joined.pop();
    }
    joined.trim().to_string()
}

pub fn prepare(member_name: Option<&str>, group_name: Option<&str>) -> Result<(), String> {
    let (grp_name, student_name) = resolve_group_and_student(member_name, group_name, 4);
    state::save_manifest(
        4,
        &json!({
            "group_name": grp_name,
            "student_name": student_name
        }),
    )?;

    let docs = paths::get_known_folder("Documents")?;
    if let Some(grp_dir) = paths::find_child_matching(&docs, &grp_name, Some(true)) {
        if let Some(student_dir) = paths::find_child_matching(&grp_dir, &student_name, Some(true)) {
            safety::safe_remove_path(&student_dir)?;
        }
    }

    Ok(())
}

pub fn verify(member_name: Option<&str>, group_name: Option<&str>) -> Result<VerifyResponse, String> {
    let (grp_name, student_name) = resolve_group_and_student(member_name, group_name, 4);
    let expected_sentence = format!("Bonjour, je suis {}", student_name);
    let expected_with_comma = norm_phrase(&expected_sentence);
    let expected_no_comma = norm_phrase(&format!("Bonjour je suis {}", student_name));

    let docs = paths::get_known_folder("Documents")?;
    let grp_dir = paths::find_child_matching(&docs, &grp_name, Some(true));
    let ok_group = grp_dir.is_some();

    let student_dir = grp_dir
        .as_ref()
        .and_then(|g| paths::find_child_matching(g, &student_name, Some(true)));
    let ok_student = student_dir.is_some();

    let mut txt_files: Vec<(String, PathBuf)> = Vec::new();
    if let Some(ref sdir) = student_dir {
        if let Ok(rd) = fs::read_dir(sdir) {
            for entry in rd.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if p.is_file() && name.to_lowercase().ends_with(".txt") {
                    txt_files.push((name, p));
                }
            }
        }
    }
    txt_files.sort_by(|a, b| a.0.cmp(&b.0));

    let ok_txt_exists = !txt_files.is_empty();
    let mut ok_content = false;
    let mut content_earned = 0.0;
    let mut content_hint = String::new();
    let mut content_items = Vec::new();

    if !ok_group {
        content_hint = format!(
            "💡 Crée d'abord le dossier du groupe « {} » dans Documents.",
            grp_name
        );
    } else if !ok_student {
        content_hint = format!(
            "💡 Crée ton dossier « {} » dans Documents/{}.",
            student_name, grp_name
        );
    } else if !ok_txt_exists {
        content_hint = format!(
            "💡 Crée un fichier texte (.txt) et écris « {} ».",
            expected_sentence
        );
    } else {
        let mut any_non_empty = false;
        let mut comma_missing_match = false;

        for (_fname, fpath) in &txt_files {
            let content = read_text_any_encoding(fpath);
            if !content.trim().is_empty() {
                any_non_empty = true;
            }
            let norm_c = norm_phrase(&content);
            if norm_c == expected_with_comma {
                ok_content = true;
                content_earned = 4.0;
                break;
            }
            if norm_c == expected_no_comma {
                comma_missing_match = true;
            }
        }

        if !ok_content {
            content_items.push(CheckItem {
                name: txt_files[0].0.clone(),
                kind: "txt".to_string(),
            });
            if comma_missing_match {
                content_earned = 2.0;
                content_hint = format!(
                    "💡 Presque ! N'oublie pas la virgule : écris « {} » et enregistre (Ctrl + S).",
                    expected_sentence
                );
            } else if !any_non_empty {
                content_hint = format!(
                    "💡 Le fichier texte est vide : écris « {} » et enregistre (Ctrl + S) !",
                    expected_sentence
                );
            } else {
                content_hint = format!(
                    "💡 Écris exactement « {} » dans le fichier texte et enregistre (Ctrl + S).",
                    expected_sentence
                );
            }
        }
    }

    let checks = vec![
        CheckResult {
            id: "group_folder".to_string(),
            ok: ok_group,
            weight: 2.0,
            earned: if ok_group { 2.0 } else { 0.0 },
            label: format!("Le dossier du groupe « {} » existe dans Documents", grp_name),
            hint: if ok_group {
                String::new()
            } else {
                format!(
                    "💡 Crée le dossier du groupe « {} » dans Documents :",
                    grp_name
                )
            },
            items: if ok_group {
                Vec::new()
            } else {
                vec![CheckItem {
                    name: grp_name.clone(),
                    kind: "folder".to_string(),
                }]
            },
        },
        CheckResult {
            id: "student_folder".to_string(),
            ok: ok_student,
            weight: 2.0,
            earned: if ok_student { 2.0 } else { 0.0 },
            label: format!(
                "Le dossier « {} » existe dans Documents/{}",
                student_name, grp_name
            ),
            hint: if ok_student {
                String::new()
            } else {
                format!(
                    "💡 Crée ton dossier « {} » à l'intérieur de Documents/{} :",
                    student_name, grp_name
                )
            },
            items: if ok_student {
                Vec::new()
            } else {
                vec![CheckItem {
                    name: format!("{}/{}", grp_name, student_name),
                    kind: "folder".to_string(),
                }]
            },
        },
        CheckResult {
            id: "txt_file_exists".to_string(),
            ok: ok_txt_exists,
            weight: 2.0,
            earned: if ok_txt_exists { 2.0 } else { 0.0 },
            label: format!(
                "Un fichier texte (.txt) existe dans Documents/{}/{}",
                grp_name, student_name
            ),
            hint: if ok_txt_exists {
                String::new()
            } else {
                format!(
                    "💡 Crée un fichier texte (.txt) dans ton dossier « {} » :",
                    student_name
                )
            },
            items: if ok_txt_exists {
                Vec::new()
            } else {
                vec![CheckItem {
                    name: "presentation.txt".to_string(),
                    kind: "txt".to_string(),
                }]
            },
        },
        CheckResult {
            id: "txt_content".to_string(),
            ok: ok_content,
            weight: 4.0,
            earned: content_earned,
            label: format!("Le fichier contient le texte « {} »", expected_sentence),
            hint: content_hint,
            items: content_items,
        },
    ];

    Ok(compute_verify_response(checks, None))
}
