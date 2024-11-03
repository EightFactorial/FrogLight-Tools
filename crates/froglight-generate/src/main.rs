//! TODO
#![allow(unreachable_pub)]

mod cli;
use cli::CliArgs;

mod datamap;
use datamap::DataMap;
use generator::PacketGenerator;

mod config;

mod generator;
// use generator::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, config) = CliArgs::parse().await?;
    let datamap = DataMap::new(&args, &config).await?;

    PacketGenerator::generate(&datamap, &args).await?;

    Ok(())
}
