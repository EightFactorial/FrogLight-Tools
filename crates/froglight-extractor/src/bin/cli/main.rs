#![doc = include_str!("../../../README.md")]

use clap::Parser;
use froglight_data::Version;
use froglight_extractor::manifest;
use tracing::{debug, error, trace, warn};
use tracing_subscriber::{fmt::SubscriberBuilder, EnvFilter};

mod classmap;

mod commands;
use commands::{Command, SubCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse the command line arguments.
    let command = Command::parse();

    // Setup logging if the verbose flag is set.
    if command.verbose {
        setup_logging();
        trace!("Parsed Command: {command:?}");
    }

    // Warn if the version is not a release version.
    if !matches!(command.version, Version::Release(_)) {
        warn!("Version `{}` is not a release version, this may cause issues!", command.version);
    }

    // Load or refresh the version manifest.
    let manifest = manifest::version_manifest(&command.cache, command.refresh).await;
    debug!("ManifestLatest: {}", manifest.latest);

    // Check for requested version in the manifest.
    if !manifest.versions.iter().any(|v| v.id == command.version) {
        error!("Version `{}` not found in the manifest", command.version);
        error!("If the version was just released, try refreshing the manifest with `--refresh`");

        anyhow::bail!("Version `{}` not found in manifest", command.version);
    }

    // Execute the subcommand.
    match &command.subcommand {
        SubCommand::Extract(_) => {
            // Extract the data.
            let result = commands::extract::extract(&command, &manifest).await;

            // Handle the result.
            if let Some(output) = command.output {
                // Write the result to the output file.
                serde_json::to_writer_pretty(
                    std::fs::File::create(output).expect("Failed to create output file"),
                    &result,
                )
                .expect("Failed to write output to file");
            } else {
                // Write the result to stdout.
                serde_json::to_writer_pretty(std::io::stdout(), &result)
                    .expect("Failed to write output");
            }

            Ok(())
        }
        SubCommand::Search(_) => commands::search::search(&command, &manifest).await,
        SubCommand::Print(_) => commands::print::print(&command, &manifest).await,
    }
}

/// Setup logging.
///
/// Override the logging levels to clean up the output
/// and force logging to write to stderr.
///
/// Allows for piping the output to text editors :)
///
/// `froglight-extractor ... extract 2>/dev/null | subl -`
fn setup_logging() {
    let builder = SubscriberBuilder::default().without_time().compact();

    let filter = EnvFilter::from_default_env()
        .add_directive("reqwest=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive(
            #[cfg(debug_assertions)]
            {
                "froglight_extractor=debug".parse().unwrap()
            },
            #[cfg(not(debug_assertions))]
            {
                "froglight_extractor=info".parse().unwrap()
            },
        );

    builder.with_writer(std::io::stderr).with_env_filter(filter).init();
}
