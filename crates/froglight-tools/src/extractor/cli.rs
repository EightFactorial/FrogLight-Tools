use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use froglight_definitions::MinecraftVersion;
use froglight_extract::sources::Modules;

#[derive(Debug, Parser)]
pub(super) struct ExtractArguments {
    #[command(flatten)]
    pub(super) verbose: Verbosity,

    /// Clears the cache and redownloads all data
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub(super) refresh: bool,

    /// The path to the cache directory
    #[arg(long, default_value = "cache")]
    pub(super) cache: PathBuf,

    /// The path to the output file
    ///
    /// If not specified, output will be written to stdout
    #[arg(short, long)]
    pub(super) output: Option<PathBuf>,

    /// The version to extract data from
    #[arg(long)]
    pub(super) version: MinecraftVersion,

    /// The modules used to extract data
    pub(super) modules: Vec<Modules>,
}
