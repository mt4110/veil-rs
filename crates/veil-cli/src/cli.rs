use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use veil_core::Severity;

#[derive(Parser)]
#[command(name = "veil")]
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
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan files for secrets
    Scan {
        /// Paths to scan
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        /// Output format (text, json, html, markdown, table)
        #[arg(long, default_value = "text")]
        format: String,
        /// Fail (exit 1) if finding score exceeds this value
        #[arg(long = "fail-on-score", alias = "fail-score", env = "VEIL_FAIL_SCORE")]
        fail_score: Option<u32>,
        /// Fail (exit 1) if any secrets found with severity at or above this level (Low, Medium, High, Critical)
        #[arg(long, value_parser = parse_severity)]
        fail_on_severity: Option<Severity>,
        /// Fail (exit 1) if findings count is at or above this threshold
        #[arg(long)]
        fail_on_findings: Option<usize>,
        /// Scan a specific commit (SHA)
        #[arg(long)]
        commit: Option<String>,
        /// Scan commits since this time (e.g. "1 week ago", "2024-01-01")
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
        #[arg(long)]
        limit: Option<usize>,
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
    /// Run project health checks
    CheckProject,
    /// Initialize a new configuration file
    Init {
        /// Run interactive wizard
        #[arg(long)]
        wizard: bool,
        /// Fail if file exists (script mode)
        #[arg(long)]
        non_interactive: bool,
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

    /// Apply shifts (write to files). If NOT specified, runs in dry-run mode.
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
