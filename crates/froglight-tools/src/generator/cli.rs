use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Parser)]
pub(super) struct GenerateArguments {
    #[command(flatten)]
    pub(super) verbose: Verbosity,

    /// Clears the cache and redownloads all data
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub(super) refresh: bool,

    /// The path to the cache directory
    #[arg(long, default_value = "cache")]
    pub(super) cache: PathBuf,

    /// The path to the configuration file
    #[arg(long, default_value = "generator.toml")]
    pub(super) config: PathBuf,

    /// The path to the project directory
    #[arg(short, long)]
    pub(super) dir: PathBuf,
    // The modules used to generate data
    // pub(super) modules: Vec<Modules>,
}
