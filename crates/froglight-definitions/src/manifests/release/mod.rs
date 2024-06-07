use compact_str::CompactString;
use serde::Deserialize;

#[cfg(test)]
mod tests;

/// Information about a specific released
/// [`version`](crate::MinecraftVersion) of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct ReleaseManifest {
    /// Information about the assets for this version.
    #[serde(rename = "assetIndex")]
    pub asset_index: ReleaseItem,
    /// Downloads for the client and server JARs.
    pub downloads: ReleaseDownloads,
}

/// Downloads for the client and server JARs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct ReleaseDownloads {
    /// The client JAR.
    pub client: ReleaseItem,
    /// The client JAR mappings.
    pub client_mappings: ReleaseItem,
    /// The server JAR.
    pub server: ReleaseItem,
    /// The server JAR mappings.
    pub server_mappings: ReleaseItem,
}

/// Information about a specific item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct ReleaseItem {
    /// The SHA-1 hash of the item.
    pub sha1: CompactString,
    /// The size of the item in bytes.
    pub size: u64,
    /// The URL of the item.
    pub url: CompactString,
}
