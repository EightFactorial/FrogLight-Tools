use std::cmp::Ordering;

use anyhow::bail;
use froglight_definitions::MinecraftVersion;
use serde_json::Value;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tracing::error;

use crate::{bundle::ExtractBundle, sources::ExtractModule};

mod codec;
mod fields;
mod registry;

/// A module that extracts packet information and fields.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize_unit_struct,
    Deserialize_unit_struct,
)]
pub struct Packets;

impl ExtractModule for Packets {
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()> {
        // Check if the version is supported
        let min_version = MinecraftVersion::new_pre_release(1, 21, 0, 1).unwrap();
        let cmp = data.manifests.version.compare(data.version, &min_version);
        if cmp.is_none() || cmp.is_some_and(|cmp| cmp == Ordering::Less) {
            bail!("Packet extraction is only supported for versions since \"1.21.0-pre1\"!");
        }

        Packets::packet_json(data).await?;
        Packets::packet_bytecode(data).await
    }
}

impl Packets {
    /// Extract packet ids from `packets.json`.
    async fn packet_json(data: &mut ExtractBundle<'_>) -> anyhow::Result<()> {
        // Get the path to the packet report
        let report_path = data.json_dir.join("reports/packets.json");
        if !report_path.exists() {
            bail!("Error extracting packet ids, \"{}\" does not exist", report_path.display());
        }

        // Directly insert the packet data
        data.output["packets"] =
            serde_json::from_str::<Value>(&tokio::fs::read_to_string(report_path).await?)?;

        Ok(())
    }

    /// Packet registry names and classes
    const PACKET_REGISTRY_CLASSES: [&'static str; 8] = [
        "net/minecraft/network/packet/LoginPackets",
        "net/minecraft/network/packet/CommonPackets",
        "net/minecraft/network/packet/PingPackets",
        "net/minecraft/network/packet/HandshakePackets",
        "net/minecraft/network/packet/CookiePackets",
        "net/minecraft/network/packet/PlayPackets",
        "net/minecraft/network/packet/StatusPackets",
        "net/minecraft/network/packet/ConfigPackets",
    ];

    /// Extract packet fields from bytecode.
    #[allow(clippy::unused_async)]
    async fn packet_bytecode(data: &mut ExtractBundle<'_>) -> anyhow::Result<()> {
        // Get a map of all packets and their classes
        let mut packet_list = Vec::new();
        for registry_class in Self::PACKET_REGISTRY_CLASSES {
            let Some(packets) = Self::packets_in_class(registry_class, data) else {
                bail!("Failed to identify packets for \"{registry_class}\"");
            };
            packet_list.extend(packets);
        }

        // Filter out specific packets
        //
        // "minecraft:bundle" since it is the same as "minecraft:bundle_delimiter"
        packet_list.retain(|(packet, _)| packet != "minecraft:bundle");

        // Append the packet classes to the output
        for (packet, class) in &packet_list {
            if !Self::append_packet_class(packet, class, data) {
                error!("Failed to append packet class to \"{packet}\"");
            }
        }

        // Get packet fields
        let Some(packet_fields) = Self::get_packet_fields(packet_list, data) else {
            bail!("Failed to get packet fields");
        };
        // Append the packet fields to the output
        for (packet, fields) in packet_fields {
            if !Self::append_packet_fields(&packet, fields, data) {
                error!("Failed to append packet fields to \"{packet}\"");
            }
        }

        Ok(())
    }
}
