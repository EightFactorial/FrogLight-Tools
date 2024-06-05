use froglight_definitions::MinecraftVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub(crate) struct GenerateConfig {
    /// A list of versions to generate data for
    #[serde(rename = "version")]
    pub(crate) versions: Vec<GenerateVersion>,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub(crate) struct GenerateVersion {
    /// The version to generate data for
    pub(crate) base: MinecraftVersion,
    /// The jar to use to generate data
    pub(crate) jar: MinecraftVersion,
}
