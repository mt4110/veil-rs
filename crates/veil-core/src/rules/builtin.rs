use crate::model::{Rule, Severity};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;
use veil_config::Config;

static DEFAULT_RULES: OnceLock<Vec<Rule>> = OnceLock::new();

pub fn get_default_rules() -> Vec<Rule> {
    DEFAULT_RULES
        .get_or_init(|| {
            vec![
                // ==========================
                // Credential Pack v1 (RED)
                // ==========================
                Rule {
                    id: "creds.aws.access_key_id".to_string(),
                    pattern: Regex::new(r"\b(AKIA|ASIA|AGPA)[0-9A-Z]{16}\b").expect("Valid Regex"),
                    description: "AWS Access Key ID".to_string(),
                    severity: Severity::High,
                    score: 85,
                    category: "secret".to_string(),
                    tags: vec![
                        "credential".to_string(),
                        "cloud".to_string(),
                        "aws".to_string(),
                        "critical".to_string(),
                    ],
                    base_score: Some(85),
                    context_lines_before: 1,
                    context_lines_after: 1,
                    validator: None,
                },
                Rule {
                    id: "creds.aws.secret_key_config".to_string(),
                    pattern: Regex::new(
                        r#"aws_secret_access_key\s*=\s*["']?([0-9A-Za-z/+]{40})["']?"#,
                    )
                    .expect("Valid Regex"),
                    description: "AWS Secret Access Key (Config style)".to_string(),
                    severity: Severity::High,
                    score: 90,
                    category: "secret".to_string(),
                    tags: vec![
                        "credential".to_string(),
                        "cloud".to_string(),
                        "aws".to_string(),
                        "critical".to_string(),
                    ],
                    base_score: Some(90),
                    context_lines_before: 1,
                    context_lines_after: 1,
                    validator: None,
                },
                Rule {
                    id: "creds.github.pat.ghp".to_string(),
                    pattern: Regex::new(r"\bghp_[0-9A-Za-z]{36}\b").expect("Valid Regex"),
                    description: "GitHub Personal Access Token (ghp_)".to_string(),
                    severity: Severity::High,
                    score: 90,
                    category: "secret".to_string(),
                    tags: vec![
                        "credential".to_string(),
                        "github".to_string(),
                        "critical".to_string(),
                    ],
                    base_score: Some(90),
                    context_lines_before: 1,
                    context_lines_after: 1,
                    validator: None,
                },
                Rule {
                    id: "creds.github.pat.long".to_string(),
                    pattern: Regex::new(r"\bgithub_pat_[0-9A-Za-z_]{80,100}\b")
                        .expect("Valid Regex"),
                    description: "GitHub Personal Access Token (Fine-grained)".to_string(),
                    severity: Severity::High,
                    score: 90,
                    category: "secret".to_string(),
                    tags: vec![
                        "credential".to_string(),
                        "github".to_string(),
                        "critical".to_string(),
                    ],
                    base_score: Some(90),
                    context_lines_before: 1,
                    context_lines_after: 1,
                    validator: None,
                },
                Rule {
                    id: "creds.slack.token.legacy".to_string(),
                    pattern: Regex::new(r"\bxox[baps]-\d{10,}-\d{10,}-[0-9A-Za-z]{24,}\b")
                        .expect("Valid Regex"),
                    description: "Slack Token (Legacy)".to_string(),
                    severity: Severity::High,
                    score: 85,
                    category: "secret".to_string(),
                    tags: vec![
                        "credential".to_string(),
                        "slack".to_string(),
                        "critical".to_string(),
                    ],
                    base_score: Some(85),
                    context_lines_before: 1,
                    context_lines_after: 1,
                    validator: None,
                },
                Rule {
                    id: "creds.key.private_pem".to_string(),
                    pattern: Regex::new(r"-----BEGIN (RSA |EC )?PRIVATE KEY-----")
                        .expect("Valid Regex"),
                    description: "Private Key (PEM)".to_string(),
                    severity: Severity::Critical,
                    score: 95,
                    category: "secret".to_string(),
                    tags: vec![
                        "credential".to_string(),
                        "key".to_string(),
                        "pem".to_string(),
                        "critical".to_string(),
                    ],
                    base_score: Some(95),
                    context_lines_before: 2,
                    context_lines_after: 2,
                    validator: None,
                },
                // ==========================
                // JP PII v1 (RED)
                // ==========================
                Rule {
                    id: "jp.mynumber".to_string(),
                    pattern: Regex::new(r"(\b\d{12}\b|\b\d{4}-\d{4}-\d{4}\b)")
                        .expect("Valid Regex"),
                    description: "Japanese My Number".to_string(),
                    severity: Severity::High,
                    score: 90,
                    category: "pii".to_string(),
                    tags: vec![
                        "pii".to_string(),
                        "jp".to_string(),
                        "id".to_string(),
                        "high_risk".to_string(),
                    ],
                    base_score: Some(90),
                    context_lines_before: 2,
                    context_lines_after: 0,
                    validator: None,
                },
                Rule {
                    id: "jp.phone.mobile".to_string(),
                    pattern: Regex::new(r"\b0[789]0[- ]?\d{4}[- ]?\d{4}\b").expect("Valid Regex"),
                    description: "Japanese Mobile Phone".to_string(),
                    severity: Severity::Medium,
                    score: 60,
                    category: "pii".to_string(),
                    tags: vec![
                        "pii".to_string(),
                        "jp".to_string(),
                        "phone".to_string(),
                        "low_risk".to_string(),
                    ],
                    base_score: Some(60),
                    context_lines_before: 1,
                    context_lines_after: 0,
                    validator: None,
                },
            ]
        })
        .clone()
}

