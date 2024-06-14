use std::cmp::Ordering;

use anyhow::bail;
use froglight_definitions::MinecraftVersion;
use hashbrown::HashMap;
use serde_json::Value;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

mod constructor;
mod create;
mod discover;
mod fields;
mod tuple;

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

    /// Extract packet fields from bytecode.
    #[allow(clippy::unused_async)]
    async fn packet_bytecode(data: &mut ExtractBundle<'_>) -> anyhow::Result<()> {
        // Discover packet classes
        let classes = Self::discover_classes(data)?;

        // Extract packet fields
        let packets: HashMap<String, (String, Vec<String>)> = classes
            .into_iter()
            .map(|(key, class)| {
                Self::packet_fields(&class, data).map(|fields| (key, (class, fields)))
            })
            .try_collect()?;

        // Append the packet data to the existing output
        //
        // {
        //     "packets": {
        //         "state": {
        //             "direction": {
        //                 "packet": {
        //                     "class": "packet_class",
        //                     "fields": ["field1", "field2"]
        //                 }
        //             }
        //         }
        //     }
        // }
        for (key, (class, fields)) in packets {
            let packet_data = data.output["packets"].as_object_mut().unwrap();
            // Get the packet states
            for (_state, state_data) in packet_data {
                // Get the directions for the state
                let states = state_data.as_object_mut().unwrap();
                for (_direction, direction_data) in states {
                    // Get the packets for the direction
                    let packets = direction_data.as_object_mut().unwrap();
                    if let Some(packet_data) = packets.get_mut(&key) {
                        // Insert the packet data
                        packet_data["class"] = class.clone().into();
                        packet_data["fields"] = fields.clone().into();
                    }
                }
            }
        }

        Ok(())
    }
}
