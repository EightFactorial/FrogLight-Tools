use std::cmp::Ordering;

use anyhow::bail;
use froglight_definitions::MinecraftVersion;
use hashbrown::HashMap;
use serde_json::Value;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tracing::error;

use crate::{bundle::ExtractBundle, sources::ExtractModule};

mod codec;
mod discover;
mod method;
mod parse;

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

        // Directly insert the packet report data
        data.output["packets"] =
            serde_json::from_str::<Value>(&tokio::fs::read_to_string(report_path).await?)?;

        Ok(())
    }

    /// Extract packet fields from bytecode.
    #[allow(clippy::unused_async)]
    async fn packet_bytecode(data: &mut ExtractBundle<'_>) -> anyhow::Result<()> {
        // Discover packet classes
        let classes = Self::discover_classes(data)?;

        // Get packet fields
        let packet_data = Self::parse(classes, data)?;
        // Append data to output
        Self::append_bytecode_info(packet_data, data);

        Ok(())
    }

    // Append the packet data to the existing output
    //
    // {
    //     "packets": {
    //         "state": {
    //             "direction": {
    //                 // Insert data matching this key
    //                 "some:packet": {
    //                     "class": "packet_class",
    //                     "fields": ["type1", "type2"]
    //                 }
    //             }
    //         }
    //     }
    // }
    fn append_bytecode_info(
        packet_data: HashMap<String, (String, Vec<String>)>,
        data: &mut ExtractBundle<'_>,
    ) {
        let output_packets = data.output["packets"].as_object_mut().unwrap();

        // Get the packet states
        for (_state, state_data) in output_packets {
            // Get the directions for the state
            //
            let states = state_data.as_object_mut().unwrap();
            for (_direction, direction_data) in states {
                // Get the packets for the direction
                //
                let packets = direction_data.as_object_mut().unwrap();
                for (packet_key, data) in packets {
                    // Check if any data was found for this packet
                    //
                    if let Some((class, fields)) = packet_data.get(packet_key) {
                        // Insert the class and fields
                        data["class"] = Value::String(class.clone());
                        data["fields"] =
                            Value::Array(fields.iter().cloned().map(Value::String).collect());
                    } else {
                        error!("Failed to find packet data for \"{packet_key}\"");
                    }
                }
            }
        }
    }
}
