use crate::cli::{SotCommand, SotNewArgs};
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
