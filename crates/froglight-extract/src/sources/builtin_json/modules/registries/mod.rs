use serde::{Deserialize, Serialize};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts registries and registry data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Registries;

impl ExtractModule for Registries {
    /// Run the extraction process.
    async fn extract<'a>(&'a self, _data: ExtractBundle<'a>) -> anyhow::Result<()> { Ok(()) }
}
