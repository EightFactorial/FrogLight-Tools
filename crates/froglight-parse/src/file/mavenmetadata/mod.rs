//! TODO

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;
mod traits;

/// The Maven metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MavenMetadata {
    /// The group ID.
    #[serde(rename = "groupId")]
    pub group_id: CompactString,
    /// The artifact ID.
    #[serde(rename = "artifactId")]
    pub artifact_id: CompactString,
    /// The versioning information.
    pub versioning: MetadataVersioning,
}

/// The versioning information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataVersioning {
    /// The latest mappings build.
    pub latest: CompactString,
    /// The latest released mappings build.
    pub release: CompactString,
    /// A list of mapping versions.
    pub versions: VersionList,
    /// The last time the metadata was updated.
    #[serde(rename = "lastUpdated")]
    pub last_updated: u64,
}

/// A list of mapping versions.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
pub struct VersionList {
    version: Vec<CompactString>,
}
