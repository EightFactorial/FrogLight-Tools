use std::{path::PathBuf, sync::Arc};

use froglight_definitions::manifests::{VersionManifest, YarnManifest};
use froglight_extract::bundle::{ExtractBundle, ManifestBundle};
use reqwest::Client;
use serde_json::{Map, Value};
use tracing::{debug, error, info};

use crate::config::GenerateVersion;

/// Generate code for a specific version.
pub(crate) async fn generate(
    version: GenerateVersion,

    version_manifest: Arc<VersionManifest>,
    yarn_manifest: Arc<YarnManifest>,
    remapper_path: PathBuf,

    mut cache: PathBuf,
    client: Client,
) {
    // Get the version entry
    let Some(version_entry) = version_manifest.get(&version.jar) else {
        error!("Version not found: {}", version.jar);
        error!("Try emptying the cache directory and trying again.");
        return;
    };
    info!("Found Version in Manifest: \"{}\"", version_entry.id);
    debug!("Version Entry: {version_entry:#?}");

    // Append the version to the cache path
    cache.push(version_entry.id.to_short_string());
    if !cache.exists() {
        if let Err(err) = tokio::fs::create_dir_all(&cache).await {
            error!("Failed to create cache directory: {err}");
            return;
        }
    }

    // Get the `ReleaseManifest`
    let release_manifest = match froglight_tools::manifests::get_release_manifest(
        version_entry,
        &cache,
        &client,
    )
    .await
    {
        Ok(manifest) => manifest,
        Err(err) => {
            error!("Failed to get `ReleaseManifest`: {err}");
            return;
        }
    };
    info!("Loaded Release Manifest for: \"{}\"", version_entry.id);

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
    info!("Loaded Asset Manifest for: \"{}\"", version_entry.id);
    debug!("Asset Manifest: {} Assets", asset_manifest.objects.len());

    // Download the `Client` JAR
    let Some(client_jar) = froglight_tools::jar_tools::download_client_jar(
        &release_manifest.downloads,
        &cache,
        &client,
    )
    .await
    else {
        error!("Failed to download `Client` JAR");
        return;
    };

    // Get the latest Yarn mappings for this version
    let Some(yarn_build) = yarn_manifest.get_latest(&version_entry.id) else {
        error!("No Yarn mappings found for: \"{}\"", version_entry.id);
        return;
    };
    info!("Latest Yarn mappings for \"{}\": {yarn_build}", version_entry.id);

    let Some(yarn_mappings) =
        froglight_tools::jar_tools::download_yarn_mappings(yarn_build, &cache, &client).await
    else {
        error!("Failed to download `Yarn` mappings");
        return;
    };

    // Get the deobfuscated `Client` JAR
    let Some(_mapped_jar) = froglight_tools::jar_tools::deobfuscate_client_jar(
        &remapper_path,
        &client_jar,
        &yarn_mappings,
        &cache,
    )
    .await
    else {
        error!("Failed to deobfuscate `Client` JAR");
        return;
    };

    // Create a `Value` to store extracted data
    let mut extract_data = Value::Object(Map::new());

    // Create an `ExtractBundle`
    let manifest_bundle =
        ManifestBundle::new(&version_manifest, &yarn_manifest, &release_manifest, &asset_manifest);
    let _extract_bundle =
        ExtractBundle::new(&version.base, &mut extract_data, &cache, manifest_bundle);

    // TODO: Iterate over the modules and extract data
    // TODO: Use the extracted data to generate code
}
