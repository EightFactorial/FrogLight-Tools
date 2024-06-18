#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use anyhow::anyhow;
use clap::Parser;
use froglight_extract::sources::Modules;
use froglight_tools::logging;
use tracing::{debug, error, info, warn};

mod cli;
use cli::ExtractArguments;

mod extract;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = ExtractArguments::parse();
    logging::setup(&args.verbose);

    // If no modules are specified, use the default
    if args.modules.is_empty() {
        args.modules.extend(Modules::DEFAULT);
    }

    // Debugging information
    info!("Version: \"{}\"", args.version);
    debug!("Cache: \"{}\"", args.cache.display());
    debug!("Modules: {:?}", args.modules);

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

    // Extract the data
    let output_value = extract::extract(&args).await?;
    let output_string = serde_json::to_string_pretty(&output_value)?;

    // Write the output to a file or print it
    if let Some(output_path) = args.output {
        info!("Writing results to: \"{}\"", output_path.display());
        tokio::fs::write(&output_path, output_string).await?;
    } else {
        println!("{output_string}");
    }

    Ok(())
}
