//! Level 5: Dessin Paint (create mon-dessin.png inside Documents\<group_name>\<member_name>).

use serde_json::json;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::levels::level4::resolve_group_and_student;
use crate::paths;
use crate::safety;
use crate::scoring::{compute_verify_response, CheckItem, CheckResult, VerifyResponse};
use crate::state;

pub const EXPECTED_DRAWING_NAME: &str = "mon-dessin.png";
const ACCEPTED_DRAWING_NORMS: &[&str] = &[
    "mon-dessin.png",
    "mon_dessin.png",
    "mon dessin.png",
    "mondessin.png",
    "mon-dessin.png.png",
    "mon_dessin.png.png",
    "mon dessin.png.png",
];
const WRONG_EXT_STEMS: &[&str] = &["mon-dessin", "mon_dessin", "mon dessin", "mondessin"];

fn find_drawing_png(folder_dir: Option<&Path>) -> Option<PathBuf> {
    let dir = folder_dir?;
    if !dir.is_dir() {
        return None;
    }
    let rd = fs::read_dir(dir).ok()?;
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let norm = paths::norm_name(&name);
        if ACCEPTED_DRAWING_NORMS.contains(&norm.as_str()) {
            return Some(p);
        }
    }
    None
}

fn find_wrong_ext_drawing(folder_dir: Option<&Path>) -> Option<String> {
    let dir = folder_dir?;
    if !dir.is_dir() {
        return None;
    }
    let rd = fs::read_dir(dir).ok()?;
    for entry in rd.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let norm = paths::norm_name(&name);
        if let Some(dot_idx) = norm.rfind('.') {
            let stem = &norm[..dot_idx];
            let ext = &norm[dot_idx..];
            if WRONG_EXT_STEMS.contains(&stem) && ext != ".png" {
                return Some(name);
            }
        }
    }
    None
}

fn is_valid_png(file_path: Option<&Path>) -> bool {
    let p = match file_path {
        Some(path) if path.is_file() => path,
        _ => return false,
    };
    let mut f = match fs::File::open(p) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut head = [0u8; 4];
    if f.read_exact(&mut head).is_ok() {
        return &head == b"\x89PNG";
    }
    false
}

pub fn prepare(member_name: Option<&str>, group_name: Option<&str>) -> Result<(), String> {
    let (grp_name, student_name) = resolve_group_and_student(member_name, group_name, 5);
    state::save_manifest(
        5,
        &json!({
            "group_name": grp_name,
            "student_name": student_name
        }),
    )?;

    let docs = paths::get_known_folder("Documents")?;
    let grp_dir = paths::find_child_matching(&docs, &grp_name, Some(true));
    let student_dir = grp_dir
        .as_ref()
        .and_then(|g| paths::find_child_matching(g, &student_name, Some(true)));

    while let Some(old_png) = find_drawing_png(student_dir.as_deref()) {
        safety::safe_remove_path(&old_png)?;
    }

    let pictures = paths::get_known_folder("Pictures")?;
    let desktop = paths::get_known_folder("Desktop")?;
    for folder in &[
        Some(docs.as_path()),
        grp_dir.as_deref(),
        Some(pictures.as_path()),
        Some(desktop.as_path()),
    ] {
        if let Some(stray) = find_drawing_png(*folder) {
            safety::safe_remove_path(&stray)?;
        }
    }

    Ok(())
}

pub fn verify(member_name: Option<&str>, group_name: Option<&str>) -> Result<VerifyResponse, String> {
    let (grp_name, student_name) = resolve_group_and_student(member_name, group_name, 5);
    let docs = paths::get_known_folder("Documents")?;
    let pictures = paths::get_known_folder("Pictures")?;
    let desktop = paths::get_known_folder("Desktop")?;

    let grp_dir = paths::find_child_matching(&docs, &grp_name, Some(true));
    let ok_group = grp_dir.is_some();

    let student_dir = grp_dir
        .as_ref()
        .and_then(|g| paths::find_child_matching(g, &student_name, Some(true)));
    let ok_student = student_dir.is_some();

    let drawing_path = find_drawing_png(student_dir.as_deref());
    let ok_exists = drawing_path.is_some();

    let target_rel = format!("Documents/{}/{}", grp_name, student_name);
    let mut exists_hint = String::new();
    let mut exists_items = Vec::new();

    if !ok_exists {
        let wrong_ext = find_wrong_ext_drawing(student_dir.as_deref());
        let in_grp_root = find_drawing_png(grp_dir.as_deref());
        let in_docs_root = find_drawing_png(Some(docs.as_path()));
        let in_pictures = find_drawing_png(Some(pictures.as_path()));
        let in_desktop = find_drawing_png(Some(desktop.as_path()));

        if let Some(w_ext) = wrong_ext {
            exists_hint = format!(
                "💡 Tu as enregistré « {} » : choisis le format PNG (.png) dans Paint !",
                w_ext
            );
            exists_items.push(CheckItem {
                name: EXPECTED_DRAWING_NAME.to_string(),
                kind: "png".to_string(),
            });
        } else if in_grp_root.is_some()
            || in_docs_root.is_some()
            || in_pictures.is_some()
            || in_desktop.is_some()
        {
            let where_str = if in_grp_root.is_some() {
                format!("Documents/{}", grp_name)
            } else if in_docs_root.is_some() {
                "Documents".to_string()
            } else if in_pictures.is_some() {
                "Images".to_string()
            } else {
                "le Bureau".to_string()
            };
            exists_hint = format!(
                "💡 Tu as enregistré « mon-dessin.png » dans {} : déplace-le dans {} !",
                where_str, target_rel
            );
            exists_items.push(CheckItem {
                name: format!("{}/{}", target_rel, EXPECTED_DRAWING_NAME),
                kind: "png".to_string(),
            });
        } else {
            exists_hint = format!(
                "💡 Dans Paint, enregistre ton dessin sous le nom « {} » dans {} :",
                EXPECTED_DRAWING_NAME, target_rel
            );
            exists_items.push(CheckItem {
                name: EXPECTED_DRAWING_NAME.to_string(),
                kind: "png".to_string(),
            });
        }
    }

    let ok_valid = if ok_exists {
        is_valid_png(drawing_path.as_deref())
    } else {
        false
    };
    let valid_hint = if ok_exists && !ok_valid {
        "💡 Le fichier « mon-dessin.png » n'est pas une vraie image PNG enregistrée par Paint."
            .to_string()
    } else if !ok_exists {
        "💡 Enregistre ton dessin au format PNG dans Paint.".to_string()
    } else {
        String::new()
    };

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
                    "💡 Crée ton dossier « {} » dans Documents/{} :",
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
            id: "png_exists".to_string(),
            ok: ok_exists,
            weight: 4.0,
            earned: if ok_exists { 4.0 } else { 0.0 },
            label: format!(
                "Le fichier « {} » se trouve dans {}",
                EXPECTED_DRAWING_NAME, target_rel
            ),
            hint: exists_hint,
            items: exists_items,
        },
        CheckResult {
            id: "png_valid".to_string(),
            ok: ok_valid,
            weight: 2.0,
            earned: if ok_valid { 2.0 } else { 0.0 },
            label: "Le dessin est bien une image PNG valide".to_string(),
            hint: valid_hint,
            items: if ok_valid {
                Vec::new()
            } else {
                vec![CheckItem {
                    name: EXPECTED_DRAWING_NAME.to_string(),
                    kind: "png".to_string(),
                }]
            },
        },
    ];

    Ok(compute_verify_response(checks, None))
}
