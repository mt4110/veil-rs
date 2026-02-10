use crate::cli::{
    ExceptionsAddArgs, ExceptionsArgs, ExceptionsCleanupArgs, ExceptionsRemoveArgs,
    ExceptionsSubcommand,
};
use anyhow::Result;
use chrono::Utc;
use prettytable::{format, Cell, Row, Table};
use std::path::{Path, PathBuf};
use veil_core::registry::{Registry, RegistryError};

/// Registry path resolution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrySource {
    SystemDefault,
    ExplicitPath,
    RepoDefault,
    None,
}

#[derive(Debug, Clone)]
pub struct RegistryResolved {
    pub source: RegistrySource,
    pub path: Option<PathBuf>,
}

/// Resolve registry path according to SOT 4.1 priority:
/// 1. --system-registry → SystemDefaultPath
/// 2. --registry-path <PATH> → ExplicitPath
/// 3. ops/exceptions.toml (exists) → RepoDefault
/// 4. None
pub fn resolve_registry_path(args: &ExceptionsArgs, repo_root: &Path) -> RegistryResolved {
    if args.system_registry {
        // Priority 1: System default
        RegistryResolved {
            source: RegistrySource::SystemDefault,
            path: Some(PathBuf::from("/etc/veil/exceptions.toml")),
        }
    } else if let Some(path) = &args.registry_path {
        // Priority 2: Explicit path
        RegistryResolved {
            source: RegistrySource::ExplicitPath,
            path: Some(path.clone()),
        }
    } else {
        // Priority 3: Repo default (only if exists)
        let repo_default = repo_root.join("ops/exceptions.toml");
        if repo_default.exists() {
            RegistryResolved {
                source: RegistrySource::RepoDefault,
                path: Some(repo_default),
            }
        } else {
            // Priority 4: None
            RegistryResolved {
                source: RegistrySource::None,
                path: None,
            }
        }
    }
}

/// Result of registry loading with strict handling
enum RegistryLoadResult {
    Ok(Registry),
    MissingWarning,
    Error(anyhow::Error),
}

/// Load registry with strict/non-strict handling per SOT 4.3
/// 
/// Strict mode (fail fast):
/// - missing/unreadable → Error
/// - parse error → Error
/// - schema validation → Error
/// 
/// Non-strict mode (warn and continue):
/// - missing/unreadable → Warning + empty registry
/// - parse/schema error → Warning (but FAIL for mutating ops - safety first)
fn load_registry_strict(
    path: &PathBuf,
    strict: bool,
    is_mutating: bool,
    source: &RegistrySource,
) -> RegistryLoadResult {
    let source_label = match source {
        RegistrySource::SystemDefault => "source: system_default",
        RegistrySource::ExplicitPath => "source: explicit_path",
        RegistrySource::RepoDefault => "source: repo_default",
        RegistrySource::None => "source: none",
    };
    
    match Registry::load(path) {
        Ok(registry) => RegistryLoadResult::Ok(registry),
        Err(RegistryError::NotFound(_)) => {
            if strict {
                RegistryLoadResult::Error(anyhow::anyhow!(
                    "Registry not found at {} ({})\n\nNext steps:\n  1. Create registry: veil exceptions add <finding-id> --reason \"...\" --expires 30d\n  2. Or use --registry-path to specify alternative location\n  3. Or disable strict mode (remove --strict-exceptions)",
                    path.display(),
                    source_label
                ))
            } else {
                eprintln!(
                    "Warning: Registry not found at {} ({}). Exceptions disabled for this operation.",
                    path.display(),
                    source_label
                );
                RegistryLoadResult::MissingWarning
            }
        }
        Err(RegistryError::ParseError(p, e)) => {
            let error = anyhow::anyhow!(
                "Parse error in registry at {} ({}): {}\n\nNext steps:\n  1. Fix TOML syntax in {}\n  2. Or use --registry-path to specify alternative registry\n  3. Rollback: git restore {}",
                p.display(),
                source_label,
                e,
                p.display(),
                p.display()
            );
            
            if strict || is_mutating {
                // Mutating ops ALWAYS fail on parse error (safety first)
                RegistryLoadResult::Error(error)
            } else {
                eprintln!(
                    "Warning: {}\nExceptions disabled for this operation.",
                    error
                );
                RegistryLoadResult::MissingWarning
            }
        }
        Err(RegistryError::VersionMismatch { found, expected }) => {
            let error = anyhow::anyhow!(
                "Schema version mismatch at {} ({}). Expected {}, found {}.\n\nNext steps:\n  1. Upgrade veil: cargo install veil-cli\n  2. Or migrate registry: veil exceptions doctor",
                path.display(),
                source_label,
                expected,
                found
            );
            
            if strict || is_mutating {
                RegistryLoadResult::Error(error)
            } else {
                eprintln!(
                    "Warning: {}\nExceptions disabled for this operation.",
                    error
                );
                RegistryLoadResult::MissingWarning
            }
        }
        Err(e) => RegistryLoadResult::Error(e.into()),
    }
}

