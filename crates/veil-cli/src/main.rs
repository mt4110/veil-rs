mod cli;
mod commands;
mod formatters;
mod output;

use clap::Parser;
use cli::{Cli, Commands};
use std::process::exit;

use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Scan {
            paths,
            format,
            fail_score,
            commit,
            since,
            staged,
        } => commands::scan::scan(
            paths,
            cli.config.as_ref(),
            format,
            *fail_score,
            commit.as_deref(),
            since.as_deref(),
            *staged,
        ),
        Commands::Filter => commands::filter::filter().map(|_| false),
        Commands::Mask { paths } => commands::mask::mask(paths, cli.config.as_ref()).map(|_| false),
        Commands::CheckProject => commands::check_project::check_project().map(|res| !res),
        Commands::Init => commands::init::init().map(|_| false),
        Commands::Ignore { path } => {
            commands::ignore::ignore(path, cli.config.as_ref()).map(|_| false)
        }
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
