use std::path::PathBuf;

use clap::Parser;
use froglight_definitions::MinecraftVersion;
use froglight_extract::sources::Modules;

#[derive(Debug, Parser)]
pub(super) struct ExtractArguments {
    #[command(flatten)]
    pub(super) verbose: clap_verbosity_flag::Verbosity,

    /// The path to the cache directory
    #[arg(short, long, default_value = "cache")]
    pub(super) cache: PathBuf,

    /// The version to extract data from
    #[arg(long = "version")]
    pub(super) version: MinecraftVersion,

    /// The modules used to extract data
    pub(super) modules: Vec<Modules>,
}
