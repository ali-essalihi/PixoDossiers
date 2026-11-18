//! Weighted check scoring and 0..3 star calculation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub ok: bool,
    pub label: String,
    pub hint: String,
    pub items: Vec<CheckItem>,
    #[serde(skip_serializing)]
    pub weight: f64,
    #[serde(skip_serializing)]
    pub earned: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub checks: Vec<CheckResult>,
    pub score: u32,
    pub stars: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn compute_verify_response(checks: Vec<CheckResult>, note: Option<String>) -> VerifyResponse {
    if checks.is_empty() {
        return VerifyResponse {
            checks,
            score: 0,
            stars: 0,
            note,
            error: None,
        };
    }

    let mut total_weight = 0.0f64;
    let mut passed_weight = 0.0f64;
    let mut all_ok = true;

    for chk in &checks {
        let w = chk.weight.max(0.0);
        if !chk.ok {
            all_ok = false;
        }
        let earned = chk.earned.clamp(0.0, w);
        total_weight += w;
        passed_weight += earned;
    }

    if total_weight <= 0.0 {
        return VerifyResponse {
            checks,
            score: 0,
            stars: 0,
            note,
            error: None,
        };
    }

    let ratio = passed_weight / total_weight;
    let (score, stars) = if all_ok && ratio >= 0.9999 {
        (100, 3)
    } else {
        let sc = ((ratio * 100.0).round() as u32).min(99);
        let st = if ratio >= 0.60 - 1e-9 {
            2
        } else if ratio >= 0.30 - 1e-9 {
            1
        } else {
            0
        };
        (sc, st)
    };

    VerifyResponse {
        checks,
        score,
        stars,
        note,
        error: None,
    }
}
