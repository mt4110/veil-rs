use anyhow::Result;
use colored::*;
use std::env;
use std::path::Path;
use veil_config::Config;
use veil_core::{get_default_rules, scan_path};

pub fn check_project() -> Result<bool> {
    let current_dir = env::current_dir()?;
    let mut score = 0;
    let mut details = Vec::new();

    println!("{}", "Running Project Health Checks...".bold().blue());
    println!(
        "{}",
        "------------------------------------------------".blue()
    );

    // 1. .gitignore (20pts)
    if Path::new(".gitignore").exists() {
        score += 20;
        println!("{} .gitignore found (+20)", "PASS".green().bold());
    } else {
        println!("{} Missing .gitignore file (0/20)", "FAIL".red().bold());
        details.push("Create a .gitignore file to exclude sensitive files".to_string());
    }

    // 2. Secrets Scan (40pts)
    // We strictly fail if CRITICAL/HIGH secrets found
    // If no secrets: +40
    println!("{} Scanning for secrets...", "INFO".blue());
    let rules = get_default_rules();
    let config = Config::default();
    let findings = scan_path(&current_dir, &rules, &config);

    if findings.is_empty() {
        score += 40;
        println!("{} No obvious secrets found (+40)", "PASS".green().bold());
    } else {
        println!(
            "{} Found {} potentially unsafe secrets (0/40)",
            "FAIL".red().bold(),
            findings.len()
        );
        details.push("Run `veil scan` to identify and remove secrets".to_string());
    }

    // 3. CI Config (20pts)
    let ci_path = Path::new(".github/workflows"); // GitHub Actions
    let gitlab_ci = Path::new(".gitlab-ci.yml");
    if (ci_path.exists() && ci_path.read_dir()?.count() > 0) || gitlab_ci.exists() {
        score += 20;
        println!("{} CI Configuration found (+20)", "PASS".green().bold());
    } else {
        println!(
            "{} No CI configuration found (0/20)",
            "WARN".yellow().bold()
        );
        details.push("Setup GitHub Actions or GitLab CI".to_string());
    }

    // 4. Pre-commit (10pts)
    if Path::new(".pre-commit-config.yaml").exists() {
        score += 10;
        println!("{} Pre-commit config found (+10)", "PASS".green().bold());
    } else {
        println!(
            "{} No pre-commit config found (0/10)",
            "WARN".yellow().bold()
        );
        details.push("Use pre-commit hooks to prevent secrets from being committed".to_string());
    }

    // 5. Documentation (README/License) (10pts)
    let has_readme = Path::new("README.md").exists();
    let license_files = [
        "LICENSE",
        "LICENSE.txt",
        "LICENSE.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
    ];
    let has_license = license_files.iter().any(|f| Path::new(f).exists());

    if has_readme && has_license {
        score += 10;
        println!("{} README & License found (+10)", "PASS".green().bold());
    } else if has_readme || has_license {
        score += 5;
        println!(
            "{} Partial documentation found (+5)",
            "WARN".yellow().bold()
        );
        details.push("Ensure both README.md and LICENSE exist for open source health".to_string());
    } else {
        println!(
            "{} Missing README and LICENSE (0/10)",
            "WARN".yellow().bold()
        );
        details.push("Add README.md and LICENSE".to_string());
    }

    println!(
        "{}",
        "------------------------------------------------".blue()
    );

    // Grading
    let grade = match score {
        90..=100 => "A",
        70..=89 => "B",
        50..=69 => "C",
        _ => "F",
    };

    let color = match grade {
        "A" => "green",
        "B" => "blue",
        "C" => "yellow",
        _ => "red",
    };

    println!(
        "Project Health Score: {}/100",
        score.to_string().color(color).bold()
    );
    println!("Grade: {}", grade.color(color).bold());

    if !details.is_empty() {
        println!("\nRecommendations:");
        for (i, msg) in details.iter().enumerate() {
            println!("{}. {}", i + 1, msg);
        }
    }

    // Return checks: currently we return true/false based on strict failures?
    // Or based on Grade? Let's say Grade F is failure (exit 1), others success.
    if grade == "F" {
        Ok(false)
    } else {
        Ok(true)
    }
}
