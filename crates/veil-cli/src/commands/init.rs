use anyhow::{Context, Result};
use colored::*;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
            max_file_count: None,
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
    preset: Option<String>,
    pin_tag: String,
) -> Result<()> {
    if let Some(provider) = ci_provider {
        if preset.is_some() {
            anyhow::bail!(
                "--preset cannot be combined with --ci. Run `veil init --preset <preset>` first, then `veil init --ci {}`.",
                provider
            );
        }
        if profile_override.is_some() {
            anyhow::bail!(
                "--profile cannot be combined with --ci. Run `veil init --profile <profile>` first, then `veil init --ci {}`.",
                provider
            );
        }
        if wizard {
            anyhow::bail!(
                "--wizard cannot be combined with --ci. Run `veil init --wizard` first, then `veil init --ci {}`.",
                provider
            );
        }
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
        } else if preset.as_deref() == Some("logs-jp") {
            Profile::Logs
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
        if profile == Profile::Logs || preset.as_deref() == Some("logs-jp") {
            println!(
                "{}",
                "Note: This setup generates a log-focused RulePack under rules/log.".dimmed()
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

    let mut config = build_config(&answers);
    if let Some(preset_id) = preset.as_deref() {
        config = veil_config::apply_builtin_preset_as_base(config, preset_id)?;
        if preset_id == "logs-jp" && config.core.rules_dir.is_none() {
            config.core.rules_dir = Some("rules/log".to_string());
        }
        println!(
            "{}",
            format!(
                "Applied preset '{}' as the base config layer; generated config can override it.",
                preset_id
            )
            .dimmed()
        );
    }
    let toml_str = toml::to_string_pretty(&config)?;

    let path = answers.target_path.as_deref().unwrap_or(path);
    let should_init_log_pack =
        answers.profile == Profile::Logs || preset.as_deref() == Some("logs-jp");
    let log_pack_rules_dir = log_pack_rules_dir_for_config(path);
    let log_pack_created_any = if should_init_log_pack {
        Some(ensure_log_pack_for_init(&log_pack_rules_dir)?)
    } else {
        None
    };

    fs::write(path, toml_str)?;

    // Post-generation for Logs profile or logs preset
    if let Some(created_any) = log_pack_created_any {
        if created_any {
            println!(
                "{}",
                format!(
                    "Log RulePack initialized at {} (wired via core.rules_dir).",
                    log_pack_rules_dir.display()
                )
                .green()
            );
            println!(
                "{}",
                format!(
                    "Tip: In {}/00_manifest.toml, uncomment the `ext = \"aggressive\"` line to enable aggressive rules.",
                    log_pack_rules_dir.display()
                )
                .bright_black()
            );
        } else {
            println!(
                "{}",
                format!(
                    "Directory {} already contains a usable log RulePack.",
                    log_pack_rules_dir.display()
                )
                .yellow()
            );
        }
    }

    println!(
        "{}",
        format!("Successfully created {}", path.display())
            .green()
            .bold()
    );
    if let Some(score) = config.core.fail_on_score {
        println!("Policy: Fail on score >= {}", score);
    }

    Ok(())
}

fn log_pack_rules_dir_for_config(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join("rules/log"))
        .unwrap_or_else(|| PathBuf::from("rules/log"))
}

fn ensure_log_pack_for_init(rules_dir: &Path) -> Result<bool> {
    if !rules_dir.exists() {
        fs::create_dir_all(rules_dir)?;
        println!(
            "{}",
            format!("Created directory {}", rules_dir.display()).green()
        );
    }

    let mut created_any = false;
    for file in LogPackAssets::iter() {
        let file_path = rules_dir.join(file.as_ref());
        if !file_path.exists() {
            if let Some(content) = LogPackAssets::get(file.as_ref()) {
                fs::write(&file_path, content.data.as_ref())?;
                println!("  - Created {}", file.as_ref());
                created_any = true;
            }
        }
    }

    validate_log_pack_for_init(rules_dir)?;

    Ok(created_any)
}

fn validate_log_pack_for_init(rules_dir: &Path) -> Result<()> {
    let rules = veil_core::rules::pack::load_rule_pack(rules_dir).with_context(|| {
        format!(
            "Existing log RulePack at {} is not valid",
            rules_dir.display()
        )
    })?;
    let missing_ids: Vec<_> = veil_config::LOGS_JP_REQUIRED_RULE_IDS
        .iter()
        .copied()
        .filter(|required_id| !rules.iter().any(|rule| rule.id == *required_id))
        .collect();

    if !missing_ids.is_empty() {
        anyhow::bail!(
            "Existing log RulePack at {} is missing required logs-jp rules: {}. Repair or remove {}, then rerun `veil init --preset logs-jp`.",
            rules_dir.display(),
            missing_ids.join(", "),
            rules_dir.display()
        );
    }

    Ok(())
}

#[cfg(any(feature = "wizard", test))]
pub fn infer_wizard_preset(root: &Path) -> Option<&'static str> {
    if has_log_artifact(root) {
        return Some("logs-jp");
    }

    if has_fintech_directory(root) {
        return Some("fintech-jp");
    }

    if readme_contains_japanese(root) {
        return Some("standard-jp");
    }

    None
}

#[cfg(any(feature = "wizard", test))]
fn has_log_artifact(root: &Path) -> bool {
    if root.join("logs").is_dir() {
        return true;
    }

    fs::read_dir(root)
        .map(|entries| {
            entries.flatten().any(|entry| {
                let path = entry.path();
                path.is_file()
                    && path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| matches!(ext, "log" | "jsonl" | "ndjson"))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

#[cfg(any(feature = "wizard", test))]
fn has_fintech_directory(root: &Path) -> bool {
    ["payments", "billing", "kyc", "account"]
        .iter()
        .any(|name| root.join(name).is_dir())
}

#[cfg(any(feature = "wizard", test))]
fn readme_contains_japanese(root: &Path) -> bool {
    fs::read_to_string(root.join("README.md"))
        .map(|content| contains_japanese_text(&content))
        .unwrap_or(false)
}

#[cfg(any(feature = "wizard", test))]
fn contains_japanese_text(content: &str) -> bool {
    content.chars().any(|ch| {
        matches!(
            ch,
            '\u{3040}'..='\u{309f}' | '\u{30a0}'..='\u{30ff}' | '\u{ff66}'..='\u{ff9f}'
        )
    })
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
    if let Some(preset_id) = infer_wizard_preset(Path::new(".")) {
        let message = match lang_selection {
            Language::En => format!("Detected preset candidate: {}", preset_id),
            Language::Ja => format!("推奨プリセット候補: {}", preset_id),
        };
        println!("{}", message.dimmed());
    }

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

    #[test]
    fn log_pack_rules_dir_follows_config_parent() {
        assert_eq!(
            log_pack_rules_dir_for_config(Path::new("veil.toml")),
            PathBuf::from("rules/log")
        );
        assert_eq!(
            log_pack_rules_dir_for_config(Path::new("configs/veil.logs.toml")),
            PathBuf::from("configs/rules/log")
        );
    }

    #[test]
    fn infer_wizard_preset_prefers_logs_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("logs")).unwrap();
        fs::create_dir(dir.path().join("payments")).unwrap();

        assert_eq!(infer_wizard_preset(dir.path()), Some("logs-jp"));
    }

    #[test]
    fn infer_wizard_preset_detects_root_log_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("audit.jsonl"), "{}\n").unwrap();

        assert_eq!(infer_wizard_preset(dir.path()), Some("logs-jp"));
    }

    #[test]
    fn infer_wizard_preset_detects_fintech_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("kyc")).unwrap();

        assert_eq!(infer_wizard_preset(dir.path()), Some("fintech-jp"));
    }

    #[test]
    fn infer_wizard_preset_detects_japanese_readme() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "これは日本語のREADMEです。\n").unwrap();

        assert_eq!(infer_wizard_preset(dir.path()), Some("standard-jp"));
    }

    #[test]
    fn infer_wizard_preset_returns_none_without_signals() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(infer_wizard_preset(dir.path()), None);
    }
}
