use froglight_generate::bundle::GenerateVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub(crate) struct GenerateConfig {
    /// A list of versions to generate data for
    #[serde(rename = "version")]
    pub(crate) versions: Vec<GenerateVersion>,
}
