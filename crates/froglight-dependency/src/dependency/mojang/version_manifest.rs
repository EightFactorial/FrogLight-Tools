//! TODO

use chrono::{DateTime, Utc};
use froglight_tool_macros::Dependency;
use serde::{Deserialize, Serialize};

use crate::{container::DependencyContainer, version::Version};

/// A manifest containing information about all Minecraft versions.
#[derive(Debug, Clone, PartialEq, Eq, Dependency, Serialize, Deserialize)]
#[dep(path = crate, retrieve = Self::retrieve)]
pub struct VersionManifest {
    /// The latest versions.
    pub latest: VersionManifestLatest,
    /// The list of all versions.
    pub versions: Vec<VersionManifestEntry>,
}

/// The latest release and snapshot versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionManifestLatest {
    /// The latest release.
    pub release: Version,
    /// The latest snapshot.
    pub snapshot: Version,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[expect(missing_docs)]
pub struct VersionManifestEntry {
    /// The [`Version`].
    pub id: Version,
    #[serde(rename = "type")]
    pub version_type: String,
    /// The URL to the version's [`ReleaseManifest`](super::ReleaseManifest).
    pub url: String,
    /// The last time the manifest was updated.
    pub time: DateTime<Utc>,
    /// When the version was released.
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// The SHA1 hash of the [`ReleaseManifest`](super::ReleaseManifest).
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: u8,
}

impl VersionManifest {
    const MANIFEST_URL: &'static str =
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    const MANIFEST_FILENAME: &'static str = "version_manifest_v2.json";

    async fn retrieve(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let file = deps.cache.join(Self::MANIFEST_FILENAME);

        if tokio::fs::try_exists(&file).await? {
            tracing::debug!("Reading \"{}\"", file.display());

            // Read the file from disk and parse it
            let content = tokio::fs::read(&file).await?;
            serde_json::from_slice(&content).map_err(Into::into)
        } else {
            tracing::debug!("Retrieving \"{}\"", Self::MANIFEST_URL);

            // Download the file, save it to disk, and parse it
            let response = deps.client.get(Self::MANIFEST_URL).send().await?.bytes().await?;
            tokio::fs::write(&file, &response).await?;
            serde_json::from_slice(&response).map_err(Into::into)
        }
    }

    /// Get the [`VersionManifestEntry`] for the latest release.
    #[inline]
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn latest_release(&self) -> &VersionManifestEntry {
        self.get(&self.latest.release).expect("No Entry for `latest.release` found!")
    }

    /// Get the [`VersionManifestEntry`] for the latest snapshot.
    #[inline]
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn latest_snapshot(&self) -> &VersionManifestEntry {
        self.get(&self.latest.snapshot).expect("No Entry for `latest.snapshot` found!")
    }

    /// Get the [`VersionManifestEntry`] for a specific [`Version`].
    #[must_use]
    pub fn get(&self, version: &Version) -> Option<&VersionManifestEntry> {
        self.versions.iter().find(|v| &v.id == version)
    }

    /// Compare two [`Version`]s based on their release times.
    #[must_use]
    pub fn compare(&self, a: &Version, b: &Version) -> Option<std::cmp::Ordering> {
        match (self.get(a), self.get(b)) {
            (Some(a), Some(b)) => Some(a.release_time.cmp(&b.release_time)),
            _ => None,
        }
    }
}

#[test]
#[cfg(test)]
fn parse() {
    let example: VersionManifest = serde_json::from_str(TRIMMED_EXAMPLE).unwrap();
    assert_eq!(example.latest_release().sha1, "c440b9ef34fec9d69388de8650cd55b465116587");
    assert_eq!(example.latest_snapshot().sha1, "af26a4b3605f891007f08000846909840e80784a");
}

