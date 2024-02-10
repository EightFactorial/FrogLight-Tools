#![doc = include_str!("../README.md")]

use cargo_metadata::MetadataCommand;
use clap::Parser;
use tracing::trace;

mod command;
use command::GeneratorArgs;

mod config;
use config::GeneratorConfig;

mod logging;
mod modules;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup();

    // Parse the command line arguments
    let args = GeneratorArgs::parse();
    trace!("{args:#?}");

    let config = GeneratorConfig::from_args(&args).await?;
    trace!("{config:#?}");

    // Get the workspace configuration
    let workspace = {
        let mut meta_cmd = MetadataCommand::new();
        meta_cmd.manifest_path(args.directory.join("Cargo.toml"));
        meta_cmd.no_deps().exec()
    }?;

    // Run all generator modules
    modules::run(args, config, workspace).await?;

    Ok(())
}
