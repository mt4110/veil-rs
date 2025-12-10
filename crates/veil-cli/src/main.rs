mod cli;
mod commands;
mod config_loader;
mod formatters;
mod output;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::exit;

use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    let result = match &cli.command {
        Commands::Scan {
            paths,
            format,
            fail_score,
            commit,
            since,
            staged,
            progress,
            mask_mode,
            unsafe_output,
            limit,
            fail_on_findings,
            fail_on_severity,
        } => {
            // Quiet overrides progress
            let show_progress = *progress && !cli.quiet;
            commands::scan::scan(
                paths,
                cli.config.as_ref(),
                format,
                *fail_score,
                commit.as_deref(),
                since.as_deref(),
                *staged,
                show_progress,
                mask_mode.as_deref(),
                *unsafe_output,
                *limit,
                *fail_on_findings,
                fail_on_severity.clone(),
            )
        }
        Commands::Filter => commands::filter::filter().map(|_| false),
        Commands::Mask {
            paths,
            dry_run,
            backup_suffix,
        } => commands::mask::mask(paths, cli.config.as_ref(), *dry_run, backup_suffix.clone())
            .map(|_| false),
        Commands::CheckProject => commands::check_project::check_project().map(|res| !res),
        Commands::Init {
            wizard,
            non_interactive,
        } => commands::init::init(*wizard, *non_interactive).map(|_| false),
        Commands::Ignore { path } => {
            commands::ignore::ignore(path, cli.config.as_ref()).map(|_| false)
        }
        Commands::Config(cmd) => match cmd {
            crate::cli::ConfigCommand::Check { config_path } => {
                let path = config_path.clone().or_else(|| cli.config.clone());
                commands::config::check(path.as_ref())
            }
        },
        Commands::PreCommit(cmd) => match cmd {
            crate::cli::PreCommitCommand::Init => commands::pre_commit::init().map(|_| false),
        },
        Commands::Triage(args) => commands::triage::triage(args).map(|_| false),
        Commands::Fix(args) => commands::fix::fix(args).map(|_| false),
        Commands::Git(cmd) => match cmd {
            crate::cli::GitCommand::Scan(args) => commands::git::scan(args).map(|_| false),
        },
    };

    match result {
        Ok(found_secrets) => {
            if found_secrets {
                exit(1);
            } else {
                exit(0);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(2);
        }
    }
}
