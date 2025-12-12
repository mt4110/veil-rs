use crate::cli::{GuardianArgs, GuardianCommands, OutputFormatCli};
use veil_guardian::report::OutputFormat;
use veil_guardian::scan_lockfile;

pub fn run(args: GuardianArgs) -> anyhow::Result<()> {
    match args.command {
        GuardianCommands::Check { lockfile, format } => {
            let scan_result = scan_lockfile(&lockfile).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to scan lockfile at {:?}: {}\n\nTip: Ensure the file exists and is a valid Cargo.lock.",
                    lockfile,
                    e
                )
            })?;

            let output_format = match format {
                OutputFormatCli::Human => OutputFormat::Human,
                OutputFormatCli::Json => OutputFormat::Json,
            };

            println!("{}", scan_result.display(output_format));

            if !scan_result.is_clean() {
                std::process::exit(1);
            }

            Ok(())
        }
    }
}
