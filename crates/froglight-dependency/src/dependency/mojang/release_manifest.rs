//! TODO

use chrono::{DateTime, Utc};
use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::VersionManifest;
use crate::{container::DependencyContainer, version::Version};

/// A collection of [`ReleaseManifest`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct ReleaseManifests(HashMap<Version, ReleaseManifest>);

impl ReleaseManifests {
    /// Get the [`ReleaseManifest`] for a given [`Version`].
    ///
    /// Returns `None` if the manifest is not already known.
    #[must_use]
    pub fn release(&self, version: &Version) -> Option<&ReleaseManifest> { self.0.get(version) }

    /// Get the [`ReleaseManifest`] for a given [`Version`].
    ///
    /// # Errors
    /// Returns an error if there is an issue retrieving the manifest.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_release(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&ReleaseManifest> {
        if !self.0.contains_key(version) {
            let cache_dir = deps.cache.join(version.to_long_string());
            if !tokio::fs::try_exists(&cache_dir).await? {
                tokio::fs::create_dir_all(&cache_dir).await?;
            }

            deps.get_or_retrieve::<VersionManifest>().await?;
            deps.scoped_fut::<VersionManifest, _>(async |manifest, deps| {
                if let Some(entry) = manifest.get(version) {
                    let manifest_path = cache_dir.join(entry.url.split('/').next_back().unwrap());

                    self.0.insert(
                        version.clone(),
                        if tokio::fs::try_exists(&manifest_path).await? {
                            tracing::debug!("Reading \"{}\"", manifest_path.display());

                            // Read the file from disk and parse it
                            let content = tokio::fs::read(manifest_path).await?;
                            serde_json::from_slice(&content)?
                        } else {
                            tracing::debug!("Retrieving \"{}\"", entry.url);

                            // Download the file, save it to disk, and parse it
                            let response =
                                deps.client.get(&entry.url).send().await?.bytes().await?;
                            tokio::fs::write(manifest_path, &response).await?;
                            serde_json::from_slice(&response)?
                        },
                    );

                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Version \"{version}\" not found in VersionManifest!"))
                }
            })
            .await?;
        }

        Ok(self.0.get(version).unwrap())
    }
}

/// A version's release information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    /// The version's [`AssetManifest`](super::AssetManifest).
    #[serde(rename = "assetIndex")]
    pub asset_index: ReleaseAssetIndex,
    /// The version's downloads.
    pub downloads: ReleaseDownloads,
    /// The [`Version`].
    pub id: Version,
    /// The Java version.
    #[serde(rename = "javaVersion")]
    pub java_version: ReleaseJavaVersion,
    /// The main class.
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// The minimum launcher version.
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: u32,
    /// When the version was released.
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// The last time the manifest was updated.
    pub time: DateTime<Utc>,
    /// The release type.
    #[serde(rename = "type")]
    pub release_type: String,
}

/// Information about the version's [`AssetManifest`](super::AssetManifest).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseAssetIndex {
    /// The asset index ID.
    pub id: String,
    /// The SHA1 hash of the asset index.
    pub sha1: String,
    /// The size of the version.
    pub size: u32,
    /// The total size of the version.
    #[serde(rename = "totalSize")]
    pub total_size: u32,
    /// The URL of the [`AssetManifest`](super::AssetManifest).
    pub url: String,
}

/// Download information for the version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseDownloads {
    /// The client jar.
    pub client: ReleaseDownload,
    /// The mappings for the client.
    pub client_mappings: ReleaseDownload,
    /// The server jar.
    pub server: ReleaseDownload,
    /// The mappings for the server.
    pub server_mappings: ReleaseDownload,
}

/// Information about a download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseDownload {
    /// The SHA1 hash of the download.
    pub sha1: String,
    /// The size of the download.
    pub size: u32,
    /// The URL of the download.
    pub url: String,
}

/// Information about the Java version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseJavaVersion {
    /// The component of the Java version.
    pub component: String,
    /// The major version of the Java version.
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[test]
#[cfg(test)]
fn parse() {
    let example: ReleaseManifest = serde_json::from_str(TRIMMED_EXAMPLE).unwrap();
    assert_eq!(example.asset_index.sha1, "d6d68dd7dbd932e01d730fbc34d7f81b8ea3f813");
    assert_eq!(example.downloads.client.sha1, "5bc08371cd4da86bcd5afd12bea91c890a3c63bb");
    assert_eq!(example.downloads.client_mappings.sha1, "98c9a121ce9d560fd9d5aa2ea576f117c0950c26");
    assert_eq!(example.downloads.server.sha1, "2c873903a90c9633dd6bd2e3501046100daceafd");
    assert_eq!(example.downloads.server_mappings.sha1, "bed2cd62c9c5cf4c173360647c577aedb65c8a1c");
}

#[cfg(test)]
const TRIMMED_EXAMPLE: &str = r#"{
  "assetIndex": {
    "id": "22",
    "sha1": "d6d68dd7dbd932e01d730fbc34d7f81b8ea3f813",
    "size": 471871,
    "totalSize": 404648628,
    "url": "https://piston-meta.mojang.com/v1/packages/d6d68dd7dbd932e01d730fbc34d7f81b8ea3f813/22.json"
  },
  "downloads": {
    "client": {
      "sha1": "5bc08371cd4da86bcd5afd12bea91c890a3c63bb",
      "size": 28652475,
      "url": "https://piston-data.mojang.com/v1/objects/5bc08371cd4da86bcd5afd12bea91c890a3c63bb/client.jar"
    },
    "client_mappings": {
      "sha1": "98c9a121ce9d560fd9d5aa2ea576f117c0950c26",
      "size": 10473590,
      "url": "https://piston-data.mojang.com/v1/objects/98c9a121ce9d560fd9d5aa2ea576f117c0950c26/client.txt"
    },
    "server": {
      "sha1": "2c873903a90c9633dd6bd2e3501046100daceafd",
      "size": 57104369,
      "url": "https://piston-data.mojang.com/v1/objects/2c873903a90c9633dd6bd2e3501046100daceafd/server.jar"
    },
    "server_mappings": {
      "sha1": "bed2cd62c9c5cf4c173360647c577aedb65c8a1c",
      "size": 7870644,
      "url": "https://piston-data.mojang.com/v1/objects/bed2cd62c9c5cf4c173360647c577aedb65c8a1c/server.txt"
    }
  },
  "id": "25w05a",
  "javaVersion": {
    "component": "java-runtime-delta",
    "majorVersion": 21
  },
  "mainClass": "net.minecraft.client.main.Main",
  "minimumLauncherVersion": 21,
  "releaseTime": "2025-01-29T14:03:54+00:00",
  "time": "2025-01-29T14:03:54+00:00",
  "type": "snapshot"
}"#;
