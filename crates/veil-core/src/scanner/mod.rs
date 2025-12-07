use crate::masking::mask_string;
use crate::model::{Finding, Rule};
use crate::rules::grade::calculate_grade;
use crate::rules::scoring::{calculate_base_score, calculate_context_score};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use veil_config::Config;

pub fn scan_path(root: &Path, rules: &[Rule], config: &Config) -> Vec<Finding> {
    let ignore_patterns = &config.core.ignore;

    // 1. Collect all valid paths first (sequential walk, usually fast enough)
    // Use ignore::WalkBuilder to respect .gitignore
    let entries: Vec<_> = WalkBuilder::new(root)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .filter(|e| {
            let path = e.path();
            let path_str = path.to_string_lossy();
            // User configured ignores (veil.toml)
            for pattern in ignore_patterns {
                if path_str.contains(pattern) {
                    return false;
                }
            }
            true
        })
        .collect();

    // 2. Process files in parallel
    entries
        .par_iter()
        .flat_map(|entry| {
            let path = entry.path();
            let mut local_findings = Vec::new();

            // Check file size using metadata associated with the entry if possible, or path
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > config.core.max_file_size {
                    local_findings.push(Finding {
                        path: path.to_path_buf(),
                        line_number: 0,
                        line_content: format!(
                            "File size ({} bytes) exceeds limit ({} bytes)",
                            metadata.len(),
                            config.core.max_file_size
                        ),
                        rule_id: "MAX_FILE_SIZE".to_string(),
                        masked_line: "".to_string(),
                        severity: crate::model::Severity::High, // Treat as High/Critical
                        score: 100,                             // Force high score
                        grade: crate::rules::grade::Grade::Critical,
                    });
                    return local_findings;
                }
            }

            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for (line_idx, line) in reader.lines().enumerate() {
                    if let Ok(content) = line {
                        let line_findings = scan_line(&content, line_idx + 1, path, rules, config);
                        local_findings.extend(line_findings);
                    }
                }
            }
            local_findings
        })
        .collect()
}

pub fn scan_content(content: &str, path: &Path, rules: &[Rule], config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        let line_findings = scan_line(line, line_idx + 1, path, rules, config);
        findings.extend(line_findings);
    }
    findings
}

fn scan_line(
    content: &str,
    line_number: usize,
    path: &Path,
    rules: &[Rule],
    config: &Config,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for rule in rules {
        let rule_enabled = config
            .rules
            .get(&rule.id)
            .map(|r| r.enabled)
            .unwrap_or(true);
        if !rule_enabled {
            continue;
        }

        // Inline Ignore Logic
        // Simple check: if line contains "# veil:ignore", ignore all
        // Improved: check "# veil:ignore=<rule_id>" or "# veil:ignore"
        if content.contains("# veil:ignore") {
            if content.contains(&format!("# veil:ignore={}", rule.id)) || !content.contains("=") {
                continue;
            }
        }

        if let Some(mat) = rule.pattern.find(content) {
            let matched_str = mat.as_str();
            if let Some(validator) = rule.validator {
                if !validator(matched_str) {
                    continue;
                }
            }

            let masked_line = mask_string(content, mat.range());
            // Use rule.score as base, falling back to severity-based calculation if 0 (though we initialized all to >0)
            let base_score = if rule.score > 0 {
                rule.score
            } else {
                calculate_base_score(&rule.severity)
            };
            let context_score = calculate_context_score(content);
            let final_score = base_score + context_score;
            let grade = calculate_grade(final_score);

            findings.push(Finding {
                path: path.to_path_buf(),
                line_number,
                line_content: content.to_string(),
                rule_id: rule.id.clone(),
                masked_line,
                severity: rule.severity.clone(),
                score: final_score,
                grade,
            });
        }
    }
    findings
}
