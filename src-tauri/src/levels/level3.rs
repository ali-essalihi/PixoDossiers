//! Level 3: Ordinateur (Pictures -> Desktop\ordinateur\{unite-centrale.png, clavier.png, souris.png, ecran.png}).

use std::fs;
use std::path::{Path, PathBuf};

use crate::config;
use crate::paths;
use crate::safety;
use crate::scoring::{compute_verify_response, CheckItem, CheckResult, VerifyResponse};

pub const ALL_LEVEL3_PNGS: &[&str] = &[
    "unite-centrale.png",
    "clavier.png",
    "souris.png",
    "ecran.png",
];

const ORDINATEUR_NAMES: &[&str] = &["ordinateur", "ordinateurs"];

fn find_matching_dirs(parent_dir: &Path, accepted_norms: &[&str]) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    if let Ok(rd) = fs::read_dir(parent_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let norm = paths::norm_name(&name);
            if accepted_norms.contains(&norm.as_str()) {
                let full = entry.path();
                if full.is_dir() {
                    matches.push(full);
                }
            }
        }
    }
    matches
}

fn has_file_variant(folder_dir: Option<&Path>, canonical_name: &str) -> bool {
    let dir = match folder_dir {
        Some(d) if d.is_dir() => d,
        _ => return false,
    };
    let variants: &[&str] = match canonical_name {
        "unite-centrale.png" => &[
            "unite-centrale.png",
            "unite_centrale.png",
            "unite centrale.png",
        ],
        "clavier.png" => &["clavier.png"],
        "souris.png" => &["souris.png"],
        "ecran.png" => &["ecran.png"],
        _ => &[canonical_name],
    };

    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let norm = paths::norm_name(&name);
            if variants.contains(&norm.as_str()) && entry.path().is_file() {
                return true;
            }
        }
    }
    false
}

pub fn prepare(_member_name: Option<&str>, _group_name: Option<&str>) -> Result<(), String> {
    let pictures = paths::get_known_folder("Pictures")?;
    let desktop = paths::get_known_folder("Desktop")?;
    let payload_dir = paths::get_payload_dir();

    if config::LEVEL2_WIPE_WHOLE_FOLDERS {
        safety::safe_clear_folder_contents(&pictures)?;
    } else {
        for &png in ALL_LEVEL3_PNGS {
            if let Some(existing) = paths::find_child_matching(&pictures, png, Some(false)) {
                safety::safe_remove_path(&existing)?;
            }
        }
    }

    for folder in find_matching_dirs(
        &desktop,
        &["ordinateur", "ordinateurs", "peripheriques", "peripherique"],
    ) {
        safety::safe_remove_path(&folder)?;
    }

    for &png in ALL_LEVEL3_PNGS {
        let src = payload_dir.join(png);
        let dst = pictures.join(png);
        safety::safe_copy_file(&src, &dst)?;
    }

    Ok(())
}

pub fn verify(_member_name: Option<&str>, _group_name: Option<&str>) -> Result<VerifyResponse, String> {
    let desktop = paths::get_known_folder("Desktop")?;
    let pictures = paths::get_known_folder("Pictures")?;

    let ord_matches = find_matching_dirs(&desktop, ORDINATEUR_NAMES);
    let ord_dir = ord_matches.first().map(|p| p.as_path());
    let ok_ord = ord_dir.is_some();

    let missing_files: Vec<&str> = ALL_LEVEL3_PNGS
        .iter()
        .copied()
        .filter(|&png| !has_file_variant(ord_dir, png))
        .collect();
    let files_earned = (2 * (ALL_LEVEL3_PNGS.len() - missing_files.len())) as f64;

    let left_in_pictures: Vec<&str> = ALL_LEVEL3_PNGS
        .iter()
        .copied()
        .filter(|&png| paths::find_child_matching(&pictures, png, Some(false)).is_some())
        .collect();
    let pic_earned = 3.0 * ((ALL_LEVEL3_PNGS.len() - left_in_pictures.len()) as f64)
        / (ALL_LEVEL3_PNGS.len() as f64);

    let checks = vec![
        CheckResult {
            id: "ordinateur_folder".to_string(),
            ok: ok_ord,
            weight: 2.0,
            earned: if ok_ord { 2.0 } else { 0.0 },
            label: "Le dossier « ordinateur » existe sur le Bureau".to_string(),
            hint: if ok_ord {
                String::new()
            } else {
                "💡 Crée le dossier « ordinateur » sur le Bureau :".to_string()
            },
            items: if ok_ord {
                Vec::new()
            } else {
                vec![CheckItem {
                    name: "ordinateur".to_string(),
                    kind: "folder".to_string(),
                }]
            },
        },
        CheckResult {
            id: "files_placed".to_string(),
            ok: missing_files.is_empty(),
            weight: 8.0,
            earned: files_earned,
            label: "Les 4 images (unite-centrale, clavier, souris, ecran) sont dans « ordinateur »"
                .to_string(),
            hint: if missing_files.is_empty() {
                String::new()
            } else {
                "💡 Déplace ces images dans le dossier « ordinateur » :".to_string()
            },
            items: missing_files
                .iter()
                .map(|f| CheckItem {
                    name: (*f).to_string(),
                    kind: "png".to_string(),
                })
                .collect(),
        },
        CheckResult {
            id: "pictures_clean".to_string(),
            ok: left_in_pictures.is_empty(),
            weight: 3.0,
            earned: pic_earned,
            label: "Aucune des 4 images n'est restée dans Images (déplacée, pas copiée)"
                .to_string(),
            hint: if left_in_pictures.is_empty() {
                String::new()
            } else {
                "💡 Déplace (ne copie pas) ces images hors du dossier Images :".to_string()
            },
            items: left_in_pictures
                .iter()
                .map(|f| CheckItem {
                    name: (*f).to_string(),
                    kind: "png".to_string(),
                })
                .collect(),
        },
    ];

    Ok(compute_verify_response(checks, None))
}
