use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

use crate::config_loader::load_config_layers;
use veil_config::validate::validate_config;

pub fn doctor() -> Result<()> {
    println!("{}", "Veil Doctor \u{1FA7A}".bold().green());
    println!("{}", "--------------".dimmed());

    // 1. Veil Version
    println!("Veil Version: {}", env!("CARGO_PKG_VERSION").bold());

    // 2. OS Info
    let os_info = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    println!("OS: {}", os_info);

    // 3. Rust Version
    match Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("Rust: {}", version);
        }
        Err(_) => {
            println!("Rust: {}", "(rustc not found)".yellow());
        }
    }

    println!();
    println!("{}", "Configuration Layers:".bold());

    // Load config layers
    let layers = match load_config_layers(None) {
        Ok(l) => l,
        Err(e) => {
            println!("{} Failed to parse config layers: {}", "[ERR]".red(), e);
            return Ok(());
        }
    };

    if layers.org.is_some() {
        println!(
            "{} Org config loaded (VEIL_ORG_CONFIG / org.toml)",
            "[OK]".green()
        );
    } else {
        println!("{} Org config not present", "[-]".dimmed());
    }

    if layers.user.is_some() {
        println!(
            "{} User config loaded (~/.config/veil/veil.toml)",
            "[OK]".green()
        );
    } else {
        println!("{} User config not present", "[-]".dimmed());
    }

    if layers.repo.is_some() {
        println!("{} Repo config loaded (veil.toml)", "[OK]".green());
    } else {
        println!("{} Repo config not present", "[-]".dimmed());
    }

    let effective = &layers.effective;

    println!();
    println!("{}", "Rules Directory:".bold());
    if let Some(rules_dir) = &effective.core.rules_dir {
        let path = PathBuf::from(rules_dir);
        if path.exists() {
            println!("{} Directory exists: {:?}", "[OK]".green(), path);
            let manifest_path = path.join("00_manifest.toml");
            if manifest_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = veil_core::rules::pack::parse_manifest(&content) {
                        println!("   - Manifest version: {}", manifest.pack.version);
                    }
                }
            }
            match veil_core::rules::pack::load_rule_pack(&path) {
                Ok(rules) => {
                    println!("   - Rules loaded: {}", rules.len());
                }
                Err(e) => {
                    println!("{} Failed to load rules: {}", "[ERR]".red(), e);
                }
            }
        } else {
            println!("{} Directory not found: {:?}", "[ERR]".red(), path);
        }
    } else {
        println!(
            "{} No rules_dir configured (using built-in defaults)",
            "[-]".dimmed()
        );
    }

    println!();
    println!("{}", "Scan Boundaries (Effective Config):".bold());
    println!(
        "  max_file_size:  {}",
        effective.core.max_file_size.unwrap_or(500_000_000)
    );
    println!(
        "  max_file_count: {}",
        effective.core.max_file_count.unwrap_or(1_000_000)
    );
    println!(
        "  max_findings:   {}",
        effective.output.max_findings.unwrap_or(std::usize::MAX)
    );

    if let Err(err) = validate_config(effective) {
        println!("{} Validations Failed:", "[ERR]".red());
        println!("   - {}", err);
    } else {
        println!("{} Config limits are valid", "[OK]".green());
    }

    println!();
    println!("{}", "OSV Cache (Guardian):".bold());
    let cache_dir = if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg).join("veil").join("guardian").join("osv")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".cache")
            .join("veil")
            .join("guardian")
            .join("osv")
    } else {
        PathBuf::from(".cache")
            .join("veil")
            .join("guardian")
            .join("osv")
    };

    if cache_dir.exists() {
        let count = std::fs::read_dir(&cache_dir)
            .map(|i| i.count())
            .unwrap_or(0);
        println!("{} Cache directory exists: {:?}", "[OK]".green(), cache_dir);
        println!("   - Cached vulnerability files: {}", count);
    } else {
        println!(
            "{} Cache directory not found: {:?}",
            "[-]".dimmed(),
            cache_dir
        );
    }

    println!();
    println!("{}", "Environment Variables:".bold());
    let env_vars = [
        "VEIL_ORG_CONFIG",
        "VEIL_ORG_RULES",
        "VEIL_USER_CONFIG",
        "VEIL_FAIL_SCORE",
        "VEIL_OSV_FORCE_REFRESH",
        "RUST_LOG",
    ];
    for var in env_vars {
        match std::env::var(var) {
            Ok(val) => println!("{}: {}", var, val),
            Err(_) => println!("{}: {}", var, "(not set)".dimmed()),
        }
    }

    Ok(())
}
