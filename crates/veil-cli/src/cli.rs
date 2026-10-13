use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use veil_core::Severity;

#[derive(Parser)]
#[command(name = "veil")]
#[command(version)]
#[command(about = "Local secret scanning and masking", long_about = None)]
pub struct Cli {
    /// Path to config file (default: ./veil.toml)
    #[arg(long, short, global = true)]
    pub config: Option<PathBuf>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    // Standard local CLI commands; secondary compatibility commands remain hidden.
    /// Scan files and directories using local rules
    Scan(ScanArgs),
    /// Filter STDIN and mask secrets (outputs to STDOUT)
    Filter,
    #[command(hide = true)]
    /// Rewrite files in-place masking found secrets
    Mask {
        /// Paths to mask
        paths: Vec<PathBuf>,
        /// Dry run (print change summary without modifying files)
        #[arg(long)]
        dry_run: bool,
        /// Backup original file with this suffix (e.g. .bak)
        #[arg(long)]
        backup_suffix: Option<String>,
    },
    /// Run project health checks (secrets, gitignore, CI config, etc)
    #[command(hide = true)]
    /// This is a passive check that suggests improvements, it does not modify files.
    CheckProject,
    /// Initialize a new configuration file
    Init {
        /// Run interactive wizard
        #[arg(long)]
        wizard: bool,
        /// Fail if file exists (script mode)
        #[arg(long)]
        non_interactive: bool,
        /// Generate CI configuration for a specific provider (e.g. "github")
        #[arg(long)]
        ci: Option<String>,
        /// Initialize with a specific profile (e.g. "Logs") without wizard
        #[arg(long)]
        profile: Option<String>,
        /// Apply a built-in preset to the generated config
        #[arg(long, value_name = "ID")]
        preset: Option<String>,
        /// CI source: "checkout" for this repository, or a full 40-character Veil commit SHA
        #[arg(long, requires = "ci", conflicts_with = "pin_tag")]
        ci_source: Option<String>,
        /// Explicit release tag (compatibility); does not identify the current working tree
        #[arg(long, requires = "ci", conflicts_with = "ci_source")]
        pin_tag: Option<String>,
    },
    #[command(hide = true)]
    /// Add path to ignore list
    Ignore {
        /// Path to ignore
        path: String,
    },
    /// Configuration tools
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Pre-commit hook tools
    #[command(subcommand)]
    PreCommit(PreCommitCommand),
    #[command(hide = true)]
    /// Interactive Triage
    Triage(TriageArgs),
    #[command(hide = true)]
    /// Automatically fix findings (add inline ignores)
    Fix(FixArgs),
    /// Git history and operations
    #[command(subcommand)]
    Git(GitCommand),
    /// Show diagnostic definition and system info
    Doctor,

    /// Rules related commands
    #[command(subcommand)]
    Rules(RulesCommand),
    /// Run the Veil language server over stdio
    Lsp {
        /// Apply a built-in preset as the base config layer
        #[arg(long, value_name = "ID")]
        preset: Option<String>,
    },
    /// SOT (Source of Truth) tools
    #[command(subcommand, hide = true)]
    Sot(SotCommand),
    #[command(hide = true)]
    /// Exception Registry management
    Exceptions(ExceptionsArgs),
    /// Install a local signed RulePack (requires --local-pack)
    Update(UpdateArgs),
    /// Verify an Evidence Pack v1 ZIP
    Verify(VerifyArgs),
}

#[derive(Subcommand)]
pub enum RulesCommand {
    /// List all effective local rules
    List {
        /// Filter by minimum severity (e.g. HIGH -> HIGH & CRITICAL)
        #[arg(long, value_parser = parse_severity)]
        severity: Option<Severity>,
    },
    /// Show detailed information for a specific rule
    Explain {
        /// Rule ID (e.g. creds.aws.access_key_id)
        rule_id: String,
    },
    /// Promote selected inactive JP templates into an executable RulePack
    PromoteTemplates(TemplatePromoteArgs),
}

#[derive(Args, Debug)]
pub struct TemplatePromoteArgs {
    /// Template pack root containing MANIFEST.csv and templates/
    #[arg(long)]
    pub templates_dir: PathBuf,

    /// Output RulePack directory to create or update
    #[arg(long)]
    pub out_dir: PathBuf,

    /// Include only these categories. Repeat for multiple categories.
    #[arg(long)]
    pub category: Vec<String>,

    /// Include only these variants. Repeat for multiple variants.
    #[arg(long)]
    pub variant: Vec<String>,

    /// Include only these severities. Repeat for multiple severities.
    #[arg(long, value_parser = parse_severity)]
    pub severity: Vec<Severity>,

    /// Include only templates with score >= this value.
    #[arg(long)]
    pub min_score: Option<u32>,

