use std::{path::PathBuf, sync::Arc};

use async_zip::tokio::read::fs::ZipFileReader;
use froglight_definitions::manifests::{VersionManifest, YarnManifest};
use froglight_extract::{
    bundle::{ExtractBundle, ManifestBundle},
    bytecode::JarContainer,
};
use reqwest::Client;
use serde_json::{Map, Value};
use tracing::{debug, error, info};

use crate::config::GenerateVersion;

/// Generate code for a specific version.
#[allow(clippy::too_many_lines)]
pub(crate) async fn generate(
    version: GenerateVersion,

    version_manifest: Arc<VersionManifest>,
    yarn_manifest: Arc<YarnManifest>,
    remapper_path: PathBuf,

    mut cache: PathBuf,
    client: Client,
) {
    // --- Prepare Files ---

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
    let Some(client_jar) = froglight_tools::deobfuscate::download_client_jar(
        &release_manifest.downloads,
        &cache,
        &client,
    )
    .await
    else {
        error!("Failed to download `Client` JAR");
        return;
    };

    // --- Deobfuscate Jar ---

    // Get the latest Yarn mappings for this version
    let Some(yarn_build) = yarn_manifest.get_latest(&version_entry.id) else {
        error!("No Yarn mappings found for: \"{}\"", version_entry.id);
        return;
    };
    info!("Using Yarn: \"{yarn_build}\"");

    let Some(yarn_mappings) =
        froglight_tools::mappings::download_yarn_mappings(yarn_build, &cache, &client).await
    else {
        error!("Failed to download `Yarn` mappings");
        return;
    };

    // Get the deobfuscated `Client` JAR
    let Some(mapped_jar) = froglight_tools::deobfuscate::deobfuscate_client_jar(
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

    // Read and parse the deobfuscated `Client` JAR
    let jar_reader = match ZipFileReader::new(mapped_jar).await {
        Ok(reader) => reader,
        Err(err) => {
            error!("Failed to create ZIP reader: {err}");
            return;
        }
    };
    let jar_container = match JarContainer::new_tokio_fs(&jar_reader).await {
        Ok(jar) => jar,
        Err(err) => {
            error!("Failed to parse `Client` JAR: {err}");
            return;
        }
    };
    info!("Successfully parsed \"{}\" JAR", version_entry.id);
    debug!("\"{}\" parsed {} classes", version_entry.id, jar_container.len());

    // --- Extract and Generate ---

    // Create a `Value` to store extracted data
    let mut extract_data = Value::Object(Map::new());

    // Extract data from the `Client` JAR
    {
        // Create a `ManifestBundle`
        let manifest_bundle = ManifestBundle::new(
            &version_manifest,
            &yarn_manifest,
            &release_manifest,
            &asset_manifest,
        );

        // Create an `ExtractBundle`
        let _extract_bundle = ExtractBundle::new(
            &version_entry.id,
            &jar_container,
            &jar_reader,
            manifest_bundle,
            &mut extract_data,
            &cache,
        );

        // TODO: Iterate over the generate modules
        // and find required extract modules

        // TODO: Run the required extract modules
    }

    // Generate code from the extracted data
    {
        // TODO: Generate code
    }
}
