use crate::cli::GitScanArgs;
use anyhow::{Context, Result};
use colored::Colorize;
use git2::{DiffOptions, Repository};
use std::path::{Path, PathBuf};

use veil_core::Finding;

pub fn scan(args: &GitScanArgs, config_path: Option<&PathBuf>) -> Result<()> {
    let root = std::env::current_dir()?;
    let repo = Repository::open(&root).context("Failed to open git repository")?;

    // Load config (we need rules)
    let config = crate::config_loader::load_effective_config(config_path)?;

    // Use all built-in rules
    let all_rules = veil_core::try_get_all_rules(&config, vec![])?;

    // Determine range
    let range = if args.pr {
        // PR Mode: Try to find upstream, then origin/main, then origin/master
        resolve_pr_base(&repo)?
    } else {
        // Explicit range or default to HEAD
        args.range.clone().unwrap_or_else(|| "HEAD".to_string())
    };

    println!("{} {}", "Scanning git history range:".cyan(), range.bold());

    let mut revwalk = repo.revwalk()?;

    if range.contains("..") {
        revwalk.push_range(&range)?;
    } else {
        // Implementation:
        let obj = repo.revparse_single(&range)?;
        let commit = obj
            .peel_to_commit()
            .context("Git scan range must resolve to a commit")?;
        revwalk.push(commit.id())?;
    }

    // Config: Sort by time?
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut findings = Vec::new();
    let mut scanned_commits = 0;
    let mut checked_files = 0;
    let max_files = config
        .core
        .max_file_count
        .unwrap_or(veil_core::DEFAULT_MAX_FILE_COUNT);

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Scan this commit against parent(s)
        let parents = commit.parents();
        let parent = parents.into_iter().next(); // Handle first parent only for now (merge commits?)

        let tree = commit.tree()?;
        let parent_tree = parent.as_ref().map(|p| p.tree()).transpose()?;

        let mut diff_opts = DiffOptions::new();
        let diff =
            repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

        // Rewrite without `print` to collect findings comfortably
        // `diff.deltas()` gives deltas. But we need hunks/lines.
        // `diff.foreach` allows callbacks.
        // We can capture `&mut findings` if we use `foreach`.

        let mut binary_paths = std::collections::HashSet::new();
        // Validate complete changed blobs even though detection only inspects added lines.
        // Invalid text must not disappear through the diff callback's UTF-8 conversion.
        for delta in diff.deltas() {
            if delta.new_file().id().is_zero() || delta.new_file().mode() == git2::FileMode::Commit
            {
                continue;
            }
            checked_files += 1;
            if checked_files > max_files {
                anyhow::bail!(
                    "Git scan incomplete: file count limit reached (core.max_file_count)"
                );
            }
            let blob = repo.find_blob(delta.new_file().id())?;
            let path = delta.new_file().path().unwrap_or(Path::new("unknown"));
            let validation = veil_core::scan_data(path, blob.content(), &[], &config);
            use veil_core::scanner::result::FileScanStatus;
            match FileScanStatus::from_findings(&validation) {
                FileScanStatus::TooLarge => anyhow::bail!(
                    "Git scan incomplete: file size limit reached (core.max_file_size)"
                ),
                FileScanStatus::ReadError => {
                    anyhow::bail!("Git scan incomplete: file read or UTF-8 decoding error")
                }
                FileScanStatus::Binary => {
                    binary_paths.insert(path.to_path_buf());
                }
                FileScanStatus::Scanned => {}
            }
        }

        let mut commit_findings = Vec::new();
        diff.foreach(
            &mut |_delta, _progress| true, // file_cb
            None,                          // binary_cb
            None,                          // hunk_cb
            Some(&mut |delta, _hunk, line| {
                if line.origin() == '+'
                    && !delta
                        .new_file()
                        .path()
                        .is_some_and(|path| binary_paths.contains(path))
                {
                    if let Ok(content) = std::str::from_utf8(line.content()) {
                        let content = content.trim_end();
                        if !content.is_empty() {
                            let path = delta.new_file().path().unwrap_or(Path::new("unknown"));
                            let line_findings =
                                veil_core::scan_content(content, path, &all_rules, &config);
                            for mut f in line_findings {
                                f.commit_sha = Some(oid.to_string());
                                f.author = Some(commit.author().name().unwrap_or("").to_string());
                                f.date = Some(commit.time().seconds().to_string());
                                commit_findings.push(f);
                            }
                        }
                    }
                }
                true
            }),
        )?;

        findings.extend(commit_findings);
        if config
            .output
            .max_findings
            .is_some_and(|max| findings.len() >= max)
        {
            anyhow::bail!("Git scan incomplete: finding limit reached (output.max_findings)");
        }
        scanned_commits += 1;
    }

    // Print summary
    println!(
        "\nScanned {} commits. Found {} secrets.",
        scanned_commits,
        findings.len()
    );

    for f in &findings {
        println!("{}", format_finding(f));
    }

    Ok(())
}

fn format_finding(f: &Finding) -> String {
    let sha = f
        .commit_sha
        .as_deref()
        .unwrap_or("?")
        .chars()
        .take(7)
        .collect::<String>();
    let author = f.author.as_deref().unwrap_or("?");
    format!(
        "{} [{}] {}:{} - {} ({})",
        sha.yellow(),
        f.rule_id.cyan(),
        f.path.display(),
        f.line_number,
        f.masked_snippet,
        author.dimmed()
    )
}

fn resolve_pr_base(repo: &Repository) -> Result<String> {
    // 1. Try upstream of current HEAD
    if let Ok(head) = repo.head() {
        if let Some(head_name) = head.shorthand() {
            if let Ok(branch) = repo.find_branch(head_name, git2::BranchType::Local) {
                if let Ok(upstream) = branch.upstream() {
                    if let Ok(Some(upstream_name)) = upstream.name() {
                        return Ok(format!("{}..HEAD", upstream_name));
                    }
                }
            }
        }
    }

    // 2. Fallback to origin/main
    if repo
        .find_branch("origin/main", git2::BranchType::Remote)
        .is_ok()
    {
        return Ok("origin/main..HEAD".to_string());
    }

    // 3. Fallback to origin/master
    if repo
        .find_branch("origin/master", git2::BranchType::Remote)
        .is_ok()
    {
        return Ok("origin/master..HEAD".to_string());
    }

    anyhow::bail!(
        "Could not determine base branch for PR scan. Tried upstream, origin/main, origin/master."
    )
}
