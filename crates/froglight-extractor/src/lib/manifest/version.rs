use std::path::Path;

use froglight_data::VersionManifest;
use tracing::{debug, error, info};

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const MANIFEST_FILE: &str = "version_manifest_v2.json";

/// Load the version manifest from the cache or download it from the server
///
/// # Panics
/// - If the cache directory cannot be created
/// - If the version manifest fails to download
/// - If the version manifest fails to write to the cache
/// - If the version manifest fails to parse
pub async fn version_manifest(cache: &Path, refresh: bool) -> VersionManifest {
    let mut manifest_path = cache.join("froglight");

    // Create the cache directory if it doesn't exist
    if !manifest_path.exists() {
        tokio::fs::create_dir_all(&manifest_path).await.expect("Failed to create cache directory");
    }

    manifest_path.push(MANIFEST_FILE);
    debug!("VersionManifest Path: {}", manifest_path.display());

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
                    debug!("Loaded VersionManifest from cache");
                    manifest = Some(res);
                }
            },
        }
    }

    // Download the manifest from the server and save it to the cache
    if manifest.is_none() {
        info!("Downloading VersionManifest...");
        debug!("VersionManifest URL: {}", MANIFEST_URL);

        // Download the manifest
        let response = match reqwest::get(MANIFEST_URL).await {
            Err(err) => panic!("Failed to download VersionManifest: `{err}`"),
            Ok(response) => response.bytes().await.expect("Failed to read response"),
        };

        // Save the manifest to the cache
        if let Err(err) = tokio::fs::write(&manifest_path, &response).await {
            panic!("Failed to write manifest to cache: `{err}`");
        } else {
            debug!("Saved VersionManifest to cache");
        }

        // Parse the manifest
        match serde_json::from_slice(&response) {
            Err(err) => panic!("Failed to parse VersionManifest: `{err}`"),
            Ok(res) => manifest = Some(res),
        }
    }

    manifest.unwrap()
}
