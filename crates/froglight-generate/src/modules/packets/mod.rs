use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        bytecode::{BytecodeModule, Packets as ExtractPackets},
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use super::sealed::GenerateRequired;
use crate::{bundle::GenerateBundle, modules::GenerateModule};

/// A module that generates states and packets.
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

impl GenerateRequired for Packets {
    const REQUIRED: &'static [ExtractModules] =
        &[ExtractModules::Bytecode(BytecodeModule::Packets(ExtractPackets))];
}

impl GenerateModule for Packets {
    /// Run the generation process.
    async fn generate(
        &self,
        _generate: &GenerateBundle<'_>,
        _extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
