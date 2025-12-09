use crate::model::{Finding, Rule, Severity};
use crate::rules::grade::Grade;
use std::collections::HashMap;

pub struct ScoreParams {
    pub test_words: Vec<String>,
    pub prod_words: Vec<String>,
    pub tag_weights: HashMap<String, i32>,
}

impl Default for ScoreParams {
    fn default() -> Self {
        let mut tag_weights = HashMap::new();
        tag_weights.insert("critical".to_string(), 20);
        tag_weights.insert("pii".to_string(), 10);
        tag_weights.insert("low_risk".to_string(), -10);

        Self {
            test_words: vec![
                "test".to_string(),
                "example".to_string(),
                "sample".to_string(),
                "dummy".to_string(),
                "mock".to_string(),
            ],
            prod_words: vec![
                "prod".to_string(),
                "production".to_string(),
                "secret".to_string(),
                "credential".to_string(),
                "key".to_string(),
                "password".to_string(),
            ],
            tag_weights,
        }
    }
}

pub fn calculate_score(rule: &Rule, finding: &Finding, params: &ScoreParams) -> u32 {
    let mut score = rule
        .base_score
        .unwrap_or_else(|| severity_default(&rule.severity)) as i32;

    // Context adjustments
    let context_blob = format!(
        "{}\n{}\n{}",
        finding.line_content,
        finding.context_before.join("\n"),
        finding.context_after.join("\n")
    )
    .to_lowercase();

    if params.test_words.iter().any(|w| context_blob.contains(w)) {
        score -= 10;
    }
    if params.prod_words.iter().any(|w| context_blob.contains(w)) {
        score += 10;
    }

    // Tag adjustments
    for tag in &rule.tags {
        if let Some(weight) = params.tag_weights.get(tag) {
            score += *weight;
        }
    }

    score.clamp(0, 100) as u32
}

pub fn grade_from_score(score: u32) -> Grade {
    match score {
        0..=39 => Grade::Low,
        40..=69 => Grade::Medium,
        70..=89 => Grade::High,
        _ => Grade::Critical,
    }
}

fn severity_default(sev: &Severity) -> u32 {
    match sev {
        Severity::Low => 30,
        Severity::Medium => 60,
        Severity::High => 80,
        Severity::Critical => 90,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Severity;
    use crate::rules::grade::Grade;

    #[test]
    fn test_base_score_default() {
        let rule = Rule {
            severity: Severity::High,
            base_score: None,
            // ... other fields via struct literal or helper helper
            id: "test".to_string(),
            pattern: regex::Regex::new(".").unwrap(),
            description: "".to_string(),
            score: 0,
            category: "".to_string(),
            tags: vec![],
            context_lines_before: 0,
            context_lines_after: 0,
            validator: None,
        };
        let finding = Finding {
            path: std::path::PathBuf::from("."),
            line_number: 1,
            line_content: "".to_string(),
            rule_id: "test".to_string(),
            matched_content: "".to_string(),
            masked_snippet: "".to_string(),
            severity: Severity::High,
            score: 0,
            grade: Grade::Low,
            context_before: vec![],
            context_after: vec![],
        };

        let score = calculate_score(&rule, &finding, &ScoreParams::default());
        assert_eq!(score, 80); // High default
    }

    #[test]
    fn test_context_modifiers() {
        let rule = Rule {
            severity: Severity::Medium, // 60
            base_score: None,
            id: "test".to_string(),
            pattern: regex::Regex::new(".").unwrap(),
            description: "".to_string(),
            score: 0,
            category: "".to_string(),
            tags: vec![],
            context_lines_before: 0,
            context_lines_after: 0,
            validator: None,
        };
        let mut finding = Finding {
            path: std::path::PathBuf::from("."),
            line_number: 1,
            line_content: "some_var".to_string(),
            rule_id: "test".to_string(),
            matched_content: "".to_string(),
            masked_snippet: "".to_string(),
            severity: Severity::Medium,
            score: 0,
            grade: Grade::Low,
            context_before: vec![],
            context_after: vec![],
        };

        let params = ScoreParams::default();

        // No context
        assert_eq!(calculate_score(&rule, &finding, &params), 60);

        // Test context
        finding.context_before = vec!["// this is a test case".to_string()];
        assert_eq!(calculate_score(&rule, &finding, &params), 50); // 60 - 10

        // Prod context
        finding.context_before = vec!["// production config".to_string()];
        assert_eq!(calculate_score(&rule, &finding, &params), 70); // 60 + 10
    }
}
