use std::cmp::Ordering;

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::Deserialize;

use crate::MinecraftVersion;

#[cfg(test)]
mod tests;

/// A manifest of all [`MinecraftVersions`](MinecraftVersion).
#[derive(Debug, Clone, Hash, Deserialize)]
pub struct ReleasesManifest {
    /// The latest versions.
    pub latest: ReleasesLatest,
    /// All versions.
    pub versions: Vec<ReleasesManifestEntry>,
}

impl ReleasesManifest {
    /// Get the latest [`MinecraftVersion::Release`].
    #[must_use]
    pub fn latest_release(&self) -> &MinecraftVersion { &self.latest.release }

    /// Get the latest non-release [`MinecraftVersion`].
    ///
    /// This can be a [`MinecraftVersion::ReleaseCandidate`],
    /// [`MinecraftVersion::PreRelease`], or [`MinecraftVersion::Snapshot`].
    #[must_use]
    pub fn latest_snapshot(&self) -> &MinecraftVersion { &self.latest.snapshot }

    /// Get the latest [`MinecraftVersion`].
    #[must_use]
    pub fn latest_version(&self) -> &MinecraftVersion {
        if Some(Ordering::Greater) == self.compare(&self.latest.release, &self.latest.snapshot) {
            &self.latest.release
        } else {
            &self.latest.snapshot
        }
    }

    /// Compare any two [`MinecraftVersions`](MinecraftVersion) using the
    /// manifest.
    ///
    /// Returns `None` if either version is not found.
    #[must_use]
    pub fn compare(&self, rhs: &MinecraftVersion, lhs: &MinecraftVersion) -> Option<Ordering> {
        let rhs = &self.versions.iter().find(|&x| rhs.is_same(&x.id))?.release_time;
        let lhs = &self.versions.iter().find(|&x| lhs.is_same(&x.id))?.release_time;
        Some(rhs.cmp(lhs))
    }
}

/// The latest [`MinecraftVersions`](MinecraftVersion) in a
/// [`ReleasesManifest`].
#[derive(Debug, Clone, Hash, Deserialize)]
pub struct ReleasesLatest {
    /// The latest release.
    pub release: MinecraftVersion,
    /// The latest snapshot.
    pub snapshot: MinecraftVersion,
}

/// A single entry in a [`ReleasesManifest`].
#[derive(Debug, Clone, Hash, Deserialize)]
pub struct ReleasesManifestEntry {
    /// The [`MinecraftVersion`].
    pub id: MinecraftVersion,
    /// The type of version.
    #[serde(rename = "type")]
    pub kind: CompactString,
    /// The URL to the [`VersionManifest`].
    pub url: CompactString,
    /// The time this version was published.
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
}
