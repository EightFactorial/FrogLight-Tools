use std::{path::PathBuf, sync::Arc};

use froglight_definitions::manifests::VersionManifest;
use froglight_extract::bundle::{ExtractBundle, ManifestBundle};
use reqwest::Client;
use serde_json::{Map, Value};
use tracing::{debug, error, info};

use crate::config::GenerateVersion;

/// Generate code for a specific version.
pub(crate) async fn generate(
    version: GenerateVersion,
    manifest: Arc<VersionManifest>,
    mut cache: PathBuf,
    client: Client,
) {
    // Get the version entry
    let version_manifest = manifest.as_ref();
    let Some(entry) = version_manifest.get(&version.jar) else {
        error!("Version not found: {}", version.jar);
        return;
    };
    info!("Found Version in Manifest: \"{}\"", entry.id);
    debug!("Version Entry: {entry:#?}");

    // Append the version to the cache path
    cache.push(entry.id.to_short_string());
    if !cache.exists() {
        if let Err(err) = tokio::fs::create_dir_all(&cache).await {
            error!("Failed to create cache directory: {err}");
            return;
        }
    }

    // Get the `ReleaseManifest`
    let release_manifest =
        match froglight_tools::manifests::get_release_manifest(entry, &cache, &client).await {
            Ok(manifest) => manifest,
            Err(err) => {
                error!("Failed to get `ReleaseManifest`: {err}");
                return;
            }
        };
    info!("Loaded Release Manifest for: \"{}\"", entry.id);

    // Get the `AssetManifest`
    let asset_manifest =
        match froglight_tools::manifests::get_asset_manifest(&release_manifest, &cache, &client)
            .await
        {
            Ok(manifest) => manifest,
            Err(err) => {
                error!("Failed to get `AssetManifest`: {err}");
                return;
            }
        };
    info!("Loaded Asset Manifest for: \"{}\"", entry.id);
    debug!("Asset Manifest: {} Assets", asset_manifest.objects.len());

    // Store the Extract Data
    let mut extract_data = Value::Object(Map::default());

    // Create an `ExtractBundle`
    let manifest_bundle = ManifestBundle::new(version_manifest, &release_manifest, &asset_manifest);
    let _extract_bundle =
        ExtractBundle::new(&version.base, &mut extract_data, &cache, manifest_bundle);

    // TODO: Iterate over the modules and extract data
    // TODO: Use the extracted data to generate code
}
