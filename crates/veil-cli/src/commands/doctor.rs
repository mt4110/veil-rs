use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;
use veil_config::load_config;

pub fn doctor() -> Result<()> {
    println!("{}", "Veil Doctor 🩺".bold().green());
    println!("{}", "--------------".dimmed());

    // 1. Veil Version
    println!("Veil Version: {}", env!("CARGO_PKG_VERSION").bold());

    // 2. OS Info
    let os_info = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    println!("OS: {}", os_info);

    // 3. Rust Version
    // Try to run `rustc --version`
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
    println!("{}", "Config Status:".bold());

    // 4. Config Check
    // Check Global Config
    if let Ok(org_path) = std::env::var("VEIL_ORG_RULES") {
        let path = PathBuf::from(&org_path);
        if path.exists() {
            match load_config(&path) {
                Ok(_) => println!("{} Global config loaded ({:?})", "[OK]".green(), path),
                Err(e) => println!("{} Global config error: {}", "[ERR]".red(), e),
            }
        } else {
            println!(
                "{} VEIL_ORG_RULES set but file missing ({:?})",
                "[WARN]".yellow(),
                path
            );
        }
    } else {
        println!("{} Global config (VEIL_ORG_RULES) not set", "[-]".dimmed());
    }

    // Check Local Config
    let local_path = Path::new("veil.toml");
    if local_path.exists() {
        match load_config(local_path) {
            Ok(cfg) => {
                println!("{} Local config loaded (veil.toml)", "[OK]".green());
                if !cfg.rules.is_empty() {
                    println!("   - {} rules defined in local config", cfg.rules.len());
                }
            }
            Err(e) => println!("{} Local config error: {}", "[ERR]".red(), e),
        }
    } else {
        println!("{} Local config (veil.toml) not found", "[-]".yellow());
    }

    // 5. Env Vars
    println!();
    println!("{}", "Environment:".bold());
    let env_vars = ["VEIL_ORG_RULES", "VEIL_FAIL_SCORE", "RUST_LOG"];
    for var in env_vars {
        match std::env::var(var) {
            Ok(val) => println!("{}: {}", var, val),
            Err(_) => println!("{}: {}", var, "(not set)".dimmed()),
        }
    }

    Ok(())
}
