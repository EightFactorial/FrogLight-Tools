use serde::{Deserialize, Serialize};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A placeholder module.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Placeholder;

impl ExtractModule for Placeholder {
    /// Run the extraction process.
    async fn extract<'a>(&'a self, _: ExtractBundle<'a>) -> anyhow::Result<()> { Ok(()) }
}
