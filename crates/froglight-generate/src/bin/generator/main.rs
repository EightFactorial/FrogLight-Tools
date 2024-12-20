//! Generate code for FrogLight
//!
//! Run using `just generate [-v, --verbose]`.

use froglight_generate::{
    modules::EntityGenerator, BlockGenerator, CliArgs, DataMap, RegistryGenerator,
};

mod modules;
use modules::ModuleGenerator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, config) = CliArgs::parse().await?;
    let datamap = DataMap::new(&args, &config).await?;

    // PacketGenerator::generate(&datamap, &args).await?;
    BlockGenerator::generate(&datamap, &args).await?;
    EntityGenerator::generate(&datamap, &args).await?;
    RegistryGenerator::generate(&datamap, &args).await?;

    Ok(())
}
