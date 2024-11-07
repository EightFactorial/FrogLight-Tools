//! TODO

use froglight_generate::{CliArgs, DataMap, PacketGenerator};

mod modules;
use modules::ModuleGenerator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, config) = CliArgs::parse().await?;
    let datamap = DataMap::new(&args, &config).await?;

    PacketGenerator::generate(&datamap, &args).await?;

    Ok(())
}
