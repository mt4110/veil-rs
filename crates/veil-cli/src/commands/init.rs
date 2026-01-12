use anyhow::Result;
use colored::*;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use veil_config::config::{Config, CoreConfig, MaskingConfig};

#[derive(RustEmbed)]
#[folder = "../veil/rules/log/"]
struct LogPackAssets;

#[cfg(feature = "wizard")]
use inquire::{Confirm, Select, Text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Ja,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::En => write!(f, "English"),
            Language::Ja => write!(f, "日本語 (Japanese)"),
        }
    }
}

/// Profile defines preset configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Application,
    Library,
    Logs,
}

impl Profile {
    fn display(&self, lang: Language) -> String {
        match lang {
            Language::En => match self {
                Profile::Application => {
                    "Application (Standard security - Balanced checks)".to_string()
                }
                Profile::Library => "Library (Strict compliance - High sensitivity)".to_string(),
                Profile::Logs => "Logs (Log scrubbing only - Minimal context)".to_string(),
            },
            Language::Ja => match self {
                Profile::Application => "Application (標準セキュリティ - バランス重視)".to_string(),
                Profile::Library => "Library (厳格コンプライアンス - 外部公開用など)".to_string(),
                Profile::Logs => "Logs (ログ秘匿のみ - ソースコードは対象外)".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Goal {
    CreateNew,
    CreateSeparate,
    Inspect,
    Exit,
}

impl Goal {
    fn display(&self, lang: Language) -> String {
        match lang {
            Language::En => match self {
                Goal::CreateNew => "Create new configuration (Overwrite)".to_string(),
                Goal::CreateSeparate => "Create new configuration (Separate file)".to_string(),
                Goal::Inspect => "Inspect existing configuration".to_string(),
                Goal::Exit => "Exit".to_string(),
            },
            Language::Ja => match self {
                Goal::CreateNew => "新しく設定を作成する (上書き)".to_string(),
                Goal::CreateSeparate => "別の設定ファイルを作成する".to_string(),
                Goal::Inspect => "既存の設定を確認する".to_string(),
                Goal::Exit => "終了".to_string(),
            },
        }
    }
}

// Wrapper for Select to use dynamic display based on Language is hard with Inquire's generic types directly
// unless we implement Display.
// Instead, we can use strings for selection and map back.

pub struct InitAnswers {
    pub profile: Profile,
    #[allow(dead_code)]
    pub languages: Vec<String>,
    pub fail_score: Option<u32>,
    pub remote_rules_url: Option<String>,
    pub ignore_test_data: bool,
    pub ci_strategy: Option<CiStrategy>,
    pub target_path: Option<std::path::PathBuf>,
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
            ignore.push("src".to_string());
            None
        }
    };

    let rules_dir = if matches!(answers.profile, Profile::Logs) {
        Some("rules/log".to_string())
    } else {
        None
    };

    let placeholder = if matches!(answers.profile, Profile::Logs) {
        Some("<REDACTED:PII>".to_string())
    } else {
        None
    };

    let derived_score = match answers.ci_strategy {
        Some(CiStrategy::FailHigh) => Some(70),
        Some(CiStrategy::MonitorOnly) => None,
        None => fail_score,
    };

    let final_fail_score = answers.fail_score.or(derived_score);

    Config {
        core: CoreConfig {
            include: vec![".".to_string()],
            ignore,
            max_file_size: None,
            fail_on_score: final_fail_score,
            remote_rules_url: answers.remote_rules_url.clone(),
            rules_dir,
        },
        masking: MaskingConfig {
            placeholder: placeholder.unwrap_or_else(|| MaskingConfig::default().placeholder),
        },
        output: veil_config::OutputConfig::default(),
        rules,
    }
}

pub fn init(
    wizard: bool,
    non_interactive: bool,
    ci_provider: Option<String>,
    profile_override: Option<String>,
    pin_tag: String,
) -> Result<()> {
    if let Some(provider) = ci_provider {
        return generate_ci_template(&provider, &pin_tag);
    }

    let path = Path::new("veil.toml");
    let file_exists = path.exists();

    // Check existence for non-interactive mode
    if file_exists && non_interactive {
        anyhow::bail!("veil.toml already exists! (non-interactive mode)");
    }

    let answers = if wizard {
        #[cfg(feature = "wizard")]
        {
            // If wizard, we handle existence check inside to ask for overwrite (localized)
            if let Some(ans) = run_wizard(file_exists)? {
                ans
            } else {
                return Ok(());
            }
        }
        #[cfg(not(feature = "wizard"))]
        {
            anyhow::bail!("feature 'wizard' is not enabled in this build.")
        }
    } else {
        if file_exists {
            anyhow::bail!("veil.toml already exists! Use --wizard to reconfigure.");
        }
        // Default non-interactive defaults
        let profile = if let Some(p_str) = profile_override.as_deref() {
            match p_str.to_lowercase().as_str() {
                "logs" | "log" => Profile::Logs,
                "library" | "lib" => Profile::Library,
                "application" | "app" => Profile::Application,
                _ => Profile::Application,
            }
        } else {
            Profile::Application
        };

        let profile_label = match profile {
            Profile::Application => "Application",
            Profile::Library => "Library",
            Profile::Logs => "Logs",
        };
        println!(
            "{}",
            format!(
                "Generating default configuration ({} profile)...",
                profile_label
            )
            .blue()
        );
        println!(
            "{}",
            "Tip: Run `veil init --wizard` for an interactive setup (recommended).".dimmed()
        );
        if profile == Profile::Logs {
            println!(
                "{}",
                "Note: This profile generates a log-focused RulePack under rules/log.".dimmed()
            );
        }

        InitAnswers {
            profile,
            languages: vec![],
            fail_score: None,
            remote_rules_url: None,
            ignore_test_data: false,
            ci_strategy: None,
            target_path: None,
        }
    };

    let config = build_config(&answers);
    let toml_str = toml::to_string_pretty(&config)?;

    let path = answers.target_path.as_deref().unwrap_or(path);
    fs::write(path, toml_str)?;

    println!(
        "{}",
        format!("Successfully created {}", path.display())
            .green()
            .bold()
    );
    if let Some(score) = config.core.fail_on_score {
        println!("Policy: Fail on score >= {}", score);
    }

    // Post-generation for Logs profile
    if answers.profile == Profile::Logs {
        let rules_dir = Path::new("rules/log");
        if !rules_dir.exists() {
            fs::create_dir_all(rules_dir)?;
            println!(
                "{}",
                format!("Created directory {}", rules_dir.display()).green()
            );
            let mut created_any = false;
            for file in LogPackAssets::iter() {
                let file_path = rules_dir.join(file.as_ref());
                if let Some(content) = LogPackAssets::get(file.as_ref()) {
                    fs::write(&file_path, content.data.as_ref())?;
                    println!("  - Created {}", file.as_ref());
                    created_any = true;
                }
            }

            if created_any {
                println!(
                    "{}",
                    "Log RulePack initialized at rules/log (wired via core.rules_dir).".green()
                );
                println!(
                    "{}",
                    "Tip: In rules/log/00_manifest.toml, uncomment the `ext = \"aggressive\"` line to enable aggressive rules.".bright_black()
                );
            }
        } else {
            println!(
                "{}",
                format!(
                    "Directory {} already exists, skipping rule pack generation.",
                    rules_dir.display()
                )
                .yellow()
            );
        }
    }
    Ok(())
}

#[cfg(feature = "wizard")]
fn run_wizard(file_exists: bool) -> Result<Option<InitAnswers>> {
    // 0. Language Selection
    let langs = vec![Language::En, Language::Ja];
    let lang_selection = Select::new("Choose Language / 言語を選択:", langs)
        .with_help_message("Use ↑/↓ and Enter")
        .prompt()?;

    // Helper for texts
    let t = |key: &str| -> String {
        match (lang_selection, key) {
            (Language::En, "welcome") => "Welcome to Veil Init Wizard".to_string(),
            (Language::Ja, "welcome") => "Veil セットアップウィザードへようこそ".to_string(),

            (Language::En, "goal_q") => "What would you like to do?".to_string(),
            (Language::Ja, "goal_q") => "何をしたいですか？".to_string(),

            (Language::En, "found_existing") => "Found existing configuration.".to_string(),
            (Language::Ja, "found_existing") => "既存の設定が見つかりました。".to_string(),

            (Language::En, "filename_q") => "Enter new filename (e.g. veil.dev.toml):".to_string(),
            (Language::Ja, "filename_q") => {
                "新しいファイル名を入力してください (例: veil.dev.toml):".to_string()
            }

            (Language::En, "inspect_header") => "--- Current Configuration ---".to_string(),
            (Language::Ja, "inspect_header") => "--- 現在の設定内容 ---".to_string(),

            (Language::En, "abort") => "Aborted.".to_string(),
            (Language::Ja, "abort") => "中断しました。".to_string(),

            (Language::En, "profile_q") => "Choose a project profile:".to_string(),
            (Language::Ja, "profile_q") => {
                "プロジェクトのプロファイルを選択してください:".to_string()
            }

            (Language::En, "profile_help") => {
                "Use ↑/↓, Enter to select. Esc to go back (if supported).".to_string()
            }
            (Language::Ja, "profile_help") => {
                "↑/↓で移動, Enterで決定. Escで戻る(可能な場合)".to_string()
            }

            (Language::En, "remote_q") => "Configure remote rules (veil-server)?".to_string(),
            (Language::Ja, "remote_q") => {
                "リモート詳細ルール (veil-server) を設定しますか？".to_string()
            }

            (Language::En, "remote_help") => {
                "If you use a centralized Veil Server for rule management.".to_string()
            }
            (Language::Ja, "remote_help") => {
                "組織で管理されたルールサーバーを利用する場合に選択します。".to_string()
            }

            (Language::En, "remote_url_q") => "Remote Rules URL:".to_string(),
            (Language::Ja, "remote_url_q") => "リモート・ルール・サーバーのURL:".to_string(),

            (Language::En, "https_warn") => {
                "Warning: Only HTTPS URLs are recommended for security.".to_string()
            }
            (Language::Ja, "https_warn") => {
                "警告: セキュリティのためHTTPSの使用を推奨します。".to_string()
            }

            (Language::En, "test_data_q") => {
                "Ignore potential test data folders (tests, spec)?".to_string()
            }
            (Language::Ja, "test_data_q") => {
                "テスト用データフォルダ (tests, spec等) を無視しますか？".to_string()
            }

            (Language::En, "test_data_help") => {
                "Prevents false positives from fake secrets in tests.".to_string()
            }
            (Language::Ja, "test_data_help") => {
                "テストコード内のダミー鍵による誤検知を防ぎます。".to_string()
            }

            (Language::En, "ci_q") => "CI/CD Failure Strategy:".to_string(),
            (Language::Ja, "ci_q") => "CI/CDでの検出時の動作設定:".to_string(),

            (Language::En, "ci_fail") => "Fail on High/Critical (Recommended)".to_string(),
            (Language::Ja, "ci_fail") => "High/Criticalでジョブを失敗させる (推奨)".to_string(),

            (Language::En, "ci_monitor") => "Monitor Only (Report but don't fail)".to_string(),
            (Language::Ja, "ci_monitor") => "モニターのみ (報告するが失敗させない)".to_string(),

            _ => key.to_string(),
        }
    };

    println!("{}", t("welcome").bold().cyan());

    let mut target_path = None;

    // 1. Goal Selection (if file exists)
    if file_exists {
        println!("{}", t("found_existing").blue());

        let goals = vec![
            Goal::CreateNew,
            Goal::CreateSeparate,
            Goal::Inspect,
            Goal::Exit,
        ];
        let goal_options: Vec<String> = goals.iter().map(|g| g.display(lang_selection)).collect();

        let selected_goal_str = Select::new(&t("goal_q"), goal_options).prompt()?;
        let selected_goal = goals
            .into_iter()
            .find(|g| g.display(lang_selection) == selected_goal_str)
            .expect("Selected goal match");

        match selected_goal {
            Goal::Exit => {
                println!("{}", t("abort"));
                return Ok(None);
            }
            Goal::Inspect => {
                println!("{}", t("inspect_header").bold());
                if let Ok(content) = fs::read_to_string("veil.toml") {
                    println!("{}", content);
                } else {
                    println!("(Error reading file)");
                }
                return Ok(None);
            }
            Goal::CreateSeparate => {
                let filename = Text::new(&t("filename_q")).prompt()?;
                target_path = Some(std::path::PathBuf::from(filename));
            }
            Goal::CreateNew => {
                // Proceed to generation (overwrite implied by target_path = None which means default)
            }
        }
    }

    // ... Continues below for generation ...
    // Since we need to share the generation flow (profile selection etc) between "No file" and "CreateNew" and "CreateSeparate",
    // we should restructure.

    // START GENERATION FLOW

    // 2. Profile
    let profiles = vec![Profile::Application, Profile::Library, Profile::Logs];
    let profile_options: Vec<String> = profiles.iter().map(|p| p.display(lang_selection)).collect();

    let selected_profile_str = Select::new(&t("profile_q"), profile_options)
        .with_help_message(&t("profile_help"))
        .prompt()?;

    let profile = profiles
        .into_iter()
        .find(|p| p.display(lang_selection) == selected_profile_str)
        .expect("Selected profile matches one option");

    // 3. Remote Rules
    let use_remote = Confirm::new(&t("remote_q"))
        .with_default(false)
        .with_help_message(&t("remote_help"))
        .prompt()?;

    let remote_rules_url = if use_remote {
        let url = Text::new(&t("remote_url_q")).prompt()?.trim().to_string();
        if !url.starts_with("https://") {
            eprintln!("{}", t("https_warn").yellow());
        }
        Some(url)
    } else {
        None
    };

    // 4. Test Data
    let ignore_test_data = Confirm::new(&t("test_data_q"))
        .with_default(true)
        .with_help_message(&t("test_data_help"))
        .prompt()?;

    // 5. CI Strategy
    let ci_fail_str = t("ci_fail");
    let ci_monitor_str = t("ci_monitor");
    let ci_options = vec![ci_fail_str.clone(), ci_monitor_str];

    let ci_choice = Select::new(&t("ci_q"), ci_options).prompt()?;

    let ci_strategy = if ci_choice == ci_fail_str {
        Some(CiStrategy::FailHigh)
    } else {
        Some(CiStrategy::MonitorOnly)
    };

    // 2. Languages (Auto-detect)
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

    Ok(Some(InitAnswers {
        profile,
        languages: distinct_exts,
        fail_score: None,
        remote_rules_url,
        ignore_test_data,
        ci_strategy,
        target_path,
    }))
}

fn generate_ci_template(provider: &str, pin_tag: &str) -> Result<()> {
    match provider.to_lowercase().as_str() {
        "github" | "gh" | "actions" => {
            let dir_path = Path::new(".github/workflows");
            let file_path = dir_path.join("veil.yml");

            if file_path.exists() {
                anyhow::bail!("{} already exists!", file_path.display());
            }

            fs::create_dir_all(dir_path)?;

            let build_version = env!("CARGO_PKG_VERSION");
            let tag_opt = resolve_pin_tag(pin_tag, build_version)?;

            let mut content = include_str!("../templates/ci_github.yml")
                .replace("{{VEIL_VERSION}}", build_version);

            if let Some(tag) = tag_opt {
                content = content.replace("__VEIL_TAG__", &tag);
            } else {
                content = content.replace(" --tag __VEIL_TAG__", "");
            }

            fs::write(&file_path, content)?;
            println!(
                "{}",
                format!(
                    "Generated GitHub Actions workflow at {}",
                    file_path.display()
                )
                .green()
            );

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

fn resolve_pin_tag(pin_tag: &str, build_version: &str) -> Result<Option<String>> {
    match pin_tag {
        "auto" => {
            if is_prerelease(build_version) {
                println!(
                    "{}",
                    format!(
                        "Warning: Current version '{}' is a prerelease/dev build.",
                        build_version
                    )
                    .yellow()
                );
                println!(
                    "{}",
                    "Skipping version pinning for CI workflow to avoid install errors."
                        .yellow()
                );
                println!(
                    "{}",
                    "To pin a specific version, use --pin-tag vX.Y.Z".dimmed()
                );
                Ok(None)
            } else {
                Ok(Some(format!("v{}", build_version)))
            }
        }
        "none" => Ok(None),
        val if val.starts_with('v') => Ok(Some(val.to_string())),
        _ => anyhow::bail!("Invalid --pin-tag value: '{}'. Must be 'auto', 'none', or start with 'v' (e.g. v1.0.0)", pin_tag),
    }
}

fn is_prerelease(version: &str) -> bool {
    version.contains('-') || version.contains('+')
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
            target_path: None,
        };
        let config = build_config(&answers);
        assert_eq!(config.core.fail_on_score, Some(80)); // App default
        assert!(!config.core.ignore.contains(&"tests".to_string()));
    }
}
