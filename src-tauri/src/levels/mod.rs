//! Registry of levels and payload manifest verification.

pub mod level1;
pub mod level2;
pub mod level3;
pub mod level4;
pub mod level5;

use crate::paths;
use crate::scoring::VerifyResponse;

/// Return expected payload filenames for a given `level_id` (1..=5).
pub fn expected_payload_for_level(level_id: u32) -> &'static [&'static str] {
    match level_id {
        1 => &[
            "clavier.png",
            "souris.png",
            "ecran.png",
            "unite-centrale.png",
            "imprimante-demo.mp4",
            "scanner-demo.mp4",
            "pluie.mp3",
            "tonnerre.mp3",
        ],
        2 => &[
            "orange.jpg",
            "citron.jpg",
            "fraise.jpg",
            "cerise.jpg",
            "pomme.jpg",
            "banane.jpg",
            "brocoli.jpg",
            "salade.jpg",
            "carotte.jpg",
            "pomme_de_terre.jpg",
            "cafe.jpg",
            "the.jpg",
            "jus_de_pomme.jpg",
            "eau.jpg",
        ],
        3 => &[
            "unite-centrale.png",
            "clavier.png",
            "souris.png",
            "ecran.png",
        ],
        _ => &[],
    }
}

/// Return all unique expected payload filenames across all levels.
pub fn get_all_expected_payload_files() -> Vec<String> {
    let mut seen = Vec::new();
    for lv in 1..=5 {
        for &fname in expected_payload_for_level(lv) {
            let s = fname.to_string();
            if !seen.contains(&s) {
                seen.push(s);
            }
        }
    }
    seen
}

/// Return missing payload files for a specific level (or all levels if `level_id == None`).
pub fn check_missing_payload(level_id: Option<u32>) -> Vec<String> {
    let payload_dir = paths::get_payload_dir();
    let expected: Vec<String> = match level_id {
        Some(lv) => expected_payload_for_level(lv)
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        None => get_all_expected_payload_files(),
    };

    let mut missing = Vec::new();
    for fname in expected {
        if !payload_dir.join(&fname).is_file() {
            missing.push(fname);
        }
    }
    missing
}

/// Dispatch `prepare()` for `level_id`.
pub fn prepare_level(
    level_id: u32,
    member_name: Option<&str>,
    group_name: Option<&str>,
) -> Result<(), String> {
    match level_id {
        1 => level1::prepare(member_name, group_name),
        2 => level2::prepare(member_name, group_name),
        3 => level3::prepare(member_name, group_name),
        4 => level4::prepare(member_name, group_name),
        5 => level5::prepare(member_name, group_name),
        _ => Err(format!("Niveau introuvable : {}", level_id)),
    }
}

/// Dispatch `verify()` for `level_id`.
pub fn verify_level(
    level_id: u32,
    member_name: Option<&str>,
    group_name: Option<&str>,
) -> Result<VerifyResponse, String> {
    match level_id {
        1 => level1::verify(member_name, group_name),
        2 => level2::verify(member_name, group_name),
        3 => level3::verify(member_name, group_name),
        4 => level4::verify(member_name, group_name),
        5 => level5::verify(member_name, group_name),
        _ => Err(format!("Niveau introuvable : {}", level_id)),
    }
}
