use crate::formatters::html::HtmlFormatter;
use crate::formatters::json::JsonFormatter;
use crate::formatters::markdown::MarkdownFormatter;
use crate::formatters::{Formatter, Summary};
use crate::output::formatter::print_finding;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{Delta, DiffOptions, Repository};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use veil_core::get_all_rules;

pub struct ScanResultForCli {
    pub summary: Summary,
    pub findings: Vec<veil_core::model::Finding>,
}

#[allow(clippy::too_many_arguments)]
pub fn collect_findings(
    paths: &[PathBuf],
    config_override: Option<&veil_config::Config>,
    commit: Option<&str>,
    since: Option<&str>,
    staged: bool,
    mask_mode_arg: Option<&str>,
    unsafe_output: bool,
    limit: Option<usize>,
) -> Result<ScanResultForCli> {
    let start_time = Instant::now();

    // 1. Load Config (Merge with Defaults)
    let mut config = if let Some(cfg) = config_override {
        cfg.clone()
    } else {
        crate::config_loader::load_effective_config(None)?
    };

    // 2. Override Config with CLI args
    if let Some(mode) = mask_mode_arg {
        config.output.mask_mode = Some(match mode {
            "partial" => veil_config::MaskMode::Partial,
            "plain" => veil_config::MaskMode::Plain,
            _ => veil_config::MaskMode::Redact,
        });
    }
    if unsafe_output {
        config.output.mask_mode = Some(veil_config::MaskMode::Plain);
    }

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

    // Stats counters
    let scanned_files_atomic = AtomicUsize::new(0);
    let skipped_files_atomic = AtomicUsize::new(0);
    let mut all_findings = Vec::new();

    // Limit Logic (Global)
    let limit_val = if let Some(l) = limit {
        if l == 0 {
            None
        } else {
            Some(l)
        }
    } else {
        config.output.max_findings
    };

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
                                    let file_findings = veil_core::scan_data(
                                        path_val,
                                        blob.content(),
                                        &rules,
                                        &config,
                                    );

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
                                    } else {
                                        scanned_files_atomic.fetch_add(1, Ordering::Relaxed);
                                        all_findings.extend(file_findings);
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
            .context("Failed to parse time. Use RFC3339 or YYYY-MM-DD")?;

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
                                        let file_findings = veil_core::scan_data(
                                            path,
                                            blob.content(),
                                            &rules,
                                            &config,
                                        );
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
                                        } else {
                                            scanned_files_atomic.fetch_add(1, Ordering::Relaxed);
                                            all_findings.extend(file_findings);
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
                return Ok(ScanResultForCli {
                    summary: Summary::new(
                        0,
                        0,
                        0,
                        0,
                        0,
                        false,
                        start_time.elapsed(),
                        HashMap::new(),
                    ),
                    findings: vec![],
                });
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
                            let file_findings =
                                veil_core::scan_data(path_val, blob.content(), &rules, &config);
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
                            } else {
                                scanned_files_atomic.fetch_add(1, Ordering::Relaxed);
                                all_findings.extend(file_findings);
                            }
                        }
                    }
                }
            }
        }
    } else {
        // 4. Default FS scan
        let targets = resolve_paths(paths)?;
        let mut current_total = all_findings.len();

        for path in targets {
            if let Some(max) = limit_val {
                if current_total >= max {
                    break;
                }
            }
            let mut run_config = config.clone();
            if let Some(max) = limit_val {
                let remaining = max.saturating_sub(current_total);
                run_config.output.max_findings = Some(remaining);
            } else {
                run_config.output.max_findings = None;
            }

            let result = veil_core::scan_path(&path, &rules, &run_config);
            scanned_files_atomic.fetch_add(result.scanned_files, Ordering::Relaxed);
            skipped_files_atomic.fetch_add(result.skipped_files, Ordering::Relaxed);
            let count = result.findings.len();
            all_findings.extend(result.findings);
            current_total += count;
        }
    }

    let scanned_files = scanned_files_atomic.load(Ordering::Relaxed);
    let skipped_files = skipped_files_atomic.load(Ordering::Relaxed);
    let total_files = scanned_files + skipped_files;
    let duration = start_time.elapsed();

    let mut severity_counts = HashMap::new();
    for f in &all_findings {
        *severity_counts.entry(f.severity.clone()).or_insert(0) += 1;
    }

    let is_truncated = if let Some(max) = limit_val {
        all_findings.len() >= max
    } else {
        false
    };

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
        all_findings.len(),
        all_findings.len(),
        is_truncated,
        duration,
        severity_counts,
    );

    Ok(ScanResultForCli {
        summary,
        findings: all_findings,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn scan(
    paths: &[PathBuf],
    config_path: Option<&PathBuf>,
    format: &str,
    fail_score: Option<u32>,
    commit: Option<&str>,
    since: Option<&str>,
    staged: bool,
    show_progress: bool,
    mask_mode_arg: Option<&str>,
    unsafe_output: bool,
    limit: Option<usize>,
    fail_on_findings: Option<usize>,
    fail_on_severity: Option<veil_core::Severity>,
    write_baseline: Option<PathBuf>,
) -> Result<bool> {
    if show_progress {
        println!("Scanning...");
    }

    // Load config here for scan command to support --config arg
    let config = if let Some(path) = config_path {
        Some(crate::config_loader::load_effective_config(Some(path))?)
    } else {
        None
    };

    let result = collect_findings(
        paths,
        config.as_ref(), // Passed overridden config or None (defaults loaded inside)
        commit,
        since,
        staged,
        mask_mode_arg,
        unsafe_output,
        limit,
    )?;

    // Handle Write Baseline (S26)
    if let Some(path) = &write_baseline {
        use veil_core::baseline::{from_findings, save_baseline};

        // Convert findings to snapshot
        let snapshot = from_findings(&result.findings, env!("CARGO_PKG_VERSION"));

        // Save to file
        save_baseline(path, &snapshot).context("Failed to save baseline file")?;

        println!(
            "Baseline written to {:?} ({} findings, schema={})",
            path,
            snapshot.entries.len(),
            snapshot.schema
        );

        // Bootstrap is always success
        return Ok(false);
    }

    // Output Formatting
    let formatter: Box<dyn Formatter> = match format.to_lowercase().as_str() {
        "json" => Box::new(JsonFormatter),
        "html" => Box::new(HtmlFormatter::new()),
        #[cfg(feature = "table")]
        "table" => Box::new(crate::formatters::table::TableFormatter),
        "md" | "markdown" => Box::new(MarkdownFormatter),
        _ => Box::new(TextFormatterWrapper),
    };

    formatter.print(&result.findings, &result.summary)?;

    // Reload config for default fail_score if needed
    let effective_fail_score = if let Some(fs) = fail_score {
        Some(fs)
    } else {
        config
            .as_ref()
            .and_then(|c| c.core.fail_on_score)
            .or_else(|| {
                crate::config_loader::load_effective_config(None)
                    .ok()
                    .and_then(|c| c.core.fail_on_score)
            })
    };

    let should_fail = determine_exit_code(
        &result.summary,
        &result.findings,
        fail_on_findings,
        fail_on_severity.as_ref(),
        effective_fail_score,
    );

    Ok(should_fail)
}

fn determine_exit_code(
    summary: &Summary,
    findings: &[veil_core::model::Finding],
    fail_on_findings: Option<usize>,
    fail_on_severity: Option<&veil_core::Severity>,
    fail_on_score: Option<u32>,
) -> bool {
    // 1) fail-on-findings
    if let Some(threshold) = fail_on_findings {
        if summary.findings_count >= threshold {
            eprintln!(
                "CI failed: findings_count ({}) exceeded threshold {}",
                summary.findings_count, threshold
            );
            return true;
        }
    }

    // 2) fail-on-severity
    if let Some(min_level) = fail_on_severity {
        if findings.iter().any(|f| &f.severity >= min_level) {
            let count = findings.iter().filter(|f| &f.severity >= min_level).count();
            let max_sev = findings
                .iter()
                .map(|f| &f.severity)
                .max()
                .unwrap_or(min_level);
            eprintln!(
                "CI failed: found {} findings >= severity {} (max: {})",
                count, min_level, max_sev
            );
            return true;
        }
    }

    // 3) fail-on-score
    if let Some(min_score) = fail_on_score {
        if let Some(max_score) = findings.iter().map(|f| f.score).max() {
            if max_score >= min_score {
                let count = findings.iter().filter(|f| f.score >= min_score).count();
                eprintln!(
                    "CI failed: found {} finding(s) with score >= {} (max: {})",
                    count, min_score, max_score
                );
                return true;
            }
        }
    }

    false
}

fn resolve_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut resolved = Vec::new();
    if paths.is_empty() {
        resolved.push(std::env::current_dir()?);
        return Ok(resolved);
    }
    for p in paths {
        // Simple existence check or just pass through.
        // realpath/canonicalize might be good but let's just pass paths.
        resolved.push(p.clone());
    }
    Ok(resolved)
}

use colored::Colorize;

// ... (existing imports, keep them)

// (Inside scan function, remove debug print)

struct TextFormatterWrapper;
impl Formatter for TextFormatterWrapper {
    fn print(&self, findings: &[veil_core::model::Finding], summary: &Summary) -> Result<()> {
        for finding in findings {
            print_finding(finding);
        }

        println!();
        println!("{}", "Scan Summary".bold().underline());
        println!(
            "  Time Taken:    {:.2}s",
            summary.duration_ms as f64 / 1000.0
        );
        println!("  Total Files:   {}", summary.total_files);
        println!("  Scanned Files: {}", summary.scanned_files);
        if summary.skipped_files > 0 {
            println!(
                "  Skipped Files: {} (binary/large)",
                summary.skipped_files.to_string().yellow()
            );
        }

        if summary.findings_count == 0 {
            println!("{}", "  No secrets found.".green());
        } else {
            println!(
                "  Findings:      {}",
                summary.findings_count.to_string().red().bold()
            );
        }

        if summary.limit_reached {
            println!("{}", "  (Output truncated due to limit)".yellow());
        }

        Ok(())
    }
}
