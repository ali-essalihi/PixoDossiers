//! Level 2: Nourriture (Pictures -> create fruits, legumes, boissons folders and sort 14 JPGs).

use std::fs;
use std::path::{Path, PathBuf};

use crate::config;
use crate::paths;
use crate::safety;
use crate::scoring::{compute_verify_response, CheckItem, CheckResult, VerifyResponse};

pub const FRUITS_FILES: &[&str] = &[
    "pomme.jpg",
    "banane.jpg",
    "orange.jpg",
    "fraise.jpg",
    "citron.jpg",
    "cerise.jpg",
];

pub const LEGUMES_FILES: &[&str] = &[
    "carotte.jpg",
    "pomme_de_terre.jpg",
    "brocoli.jpg",
    "salade.jpg",
];

pub const BOISSONS_FILES: &[&str] = &[
    "eau.jpg",
    "jus_de_pomme.jpg",
    "cafe.jpg",
    "the.jpg",
];

pub fn all_food_files() -> Vec<&'static str> {
    let mut v = Vec::new();
    v.extend_from_slice(FRUITS_FILES);
    v.extend_from_slice(LEGUMES_FILES);
    v.extend_from_slice(BOISSONS_FILES);
    v
}

fn category_variants(cat: &str) -> &'static [&'static str] {
    match cat {
        "fruits" => &["fruits", "fruit"],
        "legumes" => &["legumes", "legume"],
        "boissons" => &["boissons", "boisson"],
        _ => &[],
    }
}

fn find_category_dir(pictures_dir: &Path, cat: &str) -> Option<PathBuf> {
    if !pictures_dir.is_dir() {
        return None;
    }
    let variants = category_variants(cat);
    let rd = fs::read_dir(pictures_dir).ok()?;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let norm = paths::norm_name(&name);
        if variants.contains(&norm.as_str()) {
            let full = entry.path();
            if full.is_dir() {
                return Some(full);
            }
        }
    }
    None
}

pub fn prepare(_member_name: Option<&str>, _group_name: Option<&str>) -> Result<(), String> {
    let pictures = paths::get_known_folder("Pictures")?;
    let payload_dir = paths::get_payload_dir();

    if config::LEVEL2_WIPE_WHOLE_FOLDERS {
        safety::safe_clear_folder_contents(&pictures)?;
    } else {
        for cat in &["fruits", "legumes", "boissons"] {
            if let Some(cdir) = find_category_dir(&pictures, cat) {
                safety::safe_remove_path(&cdir)?;
            }
        }
        for fname in all_food_files() {
            if let Some(existing) = paths::find_child_matching(&pictures, fname, Some(false)) {
                safety::safe_remove_path(&existing)?;
            }
        }
    }

    for fname in all_food_files() {
        let src = payload_dir.join(fname);
        let dst = pictures.join(fname);
        safety::safe_copy_file(&src, &dst)?;
    }

    Ok(())
}

