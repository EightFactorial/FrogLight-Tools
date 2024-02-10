#![doc = include_str!("README.md")]

use clap::Parser;
use froglight_data::Version;
use froglight_extractor::manifest;
use tracing::{error, trace, warn};
use tracing_subscriber::{fmt::SubscriberBuilder, EnvFilter};

mod commands;
use commands::{ExtractCommand, ExtractSubCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse the command line arguments.
    let command = ExtractCommand::parse();

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
    let manifest = manifest::version_manifest(&command.cache, command.refresh).await?;
    trace!("ManifestLatest: {}", manifest.latest);

    // Check for requested version in the manifest.
    if !manifest.versions.iter().any(|v| v.id == command.version) {
        error!("Version `{}` not found in the manifest", command.version);
        error!("If the version was just released, try refreshing the manifest with `--refresh`");

        anyhow::bail!("Version `{}` not found in manifest", command.version);
    }

    // Execute the subcommand.
    match &command.subcommand {
        ExtractSubCommand::Extract(_) => commands::extract::extract(&command, &manifest).await,
        ExtractSubCommand::Search(_) => commands::search::search(&command, &manifest).await,
        ExtractSubCommand::Print(_) => commands::print::print(&command, &manifest).await,
    }
}

/// Setup logging.
///
/// Override the logging levels to clean up the output
/// and force logging to write to stderr.
///
/// Allows for piping the output to text editors :)
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
