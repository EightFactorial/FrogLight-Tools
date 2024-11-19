#![allow(clippy::module_inception)]
use froglight_generate::{BlockGenerator, CliArgs, DataMap};

use super::ModuleGenerator;

mod attribute;
mod block;
mod test;
mod vanilla;

impl ModuleGenerator for BlockGenerator {
    /// Generate blocks from the given [`DataMap`].
    async fn generate(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
        if datamap.version_data.is_empty() {
            tracing::warn!("BlockGenerator: No data to generate blocks from!");
            return Ok(());
        }

        // Generate the blocks file.
        block::generate_blocks(datamap, args).await?;

        // Generate the attribute and attribute implementations files.
        let modified = attribute::generate_attributes(datamap, args).await?;
        block::generate_block_impls(datamap, &modified, args).await?;

        // Generate the vanilla block storage.
        vanilla::generate_storage(datamap, args).await?;

        // Generate the test files.
        test::generate_tests(datamap, args).await?;

        Ok(())
    }
}
