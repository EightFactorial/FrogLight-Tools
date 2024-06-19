use anyhow::bail;
use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        bytecode::{BytecodeModule, Packets as ExtractPackets},
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tracing::debug;

use super::sealed::GenerateRequired;
use crate::{bundle::GenerateBundle, helpers::update_tag, modules::GenerateModule};

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

impl Packets {
    /// The path to the `froglight-protocol` crate,
    /// relative to the root directory.
    const CRATE_PATH: &'static str = "crates/froglight-protocol/src";
}

impl GenerateRequired for Packets {
    const REQUIRED: &'static [ExtractModules] =
        &[ExtractModules::Bytecode(BytecodeModule::Packets(ExtractPackets))];
}

impl GenerateModule for Packets {
    /// Run the generation process.
    async fn generate(
        &self,
        generate: &GenerateBundle<'_>,
        _extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        let src_path = generate.root_dir.join(Self::CRATE_PATH);

        if !src_path.is_dir() {
            bail!("Could not find `froglight-protocol` crate at \"{}\"!", src_path.display());
        }
        debug!("Found `froglight-protocol` crate at \"{}\"", src_path.display());

        let ver_path = src_path.join("versions/mod.rs");
        update_tag(&ver_path).await?;

        Ok(())
    }
}
