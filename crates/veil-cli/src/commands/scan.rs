use crate::formatters::{
    DisplayFinding, FindingStatus, Formatter, HtmlFormatter, JsonFormatter, MarkdownFormatter,
    Summary, TableFormatter,
};
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
    pub suppressed_findings: Vec<veil_core::model::Finding>,
}

#[derive(Debug, Clone, Copy, PartialEq)] // Local Format enum
pub enum Format {
    Text,
    Json,
    Html,
    Table,
    Markdown,
}

impl From<&str> for Format {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Format::Json,
            "html" => Format::Html,
            "table" => Format::Table,
            "md" | "markdown" => Format::Markdown,
            _ => Format::Text,
        }
    }
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
    baseline_path: Option<&PathBuf>,
) -> Result<ScanResultForCli> {
    let start_time = Instant::now();

    // 1. Load Config (Merge with Defaults)
    let mut config = if let Some(cfg) = config_override {
        cfg.clone()
    } else {
        crate::config_loader::load_effective_config(None)?
    };

    // If baseline is provided, do not scan the baseline file itself.
    if let Some(bp) = baseline_path {
        if let Some(name) = bp.file_name().and_then(|s| s.to_str()) {
            if !config.core.ignore.iter().any(|p| p == name) {
                config.core.ignore.push(name.to_string());
            }
        }
        let raw = bp.to_string_lossy().to_string();
        if !raw.is_empty() && !config.core.ignore.iter().any(|p| p == &raw) {
            config.core.ignore.push(raw);
        }
    }

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
    let mut all_builtin_skips = std::collections::HashSet::new();

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

    let mut any_file_limit_reached = false;

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
            eprintln!("Scanning root commit not fully optimized yet.");
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
                        0, // baseline_suppressed
                        false,
                        false,
                        start_time.elapsed(),
                        None, // baseline_path
                        HashMap::new(),
                        Vec::new(),
                    ),
                    findings: vec![],
                    suppressed_findings: vec![],
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
            any_file_limit_reached |= result.file_limit_reached;
            all_builtin_skips.extend(result.builtin_skips);
            let count = result.findings.len();
            all_findings.extend(result.findings);
            current_total += count;
        }

        // Store this so we can reference it out of the block, we'll assign it to a boolean in the outer scope
        // Actually, it's easier to just pass it through the struct.
        // We'll mutate a variable defined outside the if/else block.
    }

    let scanned_files = scanned_files_atomic.load(Ordering::Relaxed);
    let skipped_files = skipped_files_atomic.load(Ordering::Relaxed);
    let total_files = scanned_files + skipped_files;
    let duration = start_time.elapsed();

    // Baseline Application (S27)
    let (final_findings, suppressed_findings, _new_count) = if let Some(path) = baseline_path {
        let baseline = veil_core::baseline::load_baseline(path)
            .with_context(|| format!("Failed to load baseline from {:?}", path))?;

        // Apply baseline
        let result = veil_core::baseline::apply_baseline(all_findings, Some(&baseline));

        let new_findings = result.new;
        let new_len = new_findings.len();
        // Keep suppressed for HTML display
        let suppressed = result.suppressed;

        (new_findings, suppressed, new_len)
    } else {
        let count = all_findings.len();
        (all_findings, Vec::new(), count)
    };

    let mut severity_counts: HashMap<veil_core::Severity, usize> = HashMap::new();
    for f in &final_findings {
        *severity_counts.entry(f.severity.clone()).or_insert(0) += 1;
    }

    let is_truncated = if let Some(max) = limit_val {
        final_findings.len() >= max
    } else {
        false
    };

    if is_truncated {
        // Will be handled in `scan` for detailed UX Output
    }

    let summary = Summary::new(
        total_files,
        scanned_files,
        skipped_files,
        final_findings.len() + suppressed_findings.len(), // total = new + suppressed
        final_findings.len(),                             // new
        suppressed_findings.len(),
        is_truncated,
        any_file_limit_reached,
        duration,
        baseline_path.map(|p| p.to_string_lossy().to_string()),
        severity_counts,
        all_builtin_skips.into_iter().collect(),
    );

    Ok(ScanResultForCli {
        summary,
        findings: final_findings,
        suppressed_findings,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn scan(
    paths: &[PathBuf],
    config_path: Option<&PathBuf>,
    format_str: String, // Changed from Format to String
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
    baseline: Option<PathBuf>,
    no_color: bool, // Passed from cli args
) -> Result<bool> {
    let format: Format = format_str.as_str().into();

    if show_progress {
        eprintln!("Scanning...");
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
        baseline.as_ref(),
    )?;

    // Handle Write Baseline (S26)
    if let Some(path) = &write_baseline {
        use veil_core::baseline::{from_findings, save_baseline};

        // Convert findings to snapshot
        let snapshot = from_findings(&result.findings, env!("CARGO_PKG_VERSION"));

        // Save to file
        save_baseline(path, &snapshot).context("Failed to save baseline file")?;

        eprintln!(
            "Baseline written to {:?} ({} findings, schema={})",
            path,
            snapshot.entries.len(),
            snapshot.schema
        );

        // Bootstrap is always success
        return Ok(false);
    }

    // Output Formatting
    let displays: Vec<DisplayFinding> = if let Format::Html = format {
        // Collect new findings
        let mut listing =
            Vec::with_capacity(result.findings.len() + result.suppressed_findings.len());
        for f in result.findings.iter() {
            listing.push(DisplayFinding {
                inner: f.clone(),
                status: FindingStatus::New,
            });
        }
        for f in result.suppressed_findings.iter() {
            listing.push(DisplayFinding {
                inner: f.clone(),
                status: FindingStatus::Suppressed,
            });
        }
        listing
    } else {
        result
            .findings
            .iter()
            .map(|f| DisplayFinding {
                inner: f.clone(),
                status: FindingStatus::New,
            })
            .collect()
    };

    let formatter: Box<dyn Formatter> = match format {
        Format::Json => Box::new(JsonFormatter),
        Format::Html => Box::new(HtmlFormatter::new()),
        #[cfg(feature = "table")]
        Format::Table => Box::new(TableFormatter),
        #[cfg(not(feature = "table"))]
        Format::Table => {
            eprintln!("Table format not compiled in");
            Box::new(TextFormatterWrapper { no_color })
        }
        Format::Markdown => Box::new(MarkdownFormatter),
        Format::Text => Box::new(TextFormatterWrapper { no_color }),
    };

    formatter.print(&displays, &result.summary)?;

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

    if !result.summary.builtin_skips.is_empty() {
        eprintln!(
            "{} Skipped by default: {}",
            "ℹ".cyan(),
            result.summary.builtin_skips.join(", ")
        );
    }

    if result.summary.file_limit_reached {
        eprintln!();
        eprintln!("{}", "❌ Scan Incomplete (Exit Code 2)".red().bold());
        eprintln!("  What: The maximum file count limit (core.max_file_count) was reached.");
        eprintln!("  Why:  To prevent runaway scans, Veil stops when this safety boundary is hit.");
        eprintln!("        However, passing CI with a truncated scan is dangerous.");
        eprintln!();
        eprintln!("{}", "  How to fix:".bold());
        eprintln!("    A) Reduce your scanning scope by pointing Veil to specific targets:");
        eprintln!("       veil scan src/ configs/");
        eprintln!("    B) Exclude the vast/noise directories in your veil.toml:");
        eprintln!("       [core]");
        eprintln!("       ignore = [\"tests/data\", \"docs/images\"]");
        eprintln!("    C) Increase the limit if you genuinely have a massive repository.");
        std::process::exit(2);
    }

    if result.summary.limit_reached {
        eprintln!();
        eprintln!("{}", "❌ Scan Incomplete (Exit Code 2)".red().bold());
        eprintln!("  What: The maximum findings limit (output.max_findings) was reached.");
        eprintln!("  Why:  Too many secrets were detected, risking OOM or unreadable reports.");
        eprintln!("        Passing CI with truncated findings hides remaining vulnerabilities.");
        eprintln!();
        eprintln!("{}", "  How to fix:".bold());
        eprintln!("    A) Address the current findings and re-scan.");
        eprintln!("    B) Filter out noisy rules in your veil.toml if they are false positives:");
        eprintln!("       [rules.\"rule.id.here\"]");
        eprintln!("       enabled = false");
        eprintln!(
            "    C) Use Baseline to suppress existing known-issues so you only see new ones."
        );
        std::process::exit(2);
    }

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
        if summary.new_findings >= threshold {
            eprintln!(
                "CI failed: new_findings ({}) exceeded threshold {}",
                summary.new_findings, threshold
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

struct TextFormatterWrapper {
    no_color: bool,
}
impl Formatter for TextFormatterWrapper {
    fn print(&self, findings: &[DisplayFinding], summary: &Summary) -> Result<()> {
        if self.no_color {
            colored::control::set_override(false);
        }

        for finding in findings {
            // Only print new findings (suppressed status are skipped in text output by contract?)
            // Wait, DisplayFindings passed here might include Suppressed if logic in `scan` didn't filter them.
            // But we conditionally prepare `displays` in `scan`.
            // `scan` helper: `if let Format::Html = format { ... includes suppressed ... } else { ... only new ... }`
            // So if we are here, we are using Text format, so `displays` should only contain New findings.
            // But good to be safe.
            if let FindingStatus::New = finding.status {
                print_finding(&finding.inner);
            }
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

        if summary.baseline_suppressed > 0 && summary.new_findings == 0 {
            println!(
                "{}",
                format!(
                    "  No new secrets found. (Baseline suppressed: {})",
                    summary.baseline_suppressed
                )
                .green()
            );
        } else if summary.new_findings == 0 {
            // total might be 0 or just all suppressed?
            // Actually, if new_findings == 0 it covers both cases unless we strictly check total==0 for "No secrets"
            // Case A: total=0 -> "No secrets found"
            // Case B: total>0, new=0 -> "No new secrets found"

            if summary.total_findings == 0 {
                println!("{}", "  No secrets found.".green());
            } else {
                println!(
                    "{}",
                    format!(
                        "  No new secrets found. (Baseline suppressed: {})",
                        summary.baseline_suppressed
                    )
                    .green()
                );
            }
        } else {
            let suppressed_msg = if summary.baseline_suppressed > 0 {
                format!(" (Baseline suppressed: {})", summary.baseline_suppressed)
            } else {
                "".to_string()
            };

            println!(
                "{}",
                format!(
                    "  Found {} new secrets.{}",
                    summary.new_findings, suppressed_msg
                )
                .red()
            );
        }

        if summary.limit_reached {
            println!("{}", "  (Output truncated due to finding limit)".yellow());
        }

        if summary.file_limit_reached {
            println!(
                "{}",
                "  (Scan incomplete: stopped early due to max_file_count limit)"
                    .red()
                    .bold()
            );
        }

        Ok(())
    }
}