pub fn run(args: &ExceptionsArgs) -> Result<bool> {
    // Get repo root (current working directory for CLI)
    let repo_root = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    
    let resolved = resolve_registry_path(args, &repo_root);
    let registry_path = resolved.path.unwrap_or_else(|| {
        // Fallback for commands that require a path (they will fail gracefully)
        PathBuf::from("ops/exceptions.toml")
    });

    match &args.command {
        ExceptionsSubcommand::List => run_list(&registry_path, args.strict_exceptions, &resolved.source),
        ExceptionsSubcommand::Add(cmd_args) => run_add(cmd_args, &registry_path, args.strict_exceptions, &resolved.source),
        ExceptionsSubcommand::Remove(cmd_args) => run_remove(cmd_args, &registry_path, args.strict_exceptions, &resolved.source),
        ExceptionsSubcommand::Cleanup(cmd_args) => run_cleanup(cmd_args, &registry_path, args.strict_exceptions, &resolved.source),
        ExceptionsSubcommand::Doctor => run_doctor(&registry_path, args.strict_exceptions, &resolved.source),
    }
}

fn run_list(registry_path: &PathBuf, strict: bool, source: &RegistrySource) -> Result<bool> {
    let registry = match load_registry_strict(registry_path, strict, false, source) {
        RegistryLoadResult::Ok(reg) => reg,
        RegistryLoadResult::MissingWarning => Registry::default(),
        RegistryLoadResult::Error(e) => return Err(e),
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

fn run_add(args: &ExceptionsAddArgs, registry_path: &PathBuf, strict: bool, source: &RegistrySource) -> Result<bool> {
    let id = FindingId::from_str(&args.id).map_err(|e| anyhow::anyhow!(e))?;

    let expires_at = if let Some(s) = &args.expires {
        Some(parse_expiry(s)?)
    } else {
        None
    };

    // Load registry first to check for existing entry
    let mut registry = match load_registry_strict(registry_path, strict, true, source) {
        RegistryLoadResult::Ok(reg) => reg,
        RegistryLoadResult::MissingWarning => Registry::default(),
        RegistryLoadResult::Error(e) => return Err(e),
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

fn run_remove(args: &ExceptionsRemoveArgs, registry_path: &PathBuf, strict: bool, source: &RegistrySource) -> Result<bool> {
    let id = FindingId::from_str(&args.id).map_err(|e| anyhow::anyhow!(e))?;

    let mut registry = match load_registry_strict(registry_path, strict, true, source) {
        RegistryLoadResult::Ok(reg) => reg,
        RegistryLoadResult::MissingWarning => {
            return Err(anyhow::anyhow!(
                "Registry not found at {}",
                registry_path.display()
            ))
        }
        RegistryLoadResult::Error(e) => return Err(e),
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

fn run_cleanup(args: &ExceptionsCleanupArgs, registry_path: &PathBuf, strict: bool, source: &RegistrySource) -> Result<bool> {
    let mut registry = match load_registry_strict(registry_path, strict, true, source) {
        RegistryLoadResult::Ok(reg) => reg,
        RegistryLoadResult::MissingWarning => {
            println!("Registry not found, nothing to cleanup.");
            return Ok(false);
        }
        RegistryLoadResult::Error(e) => return Err(e),
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

fn run_doctor(registry_path: &PathBuf, strict: bool, source: &RegistrySource) -> Result<bool> {
    match load_registry_strict(registry_path, strict, false, source) {
        RegistryLoadResult::Ok(reg) => {
            println!("OK");
            println!(
                "Registry loaded successfully from {}",
                registry_path.display()
            );
            println!("Version: {}", reg.version);
            println!("Exceptions: {}", reg.exceptions.len());
        }
        RegistryLoadResult::MissingWarning => {
            println!("Warning: Registry missing");
            return Ok(false);
        }
        RegistryLoadResult::Error(e) => {
            return Err(e);
        }
    }
    Ok(false)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ExceptionsSubcommand;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_resolve_priority_system_registry() {
        let temp_dir = TempDir::new().unwrap();
        let args = ExceptionsArgs {
            system_registry: true,
            registry_path: None,
            strict_exceptions: false,
            command: ExceptionsSubcommand::List,
        };

        let resolved = resolve_registry_path(&args, temp_dir.path());
        assert_eq!(resolved.source, RegistrySource::SystemDefault);
        assert_eq!(
            resolved.path,
            Some(PathBuf::from("/etc/veil/exceptions.toml"))
        );
    }

    #[test]
    fn test_resolve_priority_explicit_path() {
        let temp_dir = TempDir::new().unwrap();
        let args = ExceptionsArgs {
            system_registry: false,
            registry_path: Some(PathBuf::from("/custom/path.toml")),
            strict_exceptions: false,
            command: ExceptionsSubcommand::List,
        };

        let resolved = resolve_registry_path(&args, temp_dir.path());
        assert_eq!(resolved.source, RegistrySource::ExplicitPath);
        assert_eq!(resolved.path, Some(PathBuf::from("/custom/path.toml")));
    }

    #[test]
    fn test_resolve_priority_repo_default_exists() {
        // Create temp directory with ops/exceptions.toml
        let temp_dir = TempDir::new().unwrap();
        let ops_dir = temp_dir.path().join("ops");
        fs::create_dir_all(&ops_dir).unwrap();
        fs::write(ops_dir.join("exceptions.toml"), "# test").unwrap();

        let args = ExceptionsArgs {
            system_registry: false,
            registry_path: None,
            strict_exceptions: false,
            command: ExceptionsSubcommand::List,
        };

        let resolved = resolve_registry_path(&args, temp_dir.path());
        assert_eq!(resolved.source, RegistrySource::RepoDefault);
        assert_eq!(resolved.path, Some(temp_dir.path().join("ops/exceptions.toml")));
    }

    #[test]
    fn test_resolve_priority_none() {
        // Create temp directory with NO ops/exceptions.toml
        let temp_dir = TempDir::new().unwrap();

        let args = ExceptionsArgs {
            system_registry: false,
            registry_path: None,
            strict_exceptions: false,
            command: ExceptionsSubcommand::List,
        };

        let resolved = resolve_registry_path(&args, temp_dir.path());
        assert_eq!(resolved.source, RegistrySource::None);
        assert_eq!(resolved.path, None);
    }

    #[test]
    fn test_system_registry_overrides_explicit() {
        let temp_dir = TempDir::new().unwrap();
        // system-registry should take priority even if registry-path is set
        // (though CLI flags prevent this via conflicts_with)
        let args = ExceptionsArgs {
            system_registry: true,
            registry_path: Some(PathBuf::from("/custom/path.toml")),
            strict_exceptions: false,
            command: ExceptionsSubcommand::List,
        };

        let resolved = resolve_registry_path(&args, temp_dir.path());
        assert_eq!(resolved.source, RegistrySource::SystemDefault);
        assert_eq!(
            resolved.path,
            Some(PathBuf::from("/etc/veil/exceptions.toml"))
        );
    }
}
