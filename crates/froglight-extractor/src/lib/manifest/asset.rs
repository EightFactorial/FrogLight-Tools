use std::path::Path;

use froglight_data::{AssetManifest, ReleaseManifest, Version};
use tracing::{debug, error, info};

/// Load the asset manifest from the cache or download it from the server
///
/// # Errors
/// - If the cache directory cannot be created
/// - If the asset manifest fails to download
/// - If the asset manifest fails to write to the cache
/// - If the asset manifest fails to parse
pub async fn asset_manifest(
    version: &Version,
    release: &ReleaseManifest,
    cache: &Path,
    refresh: bool,
) -> anyhow::Result<AssetManifest> {
    let mut manifest_path = cache.join("froglight");
    manifest_path.push(&version.to_short_string());

    // Create the cache directory if it doesn't exist
    if !manifest_path.exists() {
        tokio::fs::create_dir_all(&manifest_path).await?;
    }

    manifest_path.push("assets.json");

    debug!("AssetManifest Path: {}", manifest_path.display());

    let mut manifest: Option<AssetManifest> = None;

    // Try to read the manifest from the cache
    if !refresh && manifest_path.exists() {
        // Read the manifest from the cache
        match tokio::fs::read_to_string(&manifest_path).await {
            Err(err) => error!("Failed to read manifest from cache: `{err}`"),
            // Parse the manifest
            Ok(contents) => match serde_json::from_str(&contents) {
                Err(err) => error!("Failed to parse manifest from cache: `{err}`"),
                Ok(res) => {
                    debug!("Loaded AssetManifest from cache");
                    manifest = Some(res);
                }
            },
        }
    }

    // Download the manifest from the server and save it to the cache
    if manifest.is_none() {
        info!("Downloading AssetManifest...");
        debug!("AssetManifest URL: {}", release.asset_index.url);

        // Download the manifest and save it to the cache
        let response = reqwest::get(&release.asset_index.url).await?.bytes().await?;
        tokio::fs::write(&manifest_path, &response).await?;

        // Parse the manifest
        if let Ok(res) = serde_json::from_slice(&response) {
            debug!("Saved AssetManifest to cache");
            manifest = Some(res);
        } else {
            error!("Failed to parse AssetManifest");
        }
    }

    if let Some(manifest) = manifest {
        Ok(manifest)
    } else {
        anyhow::bail!("Failed to load AssetManifest");
    }
}
