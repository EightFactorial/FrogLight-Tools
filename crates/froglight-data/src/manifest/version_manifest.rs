use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::Version;

/// Data from Mojang on all versions of Minecraft, including
/// releases, release candidates, snapshots, and more.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersionManifest {
    /// The latest versions of Minecraft.
    pub latest: ManifestLatest,
    /// Information for all versions of Minecraft.
    pub versions: Vec<ManifestVersion>,
}

/// The latest versions of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ManifestLatest {
    /// The latest release version of Minecraft.
    pub release: Version,
    /// The latest snapshot version of Minecraft.
    pub snapshot: Version,
}

impl Display for ManifestLatest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Release: {}, Snapshot: {}", self.release, self.snapshot)
    }
}

/// Information for a specific version of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ManifestVersion {
    /// The [`Version`] of Minecraft.
    pub id: Version,
    /// The release type.
    #[serde(rename = "type")]
    pub kind: String,
    /// The Url for the [`ReleaseManifest`](crate::ReleaseManifest).
    pub url: String,
    /// The release time.
    pub time: DateTime<Utc>,
}

impl Display for ManifestVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.kind)
    }
}

#[test]
fn manifest_deserialization() {
    let manifest: &str = r#"{
        "latest": {
            "release": "1.20.4",
            "snapshot": "24w03b"
        },
        "versions": [
            {
                "id": "24w03b",
                "type": "snapshot",
                "url": "https://piston-meta.mojang.com/v1/packages/ea3ab7762af9fd43565a5d8d96652899a4dc6303/24w03b.json",
                "time": "2024-01-18T12:49:51+00:00",
                "releaseTime": "2024-01-18T12:42:37+00:00",
                "sha1":	"ea3ab7762af9fd43565a5d8d96652899a4dc6303"
            },
            {
                "id": "1.20.4",
                "type": "release",
                "url": "https://piston-meta.mojang.com/v1/packages/c98adde5094a3041f486b4d42d0386cf87310559/1.20.4.json",
                "time":	"2024-01-18T12:24:32+00:00",
                "releaseTime": "2023-12-07T12:56:20+00:00",
                "sha1":	"ea3ab7762af9fd43565a5d8d96652899a4dc6303"
            }
        ]
    }"#;
    let manifest: VersionManifest = serde_json::from_str(manifest).unwrap();

    // Test ManifestLatest
    assert_eq!(manifest.latest.release, Version::new_rel(1, 20, 4));
    assert_eq!(manifest.latest.snapshot, Version::new_snapshot("24w03b"));

    // Test ManifestVersion
    assert_eq!(manifest.versions.len(), 2);

    assert_eq!(manifest.versions[0].id, Version::new_snapshot("24w03b"));
    assert_eq!(manifest.versions[0].kind, "snapshot");

    assert_eq!(manifest.versions[1].id, Version::new_rel(1, 20, 4));
    assert_eq!(manifest.versions[1].kind, "release");
}
