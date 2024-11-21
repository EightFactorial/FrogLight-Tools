use derive_more::derive::{Deref, DerefMut};
use froglight_generate::{CliArgs, DataMap, PacketGenerator};
use hashbrown::HashMap;

use super::ModuleGenerator;

mod common;
mod process;

impl ModuleGenerator for PacketGenerator {
    /// Generate packets from the given [`DataMap`].
    async fn generate(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
        if datamap.version_data.is_empty() {
            tracing::warn!("PacketGenerator: No data to generate packets from!");
            return Ok(());
        }

        common::generate_common(datamap, args).await?;

        Ok(())
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub(super) struct GeneratedTypes {
    /// Stored as (protocol name, (module, item name))
    pub(super) items: HashMap<String, (String, String)>,
}
