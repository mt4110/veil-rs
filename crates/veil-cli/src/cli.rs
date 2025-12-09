use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        #[arg(long, env = "VEIL_FAIL_SCORE")]
        fail_score: Option<u32>,
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
}
