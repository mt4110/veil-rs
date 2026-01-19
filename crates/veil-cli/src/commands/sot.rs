use crate::cli::{SotCommand, SotNewArgs, SotRenameArgs};
use anyhow::{anyhow, Context, Result};
use chrono::Local;
use colored::Colorize;
use regex::Regex;
use std::env;
use std::fs;
use std::process::Command;

pub fn run(cmd: &SotCommand) -> Result<bool> {
    match cmd {
        SotCommand::New(args) => run_new(args),
        SotCommand::Rename(args) => run_rename(args),
    }
}

fn run_new(args: &SotNewArgs) -> Result<bool> {
    // 1. Determine release
    let release = match &args.release {
        Some(r) => r.clone(),
        None => infer_release()?,
    };

    // 2. Prepare other vars
    let epic = &args.epic;
    let slug = args.slug.as_deref().unwrap_or("");
    let sanitized_slug = sanitize_slug(slug);

    // 3. Filename
    // slugなし: docs/pr/PR-TBD-<release>-epic-<epic>.md
    // slugあり: docs/pr/PR-TBD-<release>-epic-<epic>-<slug>.md
    let filename = if sanitized_slug.is_empty() {
        format!("PR-TBD-{}-epic-{}.md", release, epic.to_lowercase())
    } else {
        format!(
            "PR-TBD-{}-epic-{}-{}.md",
            release,
            epic.to_lowercase(),
            sanitized_slug
        )
    };

    let out_dir = &args.out;
    let out_path = out_dir.join(&filename);

    // 4. Git info
    let (branch, commit) =
        get_git_info().unwrap_or_else(|_| ("unknown".to_string(), "unknown".to_string()));
    let created_at = Local::now().format("%Y-%m-%d").to_string();
    let title = args.title.as_deref().unwrap_or("TBD");

    // 5. Template replacement
    // We use include_str! relative to this file location
    let template = include_str!("../templates/sot/sot_v1.md");
    let content = template
        .replace("{{release}}", &release)
        .replace("{{epic}}", epic)
        .replace("{{created_at}}", &created_at)
        .replace("{{branch}}", &branch)
        .replace("{{commit}}", &commit)
        .replace("{{title}}", title);

    // 6. Action
    if args.dry_run {
        println!("Dry run: would create {}", out_path.display());
        println!();
        println!("--- Content ---");
        println!("{}", content);
    } else {
        // Validation
        if out_path.exists() && !args.force {
            return Err(anyhow!(
                "File {} already exists. Use --force to overwrite.",
                out_path.display()
            ));
        }

        fs::create_dir_all(out_dir)
            .context(format!("Failed to create output directory {:?}", out_dir))?;
        fs::write(&out_path, &content)
            .context(format!("Failed to write SOT file {:?}", out_path))?;

        // Output success info
        println!("{} Created SOT: {}", "✅".green(), out_path.display());
        println!();
        println!("Copy-paste into PR body:");
        println!("### SOT");
        println!("- {}", out_path.display());
        println!();
        println!("Next:");
        println!("git add {}", out_path.display());
        println!(
            "git commit -m \"docs: add SOT ({} Epic {})\"",
            release, epic
        );
    }

    Ok(false)
}


