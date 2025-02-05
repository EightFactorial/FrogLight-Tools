//! TODO

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::ReleaseManifests;
use crate::{container::DependencyContainer, version::Version};

/// A collection of [`AssetManifest`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct AssetManifests(HashMap<Version, AssetManifest>);

impl AssetManifests {
    /// Get the [`AssetManifest`] for a given [`Version`].
    ///
    /// Returns `None` if the manifest is not already known.
    #[must_use]
    pub fn assets(&self, version: &Version) -> Option<&AssetManifest> { self.0.get(version) }

    /// Get the [`AssetManifest`] for a given [`Version`].
    ///
    /// # Errors
    /// Returns an error if there is an issue retrieving the manifest.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_assets(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&AssetManifest> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<ReleaseManifests>().await?;
            deps.scoped_fut::<ReleaseManifests, anyhow::Result<()>>(
                async |manifest: &mut ReleaseManifests, deps: &mut DependencyContainer| {
                    let release = manifest.get_release(version, deps).await?;

                    let filename = release.asset_index.url.split('/').next_back().unwrap();
                    let assets_path = deps.cache.join(version.to_long_string()).join(filename);

                    self.0.insert(
                        version.clone(),
                        if tokio::fs::try_exists(&assets_path).await? {
                            tracing::debug!("Reading \"{}\"", assets_path.display());

                            // Read the file from disk and parse it
                            let content = tokio::fs::read(assets_path).await?;
                            serde_json::from_slice(&content)?
                        } else {
                            tracing::debug!("Retrieving \"{}\"", release.asset_index.url);

                            // Download the file, save it to disk, and parse it
                            let response = deps.client.get(&release.asset_index.url).send().await?;
                            let bytes = response.bytes().await?;
                            tokio::fs::write(&assets_path, &bytes).await?;
                            serde_json::from_slice(&bytes)?
                        },
                    );

                    Ok(())
                },
            )
            .await?;
        }

        Ok(self.0.get(version).unwrap())
    }
}

/// An [`AssetManifest`] for a specific version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifest {
    /// The objects in the manifest.
    pub objects: HashMap<String, AssetManifestEntry>,
}
impl std::ops::Deref for AssetManifest {
    type Target = HashMap<String, AssetManifestEntry>;
    fn deref(&self) -> &Self::Target { &self.objects }
}
impl std::ops::DerefMut for AssetManifest {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.objects }
}

/// An entry in an [`AssetManifest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetManifestEntry {
    /// The hash of the asset.
    pub hash: String,
    /// The size of the asset.
    pub size: u64,
}

impl AssetManifestEntry {
    /// Get the URL of the asset.
    #[must_use]
    pub fn get_url(&self) -> String { todo!("TODO: Get URL from hash") }
}

#[test]
#[cfg(test)]
fn parse() {
    let manifest: AssetManifest = serde_json::from_str(TRIMMED_EXAMPLE).unwrap();

    assert_eq!(manifest.len(), 5);
    assert_eq!(manifest["icons/icon_16x16.png"].hash, "5ff04807c356f1beed0b86ccf659b44b9983e3fa");
    assert_eq!(manifest["icons/icon_32x32.png"].hash, "af96f55a90eaf11b327f1b5f8834a051027dc506");
    assert_eq!(manifest["icons/icon_128x128.png"].hash, "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356");
    assert_eq!(manifest["icons/icon_256x256.png"].hash, "8030dd9dc315c0381d52c4782ea36c6baf6e8135");
}

#[cfg(test)]
const TRIMMED_EXAMPLE: &str = r#"{
  "objects": {
    "icons/icon_128x128.png": {
      "hash": "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356",
      "size": 9101
    },
    "icons/icon_16x16.png": {
      "hash": "5ff04807c356f1beed0b86ccf659b44b9983e3fa",
      "size": 781
    },
    "icons/icon_256x256.png": {
      "hash": "8030dd9dc315c0381d52c4782ea36c6baf6e8135",
      "size": 19642
    },
    "icons/icon_32x32.png": {
      "hash": "af96f55a90eaf11b327f1b5f8834a051027dc506",
      "size": 2063
    },
    "icons/icon_48x48.png": {
      "hash": "b80b6e9ff01c78c624df5429e1d3dcd3d5130834",
      "size": 3409
    }
  }
}"#;