    /// Include only templates with score <= this value.
    #[arg(long)]
    pub max_score: Option<u32>,

    /// RulePack id written to 00_manifest.toml.
    #[arg(long, default_value = "veil.jp.security.promoted")]
    pub pack_id: String,

    /// Preview selected templates without writing files.
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite generated manifests and selected template copies in an existing output directory.
    #[arg(long)]
    pub force: bool,

    /// Allow promotion with no filters, selecting every template.
    #[arg(long)]
    pub allow_all: bool,
}

#[derive(Subcommand, Debug)]
pub enum GitCommand {
    /// Scan git history
    Scan(GitScanArgs),
}

#[derive(Args, Debug)]
pub struct GitScanArgs {
    /// Git range to scan (e.g. "HEAD", "HEAD~5..HEAD", "main..feature")
    /// Defaults to "HEAD" if not specified.
    pub range: Option<String>,

    /// Scan changes against the upstream branch or origin/main (PR mode)
    #[arg(long, conflicts_with = "range")]
    pub pr: bool,
}

#[derive(Args, Debug)]
pub struct TriageArgs {
    /// Path(s) to scan. Defaults to current directory.
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,
}

#[derive(Args, Debug)]
pub struct FixArgs {
    /// Path(s) to fix. Defaults to current directory.
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Apply fixes (write to files). If NOT specified, runs in dry-run mode.
    #[arg(long)]
    pub apply: bool,
}

#[derive(Subcommand, Debug)]
pub enum PreCommitCommand {
    /// Install the pre-commit hook
    Init,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Validate configuration and rules
    Check {
        /// Path to config file (optional override)
        #[arg(long)]
        config_path: Option<PathBuf>,
    },
    /// Dump configuration (preset/org/user/repo/effective)
    Dump {
        /// Apply a built-in preset before other config layers
        #[arg(long, value_name = "ID")]
        preset: Option<String>,

        /// Which layer to dump (default: effective)
        #[arg(long, value_enum)]
        layer: Option<ConfigLayer>,

        /// Output format (json/toml). Default: json
        #[arg(long, value_enum)]
        format: Option<ConfigFormat>,
    },
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConfigLayer {
    Preset,
    Org,
    User,
    Repo,
    Effective,
}

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum ConfigFormat {
    Json,
    Toml,
}

#[derive(Subcommand, Debug)]
pub enum SotCommand {
    /// Create a new SOT file
    New(SotNewArgs),

    /// Rename a PR-TBD SOT file to PR-<number>-... and update front matter
    Rename(SotRenameArgs),
}

#[derive(Args, Debug)]
pub struct SotNewArgs {
    /// Pull Request number. If omitted, creates a PR-TBD SOT file.
    #[arg(long)]
    pub pr: Option<u32>,

    /// Target release version (e.g. v0.19.0). If omitted, a slug-only filename is used.
    #[arg(long)]
    pub release: Option<String>,

    /// Epic identifier (A, B, C...)
    #[arg(long, default_value = "A")]
    pub epic: String,

    /// Optional slug for filename
    #[arg(long)]
    pub slug: Option<String>,

    /// Output directory
    #[arg(long, default_value = "docs/pr")]
    pub out: PathBuf,

    /// Optional title overlap
    #[arg(long)]
    pub title: Option<String>,

    /// SOT status
    #[arg(long, default_value = "Draft")]
    pub status: String,

    /// Creation date to write into front matter. Defaults to TBD for deterministic output.
    #[arg(long)]
    pub date: Option<String>,

