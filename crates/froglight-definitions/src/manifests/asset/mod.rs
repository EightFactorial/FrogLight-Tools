use compact_str::CompactString;
use hashbrown::HashMap;
use serde::Deserialize;

#[cfg(test)]
mod tests;

/// Information about the assets for a
/// [`version`](crate::MinecraftVersion) of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AssetManifest {
    /// The objects in the asset manifest.
    pub objects: HashMap<CompactString, AssetObject>,
}

/// Information about a specific asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct AssetObject {
    /// The SHA-1 hash of the asset.
    pub hash: CompactString,
    /// The size of the asset in bytes.
    pub size: u64,
}

impl AssetObject {
    /// Get the hash of an asset by its path.
    #[must_use]
    pub fn get_url(&self) -> String { url_from_hash(&self.hash) }
}

/// Get the URL of an asset using its hash.
#[must_use]
pub fn url_from_hash(hash: &str) -> String {
    format!("https://resources.download.minecraft.net/{}/{}", &hash[..2], &hash)
}
