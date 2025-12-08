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

    // Override if user provided specific score (though our wizard strictly sets defaults currently,
    // we could allow override in answers)
    let final_fail_score = answers.fail_score.or(fail_score);

    Config {
        core: CoreConfig {
            include: vec![".".to_string()],
            ignore,
            max_file_size: None,
            fail_on_score: final_fail_score,
            remote_rules_url: answers.remote_rules_url.clone(),
        },
        masking: MaskingConfig::default(),
        rules,
    }
}

pub fn init(wizard: bool, non_interactive: bool) -> Result<()> {
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
                "veil.toml already exists! Use --wizard to reconfiguration or delete it manually."
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
        Some(Text::new("Remote Rules URL:").prompt()?)
    } else {
        None
    };

    Ok(InitAnswers {
        profile,
        languages: distinct_exts,
        fail_score: None, // Will use profile default
        remote_rules_url,
    })
}