#[cfg(test)]
const TRIMMED_EXAMPLE: &str = r#"{
  "latest": {
    "release": "1.21.4",
    "snapshot": "25w05a"
  },
  "versions": [
    {
      "id": "25w05a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/af26a4b3605f891007f08000846909840e80784a/25w05a.json",
      "time": "2025-01-29T14:14:42+00:00",
      "releaseTime": "2025-01-29T14:03:54+00:00",
      "sha1": "af26a4b3605f891007f08000846909840e80784a",
      "complianceLevel": 1
    },
    {
      "id": "25w04a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/ee39eea919be9d3b90a0af509897d03f9317b84d/25w04a.json",
      "time": "2025-01-24T15:41:24+00:00",
      "releaseTime": "2025-01-22T13:14:44+00:00",
      "sha1": "ee39eea919be9d3b90a0af509897d03f9317b84d",
      "complianceLevel": 1
    },
    {
      "id": "25w03a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/efc1427211f7dc0756f6c77d436dc5e7206246a4/25w03a.json",
      "time": "2025-01-24T15:41:24+00:00",
      "releaseTime": "2025-01-15T14:28:04+00:00",
      "sha1": "efc1427211f7dc0756f6c77d436dc5e7206246a4",
      "complianceLevel": 1
    },
    {
      "id": "25w02a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/713542cedfa0d1696416a8c96345e4abb4c37463/25w02a.json",
      "time": "2025-01-24T06:35:25+00:00",
      "releaseTime": "2025-01-08T13:42:18+00:00",
      "sha1": "713542cedfa0d1696416a8c96345e4abb4c37463",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4",
      "type": "release",
      "url": "https://piston-meta.mojang.com/v1/packages/c440b9ef34fec9d69388de8650cd55b465116587/1.21.4.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-12-03T10:12:57+00:00",
      "sha1": "c440b9ef34fec9d69388de8650cd55b465116587",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4-rc3",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/1fa7edc9fe16ed00dfffdbbb300c45a11af7ecfe/1.21.4-rc3.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-29T17:02:53+00:00",
      "sha1": "1fa7edc9fe16ed00dfffdbbb300c45a11af7ecfe",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4-rc2",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/55b44c5f9f196535af50e206192858854294856a/1.21.4-rc2.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-29T10:33:13+00:00",
      "sha1": "55b44c5f9f196535af50e206192858854294856a",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4-rc1",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/de365e5d43a8f816ce905c6f1a78101b256186b9/1.21.4-rc1.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-28T10:19:01+00:00",
      "sha1": "de365e5d43a8f816ce905c6f1a78101b256186b9",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4-pre3",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/ee7e2bf0b97f30b1311aad8ddbd20e6e9f26ff2a/1.21.4-pre3.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-26T15:07:29+00:00",
      "sha1": "ee7e2bf0b97f30b1311aad8ddbd20e6e9f26ff2a",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4-pre2",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/aee63ab6513fb2e19b91e9c81b5efcdf9c62070f/1.21.4-pre2.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-25T13:18:35+00:00",
      "sha1": "aee63ab6513fb2e19b91e9c81b5efcdf9c62070f",
      "complianceLevel": 1
    },
    {
      "id": "1.21.4-pre1",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/34d6efe08069c279e9ac2299d1bd747e661f66d3/1.21.4-pre1.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-20T13:45:00+00:00",
      "sha1": "34d6efe08069c279e9ac2299d1bd747e661f66d3",
      "complianceLevel": 1
    },
    {
      "id": "24w46a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/7538cfc142e32cdc3da84a32ac0b0021e5f8ef95/24w46a.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-13T13:12:38+00:00",
      "sha1": "7538cfc142e32cdc3da84a32ac0b0021e5f8ef95",
      "complianceLevel": 1
    },
    {
      "id": "24w45a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/b6d13e5d58a7e4900b75a1041a9c36fc2b3c97fd/24w45a.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-11-06T13:31:58+00:00",
      "sha1": "b6d13e5d58a7e4900b75a1041a9c36fc2b3c97fd",
      "complianceLevel": 1
    },
    {
      "id": "24w44a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/75356811656e584f53be7a9aa7db0b94a0c0aec8/24w44a.json",
      "time": "2025-01-24T06:34:26+00:00",
      "releaseTime": "2024-10-30T12:53:55+00:00",
      "sha1": "75356811656e584f53be7a9aa7db0b94a0c0aec8",
      "complianceLevel": 1
    },
    {
      "id": "1.21.3",
      "type": "release",
      "url": "https://piston-meta.mojang.com/v1/packages/b64c551553e59c369f4a3529b15c570ac6b9b73e/1.21.3.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-23T12:28:15+00:00",
      "sha1": "b64c551553e59c369f4a3529b15c570ac6b9b73e",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2",
      "type": "release",
      "url": "https://piston-meta.mojang.com/v1/packages/9f5bf4a9654fcff8cd17302846fe60e2f8e68415/1.21.2.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-22T09:58:55+00:00",
      "sha1": "9f5bf4a9654fcff8cd17302846fe60e2f8e68415",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-rc2",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/c6dd15a3806bd22296a532d619e5178bc683a40d/1.21.2-rc2.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-21T15:53:05+00:00",
      "sha1": "c6dd15a3806bd22296a532d619e5178bc683a40d",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-rc1",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/2bd9f66740a9bc86b96a2ba807b66a244fdc866f/1.21.2-rc1.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-17T12:43:18+00:00",
      "sha1": "2bd9f66740a9bc86b96a2ba807b66a244fdc866f",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-pre5",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/786597de32f2d2b8b12fc700df78c81e2361ab6b/1.21.2-pre5.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-16T13:30:35+00:00",
      "sha1": "786597de32f2d2b8b12fc700df78c81e2361ab6b",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-pre4",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/8876286477bd917381fd85fa7c2449b8312bda83/1.21.2-pre4.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-15T11:59:11+00:00",
      "sha1": "8876286477bd917381fd85fa7c2449b8312bda83",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-pre3",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/ac998b80619917d7bfd6c4f5f334bc793f49d988/1.21.2-pre3.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-11T12:32:27+00:00",
      "sha1": "ac998b80619917d7bfd6c4f5f334bc793f49d988",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-pre2",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/45f98dfc4a2f0fcae3bb077132b7df512fea8b5d/1.21.2-pre2.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-10T12:59:14+00:00",
      "sha1": "45f98dfc4a2f0fcae3bb077132b7df512fea8b5d",
      "complianceLevel": 1
    },
    {
      "id": "1.21.2-pre1",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/c43b93a23e61ed4dd35f69f9038a29f05e51b15f/1.21.2-pre1.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-08T13:22:12+00:00",
      "sha1": "c43b93a23e61ed4dd35f69f9038a29f05e51b15f",
      "complianceLevel": 1
    },
    {
      "id": "24w40a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/de75c7916d51d96889ddcf1f5284e637db3162fe/24w40a.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-10-02T13:15:42+00:00",
      "sha1": "de75c7916d51d96889ddcf1f5284e637db3162fe",
      "complianceLevel": 1
    },
    {
      "id": "24w39a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/3518c37d168da68622a350cb72bcb81493cf6153/24w39a.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-09-25T13:08:41+00:00",
      "sha1": "3518c37d168da68622a350cb72bcb81493cf6153",
      "complianceLevel": 1
    },
    {
      "id": "24w38a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/d3583ad06df6c1ca6f73a785351baa1bd03f1016/24w38a.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-09-18T12:32:07+00:00",
      "sha1": "d3583ad06df6c1ca6f73a785351baa1bd03f1016",
      "complianceLevel": 1
    },
    {
      "id": "24w37a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/c268069f2b8a2da25feb43b147a21d257c368c51/24w37a.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-09-11T13:01:31+00:00",
      "sha1": "c268069f2b8a2da25feb43b147a21d257c368c51",
      "complianceLevel": 1
    },
    {
      "id": "24w36a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/63168585feab277a0da5c51052b2412bfe8afe5e/24w36a.json",
      "time": "2025-01-24T06:33:55+00:00",
      "releaseTime": "2024-09-04T12:44:12+00:00",
      "sha1": "63168585feab277a0da5c51052b2412bfe8afe5e",
      "complianceLevel": 1
    },
    {
      "id": "24w35a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/1e8e30afe25fc63e7c75badee2f5621100a9738e/24w35a.json",
      "time": "2025-01-24T06:33:24+00:00",
      "releaseTime": "2024-08-28T12:25:10+00:00",
      "sha1": "1e8e30afe25fc63e7c75badee2f5621100a9738e",
      "complianceLevel": 1
    },
    {
      "id": "24w34a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/8b846fda86dc357d386a0e5e9eedd5c11986c5d2/24w34a.json",
      "time": "2025-01-24T06:33:24+00:00",
      "releaseTime": "2024-08-21T14:14:13+00:00",
      "sha1": "8b846fda86dc357d386a0e5e9eedd5c11986c5d2",
      "complianceLevel": 1
    },
    {
      "id": "24w33a",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/a3374cb6058f7e0e19453098ccffb455c43bbb25/24w33a.json",
      "time": "2025-01-24T06:33:24+00:00",
      "releaseTime": "2024-08-15T12:39:34+00:00",
      "sha1": "a3374cb6058f7e0e19453098ccffb455c43bbb25",
      "complianceLevel": 1
    },
    {
      "id": "1.21.1",
      "type": "release",
      "url": "https://piston-meta.mojang.com/v1/packages/28d657e6a3ce6c148b3edc232f3a0547dbc82ec9/1.21.1.json",
      "time": "2025-01-24T06:33:24+00:00",
      "releaseTime": "2024-08-08T12:24:45+00:00",
      "sha1": "28d657e6a3ce6c148b3edc232f3a0547dbc82ec9",
      "complianceLevel": 1
    },
    {
      "id": "1.21.1-rc1",
      "type": "snapshot",
      "url": "https://piston-meta.mojang.com/v1/packages/f0d270b611a3b44f29957a37146c32d135588b3a/1.21.1-rc1.json",
      "time": "2025-01-24T06:33:24+00:00",
      "releaseTime": "2024-08-07T14:29:18+00:00",
      "sha1": "f0d270b611a3b44f29957a37146c32d135588b3a",
      "complianceLevel": 1
    },
    {
      "id": "1.21",
      "type": "release",
      "url": "https://piston-meta.mojang.com/v1/packages/1595fcf4724f6a58cc0b84f005c9cfe109933f0d/1.21.json",
      "time": "2025-01-24T06:33:24+00:00",
      "releaseTime": "2024-06-13T08:24:03+00:00",
      "sha1": "1595fcf4724f6a58cc0b84f005c9cfe109933f0d",
      "complianceLevel": 1
    }
  ]
}"#;
