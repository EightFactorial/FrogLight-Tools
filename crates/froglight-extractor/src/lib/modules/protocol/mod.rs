use std::path::Path;

use froglight_data::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Extract;
use crate::classmap::ClassMap;

mod fields;
mod packets;

/// A module that extracts protocol information.
///
/// This includes things like the possible states and packets.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ProtocolModule;

impl Extract for ProtocolModule {
    async fn extract(
        &self,
        _: &Version,
        classmap: &ClassMap,
        _: &Path,
        output: &mut Value,
    ) -> anyhow::Result<()> {
        let Some(class) = classmap.get("net/minecraft/network/NetworkState") else {
            anyhow::bail!("Could not find NetworkState");
        };

        // Get state and packet information
        let states = packets::get_states(&class);
        let packets = packets::get_packets(&class, &states)?;
        output["protocol"]["states"] = serde_json::to_value(&packets)?;

        // Get packet field information
        let fields = fields::get_fields(&packets, classmap);
        output["protocol"]["fields"] = fields;

        Ok(())
    }
}