fn run_rename(args: &SotRenameArgs) -> Result<bool> {
    // 1) Locate source file
    let src_path = match &args.path {
        Some(p) => p.clone(),
        None => {
            let dir = &args.dir;
            let mut candidates: Vec<std::path::PathBuf> = Vec::new();

            let entries = fs::read_dir(dir)
                .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

            for e in entries {
                let e = e?;
                let path = e.path();
                if !path.is_file() {
                    continue;
                }
                let name = match path.file_name().and_then(|s| s.to_str()) {
                    Some(s) => s,
                    None => continue,
                };
                if name.starts_with("PR-TBD-") && name.ends_with(".md") {
                    candidates.push(path);
                }
            }

            match candidates.len() {
                0 => {
                    return Err(anyhow!(
                        "No PR-TBD SOT file found under {}. Provide --path or create one via `veil sot new`.",
                        dir.display()
                    ));
                }
                1 => candidates.remove(0),
                _ => {
                    let mut msg = String::from("Multiple PR-TBD SOT files found. Please specify --path:\n");
                    for c in &candidates {
                        msg.push_str(&format!("- {}\n", c.display()));
                    }
                    return Err(anyhow!(msg));
                }
            }
        }
    };

    // 2) Compute destination path
    let file_name = src_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid filename: {}", src_path.display()))?;

    if !file_name.starts_with("PR-TBD-") {
        return Err(anyhow!(
            "Expected a PR-TBD SOT file, but got: {}",
            src_path.display()
        ));
    }

    let dst_name = file_name.replacen("PR-TBD-", &format!("PR-{}-", args.pr), 1);
    let dst_path = src_path.with_file_name(dst_name);

    // 3) Load content and update front matter
    let content = fs::read_to_string(&src_path)
        .with_context(|| format!("Failed to read SOT file: {}", src_path.display()))?;

    // Update `pr:` line.
    // - If `pr: TBD` -> set to pr
    // - If `pr: <n>` differs -> require --force
    let re_pr = Regex::new(r"(?m)^pr:\s*(?P<val>\S+)\s*$").unwrap();
    let mut existing_val: Option<String> = None;
    if let Some(caps) = re_pr.captures(&content) {
        existing_val = Some(caps.name("val").unwrap().as_str().to_string());
    }

    if let Some(v) = existing_val.as_deref() {
        let v_norm = v.trim();
        if v_norm != "TBD" {
            if v_norm != args.pr.to_string() {
                if !args.force {
                    return Err(anyhow!(
                        "SOT already has pr: {} (expected TBD). Use --force to overwrite to pr: {}.",
                        v_norm,
                        args.pr
                    ));
                }
            }
        }
    }

    let updated = if re_pr.is_match(&content) {
        re_pr
            .replace(&content, format!("pr: {}", args.pr))
            .to_string()
    } else {
        // If the file has no pr: line, add it to the front matter (best-effort).
        // Insert after the first '---' line.
        let re_front = Regex::new(r"(?m)^---\s*$").unwrap();
        if let Some(mat) = re_front.find(&content) {
            let insert_at = mat.end();
            format!(
                "{}\npr: {}{}",
                &content[..insert_at],
                args.pr,
                &content[insert_at..]
            )
        } else {
            content.clone()
        }
    };

    // 4) Action
    if args.dry_run {
        println!(
            "Dry run: would rename {} -> {}",
            src_path.display(),
            dst_path.display()
        );
        println!("Dry run: would set front matter pr: {}", args.pr);
        return Ok(false);
    }

    if dst_path.exists() && !args.force {
        return Err(anyhow!(
            "Destination {} already exists. Use --force to overwrite.",
            dst_path.display()
        ));
    }

    fs::write(&dst_path, &updated)
        .with_context(|| format!("Failed to write SOT file: {}", dst_path.display()))?;

    if dst_path != src_path {
        fs::remove_file(&src_path)
            .with_context(|| format!("Failed to remove old SOT file: {}", src_path.display()))?;
    }

    println!(
        "{} Renamed SOT: {} -> {}",
        "✅".green(),
        src_path.display(),
        dst_path.display()
    );
    println!();
    println!("Copy-paste into PR body:");
    println!("### SOT");
    println!("- {}", dst_path.display());

    Ok(false)
}

fn infer_release() -> Result<String> {
    // Priority: env var > git branch
    if let Ok(v) = env::var("VEIL_RELEASE") {
        if !v.is_empty() {
            return Ok(v);
        }
    }

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    // If git command fails or returns valid output but no match
    match output {
        Ok(out) if out.status.success() => {
            let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Regex extract v\d+\.\d+\.\d+
            let re = Regex::new(r"v\d+\.\d+\.\d+").unwrap();
            if let Some(mat) = re.find(&branch) {
                Ok(mat.as_str().to_string())
            } else {
                Err(anyhow!("Could not infer release version from branch '{}'. Please specify --release <vX.Y.Z>", branch))
            }
        }
        _ => Err(anyhow!(
            "Failed to infer release (git unavailable?). Please specify --release <vX.Y.Z>"
        )),
    }
}

fn get_git_info() -> Result<(String, String)> {
    let branch_out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    let branch = if branch_out.status.success() {
        String::from_utf8_lossy(&branch_out.stdout)
            .trim()
            .to_string()
    } else {
        "unknown".to_string()
    };

    let commit_out = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    let commit = if commit_out.status.success() {
        String::from_utf8_lossy(&commit_out.stdout)
            .trim()
            .to_string()
    } else {
        "unknown".to_string()
    };

    Ok((branch, commit))
}

fn sanitize_slug(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    // [a-z0-9-] only, others to -, collapse -, trim
    let s = s.to_lowercase();
    let re = Regex::new(r"[^a-z0-9-]+").unwrap();
    let s = re.replace_all(&s, "-");
    let re_collapse = Regex::new(r"-+").unwrap();
    let s = re_collapse.replace_all(&s, "-");
    s.trim_matches('-').to_string()
}
