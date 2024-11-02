//! TODO

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use super::manifest::ReleaseType;
use crate::Version;

#[cfg(test)]
mod test;
mod traits;

/// Information about a [`Version`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    /// The version's assets.
    #[serde(rename = "assetIndex")]
    pub asset_index: VersionAssetIndex,
    /// The version's downloads.
    pub downloads: VersionDownloads,
    /// The version id.
    pub id: Version,
    /// The main class of the jar file.
    #[serde(rename = "mainClass")]
    pub main_class: CompactString,
    /// The minimum launcher version required to run the version.
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i32,
    /// The release time of the version.
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// The last time the version was updated.
    pub time: DateTime<Utc>,
    /// The type of the release.
    #[serde(rename = "type")]
    pub kind: ReleaseType,
}

/// Information about the assets file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionAssetIndex {
    /// The assets file version.
    pub id: CompactString,
    /// The assets file sha1 hash.
    pub sha1: CompactString,
    /// The size of the assets file.
    pub size: u64,
    /// The total size of the version.
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    /// The url of the assets file.
    pub url: CompactString,
}

/// Information about the downloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionDownloads {
    /// The client jar download.
    pub client: VersionDownload,
    /// The client mappings download.
    pub client_mappings: VersionDownload,
    /// The server jar download.
    pub server: VersionDownload,
    /// The server mappings download.
    pub server_mappings: VersionDownload,
}

/// Information about a download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionDownload {
    /// The sha1 hash of the download.
    pub sha1: CompactString,
    /// The size of the download.
    pub size: u64,
    /// The url of the download.
    pub url: CompactString,
}
