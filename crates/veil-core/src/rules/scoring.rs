use crate::model::Severity;

// Base scores
const BASE_SCORE_LOW: u32 = 10;
const BASE_SCORE_MEDIUM: u32 = 40;
const BASE_SCORE_HIGH: u32 = 70;
const BASE_SCORE_CRITICAL: u32 = 90;

pub fn calculate_base_score(severity: &Severity) -> u32 {
    match severity {
        Severity::Low => BASE_SCORE_LOW,
        Severity::Medium => BASE_SCORE_MEDIUM,
        Severity::High => BASE_SCORE_HIGH,
        Severity::Critical => BASE_SCORE_CRITICAL,
    }
}

// Simple context scoring (bonus if keywords found in line)
pub fn calculate_context_score(line_content: &str) -> u32 {
    let lower = line_content.to_lowercase();
    let keywords = [
        "password",
        "secret",
        "token",
        "key",
        "auth",
        "credential",
        "private",
        "passwd",
        "pwd",
        "api_key",
        "apikey",
    ];

    let mut score = 0;
    for keyword in keywords {
        if lower.contains(keyword) {
            score += 10;
        }
    }

    // Cap bonus
    if score > 30 {
        score = 30;
    }

    score
}
