//! TODO

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

use crate::Version;

#[cfg(test)]
mod test;
mod traits;

/// A manifest of versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionManifest {
    /// The latest versions in the manifest.
    pub latest: VersionManifestLatest,
    /// The versions in the manifest.
    pub versions: VersionManifestMap,
}

/// The latest [`Version`]s in the [`VersionManifest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionManifestLatest {
    /// The latest release version.
    pub release: Version,
    /// The latest snapshot version.
    pub snapshot: Version,
}

/// A map of [`Version`]s to their [`VersionManifestData`].
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct VersionManifestMap(HashMap<Version, VersionManifestData>);

impl Serialize for VersionManifestMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for value in self.values() {
            seq.serialize_element(value)?;
        }
        seq.end()
    }
}
impl<'de> Deserialize<'de> for VersionManifestMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<VersionManifestData>::deserialize(deserializer).map(|vec| {
            VersionManifestMap(vec.into_iter().fold(HashMap::new(), |mut map, data| {
                map.insert(data.id.clone(), data);
                map
            }))
        })
    }
}

/// The data for a [`Version`] in the [`VersionManifest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionManifestData {
    /// The version of the game this data is for.
    pub id: Version,
    /// The type of release.
    #[serde(rename = "type")]
    pub kind: ReleaseType,
    /// The URL to the version info file.
    pub url: CompactString,
    /// The last time the version files were updated.
    pub time: DateTime<Utc>,
    /// The release time of the version.
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// The SHA-1 hash of the version info file.
    pub sha1: CompactString,
    /// If the version has the latest player safety features.
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u32,
}

/// The type of release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseType {
    /// A release version.
    Release,
    /// A snapshot version.
    Snapshot,
    /// An old beta version.
    #[serde(rename = "old_beta")]
    OldBeta,
    /// An old alpha version.
    #[serde(rename = "old_alpha")]
    OldAlpha,
}
