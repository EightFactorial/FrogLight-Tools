use froglight_data::VersionManifest;
use tracing::{debug, error, info};

use crate::Command;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Load the version manifest from the cache or download it from the server
pub(crate) async fn version_manifest(command: &Command) -> VersionManifest {
    let mut manifest_path = command.cache.join("froglight");
    manifest_path.push("version_manifest_v2.json");
    debug!("VersionManifest Path: {:?}", manifest_path);

    let mut manifest = None;

    // Try to read the manifest from the cache
    if !command.refresh && manifest_path.exists() {
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
