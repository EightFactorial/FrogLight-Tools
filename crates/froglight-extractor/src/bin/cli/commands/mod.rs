#![allow(clippy::unused_async)]

use std::{convert::Infallible, path::PathBuf};

use clap::{ArgAction, Args, Parser, Subcommand};
use froglight_data::Version;
use froglight_extractor::modules::ExtractModule;

pub(super) mod extract;
pub(super) mod print;
pub(super) mod search;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub(crate) struct ExtractCommand {
    /// Disable/enable logging.
    ///
    /// By default, logging is enabled.
    #[arg(short = 'q', long, default_value = "true", action = ArgAction::SetFalse)]
    pub(crate) verbose: bool,

    /// Redownload all cached manifest files.
    ///
    /// This includes the version manifest, the release manifest, and the asset
    /// manifest if relevant.
    #[arg(short = 'r', long = "refresh")]
    pub(crate) refresh: bool,

    /// The cache directory to use.
    ///
    /// If you've used the `froglight-generator` tool, you can use
    /// the project's `target` directory as the cache directory.
    #[arg(short = 'c', long = "cache")]
    pub(crate) cache: PathBuf,

    /// The version to extract data from.
    ///
    /// For example, `1.20`, `1.20.0`, `1.20.2` and `1.20.4-pre1`
    /// are all valid versions.
    #[arg(short, long, value_parser = version_infallible)]
    pub(crate) version: Version,

    /// The subcommand to run.
    #[command(subcommand)]
    pub(crate) subcommand: ExtractSubCommand,

    /// An optional output file to write the result to.
    ///
    /// If not provided, the result will be printed to the console.
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(crate) enum ExtractSubCommand {
    /// Extract data from the version.
    Extract(ExtractArgs),
    /// Search for data in the version.
    Search(SearchArgs),
    /// Print a class from the version.
    Print(PrintArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct ExtractArgs {
    /// A list of modules to use for extracting data.
    #[arg(short, long)]
    pub(crate) modules: Vec<ExtractModule>,
}

/// Forcefully parse the version as a `Version`.
#[allow(clippy::unnecessary_wraps)]
fn version_infallible(s: &str) -> Result<Version, Infallible> {
    Ok(Version::from_string(s).unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct SearchArgs {
    /// The query to search for.
    pub(crate) query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(crate) struct PrintArgs {
    /// The class to print.
    pub(crate) class: String,
}
