//! TODO

use std::path::PathBuf;

use froglight_generate::{CliArgs, DataMap, PacketGenerator};
use froglight_parse::{file::protocol::ProtocolStatePackets, Version};

/// The version to generate packets for.
const GENERATE_VERSION: Version = Version::new_release(1, 21, 1);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, _) = CliArgs::parse().await?;
    let datamap =
        DataMap::new_from(&args.cache.unwrap(), &[GENERATE_VERSION], args.redownload).await?;

    if let Some(dataset) = datamap.version_data.get(&GENERATE_VERSION) {
        tracing::info!("Version: {GENERATE_VERSION}");
        for (state, packets) in dataset.proto.packets.iter() {
            generate_packets(state, "clientbound", &packets.clientbound).await?;
            generate_packets(state, "serverbound", &packets.serverbound).await?;
        }
    }

    Ok(())
}

async fn generate_packets(
    state: &str,
    direction: &str,
    packets: &ProtocolStatePackets,
) -> anyhow::Result<()> {
    // Create the output directory
    let output_dir = PathBuf::from(file!()).parent().unwrap().to_path_buf().join("generated");
    tokio::fs::create_dir_all(&output_dir).await?;

    // Generate the packets
    let (generated, _) = PacketGenerator::generate_state_file(packets);

    // If there are no packets, skip writing to the file
    if generated.items.is_empty() {
        Ok(())
    } else {
        let output = output_dir.join(format!("{state}_{direction}.rs"));
        tracing::info!("Writing: {}", output.display());
        tokio::fs::write(output, prettyplease::unparse(&generated)).await?;
        Ok(())
    }
}
