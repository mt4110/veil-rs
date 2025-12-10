use crate::cli::TriageArgs;
use crate::commands::scan::collect_findings;
use anyhow::{Context, Result};
use colored::Colorize;
use inquire::Select;
use std::path::Path;
use toml_edit::{Array, DocumentMut};

pub fn triage(args: &TriageArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let config_path = root.join("veil.toml");

    // 1. Gather Findings
    println!("{}", "Running scan to gather findings...".dimmed());
    let result = collect_findings(
        &args.paths,
        None,  // load default config
        None,  // no commit limit
        None,  // no time limit
        false, // not staged
        None,  // default mask mode
        false, // not unsafe
        None,  // no limit (we want all to triage)
    )?;

    if result.findings.is_empty() {
        println!("{}", "No findings to triage. Great job! 🎉".green());
        return Ok(());
    }

    println!(
        "Found {} findings. Starting interactive triage...",
        result.findings.len().to_string().bold()
    );

    // 2. Load Config Document for Editing
    let config_content = if config_path.exists() {
        std::fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = config_content
        .parse::<DocumentMut>()
        .or_else(|_| "".parse::<DocumentMut>())
        .context("Failed to parse veil.toml")?;

    // Sort findings for better UX (Severity Desc, then Path)
    let mut findings = result.findings;
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.line_number.cmp(&b.line_number))
    });

    // 3. Interactive Loop
    let mut modified = false;
    let mut skipped_files = Vec::new();

    for (i, finding) in findings.iter().enumerate() {
        // Skip if we already ignored this file in this session
        if skipped_files.contains(&finding.path) {
            continue;
        }

        println!("\n{}: {}/{}", "Finding".bold(), i + 1, findings.len());
        println!("{} [{}]", "Rule:".cyan(), finding.rule_id);
        println!(
            "{} {}:{}",
            "File:".cyan(),
            finding.path.display(),
            finding.line_number
        );
        println!("{} {}", "Severity:".cyan(), finding.severity);
        println!("{}", "Snippet:".dimmed());
        println!("{}", finding.masked_snippet.trim());
        println!();

        let options = vec![
            "Keep (Do nothing)",
            "Ignore File (Add to veil.toml)",
            "Skip (Skip rest of this file)",
            "Quit (Save & Exit)",
        ];

        let ans = Select::new("Action?", options).prompt();

        match ans {
            Ok("Keep (Do nothing)") => {
                println!("{}", "Kept.".dimmed());
            }
            Ok("Ignore File (Add to veil.toml)") => {
                add_ignore_path(&mut doc, &finding.path, &root)?;
                modified = true;
                skipped_files.push(finding.path.clone());
                println!(
                    "{}",
                    format!("Added {} to ignores.", finding.path.display()).green()
                );
            }
            Ok("Skip (Skip rest of this file)") => {
                skipped_files.push(finding.path.clone());
                println!("{}", "Skipping rest of file.".dimmed());
            }
            Ok("Quit (Save & Exit)") => {
                break;
            }
            Err(_) => break, // Handle C-c
            _ => {}
        }
    }

    // 4. Save
    if modified {
        println!("\n{}", "Saving changes to veil.toml...".yellow());
        std::fs::write(&config_path, doc.to_string())?;
        println!("{}", "Saved successfully.".green());
    } else {
        println!("\nNo changes made.");
    }

    Ok(())
}

fn add_ignore_path(doc: &mut DocumentMut, path: &Path, root: &Path) -> Result<()> {
    // 1. Resolve relative path
    let rel_path = path.strip_prefix(root).unwrap_or(path);
    // Force Unix style separators for config consistency
    let path_str = rel_path.to_string_lossy().replace("\\", "/");

    // 2. Ensure structure: [core] -> ignore = []
    let core = doc.entry("core").or_insert(toml_edit::table());
    let core_tbl = core.as_table_mut().context("core is not a table")?;

    let ignore = core_tbl
        .entry("ignore")
        .or_insert(toml_edit::value(Array::new()));

    if let Some(arr) = ignore.as_array_mut() {
        // Check duplicates
        if !arr.iter().any(|v| v.as_str() == Some(&path_str)) {
            arr.push(path_str);
        }
    } else {
        return Err(anyhow::anyhow!("core.ignore is not an array"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_ignore_path_preserves_comments() {
        let root = Path::new("/repo");
        let input = r#"
[core]
# Pre-existing comment
ignore = ["old/path"]
"#;
        let mut doc = input.parse::<DocumentMut>().unwrap();
        let path = Path::new("/repo/new/secret.txt"); // Variable name casing fix handled by using lower path

        add_ignore_path(&mut doc, path, root).unwrap();

        let output = doc.to_string();
        println!("Output TOML:\n{}", output);

        assert!(output.contains(r#""old/path""#));
        assert!(output.contains(r#""new/secret.txt""#));
        assert!(output.contains("# Pre-existing comment"));

        // Verify structure correctness
        let parsed = output.parse::<DocumentMut>().unwrap();
        let arr = parsed["core"]["ignore"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_add_ignore_path_creates_structure() {
        let root = Path::new("/repo");
        let input = r#"
# Just a comment
version = "1"
"#;
        let mut doc = input.parse::<DocumentMut>().unwrap();
        let path = Path::new("/repo/secret.txt");

        add_ignore_path(&mut doc, path, root).unwrap();

        let output = doc.to_string();
        println!("Output TOML:\n{}", output);

        assert!(output.contains(r#""secret.txt""#));
        assert!(output.contains("[core]"));
    }
}