    /// Dry run (do not write files)
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing files
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct SotRenameArgs {
    /// Pull Request number (e.g. 123)
    #[arg(long)]
    pub pr: u32,

    /// Path to the SOT file (optional). If omitted, auto-detects a single PR-TBD-*.md under --dir.
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Directory to search for PR-TBD-*.md (default: docs/pr)
    #[arg(long, default_value = "docs/pr")]
    pub dir: PathBuf,

    /// Dry run (do not rename/write files)
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite existing destination file, and allow updating an existing pr: <n> even if different
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct ExceptionsArgs {
    /// Use system-wide exception registry (e.g., /etc/veil/exceptions.toml)
    #[arg(long, conflicts_with = "registry_path")]
    pub system_registry: bool,

    /// Path to explicit exception registry file
    #[arg(long, value_name = "PATH", conflicts_with = "system_registry")]
    pub registry_path: Option<PathBuf>,

    /// Fail fast on registry issues (missing/invalid/expired)
    #[arg(long)]
    pub strict_exceptions: bool,

    #[command(subcommand)]
    pub command: ExceptionsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ExceptionsSubcommand {
    /// List all exceptions
    List,
    /// Add a new exception
    Add(ExceptionsAddArgs),
    /// Remove an exception by ID
    Remove(ExceptionsRemoveArgs),
    /// Clean up expired exceptions
    Cleanup(ExceptionsCleanupArgs),
    /// Check registry health
    Doctor,
}

#[derive(Args, Debug)]
pub struct ExceptionsAddArgs {
    /// Exception ID (e.g., VL-001-abc123)
    pub id: String,
    /// Reason for the exception
    #[arg(long)]
    pub reason: String,
    /// Expiration (e.g., 30d, 1w, 1y)
    #[arg(long)]
    pub expires: Option<String>,
    /// Dry run (don't write changes)
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct ExceptionsRemoveArgs {
    /// Exception ID to remove
    pub id: String,
}

#[derive(Args, Debug)]
pub struct ExceptionsCleanupArgs {
    /// Dry run (don't write changes)
    #[arg(long)]
    pub dry_run: bool,
}

fn parse_severity(s: &str) -> Result<Severity, String> {
    let v = s.to_lowercase();
    match v.as_str() {
        "low" => Ok(Severity::Low),
        "medium" | "med" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        "critical" | "crit" => Ok(Severity::Critical),
        _ => Err(format!(
            "Invalid severity: {s}. Use Low|Medium|High|Critical"
        )),
    }
}

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Paths to scan
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,
    /// Apply a built-in preset as the base config layer
    #[arg(long, value_name = "ID")]
    pub preset: Option<String>,
    /// Output format (text, json, html, markdown, table)
    ///
    /// - text:     Standard colorful terminal output (default)
    /// - json:     Machine-readable JSON for integration
    /// - html:     Self-contained HTML report
    /// - table:    Simple key-value table
    #[arg(long, default_value = "text", value_parser = ["text", "json", "html", "table", "md", "markdown"], ignore_case = true)]
    pub format: String,
    /// Fail (exit 1) if the highest finding score exceeds this value (0-100)
    /// This compares against the single highest score among all findings.
    #[arg(long = "fail-on-score", alias = "fail-score", env = "VEIL_FAIL_SCORE")]
    pub fail_score: Option<u32>,
    /// Fail (exit 1) if any finding score reaches this severity's score floor.
    /// Values: Low, Medium, High, Critical
    #[arg(long, value_parser = parse_severity)]
    pub fail_on_severity: Option<Severity>,
    /// Fail (exit 1) if the total number of findings is at or above this threshold (must be >= 1)
    #[arg(long)]
    pub fail_on_findings: Option<usize>,
    /// Scan a specific commit (SHA)
    #[arg(long)]
    pub commit: Option<String>,
    /// Scan commits since this time (e.g. "1 week ago", "2024-01-01").
    /// This scans the Git history, not valid for file system scans.
    #[arg(long)]
    pub since: Option<String>,
    /// Scan staged files only
    #[arg(long)]
    pub staged: bool,
    /// Start interactive review mode
    #[arg(long)]
    pub interactive: bool,
    /// Show progress bar (if TTY)
    #[arg(long)]
    pub progress: bool,
    /// Masking mode (redact, partial, plain)
    #[arg(long, value_name = "MODE")]
    pub mask_mode: Option<String>,
    /// Force plain/unsafe output (alias for --mask-mode plain)
    #[arg(long = "unsafe")]
    pub unsafe_output: bool,
    /// Limit the number of findings (0 = unlimited)
    #[arg(long)]
    pub limit: Option<usize>,
    /// Use a baseline file to suppress existing findings (S27)
    #[arg(long, value_name = "PATH", conflicts_with = "write_baseline")]
    pub baseline: Option<PathBuf>,
    /// Write all current findings to a baseline file (exit 0)
    #[arg(long, value_name = "PATH")]
    pub write_baseline: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    pub path: PathBuf,
    #[arg(long, default_value = "text")]
    pub format: String,
    #[arg(long)]
    pub require_complete: bool,
    #[arg(long)]
    pub expect_run_meta_sha256: Option<String>,
    #[arg(long)]
    pub fail_on_findings: Option<usize>,
    #[arg(long, default_value_t = 500 * 1024 * 1024)]
    pub max_zip_bytes: u64,
    #[arg(long, default_value_t = 200 * 1024 * 1024)]
    pub max_entry_bytes: u64,
    #[arg(long, default_value_t = 1024 * 1024 * 1024)]
    pub max_total_bytes: u64,
    #[arg(long, default_value_t = 64)]
    pub max_files: usize,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    #[arg(
        long,
        help = "Path to a hex-encoded public key to verify RulePack signature"
    )]
    pub public_key: Option<PathBuf>,
    #[arg(long, hide = true, help = "Retired: remote downloads are disabled")]
    pub url: Option<String>,
    #[arg(long, help = "Path to a local signed RulePack for offline application")]
    pub local_pack: Option<PathBuf>,
}
