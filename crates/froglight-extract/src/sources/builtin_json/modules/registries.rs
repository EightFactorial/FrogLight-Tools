use anyhow::bail;
use serde_json::Value;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts registries and registry data.
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
pub struct Registries;

impl ExtractModule for Registries {
    async fn extract(&self, data: &mut ExtractBundle) -> anyhow::Result<()> {
        // Get the path to the registry report
        let report_path = data.json_dir.join("reports/registries.json");
        if !report_path.exists() {
            bail!("Error extracting registry data, \"{}\" does not exist", report_path.display());
        }

        // Directly insert the registry data
        data.output["registries"] =
            serde_json::from_str::<Value>(&tokio::fs::read_to_string(report_path).await?)?;

        Ok(())
    }
}
