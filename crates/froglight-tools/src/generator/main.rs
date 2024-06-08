#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::sync::Arc;

use anyhow::anyhow;
use clap::Parser;
use froglight_tools::logging;
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};

mod cli;
use cli::GenerateArguments;

mod config;
use config::GenerateConfig;

mod generate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = GenerateArguments::parse();
    logging::setup(&args.verbose);

    // Debugging information
    debug!("Cache: \"{}\"", args.cache.display());
    debug!("Config: \"{}\"", args.config.display());
    debug!("Directory: \"{}\"", args.dir.display());
    debug!("Refresh: {}", args.refresh);
    debug!("");

    // Make sure `dir` points to a valid project directory
    info!("Checking directory: \"{}\"", args.dir.display());
    if !args.dir.is_dir() || !args.dir.join("Cargo.toml").is_file() {
        let error = format!("Invalid project directory: \"{}\"", args.dir.display());

        error!("{error}");
        return Err(anyhow!(error));
    }
    debug!("Project directory is valid!");
    debug!("");

    // If `refresh` is set, delete the cache directory
    if args.refresh {
        warn!("Clearing cache directory: \"{}\"", args.cache.display());
        match tokio::fs::remove_dir_all(&args.cache).await {
            Ok(()) => {
                debug!("Cache directory cleared!");
                debug!("");
            }
            Err(err) => {
                let error = format!("Failed to clear cache directory: {err}");

                error!("{error}");
                return Err(anyhow!(error));
            }
        }
    }

    // Make sure `cache` points to a directory or create it
    if !args.cache.exists() {
        info!("Creating cache directory: \"{}\"", args.cache.display());
        match tokio::fs::create_dir_all(&args.cache).await {
            Ok(()) => {
                debug!("Cache directory created!");
                debug!("");
            }
            Err(err) => {
                let error = format!("Failed to create cache directory: {err}");

                error!("{error}");
                return Err(anyhow!(error));
            }
        }
    } else if args.cache.is_file() {
        let error = format!("Invalid cache directory: \"{}\"", args.cache.display());

        error!("{error}");
        return Err(anyhow!(error));
    }

    // Load the configuration file
    info!("Loading configuration: \"{}\"", args.config.display());
    let config: GenerateConfig = match tokio::fs::read_to_string(&args.config).await {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(err) => {
                let error = format!("Failed to parse configuration: {err}");

                error!("{error}");
                Err(anyhow!(error))
            }
        },
        Err(err) => {
            let error = format!("Failed to read configuration: {err}");

            error!("{error}");
            Err(anyhow!(error))
        }
    }?;
    debug!("Configuration: {:#?}", config.versions);

    // Create a `Client` for downloading files
    let client = reqwest::Client::new();

    // Get the `VersionManifest`
    let version_manifest =
        Arc::new(froglight_tools::manifests::get_version_manifest(&args.cache, &client).await?);

    // Get the `YarnManifest`
    let yarn_manifest =
        Arc::new(froglight_tools::manifests::get_yarn_manifest(&args.cache, &client).await?);

    // Download the `Enigma` JAR
    let Some(enimga_path) =
        froglight_tools::jar_tools::download_tinyremapper(&args.cache, &client).await
    else {
        let error = "Failed to download `Enigma` JAR";

        error!("{error}");
        return Err(anyhow!(error));
    };

    // Generate all versions simultaneously
    let mut joinset = JoinSet::new();
    for version in config.versions {
        joinset.spawn(generate::generate(
            version,
            version_manifest.clone(),
            yarn_manifest.clone(),
            enimga_path.clone(),
            args.cache.clone(),
            client.clone(),
        ));
    }
    while joinset.join_next().await.is_some() {}

    Ok(())
}
