use crate::cli::{
    ExceptionsAddArgs, ExceptionsArgs, ExceptionsCleanupArgs, ExceptionsRemoveArgs,
    ExceptionsSubcommand,
};
use anyhow::Result;
use chrono::Utc;
use prettytable::{format, Cell, Row, Table};
use std::path::PathBuf;
use veil_core::registry::{Registry, RegistryError};

pub fn run(args: &ExceptionsArgs) -> Result<bool> {
    let registry_path = if args.system_registry {
        // System default path (Unix convention)
        PathBuf::from("/etc/veil/exceptions.toml")
    } else if let Some(path) = &args.registry_path {
        path.clone()
    } else {
        // Repo-local default
        PathBuf::from(".veil/exception_registry.toml")
    };

    match &args.command {
        ExceptionsSubcommand::List => run_list(&registry_path),
        ExceptionsSubcommand::Add(cmd_args) => run_add(cmd_args, &registry_path),
        ExceptionsSubcommand::Remove(cmd_args) => run_remove(cmd_args, &registry_path),
        ExceptionsSubcommand::Cleanup(cmd_args) => run_cleanup(cmd_args, &registry_path),
        ExceptionsSubcommand::Doctor => run_doctor(&registry_path),
    }
}

fn run_list(registry_path: &PathBuf) -> Result<bool> {
    let registry = match Registry::load(registry_path) {
        Ok(reg) => reg,
        Err(RegistryError::NotFound(_)) => Registry::default(),
        Err(e) => return Err(e.into()),
    };

    if registry.exceptions.is_empty() {
        println!("No exceptions found in registry.");
        return Ok(false);
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(Row::new(vec![
        Cell::new("ID").style_spec("b"),
        Cell::new("Status").style_spec("b"),
        Cell::new("Expiry").style_spec("b"),
        Cell::new("Reason").style_spec("b"),
    ]));

    let now = Utc::now();

    for entry in &registry.exceptions {
        let status = registry.check(&entry.id, now);
        let status_str = match status {
            veil_core::registry::ExceptionStatus::Active => "Active",
            veil_core::registry::ExceptionStatus::Expired(_) => "Expired",
            veil_core::registry::ExceptionStatus::NotExcepted => "Inactive", // Should not happen for entry in registry unless logic changes
        };

        let expiry_str = entry
            .expires_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Never".to_string());

        table.add_row(Row::new(vec![
            Cell::new(&entry.id.to_string()),
            Cell::new(status_str),
            Cell::new(&expiry_str),
            Cell::new(&entry.reason),
        ]));
    }

    table.printstd();
    Ok(false)
}

use std::str::FromStr;
use veil_core::finding_id::FindingId;
use veil_core::registry::ExceptionEntry;

fn run_add(args: &ExceptionsAddArgs, registry_path: &PathBuf) -> Result<bool> {
    let id = FindingId::from_str(&args.id).map_err(|e| anyhow::anyhow!(e))?;

    let expires_at = if let Some(s) = &args.expires {
        Some(parse_expiry(s)?)
    } else {
        None
    };

    // Load registry first to check for existing entry
    let mut registry = match Registry::load(registry_path) {
        Ok(reg) => reg,
        Err(RegistryError::NotFound(_)) => Registry::default(),
        Err(e) => return Err(e.into()),
    };

    // Check if ID exists to preserve creation metadata
    let (created_at, created_by) = if let Some(existing) = registry.exceptions.iter().find(|e| e.id == id) {
        (existing.created_at, existing.created_by.clone())
    } else {
        (Some(Utc::now()), std::env::var("USER").ok())
    };

    let entry = ExceptionEntry {
        id: id.clone(),
        reason: args.reason.clone(),
        expires_at,
        created_at,
        created_by,
    };

    if args.dry_run {
        println!("Dry Run: Would add exception:");
        // Serialize to TOML for display?
        // Or just print fields
        println!("ID: {}", entry.id);
        println!("Reason: {}", entry.reason);
        if let Some(exp) = entry.expires_at {
            println!("Expires: {}", exp.to_rfc3339());
        } else {
            println!("Expires: Never");
        }
        return Ok(false);
    }

    // Remove existing entry for same ID (update semantics)
    registry.exceptions.retain(|e| e.id != id);
    registry.exceptions.push(entry);

    if let Some(parent) = registry_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    registry.save(registry_path)?;
    println!("Added exception for {}", id);
    Ok(false)
}

fn parse_expiry(s: &str) -> Result<chrono::DateTime<Utc>> {
    let s = s.trim();
    let split_idx = s
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .ok_or_else(|| anyhow::anyhow!("Invalid duration format (e.g. 30d)"))?;
    let (num_str, unit) = s.split_at(split_idx);
    let num: i64 = num_str.parse()?;

    let duration = match unit {
        "d" => chrono::Duration::days(num),
        "w" => chrono::Duration::weeks(num),
        "y" => chrono::Duration::days(num * 365),
        "h" => chrono::Duration::hours(num),
        "m" => chrono::Duration::minutes(num),
        _ => return Err(anyhow::anyhow!("Unknown unit '{}' (use d, w, y, h, m)", unit)),
    };

    Ok(Utc::now() + duration)
}

fn run_remove(args: &ExceptionsRemoveArgs, registry_path: &PathBuf) -> Result<bool> {
    let id = FindingId::from_str(&args.id).map_err(|e| anyhow::anyhow!(e))?;

    let mut registry = match Registry::load(registry_path) {
        Ok(reg) => reg,
        Err(RegistryError::NotFound(_)) => {
            return Err(anyhow::anyhow!(
                "Registry not found at {}",
                registry_path.display()
            ))
        }
        Err(e) => return Err(e.into()),
    };

    let initial_len = registry.exceptions.len();
    registry.exceptions.retain(|e| e.id != id);

    if registry.exceptions.len() == initial_len {
        return Err(anyhow::anyhow!("Exception {} not found", id));
    }

    if let Some(parent) = registry_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    registry.save(registry_path)?;

    println!("Removed exception {}", id);
    Ok(false)
}

fn run_cleanup(args: &ExceptionsCleanupArgs, registry_path: &PathBuf) -> Result<bool> {
    let mut registry = match Registry::load(registry_path) {
        Ok(reg) => reg,
        Err(RegistryError::NotFound(_)) => {
            println!("Registry not found, nothing to cleanup.");
            return Ok(false);
        }
        Err(e) => return Err(e.into()),
    };

    let now = Utc::now();
    let expired_count = registry
        .exceptions
        .iter()
        .filter(|e| {
            if let Some(expires_at) = e.expires_at {
                expires_at <= now
            } else {
                false
            }
        })
        .count();

    if args.dry_run {
        println!("Dry Run: Would remove {} expired exceptions.", expired_count);
        return Ok(false);
    }

    if expired_count > 0 {
        registry.exceptions.retain(|e| {
            if let Some(expires_at) = e.expires_at {
                expires_at > now
            } else {
                true
            }
        });

        if let Some(parent) = registry_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        registry.save(registry_path)?;
        println!("Removed {} expired exceptions.", expired_count);
    } else {
        println!("No expired exceptions found.");
    }

    Ok(false)
}

fn run_doctor(registry_path: &PathBuf) -> Result<bool> {
    match Registry::load(registry_path) {
        Ok(reg) => {
            println!("OK");
            println!(
                "Registry loaded successfully from {}",
                registry_path.display()
            );
            println!("Version: {}", reg.version);
            println!("Exceptions: {}", reg.exceptions.len());
        }
        Err(RegistryError::NotFound(_)) => {
            println!("Error: Registry missing");
            // Return Ok to avoid duplicate error printing and exit code 2
            return Ok(false);
        }
        Err(RegistryError::ParseError(path, e)) => {
            return Err(anyhow::anyhow!(
                "Parse Error in {}: {}. Manual fix required.",
                path.display(),
                e
            ));
        }
        Err(RegistryError::VersionMismatch { found, expected }) => {
            return Err(anyhow::anyhow!(
                "Version mismatch. Expected {}, found {}.",
                expected,
                found
            ));
        }
        Err(e) => return Err(e.into()),
    }
    Ok(false)
}
