#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use anyhow::anyhow;
use clap::Parser;
use froglight_tools::logging;
use tracing::{debug, error, info};

mod cli;
use cli::ExtractArguments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = ExtractArguments::parse();
    logging::setup(&args.verbose);

    // Debugging information
    info!("Version: \"{}\"", args.version);
    debug!("Cache: \"{}\"", args.cache.display());
    debug!("Modules: {:?}", args.modules);

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

    Ok(())
}
