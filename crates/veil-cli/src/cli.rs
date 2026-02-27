use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use veil_core::Severity;

#[derive(Parser)]
#[command(name = "veil")]
#[command(version)]
#[command(about = "A high-performance secret detection tool", long_about = None)]
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
    /// Scan files for secrets
    Scan {
        /// Paths to scan
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        /// Output format (text, json, html, markdown, table)
        ///
        /// - text:     Standard colorful terminal output (default)
        /// - json:     Machine-readable JSON for integration
        /// - html:     Self-contained HTML report
        /// - table:    Simple key-value table
        #[arg(long, default_value = "text")]
        format: String,
        /// Fail (exit 1) if the highest finding score exceeds this value (0-100)
        /// This compares against the single highest score among all findings.
        #[arg(long = "fail-on-score", alias = "fail-score", env = "VEIL_FAIL_SCORE")]
        fail_score: Option<u32>,
        /// Fail (exit 1) if any secrets found with severity at or above this level.
        /// Values: Low, Medium, High, Critical
        #[arg(long, value_parser = parse_severity)]
        fail_on_severity: Option<Severity>,
        /// Fail (exit 1) if the total number of findings is at or above this threshold
        #[arg(long)]
        fail_on_findings: Option<usize>,
        /// Scan a specific commit (SHA)
        #[arg(long)]
        commit: Option<String>,
        /// Scan commits since this time (e.g. "1 week ago", "2024-01-01").
        /// This scans the Git history, not valid for file system scans.
        #[arg(long)]
        since: Option<String>,
        /// Scan staged files only
        #[arg(long)]
        staged: bool,
        /// Show progress bar (if TTY)
        #[arg(long)]
        progress: bool,
        /// Masking mode (redact, partial, plain)
        #[arg(long, value_name = "MODE")]
        mask_mode: Option<String>,
        /// Force plain/unsafe output (alias for --mask-mode plain)
        #[arg(long = "unsafe")]
        unsafe_output: bool,
        /// Limit the number of findings (0 = unlimited)
        /// Limit the number of findings (0 = unlimited)
        #[arg(long)]
        limit: Option<usize>,
        /// Use a baseline file to suppress existing findings (S27)
        #[arg(long, value_name = "PATH", conflicts_with = "write_baseline")]
        baseline: Option<PathBuf>,
        /// Write all current findings to a baseline file (exit 0)
        #[arg(long, value_name = "PATH")]
        write_baseline: Option<PathBuf>,
    },
    /// Filter STDIN and mask secrets (outputs to STDOUT)
    Filter,
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
        /// Pin the GitHub Actions workflow to a specific version tag.
        /// - "auto" (default): Use current version (if stable) or display warning (if prerelease)
        /// - "none": Do not pin version (use latest from main/master)
        /// - "vX.Y.Z": Pin to specific version
        #[arg(long, default_value = "auto")]
        pin_tag: String,
    },
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
    /// Interactive Triage
    Triage(TriageArgs),
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
    /// Scan dependencies for vulnerabilities
    Guardian(GuardianArgs),
    /// SOT (Source of Truth) tools
    #[command(subcommand)]
    Sot(SotCommand),
    /// Exception Registry management
    Exceptions(ExceptionsArgs),
    /// Check for updates (stub)
    Update,
    /// Verify an Evidence Pack (ZIP)
    Verify {
        /// Path to the evidence.zip file
        path: PathBuf,
        /// Output format (json/text)
        #[arg(long, default_value = "text")]
        format: String,
        /// Fail (exit 1) if the scan was incomplete
        #[arg(long)]
        require_complete: bool,
        /// Fail (exit 1) if findings exceed this threshold
        #[arg(long)]
        fail_on_findings: Option<usize>,
        /// Expect this specific SHA-256 for the run_meta.json (External Anchor)
        #[arg(long)]
        expect_run_meta_sha256: Option<String>,
        /// Maximum allowed size for the ZIP file (bytes)
        #[arg(long, default_value_t = 500 * 1024 * 1024)]
        max_zip_bytes: u64,
        /// Maximum allowed size for any single uncompressed entry (bytes)
        #[arg(long, default_value_t = 200 * 1024 * 1024)]
        max_entry_bytes: u64,
        /// Maximum allowed total uncompressed size for the ZIP (bytes)
        #[arg(long, default_value_t = 1024 * 1024 * 1024)]
        max_total_bytes: u64,
        /// Maximum allowed number of files in the ZIP
        #[arg(long, default_value_t = 64)]
        max_files: usize,
    },
}

#[derive(Args, Debug, Clone)]
pub struct GuardianArgs {
    #[command(subcommand)]
    pub command: GuardianCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GuardianCommands {
    /// Check lockfile for vulnerabilities
    Check {
        /// Path to Cargo.lock or package-lock.json
        #[arg(default_value = "Cargo.lock")]
        lockfile: std::path::PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormatCli::Human)]
        format: OutputFormatCli,

        /// Offline mode (use cache only)
        #[arg(long)]
        offline: bool,

        /// Fetch detailed vulnerability info from OSV (requires online or cache)
        #[arg(long)]
        osv_details: bool,

        /// Show performance metrics to stderr
        #[arg(long)]
        debug_metrics: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormatCli {
    Human,
    Json,
}

#[derive(Subcommand)]
pub enum RulesCommand {
    /// List all effective rules (after config/remote merge)
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
}

#[derive(Subcommand)]
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

#[derive(Subcommand)]
pub enum PreCommitCommand {
    /// Install the pre-commit hook
    Init,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Validate configuration and rules
    Check {
        /// Path to config file (optional override)
        #[arg(long)]
        config_path: Option<PathBuf>,
    },
    /// Dump configuration (org/user/repo/effective)
    Dump {
        /// Which layer to dump (default: effective)
        #[arg(long, value_enum)]
        layer: Option<ConfigLayer>,

        /// Output format (json/toml). Default: json
        #[arg(long, value_enum)]
        format: Option<ConfigFormat>,
    },
}

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum ConfigLayer {
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

#[derive(Subcommand)]
pub enum SotCommand {
    /// Create a new SOT file
    New(SotNewArgs),

    /// Rename a PR-TBD SOT file to PR-<number>-... and update front matter
    Rename(SotRenameArgs),
}

#[derive(Args, Debug)]
pub struct SotNewArgs {
    /// Target release version (e.g. v0.19.0). If not specified, inferred from branch.
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
