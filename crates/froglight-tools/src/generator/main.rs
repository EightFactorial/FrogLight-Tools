#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use anyhow::anyhow;
use clap::Parser;
use froglight_tools::logging;
use tracing::{debug, error, info};

mod cli;
use cli::GenerateArguments;

mod config;
use config::GenerateConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = GenerateArguments::parse();
    logging::setup(&args.verbose);

    // Debugging information
    debug!("Cache: \"{}\"", args.cache.display());
    debug!("Config: \"{}\"", args.config.display());
    debug!("Directory: \"{}\"", args.dir.display());
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

    Ok(())
}
