//! TODO

use std::path::{Path, PathBuf};

use froglight_generate::{CliArgs, DataMap, PacketGenerator};
use froglight_parse::{
    file::protocol::{ProtocolStatePackets, ProtocolTypeMap},
    Version,
};

/// The version to generate packets for.
const GENERATE_VERSION: Version = Version::new_release(1, 21, 1);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, _) = CliArgs::parse().await?;
    let datamap =
        DataMap::new_from(&args.cache.unwrap(), &[GENERATE_VERSION], args.redownload).await?;

    if let Some(dataset) = datamap.version_data.get(&GENERATE_VERSION) {
        let output = PathBuf::from(file!()).parent().unwrap().to_path_buf().join("generated");
        tracing::info!("Version: {GENERATE_VERSION}");

        generate_types(&output, &dataset.proto.types).await?;
        for (state, packets) in dataset.proto.packets.iter() {
            generate_packets(state, "clientbound", &output, &packets.clientbound).await?;
            generate_packets(state, "serverbound", &output, &packets.serverbound).await?;
        }
    }

    Ok(())
}

async fn generate_types(directory: &Path, types: &ProtocolTypeMap) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(directory).await?;
    let (generated, _) = PacketGenerator::generate_types(types);

    // If there are no types, skip writing to the file
    if !generated.items.is_empty() {
        let content = prettyplease::unparse(&generated);
        let output = directory.join("protocol_types.rs");
        tracing::info!("Writing: {}", output.display());
        tokio::fs::write(output, &content).await?;
    }

    Ok(())
}

async fn generate_packets(
    state: &str,
    direction: &str,
    directory: &Path,
    packets: &ProtocolStatePackets,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(directory).await?;
    let (generated, _) = PacketGenerator::generate_packets(packets);

    // If there are no packets, skip writing to the file
    if !generated.items.is_empty() {
        let content = prettyplease::unparse(&generated);
        let output = directory.join(format!("{state}_{direction}.rs"));
        tracing::info!("Writing: {}", output.display());
        tokio::fs::write(output, &content).await?;
    }

    Ok(())
}
