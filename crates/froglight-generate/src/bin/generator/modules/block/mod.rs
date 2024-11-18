use froglight_generate::{BlockGenerator, CliArgs, DataMap};
use hashbrown::HashSet;

use super::ModuleGenerator;

impl ModuleGenerator for BlockGenerator {
    /// Generate blocks from the given [`DataMap`].
    async fn generate(datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> {
        if datamap.version_data.is_empty() {
            tracing::warn!("BlockGenerator: No data to generate blocks from!");
            return Ok(());
        }

        for block in universal_blocks(datamap) {
            tracing::info!("BlockGenerator: Block \"{block}\" is identical across all versions.");
        }

        Ok(())
    }
}

fn universal_blocks(datamap: &DataMap) -> HashSet<&str> {
    let mut universal = HashSet::new();
    // For all blocks in the first version
    if let Some(data) = datamap.version_data.values().next() {
        for block_data in data.blocks.iter() {
            // If all versions contain it *and* it's identical
            if datamap.version_data.values().all(|data| {
                if let Some(data) = data.blocks.iter().find(|data| data.name == block_data.name) {
                    data == block_data
                } else {
                    false
                }
            }) {
                universal.insert(block_data.name.as_str());
            }
        }
    }
    universal
}
