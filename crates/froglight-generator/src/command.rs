use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub(crate) struct GeneratorArgs {
    /// The root directory of the FrogLight project
    ///
    /// From inside the tools submodule, this should be "../"
    #[arg(short = 'd', long = "dir")]
    pub(crate) directory: PathBuf,

    /// The path to the generator config file
    ///
    /// By default, this is "generator.toml" in the root directory
    #[arg(short = 'c', long = "config", default_value = "generator.toml")]
    pub(crate) config: PathBuf,
}
