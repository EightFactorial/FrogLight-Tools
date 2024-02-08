use serde::Deserialize;

/// Information for a specific version of Minecraft.
///
/// Mainly used for downloading the client and server jars,
/// mappings, and assets.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReleaseManifest {
    /// Information about the [`AssetManifest`](crate::AssetManifest).
    #[serde(rename = "assetIndex")]
    pub asset_index: ReleaseAssetIndex,
    /// The [`ReleaseDownloads`] for this version.
    pub downloads: ReleaseDownloads,
}

/// Information about the [`AssetManifest`](crate::AssetManifest).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReleaseAssetIndex {
    /// The SHA1 hash of the [`AssetManifest`](crate::AssetManifest).
    pub sha1: String,
    /// The size of the [`AssetManifest`](crate::AssetManifest).
    pub size: u64,
    /// The total size of all assets in the
    /// [`AssetManifest`](crate::AssetManifest).
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    /// The Url for the [`AssetManifest`](crate::AssetManifest).
    pub url: String,
}

/// Downloads for a specific version of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReleaseDownloads {
    /// The client jar.
    pub client: ReleaseDownload,
    /// The client mappings.
    pub client_mappings: Option<ReleaseDownload>,
    /// The server jar.
    pub server: ReleaseDownload,
    /// The server mappings.
    pub server_mappings: Option<ReleaseDownload>,
}

/// A download for a specific version of Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReleaseDownload {
    /// The SHA1 hash of the file.
    pub sha1: String,
    /// The size of the file.
    pub size: u64,
    /// The Url for the file.
    pub url: String,
}

#[test]
fn release_deserialization() {
    let manifest: &str = r#"{
        "assetIndex": {
            "sha1": "bf344b386e565174a16f02c738adff7b3ba3b9c8",
            "size": 432505,
            "totalSize": 625503756,
            "url": "https://piston-meta.mojang.com/v1/packages/bf344b386e565174a16f02c738adff7b3ba3b9c8/12.json"
        },
        "downloads": {
            "client": {
                "sha1": "b991734308e2cc6dcfbdb9338ea0453009d8e9e1",
                "size": 24670363,
                "url": "https://piston-data.mojang.com/v1/objects/b991734308e2cc6dcfbdb9338ea0453009d8e9e1/client.jar"
            },
            "server": {
                "sha1": "5b9a529dc40d8394cbd6203a8ebe66c8e2f86fd4",
                "size": 49337911,
                "url": "https://piston-data.mojang.com/v1/objects/5b9a529dc40d8394cbd6203a8ebe66c8e2f86fd4/server.jar"
            }
        }
    }"#;
    let manifest: ReleaseManifest = serde_json::from_str(manifest).unwrap();

    // Test AssetIndex
    assert_eq!(manifest.asset_index.sha1, "bf344b386e565174a16f02c738adff7b3ba3b9c8");
    assert_eq!(manifest.asset_index.size, 432_505);
    assert_eq!(manifest.asset_index.total_size, 625_503_756);

    // Test ReleaseDownloads
    assert_eq!(manifest.downloads.client.sha1, "b991734308e2cc6dcfbdb9338ea0453009d8e9e1");
    assert_eq!(manifest.downloads.client.size, 24_670_363);

    assert_eq!(manifest.downloads.server.sha1, "5b9a529dc40d8394cbd6203a8ebe66c8e2f86fd4");
    assert_eq!(manifest.downloads.server.size, 49_337_911);

    assert_eq!(manifest.downloads.client_mappings, None);
    assert_eq!(manifest.downloads.server_mappings, None);
}
