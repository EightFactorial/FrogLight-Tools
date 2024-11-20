use froglight_generate::{CliArgs, DataMap, RegistryGenerator};

use super::ModuleGenerator;

impl ModuleGenerator for RegistryGenerator {
    /// Generate packets from the given [`DataMap`].
    async fn generate(_datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> { Ok(()) }
}
