use anyhow::bail;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tracing::error;

use crate::{bundle::ExtractBundle, sources::ExtractModule};

mod report;
pub use report::{BlockData, BlockState, BlocksReport};

/// A module that extracts blocks and block data.
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
    Deserialize_unit_struct,
    Serialize_unit_struct,
)]
pub struct Blocks;

impl ExtractModule for Blocks {
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()> {
        Blocks::block_json(data).await?;
        Blocks::block_bytecode(data).await
    }
}

impl Blocks {
    /// Extract block states and state ids from `blocks.json`.
    async fn block_json(data: &mut ExtractBundle<'_>) -> anyhow::Result<()> {
        // Get the path to the block report
        let report_path = data.json_dir.join("reports/blocks.json");
        if !report_path.exists() {
            bail!(
                "Error extracting block state data, \"{}\" does not exist",
                report_path.display()
            );
        }

        // Parse the report
        let mut report: BlocksReport =
            serde_json::from_str(&tokio::fs::read_to_string(report_path).await?)?;

        // Append blockstates in order of their id
        let mut target_id = 0u32;
        while let Some((name, target)) = report
            .iter_mut()
            .find(|(_, data)| data.states.iter().any(|state| state.id == target_id))
        {
            // Sort the states by id
            target.states.sort_by(|a, b| a.id.cmp(&b.id));

            // Update the target id
            let last = target.states.last().unwrap();
            target_id = last.id + 1;

            // Insert the target into final output
            data.output["blocks"][name] = serde_json::to_value(target)?;
        }

        // Check if all block states were extracted into the final output
        if report.len() != data.output["blocks"].as_object().unwrap().len() {
            error!(
                "Report: {}, Output: {}",
                report.len(),
                data.output["blocks"].as_object().unwrap().len()
            );
            bail!("Error extracting block state data, some block states are missing!");
        }

        Ok(())
    }

    /// Extract block properties and data from bytecode.
    #[allow(clippy::unused_async)]
    async fn block_bytecode(_data: &mut ExtractBundle<'_>) -> anyhow::Result<()> { Ok(()) }
}
