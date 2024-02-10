use std::path::Path;

use froglight_data::{manifest::ManifestVersion, ReleaseManifest};
use tracing::{debug, error, info};

/// Load the release manifest from the cache or download it from the server
///
/// # Panics
/// - If the cache directory cannot be created
/// - If the release manifest fails to download
/// - If the release manifest fails to write to the cache
/// - If the release manifest fails to parse
pub async fn release_manifest(
    version: &ManifestVersion,
    cache: &Path,
    refresh: bool,
) -> ReleaseManifest {
    let mut manifest_path = cache.join("froglight");
    manifest_path.push(&version.id.to_short_string());

    // Create the cache directory if it doesn't exist
    if !manifest_path.exists() {
        tokio::fs::create_dir_all(&manifest_path).await.expect("Failed to create cache directory");
    }

    manifest_path.push(&format!("{}.json", version.id.to_short_string()));

    debug!("ReleaseManifest Path: {}", manifest_path.display());

    let mut manifest = None;

    // Try to read the manifest from the cache
    if !refresh && manifest_path.exists() {
        // Read the manifest from the cache
        match tokio::fs::read_to_string(&manifest_path).await {
            Err(err) => error!("Failed to read manifest from cache: `{err}`"),
            // Parse the manifest
            Ok(contents) => match serde_json::from_str(&contents) {
                Err(err) => error!("Failed to parse manifest from cache: `{err}`"),
                Ok(res) => {
                    debug!("Loaded ReleaseManifest from cache");
                    manifest = Some(res);
                }
            },
        }
    }

    // Download the manifest from the server and save it to the cache
    if manifest.is_none() {
        info!("Downloading ReleaseManifest...");
        debug!("ReleaseManifest URL: {}", version.url);

        // Download the manifest
        let response = match reqwest::get(&version.url).await {
            Err(err) => panic!("Failed to download ReleaseManifest: `{err}`"),
            Ok(response) => response.bytes().await.expect("Failed to read response"),
        };

        // Save the manifest to the cache
        if let Err(err) = tokio::fs::write(&manifest_path, &response).await {
            panic!("Failed to write manifest to cache: `{err}`");
        } else {
            debug!("Saved ReleaseManifest to cache");
        }

        // Parse the manifest
        match serde_json::from_slice(&response) {
            Err(err) => panic!("Failed to parse ReleaseManifest: `{err}`"),
            Ok(res) => manifest = Some(res),
        }
    }

    manifest.unwrap()
}
