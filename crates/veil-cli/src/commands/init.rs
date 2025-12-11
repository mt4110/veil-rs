use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use veil_config::config::{Config, CoreConfig, MaskingConfig};

#[cfg(feature = "wizard")]
use inquire::{Confirm, Select, Text};

/// Profile defines preset configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Application,
    Library,
    Logs,
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Application => write!(f, "Application (Standard security)"),
            Profile::Library => write!(f, "Library (Strict compliance)"),
            Profile::Logs => write!(f, "Logs (Log scrubbing only)"),
        }
    }
}

pub struct InitAnswers {
    pub profile: Profile,
    #[allow(dead_code)]
    pub languages: Vec<String>,
    pub fail_score: Option<u32>,
    pub remote_rules_url: Option<String>,
    pub ignore_test_data: bool,
    pub ci_strategy: Option<CiStrategy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiStrategy {
    FailHigh,
    MonitorOnly,
}

/// Pure logic to build Config from Answers
pub fn build_config(answers: &InitAnswers) -> Config {
    let mut ignore = vec![
        "target".to_string(),
        ".git".to_string(),
        "node_modules".to_string(),
        "vendor".to_string(),
        "dist".to_string(),
        "build".to_string(),
    ];

    if answers.ignore_test_data {
        ignore.push("tests".to_string());
        ignore.push("test".to_string());
        ignore.push("spec".to_string());
    }

    let rules = HashMap::new();

    let fail_score = match answers.profile {
        Profile::Application => Some(80),
        Profile::Library => Some(70),
        Profile::Logs => {
            // Logs profile ignores source directories, focuses on log files if possible
            // For now just add standard source dirs to ignore?
            ignore.push("src".to_string());
            None // Usually don't fail logs scanning unless critical?
        }
    };

    // If CI Strategy is Explicitly set, it takes precedence for the "default" expectation
    // But fail_on_score in TOML usually controls scan exit code directly
    let _fail_on_severity = match answers.ci_strategy {
        Some(CiStrategy::FailHigh) => Some(veil_core::Severity::High),
        Some(CiStrategy::MonitorOnly) => None,
        None => None,
        // Note: fail_on_severity is usually a CLI flag, but can be in config?
        // Wait, Config struct has core.fail_on_score, but not fail_on_severity in CoreConfig yet?
        // Let's check CoreConfig definition.
        // Assuming we only touch fail_on_score for now if CoreConfig doesn't support severity.
        // If user wants FailHigh, maybe we set fail_on_score to 70 (High)?
    };

    // Strategy: If FailHigh -> Score 70. If MonitorOnly -> Score 0?
    // Actually, MonitorOnly means we don't fail. So explicitly set fail_on_score = 0? (Wait 0 means fail on everything usually? No, fail_on_score in veil is threshold. 0 means fail on >=0?
    // Let's check cli scan default.
    // If we want "Monitor Only", we should set fail_on_score to None or 101.

    let derived_score = match answers.ci_strategy {
        Some(CiStrategy::FailHigh) => Some(70), // High = 70+
        Some(CiStrategy::MonitorOnly) => None,
        None => fail_score,
    };

    // Override if user provided specific score (though our wizard strictly sets defaults currently,
    // we could allow override in answers)
    let final_fail_score = answers.fail_score.or(derived_score);

    Config {
        core: CoreConfig {
            include: vec![".".to_string()],
            ignore,
            max_file_size: None,
            fail_on_score: final_fail_score,
            remote_rules_url: answers.remote_rules_url.clone(),
        },
        masking: MaskingConfig::default(),
        output: veil_config::OutputConfig::default(),
        rules,
    }
}

pub fn init(wizard: bool, non_interactive: bool, ci_provider: Option<String>) -> Result<()> {
    if let Some(provider) = ci_provider {
        return generate_ci_template(&provider);
    }

    let path = Path::new("veil.toml");

    // Check existence
    if path.exists() {
        if non_interactive {
            anyhow::bail!("veil.toml already exists! (non-interactive mode)");
        }
        println!("{}", "veil.toml already exists.".yellow());
        // If wizard, we ask permission. If simple init, we fail or ask?
        // Current simple init fails.
        if !wizard {
            anyhow::bail!(
                "veil.toml already exists! Use --wizard to reconfigure or delete it manually."
            );
        }
    }

    let answers = if wizard {
        #[cfg(feature = "wizard")]
        {
            run_wizard()?
        }
        #[cfg(not(feature = "wizard"))]
        {
            anyhow::bail!("feature 'wizard' is not enabled in this build.")
        }
    } else {
        // Default non-interactive defaults
        println!(
            "{}",
            "Generating default configuration (Application profile)...".blue()
        );
        println!(
            "{}",
            "Tip: Run `veil init --wizard` for an interactive setup.".dimmed()
        );
        InitAnswers {
            profile: Profile::Application,
            languages: vec![],
            fail_score: Some(80),
            remote_rules_url: None,
            ignore_test_data: false,
            ci_strategy: None,
        }
    };

    let config = build_config(&answers);

    // In wizard mode, if file exists, ask overwrite
    if path.exists() && wizard {
        #[cfg(feature = "wizard")]
        {
            let overwrite = Confirm::new("veil.toml exists. Overwrite?")
                .with_default(false)
                .prompt()?;
            if !overwrite {
                println!("Aborted.");
                return Ok(());
            }
        }
    }

    let toml_str = toml::to_string_pretty(&config)?;
    fs::write(path, toml_str)?;

    println!("{}", "Successfully created veil.toml".green().bold());
    if let Some(score) = config.core.fail_on_score {
        println!("Policy: Fail on score >= {}", score);
    }
    Ok(())
}

#[cfg(feature = "wizard")]
fn run_wizard() -> Result<InitAnswers> {
    println!("{}", "Welcome to Veil Init Wizard".bold().cyan());

    // 1. Profile
    let options = vec![Profile::Application, Profile::Library, Profile::Logs];
    let profile = Select::new("Choose a project profile:", options).prompt()?;

    // 2. Languages (Auto-detect?)
    // A simple heuristic
    let mut distinct_exts = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                if !distinct_exts.contains(&ext.to_string())
                    && ["rs", "ts", "js", "go", "py", "php", "java", "c"].contains(&ext)
                {
                    distinct_exts.push(ext.to_string());
                }
            }
        }
    }
    let detected_msg = if distinct_exts.is_empty() {
        "(None detected)".to_string()
    } else {
        format!("(Detected: {})", distinct_exts.join(", "))
    };
    println!("{}", detected_msg.dimmed());

    // Future: Use these to add specific ignores or rules
    // For now we just ack them.

    // 3. Remote Rules
    let use_remote = Confirm::new("Configure remote rules (veil-server)?")
        .with_default(false)
        .with_help_message("If you use a centralized Veil Server for rule management.")
        .prompt()?;

    let remote_rules_url = if use_remote {
        let url = Text::new("Remote Rules URL:").prompt()?.trim().to_string();
        if !url.starts_with("https://") {
            eprintln!(
                "{}",
                "Warning: Only HTTPS URLs are recommended for security.".yellow()
            );
        }
        Some(url)
    } else {
        None
    };

    // 4. Test Data
    let ignore_test_data = Confirm::new("Ignore potential test data folders (tests, spec)?")
        .with_default(true)
        .with_help_message("Prevents false positives from fake secrets in tests.")
        .prompt()?;

    // 5. CI Strategy
    let ci_options = vec![
        "Fail on High/Critical (Recommended)",
        "Monitor Only (Report but don't fail)",
    ];
    let ci_choice = Select::new("CI/CD Failure Strategy:", ci_options.clone()).prompt()?;

    let ci_strategy = if ci_choice == ci_options[0] {
        Some(CiStrategy::FailHigh)
    } else {
        Some(CiStrategy::MonitorOnly)
    };

    Ok(InitAnswers {
        profile,
        languages: distinct_exts,
        fail_score: None, // Will use profile default -> now controlled by ci_strategy
        remote_rules_url,
        ignore_test_data,
        ci_strategy,
    })
}

