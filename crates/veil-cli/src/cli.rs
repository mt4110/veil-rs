use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "veil")]
#[command(about = "A high-performance secret detection tool", long_about = None)]
pub struct Cli {
    /// Path to config file (default: ./veil.toml)
    #[arg(long, short, global = true)]
    pub config: Option<PathBuf>,

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
        /// Output format (text, json)
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
    Init,
    /// Add path to ignore list
    Ignore {
        /// Path to ignore
        path: String,
    },
}