pub fn verify(_member_name: Option<&str>, _group_name: Option<&str>) -> Result<VerifyResponse, String> {
    let pictures = paths::get_known_folder("Pictures")?;

    let fruits_dir = find_category_dir(&pictures, "fruits");
    let legumes_dir = find_category_dir(&pictures, "legumes");
    let boissons_dir = find_category_dir(&pictures, "boissons");

    let mut missing_cats = Vec::new();
    if fruits_dir.is_none() {
        missing_cats.push("fruits");
    }
    if legumes_dir.is_none() {
        missing_cats.push("legumes");
    }
    if boissons_dir.is_none() {
        missing_cats.push("boissons");
    }

    let missing_fruits: Vec<&str> = FRUITS_FILES
        .iter()
        .copied()
        .filter(|f| {
            fruits_dir
                .as_ref()
                .and_then(|d| paths::find_child_matching(d, f, Some(false)))
                .is_none()
        })
        .collect();

    let missing_legumes: Vec<&str> = LEGUMES_FILES
        .iter()
        .copied()
        .filter(|f| {
            legumes_dir
                .as_ref()
                .and_then(|d| paths::find_child_matching(d, f, Some(false)))
                .is_none()
        })
        .collect();

    let missing_boissons: Vec<&str> = BOISSONS_FILES
        .iter()
        .copied()
        .filter(|f| {
            boissons_dir
                .as_ref()
                .and_then(|d| paths::find_child_matching(d, f, Some(false)))
                .is_none()
        })
        .collect();

    let all_files = all_food_files();
    let left_in_root: Vec<&str> = all_files
        .iter()
        .copied()
        .filter(|f| paths::find_child_matching(&pictures, f, Some(false)).is_some())
        .collect();
    let root_earned =
        3.0 * ((all_files.len() - left_in_root.len()) as f64) / (all_files.len() as f64);

    let checks = vec![
        CheckResult {
            id: "folders_created".to_string(),
            ok: missing_cats.is_empty(),
            weight: 3.0,
            earned: (3 - missing_cats.len()) as f64,
            label: "Les 3 dossiers (fruits, legumes, boissons) existent dans Images".to_string(),
            hint: if missing_cats.is_empty() {
                String::new()
            } else {
                "💡 Crée ces dossiers dans Images :".to_string()
            },
            items: missing_cats
                .iter()
                .map(|c| CheckItem {
                    name: (*c).to_string(),
                    kind: "folder".to_string(),
                })
                .collect(),
        },
        CheckResult {
            id: "fruits_placed".to_string(),
            ok: missing_fruits.is_empty(),
            weight: 6.0,
            earned: (6 - missing_fruits.len()) as f64,
            label: "Les 6 images de fruits sont dans le dossier « fruits »".to_string(),
            hint: if missing_fruits.is_empty() {
                String::new()
            } else {
                "💡 Mets ces fruits dans le dossier « fruits » :".to_string()
            },
            items: missing_fruits
                .iter()
                .map(|f| CheckItem {
                    name: (*f).to_string(),
                    kind: "jpg".to_string(),
                })
                .collect(),
        },
        CheckResult {
            id: "legumes_placed".to_string(),
            ok: missing_legumes.is_empty(),
            weight: 4.0,
            earned: (4 - missing_legumes.len()) as f64,
            label: "Les 4 images de légumes sont dans le dossier « legumes »".to_string(),
            hint: if missing_legumes.is_empty() {
                String::new()
            } else {
                "💡 Mets ces légumes dans le dossier « legumes » :".to_string()
            },
            items: missing_legumes
                .iter()
                .map(|f| CheckItem {
                    name: (*f).to_string(),
                    kind: "jpg".to_string(),
                })
                .collect(),
        },
        CheckResult {
            id: "boissons_placed".to_string(),
            ok: missing_boissons.is_empty(),
            weight: 4.0,
            earned: (4 - missing_boissons.len()) as f64,
            label: "Les 4 images de boissons sont dans le dossier « boissons »".to_string(),
            hint: if missing_boissons.is_empty() {
                String::new()
            } else {
                "💡 Mets ces boissons dans le dossier « boissons » :".to_string()
            },
            items: missing_boissons
                .iter()
                .map(|f| CheckItem {
                    name: (*f).to_string(),
                    kind: "jpg".to_string(),
                })
                .collect(),
        },
        CheckResult {
            id: "pictures_root_clean".to_string(),
            ok: left_in_root.is_empty(),
            weight: 3.0,
            earned: root_earned,
            label: "Aucune image n'est restée en vrac dans Images (déplacée, pas copiée)"
                .to_string(),
            hint: if left_in_root.is_empty() {
                String::new()
            } else {
                "💡 Déplace (ne copie pas) ces images dans leur dossier :".to_string()
            },
            items: left_in_root
                .iter()
                .take(6)
                .map(|f| CheckItem {
                    name: (*f).to_string(),
                    kind: "jpg".to_string(),
                })
                .collect(),
        },
    ];

    Ok(compute_verify_response(checks, None))
}
