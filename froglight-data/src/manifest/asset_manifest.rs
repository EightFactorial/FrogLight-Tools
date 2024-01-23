use std::collections::HashMap;

use serde::Deserialize;

/// Information about the assets for a specific version of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetManifest {
    /// A map of file paths to [`AssetObject`]s.
    pub objects: HashMap<String, AssetObject>,
}

/// Information about an asset.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetObject {
    /// The SHA1 hash of the file.
    pub hash: String,
    /// The size of the file.
    pub size: u64,
}

impl AssetObject {
    /// Get the Url for this asset.
    #[must_use]
    pub fn url(&self) -> String {
        format!("https://resources.download.minecraft.net/{}/{}", &self.hash[..2], &self.hash)
    }
}
