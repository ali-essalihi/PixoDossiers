//! Level 1: Tri (Downloads -> Pictures / Music / Videos / Recycle Bin with 2 dump files).

use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::paths;
use crate::recyclebin;
use crate::safety;
use crate::scoring::{compute_verify_response, CheckItem, CheckResult, VerifyResponse};
use crate::state;

pub const PNG_FILES: &[&str] = &[
    "clavier.png",
    "souris.png",
    "ecran.png",
    "unite-centrale.png",
];
pub const MP4_FILES: &[&str] = &["imprimante-demo.mp4", "scanner-demo.mp4"];
pub const MP3_FILES: &[&str] = &["pluie.mp3", "tonnerre.mp3"];
pub const DUMP_PREFIXES: &[(&str, &str)] = &[("vieux_doc", ".tmp"), ("brouillon_sale", ".bak")];

static COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_random_suffix() -> String {
    const CHARS: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(123456789);
    let cnt = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut seed = nanos
        ^ ((process::id() as u64) << 16)
        ^ cnt.wrapping_mul(0x9E3779B97F4A7C15);
    let mut out = String::with_capacity(4);
    for _ in 0..4 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let idx = (seed as usize) % CHARS.len();
        out.push(CHARS[idx] as char);
    }
    out
}

fn all_media_files() -> Vec<&'static str> {
    let mut v = Vec::new();
    v.extend_from_slice(PNG_FILES);
    v.extend_from_slice(MP4_FILES);
    v.extend_from_slice(MP3_FILES);
    v
}

fn kind_for_name(fname: &str) -> &'static str {
    let low = fname.to_lowercase();
    if low.ends_with(".png") || low.ends_with(".jpg") {
        "png"
    } else if low.ends_with(".mp4") {
        "mp4"
    } else if low.ends_with(".mp3") {
        "mp3"
    } else {
        "txt"
    }
}

pub fn prepare(_member_name: Option<&str>, _group_name: Option<&str>) -> Result<(), String> {
    let downloads = paths::get_known_folder("Downloads")?;
    let pictures = paths::get_known_folder("Pictures")?;
    let music = paths::get_known_folder("Music")?;
    let videos = paths::get_known_folder("Videos")?;
    let payload_dir = paths::get_payload_dir();

    let suffix = generate_random_suffix();
    let dump_files: Vec<String> = DUMP_PREFIXES
        .iter()
        .map(|(p, e)| format!("{}_{}{}", p, suffix, e))
        .collect();

    let old_manifest = state::load_manifest(1);
    let old_dumps: Vec<String> = old_manifest
        .get("dumps")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Save manifest FIRST
    state::save_manifest(
        1,
        &json!({
            "dumps": dump_files,
            "suffix": suffix
        }),
    )?;

    if config::LEVEL2_WIPE_WHOLE_FOLDERS {
        for folder in &[&downloads, &pictures, &music, &videos] {
            safety::safe_clear_folder_contents(folder)?;
        }
    } else {
        for folder in &[&downloads, &pictures, &music, &videos] {
            for fname in all_media_files() {
                if let Some(found) = paths::find_child_matching(folder, fname, Some(false)) {
                    safety::safe_remove_path(&found)?;
                }
            }
            for d in old_dumps.iter().chain(dump_files.iter()) {
                if let Some(found) = paths::find_child_matching(folder, d, Some(false)) {
                    safety::safe_remove_path(&found)?;
                }
            }
        }
    }

    for fname in all_media_files() {
        let src = payload_dir.join(fname);
        let dst = downloads.join(fname);
        safety::safe_copy_file(&src, &dst)?;
    }

    for dname in &dump_files {
        let dst = downloads.join(dname);
        let text = format!(
            "Fichier jetable Pixo ({}) : mets-moi dans la Corbeille !\n",
            dname
        );
        safety::safe_write_text(&dst, &text)?;
    }

    Ok(())
}

fn scan_files_recursive(dir: &Path, out: &mut HashSet<String>) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                scan_files_recursive(&p, out);
            } else if p.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                out.insert(paths::norm_name(&name));
            }
        }
    }
}

