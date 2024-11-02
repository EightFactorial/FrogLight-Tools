//! TODO

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;
mod traits;

/// The Yarn Maven metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarnMavenMetadata {
    /// The group ID.
    #[serde(rename = "groupId")]
    pub group_id: CompactString,
    /// The artifact ID.
    #[serde(rename = "artifactId")]
    pub artifact_id: CompactString,
    /// The versioning information.
    pub versioning: YarnVersioning,
}

/// Information about the versioning of Yarn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarnVersioning {
    /// The latest yarn build.
    pub latest: CompactString,
    /// The latest released yarn build.
    pub release: CompactString,
    /// A list of yarn versions.
    pub versions: YarnVersionList,
    /// The last time the metadata was updated.
    #[serde(rename = "lastUpdated")]
    pub last_updated: u64,
}

/// A list of Yarn versions.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
pub struct YarnVersionList {
    version: Vec<CompactString>,
}