fn generate_ci_template(provider: &str) -> Result<()> {
    match provider.to_lowercase().as_str() {
        "github" | "gh" | "actions" => {
            let dir_path = Path::new(".github/workflows");
            let file_path = dir_path.join("veil.yml");

            if file_path.exists() {
                anyhow::bail!("{} already exists!", file_path.display());
            }

            fs::create_dir_all(dir_path)?;

            let content = r#"name: Veil Security Scan

on:
  push:
    branches: [ "main", "develop" ]
  pull_request:
    branches: [ "main", "develop" ]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      # Use the install script to get the latest version
      - name: Install Veil
        run: |
          curl -sSfL https://get.veil.sh | sh
          echo "$HOME/.cargo/bin" >> $GITHUB_PATH

      # Run scan. 
      # --deny-severity High ensures we block the build on significant secrets.
      # --format html > report.html generates an artifact for review.
      - name: Veil Scan
        run: |
          veil scan . --format html > veil-report.html
          # Also run check for exit code (the above pipe might mask it unless using set -o pipefail)
          # We can do it in one go if we tee, or run twice (fast). 
          # Let's trust veil's exit code with output.
          # Ideally: veil scan . --format html --output-file veil-report.html --fail-on-severity High
          # But v0.8.0 doesn't support --output-file yet (it uses stdout).
          
      - name: Fail on High Severity
        run: |
          veil scan . --fail-on-severity High --no-color --format text

      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: veil-security-report
          path: veil-report.html
"#;
            fs::write(&file_path, content)?;
            println!(
                "{}",
                format!(
                    "Generated GitHub Actions workflow at {}",
                    file_path.display()
                )
                .green()
            );

            // Suggest pre-commit
            println!(
                "\n{}",
                "Tip: You can also verify secrets locally with a pre-commit hook.".dimmed()
            );
            println!("See: https://github.com/mt4110/veil-rs#pre-commit-hook");

            Ok(())
        }
        _ => anyhow::bail!(
            "Unsupported CI provider: {}. Currently only 'github' is supported.",
            provider
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_standard() {
        let answers = InitAnswers {
            profile: Profile::Application,
            languages: vec![],
            fail_score: None,
            remote_rules_url: None,
            ignore_test_data: false,
            ci_strategy: None,
        };
        let config = build_config(&answers);
        assert_eq!(config.core.fail_on_score, Some(80)); // App default
        assert!(!config.core.ignore.contains(&"tests".to_string()));
    }

    #[test]
    fn test_build_config_ignore_tests() {
        let answers = InitAnswers {
            profile: Profile::Application,
            languages: vec![],
            fail_score: None,
            remote_rules_url: None,
            ignore_test_data: true,
            ci_strategy: None,
        };
        let config = build_config(&answers);
        assert!(config.core.ignore.contains(&"tests".to_string()));
    }

    #[test]
    fn test_build_config_ci_monitor_only() {
        let answers = InitAnswers {
            profile: Profile::Application,
            languages: vec![],
            fail_score: None,
            remote_rules_url: None,
            ignore_test_data: false,
            ci_strategy: Some(CiStrategy::MonitorOnly),
        };
        let config = build_config(&answers);
        // MonitorOnly -> derive score None (so doesn't use 80)
        assert_eq!(config.core.fail_on_score, None);
    }

    #[test]
    fn test_build_config_ci_fail_high() {
        let answers = InitAnswers {
            // Profile library default is 70
            profile: Profile::Library,
            languages: vec![],
            fail_score: None,
            remote_rules_url: None,
            ignore_test_data: false,
            // Strategy FailHigh -> 70
            ci_strategy: Some(CiStrategy::FailHigh),
        };
        let config = build_config(&answers);
        assert_eq!(config.core.fail_on_score, Some(70));
    }
}
