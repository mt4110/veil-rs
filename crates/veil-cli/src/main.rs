mod cli;
mod commands;
mod config_loader;
mod formatters;
mod output;

use clap::Parser;
use cli::{Cli, Commands, RulesCommand};
use colored::Colorize;

use std::process::exit;

use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    // Anti-Zombie 🧟‍♂️: Handle Ctrl+C to ensure all threads die immediately
    // Rayon threads can sometimes keep the process alive if not explicitly killed.
    // We use a "forced exit" strategy here to guarantee cleanup.
    ctrlc::set_handler(move || {
        eprintln!(
            "\n{} Received Ctrl+C. Force exiting to prevent zombie processes...",
            "⚠️".yellow()
        );
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();
    let Cli {
        config,
        no_color,
        quiet,
        command,
    } = cli;

    if no_color {
        colored::control::set_override(false);
    }

    let result = match command {
        Some(Commands::Scan(args)) => {
            let show_progress = args.progress && !quiet;
            commands::scan::scan(
                &args.paths,
                config.as_ref(),
                args.format.clone(),
                args.fail_score,
                args.commit.as_deref(),
                args.since.as_deref(),
                args.staged,
                args.interactive,
                show_progress,
                args.mask_mode.as_deref(),
                args.unsafe_output,
                args.limit,
                args.fail_on_findings,
                args.fail_on_severity.clone(),
                args.write_baseline.clone(),
                args.baseline.clone(),
                args.preset.clone(),
                quiet,
                no_color,
            )
        }
        Some(Commands::Filter) => commands::filter::filter(config.as_ref()).map(|_| false),
        Some(Commands::Mask {
            paths,
            dry_run,
            backup_suffix,
        }) => commands::mask::mask(&paths, config.as_ref(), dry_run, backup_suffix.clone())
            .map(|_| false),
        Some(Commands::CheckProject) => commands::check_project::check_project().map(|res| !res),
        Some(Commands::Init {
            wizard,
            non_interactive,
            ci,
            profile,
            preset,
            pin_tag,
            ci_source,
        }) => commands::init::init(
            wizard,
            non_interactive,
            ci.clone(),
            profile.clone(),
            preset.clone(),
            pin_tag.clone(),
            ci_source.clone(),
        )
        .map(|_| false),
        Some(Commands::Ignore { path }) => {
            commands::ignore::ignore(&path, config.as_ref()).map(|_| false)
        }
        Some(Commands::Config(cmd)) => match cmd {
            crate::cli::ConfigCommand::Check { config_path } => {
                let path = config_path.clone().or_else(|| config.clone());
                commands::config::check(path.as_ref())
            }
            crate::cli::ConfigCommand::Dump {
                preset,
                layer,
                format,
            } => commands::config::dump(config.as_ref(), preset.as_deref(), layer, format)
                .map(|_| false),
        },
        Some(Commands::PreCommit(cmd)) => match cmd {
            crate::cli::PreCommitCommand::Init => commands::pre_commit::init().map(|_| false),
        },
        Some(Commands::Triage(args)) => commands::triage::triage(&args).map(|_| false),
        Some(Commands::Fix(args)) => commands::fix::fix(&args).map(|_| false),
        Some(Commands::Git(cmd)) => match cmd {
            crate::cli::GitCommand::Scan(args) => {
                commands::git::scan(&args, config.as_ref()).map(|_| false)
            }
        },
        Some(Commands::Doctor) => commands::doctor::doctor().map(|_| false),
        Some(Commands::Rules(cmd)) => match cmd {
            RulesCommand::List { severity } => {
                commands::rules::list(config.as_ref(), severity).map(|_| false)
            }
            RulesCommand::Explain { rule_id } => {
                commands::rules::explain(config.as_ref(), &rule_id).map(|_| false)
            }
            RulesCommand::PromoteTemplates(args) => {
                commands::rules::promote_templates(&args).map(|_| false)
            }
        },
        Some(Commands::Lsp { preset }) => {
            commands::lsp::run(config.as_ref(), preset.as_deref(), quiet).map(|_| false)
        }
        Some(Commands::Sot(cmd)) => commands::sot::run(&cmd).map(|_| false),
        Some(Commands::Exceptions(args)) => commands::exceptions::run(&args).map(|_| false),
        Some(Commands::Update(args)) => commands::update::update(&args).map(|_| false),
        Some(Commands::Verify(args)) => commands::verify::verify(args).map(|_| false),
        None => {
            // If no subcommand is provided, print help
            use clap::CommandFactory;
            let _ = Cli::command().print_help();
            exit(0);
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
            for cause in e.chain().skip(1) {
                eprintln!("  Caused by: {}", cause);
            }
            exit(2);
        }
    }
}