pub fn get_all_rules(config: &Config, extra_rules: Vec<Rule>) -> Vec<Rule> {
    // defaults is Vec<Rule> (cloned from static)
    let defaults = get_default_rules();
    let mut rule_map: HashMap<String, Rule> =
        defaults.into_iter().map(|r| (r.id.clone(), r)).collect();

    // Merge extra/remote rules
    // If ID exists (matches builtin), we overwrite it (Remote updates/fixes builtin)
    // If ID is new, we insert it.
    for rule in extra_rules {
        rule_map.insert(rule.id.clone(), rule);
    }

    for (id, rule_conf) in &config.rules {
        if let Some(pattern_str) = &rule_conf.pattern {
            // New rule or Overwrite pattern (Pure TOML Rule)
            if let Ok(regex) = Regex::new(pattern_str) {
                let rule = Rule {
                    id: id.clone(),
                    pattern: regex,
                    description: rule_conf
                        .description
                        .clone()
                        .unwrap_or_else(|| "Custom rule".to_string()),
                    severity: rule_conf.severity.as_deref().unwrap_or("medium").into(),
                    score: rule_conf.score.unwrap_or(50) as u32,
                    category: rule_conf.category.clone().unwrap_or("custom".to_string()),
                    tags: rule_conf.tags.clone().unwrap_or_default(),
                    base_score: rule_conf.base_score,
                    context_lines_before: rule_conf.context_lines_before.unwrap_or(2),
                    context_lines_after: rule_conf.context_lines_after.unwrap_or(0),
                    validator: None,
                };
                rule_map.insert(id.clone(), rule);
            } else {
                eprintln!("Invalid Regex pattern for rule {}: {}", id, pattern_str);
            }
        } else {
            // Override existing rule (Builtin OR Remote)
            if let Some(rule) = rule_map.get_mut(id) {
                if let Some(sev) = &rule_conf.severity {
                    rule.severity = sev.as_str().into();
                }
                if let Some(desc) = &rule_conf.description {
                    rule.description = desc.clone();
                }
                if let Some(score) = rule_conf.score {
                    rule.score = score as u32;
                }
                if let Some(cat) = &rule_conf.category {
                    rule.category = cat.clone();
                }
                if let Some(tags) = &rule_conf.tags {
                    rule.tags = tags.clone();
                }
            }
        }
    }

    rule_map
        .into_values()
        .filter(|r| config.rules.get(&r.id).map(|rc| rc.enabled).unwrap_or(true))
        .collect()
}
