use crate::model::{Finding, Rule};
use crate::scoring::{calculate_score, grade_from_score, ScoreParams};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use veil_config::Config;

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ScanLimit {
    max: Option<usize>,
    counter: AtomicUsize,
}

impl ScanLimit {
    pub fn new(max: Option<usize>) -> Self {
        Self {
            max,
            counter: AtomicUsize::new(0),
        }
    }

    pub fn check(&self) -> bool {
        if let Some(max) = self.max {
            self.counter.load(Ordering::Relaxed) >= max
        } else {
            false
        }
    }

    pub fn try_add(&self, n: usize) -> bool {
        if let Some(max) = self.max {
            let prev = self.counter.fetch_add(n, Ordering::Relaxed);
            prev < max // Return true if we were below limit before adding
        } else {
            true
        }
    }

    pub fn current(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

pub mod result;
pub mod utils;
use result::ScanResult;

pub const RULE_ID_BINARY_FILE: &str = "BINARY_FILE";
pub const RULE_ID_MAX_FILE_SIZE: &str = "MAX_FILE_SIZE";

pub fn scan_path(root: &Path, rules: &[Rule], config: &Config) -> ScanResult {
    let ignore_patterns = &config.core.ignore;
    let limit = ScanLimit::new(config.output.max_findings);

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

    let total_files = entries.len();
    let scanned_counter = AtomicUsize::new(0);
    let skipped_counter = AtomicUsize::new(0);

    // 2. Process files in parallel
    let findings: Vec<Finding> = entries
        .par_iter()
        .flat_map(|entry| {
            if limit.check() {
                return Vec::new();
            }

            let file_findings = scan_file(entry.path(), rules, config, Some(&limit));

            // Check if file was skipped due to binary/size
            let mut is_skipped = false;
            if let Some(first) = file_findings.first() {
                if first.rule_id == RULE_ID_BINARY_FILE || first.rule_id == RULE_ID_MAX_FILE_SIZE {
                    is_skipped = true;
                }
            }

            if is_skipped {
                skipped_counter.fetch_add(1, Ordering::Relaxed);
                Vec::new() // Do not return these as findings
            } else {
                scanned_counter.fetch_add(1, Ordering::Relaxed);
                file_findings
            }
        })
        .collect();

    ScanResult {
        findings,
        total_files,
        scanned_files: scanned_counter.load(Ordering::Relaxed),
        skipped_files: skipped_counter.load(Ordering::Relaxed),
        limit_reached: limit.check(),
    }
}

pub fn scan_file(
    path: &Path,
    rules: &[Rule],
    config: &Config,
    limit: Option<&ScanLimit>,
) -> Vec<Finding> {
    let mut local_findings = Vec::new();

    // Check file size
    if let Ok(metadata) = std::fs::metadata(path) {
        let max_size = config.core.max_file_size.unwrap_or(1_000_000);
        if metadata.len() > max_size {
            local_findings.push(crate::scanner::utils::create_skipped_finding(
                path,
                RULE_ID_MAX_FILE_SIZE,
                format!(
                    "File size ({} bytes) exceeds limit ({} bytes)",
                    metadata.len(),
                    max_size
                ),
                crate::model::Severity::High,
            ));
            return local_findings;
        }
    }

    let score_params = ScoreParams::default();

    if let Ok(mut file) = File::open(path) {
        // Binary Check
        let mut buffer = [0; 1024];
        let n = file.read(&mut buffer).unwrap_or(0);
        if buffer[..n].contains(&0) {
            local_findings.push(crate::scanner::utils::create_skipped_finding(
                path,
                RULE_ID_BINARY_FILE,
                "Binary file detected (skipped)".to_string(),
                crate::model::Severity::Medium,
            ));
            return local_findings;
        }

        // Reset cursor
        let _ = file.seek(SeekFrom::Start(0));

        let reader = BufReader::new(file);
        let mut context_buffer = VecDeque::with_capacity(5);

        for (line_idx, line) in reader.lines().enumerate() {
            // Early exit if limit reached globally
            if let Some(lim) = limit {
                if lim.check() {
                    break;
                }
            }

            if let Ok(content) = line {
                let line_findings = scan_line(
                    &content,
                    line_idx + 1,
                    path,
                    rules,
                    config,
                    &context_buffer,
                    &score_params,
                );

                if !line_findings.is_empty() {
                    local_findings.extend(line_findings);
                }

                // Context buffer maintenance
                if context_buffer.len() >= 5 {
                    context_buffer.pop_front();
                }
                context_buffer.push_back(content);
            }
        }
    }

    if local_findings.is_empty() {
        return local_findings;
    }

    // Update global limit counter and truncate if necessary
    if let Some(lim) = limit {
        if let Some(max) = lim.max {
            let count = local_findings.len();
            let prev = lim.counter.fetch_add(count, Ordering::Relaxed);

            if prev >= max {
                // Already over limit before we added
                return Vec::new();
            }

            let room = max.saturating_sub(prev);
            if count > room {
                local_findings.truncate(room);
            }
        } else {
            lim.counter
                .fetch_add(local_findings.len(), Ordering::Relaxed);
        }
    }

    local_findings
}

pub fn scan_content(content: &str, path: &Path, rules: &[Rule], config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut context_buffer = VecDeque::with_capacity(5);
    let score_params = ScoreParams::default();

    for (line_idx, line) in content.lines().enumerate() {
        let line_findings = scan_line(
            line,
            line_idx + 1,
            path,
            rules,
            config,
            &context_buffer,
            &score_params,
        );
        findings.extend(line_findings);

        if context_buffer.len() >= 5 {
            context_buffer.pop_front();
        }
        context_buffer.push_back(line.to_string());
    }
    findings
}

fn scan_line(
    content: &str,
    line_number: usize,
    path: &Path,
    rules: &[Rule],
    config: &Config,
    context_buffer: &VecDeque<String>,
    score_params: &ScoreParams,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    // 1. Collect all matches
    let mut all_matches = Vec::new();

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
        // Support "# veil:ignore" (Python/Shell/YAML) and "// veil:ignore" (Rust/JS/C family)
        if content.contains("veil:ignore") {
            // Check specific ignore first: "veil:ignore=rule.id"
            // Both "# veil:ignore=X" and "// veil:ignore=X"
            if content.contains(&format!("veil:ignore={}", rule.id)) {
                continue;
            }
            // Generic ignore: "veil:ignore" NOT immediately followed by "="
            // e.g. "// veil:ignore" or "# veil:ignore"
            // We need to be careful not to match "veil:ignore_something_else" if that existed,
            // but for now "veil:ignore" is the keyword.
            // Simplest check: if it contains "veil:ignore" and doesn't contain "veil:ignore=" it might be a generic ignore.
            // But what if it contains "veil:ignore=OTHER_ID"?
            // If the line has "veil:ignore=OTHER", we should NOT skip THIS rule unless it also has "veil:ignore" generic or "veil:ignore=THIS".
            // Implementation Decision for v0.7.4:
            // "If line contains `veil:ignore` and does NOT contain `=`, it is a generic ignore for ALL rules."
            // This is a heuristic. Better: check if "veil:ignore" exists as a token?
            // User requested: "Simple line ignore".
            // Let's stick to the existing logic extended for `//`.

            let is_comment_ignore =
                content.contains("# veil:ignore") || content.contains("// veil:ignore");
            if is_comment_ignore && !content.contains("veil:ignore=") {
                continue;
            }
        }

        // Find ALL matches for this rule on the line
        for mat in rule.pattern.find_iter(content) {
            let matched_str = mat.as_str();
            if let Some(validator) = rule.validator {
                if !validator(matched_str) {
                    continue;
                }
            }
            all_matches.push((rule, mat));
        }
    }

    if all_matches.is_empty() {
        return findings;
    }

    // 2. Generate Masked Snippet (Safe Output: Mask ALL secrets on the line)
    // Collect all ranges from all matches
    let ranges: Vec<_> = all_matches.iter().map(|(_, m)| m.range()).collect();
    let placeholder = config.masking.placeholder.as_str();
    let masked_snippet = crate::masking::apply_masks(
        content,
        ranges,
        config.output.mask_mode.unwrap_or_default(),
        placeholder,
    );

    // 3. Create Findings
    for (rule, mat) in all_matches {
        let matched_content = mat.as_str().to_string();

        // Context Capture (Lookback only)
        let needed = rule.context_lines_before as usize;
        let available = context_buffer.len();
        let take = std::cmp::min(needed, available);
        let start = available - take;
        let context_before: Vec<String> = context_buffer.iter().skip(start).cloned().collect();
        let context_after = Vec::new();

        let mut finding = Finding {
            path: path.to_path_buf(),
            line_number,
            line_content: content.to_string(),
            rule_id: rule.id.clone(),
            matched_content,
            masked_snippet: masked_snippet.clone(), // Share the safe snippet
            severity: rule.severity.clone(),
            score: 0,
            grade: crate::rules::grade::Grade::Safe,
            context_before,
            context_after,
            commit_sha: None,
            author: None,
            date: None,
        };

        let final_score = calculate_score(rule, &finding, score_params);
        let grade = grade_from_score(final_score);

        finding.score = final_score;
        finding.grade = grade;

        findings.push(finding);
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Severity;
    use regex::Regex;

    #[test]
    fn test_context_capture() {
        let rule = Rule {
            id: "test".to_string(),
            pattern: Regex::new("SECRET").unwrap(),
            description: "test".to_string(),
            severity: Severity::High,
            score: 50,
            category: "test".to_string(),
            tags: vec![],
            base_score: None,
            context_lines_before: 2,
            context_lines_after: 0,
            validator: None,
            placeholder: None,
        };
        let rules = vec![rule];
        let config = Config::default();

        // line1
        // line2
        // line3
        // SECRET
        let content = "line1\nline2\nline3\nSECRET\nline5";
        let findings = scan_content(content, Path::new("test"), &rules, &config);

        assert_eq!(findings.len(), 1);
        let f = &findings[0];
        assert_eq!(f.line_number, 4);
        assert_eq!(f.context_before.len(), 2, "Should capture 2 context lines");
        assert_eq!(f.context_before[0], "line2");
        assert_eq!(f.context_before[1], "line3");
    }

    #[test]
    fn test_context_capture_start_of_file() {
        let rule = Rule {
            id: "test".to_string(),
            pattern: Regex::new("SECRET").unwrap(),
            description: "test".to_string(),
            severity: Severity::High,
            score: 50,
            category: "test".to_string(),
            tags: vec![],
            base_score: None,
            context_lines_before: 2,
            context_lines_after: 0,
            validator: None,
            placeholder: None,
        };
        let rules = vec![rule];
        let config = Config::default();

        let content = "SECRET\nline2";
        let findings = scan_content(content, Path::new("test"), &rules, &config);

        assert_eq!(findings.len(), 1);
        let f = &findings[0];
        assert_eq!(f.line_number, 1);
        assert_eq!(f.context_before.len(), 0);
    }

    #[test]
    fn test_inline_ignore() {
        let rule = Rule {
            id: "test".to_string(),
            pattern: Regex::new("SECRET").unwrap(),
            description: "test".to_string(),
            severity: Severity::High,
            score: 50,
            category: "test".to_string(),
            tags: vec![],
            base_score: None,
            context_lines_before: 0,
            context_lines_after: 0,
            validator: None,
            placeholder: None,
        };
        let rules = vec![rule];
        let config = Config::default();

        // 1. Python style #
        let content = "SECRET # veil:ignore";
        let findings = scan_content(content, Path::new("test.py"), &rules, &config);
        assert!(findings.is_empty(), "Should ignore with #");

        // 2. Rust/JS style //
        let content = "SECRET // veil:ignore";
        let findings = scan_content(content, Path::new("test.rs"), &rules, &config);
        assert!(findings.is_empty(), "Should ignore with //");

        // 3. No ignore
        let content = "SECRET";
        let findings = scan_content(content, Path::new("test.rs"), &rules, &config);
        assert_eq!(findings.len(), 1, "Should find secret without ignore");

        // 4. Specific ignore (match)
        let content = "SECRET // veil:ignore=test";
        let findings = scan_content(content, Path::new("test.rs"), &rules, &config);
        assert!(
            findings.is_empty(),
            "Should ignore with matches specific ID"
        );

        // 5. Specific ignore (mismatch)
        let content = "SECRET // veil:ignore=other";
        let findings = scan_content(content, Path::new("test.rs"), &rules, &config);
        assert_eq!(findings.len(), 1, "Should NOT ignore if ID mismatch");
    }
}