pub fn verify(_member_name: Option<&str>, _group_name: Option<&str>) -> Result<VerifyResponse, String> {
    let downloads = paths::get_known_folder("Downloads")?;
    let pictures = paths::get_known_folder("Pictures")?;
    let music = paths::get_known_folder("Music")?;
    let videos = paths::get_known_folder("Videos")?;

    let manifest = state::load_manifest(1);
    let mut dump_files: Vec<String> = manifest
        .get("dumps")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    if dump_files.is_empty() {
        dump_files = DUMP_PREFIXES
            .iter()
            .map(|(p, e)| format!("{}{}", p, e))
            .collect();
    }

    let mut in_downloads = HashSet::new();
    let mut in_pictures = HashSet::new();
    let mut in_music = HashSet::new();
    let mut in_videos = HashSet::new();
    scan_files_recursive(&downloads, &mut in_downloads);
    scan_files_recursive(&pictures, &mut in_pictures);
    scan_files_recursive(&music, &mut in_music);
    scan_files_recursive(&videos, &mut in_videos);

    let mut checks = Vec::new();

    // 1. PNGs in Pictures (weight 4)
    let missing_png: Vec<&str> = PNG_FILES
        .iter()
        .copied()
        .filter(|f| !in_pictures.contains(&paths::norm_name(f)))
        .collect();
    checks.push(CheckResult {
        id: "png_in_pictures".to_string(),
        ok: missing_png.is_empty(),
        weight: 4.0,
        earned: (4 - missing_png.len()) as f64,
        label: "Les 4 images (.png) sont dans Images".to_string(),
        hint: if missing_png.is_empty() {
            String::new()
        } else {
            "💡 Déplace ces images dans le dossier Images :".to_string()
        },
        items: missing_png
            .iter()
            .map(|f| CheckItem {
                name: (*f).to_string(),
                kind: "png".to_string(),
            })
            .collect(),
    });

    // 2. MP4s in Videos (weight 2)
    let missing_mp4: Vec<&str> = MP4_FILES
        .iter()
        .copied()
        .filter(|f| !in_videos.contains(&paths::norm_name(f)))
        .collect();
    checks.push(CheckResult {
        id: "mp4_in_videos".to_string(),
        ok: missing_mp4.is_empty(),
        weight: 2.0,
        earned: (2 - missing_mp4.len()) as f64,
        label: "Les 2 vidéos (.mp4) sont dans Vidéos".to_string(),
        hint: if missing_mp4.is_empty() {
            String::new()
        } else {
            "💡 Déplace ces vidéos dans le dossier Vidéos :".to_string()
        },
        items: missing_mp4
            .iter()
            .map(|f| CheckItem {
                name: (*f).to_string(),
                kind: "mp4".to_string(),
            })
            .collect(),
    });

    // 3. MP3s in Music (weight 2)
    let missing_mp3: Vec<&str> = MP3_FILES
        .iter()
        .copied()
        .filter(|f| !in_music.contains(&paths::norm_name(f)))
        .collect();
    checks.push(CheckResult {
        id: "mp3_in_music".to_string(),
        ok: missing_mp3.is_empty(),
        weight: 2.0,
        earned: (2 - missing_mp3.len()) as f64,
        label: "Les 2 sons (.mp3) sont dans Musique".to_string(),
        hint: if missing_mp3.is_empty() {
            String::new()
        } else {
            "💡 Déplace ces sons dans le dossier Musique :".to_string()
        },
        items: missing_mp3
            .iter()
            .map(|f| CheckItem {
                name: (*f).to_string(),
                kind: "mp3".to_string(),
            })
            .collect(),
    });

    // 4. No media or dump file in a wrong folder (weight 2)
    let mut wrong_items = Vec::new();
    let mut wrong_details = Vec::new();
    for &f in PNG_FILES {
        let nf = paths::norm_name(f);
        if in_music.contains(&nf) || in_videos.contains(&nf) {
            wrong_items.push(CheckItem {
                name: f.to_string(),
                kind: "png".to_string(),
            });
            wrong_details.push(format!("{} → Images", f));
        }
    }
    for &f in MP4_FILES {
        let nf = paths::norm_name(f);
        if in_pictures.contains(&nf) || in_music.contains(&nf) {
            wrong_items.push(CheckItem {
                name: f.to_string(),
                kind: "mp4".to_string(),
            });
            wrong_details.push(format!("{} → Vidéos", f));
        }
    }
    for &f in MP3_FILES {
        let nf = paths::norm_name(f);
        if in_pictures.contains(&nf) || in_videos.contains(&nf) {
            wrong_items.push(CheckItem {
                name: f.to_string(),
                kind: "mp3".to_string(),
            });
            wrong_details.push(format!("{} → Musique", f));
        }
    }
    for d in &dump_files {
        let nd = paths::norm_name(d);
        if in_pictures.contains(&nd) || in_music.contains(&nd) || in_videos.contains(&nd) {
            wrong_items.push(CheckItem {
                name: d.clone(),
                kind: "txt".to_string(),
            });
            wrong_details.push(format!("{} → Corbeille", d));
        }
    }

    let wrong_ok = wrong_items.is_empty();
    let preview_details: Vec<String> = wrong_details.into_iter().take(4).collect();
    checks.push(CheckResult {
        id: "no_wrong_folder".to_string(),
        ok: wrong_ok,
        weight: 2.0,
        earned: if wrong_ok { 2.0 } else { 0.0 },
        label: "Aucun fichier rangé dans le mauvais dossier".to_string(),
        hint: if wrong_ok {
            String::new()
        } else {
            format!("💡 Mauvais dossier ({}) :", preview_details.join(", "))
        },
        items: wrong_items.into_iter().take(6).collect(),
    });

    // 5. Downloads clean (moved, not copied) (weight 3)
    let mut all_managed: Vec<String> = all_media_files().iter().map(|s| (*s).to_string()).collect();
    all_managed.extend(dump_files.clone());

    let left_in_downloads: Vec<String> = all_managed
        .iter()
        .filter(|f| in_downloads.contains(&paths::norm_name(f)))
        .cloned()
        .collect();
    let copied_not_moved = left_in_downloads.iter().any(|f| {
        let nf = paths::norm_name(f);
        in_pictures.contains(&nf) || in_music.contains(&nf) || in_videos.contains(&nf)
    });

    let dl_earned = 3.0 * ((all_managed.len() - left_in_downloads.len()) as f64)
        / (all_managed.len() as f64);
    let dl_hint = if copied_not_moved {
        "💡 Tu as copié au lieu de déplacer ! Enlève de Téléchargements :".to_string()
    } else if !left_in_downloads.is_empty() {
        "💡 Il reste encore ces fichiers dans Téléchargements :".to_string()
    } else {
        String::new()
    };

    checks.push(CheckResult {
        id: "downloads_clean".to_string(),
        ok: left_in_downloads.is_empty(),
        weight: 3.0,
        earned: dl_earned,
        label: "Téléchargements est vide (fichiers déplacés, pas copiés)".to_string(),
        hint: dl_hint,
        items: left_in_downloads
            .iter()
            .take(6)
            .map(|f| CheckItem {
                name: f.clone(),
                kind: kind_for_name(f).to_string(),
            })
            .collect(),
    });

    // 6. Dumps in the Recycle Bin (weight 2)
    let (rb_accessible, rb_entries) =
        recyclebin::read_recycle_bin_entries(&[downloads.as_path()]);
    let mut dumps_ok_count = 0usize;
    let mut still_there_dumps = Vec::new();
    let mut shift_deleted_dumps = Vec::new();

    for d in &dump_files {
        let nd = paths::norm_name(d);
        let anywhere = in_downloads.contains(&nd)
            || in_pictures.contains(&nd)
            || in_music.contains(&nd)
            || in_videos.contains(&nd);
        if anywhere {
            still_there_dumps.push(d.clone());
            continue;
        }
        if rb_accessible {
            if recyclebin::is_file_in_recycle_bin(d, &downloads, &rb_entries) {
                dumps_ok_count += 1;
            } else {
                shift_deleted_dumps.push(d.clone());
            }
        } else {
            dumps_ok_count += 1;
        }
    }

    let note = if !rb_accessible {
        Some(
            "Note : la Corbeille n'est pas lisible directement sur ce PC, suppression vérifiée."
                .to_string(),
        )
    } else {
        None
    };

    let rb_ok = dumps_ok_count == dump_files.len();
    let rb_hint = if !shift_deleted_dumps.is_empty() {
        "💡 Fichier supprimé définitivement : il n'est pas dans la Corbeille :".to_string()
    } else if !still_there_dumps.is_empty() {
        "💡 Mets ces fichiers inutiles dans la Corbeille :".to_string()
    } else {
        String::new()
    };

    let mut bad_dumps = shift_deleted_dumps;
    bad_dumps.extend(still_there_dumps);

    checks.push(CheckResult {
        id: "dumps_in_recycle_bin".to_string(),
        ok: rb_ok,
        weight: dump_files.len() as f64,
        earned: dumps_ok_count as f64,
        label: "Les 2 fichiers inutiles sont dans la Corbeille".to_string(),
        hint: rb_hint,
        items: bad_dumps
            .into_iter()
            .take(5)
            .map(|d| CheckItem {
                name: d,
                kind: "txt".to_string(),
            })
            .collect(),
    });

    Ok(compute_verify_response(checks, note))
}
