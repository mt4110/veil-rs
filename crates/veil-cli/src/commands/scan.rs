use crate::output::formatter::print_finding;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{Delta, DiffOptions, Repository};
use std::path::PathBuf;
use veil_config::{load_config, Config};
use veil_core::{get_all_rules, scan_content, scan_path};

pub fn scan(
    paths: &[PathBuf],
    config_path: Option<&PathBuf>,
    format: &str,
    fail_score: Option<u32>,
    commit: Option<&str>,
    since: Option<&str>,
    staged: bool,
) -> Result<bool> {
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
    let config = final_config;

    let rules = get_all_rules(&config);
    let mut all_findings = Vec::new();
    // Strategy Selection
    if let Some(commit_sha) = commit {
        // 1. Scan specific commit
        let repo = Repository::open(".")?;
        let obj = repo.revparse_single(commit_sha)?;

        let commit = obj.as_commit().context("Not a commit")?;

        let tree = commit.tree()?;

        // Scan changes introduced by this commit (diff against parent)
        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            let mut diff_opts = DiffOptions::new();
            let diff =
                repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut diff_opts))?;

            // Process diff deltas
            for delta in diff.deltas() {
                if delta.status() == Delta::Added || delta.status() == Delta::Modified {
                    if let Some(path) = delta.new_file().path() {
                        let path_val = path;
                        // Retrieve blob content from tree
                        if let Ok(entry) = tree.get_path(path_val) {
                            if let Ok(object) = entry.to_object(&repo) {
                                if let Some(blob) = object.as_blob() {
                                    // Skip large files (re-implement MAX_FILE_SIZE checks logic for git blob?)
                                    let max_size = config.core.max_file_size.unwrap_or(1_000_000);
                                    if blob.size() as u64 > max_size {
                                        all_findings.push(veil_core::model::Finding {
                                            path: path_val.to_path_buf(),
                                            line_number: 0,
                                            line_content: format!(
                                                "File size ({} bytes) exceeds limit",
                                                blob.size()
                                            ),
                                            rule_id: "MAX_FILE_SIZE".to_string(),
                                            masked_line: "".to_string(),
                                            severity: veil_core::model::Severity::High,
                                            score: 100,
                                            grade: veil_core::rules::grade::Grade::Critical,
                                        });
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
            // Root commit, scan full tree? For now, support only non-root or extensive logic
            println!("Scanning root commit not fully optimized yet.");
        }
    } else if let Some(since_str) = since {
        // 2. Scan history since time
        // Try parsing RFC3339 first, else fallback or error
        // Example: 2024-01-01T00:00:00Z
        let since_dt = DateTime::parse_from_rfc3339(since_str)
            .map(|dt| dt.with_timezone(&Local))
            .or_else(|_| {
                // Try YYYY-MM-DD (append T00:00:00Z)
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

            // Scan this commit (same logic as single commit scan, but we need to reuse it)
            // For MVP: just scan the diff of this commit against parent
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
                            if let Ok(entry) = tree.get_path(path) {
                                if let Ok(object) = entry.to_object(&repo) {
                                    if let Some(blob) = object.as_blob() {
                                        // Skip large files
                                        let max_size =
                                            config.core.max_file_size.unwrap_or(1_000_000);
                                        if blob.size() as u64 > max_size {
                                            continue;
                                        }

                                        if let Ok(content) = std::str::from_utf8(blob.content()) {
                                            let findings =
                                                scan_content(content, path, &rules, &config);
                                            // Tag findings with commit hash/author?
                                            // For now standard output findings
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
        // 3. Scan staged (Changes to be committed)
        let repo = Repository::open(".")?;
        // Handle case where there is no HEAD (initial commit)
        let diff = if let Ok(head) = repo.head() {
            if let Ok(head_tree) = head.peel_to_tree() {
                let index = repo.index()?;
                let mut diff_opts = DiffOptions::new();
                repo.diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut diff_opts))?
            } else {
                // Should not happen if head exists but careful
                return Ok(false);
            }
        } else {
            // Initial commit, diff empty tree to index?
            let index = repo.index()?;
            let mut diff_opts = DiffOptions::new();
            repo.diff_tree_to_index(None, Some(&index), Some(&mut diff_opts))?
        };

        let index = repo.index()?;

        for delta in diff.deltas() {
            if delta.status() == Delta::Added || delta.status() == Delta::Modified {
                if let Some(path) = delta.new_file().path() {
                    let path_val = path;
                    // Check ignore (index might map ignored files if forcibly added, but usually fine)
                    // repo.is_path_ignored(path_val) check?
                    if let Ok(true) = repo.is_path_ignored(path_val) {
                        continue;
                    }

                    // Retrieve blob from Index
                    if let Some(entry) = index.get_path(path_val, 0) {
                        if let Ok(blob) = repo.find_blob(entry.id) {
                            let max_size = config.core.max_file_size.unwrap_or(1_000_000);
                            if blob.size() as u64 > max_size {
                                // Add finding? Or silent skip?
                                // Consistent with others: add finding
                                all_findings.push(veil_core::model::Finding {
                                    path: path_val.to_path_buf(),
                                    line_number: 0,
                                    line_content: format!(
                                        "File size ({} bytes) exceeds limit",
                                        blob.size()
                                    ),
                                    rule_id: "MAX_FILE_SIZE".to_string(),
                                    masked_line: "".to_string(),
                                    severity: veil_core::model::Severity::High,
                                    score: 100,
                                    grade: veil_core::rules::grade::Grade::Critical,
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
        // 4. Default FS scan
        // Determine targets
        let targets = paths.iter().collect::<Vec<_>>();

        for path in targets {
            let findings = scan_path(path, &rules, &config);
            all_findings.extend(findings);
        }
    }

    if format.eq_ignore_ascii_case("json") {
        println!("{}", serde_json::to_string_pretty(&all_findings)?);
    } else if format.eq_ignore_ascii_case("html") {
        let formatter = crate::formatters::html::HtmlFormatter::new();
        let html_report = formatter.generate_report(&all_findings);
        println!("{}", html_report);
    } else {
        for finding in &all_findings {
            print_finding(finding);
        }
    }

    // Determine exit code
    let threshold = fail_score.or(config.core.fail_on_score).unwrap_or(0);

    let should_fail = if all_findings.is_empty() {
        false
    } else if threshold == 0 {
        // Legacy behavior: fail if any finding
        true
    } else {
        // Fail only if max score >= threshold
        all_findings.iter().any(|f| f.score >= threshold)
    };

    Ok(should_fail)
}
