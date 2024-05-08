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
    pub asset_index: ReleaseDownload,
    /// Downloads for the client and server JARs.
    pub downloads: ReleaseDownloads,
}

/// Downloads for the client and server JARs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct ReleaseDownloads {
    /// The client JAR download.
    pub client: ReleaseDownload,
    /// The client JAR mappings.
    pub client_mappings: ReleaseDownload,
    /// The server JAR download.
    pub server: ReleaseDownload,
    /// The server JAR mappings.
    pub server_mappings: ReleaseDownload,
}

/// Information about a specific download.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct ReleaseDownload {
    /// The SHA-1 hash of the download.
    pub sha1: CompactString,
    /// The size of the download in bytes.
    pub size: u64,
    /// The URL to download the file from.
    pub url: CompactString,
}
