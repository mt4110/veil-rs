use crate::cli::FixArgs;
use crate::commands::scan::collect_findings;
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn fix(args: &FixArgs) -> Result<()> {
    if !args.apply {
        println!(
            "{}",
            "Running in DRY-RUN mode. Use --apply to write changes."
                .yellow()
                .bold()
        );
    } else {
        println!(
            "{}",
            "Running in APPLY mode. Files will be modified."
                .red()
                .bold()
        );
    }

    // 1. Gather Findings
    println!("{}", "Scanning for findings to fix...".dimmed());
    let result = collect_findings(
        &args.paths,
        None,
        None,
        None,
        false, // staged
        None,  // mask
        false, // unsafe
        None,  // limit
        None,  // baseline
    )?;

    if result.findings.is_empty() {
        println!("{}", "No findings to fix. Clean! ✨".green());
        return Ok(());
    }

    // 2. Group findings by file
    let mut file_findings: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    for f in &result.findings {
        file_findings
            .entry(f.path.clone())
            .or_default()
            .push(f.line_number);
    }

    // Sort line numbers and deduplicate
    for lines in file_findings.values_mut() {
        lines.sort_unstable();
        lines.dedup();
    }

    // 3. Process each file
    let mut modified_count = 0;

    for (path, lines) in file_findings {
        if lines.is_empty() {
            continue;
        }

        if let Err(e) = process_file(&path, &lines, args.apply) {
            eprintln!("Failed to process {}: {}", path.display(), e);
        } else {
            modified_count += 1;
        }
    }

    if args.apply {
        println!("\nApplied fixes to {} files.", modified_count);
    } else {
        println!("\nDry-run complete. Would modify {} files.", modified_count);
    }

    Ok(())
}

fn process_file(path: &Path, target_lines: &[usize], apply: bool) -> Result<()> {
    // 1. Determine comment style
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let comment_prefix = match ext {
        "rs" | "js" | "ts" | "tsx" | "jsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp" => "//",
        "py" | "rb" | "sh" | "yaml" | "yml" | "toml" => "#",
        _ => {
            eprintln!(
                "Skipping {}: Unknown comment style for extension '{}'",
                path.display(),
                ext
            );
            return Ok(());
        }
    };

    let content = fs::read_to_string(path).context("read file")?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Ensure we handle files ending with newline correctly when splitting/joining?
    // content.lines() strips newlines.
    // We will reconstruct with newlines.

    let target_set: HashSet<usize> = target_lines.iter().cloned().collect();
    let mut modified_lines_list: Vec<(usize, String, String)> = Vec::new(); // (line_num, old, new)

    let mut new_content_lines = Vec::new();
    // lines are 0-indexed in vec, finding.line_number is 1-indexed

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        if target_set.contains(&line_num) {
            // Check if already ignored
            if line.contains("veil:ignore") {
                new_content_lines.push(line.clone());
                continue;
            }

            let new_line = format!("{} {} veil:ignore", line, comment_prefix);
            modified_lines_list.push((line_num, line.clone(), new_line.clone()));
            new_content_lines.push(new_line);
        } else {
            new_content_lines.push(line.clone());
        }
    }

    // Reconstruct content
    // content.lines() eats newlines. If original file had newline at end, join("\n") matches mostly.
    // Ideally we detect EOL style but for now "\n" is standard.
    let new_content = new_content_lines.join("\n");
    // Append final newline if original had one?
    // fs::read_to_string preserves it. lines() removes it.
    // If we want to be exact, checking if content ends with \n.
    let final_content = if content.ends_with('\n') {
        new_content + "\n"
    } else {
        new_content
    };

    if modified_lines_list.is_empty() {
        return Ok(());
    }

    if apply {
        fs::write(path, final_content).context("write file")?;
        println!("{} {}", "Fixed".green(), path.display());
    } else {
        println!("\n{} {}", "Would fix".yellow(), path.display());
        for (ln, old, new) in modified_lines_list {
            println!("  Line {}:", ln);
            println!("    - {}", old.red());
            println!("    + {}", new.green());
        }
    }

    Ok(())
}
