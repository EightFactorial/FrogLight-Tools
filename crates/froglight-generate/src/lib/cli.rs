use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use tracing_log::{log::LevelFilter, AsTrace};

use crate::config::Config;

/// The command line arguments.
#[derive(Debug, Parser)]
pub struct CliArgs {
    /// The path to the configuration file
    #[clap(short, long)]
    pub config: PathBuf,
    /// The path to the cache directory
    #[clap(long)]
    pub cache: Option<PathBuf>,
    /// The path to the root of the repository
    #[clap(short, long)]
    pub dir: PathBuf,

    /// Whether to redownload all files
    #[clap(short, long)]
    pub redownload: bool,

    /// The verbosity level
    #[clap(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}

impl CliArgs {
    /// Parse the command line arguments and configuration file.
    ///
    /// Also initializes logging.
    #[inline]
    #[expect(clippy::missing_errors_doc)]
    pub async fn parse() -> anyhow::Result<(Self, Config)> {
        // Parse the command line arguments
        let mut args = <Self as clap::Parser>::parse();
        args.init_logging();

        // If no cache directory was provided, try to find one
        if args.cache.is_none() {
            if let Some(cache) = Self::find_cache_dir() {
                args.cache = Some(cache);
            } else {
                anyhow::bail!("Could not find cache directory");
            }
        }

        // Create the cache directory if it doesn't exist
        if let Some(cache) = &args.cache {
            if !cache.exists() {
                tokio::fs::create_dir_all(cache).await?;
            }
        }

        // Parse the configuration file
        let config_str = tokio::fs::read_to_string(&args.config).await?;
        let config: Config = toml::from_str(&config_str)?;

        // Debug print the args and config
        if args.verbosity.log_level_filter().ge(&LevelFilter::Debug) {
            tracing::debug!("{args:#?}");
            tracing::debug!("{config:#?}");
        }

        Ok((args, config))
    }

    /// The name of the folder to store cached files in.
    const CACHE_FOLDER: &'static str = "generate";

    /// Find the cache directory.
    ///
    /// Looks for any folder named `target` in any of the parent directories.
    fn find_cache_dir() -> Option<PathBuf> {
        // Find the folder with a `target` directory
        let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !dir.join("target").is_dir() {
            dir = dir.parent()?.to_path_buf();
        }
        // Append the cache folder
        dir.push("target");
        dir.push(Self::CACHE_FOLDER);
        Some(dir)
    }

    /// Initialize logging.
    fn init_logging(&self) {
        let max_level = self.verbosity.log_level_filter().as_trace();
        tracing_subscriber::fmt().with_max_level(max_level).init();
    }
}
