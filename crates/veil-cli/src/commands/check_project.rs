use anyhow::Result;
use colored::*;
use std::env;
use std::path::Path;
use veil_config::Config;
use veil_core::{get_default_rules, scan_path};

pub fn check_project() -> Result<bool> {
    let current_dir = env::current_dir()?;
    let mut failures = 0;
    let mut warnings = 0;

    println!("{}", "Running Project Health Checks...".bold().blue());

    // 1. Mandatory Check: .gitignore
    if !Path::new(".gitignore").exists() {
        println!("{} Missing .gitignore file", "FAIL".red().bold());
        failures += 1;
    } else {
        println!("{} .gitignore found", "PASS".green().bold());
    }

    // 2. Mandatory Check: License
    let license_files = [
        "LICENSE",
        "LICENSE.txt",
        "LICENSE.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
    ];
    let has_license = license_files.iter().any(|f| Path::new(f).exists());
    if !has_license {
        println!("{} Missing LICENSE file", "FAIL".red().bold());
        failures += 1;
    } else {
        println!("{} LICENSE found", "PASS".green().bold());
    }

    // 3. Mandatory Check: Obvious hardcoded secrets
    // We use default rules for a quick scan of the current directory (non-recursive or limited depth ideally, but full scan for now)
    println!("{} Scanning for secrets...", "INFO".blue());
    let rules = get_default_rules();
    // Use default config, assuming we check the project root
    let config = Config::default();

    // We should strictly fail if CRITICAL/HIGH secrets found
    let findings = scan_path(&current_dir, &rules, &config);
    if !findings.is_empty() {
        println!(
            "{} Found potentially unsafe secrets in codebase",
            "FAIL".red().bold()
        );
        // failure count is not just 1, but let's just mark it as failure condition
        failures += 1;
        // Maybe print a summary?
        println!(
            "   Found {} issues. Run `veil scan` for details.",
            findings.len()
        );
    } else {
        println!("{} No obvious secrets found", "PASS".green().bold());
    }

    // 4. Optional Check: README
    if !Path::new("README.md").exists() {
        println!("{} Missing README.md", "WARN".yellow().bold());
        warnings += 1;
    } else {
        // Optional: Check size?
        println!("{} README.md found", "PASS".green().bold());
    }

    // 5. Optional Check: CI Workflows
    let ci_path = Path::new(".github/workflows");
    if !ci_path.exists() || ci_path.read_dir()?.count() == 0 {
        println!(
            "{} No CI workflows found in .github/workflows",
            "WARN".yellow().bold()
        );
        warnings += 1;
    } else {
        println!("{} CI workflows found", "PASS".green().bold());
    }

    println!("\nSummary: {} failures, {} warnings", failures, warnings);

    if failures > 0 {
        Ok(false) // Failed
    } else {
        Ok(true) // Passed
    }
}
