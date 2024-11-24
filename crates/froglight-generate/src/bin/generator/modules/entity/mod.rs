use froglight_generate::{modules::EntityGenerator, CliArgs, DataMap};

use super::ModuleGenerator;

mod entities;
mod metadata;

impl ModuleGenerator for EntityGenerator {
    /// Generate entities from the given [`DataMap`].
    async fn generate(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
        if datamap.version_data.is_empty() {
            tracing::warn!("EntityGenerator: No data to generate entities from!");
            return Ok(());
        }

        entities::generate_entities(datamap, args).await?;
        metadata::generate_metadata(datamap, args).await?;

        Ok(())
    }
}
