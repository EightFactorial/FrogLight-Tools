use froglight_generate::{CliArgs, DataMap, PacketGenerator};

use super::ModuleGenerator;

impl ModuleGenerator for PacketGenerator {
    /// Generate packets from the given [`DataMap`].
    async fn generate(_datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> { todo!() }
}
