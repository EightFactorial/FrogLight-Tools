//! Generate blocks and block attributes for a specific version.
//!
//! Run using `just example block [-v, --verbose]`.

use std::path::{Path, PathBuf};

use froglight_generate::{BlockGenerator, CliArgs, DataMap};
use froglight_parse::{file::VersionBlocks, Version};

/// The version to generate packets for.
const GENERATE_VERSION: Version = Version::new_release(1, 21, 1);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, _) = CliArgs::parse().await?;
    let datamap =
        DataMap::new_from(&args.cache.unwrap(), &[GENERATE_VERSION], args.redownload).await?;

    if let Some(dataset) = datamap.version_data.get(&GENERATE_VERSION) {
        let output = PathBuf::from(file!()).parent().unwrap().to_path_buf().join("generated");
        tracing::info!("Version: v{GENERATE_VERSION}");

        generate_blocks(&output, &dataset.blocks).await?;
    }

    Ok(())
}

async fn generate_blocks(directory: &Path, blocks: &VersionBlocks) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(directory).await?;
    let (attrib, blocks) = BlockGenerator::generate_blocks(blocks);

    let content = prettyplease::unparse(&attrib);
    let output = directory.join("attributes.rs");
    tracing::info!("Writing: {}", output.display());
    tokio::fs::write(output, &content).await?;

    let content = prettyplease::unparse(&blocks);
    let output = directory.join("blocks.rs");
    tracing::info!("Writing: {}", output.display());
    tokio::fs::write(output, &content).await?;

    Ok(())
}
