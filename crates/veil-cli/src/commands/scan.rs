use crate::formatters::{Formatter, Summary};
use crate::output::formatter::print_finding;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{Delta, DiffOptions, Repository};
use ignore::WalkBuilder;
// use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use veil_config::{load_config, Config};
use veil_core::{get_all_rules, scan_content, scan_file};

#[allow(clippy::too_many_arguments)]
pub fn scan(
    paths: &[PathBuf],
    config_path: Option<&PathBuf>,
    format: &str,
    fail_score: Option<u32>,
    commit: Option<&str>,
    since: Option<&str>,
    staged: bool,
    _progress: bool,
    mask_mode_arg: Option<&str>,
    unsafe_output: bool,
    limit: Option<usize>,
) -> Result<bool> {
    let start_time = Instant::now();

    // Stats counters
    let scanned_files_atomic = AtomicUsize::new(0);
    let skipped_files_atomic = AtomicUsize::new(0);

    // Load configs
    let mut final_config = Config::default();

    // 1. Org Config (VEIL_ORG_RULES)
    if let Ok(org_path) = std::env::var("VEIL_ORG_RULES") {
        let path = PathBuf::from(&org_path);
        if path.exists() {
            match load_config(&path) {
                Ok(org_config) => final_config.merge(org_config),
                Err(e) => eprintln!("Warning: Failed to load Org config at {:?}: {}", path, e),
            }
        } else {
            eprintln!(
                "Warning: VEIL_ORG_RULES set to {:?} but file not found.",
                path
            );
        }
    }

    // 2. Project Config
    let config_file = config_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from("veil.toml"));

    // If explicit config path given, fail if missing. If default, fallback to default.
    let project_config = match load_config(&config_file) {
        Ok(c) => c,
        Err(e) => {
            if config_path.is_some() && !config_file.exists() {
                anyhow::bail!("Config file not found: {:?}", config_file);
            }
            if config_file.exists() {
                return Err(e);
            }
            Config::default()
        }
    };

    final_config.merge(project_config);

    // Apply CLI overrides for mask_mode
    if unsafe_output {
        final_config.output.mask_mode = Some(veil_config::MaskMode::Plain);
    } else if let Some(mode) = mask_mode_arg {
        match mode.to_lowercase().as_str() {
            "plain" => final_config.output.mask_mode = Some(veil_config::MaskMode::Plain),
            "partial" => final_config.output.mask_mode = Some(veil_config::MaskMode::Partial),
            "redact" => final_config.output.mask_mode = Some(veil_config::MaskMode::Redact),
            _ => eprintln!("Warning: Unknown mask mode '{}', using default.", mode),
        }
    }

    let config = final_config;

    // Remote Rules
    let mut remote_rules = Vec::new();
    let remote_url = std::env::var("VEIL_REMOTE_RULES_URL")
        .ok()
        .or_else(|| config.core.remote_rules_url.clone());

    if let Some(url) = remote_url {
        // Timeout 3s for CLI responsiveness
        match veil_core::remote::fetch_remote_rules(&url, 3) {
            Ok(rules) => {
                remote_rules = rules;
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch remote rules from {}: {}. Continuing with local rules only.", url, e);
            }
        }
    }

    let rules = get_all_rules(&config, remote_rules);
    let mut all_findings = Vec::new();

    // Limit Logic (Global)
    use veil_core::scanner::ScanLimit;
    // Explicit limit arg > Config > Default
    let limit_val = if let Some(l) = limit {
        if l == 0 {
            None
        } else {
            Some(l)
        }
    } else {
        config.output.max_findings
    };
    let limit_tracker = ScanLimit::new(limit_val);

    // Strategy Selection
    if let Some(commit_sha) = commit {
        // 1. Scan specific commit
        let repo = Repository::open(".")?;
        let obj = repo.revparse_single(commit_sha)?;
        let commit = obj.as_commit().context("Not a commit")?;
        let tree = commit.tree()?;

        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            let mut diff_opts = DiffOptions::new();
            let diff =
                repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut diff_opts))?;

            for delta in diff.deltas() {
                if delta.status() == Delta::Added || delta.status() == Delta::Modified {
                    if let Some(path) = delta.new_file().path() {
                        let path_val = path;
                        if let Ok(entry) = tree.get_path(path_val) {
                            if let Ok(object) = entry.to_object(&repo) {
                                if let Some(blob) = object.as_blob() {
                                    scanned_files_atomic.fetch_add(1, Ordering::Relaxed);

                                    let max_size = config.core.max_file_size.unwrap_or(1_000_000);
                                    if blob.size() as u64 > max_size {
                                        all_findings.push(veil_core::model::Finding {
                                            path: path_val.to_path_buf(),
                                            line_number: 0,
                                            line_content: format!(
                                                "File size ({} bytes) exceeds limit",
                                                blob.size()
                                            ),
                                            rule_id: veil_core::RULE_ID_MAX_FILE_SIZE.to_string(),
                                            matched_content: "".to_string(),
                                            masked_snippet: "".to_string(),
                                            severity: veil_core::model::Severity::High,
                                            score: 100,
                                            grade: veil_core::rules::grade::Grade::Critical,
                                            context_before: vec![],
                                            context_after: vec![],
                                        });
                                        skipped_files_atomic.fetch_add(1, Ordering::Relaxed);
                                        continue;
                                    }

                                    if let Ok(content) = std::str::from_utf8(blob.content()) {
                                        let findings =
                                            scan_content(content, path_val, &rules, &config);
                                        all_findings.extend(findings);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!("Scanning root commit not fully optimized yet.");
        }
    } else if let Some(since_str) = since {
        // 2. Scan history since time
        let since_dt = DateTime::parse_from_rfc3339(since_str)
            .map(|dt| dt.with_timezone(&Local))
            .or_else(|_| {
                let s = format!("{}T00:00:00Z", since_str);
                DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Local))
            })
            .context(
                "Failed to parse time. Use RFC3339 (e.g. 2024-01-01T00:00:00Z) or YYYY-MM-DD",
            )?;

        let repo = Repository::open(".")?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let commit_time = TimeZone::timestamp_opt(&Local, commit.time().seconds(), 0).unwrap();

            if commit_time < since_dt {
                break;
            }

            if commit.parent_count() > 0 {
                let parent = commit.parent(0)?;
                let parent_tree = parent.tree()?;
                let tree = commit.tree()?;
                let mut diff_opts = DiffOptions::new();
                let diff =
                    repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut diff_opts))?;

                for delta in diff.deltas() {
                    if delta.status() == Delta::Added || delta.status() == Delta::Modified {
                        if let Some(path) = delta.new_file().path() {
                            scanned_files_atomic.fetch_add(1, Ordering::Relaxed);
                            if let Ok(entry) = tree.get_path(path) {
                                if let Ok(object) = entry.to_object(&repo) {
                                    if let Some(blob) = object.as_blob() {
                                        let max_size =
                                            config.core.max_file_size.unwrap_or(1_000_000);
                                        if blob.size() as u64 > max_size {
                                            skipped_files_atomic.fetch_add(1, Ordering::Relaxed);
                                            continue;
                                        }
                                        if let Ok(content) = std::str::from_utf8(blob.content()) {
                                            let findings =
                                                scan_content(content, path, &rules, &config);
                                            all_findings.extend(findings);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if staged {
        // 3. Scan staged
        let repo = Repository::open(".")?;
        let diff = if let Ok(head) = repo.head() {
            if let Ok(head_tree) = head.peel_to_tree() {
                let index = repo.index()?;
                let mut diff_opts = DiffOptions::new();
                repo.diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut diff_opts))?
            } else {
                return Ok(false);
            }
        } else {
            let index = repo.index()?;
            let mut diff_opts = DiffOptions::new();
            repo.diff_tree_to_index(None, Some(&index), Some(&mut diff_opts))?
        };

        let index = repo.index()?;
        for delta in diff.deltas() {
            if delta.status() == Delta::Added || delta.status() == Delta::Modified {
                if let Some(path) = delta.new_file().path() {
                    let path_val = path;
                    if let Ok(true) = repo.is_path_ignored(path_val) {
                        continue;
                    }
                    if let Some(entry) = index.get_path(path_val, 0) {
                        if let Ok(blob) = repo.find_blob(entry.id) {
                            scanned_files_atomic.fetch_add(1, Ordering::Relaxed);
                            let max_size = config.core.max_file_size.unwrap_or(1_000_000);
                            if blob.size() as u64 > max_size {
                                all_findings.push(veil_core::model::Finding {
                                    path: path_val.to_path_buf(),
                                    line_number: 0,
                                    line_content: format!(
                                        "File size ({} bytes) exceeds limit",
                                        blob.size()
                                    ),
                                    rule_id: veil_core::RULE_ID_MAX_FILE_SIZE.to_string(),
                                    matched_content: "".to_string(),
                                    masked_snippet: "".to_string(),
                                    severity: veil_core::model::Severity::High,
                                    score: 100,
                                    grade: veil_core::rules::grade::Grade::Critical,
                                    context_before: vec![],
                                    context_after: vec![],
                                });
                                continue;
                            }

                            if let Ok(content) = std::str::from_utf8(blob.content()) {
                                let findings = scan_content(content, path_val, &rules, &config);
                                all_findings.extend(findings);
                            }
                        }
                    }
                }
            }
        }
    } else {
        // 4. Default FS scan (Updated for v0.4 with Progress)
        let targets = paths.iter().collect::<Vec<_>>();
        if !targets.is_empty() {
            let mut builder = WalkBuilder::new(targets[0].clone());
            for path in &targets[1..] {
                builder.add(path);
            }

            let walker = builder.build();

            // Setup Progress (Disabled for now to fix clippy, pending properly parallel implementation)
            // let pb = ...

            let findings: Vec<_> = walker
                .par_bridge()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    let path_str = path.to_string_lossy();
                    for pattern in &config.core.ignore {
                        if path_str.contains(pattern) {
                            return false;
                        }
                    }
                    true
                })
                .flat_map(|entry| {
                    if limit_tracker.check() {
                        return Vec::new();
                    }

                    let file_findings =
                        scan_file(entry.path(), &rules, &config, Some(&limit_tracker));

                    // Filter out BINARY_FILE / MAX_FILE_SIZE
                    let mut is_skipped = false;
                    if let Some(first) = file_findings.first() {
                        if first.rule_id == veil_core::RULE_ID_BINARY_FILE
                            || first.rule_id == veil_core::RULE_ID_MAX_FILE_SIZE
                        {
                            is_skipped = true;
                        }
                    }

                    if is_skipped {
                        skipped_files_atomic.fetch_add(1, Ordering::Relaxed);
                        Vec::new()
                    } else {
                        scanned_files_atomic.fetch_add(1, Ordering::Relaxed);
                        file_findings
                    }
                })
                .collect();

            all_findings.extend(findings);
        }
    }

    // Formatting & Output
    let scanned_files = scanned_files_atomic.load(Ordering::Relaxed);
    let skipped_files = skipped_files_atomic.load(Ordering::Relaxed);
    // Total files is scanned + skipped (approximation for streaming walker)
    // Actually, `scan_path` defines total_files as entries.len().
    // Here we don't easily have entries.len() unless we collect first.
    // For consistency with `scan_path`, let's sum them up.
    let total_files = scanned_files + skipped_files;
    let duration = start_time.elapsed();

    // Build Summary
    let mut severity_counts = HashMap::new();
    for f in &all_findings {
        *severity_counts.entry(f.severity.clone()).or_insert(0) += 1;
    }

    // Check truncation
    let is_truncated = limit_tracker.check();
    if is_truncated {
        eprintln!(
            "⚠ Reached finding limit ({}). Further findings were not scanned.",
            limit_val.unwrap_or(0)
        );
    }

    let summary = Summary::new(
        total_files,
        scanned_files,
        skipped_files,
        all_findings.len(), // findings_count (total found)
        all_findings.len(), // shown_findings
        is_truncated,
        duration,
        severity_counts,
    );

    // Select formatter
    let formatter: Box<dyn Formatter> = match format.to_lowercase().as_str() {
        "json" => Box::new(crate::formatters::json::JsonFormatter),
        "html" => Box::new(crate::formatters::html::HtmlFormatter::new()),
        #[cfg(feature = "table")]
        "table" => Box::new(crate::formatters::table::TableFormatter),
        "md" | "markdown" => Box::new(crate::formatters::markdown::MarkdownFormatter),
        _ => Box::new(TextFormatterWrapper),
    };

    formatter.print(&all_findings, &summary)?;

    // Determine exit code
    let threshold = fail_score.or(config.core.fail_on_score).unwrap_or(0);

    let should_fail = if all_findings.is_empty() {
        false
    } else if threshold == 0 {
        true
    } else {
        all_findings.iter().any(|f| f.score >= threshold)
    };

    Ok(should_fail)
}

struct TextFormatterWrapper;
impl Formatter for TextFormatterWrapper {
    fn print(&self, findings: &[veil_core::model::Finding], _summary: &Summary) -> Result<()> {
        for finding in findings {
            print_finding(finding);
        }
        Ok(())
    }
}
